#!/usr/bin/env python3
"""Freeze MAC packet-RAM encodings behind the packet-RAM owner."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
mac = (ROOT / "src/mac.rs").read_text()
packet_ram = (ROOT / "src/packet_ram.rs").read_text()

errors: list[str] = []
for mask in ("0x007f_ffff", "0xf6ff_ffff"):
    if mask in mac.lower():
        errors.append(f"src/mac.rs still contains open-coded packet-RAM mask {mask}")

if mac.count("packet_ram::mac_packet_offset_unchecked(") != 1:
    errors.append("MAC 23-bit offsets must use one local wrapper around the packet-RAM owner")
if mac.count("packet_ram::response_command_bus_address(") != 1:
    errors.append("MAC response-pointer publication must use the exact-command validator")

requirements = (
    "pub const unsafe fn mac_packet_offset_unchecked(",
    "pub fn response_command_index(",
    "pub fn response_command_bus_address(",
)
for definition in requirements:
    if packet_ram.count(definition) != 1:
        errors.append(f"packet-RAM owner has an unexpected definition count for {definition}")

if errors:
    raise SystemExit("\n".join(errors))

print("MAC packet-RAM address encoding is centralized")
