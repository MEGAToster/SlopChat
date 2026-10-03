<script lang="ts">
  import { applyCitations } from "../lib/markdown/citations";
  import { mathState } from "../lib/markdown/math.svelte";
  import { renderMermaid } from "../lib/markdown/mermaid";
  import { renderMarkdown } from "../lib/markdown/render";
  import { MarkdownStream } from "../lib/markdown/stream";
  import { basename, domain } from "../lib/text";
  import { retryLast } from "../lib/state/chat.svelte";
  import type { ChatMessage } from "../lib/state/chatStore.svelte";
  import Icon from "./Icon.svelte";
  import MessageSteps from "./MessageSteps.svelte";

  let {
    message,
    streaming = false,
    thinking = false,
    last = false,
  }: { message: ChatMessage; streaming?: boolean; thinking?: boolean; last?: boolean } = $props();

  let copied = $state(false);

  async function copyAnswer(): Promise<void> {
    try {
      await navigator.clipboard.writeText(message.content);
    } catch {
      return;
    }
    copied = true;
    setTimeout(() => (copied = false), 1200);
  }

  const stream = new MarkdownStream();
  let html = $state("");
  let markdown = $state<HTMLElement | null>(null);
  let elapsed = $state(0);

  const rendered = $derived(applyCitations(html, message.sources ?? []));

  // Ticks while this message streams so a long turn doesn't look frozen.
  $effect(() => {
    if (!streaming) {
      elapsed = 0;
      return;
    }
    const started = Date.now();
    const timer = setInterval(() => {
      elapsed = Math.floor((Date.now() - started) / 1000);
    }, 1000);
    return () => clearInterval(timer);
  });

  function duration(seconds: number): string {
    const minutes = Math.floor(seconds / 60);
    const rest = seconds % 60;
    return minutes > 0 ? `${minutes}:${String(rest).padStart(2, "0")}` : `${rest}s`;
  }

  // `mathState.ready` re-renders once KaTeX finishes lazy-loading.
  $effect(() => {
    void mathState.ready;
    html = streaming ? stream.update(message.content) : renderMarkdown(message.content);
  });

  $effect(() => {
    void rendered;
    void mathState.ready;
    if (!streaming && markdown) void renderMermaid(markdown);
  });
</script>

{#if message.role === "user"}
  <div class="message user">
    {#if message.attachments?.length}
      <div class="attachments">
        {#each message.attachments as file (file.id)}
          <span class="attachment"><Icon name="attach" size={12} />{basename(file.path)}</span>
        {/each}
      </div>
    {/if}
    {#if message.content}<span class="text">{message.content}</span>{/if}
  </div>
{:else}
  <div class="message assistant">
    {#if message.steps?.length}
      <MessageSteps steps={message.steps} />
    {/if}

    {#if streaming && !message.content}
      {#if thinking}
        <span class="thinking">thinking… {duration(elapsed)}</span>
      {:else}
        <span class="caret">▍</span>
      {/if}
    {:else}
      <div class="markdown" bind:this={markdown}>{@html rendered}</div>
      {#if streaming}<span class="caret">▍</span>{/if}
    {/if}

    {#if !streaming}
      <div class="answer-bar">
        <span class="rule"></span>
        <div class="answer-actions">
          <button type="button" class="action" title="Copy answer" onclick={() => void copyAnswer()}>
            <Icon name={copied ? "check" : "copy"} size={13} />
          </button>
          {#if last}
            <button type="button" class="action" title="Retry answer" onclick={() => void retryLast()}>
              <Icon name="refresh" size={13} />
            </button>
          {/if}
        </div>
        <span class="rule"></span>
      </div>
    {/if}

    {#if message.sources?.length}
      <div class="sources">
        {#each message.sources as source (source.n)}
          <a class="source" href={source.url} title={source.title}>
            {source.n} {domain(source.url)}
          </a>
        {/each}
      </div>
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

  .answer-bar {
    display: grid;
    grid-template-columns: 7fr auto 1fr;
    align-items: center;
    gap: var(--sp-2);
    margin-top: var(--sp-3);
  }

  .rule {
    height: 1px;
    background: var(--border);
  }

  .answer-actions {
    display: inline-flex;
    gap: var(--sp-1);
  }

  .action {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 2px;
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-tertiary);
    cursor: pointer;
  }

  .action:hover {
    background: var(--surface-hover);
    color: var(--text);
  }

  .sources {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-1);
    margin-top: var(--sp-2);
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

  .source {
    padding: 1px 7px;
    border: 1px solid var(--border);
    border-radius: 999px;
    color: var(--text-tertiary);
    font-size: var(--fs-xs);
    text-decoration: none;
  }

  .source:hover {
    border-color: var(--accent);
    color: var(--accent);
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
