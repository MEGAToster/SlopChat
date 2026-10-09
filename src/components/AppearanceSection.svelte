<script lang="ts">
  import { saveFont, setReadingWidth } from "../lib/ipc";
  import { clampFontSize, FONT_DEFAULT, FONT_MAX, FONT_MIN } from "../lib/fontSize";
  import { settings } from "../lib/state/settings.svelte";
  import { setStyle, setTheme, themes } from "../lib/state/theme.svelte";
  import type { ReadingWidth } from "../lib/types";

  const font = $derived(settings().font);

  function changeTheme(event: Event): void {
    void setTheme((event.currentTarget as HTMLSelectElement).value);
  }

  function changeStyle(event: Event): void {
    void setStyle((event.currentTarget as HTMLSelectElement).value);
  }

  function changeFontSize(event: Event): void {
    const value = Number((event.currentTarget as HTMLInputElement).value);
    void saveFont({ ...font, size: clampFontSize(value) });
  }

  function changeFont(field: "ui" | "mono", event: Event): void {
    const value = (event.currentTarget as HTMLInputElement).value.trim();
    void saveFont({ ...font, [field]: value || undefined });
  }

  function changeReadingWidth(event: Event): void {
    void setReadingWidth((event.currentTarget as HTMLSelectElement).value as ReadingWidth);
  }
</script>

<div class="section-title">Appearance</div>

<label class="adv-row">
  <span class="adv-label">Theme</span>
  <select class="control" value={settings().theme} onchange={changeTheme}>
    {#each themes() as entry (entry.name)}
      <option value={entry.name}>{entry.name}</option>
    {/each}
  </select>
</label>

<label class="adv-row">
  <span class="adv-label">Style</span>
  <select class="control" value={settings().style} onchange={changeStyle}>
    <option value="auto">auto</option>
    <option value="minimal">minimal</option>
    <option value="tui">tui</option>
  </select>
</label>

<label class="adv-row">
  <span class="adv-label">Font size <span class="adv-unit">(px)</span></span>
  <input
    class="control"
    type="number"
    min={FONT_MIN}
    max={FONT_MAX}
    step="1"
    value={font?.size ?? FONT_DEFAULT}
    onchange={changeFontSize}
  />
</label>

<label class="adv-row">
  <span class="adv-label">UI font</span>
  <input
    class="control wide"
    type="text"
    placeholder="system default"
    value={font?.ui ?? ""}
    onchange={(event) => changeFont("ui", event)}
  />
</label>

<label class="adv-row">
  <span class="adv-label">Mono font</span>
  <input
    class="control wide"
    type="text"
    placeholder="system default"
    value={font?.mono ?? ""}
    onchange={(event) => changeFont("mono", event)}
  />
</label>

<label class="adv-row">
  <span class="adv-label">Reading width</span>
  <select class="control" value={settings().readingWidth} onchange={changeReadingWidth}>
    <option value="narrow">narrow</option>
    <option value="wide">wide</option>
    <option value="full">full</option>
  </select>
</label>

