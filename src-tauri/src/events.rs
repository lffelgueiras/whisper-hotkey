use crate::app_state::RecordingState;
use tauri::{AppHandle, Emitter, Manager};

#[cfg(target_os = "macos")]
fn promote_overlay_window(app: &AppHandle) {
    use objc::{msg_send, sel, sel_impl};
    let Some(overlay) = app.get_webview_window("overlay") else {
        tracing::warn!("promote_overlay_window: overlay window not found");
        return;
    };
    match overlay.ns_window() {
        Ok(ns_window) => unsafe {
            let ns_window = ns_window as *mut objc::runtime::Object;
            // CanJoinAllSpaces (1<<0) | Stationary (1<<4)
            //   | IgnoresCycle (1<<6) | FullScreenAuxiliary (1<<8)
            let behavior: u64 = (1 << 0) | (1 << 4) | (1 << 6) | (1 << 8);
            let _: () = msg_send![ns_window, setCollectionBehavior: behavior];
            // NSScreenSaverWindowLevel = 1000 — sits above status bar
            // and fullscreen apps.
            let _: () = msg_send![ns_window, setLevel: 1000_i64];
            let _: () = msg_send![ns_window, setHidesOnDeactivate: false];
            let _: () = msg_send![ns_window, setCanHide: false];

            // Read back to confirm flags stuck.
            let cb: u64 = msg_send![ns_window, collectionBehavior];
            let lvl: i64 = msg_send![ns_window, level];
            tracing::info!(
                "overlay NSWindow promoted: collectionBehavior=0x{cb:x} ({cb}) level={lvl}"
            );
        },
        Err(e) => tracing::warn!("ns_window() failed: {e}"),
    }
}

/// One-shot overlay window configuration. Must be called from the main thread
/// (e.g. from Tauri's `setup` closure).
pub fn configure_overlay_window(app: &AppHandle) {
    #[cfg(target_os = "macos")]
    promote_overlay_window(app);
}

pub fn emit_state(app: &AppHandle, state: RecordingState) {
    let payload = match state {
        RecordingState::Idle => "idle",
        RecordingState::Recording => "recording",
        RecordingState::Transcribing => "transcribing",
    };
    if let Err(e) = app.emit("state-changed", payload) {
        tracing::error!("emit state-changed: {e}");
    }
    let app_cloned = app.clone();
    let _ = app.run_on_main_thread(move || {
        if let Some(overlay) = app_cloned.get_webview_window("overlay") {
            match state {
                RecordingState::Idle => {
                    let _ = overlay.hide();
                }
                RecordingState::Recording | RecordingState::Transcribing => {
                    position_overlay_top_center(&app_cloned);
                    let _ = overlay.show();
                    // Re-apply NSWindow flags after show(): showing the
                    // window can reset level / collectionBehavior.
                    #[cfg(target_os = "macos")]
                    promote_overlay_window(&app_cloned);
                }
            }
        }
    });
}

/// Place the overlay near the top-center of whichever monitor currently
/// contains the mouse cursor. Falls back to the primary monitor.
pub fn position_overlay_top_center(app: &AppHandle) {
    let Some(overlay) = app.get_webview_window("overlay") else {
        return;
    };

    let cursor = app.cursor_position().ok();
    let monitor = cursor
        .as_ref()
        .and_then(|p| app.monitor_from_point(p.x, p.y).ok().flatten())
        .or_else(|| overlay.primary_monitor().ok().flatten());

    tracing::info!(
        "position_overlay: cursor={cursor:?} monitor={:?}",
        monitor.as_ref().map(|m| (m.position(), m.size()))
    );

    let Some(m) = monitor else { return };

    let size = m.size();
    let pos = m.position();
    let scale = m.scale_factor();
    let width_pts = size.width as f64 / scale;
    let x_pts = pos.x as f64 / scale + (width_pts - 220.0) / 2.0;
    let y_pts = pos.y as f64 / scale + 20.0;
    let _ = overlay.set_position(tauri::PhysicalPosition::new(
        (x_pts * scale) as i32,
        (y_pts * scale) as i32,
    ));
}
