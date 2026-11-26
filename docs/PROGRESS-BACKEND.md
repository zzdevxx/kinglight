# Progres Backend Kinglight

Status: backend release candidate. Source Rust berhasil dikompilasi dan seluruh unit test lulus. Tidak ada commit atau push pada sesi ini.

## Ruang lingkup yang dikerjakan

File sumber dan konfigurasi yang dibuat atau diubah:

- `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`
- `src-tauri/build.rs`
- `src-tauri/tauri.conf.json`
- `src-tauri/capabilities/default.json`
- `src-tauri/icons/32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.png`, `icon.ico`
- `src-tauri/src/main.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/model.rs`
- `src-tauri/src/artnet.rs`
- `src-tauri/src/engine.rs`
- `src-tauri/src/storage.rs`
- `src-tauri/src/commands.rs`
- `package.json`, `package-lock.json`
- `vite.config.ts`
- `tsconfig.json`
- `index.html`
- `docs/PROGRESS-BACKEND.md`

Direktori `src-tauri/gen` dan `src-tauri/target` adalah hasil generate `tauri-build` dan `cargo`. Keduanya bukan sumber yang perlu dikirim.

File yang tidak disentuh: `src/**`, `PRD.md`, `BLUEPRINT.md`, `docs/IPC-CONTRACT.md`, `docs/ENGINE-CONTRACT.md`, `docs/PROGRESS-FRONTEND.md`, dan file lain di luar daftar di atas.

## Arsitektur yang diimplementasikan

- Engine adalah satu-satunya sumber state. `EngineState` disimpan di `std::sync::RwLock`. Tick engine berjalan pada 20 ms dan snapshot dikirim ke subscriber paling cepat 30 Hz.
- Output actor memiliki satu `std::net::UdpSocket`. Komunikasi dengan actor memakai `tokio::mpsc` dan `oneshot`. Tidak ada `MutexGuard` std yang dipegang melintasi `await`.
- Status output disimpan di `tokio::sync::RwLock`. Frame terbaru berupa `Arc<FrameBundle>` dipublikasikan ke actor output.
- ArtDmx dibuat dengan header 18 byte dan payload 512 byte, UDP port 6454. Sequence 1..255 lalu kembali ke 1.
- Port-Address raw 15 bit. Nilai 0 ditolak kecuali `compatZero: true`.
- `stop_output` memakai satu frame blackout yang sama untuk dua datagram berurutan pada jarak 30 Hz, lalu actor kembali `disarmed`.
- Blackout menyimpan pan, tilt, dan warna yang sedang resolvenya, lalu memaksa intensity 0 sampai dilepas. Master hanya mengalikan intensity dan tidak mengubah pan, tilt, atau warna.
- IPC API memakai field camelCase persis sesuai kontrak IPC.
- Showfile disimpan sebagai JSON `.kinglight` dengan `schemaVersion: 1`, `profileDigest`, 16 fixture, 6 page, dan shape. Save memakai file sementara lalu replace atomik. Load memvalidasi seluruh dokumen sebelum mengganti state aktif.

## Cakupan test

Unit test mencakup:

- byte header dan payload ArtDmx
- sequence wrap 255 ke 1 tanpa nilai 0
- encoding Port-Address 15 bit
- penolakan Port-Address 0 tanpa `compatZero`
- dua frame stop dengan payload sama dan sequence berurutan
- HTP playback dan override programmer absolut
- latch blackout saat phase shape terus berjalan
- determinisme delapan shape
- master hanya mengubah intensity
- frame 512 byte dengan kanal 129..512 nol
- validasi nama show dan path traversal
- round trip showfile
- penolakan schema tidak didukung, digest mismatch, dan JSON tidak valid

## Verifikasi

Perintah:

```powershell
& C:\Users\andre\.cargo\bin\cargo.exe test --manifest-path D:/ANDREAS/PERSONAL/kinglight/src-tauri/Cargo.toml
```

Toolchain:

```text
rustc 1.100.0 (0ef1bc386 2026-11-09)
cargo 1.100.0 (3acc79c0c 2026-11-05)
node v26.13.1
npm 12.2.3
```

Hasil akhir:

```text
Compiling kinglight v0.1.0 (D:\ANDREAS\PERSONAL\kinglight\src-tauri)
Finished `test` profile [unoptimized + debuginfo] target(s) in 1m 12s
Running unittests src\lib.rs (src-tauri\target\debug\build\kinglight/750408a829fa0e59\out\kinglight_lib-750408a829fa0e59.exe)

running 16 tests
test artnet::tests::port_address_is_encoded_as_raw_15_bits ... ok
test artnet::tests::art_dmx_header_and_payload_match_wire_contract ... ok
test artnet::tests::sequence_wraps_from_255_to_1_and_never_emits_zero ... ok
test artnet::tests::port_address_rejects_zero_unless_compat_zero_is_enabled ... ok
test artnet::tests::stop_frames_share_payload_and_use_consecutive_sequences ... ok
test engine::tests::htp_selects_highest_playback_and_programmer_overrides_absolutely ... ok
test engine::tests::blackout_latch_holds_pan_tilt_and_color_while_phase_advances ... ok
test engine::tests::frame_has_512_bytes_and_only_first_128_channels_are_written ... ok
test engine::tests::master_scales_intensity_without_changing_pan_or_tilt ... ok
test engine::tests::shape_kind_is_validated_against_the_eight_names ... ok
test engine::tests::programmer_override_applies_only_to_masked_attributes ... ok
test engine::tests::shape_values_are_deterministic_and_distinct_per_kind ... ok
test storage::tests::path_traversal_names_are_rejected_before_touching_disk ... ok
test storage::tests::profile_digest_mismatch_is_rejected ... ok
test storage::tests::show_document_round_trip_preserves_fixtures_pages_and_shape ... ok
test storage::tests::unsupported_schema_and_bad_json_do_not_replace_show ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

Running unittests src\main.rs (src-tauri\target\debug\build\kinglight/fb9d11d191a63f84\out\kinglight-fb9d11d191a63f84.exe)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

Doc-tests kinglight_lib

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Dependensi frontend dipasang dari versi latest dengan flag save exact. Versi yang terkunci:

```text
@tauri-apps/api 2.12.5
@tauri-apps/cli 2.12.6
@types/three 0.187.0
@vitejs/plugin-react 6.3.0
react 19.3.1
react-dom 19.3.1
three 0.187.0
typescript 7.0.2
vite 8.3.8
vitest 5.1.2
```

`npm install` selesai dengan 0 vulnerabilities. `package-lock.json` dibuat.

Scan sederhana pada file sumber yang dimiliki backend tidak menemukan penanda placeholder, tanda hubung panjang, atau karakter non-ASCII.

## Batas yang belum diuji

- `tauri build` tidak dijalankan sesuai instruksi.
- Build dan typecheck frontend tidak dijalankan karena `src/**` dimiliki agen lain. Konfigurasi Vite, TypeScript, dan HTML sudah disiapkan.
- Uji loopback UDP dan penerima Art-Net fisik belum dijalankan. Yang terbukti adalah konstruksi paket dan frame stop pada unit test.
- Target 50 Hz engine dan 30 Hz output belum diukur pada mesin sasaran.
- `save_show` dan `load_show` memakai direktori data aplikasi dengan nama yang disanitasi, bukan dialog path bebas. Bila dialog file diwajibkan, tanda tangan command harus diubah.
- Tidak ada commit, push, sinkronisasi tiga remote, instalasi toolchain, publishing, atau output fisik ke node panggung pada sesi ini.
