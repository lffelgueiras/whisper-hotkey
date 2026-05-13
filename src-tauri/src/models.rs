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
    /// Approximate peak RAM required to run inference on this model, in GB.
    /// Used by the onboarding UI to flag models that exceed the host's
    /// installed memory.
    pub min_ram_gb: f32,
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
            sha256: "be07e048e1e599ad46341c8d2a135645097a538221678b7acdd1b1919c6e1b21"
                .into(),
            size_bytes: 77_691_713,
            display_name: "Whisper Tiny (78 MB) — fastest, baseline quality".into(),
            min_ram_gb: 1.0,
        },
        ModelInfo {
            id: "whisper-base".into(),
            kind: ModelKind::Asr,
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin".into(),
            sha256: "60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe"
                .into(),
            size_bytes: 147_951_465,
            display_name: "Whisper Base (148 MB) — fast, decent quality".into(),
            min_ram_gb: 1.0,
        },
        ModelInfo {
            id: "whisper-large-v3-turbo-q5_0".into(),
            kind: ModelKind::Asr,
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo-q5_0.bin"
                .into(),
            sha256: "394221709cd5ad1f40c46e6031ca61bce88931e6e088c188294c6d5a55ffa7e2"
                .into(),
            size_bytes: 574_041_195,
            display_name: "Whisper Large v3 Turbo q5_0 (574 MB) — recommended, great quality + fast"
                .into(),
            min_ram_gb: 2.0,
        },
        ModelInfo {
            id: "whisper-large-v3-turbo-q8_0".into(),
            kind: ModelKind::Asr,
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo-q8_0.bin"
                .into(),
            sha256: "317eb69c11673c9de1e1f0d459b253999804ec71ac4c23c17ecf5fbe24e259a1"
                .into(),
            size_bytes: 874_188_075,
            display_name: "Whisper Large v3 Turbo q8_0 (874 MB) — higher fidelity turbo".into(),
            min_ram_gb: 3.0,
        },
        ModelInfo {
            id: "whisper-large-v3-turbo".into(),
            kind: ModelKind::Asr,
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin"
                .into(),
            sha256: "1fc70f774d38eb169993ac391eea357ef47c88757ef72ee5943879b7e8e2bc69"
                .into(),
            size_bytes: 1_624_555_275,
            display_name: "Whisper Large v3 Turbo full (1.6 GB) — full-precision turbo".into(),
            min_ram_gb: 4.0,
        },
        ModelInfo {
            id: "whisper-large-v3-q5_0".into(),
            kind: ModelKind::Asr,
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-q5_0.bin"
                .into(),
            sha256: "d75795ecff3f83b5faa89d1900604ad8c780abd5739fae406de19f23ecd98ad1"
                .into(),
            size_bytes: 1_081_140_203,
            display_name: "Whisper Large v3 q5_0 (1.08 GB) — best quality, slower than turbo".into(),
            min_ram_gb: 3.0,
        },
        ModelInfo {
            id: "whisper-large-v3".into(),
            kind: ModelKind::Asr,
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin"
                .into(),
            sha256: "64d182b440b98d5203c4f9bd541544d84c605196c4f7b845dfa11fb23594d1e2"
                .into(),
            size_bytes: 3_095_033_483,
            display_name: "Whisper Large v3 full (3.1 GB) — máxima qualidade oficial v3".into(),
            min_ram_gb: 6.0,
        },
        ModelInfo {
            id: "whisper-large-v2-q8_0".into(),
            kind: ModelKind::Asr,
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v2-q8_0.bin"
                .into(),
            sha256: "fef54e6d898246a65c8285bfa83bd1807e27fadf54d5d4e81754c47634737e8c"
                .into(),
            size_bytes: 1_656_129_691,
            display_name: "Whisper Large v2 q8_0 (1.66 GB) — v2 quantizado, ótimo para PT-BR".into(),
            min_ram_gb: 4.0,
        },
        ModelInfo {
            id: "whisper-large-v2".into(),
            kind: ModelKind::Asr,
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v2.bin"
                .into(),
            sha256: "9a423fe4d40c82774b6af34115b8b935f34152246eb19e80e376071d3f999487"
                .into(),
            size_bytes: 3_094_623_691,
            display_name: "Whisper Large v2 full (3.1 GB) — máxima qualidade v2, alternativa ao v3".into(),
            min_ram_gb: 6.0,
        },
        ModelInfo {
            id: "gemma-4-e2b-it-q4_k_m".into(),
            kind: ModelKind::Llm,
            url: "https://huggingface.co/bartowski/google_gemma-4-E2B-it-GGUF/resolve/main/google_gemma-4-E2B-it-Q4_K_M.gguf".into(),
            sha256: "b5310340b3a23d31655d7119d100d5df1b2d8ee17b3ca8b0a23ad7e9eb5fa705"
                .into(),
            size_bytes: 3_462_678_272,
            display_name: "Gemma 4 E2B Instruct Q4_K_M (3.5 GB) — leve, rápido".into(),
            min_ram_gb: 6.0,
        },
        ModelInfo {
            id: "gemma-4-e4b-it-q4_k_m".into(),
            kind: ModelKind::Llm,
            url: "https://huggingface.co/bartowski/google_gemma-4-E4B-it-GGUF/resolve/main/google_gemma-4-E4B-it-Q4_K_M.gguf".into(),
            sha256: "51865750adafd22de56994a343d5a887cc1a589b9bae41d62b748c8bd0ca9c76"
                .into(),
            size_bytes: 5_405_168_384,
            display_name: "Gemma 4 E4B Instruct Q4_K_M (5.4 GB) — qualidade maior".into(),
            min_ram_gb: 9.0,
        },
        ModelInfo {
            id: "qwen3-4b-q5_k_m".into(),
            kind: ModelKind::Llm,
            url: "https://huggingface.co/Qwen/Qwen3-4B-GGUF/resolve/main/Qwen3-4B-Q5_K_M.gguf".into(),
            sha256: "aca596860e8cb40af6539e3f2ea40df305f42515deac56d49c08d39a02e6533f"
                .into(),
            size_bytes: 2_889_513_184,
            display_name: "Qwen3 4B Q5_K_M (2.9 GB) — leve, multilingual forte".into(),
            min_ram_gb: 6.0,
        },
        ModelInfo {
            id: "qwen3-8b-q4_k_m".into(),
            kind: ModelKind::Llm,
            url: "https://huggingface.co/Qwen/Qwen3-8B-GGUF/resolve/main/Qwen3-8B-Q4_K_M.gguf".into(),
            sha256: "d98cdcbd03e17ce47681435b5150e34c1417f50b5c0019dd560e4882c5745785"
                .into(),
            size_bytes: 5_027_783_488,
            display_name: "Qwen3 8B Q4_K_M (5.0 GB) — qualidade alta, raciocínio melhor".into(),
            min_ram_gb: 9.0,
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
