# Kontrak Engine Kinglight, usulan

Status: draf untuk ditinjau pemilik. Rujukan: [ERD](../ERD.md), [BLUEPRINT](../BLUEPRINT.md), [PRD](../PRD.md). Angka laju masih target, bukan hasil ukur.

## 1. Kepemilikan dan provenance frame

| Aktor | Memiliki | Tidak memiliki |
|---|---|---|
| Engine | state otoritatif, `modelRevision` sesi, programmer, playback, shape, blackout, frame akhir | widget, socket, kamera |
| Output actor | socket, NIC, target, Port-Address, sequence, pacing, status armed | editor show, state show kedua, akses renderer |
| Preview | scene GPU, pose fixture, kualitas, frame terakhir untuk gambar | clock engine, socket, state otoritatif |
| UI | fokus, seleksi, command, representasi state | salinan buffer yang bisa jadi sumber output |

Batas wire engine hanya dilintasi `encodedFrame[512]` final beserta `frameId`. Renderer tidak pernah melintasi batas ini. Engine menyusun frame akhir pada tick logis usulan 50 Hz. Output actor memilih frame terbaru pada laju ArtDmx usulan 30 Hz, sehingga bisa melewati beberapa `frameId`. Preview boleh merender dengan lajunya sendiri. Frame yang dikirim dan frame yang ditampilkan berasal dari urutan `frameId` engine yang sama, bukan satu paket untuk tiap tick engine.

## 2. Clock

Runtime memakai monotonic clock dan deadline absolut. Setelah stall, phase dihitung dari waktu, bukan dengan memutar ulang semua tick lama ke jaringan. Fake clock hanya dipakai pengujian deterministik dan tidak pernah menjadi field showfile. Target 50 Hz engine, 30 Hz ArtDmx default, dan batas 44 Hz belum diukur pada mesin sasaran. Windows desktop bukan sistem real time.

## 3. Kontrak IPC

Bentuk command memakai `kind` konkret. `id` unik per sesi. `expectedRevision` hanya dipakai transaksi struktural atau destruktif dan dibandingkan dengan `modelRevision` runtime.

```json
[
  { "id": "c1", "kind": "blackout.set", "payload": { "active": true } },
  { "id": "c2", "kind": "patch.set", "payload": { "fixtureId": "f1", "modeId": "m1", "start": 1 }, "expectedRevision": 17 },
  { "id": "c3", "kind": "fader.set", "payload": { "fixtureId": "f1", "attributeId": "dimmer", "value16": 80, "patchRevision": 4 } }
]
```

Ack sukses:

```json
{ "id": "c2", "ok": true, "appliedRevision": 18, "frameId": 2401 }
```

Ack gagal:

```json
{ "id": "c2", "ok": false, "error": { "code": "INVALID_PATCH", "message": "Patch melewati channel 256", "field": "patch.start", "retryable": false } }
```

| Kind | Pemakaian | `expectedRevision` |
|---|---|---|
| `blackout.set`, `blackout.release`, `fader.set`, `playback.go` | kendali langsung | tidak |
| `output.arm`, `output.stop`, `output.disarm`, `preview.pause`, `ping` | kendali sesi | tidak |
| `patch.set`, `fixture.delete`, `cue.delete`, `chase.delete`, `show.load`, `show.save` | struktural atau destruktif | ya |

