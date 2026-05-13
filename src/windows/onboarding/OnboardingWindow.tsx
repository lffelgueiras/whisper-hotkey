import { useState } from "react";
import { WelcomeStep } from "./steps/WelcomeStep";
import { PermissionsStep } from "./steps/PermissionsStep";
import { ModelDownloadStep } from "./steps/ModelDownloadStep";
import { useConfigStore } from "@/store/configStore";

export function OnboardingWindow() {
  const [step, setStep] = useState(0);
  const { update } = useConfigStore();
  async function finish() {
    await update({ onboarding_complete: true });
    window.location.reload();
  }
  return (
    <div className="h-screen p-10">
      {step === 0 && <WelcomeStep onNext={() => setStep(1)} />}
      {step === 1 && <PermissionsStep onNext={() => setStep(2)} />}
      {step === 2 && <ModelDownloadStep onDone={finish} />}
    </div>
  );
}
