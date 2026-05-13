use super::PostProcessor;
use crate::error::AppError;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::pin::pin;
use std::sync::OnceLock;

static BACKEND: OnceLock<LlamaBackend> = OnceLock::new();

fn backend() -> Result<&'static LlamaBackend, AppError> {
    if let Some(b) = BACKEND.get() {
        return Ok(b);
    }
    let b = LlamaBackend::init().map_err(|e| AppError::Llm(e.to_string()))?;
    let _ = BACKEND.set(b);
    BACKEND
        .get()
        .ok_or_else(|| AppError::Llm("backend init race".into()))
}

pub struct LlamaPostProcessor {
    model_path: PathBuf,
}

impl LlamaPostProcessor {
    pub fn new(model_path: PathBuf) -> Self {
        Self { model_path }
    }
}

const SYSTEM_PROMPT: &str = "Você corrige textos transcritos: adicione pontuação, acentuação e capitalização corretas. NÃO altere o conteúdo, NÃO traduza, NÃO resuma. Responda apenas com o texto corrigido.";
const MAX_NEW_TOKENS: i32 = 512;

fn run_inference(model_path: PathBuf, text: String) -> Result<String, AppError> {
    let backend = backend()?;
    let model_params = LlamaModelParams::default();
    let model_params = pin!(model_params);
    let model = LlamaModel::load_from_file(backend, &model_path, &model_params)
        .map_err(|e| AppError::Llm(format!("model load: {e}")))?;

    let prompt = format!("{SYSTEM_PROMPT}\n\nTexto: {text}\n\nTexto corrigido:");
    let ctx_params = LlamaContextParams::default().with_n_ctx(NonZeroU32::new(2048));
    let mut ctx = model
        .new_context(backend, ctx_params)
        .map_err(|e| AppError::Llm(format!("ctx: {e}")))?;

    let tokens = model
        .str_to_token(&prompt, AddBos::Always)
        .map_err(|e| AppError::Llm(format!("tokenize: {e}")))?;

    let mut batch = LlamaBatch::new(512, 1);
    let last = (tokens.len() - 1) as i32;
    for (i, t) in (0i32..).zip(tokens.iter().copied()) {
        batch
            .add(t, i, &[0], i == last)
            .map_err(|e| AppError::Llm(format!("batch add: {e}")))?;
    }
    ctx.decode(&mut batch)
        .map_err(|e| AppError::Llm(format!("decode: {e}")))?;

    let mut sampler =
        LlamaSampler::chain_simple([LlamaSampler::dist(1234), LlamaSampler::greedy()]);

    let mut n_cur = batch.n_tokens();
    let n_end = n_cur + MAX_NEW_TOKENS;
    let mut out = String::new();
    let mut decoder = encoding_rs::UTF_8.new_decoder();

    while n_cur < n_end {
        let token = sampler.sample(&ctx, batch.n_tokens() - 1);
        sampler.accept(token);
        if model.is_eog_token(token) {
            break;
        }
        let piece = model
            .token_to_piece(token, &mut decoder, true, None)
            .map_err(|e| AppError::Llm(format!("detok: {e}")))?;
        if let Some(idx) = piece.find('\n') {
            out.push_str(&piece[..idx]);
            break;
        }
        out.push_str(&piece);
        batch.clear();
        batch
            .add(token, n_cur, &[0], true)
            .map_err(|e| AppError::Llm(format!("batch add: {e}")))?;
        n_cur += 1;
        ctx.decode(&mut batch)
            .map_err(|e| AppError::Llm(format!("decode: {e}")))?;
    }

    Ok(out.trim().to_string())
}

#[async_trait::async_trait]
impl PostProcessor for LlamaPostProcessor {
    async fn refine(&self, text: &str) -> Result<String, AppError> {
        let mp = self.model_path.clone();
        let t = text.to_string();
        tokio::task::spawn_blocking(move || run_inference(mp, t))
            .await
            .map_err(|e| AppError::Internal(format!("join: {e}")))?
    }
}
