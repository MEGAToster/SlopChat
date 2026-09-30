<script lang="ts">
  import { applyCitations } from "../lib/markdown/citations";
  import { renderMarkdown } from "../lib/markdown/render";
  import { MarkdownStream } from "../lib/markdown/stream";
  import type { ChatMessage } from "../lib/state/chat.svelte";
  import type { ToolStep } from "../lib/types";
  import Icon from "./Icon.svelte";

  let {
    message,
    streaming = false,
    thinking = false,
  }: { message: ChatMessage; streaming?: boolean; thinking?: boolean } = $props();

  const stream = new MarkdownStream();
  let html = $state("");

  const rendered = $derived(applyCitations(html, message.sources ?? []));

  $effect(() => {
    html = streaming ? stream.update(message.content) : renderMarkdown(message.content);
  });

  function name(path: string): string {
    return path.split("/").pop() ?? path;
  }

  function query(step: ToolStep): string {
    const value = step.args?.query;
    return typeof value === "string" ? value : step.name;
  }

  function domain(url: string): string {
    try {
      return new URL(url).hostname.replace(/^www\./, "");
    } catch {
      return url;
    }
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
    {#if message.steps?.length}
      <div class="steps">
        {#each message.steps as step (step.id)}
          <div class="step" class:failed={step.status === "error"}>
            <span class="glyph">⌕</span>
            {#if step.status === "running"}
              searching “{query(step)}”…
            {:else if step.status === "error"}
              search failed · {step.summary}
            {:else}
              searched “{query(step)}” · {step.summary}
            {/if}
          </div>
        {/each}
      </div>
    {/if}

    {#if streaming && !message.content}
      {#if thinking}
        <span class="thinking">thinking…</span>
      {:else}
        <span class="caret">▍</span>
      {/if}
    {:else}
      <div class="markdown">{@html rendered}</div>
      {#if streaming}<span class="caret">▍</span>{/if}
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

  .attachments,
  .sources,
  .steps {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-1);
  }

  .steps {
    flex-direction: column;
    margin-bottom: var(--sp-1);
  }

  .step {
    color: var(--text-tertiary);
    font-size: var(--fs-sm);
  }

  .step.failed {
    color: var(--danger);
  }

  .glyph {
    margin-right: 4px;
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
    padding: 2px 8px;
    border-radius: 999px;
    background: var(--surface);
    color: var(--text-secondary);
    font-size: var(--fs-xs);
    text-decoration: none;
  }

  .source:hover {
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
