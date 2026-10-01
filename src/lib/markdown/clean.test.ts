import { describe, expect, it } from "vitest";

import { stripControlTokens } from "./clean";

describe("stripControlTokens", () => {
  it("removes DSML tool-call blocks but keeps surrounding prose", () => {
    const text = [
      "Here you go.",
      "<|DSML| calls>",
      '<|DSML| invoke name="web_search">',
      '<|DSML| parameter name="query" string="true">hi</|DSML| parameter>',
      "</|DSML| invoke>",
      "</|DSML| calls>",
      "Done.",
    ].join("\n");

    expect(stripControlTokens(text)).toBe("Here you go.\n\nDone.");
  });

  it("removes stray control tokens", () => {
    expect(stripControlTokens("a <|DSML| invoke name=\"x\"> b")).toBe("a  b");
  });

  it("leaves normal text untouched", () => {
    expect(stripControlTokens("plain **markdown**")).toBe("plain **markdown**");
  });
});
