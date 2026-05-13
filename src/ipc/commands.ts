import { invoke } from "@tauri-apps/api/core";
import type { Config } from "./generated/Config";

export const cmd = {
  getConfig: () => invoke<Config>("get_config"),
  updateConfig: (patch: Partial<Config>) => invoke<Config>("update_config", { patch }),
};
