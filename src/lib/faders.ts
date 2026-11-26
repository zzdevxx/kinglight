import { clamp } from "./format";

export type ConsoleMode = "fixture" | "playback";

export interface FaderDefinition {
  key: string;
  label: string;
  attribute: string;
  channel: number | null;
}

export const MASTER_ATTRIBUTE = "master";

function channelLabel(channel: number): string {
  return `CH ${String(channel).padStart(2, "0")}`;
}

function playbackLabel(slot: number): string {
  return `PB ${String(slot).padStart(2, "0")}`;
}

export function faderDefinitions(
  mode: ConsoleMode,
  bank: number,
  page: number,
): FaderDefinition[] {
  const normalizedBank = bank === 2 ? 2 : 1;
  const normalizedPage = Math.round(clamp(page, 1, 6));
  const firstChannel = normalizedBank === 1 ? 1 : 17;

  return Array.from({ length: 16 }, (_, index) => {
    if (mode === "fixture") {
      const channel = firstChannel + index;
      return {
        key: `channel-${channel}`,
        label: channelLabel(channel),
        attribute: `channel.${channel}`,
        channel,
      };
    }
    const slot = index + 1;
    return {
      key: `playback-${normalizedPage}-${slot}`,
      label: playbackLabel(slot),
      attribute: `playback.${normalizedPage}.${slot}`,
      channel: null,
    };
  });
}
