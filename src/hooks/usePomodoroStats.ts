import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { DayStat } from "../types/pomodoro";

export function usePomodoroStats() {
  const [todayCount, setTodayCount] = useState(0);
  const [weeklyStats, setWeeklyStats] = useState<DayStat[]>([]);

  const refresh = useCallback(async () => {
    try {
      const [count, stats] = await Promise.all([
        invoke<number>("get_today_pomodoro_count"),
        invoke<DayStat[]>("get_weekly_pomodoro_stats"),
      ]);
      setTodayCount(count);
      setWeeklyStats(stats);
    } catch (err) {
      console.error("Failed to load pomodoro stats:", err);
    }
  }, []);

  useEffect(() => {
    refresh();
    const interval = setInterval(refresh, 30_000);
    return () => clearInterval(interval);
  }, [refresh]);

  return { todayCount, weeklyStats, refresh };
}
