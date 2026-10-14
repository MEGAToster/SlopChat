import { appendMessage, getChat, readAttachment, rewindTo, saveAttachment } from "../ipc";
import type { Attachment, ContentPart, NewAttachment, RequestMessage } from "../types";
import { decodeBase64 } from "../bytes";
import { basename } from "../text";
import { base64Of, clearPending, pending } from "./attachments.svelte";
import { ensureChat, refreshChats } from "./chatHistory.svelte";
import { adoptWebDraft, chatState, lastUserMessage, type SendOptions } from "./chatStore.svelte";
import { currentSendOptions, adoptModelDraft, notify } from "./ui.svelte";
import { adoptReasoningDraft } from "./reasoning.svelte";
import { toChatMessage } from "./messages";
import { replyOutcome } from "./replyOutcome";
import { run as runStream, stop as stopStream } from "./stream.svelte";

// Set by Stop before the turn owns a stream id, so the setup awaits (chat
// creation, attachments, history) abort instead of racing ahead to the request.
let cancelSend = false;

export async function stop(): Promise<void> {
  cancelSend = true;
  if (chatState.streamingChatId) await stopStream(chatState.streamingChatId);
}

export async function submit(text: string, options: SendOptions): Promise<boolean> {
  const content = text.trim();
  const attachments = pending();
  if (chatState.streaming) {
    if (chatState.streamingChatId !== chatState.activeChatId) {
      notify("Another chat is still answering — wait or stop it first.", "danger");
    }
    return false;
  }
  if (!content && attachments.length === 0) return false;

  cancelSend = false;
  const chatId = await ensureChat(options.profileName);
  if (cancelSend) return false;
  adoptWebDraft(chatId);
  adoptReasoningDraft(chatId);
  adoptModelDraft(chatId);

  const saved: NewAttachment[] = [];
  for (const item of attachments) {
    if (item.stored) {
      saved.push({
        id: item.stored.id,
        path: item.stored.path,
        mime: item.stored.mime ?? item.mime,
      });
      continue;
    }
    const stored = await saveAttachment(item.name, item.mime, base64Of(item.dataUrl ?? ""));
    if (cancelSend) return false;
    saved.push({ id: stored.id, path: stored.path, mime: stored.mime ?? item.mime });
  }

  if (cancelSend) return false;
  const stored = await appendMessage({ chatId, role: "user", content, attachments: saved });
  chatState.messages.push(toChatMessage(stored));
  clearPending();
  await refreshChats();

  await streamReply(chatId, options);
  return true;
}

export async function regenerate(
  messageId: string,
  text: string,
  options: SendOptions,
): Promise<boolean> {
  const content = text.trim();
  if (chatState.streaming) {
    if (chatState.streamingChatId !== chatState.activeChatId) {
      notify("Another chat is still answering — wait or stop it first.", "danger");
    }
    return false;
  }
  if (!chatState.activeChatId || !content) return false;

  cancelSend = false;
  const chatId = chatState.activeChatId;
  await rewindTo(chatId, messageId, content);
  if (cancelSend) return false;
  const view = await getChat(chatId);
  if (cancelSend) return false;
  chatState.messages = view ? view.messages.map(toChatMessage) : [];
  await refreshChats();
  if (cancelSend) return false;

  await streamReply(chatId, options);
  return true;
}

// Re-answer the last user message with the current profile/model.
export async function retryLast(): Promise<void> {
  if (chatState.sending) return;
  const options = currentSendOptions();
  const last = lastUserMessage();
  if (!options || !last?.id) return;

  chatState.sending = true;
  try {
    await regenerate(last.id, last.content, options);
  } catch (cause) {
    notify(String(cause), "danger");
  } finally {
    chatState.sending = false;
  }
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
        text: `File: ${basename(attachment.path)}\n\n${decodeBase64(data)}`,
      });
    }
  }

  return parts.length > 0 ? parts : text;
}

async function streamReply(chatId: string, options: SendOptions): Promise<void> {
  if (cancelSend) return;
  const tempId = `pending-${Date.now()}`;
  chatState.messages.push({
    id: tempId,
    role: "assistant",
    content: "",
    model: options.model,
    steps: [],
    sources: [],
  });
  const index = chatState.messages.length - 1;
  chatState.streaming = true;
  chatState.streamingChatId = chatId;
  chatState.thinking = false;

  const history: RequestMessage[] = await Promise.all(
    chatState.messages
      .slice(0, index)
      // Pending placeholders never reached the DB and must not be replayed.
      .filter((message) => !message.id?.startsWith("pending-"))
      .map(async (message) => ({
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
      toolCalling: options.toolCalling,
      webSearch: options.webSearch,
      reasoning: options.reasoning,
      fallback: options.fallback,
      maxCompletionTokens: options.maxCompletionTokens,
      messages: history,
    },
    {
      onUpdate: (content) => {
        if (chatState.activeChatId !== chatId) return;
        const message = find();
        if (message) message.content = content;
      },
      onThinking: (active) => {
        chatState.thinking = active;
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
      onModel: (model) => {
        if (chatState.activeChatId !== chatId) return;
        const message = find();
        if (message) message.model = model;
      },
    },
  );

  chatState.streaming = false;
  chatState.streamingChatId = null;
  chatState.thinking = false;

  const message = find();
  if (outcome.error) {
    // The error belongs to the answer when it is on screen; otherwise toast it.
    if (message) message.error = outcome.error;
    else notify(outcome.error, "danger");
  }

  const hasDetail = (message?.steps?.length ?? 0) > 0 || (message?.sources?.length ?? 0) > 0;
  const result = replyOutcome({
    content: outcome.content,
    error: outcome.error,
    hasDetail,
    placeholderVisible: message !== undefined,
  });

  switch (result.kind) {
    case "store": {
      const stored = await appendMessage({
        chatId,
        role: "assistant",
        content: outcome.content,
        model: outcome.model ?? options.model,
        tokensIn: outcome.usage?.tokensIn,
        tokensOut: outcome.usage?.tokensOut,
        cost: outcome.usage?.cost ?? undefined,
        provider: outcome.usage?.provider ?? undefined,
        sources: outcome.sources,
      });
      const target = find();
      if (target) {
        target.id = stored.id;
      } else if (chatState.activeChatId === chatId) {
        // The user switched chats mid-stream, so the placeholder is gone; reload
        // so the stored reply shows without reopening the chat.
        const view = await getChat(chatId);
        if (view && chatState.activeChatId === chatId) {
          chatState.messages = view.messages.map(toChatMessage);
        }
      }
      if (chatState.activeChatId === chatId && outcome.usage) chatState.usage = outcome.usage;
      await refreshChats();
      break;
    }
    case "keep":
      if (message) message.content = result.content;
      break;
    case "remove": {
      const emptyIndex = chatState.messages.findIndex((entry) => entry.id === tempId);
      if (emptyIndex !== -1) chatState.messages.splice(emptyIndex, 1);
      break;
    }
    case "toast":
      // The error was already reported through notify().
      break;
  }
}

function isText(mime: string | null): boolean {
  if (!mime) return false;
  return mime.startsWith("text/") || mime === "application/json" || mime.endsWith("+json");
}


