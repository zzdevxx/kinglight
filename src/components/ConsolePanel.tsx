import { useEffect, useMemo, useState } from "react";
import { commands } from "../ipc";
import {
  faderDefinitions,
  MASTER_ATTRIBUTE,
  type ConsoleMode,
  type FaderDefinition,
} from "../lib/faders";
import { byteTo16, clamp16, formatMode16 } from "../lib/format";
import type { EncoderMode, FixtureSnapshot, FrameSnapshot, RunCommand } from "../types";

interface ConsolePanelProps {
  frame: FrameSnapshot;
  run: RunCommand;
  busy: string | null;
  notify: (kind: "ok" | "error", text: string) => void;
}

interface FaderStripProps {
  definition: FaderDefinition;
  value: number;
  disabled?: boolean;
  onValue: (value: number) => void;
}

const ENCODER_MODES: EncoderMode[] = ["normal", "fine", "ultra"];
const ENCODER_STEPS: Record<EncoderMode, number> = {
  normal: 1024,
  fine: 128,
  ultra: 16,
};
const PAGES = [1, 2, 3, 4, 5, 6];
const GROUP_NAMES = ["Off", "A", "B", "C", "D"];

function FaderStrip({ definition, value, disabled = false, onValue }: FaderStripProps) {
  const safeValue = clamp16(value);
  return (
    <div className="fader-strip">
      <output className="fader-value num" aria-label={`${definition.label} value`}>
        {formatMode16(safeValue)}
      </output>
      <input
        className="fader-range"
        type="range"
        min={0}
        max={65535}
        step={1}
        value={safeValue}
        disabled={disabled}
        aria-label={`${definition.label} fader`}
        onChange={(event) => onValue(Number(event.target.value))}
      />
      <span className="fader-label">{definition.label}</span>
      <input
        className="fader-number num"
        type="number"
        min={0}
        max={65535}
        step={1}
        value={safeValue}
        disabled={disabled}
        aria-label={`${definition.label} numeric value`}
        onChange={(event) => onValue(Number(event.target.value))}
      />
    </div>
  );
}

