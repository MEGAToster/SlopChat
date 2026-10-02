import { describe, expect, it } from "vitest";

import { promoteNote } from "./promote";
import type { ToolStep } from "../types";

function step(note?: string): ToolStep {
  return { id: "1", name: "web_search", args: {}, status: "done", summary: "", note };
}

const long = "A".repeat(300);

describe("promoteNote", () => {
  it("promotes a long last note when the answer is empty and clears it", () => {
    const steps = [step("short note"), step(long)];
    const content = promoteNote("", steps);
    expect(content).toBe(long);
    expect(steps[1].note).toBeUndefined();
    expect(steps[0].note).toBe("short note");
  });

  it("does not promote a short note", () => {
    const steps = [step("too short")];
    expect(promoteNote("", steps)).toBe("");
    expect(steps[0].note).toBe("too short");
  });

  it("leaves a non-empty answer unchanged", () => {
    const steps = [step(long)];
    expect(promoteNote("real answer", steps)).toBe("real answer");
    expect(steps[0].note).toBe(long);
  });
});
