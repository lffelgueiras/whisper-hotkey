import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import type { PermissionStatus } from "@/ipc/generated/PermissionStatus";

export function PermissionsStep({ onNext }: { onNext: () => void }) {
  const [status, setStatus] = useState<PermissionStatus | null>(null);

  async function refresh() {
    setStatus(await invoke<PermissionStatus>("check_permissions"));
  }
  useEffect(() => {
    void refresh();
    const i = setInterval(refresh, 1500);
    return () => clearInterval(i);
  }, []);

  if (!status) return null;
  const allGood = status.accessibility && status.microphone;

  return (
    <div className="max-w-md mx-auto">
      <h2 className="text-xl font-semibold mb-4">Permissions</h2>
      <div className="space-y-4">
        <Row
          ok={status.accessibility}
          title="Accessibility"
          help="Required for global hotkey and pasting into other apps."
          action={
            <Button variant="outline" onClick={() => void invoke("open_accessibility_panel")}>
              Open System Settings
            </Button>
          }
        />
        <Row
          ok={status.microphone}
          title="Microphone"
          help="Will be requested when you start your first recording."
        />
      </div>
      <div className="mt-6 flex justify-end">
        <Button disabled={!allGood} onClick={onNext}>
          Continue
        </Button>
      </div>
    </div>
  );
}

function Row({
  ok,
  title,
  help,
  action,
}: {
  ok: boolean;
  title: string;
  help: string;
  action?: React.ReactNode;
}) {
  return (
    <div className="flex items-center gap-3">
      <span
        className={`inline-block h-3 w-3 rounded-full ${ok ? "bg-green-500" : "bg-red-500"}`}
      />
      <div className="flex-1">
        <div className="font-medium">{title}</div>
        <div className="text-xs text-muted-foreground">{help}</div>
      </div>
      {action}
    </div>
  );
}
