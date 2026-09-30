import { parseBinding } from "./parse";

const MAC =
  typeof navigator !== "undefined" && /Mac|iPhone|iPad/.test(navigator.platform || navigator.userAgent);

const GLYPHS: Record<string, string> = {
  Ctrl: "⌃",
  Alt: "⌥",
  Shift: "⇧",
  Meta: "⌘",
  Enter: "↵",
  Esc: "⎋",
  Tab: "⇥",
  Space: "␣",
  Up: "↑",
  Down: "↓",
  Left: "←",
  Right: "→",
};

export function formatBinding(binding: string | undefined): string {
  if (!binding) return "";
  const parts = parseBinding(binding).split("+");
  if (MAC) return parts.map((part) => GLYPHS[part] ?? part).join("");
  return parts.map((part) => (part === "Meta" ? "Cmd" : part)).join("+");
}

export function formatKeys(bindings: string[] | undefined): string {
  return formatBinding(bindings?.[0]);
}
