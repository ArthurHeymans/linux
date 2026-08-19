#!/usr/bin/env python3
"""Focused regression tests for section-aware XR819 ELF packing."""

from __future__ import annotations

import importlib.util
import struct
import sys
import unittest
from dataclasses import dataclass
from pathlib import Path


def load_packer():
    path = Path(__file__).with_name("pack-sectioned-elf.py")
    spec = importlib.util.spec_from_file_location("pack_sectioned_elf", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


PACKER = load_packer()


@dataclass(frozen=True)
class FixtureSection:
    name: str
    section_type: int
    flags: int
    address: int
    size: int
    alignment: int = 4


def elf_fixture(
    *,
    destination: int,
    file_size: int,
    memory_size: int,
    sections: list[FixtureSection],
) -> bytes:
    names = bytearray(b"\0.text\0")
    name_offsets = {".text": 1}
    for section in sections:
        name_offsets[section.name] = len(names)
        names += section.name.encode() + b"\0"
    name_offsets[".shstrtab"] = len(names)
    names += b".shstrtab\0"

    payload_offset = 0x100
    payload = bytes(range(file_size))
    names_offset = payload_offset + len(payload)
    section_offset = (names_offset + len(names) + 3) & ~3
    section_count = 3 + len(sections)
    data = bytearray(section_offset + section_count * 40)
    data[:16] = b"\x7fELF\x01\x01\x01" + bytes(9)
    struct.pack_into(
        "<HHIIIIIHHHHHH",
        data,
        16,
        2,
        PACKER.EM_ARM,
        1,
        destination,
        52,
        section_offset,
        0,
        52,
        32,
        1,
        40,
        section_count,
        section_count - 1,
    )
    struct.pack_into(
        "<IIIIIIII",
        data,
        52,
        PACKER.PT_LOAD,
        payload_offset,
        destination,
        destination,
        file_size,
        memory_size,
        PACKER.PF_R | PACKER.PF_W,
        4,
    )
    data[payload_offset : payload_offset + file_size] = payload
    data[names_offset : names_offset + len(names)] = names

    headers = [
        (0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
        (
            name_offsets[".text"],
            1,
            PACKER.SHF_ALLOC,
            destination,
            payload_offset,
            file_size,
            0,
            0,
            4,
            0,
        ),
    ]
    for section in sections:
        headers.append(
            (
                name_offsets[section.name],
                section.section_type,
                section.flags,
                section.address,
                payload_offset,
                section.size,
                0,
                0,
                section.alignment,
                0,
            )
        )
    headers.append(
        (
            name_offsets[".shstrtab"],
            3,
            0,
            0,
            names_offset,
            len(names),
            0,
            0,
            1,
            0,
        )
    )
    for index, header in enumerate(headers):
        struct.pack_into("<IIIIIIIIII", data, section_offset + index * 40, *header)
    return bytes(data)


class PackerTests(unittest.TestCase):
    def test_packet_nobits_alloc_outside_load_is_not_packed(self) -> None:
        data = elf_fixture(
            destination=0x1000,
            file_size=4,
            memory_size=4,
            sections=[
                FixtureSection(
                    ".packet_ram.fixture", PACKER.SHT_NOBITS, PACKER.SHF_ALLOC, 0x09007000, 0x80
                )
            ],
        )
        packed, _entry, _segments = PACKER.pack_elf(data)
        self.assertEqual(struct.unpack_from("<III", packed, 4), (PACKER.TYPE_COPY, 0x1000, 4))
        self.assertNotIn(struct.pack("<I", 0x09007000), packed)

    def test_packet_section_must_be_nobits(self) -> None:
        data = elf_fixture(
            destination=0x1000,
            file_size=4,
            memory_size=4,
            sections=[FixtureSection(".packet_ram.fixture", 1, PACKER.SHF_ALLOC, 0x09007000, 4)],
        )
        with self.assertRaisesRegex(ValueError, "not SHT_NOBITS"):
            PACKER.pack_elf(data)

    def test_packet_section_must_be_allocated(self) -> None:
        data = elf_fixture(
            destination=0x1000,
            file_size=4,
            memory_size=4,
            sections=[FixtureSection(".packet_ram.fixture", PACKER.SHT_NOBITS, 0, 0x09007000, 4)],
        )
        with self.assertRaisesRegex(ValueError, "not SHF_ALLOC"):
            PACKER.pack_elf(data)

    def test_packet_section_cannot_intersect_load_segment(self) -> None:
        data = elf_fixture(
            destination=0x4000,
            file_size=4,
            memory_size=0x100,
            sections=[
                FixtureSection(".packet_ram.fixture", PACKER.SHT_NOBITS, PACKER.SHF_ALLOC, 0x4080, 4)
            ],
        )
        with self.assertRaisesRegex(ValueError, "intersects PT_LOAD"):
            PACKER.pack_elf(data)

    def test_zero_file_load_still_emits_dtcm_fill(self) -> None:
        data = elf_fixture(
            destination=0x04002000,
            file_size=0,
            memory_size=0x20,
            sections=[
                FixtureSection(
                    ".dtcm.context_pool", PACKER.SHT_NOBITS, PACKER.SHF_ALLOC, 0x04002000, 0x20
                )
            ],
        )
        packed, _entry, _segments = PACKER.pack_elf(data)
        self.assertEqual(
            struct.unpack_from("<IIII", packed, 4),
            (PACKER.TYPE_FILL, 0x04002000, 0, 0x20),
        )

    def test_load_destination_in_09_window_is_rejected(self) -> None:
        data = elf_fixture(destination=0x09010000, file_size=4, memory_size=4, sections=[])
        with self.assertRaisesRegex(ValueError, "forbidden 0x09"):
            PACKER.pack_elf(data)


if __name__ == "__main__":
    unittest.main()
