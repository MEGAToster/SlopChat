<script lang="ts">
  import { focusComposer } from "../lib/dom";
  import { activeChatId, isBlank } from "../lib/state/chat.svelte";
  import { settings } from "../lib/state/settings.svelte";
  import {
    chatGroups,
    filterQuery,
    filtering,
    newChatWithProfile,
    openChatWithProfile,
    pendingDelete,
    schedulePreview,
    selectedIndex,
    setContext,
    setFiltering,
    setFilterQuery,
    setSelectedIndex,
    keyContext,
  } from "../lib/state/ui.svelte";
  import Frame from "./Frame.svelte";
  import Icon from "./Icon.svelte";
  import Kbd from "./Kbd.svelte";

  let filterInput = $state<HTMLInputElement | null>(null);
  const groups = $derived(chatGroups());
  const keybinds = $derived(settings().keybinds);

  $effect(() => {
    if (filtering()) filterInput?.focus();
  });

  function flatIndex(groupIndex: number, chatIndex: number): number {
    let offset = 0;
    for (let i = 0; i < groupIndex; i += 1) offset += groups[i].chats.length;
    return offset + chatIndex;
  }

  function startFilter() {
    setContext("sidebar");
    setFiltering(true);
  }

  async function open(index: number, chatId: string) {
    setSelectedIndex(index);
    await openChatWithProfile(chatId);
    focusComposer();
  }
</script>

<aside class="sidebar">
  <Frame
    title="chats"
    hint="↵ open · p pin · d delete"
    focused={keyContext() === "sidebar"}
    count={`${groups.reduce((total, group) => total + group.chats.length, 0)}`}
  >
    {#if filtering()}
    <input
      bind:this={filterInput}
      class="filter"
      placeholder="Filter"
      value={filterQuery()}
      oninput={(event) => setFilterQuery(event.currentTarget.value)}
    />
  {:else}
    <button type="button" class="search" onclick={startFilter}>
      <span class="search-icon"><Icon name="search" size={14} /></span>
      <span class="search-label">Search chats</span>
      <Kbd binding="/" />
    </button>
  {/if}

  <div class="list">
    {#each groups as group, groupIndex (group.title)}
      <div class="section">{group.title}</div>
      {#each group.chats as chat, chatIndex (chat.id)}
        {@const flat = flatIndex(groupIndex, chatIndex)}
        <button
          type="button"
          class="chat"
          class:selected={flat === selectedIndex()}
          class:active={chat.id === activeChatId()}
          class:confirming={pendingDelete() === chat.id}
          onmouseenter={() => {
            setSelectedIndex(flat);
            schedulePreview();
          }}
          onclick={() => void open(flat, chat.id)}
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
      {/each}
    {/each}
  </div>

    <button type="button" class="new" disabled={isBlank()} onclick={() => void newChatWithProfile()}>
      <span class="plus">+</span> New chat
      <Kbd binding={keybinds.nextChat?.[0]} />
    </button>
  </Frame>
</aside>

<style>
  .sidebar {
    display: flex;
    flex: none;
    flex-direction: column;
    width: 240px;
    padding: var(--sp-2);
    border-right: 1px solid var(--border);
    background: var(--sidebar);
    overflow: hidden;
  }

  .search,
  .filter {
    height: 32px;
    margin-bottom: var(--sp-1);
    border: none;
    border-radius: var(--radius-md);
    background: var(--surface);
    color: var(--text-secondary);
    font: inherit;
    font-size: var(--fs-md);
  }

  .search {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    padding: 0 var(--sp-2);
    cursor: default;
  }

  .search:hover {
    background: var(--surface-hover);
  }

  .search-icon {
    display: inline-flex;
    color: var(--text-tertiary);
  }

  .search-label {
    flex: 1;
    text-align: left;
  }

  .filter {
    padding: 0 var(--sp-2);
    outline: none;
  }

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
    font-size: 13px;
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
    font-size: 13px;
    cursor: default;
  }

  .new:hover:not(:disabled) {
    background: var(--surface-hover);
  }

  .new:disabled {
    opacity: 0.4;
  }

  .new .plus {
    font-size: var(--fs-lg);
  }
</style>
