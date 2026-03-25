use hound::{SampleFormat, WavSpec, WavWriter};
use std::io::Cursor;

pub fn encode_wav(samples: &[f32], sample_rate: u32) -> Result<Vec<u8>, String> {
    if samples.is_empty() {
        return Err("No audio samples to encode".to_string());
    }

    let spec = WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };

    let mut buffer = Cursor::new(Vec::new());
    let mut writer =
        WavWriter::new(&mut buffer, spec).map_err(|e| format!("Failed to create WAV writer: {e}"))?;

    for &sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let int_sample = (clamped * i16::MAX as f32) as i16;
        writer
            .write_sample(int_sample)
            .map_err(|e| format!("Failed to write sample: {e}"))?;
    }

    writer
        .finalize()
        .map_err(|e| format!("Failed to finalize WAV: {e}"))?;

    Ok(buffer.into_inner())
}
