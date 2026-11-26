# ADR-002: Portal kinglight-download statis tanpa framework

Status: diterima pemilik lewat delegasi penuh, 2026-11-26.

## Konteks

Portal hanya perlu menampilkan status rilis, artifact metadata, dan tiga halaman kebijakan. Tidak ada akun, basis data, server aplikasi, atau konten yang sering berubah selain rilis.

## Keputusan

HTML, CSS, dan JavaScript statis tanpa build step, disajikan dari cabang `main` melalui GitHub Pages. `release.json` menjadi satu-satunya sumber metadata yang dirender.

## Alasan

Lapisan build tambahan menambah kegagalan deploy tanpa manfaat untuk halaman sesederhana ini. GitHub Pages tersedia pada repo publik yang sudah ada.

## Konsekuensi

Tidak ada komponen UI bersama dengan aplikasi desktop. Kesamaan visual dijaga manual lewat `DESIGN.md`. Metadata rilis harus divalidasi di sisi portal sebelum ditampilkan.
