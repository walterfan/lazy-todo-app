import React from "react";
import ReactDOM from "react-dom/client";
import { open } from "@tauri-apps/plugin-shell";
import App from "./App";
import { NoteWindow } from "./components/NoteWindow";

document.addEventListener("click", (e) => {
  const anchor = (e.target as HTMLElement).closest("a");
  if (anchor?.href && anchor.href.startsWith("http")) {
    e.preventDefault();
    open(anchor.href);
  }
});

const params = new URLSearchParams(window.location.search);
const noteId = params.get("note");

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    {noteId ? <NoteWindow noteId={Number(noteId)} /> : <App />}
  </React.StrictMode>,
);
