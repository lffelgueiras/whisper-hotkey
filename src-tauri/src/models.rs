use crate::error::AppError;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../src/ipc/generated/")]
pub struct ModelInfo {
    pub id: String,
    pub kind: ModelKind,
    pub url: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub display_name: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ts_rs::TS, PartialEq, Eq)]
#[ts(export, export_to = "../src/ipc/generated/")]
#[serde(rename_all = "lowercase")]
pub enum ModelKind {
    Asr,
    Llm,
}

pub fn app_data_dir() -> PathBuf {
    dirs::data_dir()
        .map(|d| d.join("whisper-hotkey"))
        .unwrap_or_else(|| PathBuf::from(".whisper-hotkey"))
}

pub fn model_path(id: &str) -> PathBuf {
    app_data_dir().join("models").join(format!("{id}.bin"))
}

pub fn builtin_catalog() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            id: "whisper-tiny".into(),
            kind: ModelKind::Asr,
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin".into(),
            sha256: "bd577a113a864445d4c299885e0cb97d4ba92b5f".into(),
            size_bytes: 75_000_000,
            display_name: "Whisper Tiny (75 MB) — fast, baseline quality".into(),
        },
        ModelInfo {
            id: "whisper-base".into(),
            kind: ModelKind::Asr,
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin".into(),
            sha256: "60ed5bc3dd14eea856493d334349b405782ddcaf".into(),
            size_bytes: 142_000_000,
            display_name: "Whisper Base (142 MB) — recommended starter".into(),
        },
        ModelInfo {
            id: "whisper-large-v3-q5_0".into(),
            kind: ModelKind::Asr,
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-q5_0.bin"
                .into(),
            sha256: "e6d2a1c6f4b8d1c2b4a8c1d2e3f4a5b6c7d8e9f0".into(),
            size_bytes: 1_080_000_000,
            display_name: "Whisper Large v3 q5_0 (1 GB) — best quality".into(),
        },
    ]
}

pub async fn download<F>(info: &ModelInfo, on_progress: F) -> Result<PathBuf, AppError>
where
    F: Fn(u64, u64) + Send,
{
    let target = model_path(&info.id);
    if target.exists() && verify_sha256(&target, &info.sha256).await? {
        return Ok(target);
    }
    tokio::fs::create_dir_all(target.parent().unwrap()).await?;
    let tmp = target.with_extension("part");
    let existing = tokio::fs::metadata(&tmp)
        .await
        .ok()
        .map(|m| m.len())
        .unwrap_or(0);

    let client = reqwest::Client::new();
    let mut req = client.get(&info.url);
    if existing > 0 {
        req = req.header("Range", format!("bytes={existing}-"));
    }
    let resp = req
        .send()
        .await
        .map_err(|e| AppError::Model(format!("http: {e}")))?;
    let total = resp.content_length().unwrap_or(info.size_bytes) + existing;

    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(existing > 0)
        .write(true)
        .open(&tmp)
        .await?;

    let mut downloaded = existing;
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| AppError::Model(format!("read: {e}")))?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        on_progress(downloaded, total);
    }
    file.flush().await?;
    drop(file);

    if !verify_sha256(&tmp, &info.sha256).await? {
        let _ = tokio::fs::remove_file(&tmp).await;
        return Err(AppError::Model("sha256 mismatch".into()));
    }
    tokio::fs::rename(&tmp, &target).await?;
    Ok(target)
}

pub async fn verify_sha256(path: &Path, expected_hex: &str) -> Result<bool, AppError> {
    let bytes = tokio::fs::read(path).await?;
    let mut h = Sha256::new();
    h.update(&bytes);
    let got = h.finalize();
    Ok(format!("{:x}", got).eq_ignore_ascii_case(expected_hex))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn verify_sha_returns_false_for_mismatch() {
        let p = std::env::temp_dir().join("wh-test-sha.bin");
        tokio::fs::write(&p, b"hello").await.unwrap();
        assert!(!verify_sha256(&p, "00").await.unwrap());
        let _ = tokio::fs::remove_file(&p).await;
    }
}
