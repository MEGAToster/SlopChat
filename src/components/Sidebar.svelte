<script lang="ts">
  import { focusComposer } from "../lib/dom";
  import { activeChatId, activeChatIsEmpty } from "../lib/state/chat.svelte";
  import {
    filterQuery,
    filtering,
    newChatWithProfile,
    openChatWithProfile,
    pendingDelete,
    selectedIndex,
    setFilterQuery,
    setSelectedIndex,
    visibleChats,
  } from "../lib/state/ui.svelte";

  let filterInput = $state<HTMLInputElement | null>(null);

  $effect(() => {
    if (filtering()) filterInput?.focus();
  });

  async function open(index: number, chatId: string) {
    setSelectedIndex(index);
    await openChatWithProfile(chatId);
    focusComposer();
  }

  async function create() {
    await newChatWithProfile();
    focusComposer();
  }
</script>

<aside class="sidebar">
  {#if filtering()}
    <input
      bind:this={filterInput}
      class="filter"
      placeholder="Filter chats"
      value={filterQuery()}
      oninput={(event) => setFilterQuery(event.currentTarget.value)}
    />
  {/if}

  <div class="chat-list">
    {#each visibleChats() as chat, index (chat.id)}
      <button
        type="button"
        class="chat-item"
        class:selected={index === selectedIndex()}
        class:active={chat.id === activeChatId()}
        onclick={() => void open(index, chat.id)}
      >
        <span class="chat-title">{chat.title || "New chat"}</span>
        {#if chat.pinned}<span class="chat-pin">pinned</span>{/if}
        {#if pendingDelete() === chat.id}<span class="chat-confirm">delete? y/n</span>{/if}
      </button>
    {/each}
  </div>

  <button
    type="button"
    class="new-tab"
    disabled={activeChatIsEmpty()}
    onclick={() => void create()}
  >
    + New chat
  </button>
</aside>
