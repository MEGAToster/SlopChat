import { messagesViewport } from "../dom";

const state = $state({ tracking: true });

export function tracking(): boolean {
  return state.tracking;
}

export function setTracking(value: boolean): void {
  state.tracking = value;
}

// At the bottom: toggle following. Scrolled up: jump to present and resume.
export function followOutput(): void {
  const viewport = messagesViewport();
  if (!viewport) return;

  const atBottom = viewport.scrollHeight - viewport.scrollTop - viewport.clientHeight < 80;
  if (atBottom) {
    state.tracking = !state.tracking;
  } else {
    viewport.scrollTop = viewport.scrollHeight;
    state.tracking = true;
  }
}
