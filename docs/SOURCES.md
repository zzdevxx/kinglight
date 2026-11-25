# Sumber dan provenance riset

Diakses/diperiksa pada 2026-11-26. Isi situs dan branch upstream dapat berubah; commit lokal yang digunakan dicatat di bawah. Fakta manual bukan bukti firmware varian lain identik.

## Sumber utama

| ID | Sumber | Pemakaian dan batas |
|---|---|---|
| V1 | Video yang diberikan pemilik | Metadata, label UI dan perubahan beam; lihat [analisis](VIDEO-ANALYSIS.md). Source dan lisensi aplikasinya tidak diketahui |
| M1 | [Manual KK-256, PDF](https://www.roccer.net/wp-content/uploads/2023/06/KK-256.pdf) | 23 halaman; p.4 kapasitas, p.7 panel, p.14-18 programmer/scene, p.19-22 chase. Bukan manual revisi A yang terverifikasi |
| M2 | [Halaman produk KK-256A](https://www.roccer.net/product/kk-256a-dmx512-performance-lighting-controller/) | Pembanding varian; jangan menghapus konflik angka dengan manual |
| M3 | [Mingjing product page](http://www.mingjlight.com/en/Products/Show_203.html) | Pembanding vendor; HTTP dan isi historis bukan pengganti spesimen hardware |
| A1 | [Art-Net 4 specification](https://art-net.org.uk/downloads/art-net.pdf) | PDF yang dibaca: Protocol Release V1.4, Document Revision 1.4dp, 23/10/2025, 109 halaman |
| A2 | [Indeks spesifikasi Art-Net](https://art-net.org.uk/art-net-specification/) | Cek ulang revisi sebelum membuat klaim kesesuaian protokol |

## Catatan Art-Net yang memengaruhi desain

Pada A1, nomor halaman berikut adalah posisi halaman PDF, bukan nomor halaman cetak:

- P.20: Port-Address 15-bit; rentang 1..32767; nol deprecated untuk kompatibilitas sACN. Nol tetap perlu dipertimbangkan untuk receiver lama.
- P.73-75: ArtDmx, ID, opcode little endian, protocol 14, sequence 1..255 atau nol untuk menonaktifkan sequencing, length genap 2..512.
- P.74: unicast ke subscriber, polling untuk menemukan perubahan subscription, tidak mengizinkan broadcast ArtDmx pada revisi ini.
- P.75: batas refresh gateway DMX512 44 Hz; gateway non-DMX dapat mengumumkan batas berbeda.
- P.73: retransmit tidak berubah direkomendasikan 800..1000 ms. Pengiriman tetap 30 Hz pada proposal Kinglight lebih sering dari keepalive itu.

MVP sender unicast manual belum otomatis memenuhi seluruh perilaku controller Art-Net 4. Discovery, identitas controller, subscriber selection dan pengujian lengkap harus diperiksa sebelum klaim tersebut.

## Referensi source lokal, seluruhnya read-only

| Repo | Snapshot | Bagian yang diperiksa |
|---|---|---|
| QLC+ | `003fdfdf821fdb65eb19b252d7e54d9f0354bcd3`, branch master, bersih saat inspeksi | `plugins/artnet/src/artnetpacketizer.cpp`, `artnetcontroller.cpp`, `engine/src/scene.cpp`, `chaser.cpp`, dokumentasi concept/Art-Net/fixture monitor |
| ZZLUXORA | `55d58fb9b938528327a6fdc3444cd4dc63b9041e`, branch main; ada perubahan aktif agen lain | `core/artnet_sender.py`, `core/project_io.py`, `core/models.py`, `ui/main_window.py`, preview dan dokumentasi arsitektur |
| REA | Branch main, bersih saat pemeriksaan awal | README dan aturan investigasi; tidak menjalankan setup, build atau dekompilasi |

Bagian ZZLUXORA yang berubah tidak diperlakukan sebagai snapshot release stabil. Referensi dibaca untuk ownership buffer, atomisitas save, keterbatasan timing UI, dan pola interoperabilitas. Tidak ada source/aset yang disalin ke Kinglight.

## Referensi GitHub

| Proyek | Nilai sebagai referensi | Keputusan pemakaian |
|---|---|---|
| [QLC+](https://github.com/mcallegari/qlcplus) | Fixture model, scene/chaser, HTP/LTP, Art-Net, Virtual Console | Referensi semantik/wire; Virtual Console adalah editor generik, bukan emulasi Kingkong |
| [BlinderKitten](https://github.com/norbertrostaing/BlinderKitten) | Programmer, cue list, groups, effects/carousels, Art-Net | Bandingkan workflow; root license terdeteksi GPL-3.0, jangan menyalin ke proyek dengan lisensi lain tanpa kajian |
| [Open Fixture Library](https://github.com/OpenLightingProject/open-fixture-library) | Coarse/fine, channel capabilities, wheel dan fixture mode | Konversi ke schema internal yang dipin; jangan menggunakan format upstream sebagai show schema stabil |
| [OFL format](https://github.com/OpenLightingProject/open-fixture-library/blob/master/docs/fixture-format.md) | Dokumentasi memperingatkan format dapat berubah tidak kompatibel | Importer terpisah bila diperlukan; root LICENSE MIT, provenance aset/gobo tetap diperiksa per berkas |
| [Open Lighting Architecture](https://github.com/OpenLightingProject/ola) | Receiver/tooling uji dan protokol lighting | Kandidat pengujian, bukan dependency wajib desktop Windows |
| [PyArtNet](https://github.com/spacemanspiff2007/PyArtNet) | Contoh output dan test harness Python | Fade library bukan pengganti semantik scene Kingkong; API harus sesuai versi yang dipasang |
| [Blender Lighting Visualizer](https://github.com/potkolainen/Lighting-visualizer-blender-addon) | Visualizer penerima lighting, batas akurasi | Pembanding ruang lingkup; bukan target untuk diintegrasikan seluruhnya ke versi awal |
| [Tapetum](https://github.com/julius-b-mueller/tapetum) | Contoh visualizer lighting dan jalur Art-Net | Petunjuk riset, tidak diadopsi sebagai fondasi tanpa audit |

QLC+ lokal memiliki root `COPYING` Apache-2.0; header dan dependency harus diperiksa per berkas sebelum reuse. Lisensi repo Kinglight sendiri belum dipilih. Referensi berarti dipelajari, bukan otomatis izin menyalin logo, firmware, fixture corpus, atau binary.

## Dokumentasi framework yang diperiksa

Riset memakai Context7 untuk dokumentasi terkini, lalu membedakan mekanisme yang didokumentasikan dari performa yang belum diuji:

| Dokumentasi | Fakta relevan |
|---|---|
| [Qt QTimer](https://doc.qt.io/qtforpython-6/PySide6/QtCore/QTimer.html) | Timer membutuhkan event loop dan start/stop pada thread pemilik; busy thread/OS dapat menunda firing |
| [Qt thread/QObject](https://doc.qt.io/qtforpython-6/overviews/qtdoc-threads-qobject.html) | QTimer/QUdpSocket dibuat dan digunakan pada satu thread dengan ownership yang jelas |
| [Tauri 2](https://v2.tauri.app/) | State Rust dan IPC/capabilities dapat memisahkan UI web dari resource native; tidak membuktikan latency Kinglight |
| [Three.js](https://threejs.org/docs/) | Render loop, materials dan GPU scene menjadi kandidat preview; beam tetap pendekatan visual |
| [ModernGL](https://moderngl.readthedocs.io/) | Context/framebuffer offscreen tersedia; keberhasilan standalone belum membuktikan embedding Qt |
| [Electron utility process](https://www.electronjs.org/docs/latest/api/utility-process) | Kandidat pemisahan proses output jika stack Electron dipilih |

## Kebijakan riset berikutnya

Sebelum menyatakan perilaku hardware: cocokkan model/firmware, fixture mode, sample show atau packet capture dan hasil receiver. Sebelum menyatakan performa: jalankan benchmark dengan commit/toolchain/mesin/konfigurasi yang dicatat. Sebelum reuse source/aset: catat license dan provenance pada artifact yang sebenarnya.
