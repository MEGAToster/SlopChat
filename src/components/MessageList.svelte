<script lang="ts">
  import { isStreamingHere, isThinking, messages } from "../lib/state/chat.svelte";
  import Message from "./Message.svelte";

  let viewport = $state<HTMLDivElement | null>(null);
  const all = $derived(messages());
  const streamingHere = $derived(isStreamingHere());
  const thinking = $derived(isThinking());

  let pinned = true;

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

<div class="messages" bind:this={viewport}>
  {#each all as message, index (index)}
    <Message
      {message}
      streaming={streamingHere && index === all.length - 1 && message.role === "assistant"}
      thinking={thinking && index === all.length - 1}
    />
  {/each}
</div>
