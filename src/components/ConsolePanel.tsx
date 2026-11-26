import { useState } from "react";
import { DisplayPanel } from "./DisplayPanel";
import { EncoderDeck } from "./EncoderDeck";
import { commands } from "../ipc";
import type { DisplayView, EncoderPair } from "../lib/console";
import { faderDefinitions, MASTER_ATTRIBUTE, SLOT_NUMBERS, type ConsoleMode } from "../lib/faders";
import { clamp } from "../lib/format";
import type { FrameSnapshot, RunCommand } from "../types";

interface ConsolePanelProps {
  frame: FrameSnapshot;
  run: RunCommand;
  compact: boolean;
  view: DisplayView;
  encoderPair: EncoderPair;
  onCompact: (compact: boolean) => void;
  onView: (view: DisplayView) => void;
  onSlot: (slot: number) => void;
  onEnter: () => void;
  onConsoleAction: (action: string, slot?: number, source?: number) => void;
  onPair: (pair: EncoderPair) => void;
  onOpenDialog: (name: "show" | "artnet" | "dmx") => void;
}

const PAGES = [1, 2, 3, 4, 5, 6];
const GROUPS = [
  { value: 0, label: "Off" },
  { value: 1, label: "A" },
  { value: 2, label: "B" },
  { value: 3, label: "C" },
  { value: 4, label: "D" },
];

function toByte(value16: number | undefined): number {
  if (typeof value16 !== "number" || !Number.isFinite(value16)) return 0;
  return Math.round(clamp(value16, 0, 65535) / 257);
}

