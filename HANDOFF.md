# Handoff Kinglight

Diperbarui 2026-11-26. Remake v0.2.0 selesai, dirilis, dan disinkronkan ke tiga remote. Portal live menampilkan metadata dan aset yang terverifikasi. Art-Net hanya diuji ke receiver lokal; node fisik belum diuji. Installer tidak ditandatangani.

## Arah yang ditetapkan pemilik

- Simulator mengikuti panel pada video lokal: konsol biru, master dan 16 fader sejajar, display konteks, dua encoder, menu shape bertingkat, mode penuh dan compact.
- Panggung hitam dengan kipas beam putih menjadi tampilan utama. Pengaturan show, DMX dan Art-Net dibuka sesuai kebutuhan.
- Portal dirombak dengan identitas berbeda dari Banklycs dan ZZLUXORA, responsif, memakai screenshot aplikasi asli dan metadata rilis nyata.
- Stack desktop Tauri 2 + Rust + React + Three.js, portal HTML/CSS/JS statis, lisensi MIT.
- Build, pengujian lokal, rilis baru, deploy portal, commit dan sinkronisasi tiga remote telah diotorisasi.

## Batas tetap

Perubahan hanya di dua repo Kinglight. ZZLUXORA dan referensi lain tetap read-only. Tidak ada pengiriman ke node fisik. Output mulai disarmed; preview tidak mengirim UDP atau memiliki buffer show alternatif. Video, capture lokal, installer dan cache tidak masuk Git.

## Dokumen aktif

[DESIGN.md](DESIGN.md) menjelaskan remake. [PROGRESS.md](PROGRESS.md) menyimpan bukti pengujian. [docs/IPC-CONTRACT.md](docs/IPC-CONTRACT.md) menjadi kontrak implementasi. PRD, blueprint dan roadmap awal menyimpan konteks proposal; keputusan yang sudah diambil di sini menggantikan daftar persetujuan tertunda di dokumen awal.

## Verifikasi rilis v0.2.0

Build Tauri berasal dari commit `90cccb67b3dc3504926fbb1c2b6673c0af0b3b46`. Tes Rust, frontend, portal, native UI, dan Art-Net loopback lulus. Unduhan ulang installer dan portable ZIP cocok ukuran serta SHA-256 dengan snapshot. HEAD kedua repo sama di GitLab, Codeberg, dan GitHub; tag `v0.2.0` menunjuk commit build di ketiga remote. Verifier portal live lulus seluruh pemeriksaan.
