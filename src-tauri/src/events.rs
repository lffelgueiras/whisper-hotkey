use crate::app_state::RecordingState;
use tauri::{AppHandle, Emitter, Manager};

pub fn emit_state(app: &AppHandle, state: RecordingState) {
    let payload = match state {
        RecordingState::Idle => "idle",
        RecordingState::Recording => "recording",
        RecordingState::Transcribing => "transcribing",
    };
    if let Err(e) = app.emit("state-changed", payload) {
        tracing::error!("emit state-changed: {e}");
    }
    if matches!(state, RecordingState::Recording) {
        position_overlay_top_center(app);
    }
}

fn position_overlay_top_center(app: &AppHandle) {
    if let Some(overlay) = app.get_webview_window("overlay") {
        if let Ok(Some(m)) = overlay.primary_monitor() {
            let size = m.size();
            let scale = m.scale_factor();
            let x = (size.width as f64 / scale - 140.0) / 2.0;
            let y = 20.0;
            let _ = overlay.set_position(tauri::PhysicalPosition::new(
                (x * scale) as i32,
                (y * scale) as i32,
            ));
        }
    }
}
