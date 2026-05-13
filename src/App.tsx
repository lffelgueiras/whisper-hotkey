import { useEffect } from "react";
import { Toaster, toast } from "sonner";
import { listen } from "@tauri-apps/api/event";
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
  useEffect(() => {
    const u = listen<{ kind: string; message: string }>("error", (e) => {
      toast.error(`${e.payload.kind}: ${e.payload.message}`);
    });
    return () => {
      void u.then((f) => f());
    };
  }, []);
  if (!config) return null;
  return (
    <>
      {config.onboarding_complete ? <SettingsWindow /> : <OnboardingWindow />}
      <Toaster />
    </>
  );
}
