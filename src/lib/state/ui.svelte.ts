import type { Profile } from "../types";
import { settings } from "./settings.svelte";

const store = $state({ profileIndex: 0 });

export function activeProfile(): Profile | undefined {
  const profiles = settings().profiles;
  if (profiles.length === 0) return undefined;
  return profiles[store.profileIndex % profiles.length];
}

export function cycleProfile(): void {
  const count = settings().profiles.length;
  if (count > 0) store.profileIndex = (store.profileIndex + 1) % count;
}
