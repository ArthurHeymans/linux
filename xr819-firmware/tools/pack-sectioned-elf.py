#!/usr/bin/env python3
"""Pack ARM ELF PT_LOAD segments into an XR819 copy/fill section stream.

This preserves one coherent ELF and its linker-selected virtual addresses while
avoiding the enormous holes produced by `objcopy -O binary` for ITCM, DTCM and
high-SRAM segments. The output uses the copy/fill subset of the vendor firmware
container consumed by `inspect-vendor-container.py`.
"""

from __future__ import annotations

import argparse
import hashlib
import struct
from dataclasses import dataclass
from pathlib import Path

ELF_MAGIC = b"\x7fELF"
XR819_MAGIC = b"XR01"
PT_LOAD = 1
EM_ARM = 40
TYPE_COPY = 0
TYPE_FILL = 1
TYPE_ENTRY = 4
PF_X = 1
PF_W = 2
PF_R = 4
MAX_U32 = 0xFFFF_FFFF


@dataclass(frozen=True)
class LoadSegment:
    index: int
    file_offset: int
    destination: int
    file_size: int
    memory_size: int
    flags: int
    alignment: int


def align_up(value: int, alignment: int) -> int:
    return (value + alignment - 1) & -alignment


def checked_u32(name: str, value: int) -> int:
    if not 0 <= value <= MAX_U32:
        raise ValueError(f"{name} does not fit u32: {value:#x}")
    return value


def parse_elf32_arm(data: bytes) -> tuple[int, list[LoadSegment]]:
    if len(data) < 52 or data[:4] != ELF_MAGIC:
        raise ValueError("input is not an ELF file")
    if data[4] != 1:
        raise ValueError("only ELF32 images are supported")
    if data[5] != 1:
        raise ValueError("only little-endian ELF images are supported")

    machine = struct.unpack_from("<H", data, 18)[0]
    if machine != EM_ARM:
        raise ValueError(f"ELF machine is {machine}, expected ARM ({EM_ARM})")

    entry = struct.unpack_from("<I", data, 24)[0]
    phoff = struct.unpack_from("<I", data, 28)[0]
    phentsize = struct.unpack_from("<H", data, 42)[0]
    phnum = struct.unpack_from("<H", data, 44)[0]
    if phentsize < 32:
        raise ValueError(f"ELF program-header size is too small: {phentsize}")
    if phoff + phentsize * phnum > len(data):
        raise ValueError("ELF program-header table exceeds the file")

    segments: list[LoadSegment] = []
    for index in range(phnum):
        offset = phoff + index * phentsize
        (
            segment_type,
            file_offset,
            virtual_address,
            physical_address,
            file_size,
            memory_size,
            flags,
            alignment,
        ) = struct.unpack_from("<IIIIIIII", data, offset)
        if segment_type != PT_LOAD or memory_size == 0:
            continue
        if file_size > memory_size:
            raise ValueError(f"PT_LOAD {index} has file size larger than memory size")
        if file_offset + file_size > len(data):
            raise ValueError(f"PT_LOAD {index} payload exceeds the ELF file")
        destination = physical_address if physical_address != 0 else virtual_address
        if destination & 3:
            raise ValueError(f"PT_LOAD {index} destination is not word aligned")
        if file_size & 3:
            raise ValueError(f"PT_LOAD {index} file size is not word aligned")
        segments.append(
            LoadSegment(
                index,
                file_offset,
                checked_u32("segment destination", destination),
                checked_u32("segment file size", file_size),
                checked_u32("segment memory size", memory_size),
                flags,
                alignment,
            )
        )

    if not segments:
        raise ValueError("ELF has no non-empty PT_LOAD segments")
    return entry, segments


def validate_runtime_ranges(segments: list[LoadSegment]) -> None:
    ranges: list[tuple[int, int, int]] = []
    for segment in segments:
        end = segment.destination + align_up(segment.memory_size, 4)
        checked_u32("segment end", end)
        ranges.append((segment.destination, end, segment.index))
    ranges.sort()
    for previous, current in zip(ranges, ranges[1:]):
        if current[0] < previous[1]:
            raise ValueError(
                f"PT_LOAD {previous[2]} [{previous[0]:#x}, {previous[1]:#x}) overlaps "
                f"PT_LOAD {current[2]} [{current[0]:#x}, {current[1]:#x})"
            )


def pack_elf(data: bytes) -> tuple[bytes, int, list[LoadSegment]]:
    entry, segments = parse_elf32_arm(data)
    validate_runtime_ranges(segments)

    output = bytearray(XR819_MAGIC)
    for segment in segments:
        if segment.file_size:
            output += struct.pack(
                "<III", TYPE_COPY, segment.destination, segment.file_size
            )
            output += data[
                segment.file_offset : segment.file_offset + segment.file_size
            ]
        zero_length = align_up(segment.memory_size, 4) - segment.file_size
        if zero_length:
            output += struct.pack(
                "<IIII",
                TYPE_FILL,
                segment.destination + segment.file_size,
                0,
                zero_length,
            )

    # The vendor trailer field is not a normal CRC32. Custom downloaders treat
    # it as an opaque format word and validate the complete stream themselves.
    output += struct.pack("<III", TYPE_ENTRY, entry, 0)
    return bytes(output), entry, segments


def flags_string(flags: int) -> str:
    return "".join(
        flag if flags & mask else "-"
        for mask, flag in ((PF_R, "R"), (PF_W, "W"), (PF_X, "X"))
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("elf", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()

    elf = args.elf.read_bytes()
    packed, entry, segments = pack_elf(elf)
    args.output.write_bytes(packed)

    print(f"elf={args.elf}")
    print(f"elf_sha256={hashlib.sha256(elf).hexdigest()}")
    print(f"entry={entry:#010x}")
    for segment in segments:
        print(
            f"PT_LOAD[{segment.index}] dst={segment.destination:#010x} "
            f"file={segment.file_size:#x} mem={segment.memory_size:#x} "
            f"flags={flags_string(segment.flags)} align={segment.alignment:#x}"
        )
    print(f"output={args.output}")
    print(f"packed_size={len(packed):#x}")
    print(f"packed_sha256={hashlib.sha256(packed).hexdigest()}")


if __name__ == "__main__":
    main()
