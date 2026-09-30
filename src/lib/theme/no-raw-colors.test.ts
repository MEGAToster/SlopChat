import { describe, expect, it } from "vitest";

const components = import.meta.glob("../../components/**/*.svelte", {
  query: "?raw",
  import: "default",
  eager: true,
}) as Record<string, string>;

const app = import.meta.glob("../../App.svelte", {
  query: "?raw",
  import: "default",
  eager: true,
}) as Record<string, string>;

describe("component styles", () => {
  it("reference theme variables instead of raw colors", () => {
    const offenders: string[] = [];

    for (const [path, content] of Object.entries({ ...components, ...app })) {
      const matches = content.match(/#[0-9a-fA-F]{3,8}\b|rgba?\(/g);
      if (matches) offenders.push(`${path}: ${matches.join(", ")}`);
    }

    expect(offenders).toEqual([]);
  });
});
