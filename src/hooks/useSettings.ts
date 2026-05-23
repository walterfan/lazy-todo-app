import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AppSettings } from "../types/settings";

const DEFAULT_APP_BG_COLOR = "#2f3a33";

function normalizeBackgroundColor(color: string): string {
  const candidate = color.trim();
  return /^#[0-9a-fA-F]{6}$/.test(candidate) ? candidate : DEFAULT_APP_BG_COLOR;
}

const DEFAULTS: AppSettings = {
  page_size: 10,
  note_page_size: 10,
  todo_display: "list",
  note_display: "list",
  app_background_color: "#2f3a33",
  note_template: "",
  note_folder: "",
  language: "en",
  note_template_files: [],
};

export function useSettings() {
  const [settings, setSettings] = useState<AppSettings>(DEFAULTS);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    invoke<AppSettings>("get_app_settings")
      .then((loaded) => {
        setSettings({
          ...DEFAULTS,
          ...loaded,
          app_background_color: normalizeBackgroundColor(loaded.app_background_color),
        });
      })
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  const updateSettings = useCallback(async (patch: Partial<AppSettings>) => {
    const normalizedPatch = {
      ...patch,
      ...(patch.app_background_color !== undefined
        ? { app_background_color: normalizeBackgroundColor(patch.app_background_color) }
        : {}),
    };
    const next = { ...settings, ...normalizedPatch };
    setSettings(next);
    await invoke("save_app_settings", { settings: next });
  }, [settings]);

  return { settings, loading, updateSettings };
}
