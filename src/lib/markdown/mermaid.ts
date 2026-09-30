type MermaidApi = {
  initialize(options: Record<string, unknown>): void;
  render(id: string, source: string): Promise<{ svg: string }>;
};

let loading: Promise<MermaidApi> | null = null;

function load(): Promise<MermaidApi> {
  if (!loading) {
    loading = import("mermaid")
      .then((module) => {
        const api = (module as unknown as { default: MermaidApi }).default;
        api.initialize({
          startOnLoad: false,
          securityLevel: "strict",
          theme: window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "default",
        });
        return api;
      })
      .catch((error) => {
        loading = null;
        throw error;
      });
  }
  return loading;
}

export async function renderMermaid(container: HTMLElement): Promise<void> {
  if (!container.querySelector("pre.mermaid:not([data-done])")) return;

  let api: MermaidApi;
  try {
    api = await load();
  } catch (error) {
    fail(container, error);
    return;
  }

  // The node list must be read after the async load: a Svelte re-render may
  // have replaced the previous nodes while mermaid was loading.
  const nodes = container.querySelectorAll<HTMLElement>("pre.mermaid:not([data-done])");
  for (const node of nodes) {
    node.setAttribute("data-done", "1");
    const source = node.textContent ?? "";
    try {
      const { svg } = await api.render(`mermaid-${Math.random().toString(36).slice(2)}`, source);
      const wrapper = document.createElement("div");
      wrapper.innerHTML = svg;
      node.replaceWith(wrapper.firstElementChild ?? wrapper);
    } catch (error) {
      node.replaceWith(message(String(error)));
    }
  }
}

function fail(container: HTMLElement, error: unknown): void {
  for (const node of container.querySelectorAll<HTMLElement>("pre.mermaid:not([data-done])")) {
    node.setAttribute("data-done", "1");
    node.replaceWith(message(String(error)));
  }
}

function message(text: string): HTMLElement {
  const failure = document.createElement("div");
  failure.className = "mermaid-error";
  failure.textContent = text;
  return failure;
}
