use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, SampleRate, Stream, StreamConfig};
use std::sync::{Arc, Mutex};

const TARGET_SAMPLE_RATE: u32 = 16000;
const MAX_RECORDING_SECS: u64 = 600; // 10 minutes

pub struct AudioRecorder {
    host: Host,
    device: Option<Device>,
    stream: Option<Stream>,
    buffer: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
}

impl AudioRecorder {
    pub fn new() -> Self {
        let host = cpal::default_host();
        Self {
            host,
            device: None,
            stream: None,
            buffer: Arc::new(Mutex::new(Vec::new())),
            sample_rate: TARGET_SAMPLE_RATE,
        }
    }

    pub fn list_devices(&self) -> Vec<String> {
        self.host
            .input_devices()
            .map(|devices| {
                devices
                    .filter_map(|d| d.name().ok())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn select_device(&mut self, name: Option<&str>) -> Result<(), String> {
        self.device = match name {
            Some(name) => self
                .host
                .input_devices()
                .map_err(|e| format!("Failed to list devices: {e}"))?
                .find(|d| d.name().ok().as_deref() == Some(name)),
            None => self.host.default_input_device(),
        };
        if self.device.is_none() {
            return Err("No input device found".to_string());
        }
        Ok(())
    }

    pub fn start_recording(&mut self) -> Result<(), String> {
        let fallback_device;
        let device: &Device = if let Some(d) = self.device.as_ref() {
            d
        } else {
            fallback_device = self
                .host
                .default_input_device()
                .ok_or("No input device available")?;
            &fallback_device
        };

        let config = StreamConfig {
            channels: 1,
            sample_rate: SampleRate(TARGET_SAMPLE_RATE),
            buffer_size: cpal::BufferSize::Default,
        };

        let (actual_config, needs_resample) =
            match device.supported_input_configs() {
                Ok(mut configs) => {
                    let supports_target = configs.any(|c| {
                        c.channels() == 1
                            && c.min_sample_rate().0 <= TARGET_SAMPLE_RATE
                            && c.max_sample_rate().0 >= TARGET_SAMPLE_RATE
                    });
                    if supports_target {
                        (config, false)
                    } else {
                        let default_config = device
                            .default_input_config()
                            .map_err(|e| format!("Failed to get default config: {e}"))?;
                        self.sample_rate = default_config.sample_rate().0;
                        (
                            StreamConfig {
                                channels: default_config.channels(),
                                sample_rate: default_config.sample_rate(),
                                buffer_size: cpal::BufferSize::Default,
                            },
                            true,
                        )
                    }
                }
                Err(_) => (config, false),
            };

        let buffer = self.buffer.clone();
        let max_samples = (TARGET_SAMPLE_RATE as u64 * MAX_RECORDING_SECS) as usize;
        let channels = actual_config.channels as usize;
        let source_rate = actual_config.sample_rate.0;
        let target_rate = TARGET_SAMPLE_RATE;

        {
            let mut buf = buffer.lock().map_err(|e| format!("Lock error: {e}"))?;
            buf.clear();
        }

        let stream = device
            .build_input_stream(
                &actual_config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let mut buf = match buffer.lock() {
                        Ok(b) => b,
                        Err(_) => return,
                    };
                    if buf.len() >= max_samples {
                        return;
                    }

                    let mono_samples: Vec<f32> = if channels > 1 {
                        data.chunks(channels)
                            .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
                            .collect()
                    } else {
                        data.to_vec()
                    };

                    if needs_resample && source_rate != target_rate {
                        let ratio = source_rate as f64 / target_rate as f64;
                        let mut i = 0.0_f64;
                        while (i as usize) < mono_samples.len() {
                            buf.push(mono_samples[i as usize]);
                            i += ratio;
                        }
                    } else {
                        buf.extend_from_slice(&mono_samples);
                    }
                },
                |err| {
                    eprintln!("Audio stream error: {err}");
                },
                None,
            )
            .map_err(|e| format!("Failed to build input stream: {e}"))?;

        stream
            .play()
            .map_err(|e| format!("Failed to start stream: {e}"))?;

        self.stream = Some(stream);
        Ok(())
    }

    pub fn stop_recording(&mut self) -> Vec<f32> {
        self.stream = None;
        let buf = self.buffer.lock().unwrap_or_else(|e| e.into_inner());
        buf.clone()
    }

    pub fn recording_duration_secs(&self) -> f32 {
        let buf = self.buffer.lock().unwrap_or_else(|e| e.into_inner());
        buf.len() as f32 / TARGET_SAMPLE_RATE as f32
    }

    pub fn buffer_ref(&self) -> Arc<Mutex<Vec<f32>>> {
        self.buffer.clone()
    }
}