| Kode | Makna |
|---|---|
| `STALE_REVISION` | `expectedRevision` tidak cocok dengan `modelRevision` runtime |
| `PATCH_REVISION_STALE` | `patchRevision` fader tidak cocok dengan patch aktif |
| `TARGET_NOT_FOUND` | fixture atau atribut fader tidak ada di patch aktif |
| `INVALID_PATCH` | alamat di luar 1..256 atau rentang patch bertabrakan |
| `INVALID_ATTRIBUTE`, `ATTRIBUTE_NOT_FOUND` | mode invalid atau atribut tidak dikenal |
| `INVALID_PLAYBACK_SLOT`, `INVALID_TIMING`, `INVALID_SHAPE` | target slot, durasi, atau phase parameter invalid |
| `OUTPUT_ARMED` | konfigurasi jaringan atau load ditolak saat output armed |
| `INVALID_OUTPUT_CONFIG` | NIC target, alamat, Port-Address, atau kecepatan tidak valid saat arm |
| `INVALID_PORT_ADDRESS` | `portAddress` di luar 1..32767 atau `compatZero` tidak aktif saat nilai nol |
| `PROFILE_DIGEST_MISMATCH` | digest profil tidak cocok dengan pin di instance |
| `UNSUPPORTED_SCHEMA`, `INVALID_JSON`, `FILE_TOO_LARGE` | showfile ditolak |

Antrean bounded, usulan 64 command per kelompok.

| Kelompok | Kebijakan |
|---|---|
| Fader dan encoder | latest wins per `fixtureId`, `attributeId`, dan `patchRevision`. Target wajib ada di patch aktif |
| Blackout | jalur prioritas, latched sampai release eksplisit, diterima walaupun UI membawa revision lama |
| Konfigurasi jaringan | hanya saat output disarmed |
| Transaksi struktural | menolak `expectedRevision` lama dengan `STALE_REVISION` |
| Subscription | snapshot terbaru, bukan antrean penuh frame |

## 4. Resolusi nilai per tick engine

1. Selesaikan playback dari page, chase, dan cue. HTP dipakai di antara playback untuk intensitas. LTP dipakai untuk atribut event dan memakai `revision` serta timestamp agar nilai berubah saat kejadian, bukan tiap tick.
2. Terapkan override programmer sesuai mask atribut. Programmer bersifat absolut: nilai programmer yang lebih rendah mengalahkan nilai playback. Intensitas playback 120 dengan programmer 80 menghasilkan 80. HTP tidak dipakai untuk melawan programmer.
3. Terapkan layer shape hanya pada atribut yang tidak sedang diambil programmer dan hanya bila owner shape menang arbitrasi. Shape tidak pernah mengalahkan programmer.
4. Interpolasi pan/tilt dan atribut lain di ruang logis 16 bit, rentang 0..65535 dengan tengah 32768. Encode ke byte DMX terjadi setelah semua interpolasi selesai.
5. Roda warna dan langkah diskrit memakai nilai diskrit. Tidak ada fade numerik antar warna.
6. Master hanya memengaruhi intensitas. Master tidak mengubah pan, tilt, warna, atau shutter.
7. Blackout memakai kebijakan fixture: intensitas 0, shutter ke nilai aman, pan dan tilt dikunci pada nilai yang sedang resolvenya saat latch. Phase engine boleh terus berjalan, tetapi nilai pan dan tilt output tetap terkunci sampai release.
8. Susun `encodedFrame[512]`. Kanal 257..512 nol pada profil 256. Naikkan `frameId`, lalu publikasikan frame akhir ke output actor dan preview.

Pelepasan flash dan pelepasan LTP menghitung ulang layer yang masih hidup dari live state terkini, bukan memulihkan salinan lama. Perubahan fader saat flash aktif ikut terhitung saat pelepasan.

## 5. Siklus output

