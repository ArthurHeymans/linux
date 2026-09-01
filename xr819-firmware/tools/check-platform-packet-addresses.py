#!/usr/bin/env python3
"""Freeze platform startup packet offsets behind the packet-RAM owner."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
source = (ROOT / "src/platform.rs").read_text()

errors: list[str] = []
if "0x007f_ffff" in source.lower():
    errors.append("src/platform.rs still contains an open-coded MAC packet offset")

calls = source.count("packet_ram::mac_packet_offset_unchecked(")
if calls != 3:
    errors.append(f"platform startup has {calls} owned packet-offset encodings, expected 3")

if errors:
    raise SystemExit("\n".join(errors))

print("platform packet-RAM address encoding is centralized")
