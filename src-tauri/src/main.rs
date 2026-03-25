#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod audio;
mod clipboard;
mod config;
mod encoder;
mod hotkey;
mod state;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItem, MenuItemBuilder},
    tray::TrayIconBuilder,
    AppHandle, Manager, Wry, WebviewUrl, WebviewWindowBuilder,
};

use state::{AppState, AppStatus};

/// Helper to load a tray icon image from the icons directory.
fn load_icon(app: &AppHandle, name: &str) -> Option<Image<'static>> {
    let path = app
        .path()
        .resource_dir()
        .ok()?
        .join("icons")
        .join(name);
    Image::from_path(&path).ok().map(|img| img.to_owned())
}

/// Return the icon filename for the given status.
fn icon_for_status(status: AppStatus) -> &'static str {
    match status {
        AppStatus::Idle => "icon-idle.png",
        AppStatus::Recording => "icon-recording.png",
        AppStatus::Transcribing => "icon-transcribing.png",
        AppStatus::Done => "icon-done.png",
        AppStatus::Error => "icon-error.png",
    }
}

/// Update the tray icon to match the current app status.
fn update_tray_icon(app: &AppHandle, status: AppStatus) {
    use tauri::tray::TrayIconId;
    if let Some(tray) = app.tray_by_id(&TrayIconId::new("main")) {
        if let Some(icon) = load_icon(app, icon_for_status(status)) {
            let _ = tray.set_icon(Some(icon));
        }
    }
}

/// Truncate text to at most 30 Unicode scalar values (safe UTF-8 truncation).
fn truncate_for_menu(text: &str) -> String {
    text.chars().take(30).collect::<String>()
}

