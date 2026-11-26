import { formatChannelValue } from "../lib/format";
import type { FrameSnapshot } from "../types";

interface DmxMonitorProps {
  frame: FrameSnapshot;
}

const CHANNELS = Array.from({ length: 256 }, (_, index) => index + 1);

export function DmxMonitor({ frame }: DmxMonitorProps) {
  return (
    <section className="panel dmx-panel" aria-labelledby="dmx-title">
      <div className="panel-head">
        <div>
          <h2 id="dmx-title" className="panel-title">DMX monitor</h2>
          <p className="panel-sub num">
            Channels 1-256 of frame {frame.frameId}. Buffer has 512 bytes.
          </p>
        </div>
        <span className="status-pill num">Bank {frame.channelBank === 2 ? "17-32" : "1-16"}</span>
      </div>
      <div className="dmx-grid" role="grid" aria-label="DMX channel values">
        {CHANNELS.map((channel) => {
          const byte = frame.dmx[channel - 1];
          const active = typeof byte === "number" && byte > 0;
          const inBank =
            frame.channelBank === 1 ? channel <= 16 : channel >= 17 && channel <= 32;
          return (
            <div
              key={channel}
              role="gridcell"
              className={`dmx-cell num ${active ? "dmx-active" : ""} ${inBank ? "dmx-bank" : ""}`}
              title={`Channel ${channel}: ${formatChannelValue(byte)}`}
            >
              <span className="dmx-channel">{String(channel).padStart(3, "0")}</span>
              <span className="dmx-value">{formatChannelValue(byte)}</span>
            </div>
          );
        })}
      </div>
    </section>
  );
}
