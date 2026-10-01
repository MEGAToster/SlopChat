<script lang="ts">
  import { onMount, tick } from "svelte";
  import {
    removeModel,
    removeProfile,
    saveAdvanced,
    saveModel,
    saveProfile,
    searchUsage,
    setReaderFallback,
    setResumeWithin,
    setStartup,
  } from "../lib/ipc";
  import type { Advanced, ModelConfig, Profile, SearchProviderUsage, Startup } from "../lib/types";
  import {
    composedModel,
    modelId,
    registerEditorBack,
  } from "../lib/state/settingsPanel.svelte";
  import { settings } from "../lib/state/settings.svelte";
  import ApiKeyRow from "./ApiKeyRow.svelte";
  import Frame from "./Frame.svelte";
  import ModelEditor from "./ModelEditor.svelte";
  import ProfileEditor from "./ProfileEditor.svelte";

  let { onclose }: { onclose: () => void } = $props();

  const providers = $derived(Object.keys(settings().providers));
  const models = $derived(settings().models);
  const profiles = $derived(settings().profiles);

  const searchProviders = [
    { name: "exa", label: "Exa", placeholder: "exa-…", hint: "$10 free credit / month" },
    { name: "serper", label: "Serper", placeholder: "…", hint: "2,500 free queries once" },
    { name: "tavily", label: "Tavily", placeholder: "tvly-…", hint: "1,000 free credits / month" },
    { name: "brave", label: "Brave", placeholder: "…", hint: "$5 free credit / month" },
  ];

  let usage = $state<Record<string, SearchProviderUsage>>({});
  let order = $state("exa → serper → tavily → brave");

  let editingModel = $state<ModelConfig | null>(null);
  let modelPrevious = $state<string | null>(null);
  let editingProfile = $state<Profile | null>(null);
  let profilePrevious = $state<string | null>(null);
  let advancedEl = $state<HTMLDetailsElement | null>(null);

  function blankModel(): ModelConfig {
    return { model: "", useContext: true, tools: [], toolCalling: true, webSearch: "auto" };
  }

  function openModel(entry: ModelConfig): void {
    editingModel = { ...entry };
    modelPrevious = entry.model;
  }

  function openNewModel(): void {
    editingModel = blankModel();
    modelPrevious = null;
  }

  function openProfile(entry: Profile): void {
    editingProfile = { ...entry };
    profilePrevious = entry.name;
  }

  function openNewProfile(): void {
    editingProfile = { name: "", model: models[0]?.model ?? "", useContext: true };
    profilePrevious = null;
  }

  async function saveModelEdit(config: ModelConfig, previous: string | null): Promise<void> {
    await saveModel(config, previous ?? undefined);
    editingModel = null;
  }

  async function deleteModelEdit(): Promise<void> {
    if (modelPrevious) await removeModel(modelPrevious);
    editingModel = null;
  }

  async function saveProfileEdit(profile: Profile): Promise<void> {
    await saveProfile(profile);
    editingProfile = null;
  }

  async function deleteProfileEdit(): Promise<void> {
    if (profilePrevious) await removeProfile(profilePrevious);
    editingProfile = null;
  }

  async function refreshUsage(): Promise<void> {
    const list = await searchUsage();
    usage = Object.fromEntries(list.map((entry) => [entry.provider, entry]));
    if (list.length > 0) order = list.map((entry) => entry.provider).join(" → ");
  }

  onMount(() => {
    registerEditorBack(() => {
      if (editingModel || editingProfile) {
        editingModel = null;
        editingProfile = null;
        return true;
      }
      return false;
    });

    void refreshUsage();
    // Opened from the palette's "Add models…" with a prefilled provider/model.
    if (modelId()) openModel({ ...blankModel(), model: composedModel() });

    return () => registerEditorBack(null);
  });

  function dim(name: string): boolean {
    const entry = usage[name];
    return entry !== undefined && (!entry.hasKey || entry.exhausted);
  }

  function clamp(event: Event, min: number, max: number): number {
    const value = Number((event.currentTarget as HTMLInputElement).value);
    return Math.min(max, Math.max(min, Math.round(value)));
  }

  function saveLimits(patch: Partial<Advanced>): void {
    void saveAdvanced({ ...settings().advanced, ...patch });
  }

  function changeStartup(event: Event): void {
    void setStartup((event.currentTarget as HTMLSelectElement).value as Startup);
  }

  async function onAdvancedToggle(): Promise<void> {
    await tick();
    advancedEl?.scrollIntoView({ block: "nearest", behavior: "smooth" });
  }
</script>

