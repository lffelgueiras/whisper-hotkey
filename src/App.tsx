import { useEffect } from "react";
import { useConfigStore } from "@/store/configStore";
import { OnboardingWindow } from "@/windows/onboarding/OnboardingWindow";
import { SettingsWindow } from "@/windows/settings/SettingsWindow";
import { useApplyTheme } from "@/lib/theme";

export default function App() {
  useApplyTheme();
  const { config, load } = useConfigStore();
  useEffect(() => {
    void load();
  }, [load]);
  if (!config) return null;
  return config.onboarding_complete ? <SettingsWindow /> : <OnboardingWindow />;
}
