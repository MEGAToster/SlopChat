<script lang="ts">
  import { clampFloat, clampInt } from "../lib/input";
  import { saveWindow } from "../lib/ipc";
  import { settings } from "../lib/state/settings.svelte";
  import type { WindowConfig } from "../lib/types";
  import { WINDOW_MAX, WINDOW_MIN } from "../lib/window";

  function saveWindowSize(patch: Partial<WindowConfig>): void {
    void saveWindow({ ...settings().window, ...patch });
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
    min={WINDOW_MIN}
    max={WINDOW_MAX}
    step="10"
    value={settings().window.compactWidth}
    onchange={(event) => saveWindowSize({ compactWidth: clampInt(event, WINDOW_MIN, WINDOW_MAX) })}
  />
</label>

<label class="adv-row">
  <span class="adv-label">
    Compact height <span class="adv-unit">(px, filled chat)</span>
  </span>
  <input
    class="control"
    type="number"
    min={WINDOW_MIN}
    max={WINDOW_MAX}
    step="10"
    value={settings().window.compactHeight}
    onchange={(event) => saveWindowSize({ compactHeight: clampInt(event, WINDOW_MIN, WINDOW_MAX) })}
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
