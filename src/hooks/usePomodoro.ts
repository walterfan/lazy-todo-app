import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { sendNotification, isPermissionGranted, requestPermission } from "@tauri-apps/plugin-notification";
import type {
  PomodoroMilestone,
  PomodoroMilestoneStatus,
  PomodoroSettings,
  TimerPhase,
  TimerState,
} from "../types/pomodoro";

const DEFAULT_SETTINGS: PomodoroSettings = {
  work_minutes: 25,
  short_break_min: 5,
  long_break_min: 15,
  rounds_per_cycle: 4,
  milestones: [],
};

function normalizeMilestones(milestones: PomodoroMilestone[]): PomodoroMilestone[] {
  return milestones
    .map((milestone) => ({
      name: milestone.name.trim(),
      deadline: milestone.deadline,
      status: milestone.status ?? "active",
    }))
    .filter((milestone) => milestone.name && milestone.deadline)
    .slice(0, 3);
}

function normalizeSettings(settings: PomodoroSettings): PomodoroSettings {
  return {
    ...settings,
    milestones: normalizeMilestones(settings.milestones ?? []),
  };
}

function phaseMs(phase: TimerPhase, settings: PomodoroSettings): number {
  switch (phase) {
    case "work": return settings.work_minutes * 60_000;
    case "short_break": return settings.short_break_min * 60_000;
    case "long_break": return settings.long_break_min * 60_000;
  }
}

function phaseLabel(phase: TimerPhase): string {
  switch (phase) {
    case "work": return "Working";
    case "short_break": return "Short Break";
    case "long_break": return "Long Break";
  }
}

interface TransitionOptions {
  recordSession?: boolean;
  notifyUser?: boolean;
  showAlert?: boolean;
}

