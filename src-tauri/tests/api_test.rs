#[path = "../src/api.rs"]
mod api;

#[test]
fn test_build_request_url() {
    assert_eq!(
        api::GROQ_TRANSCRIPTION_URL,
        "https://api.groq.com/openai/v1/audio/transcriptions"
    );
}

#[test]
fn test_parse_transcription_response() {
    let json = r#"{"text": "Hello, world!"}"#;
    let result = api::parse_transcription_response(json).unwrap();
    assert_eq!(result, "Hello, world!");
}

#[test]
fn test_parse_invalid_response() {
    let json = r#"{"error": "something went wrong"}"#;
    let result = api::parse_transcription_response(json);
    assert!(result.is_err());
}

#[test]
fn test_parse_empty_text_response() {
    let json = r#"{"text": ""}"#;
    let result = api::parse_transcription_response(json).unwrap();
    assert_eq!(result, "");
}
