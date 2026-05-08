import { Parser } from "expr-eval";

export type CalculationResult =
  | { ok: true; value: number; display: string }
  | { ok: false; error: string };

const parser = new Parser({
  allowMemberAccess: false,
  operators: {
    assignment: false,
    comparison: false,
    concatenate: false,
    conditional: false,
    fndef: false,
    in: false,
    logical: false,
    random: false,
  },
});

function formatNumber(value: number): string {
  const normalized = Object.is(value, -0) ? 0 : Number(value.toPrecision(15));
  return normalized.toString();
}

export function evaluateCalculation(expression: string): CalculationResult {
  const trimmed = expression.trim();
  if (!trimmed) return { ok: false, error: "Expression is required." };

  try {
    const parsed = parser.parse(trimmed);
    const variables = parsed.variables();
    if (variables.length > 0) {
      return { ok: false, error: `Unknown symbol: ${variables[0]}.` };
    }

    const value = parsed.evaluate();
    if (typeof value !== "number" || !Number.isFinite(value)) {
      return { ok: false, error: "Result is not a finite number." };
    }

    return { ok: true, value, display: formatNumber(value) };
  } catch (err) {
    const message = err instanceof Error ? err.message : "invalid expression";
    return { ok: false, error: `Invalid expression: ${message}.` };
  }
}
