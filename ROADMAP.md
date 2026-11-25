# Tahapan pengerjaan

Status: P0 perencanaan. Estimasi durasi belum dibuat karena stack, fixture uji, dan target hardware belum dipilih.

Satu tahap ditutup dengan bukti, daftar batas yang tersisa, dan handoff. Fitur tahap berikutnya tidak diberi tombol aktif sebelum berfungsi.

| Tahap | Hasil konkret | Gerbang selesai |
|---|---|---|
| P0, riset dan keputusan | Analisis video/manual, PRD, arsitektur, model show, roadmap, dua repo publik | Referensi bertimestamp, konflik varian dicatat, pilihan stack dan prioritas ditinjau pemilik |
| P1, pembuktian teknis | Aplikasi minimal: 12 beam, pan/tilt 16-bit, kontrol sederhana, monitor byte, satu sender loopback | Paket ArtDmx benar; renderer dan output tetap berjalan terpisah; benchmark awal dan instalasi toolchain dicatat |
| P2, engine konsol | Patch, programmer, HTP/LTP, master, blackout, fixture calibration, save/load minimal | Replay deterministik; 16-bit tidak terpecah saat fade; patch invalid dan load rusak tidak mengubah state |
| P3, panel dan scene | Bank 16 fader, master, unit/group/page, encoder, display, record/include/copy/move/delete, scene modes | Alur lengkap select, edit, record, clear, playback menghasilkan frame yang diharapkan; keyboard dan fokus diuji |
| P4, chase dan shape | Timing HW/HF/LW/LF, chase steps, pan/tilt shapes dan kontrol dalam video | Semua parameter memiliki definisi; perubahan direction/interleave/fixture order menghasilkan urutan deterministik; batas phase dan release diuji |
| P5, beta Art-Net dan preview | NIC, target raw Port-Address, error, reconnect, output ownership, kualitas preview | Uji QLC+ atau receiver terkontrol, lalu node fisik; p95/p99 terukur; cable loss, sleep/resume, renderer loss dan shutdown diuji. ArtPoll/subscription menjadi spike terpisah sebelum klaim controller Art-Net 4 lengkap |
| P6, packaging dan portal | Installer Windows, source version, hash/size nyata, release metadata; portal dengan halaman kebijakan | Fresh-install pada lingkungan bersih, uninstall, link/checksum benar, metadata tidak bisa menunjuk asset belum tersedia |
| P7, kandidat rilis | Regression, satu jam lalu delapan jam soak, dokumentasi operator dan lisensi | Tidak ada backlog tak terbatas; hasil benchmark dapat direproduksi; semua batas release dinyatakan; keputusan publish spesifik tersedia |

## Urutan yang disengaja

Art-Net loopback dan preview sudah hadir pada P1. Output fisik baru diuji setelah kebijakan blackout dan fixture diketahui. Ini menghindari pembangunan panel panjang sebelum jalur utama terbukti.

Showfile minimal hadir pada P2 agar pekerjaan operator tidak hilang selama pengembangan. Format backup hardware proprietary belum termasuk.

Portal direncanakan sejak P0 dan mulai dibangun setelah framework dipilih. Portal boleh terbit dengan status belum ada rilis, tetapi tidak boleh menawarkan installer fiktif. Integrasi rilis nyata ditutup pada P6.

## Gate P1 yang menentukan stack

1. Build bersih dari checkout dengan toolchain yang dipilih.
2. Render 12 beam pada 1080p; ukur frame time dan penggunaan memori.
3. Jalankan satu universe ke receiver loopback dengan payload yang dapat diperiksa.
4. Gerakkan fader, ubah shape, resize dan minimize; output tidak bergantung pada render callback.
5. Engkoder 16-bit dan blackout terlihat konsisten pada byte, monitor dan preview.
6. Jika target tidak tercapai, ukur penyebab sebelum beralih stack atau menambah proses.

## Matriks pengujian

| Area | Kasus yang harus membedakan hasil benar/salah |
|---|---|
| Patch | Start 1, end 256, coarse/fine keluar mode, overlap, 16 x 32 channel ditolak pada budget 256 |
| Merge | HTP max, LTP activation order, programmer Clear, release flash saat cue dasar masih bergerak |
| Fade | 65535 ke 0, coarse-byte boundary, discrete wheel, hold/fade nol, stall clock, reverse chase |
| Shape | Phase boundary, BPM 0, negative/invalid size, fixture order, odd/even, repeat dan spread |
| Blackout | Ditekan saat drag, change page, cue load, save, output reconfigure; pan/tilt tetap sesuai kebijakan |
| Paket | ID, endian opcode, protocol 14, length 512, raw Port-Address, sequence 255 ke 1, reserved channels |
| Lifecycle | Repeated Arm/Stop, NIC hilang, error socket, suspend/resume, keluar aplikasi, engine gagal |
| Showfile | Round-trip, future schema, profile digest mismatch, file terpotong, bounded input, save gagal |
| Preview | Fixture calibration, beam hit floor, occlusion sederhana, hidden/minimized, GPU context loss, low quality |
| UI | Lima state interaksi, tab order, keyboard fader, modal Escape, status loading/empty/error |
| Release | Tag/version mismatch, checksum mismatch, missing asset, wrong host URL, schema baru, portal belum ada release |

Pengujian domain memakai clock deterministik. Mock dibatasi ke fixture pengujian; klaim dukungan node fisik memerlukan pengujian node nyata.

## Setelah versi awal

Pertimbangkan dimmer/RGB/color shapes lanjutan, import QXF/OFL dengan provenance, sACN, MIDI/OSC, multi-universe, stage editor dan integrasi visualizer eksternal sesuai kebutuhan yang terbukti. Setiap tambahan harus menyebut perubahan format show, output timing, dan lisensi.
