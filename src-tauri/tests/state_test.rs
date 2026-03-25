#[path = "../src/state.rs"]
mod state;

#[test]
fn test_initial_state_is_idle() {
    let app_state = state::AppState::new();
    assert_eq!(app_state.status(), state::AppStatus::Idle);
    assert!(!app_state.is_recording());
}

#[test]
fn test_state_transitions() {
    let mut app_state = state::AppState::new();

    app_state.set_status(state::AppStatus::Recording);
    assert_eq!(app_state.status(), state::AppStatus::Recording);
    assert!(app_state.is_recording());

    app_state.set_status(state::AppStatus::Transcribing);
    assert_eq!(app_state.status(), state::AppStatus::Transcribing);
    assert!(!app_state.is_recording());

    app_state.set_status(state::AppStatus::Done);
    assert_eq!(app_state.status(), state::AppStatus::Done);
}

#[test]
fn test_last_transcription() {
    let mut app_state = state::AppState::new();
    assert_eq!(app_state.last_transcription(), None);

    app_state.set_last_transcription("Hello world".to_string());
    assert_eq!(app_state.last_transcription(), Some("Hello world"));
}
