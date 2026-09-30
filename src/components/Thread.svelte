<script lang="ts">
  import { isStreamingHere, stop } from "../lib/state/chat.svelte";
  import Composer from "./Composer.svelte";
  import MessageList from "./MessageList.svelte";
  import ProfileBadge from "./ProfileBadge.svelte";
  import StatusLine from "./StatusLine.svelte";

  let {
    text = $bindable(),
    onfocus,
    sticky = false,
  }: { text: string; onfocus: () => void; sticky?: boolean } = $props();
</script>

<MessageList />

<div class="footer">
  <StatusLine />
  <div class="footer-row">
    <ProfileBadge />
    {#if sticky}<span class="pin">pinned</span>{/if}
    {#if isStreamingHere()}
      <button type="button" class="stop" onclick={() => void stop()}>Stop</button>
    {/if}
  </div>
</div>

<Composer bind:value={text} {onfocus} />
