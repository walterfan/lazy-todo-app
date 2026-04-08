import React from "react";
import ReactDOM from "react-dom/client";
import { open } from "@tauri-apps/plugin-shell";
import App from "./App";

document.addEventListener("click", (e) => {
  const anchor = (e.target as HTMLElement).closest("a");
  if (anchor?.href && anchor.href.startsWith("http")) {
    e.preventDefault();
    open(anchor.href);
  }
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
