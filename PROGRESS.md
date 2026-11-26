# Progres Kinglight

Diperbarui: 2026-11-26.

## Bukti remake v0.2.0

Status: rilis v0.2.0 diterbitkan dari commit `90cccb67b3dc3504926fbb1c2b6673c0af0b3b46`. Kedua repo tersinkron ke tiga remote. GitHub Pages live dan metadata serta hash aset terverifikasi.

| Pengujian | Hasil teramati |
|---|---|
| `cargo test --manifest-path src-tauri/Cargo.toml` | 16 tes library dan 18 tes kontrak native lulus; 0 gagal |
| `npm test` | 21 tes frontend lulus; 0 gagal |
| `npm run build` | TypeScript dan Vite lulus; Vite memperingatkan bundle JavaScript setelah minifikasi melebihi 500 kB |
| `npm run tauri build` | `kinglight.exe` dan installer NSIS Windows x64 v0.2.0 berhasil dibuat |
| Portable ZIP | Memuat satu `kinglight.exe`; executable dijalankan langsung dari arsip yang diekstrak |
| Smoke native | 19 pemeriksaan lulus: output awal disarmed, 12 fixture putih, fader, encoder, record/copy/move/include, show round-trip, shape, dan blackout |
| Smoke UI native | 5 pemeriksaan lulus: compact awal, 16 fader dan master, posisi encoder Normal/Fine/Ultra, Shape Parameter, serta Play/Stop tanpa mengaktifkan output |
| Art-Net loopback | 11 paket ArtDmx, masing-masing 530 byte; sequence 1..11 berurutan, Port-Address 1, output disarmed sesudah Stop Output dan tidak ada paket lanjutan |
| Portal | 27 tes lulus; render lokal 320, 375, 768, 1024, 1440, 1920 px tanpa overflow; screenshot 1440x810 dimuat dan fallback tersembunyi; console error 0 |
| Preview | Fan pada startup diukur dan dipersempit terhadap frame awal video; screenshot aplikasi aktual tersedia di portal |
| GitHub Release | `https://github.com/zzdevxx/kinglight/releases/tag/v0.2.0`, dipublikasikan `2026-11-26T06:18:17Z`, tag menunjuk commit build |
| Installer NSIS | 2.126.488 byte; SHA-256 `8166f838dcf283ba17e36a034dca5fc1f45a62582d2eb2f52a30e32b5a9e88eb` |
| Portable ZIP | 2.382.842 byte; SHA-256 `3f18c8e324bf919b50443f11110c3c59663efcc4ed44cdec05115c64c7a7f357` |
| Sinkronisasi | HEAD kedua repo sama di GitLab, Codeberg, dan GitHub; tag `v0.2.0` menunjuk commit build |
| Portal live | Verifier lulus sembilan pemeriksaan; browser menampilkan v0.2.0, dua unduhan, screenshot asli, dan tanpa error console |

Batas bukti: Art-Net hanya diuji ke receiver lokal, bukan node fisik. Installer belum dipasang pada sesi ini karena terdapat instalasi uji v0.1.0 terdaftar. Installer tidak ditandatangani. Tidak ada kredensial atau output ke fixture fisik yang digunakan.

## Riwayat implementasi remake

Status: implementasi dan pengujian lokal selesai. Pemilik meminta tampilan mengikuti video, termasuk mode compact sebagai tampilan awal. Desain dashboard rilis awal diganti dengan konsol biru di atas panggung hitam. Portal memakai screenshot aplikasi yang benar-benar dijalankan.

Koreksi yang ditemukan saat review: pemetaan fader harus relatif terhadap fixture pilihan, nilai encoder tidak boleh dikalikan dua kali, pergantian resolusi tidak boleh membuat mask programmer, dan input repeat harus mengubah satuan display ke nilai internal. Menu, warna tombol dan susunan strip ditinjau terhadap frame video.

Tes kalibrasi panggung: lima tes lulus. Generator metadata diuji dengan dua artefak fixture dalam direktori sementara; ukuran dan SHA-256 keduanya cocok. Build ulang, unduhan aset rilis, dan verifikasi portal live selesai.

## Riwayat rilis awal

