import { describe, expect, it } from "vitest";
import {
  applyEncoderStep,
  encoderDisplayValue,
  encoderTargets,
  HOME_VIEW,
  viewBack,
} from "./console";
import { faderDefinitions } from "./faders";
import { byteTo16, clamp16, formatChannelValue, formatError } from "./format";
import {
  categoryForKind,
  defaultShapeParams,
  degreesToRepeat,
  displayToSize,
  displayToSpread,
  kindsForCategory,
  primaryKindForCategory,
  repeatToDegrees,
  shapeCategoryList,
  shapeKindList,
  shapeLabel,
  sizeToDisplay,
  spreadToDisplay,
} from "./shapes";
import { SHAPE_KINDS } from "../types";

describe("shape helpers", () => {
  it("keeps the eleven contract shape names in order", () => {
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
      "Dimmer",
      "RGB",
      "Color",
    ]);
  });

  it("uses the original console labels", () => {
    expect(SHAPE_KINDS.map(shapeLabel)).toEqual([
      "Circle",
      "Circle2Arah",
      "Figure8",
      "Spiral",
      "Square",
      "Diamond",
      "PanSwing",
      "TiltSwing",
      "Dimmer",
      "RGB",
      "Color",
    ]);
    expect(shapeLabel("CustomShape")).toBe("CustomShape");
  });

  it("groups shapes into the four video categories", () => {
    expect(shapeCategoryList()).toEqual(["Tilt/Pan", "Dimmer", "RGB", "Color"]);
    expect(kindsForCategory("Tilt/Pan")).toHaveLength(8);
    expect(kindsForCategory("Dimmer")).toEqual(["Dimmer"]);
    expect(categoryForKind("RGB")).toBe("RGB");
    expect(categoryForKind("Unknown")).toBe("Tilt/Pan");
  });

  it("chooses a native kind for every category press", () => {
    expect(primaryKindForCategory("Tilt/Pan")).toBe("Circle");
    expect(primaryKindForCategory("Tilt/Pan", "PanSwing")).toBe("PanSwing");
    expect(primaryKindForCategory("Tilt/Pan", "Dimmer")).toBe("Circle");
    expect(primaryKindForCategory("Dimmer")).toBe("Dimmer");
    expect(primaryKindForCategory("RGB")).toBe("RGB");
    expect(primaryKindForCategory("Color")).toBe("Color");
  });

  it("creates the video defaults for each category", () => {
    expect(defaultShapeParams()).toEqual({
      speed: 1,
      bpm: 6,
      size: 25 / 255,
      spread: 1,
      repeat: 8,
      interleave: "none",
      direction: "left",
      amplitude: 0.5,
    });

    const intensity = defaultShapeParams("Dimmer");
    expect(intensity.bpm).toBe(60);
    expect(intensity.size).toBe(1);
    expect(intensity.repeat).toBe(0);
    expect(intensity.spread).toBe(0);
    expect(defaultShapeParams("RGB")).toEqual(intensity);
    expect(defaultShapeParams("Color")).toEqual(intensity);
  });

  it("maps size, repeat and spread to the display ranges", () => {
    expect(sizeToDisplay(defaultShapeParams().size)).toBe(25);
    expect(repeatToDegrees(8)).toBe(180);
    expect(repeatToDegrees(0)).toBe(0);
    expect(repeatToDegrees(16)).toBe(360);
    expect(degreesToRepeat(180)).toBe(8);
    expect(degreesToRepeat(0)).toBe(0);
    expect(degreesToRepeat(360)).toBe(16);
    expect(spreadToDisplay(defaultShapeParams().spread)).toBe(63);
    expect(displayToSize(255)).toBe(1);
    expect(displayToSpread(63)).toBe(1);
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
  it("maps bank 1 to eight usable channels and eight disabled slots", () => {
    const first = faderDefinitions("fixture", 1, 1);
    expect(first).toHaveLength(16);
    expect(first[0].attribute).toBe("ch1");
    expect(first[7].attribute).toBe("ch8");
    expect(first[15].attribute).toBe("ch16");
    expect(first.slice(0, 8).every((strip) => !strip.disabled)).toBe(true);
    expect(first.slice(8).every((strip) => strip.disabled && strip.label === "-")).toBe(true);
  });

  it("keeps bank 2 attributes relative to the selected fixture", () => {
    const second = faderDefinitions("fixture", 2, 1);
    expect(second[0].channel).toBe(17);
    expect(second[15].channel).toBe(32);
    expect(second[0].attribute).toBe("ch1");
    expect(second[7].attribute).toBe("ch8");
    expect(second[15].attribute).toBe("ch16");
    expect(second.every((strip) => strip.disabled)).toBe(true);
  });

  it("maps playback strips to the active page and slot", () => {
    const strips = faderDefinitions("playback", 1, 3);
    expect(strips).toHaveLength(16);
    expect(strips[0].slot).toBe(1);
    expect(strips[15].slot).toBe(16);
    expect(strips[0].attribute).toBe("playback.3.1");
    expect(strips[0].disabled).toBe(false);
  });
});

describe("console menu helpers", () => {
  it("steps display views back through the shape menu", () => {
    const params = { kind: "shape-params", category: "Tilt/Pan" } as const;
    expect(viewBack(params)).toEqual({ kind: "shape-edit", category: "Tilt/Pan" });
    expect(viewBack(viewBack(params))).toEqual({ kind: "shape-category" });
    expect(viewBack({ kind: "shape-menu" })).toEqual(HOME_VIEW);
  });

  it("assigns encoder pairs and clamps shape parameter ticks", () => {
    expect(encoderTargets("pan-tilt")).toEqual(["pan", "tilt"]);
    expect(encoderTargets("rep-spr")).toEqual(["repeat", "spread"]);
    const params = defaultShapeParams();
    expect(sizeToDisplay(applyEncoderStep(params, "size", 1, "normal").size)).toBe(26);
    expect(applyEncoderStep(params, "speed", 1, "fine").bpm).toBe(6.1);
    expect(applyEncoderStep({ ...params, repeat: 16 }, "repeat", 1, "normal").repeat).toBe(16);
    expect(applyEncoderStep({ ...params, repeat: 0 }, "repeat", -1, "normal").repeat).toBe(0);
  });

  it("renders contextual encoder values", () => {
    const params = defaultShapeParams();
    expect(encoderDisplayValue("pan", params, null, null)).toBe("-----");
    expect(encoderDisplayValue("size", params, null, null)).toBe("25/255");
    expect(encoderDisplayValue("repeat", params, null, null)).toBe("180/360");
    expect(encoderDisplayValue("speed", { ...params, bpm: 6.1 }, null, null)).toBe("6.1 BPM");
    expect(encoderDisplayValue("speed", { ...params, bpm: 6.01 }, null, null)).toBe("6.01 BPM");
    expect(
      encoderDisplayValue("repeat", { ...params, repeat: 0 }, null, null),
    ).toBe("0/360");
  });
});