export function ConsolePanel({ frame, run, busy, notify }: ConsolePanelProps) {
  const [mode, setMode] = useState<ConsoleMode>("fixture");
  const [masterValue, setMasterValue] = useState(65535);
  const [playbackValues, setPlaybackValues] = useState<Record<string, number>>({});
  const [encoderMode, setEncoderMode] = useState<EncoderMode>(frame.encoderMode);

  useEffect(() => {
    setEncoderMode(frame.encoderMode);
  }, [frame.encoderMode]);

  const definitions = useMemo(
    () => faderDefinitions(mode, frame.channelBank, frame.activePage),
    [mode, frame.channelBank, frame.activePage],
  );

  const selectedFixture: FixtureSnapshot | undefined = frame.fixtures.find(
    (fixture) => fixture.selected,
  );
  const panValue = selectedFixture ? Math.round(selectedFixture.pan * 65535) : null;
  const tiltValue = selectedFixture ? Math.round(selectedFixture.tilt * 65535) : null;
  const step = ENCODER_STEPS[encoderMode];

  const valueFor = (definition: FaderDefinition): number => {
    if (definition.channel === null) {
      return playbackValues[definition.key] ?? 0;
    }
    return byteTo16(frame.dmx[definition.channel - 1]);
  };

  const setFaderValue = (definition: FaderDefinition, raw: number) => {
    const value = clamp16(raw);
    if (definition.channel === null) {
      setPlaybackValues((current) => ({ ...current, [definition.key]: value }));
    }
    void run(`fader-${definition.key}`, () =>
      commands.setFader(definition.attribute, value),
    );
  };

  const setMaster = (raw: number) => {
    const value = clamp16(raw);
    setMasterValue(value);
    void run("master", () => commands.setFader(MASTER_ATTRIBUTE, value));
  };

  const resetFaders = async () => {
    for (const definition of definitions) {
      await run(`reset-${definition.key}`, () =>
        commands.setFader(definition.attribute, 0),
      );
    }
    setPlaybackValues({});
    notify("ok", "Fader bank set to zero through the engine.");
  };

  const moveEncoder = (axis: "a" | "b", sign: 1 | -1) => {
    if (!selectedFixture) return;
    void run(`encoder-${axis}`, () =>
      commands.setEncoder(axis, sign * step, encoderMode),
    );
  };

  return (
    <section className="panel console-panel" aria-labelledby="console-title">
      <div className="panel-head">
        <div>
          <h2 id="console-title" className="panel-title">Console</h2>
          <p className="panel-sub">Fixture and playback controls</p>
        </div>
        <button
          type="button"
          className="btn"
          disabled={busy !== null}
          onClick={() => void resetFaders()}
        >
          Reset faders
        </button>
      </div>

      <div className="control-block">
        <div className="control-label">Mode</div>
        <div className="segmented" role="group" aria-label="Console mode">
          {(["fixture", "playback"] as ConsoleMode[]).map((item) => (
            <button
              key={item}
              type="button"
              className="btn"
              aria-pressed={mode === item}
              onClick={() => setMode(item)}
            >
              {item === "fixture" ? "Fixture" : "Playback"}
            </button>
          ))}
        </div>
      </div>

      <div className="control-block">
        <div className="control-label">Channel bank</div>
        <div className="segmented" role="group" aria-label="Channel bank">
          {[1, 2].map((bank) => (
            <button
              key={bank}
              type="button"
              className="btn"
              aria-pressed={frame.channelBank === bank}
              onClick={() => void run("bank", () => commands.setBank(bank))}
            >
              {bank === 1 ? "1-16" : "17-32"}
            </button>
          ))}
        </div>
      </div>

      <div className="control-block">
        <div className="control-label">Playback page</div>
        <div className="segmented" role="group" aria-label="Playback page">
          {PAGES.map((page) => (
            <button
              key={page}
              type="button"
              className="btn btn-compact num"
              aria-pressed={frame.activePage === page}
              onClick={() => void run("page", () => commands.setPage(page))}
            >
              {page}
            </button>
          ))}
        </div>
      </div>

      <div className="control-block">
        <div className="control-label">Groups</div>
        <div className="segmented" role="group" aria-label="Fixture groups">
          {GROUP_NAMES.map((name, group) => (
            <button
              key={name}
              type="button"
              className="btn"
              aria-pressed={frame.activeGroup === group}
              onClick={() => void run("group", () => commands.setGroup(group))}
            >
              {name}
            </button>
          ))}
        </div>
      </div>

      {!selectedFixture ? (
        <div className="empty-state" role="status">
          NO FIXTURE SELECTED. Choose a unit for pan and tilt encoders.
        </div>
      ) : null}

      <div className="control-block">
        <div className="control-label">Units</div>
        <div className="unit-grid" role="group" aria-label="Fixture units">
          {frame.fixtures.map((fixture) => (
            <button
              key={fixture.id}
              type="button"
              className="btn btn-unit num"
              aria-pressed={fixture.selected}
              title={`${fixture.label} (slot ${fixture.slot})`}
              onClick={() => void run(`fixture-${fixture.slot}`, () =>
                commands.selectFixture(fixture.slot),
              )}
            >
              {fixture.slot}
            </button>
          ))}
        </div>
      </div>

      <div className="encoder-row">
        {(["a", "b"] as const).map((axis) => {
          const value = axis === "a" ? panValue : tiltValue;
          const degrees = value === null ? 0 : (value / 65535) * 300 - 150;
          return (
            <div className="encoder" key={axis}>
              <div className="encoder-knob" style={{ transform: `rotate(${degrees}deg)` }} aria-hidden="true">
                <span />
              </div>
              <div className="encoder-body">
                <div className="control-label">Encoder {axis.toUpperCase()} / {axis === "a" ? "PAN" : "TILT"}</div>
                <output className="num encoder-value">{value === null ? "-----" : formatMode16(value)}</output>
                <div className="segmented">
                  <button type="button" className="btn btn-compact" disabled={!selectedFixture} onClick={() => moveEncoder(axis, -1)}>
                    Decrease
                  </button>
                  <button type="button" className="btn btn-compact" disabled={!selectedFixture} onClick={() => moveEncoder(axis, 1)}>
                    Increase
                  </button>
                </div>
              </div>
            </div>
          );
        })}
      </div>

      <div className="control-block">
        <div className="control-label">Encoder resolution</div>
        <div className="segmented" role="group" aria-label="Encoder resolution">
          {ENCODER_MODES.map((item) => (
            <button
              key={item}
              type="button"
              className="btn"
              aria-pressed={encoderMode === item}
              onClick={() => setEncoderMode(item)}
            >
              {item === "normal" ? "Normal" : item === "fine" ? "Fine" : "Ultra"}
            </button>
          ))}
        </div>
        <p className="hint num">Step {step} of 65535 per encoder command.</p>
      </div>

      <div className="master-row">
        <FaderStrip
          definition={{ key: "master", label: "MASTER", attribute: MASTER_ATTRIBUTE, channel: null }}
          value={masterValue}
          onValue={setMaster}
        />
      </div>

      <div className="fader-bank" role="group" aria-label="Fader bank">
        {definitions.map((definition) => (
          <FaderStrip
            key={definition.key}
            definition={definition}
            value={valueFor(definition)}
            onValue={(value) => setFaderValue(definition, value)}
          />
        ))}
      </div>
      <p className="hint">
        {mode === "fixture"
          ? "Channel fader values follow the engine DMX frame."
          : "Playback fader values are command drafts until the engine reports them."}
      </p>
    </section>
  );
}
