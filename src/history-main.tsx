import { useEffect } from "react";
import ReactDOM from "react-dom/client";
import "./styles/globals.css";
import { HistoryWindow } from "./windows/history/HistoryWindow";
import { useConfigStore } from "./store/configStore";
import { useApplyTheme } from "./lib/theme";

function HistoryRoot() {
  useApplyTheme();
  const load = useConfigStore((s) => s.load);
  useEffect(() => {
    void load();
  }, [load]);
  return <HistoryWindow />;
}

ReactDOM.createRoot(document.getElementById("history-root")!).render(<HistoryRoot />);
