import { describe, expect, it } from "vitest";
import { faderDefinitions } from "./faders";
import { byteTo16, clamp16, formatChannelValue, formatError } from "./format";
import { defaultShapeParams, shapeKindList, shapeLabel } from "./shapes";
import { SHAPE_KINDS } from "../types";

describe("shape helpers", () => {
  it("keeps the eight contract shape names in order", () => {
    expect(shapeKindList()).toEqual([...SHAPE_KINDS]);
    expect(shapeKindList()).toEqual([
      "Circle",
      "CircleTwoWay",
      "Figure8",
      "Spiral",
      "Square",
      "Diamond",
      "PanSwing",
      "TiltSwing",
    ]);
  });

  it("maps every contract shape to a readable label", () => {
    const labels = SHAPE_KINDS.map(shapeLabel);
    expect(new Set(labels).size).toBe(SHAPE_KINDS.length);
    expect(shapeLabel("CircleTwoWay")).toBe("Circle two-way");
    expect(shapeLabel("CustomShape")).toBe("CustomShape");
  });

  it("creates complete shape parameters", () => {
    const params = defaultShapeParams();
    expect(params.bpm).toBeNull();
    expect(params.interleave).toBe("none");
    expect(params.direction).toBe("left");
    expect(Object.keys(params).sort()).toEqual(
      [
        "amplitude",
        "bpm",
        "direction",
        "interleave",
        "repeat",
        "size",
        "spread",
        "speed",
      ].sort(),
    );
  });
});

describe("value formatting", () => {
  it("maps bytes and clamps 16-bit values", () => {
    expect(byteTo16(0)).toBe(0);
    expect(byteTo16(255)).toBe(65535);
    expect(byteTo16(undefined)).toBe(0);
    expect(clamp16(70000)).toBe(65535);
    expect(clamp16(-4)).toBe(0);
  });

  it("formats DMX channel values with fixed width", () => {
    expect(formatChannelValue(7)).toBe("007");
    expect(formatChannelValue(255)).toBe("255");
    expect(formatChannelValue(undefined)).toBe("---");
  });

  it("formats IPC errors without swallowing context", () => {
    expect(formatError(new Error("arm failed"))).toBe("arm failed");
    expect(formatError("load failed")).toBe("load failed");
    expect(formatError({ code: "INVALID_PATCH" })).toBe('{"code":"INVALID_PATCH"}');
  });
});

describe("fader mapping", () => {
  it("maps bank 1 and bank 2 to disjoint channel sets", () => {
    const first = faderDefinitions("fixture", 1, 1);
    const second = faderDefinitions("fixture", 2, 1);
    expect(first).toHaveLength(16);
    expect(second).toHaveLength(16);
    expect(first[0].attribute).toBe("channel.1");
    expect(first[15].attribute).toBe("channel.16");
    expect(second[0].attribute).toBe("channel.17");
    expect(second[15].attribute).toBe("channel.32");
  });

  it("maps playback strips to the active page", () => {
    const strips = faderDefinitions("playback", 1, 3);
    expect(strips[0].attribute).toBe("playback.3.1");
    expect(strips[15].attribute).toBe("playback.3.16");
    expect(strips[0].channel).toBeNull();
  });
});
