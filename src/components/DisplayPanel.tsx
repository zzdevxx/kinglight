import { useState } from "react";
import { commands } from "../ipc";
import { HOME_VIEW, viewBack, type DisplayView } from "../lib/console";
import { clamp } from "../lib/format";
import {
  categoryForKind,
  defaultShapeParams,
  degreesToRepeat,
  directionList,
  displayToSize,
  displayToSpread,
  interleaveList,
  kindsForCategory,
  primaryKindForCategory,
  repeatToDegrees,
  shapeCategoryList,
  shapeLabel,
  sizeToDisplay,
  spreadToDisplay,
} from "../lib/shapes";
import type {
  FrameSnapshot,
  RunCommand,
  ShapeCategory,
  ShapeDirection,
  ShapeInterleave,
  ShapeParams,
} from "../types";

interface DisplayPanelProps {
  frame: FrameSnapshot;
  run: RunCommand;
  view: DisplayView;
  onView: (view: DisplayView) => void;
  onSlot: (slot: number) => void;
  onEnter: () => void;
  onConsoleAction: (action: string, slot?: number, source?: number) => void;
  onOpenDialog: (name: "show" | "artnet" | "dmx") => void;
}

type NumericParam = "size" | "bpm" | "repeat" | "spread";

const SHAPE_MENU = [
  "Playback Shape",
  "Shape Parameter",
  "Edit Shape",
  "Playback Parameter",
  "Delete/Off Shape",
  "Include Scene untuk Edit",
] as const;

const DIRECTION_LABELS: Record<ShapeDirection, string> = {
  left: "Arah kiri",
  right: "Arah kanan",
  bounce: "Arah bolak balik",
  cross: "Arah silang",
};

const DIRECTION_GLYPHS: Record<ShapeDirection, string> = {
  left: "←",
  right: "→",
  bounce: "↔",
  cross: "✕",
};

const INTERLEAVE_LABELS: Record<ShapeInterleave, string> = {
  none: "None",
  even: "Even",
  odd: "Odd",
};

const PARAM_RANGES: Record<NumericParam, { range: string; min: number; max: number }> = {
  size: { range: "0..255", min: 0, max: 255 },
  bpm: { range: "1..240", min: 1, max: 240 },
  repeat: { range: "0..360", min: 0, max: 360 },
  spread: { range: "0..63", min: 0, max: 63 },
};

