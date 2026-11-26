# Desain simulator Kinglight

Arahan pemilik 2026-11-26: remake mengikuti video acuan, bukan memperluas tampilan menjadi dashboard teknis.

## Acuan dan koreksi

Frame 00:03 memperlihatkan konsol biru dengan master dan 16 fader dalam satu baris, selector page di atas, serta display dan encoder di kanan. Frame 00:06 sampai 00:51 memperlihatkan display compact di kiri panggung. Menu shape berpindah di dalam display yang sama. Panggung hitam dan beam putih mendominasi tampilan.

| Tampilan awal | Remake | Tujuan |
|---|---|---|
| Stage, konsol, display, monitor DMX dan show tampil bersamaan | Panggung dengan konsol overlay; detail teknis dibuka saat dibutuhkan | Mengikuti susunan yang terlihat dalam video |
| Fader dan encoder terpisah menjadi blok vertikal | Master dan 16 strip sejajar; encoder berada tepat di bawah display | Menjaga hubungan fisik antarkontrol |
| Form shape panjang dengan tombol Apply terpisah | Menu bertingkat dan empat bidang parameter pada display | Memakai urutan operasi yang terlihat pada video |
| Dua baris fixture di atas lantai grid | Satu baris fixture menghadap penonton pada latar hitam | Menampilkan kipas dan persilangan beam seperti acuan |

## Susunan

Mode penuh memakai panel biru di kiri bawah. Bagian fader memuat Fixture/Playback, channel bank, page 1 sampai 6, grup A sampai D, tombol unit, blackout, nilai channel, track dan tombol bawah strip. Bagian kanan memuat display, Exit/Record/Enter/Clear dan dua encoder. Mode compact menyembunyikan bagian fader, mempertahankan display, action keys dan encoder.

Menu mengikuti label pada video: Shape Menu, Playback Shape, Tilt/Pan Shape dan Shape Parameter. Ukuran, BPM, repeat, spread, interleave dan direction dibaca dari state engine. Angka panel ditampilkan dalam satuan operator; konversi ke nilai internal tidak mengubah buffer dari frontend.

Toolbar ringkas memberi akses pergantian mode panel, show, Art-Net dan monitor. Tidak ada frame counter, penjelasan IPC atau formulir jaringan permanen dalam alur latihan.

## Sistem visual

- Biru konsol dan tombol biru muda mengikuti acuan eksplisit pemilik; warna ini menggantikan proposal charcoal/amber awal.
- Latar panggung hitam mengikuti video; putih terang dipakai untuk beam yang berasal dari state fixture.
- Panel encoder terang, knob hitam dengan marker posisi, angka tabular dan label PAN/TILT.
- Kontrol persegi dengan radius kecil dan border jelas. State fokus, hover, tekan, seleksi dan disabled tersedia.
- Display padat mengikuti perangkat konsol. Batas measure prosa umum tidak diterapkan pada label instrumen pendek.
- Motion hanya untuk feedback kontrol dan isi show. Jeda preview tidak menghentikan engine maupun output.

## Batas kesetiaan

Posisi, teks fungsi, warna kelompok dan alur tampilan dibandingkan dengan frame video. Source, logo dan aset VE Project tidak disalin. Formula gerakan, kalibrasi beam dan model show adalah implementasi Kinglight; video tidak menyediakan kode atau spesifikasi numeriknya. Tidak ada klaim kesetaraan firmware atau pencahayaan fisik.

## Verifikasi

Uji mode penuh dan compact pada 960x600, 1366x768 dan 1920x1080. Operasikan menu shape, encoder, fader, scene, show, blackout dan output loopback. Bukti visual serta hasil tes dicatat di PROGRESS.md setelah dijalankan.
