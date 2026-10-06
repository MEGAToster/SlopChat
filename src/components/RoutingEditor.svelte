<script lang="ts">
  import { untrack } from "svelte";

  import type { Routing } from "../lib/types";

  let {
    value,
    onchange,
  }: {
    value: Routing;
    onchange: (next: Routing) => void;
  } = $props();

  const SORT_OPTIONS = ["", "price", "throughput", "latency"];

  function listText(items?: string[]): string {
    return (items ?? []).join(", ");
  }

  function parseList(text: string): string[] {
    return text
      .split(",")
      .map((item) => item.trim())
      .filter(Boolean);
  }

  let order = $state(untrack(() => listText(value.order)));
  let only = $state(untrack(() => listText(value.only)));
  let ignore = $state(untrack(() => listText(value.ignore)));

  function commitLists(): void {
    onchange({ ...value, order: parseList(order), only: parseList(only), ignore: parseList(ignore) });
  }

  function setFallbacks(text: string): void {
    onchange({ ...value, allowFallbacks: text === "" ? undefined : text === "true" });
  }

  function setSort(text: string): void {
    onchange({ ...value, sort: text || undefined });
  }

  // USD per 1M tokens. An empty input clears that side of the price cap.
  function setPrice(which: "prompt" | "completion", text: string): void {
    const price = { ...value.maxPrice };
    const parsed = text.trim() === "" ? undefined : Number(text);
    if (parsed === undefined || Number.isNaN(parsed)) delete price[which];
    else price[which] = parsed;
    const empty = price.prompt === undefined && price.completion === undefined;
    onchange({ ...value, maxPrice: empty ? undefined : price });
  }
</script>

<details class="routing">
  <summary class="section-title">Routing</summary>

  <label class="row">
    <span class="label">Order</span>
    <input class="control" bind:value={order} onchange={commitLists} placeholder="anthropic, openai" />
  </label>

  <label class="row">
    <span class="label">Only</span>
    <input class="control" bind:value={only} onchange={commitLists} placeholder="anthropic" />
  </label>

  <label class="row">
    <span class="label">Ignore</span>
    <input class="control" bind:value={ignore} onchange={commitLists} placeholder="deepinfra" />
  </label>

  <label class="row">
    <span class="label">Allow fallbacks</span>
    <select
      class="control"
      value={value.allowFallbacks === undefined ? "" : String(value.allowFallbacks)}
      onchange={(event) => setFallbacks(event.currentTarget.value)}
    >
      <option value="">default</option>
      <option value="true">yes</option>
      <option value="false">no</option>
    </select>
  </label>

  <label class="row">
    <span class="label">Sort</span>
    <select class="control" value={value.sort ?? ""} onchange={(event) => setSort(event.currentTarget.value)}>
      {#each SORT_OPTIONS as option (option)}
        <option value={option}>{option || "default"}</option>
      {/each}
    </select>
  </label>

  <label class="row">
    <span class="label">Max $/M prompt</span>
    <input
      class="control"
      type="number"
      min="0"
      step="0.01"
      value={value.maxPrice?.prompt ?? ""}
      onchange={(event) => setPrice("prompt", event.currentTarget.value)}
    />
  </label>

  <label class="row">
    <span class="label">Max $/M completion</span>
    <input
      class="control"
      type="number"
      min="0"
      step="0.01"
      value={value.maxPrice?.completion ?? ""}
      onchange={(event) => setPrice("completion", event.currentTarget.value)}
    />
  </label>
</details>

<style>
  .routing > summary {
    cursor: pointer;
    list-style: none;
  }

  .routing > summary::before {
    content: "▸";
    margin-right: var(--sp-1);
  }

  .routing[open] > summary::before {
    content: "▾";
  }
</style>
