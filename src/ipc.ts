import { Channel, invoke } from "@tauri-apps/api/core";
import type {
  EncoderMode,
  FrameSnapshot,
  ShapeParams,
} from "./types";

export async function subscribeFrames(
  onFrame: (snapshot: FrameSnapshot) => void,
): Promise<() => void> {
  const channel = new Channel<FrameSnapshot>();
  channel.onmessage = onFrame;
  await invoke<void>("subscribe_frames", { onFrame: channel });
  return () => {
    channel.onmessage = () => undefined;
  };
}

export const commands = {
  getSnapshot: () => invoke<FrameSnapshot>("get_snapshot"),

  selectFixture: (slot: number) =>
    invoke<FrameSnapshot>("select_fixture", { slot }),

  setGroup: (group: number) =>
    invoke<FrameSnapshot>("set_group", { group }),

  setPage: (page: number) => invoke<FrameSnapshot>("set_page", { page }),

  setBank: (bank: number) => invoke<FrameSnapshot>("set_bank", { bank }),

  setFader: (attribute: string, value: number) =>
    invoke<FrameSnapshot>("set_fader", { attribute, value }),

  setEncoder: (axis: "a" | "b", delta: number, mode: EncoderMode) =>
    invoke<FrameSnapshot>("set_encoder", { axis, delta, mode }),

  setShape: (kind: string, params: ShapeParams) =>
    invoke<FrameSnapshot>("set_shape", { kind, params }),

  shapePlay: (active: boolean) =>
    invoke<FrameSnapshot>("shape_play", { active }),

  setBlackout: (active: boolean) =>
    invoke<FrameSnapshot>("set_blackout", { active }),

  setPlaying: (active: boolean) =>
    invoke<FrameSnapshot>("set_playing", { active }),

  configureOutput: (
    target: string,
    portAddress: number,
    compatZero: boolean,
  ) =>
    invoke<FrameSnapshot>("configure_output", {
      target,
      portAddress,
      compatZero,
    }),

  armOutput: () => invoke<FrameSnapshot>("arm_output"),

  stopOutput: () => invoke<FrameSnapshot>("stop_output"),

  saveShow: (name: string) => invoke<string>("save_show", { name }),

  loadShow: (name: string) => invoke<FrameSnapshot>("load_show", { name }),

  listShows: () => invoke<string[]>("list_shows"),
};
