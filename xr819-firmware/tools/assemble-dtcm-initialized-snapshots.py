#!/usr/bin/env python3
"""Assemble paginated XR819 DTCM diagnostic MIB responses into raw snapshots."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from pathlib import Path

MAGIC = b"DTCM"
SCHEMA = 1
MIB_BASE = 0xFF00
IMAGE_SIZE = 0x2078
CHUNK_PAYLOAD_SIZE = 352
CHUNK_COUNT = 24
HEADER_SIZE = 16
STAGE_NAMES = ("entry", "platform", "startup")
ROOT = Path(__file__).resolve().parents[1]


def validate_firmware_contract() -> None:
    inventories = {
        "Cargo.toml": ("dtcm-contract-diagnostics = []",),
        "src/packet_ram.rs": ("pub const HIF_OUTPUT_SIZE: usize = 0x180;",),
        "src/dtcm.rs": (
            "pub const INITIALIZED_IMAGE_SIZE: usize = 0x2078;",
            "pub const SNAPSHOT_MIB_BASE: u16 = 0xff00;",
            "pub const SNAPSHOT_CHUNK_PAYLOAD_SIZE: usize = 352;",
        ),
        "src/bin/hif_startup.rs": (
            "SnapshotStage::Entry",
            "SnapshotStage::Platform",
            "SnapshotStage::Startup",
        ),
        "src/command.rs": (
            "write_initialized_image_snapshot_mib",
            "encode_read_mib_data_response_in_place",
        ),
    }
    failures = [
        f"{relative}: missing DTCM snapshot diagnostic inventory: {item}"
        for relative, required in inventories.items()
        for item in required
        if item not in (ROOT / relative).read_text()
    ]
    if failures:
        raise SystemExit("\n".join(failures))
    if HEADER_SIZE + CHUNK_PAYLOAD_SIZE + 12 > 0x180:
        raise SystemExit("DTCM snapshot MIB response exceeds one HIF output slot")


@dataclass(frozen=True)
class Chunk:
    stage: int
    index: int
    offset: int
    payload: bytes


def u16(data: bytes, offset: int) -> int:
    return int.from_bytes(data[offset : offset + 2], "little")


def diagnostic_data(data: bytes, source: str) -> tuple[bytes, int | None]:
    if data.startswith(MAGIC):
        return data, None
    if len(data) < 12 + HEADER_SIZE or data[12:16] != MAGIC:
        raise SystemExit(f"{source}: neither diagnostic data nor a WSM read-MIB response")
    wire_length = u16(data, 0)
    if wire_length > len(data) or wire_length < 12 + HEADER_SIZE:
        raise SystemExit(f"{source}: invalid WSM response length {wire_length}")
    if u16(data, 2) & 0x0FFF != 0x0405:
        raise SystemExit(f"{source}: response is not WSM READ_MIB.confirm")
    if int.from_bytes(data[4:8], "little") != 0:
        raise SystemExit(f"{source}: read-MIB response status is nonzero")
    mib_id = u16(data, 8)
    data_length = u16(data, 10)
    if 12 + data_length != wire_length:
        raise SystemExit(f"{source}: WSM data length does not match response length")
    return data[12:wire_length], mib_id


def parse_chunk(data: bytes, source: str = "<memory>") -> Chunk:
    data, mib_id = diagnostic_data(data, source)
    if len(data) < HEADER_SIZE or data[:4] != MAGIC:
        raise SystemExit(f"{source}: truncated DTCM diagnostic header")
    if data[4] != SCHEMA:
        raise SystemExit(f"{source}: unsupported DTCM snapshot schema {data[4]}")
    stage, index, count = data[5], data[6], data[7]
    offset, length, total = u16(data, 8), u16(data, 10), u16(data, 12)
    if stage >= len(STAGE_NAMES) or count != CHUNK_COUNT or index >= count:
        raise SystemExit(f"{source}: invalid stage/chunk identity")
    if total != IMAGE_SIZE or offset + length > total or len(data) != HEADER_SIZE + length:
        raise SystemExit(f"{source}: invalid chunk bounds or payload length")
    expected_mib = MIB_BASE + stage * CHUNK_COUNT + index
    if mib_id is not None and mib_id != expected_mib:
        raise SystemExit(
            f"{source}: MIB 0x{mib_id:04x} does not match stage {stage} chunk {index}"
        )
    return Chunk(stage, index, offset, data[HEADER_SIZE:])


def assemble(chunks: list[Chunk]) -> dict[str, bytes]:
    indexed: dict[tuple[int, int], Chunk] = {}
    for chunk in chunks:
        key = (chunk.stage, chunk.index)
        if key in indexed:
            raise SystemExit(f"duplicate {STAGE_NAMES[chunk.stage]} chunk {chunk.index}")
        indexed[key] = chunk

    snapshots: dict[str, bytes] = {}
    for stage, name in enumerate(STAGE_NAMES):
        image = bytearray(IMAGE_SIZE)
        cursor = 0
        for index in range(CHUNK_COUNT):
            try:
                chunk = indexed[(stage, index)]
            except KeyError as error:
                raise SystemExit(f"missing {name} chunk {index}") from error
            if chunk.offset != cursor:
                raise SystemExit(
                    f"{name} chunk {index}: expected offset 0x{cursor:04x}, found 0x{chunk.offset:04x}"
                )
            image[chunk.offset : chunk.offset + len(chunk.payload)] = chunk.payload
            cursor += len(chunk.payload)
        if cursor != IMAGE_SIZE:
            raise SystemExit(f"{name}: assembled size 0x{cursor:x}, expected 0x{IMAGE_SIZE:x}")
        snapshots[name] = bytes(image)
    return snapshots


def make_fixture(stage: int, index: int, full_wsm: bool) -> bytes:
    offset = index * CHUNK_PAYLOAD_SIZE
    length = min(CHUNK_PAYLOAD_SIZE, IMAGE_SIZE - offset)
    payload = bytes(((stage * 37 + offset + byte) & 0xFF) for byte in range(length))
    data = bytearray(HEADER_SIZE + length)
    data[:4] = MAGIC
    data[4:8] = bytes((SCHEMA, stage, index, CHUNK_COUNT))
    data[8:10] = offset.to_bytes(2, "little")
    data[10:12] = length.to_bytes(2, "little")
    data[12:14] = IMAGE_SIZE.to_bytes(2, "little")
    data[16:] = payload
    if not full_wsm:
        return bytes(data)
    response = bytearray(12 + len(data))
    response[0:2] = len(response).to_bytes(2, "little")
    response[2:4] = (0xA405).to_bytes(2, "little")
    response[8:10] = (MIB_BASE + stage * CHUNK_COUNT + index).to_bytes(2, "little")
    response[10:12] = len(data).to_bytes(2, "little")
    response[12:] = data
    return bytes(response)


def self_test() -> None:
    chunks = [
        parse_chunk(make_fixture(stage, index, (stage + index) % 2 == 0))
        for stage in range(3)
        for index in range(CHUNK_COUNT)
    ]
    snapshots = assemble(chunks)
    if set(snapshots) != set(STAGE_NAMES) or any(len(data) != IMAGE_SIZE for data in snapshots.values()):
        raise SystemExit("snapshot assembler rejected its complete fixture")
    try:
        assemble(chunks[:-1])
    except SystemExit:
        pass
    else:
        raise SystemExit("snapshot assembler accepted an incomplete fixture")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("chunks", type=Path, nargs="*")
    parser.add_argument("--output-dir", type=Path)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()

    validate_firmware_contract()
    self_test()
    if args.self_test:
        if args.chunks or args.output_dir is not None:
            raise SystemExit("--self-test does not accept chunk files or --output-dir")
        print("DTCM INITIALIZED-IMAGE SNAPSHOT ASSEMBLER SELF-TEST PASSED")
        return
    if not args.chunks or args.output_dir is None:
        raise SystemExit("provide 72 chunk files and --output-dir, or use --self-test")

    chunks = [parse_chunk(path.read_bytes(), str(path)) for path in args.chunks]
    snapshots = assemble(chunks)
    args.output_dir.mkdir(parents=True, exist_ok=True)
    for name, data in snapshots.items():
        path = args.output_dir / f"{name}.bin"
        path.write_bytes(data)
        print(f"wrote {path} size=0x{len(data):x}")


if __name__ == "__main__":
    main()
