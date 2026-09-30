<script lang="ts">
  import { onMount } from "svelte";
  import {
    index,
    items,
    loading,
    mode,
    query,
    setQuery,
    type PaletteItem,
  } from "../lib/state/palette.svelte";
  import Icon from "./Icon.svelte";

  let { onactivate }: { onactivate: (index: number) => void } = $props();

  let input = $state<HTMLInputElement | null>(null);
  const list = $derived(items());

  function iconFor(item: PaletteItem): string {
    return item.kind;
  }

  onMount(() => input?.focus());
</script>

<div class="palette">
  <div class="palette-search">
    <Icon name="search" size={18} />
    <input
      bind:this={input}
      class="palette-input"
      placeholder={mode() === "add" ? "Search provider models…" : "Search actions, chats, models…"}
      value={query()}
      oninput={(event) => setQuery(event.currentTarget.value)}
    />
  </div>

  {#if mode() === "add"}
    <p class="palette-hint">Pick a model to open settings and add it.</p>
  {/if}

  <div class="palette-list">
    {#if loading()}
      <p class="muted">Loading models…</p>
    {:else}
      {#each list as item, i (item.kind + ":" + item.id)}
        <button
          type="button"
          class="palette-item"
          class:selected={i === index()}
          onclick={() => onactivate(i)}
        >
          <Icon name={iconFor(item)} />
          <span class="palette-label">{item.label}</span>
          {#if item.kind === "provider"}
            <span class="palette-provider">{item.provider}</span>
          {/if}
          {#if i === index()}
            <span class="palette-enter"><Icon name="enter" size={15} /></span>
          {/if}
        </button>
      {/each}
    {/if}
  </div>
</div>
