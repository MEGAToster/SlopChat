<script lang="ts">
  import { duration } from "../lib/duration";
  import { shortModel, splitModelRef } from "../lib/modelRef";
  import { retryLast } from "../lib/state/chat.svelte";
  import type { ChatMessage } from "../lib/state/chatStore.svelte";
  import { usageLine } from "../lib/usageInfo";
  import Icon from "./Icon.svelte";

  let {
    message,
    last = false,
    streaming = false,
    elapsed = 0,
  }: { message: ChatMessage; last?: boolean; streaming?: boolean; elapsed?: number } = $props();

  let copied = $state(false);

  const provider = $derived(splitModelRef(message.model ?? "").provider);

  async function copyAnswer(): Promise<void> {
    try {
      await navigator.clipboard.writeText(message.content);
    } catch {
      return;
    }
    copied = true;
    setTimeout(() => (copied = false), 1200);
  }
</script>

{#if message.error}
  <div class="error">{message.content ? "Interrupted" : "Failed"}: {message.error}</div>
{/if}

<div class="answer-bar" class:last>
  <span class="meta">
    {#if message.model}<span>{shortModel(message.model)}</span>{/if}
    {#if streaming}
      {#if message.model}<span class="sep">·</span>{/if}
      <span>{duration(elapsed)}</span>
    {:else if message.usage}
      {#if message.model}<span class="sep">·</span>{/if}
      <span>{usageLine(message.usage, provider)}</span>
    {/if}
  </span>

  {#if !streaming}
    <div class="answer-actions">
      <button
        type="button"
        class="action"
        title="Copy answer"
        aria-label="Copy answer"
        onclick={() => void copyAnswer()}
      >
        <Icon name={copied ? "check" : "copy"} size={13} />
      </button>
      {#if last}
        <button
          type="button"
          class="action"
          title="Retry answer"
          aria-label="Retry answer"
          onclick={() => void retryLast()}
        >
          <Icon name="refresh" size={13} />
        </button>
      {/if}
    </div>
  {/if}
</div>

<style>
  .answer-bar {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    margin-top: var(--sp-3);
    color: var(--text-tertiary);
    font-size: var(--fs-xs);
    font-variant-numeric: tabular-nums;
  }

  /* Quiet for old answers; reveal on hover or when focus enters the message. */
  .answer-bar:not(.last) {
    opacity: 0;
  }

  :global(.message.assistant:hover) .answer-bar,
  :global(.message.assistant:focus-within) .answer-bar {
    opacity: 1;
  }

  .meta {
    display: inline-flex;
    align-items: baseline;
    gap: 4px;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .sep {
    opacity: 0.6;
  }

  .answer-actions {
    display: inline-flex;
    gap: var(--sp-1);
    margin-left: auto;
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

  .error {
    margin-top: var(--sp-2);
    padding-left: var(--sp-2);
    border-left: 2px solid var(--danger);
    color: var(--danger);
    font-size: var(--fs-sm);
  }
</style>
