import { describe, expect, it } from "vitest";

import { lastUsageOf, toChatMessage } from "./messages";
import type { StoredMessage } from "../types";

function stored(overrides: Partial<StoredMessage> = {}): StoredMessage {
  return {
    id: "m1",
    chatId: "c1",
    role: "assistant",
    content: "hi",
    model: "openrouter/x",
    tokensIn: 10,
    tokensOut: 2,
    cost: 0.5,
    provider: "DeepInfra",
    createdAt: 0,
    sources: null,
    attachments: [],
    ...overrides,
  };
}

describe("toChatMessage", () => {
  it("maps the stored provider through", () => {
    expect(toChatMessage(stored()).usage?.provider).toBe("DeepInfra");
  });

  it("keeps a missing provider null", () => {
    expect(toChatMessage(stored({ provider: null })).usage?.provider).toBeNull();
  });
});

describe("lastUsageOf", () => {
  it("reads the provider from the last usage-bearing message", () => {
    expect(lastUsageOf([stored()])?.provider).toBe("DeepInfra");
  });
});
