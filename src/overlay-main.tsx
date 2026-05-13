import ReactDOM from "react-dom/client";
import "./styles/globals.css";
import { OverlayWindow } from "./windows/overlay/OverlayWindow";

ReactDOM.createRoot(document.getElementById("overlay-root")!).render(<OverlayWindow />);
