import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useRecordingStore } from "@/store/recordingStore";
import { bindRecordingEvents } from "@/ipc/events";
import { getCurrentWindow } from "@tauri-apps/api/window";

const BAR_COUNT = 24;

export function OverlayWindow() {
  const state = useRecordingStore((s) => s.state);
  const [bars, setBars] = useState<number[]>(() => new Array(BAR_COUNT).fill(0));
  const [elapsed, setElapsed] = useState(0);
  const barsRef = useRef(bars);
  barsRef.current = bars;
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    bindRecordingEvents().then((u) => (unlisten = u));
    return () => unlisten?.();
  }, []);

  useEffect(() => {
    let alive = true;
    const tick = async () => {
      while (alive) {
        try {
          const lvl = await invoke<number>("get_audio_level");
          const clamped = Math.min(1, Math.max(0, lvl));
          const next = barsRef.current.slice(1);
          next.push(clamped);
          setBars(next);
        } catch {
          /* ignore */
        }
        await new Promise((r) => setTimeout(r, 50));
      }
    };
    void tick();
    return () => {
      alive = false;
    };
  }, []);

  useEffect(() => {
    const w = getCurrentWindow();
    if (state === "idle") {
      w.hide();
      setBars(new Array(BAR_COUNT).fill(0));
      setElapsed(0);
      return;
    }
    w.show();
    if (state !== "recording") return;
    const start = Date.now();
    const id = window.setInterval(() => {
      setElapsed(Math.floor((Date.now() - start) / 1000));
    }, 250);
    return () => window.clearInterval(id);
  }, [state]);

  const mm = String(Math.floor(elapsed / 60)).padStart(2, "0");
  const ss = String(elapsed % 60).padStart(2, "0");

  return (
    <div className="flex h-full w-full items-center justify-center">
      <div
        className="flex h-full w-full items-center gap-2.5 rounded-full px-3.5 py-1.5"
        style={{
          backgroundColor: "rgba(10, 12, 18, 0.7)",
          backdropFilter: "blur(24px) saturate(180%)",
          WebkitBackdropFilter: "blur(24px) saturate(180%)",
          border: "1px solid rgba(255, 255, 255, 0.1)",
          boxShadow: "0 0 0 1px rgba(255, 255, 255, 0.05) inset",
        }}
      >
        {state === "transcribing" ? (
          <>
            <span className="relative inline-flex h-2 w-2 shrink-0">
              <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-cyan-400 opacity-60" />
              <span className="relative inline-flex h-2 w-2 rounded-full bg-cyan-400" />
            </span>
            <span className="text-xs font-medium text-white">Transcrevendo</span>
            <span className="ml-auto inline-block h-3 w-3 animate-spin rounded-full border-2 border-white/70 border-t-transparent" />
          </>
        ) : (
          <>
            <span className="relative inline-flex h-2 w-2 shrink-0">
              <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-red-500 opacity-70" />
              <span className="relative inline-flex h-2 w-2 rounded-full bg-red-500" />
            </span>
            <div className="flex h-full flex-1 items-center justify-center gap-[2px]">
              {bars.map((v, i) => {
                const db = v > 0 ? 20 * Math.log10(v) : -80;
                const norm = Math.max(0, Math.min(1, (db + 50) / 50));
                const h = Math.max(3, norm * 100);
                return (
                  <span
                    key={i}
                    className="inline-block w-[2px] rounded-full transition-[height] duration-75 ease-out"
                    style={{
                      height: `${h}%`,
                      background:
                        "linear-gradient(180deg, #f87171 0%, #ef4444 100%)",
                    }}
                  />
                );
              })}
            </div>
            <span className="font-mono text-[11px] tabular-nums text-white/70">
              {mm}:{ss}
            </span>
          </>
        )}
      </div>
    </div>
  );
}
