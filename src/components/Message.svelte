<script lang="ts">
  import { renderMarkdown } from "../lib/markdown/render";
  import { MarkdownStream } from "../lib/markdown/stream";
  import type { ChatMessage } from "../lib/state/chat.svelte";
  import Icon from "./Icon.svelte";

  let {
    message,
    streaming = false,
    thinking = false,
  }: { message: ChatMessage; streaming?: boolean; thinking?: boolean } = $props();

  const stream = new MarkdownStream();
  let html = $state("");

  $effect(() => {
    html = streaming ? stream.update(message.content) : renderMarkdown(message.content);
  });

  function name(path: string): string {
    return path.split("/").pop() ?? path;
  }
</script>

{#if message.role === "user"}
  <div class="message user">
    {#if message.attachments?.length}
      <div class="attachments">
        {#each message.attachments as file (file.id)}
          <span class="attachment"><Icon name="attach" size={12} />{name(file.path)}</span>
        {/each}
      </div>
    {/if}
    {#if message.content}<span class="text">{message.content}</span>{/if}
  </div>
{:else}
  <div class="message assistant">
    {#if streaming && !message.content}
      {#if thinking}
        <span class="thinking">thinking…</span>
      {:else}
        <span class="caret">▍</span>
      {/if}
    {:else}
      <div class="markdown">{@html html}</div>
      {#if streaming}<span class="caret">▍</span>{/if}
    {/if}
  </div>
{/if}

<style>
  .message {
    font-size: var(--fs-md);
  }

  .message.user {
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
    align-self: flex-end;
    max-width: 80%;
    padding: 8px 12px;
    border-radius: var(--radius-lg);
    background: var(--surface);
    word-break: break-word;
  }

  .text {
    white-space: pre-wrap;
  }

  .attachments {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-1);
  }

  .attachment {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    max-width: 200px;
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    background: var(--surface-hover);
    color: var(--text-secondary);
    font-size: var(--fs-xs);
  }

  .message.assistant {
    align-self: stretch;
    min-width: 0;
  }

  .thinking {
    color: var(--text-tertiary);
    animation: pulse 1.2s ease-in-out infinite;
  }

  .caret {
    display: inline-block;
    color: var(--accent);
    animation: blink 1s steps(1) infinite;
  }

  @keyframes blink {
    50% {
      opacity: 0;
    }
  }

  @keyframes pulse {
    50% {
      opacity: 0.4;
    }
  }
</style>
