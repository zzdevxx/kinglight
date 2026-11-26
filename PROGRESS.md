# Progres Kinglight

Diperbarui: 2026-11-26.

## Tahap selesai

| Tahap | Status | Bukti |
|---|---|---|
| P0 riset dan dokumen | selesai | PRD, BLUEPRINT, ERD, desain, roadmap, kontrak engine, analisis video, sumber |
| P0 repo tiga platform | selesai | commit `b6754ad` identik di GitLab, Codeberg, GitHub |
| P1 keputusan stack | selesai | Tauri 2 + Rust + React 19 + Three.js, portal HTML/CSS/JS statis |
| P1 toolchain | selesai | Rust 1.100, MSVC Build Tools 17.14, NSIS 3.13 |
| P1 build installer | selesai | `Kinglight_0.1.0_x64-setup.exe`, 2.101.309 byte, build ke-2 setelah perbaikan runtime |

## Temuan dan perbaikan

| Temuan | Bukti | Perbaikan |
|---|---|---|
| Aplikasi crash saat start, ke-1 | exit code `-1073740791`, log: `thread 'main' panicked at src/artnet.rs:140:9: there is no reactor running` | `tokio::spawn` diganti `tauri::async_runtime::spawn` di `src-tauri/src/artnet.rs` |
| Aplikasi crash saat start, ke-2 | exit code `-1073740791`, log: `thread 'main' panicked at src/lib.rs:72:9: there is no reactor running` | `tokio::spawn` diganti `tauri::async_runtime::spawn` untuk loop engine di `src-tauri/src/lib.rs` |
| Build frontend gagal | `TS7026` karena `@types/react` tidak terpasang | `@types/react` dan `@types/react-dom` ditambahkan |
| DMX monitor menutupi panel output | uji klik: input target tidak bisa menerima teks, snapshot area output | tinggi workspace dibatasi, `dmx-grid` memakai `height: 100%`, `output-dock` diberi z-index dan latar |
| Show manager tertutup footer output | uji klik tombol Save tidak mencapai elemen | `app-shell` diubah dari grid menjadi flex column, header/notice/footer `flex: 0`, workspace `flex: 1` dengan `min-height: 0` |
| Baris Saved shows terpotong | panel show tidak bisa di-scroll | `.show-panel` diberi `overflow-y: auto` |

## Bukti pengujian aplikasi

| Pengujian | Hasil |
|---|---|
| `cargo test` setelah perbaikan runtime | 16 lulus, 0 gagal |
| `npm test` | 8 lulus, 0 gagal |
| `npm run build` | lulus, bundle 792,97 kB (211,46 kB gzip) |
| anti-slop strict | 0 error, 0 warning |
| Instalasi NSIS silent ke folder uji | `kinglight.exe` dan `uninstall.exe` hadir |
| Start aplikasi | proses tetap hidup, jendela `Kinglight` responsive |
| UI | preview panggung, 16 unit, 6 page, grup, encoder, shape, output bar, DMX monitor, show manager |
| Arm + Play ke loopback | Armed, Playing, 2.389 frame terkirim, sequence berubah |
| Verifikasi paket UDP 60 detik | 1.121 paket, 530 byte, Port-Address 0, sequence 1..255, ProtVer 14, 0 error |
| Blackout | status berubah, intensity kanal 1 tiap fixture nol, pan/tilt dan warna ditahan |
| Stop Output | status Disarmed, tidak ada paket dalam 4 detik berikutnya |
| Save show | berkas `smoke.kinglight` 19.731 byte, `schemaVersion: 1`, 16 fixture, 6 page |
| Load show | notifikasi "Show loaded." muncul setelah memilih `smoke` dan menekan Load |
| Panel show dapat di-scroll | baris Saved shows dan tombol Load terjangkau setelah scroll |

## Tahap berjalan

| ID | Pekerjaan | Owner |
|---|---|---|
| A | Backend Rust: engine, ArtDmx, perintah Tauri, tes | agen backend |
| B | Frontend React + Three.js, panel dan preview | agen frontend |
| C | Portal statis dan penguji kontrak rilis | agen portal |

## Gerbang berikutnya

1. `cargo test` lulus untuk packet, sequence, merge, blackout, shape, storage.
2. `node --test` portal lulus.
3. `npm run build` frontend lulus.
4. `npm run tauri build` menghasilkan installer NSIS.
5. Uji loopback ArtDmx dan klik-through UI dengan Playwright.
6. Release GitHub, snapshot portal, GitHub Pages aktif.
7. Sinkronisasi tiga remote dan verifikasi hash.

## Aturan progres

Tahap tidak ditandai selesai sebelum perintah verifikasi benar-benar dijalankan dan hasilnya dicatat di sini. Tidak ada klaim performa tanpa pengukuran.
