import { describe, expect, it } from "vitest";

import { DEFAULT_KEYBINDS } from "./defaults";
import { resolve, resolveSettings, type KeymapState } from "./dispatch";

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
  return {
    composerEmpty: true,
    chatHasMessages: false,
    streaming: false,
    sending: false,
    editing: false,
    ...overrides,
  };
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

  it("renames a chat with r in the sidebar", () => {
    expect(resolve("sidebar", key({ key: "r" }), bindings, state())).toBe("renameChat");
    expect(resolve("composer", key({ key: "r" }), bindings, state())).toBeNull();
  });

  it("routes Esc by state and context", () => {
    expect(
      resolve("composer", key({ key: "Escape" }), bindings, state({ editing: true })),
    ).toBe("cancelEdit");
    expect(resolve("composer", key({ key: "Escape" }), bindings, state())).toBe("hide");
    expect(resolve("palette", key({ key: "Escape" }), bindings, state())).toBe("paletteClose");
    expect(resolve("filter", key({ key: "Escape" }), bindings, state())).toBe("filterClose");
    expect(resolve("confirm", key({ key: "y" }), bindings, state())).toBe("confirmDelete");
    expect(resolve("confirm", key({ key: "n" }), bindings, state())).toBe("cancelDelete");
  });
});

describe("resolveSettings", () => {
  it("backs out on Escape", () => {
    expect(resolveSettings(key({ key: "Escape" }), "control")).toBe("settingsBack");
  });

  it("moves with Tab and Shift+Tab, including WebKitGTK's Unidentified", () => {
    expect(resolveSettings(key({ key: "Tab" }), "control")).toBe("settingsNext");
    expect(resolveSettings(key({ key: "Tab", shiftKey: true }), "control")).toBe("settingsPrev");
    expect(
      resolveSettings(
        key({ key: "Unidentified", code: "Tab", keyCode: 9, shiftKey: true }),
        "control",
      ),
    ).toBe("settingsPrev");
  });

  it("navigates with arrows except in native fields", () => {
    expect(resolveSettings(key({ key: "ArrowDown" }), "control")).toBe("settingsNext");
    expect(resolveSettings(key({ key: "ArrowUp" }), "control")).toBe("settingsPrev");
    expect(resolveSettings(key({ key: "ArrowDown" }), "textarea")).toBeNull();
    expect(resolveSettings(key({ key: "ArrowUp" }), "select")).toBeNull();
    expect(resolveSettings(key({ key: "ArrowDown" }), "number")).toBeNull();
  });

  it("expands or collapses only a focused summary", () => {
    expect(resolveSettings(key({ key: "ArrowRight" }), "summary")).toBe("settingsExpand");
    expect(resolveSettings(key({ key: "ArrowLeft" }), "summary")).toBe("settingsCollapse");
    expect(resolveSettings(key({ key: "ArrowRight" }), "control")).toBeNull();
  });

  it("activates controls and summaries, not fields", () => {
    expect(resolveSettings(key({ key: "Enter" }), "control")).toBe("settingsActivate");
    expect(resolveSettings(key({ key: " " }), "control")).toBe("settingsActivate");
    expect(resolveSettings(key({ key: "Enter" }), "text")).toBeNull();
    expect(resolveSettings(key({ key: " " }), "textarea")).toBeNull();
  });
});

describe("configured bindings", () => {
  it("resolves default actions", () => {
    expect(resolve("composer", key({ key: "h", ctrlKey: true }), bindings, state())).toBe(
      "toggleSidebar",
    );
    expect(resolve("composer", key({ key: "p", ctrlKey: true }), bindings, state())).toBe(
      "palette",
    );
    expect(
      resolve("composer", key({ key: "p", ctrlKey: true, shiftKey: true }), bindings, state()),
    ).toBe("sticky");
  });

  it("scrolls with Ctrl+U/D, and Ctrl+Q stops while streaming", () => {
    expect(resolve("composer", key({ key: "u", ctrlKey: true }), bindings, state())).toBe(
      "scrollUp",
    );
    expect(resolve("composer", key({ key: "d", ctrlKey: true }), bindings, state())).toBe(
      "scrollDown",
    );
    expect(
      resolve("composer", key({ key: "q", ctrlKey: true }), bindings, state({ streaming: true })),
    ).toBe("stop");
  });

  it("cycles profiles regardless of composer content", () => {
    const event = key({ key: "Unidentified", code: "Tab", keyCode: 9, shiftKey: true });
    expect(resolve("composer", event, bindings, state({ composerEmpty: false }))).toBe(
      "cycleProfile",
    );
    expect(resolve("composer", event, bindings, state())).toBe("cycleProfile");
  });

  it("toggles the sidebar from both contexts", () => {
    expect(resolve("composer", key({ key: "ArrowLeft" }), bindings, state())).toBe(
      "toggleSidebar",
    );
    expect(resolve("sidebar", key({ key: "ArrowLeft" }), bindings, state())).toBe(
      "toggleSidebar",
    );
  });

  it("follows a rebound keybind", () => {
    const rebound = { ...bindings, palette: ["Ctrl+Space"] };
    expect(resolve("composer", key({ key: "p", ctrlKey: true }), rebound, state())).toBeNull();
    expect(resolve("composer", key({ key: " ", ctrlKey: true }), rebound, state())).toBe("palette");
  });
});
