<script lang="ts">
  import { activeChatId, chats, messages } from "../lib/state/chat.svelte";
  import { activeProfile, keyContext } from "../lib/state/ui.svelte";
  import Composer from "./Composer.svelte";
  import Footer from "./Footer.svelte";
  import Frame from "./Frame.svelte";
  import MessageList from "./MessageList.svelte";

  let {
    text = $bindable(),
    onfocus,
    sticky = false,
    expanded = false,
  }: { text: string; onfocus: () => void; sticky?: boolean; expanded?: boolean } = $props();

  const title = $derived(chats().find((chat) => chat.id === activeChatId())?.title || "New chat");
  const profileName = $derived(activeProfile()?.name ?? "");
  const context = $derived(keyContext());
</script>

{#if messages().length > 0 || expanded}
  <Frame {title}>
    <MessageList />
  </Frame>
{/if}

<Frame
  title={`ask · ${profileName}`}
  focused={context === "composer"}
  hint="↵ send · ⇧⇥ profile · ⌃P cmds"
>
  <Composer bind:value={text} {onfocus} />
</Frame>

<Footer {sticky} />
