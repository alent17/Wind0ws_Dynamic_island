use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rustfft::{FftPlanner, num_complex::Complex};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tauri::{AppHandle, Emitter};

const FFT_SIZE: usize = 2048;
const HOP_SIZE: usize = 256;
const NUM_BARS: usize = 6;

const FREQ_BANDS: [(f32, f32); NUM_BARS] = [
    (20.0,    250.0),
    (250.0,   600.0),
    (600.0,   2000.0),
    (2000.0,  5000.0),
    (5000.0,  10000.0),
    (10000.0, 20000.0),
];

const BAND_GAINS: [f32; NUM_BARS] = [
    1.2,
    1.8,
    2.5,
    4.0,
    7.0,
    12.0,
];

const SMOOTH_ATTACK: f32  = 0.3;
const SMOOTH_RELEASE: f32 = 0.75;

const MIN_DB: f32 = -85.0;
const MAX_DB: f32 = -5.0;

pub struct SpectrumCapture {
    running: Arc<AtomicBool>,
}

impl SpectrumCapture {
    pub fn new() -> Self {
        SpectrumCapture {
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&self, app: AppHandle) -> Result<(), String> {
        if self.running.load(Ordering::Relaxed) {
            return Ok(());
        }

        self.running.store(true, Ordering::Relaxed);
        let running = self.running.clone();

        std::thread::spawn(move || {
            let host = cpal::default_host();

            let device = match host.default_output_device() {
                Some(d) => d,
                None => {
                    eprintln!("[Spectrum] 无法获取音频输出设备");
                    running.store(false, Ordering::Relaxed);
                    return;
                }
            };

            let config = match device.default_output_config() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("[Spectrum] 获取音频配置失败: {e}");
                    running.store(false, Ordering::Relaxed);
                    return;
                }
            };

            let sample_rate = config.sample_rate().0 as f32;
            let channels = config.channels() as usize;

            let hann_window: Vec<f32> = (0..FFT_SIZE)
                .map(|i| {
                    0.5 * (1.0
                        - (2.0 * std::f32::consts::PI * i as f32
                            / (FFT_SIZE - 1) as f32)
                            .cos())
                })
                .collect();

            let mut planner = FftPlanner::<f32>::new();
            let fft = planner.plan_fft_forward(FFT_SIZE);

            let mut ring_buf = vec![0.0f32; FFT_SIZE];
            let mut ring_pos: usize = 0;
            let mut hop_counter: usize = 0;
            let mut smoothed = vec![0.0f32; NUM_BARS];

            let stream = device
                .build_input_stream(
                    &config.into(),
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        let mono: Vec<f32> = data
                            .chunks(channels)
                            .map(|frame| frame.iter().sum::<f32>() / channels as f32)
                            .collect();

                        for sample in mono {
                            ring_buf[ring_pos % FFT_SIZE] = sample;
                            ring_pos += 1;
                            hop_counter += 1;

                            if hop_counter < HOP_SIZE {
                                continue;
                            }
                            hop_counter = 0;

                            let mut fft_buf: Vec<Complex<f32>> = (0..FFT_SIZE)
                                .map(|i| {
                                    let idx = (ring_pos + i) % FFT_SIZE;
                                    Complex::new(ring_buf[idx] * hann_window[i], 0.0)
                                })
                                .collect();

                            fft.process(&mut fft_buf);

                            let freq_bins = FFT_SIZE / 2;
                            let magnitudes: Vec<f32> = fft_buf[..freq_bins]
                                .iter()
                                .map(|c| {
                                    let mag = c.norm() / FFT_SIZE as f32;
                                    let db = 20.0 * mag.max(1e-10_f32).log10();
                                    ((db - MIN_DB) / (MAX_DB - MIN_DB)).clamp(0.0, 1.0)
                                })
                                .collect();

                            let new_bars: Vec<f32> = FREQ_BANDS
                                .iter()
                                .enumerate()
                                .map(|(idx, &(freq_lo, freq_hi))| {
                                    let bin_lo = (freq_lo * FFT_SIZE as f32 / sample_rate)
                                        as usize;
                                    let bin_hi = (freq_hi * FFT_SIZE as f32 / sample_rate)
                                        as usize;
                                    let bin_lo = bin_lo.min(freq_bins - 1);
                                    let bin_hi = bin_hi.min(freq_bins).max(bin_lo + 1);

                                    let n = (bin_hi - bin_lo) as f32;
                                    let rms = (magnitudes[bin_lo..bin_hi]
                                        .iter()
                                        .map(|x| x * x)
                                        .sum::<f32>()
                                        / n)
                                        .sqrt();

                                    let v = (rms * BAND_GAINS[idx]).min(1.0);
                                    v.sqrt()
                                })
                                .collect();

                            for i in 0..NUM_BARS {
                                smoothed[i] = if new_bars[i] > smoothed[i] {
                                    SMOOTH_ATTACK * smoothed[i]
                                        + (1.0 - SMOOTH_ATTACK) * new_bars[i]
                                } else {
                                    SMOOTH_RELEASE * smoothed[i]
                                        + (1.0 - SMOOTH_RELEASE) * new_bars[i]
                                };
                            }

                            let _ = app.emit("spectrum-data", smoothed.clone());
                        }
                    },
                    |err| eprintln!("[Spectrum] 音频流错误: {err}"),
                    None,
                )
                .map_err(|e| format!("创建音频流失败: {e}"));

            let stream = match stream {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[Spectrum] {e}");
                    running.store(false, Ordering::Relaxed);
                    return;
                }
            };

            if let Err(e) = stream.play() {
                eprintln!("[Spectrum] 启动音频流失败: {e}");
                running.store(false, Ordering::Relaxed);
                return;
            }

            println!("[Spectrum] 启动成功 | 采样率: {sample_rate} Hz | {NUM_BARS} 段");

            while running.load(Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            drop(stream);
            println!("[Spectrum] 已停止");
        });

        Ok(())
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

impl Default for SpectrumCapture {
    fn default() -> Self {
        Self::new()
    }
}
