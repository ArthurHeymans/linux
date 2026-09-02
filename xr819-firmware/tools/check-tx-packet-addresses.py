#!/usr/bin/env python3
"""Freeze TX command address encodings behind the packet-RAM owner."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
source = (ROOT / "src/tx.rs").read_text()
production = source.split("#[cfg(test)]\nmod tests {", 1)[0]

errors: list[str] = []
for mask in ("0x007f_ffff", "0xf6ff_ffff"):
    if mask in production.lower():
        errors.append(f"production TX still contains open-coded packet-RAM mask {mask}")

expected = {
    "packet_ram::mac_packet_offset_unchecked(": 1,
    "packet_ram::encode_mac_packet_offset_u32(": 3,
    "packet_ram::encode_tx_payload_bus_address(": 0,
    "packet_ram::RuntimePacketAddress::new(": 8,
    "validated_tx_frame(": 4,
    "validated_context_tx_frame(": 5,
    "FrameNodeAddress::from_raw(": 4,
    "LiveTxSlot::from_pipe_slot(": 4,
    "RetirableTxSlot::from_pipe_slot(": 1,
    "LiveTxSlot::from_mmio(": 3,
    "release_aggregate_retry_command_mask(": 2,
    "release_retiring_aggregate_command_mask(": 2,
    "retained_hardware_ring(": 6,
    "TxHardwareRingAddress::for_pipe(": 12,
    "TxHardwareRingAddress::new(": 0,
    "PublishedSlotIdentity::new(": 2,
    ".live_slot()": 2,
    ".mac_offset()": 2,
    ".tx_payload_bus_address(": 2,
}
for call, count in expected.items():
    actual = production.count(call)
    if actual != count:
        errors.append(f"production TX has {actual} {call} calls, expected {count}")

if errors:
    raise SystemExit("\n".join(errors))

print("TX packet-RAM address encoding is centralized")
