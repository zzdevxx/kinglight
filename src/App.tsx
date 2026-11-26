import { ConsolePanel } from "./components/ConsolePanel";
import { DisplayPanel } from "./components/DisplayPanel";
import { DmxMonitor } from "./components/DmxMonitor";
import { OutputBar } from "./components/OutputBar";
import { ShowManager } from "./components/ShowManager";
import { StagePreview } from "./components/StagePreview";
import { useKinglight } from "./hooks/useKinglight";
import { formatCount } from "./lib/format";

export default function App() {
  const kinglight = useKinglight();

  if (kinglight.loadState === "loading") {
    return (
      <main className="boot" role="status">
        <h1>Kinglight</h1>
        <p>Connecting to engine over Tauri IPC.</p>
      </main>
    );
  }

  if (kinglight.loadState === "error" || !kinglight.frame) {
    return (
      <main className="boot boot-error" role="alert">
        <h1>Kinglight</h1>
        <p>Engine connection failed: {kinglight.loadError ?? "No snapshot received."}</p>
        <button type="button" className="btn btn-primary" onClick={kinglight.retryConnection}>
          Retry connection
        </button>
      </main>
    );
  }

  const frame = kinglight.frame;

  return (
    <div className="app-shell">
      <header className="app-header">
        <div className="brand">
          <h1>Kinglight</h1>
          <span className="brand-sub">console / preview</span>
        </div>
        <div className="header-status num">
          <span>frame {frame.frameId}</span>
          <span>page {frame.activePage}</span>
          <span>bank {frame.channelBank}</span>
          <span>{frame.fixtures.length} fixtures</span>
        </div>
      </header>

      {kinglight.notice ? (
        <div
          className={`notice ${kinglight.notice.kind === "error" ? "notice-error" : ""}`}
          role={kinglight.notice.kind === "error" ? "alert" : "status"}
        >
          <span>{kinglight.notice.text}</span>
          <button type="button" className="btn btn-compact" onClick={kinglight.clearNotice}>
            Dismiss
          </button>
        </div>
      ) : null}

      <main className="workspace">
        <div className="stage-column">
          <StagePreview
            fixtures={frame.fixtures}
            armed={frame.output.armed}
            blackout={frame.output.blackout}
          />
          <DmxMonitor frame={frame} />
        </div>
        <ConsolePanel
          frame={frame}
          run={kinglight.run}
          busy={kinglight.busy}
          notify={kinglight.notify}
        />
        <aside className="side-column">
          <DisplayPanel frame={frame} run={kinglight.run} />
          <ShowManager
            frame={frame}
            shows={kinglight.shows}
            run={kinglight.run}
            refreshShows={kinglight.refreshShows}
            busy={kinglight.busy}
            notify={kinglight.notify}
          />
        </aside>
      </main>

      <footer className="output-dock">
        <OutputBar frame={frame} run={kinglight.run} busy={kinglight.busy} />
        <span className="footer-note num">
          {formatCount(frame.output.sentFrames)} frames sent since output start
        </span>
      </footer>
    </div>
  );
}
