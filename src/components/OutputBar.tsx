import { useEffect, useRef, useState } from "react";
import { commands } from "../ipc";
import { formatCount } from "../lib/format";
import type { FrameSnapshot, RunCommand } from "../types";

interface OutputBarProps {
  frame: FrameSnapshot;
  run: RunCommand;
  busy: string | null;
}

export function OutputBar({ frame, run, busy }: OutputBarProps) {
  const output = frame.output;
  const [targetDraft, setTargetDraft] = useState(output.target);
  const [portDraft, setPortDraft] = useState(String(output.portAddress));
  const [compatZero, setCompatZero] = useState(output.portAddress === 0);
  const [configError, setConfigError] = useState<string | null>(null);
  const dirtyRef = useRef(false);

  useEffect(() => {
    if (dirtyRef.current) return;
    setTargetDraft(output.target);
    setPortDraft(String(output.portAddress));
    setCompatZero(output.portAddress === 0);
  }, [output.target, output.portAddress]);

  const applyConfig = async () => {
    const port = Number(portDraft);
    if (!targetDraft.trim()) {
      setConfigError("Target cannot be empty.");
      return;
    }
    if (!Number.isInteger(port) || port < 0 || port > 32767) {
      setConfigError("Port-Address must be an integer from 0 to 32767.");
      return;
    }
    if (port === 0 && !compatZero) {
      setConfigError("Raw address 0 requires the compatibility option.");
      return;
    }
    setConfigError(null);
    const snapshot = await run("configure-output", () =>
      commands.configureOutput(targetDraft.trim(), port, compatZero),
    );
    if (snapshot) {
      dirtyRef.current = false;
      setTargetDraft(snapshot.output.target);
      setPortDraft(String(snapshot.output.portAddress));
    }
  };

  return (
    <section className="panel output-panel" aria-labelledby="output-title">
      <div className="output-head">
        <h2 id="output-title" className="panel-title">Output</h2>
        <div className="status-row">
          <span className={`status-pill ${output.armed ? "status-ok" : ""}`}>
            {output.armed ? "Armed" : "Disarmed"}
          </span>
          <span className={`status-pill ${output.blackout ? "status-danger" : ""}`}>
            {output.blackout ? "Blackout" : "Normal"}
          </span>
          <span className={`status-pill ${output.playing ? "status-ok" : ""}`}>
            {output.playing ? "Playing" : "Stopped"}
          </span>
        </div>
        <div className="output-actions">
          <button
            type="button"
            className={`btn ${output.blackout ? "btn-danger" : ""}`}
            aria-pressed={output.blackout}
            onClick={() => void run("blackout", () => commands.setBlackout(!output.blackout))}
          >
            {output.blackout ? "Release blackout" : "Blackout"}
          </button>
          <button
            type="button"
            className="btn btn-primary"
            disabled={output.armed || busy === "arm-output"}
            onClick={() => void run("arm-output", () => commands.armOutput())}
          >
            Arm Output
          </button>
          <button
            type="button"
            className="btn"
            disabled={busy === "stop-output"}
            onClick={() => void run("stop-output", () => commands.stopOutput())}
          >
            Stop Output
          </button>
          <button
            type="button"
            className="btn"
            onClick={() => void run("playback", () => commands.setPlaying(!output.playing))}
          >
            {output.playing ? "Stop" : "Play"}
          </button>
        </div>
      </div>

      <div className="output-grid">
        <div className="readout">
          <span className="readout-label">Target</span>
          <span className="readout-value">{output.target || "not set"}</span>
        </div>
        <div className="readout">
          <span className="readout-label">Port-Address</span>
          <span className="readout-value num">{output.portAddress}</span>
        </div>
        <div className="readout">
          <span className="readout-label">Sequence</span>
          <span className="readout-value num">{output.sequence}</span>
        </div>
        <div className="readout">
          <span className="readout-label">Frames sent</span>
          <span className="readout-value num">{formatCount(output.sentFrames)}</span>
        </div>
        <div className="readout readout-wide">
          <span className="readout-label">Last error</span>
          <span className={`readout-value ${output.lastError ? "text-danger" : ""}`}>
            {output.lastError ?? "none"}
          </span>
        </div>
      </div>

      <div className="output-config">
        <label className="field-label">
          Target / NIC
          <input
            className="field"
            value={targetDraft}
            onChange={(event) => {
              dirtyRef.current = true;
              setTargetDraft(event.target.value);
            }}
          />
        </label>
        <label className="field-label">
          Port-Address
          <input
            className="field num"
            inputMode="numeric"
            value={portDraft}
            onChange={(event) => {
              dirtyRef.current = true;
              setPortDraft(event.target.value);
            }}
          />
        </label>
        <label className="check-label">
          <input
            type="checkbox"
            checked={compatZero}
            onChange={(event) => {
              dirtyRef.current = true;
              setCompatZero(event.target.checked);
            }}
          />
          Allow raw 0
        </label>
        <button
          type="button"
          className="btn"
          disabled={busy === "configure-output"}
          onClick={() => void applyConfig()}
        >
          Apply network config
        </button>
        {configError ? <span className="text-danger config-error" role="alert">{configError}</span> : null}
      </div>
    </section>
  );
}
