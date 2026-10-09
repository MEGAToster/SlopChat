<script lang="ts">
  import {
    filterQuery,
    filtering,
    setFiltering,
    setFilterQuery,
  } from "../lib/state/chatFilter.svelte";
  import { chatGroups, keyContext, setContext, setSelectedIndex } from "../lib/state/ui.svelte";
  import Frame from "./Frame.svelte";
  import Icon from "./Icon.svelte";
  import Kbd from "./Kbd.svelte";
  import SidebarList from "./SidebarList.svelte";

  let filterInput = $state<HTMLInputElement | null>(null);
  const groups = $derived(chatGroups());
  const total = $derived(groups.reduce((sum, group) => sum + group.chats.length, 0));

  $effect(() => {
    if (filtering()) filterInput?.focus();
  });

  function startFilter() {
    setContext("sidebar");
    setFiltering(true);
    setSelectedIndex(0);
  }
</script>

<aside class="sidebar">
  <Frame
    title="chats"
    hint="↵ open · p pin · d delete"
    focused={keyContext() === "sidebar"}
    count={`${total}`}
  >
    {#if filtering()}
      <input
        bind:this={filterInput}
        class="filter"
        placeholder="Search chats"
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

    <SidebarList {groups} />
  </Frame>
</aside>

<style>
  .sidebar {
    display: flex;
    flex: none;
    flex-direction: column;
    width: calc(240px * var(--scale));
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
</style>
