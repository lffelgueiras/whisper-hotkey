use super::PostProcessor;
use crate::error::AppError;
use std::path::PathBuf;

pub struct LlamaPostProcessor {
    model_path: PathBuf,
}

impl LlamaPostProcessor {
    pub fn new(model_path: PathBuf) -> Self {
        Self { model_path }
    }
}

const SYSTEM_PROMPT: &str = "Você corrige textos transcritos: adicione pontuação, acentuação e capitalização corretas. NÃO altere o conteúdo, NÃO traduza, NÃO resuma. Responda apenas com o texto corrigido.";

#[async_trait::async_trait]
impl PostProcessor for LlamaPostProcessor {
    async fn refine(&self, text: &str) -> Result<String, AppError> {
        let mp = self.model_path.clone();
        let _prompt = format!("{SYSTEM_PROMPT}\n\nTexto: {text}\n\nTexto corrigido:");
        let result = tokio::task::spawn_blocking(move || -> Result<String, AppError> {
            Err(AppError::Llm(format!(
                "llama-cpp-2 integration pending; model at {:?}",
                mp
            )))
        })
        .await
        .map_err(|e| AppError::Internal(format!("join: {e}")))??;
        Ok(result)
    }
}
