use crate::error::{AppError, AppErrorDto};
use crate::models::{builtin_catalog, download, ModelInfo};
use crate::storage::config::{self, Config};
use parking_lot::Mutex as PMutex;
use serde::Serialize;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

pub struct ConfigState(pub Arc<PMutex<Config>>);

#[tauri::command]
pub fn get_config(state: State<'_, ConfigState>) -> Config {
    state.0.lock().clone()
}

#[tauri::command]
pub fn update_config(
    patch: serde_json::Value,
    state: State<'_, ConfigState>,
) -> Result<Config, AppErrorDto> {
    let mut cfg = state.0.lock();
    let mut v = serde_json::to_value(&*cfg).map_err(|e| AppError::Internal(e.to_string()).to_dto())?;
    if let (Some(obj), Some(p)) = (v.as_object_mut(), patch.as_object()) {
        for (k, val) in p {
            obj.insert(k.clone(), val.clone());
        }
    }
    let new: Config =
        serde_json::from_value(v).map_err(|e| AppError::Internal(e.to_string()).to_dto())?;
    config::save(&new).map_err(|e| e.to_dto())?;
    *cfg = new.clone();
    Ok(new)
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
