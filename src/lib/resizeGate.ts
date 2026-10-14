export type ResizeMode = "panel" | "overlay";

// Ignore sub-pixel echoes and the A→B→A ping-pong a native resize can cause when
// its own viewport change is fed back through a ResizeObserver as a new measure.
const EPSILON = 2;

// An echo returns within a frame or two of the resize that caused it, so a
// return to an earlier height after this window is a real content change.
const ECHO_WINDOW_MS = 250;

// Remembers the last two applied sizes so the panel stops resizing once it has
// settled instead of locking the GTK main loop.
export class ResizeGate {
  private lastMode: ResizeMode | null = null;
  private lastHeight = Number.NEGATIVE_INFINITY;
  private lastAt = Number.NEGATIVE_INFINITY;
  private previousMode: ResizeMode | null = null;
  private previousHeight = Number.NEGATIVE_INFINITY;

  accept(mode: ResizeMode, height: number, now = performance.now()): boolean {
    if (mode !== this.lastMode) return true;
    if (Math.abs(height - this.lastHeight) < EPSILON) return false;
    const echo = now - this.lastAt < ECHO_WINDOW_MS;
    return !(echo && mode === this.previousMode && Math.abs(height - this.previousHeight) < EPSILON);
  }

  applied(mode: ResizeMode, height: number, now = performance.now()): void {
    this.previousMode = this.lastMode;
    this.previousHeight = this.lastHeight;
    this.lastMode = mode;
    this.lastHeight = height;
    this.lastAt = now;
  }

  // Dropped when a size change is known to be content-driven (e.g. a history
  // walk ending), so its measurement is never mistaken for an echo.
  reset(): void {
    this.lastMode = null;
    this.lastHeight = Number.NEGATIVE_INFINITY;
    this.lastAt = Number.NEGATIVE_INFINITY;
    this.previousMode = null;
    this.previousHeight = Number.NEGATIVE_INFINITY;
  }
}
