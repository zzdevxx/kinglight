# Handoff Kinglight

Diperbarui 2026-11-26. Remake v0.2.0, build dari commit sumber, pengujian native, dan GitHub Release selesai. Snapshot metadata sudah dibuat dari aset terverifikasi. Commit snapshot producer, commit portal, sinkronisasi tiga remote, dan verifikasi GitHub Pages masih tertunda.

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

## Verifikasi sebelum rilis

Tes Rust, frontend, portal, dan operasi native sudah dijalankan. Sebelum menutup rilis, build ulang Tauri dari commit sumber, unggah serta unduh kembali dua artefak, cocokkan ukuran dan hash, buat snapshot portal, jalankan tes portal dan verifikasi live. Setelah itu sinkronkan ketiga remote dan cocokkan HEAD serta tag.
