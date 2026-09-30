import { describe, expect, it, vi } from "vitest";

import { MarkdownStream, splitBlocks } from "./stream";

describe("splitBlocks", () => {
  it("splits on blank lines", () => {
    expect(splitBlocks("a\n\nb")).toEqual(["a", "b"]);
  });

  it("keeps blank lines inside fenced code blocks", () => {
    expect(splitBlocks("```\na\n\nb\n```")).toEqual(["```\na\n\nb\n```"]);
  });
});

describe("MarkdownStream", () => {
  it("renders every block once on the first update", () => {
    const render = vi.fn((source: string) => `<${source}>`);
    new MarkdownStream(render).update("one\n\ntwo");
    expect(render).toHaveBeenCalledTimes(2);
  });

  it("only re-renders the last block while it grows", () => {
    const render = vi.fn((source: string) => `<${source}>`);
    const stream = new MarkdownStream(render);
    stream.update("one\n\ntwo");

    render.mockClear();
    stream.update("one\n\ntwo more");
    expect(render).toHaveBeenCalledTimes(1);
    expect(render).toHaveBeenLastCalledWith("two more");
  });

  it("renders only the new block when a finished block is appended", () => {
    const render = vi.fn((source: string) => `<${source}>`);
    const stream = new MarkdownStream(render);
    stream.update("one\n\ntwo more");

    render.mockClear();
    stream.update("one\n\ntwo more\n\nthree");
    expect(render).toHaveBeenCalledTimes(1);
    expect(render).toHaveBeenLastCalledWith("three");
  });
});
