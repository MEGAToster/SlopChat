<script lang="ts">
  import { activeProfile, cycleProfile, modelOverride } from "../lib/state/ui.svelte";

  const profile = $derived(activeProfile());
  const override = $derived(modelOverride());

  const label = $derived.by(() => {
    const name = profile?.name ?? "no profile";
    if (!override) return name;
    return `${name} · ${override.split("/").pop()}`;
  });
</script>

<button type="button" class="badge" title="Shift+Tab to cycle profile" onclick={cycleProfile}>
  {label}
</button>
