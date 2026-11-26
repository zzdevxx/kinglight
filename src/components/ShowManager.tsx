import { useEffect, useState } from "react";
import { commands } from "../ipc";
import type { FrameSnapshot, RunCommand } from "../types";

interface ShowManagerProps {
  frame: FrameSnapshot;
  shows: string[];
  run: RunCommand;
  refreshShows: () => Promise<void>;
  busy: string | null;
  notify: (kind: "ok" | "error", text: string) => void;
}

export function ShowManager({
  frame,
  shows,
  run,
  refreshShows,
  busy,
  notify,
}: ShowManagerProps) {
  const [name, setName] = useState("");
  const [selected, setSelected] = useState("");
  const [feedback, setFeedback] = useState<string | null>(null);

  useEffect(() => {
    if (shows.length === 0) {
      setSelected("");
      return;
    }
    if (!shows.includes(selected)) setSelected(shows[0]);
  }, [shows, selected]);

  const save = async () => {
    const trimmed = name.trim();
    if (!trimmed) {
      setFeedback("Enter a show name before saving.");
      return;
    }
    const path = await run("save-show", () => commands.saveShow(trimmed));
    if (path) {
      setFeedback(`Saved to ${path}`);
      notify("ok", "Show saved.");
      await refreshShows();
    }
  };

  const load = async () => {
    if (!selected) {
      setFeedback("Select a saved show first.");
      return;
    }
    if (frame.output.armed) {
      setFeedback("Stop output before loading a show.");
      return;
    }
    const snapshot = await run("load-show", () => commands.loadShow(selected));
    if (snapshot) {
      setFeedback(`Loaded ${selected}. Output is disarmed.`);
      notify("ok", "Show loaded.");
      await refreshShows();
    }
  };

  return (
    <section className="panel show-panel" aria-labelledby="show-title">
      <div className="panel-head">
        <div>
          <h2 id="show-title" className="panel-title">Show manager</h2>
          <p className="panel-sub">Local show files</p>
        </div>
        <button
          type="button"
          className="btn"
          disabled={busy !== null}
          onClick={() => void refreshShows()}
        >
          Refresh
        </button>
      </div>

      <div className="show-form">
        <label className="field-label">
          Show name
          <input
            className="field"
            value={name}
            placeholder="show name"
            onChange={(event) => setName(event.target.value)}
          />
        </label>
        <button
          type="button"
          className="btn"
          disabled={busy !== null || !name.trim()}
          onClick={() => void save()}
        >
          Save
        </button>
      </div>

      <div className="show-form">
        <label className="field-label">
          Saved shows
          <select
            className="field"
            value={selected}
            onChange={(event) => setSelected(event.target.value)}
          >
            <option value="">No show selected</option>
            {shows.map((show) => (
              <option key={show} value={show}>{show}</option>
            ))}
          </select>
        </label>
        <button
          type="button"
          className="btn btn-primary"
          disabled={busy !== null || !selected}
          onClick={() => void load()}
        >
          Load
        </button>
      </div>

      {shows.length === 0 ? (
        <p className="empty-inline" role="status">No saved shows found.</p>
      ) : null}
      {feedback ? <p className="feedback" role="status">{feedback}</p> : null}
    </section>
  );
}
