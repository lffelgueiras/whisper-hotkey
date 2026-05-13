import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";

function eventToAccelerator(e: KeyboardEvent): string | null {
  const parts: string[] = [];
  if (e.metaKey) parts.push("Cmd");
  if (e.ctrlKey) parts.push("Control");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");
  const code = e.code;
  if (["ControlLeft", "ControlRight", "MetaLeft", "MetaRight", "AltLeft", "AltRight", "ShiftLeft", "ShiftRight"].includes(code)) {
    return null;
  }
  let key: string;
  if (code.startsWith("Key")) key = code.slice(3);
  else if (code.startsWith("Digit")) key = code.slice(5);
  else if (code === "Space") key = "Space";
  else if (code.startsWith("Arrow")) key = code.slice(5);
  else if (/^F\d+$/.test(code)) key = code;
  else if (e.key.length === 1) key = e.key.toUpperCase();
  else key = e.key;
  parts.push(key);
  return parts.join("+");
}

export function HotkeyCapture({
  value,
  onChange,
}: {
  value: string;
  onChange: (v: string) => void;
}) {
  const [recording, setRecording] = useState(false);
  const [debug, setDebug] = useState<string>("");
  const btnRef = useRef<HTMLButtonElement>(null);
  const onChangeRef = useRef(onChange);
  onChangeRef.current = onChange;
  const stableOnChange = useCallback((v: string) => onChangeRef.current(v), []);

  useEffect(() => {
    if (!recording) {
      setDebug("");
      return;
    }
    let cancelled = false;
    void invoke("pause_hotkey").then(() => {
      if (!cancelled) btnRef.current?.focus();
    });
    const handler = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();
      if (e.key === "Escape") {
        setRecording(false);
        return;
      }
      setDebug(`code=${e.code} key=${e.key} meta=${e.metaKey} ctrl=${e.ctrlKey} alt=${e.altKey} shift=${e.shiftKey}`);
      const acc = eventToAccelerator(e);
      if (acc) {
        stableOnChange(acc);
        setRecording(false);
      }
    };
    document.addEventListener("keydown", handler, { capture: true });
    return () => {
      cancelled = true;
      document.removeEventListener("keydown", handler, { capture: true });
      void invoke("resume_hotkey");
    };
  }, [recording, stableOnChange]);

  return (
    <div className="grid gap-1">
      <Button
        ref={btnRef}
        type="button"
        variant={recording ? "default" : "outline"}
        onClick={() => setRecording((r) => !r)}
        className={`w-full ${recording ? "animate-pulse" : ""}`}
      >
        {recording ? "Pressione a nova combinação… (Esc para cancelar)" : value}
      </Button>
      {recording && (
        <p className="text-xs text-muted-foreground">
          Pressione modificadores (Cmd/Ctrl/Alt/Shift) <strong>+ uma tecla</strong> (letra, número, F1-F12 ou espaço). Apenas modificadores não contam.
        </p>
      )}
      {recording && debug && (
        <p className="text-[10px] font-mono text-muted-foreground/60">{debug}</p>
      )}
    </div>
  );
}
