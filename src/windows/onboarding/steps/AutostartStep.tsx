import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";

export function AutostartStep({ onNext }: { onNext: () => void }) {
  async function choose(enable: boolean) {
    try {
      await invoke("set_autostart", { enable });
    } catch (e) {
      console.error("set_autostart failed", e);
    }
    onNext();
  }
  return (
    <div className="text-center">
      <div className="mx-auto mb-6 flex h-16 w-16 items-center justify-center rounded-2xl grad-accent">
        <svg
          width="32"
          height="32"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
        >
          <circle cx="12" cy="12" r="10" />
          <polyline points="12 6 12 12 16 14" />
        </svg>
      </div>
      <h1 className="text-2xl font-semibold tracking-tight">
        Iniciar com o sistema?
      </h1>
      <p className="mx-auto mt-2 max-w-sm text-sm text-muted-foreground">
        Quer que o Whisper Hotkey abra automaticamente quando você ligar o
        computador? Você pode mudar isso depois em Settings.
      </p>
      <div className="mt-8 flex justify-center gap-3">
        <Button variant="outline" onClick={() => void choose(false)}>
          Agora não
        </Button>
        <Button className="grad-accent border-0 px-6" onClick={() => void choose(true)}>
          Sim, iniciar com o sistema
        </Button>
      </div>
    </div>
  );
}
