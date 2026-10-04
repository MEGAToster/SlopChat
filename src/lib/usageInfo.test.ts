import { describe, expect, it } from "vitest";

import { providerLabel, usageLine } from "./usageInfo";
import type { Usage } from "./types";

function usage(provider: string | null): Usage {
  return { tokensIn: 100, tokensOut: 5, cost: 0.001, provider };
}

describe("providerLabel", () => {
  it("uses the host name when present", () => {
    expect(providerLabel(usage("DeepInfra"), "openrouter")).toBe("DeepInfra");
  });

  it("is empty for OpenRouter without a host", () => {
    expect(providerLabel(usage(null), "openrouter")).toBe("");
  });

  it("labels Ollama as local", () => {
    expect(providerLabel(usage(null), "ollama")).toBe("local");
  });

  it("labels Antigravity", () => {
    expect(providerLabel(usage(null), "antigravity")).toBe("antigravity");
  });
});

describe("usageLine", () => {
  it("joins tokens, cost and host", () => {
    expect(usageLine(usage("DeepInfra"), "openrouter")).toBe(
      "↑100 ↓5 · $0.0010 · DeepInfra",
    );
  });
});
