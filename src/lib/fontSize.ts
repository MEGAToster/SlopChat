export const FONT_MIN = 10;
export const FONT_MAX = 24;
export const FONT_DEFAULT = 14;

export function clampFontSize(size: number): number {
  return Math.min(FONT_MAX, Math.max(FONT_MIN, Math.round(size)));
}

export function scaleForSize(size: number): number {
  return size / FONT_DEFAULT;
}
