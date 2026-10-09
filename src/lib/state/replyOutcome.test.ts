import { describe, expect, it } from "vitest";

import { replyOutcome } from "./replyOutcome";

describe("replyOutcome", () => {
  it("stores a reply with content", () => {
    expect(replyOutcome({ content: "hi", hasDetail: false, placeholderVisible: true })).toEqual({
      kind: "store",
    });
  });

  it("keeps an errored placeholder visible with empty content", () => {
    expect(
      replyOutcome({ content: "", error: "boom", hasDetail: false, placeholderVisible: true }),
    ).toEqual({ kind: "keep", content: "" });
  });

  it("toasts when the errored placeholder is gone", () => {
    expect(
      replyOutcome({ content: "", error: "boom", hasDetail: false, placeholderVisible: false }),
    ).toEqual({ kind: "toast" });
  });

  it("keeps the no-text placeholder when there is detail", () => {
    expect(
      replyOutcome({ content: "", hasDetail: true, placeholderVisible: true }),
    ).toEqual({ kind: "keep", content: "_No text response._" });
  });

  it("removes a bare empty placeholder", () => {
    expect(replyOutcome({ content: "", hasDetail: false, placeholderVisible: true })).toEqual({
      kind: "remove",
    });
  });
});
