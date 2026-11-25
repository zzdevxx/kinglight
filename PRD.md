# Kebutuhan produk Kinglight

Tanggal: 2026-11-26. Status: draf untuk ditinjau pemilik sebelum implementasi.

## 1. Tujuan

Kinglight membantu operator lighting berlatih membuat dan memainkan show dengan alur konsol bergaya Kingkong 256A. Operator melihat hasilnya melalui preview beam dan dapat mengirim frame yang sama ke node Art-Net.

Rujukan kebutuhan adalah video yang diberikan pemilik, manual KK-256, serta studi source QLC+ dan ZZLUXORA secara read-only. Video dan manual menunjukkan varian berbeda; keduanya tidak boleh digabung menjadi klaim spesifikasi hardware tunggal.

## 2. Pengguna dan pekerjaan utama

| Pengguna | Pekerjaan yang harus bisa diselesaikan |
|---|---|
| Operator yang belajar konsol | Memilih fixture, mengubah channel, merekam scene, menjalankan chase dan shape |
| Operator yang menyiapkan show | Mengatur patch, menyimpan show lokal, membuka ulang tanpa otomatis menyalakan output |
| Operator dengan node Art-Net | Memilih NIC, alamat tujuan, Port-Address, lalu mengaktifkan output dengan sadar |

Tidak ada target jumlah pengguna, klaim waktu belajar, atau klaim performa yang telah diukur.

## 3. Ruang lingkup awal

| ID | Kebutuhan | Kriteria penerimaan |
|---|---|---|
| K-01 | Panel konsol yang dapat dioperasikan | Bank 16 fader, master, 16 tombol unit, pergantian fungsi fixture/playback, tampilan konteks, dan encoder dapat dipakai dengan mouse dan keyboard |
| K-02 | Programmer | Perubahan atribut memiliki mask eksplisit; Clear melepaskan programmer tanpa menghapus cue tersimpan |
| K-03 | Patch 256 channel | Alamat DMX ditampilkan 1-based; overlap dan footprint melewati channel 256 ditolak sebelum state berubah |
| K-04 | Atribut 8/16-bit | Pan/tilt coarse-fine diinterpolasi sebagai satu nilai logis; pemecahan byte hanya di tahap encode |
| K-05 | Scene dan playback | Capture by fixture/channel, record, replace/merge, include, copy, move, delete, dan release memiliki aturan yang diuji |
| K-06 | Chase | Urutan step, hold/fade, arah, loop, waktu bersama/per-step, dan pergantian step deterministik pada clock uji |
| K-07 | Shape | Circle, Circle dua arah, Figure 8, Spiral, Square, Diamond, Pan Swing, Tilt Swing; size, BPM, repeat, spread, interleave, direction dan urutan fixture didefinisikan |
| K-08 | Preview beam | Panggung, penempatan fixture, pan/tilt, dimmer, RGB, zoom dasar, dan beam kerucut mengikuti frame DMX akhir; target awal adegan 12 beam putih seperti referensi |
| K-09 | Art-Net nyata | Native UDP ArtDmx, satu universe, payload 512 byte, Port-Address eksplisit, sequence, adapter jaringan dan telemetri error |
| K-10 | Kendali output | Start/load selalu disarmed; blackout, master, Stop Output, dan Pause Preview memiliki perilaku terpisah dan jelas |
| K-11 | Showfile | Save atomik dan load tervalidasi; file rusak/versi tidak dikenal tidak mengganti show yang sedang dibuka |
| K-12 | Distribusi | Installer terverifikasi menghasilkan metadata untuk kinglight-download tanpa pengetikan ulang versi/hash |

Katalog shape dimmer/RGB/color yang tampak pada video masuk tahapan lanjutan setelah shape pan/tilt. Tidak boleh ada tombol aktif untuk fungsi yang belum diimplementasikan.

## 4. Profil konsol

| Profil riset | Bukti | Perlakuan |
|---|---|---|
| Video VE Project | 16 tombol unit, enam selector playback page, 16 fader dan master | Acuan pengalaman Kinglight; enam page memberi rancangan 96 slot software, belum bukti 96 slot berfungsi di aplikasi referensi |
| Manual KK-256 | 12 fixture, 5 page x 16 playback, 100 chase steps, maksimum 32 channel per fixture | Baseline semantik programmer, HTP/LTP, scene dan chase |
| Vendor KK-256A | Sebagian halaman menyebut 16 fixture; jumlah shape berbeda antarsumber | Dicatat sebagai varian, tidak dijadikan angka universal |

Proposal awal: 16 slot fixture dan enam page pada profil Kinglight yang mengikuti video. Jumlah fixture dan slot memori bukan jumlah channel. Enam belas fixture dengan footprint 32 tidak dapat semuanya menempati 256 channel tanpa overlap.

Mode kompatibilitas hardware lengkap, konversi showfile USB, dan pemilihan profil manual alternatif menunggu bukti tambahan serta keputusan pemilik.

## 5. Alur inti

1. Buka aplikasi dalam kondisi output disarmed.
2. Pilih profil fixture dan patch yang valid, lalu tempatkan fixture pada panggung.
3. Pilih fixture/group, ubah fader atau encoder, dan lihat preview serta monitor DMX.
4. Rekam scene; Clear programmer; jalankan playback untuk memeriksa hasil yang benar-benar tersimpan.
5. Tambahkan shape/chase dan atur timing.
6. Simpan show. Pengaturan jaringan portabel dipisahkan dari pilihan NIC komputer.
7. Pilih NIC, target, dan Port-Address; aktifkan output secara eksplisit.
8. Gunakan blackout untuk memadamkan intensitas; Stop Output mengirim tampilan blackout sebelum menutup pengiriman terjadwal.

