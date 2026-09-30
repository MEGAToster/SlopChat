<script lang="ts">
  import { activeChatId, chats } from "../lib/state/chat.svelte";
  import { settings } from "../lib/state/settings.svelte";
  import Icon from "./Icon.svelte";
  import Kbd from "./Kbd.svelte";

  let { sticky = false }: { sticky?: boolean } = $props();

  const title = $derived(chats().find((chat) => chat.id === activeChatId())?.title || "New chat");
  const keybinds = $derived(settings().keybinds);
</script>

<header class="header">
  <span class="title">{title}</span>
  <span class="sticky" class:on={sticky}>
    <Icon name="pin" size={14} />
    <Kbd binding={keybinds.sticky?.[0]} />
  </span>
</header>

<style>
  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-2);
    height: 44px;
    padding: 0 var(--sp-3);
    border-bottom: 1px solid var(--border);
  }

  .title {
    overflow: hidden;
    font-size: var(--fs-md);
    font-weight: 600;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .sticky {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: var(--text-tertiary);
  }

  .sticky.on {
    color: var(--accent);
  }
</style>
