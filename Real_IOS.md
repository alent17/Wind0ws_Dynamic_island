### 第一步：修改后端 (`lib.rs`) - 剥离阻尼，专注信号提取

打开 `lib.rs`，彻底替换掉 `start_audio_visualizer` 和 `calculate_bands` 函数。我们将去掉 Rust 端的平滑，并引入 iOS 级别的**噪音门（Noise Gate）和高频补偿**。

Rust

```
#[derive(Clone, Serialize)]
struct SpectrumPayload {
    bands: Vec<f32>,
    bands_expanded: Vec<f32>,
}

fn start_audio_visualizer(app: AppHandle) {
    std::thread::spawn(move || {
        let host = cpal::default_host();
        let device = match host.default_output_device() {
            Some(d) => d,
            None => {
                eprintln!("[音频] 未找到默认音频输出设备");
                return;
            }
        };

        let config = match device.default_output_config() {
            Ok(c) => c.config(),
            Err(e) => {
                eprintln!("[音频] 获取音频配置失败: {}", e);
                return;
            }
        };

        let channels = config.channels as usize;
        let sample_rate = config.sample_rate.0 as f32;

        const FFT_SIZE: usize = 1024;
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(FFT_SIZE);
        let mut sample_buffer: Vec<f32> = Vec::with_capacity(FFT_SIZE);

        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                for frame in data.chunks(channels) {
                    let mono = frame.iter().sum::<f32>() / channels as f32;
                    sample_buffer.push(mono);

                    if sample_buffer.len() == FFT_SIZE {
                        // 汉宁窗 (Hanning Window) 减少边缘频谱泄漏
                        let mut buffer: Vec<Complex<f32>> = sample_buffer
                            .iter()
                            .enumerate()
                            .map(|(i, &val)| {
                                let window = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (FFT_SIZE as f32 - 1.0)).cos());
                                Complex { re: val * window, im: 0.0 }
                            })
                            .collect();

                        fft.process(&mut buffer);

                        let magnitudes: Vec<f32> = buffer.iter()
                            .take(FFT_SIZE / 2)
                            .map(|c| c.norm())
                            .collect();

                        // 直接计算原始频段能量并发送给前端，绝对不要在 Rust 里做缓动
                        let bands = calculate_bands(&magnitudes, 5, sample_rate, FFT_SIZE);
                        let bands_expanded = calculate_bands(&magnitudes, 20, sample_rate, FFT_SIZE);

                        let _ = app.emit("audio-spectrum", SpectrumPayload {
                            bands,
                            bands_expanded
                        });

                        // 保留 50% 重叠 (Overlap)，提高 FFT 刷新率
                        sample_buffer.drain(0..FFT_SIZE / 2);
                    }
                }
            },
            |err| eprintln!("[音频] 捕获流错误: {}", err),
            None
        );

        match stream {
            Ok(s) => {
                s.play().unwrap();
                loop { std::thread::sleep(std::time::Duration::from_secs(1)); }
            }
            Err(e) => eprintln!("[音频] 构建捕获流失败: {}", e),
        }
    });
}

fn calculate_bands(magnitudes: &[f32], num_bands: usize, sample_rate: f32, fft_size: usize) -> Vec<f32> {
    let mut bands = vec![0.0; num_bands];
    let freq_resolution = sample_rate / fft_size as f32;

    // iOS 的频响范围设定
    let min_freq: f32 = 80.0;
    let max_freq: f32 = 8000.0;
    let log_min = min_freq.log2();
    let log_max = max_freq.log2();
    let log_step = (log_max - log_min) / num_bands as f32;

    for i in 0..num_bands {
        let start_freq = 2.0_f32.powf(log_min + i as f32 * log_step);
        let end_freq = 2.0_f32.powf(log_min + (i + 1) as f32 * log_step);

        let mut start_bin = (start_freq / freq_resolution).round() as usize;
        let mut end_bin = (end_freq / freq_resolution).round() as usize;

        start_bin = start_bin.clamp(1, magnitudes.len() - 1);
        end_bin = end_bin.clamp(start_bin + 1, magnitudes.len());

        let mut sum = 0.0;
        for j in start_bin..end_bin {
            // 高频补偿：频率越高，权重越大，否则高音柱子跳不起来
            let weight = (j as f32 * freq_resolution / 1000.0).powf(0.85).clamp(1.0, 4.0);
            sum += magnitudes[j] * weight;
        }
        let avg = sum / (end_bin - start_bin).max(1) as f32;

        // 转分贝 (dB)
        let db = if avg > 0.0001 { 20.0 * avg.log10() } else { -100.0 };

        // 动态范围映射：-40dB 到 0dB 映射到 0.0 ~ 1.0
        let mut normalized = (db + 40.0) / 40.0;

        // 【噪音门 Noise Gate】彻底切断底噪，防止静音时柱子抖动
        if normalized < 0.12 { normalized = 0.0; }
        
        // 非线性视觉放大，让中等音量也能看起来很活跃
        bands[i] = normalized.clamp(0.0, 1.0).powf(0.75);
    }
    bands
}

```

