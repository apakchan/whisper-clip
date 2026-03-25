#[path = "../src/encoder.rs"]
mod encoder;

#[test]
fn test_encode_silence_produces_valid_wav() {
    let samples: Vec<f32> = vec![0.0; 16000];
    let wav_bytes = encoder::encode_wav(&samples, 16000).unwrap();
    assert_eq!(&wav_bytes[0..4], b"RIFF");
    assert_eq!(&wav_bytes[8..12], b"WAVE");
    assert!(wav_bytes.len() > 44);
}

#[test]
fn test_encode_wav_correct_size() {
    let samples: Vec<f32> = vec![0.5; 8000];
    let wav_bytes = encoder::encode_wav(&samples, 16000).unwrap();
    // 16-bit mono WAV: header(44) + samples(8000 * 2 bytes) = 16044
    assert_eq!(wav_bytes.len(), 16044);
}

#[test]
fn test_encode_empty_samples_returns_error() {
    let samples: Vec<f32> = vec![];
    let result = encoder::encode_wav(&samples, 16000);
    assert!(result.is_err());
}
