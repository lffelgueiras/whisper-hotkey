use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[ts(export, export_to = "../src/ipc/generated/")]
#[serde(rename_all = "lowercase")]
pub enum RecordingState {
    Idle,
    Recording,
    Transcribing,
}

#[derive(Debug, Clone, Copy)]
pub enum Intent {
    Toggle,
    Done,
    Failed,
}

pub fn next(state: RecordingState, intent: Intent) -> RecordingState {
    use Intent::*;
    use RecordingState::*;
    match (state, intent) {
        (Idle, Toggle) => Recording,
        (Recording, Toggle) => Transcribing,
        (Transcribing, Toggle) => Transcribing,
        (Transcribing, Done) => Idle,
        (Transcribing, Failed) => Idle,
        (other, _) => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_to_recording_on_toggle() {
        assert_eq!(
            next(RecordingState::Idle, Intent::Toggle),
            RecordingState::Recording
        );
    }

    #[test]
    fn recording_to_transcribing_on_toggle() {
        assert_eq!(
            next(RecordingState::Recording, Intent::Toggle),
            RecordingState::Transcribing
        );
    }

    #[test]
    fn toggle_during_transcription_is_ignored() {
        assert_eq!(
            next(RecordingState::Transcribing, Intent::Toggle),
            RecordingState::Transcribing
        );
    }

    #[test]
    fn done_returns_to_idle() {
        assert_eq!(
            next(RecordingState::Transcribing, Intent::Done),
            RecordingState::Idle
        );
        assert_eq!(
            next(RecordingState::Transcribing, Intent::Failed),
            RecordingState::Idle
        );
    }
}
