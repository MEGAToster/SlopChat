<script lang="ts">
  import type { ChatGroup } from "../lib/chatGroups";
  import { focusComposer } from "../lib/dom";
  import { flatIndex } from "../lib/flat";
  import { renameChat } from "../lib/state/chatHistory.svelte";
  import { activeChatId, isBlank } from "../lib/state/chatStore.svelte";
  import { settings } from "../lib/state/settings.svelte";
  import {
    newChatWithProfile,
    openChatWithProfile,
    pendingDelete,
    renamingId,
    schedulePreview,
    selectedIndex,
    setRenaming,
    setSelectedIndex,
  } from "../lib/state/ui.svelte";
  import Icon from "./Icon.svelte";
  import Kbd from "./Kbd.svelte";

  let { groups }: { groups: ChatGroup[] } = $props();

  const counts = $derived(groups.map((group) => group.chats.length));
  const total = $derived(counts.reduce((sum, count) => sum + count, 0));
  const keybinds = $derived(settings().keybinds);

  // Async content matches can shrink the list under the selection.
  $effect(() => {
    if (selectedIndex() > total) setSelectedIndex(total);
  });

  function flat(groupIndex: number, chatIndex: number): number {
    return flatIndex(counts, groupIndex, chatIndex);
  }

  async function open(index: number, chatId: string) {
    setSelectedIndex(index);
    await openChatWithProfile(chatId);
    focusComposer();
  }

  function autofocus(node: HTMLInputElement) {
    node.focus();
    node.select();
  }

  function commitRename(event: FocusEvent, chatId: string) {
    const value = (event.currentTarget as HTMLInputElement).value.trim();
    setRenaming(null);
    if (value) void renameChat(chatId, value);
  }

  function onRenameKey(event: KeyboardEvent) {
    event.stopPropagation();
    if (event.key === "Enter") {
      event.preventDefault();
      (event.currentTarget as HTMLInputElement).blur();
    } else if (event.key === "Escape") {
      event.preventDefault();
      setRenaming(null);
    }
  }
</script>

<div class="list">
  {#each groups as group, groupIndex (group.title)}
    <div class="section">{group.title}</div>
    {#each group.chats as chat, chatIndex (chat.id)}
      {@const row = flat(groupIndex, chatIndex)}
      {#if renamingId() === chat.id}
        <div class="chat renaming">
          <input
            class="rename"
            value={chat.title}
            use:autofocus
            onkeydown={onRenameKey}
            onblur={(event) => commitRename(event, chat.id)}
          />
        </div>
      {:else}
        <button
          type="button"
          class="chat"
          class:selected={row === selectedIndex()}
          class:active={chat.id === activeChatId()}
          class:confirming={pendingDelete() === chat.id}
          onmouseenter={() => {
            setSelectedIndex(row);
            schedulePreview();
          }}
          onclick={() => void open(row, chat.id)}
        >
          {#if chat.id === activeChatId()}<span class="active-bar"></span>{/if}
          {#if pendingDelete() === chat.id}
            <span class="title">Delete “{chat.title || "New chat"}”?</span>
            <span class="keys"><Kbd binding="y" /><Kbd binding="n" /></span>
          {:else}
            <span class="title">{chat.title || "New chat"}</span>
            {#if chat.pinned}<span class="pin"><Icon name="pin" size={13} /></span>{/if}
          {/if}
        </button>
      {/if}
    {/each}
  {/each}
</div>

<button
  type="button"
  class="new"
  class:selected={selectedIndex() === total}
  disabled={isBlank()}
  onmouseenter={() => setSelectedIndex(total)}
  onclick={() => void newChatWithProfile()}
>
  <span class="plus">+</span> New chat
  <Kbd binding={keybinds.nextChat?.[0]} />
</button>

<style>
  .list {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }

  .section {
    display: flex;
    align-items: center;
    height: 28px;
    padding: 0 var(--sp-2);
    color: var(--text-tertiary);
    font-size: var(--fs-xs);
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .chat {
    position: relative;
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    width: 100%;
    height: 32px;
    padding: 0 var(--sp-2);
    border: none;
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--text-secondary);
    font: inherit;
    font-size: var(--fs-ui);
    text-align: left;
    cursor: default;
  }

  .chat.selected {
    background: var(--selection);
  }

  .chat.active,
  .chat.selected {
    color: var(--text);
  }

  .chat.confirming {
    background: color-mix(in srgb, var(--danger) 12%, transparent);
  }

  .chat.renaming {
    background: var(--selection);
  }

  .rename {
    width: 100%;
    height: 100%;
    padding: 0 var(--sp-2);
    border: none;
    outline: none;
    background: transparent;
    color: var(--text);
    font: inherit;
    font-size: var(--fs-ui);
  }

  .active-bar {
    position: absolute;
    left: 2px;
    top: 7px;
    bottom: 7px;
    width: 2px;
    border-radius: 1px;
    background: var(--accent);
  }

  .title {
    flex: 1;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  .pin,
  .keys {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: var(--text-tertiary);
  }

  .new {
    display: flex;
    align-items: center;
    gap: var(--sp-1);
    height: 32px;
    margin-top: var(--sp-1);
    padding: 0 var(--sp-2);
    border: none;
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--text-secondary);
    font: inherit;
    font-size: var(--fs-ui);
    cursor: default;
  }

  .new:hover:not(:disabled),
  .new.selected {
    background: var(--selection);
    color: var(--text);
  }

  .new:disabled {
    opacity: 0.4;
  }

  .new .plus {
    font-size: var(--fs-lg);
  }
</style>
