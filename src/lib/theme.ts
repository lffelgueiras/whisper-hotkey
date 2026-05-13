import { useConfigStore } from "@/store/configStore";
import { useEffect } from "react";

export function useApplyTheme() {
  const theme = useConfigStore((s) => s.config?.theme);
  useEffect(() => {
    if (!theme) return;
    const sysDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    const dark = theme === "dark" || (theme === "system" && sysDark);
    document.documentElement.classList.toggle("dark", dark);
  }, [theme]);
}
