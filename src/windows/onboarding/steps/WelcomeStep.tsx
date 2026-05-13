import { Button } from "@/components/ui/button";

export function WelcomeStep({ onNext }: { onNext: () => void }) {
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
          <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3z" />
          <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
          <line x1="12" y1="19" x2="12" y2="22" />
        </svg>
      </div>
      <h1 className="text-2xl font-semibold tracking-tight">
        Bem-vindo ao Whisper Hotkey
      </h1>
      <p className="mx-auto mt-2 max-w-sm text-sm text-muted-foreground">
        Ditado por voz, 100% local. Pressione o atalho, fale, e o texto aparece
        onde você está digitando.
      </p>
      <Button className="mt-8 grad-accent border-0 px-8" onClick={onNext}>
        Começar
      </Button>
    </div>
  );
}
