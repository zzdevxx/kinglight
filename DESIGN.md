# Arah desain Kinglight

Status: proposal berdasarkan video pemilik, belum desain final yang disetujui.

Design Read: konsol desktop untuk operator lighting, dengan panel instrumen yang padat dan stage preview gelap; ENERGY 1 / RHYTHM 1 / MOTION 1 untuk chrome aplikasi.

Gerakan beam adalah isi show, bukan animasi dekoratif. Motion dial tidak membatasi playback fixture; `Pause Preview` harus tersedia dan tidak menghentikan output.

## Identitas dan komposisi

- Pusat perhatian adalah stage preview serta nilai channel yang sedang diedit. Panel fader tetap dikenali sebagai bank konsol 16 strip.
- Gunakan panel console, viewport, dan display konteks yang dapat di-dock/diperbesar. Tampilan penuh dan compact mengikuti operasi video, bukan dashboard statistik.
- Motif identitas adalah strip bernomor, marker posisi fader, dan pembacaan 8/16-bit yang sejajar. Nomor fixture, channel, page dan playback memiliki label yang membedakannya.
- Teks `Kinglight` menjadi identitas sementara. Logo, ikon aplikasi, favicon dan aset promosi menunggu arahan pemilik.
- Jangan menyalin logo, skin atau aset `VE Project`. Label fungsi konsol dipakai untuk menjelaskan operasi, dengan copy Kinglight sendiri.

## Keputusan visual proposal

| Elemen | Arah | Alasan |
|---|---|---|
| Stage | Latar gelap netral | Bentuk dan intensitas relatif beam mudah terlihat dan tidak menambah silau saat latihan |
| Panel | Charcoal dengan panel navy redup | Membedakan kontrol dari stage sambil mempertahankan hubungan dengan referensi konsol biru |
| Aksen utama | Amber terkontrol | Menandai parameter/seleksi aktif tanpa menyamarkannya sebagai warna output lampu |
| Status merah | Blackout/error yang benar-benar aktif | Operator mengenali keadaan yang memengaruhi output |
| Tipografi | Sans sistem Windows, angka tabular; monospace hanya pada nilai DMX | Label ringkas tetap terbaca; angka tidak menggeser posisi saat berubah |
| Geometri | Border 1 px, radius kecil pada kontrol | Bank fader padat tetap tegas; rotary encoder bulat karena fungsinya |
| Spasi | Rapat di dalam strip, lebih lega di antara kelompok operasi | Mengurangi jarak gerak pointer sambil mempertahankan kelompok fungsi |
| Motion chrome | Feedback tekan/fokus singkat | Membantu mengetahui kontrol yang diterima tanpa mengganggu pembacaan beam |

Warna final dan kontras belum diuji karena UI belum dibuat. Jangan menandai rancangan ini sebagai lulus WCAG sebelum render, kalkulasi kontras dan pengoperasian diuji.

## Permukaan utama

1. Stage: camera orbit/preset, posisi fixture, Pause Preview, kualitas, dan label preview indikatif.
2. Console: Fixture/Playback, channel page, grup A-D, unit 1-16, bank fader, master.
3. Display: menu, parameter, record/include, shape category, timing dan error konteks.
4. Encoders: A/B, Normal/Fine/Ultra, nilai logis dan coarse/fine bila diperlukan.
5. Output bar: disarmed/armed/blackout/error, NIC, target, raw Port-Address, laju kirim terukur dan last error.
6. Monitor/patch/show manager: detail teknis tersedia sesuai pekerjaan, bukan selalu memenuhi layar.

## Interaksi yang harus nyata

- Semua tombol dan fader memiliki default, hover, focus-visible, active dan disabled state. Seleksi tidak disampaikan lewat warna saja.
- Fader mendukung drag, keyboard step, nilai numerik yang bisa diketik, serta reset yang terdefinisi. Encoder tidak mengandalkan gerakan memutar mouse sebagai satu-satunya cara.
- Blackout dan Stop Output selalu terjangkau; shortcut dipilih agar tidak berbenturan dengan edit teks atau dialog.
- Mode fader berubah bersama label dan indikator. Page change tidak boleh membuat nilai slider software meloncat ke channel yang salah.
- Empty show, fixture belum dipatch, profile tidak dikenal, output invalid, load dan save gagal memiliki pesan serta tindakan pemulihan.
- Bentuk feedback mengikuti hasil command engine. UI boleh menampilkan drag preview sementara, tetapi tidak boleh mengklaim output telah berubah sebelum acknowledgement.
- Preview yang dijeda menampilkan label jelas ketika output masih berjalan.

## Ukuran dan aksesibilitas

Target desktop awal diuji pada 1366x768 dan 1920x1080, termasuk scaling Windows 125%, 150%, dan 200%. Kontrol penting tidak boleh hilang karena dock/resizing. Touch target 44 px disediakan pada mode sentuh bila mode tersebut benar-benar dibangun.

Keyboard harus dapat menyelesaikan select, edit, record, clear, playback dan blackout. Focus order mengikuti kelompok konsol. Reduced motion menonaktifkan animasi chrome dan menyediakan preview yang dapat dijeda. Kontras teks normal target minimum 4,5:1, kontrol/fokus minimum 3:1, diverifikasi terhadap warna hasil render.

## Filter anti-slop

Tidak ada dekorasi glow, gradient neon, kartu statistik, testimonial, AI feature, atau tombol yang hanya tampak aktif. Warna beam datang dari show fixture, bukan aksen brand. Halaman dan kontrol ditambahkan hanya untuk pekerjaan operator yang didefinisikan dalam PRD.

Delivery gate desain dijalankan setelah UI nyata dibangun: fungsi semua kontrol, lima state, keyboard, kontras, render di ukuran sasaran dan bukti click-through. Tahap dokumen hanya dapat menilai alasan desain serta kejujuran klaim.
