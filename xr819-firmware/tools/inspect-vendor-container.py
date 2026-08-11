#!/usr/bin/env python3
"""Inspect the section stream consumed by the XR819 vendor downloader.

The format is deliberately parsed without XR819-specific destination guesses:

  u32 magic = 'XR01'
  type 0: u32 type, u32 destination, u32 length, u8 data[length]
  type 1: u32 type, u32 destination, u32 value, u32 length
  type 2: u32 type, u32 length, (u32 address, u32 value)[length / 8]
  type 4: u32 type, u32 entry, u32 trailer

All integers are little-endian. Type 1 is a repeated-byte/word fill in the
vendor loader; current XR819 containers use value zero.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import struct
from dataclasses import asdict, dataclass
from pathlib import Path

MAGIC = b"XR01"


@dataclass(frozen=True)
class Section:
    index: int
    type: int
    header_offset: int
    data_offset: int | None
    destination: int | None
    length: int | None
    value: int | None
    sha256: str | None


def read_u32(data: bytes, offset: int) -> int:
    if offset + 4 > len(data):
        raise ValueError(f"truncated u32 at file offset {offset:#x}")
    return struct.unpack_from("<I", data, offset)[0]


def checked_end(offset: int, length: int, total: int) -> int:
    end = offset + length
    if end < offset or end > total:
        raise ValueError(
            f"section payload {offset:#x}+{length:#x} exceeds file size {total:#x}"
        )
    return end


def parse_container(data: bytes) -> list[Section]:
    if data[:4] != MAGIC:
        raise ValueError(f"bad magic {data[:4]!r}, expected {MAGIC!r}")

    sections: list[Section] = []
    offset = 4
    index = 0
    while offset < len(data):
        header_offset = offset
        section_type = read_u32(data, offset)

        if section_type == 0:
            destination = read_u32(data, offset + 4)
            length = read_u32(data, offset + 8)
            data_offset = offset + 12
            end = checked_end(data_offset, length, len(data))
            payload = data[data_offset:end]
            sections.append(
                Section(
                    index,
                    section_type,
                    header_offset,
                    data_offset,
                    destination,
                    length,
                    None,
                    hashlib.sha256(payload).hexdigest(),
                )
            )
            offset = end
        elif section_type == 1:
            destination = read_u32(data, offset + 4)
            value = read_u32(data, offset + 8)
            length = read_u32(data, offset + 12)
            sections.append(
                Section(
                    index,
                    section_type,
                    header_offset,
                    None,
                    destination,
                    length,
                    value,
                    None,
                )
            )
            offset += 16
        elif section_type == 2:
            length = read_u32(data, offset + 4)
            if length % 8:
                raise ValueError(
                    f"MMIO section at {header_offset:#x} has non-pair length {length:#x}"
                )
            data_offset = offset + 8
            end = checked_end(data_offset, length, len(data))
            payload = data[data_offset:end]
            sections.append(
                Section(
                    index,
                    section_type,
                    header_offset,
                    data_offset,
                    None,
                    length,
                    None,
                    hashlib.sha256(payload).hexdigest(),
                )
            )
            offset = end
        elif section_type == 4:
            entry = read_u32(data, offset + 4)
            trailer = read_u32(data, offset + 8)
            sections.append(
                Section(
                    index,
                    section_type,
                    header_offset,
                    None,
                    entry,
                    None,
                    trailer,
                    None,
                )
            )
            offset += 12
            if offset != len(data):
                raise ValueError(
                    f"{len(data) - offset:#x} trailing bytes follow terminal type-4 section"
                )
        else:
            raise ValueError(f"unknown section type {section_type} at {offset:#x}")
        index += 1

    if not sections or sections[-1].type != 4:
        raise ValueError("container has no terminal type-4 entry section")
    return sections


def format_section(section: Section) -> str:
    if section.type == 0:
        return (
            f"{section.index:2d} copy  file={section.header_offset:#08x} "
            f"data={section.data_offset:#08x} dst={section.destination:#010x} "
            f"len={section.length:#08x} sha256={section.sha256}"
        )
    if section.type == 1:
        return (
            f"{section.index:2d} fill  file={section.header_offset:#08x} "
            f"dst={section.destination:#010x} len={section.length:#08x} "
            f"value={section.value:#010x}"
        )
    if section.type == 2:
        if section.data_offset is None or section.length is None:
            raise ValueError("MMIO section is missing its payload metadata")
        return (
            f"{section.index:2d} mmio  file={section.header_offset:#08x} "
            f"data={section.data_offset:#08x} len={section.length:#08x} "
            f"pairs={section.length // 8} sha256={section.sha256}"
        )
    return (
        f"{section.index:2d} entry file={section.header_offset:#08x} "
        f"address={section.destination:#010x} trailer={section.value:#010x}"
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("container", type=Path)
    parser.add_argument("--json", action="store_true", dest="as_json")
    args = parser.parse_args()

    data = args.container.read_bytes()
    sections = parse_container(data)
    if args.as_json:
        print(
            json.dumps(
                {
                    "path": str(args.container),
                    "size": len(data),
                    "sha256": hashlib.sha256(data).hexdigest(),
                    "sections": [asdict(section) for section in sections],
                },
                indent=2,
                sort_keys=True,
            )
        )
        return

    print(f"path={args.container}")
    print(f"size={len(data):#x}")
    print(f"sha256={hashlib.sha256(data).hexdigest()}")
    for section in sections:
        print(format_section(section))


if __name__ == "__main__":
    main()
