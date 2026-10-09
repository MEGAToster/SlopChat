export type ResizeMode = "panel" | "overlay";

// Ignore sub-pixel echoes and the A→B→A ping-pong a native resize can cause when
// its own viewport change is fed back through a ResizeObserver as a new measure.
const EPSILON = 2;

// Remembers the last two applied sizes so the panel stops resizing once it has
// settled instead of locking the GTK main loop.
export class ResizeGate {
  private lastMode: ResizeMode | null = null;
  private lastHeight = Number.NEGATIVE_INFINITY;
  private previousMode: ResizeMode | null = null;
  private previousHeight = Number.NEGATIVE_INFINITY;

  accept(mode: ResizeMode, height: number): boolean {
    if (mode !== this.lastMode) return true;
    if (Math.abs(height - this.lastHeight) < EPSILON) return false;
    return !(mode === this.previousMode && Math.abs(height - this.previousHeight) < EPSILON);
  }

  applied(mode: ResizeMode, height: number): void {
    this.previousMode = this.lastMode;
    this.previousHeight = this.lastHeight;
    this.lastMode = mode;
    this.lastHeight = height;
  }
}
