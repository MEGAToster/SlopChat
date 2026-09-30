import type { Chat } from "./types";

export type ChatGroup = { title: string; chats: Chat[] };

const ORDER = ["Pinned", "Today", "Yesterday", "Previous 7 days", "Older"];

export function groupChats(chats: Chat[], now = Date.now()): ChatGroup[] {
  const day = 86_400_000;
  const date = new Date(now);
  const startOfToday = new Date(date.getFullYear(), date.getMonth(), date.getDate()).getTime();

  const buckets: Record<string, Chat[]> = {
    Pinned: [],
    Today: [],
    Yesterday: [],
    "Previous 7 days": [],
    Older: [],
  };

  for (const chat of chats) {
    if (chat.pinned) buckets.Pinned.push(chat);
    else if (chat.updatedAt >= startOfToday) buckets.Today.push(chat);
    else if (chat.updatedAt >= startOfToday - day) buckets.Yesterday.push(chat);
    else if (chat.updatedAt >= startOfToday - 7 * day) buckets["Previous 7 days"].push(chat);
    else buckets.Older.push(chat);
  }

  return ORDER.map((title) => ({
    title,
    chats: buckets[title].sort((a, b) => b.updatedAt - a.updatedAt),
  })).filter((group) => group.chats.length > 0);
}
