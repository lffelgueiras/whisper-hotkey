import ReactDOM from "react-dom/client";
import "./styles/globals.css";
import { HistoryWindow } from "./windows/history/HistoryWindow";

ReactDOM.createRoot(document.getElementById("history-root")!).render(<HistoryWindow />);
