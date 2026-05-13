export type OverlayPosition = "top_center" | "top_left" | "top_right" | "bottom_center";
export type Theme = "system" | "light" | "dark";
export interface ReplacementRule {
  from: string;
  to: string;
  regex: boolean;
}
export type HotkeyTrigger = "toggle" | "push_to_talk";
export interface Config {
  hotkey: string;
  hotkey_trigger: HotkeyTrigger;
  auto_paste: boolean;
  overlay_position: OverlayPosition;
  theme: Theme;
  asr_model: string;
  post_processing_enabled: boolean;
  llm_model: string;
  llm_timeout_ms: number;
  vocabulary: string[];
  replacements: ReplacementRule[];
  onboarding_complete: boolean;
  start_at_login: boolean;
  sound_feedback: boolean;
}
