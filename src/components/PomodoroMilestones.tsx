import type { Translator } from "../i18n";
import type { PomodoroMilestone, PomodoroMilestoneStatus } from "../types/pomodoro";

interface PomodoroMilestonesProps {
  milestones: PomodoroMilestone[];
  onToggleStatus: (index: number, status: PomodoroMilestoneStatus) => void | Promise<void>;
  t: Translator;
}

const DAY_MS = 24 * 60 * 60 * 1000;

function CheckIcon() {
  return (
    <svg viewBox="0 0 16 16" aria-hidden="true">
      <path
        d="M3.5 8.5 6.5 11.5 12.5 4.5"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.8"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

function CancelIcon() {
  return (
    <svg viewBox="0 0 16 16" aria-hidden="true">
      <path
        d="M4.5 4.5 11.5 11.5M11.5 4.5 4.5 11.5"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.8"
        strokeLinecap="round"
      />
    </svg>
  );
}

function RestoreIcon() {
  return (
    <svg viewBox="0 0 16 16" aria-hidden="true">
      <path
        d="M5 4H2v3M2.5 4.5A5.5 5.5 0 1 1 4.4 12.6"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.6"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

function startOfDay(date: Date): Date {
  return new Date(date.getFullYear(), date.getMonth(), date.getDate());
}

function formatRemainingDays(deadline: string, t: Translator): string {
  const target = new Date(`${deadline}T00:00:00`);

  if (Number.isNaN(target.getTime())) {
    return t("invalidDate");
  }

  const today = startOfDay(new Date());
  const targetDay = startOfDay(target);
  const diffDays = Math.round((targetDay.getTime() - today.getTime()) / DAY_MS);

  if (diffDays > 0) {
    return t(diffDays === 1 ? "dayLeft" : "daysLeft", { count: diffDays });
  }

  if (diffDays === 0) {
    return t("dueToday");
  }

  const overdueDays = Math.abs(diffDays);
  return t(overdueDays === 1 ? "dayOverdue" : "daysOverdue", { count: overdueDays });
}

function statusRank(status: PomodoroMilestoneStatus): number {
  switch (status) {
    case "active":
      return 0;
    case "completed":
      return 1;
    case "cancelled":
      return 2;
  }
}

function formatMilestoneStatus(milestone: PomodoroMilestone, t: Translator): string {
  switch (milestone.status) {
    case "completed":
      return t("completed");
    case "cancelled":
      return t("cancelled");
    default:
      return formatRemainingDays(milestone.deadline, t);
  }
}

export function PomodoroMilestones({ milestones, onToggleStatus, t }: PomodoroMilestonesProps) {
  const visibleMilestones = milestones
    .map((milestone, index) => ({ ...milestone, index }))
    .filter((milestone) => milestone.name && milestone.deadline)
    .sort((a, b) => statusRank(a.status) - statusRank(b.status) || a.deadline.localeCompare(b.deadline));

  return (
    <div className="pomo-milestones">
      <div className="pomo-milestones-title">{t("milestones")}</div>
      {visibleMilestones.length > 0 ? (
        <div className="pomo-milestones-list">
          {visibleMilestones.map((milestone) => (
            <div className={`pomo-milestone-card pomo-milestone-card-${milestone.status}`} key={`${milestone.name}-${milestone.deadline}-${milestone.index}`}>
              <div className="pomo-milestone-line">
                <div className="pomo-milestone-name">{milestone.name}</div>
                <div className="pomo-milestone-days">{formatMilestoneStatus(milestone, t)}</div>
                <div className="pomo-milestone-deadline">{milestone.deadline}</div>
                <div className="pomo-milestone-actions">
                  {milestone.status === "active" ? (
                    <>
                      <button
                        className="pomo-milestone-action pomo-milestone-action-complete"
                        type="button"
                        title={t("markMilestoneCompleted")}
                        aria-label={t("markMilestoneCompleted")}
                        onClick={() => {
                          void onToggleStatus(milestone.index, "completed");
                        }}
                      >
                        <CheckIcon />
                      </button>
                      <button
                        className="pomo-milestone-action pomo-milestone-action-cancel"
                        type="button"
                        title={t("cancelMilestone")}
                        aria-label={t("cancelMilestone")}
                        onClick={() => {
                          void onToggleStatus(milestone.index, "cancelled");
                        }}
                      >
                        <CancelIcon />
                      </button>
                    </>
                  ) : (
                    <button
                      className="pomo-milestone-action pomo-milestone-action-restore"
                      type="button"
                      title={t("restoreMilestone")}
                      aria-label={t("restoreMilestone")}
                      onClick={() => {
                        void onToggleStatus(milestone.index, "active");
                      }}
                    >
                      <RestoreIcon />
                    </button>
                  )}
                </div>
              </div>
            </div>
          ))}
        </div>
      ) : (
        <div className="pomo-milestones-empty">{t("noMilestonesConfigured")}</div>
      )}
    </div>
  );
}
