pub fn build_initial_prompt(vocab: &[String]) -> Option<String> {
    let cleaned: Vec<&str> = vocab
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    if cleaned.is_empty() {
        return None;
    }
    Some(format!("Glossário: {}.", cleaned.join(", ")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_vocab_returns_none() {
        assert!(build_initial_prompt(&[]).is_none());
    }

    #[test]
    fn whitespace_only_returns_none() {
        let v = vec!["  ".into(), "".into()];
        assert!(build_initial_prompt(&v).is_none());
    }

    #[test]
    fn formats_words() {
        let v = vec!["LUCABE".into(), "Ploomes".into()];
        assert_eq!(
            build_initial_prompt(&v),
            Some("Glossário: LUCABE, Ploomes.".to_string())
        );
    }
}
