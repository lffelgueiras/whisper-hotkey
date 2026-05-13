import { useState } from "react";
import { WelcomeStep } from "./steps/WelcomeStep";
import { PermissionsStep } from "./steps/PermissionsStep";
import { ModelDownloadStep } from "./steps/ModelDownloadStep";
import { AutostartStep } from "./steps/AutostartStep";
import { useConfigStore } from "@/store/configStore";
import { cn } from "@/lib/utils";

const STEPS = ["Bem-vindo", "Permissões", "Modelo", "Autostart"];

export function OnboardingWindow() {
  const [step, setStep] = useState(0);
  const { update } = useConfigStore();

  async function finish() {
    await update({ onboarding_complete: true });
    window.location.reload();
  }

  return (
    <div className="flex h-screen w-screen items-center justify-center p-8">
      <div className="glass flex max-h-full w-full max-w-xl flex-col overflow-hidden p-8">
        <div className="mb-6 flex shrink-0 items-center justify-center gap-2">
          {STEPS.map((_, i) => (
            <span
              key={i}
              className={cn(
                "h-1.5 rounded-full transition-all",
                i === step
                  ? "w-8 bg-accent"
                  : i < step
                    ? "w-1.5 bg-accent/60"
                    : "w-1.5 bg-muted",
              )}
            />
          ))}
        </div>
        <div className="flex min-h-0 flex-1 flex-col">
          {step === 0 && <WelcomeStep onNext={() => setStep(1)} />}
          {step === 1 && <PermissionsStep onNext={() => setStep(2)} />}
          {step === 2 && <ModelDownloadStep onDone={() => setStep(3)} />}
          {step === 3 && <AutostartStep onNext={finish} />}
        </div>
      </div>
    </div>
  );
}
