import { CronExpressionParser } from "cron-parser";
import cronstrue from "cronstrue";

export type CronFormat = "unix5" | "withSeconds";

export type CronExplainResult =
  | {
      ok: true;
      format: CronFormat;
      normalized: string;
      parserExpression: string;
      description: string;
      nextRuns: string[];
    }
  | { ok: false; error: string };

export type CronBuildInput =
  | { kind: "everyMinutes"; minutes: number; includeSeconds: boolean }
  | { kind: "hourly"; minute: number; includeSeconds: boolean }
  | { kind: "daily"; hour: number; minute: number; includeSeconds: boolean }
  | { kind: "weekly"; dayOfWeek: number; hour: number; minute: number; includeSeconds: boolean }
  | { kind: "monthly"; dayOfMonth: number; hour: number; minute: number; includeSeconds: boolean };

function normalizeCronExpression(expression: string): {
  format: CronFormat;
  normalized: string;
  parserExpression: string;
} {
  const normalized = expression.trim().replace(/\s+/g, " ");
  const fields = normalized ? normalized.split(" ") : [];

  if (fields.length === 5) {
    return {
      format: "unix5",
      normalized,
      parserExpression: `0 ${normalized}`,
    };
  }

  if (fields.length === 6) {
    return {
      format: "withSeconds",
      normalized,
      parserExpression: normalized,
    };
  }

  throw new Error("Cron expression must have 5 fields or 6 fields with seconds.");
}

function formatLocalDate(date: Date): string {
  return date.toLocaleString(undefined, {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false,
  });
}

export function explainCronExpression(expression: string, currentDate: Date = new Date()): CronExplainResult {
  try {
    const normalized = normalizeCronExpression(expression);
    const parsed = CronExpressionParser.parse(normalized.parserExpression, { currentDate });
    const nextRuns = Array.from({ length: 5 }, () => formatLocalDate(parsed.next().toDate()));
    const description = cronstrue.toString(normalized.normalized, {
      throwExceptionOnParseError: true,
      use24HourTimeFormat: true,
    });

    return {
      ok: true,
      ...normalized,
      description,
      nextRuns,
    };
  } catch (err) {
    return {
      ok: false,
      error: err instanceof Error ? err.message : "Invalid cron expression.",
    };
  }
}

function assertIntegerInRange(value: number, min: number, max: number, label: string): number {
  if (!Number.isInteger(value) || value < min || value > max) {
    throw new Error(`${label} must be between ${min} and ${max}.`);
  }
  return value;
}

function withOptionalSeconds(includeSeconds: boolean, fields: string[]): string {
  return includeSeconds ? ["0", ...fields].join(" ") : fields.join(" ");
}

export function buildCronExpression(input: CronBuildInput): string {
  switch (input.kind) {
    case "everyMinutes": {
      const minutes = assertIntegerInRange(input.minutes, 1, 59, "Minutes");
      return withOptionalSeconds(input.includeSeconds, [`*/${minutes}`, "*", "*", "*", "*"]);
    }
    case "hourly": {
      const minute = assertIntegerInRange(input.minute, 0, 59, "Minute");
      return withOptionalSeconds(input.includeSeconds, [minute.toString(), "*", "*", "*", "*"]);
    }
    case "daily": {
      const hour = assertIntegerInRange(input.hour, 0, 23, "Hour");
      const minute = assertIntegerInRange(input.minute, 0, 59, "Minute");
      return withOptionalSeconds(input.includeSeconds, [minute.toString(), hour.toString(), "*", "*", "*"]);
    }
    case "weekly": {
      const dayOfWeek = assertIntegerInRange(input.dayOfWeek, 0, 6, "Day of week");
      const hour = assertIntegerInRange(input.hour, 0, 23, "Hour");
      const minute = assertIntegerInRange(input.minute, 0, 59, "Minute");
      return withOptionalSeconds(input.includeSeconds, [
        minute.toString(),
        hour.toString(),
        "*",
        "*",
        dayOfWeek.toString(),
      ]);
    }
    case "monthly": {
      const dayOfMonth = assertIntegerInRange(input.dayOfMonth, 1, 31, "Day of month");
      const hour = assertIntegerInRange(input.hour, 0, 23, "Hour");
      const minute = assertIntegerInRange(input.minute, 0, 59, "Minute");
      return withOptionalSeconds(input.includeSeconds, [
        minute.toString(),
        hour.toString(),
        dayOfMonth.toString(),
        "*",
        "*",
      ]);
    }
  }
}
