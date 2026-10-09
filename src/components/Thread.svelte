<script lang="ts">
  import { formatKeys } from "../lib/keymap/format";
  import { shortModel } from "../lib/modelRef";
  import { activeChatId, chats, messages } from "../lib/state/chatStore.svelte";
  import { settings } from "../lib/state/settings.svelte";
  import { activeConfig, activeProfile, editingId, keyContext, overrideActive } from "../lib/state/ui.svelte";
  import { view } from "../lib/state/view.svelte";
  import Composer from "./Composer.svelte";
  import EmptyState from "./EmptyState.svelte";
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
  const composerTitle = $derived.by(() => {
    const config = activeConfig();
    if (overrideActive() && config?.model) return `ask · ${shortModel(config.model)}`;
    return `ask · ${profileName}`;
  });
  const context = $derived(keyContext());
  const hint = $derived.by(() => {
    const parts = [`↵ send · ${formatKeys(settings().keybinds.cycleProfile)} profile`];
    if (!view.tracking) parts.push(`↓ ${formatKeys(settings().keybinds.follow)} latest`);
    return parts.join(" · ");
  });
</script>

{#if messages().length > 0 || expanded}
  <Frame {title}>
    {#if messages().length === 0}
      <EmptyState />
    {:else}
      <MessageList />
    {/if}
  </Frame>
{/if}

<Frame
  title={composerTitle}
  focused={context === "composer"}
  {hint}
>
  <Composer bind:value={text} {onfocus} editing={editingId() !== null} />
</Frame>

<Footer {sticky} />
