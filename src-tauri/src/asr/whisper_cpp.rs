use super::{vocabulary, Transcriber};
use crate::error::AppError;
use std::path::Path;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub struct WhisperCpp {
    ctx: WhisperContext,
}

impl WhisperCpp {
    pub fn load(model_path: &Path) -> Result<Self, AppError> {
        let path = model_path
            .to_str()
            .ok_or_else(|| AppError::Model("non-utf8 model path".into()))?;
        let ctx = WhisperContext::new_with_params(path, WhisperContextParameters::default())
            .map_err(|e| AppError::Model(format!("load whisper model: {e}")))?;
        Ok(Self { ctx })
    }
}

impl Transcriber for WhisperCpp {
    fn transcribe(&self, samples: &[f32], vocab: &[String]) -> Result<String, AppError> {
        let mut state = self
            .ctx
            .create_state()
            .map_err(|e| AppError::Asr(format!("create state: {e}")))?;
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_print_progress(false);
        params.set_print_special(false);
        params.set_print_realtime(false);
        params.set_translate(false);
        params.set_language(Some("auto"));
        let prompt = vocabulary::build_initial_prompt(vocab);
        if let Some(p) = prompt.as_deref() {
            params.set_initial_prompt(p);
        }
        state
            .full(params, samples)
            .map_err(|e| AppError::Asr(format!("run: {e}")))?;
        let n = state
            .full_n_segments()
            .map_err(|e| AppError::Asr(format!("n_segments: {e}")))?;
        let mut out = String::new();
        for i in 0..n {
            let seg = state
                .full_get_segment_text(i)
                .map_err(|e| AppError::Asr(format!("seg {i}: {e}")))?;
            out.push_str(&seg);
        }
        Ok(out.trim().to_string())
    }
}
