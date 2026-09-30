import { appendMessage, getChat, rewindTo } from "../ipc";
import { chatState, type SendOptions } from "./chatStore.svelte";
import { ensureChat, refreshChats } from "./chatHistory.svelte";
import { toChatMessage } from "./messages";
import { run as runStream, stop as stopStream } from "./stream.svelte";

export * from "./chatStore.svelte";
export * from "./chatHistory.svelte";

export async function submit(text: string, options: SendOptions): Promise<void> {
  const content = text.trim();
  if (chatState.streaming || !content) return;

  const chatId = await ensureChat(options.profileName);
  const stored = await appendMessage({ chatId, role: "user", content });
  chatState.messages.push(toChatMessage(stored));
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

async function streamReply(chatId: string, options: SendOptions): Promise<void> {
  chatState.error = null;

  const tempId = `pending-${Date.now()}`;
  chatState.messages.push({ id: tempId, role: "assistant", content: "" });
  const index = chatState.messages.length - 1;
  chatState.streaming = true;
  chatState.streamingChatId = chatId;
  chatState.thinking = false;

  const history = chatState.messages
    .slice(0, index)
    .map((message) => ({ role: message.role, content: message.content }));

  const find = () => chatState.messages.find((message) => message.id === tempId);

  const outcome = await runStream(
    chatId,
    {
      model: options.model,
      system: options.system,
      temperature: options.temperature,
      useContext: options.useContext,
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
