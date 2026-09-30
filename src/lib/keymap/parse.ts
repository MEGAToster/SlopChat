const MODIFIER_ORDER = ["Ctrl", "Alt", "Shift", "Meta"] as const;

const MODIFIER_ALIASES: Record<string, string> = {
  ctrl: "Ctrl",
  control: "Ctrl",
  alt: "Alt",
  option: "Alt",
  shift: "Shift",
  meta: "Meta",
  cmd: "Meta",
  command: "Meta",
  super: "Meta",
};

const KEY_ALIASES: Record<string, string> = {
  " ": "Space",
  space: "Space",
  arrowup: "Up",
  up: "Up",
  arrowdown: "Down",
  down: "Down",
  arrowleft: "Left",
  left: "Left",
  arrowright: "Right",
  right: "Right",
  escape: "Esc",
  esc: "Esc",
  return: "Enter",
  enter: "Enter",
  tab: "Tab",
};

function canonicalKey(raw: string): string {
  const alias = KEY_ALIASES[raw.toLowerCase()];
  if (alias) return alias;
  return raw.length === 1 ? raw.toUpperCase() : raw;
}

export function parseBinding(binding: string): string {
  const modifiers = new Set<string>();
  let key = "";

  for (const part of binding.split("+")) {
    const trimmed = part.trim();
    if (!trimmed) continue;
    const modifier = MODIFIER_ALIASES[trimmed.toLowerCase()];
    if (modifier) modifiers.add(modifier);
    else key = canonicalKey(trimmed);
  }

  const ordered = MODIFIER_ORDER.filter((modifier) => modifiers.has(modifier));
  return [...ordered, key].filter(Boolean).join("+");
}

export function eventToBinding(event: KeyboardEvent): string {
  const modifiers: string[] = [];
  if (event.ctrlKey) modifiers.push("Ctrl");
  if (event.altKey) modifiers.push("Alt");
  if (event.shiftKey) modifiers.push("Shift");
  if (event.metaKey) modifiers.push("Meta");
  return [...modifiers, canonicalKey(event.key)].join("+");
}

export function matches(binding: string, event: KeyboardEvent): boolean {
  return parseBinding(binding) === eventToBinding(event);
}

export function matchesAny(bindings: string[] | undefined, event: KeyboardEvent): boolean {
  return !!bindings?.some((binding) => matches(binding, event));
}
