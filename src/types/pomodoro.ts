export interface PomodoroSettings {
  work_minutes: number;
  short_break_min: number;
  long_break_min: number;
  rounds_per_cycle: number;
}

export interface DayStat {
  date: string;
  count: number;
}

export type TimerPhase = "work" | "short_break" | "long_break";

export interface TimerState {
  phase: TimerPhase;
  remainingMs: number;
  totalMs: number;
  running: boolean;
  currentRound: number;
}
