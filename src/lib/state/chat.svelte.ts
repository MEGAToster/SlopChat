import { appendMessage, getChat, readAttachment, rewindTo, saveAttachment } from "../ipc";
import type { Attachment, ContentPart, NewAttachment, RequestMessage } from "../types";
import { base64Of, clearPending, pending } from "./attachments.svelte";
import { ensureChat, refreshChats } from "./chatHistory.svelte";
import { adoptWebDraft, chatState, type SendOptions } from "./chatStore.svelte";
import { toChatMessage } from "./messages";
import { run as runStream, stop as stopStream } from "./stream.svelte";

export * from "./chatStore.svelte";
export * from "./chatHistory.svelte";

export async function submit(text: string, options: SendOptions): Promise<void> {
  const content = text.trim();
  const attachments = pending();
  if (chatState.streaming || (!content && attachments.length === 0)) return;

  const chatId = await ensureChat(options.profileName);
  adoptWebDraft(chatId);

  const saved: NewAttachment[] = [];
  for (const item of attachments) {
    const stored = await saveAttachment(item.name, item.mime, base64Of(item.dataUrl));
    saved.push({ id: stored.id, path: stored.path, mime: stored.mime ?? item.mime });
  }

  const stored = await appendMessage({ chatId, role: "user", content, attachments: saved });
  chatState.messages.push(toChatMessage(stored));
  clearPending();
  await refreshChats();

  await streamReply(chatId, options);
}

export async function regenerate(
  messageId: string,
  text: string,
  options: SendOptions,
): Promise<void> {
  const content = text.trim();
  if (chatState.streaming || !chatState.activeChatId || !content) return;

  const chatId = chatState.activeChatId;
  await rewindTo(chatId, messageId, content);
  const view = await getChat(chatId);
  chatState.messages = view ? view.messages.map(toChatMessage) : [];
  await refreshChats();

  await streamReply(chatId, options);
}

export async function stop(): Promise<void> {
  if (chatState.streamingChatId) await stopStream(chatState.streamingChatId);
}

async function messageContent(
  text: string,
  attachments: Attachment[],
): Promise<string | ContentPart[]> {
  if (attachments.length === 0) return text;

  const parts: ContentPart[] = [];
  if (text) parts.push({ type: "text", text });

  for (const attachment of attachments) {
    let data = "";
    try {
      data = await readAttachment(attachment.path);
    } catch {
      continue;
    }

    if ((attachment.mime ?? "").startsWith("image/")) {
      parts.push({
        type: "image_url",
        image_url: { url: `data:${attachment.mime};base64,${data}` },
      });
    } else if (isText(attachment.mime)) {
      parts.push({
        type: "text",
        text: `File: ${basename(attachment.path)}\n\n${decode(data)}`,
      });
    }
  }

  return parts.length > 0 ? parts : text;
}

async function streamReply(chatId: string, options: SendOptions): Promise<void> {
  chatState.error = null;

  const tempId = `pending-${Date.now()}`;
  chatState.messages.push({ id: tempId, role: "assistant", content: "", steps: [], sources: [] });
  const index = chatState.messages.length - 1;
  chatState.streaming = true;
  chatState.streamingChatId = chatId;
  chatState.thinking = false;

  const history: RequestMessage[] = await Promise.all(
    chatState.messages.slice(0, index).map(async (message) => ({
      role: message.role,
      content: await messageContent(message.content, message.attachments ?? []),
    })),
  );

  const find = () => chatState.messages.find((message) => message.id === tempId);

  const outcome = await runStream(
    chatId,
    {
      model: options.model,
      system: options.system,
      temperature: options.temperature,
      useContext: options.useContext,
      tools: options.tools,
      webSearch: options.webSearch,
      webToggle: options.webToggle,
      messages: history,
    },
    {
      onUpdate: (content) => {
        if (chatState.activeChatId !== chatId) return;
        const message = find();
        if (message) message.content = content;
      },
      onReasoning: () => {
        chatState.thinking = true;
      },
      onUsage: (usage) => {
        if (chatState.activeChatId !== chatId) return;
        const message = find();
        if (message) message.usage = usage;
      },
      onSteps: (steps) => {
        if (chatState.activeChatId !== chatId) return;
        const message = find();
        if (message) message.steps = steps;
      },
      onSources: (sources) => {
        if (chatState.activeChatId !== chatId) return;
        const message = find();
        if (message) message.sources = sources;
      },
    },
  );

  chatState.streaming = false;
  chatState.streamingChatId = null;
  chatState.thinking = false;

  if (outcome.error) chatState.error = outcome.error;

  if (outcome.content) {
    const stored = await appendMessage({
      chatId,
      role: "assistant",
      content: outcome.content,
      model: options.model,
      tokensIn: outcome.usage?.tokensIn,
      tokensOut: outcome.usage?.tokensOut,
      cost: outcome.usage?.cost ?? undefined,
      sources: outcome.sources,
    });
    const message = find();
    if (message) message.id = stored.id;
    if (chatState.activeChatId === chatId && outcome.usage) chatState.usage = outcome.usage;
    await refreshChats();
  } else {
    const emptyIndex = chatState.messages.findIndex((message) => message.id === tempId);
    if (emptyIndex !== -1) chatState.messages.splice(emptyIndex, 1);
  }
}

function isText(mime: string | null): boolean {
  if (!mime) return false;
  return mime.startsWith("text/") || mime === "application/json" || mime.endsWith("+json");
}

function decode(data: string): string {
  try {
    return atob(data);
  } catch {
    return "";
  }
}

function basename(path: string): string {
  return path.split("/").pop() ?? path;
}