export function ConsolePanel({
  frame,
  run,
  compact,
  view,
  encoderPair,
  onCompact,
  onView,
  onSlot,
  onEnter,
  onConsoleAction,
  onPair,
  onOpenDialog,
}: ConsolePanelProps) {
  const [mode, setMode] = useState<ConsoleMode>("fixture");
  const definitions = faderDefinitions(mode, frame.channelBank, frame.activePage);
  const noFixture = mode === "fixture" && !frame.fixtures.some((fixture) => fixture.selected);
  const awaitingSlot = view.kind === "record" || view.kind === "operation";
  const levels = frame.playbackLevels ?? [];
  const channels = frame.channelValues ?? [];
  const recorded = frame.recordedSlots ?? [];

  const sendByte = (attribute: string, byte: number) => {
    void run(attribute, () => commands.setFader(attribute, byte * 257));
  };

  return (
    <section className={`console-deck${compact ? " is-compact" : ""}`} aria-label="Konsol Kinglight">
      <div className="console-faders" aria-hidden={compact}>
        <div className="console-row console-top-row">
          <div className="console-cluster" role="group" aria-label="Mode konsol">
            <button
              type="button"
              className="kl-key kl-key-blue"
              aria-pressed={mode === "fixture"}
              onClick={() => setMode("fixture")}
            >
              Fixture
            </button>
            <button
              type="button"
              className="kl-key kl-key-blue"
              aria-pressed={mode === "playback"}
              onClick={() => setMode("playback")}
            >
              Playback
            </button>
            <button
              type="button"
              className="kl-key kl-key-blue"
              aria-pressed={frame.channelBank === 1}
              onClick={() => void run("bank", () => commands.setBank(1))}
            >
              1-16
            </button>
            <button
              type="button"
              className="kl-key kl-key-blue"
              aria-pressed={frame.channelBank === 2}
              onClick={() => void run("bank", () => commands.setBank(2))}
            >
              17-32
            </button>
          </div>
          <div className="console-cluster page-keys" role="group" aria-label="Halaman playback">
            <span className="cluster-label">PLAYBACK PAGE</span>
            <div className="page-key-row">
              {PAGES.map((page) => (
                <button
                  key={page}
                  type="button"
                  className="kl-key kl-key-blue kl-key-mini num"
                  aria-pressed={frame.activePage === page}
                  onClick={() => void run("page", () => commands.setPage(page))}
                >
                  {page}
                </button>
              ))}
            </div>
          </div>
          <div className="console-cluster operation-keys" role="group" aria-label="Operasi programmer">
            <button
              type="button"
              className="kl-key kl-key-blue"
              onClick={() => onView({ kind: "operation", action: "move" })}
            >
              Move
            </button>
            <button
              type="button"
              className="kl-key kl-key-blue"
              onClick={() => onView({ kind: "operation-pick" })}
            >
              Copy/Inc
            </button>
            <button
              type="button"
              className="kl-key kl-key-blue"
              onClick={() => onView({ kind: "operation", action: "delete" })}
            >
              Del/Off
            </button>
            <button
              type="button"
              className="kl-key kl-key-blue"
              onClick={() => onView({ kind: "time" })}
            >
              Time
            </button>
            <button
              type="button"
              className="kl-key kl-key-blue"
              onClick={() => {
                onView({ kind: "shape-menu" });
                onCompact(true);
              }}
            >
              Shape
            </button>
            <button
              type="button"
              className="kl-key kl-key-blue"
              onClick={() => onView({ kind: "setup" })}
            >
              Setup
            </button>
          </div>
        </div>

        <div className="console-row console-group-row">
          <div className="console-cluster" role="group" aria-label="Grup fixture">
            {GROUPS.map((group) => (
              <button
                key={group.label}
                type="button"
                className="kl-key kl-key-blue kl-key-mini"
                aria-pressed={frame.activeGroup === group.value}
                onClick={() => void run("group", () => commands.setGroup(group.value))}
              >
                {group.label}
              </button>
            ))}
            <button
              type="button"
              className="kl-key kl-key-blue kl-key-mini"
              onClick={() => onConsoleAction("select_all")}
            >
              Sel
            </button>
            <button
              type="button"
              className="kl-key kl-key-blue kl-key-mini"
              onClick={() => onConsoleAction("locate")}
            >
              Loc
            </button>
          </div>
        </div>

        <div className="console-row kl-unit-row">
          <button
            type="button"
            className={`kl-key kl-unit-key kl-blackout${frame.output.blackout ? " is-on" : ""}`}
            aria-pressed={frame.output.blackout}
            onClick={() => void run("blackout", () => commands.setBlackout(!frame.output.blackout))}
          >
            BLACKOUT
          </button>
          {SLOT_NUMBERS.map((slot) => {
            const selected = frame.fixtures.some((fixture) => fixture.slot === slot && fixture.selected);
            const cued = recorded[slot - 1] === true;
            return (
              <button
                key={slot}
                type="button"
                className={`kl-key kl-unit-key num ${slot <= 12 ? "is-standard" : "is-extended"}${selected ? " is-selected" : ""}${cued ? " is-cued" : ""}`}
                aria-pressed={selected}
                aria-label={awaitingSlot ? `Pilih slot ${slot}` : `Unit ${slot}`}
                onClick={() => {
                  if (awaitingSlot) {
                    onSlot(slot);
                  } else if (mode === "fixture") {
                    void run(`fixture-${slot}`, () => commands.selectFixture(slot));
                  } else {
                    onConsoleAction("locate", slot);
                  }
                }}
              >
                {String(slot).padStart(2, "0")}
              </button>
            );
          })}
        </div>

        <div className="kl-fader-bank" role="group" aria-label="Master dan 16 fader">
          <div className="kl-fader-strip is-master">
            <output className="kl-fader-value num" aria-label="Master value">
              {String(toByte(frame.master)).padStart(3, "0")}
            </output>
            <input
              className="kl-fader-track"
              type="range"
              min={0}
              max={255}
              step={1}
              value={toByte(frame.master)}
              aria-label="Fader master"
              onChange={(event) => sendByte(MASTER_ATTRIBUTE, Number(event.target.value))}
            />
            <button
              type="button"
              className="kl-key kl-fader-key"
              onClick={() => sendByte(MASTER_ATTRIBUTE, 255)}
            >
              MSTR
            </button>
          </div>
          {definitions.map((definition) => {
            const byte =
              definition.channel === null
                ? toByte(levels[definition.index])
                : channels[definition.index] ?? 0;
            const unavailable = definition.disabled || noFixture;
            return (
              <div className="kl-fader-strip" key={definition.key}>
                <output className="kl-fader-value num" aria-label={`${definition.label} value`}>
                  {unavailable
                    ? "---"
                    : String(Math.round(clamp(byte, 0, 255))).padStart(3, "0")}
                </output>
                <input
                  className="kl-fader-track"
                  type="range"
                  min={0}
                  max={255}
                  step={1}
                  value={Math.round(clamp(byte, 0, 255))}
                  disabled={unavailable}
                  aria-label={`Fader ${definition.label}`}
                  onChange={(event) => {
                    const value = Number(event.target.value);
                    if (definition.channel === null && definition.slot !== null) {
                      void run(`playback-${definition.slot}`, () =>
                        commands.setPlayback(definition.slot as number, value * 257),
                      );
                    } else {
                      sendByte(definition.attribute, value);
                    }
                  }}
                />
                <button
                  type="button"
                  className="kl-key kl-fader-key"
                  disabled={unavailable}
                  aria-label={`Tombol ${definition.label}`}
                  onClick={() => {
                    if (mode === "fixture") {
                      void run(`fixture-${definition.index + 1}`, () =>
                        commands.selectFixture(definition.index + 1),
                      );
                    } else if (definition.slot !== null) {
                      if (awaitingSlot) onSlot(definition.slot);
                      else onConsoleAction("locate", definition.slot);
                    }
                  }}
                >
                  {definition.label}
                </button>
              </div>
            );
          })}
        </div>
        <button
          type="button"
          className="kl-compact-handle"
          aria-label="Tampilkan panggung penuh"
          onClick={() => onCompact(true)}
        >
          Panggung
        </button>
      </div>

      <aside className="console-side">
        <DisplayPanel
          frame={frame}
          run={run}
          view={view}
          onView={onView}
          onSlot={onSlot}
          onEnter={onEnter}
          onConsoleAction={onConsoleAction}
          onOpenDialog={onOpenDialog}
        />
        <EncoderDeck frame={frame} run={run} pair={encoderPair} onPair={onPair} />
        {compact ? (
          <button type="button" className="kl-compact-handle" onClick={() => onCompact(false)}>
            Konsol penuh
          </button>
        ) : null}
      </aside>
    </section>
  );
}
