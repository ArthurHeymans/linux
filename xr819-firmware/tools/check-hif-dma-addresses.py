#!/usr/bin/env python3
"""Freeze HIF DMA publication behind the packet-RAM address encoder."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
hif = (ROOT / "src/hif.rs").read_text()
packet_ram = (ROOT / "src/packet_ram.rs").read_text()

errors: list[str] = []

if "0xf6ff_ffff" in hif.lower():
    errors.append("src/hif.rs still contains an open-coded HIF DMA address mask")

checked_calls = hif.count("packet_ram::packet_dma_bus_address(")
known_references = hif.count("known_dma_bus_address(")
unchecked_calls = hif.count("packet_ram::packet_dma_bus_address_unchecked(")
if checked_calls != 2:
    errors.append(f"src/hif.rs has {checked_calls} checked DMA encodings, expected 2")
if known_references != 3 or unchecked_calls != 1:
    errors.append("fixed HIF roots must use one local unchecked wrapper at two call sites")

checked_definition = "pub fn packet_dma_bus_address("
unchecked_definition = "pub const unsafe fn packet_dma_bus_address_unchecked("
if packet_ram.count(checked_definition) != 1:
    errors.append("packet_ram::packet_dma_bus_address must have exactly one definition")
if packet_ram.count(unchecked_definition) != 1:
    errors.append("packet_ram::packet_dma_bus_address_unchecked must have exactly one definition")
else:
    encoder = packet_ram.split(unchecked_definition, 1)[1].split("\n}", 1)[0]
    if encoder.lower().count("0xf6ff_ffff") != 1:
        errors.append("the HIF DMA mask must occur exactly once in its packet-RAM owner")

if errors:
    raise SystemExit("\n".join(errors))

print("HIF DMA packet-RAM address encoding is centralized")
