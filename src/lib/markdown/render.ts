import DOMPurify from "dompurify";
import { Marked, type MarkedExtension, type Tokens } from "marked";

import { stripControlTokens } from "./clean";
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

function mathExtension(
  name: string,
  display: boolean,
  pattern: RegExp,
  marker: string,
): MarkedExtension {
  return {
    extensions: [
      {
        // Inline so display math mid-sentence still matches; `display` only
        // controls KaTeX's display mode.
        name,
        level: "inline",
        start(source: string) {
          const index = source.indexOf(marker);
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

function isDocument(text: string): boolean {
  return /\\(documentclass|usepackage|begin\{document\}|end\{document\})/.test(text);
}

// A fence that mixes prose with \(...\)/\[...\]/$...$ math isn't one expression.
function hasMathDelimiters(text: string): boolean {
  return /\\[(\[]|\${1,2}/.test(text);
}

// KaTeX can't render a full document, so unwrap the scaffold and let the
// markdown/math pipeline render the body instead of showing a wall of TeX.
function latexBody(text: string): string {
  const body = /\\begin\{document\}([\s\S]*?)\\end\{document\}/.exec(text);
  return (body ? body[1] : text)
    .replace(/\\(documentclass|usepackage)(\[[^\]]*\])?\{[^}]*\}/g, "")
    .replace(/\\subsection\*?\{([^}]*)\}/g, "### $1")
    .replace(/\\section\*?\{([^}]*)\}/g, "## $1")
    .replace(/\\(title|author|date)\*?\{[^}]*\}/g, "")
    .replace(/\\(maketitle|newpage|clearpage|tableofcontents)/g, "")
    .trim();
}

function stripMathDelimiters(text: string): string {
  return text
    .trim()
    .replace(/^\$\$?|^\s*\\?[\[(]/, "")
    .replace(/\$\$?$|\\?[\])]\s*$/, "")
    .trim();
}

const marked = new Marked(
  mathExtension("blockMath", true, /^\$\$([\s\S]+?)\$\$/, "$$"),
  mathExtension("inlineMath", false, /^\$(?!\s)((?:\\.|[^$\\])+?)(?<!\s)\$/, "$"),
  mathExtension("blockBracket", true, /^\\\[([\s\S]+?)\\\]/, "\\["),
  mathExtension("inlineParen", false, /^\\\(([\s\S]+?)\\\)/, "\\("),
  {
    gfm: true,
    breaks: true,
    renderer: {
      // Wrap tables so wide ones scroll instead of overflowing the transcript.
      table(token: Tokens.Table): string {
        let head = "";
        for (const cell of token.header) head += this.tablecell(cell);
        let rows = "";
        for (const row of token.rows) {
          let cells = "";
          for (const cell of row) cells += this.tablecell(cell);
          rows += this.tablerow({ text: cells });
        }
        const body = rows ? `<tbody>${rows}</tbody>` : "";
        const table = `<table><thead>${this.tablerow({ text: head })}</thead>${body}</table>`;
        return `<div class="table-wrap">${table}</div>`;
      },
      code({ text, lang }): string {
        const language = (lang ?? "").split(/\s+/)[0];
        const lower = language.toLowerCase();

        if (lower === "mermaid") {
          return [
            `<div class="code-block mermaid-block" data-source="${escapeHtml(text)}">`,
            '<div class="code-head"><span class="code-lang">mermaid</span>',
            '<button class="code-copy" type="button" aria-label="Copy">⧉</button></div>',
            `<pre class="mermaid">${escapeHtml(text)}</pre>`,
            "</div>",
          ].join("");
        }

        if (["math", "latex", "tex"].includes(lower)) {
          const document = isDocument(text);
          if (document || hasMathDelimiters(text)) {
            return marked.parse(document ? latexBody(text) : text, { async: false }) as string;
          }
          return mathHtml(stripMathDelimiters(text), true);
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
  const html = marked.parse(stripControlTokens(source), { async: false }) as string;
  return DOMPurify.sanitize(html);
}
