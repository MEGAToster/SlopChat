import { openUrl } from "./ipc";

export function handleWindowClick(event: MouseEvent): void {
  const target = event.target as HTMLElement | null;

  const copy = target?.closest(".code-copy");
  if (copy instanceof HTMLElement) {
    const code = copy.closest(".code-block")?.querySelector("code")?.textContent ?? "";
    void navigator.clipboard.writeText(code);
    copy.classList.add("copied");
    copy.textContent = "✓";
    setTimeout(() => {
      copy.classList.remove("copied");
      copy.textContent = "⧉";
    }, 1200);
    return;
  }

  const anchor = target?.closest("a[href]");
  if (!(anchor instanceof HTMLAnchorElement)) return;
  const href = anchor.getAttribute("href") ?? "";
  event.preventDefault();
  if (href.startsWith("http://") || href.startsWith("https://") || href.startsWith("mailto:")) {
    void openUrl(href);
  }
}
