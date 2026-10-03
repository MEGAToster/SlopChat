<script lang="ts">
  import { tick } from "svelte";
  import {
    activeChatId,
    isStreamingHere,
    isThinking,
    messages,
  } from "../lib/state/chatStore.svelte";
  import { settings } from "../lib/state/settings.svelte";
  import { isExpanded } from "../lib/state/ui.svelte";
  import { view } from "../lib/state/view.svelte";
  import Message from "./Message.svelte";

  const THRESHOLD = 80;
  const MIN_TRANSCRIPT = 80;

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
    const style = getComputedStyle(content);
    const gap = parseFloat(style.rowGap || style.gap) || 0;
    const padding = (parseFloat(style.paddingTop) || 0) + (parseFloat(style.paddingBottom) || 0);

    const composer = content.querySelector<HTMLElement>(".composer-box");
    const composerHeight = composer
      ? (composer.closest<HTMLElement>(".frame") ?? composer).getBoundingClientRect().height
      : 0;
    const footer = content.querySelector<HTMLElement>(".footer");
    const footerHeight = footer ? footer.getBoundingClientRect().height : 0;

    // Transcript + composer + footer are three blocks, so two gaps between them.
    const chrome = padding + composerHeight + footerHeight + gap * 2;
    const cap = Math.min(1200, Math.max(400, settings().window.compactHeight));
    element.style.maxHeight = `${Math.max(MIN_TRANSCRIPT, cap - chrome)}px`;
  }

  $effect(() => {
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
</div>

<style>
  .messages {
    display: flex;
    flex-direction: column;
    gap: var(--sp-4);
    padding-bottom: var(--sp-2);
    overflow-y: auto;
  }
</style>
