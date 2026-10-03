import { describe, expect, it } from "vitest";

import { applyCitations } from "./citations";

const sources = [
  { n: 1, title: "Docs", url: "https://docs.rs" },
  { n: 2, title: "GitHub", url: "https://github.com" },
];

describe("applyCitations", () => {
  it("leaves html untouched when there are no sources", () => {
    expect(applyCitations("see [1]", [])).toBe("see [1]");
  });

  it("links in-range citations", () => {
    expect(applyCitations("see [1] and [2]", sources)).toBe(
      'see <a class="citation" href="https://docs.rs">[1]</a> and <a class="citation" href="https://github.com">[2]</a>',
    );
  });

  it("ignores citations inside code", () => {
    expect(applyCitations("<code>[1]</code> [1]", sources)).toBe(
      '<code>[1]</code> <a class="citation" href="https://docs.rs">[1]</a>',
    );
  });

  it("leaves out-of-range citations alone", () => {
    expect(applyCitations("see [9]", sources)).toBe("see [9]");
  });

  it("escapes quotes in a source URL", () => {
    const evil = [{ n: 1, title: "x", url: 'https://e.com/" onmouseover="alert(1)' }];
    expect(applyCitations("see [1]", evil)).toBe(
      'see <a class="citation" href="https://e.com/&quot; onmouseover=&quot;alert(1)">[1]</a>',
    );
  });
});
