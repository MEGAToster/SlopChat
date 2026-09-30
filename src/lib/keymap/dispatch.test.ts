import { describe, expect, it } from "vitest";

import { DEFAULT_KEYBINDS } from "./defaults";
import { resolve, type KeymapState } from "./dispatch";

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

function state(overrides: Partial<KeymapState> = {}): KeymapState {
  return { composerEmpty: true, chatHasMessages: false, ...overrides };
}

const bindings = DEFAULT_KEYBINDS;

describe("composer arrows respect the empty rule", () => {
  it("edits the last message only when empty with messages", () => {
    expect(
      resolve("composer", key({ key: "ArrowUp" }), bindings, state({ chatHasMessages: true })),
    ).toBe("editLast");
  });

  it("goes to the previous chat when empty in a blank chat", () => {
    expect(resolve("composer", key({ key: "ArrowUp" }), bindings, state())).toBe("prevChat");
  });

  it("does nothing when the composer has text", () => {
    expect(
      resolve("composer", key({ key: "ArrowUp" }), bindings, state({ composerEmpty: false })),
    ).toBeNull();
    expect(
      resolve("composer", key({ key: "ArrowDown" }), bindings, state({ composerEmpty: false })),
    ).toBeNull();
  });
});

describe("context handling", () => {
  it("ignores plain j/k while composing", () => {
    expect(resolve("composer", key({ key: "j" }), bindings, state())).toBeNull();
    expect(resolve("composer", key({ key: "k" }), bindings, state())).toBeNull();
  });

  it("navigates the sidebar with j/k and Ctrl+J/K", () => {
    expect(resolve("sidebar", key({ key: "j" }), bindings, state())).toBe("sidebarDown");
    expect(resolve("sidebar", key({ key: "k" }), bindings, state())).toBe("sidebarUp");
    expect(resolve("sidebar", key({ key: "j", ctrlKey: true }), bindings, state())).toBe(
      "sidebarDown",
    );
    expect(resolve("sidebar", key({ key: "Enter" }), bindings, state())).toBe("openChat");
  });

  it("routes Esc by context", () => {
    expect(resolve("editing", key({ key: "Escape" }), bindings, state())).toBe("cancelEdit");
    expect(resolve("composer", key({ key: "Escape" }), bindings, state())).toBe("hide");
    expect(resolve("palette", key({ key: "Escape" }), bindings, state())).toBe("hide");
  });
});

describe("configured bindings", () => {
  it("resolves default actions", () => {
    expect(resolve("composer", key({ key: "h", ctrlKey: true }), bindings, state())).toBe(
      "focusSidebar",
    );
    expect(resolve("composer", key({ key: "p", ctrlKey: true }), bindings, state())).toBe(
      "palette",
    );
    expect(
      resolve("composer", key({ key: "p", ctrlKey: true, shiftKey: true }), bindings, state()),
    ).toBe("sticky");
  });

  it("follows a rebound keybind", () => {
    const rebound = { ...bindings, palette: ["Ctrl+Space"] };
    expect(resolve("composer", key({ key: "p", ctrlKey: true }), rebound, state())).toBeNull();
    expect(resolve("composer", key({ key: " ", ctrlKey: true }), rebound, state())).toBe("palette");
  });
});
