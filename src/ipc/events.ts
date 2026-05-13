import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useRecordingStore, RecordingState } from "@/store/recordingStore";

export async function bindRecordingEvents(): Promise<UnlistenFn> {
  return listen<RecordingState>("state-changed", (e) => {
    useRecordingStore.getState().setState(e.payload);
  });
}
