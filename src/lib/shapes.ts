import {
  SHAPE_KINDS,
  type ShapeDirection,
  type ShapeInterleave,
  type ShapeKind,
  type ShapeParams,
} from "../types";

const LABELS: Record<ShapeKind, string> = {
  Circle: "Circle",
  CircleTwoWay: "Circle two-way",
  Figure8: "Figure 8",
  Spiral: "Spiral",
  Square: "Square",
  Diamond: "Diamond",
  PanSwing: "Pan swing",
  TiltSwing: "Tilt swing",
};

const INTERLEAVES: ShapeInterleave[] = ["none", "even", "odd"];
const DIRECTIONS: ShapeDirection[] = ["left", "right", "bounce", "cross"];

export function shapeLabel(kind: string): string {
  return kind in LABELS ? LABELS[kind as ShapeKind] : kind;
}

export function shapeKindList(): string[] {
  return [...SHAPE_KINDS];
}

export function interleaveList(): ShapeInterleave[] {
  return [...INTERLEAVES];
}

export function directionList(): ShapeDirection[] {
  return [...DIRECTIONS];
}

export function defaultShapeParams(): ShapeParams {
  return {
    speed: 1,
    bpm: null,
    size: 0.4,
    spread: 0.04,
    repeat: 1,
    interleave: "none",
    direction: "left",
    amplitude: 0.4,
  };
}

export function normalizeInterleave(value: string): ShapeInterleave {
  return INTERLEAVES.find((item) => item === value) ?? "none";
}

export function normalizeDirection(value: string): ShapeDirection {
  return DIRECTIONS.find((item) => item === value) ?? "right";
}
