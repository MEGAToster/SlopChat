<script lang="ts">
  import { renderMarkdown } from "../lib/markdown/render";
  import { MarkdownStream } from "../lib/markdown/stream";
  import type { ChatMessage } from "../lib/state/chat.svelte";

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
</script>

{#if message.role === "user"}
  <div class="message user">{message.content}</div>
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
