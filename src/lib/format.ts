export function clamp(value: number, min: number, max: number): number {
  if (!Number.isFinite(value)) return min;
  return Math.min(max, Math.max(min, value));
}

export function clamp16(value: number): number {
  return Math.round(clamp(value, 0, 65535));
}

export function byteTo16(byte: number | undefined): number {
  if (byte === undefined || !Number.isFinite(byte)) return 0;
  return Math.round((clamp(byte, 0, 255) / 255) * 65535);
}

export function formatChannelValue(byte: number | undefined): string {
  if (byte === undefined || !Number.isFinite(byte)) return "---";
  return String(Math.round(clamp(byte, 0, 255))).padStart(3, "0");
}

export function formatMode16(value: number): string {
  return String(clamp16(value)).padStart(5, "0");
}

export function formatCount(value: number): string {
  if (!Number.isFinite(value)) return "--";
  return Math.max(0, Math.round(value)).toLocaleString("en-US");
}

export function formatError(error: unknown): string {
  if (error instanceof Error) return error.message || error.name;
  if (typeof error === "string") return error;
  if (typeof error === "number" && Number.isFinite(error)) return String(error);
  try {
    const serialized = JSON.stringify(error);
    return serialized && serialized !== "{}" ? serialized : "Unknown IPC failure";
  } catch {
    return "Unknown IPC failure";
  }
}
