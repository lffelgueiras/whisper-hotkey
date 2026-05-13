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
        params.set_language(Some("pt"));
        params.set_no_speech_thold(0.6);
        params.set_suppress_blank(true);
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
        Ok(clean_hallucinations(out.trim()))
    }
}

/// Strip common Whisper non-speech hallucinations. Whisper frequently inserts
/// tokens like `[Música]`, `[Aplausos]`, `*Music*`, `♪` when the audio is
/// silent or noisy. We never want those in the user's pasted text.
fn clean_hallucinations(input: &str) -> String {
    // Inner labels (lowercased) we want to drop when wrapped in [], (), or **.
    const NOISE: &[&str] = &[
        "música",
        "musica",
        "music",
        "aplausos",
        "applause",
        "risos",
        "laughter",
        "blank_audio",
        "inaudible",
        "silêncio",
        "silencio",
        "silence",
        "barulho",
        "ruído",
        "noise",
        "som",
        "sons",
        "sound",
        "sounds",
        "tosse",
        "cough",
        "suspiro",
        "sigh",
        "respiração",
    ];

    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        let close = match c {
            b'[' => Some(b']'),
            b'(' => Some(b')'),
            b'*' => Some(b'*'),
            _ => None,
        };
        if let Some(close_byte) = close {
            // Find matching close.
            if let Some(rel) = bytes[i + 1..].iter().position(|b| *b == close_byte) {
                let inner = &input[i + 1..i + 1 + rel];
                let normalized = inner.trim().to_lowercase();
                if NOISE.iter().any(|t| normalized == *t)
                    || NOISE.iter().any(|t| normalized.starts_with(t))
                {
                    i += 1 + rel + 1;
                    continue;
                }
            }
        }
        // Skip music-note glyphs.
        let ch = input[i..].chars().next().unwrap_or('\0');
        if ch == '♪' || ch == '♫' {
            i += ch.len_utf8();
            continue;
        }
        out.push(ch);
        i += ch.len_utf8();
    }
    // Collapse double spaces left behind by removed brackets.
    let collapsed = out.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed
}

#[cfg(test)]
mod tests {
    use super::clean_hallucinations;

    #[test]
    fn strips_music_brackets() {
        assert_eq!(clean_hallucinations("[Música]"), "");
        assert_eq!(clean_hallucinations("oi [Música] tudo bem"), "oi tudo bem");
        assert_eq!(clean_hallucinations("(música)"), "");
        assert_eq!(clean_hallucinations("*Music*"), "");
    }

    #[test]
    fn strips_applause_and_others() {
        assert_eq!(clean_hallucinations("[Aplausos]"), "");
        assert_eq!(clean_hallucinations("[BLANK_AUDIO]"), "");
        assert_eq!(clean_hallucinations("[risos] obrigado"), "obrigado");
    }

    #[test]
    fn keeps_legitimate_brackets() {
        assert_eq!(
            clean_hallucinations("ver [link 1] depois"),
            "ver [link 1] depois"
        );
    }

    #[test]
    fn strips_music_notes() {
        assert_eq!(clean_hallucinations("♪ ♫"), "");
    }
}
