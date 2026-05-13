import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import type { PermissionStatus } from "@/ipc/generated/PermissionStatus";

export function PermissionsStep({ onNext }: { onNext: () => void }) {
  const [status, setStatus] = useState<PermissionStatus | null>(null);

  async function refresh() {
    setStatus(await invoke<PermissionStatus>("check_permissions"));
  }
  useEffect(() => {
    void invoke<PermissionStatus>("request_accessibility").then(setStatus);
    const i = setInterval(refresh, 1500);
    return () => clearInterval(i);
  }, []);

  if (!status) return null;
  const allGood = status.accessibility && status.microphone;

  return (
    <div>
      <h2 className="text-xl font-semibold tracking-tight">Permissões</h2>
      <p className="mt-1 text-sm text-muted-foreground">
        Precisamos de dois acessos para o app funcionar.
      </p>
      <div className="mt-6 space-y-3">
        <PermRow
          ok={status.accessibility}
          title="Acessibilidade"
          help="Necessário para o atalho global e para colar nos outros apps."
          action={
            <Button
              variant="outline"
              onClick={() => void invoke("open_accessibility_panel")}
            >
              Abrir Ajustes
            </Button>
          }
        />
        <PermRow
          ok={status.microphone}
          title="Microfone"
          help="Será solicitado na primeira gravação."
        />
      </div>
      <div className="mt-8 flex justify-end">
        <Button
          disabled={!allGood}
          onClick={onNext}
          className={cn(allGood && "grad-accent border-0")}
        >
          Continuar
        </Button>
      </div>
    </div>
  );
}

function PermRow({
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
    <div className="flex items-center gap-3 rounded-lg border border-border/40 bg-background/30 p-3">
      <span
        className={cn(
          "inline-flex h-8 w-8 items-center justify-center rounded-full",
          ok
            ? "bg-emerald-500/15 text-emerald-500"
            : "bg-amber-500/15 text-amber-500",
        )}
      >
        {ok ? (
          <svg
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="3"
          >
            <polyline points="20 6 9 17 4 12" />
          </svg>
        ) : (
          <svg
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2.5"
          >
            <circle cx="12" cy="12" r="10" />
            <line x1="12" y1="8" x2="12" y2="12" />
            <line x1="12" y1="16" x2="12.01" y2="16" />
          </svg>
        )}
      </span>
      <div className="min-w-0 flex-1">
        <div className="text-sm font-medium">{title}</div>
        <div className="text-xs text-muted-foreground">{help}</div>
      </div>
      {action}
    </div>
  );
}
