import { clamp } from "./format";
import {
  SHAPE_CATEGORIES,
  SHAPE_KINDS,
  type ShapeCategory,
  type ShapeDirection,
  type ShapeInterleave,
  type ShapeKind,
  type ShapeParams,
} from "../types";

export const REPEAT_MIN = 0;
export const REPEAT_MAX = 16;

const LABELS: Record<ShapeKind, string> = {
  Circle: "Circle",
  CircleTwoWay: "Circle2Arah",
  Figure8: "Figure8",
  Spiral: "Spiral",
  Square: "Square",
  Diamond: "Diamond",
  PanSwing: "PanSwing",
  TiltSwing: "TiltSwing",
  Dimmer: "Dimmer",
  RGB: "RGB",
  Color: "Color",
};

const CATEGORY_KINDS: Record<ShapeCategory, ShapeKind[]> = {
  "Tilt/Pan": [
    "Circle",
    "CircleTwoWay",
    "Figure8",
    "Spiral",
    "Square",
    "Diamond",
    "PanSwing",
    "TiltSwing",
  ],
  Dimmer: ["Dimmer"],
  RGB: ["RGB"],
  Color: ["Color"],
};

const INTERLEAVES: ShapeInterleave[] = ["none", "even", "odd"];
const DIRECTIONS: ShapeDirection[] = ["left", "right", "bounce", "cross"];

export function shapeLabel(kind: string): string {
  return kind in LABELS ? LABELS[kind as ShapeKind] : kind;
}

export function shapeKindList(): string[] {
  return [...SHAPE_KINDS];
}

export function shapeCategoryList(): ShapeCategory[] {
  return [...SHAPE_CATEGORIES];
}

export function kindsForCategory(category: ShapeCategory): ShapeKind[] {
  return [...CATEGORY_KINDS[category]];
}

export function categoryForKind(kind: string): ShapeCategory {
  const match = SHAPE_CATEGORIES.find((category) =>
    CATEGORY_KINDS[category].some((item) => item === kind),
  );
  return match ?? "Tilt/Pan";
}

export function primaryKindForCategory(
  category: ShapeCategory,
  currentKind?: string,
): ShapeKind {
  if (category === "Tilt/Pan") {
    const current = currentKind as ShapeKind | undefined;
    return current && CATEGORY_KINDS["Tilt/Pan"].includes(current) ? current : "Circle";
  }
  return category;
}

export function interleaveList(): ShapeInterleave[] {
  return [...INTERLEAVES];
}

export function directionList(): ShapeDirection[] {
  return [...DIRECTIONS];
}

export function defaultShapeParams(category: ShapeCategory = "Tilt/Pan"): ShapeParams {
  if (category === "Tilt/Pan") {
    return {
      speed: 1,
      bpm: 6,
      size: 25 / 255,
    spread: 1,
      repeat: 8,
      interleave: "none",
      direction: "left",
      amplitude: 0.5,
    };
  }
  return {
    speed: 1,
    bpm: 60,
    size: 1,
    spread: 0,
    repeat: 0,
    interleave: "none",
    direction: "left",
    amplitude: 0.5,
  };
}

export function normalizeInterleave(value: string): ShapeInterleave {
  return INTERLEAVES.find((item) => item === value) ?? "none";
}

export function normalizeDirection(value: string): ShapeDirection {
  return DIRECTIONS.find((item) => item === value) ?? "right";
}

export function sizeToDisplay(size: number): number {
  return Math.round(clamp(size, 0, 1) * 255);
}

export function displayToSize(value: number): number {
  return clamp(value, 0, 255) / 255;
}

export function repeatToDegrees(repeat: number): number {
  return Math.round((clamp(repeat, REPEAT_MIN, REPEAT_MAX) / REPEAT_MAX) * 360);
}

export function degreesToRepeat(degrees: number): number {
  return Math.round((clamp(degrees, 0, 360) / 360) * REPEAT_MAX);
}

export function spreadToDisplay(spread: number): number {
  return Math.round(clamp(spread, 0, 1) * 63);
}

export function displayToSpread(value: number): number {
  return clamp(value, 0, 63) / 63;
}
