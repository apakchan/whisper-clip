use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, Stream, StreamConfig};
use std::sync::{Arc, Mutex};

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
            sample_rate: 16000,
        }
    }

    pub fn list_devices(&self) -> Vec<String> {
        self.host
            .input_devices()
            .map(|devices| devices.filter_map(|d| d.name().ok()).collect())
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

        // Use the device's default config — no resampling, no quality loss.
        // Whisper handles sample rate conversion internally.
        let default_config = device
            .default_input_config()
            .map_err(|e| format!("Failed to get default config: {e}"))?;

        let actual_rate = default_config.sample_rate().0;
        let actual_channels = default_config.channels() as usize;

        self.sample_rate = actual_rate;

        let config = StreamConfig {
            channels: default_config.channels(),
            sample_rate: default_config.sample_rate(),
            buffer_size: cpal::BufferSize::Default,
        };

        let buffer = self.buffer.clone();
        // Max samples for mono at actual rate
        let max_samples = (actual_rate as u64 * MAX_RECORDING_SECS) as usize;

        {
            let mut buf = buffer.lock().map_err(|e| format!("Lock error: {e}"))?;
            buf.clear();
        }

        let stream = device
            .build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let mut buf = match buffer.lock() {
                        Ok(b) => b,
                        Err(_) => return,
                    };
                    if buf.len() >= max_samples {
                        return;
                    }

                    // Mix to mono if stereo/multi-channel
                    if actual_channels > 1 {
                        for chunk in data.chunks(actual_channels) {
                            let mono = chunk.iter().sum::<f32>() / actual_channels as f32;
                            buf.push(mono);
                        }
                    } else {
                        buf.extend_from_slice(data);
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

        eprintln!("Recording at {}Hz, {} channel(s) → mono", actual_rate, actual_channels);

        Ok(())
    }

    pub fn stop_recording(&mut self) -> Vec<f32> {
        self.stream = None;
        let buf = self.buffer.lock().unwrap_or_else(|e| e.into_inner());
        eprintln!("Stopped recording: {} samples ({:.1}s at {}Hz)",
            buf.len(),
            buf.len() as f32 / self.sample_rate as f32,
            self.sample_rate);
        buf.clone()
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}
