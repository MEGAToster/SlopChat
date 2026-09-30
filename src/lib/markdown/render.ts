import DOMPurify from "dompurify";
import { Marked } from "marked";

import { hljs } from "./highlight";

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

const marked = new Marked({
  gfm: true,
  breaks: true,
  renderer: {
    code({ text, lang }) {
      const language = (lang ?? "").split(/\s+/)[0];
      const highlighted =
        language && hljs.getLanguage(language)
          ? hljs.highlight(text, { language }).value
          : escapeHtml(text);

      const className = language ? `hljs language-${language}` : "hljs";
      const label = escapeHtml(language || "text");
      return [
        '<div class="code-block">',
        `<div class="code-head"><span class="code-lang">${label}</span>`,
        '<button class="code-copy" type="button" aria-label="Copy">⧉</button></div>',
        `<pre><code class="${className}">${highlighted}</code></pre>`,
        "</div>",
      ].join("");
    },
  },
});

export function renderMarkdown(source: string): string {
  const html = marked.parse(source, { async: false }) as string;
  return DOMPurify.sanitize(html);
}
