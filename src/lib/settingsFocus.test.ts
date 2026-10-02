import { describe, expect, it } from "vitest";

import { classifyElement, nextIndex } from "./settingsFocus";

describe("nextIndex", () => {
  it("wraps and handles empty or unknown", () => {
    expect(nextIndex(-1, 3, 1)).toBe(0);
    expect(nextIndex(-1, 3, -1)).toBe(2);
    expect(nextIndex(2, 3, 1)).toBe(0);
    expect(nextIndex(0, 3, -1)).toBe(2);
    expect(nextIndex(0, 0, 1)).toBe(-1);
  });
});

describe("classifyElement", () => {
  function element(tagName: string, type?: string): { tagName: string; type?: string } {
    return type === undefined ? { tagName } : { tagName, type };
  }

  it("classifies form controls", () => {
    expect(classifyElement(element("INPUT", "number"))).toBe("number");
    expect(classifyElement(element("INPUT", "checkbox"))).toBe("control");
    expect(classifyElement(element("INPUT", "text"))).toBe("text");
    expect(classifyElement(element("TEXTAREA"))).toBe("textarea");
    expect(classifyElement(element("SELECT"))).toBe("select");
    expect(classifyElement(element("SUMMARY"))).toBe("summary");
    expect(classifyElement(element("BUTTON"))).toBe("control");
    expect(classifyElement(null)).toBe("control");
  });
});
