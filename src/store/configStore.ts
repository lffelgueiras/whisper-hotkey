import { create } from "zustand";
import { cmd } from "@/ipc/commands";
import type { Config } from "@/ipc/generated/Config";

interface State {
  config: Config | null;
  load: () => Promise<void>;
  update: (patch: Partial<Config>) => Promise<void>;
}

export const useConfigStore = create<State>((set) => ({
  config: null,
  load: async () => set({ config: await cmd.getConfig() }),
  update: async (patch) => set({ config: await cmd.updateConfig(patch) }),
}));
