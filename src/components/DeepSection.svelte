<script lang="ts">
  import { clampInt } from "../lib/input";
  import { saveAdvanced } from "../lib/ipc";
  import { settings } from "../lib/state/settings.svelte";
  import type { DeepSettings } from "../lib/types";

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
    onchange={(event) => saveDeep({ maxRounds: clampInt(event, 1, 30) })}
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
    onchange={(event) => saveDeep({ maxSearches: clampInt(event, 1, 60) })}
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
    onchange={(event) => saveDeep({ maxFetches: clampInt(event, 0, 100) })}
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
    onchange={(event) => saveDeep({ maxFetchedChars: clampInt(event, 0, 2000000) })}
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
    onchange={(event) => saveDeep({ maxTurnSecs: clampInt(event, 30, 3600) })}
  />
</label>

