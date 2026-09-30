type MermaidApi = {
  initialize(options: Record<string, unknown>): void;
  render(id: string, source: string): Promise<{ svg: string }>;
};

let mermaid: MermaidApi | null = null;
let loading: Promise<void> | null = null;

export async function renderMermaid(container: HTMLElement): Promise<void> {
  const nodes = container.querySelectorAll<HTMLElement>("pre.mermaid:not([data-done])");
  if (nodes.length === 0) return;

  if (!mermaid) {
    if (!loading) {
      loading = import("mermaid").then((module) => {
        const api = (module as unknown as { default: MermaidApi }).default;
        api.initialize({
          startOnLoad: false,
          securityLevel: "strict",
          theme: window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "default",
        });
        mermaid = api;
      });
    }
    await loading;
  }

  for (const node of nodes) {
    node.setAttribute("data-done", "1");
    const source = node.textContent ?? "";
    try {
      const { svg } = await mermaid!.render(`mermaid-${Math.random().toString(36).slice(2)}`, source);
      const wrapper = document.createElement("div");
      wrapper.innerHTML = svg;
      node.replaceWith(wrapper.firstElementChild ?? wrapper);
    } catch (error) {
      const failure = document.createElement("div");
      failure.className = "mermaid-error";
      failure.textContent = String(error);
      node.replaceWith(failure);
    }
  }
}
