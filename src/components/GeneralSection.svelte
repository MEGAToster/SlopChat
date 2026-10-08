<script lang="ts">
  import { tick } from "svelte";

  import {
    saveAdvanced,
    saveWindow,
    setAgentLog,
    setPasteAsFile,
    setReaderFallback,
    setResumeWithin,
    setStartup,
  } from "../lib/ipc";
  import type { Advanced, Startup, WindowConfig } from "../lib/types";
  import { settings } from "../lib/state/settings.svelte";
  import DeepSection from "./DeepSection.svelte";

  let advancedEl = $state<HTMLDetailsElement | null>(null);

  function clamp(event: Event, min: number, max: number): number {
    const value = Number((event.currentTarget as HTMLInputElement).value);
    return Math.min(max, Math.max(min, Math.round(value)));
  }

  function clampFloat(event: Event, min: number, max: number): number {
    const value = Number((event.currentTarget as HTMLInputElement).value);
    return Math.min(max, Math.max(min, value));
  }

  function saveLimits(patch: Partial<Advanced>): void {
    void saveAdvanced({ ...settings().advanced, ...patch });
  }

  function saveWindowSize(patch: Partial<WindowConfig>): void {
    void saveWindow({ ...settings().window, ...patch });
  }

  function changeStartup(event: Event): void {
    void setStartup((event.currentTarget as HTMLSelectElement).value as Startup);
  }

  async function onAdvancedToggle(): Promise<void> {
    await tick();
    advancedEl?.scrollIntoView({ block: "nearest", behavior: "smooth" });
  }
</script>

<div class="section-title">Window</div>

<label class="adv-row">
  <span class="adv-label">
    Compact width <span class="adv-unit">(px)</span>
  </span>
  <input
    class="control"
    type="number"
    min="400"
    max="1200"
    step="10"
    value={settings().window.compactWidth}
    onchange={(event) => saveWindowSize({ compactWidth: clamp(event, 400, 1200) })}
  />
</label>

<label class="adv-row">
  <span class="adv-label">
    Compact height <span class="adv-unit">(px, filled chat)</span>
  </span>
  <input
    class="control"
    type="number"
    min="400"
    max="1200"
    step="10"
    value={settings().window.compactHeight}
    onchange={(event) => saveWindowSize({ compactHeight: clamp(event, 400, 1200) })}
  />
</label>

<label class="adv-row">
  <span class="adv-label">Expanded screen ratio</span>
  <input
    class="control"
    type="number"
    min="0.3"
    max="1"
    step="0.05"
    value={settings().window.expandedScreenRatio}
    onchange={(event) => saveWindowSize({ expandedScreenRatio: clampFloat(event, 0.3, 1) })}
  />
</label>

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
      onchange={(event) => void setResumeWithin(clamp(event, 0, 100000))}
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
      onchange={(event) => void setPasteAsFile(clamp(event, 0, 1000000))}
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

  <label class="adv-row">
    <span class="adv-label">Max web searches / reply</span>
    <input
      class="control"
      type="number"
      min="1"
      max="20"
      value={settings().advanced.maxSearches}
      onchange={(event) => saveLimits({ maxSearches: clamp(event, 1, 20) })}
    />
  </label>

  <label class="adv-row">
    <span class="adv-label">Max page fetches / reply</span>
    <input
      class="control"
      type="number"
      min="0"
      max="100"
      value={settings().advanced.maxFetches}
      onchange={(event) => saveLimits({ maxFetches: clamp(event, 0, 100) })}
    />
  </label>

  <label class="adv-row">
    <span class="adv-label">Max tool rounds</span>
    <input
      class="control"
      type="number"
      min="1"
      max="10"
      value={settings().advanced.maxRounds}
      onchange={(event) => saveLimits({ maxRounds: clamp(event, 1, 10) })}
    />
  </label>

  <label class="adv-row">
    <span class="adv-label">
      Response timeout <span class="adv-unit">(s, no output)</span>
    </span>
    <input
      class="control"
      type="number"
      min="5"
      max="600"
      value={settings().advanced.responseTimeoutSecs}
      onchange={(event) => saveLimits({ responseTimeoutSecs: clamp(event, 5, 600) })}
    />
  </label>

  <label class="adv-row">
    <span class="adv-label">
      Max turn <span class="adv-unit">(s)</span>
    </span>
    <input
      class="control"
      type="number"
      min="30"
      max="3600"
      value={settings().advanced.maxTurnSecs}
      onchange={(event) => saveLimits({ maxTurnSecs: clamp(event, 30, 3600) })}
    />
  </label>

  <label class="adv-row">
    <span class="adv-label">
      Max fetched chars <span class="adv-unit">(0 = unlimited)</span>
    </span>
    <input
      class="control"
      type="number"
      min="0"
      max="500000"
      value={settings().advanced.maxFetchedChars}
      onchange={(event) => saveLimits({ maxFetchedChars: clamp(event, 0, 500000) })}
    />
  </label>

  <label class="adv-row">
    <span class="adv-label">
      Reasoning token cap <span class="adv-unit">(0 = off)</span>
    </span>
    <input
      class="control"
      type="number"
      min="0"
      max="100000"
      value={settings().advanced.reasoningMaxTokens}
      onchange={(event) => saveLimits({ reasoningMaxTokens: clamp(event, 0, 100000) })}
    />
  </label>

  <label class="adv-row">
    <span class="adv-label">
      Frequency penalty <span class="adv-unit">(0-2)</span>
    </span>
    <input
      class="control"
      type="number"
      min="0"
      max="2"
      step="0.1"
      value={settings().advanced.frequencyPenalty}
      onchange={(event) => saveLimits({ frequencyPenalty: clampFloat(event, 0, 2) })}
    />
  </label>

  <label class="adv-row">
    <span class="adv-label">
      Presence penalty <span class="adv-unit">(0-2)</span>
    </span>
    <input
      class="control"
      type="number"
      min="0"
      max="2"
      step="0.1"
      value={settings().advanced.presencePenalty}
      onchange={(event) => saveLimits({ presencePenalty: clampFloat(event, 0, 2) })}
    />
  </label>

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

  .adv-row {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    min-height: 36px;
    padding: 0 var(--sp-2);
  }

  .adv-label {
    flex: 1;
    color: var(--text-secondary);
    font-size: var(--fs-sm);
  }

  .adv-unit {
    color: var(--text-tertiary);
    font-size: var(--fs-sm);
  }

  .control {
    flex: none;
    width: 72px;
    padding: 4px var(--sp-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background-color: var(--surface);
    color: var(--text);
    font: inherit;
    font-size: var(--fs-sm);
  }

  .control:focus {
    outline: 1px solid var(--accent);
  }

  .check {
    flex: none;
    width: 16px;
    height: 16px;
    accent-color: var(--accent);
    cursor: pointer;
  }

  select.control {
    width: auto;
    cursor: pointer;
  }

  .muted {
    margin: var(--sp-2);
    color: var(--text-tertiary);
    font-size: var(--fs-sm);
  }
</style>
