# Kontrak IPC Kinglight v0.1

Dokumen ini mengikat backend Rust dan frontend secara eksplisit. Keduanya wajib memakai nama dan tipe persis di sini.

## Tipe bersama

```ts
type EncoderMode = "normal" | "fine" | "ultra";

interface FixtureSnapshot {
  slot: number;        // 1..16
  id: string;
  label: string;
  selected: boolean;
  pan: number;         // 0..1
  tilt: number;        // 0..1
  dimmer: number;      // 0..1
  color: [number, number, number]; // 0..1
}

interface OutputStatus {
  armed: boolean;
  blackout: boolean;
  playing: boolean;
  target: string;
  portAddress: number; // 0..32767
  sequence: number;    // 1..255
  sentFrames: number;
  lastError: string | null;
}

interface FrameSnapshot {
  frameId: number;
  fixtures: FixtureSnapshot[];
  dmx: number[];       // tepat 512 byte
  output: OutputStatus;
  activePage: number;  // 1..6
  channelBank: number; // 1 atau 2
  activeGroup: number; // 0 tidak ada, 1..4
  encoderMode: EncoderMode;
  shapeKind: string;
  shapePlaying: boolean;
}

interface ShapeParams {
  speed: number;
  bpm: number | null;
  size: number;
  spread: number;
  repeat: number;
  interleave: "none" | "even" | "odd";
  direction: "left" | "right" | "bounce" | "cross";
  amplitude: number;
}
```

## Perintah Tauri

Semua perintah dipanggil lewat `invoke` dari `@tauri-apps/api/core`. Perintah yang mengubah state mengembalikan `FrameSnapshot` kecuali dinyatakan lain.

| Nama | Argumen | Hasil |
|---|---|---|
| `subscribe_frames` | `onFrame: Channel<FrameSnapshot>` | `null`. Kirim snapshot saat frame baru tersedia, paling cepat 30/s |
| `get_snapshot` | tidak ada | `FrameSnapshot` |
| `select_fixture` | `slot: number` (1..16) | `FrameSnapshot` |
| `set_group` | `group: number` (0..4) | `FrameSnapshot` |
| `set_page` | `page: number` (1..6) | `FrameSnapshot` |
| `set_bank` | `bank: number` (1..2) | `FrameSnapshot` |
| `set_fader` | `attribute: string`, `value: number` (0..65535) | `FrameSnapshot` |
| `set_encoder` | `axis: "a" \| "b"`, `delta: number`, `mode: EncoderMode` | `FrameSnapshot` |
| `set_shape` | `kind: string`, `params: ShapeParams` | `FrameSnapshot` |
| `shape_play` | `active: boolean` | `FrameSnapshot` |
| `set_blackout` | `active: boolean` | `FrameSnapshot` |
| `set_playing` | `active: boolean` | `FrameSnapshot` |
| `configure_output` | `target: string`, `portAddress: number`, `compatZero: boolean` | `FrameSnapshot` |
| `arm_output` | tidak ada | `FrameSnapshot`, atau error string |
| `stop_output` | tidak ada | `FrameSnapshot` |
| `save_show` | `name: string` | `string` path absolut |
| `load_show` | `name: string` | `FrameSnapshot`, atau error string |
| `list_shows` | tidak ada | `string[]` |

## Aturan backend

- Engine tick 50 Hz. ArtDmx 30 Hz. Snapshot ke UI maksimal 30 Hz.
- Satu universe, 512 byte, alamat mulai 1, 16 fixture dengan footprint 8 channel.
- `portAddress` raw 15 bit. Nilai 0 hanya diterima bila `compatZero: true`.
- Start dan load selalu disarmed.
- `stop_output` mengirim dua frame blackout pada 30 Hz dari frame yang sama, lalu disarmed.
- Blackout: intensity 0, pan/tilt ditahan, warna ditahan.
- Master tidak mengubah pan, tilt, atau warna.
- Hanya backend yang membuka UDP socket.

## Nama shape yang diterima

`Circle`, `CircleTwoWay`, `Figure8`, `Spiral`, `Square`, `Diamond`, `PanSwing`, `TiltSwing`.

Nama di luar daftar ditolak dengan error string.
