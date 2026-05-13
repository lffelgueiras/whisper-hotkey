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
pub mod permissions;
pub mod replacements;
pub mod sound;
pub mod storage;
pub mod system_specs;

use app_state::{next, Intent, RecordingState};
use asr::{whisper_cpp::WhisperCpp, Transcriber};
use llm::{llama_cpp::LlamaPostProcessor, PostProcessor};
use audio::AudioCapturer;
use error::AppError;
use hotkey::HotkeyEdge;
use models::{builtin_catalog, download};
use paste::Paster;
use std::sync::Arc;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};
use tokio::sync::{mpsc, Mutex};

const DEFAULT_HOTKEY: &str = "Control+Space";
const DEFAULT_MODEL_ID: &str = "whisper-base";

struct App {
    state: Mutex<RecordingState>,
    audio: Arc<AudioCapturer>,
    asr: Mutex<Option<Arc<dyn Transcriber>>>,
    llm: Mutex<Option<Arc<dyn PostProcessor>>>,
    paster: Box<dyn Paster>,
    handle: AppHandle,
    tone: Option<sound::ToneOutput>,
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

    async fn ensure_llm_loaded(
        &self,
        model_id: &str,
    ) -> Result<Option<Arc<dyn PostProcessor>>, AppError> {
        let mut guard = self.llm.lock().await;
        if let Some(a) = guard.as_ref() {
            return Ok(Some(a.clone()));
        }
        let info = builtin_catalog()
            .into_iter()
            .find(|m| m.id == model_id)
            .ok_or_else(|| AppError::Model(format!("unknown llm model {model_id}")))?;
        let path = download(&info, |d, t| tracing::info!("llm download {d}/{t}")).await?;
        let pp: Arc<dyn PostProcessor> = Arc::new(LlamaPostProcessor::new(path));
        *guard = Some(pp.clone());
        Ok(Some(pp))
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

        let sound_enabled = {
            let cfg_state = self.handle.state::<commands::ConfigState>();
            let guard = cfg_state.0.lock();
            guard.sound_feedback
        };

        match (prev, new) {
            (RecordingState::Idle, RecordingState::Recording) => {
                if sound_enabled {
                    if let Some(t) = self.tone.as_ref() {
                        t.play_start();
                    }
                }
                if let Err(e) = self.audio.start() {
                    tracing::error!("audio start failed: {e}");
                    self.set_state(RecordingState::Idle).await;
                }
            }
            (RecordingState::Recording, RecordingState::Transcribing) => {
                if sound_enabled {
                    if let Some(t) = self.tone.as_ref() {
                        t.play_stop();
                    }
                }
                let me = self.clone();
                tokio::spawn(async move {
                    let result: Result<String, AppError> = async {
                        let mut samples = me.audio.stop()?;
                        let peak = samples.iter().fold(0.0f32, |a, b| a.max(b.abs()));
                        tracing::info!("captured {} samples, peak={:.4}", samples.len(), peak);
                        if samples.is_empty() {
                            return Ok(String::new());
                        }
                        if peak < 0.02 {
                            tracing::info!("audio too quiet, skipping");
                            return Ok(String::new());
                        }
                        if peak < 0.5 {
                            let gain = (0.7 / peak).min(20.0);
                            for s in samples.iter_mut() {
                                *s *= gain;
                            }
                            tracing::info!("applied gain {:.2}", gain);
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
                        tracing::info!("whisper raw text: {:?}", text);
                        let text = replacements::apply(&text, &cfg.replacements);
                        let text = if cfg.post_processing_enabled && !cfg.llm_model.is_empty() {
                            match me.ensure_llm_loaded(&cfg.llm_model).await {
                                Ok(Some(pp)) => {
                                    let to_ms = cfg.llm_timeout_ms;
                                    match tokio::time::timeout(
                                        std::time::Duration::from_millis(to_ms),
                                        pp.refine(&text),
                                    )
                                    .await
                                    {
                                        Ok(Ok(t)) if !t.trim().is_empty() => t,
                                        Ok(Ok(_)) => {
                                            tracing::warn!("llm returned empty, falling back to raw whisper text");
                                            text
                                        }
                                        Ok(Err(e)) => {
                                            tracing::warn!("llm error, falling back: {e}");
                                            text
                                        }
                                        Err(_) => {
                                            tracing::warn!("llm timeout, falling back");
                                            text
                                        }
                                    }
                                }
                                Ok(None) => text,
                                Err(e) => {
                                    tracing::warn!("llm load failed: {e}");
                                    text
                                }
                            }
                        } else {
                            text
                        };
                        Ok(text)
                    }
                    .await;

                    match result {
                        Ok(text) if !text.is_empty() => {
                            let (model, post_processed, auto_paste) = {
                                let cfg_state = me.handle.state::<commands::ConfigState>();
                                let cfg = cfg_state.0.lock();
                                (
                                    cfg.asr_model.clone(),
                                    cfg.post_processing_enabled,
                                    cfg.auto_paste,
                                )
                            };
                            let entry = storage::history::HistoryEntry {
                                ts: chrono::Utc::now().to_rfc3339(),
                                text: text.clone(),
                                model,
                                post_processed,
                            };
                            if let Err(e) = storage::history::append(&entry) {
                                tracing::error!("history append failed: {e}");
                            }
                            let paste_res = if auto_paste {
                                me.paster.paste(&text)
                            } else {
                                me.paster.copy(&text)
                            };
                            if let Err(e) = paste_res {
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
                            use tauri::Emitter;
                            let _ = me.handle.emit("error", e.to_dto());
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
    let _log_guard = logging::init();
    #[cfg(target_os = "macos")]
    {
        permissions::request_microphone_if_needed();
    }

    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    let (tx, mut rx) = mpsc::unbounded_channel::<HotkeyEdge>();

    let initial_cfg = storage::config::load();
    let mut hk = hotkey::HotkeyService::new().expect("hotkey service");
    let initial_hotkey = if initial_cfg.hotkey.is_empty() {
        DEFAULT_HOTKEY.into()
    } else {
        initial_cfg.hotkey.clone()
    };
    if let Err(e) = hk.register(&initial_hotkey) {
        tracing::error!("register initial hotkey '{}': {}", initial_hotkey, e);
        let _ = hk.register(DEFAULT_HOTKEY);
    }
    hotkey::HotkeyService::start_listener(tx);
    let hk_state = commands::HotkeyState(std::sync::Arc::new(parking_lot::Mutex::new(hk)));

    tauri::Builder::default()
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let label = window.label();
                if label == "main" || label == "history" {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(commands::ConfigState(std::sync::Arc::new(
            parking_lot::Mutex::new(initial_cfg),
        )))
        .manage(hk_state)
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::update_config,
            commands::pause_hotkey,
            commands::resume_hotkey,
            commands::get_audio_level,
            commands::list_models,
            commands::get_system_specs,
            commands::download_model,
            commands::delete_model,
            commands::is_model_present,
            commands::get_history,
            commands::delete_history_entry,
            commands::clear_history,
            commands::export_history,
            commands::check_permissions,
            commands::request_accessibility,
            commands::open_accessibility_panel,
            commands::set_autostart,
            commands::get_autostart,
        ])
        .setup(move |app| {
            let audio = Arc::new(AudioCapturer::new());
            app.manage(commands::AudioState(audio.clone()));
            let app_obj = Arc::new(App {
                state: Mutex::new(RecordingState::Idle),
                audio,
                asr: Mutex::new(None),
                llm: Mutex::new(None),
                paster: paste::default_paster(),
                handle: app.handle().clone(),
                tone: sound::ToneOutput::new(),
            });

            let settings = MenuItemBuilder::with_id("settings", "Settings…").build(app)?;
            let history = MenuItemBuilder::with_id("history", "History…").build(app)?;
            let logs = MenuItemBuilder::with_id("logs", "Open logs folder").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app)
                .items(&[&settings, &history, &logs, &quit])
                .build()?;
            let _tray = TrayIconBuilder::with_id("main")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.unminimize();
                            let _ = w.set_focus();
                        }
                    }
                })
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
                    "logs" => {
                        let dir = logging::logs_dir();
                        std::fs::create_dir_all(&dir).ok();
                        #[cfg(target_os = "macos")]
                        {
                            let _ = std::process::Command::new("open").arg(&dir).status();
                        }
                        #[cfg(target_os = "windows")]
                        {
                            let _ = std::process::Command::new("explorer").arg(&dir).status();
                        }
                    }
                    _ => {}
                })
                .build(app)?;

            let app_for_loop = app_obj.clone();
            rt.spawn(async move {
                while let Some(edge) = rx.recv().await {
                    let trigger = {
                        let cfg_state = app_for_loop.handle.state::<commands::ConfigState>();
                        let cfg = cfg_state.0.lock();
                        cfg.hotkey_trigger
                    };
                    use storage::config::HotkeyTrigger;
                    match (trigger, edge) {
                        (HotkeyTrigger::Toggle, HotkeyEdge::Press) => {
                            app_for_loop.clone().handle_toggle().await;
                        }
                        (HotkeyTrigger::Toggle, HotkeyEdge::Release) => {}
                        (HotkeyTrigger::PushToTalk, HotkeyEdge::Press) => {
                            let state = *app_for_loop.state.lock().await;
                            if matches!(state, RecordingState::Idle) {
                                app_for_loop.clone().handle_toggle().await;
                            }
                        }
                        (HotkeyTrigger::PushToTalk, HotkeyEdge::Release) => {
                            let state = *app_for_loop.state.lock().await;
                            if matches!(state, RecordingState::Recording) {
                                app_for_loop.clone().handle_toggle().await;
                            }
                        }
                    }
                }
            });

            events::position_overlay_top_center(&app.handle());
            events::configure_overlay_window(&app.handle());

            {
                let app_for_warmup = app_obj.clone();
                rt.spawn(async move {
                    let cfg = {
                        let cfg_state =
                            app_for_warmup.handle.state::<commands::ConfigState>();
                        let g = cfg_state.0.lock();
                        g.clone()
                    };
                    if let Err(e) = app_for_warmup.ensure_asr_loaded().await {
                        tracing::warn!("asr preload failed: {e}");
                    }
                    if cfg.post_processing_enabled && !cfg.llm_model.is_empty() {
                        match app_for_warmup.ensure_llm_loaded(&cfg.llm_model).await {
                            Ok(Some(pp)) => {
                                if let Err(e) = pp.warmup().await {
                                    tracing::warn!("llm warmup failed: {e}");
                                } else {
                                    tracing::info!("llm warmup complete");
                                }
                            }
                            Ok(None) => {}
                            Err(e) => tracing::warn!("llm preload failed: {e}"),
                        }
                    }
                });
            }

            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
                let _ = w.unminimize();
                let _ = w.center();
            }
            app.manage(rt);
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            match &event {
                tauri::RunEvent::ExitRequested { .. } | tauri::RunEvent::Exit => {
                    llm::llama_cpp::kill_all_llama_servers();
                }
                #[cfg(target_os = "macos")]
                tauri::RunEvent::Reopen { .. } => {
                    if let Some(w) = app.get_webview_window("main") {
                        let _ = w.show();
                        let _ = w.unminimize();
                        let _ = w.set_focus();
                    }
                }
                _ => {}
            }
            let _ = app;
        });
}
