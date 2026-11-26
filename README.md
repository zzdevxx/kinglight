# Kinglight

Simulator konsol lighting untuk Windows, dengan preview beam dan output Art-Net native. Panel direkonstruksi dari video acuan pemilik: konsol biru, master dan 16 fader, enam playback page, display shape dan encoder A/B.

[Unduh aplikasi](https://zzdevxx.github.io/kinglight-download/) · [Rilis](https://github.com/zzdevxx/kinglight/releases/latest)

## Memakai simulator

Buka aplikasi untuk melihat panggung. Pilih unit atau grup, lalu buka **Shape > Playback Shape** untuk memilih gerakan. **Play Shape** menjalankan bentuk di preview. Mode **Compact** menyisakan display dan encoder agar panggung terlihat lebih luas. Pengaturan show dan jaringan tersedia lewat toolbar.

Preview dapat digunakan tanpa perangkat lighting. Art-Net selalu mulai nonaktif; isi target jaringan dan aktifkan output secara eksplisit saat diperlukan. Blackout memadamkan intensitas sambil menahan pan, tilt dan warna. Pengujian node fisik belum dilakukan.

## Pengembangan

Stack: Tauri 2, Rust, React, TypeScript, Vite dan Three.js. Windows memerlukan WebView2. Build Rust memerlukan MSVC Build Tools; installer memakai NSIS.

```powershell
npm ci
npm test
npm run build
pwsh -File scripts/with-msvc.ps1 -Command "cargo test --manifest-path src-tauri/Cargo.toml"
pwsh -File scripts/with-msvc.ps1 -Command "npm run tauri build"
```

Versi ada pada `VERSION`; metadata package, Cargo dan Tauri harus sama. `scripts/make-release.mjs` menghasilkan snapshot rilis dari installer dan arsip portable yang sudah diverifikasi. Biner tidak masuk Git.

## Dokumen

- [Desain dan acuan panel](DESIGN.md)
- [Analisis video](docs/VIDEO-ANALYSIS.md)
- [Kontrak IPC](docs/IPC-CONTRACT.md)
- [Dokumentasi teknis](DOCUMENTATION.md)
- [Bukti pengujian](PROGRESS.md)
- [Handoff](HANDOFF.md)

Implementasi mandiri, tidak menggunakan source, logo atau skin VE Project. Tidak ada klaim kompatibilitas firmware Kingkong atau prediksi fotometrik. Source berlisensi [MIT](LICENSE).

[GitLab](https://gitlab.com/zzdevxx/kinglight) · [Codeberg](https://codeberg.org/zzdevxx/kinglight) · [GitHub](https://github.com/zzdevxx/kinglight). Sinkronisasi Git mencakup commit dan tag, bukan aset rilis atau hosting portal.
