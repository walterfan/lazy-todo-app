import { useCallback, useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

/**
 * Manages the `alwaysOnTop` flag for the current Tauri window.
 *
 * - Reads the initial value from the OS window on mount.
 * - Binds `Cmd/Ctrl+Shift+T` to toggle the flag.
 * - Returns `{ pinned, toggle, setPinned }` for UI controls.
 */
export function useAlwaysOnTop() {
  const [pinned, setPinnedState] = useState(false);
  const [ready, setReady] = useState(false);

  useEffect(() => {
    let cancelled = false;
    getCurrentWindow()
      .isAlwaysOnTop()
      .then((value) => {
        if (!cancelled) {
          setPinnedState(value);
          setReady(true);
        }
      })
      .catch(() => {
        if (!cancelled) setReady(true);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const setPinned = useCallback(async (value: boolean) => {
    try {
      await getCurrentWindow().setAlwaysOnTop(value);
      setPinnedState(value);
    } catch (err) {
      console.error("Failed to set always-on-top:", err);
    }
  }, []);

  const toggle = useCallback(async () => {
    await setPinned(!pinned);
  }, [pinned, setPinned]);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const mod = e.metaKey || e.ctrlKey;
      if (mod && e.shiftKey && (e.key === "T" || e.key === "t")) {
        e.preventDefault();
        void toggle();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [toggle]);

  return { pinned, ready, toggle, setPinned };
}
