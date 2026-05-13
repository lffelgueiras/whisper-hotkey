import { Button } from "@/components/ui/button";

export function ModelDownloadStep({ onDone }: { onDone: () => void }) {
  return (
    <div className="max-w-xl mx-auto">
      <h2 className="text-xl font-semibold mb-2">Pick a transcription model</h2>
      <p className="text-sm text-muted-foreground mb-6">Stub — fleshed out in M7.4.</p>
      <Button onClick={onDone}>Finish</Button>
    </div>
  );
}
