import { clamp } from "./format";

export type ConsoleMode = "fixture" | "playback";

export const MASTER_ATTRIBUTE = "master";
export const FADER_COUNT = 16;
export const MAX_FIXTURE_CHANNEL = 8;
export const SLOT_NUMBERS = Array.from({ length: FADER_COUNT }, (_, index) => index + 1);

export interface FaderDefinition {
  key: string;
  label: string;
  attribute: string;
  index: number;
  channel: number | null;
  slot: number | null;
  disabled: boolean;
}

export function faderDefinitions(
  mode: ConsoleMode,
  bank: number,
  page: number,
): FaderDefinition[] {
  const normalizedBank = bank === 2 ? 2 : 1;
  const normalizedPage = Math.round(clamp(page, 1, 6));
  const firstChannel = normalizedBank === 1 ? 1 : 17;

  return SLOT_NUMBERS.map((slot, index) => {
    if (mode === "fixture") {
      const channel = firstChannel + index;
      const available = channel <= MAX_FIXTURE_CHANNEL;
      return {
        key: `channel-${channel}`,
        label: available ? String(channel).padStart(2, "0") : "-",
        attribute: `ch${index + 1}`,
        index,
        channel,
        slot: null,
        disabled: !available,
      };
    }
    return {
      key: `playback-${normalizedPage}-${slot}`,
      label: String(slot).padStart(2, "0"),
      attribute: `playback.${normalizedPage}.${slot}`,
      index,
      channel: null,
      slot,
      disabled: false,
    };
  });
}
