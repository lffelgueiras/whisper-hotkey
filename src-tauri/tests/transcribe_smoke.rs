//! Run with: cargo test --test transcribe_smoke -- --ignored
//! Requires the whisper-base model already downloaded in the app data dir.

use whisper_hotkey_lib::asr::{whisper_cpp::WhisperCpp, Transcriber};
use whisper_hotkey_lib::models::model_path;

#[test]
#[ignore]
fn transcribes_jfk_sample() {
    let path = model_path("whisper-base");
    assert!(path.exists(), "model not downloaded at {:?}", path);

    let mut reader = hound::WavReader::open("tests/fixtures/jfk.wav").unwrap();
    let samples: Vec<f32> = reader
        .samples::<i16>()
        .map(|s| s.unwrap() as f32 / i16::MAX as f32)
        .collect();

    let w = WhisperCpp::load(&path).unwrap();
    let text = w.transcribe(&samples, &[]).unwrap().to_lowercase();
    assert!(
        text.contains("ask not what your country"),
        "unexpected transcription: {text}"
    );
}
