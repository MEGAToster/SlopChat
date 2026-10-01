type KatexApi = {
  renderToString(source: string, options?: Record<string, unknown>): string;
};

let katex: KatexApi | null = null;
let loading: Promise<void> | null = null;

export const mathState = $state({ ready: false });

function ensureKatex(): void {
  if (katex || loading) return;
  loading = (async () => {
    await import("katex/dist/katex.min.css");
    const module = (await import("katex")) as unknown as { default?: KatexApi } & KatexApi;
    katex = module.default ?? module;
    mathState.ready = true;
  })();
}

export function renderMath(source: string, display: boolean): string | null {
  if (!katex) {
    ensureKatex();
    return null;
  }
  try {
    return katex.renderToString(source, {
      displayMode: display,
      throwOnError: false,
    });
  } catch {
    return null;
  }
}
