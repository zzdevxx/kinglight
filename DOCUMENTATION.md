# Dokumentasi teknis dan batas tahap ini

Status: dokumentasi perencanaan. Tidak ada perintah build/test aplikasi yang telah tersedia.

## Kontrak yang perlu dibaca

| Kontrak | Pemilik | Dokumen |
|---|---|---|
| Kebutuhan produk dan kapasitas | Kinglight | [PRD](PRD.md) |
| Arsitektur dan pilihan toolchain | Kinglight | [BLUEPRINT](BLUEPRINT.md) |
| Entitas dan validasi show | Kinglight | [ERD](ERD.md) |
| Command, merge, frame, blackout | Kinglight | [ENGINE-CONTRACT](docs/ENGINE-CONTRACT.md) |
| Metadata artifact dan portal | Producer Kinglight, consumer kinglight-download | [Release contract](https://github.com/zzdevxx/kinglight-download/blob/main/docs/RELEASE-CONTRACT.md) |
| Bukti dan batas interpretasi | Riset | [VIDEO-ANALYSIS](docs/VIDEO-ANALYSIS.md), [SOURCES](docs/SOURCES.md) |

## Terminologi

- DMX slot/channel: byte alamat 1..512 pada wire; profil konsol awal mengendalikan 1..256.
- Fixture slot: pemilihan unit di panel; jumlahnya berbeda dari footprint DMX fixture.
- Attribute: nilai logis seperti dimmer atau pan, dapat memakai satu atau dua slot.
- Channel page: memilih bank atribut 1..16 atau 17..32, bukan bukti 32 fixture.
- Playback page: bank scene/chase tersimpan; video menampilkan enam selector, manual mendokumentasikan lima.
- Programmer: layer edit sementara dengan mask atribut eksplisit.
- HTP/LTP: aturan pemilihan atribut, bukan cara dua controller jaringan mengirim paket.
- Port-Address: identitas raw universe Art-Net 15-bit pada paket, terpisah dari nomor universe tampilan QLC+.
- Blackout: transformasi intensitas/shutter sesuai profile, berbeda dari Clear programmer atau Stop Output.

## ArtDmx wire contract

| Offset byte | Field | Nilai/aturan rencana |
|---|---|---|
| 0..7 | ID | ASCII `Art-Net` diikuti byte nol |
| 8..9 | OpCode | `0x5000`, little endian |
| 10..11 | ProtVer | 14, big endian |
| 12 | Sequence | 1..255 lalu 1, terpisah per Port-Address |
| 13 | Physical | 0 untuk sumber controller ini |
| 14 | SubUni | Low 8 bit Port-Address |
| 15 | Net | High 7 bit Port-Address |
| 16..17 | Length | 512, big endian |
| 18..529 | Data | 512 byte final; 257..512 nol dalam profil 256 |

Target port UDP 6454. Jangan menambah DMX start code ke payload ArtDmx. Paket dikirim native, bukan melalui halaman web biasa. Dukungan ArtDmx bukan klaim semua fitur Art-Net 4 telah dipenuhi.

## Penyimpanan dan output

Showfile berupa JSON dengan `schemaVersion`; semua entitas divalidasi menjadi kandidat sebelum mengganti state aktif. Runtime armed flag, handle socket, kredensial, dan NIC host tidak dibawa di showfile. Byte output adalah hasil perhitungan, bukan satu-satunya sumber scene.

Atomic save memakai temporary file dalam direktori tujuan, flush dan replace sesuai platform, dengan penanganan gagal yang terlihat. Migrasi schema menulis file baru/backup sebelum mengubah show lama. Semua keputusan implementasi detail diverifikasi pada filesystem Windows yang diuji.

## Rilis dan sinkronisasi

Versi aplikasi dimiliki satu file `VERSION` yang baru dibuat saat scaffold disetujui. Tag rilis harus cocok. Kontrak sisi producer yang direncanakan adalah `docs/contracts/release.json`, dengan `schemaVersion: 1` dan field yang sama seperti snapshot [portal](https://github.com/zzdevxx/kinglight-download/blob/main/docs/RELEASE-CONTRACT.md): `status`, `version`, `tag`, `sourceCommit`, `publishedAt`, `channel`, `releaseUrl`, `notesUrl`, dan `artifacts` berisi `platform`, `arch`, `fileName`, `url`, `sizeBytes`, `sha256`. File itu belum ada dan hanya menjadi kontrak saat implementasi rilis disetujui.

Producer menghitung ukuran dan SHA-256 dari artifact nyata, lalu menghasilkan snapshot metadata untuk portal. Hash tidak membuktikan identitas penerbit; code signing/updater signing adalah kontrak tambahan jika kelak dipakai.

Tidak ada installer, video referensi, database runtime, dependency cache, secret atau packet capture yang masuk Git. Tiga remote harus sama-sama terkonfigurasi sebelum menjalankan sync. Verifikasi memakai HEAD lokal dan `git ls-remote` pada `refs/heads/main`; exit code script saja tidak cukup.

## Pengujian dan klaim

Matriks pengujian ada pada [ROADMAP](ROADMAP.md). Hasil pengujian harus menyebut commit, versi toolchain, OS, GPU/receiver, perintah dan batasnya. Fake clock membuktikan semantik engine; loopback membuktikan paket lokal; pengujian node fisik diperlukan untuk klaim interoperabilitas hardware.

Dokumen ini tidak menyatakan build, output jaringan, rendering, atau performa telah lulus.
