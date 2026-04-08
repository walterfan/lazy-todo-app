import { usePomodoro } from "../hooks/usePomodoro";
import { usePomodoroStats } from "../hooks/usePomodoroStats";
import { PomodoroRing } from "./PomodoroRing";
import { PomodoroControls } from "./PomodoroControls";
import { PomodoroSettings } from "./PomodoroSettings";
import { PomodoroStats } from "./PomodoroStats";
import { PomodoroAlert } from "./PomodoroAlert";

export function PomodoroPanel() {
  const { timer, settings, start, pause, reset, saveSettings, phaseLabel, alertPhase, dismissAlert } = usePomodoro();
  const { todayCount, weeklyStats } = usePomodoroStats();

  return (
    <div className="pomodoro-panel">
      <PomodoroRing timer={timer} phaseLabel={phaseLabel} settings={settings} />
      <PomodoroControls running={timer.running} onStart={start} onPause={pause} onReset={reset} />
      <PomodoroStats todayCount={todayCount} weeklyStats={weeklyStats} />
      <PomodoroSettings settings={settings} onSave={saveSettings} />
      {alertPhase && <PomodoroAlert completedPhase={alertPhase} onDismiss={dismissAlert} />}
    </div>
  );
}
