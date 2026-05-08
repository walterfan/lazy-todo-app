import { describe, expect, it } from "vitest";
import { evaluateCalculation } from "./calculator";

describe("evaluateCalculation", () => {
  it("calculates arithmetic with precedence, parentheses, and functions", () => {
    expect(evaluateCalculation("1 + 2 * (3 + 4)")).toEqual({
      ok: true,
      value: 15,
      display: "15",
    });
    expect(evaluateCalculation("sqrt(81) + max(1, 5)")).toEqual({
      ok: true,
      value: 14,
      display: "14",
    });
  });

  it("rounds floating point display noise without changing the numeric value", () => {
    const result = evaluateCalculation("0.1 + 0.2");

    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.value).toBeCloseTo(0.3);
      expect(result.display).toBe("0.3");
    }
  });

  it("returns validation errors for empty, unknown, or non-finite expressions", () => {
    expect(evaluateCalculation("")).toEqual({ ok: false, error: "Expression is required." });
    expect(evaluateCalculation("abc + 1")).toEqual({
      ok: false,
      error: "Unknown symbol: abc.",
    });
    expect(evaluateCalculation("1 / 0")).toEqual({
      ok: false,
      error: "Result is not a finite number.",
    });
  });
});
