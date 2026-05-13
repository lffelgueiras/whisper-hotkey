import { useEffect, useState } from "react";
import { useHistoryStore } from "@/store/historyStore";
import { invoke } from "@tauri-apps/api/core";
import { save as saveDialog } from "@tauri-apps/plugin-dialog";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";

export function HistoryWindow() {
  const { entries, load, remove, clearAll } = useHistoryStore();
  const [q, setQ] = useState("");
  useEffect(() => {
    void load();
  }, [load]);

  const filtered = entries
    .filter((e) => e.text.toLowerCase().includes(q.toLowerCase()))
    .reverse();

  async function copy(text: string) {
    await navigator.clipboard.writeText(text);
  }

  async function exportAll() {
    const path = await saveDialog({
      defaultPath: "history.md",
      filters: [{ name: "Markdown", extensions: ["md"] }],
    });
    if (path) await invoke("export_history", { path });
  }

  return (
    <div className="flex h-screen w-screen flex-col p-4 gap-3">
      <div className="flex gap-2">
        <Input placeholder="Search…" value={q} onChange={(e) => setQ(e.target.value)} />
        <Button variant="outline" onClick={exportAll}>
          Export
        </Button>
        <Button variant="ghost" onClick={() => void clearAll()}>
          Clear all
        </Button>
      </div>
      <div className="flex-1 overflow-y-auto space-y-2">
        {filtered.map((e) => (
          <div key={e.ts} className="rounded border p-3 text-sm">
            <div className="mb-1 flex items-center justify-between">
              <span className="text-xs text-muted-foreground">
                {new Date(e.ts).toLocaleString()}
              </span>
              <div className="flex gap-1">
                <Button variant="ghost" onClick={() => void copy(e.text)}>
                  Copy
                </Button>
                <Button variant="ghost" onClick={() => void remove(e.ts)}>
                  Delete
                </Button>
              </div>
            </div>
            <div className="whitespace-pre-wrap">{e.text}</div>
          </div>
        ))}
        {filtered.length === 0 && (
          <div className="text-center text-sm text-muted-foreground py-12">
            No transcriptions yet.
          </div>
        )}
      </div>
    </div>
  );
}
