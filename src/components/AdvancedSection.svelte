<script lang="ts">
  import { tick } from "svelte";

  import { clampInt } from "../lib/input";
  import {
    setAgentLog,
    setPasteAsFile,
    setReaderFallback,
    setResumeWithin,
    setStartup,
  } from "../lib/ipc";
  import type { Startup } from "../lib/types";
  import { settings } from "../lib/state/settings.svelte";
  import AdvancedLimits from "./AdvancedLimits.svelte";
  import DeepSection from "./DeepSection.svelte";

  let advancedEl = $state<HTMLDetailsElement | null>(null);

  function changeStartup(event: Event): void {
    void setStartup((event.currentTarget as HTMLSelectElement).value as Startup);
  }

  async function onAdvancedToggle(): Promise<void> {
    await tick();
    advancedEl?.scrollIntoView({ block: "nearest", behavior: "smooth" });
  }
</script>

<details class="advanced" bind:this={advancedEl} ontoggle={onAdvancedToggle}>
  <summary class="section-title">Advanced</summary>

  <label class="adv-row">
    <span class="adv-label">On open</span>
    <select class="control" value={settings().startup} onchange={changeStartup}>
      <option value="pinned">Pinned chat</option>
      <option value="latest">Latest chat</option>
      <option value="new">New chat</option>
    </select>
  </label>

  <label class="adv-row">
    <span class="adv-label">Resume within <span class="adv-unit">(in min)</span></span>
    <input
      class="control"
      type="number"
      min="0"
      title="0 = no limit (always resume)"
      value={settings().resumeWithinMinutes}
      onchange={(event) => void setResumeWithin(clampInt(event, 0, 100000))}
    />
  </label>

  <label class="adv-row">
    <span class="adv-label">
      Convert pastes over <span class="adv-unit">(chars)</span>
    </span>
    <input
      class="control"
      type="number"
      min="0"
      title="0 = never convert manually (very large pastes still attach automatically)"
      value={settings().pasteAsFileChars}
      onchange={(event) => void setPasteAsFile(clampInt(event, 0, 1000000))}
    />
  </label>

  <label class="adv-row">
    <span class="adv-label">
      Reader fallback
      <span class="adv-unit">(r.jina.ai for blocked pages)</span>
    </span>
    <input
      class="check"
      type="checkbox"
      title="Read blocked or JS-only pages through r.jina.ai, which sees the URL"
      checked={settings().search.readerFallback}
      onchange={(event) => void setReaderFallback((event.currentTarget as HTMLInputElement).checked)}
    />
  </label>

  <label class="adv-row">
    <span class="adv-label">
      Agent log
      <span class="adv-unit">(JSONL under data dir)</span>
    </span>
    <input
      class="check"
      type="checkbox"
      title="Write every agent round, tool call and result to the log file"
      checked={settings().debug.agentLog}
      onchange={(event) => void setAgentLog((event.currentTarget as HTMLInputElement).checked)}
    />
  </label>

  <AdvancedLimits />

  <DeepSection />

  <p class="muted">“On open” applies at launch and when the window is summoned with the keybind.</p>
</details>

<style>
  .advanced {
    margin-top: var(--sp-2);
    scroll-margin-block: var(--sp-2);
  }

  .advanced > summary {
    cursor: pointer;
    list-style: none;
  }

  .advanced > summary::before {
    content: "▸";
    margin-right: var(--sp-1);
  }

  .advanced[open] > summary::before {
    content: "▾";
  }
</style>
