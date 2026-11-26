import { useRef } from "react";
import type { PointerEvent as ReactPointerEvent, WheelEvent as ReactWheelEvent } from "react";
import { commands } from "../ipc";
import {
  applyEncoderStep,
  encoderDisplayValue,
  encoderTargetLabel,
  encoderTargets,
  type EncoderPair,
  type EncoderTarget,
} from "../lib/console";
import { clamp } from "../lib/format";
import { defaultShapeParams } from "../lib/shapes";
import type { EncoderMode, FrameSnapshot, RunCommand } from "../types";

interface EncoderDeckProps {
  frame: FrameSnapshot;
  run: RunCommand;
  pair: EncoderPair;
  onPair: (pair: EncoderPair) => void;
}

const PAIRS: Array<{ value: EncoderPair; label: string }> = [
  { value: "pan-tilt", label: "PAN/TILT" },
  { value: "size-speed", label: "SIZE/BPM" },
  { value: "rep-spr", label: "REP/SPR" },
];

const ENCODER_MODES: EncoderMode[] = ["normal", "fine", "ultra"];

function maximumFor(target: EncoderTarget): number {
  if (target === "pan" || target === "tilt") return 65535;
  if (target === "size") return 255;
  if (target === "speed") return 240;
  if (target === "repeat") return 360;
  return 63;
}

function numericValue(text: string): number {
  const parsed = Number.parseFloat(text);
  return Number.isFinite(parsed) ? parsed : 0;
}

export function EncoderDeck({ frame, run, pair, onPair }: EncoderDeckProps) {
  const dragRef = useRef<{ axis: "a" | "b"; lastX: number; remainder: number } | null>(null);
  const selected = frame.fixtures.find((fixture) => fixture.selected);
  const targets = encoderTargets(pair);

  const valueFor = (axis: "a" | "b") => {
    const target = axis === "a" ? targets[0] : targets[1];
    const pan = selected ? Math.round(selected.pan * 65535) : null;
    const tilt = selected ? Math.round(selected.tilt * 65535) : null;
    return { target, text: encoderDisplayValue(target, frame.shapeParams, pan, tilt) };
  };

  const emit = (axis: "a" | "b", sign: 1 | -1) => {
    const target = axis === "a" ? targets[0] : targets[1];
    if (target === "pan" || target === "tilt") {
      if (!selected) return;
      void run(`encoder-${axis}`, () => commands.setEncoder(axis, sign, frame.encoderMode));
      return;
    }
    const next = applyEncoderStep(
      frame.shapeParams ?? defaultShapeParams(),
      target,
      sign,
      frame.encoderMode,
    );
    void run("shape-encoder", () => commands.setShape(frame.shapeKind, next));
  };

  const onPointerDown = (axis: "a" | "b") => (event: ReactPointerEvent<HTMLButtonElement>) => {
    event.currentTarget.setPointerCapture(event.pointerId);
    dragRef.current = { axis, lastX: event.clientX, remainder: 0 };
  };

  const onPointerMove = (event: ReactPointerEvent<HTMLButtonElement>) => {
    const drag = dragRef.current;
    if (!drag) return;
    drag.remainder += event.clientX - drag.lastX;
    drag.lastX = event.clientX;
    while (Math.abs(drag.remainder) >= 12) {
      const sign = drag.remainder > 0 ? 1 : -1;
      drag.remainder -= sign * 12;
      emit(drag.axis, sign);
    }
  };

  const endDrag = (event: ReactPointerEvent<HTMLButtonElement>) => {
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
    dragRef.current = null;
  };

  const onWheel = (axis: "a" | "b") => (event: ReactWheelEvent<HTMLButtonElement>) => {
    if (event.deltaY === 0) return;
    emit(axis, event.deltaY < 0 ? 1 : -1);
  };

  return (
    <section className="encoder-deck" aria-label="Encoder A dan B">
      <div className="encoder-pair-bar" role="group" aria-label="Fungsi encoder">
        {PAIRS.map((item) => (
          <button
            key={item.value}
            type="button"
            className="kl-key kl-key-mini"
            aria-pressed={pair === item.value}
            onClick={() => onPair(item.value)}
          >
            {item.label}
          </button>
        ))}
      </div>
      <div className="encoder-body">
        {(["a", "b"] as const).map((axis) => {
          const { target, text } = valueFor(axis);
          const maximum = maximumFor(target);
          const numeric = numericValue(text);
          const rotation = clamp(numeric / maximum, 0, 1) * 270 - 135;
          const disabled = (target === "pan" || target === "tilt") && !selected;
          return (
            <div className="encoder-unit" key={axis}>
              <button
                type="button"
                role="slider"
                className="encoder-knob"
                disabled={disabled}
                aria-label={`Encoder ${axis.toUpperCase()} ${encoderTargetLabel(target)}`}
                aria-valuemin={0}
                aria-valuemax={maximum}
                aria-valuenow={numeric}
                aria-valuetext={text}
                onPointerDown={onPointerDown(axis)}
                onPointerMove={onPointerMove}
                onPointerUp={endDrag}
                onPointerCancel={endDrag}
                onWheel={onWheel(axis)}
                onKeyDown={(event) => {
                  if (event.key === "ArrowUp" || event.key === "ArrowRight") {
                    event.preventDefault();
                    emit(axis, 1);
                  } else if (event.key === "ArrowDown" || event.key === "ArrowLeft") {
                    event.preventDefault();
                    emit(axis, -1);
                  }
                }}
              >
                <span className="encoder-marker-track" style={{ transform: `rotate(${rotation}deg)` }}>
                  <span className="encoder-marker" aria-hidden="true" />
                </span>
              </button>
              <div className="encoder-caption">
                <span className="encoder-name">{axis.toUpperCase()} / {encoderTargetLabel(target)}</span>
                <output className="encoder-value num">{text}</output>
              </div>
            </div>
          );
        })}
        <div className="encoder-resolution" role="group" aria-label="Resolusi encoder">
          {ENCODER_MODES.map((mode) => (
            <button
              key={mode}
              type="button"
              className="kl-key kl-key-mini"
              aria-pressed={frame.encoderMode === mode}
              onClick={() => void run("encoder-mode", () => commands.setEncoderMode(mode))}
            >
              {mode === "normal" ? "Normal" : mode === "fine" ? "Fine" : "Ultra"}
            </button>
          ))}
        </div>
      </div>
    </section>
  );
}
