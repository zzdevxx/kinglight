import { clamp } from "./format";
import {
  displayToSize,
  displayToSpread,
  REPEAT_MAX,
  REPEAT_MIN,
  repeatToDegrees,
  sizeToDisplay,
  spreadToDisplay,
} from "./shapes";
import type {
  ConsoleSlotAction,
  EncoderMode,
  ShapeCategory,
  ShapeParams,
} from "../types";

export type DisplayView =
  | { kind: "home" }
  | { kind: "record" }
  | { kind: "operation-pick" }
  | { kind: "operation"; action: ConsoleSlotAction; source?: number }
  | { kind: "shape-menu" }
  | { kind: "shape-category" }
  | { kind: "shape-edit"; category: ShapeCategory }
  | { kind: "shape-params"; category: ShapeCategory }
  | { kind: "time" }
  | { kind: "setup" };

export type EncoderPair = "pan-tilt" | "size-speed" | "rep-spr";
export type EncoderTarget = "pan" | "tilt" | "size" | "speed" | "repeat" | "spread";

export const HOME_VIEW: DisplayView = { kind: "home" };

export function viewBack(view: DisplayView): DisplayView {
  switch (view.kind) {
    case "shape-params":
      return { kind: "shape-edit", category: view.category };
    case "shape-edit":
      return { kind: "shape-category" };
    case "shape-category":
      return { kind: "shape-menu" };
    case "shape-menu":
    case "record":
    case "operation-pick":
    case "operation":
    case "time":
    case "setup":
      return HOME_VIEW;
    default:
      return HOME_VIEW;
  }
}

export function encoderTargets(pair: EncoderPair): [EncoderTarget, EncoderTarget] {
  if (pair === "size-speed") return ["size", "speed"];
  if (pair === "rep-spr") return ["repeat", "spread"];
  return ["pan", "tilt"];
}

export function encoderTargetLabel(target: EncoderTarget): string {
  switch (target) {
    case "pan":
      return "PAN";
    case "tilt":
      return "TILT";
    case "size":
      return "SIZE";
    case "speed":
      return "BPM";
    case "repeat":
      return "REP";
    case "spread":
      return "SPR";
  }
}

export function encoderDisplayValue(
  target: EncoderTarget,
  params: ShapeParams,
  pan: number | null,
  tilt: number | null,
): string {
  switch (target) {
    case "pan":
      return pan === null ? "-----" : String(Math.round(clamp(pan, 0, 65535)));
    case "tilt":
      return tilt === null ? "-----" : String(Math.round(clamp(tilt, 0, 65535)));
    case "size":
      return `${sizeToDisplay(params.size)}/255`;
    case "speed":
      return `${Number((params.bpm ?? 6).toFixed(2))} BPM`;
    case "repeat":
      return `${repeatToDegrees(params.repeat)}/360`;
    case "spread":
      return `${spreadToDisplay(params.spread)}/63`;
  }
}

function precision(value: number, digits: number): number {
  const factor = 10 ** digits;
  return Math.round(value * factor) / factor;
}

export function applyEncoderStep(
  params: ShapeParams,
  target: EncoderTarget,
  sign: 1 | -1,
  mode: EncoderMode,
): ShapeParams {
  const scale = mode === "normal" ? 1 : mode === "fine" ? 0.1 : 0.01;
  switch (target) {
    case "size":
      return {
        ...params,
        size: displayToSize(precision(sizeToDisplay(params.size) + sign * scale, 2)),
      };
    case "speed":
      return { ...params, bpm: precision(clamp((params.bpm ?? 6) + sign * scale, 1, 240), 2) };
    case "repeat":
      return {
        ...params,
        repeat: Math.round(clamp(params.repeat + sign, REPEAT_MIN, REPEAT_MAX)),
      };
    case "spread":
      return {
        ...params,
        spread: displayToSpread(precision(spreadToDisplay(params.spread) + sign * scale, 2)),
      };
    default:
      return params;
  }
}
