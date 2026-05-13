import { useEffect } from "react";
import { useRecordingStore } from "@/store/recordingStore";
import { bindRecordingEvents } from "@/ipc/events";
import { getCurrentWindow } from "@tauri-apps/api/window";

export function OverlayWindow() {
  const state = useRecordingStore((s) => s.state);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    bindRecordingEvents().then((u) => (unlisten = u));
    return () => unlisten?.();
  }, []);

  useEffect(() => {
    const w = getCurrentWindow();
    if (state === "idle") w.hide();
    else w.show();
  }, [state]);

  const label =
    state === "recording" ? "Recording" : state === "transcribing" ? "Transcribing…" : "";
  const color =
    state === "recording" ? "bg-red-500" : state === "transcribing" ? "bg-blue-500" : "bg-transparent";

  return (
    <div className="flex h-full w-full items-center justify-center">
      <div
        className={`flex items-center gap-2 rounded-full px-3 py-1 text-sm text-white ${color} ${state === "recording" ? "animate-pulse" : ""}`}
      >
        {state === "transcribing" && (
          <span className="inline-block h-3 w-3 animate-spin rounded-full border-2 border-white border-t-transparent" />
        )}
        {state === "recording" && (
          <span className="inline-block h-2 w-2 rounded-full bg-white" />
        )}
        <span>{label}</span>
      </div>
    </div>
  );
}
