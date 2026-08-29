#!/usr/bin/env python3
"""Verify the link-placed XR819 DTCM startup-contract sections and packing."""

from __future__ import annotations

import argparse
import importlib.util
import struct
import sys
from pathlib import Path

DTCM_STATE = (0x04000000, 0x0400A000)
DTCM_STACKS = (0x0400A000, 0x0400C000)
DTCM_ALIAS_RANGE = (0x04000000, 0x04010000)
PACKET_WINDOW = (0x09000000, 0x0A000000)
EXPECTED_SECTIONS = [
    (".dtcm.data", DTCM_STATE[0], 0x2078),
    (".dtcm.bss", DTCM_STATE[0] + 0x2078, 0x7BCC),
    (".dtcm.noinit", DTCM_STATE[0] + 0x9C44, 0x03BC),
]
EXPECTED_SYMBOLS = {
    "DTCM_INITIALIZED_DATA": DTCM_STATE[0],
    "DTCM_BSS_PREFIX": DTCM_STATE[0] + 0x2078,
    "DTCM_HOST_TX_CONTEXTS": DTCM_STATE[0] + 0x2078 + 0x39AC,
    "DTCM_BSS_MIDDLE_PREFIX": DTCM_STATE[0] + 0x2078 + 0x39AC + 30 * 0x170,
    "DTCM_HOST_CONTEXT_ACCOUNTING": DTCM_STATE[0] + 0x2078 + 0x39AC + 30 * 0x170 + 0x254,
    "DTCM_HOST_CONTEXT_FREE_LIST": DTCM_STATE[0] + 0x2078 + 0x39AC + 30 * 0x170 + 0x254 + 0x18,
    "DTCM_LINK_AND_SEQUENCE": DTCM_STATE[0] + 0x2078 + 0x39AC + 30 * 0x170 + 0x254 + 0x18 + 0x8,
    "DTCM_JOIN_SCAN_CONTROL": DTCM_STATE[0] + 0x2078 + 0x39AC + 30 * 0x170 + 0x254 + 0x18 + 0x8 + 0x220,
    "DTCM_WSM_RESPONSE_SCRATCH": DTCM_STATE[0] + 0x2078 + 0x39AC + 30 * 0x170 + 0x254 + 0x18 + 0x8 + 0x220 + 0x40,
    "DTCM_BA_LMC_HEADER": DTCM_STATE[0] + 0x2078 + 0x39AC + 30 * 0x170 + 0x254 + 0x18 + 0x8 + 0x220 + 0x40 + 0xA0,
    "DTCM_PENDING_BA_LMC": DTCM_STATE[0] + 0x2078 + 0x39AC + 30 * 0x170 + 0x254 + 0x18 + 0x8 + 0x220 + 0x40 + 0xA0 + 0x20,
    "DTCM_LMC_MESSAGES": DTCM_STATE[0] + 0x2078 + 0x39AC + 30 * 0x170 + 0x254 + 0x18 + 0x8 + 0x220 + 0x40 + 0xA0 + 0x20 + 0xE0,
    "DTCM_BA_SESSIONS": DTCM_STATE[0] + 0x2078 + 0x39AC + 30 * 0x170 + 0x254 + 0x18 + 0x8 + 0x220 + 0x40 + 0xA0 + 0x20 + 0xE0 + 0x2C0,
    "DTCM_BA_LINK_EVENT_STATE": DTCM_STATE[0] + 0x2078 + 0x39AC + 30 * 0x170 + 0x254 + 0x18 + 0x8 + 0x220 + 0x4A0 + 0xA0,
    "DTCM_TALA": DTCM_STATE[0] + 0x2078 + 0x39AC + 30 * 0x170 + 0x254 + 0x18 + 0x8 + 0x220 + 0x4A0 + 0xA0 + 0x30,
    "DTCM_CONTEXT_COMPLETION_PREFIX": DTCM_STATE[0] + 0x2078 + 0x39AC + 30 * 0x170 + 0x254 + 0x18 + 0x8 + 0x220 + 0x4A0 + 0xA0 + 0x30 + 0x24,
    "DTCM_BSS_MIDDLE_SUFFIX_TAIL": DTCM_STATE[0] + 0x2078 + 0x39AC + 30 * 0x170 + 0x254 + 0x18 + 0x8 + 0x220 + 0x4A0 + 0xA0 + 0x30 + 0x24 + 0x14,
    "DTCM_INTERNAL_CONTEXT_POOL": DTCM_STATE[0] + 0x9080,
    "DTCM_BSS_SUFFIX": DTCM_STATE[0] + 0x94D4,
    "DTCM_NOINIT": DTCM_STATE[0] + 0x9C44,
    "__dtcm_data_start": DTCM_STATE[0],
    "__dtcm_data_end": DTCM_STATE[0] + 0x2078,
    "__dtcm_bss_start": DTCM_STATE[0] + 0x2078,
    "__dtcm_bss_end": DTCM_STATE[0] + 0x9C44,
    "__dtcm_noinit_start": DTCM_STATE[0] + 0x9C44,
    "__dtcm_noinit_end": DTCM_STATE[1],
    "__dtcm_state_start": DTCM_STATE[0],
    "__dtcm_state_end": DTCM_STATE[1],
    "__dtcm_state_object_start": DTCM_STATE[0],
    "__dtcm_state_object_end": DTCM_STATE[1],
    "__dtcm_context_pool_start": 0x04009080,
    "__dtcm_context_pool_contexts": 0x04009084,
    "__dtcm_context_pool_end": 0x040094D4,
}


