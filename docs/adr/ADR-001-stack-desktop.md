# ADR-001: Stack desktop Kinglight

Status: diterima pemilik lewat delegasi penuh, 2026-11-26.

## Konteks

Kinglight membutuhkan panel konsol padat, preview beam bergerak, dan satu pengirim Art-Net native yang tidak terikat render UI. Video referensi menempatkan beam sebagai hasil utama, bukan dekorasi.

## Keputusan

Tauri 2 dengan backend Rust, frontend React 19 dan TypeScript, serta Three.js untuk preview. Output UDP dimiliki actor Rust. Frontend hanya mengirim perintah dan menerima snapshot.

## Alasan

Rust memberikan kepemilikan socket yang jelas dan tipe 16-bit tanpa GIL. Three.js menyediakan kamera, material, dan instancing untuk 16 beam. React cocok untuk panel stateful. Toolchain Rust tidak tersedia pada awalnya tetapi dapat dipasang, dan jaringan serta ruang disk mencukupi.

## Alternatif

PySide6 lebih cepat untuk widget desktop tetapi memerlukan argumen GIL dan embedding GPU yang harus dibuktikan. Electron lebih berat dan memisahkan UDP ke utility process. Keduanya tidak dipilih.

## Konsekuensi

Perlu Rust, NSIS, WebView2, dan build yang lebih lama. Kontrak IPC menjadi titik integrasi kritis. Showfile tetap JSON berversi sehingga dapat dipindah ke stack lain bila diperlukan.
