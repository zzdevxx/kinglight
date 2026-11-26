const clampUnit = (value: number) => Math.min(1, Math.max(0, value));

export function fixturePosition(index: number): [number, number, number] {
  const x = index < 12 ? index - 5.5 : [-7.5, -6.5, 6.5, 7.5][index - 12];
  return [x * 0.38, 1.7, -5];
}

export function beamDirection(pan: number, tilt: number): [number, number, number] {
  const yaw = (clampUnit(pan) - 0.5) * 0.17;
  const pitch = (clampUnit(tilt) - 0.5) * 2.4 + 0.16;
  const horizontal = Math.cos(pitch);
  return [Math.sin(yaw) * horizontal, Math.sin(pitch), Math.cos(yaw) * horizontal];
}
