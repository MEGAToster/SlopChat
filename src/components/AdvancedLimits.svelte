<script lang="ts">
  import { clampFloat, clampInt } from "../lib/input";
  import { saveAdvanced } from "../lib/ipc";
  import { settings } from "../lib/state/settings.svelte";
  import type { Advanced } from "../lib/types";

  function saveLimits(patch: Partial<Advanced>): void {
    void saveAdvanced({ ...settings().advanced, ...patch });
  }
</script>

<label class="adv-row">
  <span class="adv-label">Max web searches / reply</span>
  <input
    class="control"
    type="number"
    min="1"
    max="20"
    value={settings().advanced.maxSearches}
    onchange={(event) => saveLimits({ maxSearches: clampInt(event, 1, 20) })}
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
    onchange={(event) => saveLimits({ maxFetches: clampInt(event, 0, 100) })}
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
    onchange={(event) => saveLimits({ maxRounds: clampInt(event, 1, 10) })}
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
    onchange={(event) => saveLimits({ responseTimeoutSecs: clampInt(event, 5, 600) })}
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
    onchange={(event) => saveLimits({ maxTurnSecs: clampInt(event, 30, 3600) })}
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
    onchange={(event) => saveLimits({ maxFetchedChars: clampInt(event, 0, 500000) })}
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
    onchange={(event) => saveLimits({ reasoningMaxTokens: clampInt(event, 0, 100000) })}
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
