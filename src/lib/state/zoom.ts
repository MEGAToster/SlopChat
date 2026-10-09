import { saveFont } from "../ipc";
import { clampFontSize, FONT_DEFAULT, scaleForSize } from "../fontSize";
import { settings } from "./settings.svelte";

// Reads the applied scale rather than settings(), which lags behind the async
// config reload; otherwise key repeat would keep computing the same step.
function currentSize(): number {
  const scale = parseFloat(getComputedStyle(document.documentElement).getPropertyValue("--scale"));
  if (Number.isFinite(scale) && scale > 0) return Math.round(scale * FONT_DEFAULT);
  return settings().font?.size ?? FONT_DEFAULT;
}

export function zoom(step: -1 | 0 | 1): void {
  const size = step === 0 ? FONT_DEFAULT : clampFontSize(currentSize() + step);
  document.documentElement.style.setProperty("--scale", String(scaleForSize(size)));
  void saveFont({ ...settings().font, size: size === FONT_DEFAULT ? undefined : size });
}
