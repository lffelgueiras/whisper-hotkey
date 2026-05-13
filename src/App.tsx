import { useEffect } from "react";
import { useConfigStore } from "@/store/configStore";
import { OnboardingWindow } from "@/windows/onboarding/OnboardingWindow";
import { SettingsWindow } from "@/windows/settings/SettingsWindow";

export default function App() {
  const { config, load } = useConfigStore();
  useEffect(() => {
    void load();
  }, [load]);
  if (!config) return null;
  return config.onboarding_complete ? <SettingsWindow /> : <OnboardingWindow />;
}
