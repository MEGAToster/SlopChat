import { describe, expect, it } from "vitest";

import { decodeBase64 } from "./bytes";

function base64(text: string): string {
  const bytes = new TextEncoder().encode(text);
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary);
}

describe("decodeBase64", () => {
  it("round-trips non-ASCII text", () => {
    for (const text of ["Grüße", "naïve café", "emoji 🎉🚀", "日本語"]) {
      expect(decodeBase64(base64(text))).toBe(text);
    }
  });

  it("returns an empty string for invalid input", () => {
    expect(decodeBase64("!!!not base64!!!")).toBe("");
  });
});
