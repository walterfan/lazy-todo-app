interface PomodoroControlsProps {
  running: boolean;
  onStart: () => void;
  onPause: () => void;
  onReset: () => void;
}

export function PomodoroControls({ running, onStart, onPause, onReset }: PomodoroControlsProps) {
  return (
    <div className="pomodoro-controls">
      {running ? (
        <button className="pomo-btn pomo-btn-pause" onClick={onPause}>Pause</button>
      ) : (
        <button className="pomo-btn pomo-btn-start" onClick={onStart}>Start</button>
      )}
      <button className="pomo-btn pomo-btn-reset" onClick={onReset}>Reset</button>
    </div>
  );
}
