export type EncoderMode = "normal" | "fine" | "ultra";

export interface FixtureSnapshot {
  slot: number;
  id: string;
  label: string;
  selected: boolean;
  pan: number;
  tilt: number;
  dimmer: number;
  color: [number, number, number];
}

export interface OutputStatus {
  armed: boolean;
  blackout: boolean;
  playing: boolean;
  target: string;
  portAddress: number;
  sequence: number;
  sentFrames: number;
  lastError: string | null;
}

export interface FrameSnapshot {
  frameId: number;
  fixtures: FixtureSnapshot[];
  dmx: number[];
  output: OutputStatus;
  activePage: number;
  channelBank: number;
  activeGroup: number;
  encoderMode: EncoderMode;
  shapeKind: string;
  shapePlaying: boolean;
}

export type ShapeInterleave = "none" | "even" | "odd";
export type ShapeDirection = "left" | "right" | "bounce" | "cross";

export interface ShapeParams {
  speed: number;
  bpm: number | null;
  size: number;
  spread: number;
  repeat: number;
  interleave: ShapeInterleave;
  direction: ShapeDirection;
  amplitude: number;
}

export const SHAPE_KINDS = [
  "Circle",
  "CircleTwoWay",
  "Figure8",
  "Spiral",
  "Square",
  "Diamond",
  "PanSwing",
  "TiltSwing",
] as const;

export type ShapeKind = (typeof SHAPE_KINDS)[number];

export type RunCommand = <T>(
  key: string,
  action: () => Promise<T>,
) => Promise<T | undefined>;

export function isFrameSnapshot(value: unknown): value is FrameSnapshot {
  if (typeof value !== "object" || value === null) return false;
  const candidate = value as Partial<FrameSnapshot>;
  return (
    typeof candidate.frameId === "number" &&
    Array.isArray(candidate.fixtures) &&
    Array.isArray(candidate.dmx) &&
    typeof candidate.output === "object" &&
    candidate.output !== null
  );
}
