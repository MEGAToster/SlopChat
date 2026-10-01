import { messagesViewport } from "../dom";

export const view = $state({ tracking: true });

// At the bottom: toggle following. Scrolled up: jump to present and resume.
export function followOutput(): void {
  const viewport = messagesViewport();
  // No message list (empty/new chat): nothing to jump to, just toggle.
  if (!viewport) {
    view.tracking = !view.tracking;
    return;
  }

  const atBottom = viewport.scrollHeight - viewport.scrollTop - viewport.clientHeight < 80;
  if (atBottom) {
    view.tracking = !view.tracking;
  } else {
    viewport.scrollTop = viewport.scrollHeight;
    view.tracking = true;
  }
}
