import DOMPurify from "dompurify";
import { Marked, type MarkedExtension, type Tokens } from "marked";

import { hljs } from "./highlight";
import { renderMath } from "./math.svelte";

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function mathHtml(source: string, display: boolean): string {
  const html = renderMath(source, display);
  return html ?? `<span class="math-pending">${escapeHtml(source)}</span>`;
}

function mathExtension(name: string, display: boolean, pattern: RegExp): MarkedExtension {
  return {
    extensions: [
      {
        name,
        level: display ? "block" : "inline",
        start(source: string) {
          const index = source.indexOf(display ? "$$" : "$");
          return index < 0 ? undefined : index;
        },
        tokenizer(source: string) {
          const match = pattern.exec(source);
          if (!match) return undefined;
          return { type: name, raw: match[0], text: match[1] } as Tokens.Generic;
        },
        renderer(token: Tokens.Generic) {
          return mathHtml(String(token.text ?? ""), display);
        },
      },
    ],
  };
}

const marked = new Marked(
  mathExtension("blockMath", true, /^\$\$([\s\S]+?)\$\$/),
  mathExtension("inlineMath", false, /^\$(?!\s)((?:\\.|[^$\\])+?)(?<!\s)\$/),
  {
    gfm: true,
    breaks: true,
    renderer: {
      code({ text, lang }) {
        const language = (lang ?? "").split(/\s+/)[0];

        if (language === "mermaid") {
          return [
            '<div class="code-block mermaid-block">',
            '<div class="code-head"><span class="code-lang">mermaid</span>',
            '<button class="code-copy" type="button" aria-label="Copy">⧉</button></div>',
            `<pre class="mermaid">${escapeHtml(text)}</pre>`,
            "</div>",
          ].join("");
        }

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
  },
);

export function renderMarkdown(source: string): string {
  const html = marked.parse(source, { async: false }) as string;
  return DOMPurify.sanitize(html);
}
