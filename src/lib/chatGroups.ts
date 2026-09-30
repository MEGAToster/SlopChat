import type { Chat } from "./types";

export type ChatGroup = { title: string; chats: Chat[] };

const ORDER = ["Older", "Previous 7 days", "Yesterday", "Today", "Pinned"];

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

  // Groups run oldest -> newest and each group is oldest -> newest, so new
  // chats keep appearing at the bottom of the sidebar.
  return ORDER.map((title) => ({
    title,
    chats: buckets[title].sort((a, b) => a.updatedAt - b.updatedAt),
  })).filter((group) => group.chats.length > 0);
}
