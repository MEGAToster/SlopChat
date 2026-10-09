import { describe, expect, it } from "vitest";

import { mergeMatches } from "./filterChats";
import type { Chat } from "./types";

function chat(id: string, title: string, pinned = false): Chat {
  return {
    id,
    title,
    profile: "default",
    pinned,
    createdAt: 0,
    updatedAt: 0,
    model: null,
  };
}

describe("mergeMatches", () => {
  it("returns everything for an empty query", () => {
    const all = [chat("a", "Alpha")];
    expect(mergeMatches(all, "  ", [])).toBe(all);
  });

  it("puts title matches before content matches", () => {
    const all = [chat("a", "Alpha"), chat("b", "Beta"), chat("g", "Gamma")];
    const result = mergeMatches(all, "g", [chat("b", "Beta"), chat("g", "Gamma")]);
    expect(result.map((entry) => entry.id)).toEqual(["g", "b"]);
  });

  it("dedupes and prefers the full object from all", () => {
    const all = [chat("a", "Alpha", true), chat("b", "Beta")];
    const stale = { ...chat("a", "Alpha"), pinned: false };
    const result = mergeMatches(all, "alph", [stale]);
    expect(result.map((entry) => entry.id)).toEqual(["a"]);
    expect(result[0].pinned).toBe(true);
  });

  it("drops content matches that are no longer in the list", () => {
    const all = [chat("a", "Alpha")];
    expect(mergeMatches(all, "zzz", [chat("gone", "Gone")])).toEqual([]);
  });
});