| Tahap | Perilaku |
|---|---|
| Start aplikasi | state `disarmed`, tidak ada paket |
| `output.arm` | berlaku hanya dari `disarmed`, memerlukan NIC, alamat target, laju, dan `portAddress` yang valid. Bila valid, state menjadi `armed` |
| Load show | bila armed, load ditolak dengan `OUTPUT_ARMED`. Operator wajib stop lebih dulu. Setelah load berhasil, state tetap `disarmed` |
| Port-Address | angka raw 15 bit. Status default UNDECIDED sampai node pertama diperiksa dan memblokir arm. Nilai 0 hanya diterima bila `compatZero: true`, jika tidak, rentang 1..32767 |
| `output.stop` | dari `armed`, actor masuk `stopping`, mengirim tepat dua frame output berisi blackout pada laju 30 Hz, membuang frame lama antrean, menutup koneksi prospektif, lalu kembali ke `disarmed`. Bila sudah `disarmed`, stop adalah no-op yang diakui |
| `output.disarm` | pembatalan darurat langsung dari `armed` tanpa frame blackout, dipakai hanya bila pengiriman blackout justru berbahaya. Tindakan ini memerlukan konfirmasi UI dan tercatat di log lokal |
| Sequence | 1..255 lalu kembali 1, menurut Port-Address |
| Crash atau mati listrik | tidak ada jaminan lampu padam, tidak ada phase failsafe |

Transisi ringkas: `disarmed --arm(valid)--> armed --stop--> stopping --dua UDP frame--> disarmed`. Konfigurasi jaringan hanya diubah saat `disarmed`. Arm dengan konfigurasi cacat ditolak dengan `INVALID_OUTPUT_CONFIG`; `portAddress` cacat memakai `INVALID_PORT_ADDRESS`. UNDECIDED bukan nilai siap kirim.

Stop tidak boleh dihitung dari dua tick engine 50 Hz karena kedua paket bisa terlewat pada pacing 30 Hz. Kedua datagram memakai `encodedFrame[512]` dan `frameId` blackout yang sudah di-latch engine, dengan dua nilai sequence berurutan. Actor tidak menyusun buffer blackout kedua. Setelah blackout dikirim, tidak boleh ada frame lama yang masih keluar. Payload ArtDmx 512 byte diawali header sesuai [DOCUMENTATION](../DOCUMENTATION.md).

## 6. Penyimpanan

- File show `.kinglight` berisi konfigurasi domain, entitas, timing awal, dan wadah migrasi. `modelRevision`, clock, seleksi UI, flag armed, socket, NIC host, dan target jaringan tidak ikut disimpan.
- Path load dan save berasal dari dialog file yang dipilih pengguna. Validasi menolak lokasi jaringan remote. Tidak ada pembatasan buatan bahwa path harus relatif terhadap satu direktori.
- Save atomik memakai file sementara di direktori tujuan, lalu replace. Load membangun kandidat state dan hanya mengganti state aktif setelah seluruh validasi lolos.
- Versi tidak dikenal dan digest mismatch ditolak tanpa mengubah show yang sedang dibuka.
- Preferensi lokal terpisah dan tidak portabel. Flag armed selalu mulai dalam keadaan mati pada proses baru.
- Tidak ada database server, akun, autentikasi server, atau API HTTP publik pada versi awal.

## 7. Invarian engine

| Id | Invarian |
|---|---|
| I1 | Hanya engine yang menulis state. UI dan preview hanya command dan tampilan |
| I2 | Batas wire engine hanya membawa frame akhir. Tidak ada jalur output kedua |
| I3 | Frame output dan preview berasal dari urutan `frameId` engine yang sama |
| I4 | Output actor boleh melewati `frameId` sesuai pacing 30 Hz, bukan satu paket per tick 50 Hz |
| I5 | `start + footprint - 1 <= 256`, overlap ditolak, 16 slot tidak melonggarkan budget |
| I6 | Kanal 257..512 nol pada profil 256 |
| I7 | HTP berlaku di antara playback. Programmer override absolut, termasuk nilai lebih rendah |
| I8 | Atribut event memakai LTP berdasarkan revision dan timestamp, bukan nilai tiap tick |
| I9 | Shape tunduk pada owner dan prioritas programmer |
| I10 | Master hanya intensitas |
| I11 | Blackout fixture aware dan pan/tilt terkunci pada nilai latch |
| I12 | Flash release dan LTP release memakai live state yang dihitung ulang |
| I13 | Disarmed berarti nol paket, termasuk saat load |
| I14 | Load saat armed ditolak sampai operator menjalankan stop |
| I15 | Stop mengirim dua frame blackout pada 30 Hz dari frame latch yang sama, membuang frame lama, lalu kembali ke `disarmed` |
| I16 | Interpolasi 16 bit selesai sebelum encode byte, warna diskrit tanpa fade numerik |
| I17 | `expectedRevision` hanya untuk transaksi struktural. Blackout selalu diterima |
| I18 | State runtime dan konfigurasi jaringan tidak masuk showfile |

