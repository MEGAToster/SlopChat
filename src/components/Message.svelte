<script lang="ts">
  import { applyCitations } from "../lib/markdown/citations";
  import { mathState } from "../lib/markdown/math.svelte";
  import { renderMermaid } from "../lib/markdown/mermaid";
  import { renderMarkdown } from "../lib/markdown/render";
  import { MarkdownStream } from "../lib/markdown/stream";
  import { basename } from "../lib/text";
  import type { ChatMessage } from "../lib/state/chatStore.svelte";
  import type { ToolStep } from "../lib/types";
  import Icon from "./Icon.svelte";

  let {
    message,
    streaming = false,
    thinking = false,
  }: { message: ChatMessage; streaming?: boolean; thinking?: boolean } = $props();

  const stream = new MarkdownStream();
  let html = $state("");
  let markdown = $state<HTMLElement | null>(null);

  const rendered = $derived(applyCitations(html, message.sources ?? []));

  $effect(() => {
    void mathState.ready;
    html = streaming ? stream.update(message.content) : renderMarkdown(message.content);
  });

  $effect(() => {
    void rendered;
    void mathState.ready;
    if (!streaming && markdown) void renderMermaid(markdown);
  });

  function query(step: ToolStep): string {
    const value = step.args?.query;
    return typeof value === "string" ? value : step.name;
  }

  function isFetch(step: ToolStep): boolean {
    return step.name === "web_fetch" || step.name === "fetch_url";
  }

  function target(step: ToolStep): string {
    const value = step.args?.url ?? step.args?.query;
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
          <span class="attachment"><Icon name="attach" size={12} />{basename(file.path)}</span>
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
            <span class="glyph">{isFetch(step) ? "↓" : "⌕"}</span>
            {#if isFetch(step)}
              {#if step.status === "running"}
                fetching {domain(target(step))}…
              {:else if step.status === "error"}
                fetch failed · {step.summary}
              {:else}
                fetched {domain(target(step))} · {step.summary}
              {/if}
            {:else if step.status === "running"}
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
      <div class="markdown" bind:this={markdown}>{@html rendered}</div>
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
  .steps {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-1);
  }

  .sources {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-1);
    margin-top: var(--sp-3);
    padding-top: var(--sp-2);
    border-top: 1px solid var(--border);
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
