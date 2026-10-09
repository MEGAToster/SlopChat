import type { Source } from "../types";
import { escapeHtml } from "./escape";

export function applyCitations(html: string, sources: Source[]): string {
  if (sources.length === 0) return html;

  return html.replace(
    /(<code[\s\S]*?<\/code>|<pre[\s\S]*?<\/pre>)|\[(\d+)\]/g,
    (match, code: string | undefined, num: string | undefined) => {
      if (code) return code;
      const index = Number(num) - 1;
      if (index < 0 || index >= sources.length) return match;
      return `<a class="citation" href="${escapeHtml(sources[index].url)}">[${num}]</a>`;
    },
  );
}
