import type { Translator } from "../i18n";
import type { DayStat } from "../types/pomodoro";

interface PomodoroStatsProps {
  todayCount: number;
  weeklyStats: DayStat[];
  t: Translator;
}

const DAY_LABELS = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

function dayLabel(dateStr: string): string {
  const d = new Date(dateStr + "T00:00:00");
  return DAY_LABELS[d.getDay()];
}

const BAR_WIDTH = 28;
const BAR_GAP = 8;
const CHART_HEIGHT = 60;

export function PomodoroStats({ todayCount, weeklyStats, t }: PomodoroStatsProps) {
  const maxCount = Math.max(1, ...weeklyStats.map((s) => s.count));
  const chartWidth = weeklyStats.length * (BAR_WIDTH + BAR_GAP) - BAR_GAP;
  const todayStr = new Date().toISOString().slice(0, 10);

  return (
    <div className="pomo-stats">
      <div className="pomo-stats-today">
        {t("today")}: <strong>{todayCount}</strong> 🍅
      </div>
      {weeklyStats.length > 0 && (
        <div className="pomo-stats-chart">
          <svg width={chartWidth} height={CHART_HEIGHT + 20} className="pomo-bar-chart">
            {weeklyStats.map((stat, i) => {
              const barH = maxCount > 0 ? (stat.count / maxCount) * CHART_HEIGHT : 0;
              const x = i * (BAR_WIDTH + BAR_GAP);
              const isToday = stat.date === todayStr;
              return (
                <g key={stat.date}>
                  <rect
                    x={x}
                    y={CHART_HEIGHT - barH}
                    width={BAR_WIDTH}
                    height={Math.max(barH, 2)}
                    rx={4}
                    className={`pomo-bar ${isToday ? "pomo-bar-today" : ""}`}
                  />
                  {stat.count > 0 && (
                    <text x={x + BAR_WIDTH / 2} y={CHART_HEIGHT - barH - 4} textAnchor="middle" className="pomo-bar-count">
                      {stat.count}
                    </text>
                  )}
                  <text x={x + BAR_WIDTH / 2} y={CHART_HEIGHT + 14} textAnchor="middle" className="pomo-bar-label">
                    {dayLabel(stat.date)}
                  </text>
                </g>
              );
            })}
          </svg>
        </div>
      )}
    </div>
  );
}
