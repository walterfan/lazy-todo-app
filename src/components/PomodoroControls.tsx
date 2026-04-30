import type { Translator } from "../i18n";

interface PomodoroControlsProps {
  running: boolean;
  onStart: () => void;
  onPause: () => void;
  onReset: () => void;
  onSkip: () => void;
  t: Translator;
}

export function PomodoroControls({ running, onStart, onPause, onReset, onSkip, t }: PomodoroControlsProps) {
  return (
    <div className="pomodoro-controls">
      {running ? (
        <button className="pomo-btn pomo-btn-pause" onClick={onPause}>{t("pause")}</button>
      ) : (
        <button className="pomo-btn pomo-btn-start" onClick={onStart}>{t("start")}</button>
      )}
      <button className="pomo-btn pomo-btn-reset" onClick={onSkip}>{t("skip")}</button>
      <button className="pomo-btn pomo-btn-reset" onClick={onReset}>{t("reset")}</button>
    </div>
  );
}
