import { describe, expect, it } from "vitest";

import { summarizeSteps } from "./steps";
import type { ToolStep } from "./types";

function step(name: string, status: ToolStep["status"] = "done"): ToolStep {
  return { id: `${name}-${Math.random()}`, name, args: {}, status, summary: "" };
}

describe("summarizeSteps", () => {
  it("returns nothing for no steps", () => {
    expect(summarizeSteps([])).toBe("");
  });

  it("pluralizes each category and omits zeros", () => {
    const steps = [
      step("web_search"),
      step("web_search"),
      step("web_fetch"),
      step("read_file"),
      step("read_file"),
    ];
    expect(summarizeSteps(steps)).toBe("2 searches · 1 page · 2 tools");
  });

  it("counts failures separately", () => {
    expect(summarizeSteps([step("web_fetch", "error"), step("web_search")])).toBe(
      "1 search · 1 page · 1 failed",
    );
  });
});
