import { create } from "zustand";

export type RecordingState = "idle" | "recording" | "transcribing";

interface State {
  state: RecordingState;
  setState: (s: RecordingState) => void;
}

export const useRecordingStore = create<State>((set) => ({
  state: "idle",
  setState: (state) => set({ state }),
}));
