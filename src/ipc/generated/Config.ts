export type OverlayPosition = "top_center" | "top_left" | "top_right" | "bottom_center";
export type Theme = "system" | "light" | "dark";
export interface ReplacementRule {
  from: string;
  to: string;
  regex: boolean;
}
export interface Config {
  hotkey: string;
  auto_paste: boolean;
  overlay_position: OverlayPosition;
  theme: Theme;
  asr_model: string;
  post_processing_enabled: boolean;
  llm_model: string;
  llm_timeout_ms: number;
  vocabulary: string[];
  replacements: ReplacementRule[];
}
