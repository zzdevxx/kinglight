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

## Tambahan kontrak native, 2026-11-26

Bagian ini mengikat implementasi Rust dengan frontend yang menggunakan konsol penuh. Nama lama tetap berlaku.

### Field `FrameSnapshot` tambahan

| Field JSON | Tipe | Arti |
|---|---|---|
| `master` | number | Nilai master 16 bit mentah, 0..65535 |
| `shapeParams` | `ShapeParams` | Parameter shape aktif, salinan dari engine |
| `playbackLevels` | number[16] | Level fader playback pada `activePage`, indeks 0 = slot 1 |
| `recordedSlots` | boolean[16] | Slot pada `activePage` yang berisi scene terekam |
| `programmerActive` | boolean | Benar bila ada mask programmer pada satu fixture atau lebih |
| `channelValues` | number[32] | Jendela 32 byte DMX mulai dari base channel fixture pertama yang dipilih. Tanpa seleksi, base channel 1 |

`channelValues` memakai nilai 8 bit hasil resolusi engineer, bukan salinan buffer UI. `dmx` tetap berisi 512 byte universe penuh.

### Perintah baru

| Nama | Argumen | Hasil |
|---|---|---|
| `console_action` | `action: string`, `slot?: number`, `source?: number` | `FrameSnapshot` |
| `set_playback` | `slot: number` (1..16), `value: number` (0..65535) | `FrameSnapshot` |

`console_action` menerima hanya `clear`, `record`, `include`, `copy`, `move`, `delete`, `select_all`, `locate`. Nama lain ditolak dengan `INVALID_CONSOLE_ACTION`. `slot` dan `source`, bila dikirim, harus 1..16. Aksi `record`, `include`, dan `delete` memerlukan `slot`. Aksi `copy` dan `move` memerlukan `slot` serta `source`.

| Aksi | Efek |
|---|---|
| `clear` | Menghapus semua mask programmer, seleksi, dan group aktif. Tidak menghapus scene terekam atau level playback |
| `record` | Menangkap nilai resolusi logis ke-16 fixture, termasuk efek shape aktif, ke slot `slot` pada halaman aktif. Master dan blackout tidak ikut terekam |
| `include` | Memuat scene `slot` ke mask programmer seluruh fixture. Slot kosong ditolak |
| `copy` | Menyalin scene `source` ke `slot` pada halaman aktif |
| `move` | Menyalin `source` ke `slot`, lalu mengosongkan `source`. Alamat sama adalah no-op |
| `delete` | Mengosongkan scene dan level `slot` |
| `select_all` | Memilih seluruh fixture dan mengosongkan group aktif |
| `locate` | Menyetel programmer dimmer penuh pada target saat ini |

`record` menyimpan satu entri per fixture, bukan template bersama. `include` memulihkan setiap fixture dari entri yang sesuai. Playback memakai pan dan tilt dari slot pemenang per fixture, sementara dimmer dan warna tetap HTP lintas slot aktif.

`set_playback` menulis level slot pada halaman aktif. Nilai lebih dari nol mengaktifkan playback dan menaikkan revisi halaman, sehingga scene terekam dipakai oleh resolver. Nilai nol mematikan level slot itu. `set_playing(false)` tetap menjadi sakelar global playback.

### `set_fader` untuk channel mentah

Attribute `chN` dan `channel.N` bersifat relatif terhadap fixture yang dipilih. `N` = 1..8 dipetakan ke footprint delapan channel fixture pertama yang dipilih:

| `N` | Atribut |
|---|---|
| 1 | dimmer |
| 2 | pan high byte |
| 3 | pan low byte |
| 4 | tilt high byte |
| 5 | tilt low byte |
| 6 | merah |
| 7 | hijau |
| 8 | biru |

`channelValues` mengikuti urutan yang sama untuk empat fixture pertama mulai dari fixture yang dipilih. `N` = 9..32 diterima sebagai bentuk attribute tetapi ditolak dengan `INVALID_CHANNEL` karena berada di luar footprint delapan channel. Tanpa fixture yang dipilih, perintah channel ditolak. Nama lama `master`, `dimmer`, `pan`, `pancoarse`, `panfine`, `tilt`, `tiltcoarse`, `tiltfine`, `red`, `green`, `blue` tetap diterima. State bank tidak menggeser pemetaan relatif ini.

### Encoder

`delta` adalah jumlah tick, bukan nilai 16 bit. Engine mengalikan sekali: `normal` 1024, `fine` 128, `ultra` 16 per tick. Frontend tidak boleh mengalikan `delta` lagi. `delta = 0` hanya menyimpan `encoderMode`; engine kembali sebelum menulis mask programmer, sehingga shape yang sedang berjalan tidak membeku.

### Shape

Nama yang diterima: `Circle`, `CircleTwoWay`, `Figure8`, `Spiral`, `Square`, `Diamond`, `PanSwing`, `TiltSwing`, `Dimmer`, `RGB`, `Color`. Kategori Pan/Tilt, Dimmer, RGB, dan Color menjadi urusan frontend.

Rumus phase engine:

```text
cycle_ms = bpm !== null ? 60000 / bpm : 2000 / speed
temporal = phase_ms / cycle_ms
spatial = (relative_position / target_count) * spread * (repeat / 16)
t = (temporal + spatial) mod 1
```

`relative_position` adalah indeks fixture di dalam kumpulan target yang ditangkap saat shape mulai atau diperbarui, bukan indeks global. `repeat` berada di rentang 0..16. Nilai lama 1..16 tetap kompatibel. Faktor `repeat / 16` membatasi fase spasial maksimal satu siklus. `repeat = 8` menyebarkan setengah siklus, sesuai tampilan 180 derajat pada video. `repeat = 0` membuat semua target memakai phase yang sama. UI memetakan derajat video dengan `repeat * 360 / 16`. `bpm`, bila ada, adalah laju temporal penuh dan tidak dikalikan `repeat`. `spread = 1` menyebarkan phase ke seluruh target. Edit parameter saat shape berjalan tidak mereset `phase_ms`; ganti nama shape atau mulai ulang setelah stop yang mereset.

### Showfile

`schemaVersion` tetap 1. Field opsional baru `programmer`, `recordedSlots`, dan `playbackLevels` ditambahkan. Field opsional `fixtures` pada setiap `pages[][]` berisi 16 entri `{ values: [number; 6] }` dan dipakai untuk cue per fixture. Field lama `dimmer`, `pan`, `tilt`, dan `color` tetap ditulis sebagai template dari fixture pertama untuk pembaca lama. Show lama tanpa `fixtures` memakai template tersebut untuk ke-16 fixture. Show lama tanpa field opsional lainnya tetap dimuat dengan mask programmer kosong, semua slot belum terekam, dan semua level nol.

### Startup dan output

Startup menampilkan 12 fixture pertama dengan dimmer penuh, warna putih, pan merata 0.08 sampai 0.92, tilt 0.5. Empat fixture terakhir gelap dengan pan 0.5. Tidak ada fixture yang dipilih, shape berhenti, master penuh, dan output selalu `disarmed`. Engine tidak membuka jaringan atau mengarma output dengan sendirinya. Default `ShapeParams` engine adalah `bpm 6`, `size 25/255`, `spread 1`, `repeat 8`, `amplitude 0.5`, `speed 1`, `interleave none`, `direction left`.
