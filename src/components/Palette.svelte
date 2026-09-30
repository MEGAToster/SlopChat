<script lang="ts">
  import { onMount } from "svelte";
  import { highlight } from "../lib/fuzzy";
  import {
    breadcrumb,
    groups,
    index,
    loading,
    mode,
    query,
    setQuery,
    type PaletteItem,
  } from "../lib/state/palette.svelte";
  import Frame from "./Frame.svelte";
  import Icon from "./Icon.svelte";
  import Kbd from "./Kbd.svelte";

  let { onactivate }: { onactivate: (index: number) => void } = $props();

  let input = $state<HTMLInputElement | null>(null);
  let list = $state<HTMLDivElement | null>(null);
  const sections = $derived(groups());

  $effect(() => {
    void index();
    list?.querySelector(".row.selected")?.scrollIntoView({ block: "nearest" });
  });

  function flatIndex(sectionIndex: number, itemIndex: number): number {
    let offset = 0;
    for (let i = 0; i < sectionIndex; i += 1) offset += sections[i].items.length;
    return offset + itemIndex;
  }

  function iconFor(item: PaletteItem): string {
    return item.kind;
  }

  onMount(() => input?.focus());
</script>

<div class="palette">
  <Frame
    title="commands"
    hint={mode() === "add" ? "↵ add · ⎋ back" : "↵ open · ⎋ close"}
    focused
    count={`${sections.reduce((total, section) => total + section.items.length, 0)}`}
  >
    <div class="search">
    <Icon name="search" size={18} />
    <input
      bind:this={input}
      class="input"
      placeholder={mode() === "add" ? "Search provider models…" : "Search actions, chats, models…"}
      value={query()}
      oninput={(event) => setQuery(event.currentTarget.value)}
    />
  </div>

  <div class="list" bind:this={list}>
    {#if loading()}
      <p class="muted">Loading models…</p>
    {:else}
      {#each sections as section, sectionIndex (section.title)}
        <div class="section">{section.title}</div>
        {#each section.items as item, itemIndex (section.title + ":" + item.id)}
          <button
            type="button"
            class="row"
            class:selected={flatIndex(sectionIndex, itemIndex) === index()}
            onclick={() => onactivate(flatIndex(sectionIndex, itemIndex))}
          >
            <span class="icon"><Icon name={iconFor(item)} /></span>
            <span class="label">
              {#each highlight(query(), item.label) as segment}
                <span class:matched={segment.match}>{segment.text}</span>
              {/each}
            </span>
            {#if item.provider}<span class="kind">{item.provider}</span>{/if}
            {#if item.binding}<Kbd binding={item.binding} />{/if}
          </button>
        {/each}
      {/each}
    {/if}
  </div>

  <footer class="bar">
    <span class="crumb">{breadcrumb()}</span>
    <span class="hints">
      <Kbd binding="Enter" /> {mode() === "add" ? "Add" : "Open"}
      <Kbd binding="Esc" /> {mode() === "add" ? "Back" : "Close"}
    </span>
  </footer>
  </Frame>
</div>

<style>
  .palette {
    position: absolute;
    inset: 0;
    z-index: 10;
    display: flex;
    flex-direction: column;
    background: transparent;
    animation: rise-in var(--dur-fast) var(--ease);
  }

  .search {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    height: 52px;
    padding: 0 var(--sp-4);
    border-bottom: 1px solid var(--border);
    color: var(--text-tertiary);
  }

  .input {
    flex: 1;
    border: none;
    outline: none;
    background: transparent;
    color: var(--text);
    font: inherit;
    font-size: var(--fs-xl);
  }

  .input::placeholder {
    color: var(--text-tertiary);
  }

  .list {
    flex: 1;
    padding: var(--sp-2);
    overflow-y: auto;
  }

  .section {
    height: 28px;
    display: flex;
    align-items: center;
    padding: 0 var(--sp-2);
    color: var(--text-tertiary);
    font-size: var(--fs-xs);
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .row {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    width: 100%;
    height: 40px;
    padding: 0 var(--sp-2);
    border: none;
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--text);
    font: inherit;
    font-size: var(--fs-md);
    text-align: left;
    cursor: default;
  }

  .row.selected {
    background: var(--selection);
  }

  .icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex: none;
    width: 24px;
    height: 24px;
    color: var(--text-tertiary);
  }

  .row.selected .icon {
    color: var(--accent);
  }

  .label {
    flex: 1;
    overflow: hidden;
    color: var(--text-secondary);
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  .label .matched {
    color: var(--text);
    font-weight: 600;
  }

  .kind {
    flex: none;
    color: var(--text-tertiary);
    font-size: var(--fs-sm);
  }

  .muted {
    margin: var(--sp-3);
    color: var(--text-tertiary);
    font-size: var(--fs-sm);
  }

  .bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 36px;
    padding: 0 var(--sp-3);
    border-top: 1px solid var(--border);
    color: var(--text-tertiary);
    font-size: var(--fs-sm);
  }

  .hints {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-2);
  }
</style>
