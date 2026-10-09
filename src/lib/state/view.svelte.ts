import { messagesViewport } from "../dom";

export const view = $state({ tracking: true });

// "Jump to latest": scroll to the bottom and resume following.
export function followOutput(): void {
  const viewport = messagesViewport();
  if (viewport) viewport.scrollTop = viewport.scrollHeight;
  view.tracking = true;
}
