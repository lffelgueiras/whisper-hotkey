import { HistoryTab } from "./HistoryTab";

export function HistoryWindow() {
  return (
    <div className="flex h-screen w-screen flex-col p-6">
      <div className="glass flex flex-1 flex-col overflow-hidden">
        <header className="border-b border-border/40 px-6 py-4">
          <h1 className="text-base font-semibold tracking-tight">Histórico</h1>
          <p className="mt-0.5 text-xs text-muted-foreground">
            Suas transcrições recentes
          </p>
        </header>
        <div className="flex-1 overflow-y-auto px-6 py-4">
          <HistoryTab />
        </div>
      </div>
    </div>
  );
}
