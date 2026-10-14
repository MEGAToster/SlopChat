<script lang="ts">
  import { tick } from "svelte";
  import {
    activeChatId,
    isStreamingHere,
    isThinking,
    messages,
  } from "../lib/state/chatStore.svelte";
  import { settings } from "../lib/state/settings.svelte";
  import { compactCap, isExpanded } from "../lib/state/ui.svelte";
  import { followOutput, view } from "../lib/state/view.svelte";
  import Message from "./Message.svelte";
  import Kbd from "./Kbd.svelte";

  const THRESHOLD = 80;
  const MIN_TRANSCRIPT = 80;

  let viewport = $state<HTMLDivElement | null>(null);
  const all = $derived(messages());
  const streamingHere = $derived(isStreamingHere());
  const thinking = $derived(isThinking());
  const keybinds = $derived(settings().keybinds);

  function atBottom(): boolean {
    if (!viewport) return true;
    return viewport.scrollHeight - viewport.scrollTop - viewport.clientHeight < THRESHOLD;
  }

  function onScroll(): void {
    // Scrolling back down resumes following automatically.
    view.tracking = atBottom();
  }

  // Reads the last message's content so it re-runs on every streamed token,
  // after the DOM update, keeping the bottom in view while following.
  $effect(() => {
    const last = all.at(-1);
    void last?.content;
    void last?.steps;
    void all.length;
    const element = viewport;
    if (view.tracking && element) element.scrollTop = element.scrollHeight;
  });

  // Opening a chat lands at the bottom and resumes following.
  $effect(() => {
    void activeChatId();
    void tick().then(() => {
      if (!viewport) return;
      viewport.scrollTop = viewport.scrollHeight;
      view.tracking = true;
    });
  });

  // In compact mode the transcript is capped so a filled chat grows the panel
  // up to `compactHeight` and then scrolls; an empty chat collapses to content.
  function applyTranscriptCap(): void {
    const element = viewport;
    if (!element) return;
    if (isExpanded()) {
      element.style.maxHeight = "";
      return;
    }

    const content = document.querySelector<HTMLElement>(".content");
    if (!content) return;

    // Chrome is everything but the transcript, and stays fixed whether or not
    // the transcript is capped, so the cap avoids an uncap-measure reflow.
    const chrome = content.getBoundingClientRect().height - element.getBoundingClientRect().height;
    const cap = compactCap();
    const budget = cap - chrome;
    element.style.maxHeight =
      element.scrollHeight > budget ? `${Math.max(MIN_TRANSCRIPT, budget)}px` : "";
    if (view.tracking) element.scrollTop = element.scrollHeight;
  }

  $effect(() => {
    const last = all.at(-1);
    // Re-cap as the answer streams and once it settles (the answer bar mounts).
    void last?.content;
    void last?.steps;
    void streamingHere;
    void settings().window.compactHeight;
    void all.length;
    void isExpanded();
    void tick().then(applyTranscriptCap);
  });

  // Composer and footer heights change with attachments/wrapping; recompute then.
  $effect(() => {
    const content = document.querySelector<HTMLElement>(".content");
    if (!content) return;
    const observer = new ResizeObserver(applyTranscriptCap);
    for (const target of [content.querySelector(".composer-box"), content.querySelector(".footer")]) {
      if (target) observer.observe(target);
    }
    return () => observer.disconnect();
  });
</script>

<div class="messages column" bind:this={viewport} onscroll={onScroll}>
  {#each all as message, index (message.id ?? index)}
    <Message
      {message}
      streaming={streamingHere && index === all.length - 1 && message.role === "assistant"}
      thinking={thinking && index === all.length - 1}
      last={index === all.length - 1}
    />
  {/each}

  {#if !view.tracking}
    <button class="jump" onclick={followOutput}>
      ↓ Latest <Kbd binding={keybinds.follow?.[0]} />
    </button>
  {/if}
</div>

<style>
  .messages {
    display: flex;
    flex-direction: column;
    gap: var(--sp-4);
    padding-bottom: var(--sp-2);
    overflow-y: auto;
  }

  .jump {
    position: sticky;
    bottom: var(--sp-2);
    align-self: center;
    display: inline-flex;
    align-items: center;
    gap: var(--sp-1);
    padding: 4px var(--sp-3);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-lg);
    background: var(--surface);
    color: var(--text-secondary);
    box-shadow: var(--shadow-overlay);
    font: inherit;
    font-size: var(--fs-sm);
    cursor: pointer;
  }

  .jump:hover {
    color: var(--text);
  }
</style>
