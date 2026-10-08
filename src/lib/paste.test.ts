import { describe, expect, it } from "vitest";

import { PASTE_AS_FILE_HARD_CAP, shouldPasteAsFile } from "./paste";

describe("shouldPasteAsFile", () => {
  it("never converts when the limit is zero", () => {
    expect(shouldPasteAsFile("x".repeat(10_000), 0)).toBe(false);
  });

  it("keeps text below the limit inline", () => {
    expect(shouldPasteAsFile("hello", 6)).toBe(false);
  });

  it("converts at and above the limit", () => {
    expect(shouldPasteAsFile("hello!", 6)).toBe(true);
    expect(shouldPasteAsFile("a".repeat(20), 6)).toBe(true);
  });

  it("converts huge pastes even when the limit is zero", () => {
    expect(shouldPasteAsFile("a".repeat(PASTE_AS_FILE_HARD_CAP), 0)).toBe(true);
    expect(shouldPasteAsFile("a".repeat(PASTE_AS_FILE_HARD_CAP - 1), 0)).toBe(false);
  });
});
