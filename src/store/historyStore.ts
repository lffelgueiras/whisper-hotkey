import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { HistoryEntry } from "@/ipc/generated/HistoryEntry";

interface State {
  entries: HistoryEntry[];
  load: () => Promise<void>;
  remove: (ts: string) => Promise<void>;
  clearAll: () => Promise<void>;
}

export const useHistoryStore = create<State>((set, get) => ({
  entries: [],
  load: async () => set({ entries: await invoke<HistoryEntry[]>("get_history") }),
  remove: async (ts) => {
    await invoke("delete_history_entry", { ts });
    await get().load();
  },
  clearAll: async () => {
    await invoke("clear_history");
    set({ entries: [] });
  },
}));
