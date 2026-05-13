import { useEffect, useMemo, useState } from "react";
import { useHistoryStore } from "@/store/historyStore";
import { invoke } from "@tauri-apps/api/core";
import { save as saveDialog } from "@tauri-apps/plugin-dialog";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import type { HistoryEntry } from "@/ipc/generated/HistoryEntry";

type Filter = "today" | "week" | "month" | "all";

const FILTERS: { id: Filter; label: string }[] = [
  { id: "today", label: "Hoje" },
  { id: "week", label: "Semana" },
  { id: "month", label: "Mês" },
  { id: "all", label: "Tudo" },
];

function relativeTime(iso: string): string {
  const now = Date.now();
  const ts = new Date(iso).getTime();
  const diff = Math.max(0, now - ts);
  const min = Math.floor(diff / 60000);
  if (min < 1) return "agora";
  if (min < 60) return `há ${min} min`;
  const h = Math.floor(min / 60);
  if (h < 24) return `há ${h} h`;
  const d = Math.floor(h / 24);
  if (d === 1) return "ontem";
  if (d < 7) return `há ${d} dias`;
  return new Date(iso).toLocaleDateString();
}

function withinFilter(iso: string, f: Filter): boolean {
  if (f === "all") return true;
  const ts = new Date(iso).getTime();
  const ms = Date.now() - ts;
  if (f === "today") return ms < 24 * 60 * 60 * 1000;
  if (f === "week") return ms < 7 * 24 * 60 * 60 * 1000;
  if (f === "month") return ms < 30 * 24 * 60 * 60 * 1000;
  return true;
}

function wordCount(text: string): number {
  return text.trim().split(/\s+/).filter(Boolean).length;
}

interface ItemProps {
  entry: HistoryEntry;
  onCopy: () => void;
  onDelete: () => void;
}

function HistoryItem({ entry, onCopy, onDelete }: ItemProps) {
  return (
    <div className="group rounded-lg border border-border/40 bg-background/30 p-4 transition-colors hover:bg-background/50">
      <div className="mb-2 flex items-center justify-between gap-2">
        <span className="flex items-center gap-2 text-xs text-muted-foreground">
          <span className="h-1.5 w-1.5 rounded-full bg-current opacity-60" />
          {relativeTime(entry.ts)}
          <span className="opacity-40">·</span>
          <span>{entry.model}</span>
          {entry.post_processed && (
            <span className="opacity-40">· refinado</span>
          )}
        </span>
        <div className="flex gap-1 opacity-0 transition-opacity group-hover:opacity-100">
          <button
            onClick={onCopy}
            className="rounded-md p-1.5 text-muted-foreground hover:bg-muted hover:text-foreground"
            title="Copiar"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <rect x="9" y="9" width="13" height="13" rx="2" />
              <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
            </svg>
          </button>
          <button
            onClick={onDelete}
            className="rounded-md p-1.5 text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
            title="Apagar"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M3 6h18M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2m3 0v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6" />
            </svg>
          </button>
        </div>
      </div>
      <p className="whitespace-pre-wrap text-sm leading-relaxed">{entry.text}</p>
      <div className="mt-2 flex gap-4 text-xs text-muted-foreground">
        <span>{wordCount(entry.text)} palavras</span>
      </div>
    </div>
  );
}

export function HistoryTab() {
  const { entries, load, remove, clearAll } = useHistoryStore();
  const [q, setQ] = useState("");
  const [filter, setFilter] = useState<Filter>("all");

  useEffect(() => {
    void load();
  }, [load]);

  const filtered = useMemo(() => {
    const ql = q.toLowerCase();
    return entries
      .filter((e) => withinFilter(e.ts, filter))
      .filter((e) => !ql || e.text.toLowerCase().includes(ql))
      .slice()
      .reverse();
  }, [entries, q, filter]);

  const totalWords = useMemo(
    () => entries.reduce((acc, e) => acc + wordCount(e.text), 0),
    [entries],
  );

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
    <div>
      <div className="mb-4 flex items-center justify-between gap-2">
        <p className="text-xs text-muted-foreground">
          {entries.length} transcrições · {totalWords.toLocaleString()} palavras
        </p>
        <div className="flex gap-2">
          <Button variant="outline" onClick={exportAll}>
            Exportar
          </Button>
          <Button
            variant="ghost"
            onClick={() =>
              entries.length &&
              confirm("Apagar todo o histórico?") &&
              void clearAll()
            }
          >
            Limpar
          </Button>
        </div>
      </div>

      <div className="flex items-center gap-2 rounded-lg border border-border/60 bg-background/40 px-3 py-2">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="opacity-50">
          <circle cx="11" cy="11" r="7" />
          <path d="m21 21-4.3-4.3" />
        </svg>
        <input
          placeholder="Buscar transcrições..."
          value={q}
          onChange={(e) => setQ(e.target.value)}
          className="flex-1 bg-transparent text-sm outline-none placeholder:text-muted-foreground"
        />
      </div>

      <div className="mt-3 flex flex-wrap gap-1.5">
        {FILTERS.map((f) => (
          <button
            key={f.id}
            onClick={() => setFilter(f.id)}
            className={cn(
              "rounded-full border border-border/60 bg-background/40 px-3 py-1 text-xs transition-colors",
              filter === f.id
                ? "border-foreground/40 bg-foreground text-background"
                : "text-muted-foreground hover:text-foreground",
            )}
          >
            {f.label}
          </button>
        ))}
      </div>

      <div className="mt-4 space-y-2">
        {filtered.map((e) => (
          <HistoryItem
            key={e.ts}
            entry={e}
            onCopy={() => void copy(e.text)}
            onDelete={() => void remove(e.ts)}
          />
        ))}
        {filtered.length === 0 && (
          <div className="py-16 text-center text-sm text-muted-foreground">
            {q || filter !== "all"
              ? "Nenhuma transcrição corresponde ao filtro."
              : "Ainda nenhuma transcrição. Pressione o atalho e fale algo."}
          </div>
        )}
      </div>
    </div>
  );
}
