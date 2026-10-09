import type { StoredMessage, Usage } from "../types";
import type { ChatMessage } from "./chatStore.svelte";

export function toChatMessage(stored: StoredMessage): ChatMessage {
  const hasUsage =
    stored.tokensIn !== null || stored.tokensOut !== null || stored.cost !== null;
  return {
    id: stored.id,
    role: stored.role === "user" ? "user" : "assistant",
    content: stored.content,
    model: stored.model ?? undefined,
    attachments: stored.attachments,
    steps: [],
    sources: stored.sources ?? [],
    usage: hasUsage
      ? {
          tokensIn: stored.tokensIn ?? 0,
          tokensOut: stored.tokensOut ?? 0,
          cost: stored.cost,
          provider: stored.provider,
        }
      : undefined,
  };
}

export function lastUsageOf(messages: StoredMessage[]): Usage | null {
  for (let index = messages.length - 1; index >= 0; index -= 1) {
    const message = messages[index];
    if (message.tokensIn !== null || message.tokensOut !== null || message.cost !== null) {
      return {
        tokensIn: message.tokensIn ?? 0,
        tokensOut: message.tokensOut ?? 0,
        cost: message.cost,
        provider: message.provider,
      };
    }
  }
  return null;
}
