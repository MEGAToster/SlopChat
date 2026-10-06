<script lang="ts">
  import { saveAdvanced } from "../lib/ipc";
  import { settings } from "../lib/state/settings.svelte";
  import type { DeepSettings } from "../lib/types";

  function clamp(event: Event, min: number, max: number): number {
    const value = Number((event.currentTarget as HTMLInputElement).value);
    return Math.min(max, Math.max(min, Math.round(value)));
  }

  function saveDeep(patch: Partial<DeepSettings>): void {
    void saveAdvanced({
      ...settings().advanced,
      deep: { ...settings().advanced.deep, ...patch },
    });
  }
</script>

<div class="section-title">Deep research</div>

<label class="adv-row">
  <span class="adv-label">
    Sub-agent model <span class="adv-unit">(empty = single loop)</span>
  </span>
  <input
    class="control wide"
    type="text"
    placeholder="openrouter/…"
    value={settings().advanced.deep.subagentModel}
    onchange={(event) => saveDeep({ subagentModel: event.currentTarget.value.trim() })}
  />
</label>

<label class="adv-row">
  <span class="adv-label">Max tool rounds</span>
  <input
    class="control"
    type="number"
    min="1"
    max="30"
    value={settings().advanced.deep.maxRounds}
    onchange={(event) => saveDeep({ maxRounds: clamp(event, 1, 30) })}
  />
</label>

<label class="adv-row">
  <span class="adv-label">Max web searches / reply</span>
  <input
    class="control"
    type="number"
    min="1"
    max="60"
    value={settings().advanced.deep.maxSearches}
    onchange={(event) => saveDeep({ maxSearches: clamp(event, 1, 60) })}
  />
</label>

<label class="adv-row">
  <span class="adv-label">Max page fetches / reply</span>
  <input
    class="control"
    type="number"
    min="0"
    max="100"
    value={settings().advanced.deep.maxFetches}
    onchange={(event) => saveDeep({ maxFetches: clamp(event, 0, 100) })}
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
    max="2000000"
    value={settings().advanced.deep.maxFetchedChars}
    onchange={(event) => saveDeep({ maxFetchedChars: clamp(event, 0, 2000000) })}
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
    value={settings().advanced.deep.maxTurnSecs}
    onchange={(event) => saveDeep({ maxTurnSecs: clamp(event, 30, 3600) })}
  />
</label>

<style>
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

  .control.wide {
    width: 168px;
  }
</style>
