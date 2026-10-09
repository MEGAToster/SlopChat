import type { Chat } from "./types";

// Title matches first, then message-content matches; the full `all` objects win
// so pinned/updatedAt stay current.
export function mergeMatches(all: Chat[], query: string, contentMatches: Chat[]): Chat[] {
  const text = query.trim().toLowerCase();
  if (!text) return all;

  const byId = new Map(all.map((chat) => [chat.id, chat]));
  const merged: Chat[] = [];
  const seen = new Set<string>();

  for (const chat of all) {
    if (chat.title.toLowerCase().includes(text) && !seen.has(chat.id)) {
      seen.add(chat.id);
      merged.push(chat);
    }
  }

  for (const match of contentMatches) {
    const chat = byId.get(match.id);
    if (chat && !seen.has(chat.id)) {
      seen.add(chat.id);
      merged.push(chat);
    }
  }

  return merged;
}