## 6. Kebijakan output dan blackout

- Output dan preview menggunakan `frameId` serta byte DMX akhir yang sama.
- Grand master mengatur intensitas sesuai profil, bukan mengalikan pan, tilt, color wheel, atau semua byte.
- Blackout memakai kebijakan fixture: dimmer/RGB intensity dipadamkan, shutter memakai nilai tertutup yang diketahui, pan/tilt ditahan.
- Menolkan semua channel bukan definisi universal blackout dan dapat menggerakkan fixture. Fixture yang kebijakan gelapnya belum diketahui ditandai sebelum dipakai pada output fisik.
- Blackout tetap aktif ketika page, mode, atau scene berubah. Pelepasannya membutuhkan tindakan eksplisit.
- Mengirim UDP tidak membuktikan lampu menerima paket. UI menampilkan `Mengirim`, target, laju terukur, dan error terakhir.
- Aplikasi yang crash atau komputer yang mati tidak dapat menjamin lampu padam. Perilaku kehilangan sinyal node diuji dan didokumentasikan terpisah.

## 7. Target teknis awal, belum hasil pengukuran

| Parameter | Target proposal | Cara menguji |
|---|---|---|
| Engine logis | 50 tick/detik, berdasarkan monotonic clock | Fake clock dan replay timeline deterministik |
| ArtDmx | Default 30 frame/detik, batas konfigurasi 44 untuk node DMX512 | Receiver loopback menghitung timestamp dan byte |
| Perubahan kontrol ke receiver | p95 <= 75 ms pada benchmark Windows yang dicatat | Timestamp command, frame, dan receiver; termasuk beban preview |
| Blackout ke receiver | p99 <= 80 ms pada benchmark yang sama | Tekan blackout saat drag, load, render, dan antrean kontrol padat |
| Preview | Target awal p95 frame time <= 33,3 ms pada 1080p dengan 12 beam | Benchmark pada mesin sasaran dengan nama GPU dan konfigurasi dicatat |
| Showfile | Maksimum awal 10 MiB; ditolak sebelum parsing besar | Kasus batas ukuran, kedalaman JSON dan jumlah entitas |
| Stabilitas | Smoke 10 menit, soak 1 jam, gate rilis 8 jam | Tren memori, error output, backlog, frame time, suspend/resume |

Windows desktop bukan sistem hard real-time. Angka target harus direvisi jika spike membuktikan biaya atau batas hardware berbeda. Tidak ada klaim hasil benchmark pada tahap ini.

## 8. Penyimpanan, privasi, dan batas layanan

Show dan preferensi disimpan lokal dalam JSON berversi. Tidak ada akun, autentikasi server, database server, telemetry pengguna, atau cloud show sync pada versi awal. Telemetri teknis di dalam aplikasi berbeda dari pengumpulan analytics ke internet.

Portal merencanakan `/privacy`, `/terms`, dan `/data-deletion`. Halaman terakhir menjelaskan file show, preferensi lokal, dan batas penghapusan log pada penyedia hosting. Tidak diperlukan endpoint penghapusan akun karena produk tidak memiliki akun.

Tombol hapus preferensi lokal kelak harus menampilkan daftar file yang benar-benar akan dihapus dan konfirmasi. Showfile milik pengguna tidak ikut dihapus otomatis.

## 9. Di luar versi awal

- Photometric prediction, simulasi lux tersertifikasi, ray tracing, gobo rinci, dan full stage CAD.
- USB-DMX langsung, RDM, sACN, multi-universe aktif, MIDI/OSC, audio-to-light, timecode, mobile controller, dan cloud collaboration.
- Menyalin firmware/aset VE Project, mengklaim emulasi firmware identik, atau membaca format backup proprietary tanpa spesimen.
- Auto-updater yang memasang ketika output sedang aktif.

Fitur tersebut dapat dipertimbangkan setelah engine, preview, dan output satu universe terbukti.

## 10. Hubungan dua proyek

Kinglight memiliki source aplikasi, show schema, fixture bawaan, pengujian, dan release artifacts. [kinglight-download](https://github.com/zzdevxx/kinglight-download) memiliki portal, panduan pemasangan, release metadata yang tervalidasi, dan halaman kebijakan. Portal tidak menjadi dependency runtime untuk memainkan show.

## 11. Keputusan yang menunggu pemilik

- Stack utama: Tauri 2 + Rust + Three.js dengan frontend pilihan pemilik, atau Python + PySide6 + GPU preview.
- Stack portal: Astro statis atau HTML/CSS/TypeScript statis.
- Prioritas kesetiaan panel: proposal mengikuti video enam page atau baseline manual lima page.
- Lisensi source dan aset asli sebelum distribusi kode.
- Fixture serta node Art-Net fisik pertama untuk pengujian penerimaan.
- Kebijakan release blackout: lanjutkan tampilan yang masih berjalan atau tahan sampai operator reassert. Proposal awal: restore hasil engine terkini setelah pelepasan eksplisit.

## 12. Persetujuan

Pembuatan dua repo publik dan riset telah diotorisasi. Dokumen ini adalah proposal, bukan persetujuan implementasi atau klaim fitur yang sudah tersedia. Tahap berikutnya dimulai setelah pemilik meninjau rencana dan memilih stack.