export function DisplayPanel({
  frame,
  run,
  view,
  onView,
  onSlot,
  onEnter,
  onConsoleAction,
  onOpenDialog,
}: DisplayPanelProps) {
  const [draft, setDraft] = useState<{ key: NumericParam; text: string } | null>(null);
  const params: ShapeParams = frame.shapeParams ?? defaultShapeParams();
  const selected = frame.fixtures.find((fixture) => fixture.selected);
  const recorded = frame.recordedSlots ?? [];

  const commitParam = (key: NumericParam, text: string) => {
    if (text.trim() === "") return;
    const parsed = Number(text);
    if (!Number.isFinite(parsed)) return;
    const next: ShapeParams = { ...params };
    if (key === "size") next.size = displayToSize(parsed);
    else if (key === "bpm") next.bpm = clamp(parsed, 1, 240);
    else if (key === "repeat") next.repeat = degreesToRepeat(parsed);
    else next.spread = displayToSpread(parsed);
    void run("shape-param", () => commands.setShape(frame.shapeKind, next));
  };

  const paramValue = (key: NumericParam): string => {
    if (draft?.key === key) return draft.text;
    if (key === "size") return String(sizeToDisplay(params.size));
    if (key === "bpm") return String(Number((params.bpm ?? 6).toFixed(2)));
    if (key === "repeat") return String(repeatToDegrees(params.repeat));
    return String(spreadToDisplay(params.spread));
  };

  const paramTile = (key: NumericParam, label: string, unit: string) => {
    const meta = PARAM_RANGES[key];
    return (
      <label className="param-tile">
        <span className="param-label">{label}</span>
        <span className="param-control">
          <input
            className="param-input num"
            type="number"
            min={meta.min}
            max={meta.max}
            step={key === "bpm" ? 0.01 : key === "spread" ? 0.5 : 1}
            value={paramValue(key)}
            aria-label={`${label}, rentang ${meta.range}`}
            onChange={(event) => {
              const text = event.target.value;
              setDraft({ key, text });
              commitParam(key, text);
            }}
            onBlur={() => setDraft((current) => (current?.key === key ? null : current))}
          />
          <span className="param-unit num">{unit}</span>
        </span>
        <span className="param-range num">{meta.range}</span>
      </label>
    );
  };

  const setInterleave = (value: ShapeInterleave) => {
    void run("shape-interleave", () =>
      commands.setShape(frame.shapeKind, { ...params, interleave: value }),
    );
  };

  const setDirection = (value: ShapeDirection) => {
    void run("shape-direction", () =>
      commands.setShape(frame.shapeKind, { ...params, direction: value }),
    );
  };

  const playShape = (active: boolean) => {
    void run(active ? "shape-play" : "shape-stop", async () => {
      await commands.setShape(frame.shapeKind, params);
      return commands.shapePlay(active);
    });
  };

  const selectCategory = (category: ShapeCategory) => {
    const kind = primaryKindForCategory(category, frame.shapeKind);
    const next = defaultShapeParams(category);
    void run("shape-category", () => commands.setShape(kind, next));
    if (category === "Tilt/Pan") onView({ kind: "shape-edit", category });
    else onView({ kind: "shape-params", category });
  };

  const categoryKeys = (className: string) => (
    <div className={className} role="group" aria-label="Kategori shape">
      {shapeCategoryList().map((category, index) => (
        <button
          key={category}
          type="button"
          className={`kl-key kl-menu-key${category === categoryForKind(frame.shapeKind) ? " is-current" : ""}`}
          aria-pressed={category === categoryForKind(frame.shapeKind)}
          onClick={() => selectCategory(category)}
        >
          {`${index + 1}  ${category}`}
        </button>
      ))}
    </div>
  );

  const slotPicker = (prompt: string) => (
    <div className="kl-picker">
      <p className="kl-prompt">{prompt}</p>
      <div className="kl-slot-grid" role="group" aria-label={prompt}>
        {Array.from({ length: 16 }, (_, index) => index + 1).map((slot) => (
          <button
            key={slot}
            type="button"
            className={`kl-key kl-key-slot num ${recorded[slot - 1] ? "is-recorded" : ""}`}
            aria-label={`Slot ${slot}${recorded[slot - 1] ? ", tersimpan" : ""}`}
            onClick={() => onSlot(slot)}
          >
            {String(slot).padStart(2, "0")}
          </button>
        ))}
      </div>
    </div>
  );

  const renderScreen = () => {
    if (view.kind === "record") return slotPicker("RECORD / PILIH SLOT");
    if (view.kind === "operation-pick") {
      return (
        <div className="kl-menu" role="group" aria-label="Copy atau Include">
          <button
            type="button"
            className="kl-key kl-menu-key"
            onClick={() => onView({ kind: "operation", action: "copy" })}
          >
            1  Copy
          </button>
          <button
            type="button"
            className="kl-key kl-menu-key"
            onClick={() => onView({ kind: "operation", action: "include" })}
          >
            2  Include
          </button>
        </div>
      );
    }
    if (view.kind === "operation") {
      if (view.action === "copy" || view.action === "move") {
        if (view.source === undefined) return slotPicker(`${view.action.toUpperCase()} / SLOT ASAL`);
        return slotPicker(`${view.action.toUpperCase()} / SLOT TUJUAN DARI ${view.source}`);
      }
      return slotPicker(`${view.action.toUpperCase()} / PILIH SLOT`);
    }

    if (view.kind === "shape-menu") {
      return (
        <div className="kl-menu" role="group" aria-label="Shape Menu">
          <p className="kl-prompt">SHAPE MENU</p>
          {SHAPE_MENU.map((label, index) => (
            <button
              key={label}
              type="button"
              className="kl-key kl-menu-key"
              onClick={() => {
                const category = categoryForKind(frame.shapeKind);
                if (label === "Playback Shape" || label === "Edit Shape") {
                  onView({ kind: "shape-category" });
                } else if (label === "Shape Parameter" || label === "Playback Parameter") {
                  onView({ kind: "shape-params", category });
                } else if (label === "Delete/Off Shape") {
                  void run("shape-stop", () => commands.shapePlay(false));
                  onView(HOME_VIEW);
                } else {
                  onView({ kind: "operation", action: "include" });
                }
              }}
            >
              {`${index + 1}  ${label}`}
            </button>
          ))}
        </div>
      );
    }

    if (view.kind === "shape-category") {
      return (
        <div className="kl-menu" role="group" aria-label="Playback Shape">
          <p className="kl-prompt">PLAYBACK SHAPE</p>
          {categoryKeys("kl-category-list")}
        </div>
      );
    }

    if (view.kind === "shape-edit") {
      const category = view.category;
      return (
        <div className="kl-shape-screen">
          <p className="kl-prompt">{`${category.toUpperCase()} SHAPE`}</p>
          <div className="kl-shape-grid" role="group" aria-label={`Bentuk ${category}`}>
            {kindsForCategory(category).map((kind, index) => (
              <button
                key={kind}
                type="button"
                className="kl-key kl-shape-key"
                aria-pressed={frame.shapeKind === kind}
                onClick={() => {
                  void run("shape-kind", () => commands.setShape(kind, params));
                  onView({ kind: "shape-params", category });
                }}
              >
                {`${index + 1}  ${shapeLabel(kind)}`}
              </button>
            ))}
          </div>
        </div>
      );
    }

    if (view.kind === "shape-params") {
      return (
        <div className="kl-shape-screen">
          <p className="kl-prompt">SHAPE PARAMETER</p>
          <div className="kl-params">
            <div className="kl-param-grid">
              {paramTile("size", "Size", "/255")}
              {paramTile("bpm", "Speed", "BPM")}
              {paramTile("repeat", "Rep", "/360")}
              {paramTile("spread", "Spr", "/63")}
            </div>
            <div className="kl-option-line" role="group" aria-label="Interleave">
              <span className="kl-option-label">Interleave</span>
              {interleaveList().map((value) => (
                <button
                  key={value}
                  type="button"
                  className="kl-key kl-key-mini"
                  aria-pressed={params.interleave === value}
                  onClick={() => setInterleave(value)}
                >
                  {INTERLEAVE_LABELS[value]}
                </button>
              ))}
            </div>
            <div className="kl-option-line" role="group" aria-label="Arah shape">
              <span className="kl-option-label">Arah</span>
              {directionList().map((value) => (
                <button
                  key={value}
                  type="button"
                  className="kl-key kl-key-icon"
                  aria-label={DIRECTION_LABELS[value]}
                  aria-pressed={params.direction === value}
                  onClick={() => setDirection(value)}
                >
                  {DIRECTION_GLYPHS[value]}
                </button>
              ))}
            </div>
            {categoryKeys("kl-category-list kl-category-list-bottom")}
            <button
              type="button"
              className={`kl-key kl-shape-toggle${frame.shapePlaying ? " is-playing" : ""}`}
              onClick={() => playShape(!frame.shapePlaying)}
            >
              {frame.shapePlaying ? "Stop Shape" : "Play Shape"}
            </button>
          </div>
        </div>
      );
    }

    if (view.kind === "time") {
      return (
        <div className="kl-time-screen">
          <p className="kl-prompt">TIME / SHAPE BPM</p>
          <div className="kl-param-grid">{paramTile("bpm", "Speed", "BPM")}</div>
          <p className="kl-note">BPM menggerakkan shape melalui engine.</p>
        </div>
      );
    }

    if (view.kind === "setup") {
      return (
        <div className="kl-menu" role="group" aria-label="Setup">
          <button type="button" className="kl-key kl-menu-key" onClick={() => onOpenDialog("show")}>Show</button>
          <button type="button" className="kl-key kl-menu-key" onClick={() => onOpenDialog("artnet")}>Art-Net</button>
          <button type="button" className="kl-key kl-menu-key" onClick={() => onOpenDialog("dmx")}>DMX</button>
        </div>
      );
    }

    return (
      <div className="kl-home">
        <p className="kl-home-main">
          {selected ? `${selected.label} / ${selected.slot}` : "NO FIXTURE SELECTED"}
        </p>
        {selected && frame.programmerActive ? (
          <p className="kl-home-sub num">PROGRAMMER ACTIVE</p>
        ) : null}
      </div>
    );
  };

  return (
    <div className="kl-display" data-view={view.kind}>
      <div className="kl-screen" aria-live="polite">
        {renderScreen()}
        <span className="kl-brand">KINGLIGHT</span>
      </div>
      <div className="kl-display-actions" role="group" aria-label="Aksi display">
        <button type="button" className="kl-key" onClick={() => onView(viewBack(view))}>Exit</button>
        <button type="button" className="kl-key" onClick={() => onView({ kind: "record" })}>Record</button>
        <button type="button" className="kl-key" onClick={onEnter}>Enter</button>
        <button type="button" className="kl-key" onClick={() => onConsoleAction("clear")}>Clear</button>
      </div>
    </div>
  );
}
