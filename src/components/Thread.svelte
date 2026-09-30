<script lang="ts">
  import { messages } from "../lib/state/chat.svelte";
  import Composer from "./Composer.svelte";
  import Footer from "./Footer.svelte";
  import Header from "./Header.svelte";
  import MessageList from "./MessageList.svelte";

  let {
    text = $bindable(),
    onfocus,
    sticky = false,
    expanded = false,
  }: { text: string; onfocus: () => void; sticky?: boolean; expanded?: boolean } = $props();
</script>

{#if expanded}<Header {sticky} />{/if}
<MessageList />
{#if messages().length > 0}<div class="divider"></div>{/if}
<Composer bind:value={text} {onfocus} />
<Footer />

<style>
  .divider {
    height: 1px;
    background: var(--border);
  }
</style>
