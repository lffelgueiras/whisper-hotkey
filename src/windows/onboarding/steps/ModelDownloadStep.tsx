import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

interface ModelInfo {
  id: string;
  kind: string;
  url: string;
  sha256: string;
  size_bytes: number;
  display_name: string;
  min_ram_gb: number;
}

interface SystemSpecs {
  ram_gb: number;
}

interface DownloadProgress {
  id: string;
  downloaded: number;
  total: number;
}

function formatSize(bytes: number) {
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(0)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

type Compat = "recommended" | "supported" | "tight" | "insufficient";

function compatFor(model: ModelInfo, specs: SystemSpecs | null): Compat {
  if (!specs || specs.ram_gb <= 0) return "supported";
  const r = specs.ram_gb;
  if (r < model.min_ram_gb) return "insufficient";
  if (r < model.min_ram_gb * 1.25) return "tight";
  if (r >= model.min_ram_gb * 2) return "recommended";
  return "supported";
}

const BADGE: Record<Compat, { label: string; cls: string } | null> = {
  recommended: {
    label: "Recomendado",
    cls: "bg-emerald-500/15 text-emerald-300 border-emerald-500/30",
  },
  supported: null,
  tight: {
    label: "Limítrofe",
    cls: "bg-amber-500/15 text-amber-300 border-amber-500/30",
  },
  insufficient: {
    label: "RAM insuficiente",
    cls: "bg-red-500/15 text-red-300 border-red-500/30",
  },
};

export function ModelDownloadStep({ onDone }: { onDone: () => void }) {
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [present, setPresent] = useState<Record<string, boolean>>({});
  const [progress, setProgress] = useState<Record<string, DownloadProgress>>({});
  const [specs, setSpecs] = useState<SystemSpecs | null>(null);

  async function refreshPresent(ms: ModelInfo[]) {
    const map: Record<string, boolean> = {};
    for (const m of ms) {
      map[m.id] = await invoke<boolean>("is_model_present", { id: m.id });
    }
    setPresent(map);
  }

  useEffect(() => {
    void invoke<SystemSpecs>("get_system_specs").then(setSpecs);
    void invoke<ModelInfo[]>("list_models").then(async (ms) => {
      const asr = ms.filter((m) => m.kind === "asr");
      setModels(asr);
      await refreshPresent(asr);
    });
    const un = listen<DownloadProgress>("model-progress", (e) => {
      setProgress((p) => ({ ...p, [e.payload.id]: e.payload }));
      if (e.payload.downloaded >= e.payload.total && e.payload.total > 0) {
        setPresent((p) => ({ ...p, [e.payload.id]: true }));
      }
    });
    return () => {
      void un.then((u) => u());
    };
  }, []);

  async function download(id: string) {
    await invoke("download_model", { id });
    setPresent((p) => ({ ...p, [id]: true }));
  }

  const any = Object.values(present).some(Boolean);

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="shrink-0">
        <h2 className="text-xl font-semibold tracking-tight">Escolha um modelo</h2>
        <p className="mt-1 text-sm text-muted-foreground">
          O Whisper roda 100% local. Você pode trocar nas configurações depois.
          {specs && specs.ram_gb > 0 && (
            <>
              {" "}
              <span className="opacity-70">
                Detectado {specs.ram_gb.toFixed(0)} GB de RAM.
              </span>
            </>
          )}
        </p>
      </div>
      <div className="-mr-2 mt-6 grid min-h-0 flex-1 gap-2 overflow-y-auto pr-2">
        {models.map((m) => {
          const p = progress[m.id];
          const pct = p ? Math.round((p.downloaded / Math.max(p.total, 1)) * 100) : 0;
          const isPresent = present[m.id];
          const downloading = p && p.downloaded < p.total;
          const compat = compatFor(m, specs);
          const badge = BADGE[compat];
          return (
            <div
              key={m.id}
              className="rounded-lg border border-border/40 bg-background/30 p-3"
            >
              <div className="flex items-center justify-between gap-3">
                <div className="min-w-0">
                  <div className="flex flex-wrap items-center gap-2 text-sm font-medium">
                    <span>{m.display_name}</span>
                    {badge && (
                      <span
                        className={cn(
                          "rounded-full border px-1.5 py-0.5 text-[10px] font-medium uppercase tracking-wide",
                          badge.cls,
                        )}
                      >
                        {badge.label}
                      </span>
                    )}
                  </div>
                  <div className="mt-0.5 text-xs text-muted-foreground">
                    {formatSize(m.size_bytes)} · pico ~{m.min_ram_gb.toFixed(0)} GB RAM
                  </div>
                </div>
                <Button
                  variant={isPresent ? "outline" : "default"}
                  className={cn(!isPresent && !downloading && "grad-accent border-0")}
                  onClick={() => void download(m.id)}
                  disabled={isPresent || !!downloading || compat === "insufficient"}
                >
                  {isPresent ? "Baixado" : downloading ? `${pct}%` : "Baixar"}
                </Button>
              </div>
              {downloading && (
                <div className="mt-2 h-1 overflow-hidden rounded-full bg-muted">
                  <div
                    className="h-full grad-accent transition-all"
                    style={{ width: `${pct}%` }}
                  />
                </div>
              )}
            </div>
          );
        })}
      </div>
      <div className="mt-6 flex shrink-0 justify-end">
        <Button
          disabled={!any}
          onClick={onDone}
          className={cn(any && "grad-accent border-0")}
        >
          Continuar
        </Button>
      </div>
    </div>
  );
}
