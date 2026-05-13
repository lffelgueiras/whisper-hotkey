import { Button } from "@/components/ui/button";

export function PermissionsStep({ onNext }: { onNext: () => void }) {
  return (
    <div className="max-w-md mx-auto">
      <h2 className="text-xl font-semibold mb-4">Permissions</h2>
      <p className="text-sm text-muted-foreground mb-6">Stub — fleshed out in M7.3.</p>
      <Button onClick={onNext}>Continue</Button>
    </div>
  );
}
