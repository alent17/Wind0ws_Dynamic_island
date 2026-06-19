use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rustfft::{num_complex::Complex, FftPlanner};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tauri::{AppHandle, Emitter};

const FFT_SIZE: usize = 2048;
const HOP_SIZE: usize = 512;
const NUM_BARS: usize = 6;
const MIN_DB: f32 = -60.0;
const MAX_DB: f32 = -10.0;

pub struct AudioCapture {
    stop_signal: Arc<AtomicBool>,
}

impl AudioCapture {
    pub fn new() -> Self {
        AudioCapture {
            stop_signal: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&mut self, app: AppHandle) -> Result<(), String> {
        self.stop_signal.store(false, Ordering::SeqCst);
        let stop_signal = Arc::clone(&self.stop_signal);

        let host = cpal::default_host();

        let device = host.default_output_device().ok_or("无法获取音频输出设备")?;

        let config = device.default_output_config().map_err(|e| e.to_string())?;

        let channels = config.channels() as usize;

        std::thread::Builder::new()
            .name("audio-capture".to_string())
            .spawn(move || {
                let mut ring_buffer: Vec<f32> = vec![0.0; FFT_SIZE];
                let mut ring_pos = 0usize;
                let mut hop_counter = 0usize;
                let mut prev_bars = vec![0.0f32; NUM_BARS];

                let hann_window: Vec<f32> = (0..FFT_SIZE)
                    .map(|i| {
                        0.5 * (1.0
                            - (2.0 * std::f32::consts::PI * i as f32 / (FFT_SIZE - 1) as f32).cos())
                    })
                    .collect();

                let mut planner = FftPlanner::new();
                let fft = planner.plan_fft_forward(FFT_SIZE);

                let stream = match device.build_input_stream(
                    &config.into(),
                    move |data: &[f32], _| {
                        let mono: Vec<f32> = data
                            .chunks(channels)
                            .map(|frame| frame.iter().sum::<f32>() / channels as f32)
                            .collect();

                        for sample in mono {
                            ring_buffer[ring_pos % FFT_SIZE] = sample;
                            ring_pos += 1;
                            hop_counter += 1;

                            if hop_counter >= HOP_SIZE {
                                hop_counter = 0;

                                let mut fft_input: Vec<Complex<f32>> = (0..FFT_SIZE)
                                    .map(|i| {
                                        let idx = (ring_pos + i) % FFT_SIZE;
                                        Complex::new(ring_buffer[idx] * hann_window[i], 0.0)
                                    })
                                    .collect();

                                fft.process(&mut fft_input);

                                let magnitudes: Vec<f32> = fft_input[..FFT_SIZE / 2]
                                    .iter()
                                    .map(|c| {
                                        let mag = c.norm() / FFT_SIZE as f32;
                                        let db = 20.0 * mag.max(1e-10).log10();
                                        ((db - MIN_DB) / (MAX_DB - MIN_DB)).clamp(0.0, 1.0)
                                    })
                                    .collect();

                                let freq_bins = FFT_SIZE / 2;
                                let new_bars: Vec<f32> = (0..NUM_BARS)
                                    .map(|i| {
                                        let lo = (freq_bins as f32
                                            * (i as f32 / NUM_BARS as f32).powf(1.2))
                                            as usize;
                                        let hi = (freq_bins as f32
                                            * ((i + 1) as f32 / NUM_BARS as f32).powf(1.2))
                                            as usize;
                                        let lo = lo.min(freq_bins - 1);
                                        let hi = hi.max(lo + 1).min(freq_bins);

                                        let sum: f32 = magnitudes[lo..hi]
                                            .iter()
                                            .map(|x| x * x)
                                            .sum();
                                        let rms = (sum / (hi - lo) as f32).sqrt();
                                        (rms * 2.5).min(1.0)
                                    })
                                    .collect();

                                let mut bars = prev_bars.clone();
                                for (i, bar) in bars.iter_mut().enumerate() {
                                    let target = new_bars[i];
                                    if target > *bar {
                                        *bar = 0.3 * prev_bars[i] + 0.7 * target;
                                    } else {
                                        *bar = 0.72 * prev_bars[i] + 0.28 * target;
                                    }
                                }
                                prev_bars.clone_from(&bars);

                                let _ = app.emit("spectrum-data", bars);
                            }
                        }
                    },
                    |err| eprintln!("音频流错误: {}", err),
                    None,
                ) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("构建音频流失败: {}", e);
                        return;
                    }
                };

                if let Err(e) = stream.play() {
                    eprintln!("播放音频流失败: {}", e);
                    return;
                }

                while !stop_signal.load(Ordering::SeqCst) {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            })
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn stop(&mut self) {
        self.stop_signal.store(true, Ordering::SeqCst);
    }
}
