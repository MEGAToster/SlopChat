<script lang="ts">
  import { onMount, tick } from "svelte";
  import { removeModel, removeProfile, saveModel, saveProfile } from "../lib/ipc";
  import type { ModelConfig, Profile } from "../lib/types";
  import {
    composedModel,
    modelId,
    registerEditorBack,
  } from "../lib/state/settingsPanel.svelte";
  import { settings } from "../lib/state/settings.svelte";
  import { formatKeys } from "../lib/keymap/format";
  import { moveSettingsFocus } from "../lib/settingsFocus";
  import AdvancedSection from "./AdvancedSection.svelte";
  import AntigravitySection from "./AntigravitySection.svelte";
  import AppearanceSection from "./AppearanceSection.svelte";
  import Frame from "./Frame.svelte";
  import ModelEditor from "./ModelEditor.svelte";
  import ProfileEditor from "./ProfileEditor.svelte";
  import SearchSection from "./SearchSection.svelte";
  import WindowSection from "./WindowSection.svelte";

  let { onclose }: { onclose: () => void } = $props();

  const providers = $derived(Object.keys(settings().providers));
  const models = $derived(settings().models);
  const profiles = $derived(settings().profiles);
  const cycleKeys = $derived(formatKeys(settings().keybinds.cycleProfile));

  let editingModel = $state<ModelConfig | null>(null);
  let modelPrevious = $state<string | null>(null);
  let editingProfile = $state<Profile | null>(null);
  let profilePrevious = $state<string | null>(null);

  function blankModel(): ModelConfig {
    return {
      model: "",
      useContext: true,
      tools: [],
      toolCalling: true,
      webSearch: "auto",
      maxCompletionTokens: 8192,
    };
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
    editingProfile = { name: "", model: models[0]?.model ?? "" };
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

  async function saveProfileEdit(profile: Profile, previous: string | null): Promise<void> {
    await saveProfile(profile, previous ?? undefined);
    editingProfile = null;
  }

  async function deleteProfileEdit(): Promise<void> {
    if (profilePrevious) await removeProfile(profilePrevious);
    editingProfile = null;
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

    // Opened from the palette's "Add models…" with a prefilled provider/model.
    if (modelId()) openModel({ ...blankModel(), model: composedModel() });

    // Put focus inside the panel so keyboard navigation starts here.
    void tick().then(() => moveSettingsFocus(0));

    return () => registerEditorBack(null);
  });
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
          onsave={(profile, previous) => void saveProfileEdit(profile, previous)}
          oncancel={() => (editingProfile = null)}
          onremove={profilePrevious ? () => void deleteProfileEdit() : undefined}
        />
      {:else}
        <AppearanceSection />

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
          <div class="section-title">Profiles{cycleKeys ? ` (${cycleKeys})` : ""}</div>
          <button type="button" class="ghost" onclick={openNewProfile}>Add profile</button>
        </div>
        {#if profiles.length === 0}
          <p class="muted">No profiles yet.</p>
        {:else}
          {#each profiles as entry (entry.name)}
            <button type="button" class="item" onclick={() => openProfile(entry)}>
              <span class="item-name">{entry.name}</span>
              <span class="item-label">{entry.model}</span>
            </button>
          {/each}
        {/if}

        <SearchSection />

        <AntigravitySection />

        <WindowSection />

        <AdvancedSection />
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

  .item:hover,
  .item:focus-visible {
    background: var(--surface-hover);
  }

  .item-name {
    flex: 0 1 auto;
    min-width: 0;
    overflow: hidden;
    color: var(--text);
    font-size: var(--fs-sm);
    text-overflow: ellipsis;
    white-space: nowrap;
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
