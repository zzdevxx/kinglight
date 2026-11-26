# Changelog Kinglight

## 0.2.0 - 2026-11-26

### Diubah

- Aplikasi membuka konsol compact di atas panggung hitam. Panel penuh menampilkan master dan 16 fader sejajar dengan display di kanan.
- Menu shape mengikuti alur bertingkat; parameter tampil dengan satuan konsol.
- Pengaturan show, Art-Net, dan monitor DMX dibuka saat diperlukan.
- Portal unduhan memakai tampilan baru dan membaca metadata artefak installer serta portable.

### Ditambahkan

- Cue menyimpan nilai logis setiap fixture, termasuk saat dicopy, dipindahkan, dimuat, dan dimainkan kembali.
- Kategori shape Dimmer, RGB, dan Color dengan parameter kategori masing-masing.
- Arsip portable Windows x64 yang berisi executable simulator.

### Diperbaiki

- Fan beam preview dipersempit untuk mendekati komposisi frame awal video.
- Tombol resolusi encoder ditempatkan di antara encoder A dan B.
- Nilai BPM pecahan terlihat pada resolusi Fine dan Ultra.
- Repeat spasial tidak mengubah laju BPM shape.

Batas pengujian: Art-Net diuji ke receiver UDP loopback. Node lighting fisik dan penandatanganan kode belum diuji.

## 0.1.0 - 2026-11-26

### Ditambahkan

- Aplikasi desktop Tauri, engine Rust, panel React dan preview Three.js.
- Output ArtDmx native, blackout, master dan showfile lokal.
- Installer Windows, portal unduhan statis dan metadata rilis terukur.
- Dokumen acuan, kontrak IPC dan pengujian loopback.
