import { beforeEach, describe, expect, it, vi } from "vitest";

const { mockInitial, mockList } = vi.hoisted(() => ({
  mockInitial: vi.fn(),
  mockList: vi.fn(),
}));

vi.mock("../ipc", () => ({
  createChat: vi.fn(),
  deleteChat: vi.fn(),
  getChat: vi.fn(),
  initialChat: mockInitial,
  listChats: mockList,
  toggleChatPin: vi.fn(),
}));

import { chatState } from "./chatStore.svelte";
import { resumeOnShow } from "./chatHistory.svelte";

function view(id: string) {
  return {
    chat: { id, title: "", profile: "default", pinned: false, createdAt: 0, updatedAt: 0 },
    messages: [],
  };
}

beforeEach(() => {
  mockInitial.mockReset();
  mockList.mockReset().mockResolvedValue([]);
  chatState.activeChatId = null;
  chatState.messages = [];
  chatState.usage = null;
  chatState.error = null;
});

describe("resumeOnShow", () => {
  it("keeps the draft and defers while streaming", async () => {
    await resumeOnShow({ hasDraft: true, streaming: false });
    await resumeOnShow({ hasDraft: false, streaming: true });
    expect(mockInitial).not.toHaveBeenCalled();
  });

  it("opens the returned chat when it differs from the active one", async () => {
    mockInitial.mockResolvedValue(view("newest"));
    chatState.activeChatId = "old";
    await resumeOnShow({ hasDraft: false, streaming: false });
    expect(chatState.activeChatId).toBe("newest");
  });

  it("leaves the view untouched when the target is already active", async () => {
    mockInitial.mockResolvedValue(view("same"));
    chatState.activeChatId = "same";
    await resumeOnShow({ hasDraft: false, streaming: false });
    expect(chatState.activeChatId).toBe("same");
  });

  it("blanks the view for new-chat mode", async () => {
    mockInitial.mockResolvedValue(null);
    chatState.activeChatId = "old";
    await resumeOnShow({ hasDraft: false, streaming: false });
    expect(chatState.activeChatId).toBeNull();
    expect(chatState.messages).toEqual([]);
  });
});