export function usePomodoro() {
  const [settings, setSettings] = useState<PomodoroSettings>(DEFAULT_SETTINGS);
  const [timer, setTimer] = useState<TimerState>({
    phase: "work",
    remainingMs: DEFAULT_SETTINGS.work_minutes * 60_000,
    totalMs: DEFAULT_SETTINGS.work_minutes * 60_000,
    running: false,
    currentRound: 1,
  });

  const [alertPhase, setAlertPhase] = useState<TimerPhase | null>(null);

  const endTimeRef = useRef<number | null>(null);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    invoke<PomodoroSettings>("get_pomodoro_settings").then((s) => {
      const normalized = normalizeSettings(s);
      setSettings(normalized);
      setTimer((prev) => {
        if (!prev.running) {
          const ms = phaseMs(prev.phase, normalized);
          return { ...prev, remainingMs: ms, totalMs: ms };
        }
        return prev;
      });
    }).catch(console.error);
  }, []);

  const notify = useCallback(async (title: string, body: string) => {
    try {
      let granted = await isPermissionGranted();
      if (!granted) {
        const perm = await requestPermission();
        granted = perm === "granted";
      }
      if (granted) sendNotification({ title, body });
    } catch {
      // notifications unavailable
    }
  }, []);

  const updateTrayTooltip = useCallback((phase: TimerPhase, remainingMs: number, running: boolean) => {
    if (!running) {
      invoke("update_tray_tooltip", { text: "Lazy Todo App" }).catch(() => {});
      return;
    }
    const mins = Math.floor(remainingMs / 60_000);
    const secs = Math.floor((remainingMs % 60_000) / 1000);
    const mm = String(mins).padStart(2, "0");
    const ss = String(secs).padStart(2, "0");
    const icon = phase === "work" ? "🍅" : "☕";
    invoke("update_tray_tooltip", { text: `${icon} ${phaseLabel(phase)} ${mm}:${ss}` }).catch(() => {});
  }, []);

  const showWindow = useCallback(async () => {
    try {
      const win = getCurrentWindow();
      await win.show();
      await win.setFocus();
    } catch {
      // window API unavailable
    }
  }, []);

  const transitionToNext = useCallback((
    currentPhase: TimerPhase,
    currentRound: number,
    settingsRef: PomodoroSettings,
    options: TransitionOptions = {},
  ) => {
    const {
      recordSession = true,
      notifyUser = true,
      showAlert = true,
    } = options;
    let nextPhase: TimerPhase;
    let nextRound = currentRound;

    if (currentPhase === "work") {
      if (recordSession) {
        invoke("record_pomodoro_session", { durationMin: settingsRef.work_minutes }).catch(console.error);
      }
      if (currentRound >= settingsRef.rounds_per_cycle) {
        nextPhase = "long_break";
        nextRound = 1;
        if (notifyUser) {
          notify("Great job!", "Time for a long break.");
        }
      } else {
        nextPhase = "short_break";
        if (notifyUser) {
          notify("Pomodoro Complete", "Time for a short break!");
        }
      }
    } else {
      nextPhase = "work";
      if (currentPhase === "long_break") {
        nextRound = 1;
      } else {
        nextRound = currentRound + 1;
      }
      if (notifyUser) {
        notify("Break Over", "Ready to focus!");
      }
    }

    if (showAlert) {
      setAlertPhase(currentPhase);
      showWindow();
    } else {
      setAlertPhase(null);
    }

    const ms = phaseMs(nextPhase, settingsRef);
    setTimer({ phase: nextPhase, remainingMs: ms, totalMs: ms, running: false, currentRound: nextRound });
    endTimeRef.current = null;
    if (intervalRef.current) { clearInterval(intervalRef.current); intervalRef.current = null; }
    updateTrayTooltip(nextPhase, ms, false);
  }, [notify, updateTrayTooltip, showWindow]);

  const tick = useCallback(() => {
    if (!endTimeRef.current) return;
    const remaining = Math.max(0, endTimeRef.current - Date.now());
    setTimer((prev) => {
      if (remaining <= 0) {
        transitionToNext(prev.phase, prev.currentRound, settings);
        return prev;
      }
      updateTrayTooltip(prev.phase, remaining, true);
      return { ...prev, remainingMs: remaining };
    });
  }, [settings, transitionToNext, updateTrayTooltip]);

  const start = useCallback(() => {
    setTimer((prev) => {
      endTimeRef.current = Date.now() + prev.remainingMs;
      if (intervalRef.current) clearInterval(intervalRef.current);
      intervalRef.current = setInterval(tick, 200);
      return { ...prev, running: true };
    });
  }, [tick]);

  const pause = useCallback(() => {
    if (intervalRef.current) { clearInterval(intervalRef.current); intervalRef.current = null; }
    setTimer((prev) => {
      const remaining = endTimeRef.current ? Math.max(0, endTimeRef.current - Date.now()) : prev.remainingMs;
      endTimeRef.current = null;
      updateTrayTooltip(prev.phase, remaining, false);
      return { ...prev, remainingMs: remaining, running: false };
    });
  }, [updateTrayTooltip]);

  const reset = useCallback(() => {
    if (intervalRef.current) { clearInterval(intervalRef.current); intervalRef.current = null; }
    endTimeRef.current = null;
    setTimer((prev) => {
      const ms = phaseMs(prev.phase, settings);
      updateTrayTooltip(prev.phase, ms, false);
      return { ...prev, remainingMs: ms, totalMs: ms, running: false };
    });
  }, [settings, updateTrayTooltip]);

  const skip = useCallback(() => {
    transitionToNext(timer.phase, timer.currentRound, settings, {
      recordSession: false,
      notifyUser: false,
      showAlert: false,
    });
  }, [settings, timer.currentRound, timer.phase, transitionToNext]);

  const saveSettings = useCallback(async (newSettings: PomodoroSettings) => {
    const normalized = normalizeSettings(newSettings);
    await invoke("save_pomodoro_settings", { settings: normalized });
    setSettings(normalized);
    setTimer((prev) => {
      if (!prev.running) {
        const ms = phaseMs(prev.phase, normalized);
        return { ...prev, remainingMs: ms, totalMs: ms };
      }
      return prev;
    });
  }, []);

  const updateMilestoneStatus = useCallback(async (index: number, status: PomodoroMilestoneStatus) => {
    const nextMilestones = settings.milestones.map((milestone, milestoneIndex) =>
      milestoneIndex === index ? { ...milestone, status } : milestone,
    );

    await saveSettings({
      ...settings,
      milestones: nextMilestones,
    });
  }, [saveSettings, settings]);

  useEffect(() => {
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, []);

  const dismissAlert = useCallback(() => setAlertPhase(null), []);

  return {
    timer,
    settings,
    start,
    pause,
    reset,
    skip,
    saveSettings,
    updateMilestoneStatus,
    phaseLabel: phaseLabel(timer.phase),
    alertPhase,
    dismissAlert,
  };
}
