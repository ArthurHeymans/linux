#!/usr/bin/env python3
"""Verify the exact linker-owned XR819 packet-RAM section layout."""

from __future__ import annotations

import argparse
import importlib.util
import sys
from pathlib import Path

EXPECTED = {
    ".packet_ram.runtime": (0x09007000, 0xF630),
    ".packet_ram.rx_fifo_backing": (0x09400000, 0x8000),
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


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("elf", type=Path)
    args = parser.parse_args()

    data = args.elf.read_bytes()
    packer = load_packer()
    _entry, segments = packer.parse_elf32_arm(data)
    packer.validate_runtime_ranges(segments)
    sections = packer.validate_packet_sections(data, segments)
    actual = {section.name: (section.address, section.size) for section in sections}

    missing = EXPECTED.keys() - actual.keys()
    extra = actual.keys() - EXPECTED.keys()
    if missing or extra:
        raise SystemExit(
            f"packet section set mismatch: missing={sorted(missing)} extra={sorted(extra)}"
        )
    for name, expected in EXPECTED.items():
        if actual[name] != expected:
            raise SystemExit(
                f"{name} layout mismatch: expected={expected!r} actual={actual[name]!r}"
            )
        print(f"{name} address={expected[0]:#010x} size={expected[1]:#x} NOBITS ALLOC no-PT_LOAD")

    print("PACKET RAM LAYOUT PASSED")


if __name__ == "__main__":
    main()
