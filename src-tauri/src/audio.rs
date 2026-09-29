use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rustfft::{num_complex::Complex, FftPlanner};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

const FFT_SIZE: usize = 2048;
// A 48 kHz stream needs about 23 analyses per second with non-overlapping
// windows, matching the event and canvas cadence.
const NUM_BARS: usize = 6;
const SPECTRUM_CHANGE_THRESHOLD: f32 = 0.01;
const SPECTRUM_HEARTBEAT: Duration = Duration::from_secs(1);

fn should_publish_spectrum(
    previous: Option<&[f32; NUM_BARS]>,
    current: &[f32; NUM_BARS],
    elapsed: Duration,
) -> bool {
    previous.is_none_or(|previous| {
        previous
            .iter()
            .zip(current)
            .any(|(old, next)| (*old - *next).abs() >= SPECTRUM_CHANGE_THRESHOLD)
    }) || elapsed >= SPECTRUM_HEARTBEAT
}

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
        let published_bars = Arc::new(Mutex::new(([0.0f32; NUM_BARS], std::time::Instant::now())));

        let publisher_running = running.clone();
        let publisher_generation = generation.clone();
        let publisher_bars = published_bars.clone();
        let publisher_app = app.clone();
        std::thread::spawn(move || {
            let mut previous = None;
            let mut last_emit = Instant::now();
            while publisher_running.load(Ordering::Acquire)
                && publisher_generation.load(Ordering::Acquire) == token
            {
                let current = publisher_bars.lock().ok().map(|values| {
                    if values.1.elapsed() > Duration::from_millis(200) {
                        [0.0; NUM_BARS]
                    } else {
                        values.0
                    }
                });
                if let Some(current) = current {
                    if should_publish_spectrum(previous.as_ref(), &current, last_emit.elapsed()) {
                        let _ = publisher_app.emit("spectrum-data", current.to_vec());
                        previous = Some(current);
                        last_emit = Instant::now();
                    }
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        });

        std::thread::spawn(move || {
            let host = cpal::default_host();

            let device = match host.default_output_device() {
                Some(d) => d,
                None => {
                    eprintln!("[Spectrum] 无法获取音频输出设备");
                    if generation.load(Ordering::Acquire) == token {
                        running.store(false, Ordering::Release);
                    }
                    return;
                }
            };

            let config = match device.default_output_config() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("[Spectrum] 获取音频配置失败: {e}");
                    if generation.load(Ordering::Acquire) == token {
                        running.store(false, Ordering::Release);
                    }
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
            let freq_bins = FFT_SIZE / 2;
            let band_ranges = FREQ_BANDS.map(|(freq_lo, freq_hi)| {
                let bin_lo =
                    ((freq_lo * FFT_SIZE as f32 / sample_rate) as usize).min(freq_bins - 1);
                let bin_hi = ((freq_hi * FFT_SIZE as f32 / sample_rate) as usize)
                    .min(freq_bins)
                    .max(bin_lo + 1);
                (bin_lo, bin_hi)
            });

            let mut frame_buf = vec![0.0f32; FFT_SIZE];
            let mut frame_pos: usize = 0;
            let mut smoothed = [0.0f32; NUM_BARS];
            let mut fft_buf = vec![Complex::new(0.0f32, 0.0f32); FFT_SIZE];
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
                            frame_buf[frame_pos] = sample;
                            frame_pos += 1;
                            if frame_pos < FFT_SIZE {
                                continue;
                            }
                            frame_pos = 0;

                            for i in 0..FFT_SIZE {
                                fft_buf[i] = Complex::new(frame_buf[i] * hann_window[i], 0.0);
                            }

                            fft.process(&mut fft_buf);

                            for (index, &(bin_lo, bin_hi)) in band_ranges.iter().enumerate() {
                                let n = (bin_hi - bin_lo) as f32;
                                let rms = (fft_buf[bin_lo..bin_hi]
                                    .iter()
                                    .map(|bin| bin.norm_sqr())
                                    .sum::<f32>()
                                    / n)
                                    .sqrt()
                                    / FFT_SIZE as f32;
                                new_bars[index] = band_level(rms, BAND_GAINS[index]);
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
                                *values = (smoothed, std::time::Instant::now());
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
                    if generation.load(Ordering::Acquire) == token {
                        running.store(false, Ordering::Release);
                    }
                    return;
                }
            };

            if let Err(e) = stream.play() {
                eprintln!("[Spectrum] 启动音频流失败: {e}");
                if generation.load(Ordering::Acquire) == token {
                    running.store(false, Ordering::Release);
                }
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

// Apply gain in the linear amplitude domain before converting to dB.
// Multiplying an already normalized dB value saturated the treble bars.
fn band_level(rms: f32, gain: f32) -> f32 {
    let db = 20.0 * (rms * gain).max(1e-10).log10();
    ((db - MIN_DB) / (MAX_DB - MIN_DB)).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::{band_level, should_publish_spectrum, NUM_BARS};
    use std::time::Duration;

    #[test]
    fn spectrum_publisher_sends_changes_and_silent_heartbeats() {
        let zero = [0.0; NUM_BARS];
        let mut changed = zero;
        changed[0] = 0.2;
        assert!(should_publish_spectrum(None, &zero, Duration::ZERO));
        assert!(!should_publish_spectrum(
            Some(&zero),
            &zero,
            Duration::from_millis(950)
        ));
        assert!(should_publish_spectrum(
            Some(&zero),
            &changed,
            Duration::ZERO
        ));
        assert!(should_publish_spectrum(
            Some(&zero),
            &zero,
            Duration::from_secs(1)
        ));
    }
    #[test]
    fn silence_and_quiet_treble_do_not_pin_the_bars() {
        assert_eq!(band_level(0.0, 8.0), 0.0);
        let quiet = band_level(0.001, 8.0);
        let loud = band_level(0.01, 8.0);
        assert!(quiet > 0.0 && quiet < loud && loud < 1.0);
    }
}
