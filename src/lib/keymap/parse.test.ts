import { describe, expect, it } from "vitest";

import { eventToBinding, matches, parseBinding } from "./parse";

function key(init: Partial<KeyboardEvent>): KeyboardEvent {
  return {
    key: "",
    ctrlKey: false,
    altKey: false,
    shiftKey: false,
    metaKey: false,
    ...init,
  } as KeyboardEvent;
}

describe("parseBinding", () => {
  it("canonicalizes modifiers and keys", () => {
    expect(parseBinding("Ctrl+P")).toBe("Ctrl+P");
    expect(parseBinding("ctrl+shift+p")).toBe("Ctrl+Shift+P");
    expect(parseBinding("Alt+Space")).toBe("Alt+Space");
    expect(parseBinding("Shift+Tab")).toBe("Shift+Tab");
    expect(parseBinding("Escape")).toBe("Esc");
    expect(parseBinding("ArrowUp")).toBe("Up");
    expect(parseBinding("j")).toBe("J");
  });
});

describe("matches", () => {
  it("matches equivalent events regardless of order or case", () => {
    expect(matches("Ctrl+P", key({ key: "p", ctrlKey: true }))).toBe(true);
    expect(matches("Ctrl+Shift+P", key({ key: "P", ctrlKey: true, shiftKey: true }))).toBe(true);
    expect(matches("Ctrl+P", key({ key: "P", ctrlKey: true, shiftKey: true }))).toBe(false);
    expect(matches("Alt+Space", key({ key: " ", altKey: true }))).toBe(true);
    expect(eventToBinding(key({ key: " ", altKey: true }))).toBe("Alt+Space");
  });

  it("treats WebKitGTK's ISO_Left_Tab as Shift+Tab", () => {
    expect(matches("Shift+Tab", key({ key: "ISO_Left_Tab", shiftKey: true }))).toBe(true);
    expect(eventToBinding(key({ key: "ISO_Left_Tab", shiftKey: true }))).toBe("Shift+Tab");
  });

  it("falls back to code/keyCode when WebKitGTK reports key Unidentified", () => {
    const event = key({ key: "Unidentified", code: "Tab", keyCode: 9, shiftKey: true });
    expect(eventToBinding(event)).toBe("Shift+Tab");
    expect(matches("Shift+Tab", event)).toBe(true);
  });
});
