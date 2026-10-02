<script lang="ts">
  import { tick } from "svelte";
  import {
    activeChatId,
    isStreamingHere,
    isThinking,
    messages,
  } from "../lib/state/chatStore.svelte";
  import { view } from "../lib/state/view.svelte";
  import Message from "./Message.svelte";

  const THRESHOLD = 80;

  let viewport = $state<HTMLDivElement | null>(null);
  const all = $derived(messages());
  const streamingHere = $derived(isStreamingHere());
  const thinking = $derived(isThinking());

  function atBottom(): boolean {
    if (!viewport) return true;
    return viewport.scrollHeight - viewport.scrollTop - viewport.clientHeight < THRESHOLD;
  }

  function onScroll(): void {
    // Scrolling away from the bottom pauses following; the indicator reflects it.
    if (view.tracking && !atBottom()) view.tracking = false;
  }

  // Reads the last message's content so it re-runs on every streamed token,
  // after the DOM update, keeping the bottom in view while following.
  $effect(() => {
    const last = all.at(-1);
    void last?.content;
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
</div>

<style>
  .messages {
    display: flex;
    flex-direction: column;
    flex: 1 1 auto;
    min-height: 0;
    gap: var(--sp-4);
    padding-bottom: var(--sp-2);
    overflow-y: auto;
  }
</style>
