# Analisis video referensi

Tanggal analisis: 2026-11-26. Jenis bukti: metadata berkas, contact sheet, frame penuh dan crop kontrol. Audio tidak digunakan untuk menyimpulkan fitur.

## Provenance

| Properti | Hasil pembacaan lokal |
|---|---|
| Nama berkas | `TikDownloader.io_7708187518000254215_hd.mp4` |
| Ukuran | 5.814.218 byte |
| SHA-256 | `c927139f0365d1060de56afe9f2db7e3e132e16cf35e5ea67c380e28c812bfdf` |
| Video | HEVC, 1920x1080, 30 fps |
| Durasi terbaca | 54,33 detik |
| Audio stream | AAC terdeteksi; isi audio tidak dianalisis |
| Metode | 19 frame berjarak 3 detik untuk contact sheet; 11 frame penuh pada titik 0, 3, 6, 9, 12, 21, 24, 39, 45, 48, 51 detik |

Frame/crop dan video tetap lokal, tidak dimasukkan ke repo publik. Ekstraksi menggunakan FFmpeg melalui paket imageio-ffmpeg di direktori riset sementara, tanpa mengubah video. Rentang waktu pada tabel berikut adalah titik pemeriksaan, bukan batas transisi yang diukur per-frame.

Sumber 30 fps tidak ditingkatkan menjadi 60 fps. Tidak ada klaim ekstraksi spring, latensi kontrol, kualitas frame rate aplikasi asli, atau hubungan paket jaringan dari cuplikan ini.

## Temuan bertimestamp

| Titik pemeriksaan | Yang terlihat | Implikasi untuk Kinglight |
|---|---|---|
| 00:00 | Kipas beam putih pada stage gelap; panel compact di sisi kiri | Hasil gerakan beam menjadi pekerjaan utama simulator |
| 00:03 | Panel besar; master dan 16 fader, 16 unit buttons; selector Playback Page 1..6; label `VE Project` | Panel perlu full/compact atau dock yang mendukung melihat stage sambil mengoperasikan konsol |
| 00:03 | Tombol Fixture; label 1-16 dan 17-32; grup A-D; display `NO FIXTURE SELECTED`; unit 1..12 tampak berwarna berbeda dari 13..16 | Makna highlight tidak boleh disamakan dengan seleksi fixture hanya dari warnanya; label bank kemungkinan channel page |
| 00:03 | Move, Copy/Include, Delete/Off, Time, Shape, Setup; Exit, Record, Enter, Clear | Alur programmer, edit, record dan release harus menjadi state machine nyata |
| 00:03 | Dua encoder A/B; Normal/Fine/Ultra; angka pan/tilt 32768 | Nilai logis 16-bit dan increment encoder perlu ditetapkan secara eksplisit |
| 00:06 | Shape Menu berisi Playback Shape, Shape Parameter, Edit Shape, Playback Parameter, Delete/Off Shape, Include Scene untuk Edit | Shape memiliki lifecycle, konteks dan kepemilikan pada scene, bukan animasi stage yang berdiri sendiri |
| 00:09 | Menu Tilt/Pan Shape: Circle, Circle 2 Arah, Figure 8, Spiral, Square, Diamond, Pan Swing, Tilt Swing | Katalog awal dapat ditentukan dari label yang terlihat; formula persis belum diketahui |
| 00:12 | Size 25/255, Speed 6 BPM, Rep 180/360, Spr 63/63 EVEN; interleave None/Even/Odd; empat direction controls | Engine perlu ordered fixture set dan parameter phase, bukan random motion |
| 00:12, 00:15, 00:18 | Beam membentuk crossing/fan pada beberapa posisi | Preview harus mengikuti pan/tilt dinamis secara berkelompok |
| 00:21 | Parameter dengan tombol hijau Play Shape; beam kembali berupa fan | Start/stop shape berbeda dari blackout dan Stop Output |
| 00:24, 00:27, 00:30, 00:33 | Orientasi dan spread beam berubah; sebagian beam tampak menyamping atau hampir mendatar | Renderer perlu transform dan camera yang benar, tidak cukup mengubah panjang garis 2D |
| 00:39 | Tombol merah Stop Shape; interleave Even; arah berubah pada panel | Status play shape harus mencerminkan engine, bukan state tombol lokal saja |
| 00:45 | Playback Shape memilih kategori Tilt/Pan, Dimmer, RGB, Color | Tahapan setelah pan/tilt dapat menambah kategori lain; cuplikan tidak membuktikan seluruh kategori berfungsi |
| 00:48 | Dimmer dipilih; Size 255/255, Speed 60 BPM, Rep 0/360, Spr None | Parameter dan range bergantung pada kategori shape |
| 00:51, 00:54 | Parameter masih tampil; beam terlihat dengan pola intensitas/spread berbeda | Review visual kualitatif dapat dibandingkan kelak, bukan benchmark photometric |

