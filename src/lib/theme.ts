import { useConfigStore } from "@/store/configStore";
import { useEffect } from "react";

function applyTheme(theme: "system" | "light" | "dark" | undefined) {
  if (!theme) return;
  const sysDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
  const dark = theme === "dark" || (theme === "system" && sysDark);
  document.documentElement.classList.toggle("dark", dark);
}

export function useApplyTheme() {
  const theme = useConfigStore((s) => s.config?.theme);
  useEffect(() => {
    applyTheme(theme);
    if (theme !== "system") return;
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = () => applyTheme(theme);
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  }, [theme]);
}
