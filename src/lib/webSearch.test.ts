import { describe, expect, it } from "vitest";

import { effectiveWebMode, nextWebOverride, paidSearchWarning } from "./webSearch";

describe("effectiveWebMode", () => {
  it("forces off when tool calling is off", () => {
    expect(effectiveWebMode(false, "deep", "deep")).toBe("off");
  });

  it("lets the chat override win over the configured mode", () => {
    expect(effectiveWebMode(true, "deep", "auto")).toBe("deep");
    expect(effectiveWebMode(true, "off", "deep")).toBe("off");
  });

  it("falls back to the configured mode, then auto", () => {
    expect(effectiveWebMode(true, null, "auto")).toBe("auto");
    expect(effectiveWebMode(true, null, undefined)).toBe("auto");
  });
});

describe("nextWebOverride", () => {
  it("cycles off → auto → deep → off", () => {
    expect(nextWebOverride("off")).toBe("auto");
    expect(nextWebOverride("auto")).toBe("deep");
    expect(nextWebOverride("deep")).toBe("off");
  });
});

describe("paidSearchWarning", () => {
  function search(baseUrl: string, providers: string[]) {
    return { providers, readerFallback: true, searxng: { baseUrl, minIntervalMs: 700 } };
  }

  it("is null when SearXNG is configured and first", () => {
    expect(paidSearchWarning(search("http://localhost:8080", ["searxng", "exa"]))).toBeNull();
  });

  it("warns when SearXNG has no URL or is not first", () => {
    expect(paidSearchWarning(search("", ["searxng", "exa"]))).toContain("paid search");
    expect(paidSearchWarning(search("http://x", ["exa", "searxng"]))).toContain("paid search");
  });
});
