"""Verifikasi paket ArtDmx dari Kinglight pada loopback.

Jalankan: python scripts/verify-artnet.py --seconds 5 --port-address 0
Lalu aktifkan output di aplikasi dengan target 127.0.0.1 dan alamat sesuai argumen.
"""

from __future__ import annotations

import argparse
import json
import socket
import struct
import sys
import time
from collections import Counter


def parse_packet(data: bytes) -> dict:
    if len(data) < 18:
        raise ValueError(f"paket terlalu pendek: {len(data)}")
    if data[:7] != b"Art-Net" or data[7] != 0:
        raise ValueError("ID Art-Net tidak valid")
    opcode = struct.unpack_from("<H", data, 8)[0]
    if opcode != 0x5000:
        raise ValueError(f"opcode bukan ArtDmx: {opcode:#x}")
    prot_ver = struct.unpack_from(">H", data, 10)[0]
    sequence = data[12]
    port_address = data[14] | ((data[15] & 0x7F) << 8)
    length = struct.unpack_from(">H", data, 16)[0]
    if length < 2 or length > 512 or length % 2 != 0:
        raise ValueError(f"length tidak valid: {length}")
    if len(data) != 18 + length:
        raise ValueError(f"ukuran paket {len(data)} tidak cocok dengan length {length}")
    return {
        "bytes": len(data),
        "opcode": opcode,
        "protVer": prot_ver,
        "sequence": sequence,
        "portAddress": port_address,
        "length": length,
        "payloadNonZero": sum(1 for value in data[18:] if value != 0),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--seconds", type=int, default=5)
    parser.add_argument("--port-address", type=int, default=0)
    parser.add_argument("--port", type=int, default=6454)
    args = parser.parse_args()

    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    sock.settimeout(1.0)
    sock.bind(("127.0.0.1", args.port))

    packets: list[dict] = []
    errors: list[str] = []
    deadline = time.monotonic() + args.seconds
    while time.monotonic() < deadline:
        try:
            data, _ = sock.recvfrom(2048)
        except socket.timeout:
            continue
        try:
            packets.append(parse_packet(data))
        except ValueError as exc:
            errors.append(str(exc))

    sock.close()
    if not packets:
        print(json.dumps({"ok": False, "reason": "tidak ada paket diterima", "errors": errors}))
        return 1

    sequences = [packet["sequence"] for packet in packets]
    wrong_length = [p for p in packets if p["length"] != 512 or p["bytes"] != 530]
    wrong_address = [p for p in packets if p["portAddress"] != args.port_address]
    wrong_version = [p for p in packets if p["protVer"] != 14]
    distinct = Counter(sequences)

    report = {
        "ok": not errors and not wrong_length and not wrong_address and not wrong_version and len(distinct) > 1,
        "received": len(packets),
        "seconds": args.seconds,
        "expectedPortAddress": args.port_address,
        "observedPortAddresses": sorted({p["portAddress"] for p in packets}),
        "sequenceMin": min(sequences),
        "sequenceMax": max(sequences),
        "distinctSequences": len(distinct),
        "wrongLength": len(wrong_length),
        "wrongPortAddress": len(wrong_address),
        "wrongProtocolVersion": len(wrong_version),
        "errors": errors[:10],
        "firstPayloadNonZero": packets[0]["payloadNonZero"],
    }
    print(json.dumps(report, indent=2))
    return 0 if report["ok"] else 1


if __name__ == "__main__":
    sys.exit(main())
