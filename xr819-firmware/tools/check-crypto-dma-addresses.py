#!/usr/bin/env python3
"""Freeze AES/CCMP DMA publication behind the packet-RAM address encoder."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
source = (ROOT / "src/crypto.rs").read_text()

errors: list[str] = []
if "0xf6ff_ffff" in source.lower():
    errors.append("src/crypto.rs still contains an open-coded DMA address mask")

calls = source.count("crate::packet_ram::packet_dma_bus_address(")
if calls != 2:
    errors.append(f"src/crypto.rs has {calls} checked packet-DMA encodings, expected 2")

for function in ("encrypt_tx_frame_hardware", "decrypt_rx_frame_hardware"):
    body = source.split(f"fn {function}(", 1)[1].split("\n}", 1)[0]
    validation = body.find("packet_dma_bus_address(")
    source_write = body.find("registers.source.set(")
    if validation < 0 or source_write < 0 or validation > source_write:
        errors.append(f"{function} must validate the CPU address before DMA publication")

if errors:
    raise SystemExit("\n".join(errors))

print("crypto DMA packet-RAM address encoding is centralized")