fn main() {
    // Load config before builder so we can use it in the setup closure.
    let app_config = config::load_config(&config::config_path()).unwrap_or_default();
    let api_key = Arc::new(Mutex::new(app_config.api_key.clone()));
    let mic_device = app_config.microphone_device.clone();

    // Shared app state (status + last transcription).
    let app_state: Arc<Mutex<AppState>> = Arc::new(Mutex::new(AppState::new()));

    tauri::Builder::default()
        .setup(move |app| {
            let handle = app.handle().clone();

            // ---- Build tray menu ----
            let last_text_item = MenuItemBuilder::with_id("last_text", "（尚无转写结果）")
                .enabled(true)
                .build(app)?;

            let hotkey_item = MenuItemBuilder::with_id("hotkey_hint", "⌘⇧Space 按住录音")
                .enabled(false)
                .build(app)?;

            let separator1 = tauri::menu::PredefinedMenuItem::separator(app)?;

            let settings_item = MenuItemBuilder::with_id("settings", "设置")
                .enabled(true)
                .build(app)?;

            let separator2 = tauri::menu::PredefinedMenuItem::separator(app)?;

            let quit_item = MenuItemBuilder::with_id("quit", "退出")
                .enabled(true)
                .build(app)?;

            let tray_menu = MenuBuilder::new(app)
                .item(&last_text_item)
                .item(&hotkey_item)
                .item(&separator1)
                .item(&settings_item)
                .item(&separator2)
                .item(&quit_item)
                .build()?;

            // ---- Build tray icon ----
            let initial_icon = load_icon(&handle, "icon-idle.png");
            let mut tray_builder = TrayIconBuilder::with_id("main")
                .menu(&tray_menu)
                .show_menu_on_left_click(true)
                .tooltip("Whisper Clip");

            if let Some(icon) = initial_icon {
                tray_builder = tray_builder.icon(icon);
            }

            tray_builder.build(app)?;

            // Clone the last_text_item so both the menu handler and the worker thread can use it.
            let last_text_for_event = last_text_item.clone();
            let last_text_for_worker = last_text_item.clone();

            // ---- Menu event handler ----
            let menu_handle = handle.clone();
            app.on_menu_event(move |_app, event| {
                match event.id().as_ref() {
                    "last_text" => {
                        // Copy last transcription to clipboard when user clicks the text item.
                        if let Ok(text) = last_text_for_event.text() {
                            if text != "（尚无转写结果）" {
                                let _ = clipboard::copy_to_clipboard(&text);
                            }
                        }
                    }
                    "settings" => {
                        let _ = WebviewWindowBuilder::new(
                            &menu_handle,
                            "settings",
                            WebviewUrl::App("index.html".into()),
                        )
                        .title("设置 — Whisper Clip")
                        .inner_size(480.0, 360.0)
                        .resizable(false)
                        .build();
                    }
                    "quit" => {
                        menu_handle.exit(0);
                    }
                    _ => {}
                }
            });

            // ---- Dedicated worker thread for hotkey + audio pipeline ----
            // The AudioRecorder contains cpal::Stream which is !Send on CoreAudio (macOS).
            // We keep the recorder entirely on this dedicated OS thread, never crossing thread
            // boundaries with it, so no unsafe Send wrapper is needed.
            let worker_handle = handle.clone();
            let worker_state = Arc::clone(&app_state);
            let worker_api_key = Arc::clone(&api_key);

            std::thread::spawn(move || {
                let mut recorder = audio::AudioRecorder::new();

                // Pre-select microphone device if configured.
                let _ = recorder.select_device(mic_device.as_deref());

                let listener = match hotkey::HotkeyListener::start() {
                    Ok(l) => l,
                    Err(e) => {
                        eprintln!("Failed to start hotkey listener: {e}");
                        return;
                    }
                };

                loop {
                    // Blocking wait for the next hotkey event.
                    let event = match listener.recv() {
                        Ok(ev) => ev,
                        Err(_) => break, // channel closed — app is shutting down
                    };

                    let current_status = {
                        let st = worker_state.lock().unwrap();
                        st.status()
                    };

                    match event {
                        hotkey::HotkeyEvent::RecordStart => {
                            if current_status == AppStatus::Idle
                                || current_status == AppStatus::Error
                            {
                                match recorder.start_recording() {
                                    Ok(()) => {
                                        {
                                            let mut st = worker_state.lock().unwrap();
                                            st.set_status(AppStatus::Recording);
                                        }
                                        update_tray_icon(&worker_handle, AppStatus::Recording);
                                    }
                                    Err(e) => {
                                        eprintln!("start_recording error: {e}");
                                        transition_to_error(&worker_handle, &worker_state);
                                    }
                                }
                            }
                        }

                        hotkey::HotkeyEvent::RecordStop => {
                            if current_status != AppStatus::Recording {
                                continue;
                            }

                            let sample_rate = recorder.sample_rate();
                            let samples = recorder.stop_recording();

                            {
                                let mut st = worker_state.lock().unwrap();
                                st.set_status(AppStatus::Transcribing);
                            }
                            update_tray_icon(&worker_handle, AppStatus::Transcribing);

                            // Spawn an async task for the encode+transcribe pipeline.
                            // Clone all the values we need — AudioRecorder stays on this thread.
                            let pipeline_handle = worker_handle.clone();
                            let pipeline_state = Arc::clone(&worker_state);
                            let pipeline_api_key = Arc::clone(&worker_api_key);
                            let pipeline_last_item = last_text_for_worker.clone();

                            tauri::async_runtime::spawn(async move {
                                run_pipeline(
                                    samples,
                                    sample_rate,
                                    pipeline_api_key,
                                    pipeline_state,
                                    pipeline_handle,
                                    pipeline_last_item,
                                )
                                .await;
                            });
                        }
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Transition to Error state and schedule auto-recovery to Idle after 3 seconds.
fn transition_to_error(app: &AppHandle, state: &Arc<Mutex<AppState>>) {
    {
        let mut st = state.lock().unwrap();
        st.set_status(AppStatus::Error);
    }
    update_tray_icon(app, AppStatus::Error);

    let recover_handle = app.clone();
    let recover_state = Arc::clone(state);
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(3)).await;
        let mut st = recover_state.lock().unwrap();
        if st.status() == AppStatus::Error {
            st.set_status(AppStatus::Idle);
            drop(st);
            update_tray_icon(&recover_handle, AppStatus::Idle);
        }
    });
}

/// Encode audio, transcribe via API, copy to clipboard, and update tray menu.
async fn run_pipeline(
    samples: Vec<f32>,
    sample_rate: u32,
    api_key: Arc<Mutex<String>>,
    state: Arc<Mutex<AppState>>,
    app: AppHandle,
    last_text_item: MenuItem<Wry>,
) {
    // Encode WAV.
    let wav_bytes = match encoder::encode_wav(&samples, sample_rate) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("encode_wav error: {e}");
            transition_to_error(&app, &state);
            return;
        }
    };

    // Transcribe via Groq Whisper API.
    let key = api_key.lock().unwrap().clone();
    let text = match api::transcribe(&key, wav_bytes).await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("transcribe error: {e}");
            transition_to_error(&app, &state);
            return;
        }
    };

    // Copy to clipboard.
    if let Err(e) = clipboard::copy_to_clipboard(&text) {
        eprintln!("clipboard error: {e}");
    }

    // Update the last-text tray menu item (UTF-8 safe truncation).
    let display = truncate_for_menu(&text);
    let _ = last_text_item.set_text(display);

    // Store in state and transition to Done.
    {
        let mut st = state.lock().unwrap();
        st.set_last_transcription(text);
        st.set_status(AppStatus::Done);
    }
    update_tray_icon(&app, AppStatus::Done);

    // Auto-recover to Idle after 3 seconds.
    let recover_handle = app.clone();
    let recover_state = Arc::clone(&state);
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(3)).await;
        let mut st = recover_state.lock().unwrap();
        if st.status() == AppStatus::Done {
            st.set_status(AppStatus::Idle);
            drop(st);
            update_tray_icon(&recover_handle, AppStatus::Idle);
        }
    });
}
