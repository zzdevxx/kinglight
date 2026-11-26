import { useEffect, useState } from "react";
import { commands } from "../ipc";
import {
  defaultShapeParams,
  directionList,
  interleaveList,
  shapeLabel,
  shapeKindList,
} from "../lib/shapes";
import {
  SHAPE_KINDS,
  type FrameSnapshot,
  type RunCommand,
  type ShapeDirection,
  type ShapeInterleave,
  type ShapeParams,
} from "../types";

interface DisplayPanelProps {
  frame: FrameSnapshot;
  run: RunCommand;
}

type ContextView = "fixture" | "shape" | "output" | "dmx" | "show";

const CONTEXT_VIEWS: Array<{ value: ContextView; label: string }> = [
  { value: "fixture", label: "Fixture context" },
  { value: "shape", label: "Shape context" },
  { value: "output", label: "Output context" },
  { value: "dmx", label: "DMX context" },
  { value: "show", label: "Show context" },
];

export function DisplayPanel({ frame, run }: DisplayPanelProps) {
  const [context, setContext] = useState<ContextView>("shape");
  const [kind, setKind] = useState<string>(frame.shapeKind);
  const [params, setParams] = useState<ShapeParams>(defaultShapeParams());

  useEffect(() => {
    if (SHAPE_KINDS.some((item) => item === frame.shapeKind)) {
      setKind(frame.shapeKind);
    }
  }, [frame.shapeKind]);

  const updateParam = <K extends keyof ShapeParams,>(key: K, value: ShapeParams[K]) => {
    setParams((current) => ({ ...current, [key]: value }));
  };

  const updateNumber = (key: "speed" | "size" | "spread" | "repeat" | "amplitude", raw: string) => {
    const parsed = Number(raw);
    if (Number.isFinite(parsed)) updateParam(key, parsed);
  };

  const applyShape = async () => {
    await run("shape-apply", () => commands.setShape(kind, params));
  };

  const playShape = async () => {
    await run("shape-play", async () => {
      await commands.setShape(kind, params);
      return commands.shapePlay(true);
    });
  };

  const stopShape = async () => {
    await run("shape-stop", () => commands.shapePlay(false));
  };

  const contextText = (() => {
    if (context === "fixture") {
      const selected = frame.fixtures.filter((fixture) => fixture.selected);
      const labels = selected.map((fixture) => fixture.label).join(", ");
      return labels
        ? `Selected: ${labels}. Group: ${frame.activeGroup === 0 ? "off" : frame.activeGroup}. Page: ${frame.activePage}.`
        : "No fixture selected. Unit buttons select fixtures for programmer edits.";
    }
    if (context === "shape") {
      return `${frame.shapeKind}: ${shapeLabel(frame.shapeKind)}. Playback state: ${frame.shapePlaying ? "running" : "stopped"}.`;
    }
    if (context === "output") {
      return `${frame.output.armed ? "Armed" : "Disarmed"} / ${frame.output.blackout ? "blackout" : "normal"}. Target ${frame.output.target || "not set"}. Port-Address ${frame.output.portAddress}.`;
    }
    if (context === "dmx") {
      const active = frame.dmx.filter((value) => value > 0).length;
      return `512 byte frame ${frame.frameId}. ${active} of 256 console channels above zero.`;
    }
    return `Active page ${frame.activePage}. Channel bank ${frame.channelBank}. Loaded shows are listed in the show manager.`;
  })();

  return (
    <section className="panel display-panel" aria-labelledby="display-title">
      <div className="panel-head">
        <div>
          <h2 id="display-title" className="panel-title">Display</h2>
          <p className="panel-sub">Context, shape and programmer</p>
        </div>
      </div>

      <div className="control-block">
        <label className="control-label" htmlFor="context-select">Context menu</label>
        <select
          id="context-select"
          className="field"
          value={context}
          onChange={(event) => setContext(event.target.value as ContextView)}
        >
          {CONTEXT_VIEWS.map((item) => (
            <option key={item.value} value={item.value}>{item.label}</option>
          ))}
        </select>
        <p className="context-readout" role="status">{contextText}</p>
      </div>

      <div className="control-block">
        <label className="control-label" htmlFor="shape-kind">Shape kind</label>
        <select
          id="shape-kind"
          className="field"
          value={kind}
          onChange={(event) => setKind(event.target.value)}
        >
          {shapeKindList().map((item) => (
            <option key={item} value={item}>{item}</option>
          ))}
        </select>
        <p className="hint">{shapeLabel(kind)}</p>
      </div>

      <div className="parameter-grid">
        <label className="field-label">
          Speed
          <input
            className="field num"
            type="number"
            step="0.1"
            value={params.speed}
            onChange={(event) => updateNumber("speed", event.target.value)}
          />
        </label>
        <label className="field-label">
          BPM
          <span className="bpm-row">
            <input
              className="field-check"
              type="checkbox"
              checked={params.bpm !== null}
              aria-label="Use BPM"
              onChange={(event) => updateParam("bpm", event.target.checked ? 120 : null)}
            />
            <input
              className="field num"
              type="number"
              step="1"
              disabled={params.bpm === null}
              value={params.bpm ?? 0}
              onChange={(event) => updateParam("bpm", Number(event.target.value))}
            />
          </span>
        </label>
        <label className="field-label">
          Size
          <input
            className="field num"
            type="number"
            step="0.05"
            value={params.size}
            onChange={(event) => updateNumber("size", event.target.value)}
          />
        </label>
        <label className="field-label">
          Spread
          <input
            className="field num"
            type="number"
            step="0.01"
            value={params.spread}
            onChange={(event) => updateNumber("spread", event.target.value)}
          />
        </label>
        <label className="field-label">
          Repeat
          <input
            className="field num"
            type="number"
            step="1"
            value={params.repeat}
            onChange={(event) => updateNumber("repeat", event.target.value)}
          />
        </label>
        <label className="field-label">
          Interleave
          <select
            className="field"
            value={params.interleave}
            onChange={(event) => updateParam("interleave", event.target.value as ShapeInterleave)}
          >
            {interleaveList().map((item) => (
              <option key={item} value={item}>{item}</option>
            ))}
          </select>
        </label>
        <label className="field-label">
          Direction
          <select
            className="field"
            value={params.direction}
            onChange={(event) => updateParam("direction", event.target.value as ShapeDirection)}
          >
            {directionList().map((item) => (
              <option key={item} value={item}>{item}</option>
            ))}
          </select>
        </label>
        <label className="field-label">
          Amplitude
          <input
            className="field num"
            type="number"
            step="0.05"
            value={params.amplitude}
            onChange={(event) => updateNumber("amplitude", event.target.value)}
          />
        </label>
      </div>

      <div className="button-row">
        <button type="button" className="btn" onClick={() => void applyShape()}>
          Apply Shape
        </button>
        <button
          type="button"
          className="btn btn-primary"
          disabled={frame.shapePlaying}
          onClick={() => void playShape()}
        >
          Play Shape
        </button>
        <button
          type="button"
          className="btn"
          disabled={!frame.shapePlaying}
          onClick={() => void stopShape()}
        >
          Stop Shape
        </button>
      </div>

      <div className="control-block">
        <div className="control-label">Programmer</div>
        <p className="hint">
          Shape is the programmer command exposed by the IPC contract. Record, include, copy,
          delete, move, time and setup are not rendered because no matching command exists.
        </p>
      </div>
    </section>
  );
}
