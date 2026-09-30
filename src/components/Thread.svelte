<script lang="ts">
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
<Composer bind:value={text} {onfocus} />
<Footer />
