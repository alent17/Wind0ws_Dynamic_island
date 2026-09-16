use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rustfft::{num_complex::Complex, FftPlanner};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};
use tauri::{AppHandle, Emitter};

const FFT_SIZE: usize = 2048;
// A 48 kHz stream needs about 47 analyses per second at this hop size.
// The former 256-sample hop ran the 2048-point FFT about 188 times/s on
// the real-time audio callback, competing with WebView animation.
const HOP_SIZE: usize = 1024;
const NUM_BARS: usize = 6;

const FREQ_BANDS: [(f32, f32); NUM_BARS] = [
    (20.0, 250.0),
    (250.0, 600.0),
    (600.0, 2000.0),
    (2000.0, 5000.0),
    (5000.0, 10000.0),
    (10000.0, 20000.0),
];

const BAND_GAINS: [f32; NUM_BARS] = [1.15, 1.65, 2.2, 3.4, 5.4, 8.0];

const SMOOTH_ATTACK: f32 = 0.38;
const SMOOTH_RELEASE: f32 = 0.82;

const MIN_DB: f32 = -78.0;
const MAX_DB: f32 = -12.0;

pub struct SpectrumCapture {
    running: Arc<AtomicBool>,
    generation: Arc<AtomicU64>,
}

impl SpectrumCapture {
    pub fn new() -> Self {
        SpectrumCapture {
            running: Arc::new(AtomicBool::new(false)),
            generation: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn start(&self, app: AppHandle) -> Result<(), String> {
        if self
            .running
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Ok(());
        }
        let token = self.generation.fetch_add(1, Ordering::AcqRel) + 1;
        let running = self.running.clone();
        let generation = self.generation.clone();
        let published_bars = Arc::new(Mutex::new([0.0f32; NUM_BARS]));

        let publisher_running = running.clone();
        let publisher_generation = generation.clone();
        let publisher_bars = published_bars.clone();
        let publisher_app = app.clone();
        std::thread::spawn(move || {
            while publisher_running.load(Ordering::Acquire)
                && publisher_generation.load(Ordering::Acquire) == token
            {
                if let Ok(values) = publisher_bars.lock() {
                    let _ = publisher_app.emit("spectrum-data", values.to_vec());
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        });

        std::thread::spawn(move || {
            let host = cpal::default_host();

            let device = match host.default_output_device() {
                Some(d) => d,
                None => {
                    eprintln!("[Spectrum] 无法获取音频输出设备");
                    running.store(false, Ordering::Release);
                    return;
                }
            };

            let config = match device.default_output_config() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("[Spectrum] 获取音频配置失败: {e}");
                    running.store(false, Ordering::Release);
                    return;
                }
            };

            let sample_rate = config.sample_rate().0 as f32;
            let channels = config.channels() as usize;

            let hann_window: Vec<f32> = (0..FFT_SIZE)
                .map(|i| {
                    0.5 * (1.0
                        - (2.0 * std::f32::consts::PI * i as f32 / (FFT_SIZE - 1) as f32).cos())
                })
                .collect();

            let mut planner = FftPlanner::<f32>::new();
            let fft = planner.plan_fft_forward(FFT_SIZE);

            let mut ring_buf = vec![0.0f32; FFT_SIZE];
            let mut ring_pos: usize = 0;
            let mut hop_counter: usize = 0;
            let mut smoothed = [0.0f32; NUM_BARS];
            let mut fft_buf = vec![Complex::new(0.0f32, 0.0f32); FFT_SIZE];
            let mut magnitudes = vec![0.0f32; FFT_SIZE / 2];
            let mut new_bars = [0.0f32; NUM_BARS];
            let callback_running = running.clone();
            let callback_generation = generation.clone();
            let callback_bars = published_bars.clone();
            let error_running = running.clone();
            let error_generation = generation.clone();

            let stream = device
                .build_input_stream(
                    &config.into(),
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        if !callback_running.load(Ordering::Acquire)
                            || callback_generation.load(Ordering::Acquire) != token
                        {
                            return;
                        }
                        for frame in data.chunks(channels) {
                            let sample = frame.iter().sum::<f32>() / channels as f32;
                            ring_buf[ring_pos % FFT_SIZE] = sample;
                            ring_pos += 1;
                            hop_counter += 1;

                            if hop_counter < HOP_SIZE {
                                continue;
                            }
                            hop_counter = 0;

                            for i in 0..FFT_SIZE {
                                let idx = (ring_pos + i) % FFT_SIZE;
                                fft_buf[i] = Complex::new(ring_buf[idx] * hann_window[i], 0.0);
                            }

                            fft.process(&mut fft_buf);

                            let freq_bins = FFT_SIZE / 2;
                            for (index, complex) in fft_buf[..freq_bins].iter().enumerate() {
                                let mag = complex.norm() / FFT_SIZE as f32;
                                let db = 20.0 * mag.max(1e-10_f32).log10();
                                magnitudes[index] =
                                    ((db - MIN_DB) / (MAX_DB - MIN_DB)).clamp(0.0, 1.0);
                            }

                            for (index, &(freq_lo, freq_hi)) in FREQ_BANDS.iter().enumerate() {
                                let bin_lo = ((freq_lo * FFT_SIZE as f32 / sample_rate) as usize)
                                    .min(freq_bins - 1);
                                let bin_hi = ((freq_hi * FFT_SIZE as f32 / sample_rate) as usize)
                                    .min(freq_bins)
                                    .max(bin_lo + 1);
                                let n = (bin_hi - bin_lo) as f32;
                                let rms = (magnitudes[bin_lo..bin_hi]
                                    .iter()
                                    .map(|x| x * x)
                                    .sum::<f32>()
                                    / n)
                                    .sqrt();
                                new_bars[index] = (rms * BAND_GAINS[index]).min(1.0).sqrt();
                            }

                            for i in 0..NUM_BARS {
                                smoothed[i] = if new_bars[i] > smoothed[i] {
                                    SMOOTH_ATTACK * smoothed[i]
                                        + (1.0 - SMOOTH_ATTACK) * new_bars[i]
                                } else {
                                    SMOOTH_RELEASE * smoothed[i]
                                        + (1.0 - SMOOTH_RELEASE) * new_bars[i]
                                };
                            }

                            if let Ok(mut values) = callback_bars.try_lock() {
                                *values = smoothed;
                            }
                        }
                    },
                    move |err| {
                        eprintln!("[Spectrum] 音频流错误: {err}");
                        if error_generation.load(Ordering::Acquire) == token {
                            error_running.store(false, Ordering::Release);
                        }
                    },
                    None,
                )
                .map_err(|e| format!("创建音频流失败: {e}"));

            let stream = match stream {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[Spectrum] {e}");
                    running.store(false, Ordering::Release);
                    return;
                }
            };

            if let Err(e) = stream.play() {
                eprintln!("[Spectrum] 启动音频流失败: {e}");
                running.store(false, Ordering::Release);
                return;
            }

            println!("[Spectrum] 启动成功 | 采样率: {sample_rate} Hz | {NUM_BARS} 段");

            while running.load(Ordering::Acquire) && generation.load(Ordering::Acquire) == token {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            drop(stream);
            println!("[Spectrum] 已停止");
        });

        Ok(())
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Release);
        self.generation.fetch_add(1, Ordering::AcqRel);
    }
}

impl Default for SpectrumCapture {
    fn default() -> Self {
        Self::new()
    }
}
