export function focusComposer(): void {
  document.querySelector<HTMLTextAreaElement>(".composer-box textarea")?.focus();
}

export function messagesViewport(): HTMLElement | null {
  return document.querySelector<HTMLElement>(".messages");
}
