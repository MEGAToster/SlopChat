import { describe, expect, it } from "vitest";

import { composeModelRef, isModelOverride, shortModel, splitModelRef } from "./modelRef";

describe("splitModelRef", () => {
  it("splits provider, model and routing tag", () => {
    expect(splitModelRef("openrouter/anthropic/claude-sonnet:floor")).toEqual({
      provider: "openrouter",
      modelId: "anthropic/claude-sonnet",
      tag: ":floor",
    });
  });
});

describe("composeModelRef", () => {
  it("prefixes the provider and keeps the tag", () => {
    expect(composeModelRef("openrouter", "anthropic/claude", ":nitro")).toBe(
      "openrouter/anthropic/claude:nitro",
    );
  });
});

describe("shortModel", () => {
  it("uses the last path segment plus the tag", () => {
    expect(shortModel("openrouter/anthropic/claude-sonnet:floor")).toBe("claude-sonnet:floor");
  });
});

describe("isModelOverride", () => {
  it("is true only for a different picked model", () => {
    expect(isModelOverride("openrouter/b", "openrouter/a")).toBe(true);
    expect(isModelOverride("openrouter/a", "openrouter/a")).toBe(false);
    expect(isModelOverride(null, "openrouter/a")).toBe(false);
  });
});
