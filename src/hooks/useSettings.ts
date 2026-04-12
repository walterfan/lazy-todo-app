import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AppSettings } from "../types/settings";

const DEFAULTS: AppSettings = {
  page_size: 50,
  todo_display: "list",
  note_display: "grid",
  note_template: "",
  note_folder: "",
};

export function useSettings() {
  const [settings, setSettings] = useState<AppSettings>(DEFAULTS);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    invoke<AppSettings>("get_app_settings")
      .then(setSettings)
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  const updateSettings = useCallback(async (patch: Partial<AppSettings>) => {
    const next = { ...settings, ...patch };
    setSettings(next);
    await invoke("save_app_settings", { settings: next });
  }, [settings]);

  return { settings, loading, updateSettings };
}
