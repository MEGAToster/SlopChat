import { describe, expect, it } from "vitest";

import { fuzzyMatch } from "./fuzzy";

describe("fuzzyMatch", () => {
  it("matches an empty query against anything", () => {
    expect(fuzzyMatch("", "anything")).not.toBeNull();
  });

  it("matches subsequences", () => {
    expect(fuzzyMatch("nwch", "New chat")).not.toBeNull();
    expect(fuzzyMatch("goto", "Toggle sticky")).toBeNull();
  });

  it("matches case-insensitively and prefers contiguous matches", () => {
    const contiguous = fuzzyMatch("chat", "New chat");
    const scattered = fuzzyMatch("chat", "cathartic thoughts");
    expect(contiguous).not.toBeNull();
    expect(scattered).not.toBeNull();
    expect(contiguous!.score).toBeGreaterThan(scattered!.score);
  });

  it("rejects queries that are not a subsequence", () => {
    expect(fuzzyMatch("xyz", "New chat")).toBeNull();
  });
});
