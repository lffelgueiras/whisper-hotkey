pub mod vocabulary;
pub mod whisper_cpp;

use crate::error::AppError;

pub trait Transcriber: Send + Sync {
    fn transcribe(&self, samples: &[f32], vocab: &[String]) -> Result<String, AppError>;
}