## Yang belum dapat dibuktikan

- Apakah stage di-render dalam aplikasi yang sama, aplikasi terpisah, atau komposit video.
- Bahasa/framework aplikasi, engine GPU, jumlah instance, format show, dan implementasi internal.
- Bahwa enam selector page berarti 96 slot sudah benar-benar dapat merekam dan memainkan scene.
- Bahwa 16 tombol unit berarti seluruh 16 fixture pada profil tertentu dapat dipakai bersama dengan footprint 32 channel.
- Formula Circle dua arah, arti numerik repeat/spread, batas shape simultan, dan mapping Normal/Fine/Ultra.
- Dukungan Art-Net, UDP address, discovery, paket, node fisik, USB-DMX, atau sACN dari video saja.
- Identitas pemilik VE Project, repositori resminya, atau izin menggunakan source/asetnya.

## Perbandingan manual dan keputusan rancangan

Manual KK-256 yang ditemukan mendokumentasikan 12 fixture dan lima page playback. Video memperlihatkan 16 tombol unit dan enam selector page. Proposal Kinglight mengikuti pengalaman video dengan profil software sendiri; kompatibilitas manual dapat menjadi profil terpisah setelah diperlukan.

Kapasitas awal usulan adalah 256 channel aktif dalam buffer wire 512. Jika 16 fixture memakai 16 channel masing-masing, budget 256 terpenuhi. Jika masing-masing memakai 32, total 512 dan ditolak pada profil 256. UI channel bank 17-32 tidak menambah budget.

Preview beam termasuk versi awal karena menentukan tujuan cuplikan. Tahap awal dapat memakai beam kerucut GPU dan floor intersection; hasilnya indikatif, tanpa klaim pengukuran cahaya fisik.

Art-Net menjadi kebutuhan karena diminta pemilik. Bukti penerimaannya nanti adalah packet capture/receiver dan node fisik, bukan kesamaan tampilan dengan video.

## Daftar interaksi yang akan direkonstruksi

| Kelompok | Spesifikasi yang perlu ditutup pada implementasi |
|---|---|
| Select | Toggle, multi-select, urutan fixture, grup A-D, keadaan tanpa seleksi |
| Fader | Channel versus playback mode, channel bank, master dan numeric input |
| Encoder | Assignment A/B, delta Normal/Fine/Ultra, clamp/wrap sesuai atribut |
| Programmer | Edit mask, Clear, Record, Include, mode capture |
| Playback | Page mapping, scene/chase owner, take-over saat ganti page, release |
| Shape | Kategori, formula, size/BPM, repeat/spread, interleave, direction, start/stop |
| Stage | Koordinat fixture, kalibrasi pan/tilt, camera, beam, Pause Preview |
| Network | Arm, Stop Output, blackout, NIC, target, raw Port-Address, error |

Tidak ada executable atau firmware target dalam bahan yang diberikan. Analisis biner dengan REA belum memberi bukti tambahan pada tahap ini. Bila kelak ada artifact yang sah untuk diperiksa, provenance, lisensi, dan pertanyaan analisis harus ditentukan dahulu.

## Tinjauan ulang untuk remake

Pemilik menolak susunan dashboard rilis awal dan meminta susunan pada video sebagai acuan langsung. Frame awal memperlihatkan mode compact, bukan konsol penuh. Panggung memenuhi frame, sedangkan display biru berada di kiri. Pada 00:03, bagian fader diperluas ke kiri display; pada 00:06, bagian tersebut kembali tersembunyi.

Perkiraan geometri dari frame 1920x1080, untuk perbandingan visual, bukan hasil pengukuran perangkat asli:

| Bagian | Frame | Batas piksel perkiraan |
|---|---|---|
| Panel compact | 00:00 | x 0 sampai 356, y 245 sampai 895 |
| Display compact | 00:00 | x 18 sampai 331, y 258 sampai 706 |
| Action keys | 00:00 | y 715 sampai 770 |
| Encoder | 00:00 | y 782 sampai 874 |
| Panel penuh | 00:03 | x 0 sampai 1532, y 245 sampai bawah frame |
| Display pada panel penuh | 00:03 | x 1173 sampai 1510, y 258 sampai 706 |

Implikasi remake: satu display dengan menu bertingkat, bukan form shape di samping panel konsol yang terpisah. Mode compact mempertahankan action keys dan kedua encoder. Preview awal memperlihatkan 12 beam putih tanpa lantai grid atau badan lampu besar. Grup unit 13 sampai 16 tetap ada pada konsol penuh.

Warna biru konsol, tombol biru muda, action keys abu terang dan encoder hitam adalah karakter acuan yang dipertahankan. Teks merek dan aset video tidak disalin. Pengaturan jaringan dan manajemen file berada di luar permukaan latihan utama.
