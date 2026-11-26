import { useCallback, useEffect, useRef, useState } from "react";
import { ConsolePanel } from "./components/ConsolePanel";
import { DmxMonitor } from "./components/DmxMonitor";
import { OutputBar } from "./components/OutputBar";
import { ShowManager } from "./components/ShowManager";
import { StagePreview } from "./components/StagePreview";
import { useKinglight } from "./hooks/useKinglight";
import { commands } from "./ipc";
import { HOME_VIEW, type DisplayView, type EncoderPair } from "./lib/console";
import { formatError } from "./lib/format";

type DialogName = "show" | "artnet" | "dmx";

export default function App() {
  const kinglight = useKinglight();
  const [compact, setCompact] = useState(true);
  const [view, setView] = useState<DisplayView>(HOME_VIEW);
  const [encoderPair, setEncoderPair] = useState<EncoderPair>("pan-tilt");
  const [dialog, setDialog] = useState<DialogName | null>(null);
  const [fullscreen, setFullscreen] = useState(false);
  const dialogRef = useRef<HTMLDialogElement | null>(null);

  useEffect(() => {
    const element = dialogRef.current;
    if (!element) return;
    if (dialog && !element.open) element.showModal();
    if (!dialog && element.open) element.close();
  }, [dialog]);

  useEffect(() => {
    const onFullscreenChange = () => setFullscreen(document.fullscreenElement !== null);
    document.addEventListener("fullscreenchange", onFullscreenChange);
    return () => document.removeEventListener("fullscreenchange", onFullscreenChange);
  }, []);

  const dispatch = useCallback(
    (action: string, slot?: number, source?: number) => {
      void kinglight.run(`console-${action}`, () =>
        commands.consoleAction(action, slot, source),
      );
    },
    [kinglight],
  );

  const handleSlot = useCallback(
    (slot: number) => {
      if (view.kind === "record") {
        dispatch("record", slot);
        setView(HOME_VIEW);
        return;
      }
      if (view.kind === "operation") {
        const { action, source } = view;
        if ((action === "copy" || action === "move") && source === undefined) {
          setView({ kind: "operation", action, source: slot });
          return;
        }
        dispatch(action, slot, source);
        setView(HOME_VIEW);
        return;
      }
      dispatch("locate", slot);
    },
    [dispatch, view],
  );

  const handleEnter = useCallback(() => {
    if (view.kind === "record" || view.kind === "operation") {
      kinglight.notify("error", "Pilih slot pada panel konsol.");
      return;
    }
    if (view.kind === "operation-pick") {
      document.querySelector<HTMLButtonElement>('[aria-label="Copy atau Include"] button')?.focus();
      return;
    }
    if (view.kind === "shape-menu" || view.kind === "shape-category") {
      setView({ kind: "shape-edit", category: "Tilt/Pan" });
      return;
    }
    if (view.kind === "shape-edit") {
      setView({ kind: "shape-params", category: view.category });
      setEncoderPair("size-speed");
      return;
    }
    if (view.kind === "shape-params") {
      if (kinglight.frame) {
        const active = kinglight.frame.shapePlaying;
        void kinglight.run(active ? "shape-stop" : "shape-play", () =>
          commands.shapePlay(!active),
        );
      }
      return;
    }
    if (view.kind === "time") {
      setView(HOME_VIEW);
      setEncoderPair("pan-tilt");
      return;
    }
    if (view.kind === "setup") {
      setDialog("show");
      return;
    }
    dispatch("locate");
  }, [dispatch, kinglight, view]);

  const toggleFullscreen = useCallback(() => {
    const target = document.documentElement;
    if (document.fullscreenElement) {
      void document.exitFullscreen().catch((error: unknown) => {
        kinglight.notify("error", `Keluar layar penuh gagal: ${formatError(error)}`);
      });
      return;
    }
    void target.requestFullscreen().catch((error: unknown) => {
      kinglight.notify("error", `Layar penuh gagal: ${formatError(error)}`);
    });
  }, [kinglight]);

  if (kinglight.loadState === "loading") {
    return (
      <main className="kl-boot" role="status">
        <h1>Kinglight</h1>
        <p>Menghubungkan ke engine.</p>
      </main>
    );
  }

  if (kinglight.loadState === "error" || !kinglight.frame) {
    return (
      <main className="kl-boot" role="alert">
        <h1>Kinglight</h1>
        <p>{kinglight.loadError ?? "Snapshot tidak diterima."}</p>
        <button type="button" className="kl-key" onClick={kinglight.retryConnection}>
          Hubungkan ulang
        </button>
      </main>
    );
  }

  const frame = kinglight.frame;

  return (
    <div className="kl-app">
      <StagePreview
        fixtures={frame.fixtures}
        armed={frame.output.armed}
        blackout={frame.output.blackout}
      />
      <header className="kl-toolbar">
        <span className="kl-toolbar-brand">Kinglight</span>
        <button
          type="button"
          className="kl-key kl-toolbar-key"
          aria-pressed={compact}
          onClick={() => setCompact((current) => !current)}
        >
          {compact ? "Konsol penuh" : "Panggung"}
        </button>
        <button type="button" className="kl-toolbar-key kl-key" onClick={() => setDialog("show")}>
          Shows
        </button>
        <button type="button" className="kl-toolbar-key kl-key" onClick={() => setDialog("artnet")}>
          Art-Net
        </button>
        <button
          type="button"
          className="kl-toolbar-key kl-key"
          aria-pressed={fullscreen}
          onClick={toggleFullscreen}
        >
          {fullscreen ? "Keluar layar penuh" : "Layar penuh"}
        </button>
        {compact && frame.output.armed ? (
          <div className="kl-toolbar-live" role="group" aria-label="Kontrol output langsung">
            <button
              type="button"
              className={`kl-key kl-toolbar-key kl-live-key${frame.output.blackout ? " is-on" : ""}`}
              aria-pressed={frame.output.blackout}
              onClick={() =>
                void kinglight.run("blackout", () =>
                  commands.setBlackout(!frame.output.blackout),
                )
              }
            >
              Blackout
            </button>
            <button
              type="button"
              className="kl-key kl-toolbar-key kl-live-stop"
              onClick={() => void kinglight.run("stop-output", () => commands.stopOutput())}
            >
              Stop Output
            </button>
          </div>
        ) : null}
      </header>

      <ConsolePanel
        frame={frame}
        run={kinglight.run}
        compact={compact}
        view={view}
        encoderPair={encoderPair}
        onCompact={setCompact}
        onView={(next) => {
          setView(next);
          if (next.kind === "shape-params") setEncoderPair("size-speed");
          if (next.kind === "home") setEncoderPair("pan-tilt");
        }}
        onSlot={handleSlot}
        onEnter={handleEnter}
        onConsoleAction={dispatch}
        onPair={setEncoderPair}
        onOpenDialog={setDialog}
      />
      {kinglight.notice ? (
        <div
          className={`kl-notice${kinglight.notice.kind === "error" ? " is-error" : ""}`}
          role={kinglight.notice.kind === "error" ? "alert" : "status"}
        >
          <span>{kinglight.notice.text}</span>
          <button type="button" className="kl-key kl-key-mini" onClick={kinglight.clearNotice}>
            Tutup
          </button>
        </div>
      ) : null}

      <dialog
        ref={dialogRef}
        className="kl-dialog"
        aria-label="Pengaturan Kinglight"
        onClose={() => setDialog(null)}
      >
        <div className="kl-dialog-head">
          <strong>{dialog === "show" ? "Show" : dialog === "artnet" ? "Art-Net" : "DMX"}</strong>
          <button type="button" className="kl-key kl-key-mini" onClick={() => setDialog(null)}>
            Tutup
          </button>
        </div>
        <div className="kl-dialog-body">
          {dialog === "show" ? (
            <ShowManager
              frame={frame}
              shows={kinglight.shows}
              run={kinglight.run}
              refreshShows={kinglight.refreshShows}
              busy={kinglight.busy}
              notify={kinglight.notify}
            />
          ) : dialog === "artnet" ? (
            <OutputBar frame={frame} run={kinglight.run} busy={kinglight.busy} />
          ) : dialog === "dmx" ? (
            <DmxMonitor frame={frame} />
          ) : null}
        </div>
        <div className="kl-dialog-foot">
          <button type="button" className="kl-key" onClick={() => setDialog(null)}>
            Selesai
          </button>
        </div>
      </dialog>
    </div>
  );
}
