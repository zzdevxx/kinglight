# Kinglight

Rencana aplikasi desktop untuk latihan mengoperasikan konsol lighting bergaya Kingkong 256A, melihat gerakan beam, dan mengirim DMX melalui Art-Net.

**Status: tahap perencanaan. Aplikasi, installer, dan keluaran Art-Net belum diimplementasikan.**

## Arah produk

- Panel dengan bank fader, tombol fixture/playback, programmer, scene, chase, shape, dan encoder pan/tilt.
- Preview panggung dengan beam bergerak, mengikuti frame DMX akhir yang sama dengan output jaringan.
- Engine konsol yang tetap bekerja ketika preview melambat.
- Pengoperasian lokal tanpa akun atau layanan cloud saat memainkan show.
- Distribusi melalui proyek pendamping [kinglight-download](https://github.com/zzdevxx/kinglight-download).

Video referensi memperlihatkan antarmuka bertuliskan `VE Project`. Identitas, source code, dan lisensinya belum terverifikasi. Kinglight direncanakan sebagai implementasi mandiri; belum ada klaim kompatibilitas penuh dengan firmware atau format show Kingkong.

## Dokumen rencana

| Dokumen | Isi |
|---|---|
| [PRD.md](PRD.md) | Sasaran, ruang lingkup, dan kriteria penerimaan |
| [BLUEPRINT.md](BLUEPRINT.md) | Pilihan stack, batas komponen, dan keputusan yang masih terbuka |
| [ROADMAP.md](ROADMAP.md) | Tahapan pengerjaan dan gerbang verifikasi |
| [ERD.md](ERD.md) | Entitas showfile dan hubungan datanya |
| [DESIGN.md](DESIGN.md) | Arah panel, preview, aksesibilitas, dan alasan visual |
| [DOCUMENTATION.md](DOCUMENTATION.md) | Kontrak teknis dan indeks referensi |
| [Analisis video](docs/VIDEO-ANALYSIS.md) | Temuan bertimestamp dan batas buktinya |
| [Sumber riset](docs/SOURCES.md) | Manual, spesifikasi, dan referensi GitHub |
| [Kontrak engine](docs/ENGINE-CONTRACT.md) | Resolusi atribut, output, dan persistensi |
| [HANDOFF.md](HANDOFF.md) | Status dan pekerjaan sesi berikutnya |

## Repositori

[GitLab](https://gitlab.com/zzdevxx/kinglight) · [Codeberg](https://codeberg.org/zzdevxx/kinglight) · [GitHub](https://github.com/zzdevxx/kinglight)

Ketiga remote menyimpan riwayat Git. Sinkronisasi Git tidak otomatis menyalin installer, release assets, atau deployment portal.

Lisensi distribusi kode belum dipilih. Belum ada kode pihak lain yang disalin ke proyek ini.
