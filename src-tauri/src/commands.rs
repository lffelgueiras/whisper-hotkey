use crate::error::{AppError, AppErrorDto};
use crate::models::{builtin_catalog, download, ModelInfo};
use crate::storage::config::{self, Config};
use parking_lot::Mutex as PMutex;
use serde::Serialize;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

pub struct ConfigState(pub Arc<PMutex<Config>>);
pub struct HotkeyState(pub Arc<PMutex<crate::hotkey::HotkeyService>>);

#[tauri::command]
pub fn get_config(state: State<'_, ConfigState>) -> Config {
    state.0.lock().clone()
}

#[tauri::command]
pub fn update_config(
    patch: serde_json::Value,
    state: State<'_, ConfigState>,
    hk: State<'_, HotkeyState>,
) -> Result<Config, AppErrorDto> {
    let mut cfg = state.0.lock();
    let prev_hotkey = cfg.hotkey.clone();
    let mut v = serde_json::to_value(&*cfg).map_err(|e| AppError::Internal(e.to_string()).to_dto())?;
    if let (Some(obj), Some(p)) = (v.as_object_mut(), patch.as_object()) {
        for (k, val) in p {
            obj.insert(k.clone(), val.clone());
        }
    }
    let new: Config =
        serde_json::from_value(v).map_err(|e| AppError::Internal(e.to_string()).to_dto())?;
    config::save(&new).map_err(|e| e.to_dto())?;
    if new.hotkey != prev_hotkey {
        hk.0.lock().register(&new.hotkey).map_err(|e| e.to_dto())?;
    }
    *cfg = new.clone();
    Ok(new)
}

#[tauri::command]
pub fn delete_model(id: String) -> Result<(), AppErrorDto> {
    let p = crate::models::model_path(&id);
    if p.exists() {
        std::fs::remove_file(p).map_err(|e| AppError::Storage(e).to_dto())?;
    }
    Ok(())
}

#[tauri::command]
pub fn is_model_present(id: String) -> bool {
    crate::models::model_path(&id).exists()
}

#[tauri::command]
pub fn list_models() -> Vec<ModelInfo> {
    builtin_catalog()
}

#[derive(Serialize, Clone)]
struct DownloadProgress {
    id: String,
    downloaded: u64,
    total: u64,
}

#[tauri::command]
pub async fn download_model(id: String, app: AppHandle) -> Result<(), AppErrorDto> {
    let info = builtin_catalog()
        .into_iter()
        .find(|m| m.id == id)
        .ok_or_else(|| AppError::Model(format!("unknown model {id}")).to_dto())?;
    let id_for_cb = id.clone();
    download(&info, move |d, t| {
        let _ = app.emit(
            "model-progress",
            DownloadProgress {
                id: id_for_cb.clone(),
                downloaded: d,
                total: t,
            },
        );
    })
    .await
    .map(|_| ())
    .map_err(|e| e.to_dto())
}
