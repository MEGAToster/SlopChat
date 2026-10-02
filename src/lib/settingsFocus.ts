import type { SettingsFocus } from "./keymap/dispatch";

const NAVIGABLE = 'button, input, select, textarea, summary, [tabindex]:not([tabindex="-1"])';

type FocusableLike = {
  tagName: string;
  isContentEditable?: boolean;
  type?: string;
};

export function classifyElement(element: FocusableLike | null): SettingsFocus {
  if (!element) return "control";
  const tag = element.tagName.toLowerCase();
  if (tag === "textarea") return "textarea";
  if (tag === "select") return "select";
  if (tag === "summary") return "summary";
  if (tag === "input") {
    const type = element.type ?? "text";
    if (type === "number") return "number";
    if (type === "checkbox" || type === "radio" || type === "button" || type === "submit") {
      return "control";
    }
    return "text";
  }
  return "control";
}

export function nextIndex(current: number, count: number, delta: number): number {
  if (count === 0) return -1;
  if (current < 0) return delta >= 0 ? 0 : count - 1;
  return (current + delta + count) % count;
}

function navigable(root: ParentNode): HTMLElement[] {
  return Array.from(root.querySelectorAll<HTMLElement>(NAVIGABLE)).filter((element) => {
    if ((element as HTMLInputElement).disabled || element.hidden) return false;
    // Excludes controls inside a collapsed <details> or otherwise unrendered.
    return element.getClientRects().length > 0;
  });
}

function settingsRoot(): ParentNode | null {
  return document.querySelector(".settings");
}

export function moveSettingsFocus(delta: number): void {
  const root = settingsRoot();
  if (!root) return;
  const items = navigable(root);
  const current = items.indexOf(document.activeElement as HTMLElement);
  const target = items[nextIndex(current, items.length, delta)];
  if (target) {
    target.focus();
    target.scrollIntoView({ block: "nearest" });
  }
}

export function activateSettingsFocus(): void {
  (document.activeElement as HTMLElement | null)?.click();
}

function setSummaryOpen(summary: HTMLElement, open: boolean): void {
  const details = summary.parentElement as HTMLDetailsElement | null;
  if (details && "open" in details) {
    details.open = open;
    summary.focus();
  }
}

export function setFocusedSummary(open: boolean): void {
  const active = document.activeElement;
  if (active && active.tagName.toLowerCase() === "summary") {
    setSummaryOpen(active as HTMLElement, open);
  }
}