## 8. Penerimaan uji dengan fake clock

| Uji | Skenario | Hasil |
|---|---|---|
| T1 | Load show pada proses baru | disarmed, tidak ada UDP |
| T2 | Patch `start 225`, `footprint 32` | diterima, berakhir di 256 |
| T3 | Patch `start 226`, `footprint 32` | ditolak dengan `INVALID_PATCH` |
| T4 | 12 fixture masing masing 32 channel | ditolak, `384 > 256` |
| T5 | Dua rentang patch `1..8` dan `8..10` | ditolak karena overlap |
| T6 | Playback intensitas 80 dan 120 | HTP menghasilkan 120 |
| T7 | Playback 120 dengan programmer 80 | hasil 80, bukan 120 |
| T8 | Shape aktif dan programmer menimpa atribut yang sama | programmer menang, shape tetap jalan di atribut lain |
| T9 | Blackout saat shape aktif | pan/tilt terkunci pada nilai latch, phase engine boleh maju |
| T10 | Pan 16 bit dinterpolasi lalu diencode | tengah 32768 menghasilkan pasangan byte sesuai urutan profil |
| T11 | Fader diubah saat flash, lalu flash dilepas | nilai akhir berasal dari fader terbaru |
| T12 | Antrean penuh, fader lama dan blackout masuk | fader lama dibuang, blackout tetap latched |
| T13 | Patch command dengan `expectedRevision` lama | ditolak dengan `STALE_REVISION` |
| T14 | Blackout dengan revision UI lama | tetap diterima |
| T15 | Fader dengan `patchRevision` lama atau target tidak ada | `PATCH_REVISION_STALE` atau `TARGET_NOT_FOUND` |
| T16 | Ubah alamat saat armed | ditolak dengan `OUTPUT_ARMED` |
| T17 | Load saat armed | ditolak dengan `OUTPUT_ARMED`. Setelah stop dan load, disarmed |
| T18 | Load `schemaVersion: 2` | ditolak dengan `UNSUPPORTED_SCHEMA`, hash file tidak berubah |
| T19 | Digest profil tidak cocok | ditolak dengan `PROFILE_DIGEST_MISMATCH` |
| T20 | `output.stop` pada frame output ke n | tepat dua frame blackout pada n dan n+1 dari frame latch yang sama, tidak ada frame lama setelahnya, state kembali `disarmed` |
| T22 | `output.arm` dengan `portAddress: 0` tanpa `compatZero` | ditolak dengan `INVALID_PORT_ADDRESS` |
| T23 | `output.arm` dengan NIC atau target invalid | ditolak dengan `INVALID_OUTPUT_CONFIG`, state tetap `disarmed` |
| T24 | `output.stop` saat `disarmed` | diakui sebagai no-op, tidak ada UDP |
| T21 | Engine 50 Hz dan output 30 Hz | output memilih frame terbaru, `frameId` boleh meloncat, preview memakai frame akhir yang sama |

## 9. Keputusan yang menunggu pemilik

1. Stack final: Tauri 2 dengan Rust dan Three.js, atau PySide6 dengan GPU preview.
2. Default Port-Address setelah node penguji diketahui.
3. Profil enam page video atau lima page manual sebagai baseline.
4. Fixture dan receiver pertama untuk pengujian byte dan pan/tilt.
5. Satuan posisi panggung, sumbu up, format kalibrasi, dan kebijakan migrasi showfile.
