# Progres frontend Kinglight

Diperbarui: 2026-11-26.

## Lingkup

Frontend desktop dibangun hanya di `src/**` dan dokumen ini. Berkas konfigurasi build, `src-tauri/**`, dan repo lain tidak disentuh oleh agen frontend.

## Berkas yang dibuat

| Berkas | Isi |
|---|---|
| `src/main.tsx` | Entry React 19 ke `#root`. Satu mount tanpa StrictMode agar subscription IPC tidak dobel. |
| `src/App.tsx` | Layout konsol, stage, display, show manager, output bar, status koneksi, notice, loading dan error state. |
| `src/types.ts` | Tipe exact dari `docs/IPC-CONTRACT.md`, daftar delapan shape, type guard snapshot. |
| `src/ipc.ts` | `Channel<FrameSnapshot>` dari `subscribe_frames`, `get_snapshot`, dan seluruh command invoke lain sesuai kontrak. |
| `src/hooks/useKinglight.ts` | Subscription live, one-shot snapshot, list show, error, busy per command, dan adoption snapshot dari response invoke. |
| `src/components/StagePreview.tsx` | Three.js stage 16 fixture, beam cone dengan shader additive terkontrol, floor intersection, orbit mouse, preset kamera, quality toggle, Pause Preview, dan disposal GPU resource. |
| `src/components/ConsolePanel.tsx` | Fixture/Playback, bank 1-2, page 1-6, grup Off/A-D, 16 unit, master dan 16 fader, encoder A/B, resolusi Normal/Fine/Ultra. |
| `src/components/DisplayPanel.tsx` | Context menu, selector delapan shape exact, seluruh `ShapeParams`, Apply Shape, Play Shape, Stop Shape. |
| `src/components/OutputBar.tsx` | Armed/Disarmed, blackout, play/stop, target, Port-Address, sequence, sentFrames, lastError, config jaringan, Arm Output, Stop Output. |
| `src/components/ShowManager.tsx` | Save, load, list, refresh, dan feedback hasil command. |
| `src/components/DmxMonitor.tsx` | Grid 256 channel dari `FrameSnapshot.dmx`, angka tabular, monospace untuk nilai DMX. |
| `src/lib/format.ts` | Clamp 16 bit, konversi byte, format angka, dan format error IPC. |
| `src/lib/faders.ts` | Mapping bank/page ke atribut fader. |
| `src/lib/shapes.ts` | Label shape, daftar opsi, dan default `ShapeParams`. |
| `src/lib/helpers.test.ts` | Delapan tes Vitest untuk helper murni. |
| `src/styles.css` | Token charcoal/navy, satu aksen amber, merah hanya untuk blackout/error, border 1 px, focus-visible, tabular numerals, lima state interaktif. |

## Mapping command yang dipilih

Kontrak mendefinisikan `set_fader(attribute: string, value)` tanpa enumerasi nama atribut. Frontend memakai kontrak string berikut agar backend dapat memverifikasi dan menolak nama di luar daftar:

| Kontrol | `attribute` |
|---|---|
| Master | `master` |
| Fader fixture bank 1 | `channel.1` sampai `channel.16` |
| Fader fixture bank 2 | `channel.17` sampai `channel.32` |
| Fader playback page 1-6 | `playback.<page>.<1..16>` |

Nilai yang dikirim selalu 0 sampai 65535. Fader channel menampilkan nilai dari `frame.dmx`. Master dan fader playback menampilkan nilai draft command sampai engine melaporkan keadaan baru lewat response atau channel. Tidak ada nilai output yang diklaim berubah sebelum acknowledgement.

Encoder mengirim `set_encoder(axis, delta, mode)` dengan step 1024 untuk Normal, 128 untuk Fine, dan 16 untuk Ultra. Nilai yang ditampilkan berasal dari `pan` atau `tilt` fixture yang dipilih pada snapshot terbaru.

Kontrol programmer Record, Include, Copy, Delete, Move, Time, Clear, dan Setup tidak dirender karena tidak ada command dengan makna yang cocok di `docs/IPC-CONTRACT.md`. Shape dirender karena `set_shape` dan `shape_play` tersedia.

## Perilaku preview

`Pause Preview` hanya menghentikan pembaruan gambar dan render. Engine, playback, dan state output tidak diubah. Label menyatakan output tetap berjalan saat armed dan tetap disarmed saat belum armed. Renderer membuat 16 beam, 16 intersection disc, satu instanced body mesh, dan floor. Semua geometry, material, observer, listener, dan animation frame dilepas saat unmount. Quality High mengubah device pixel ratio hingga batas 2.

## Verifikasi

Perintah dijalankan dari root repo:

| Perintah | Hasil |
|---|---|
| `npx --yes vitest@latest run src/lib/helpers.test.ts --reporter=verbose` | 1 file, 8 tes, lulus |
| `npx --yes esbuild@latest src/main.tsx --bundle --format=esm --platform=browser --outdir=<temp> --external:react --external:react-dom --external:react-dom/client --external:three --external:@tauri-apps/api/core --loader:.css=css --log-level=warning` | pars dan bundling TSX/TS/CSS lulus tanpa error |
| `tsc 5.8.3 --strict --noEmit` memakai tipe React 19, Three, dan Tauri API 2 dari direktori sementara | lulus tanpa error |

`npm run build` dan `npm run tauri build` belum dijalankan karena `package.json`, `tsconfig.json`, dan toolchain frontend masih menjadi tanggung jawab agen konfigurasi pada saat verifikasi ini. Tidak ada commit atau push dari agen frontend.

## Batas yang belum diuji

- Klik-through pada WebView2 nyata belum dijalankan karena aplikasi belum dibangun.
- Kontras warna hasil render dan operasi pada 1366x768, 1920x1080, serta scaling Windows belum diukur.
- Kompatibilitas string atribut fader perlu diverifikasi terhadap implementasi command Rust setelah command tersebut tersedia.
