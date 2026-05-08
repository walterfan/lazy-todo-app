import { describe, expect, it } from "vitest";
import { buildCronExpression, explainCronExpression } from "./cron";

describe("explainCronExpression", () => {
  it("explains a 5-field Unix cron expression and previews upcoming local runs", () => {
    const result = explainCronExpression("*/15 9-17 * * 1-5", new Date("2026-01-05T09:00:00"));

    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.format).toBe("unix5");
      expect(result.normalized).toBe("*/15 9-17 * * 1-5");
      expect(result.parserExpression).toBe("0 */15 9-17 * * 1-5");
      expect(result.description).toContain("Every 15 minutes");
      expect(result.nextRuns).toHaveLength(5);
      expect(result.nextRuns[0]).toContain("2026");
    }
  });

  it("explains a 6-field cron expression with seconds", () => {
    const result = explainCronExpression("30 */10 8 * * *", new Date("2026-01-05T08:00:00"));

    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.format).toBe("withSeconds");
      expect(result.normalized).toBe("30 */10 8 * * *");
      expect(result.parserExpression).toBe("30 */10 8 * * *");
      expect(result.description).toContain("30 seconds past the minute");
    }
  });

  it("returns a clear validation error for unsupported field counts", () => {
    expect(explainCronExpression("* * * *")).toEqual({
      ok: false,
      error: "Cron expression must have 5 fields or 6 fields with seconds.",
    });
  });
});

describe("buildCronExpression", () => {
  it("builds common schedules as 5-field or 6-field cron expressions", () => {
    expect(buildCronExpression({ kind: "everyMinutes", minutes: 5, includeSeconds: false })).toBe("*/5 * * * *");
    expect(buildCronExpression({ kind: "hourly", minute: 30, includeSeconds: true })).toBe("0 30 * * * *");
    expect(buildCronExpression({ kind: "daily", hour: 9, minute: 15, includeSeconds: false })).toBe("15 9 * * *");
    expect(buildCronExpression({ kind: "weekly", dayOfWeek: 1, hour: 8, minute: 0, includeSeconds: true })).toBe("0 0 8 * * 1");
    expect(buildCronExpression({ kind: "monthly", dayOfMonth: 10, hour: 7, minute: 45, includeSeconds: false })).toBe("45 7 10 * *");
  });
});
