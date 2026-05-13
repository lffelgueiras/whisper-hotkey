use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export, export_to = "../src/ipc/generated/")]
pub enum ErrorKind {
    Mic,
    Model,
    Paste,
    Asr,
    Llm,
    Storage,
    Hotkey,
    Permission,
    Internal,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("microphone error: {0}")]
    Mic(String),
    #[error("model error: {0}")]
    Model(String),
    #[error("paste error: {0}")]
    Paste(String),
    #[error("asr error: {0}")]
    Asr(String),
    #[error("llm error: {0}")]
    Llm(String),
    #[error("storage error: {0}")]
    Storage(#[from] std::io::Error),
    #[error("hotkey error: {0}")]
    Hotkey(String),
    #[error("permission denied: {0}")]
    Permission(String),
    #[error("internal: {0}")]
    Internal(String),
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export, export_to = "../src/ipc/generated/")]
pub struct AppErrorDto {
    pub kind: ErrorKind,
    pub message: String,
    pub recoverable: bool,
}

impl AppError {
    pub fn to_dto(&self) -> AppErrorDto {
        let (kind, recoverable) = match self {
            AppError::Mic(_) => (ErrorKind::Mic, true),
            AppError::Model(_) => (ErrorKind::Model, true),
            AppError::Paste(_) => (ErrorKind::Paste, true),
            AppError::Asr(_) => (ErrorKind::Asr, true),
            AppError::Llm(_) => (ErrorKind::Llm, true),
            AppError::Storage(_) => (ErrorKind::Storage, false),
            AppError::Hotkey(_) => (ErrorKind::Hotkey, false),
            AppError::Permission(_) => (ErrorKind::Permission, true),
            AppError::Internal(_) => (ErrorKind::Internal, false),
        };
        AppErrorDto {
            kind,
            message: self.to_string(),
            recoverable,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mic_error_is_recoverable() {
        let err = AppError::Mic("no device".into());
        let dto = err.to_dto();
        assert!(matches!(dto.kind, ErrorKind::Mic));
        assert!(dto.recoverable);
    }

    #[test]
    fn storage_error_is_not_recoverable() {
        let io = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "x");
        let err: AppError = io.into();
        assert!(!err.to_dto().recoverable);
    }
}
