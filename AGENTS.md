# Panduan kerja Kinglight

## Batas wajib

1. Fokus perubahan hanya `kinglight` dan `kinglight-download`. Referensi ZZLUXORA, QLC+ dan REA read-only; jangan menjalankan build/test yang menulis cache di repo referensi.
2. Tahap sekarang perencanaan. Jangan scaffold aplikasi sebelum pemilik meninjau rencana dan memilih stack. Pembuatan remote publik, commit dokumentasi, dan sinkronisasi tiga remote untuk kedua repo telah diotorisasi pada sesi ini.
3. Baca `HANDOFF.md`, `PRD.md`, `BLUEPRINT.md`, lalu dokumen sesuai area kerja.
4. Periksa Git status, diff, branch dan remotes sebelum mengubah repo. Jangan menghapus pekerjaan agen lain.
5. State otoritatif berada pada engine. UI dan preview tidak boleh mengirim UDP atau menjadi sumber buffer alternatif.
6. Blackout fixture-aware; jangan menganggap seluruh channel nol aman untuk semua fixture.
7. Bedakan observasi video, manual hardware, proposal dan hasil pengujian. Dilarang mengklaim fitur/performa yang belum dibuktikan.
8. Gunakan anti-slop DURING untuk proyek ini. Tanpa emoji dekoratif, em dash, placeholder implementasi atau angka marketing fiktif.
9. Jangan menyalin source/aset referensi sebelum memeriksa lisensi dan provenance. Lisensi Kinglight masih menunggu keputusan.
10. Jangan commit rahasia, video, installer, capture, showfile pengguna atau cache. Repo publik.
11. Sinkronisasi wajib ke `origin` GitLab, `codeberg`, `github` melalui tool sync workspace, lalu verifikasi hash remote. Tidak ada force push atau rewrite.
12. Perubahan output fisik, instalasi toolchain, publishing/deployment mengikuti otorisasi sesi; jangan menganggap proposal dokumentasi sebagai izin menjalankan layanan atau node fisik.

## Peta dokumen

| Area | Baca |
|---|---|
| Produk dan tahap | `PRD.md`, `ROADMAP.md` |
| UI/preview | `DESIGN.md`, `docs/VIDEO-ANALYSIS.md` |
| Domain/persistence | `ERD.md`, `docs/ENGINE-CONTRACT.md` |
| Output | `DOCUMENTATION.md`, `docs/SOURCES.md` |
| Portal/release | Repo pendamping, `docs/RELEASE-CONTRACT.md` di sana |

## Definisi selesai

Untuk dokumen: tautan lokal valid, tidak ada konflik kapasitas/istilah, proposal diberi status, sumber faktual tercantum, diff dan scan anti-slop ditinjau.

Untuk kode setelah disetujui: build/test relevan benar-benar dijalankan, jalur output diuji loopback, UI yang berubah dioperasikan, hasil dan batas dicatat. Jangan menulis perintah build khayalan sebelum toolchain tersedia.
