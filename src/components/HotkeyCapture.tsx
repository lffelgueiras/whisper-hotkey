import { useState } from "react";
import { Button } from "@/components/ui/button";

function eventToAccelerator(e: KeyboardEvent): string | null {
  const parts: string[] = [];
  if (e.metaKey || e.ctrlKey) parts.push("CmdOrControl");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");
  const key = e.key.length === 1 ? e.key.toUpperCase() : e.key;
  if (["Control", "Meta", "Alt", "Shift"].includes(key)) return null;
  parts.push(key === " " ? "Space" : key);
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
  return (
    <Button
      type="button"
      variant="outline"
      onClick={() => setRecording(true)}
      onKeyDown={(e) => {
        if (!recording) return;
        e.preventDefault();
        const acc = eventToAccelerator(e.nativeEvent);
        if (acc) {
          onChange(acc);
          setRecording(false);
        }
      }}
    >
      {recording ? "Press hotkey…" : value}
    </Button>
  );
}
