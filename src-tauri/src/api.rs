use reqwest::multipart;
use serde::Deserialize;

pub const GROQ_TRANSCRIPTION_URL: &str =
    "https://api.groq.com/openai/v1/audio/transcriptions";

const TIMEOUT_SECS: u64 = 30;

#[derive(Deserialize)]
struct TranscriptionResponse {
    text: Option<String>,
}

pub fn parse_transcription_response(json: &str) -> Result<String, String> {
    let resp: TranscriptionResponse =
        serde_json::from_str(json).map_err(|e| format!("Failed to parse response: {e}"))?;
    resp.text
        .ok_or_else(|| "Response missing 'text' field".to_string())
}

pub async fn transcribe(api_key: &str, model: &str, prompt: &str, wav_bytes: Vec<u8>) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let file_part = multipart::Part::bytes(wav_bytes)
        .file_name("audio.wav")
        .mime_str("audio/wav")
        .map_err(|e| format!("Failed to create multipart: {e}"))?;

    let form = multipart::Form::new()
        .part("file", file_part)
        .text("model", model.to_string())
        .text("response_format", "json")
        .text("prompt", prompt.to_string())
        .text("temperature", "0");

    let response = client
        .post(GROQ_TRANSCRIPTION_URL)
        .header("Authorization", format!("Bearer {api_key}"))
        .multipart(form)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                "API request timed out (30s). Please try again.".to_string()
            } else {
                format!("Network error: {e}")
            }
        })?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {e}"))?;

    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return Err("Rate limited. Please try again in a few seconds.".to_string());
    }

    if !status.is_success() {
        return Err(format!("API error ({}): {}", status, body));
    }

    parse_transcription_response(&body)
}