### 第二步：修改前端 (`App.svelte`) - 注入纯正的苹果物理缓动

打开 `App.svelte`。前端需要处理两件事：接收无延迟的信号、使用极端的非对称阻尼算法。

**1. 替换监听事件（定位到约 302 行）** 用下面的代码替换掉你的 `unlistenAudioSpectrum` 块，确保数据完美映射到 iOS 的轮廓：

JavaScript

```
      const unlistenAudioSpectrum = await listen(
        "audio-spectrum",
        (event: any) => {
          if (!appSettings.real_time_spectrum || !isPlaying) return;
          const { bands, bands_expanded } = event.payload;

          // 灵敏度放大器
          const sensitivity = 1.35;

          // 核心：iOS 轮廓权重。即便声音一样大，收起状态下依然强制保持“中间高两边低”的完美圆弧
          const iOSCollapsedWeights = [0.4, 0.85, 1.1, 0.85, 0.4];

          targetSpectrumHeights = bands.map((val: number, i: number) => {
            let mapped = val * sensitivity * iOSCollapsedWeights[i] * collapsedMaxHeights[i];
            // 底座保持 2px（正圆点）
            return Math.max(2, mapped);
          });
          
          targetSpectrumHeightsExpanded = bands_expanded.map((val: number, i: number) => {
             let mapped = val * sensitivity * expandedMaxHeights[i];
             return Math.max(2, mapped);
          });
        },
      );

```

**2. 替换动画主循环（定位到约 24 行** **`startSpectrumAnimation`）** 将整个 `startSpectrumAnimation` 替换为以下代码。这是 iOS 视觉体验的核心：**跃升如闪电（0.8 的追踪率），回落如水滴（0.04 到 0.08 的粘滞感）**，并辅以极其轻微的空间融合（防止相邻柱子高低差过于尖锐）。

JavaScript

```
  function startSpectrumAnimation() {
    if (spectrumTimer) return;

    function animate() {
      spectrumTimer = requestAnimationFrame(animate);

      if (isPlaying) {
        if (appSettings.real_time_spectrum) {
          // ====== iOS 终极物理阻尼模型 ======
          
          // 1. 收起状态 (5根)
          const tempHeights = spectrumHeights.map((current, i) => {
            const target = targetSpectrumHeights[i] || 2;
            const diff = target - current;
            // 攻击（Attack）极快，衰减（Decay）极慢
            const tracking = diff > 0 ? 0.82 : 0.045; 
            return current + diff * tracking;
          });

          // 果冻平滑：让柱子之间产生极其轻微的相互拉扯（仅限内部，不影响最高点）
          spectrumHeights = tempHeights.map((val, i, arr) => {
            const left = arr[i - 1] !== undefined ? arr[i - 1] : val;
            const right = arr[i + 1] !== undefined ? arr[i + 1] : val;
            return val * 0.8 + left * 0.1 + right * 0.1;
          });

          // 2. 展开状态 (20根)
          const tempExpanded = spectrumHeightsExpanded.map((current, i) => {
            const target = targetSpectrumHeightsExpanded[i] || 2;
            const diff = target - current;
            // 展开态由于柱子多，下落阻尼可以稍微快一点点
            const tracking = diff > 0 ? 0.85 : 0.06;
            return current + diff * tracking;
          });

          spectrumHeightsExpanded = tempExpanded.map((val, i, arr) => {
            const left = arr[i - 1] !== undefined ? arr[i - 1] : val;
            const right = arr[i + 1] !== undefined ? arr[i + 1] : val;
            return val * 0.7 + left * 0.15 + right * 0.15;
          });

        } else {
          // 你的备用模拟波浪逻辑保持不变...
          spectrumPhase += 0.08;
          // ... [原代码略] ...
        }
      } else {
        // 暂停时的收缩动画：平滑恢复成 2px 的小圆点
        spectrumHeights = spectrumHeights.map((h) => {
          const diff = 2 - h;
          return h + diff * 0.08;
        });
        spectrumHeightsExpanded = spectrumHeightsExpanded.map((h) => {
          const diff = 2 - h;
          return h + diff * 0.08;
        });
      }
    }

    spectrumTimer = requestAnimationFrame(animate);
  }

```

按照这两步修改后：Rust 只负责实时提取并映射原始能量；Svelte 的高频 `requestAnimationFrame` 接管了所有的阻尼运算。当音乐节拍打下时，柱子会以闪电般的速度冲上去，然后像带有重力的糖浆一样慢慢滑落，这就达到了 iOS 1:1 的视觉标准。
