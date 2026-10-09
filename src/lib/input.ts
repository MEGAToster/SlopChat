export function clampInt(event: Event, min: number, max: number): number {
  const value = Number((event.currentTarget as HTMLInputElement).value);
  return Math.min(max, Math.max(min, Math.round(value)));
}

export function clampFloat(event: Event, min: number, max: number): number {
  const value = Number((event.currentTarget as HTMLInputElement).value);
  return Math.min(max, Math.max(min, value));
}