def load_packer():
    path = Path(__file__).with_name("pack-sectioned-elf.py")
    spec = importlib.util.spec_from_file_location("pack_sectioned_elf", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def intersects(start: int, end: int, region: tuple[int, int]) -> bool:
    return start < region[1] and region[0] < end


def parse_symbol_values(data: bytes) -> dict[str, int]:
    section_offset = struct.unpack_from("<I", data, 32)[0]
    entry_size = struct.unpack_from("<H", data, 46)[0]
    section_count = struct.unpack_from("<H", data, 48)[0]

    def header(index: int) -> tuple[int, ...]:
        return struct.unpack_from("<IIIIIIIIII", data, section_offset + index * entry_size)

    values: dict[str, int] = {}
    for index in range(section_count):
        fields = header(index)
        if fields[1] != 2:  # SHT_SYMTAB
            continue
        table_offset, table_size, strings_index, symbol_size = fields[4], fields[5], fields[6], fields[9]
        if symbol_size < 16 or strings_index >= section_count:
            raise SystemExit("ELF has an invalid symbol table")
        strings_header = header(strings_index)
        strings = data[strings_header[4] : strings_header[4] + strings_header[5]]
        for offset in range(table_offset, table_offset + table_size, symbol_size):
            name_offset, value = struct.unpack_from("<II", data, offset)
            if name_offset >= len(strings):
                raise SystemExit("ELF symbol has an invalid name offset")
            name_end = strings.find(b"\0", name_offset)
            if name_end < 0:
                raise SystemExit("ELF symbol has an unterminated name")
            name = strings[name_offset:name_end].decode("ascii", errors="strict")
            if name:
                values[name] = value
    return values


def check_elf(path: Path) -> None:
    data = path.read_bytes()
    packer = load_packer()
    _entry, segments = packer.parse_elf32_arm(data)
    sections = packer.parse_sections(data)
    dtcm_sections = [section for section in sections if section.name.startswith(".dtcm.")]
    actual_sections = [(section.name, section.address, section.size) for section in dtcm_sections]
    if actual_sections != EXPECTED_SECTIONS:
        raise SystemExit(
            "DTCM section set mismatch: "
            f"expected={EXPECTED_SECTIONS!r} actual={actual_sections!r}"
        )

    approved = set(EXPECTED_SECTIONS)
    for candidate in sections:
        end = candidate.address + candidate.size
        if (
            candidate.size
            and candidate.flags & packer.SHF_ALLOC
            and intersects(candidate.address, end, DTCM_ALIAS_RANGE)
            and (candidate.name, candidate.address, candidate.size) not in approved
        ):
            raise SystemExit(
                f"allocatable section {candidate.name!r} "
                f"[{candidate.address:#x}, {end:#x}) intersects DTCM or its alias range"
            )

    for section, (expected_name, expected_address, expected_size) in zip(
        dtcm_sections, EXPECTED_SECTIONS, strict=True
    ):
        if section.section_type != packer.SHT_NOBITS:
            raise SystemExit(f"{expected_name} is not SHT_NOBITS")
        if section.flags & packer.SHF_ALLOC == 0:
            raise SystemExit(f"{expected_name} is not SHF_ALLOC")
        if (section.address, section.size) != (expected_address, expected_size):
            raise SystemExit(
                f"{expected_name} layout mismatch: expected={(expected_address, expected_size)!r} "
                f"actual={(section.address, section.size)!r}"
            )
    if dtcm_sections[-1].address + dtcm_sections[-1].size != DTCM_STACKS[0]:
        raise SystemExit("DTCM sections do not end exactly at the stack floor")
    if intersects(dtcm_sections[0].address, DTCM_STACKS[0], PACKET_WINDOW):
        raise SystemExit("DTCM state overlaps packet RAM/MMIO")

    for segment in segments:
        end = segment.destination + packer.align_up(segment.memory_size, 4)
        if intersects(segment.destination, end, DTCM_ALIAS_RANGE):
            raise SystemExit(f"PT_LOAD {segment.index} intersects DTCM or its alias range")

    symbols = parse_symbol_values(data)
    actual_symbols = {name: symbols.get(name) for name in EXPECTED_SYMBOLS}
    if actual_symbols != EXPECTED_SYMBOLS:
        raise SystemExit(
            f"DTCM linker symbols changed: expected={EXPECTED_SYMBOLS!r} actual={actual_symbols!r}"
        )

    for section in dtcm_sections:
        print(
            f"{section.name} address={section.address:#010x} size={section.size:#x} "
            "NOBITS ALLOC no-PT_LOAD"
        )
    print(
        "internal contexts "
        f"head={symbols['__dtcm_context_pool_start']:#010x} "
        f"records={symbols['__dtcm_context_pool_contexts']:#010x}"
        f"..{symbols['__dtcm_context_pool_end']:#010x}"
    )
    print(f"stacks excluded={DTCM_STACKS[0]:#010x}..{DTCM_STACKS[1]:#010x}")


def check_packed(path: Path) -> None:
    data = path.read_bytes()
    if data[:4] != b"XR01":
        raise SystemExit(f"{path} is not an XR819 section stream")

    offset = 4
    while offset + 12 <= len(data):
        record_type, destination, third = struct.unpack_from("<III", data, offset)
        if record_type == 0:  # COPY
            length = third
            record_size = 12 + length
        elif record_type == 1:  # FILL
            if offset + 16 > len(data):
                raise SystemExit("truncated FILL record")
            length = struct.unpack_from("<I", data, offset + 12)[0]
            record_size = 16
        elif record_type == 4:  # ENTRY
            record_size = 12
            if offset + record_size != len(data):
                raise SystemExit("packed ENTRY is not terminal")
            offset += record_size
            break
        else:
            raise SystemExit(f"unknown packed record type {record_type} at {offset:#x}")

        if offset + record_size > len(data):
            raise SystemExit(f"truncated packed record at {offset:#x}")
        end = destination + length
        if intersects(destination, end, DTCM_ALIAS_RANGE):
            raise SystemExit(
                f"packed record [{destination:#x}, {end:#x}) intersects DTCM or its alias range"
            )
        if intersects(destination, end, PACKET_WINDOW):
            raise SystemExit(
                f"packed record [{destination:#x}, {end:#x}) intersects packet RAM/MMIO"
            )
        offset += record_size
    else:
        raise SystemExit("packed stream has no complete ENTRY record")

    print(f"packed image={path} has no DTCM, stack, or packet-RAM destination")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("elf", type=Path)
    parser.add_argument("packed", nargs="?", type=Path)
    args = parser.parse_args()

    check_elf(args.elf)
    if args.packed is not None:
        check_packed(args.packed)
    print("DTCM LAYOUT PASSED")


if __name__ == "__main__":
    main()
