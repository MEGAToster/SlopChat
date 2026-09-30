import { renderMarkdown } from "./render";

export function splitBlocks(source: string): string[] {
  const blocks: string[] = [];
  let current: string[] = [];
  let fence = false;

  for (const line of source.split("\n")) {
    if (/^\s*(```|~~~)/.test(line)) fence = !fence;
    if (!fence && line.trim() === "") {
      if (current.length > 0) {
        blocks.push(current.join("\n"));
        current = [];
      }
    } else {
      current.push(line);
    }
  }

  if (current.length > 0) blocks.push(current.join("\n"));
  return blocks;
}

export class MarkdownStream {
  private sources: string[] = [];
  private html: string[] = [];

  constructor(private render: (source: string) => string = renderMarkdown) {}

  update(source: string): string {
    const blocks = splitBlocks(source);

    for (let index = 0; index < blocks.length; index += 1) {
      const isLast = index === blocks.length - 1;
      if (!isLast && this.sources[index] === blocks[index]) continue;
      this.sources[index] = blocks[index];
      this.html[index] = this.render(blocks[index]);
    }

    this.sources.length = blocks.length;
    this.html.length = blocks.length;
    return this.html.join("\n");
  }
}
