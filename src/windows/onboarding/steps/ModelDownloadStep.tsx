import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Button } from "@/components/ui/button";

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

export function ModelDownloadStep({ onDone }: { onDone: () => void }) {
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [present, setPresent] = useState<Record<string, boolean>>({});
  const [progress, setProgress] = useState<Record<string, DownloadProgress>>({});

  async function refreshPresent(ms: ModelInfo[]) {
    const map: Record<string, boolean> = {};
    for (const m of ms) {
      map[m.id] = await invoke<boolean>("is_model_present", { id: m.id });
    }
    setPresent(map);
  }

  useEffect(() => {
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
    <div className="max-w-xl mx-auto">
      <h2 className="text-xl font-semibold mb-2">Pick a transcription model</h2>
      <p className="text-sm text-muted-foreground mb-6">
        You can change this anytime in Settings.
      </p>
      <ul className="grid gap-3">
        {models.map((m) => {
          const p = progress[m.id];
          const isPresent = present[m.id];
          return (
            <li
              key={m.id}
              className="flex items-center justify-between rounded-md border border-border p-3"
            >
              <div>
                <div className="font-medium">{m.display_name}</div>
                {p && (
                  <div className="text-xs text-muted-foreground">
                    {Math.round((p.downloaded / Math.max(p.total, 1)) * 100)}%
                  </div>
                )}
              </div>
              <Button
                variant={isPresent ? "default" : "outline"}
                onClick={() => void download(m.id)}
                disabled={isPresent}
              >
                {isPresent ? "Downloaded" : "Download"}
              </Button>
            </li>
          );
        })}
      </ul>
      <div className="mt-6 flex justify-end">
        <Button disabled={!any} onClick={onDone}>
          Finish
        </Button>
      </div>
    </div>
  );
}
