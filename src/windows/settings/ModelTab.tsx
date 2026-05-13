import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useConfigStore } from "@/store/configStore";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";

interface ModelInfo {
  id: string;
  kind: string;
  url: string;
  sha256: string;
  size_bytes: number;
  display_name: string;
}

interface DownloadProgress {
  id: string;
  downloaded: number;
  total: number;
}

export function ModelTab() {
  const { config, load, update } = useConfigStore();
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [progress, setProgress] = useState<Record<string, DownloadProgress>>({});

  useEffect(() => {
    void load();
    invoke<ModelInfo[]>("list_models").then(setModels).catch(() => setModels([]));
    const un = listen<DownloadProgress>("model-progress", (e) => {
      setProgress((p) => ({ ...p, [e.payload.id]: e.payload }));
    });
    return () => {
      void un.then((u) => u());
    };
  }, [load]);

  if (!config) return null;

  return (
    <div className="mt-4 grid gap-4">
      <Label>ASR model</Label>
      <ul className="grid gap-3">
        {models
          .filter((m) => m.kind === "asr")
          .map((m) => {
            const p = progress[m.id];
            const active = config.asr_model === m.id;
            return (
              <li
                key={m.id}
                className="flex items-center justify-between rounded-md border border-border p-3"
              >
                <div>
                  <div className="font-medium">{m.display_name}</div>
                  {p && (
                    <div className="text-xs text-muted-foreground">
                      {Math.round((p.downloaded / p.total) * 100)}%
                    </div>
                  )}
                </div>
                <Button
                  variant={active ? "default" : "outline"}
                  onClick={() => {
                    void update({ asr_model: m.id });
                    void invoke("download_model", { id: m.id });
                  }}
                >
                  {active ? "Active" : "Use"}
                </Button>
              </li>
            );
          })}
      </ul>
    </div>
  );
}
