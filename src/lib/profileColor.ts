import type { Profile } from "./types";

const ORDER = ["purple", "blue", "green", "orange", "pink", "cyan", "yellow", "red"];

export function profileColor(profile: Profile | undefined, index: number): string {
  return profile?.color ?? ORDER[index % ORDER.length];
}
