import { describe, expect, it } from "vitest";

import type { Theme } from "../types";
import { resolveTheme } from "./apply";
import { BUILTIN_THEMES } from "./builtin";

const map = new Map(BUILTIN_THEMES.map((entry) => [entry.name, entry.theme as Theme]));
const base = resolveTheme(map, "default-dark");

function luminance(hex: string): number {
  const value = hex.replace("#", "");
  const channels = [0, 2, 4].map((index) => {
    const channel = Number.parseInt(value.slice(index, index + 2), 16) / 255;
    return channel <= 0.03928 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2];
}

function contrast(a: string, b: string): number {
  const first = luminance(a);
  const second = luminance(b);
  const [hi, lo] = first > second ? [first, second] : [second, first];
  return (hi + 0.05) / (lo + 0.05);
}

describe("builtin themes", () => {
  for (const info of BUILTIN_THEMES) {
    it(`${info.name} resolves every token and passes contrast`, () => {
      const resolved = resolveTheme(map, info.name);

      for (const key of Object.keys(base.colors)) {
        expect(resolved.colors[key], `${info.name} colors.${key}`).toBeTruthy();
      }
      for (const key of Object.keys(base.syntax)) {
        expect(resolved.syntax[key], `${info.name} syntax.${key}`).toBeTruthy();
      }
      for (const key of Object.keys(base.palette)) {
        expect(resolved.palette[key], `${info.name} palette.${key}`).toBeTruthy();
      }

      const bg = resolved.colors.bg;
      if (bg.startsWith("#")) {
        expect(contrast(resolved.colors.text, bg), `${info.name} text/bg`).toBeGreaterThanOrEqual(4.5);
        expect(
          contrast(resolved.colors["text-tertiary"], bg),
          `${info.name} tertiary/bg`,
        ).toBeGreaterThanOrEqual(3);
      }
    });
  }
});
