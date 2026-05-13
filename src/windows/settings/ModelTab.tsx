import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useConfigStore } from "@/store/configStore";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { Row, SectionTitle } from "@/components/Row";
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
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
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

interface RowProps {
  m: ModelInfo;
  active: boolean;
  downloaded: boolean;
  progress?: DownloadProgress;
  compat: Compat;
  onUse: () => void;
  onDelete: () => void;
}

function ModelRow({ m, active, downloaded, progress, compat, onUse, onDelete }: RowProps) {
  const downloading = progress && progress.downloaded < progress.total;
  const pct = downloading
    ? Math.round((progress!.downloaded / progress!.total) * 100)
    : 0;
  const badge = BADGE[compat];
  const insufficient = compat === "insufficient";

  return (
    <div
      className={cn(
        "rounded-lg border border-border/60 bg-background/40 p-4 transition-colors",
        active && "border-accent/40 bg-accent/5",
      )}
    >
      <div className="flex items-center justify-between gap-3">
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-2">
            <span className="text-sm font-medium">{m.display_name}</span>
            {downloaded && (
              <span className="inline-flex items-center gap-1 rounded-full border border-emerald-500/30 bg-emerald-500/10 px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wider text-emerald-500">
                ● Baixado
              </span>
            )}
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
          {downloading && (
            <div className="mt-2 flex items-center gap-2">
              <div className="h-1 flex-1 overflow-hidden rounded-full bg-muted">
                <div
                  className="h-full grad-accent transition-all"
                  style={{ width: `${pct}%` }}
                />
              </div>
              <span className="text-xs text-muted-foreground tabular-nums">
                {pct}%
              </span>
            </div>
          )}
        </div>
        <div className="flex shrink-0 items-center gap-2">
          {downloaded && !active && (
            <Button variant="ghost" onClick={onDelete}>
              Apagar
            </Button>
          )}
          <Button
            className={cn(active && "grad-accent border-0")}
            variant={active ? "default" : "outline"}
            onClick={onUse}
            disabled={!active && !downloaded && insufficient}
          >
            {active ? "Ativo" : downloaded ? "Usar" : "Baixar e usar"}
          </Button>
        </div>
      </div>
    </div>
  );
}

export function ModelTab() {
  const { config, load, update } = useConfigStore();
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [progress, setProgress] = useState<Record<string, DownloadProgress>>({});
  const [present, setPresent] = useState<Record<string, boolean>>({});
  const [specs, setSpecs] = useState<SystemSpecs | null>(null);

  async function refreshPresent(list: ModelInfo[]) {
    const entries = await Promise.all(
      list.map(
        async (m) =>
          [m.id, await invoke<boolean>("is_model_present", { id: m.id })] as const,
      ),
    );
    setPresent(Object.fromEntries(entries));
  }

  async function handleDelete(id: string) {
    try {
      await invoke("delete_model", { id });
    } finally {
      setPresent((cur) => ({ ...cur, [id]: false }));
      setProgress((cur) => {
        const next = { ...cur };
        delete next[id];
        return next;
      });
    }
  }

  useEffect(() => {
    void load();
    void invoke<SystemSpecs>("get_system_specs").then(setSpecs);
    invoke<ModelInfo[]>("list_models")
      .then((list) => {
        setModels(list);
        void refreshPresent(list);
      })
      .catch(() => setModels([]));
    const un = listen<DownloadProgress>("model-progress", (e) => {
      setProgress((p) => ({ ...p, [e.payload.id]: e.payload }));
      if (e.payload.downloaded >= e.payload.total && e.payload.total > 0) {
        void invoke<boolean>("is_model_present", { id: e.payload.id }).then(
          (ok) => setPresent((cur) => ({ ...cur, [e.payload.id]: ok })),
        );
      }
    });
    return () => {
      void un.then((u) => u());
    };
  }, [load]);

  if (!config) return null;

  const asrModels = models.filter((m) => m.kind === "asr");
  const llmModels = models.filter((m) => m.kind === "llm");

  return (
    <div className="max-w-2xl">
      <SectionTitle>Reconhecimento de fala (ASR)</SectionTitle>
      {specs && specs.ram_gb > 0 && (
        <p className="-mt-2 mb-3 text-xs text-muted-foreground">
          Detectado {specs.ram_gb.toFixed(0)} GB de RAM nesta máquina.
        </p>
      )}
      <div className="grid gap-2">
        {asrModels.map((m) => (
          <ModelRow
            key={m.id}
            m={m}
            active={config.asr_model === m.id}
            downloaded={!!present[m.id]}
            progress={progress[m.id]}
            compat={compatFor(m, specs)}
            onUse={() => {
              void update({ asr_model: m.id });
              void invoke("download_model", { id: m.id });
            }}
            onDelete={() => void handleDelete(m.id)}
          />
        ))}
      </div>

      <SectionTitle>Pós-processamento (LLM)</SectionTitle>
      <Row
        label="Habilitar pós-processamento"
        desc="Refina pontuação, formatação e ortografia"
      >
        <Switch
          checked={config.post_processing_enabled}
          onCheckedChange={(v) => void update({ post_processing_enabled: v })}
        />
      </Row>

      {config.post_processing_enabled && (
        <div className="mt-4 grid gap-2">
          {llmModels.map((m) => (
            <ModelRow
              key={m.id}
              m={m}
              active={config.llm_model === m.id}
              downloaded={!!present[m.id]}
              progress={progress[m.id]}
              compat={compatFor(m, specs)}
              onUse={() => {
                void update({ llm_model: m.id });
                void invoke("download_model", { id: m.id });
              }}
              onDelete={() => void handleDelete(m.id)}
            />
          ))}
        </div>
      )}
    </div>
  );
}
