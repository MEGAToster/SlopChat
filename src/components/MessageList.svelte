<script lang="ts">
  import { isStreaming, messages } from "../lib/state/chat.svelte";
  import Message from "./Message.svelte";

  let viewport = $state<HTMLDivElement | null>(null);
  const all = $derived(messages());
  const streaming = $derived(isStreaming());

  $effect(() => {
    const last = all.at(-1);
    void last?.content;
    void all.length;
    if (viewport) viewport.scrollTop = viewport.scrollHeight;
  });
</script>

<div class="messages" bind:this={viewport}>
  {#each all as message, index (index)}
    <Message
      {message}
      streaming={streaming && index === all.length - 1 && message.role === "assistant"}
    />
  {/each}
</div>
