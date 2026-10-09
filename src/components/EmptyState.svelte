<script lang="ts">
  import { shortModel } from "../lib/modelRef";
  import { profileColor } from "../lib/profileColor";
  import { reasoningEffort, webMode } from "../lib/status";
  import { settings } from "../lib/state/settings.svelte";
  import { activeConfig, activeProfile, activeProfileIndex } from "../lib/state/ui.svelte";
  import Kbd from "./Kbd.svelte";

  const profile = $derived(activeProfile());
  const config = $derived(activeConfig());
  const color = $derived(profileColor(profile, activeProfileIndex()));
  const model = $derived(shortModel(config?.model ?? ""));
  const reasoning = $derived(reasoningEffort(config));
  const web = $derived(webMode(config));
  const keybinds = $derived(settings().keybinds);
</script>

<div class="empty">
  <div class="line">
    <span class="dot" style={`background: var(--c-${color})`}></span>
    <span class="name">{profile?.name ?? "no profile"}</span>
    {#if model}<span class="model">{model}</span>{/if}
  </div>

  <div class="line muted">
    <span>{reasoning} reasoning</span>
    <span>·</span>
    <span>{web === "off" ? "web off" : web === "deep" ? "deep research" : "web auto"}</span>
  </div>

  <div class="line hints">
    <span><Kbd binding={keybinds.palette?.[0]} /> palette</span>
    <span><Kbd binding={keybinds.cycleProfile?.[0]} /> profile</span>
    <span><Kbd binding={keybinds.models?.[0]} /> model</span>
  </div>
</div>

<style>
  .empty {
    display: flex;
    flex: 1;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--sp-2);
    min-height: 0;
    color: var(--text-tertiary);
    font-size: var(--fs-sm);
  }

  .line {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-1);
  }

  .dot {
    flex: none;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent);
  }

  .name {
    color: var(--text-secondary);
    font-weight: 500;
  }

  .model {
    color: var(--text-tertiary);
  }

  .hints {
    gap: var(--sp-3);
  }

  .hints span {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }
</style>