<div class="settings">
  <Frame title="settings" hint="⎋ close" focused>
    <header class="head">
      <h2>Settings</h2>
      <button type="button" class="ghost" onclick={onclose}>Done</button>
    </header>

    <div class="body">
      {#if editingModel}
        <ModelEditor
          {providers}
          value={editingModel}
          previous={modelPrevious}
          onsave={(config, previous) => void saveModelEdit(config, previous)}
          oncancel={() => (editingModel = null)}
          onremove={modelPrevious ? () => void deleteModelEdit() : undefined}
        />
      {:else if editingProfile}
        <ProfileEditor
          {models}
          value={editingProfile}
          previous={profilePrevious}
          onsave={(profile) => void saveProfileEdit(profile)}
          oncancel={() => (editingProfile = null)}
          onremove={profilePrevious ? () => void deleteProfileEdit() : undefined}
        />
      {:else}
        <div class="section-head">
          <div class="section-title">Models</div>
          <button type="button" class="ghost" onclick={openNewModel}>Add model</button>
        </div>
        {#if models.length === 0}
          <p class="muted">No models yet.</p>
        {:else}
          {#each models as entry (entry.model)}
            <button type="button" class="item" onclick={() => openModel(entry)}>
              <span class="item-label">{entry.model}</span>
              {#if entry.reasoning}<span class="tag">{entry.reasoning}</span>{/if}
            </button>
          {/each}
        {/if}

        <div class="section-head">
          <div class="section-title">Rotation (Shift+Tab)</div>
          <button type="button" class="ghost" onclick={openNewProfile}>Add to rotation</button>
        </div>
        {#if profiles.length === 0}
          <p class="muted">No rotation entries yet.</p>
        {:else}
          {#each profiles as entry (entry.name)}
            <button type="button" class="item" onclick={() => openProfile(entry)}>
              <span class="item-name">{entry.name}</span>
              <span class="item-label">{entry.model}</span>
            </button>
          {/each}
        {/if}

        <div class="section-title">Web search</div>
        <p class="order">{order}</p>
        {#each searchProviders as entry (entry.name)}
          <ApiKeyRow
            provider={entry.name}
            label={entry.label}
            placeholder={entry.placeholder}
            hint={entry.hint}
            dim={dim(entry.name)}
            onchange={() => void refreshUsage()}
          />
        {/each}
        <p class="muted">Providers are tried in order; a provider without a key is skipped.</p>

        <details class="advanced" bind:this={advancedEl} ontoggle={onAdvancedToggle}>
          <summary class="section-title">Advanced</summary>

          <div class="adv-row">
            <span class="adv-label">On open</span>
            <select class="control" value={settings().startup} onchange={changeStartup}>
              <option value="pinned">Pinned chat</option>
              <option value="latest">Latest chat</option>
              <option value="new">New chat</option>
            </select>
          </div>

          <div class="adv-row">
            <span class="adv-label">Resume within <span class="adv-unit">(in min)</span></span>
            <input
              class="control"
              type="number"
              min="0"
              title="0 = no limit (always resume)"
              value={settings().resumeWithinMinutes}
              onchange={(event) => void setResumeWithin(clamp(event, 0, 100000))}
            />
          </div>

          <div class="adv-row">
            <span class="adv-label">
              Reader fallback
              <span class="adv-unit">(r.jina.ai for blocked pages)</span>
            </span>
            <input
              class="check"
              type="checkbox"
              title="Read blocked or JS-only pages through r.jina.ai, which sees the URL"
              checked={settings().search.readerFallback}
              onchange={(event) =>
                void setReaderFallback((event.currentTarget as HTMLInputElement).checked)}
            />
          </div>

          <div class="adv-row">
            <span class="adv-label">Max web searches / reply</span>
            <input
              class="control"
              type="number"
              min="1"
              max="20"
              value={settings().advanced.maxSearches}
              onchange={(event) => saveLimits({ maxSearches: clamp(event, 1, 20) })}
            />
          </div>

          <div class="adv-row">
            <span class="adv-label">Max page fetches / reply</span>
            <input
              class="control"
              type="number"
              min="0"
              max="100"
              value={settings().advanced.maxFetches}
              onchange={(event) => saveLimits({ maxFetches: clamp(event, 0, 100) })}
            />
          </div>

          <div class="adv-row">
            <span class="adv-label">Max tool rounds</span>
            <input
              class="control"
              type="number"
              min="1"
              max="10"
              value={settings().advanced.maxRounds}
              onchange={(event) => saveLimits({ maxRounds: clamp(event, 1, 10) })}
            />
          </div>

          <p class="muted">“On open” applies at launch and when the window is summoned with the keybind.</p>
        </details>
      {/if}
    </div>
  </Frame>
</div>

<style>
  .settings {
    position: absolute;
    inset: 0;
    z-index: 11;
    display: flex;
    flex-direction: column;
    background: transparent;
    animation: rise-in var(--dur-fast) var(--ease);
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex: none;
    height: 52px;
    padding: 0 var(--sp-4);
    border-bottom: 1px solid var(--border);
  }

  h2 {
    margin: 0;
    font-size: var(--fs-lg);
    font-weight: 600;
  }

  .body {
    flex: 1;
    min-height: 0;
    padding: var(--sp-2);
    overflow-y: auto;
  }

  .section-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-right: var(--sp-2);
  }

  .section-title {
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

  .item {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    width: 100%;
    min-height: 40px;
    padding: 0 var(--sp-2);
    border: none;
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--text);
    font: inherit;
    text-align: left;
    cursor: default;
  }

  .item:hover {
    background: var(--surface-hover);
  }

  .item-name {
    flex: none;
    color: var(--text);
    font-size: var(--fs-sm);
  }

  .item-label {
    overflow: hidden;
    color: var(--text-tertiary);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  .tag {
    flex: none;
    margin-left: auto;
    padding: 0 6px;
    border-radius: var(--radius-sm);
    background: var(--surface-hover);
    color: var(--text-secondary);
    font-size: var(--fs-xs);
  }

  .order {
    margin: var(--sp-1) var(--sp-2);
    color: var(--text-secondary);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
  }

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
    background: var(--surface);
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

  .ghost {
    padding: 4px var(--sp-3);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--surface);
    color: var(--text);
    font: inherit;
    font-size: var(--fs-sm);
    cursor: pointer;
  }

  .muted {
    margin: var(--sp-2);
    color: var(--text-tertiary);
    font-size: var(--fs-sm);
  }
</style>
