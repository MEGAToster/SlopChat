import type { Profile } from "./types";

export const PROFILE_COLORS = ["purple", "blue", "green", "orange", "pink", "cyan", "yellow", "red"];

export function profileColor(profile: Profile | undefined, index: number): string {
  return profile?.color ?? PROFILE_COLORS[index % PROFILE_COLORS.length];
}
