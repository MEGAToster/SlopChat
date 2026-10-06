import { describe, expect, it } from "vitest";

import { hasRouting } from "./routing";

describe("hasRouting", () => {
  it("is false for an empty object and blank lists", () => {
    expect(hasRouting({})).toBe(false);
    expect(hasRouting({ order: [], only: [], ignore: [], sort: "" })).toBe(false);
  });

  it("is true when any routing hint is set", () => {
    expect(hasRouting({ only: ["anthropic"] })).toBe(true);
    expect(hasRouting({ allowFallbacks: false })).toBe(true);
    expect(hasRouting({ sort: "price" })).toBe(true);
    expect(hasRouting({ maxPrice: { prompt: 1 } })).toBe(true);
    expect(hasRouting({ maxPrice: { completion: 2 } })).toBe(true);
  });
});
