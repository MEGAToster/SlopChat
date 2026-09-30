<script lang="ts">
  import { activeProfile, cycleProfile } from "../lib/state/ui.svelte";

  const profile = $derived(activeProfile());
</script>

<button type="button" class="badge" title="Shift+Tab to cycle profile" onclick={cycleProfile}>
  {profile?.name ?? "no profile"}
</button>
