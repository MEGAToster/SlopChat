<script lang="ts">
  import { isStreamingHere, isThinking, messages } from "../lib/state/chat.svelte";
  import Message from "./Message.svelte";

  let viewport = $state<HTMLDivElement | null>(null);
  const all = $derived(messages());
  const streamingHere = $derived(isStreamingHere());
  const thinking = $derived(isThinking());

  let pinned = $state(true);

  $effect.pre(() => {
    const last = all.at(-1);
    void last?.content;
    void all.length;
    if (viewport) {
      pinned = viewport.scrollHeight - viewport.scrollTop - viewport.clientHeight < 80;
    }
  });

  $effect(() => {
    if (pinned && viewport) viewport.scrollTop = viewport.scrollHeight;
  });
</script>

<div class="messages column" class:fading={!pinned} bind:this={viewport}>
  {#each all as message, index (index)}
    <Message
      {message}
      streaming={streamingHere && index === all.length - 1 && message.role === "assistant"}
      thinking={thinking && index === all.length - 1}
    />
  {/each}
</div>

<style>
  .messages {
    display: flex;
    flex-direction: column;
    gap: var(--sp-4);
    max-height: 320px;
    overflow-y: auto;
  }

  .messages.fading {
    mask-image: var(--fade-bottom);
  }
</style>
