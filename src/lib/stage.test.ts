import { describe, expect, it } from "vitest";
import { beamDirection, fixturePosition } from "./stage";

describe("stage fixture calibration", () => {
  it("places the twelve demo fixtures symmetrically on one rear line", () => {
    const first = fixturePosition(0);
    const last = fixturePosition(11);
    expect(first[0]).toBeLessThan(0);
    expect(last[0]).toBe(-first[0]);
    expect(first.slice(1)).toEqual(last.slice(1));
  });

  it("points a centered fixture toward the audience with a unit direction", () => {
    const direction = beamDirection(0.5, 0.5);
    expect(direction[0]).toBeCloseTo(0);
    expect(direction[1]).toBeGreaterThan(0);
    expect(direction[2]).toBeGreaterThan(0.9);
    expect(Math.hypot(...direction)).toBeCloseTo(1);
  });

  it("keeps the additional four fixtures balanced outside the twelve demo fixtures", () => {
    const positions = Array.from({ length: 16 }, (_, index) => fixturePosition(index)[0]);
    expect(positions.reduce((sum, value) => sum + value, 0)).toBeCloseTo(0);
    expect(new Set(positions).size).toBe(16);
  });

  it("mirrors the fan when pan is mirrored", () => {
    const left = beamDirection(0.15, 0.5);
    const right = beamDirection(0.85, 0.5);
    expect(left[0]).toBeCloseTo(-right[0]);
    expect(left[1]).toBeCloseTo(right[1]);
    expect(left[2]).toBeCloseTo(right[2]);
  });

  it("keeps the startup fan inside the reference stage width", () => {
    const left = beamDirection(0.08, 0.5);
    const right = beamDirection(0.92, 0.5);
    expect(Math.abs(left[0] / left[2])).toBeLessThan(0.075);
    expect(Math.abs(right[0] / right[2])).toBeLessThan(0.075);
    expect(left[0]).toBeCloseTo(-right[0]);
    expect(left[2]).toBeGreaterThan(0.9);
  });

  it("keeps out of range incoming positions finite and normalized", () => {
    for (const [pan, tilt] of [[-2, 5], [3, -1]]) {
      const direction = beamDirection(pan, tilt);
      expect(direction.every(Number.isFinite)).toBe(true);
      expect(Math.hypot(...direction)).toBeCloseTo(1);
    }
  });
});
