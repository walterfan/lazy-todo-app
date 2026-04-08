import type { TimerState } from "../types/pomodoro";

interface PomodoroRingProps {
  timer: TimerState;
  phaseLabel: string;
  settings: { rounds_per_cycle: number };
}

const SIZE = 200;
const STROKE = 8;
const RADIUS = (SIZE - STROKE) / 2;
const CIRCUMFERENCE = 2 * Math.PI * RADIUS;

export function PomodoroRing({ timer, phaseLabel, settings }: PomodoroRingProps) {
  const progress = timer.totalMs > 0 ? timer.remainingMs / timer.totalMs : 1;
  const offset = CIRCUMFERENCE * (1 - progress);

  const mins = Math.floor(timer.remainingMs / 60_000);
  const secs = Math.floor((timer.remainingMs % 60_000) / 1000);
  const display = `${String(mins).padStart(2, "0")}:${String(secs).padStart(2, "0")}`;

  const phaseColor = timer.phase === "work" ? "var(--pomodoro-work)" : "var(--pomodoro-break)";

  return (
    <div className="pomodoro-ring-container">
      <svg width={SIZE} height={SIZE} className="pomodoro-ring-svg">
        <circle
          className="pomodoro-ring-bg"
          cx={SIZE / 2}
          cy={SIZE / 2}
          r={RADIUS}
          strokeWidth={STROKE}
        />
        <circle
          className="pomodoro-ring-progress"
          cx={SIZE / 2}
          cy={SIZE / 2}
          r={RADIUS}
          strokeWidth={STROKE}
          stroke={phaseColor}
          strokeDasharray={CIRCUMFERENCE}
          strokeDashoffset={offset}
          strokeLinecap="round"
          transform={`rotate(-90 ${SIZE / 2} ${SIZE / 2})`}
        />
      </svg>
      <div className="pomodoro-ring-center">
        <span className="pomodoro-time">{display}</span>
        <span className="pomodoro-phase" style={{ color: phaseColor }}>{phaseLabel}</span>
        <span className="pomodoro-round">{timer.currentRound}/{settings.rounds_per_cycle}</span>
      </div>
    </div>
  );
}
