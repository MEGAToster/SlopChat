import type { Routing } from "./types";

// True when at least one routing hint is set; empty routing is omitted from
// the saved model so it stays out of models.json.
export function hasRouting(value: Routing): boolean {
  return Boolean(
    value.order?.length ||
      value.only?.length ||
      value.ignore?.length ||
      value.allowFallbacks !== undefined ||
      value.sort ||
      value.maxPrice?.prompt !== undefined ||
      value.maxPrice?.completion !== undefined,
  );
}
