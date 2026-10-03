import type { Source } from "../types";

// Runs after DOMPurify, so the URL must be escaped here to stop a quote in a
// search result from injecting attributes.
function escapeAttr(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/"/g, "&quot;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

export function applyCitations(html: string, sources: Source[]): string {
  if (sources.length === 0) return html;

  return html.replace(
    /(<code[\s\S]*?<\/code>|<pre[\s\S]*?<\/pre>)|\[(\d+)\]/g,
    (match, code: string | undefined, num: string | undefined) => {
      if (code) return code;
      const index = Number(num) - 1;
      if (index < 0 || index >= sources.length) return match;
      return `<a class="citation" href="${escapeAttr(sources[index].url)}">[${num}]</a>`;
    },
  );
}
