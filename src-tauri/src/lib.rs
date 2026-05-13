pub mod app_state;
pub mod asr;
pub mod audio;
pub mod commands;
pub mod error;
pub mod events;
pub mod hotkey;
pub mod logging;
pub mod models;
pub mod llm;
pub mod paste;
pub mod replacements;
pub mod storage;

use app_state::{next, Intent, RecordingState};
use asr::{whisper_cpp::WhisperCpp, Transcriber};
use audio::AudioCapturer;
use error::AppError;
use models::{builtin_catalog, download};
use paste::Paster;
use std::sync::Arc;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager};
use tokio::sync::{mpsc, Mutex};

const DEFAULT_HOTKEY: &str = "CmdOrControl+Shift+Space";
const DEFAULT_MODEL_ID: &str = "whisper-base";

struct App {
    state: Mutex<RecordingState>,
    audio: AudioCapturer,
    asr: Mutex<Option<Arc<dyn Transcriber>>>,
    paster: Box<dyn Paster>,
    handle: AppHandle,
}

impl App {
    async fn set_state(&self, new: RecordingState) {
        *self.state.lock().await = new;
        events::emit_state(&self.handle, new);
    }
}

impl App {
    async fn ensure_asr_loaded(&self) -> Result<Arc<dyn Transcriber>, AppError> {
        let mut guard = self.asr.lock().await;
        if let Some(a) = guard.as_ref() {
            return Ok(a.clone());
        }
        let info = builtin_catalog()
            .into_iter()
            .find(|m| m.id == DEFAULT_MODEL_ID)
            .ok_or_else(|| AppError::Model("default model missing from catalog".into()))?;
        let path = download(&info, |d, t| {
            tracing::info!("download {d}/{t}");
        })
        .await?;
        let w = WhisperCpp::load(&path)?;
        let a: Arc<dyn Transcriber> = Arc::new(w);
        *guard = Some(a.clone());
        Ok(a)
    }

    async fn handle_toggle(self: Arc<Self>) {
        let prev = {
            let mut s = self.state.lock().await;
            let new = next(*s, Intent::Toggle);
            let prev = *s;
            *s = new;
            tracing::info!("state: {:?} -> {:?}", prev, new);
            prev
        };
        let new = next(prev, Intent::Toggle);
        events::emit_state(&self.handle, new);

        match (prev, new) {
            (RecordingState::Idle, RecordingState::Recording) => {
                if let Err(e) = self.audio.start() {
                    tracing::error!("audio start failed: {e}");
                    self.set_state(RecordingState::Idle).await;
                }
            }
            (RecordingState::Recording, RecordingState::Transcribing) => {
                let me = self.clone();
                tokio::spawn(async move {
                    let result: Result<String, AppError> = async {
                        let samples = me.audio.stop()?;
                        if samples.is_empty() {
                            return Ok(String::new());
                        }
                        let cfg = {
                            let cfg_state = me.handle.state::<commands::ConfigState>();
                            let guard = cfg_state.0.lock();
                            guard.clone()
                        };
                        let asr = me.ensure_asr_loaded().await?;
                        let samples_clone = samples.clone();
                        let asr_clone = asr.clone();
                        let vocab = cfg.vocabulary.clone();
                        let text = tokio::task::spawn_blocking(move || {
                            asr_clone.transcribe(&samples_clone, &vocab)
                        })
                        .await
                        .map_err(|e| AppError::Internal(format!("join: {e}")))??;
                        Ok(replacements::apply(&text, &cfg.replacements))
                    }
                    .await;

                    match result {
                        Ok(text) if !text.is_empty() => {
                            let model = {
                                let cfg_state = me.handle.state::<commands::ConfigState>();
                                let cfg = cfg_state.0.lock();
                                cfg.asr_model.clone()
                            };
                            let entry = storage::history::HistoryEntry {
                                ts: chrono::Utc::now().to_rfc3339(),
                                text: text.clone(),
                                model,
                                post_processed: false,
                            };
                            if let Err(e) = storage::history::append(&entry) {
                                tracing::error!("history append failed: {e}");
                            }
                            if let Err(e) = me.paster.paste(&text) {
                                tracing::error!("paste failed: {e}");
                            }
                            me.set_state(next(RecordingState::Transcribing, Intent::Done))
                                .await;
                        }
                        Ok(_) => {
                            tracing::info!("empty transcription");
                            me.set_state(next(RecordingState::Transcribing, Intent::Done))
                                .await;
                        }
                        Err(e) => {
                            tracing::error!("pipeline failed: {e}");
                            me.set_state(next(RecordingState::Transcribing, Intent::Failed))
                                .await;
                        }
                    }
                });
            }
            _ => {}
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    logging::init();

    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    let (tx, mut rx) = mpsc::unbounded_channel::<()>();

    let mut hk = hotkey::HotkeyService::new().expect("hotkey service");
    hk.register(DEFAULT_HOTKEY)
        .expect("register default hotkey");
    hotkey::HotkeyService::start_listener(tx);
    let hk_state = commands::HotkeyState(std::sync::Arc::new(parking_lot::Mutex::new(hk)));

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(commands::ConfigState(std::sync::Arc::new(
            parking_lot::Mutex::new(storage::config::load()),
        )))
        .manage(hk_state)
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::update_config,
            commands::list_models,
            commands::download_model,
            commands::delete_model,
            commands::is_model_present,
            commands::get_history,
            commands::delete_history_entry,
            commands::clear_history,
            commands::export_history,
        ])
        .setup(move |app| {
            let app_obj = Arc::new(App {
                state: Mutex::new(RecordingState::Idle),
                audio: AudioCapturer::new(),
                asr: Mutex::new(None),
                paster: paste::default_paster(),
                handle: app.handle().clone(),
            });

            let settings = MenuItemBuilder::with_id("settings", "Settings…").build(app)?;
            let history = MenuItemBuilder::with_id("history", "History…").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app)
                .items(&[&settings, &history, &quit])
                .build()?;
            let _tray = TrayIconBuilder::with_id("main")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "quit" => app.exit(0),
                    "settings" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "history" => {
                        if let Some(w) = app.get_webview_window("history") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    _ => {}
                })
                .build(app)?;

            let app_for_loop = app_obj.clone();
            rt.spawn(async move {
                while rx.recv().await.is_some() {
                    app_for_loop.clone().handle_toggle().await;
                }
            });

            app.manage(rt);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
