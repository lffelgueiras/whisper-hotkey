pub mod llama_cpp;

use crate::error::AppError;

#[async_trait::async_trait]
pub trait PostProcessor: Send + Sync {
    async fn refine(&self, text: &str) -> Result<String, AppError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EchoPP;
    #[async_trait::async_trait]
    impl PostProcessor for EchoPP {
        async fn refine(&self, t: &str) -> Result<String, AppError> {
            Ok(t.to_string())
        }
    }

    #[tokio::test]
    async fn echo_works() {
        let p = EchoPP;
        assert_eq!(p.refine("hi").await.unwrap(), "hi");
    }
}
