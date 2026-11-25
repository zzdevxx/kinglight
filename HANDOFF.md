# Handoff Kinglight

Tanggal: 2026-11-26. Tahap: P0, draf perencanaan.

## Otorisasi dan batas

- Pemilik meminta Kinglight desktop bergaya video Kingkong 256A, dengan output Art-Net, serta portal kinglight-download yang terhubung.
- Pemilik memilih dua repo publik pada GitLab, Codeberg, GitHub dan anti-slop DURING.
- Belum ada persetujuan stack, instalasi toolchain, implementasi aplikasi, pengiriman ke node fisik atau deployment situs.
- Pembuatan dua repo publik dan sinkronisasi dokumentasi ke tiga remote diotorisasi pada sesi ini.
- Repo referensi lain tidak boleh diubah; ada pekerjaan aktif di ZZLUXORA.

## Hasil fase dokumen

PRD, arsitektur, desain, model show, kontrak engine, analisis video, sumber dan roadmap tersedia. Portal memiliki rencana dan release contract sendiri. Dokumen tetap proposal sampai tinjauan pemilik.

Temuan utama: video 54,33 detik pada 1080p/30 fps menunjukkan panel VE Project, 16 unit button, enam page selector dan preview beam. Manual KK-256 berbeda: 12 fixture dan lima page. Jangan menyatakan video telah membuktikan emulasi hardware identik.

## Langkah berikutnya

1. Tinjau `PRD.md`, `BLUEPRINT.md`, `ROADMAP.md` bersama pemilik.
2. Pilih Tauri/Rust/Three.js dan frontend, atau PySide6/GPU; pilih stack portal.
3. Tentukan profil enam page software, fixture pertama, target node, dan lisensi.
4. Mulai P1: pembuktian 12 beam dan output loopback pada aplikasi minimal, bukan seluruh panel sekaligus.
5. Catat angka benchmark nyata sebelum mengunci budget P2.

Video dan frame ekstraksi tetap di penyimpanan lokal riset dan tidak disertakan dalam repo publik. Lihat provenance dan hash pada `docs/VIDEO-ANALYSIS.md`.
