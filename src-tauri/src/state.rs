#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppStatus {
    Idle,
    Recording,
    Transcribing,
    Done,
    Error,
}

pub struct AppState {
    status: AppStatus,
    last_transcription: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            status: AppStatus::Idle,
            last_transcription: None,
        }
    }

    pub fn status(&self) -> AppStatus {
        self.status
    }

    pub fn set_status(&mut self, status: AppStatus) {
        self.status = status;
    }

    pub fn is_recording(&self) -> bool {
        self.status == AppStatus::Recording
    }

    pub fn last_transcription(&self) -> Option<&str> {
        self.last_transcription.as_deref()
    }

    pub fn set_last_transcription(&mut self, text: String) {
        self.last_transcription = Some(text);
    }
}
