import { describe, expect, it, vi } from "vitest";

// DOMPurify needs a DOM; the fallback path under test skips it anyway.
vi.mock("dompurify", () => ({ default: { sanitize: (html: string) => html } }));

import { renderMarkdown } from "./render";

describe("renderMarkdown", () => {
  it("renders normal markdown", () => {
    expect(renderMarkdown("**hi**")).toContain("<strong>hi</strong>");
  });

  it("still tokenizes math delimiters", () => {
    // KaTeX is lazy, so an unmatched formula surfaces as a pending placeholder.
    expect(renderMarkdown("a $x^2$ b")).toContain("math-pending");
  });

  it("renders oversized content as escaped, truncated plain text", () => {
    const html = renderMarkdown("<script>x</script>".repeat(6000));
    expect(html).toContain('class="markdown-plain"');
    expect(html).toContain("&lt;script&gt;");
    expect(html).not.toContain("<script>");
    expect(html).toContain("markdown-truncated");
    expect(html.length).toBeLessThan(100_000);
  });
});
