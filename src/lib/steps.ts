import type { ToolStep } from "./types";

export function summarizeSteps(steps: ToolStep[]): string {
  let searches = 0;
  let pages = 0;
  let tools = 0;
  let failed = 0;

  for (const step of steps) {
    if (step.name === "web_search") searches += 1;
    else if (step.name === "web_fetch") pages += 1;
    else tools += 1;
    if (step.status === "error") failed += 1;
  }

  const parts: string[] = [];
  if (searches) parts.push(`${searches} ${plural(searches, "search", "searches")}`);
  if (pages) parts.push(`${pages} ${plural(pages, "page", "pages")}`);
  if (tools) parts.push(`${tools} ${plural(tools, "tool", "tools")}`);
  if (failed) parts.push(`${failed} failed`);
  return parts.join(" · ");
}

function plural(count: number, one: string, many: string): string {
  return count === 1 ? one : many;
}
