import { describe, expect, it } from "vitest";

import { ResizeGate } from "./resizeGate";

describe("ResizeGate", () => {
  it("accepts the first size", () => {
    const gate = new ResizeGate();
    expect(gate.accept("panel", 500)).toBe(true);
  });

  it("skips a repeated size", () => {
    const gate = new ResizeGate();
    gate.applied("panel", 500);
    expect(gate.accept("panel", 500)).toBe(false);
    expect(gate.accept("panel", 501)).toBe(false);
  });

  it("accepts a real change", () => {
    const gate = new ResizeGate();
    gate.applied("panel", 500);
    expect(gate.accept("panel", 620)).toBe(true);
  });

  it("stops the A to B to A echo", () => {
    const gate = new ResizeGate();
    gate.applied("panel", 500);
    gate.applied("panel", 520);
    expect(gate.accept("panel", 500)).toBe(false);
  });

  it("accepts the same height on a different surface", () => {
    const gate = new ResizeGate();
    gate.applied("panel", 480);
    expect(gate.accept("overlay", 480)).toBe(true);
  });

  it("treats a return as an echo only inside the window", () => {
    const gate = new ResizeGate();
    gate.applied("panel", 118, 1000);
    gate.applied("panel", 480, 1000);
    expect(gate.accept("panel", 118, 1100)).toBe(false);
    expect(gate.accept("panel", 118, 1400)).toBe(true);
  });

  it("forgets the echo history after reset", () => {
    const gate = new ResizeGate();
    gate.applied("panel", 118, 1000);
    gate.applied("panel", 480, 1000);
    gate.reset();
    expect(gate.accept("panel", 118, 1010)).toBe(true);
  });
});
