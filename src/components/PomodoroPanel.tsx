import { usePomodoro } from "../hooks/usePomodoro";
import { usePomodoroStats } from "../hooks/usePomodoroStats";
import { PomodoroRing } from "./PomodoroRing";
import { PomodoroControls } from "./PomodoroControls";
import { PomodoroSettings } from "./PomodoroSettings";
import { PomodoroStats } from "./PomodoroStats";
import { PomodoroAlert } from "./PomodoroAlert";
import { PomodoroMilestones } from "./PomodoroMilestones";
import { useTranslation } from "react-i18next";

export function PomodoroPanel() {
  const { timer, settings, start, pause, reset, skip, saveSettings, updateMilestoneStatus, phaseLabel, alertPhase, dismissAlert } = usePomodoro();
  const { todayCount, weeklyStats } = usePomodoroStats();
  const { t } = useTranslation();

  return (
    <div className="pomodoro-panel">
      <PomodoroMilestones milestones={settings.milestones} onToggleStatus={updateMilestoneStatus} t={t} />
      <PomodoroRing timer={timer} phaseLabel={t(phaseLabel)} settings={settings} />
      <PomodoroControls running={timer.running} onStart={start} onPause={pause} onReset={reset} onSkip={skip} t={t} />
      <PomodoroStats todayCount={todayCount} weeklyStats={weeklyStats} t={t} />
      <PomodoroSettings settings={settings} onSave={saveSettings} t={t} />
      {alertPhase && <PomodoroAlert completedPhase={alertPhase} onDismiss={dismissAlert} t={t} />}
    </div>
  );
}
