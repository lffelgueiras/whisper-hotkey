import { Button } from "@/components/ui/button";

export function WelcomeStep({ onNext }: { onNext: () => void }) {
  return (
    <div className="text-center max-w-md mx-auto">
      <h1 className="text-2xl font-semibold mb-2">Welcome to Whisper Hotkey</h1>
      <p className="text-sm text-muted-foreground mb-6">
        Voice dictation, 100% local. Two quick steps and you're ready.
      </p>
      <Button onClick={onNext}>Get started</Button>
    </div>
  );
}