| Tahap | Status | Bukti |
|---|---|---|
| P0 riset dan dokumen | selesai | PRD, BLUEPRINT, ERD, desain, roadmap, kontrak engine, analisis video, sumber |
| P0 repo tiga platform | selesai | commit `b6754ad` identik di GitLab, Codeberg, GitHub |
| P1 keputusan stack | selesai | Tauri 2 + Rust + React 19 + Three.js, portal HTML/CSS/JS statis |
| P1 toolchain | selesai | Rust 1.100, MSVC Build Tools 17.14, NSIS 3.13 |
| P1 build installer | selesai | `Kinglight_0.1.0_x64-setup.exe`; aset rilis final tercantum di bagian rilis awal |

## Temuan dan perbaikan

| Temuan | Bukti | Perbaikan |
|---|---|---|
| Aplikasi crash saat start, ke-1 | exit code `-1073740791`, log: `thread 'main' panicked at src/artnet.rs:140:9: there is no reactor running` | `tokio::spawn` diganti `tauri::async_runtime::spawn` di `src-tauri/src/artnet.rs` |
| Aplikasi crash saat start, ke-2 | exit code `-1073740791`, log: `thread 'main' panicked at src/lib.rs:72:9: there is no reactor running` | `tokio::spawn` diganti `tauri::async_runtime::spawn` untuk loop engine di `src-tauri/src/lib.rs` |
| Build frontend gagal | `TS7026` karena `@types/react` tidak terpasang | `@types/react` dan `@types/react-dom` ditambahkan |
| DMX monitor menutupi panel output | uji klik: input target tidak bisa menerima teks, snapshot area output | tinggi workspace dibatasi, `dmx-grid` memakai `height: 100%`, `output-dock` diberi z-index dan latar |
| Show manager tertutup footer output | uji klik tombol Save tidak mencapai elemen | `app-shell` diubah dari grid menjadi flex column, header/notice/footer `flex: 0`, workspace `flex: 1` dengan `min-height: 0` |
| Baris Saved shows terpotong | panel show tidak bisa di-scroll | `.show-panel` diberi `overflow-y: auto` |

## Bukti pengujian rilis awal v0.1.0

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

## Tahap rilis v0.2.0

| Tahap | Status |
|---|---|
| Tes ulang Rust, frontend dan portal | lulus |
| Commit source, lalu build dan smoke dari commit yang sama | lulus, commit `90cccb6` |
| Unggah dan ukur installer serta ZIP portable di GitHub Release | lulus, unduhan ulang cocok ukuran dan hash |
| Buat metadata portal dari aset yang diunduh dan diverifikasi | lulus lokal, snapshot `v0.2.0` |
| Commit metadata producer dan portal, sinkronkan tiga remote, verifikasi GitHub Pages | selesai |

## Hasil publikasi

1. Snapshot producer menunjuk source commit build `90cccb6`; snapshot portal berasal dari aset yang diunduh ulang.
2. Tes portal lulus 27 pemeriksaan. Browser lokal dan live memverifikasi layout, screenshot, CTA, tombol salin hash, dan console.
3. Sinkronisasi menyamakan HEAD kedua repo pada `origin`, `codeberg`, dan `github`. Tag `v0.2.0` tersedia di ketiganya dan menunjuk commit build.
4. GitHub Pages menampilkan v0.2.0. Skrip verifikasi live mengonfirmasi halaman, schema, ukuran, dan SHA-256 kedua aset.

## Rilis awal dan web

| Item | Nilai terverifikasi |
|---|---|
| Tag rilis | `v0.1.0`, commit `304b762310f534c542c8c91bc6518481c8148af6` |
| Installer | `Kinglight_0.1.0_x64-setup.exe`, 2.102.443 byte |
| SHA-256 | `fcf8ec0aa03411d803a56d255671368bd84d162b251dcc0a1308790e5ffb4525` |
| GitHub Release | `https://github.com/zzdevxx/kinglight/releases/tag/v0.1.0` |
| Portal live | `https://zzdevxx.github.io/kinglight-download/` |
| Hash Unduhan | installer yang diunduh ulang dari GitHub Release memiliki SHA-256 sama dengan lokal dan portal |
| Sinkronisasi | HEAD lokal sama dengan `origin`, `codeberg`, `github` untuk kedua repo |

## Aturan progres

Tahap tidak ditandai selesai sebelum perintah verifikasi benar-benar dijalankan dan hasilnya dicatat di sini. Tidak ada klaim performa tanpa pengukuran.
