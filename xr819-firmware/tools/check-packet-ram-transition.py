#!/usr/bin/env python3
"""Normalized binary/source gates for the one-shot packet-RAM transition."""

from __future__ import annotations

import argparse
import collections
import hashlib
import importlib.util
import re
import struct
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE_HASHES = {
    ("src/platform.rs", "pub fn prepare_dma_and_clocks"): "99c46ed3a7d601f298a4eb2b9cca45f22f3e577c2f0c5fe020f77c2b04131718",
    ("src/hif.rs", "fn reclaim_tx"): "25ae09c1399e327a9a3f56a20548aae1e5a9f0ca57ac3afce49dd5e92ca366f1",
    ("src/hif.rs", "pub fn publish_radio"): "f1d255b6f46d11f0e3f210b75357ca748793a3a772a97fef1a825ebd89f07925",
    ("src/radio.rs", "fn normalize_offset"): "08c048deabe08325e83e9b0ffa8e09521202e2c620c030daf3043ed71b5c99b1",
    ("src/radio.rs", "fn next_offset"): "f1aa3324d5bdd00ab4b8c89b0e86f9a59f2d5ea4a2cc509c409609e36b584e46",
    ("src/radio.rs", "fn slot_data_fits_packet_ram"): "3e923814d2e2c54f377985727dc8a02f34b677800b46ec04976772c6e5a3b829",
}
MAC_POINTERS = (0x09007000, 0x090075C0, 0x09007BC0, 0x09008008, 0x09015BCC)


def run(*arguments: str) -> str:
    return subprocess.run(arguments, check=True, text=True, stdout=subprocess.PIPE).stdout


def load_packer():
    path = Path(__file__).with_name("pack-sectioned-elf.py")
    spec = importlib.util.spec_from_file_location("pack_sectioned_elf", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def text_bytes(path: Path, packer) -> bytes:
    data = path.read_bytes()
    section = next(section for section in packer.parse_sections(data) if section.name == ".text")
    return data[section.file_offset : section.file_offset + section.size]


def aligned_literal_sequence(data: bytes, start: int, end: int, excluded: set[int] = set()) -> list[int]:
    words = struct.iter_unpack("<I", data[: len(data) & ~3])
    return [
        word
        for (word,) in words
        if word % 4 == 0 and start <= word < end and word not in excluded
    ]


def extract_function(source: str, needle: str) -> str:
    start = source.index(needle)
    brace = source.index("{", start)
    depth = 0
    for index in range(brace, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[start : index + 1]
    raise RuntimeError(f"unterminated function {needle}")


def normalized_source(source: str) -> str:
    source = re.sub(r"//.*", "", source)
    source = re.sub(r"/\*.*?\*/", "", source, flags=re.DOTALL)
    return " ".join(source.split())


def check_preserved_source() -> None:
    for (relative, needle), expected in SOURCE_HASHES.items():
        body = extract_function((ROOT / relative).read_text(), needle)
        actual = hashlib.sha256(normalized_source(body).encode()).hexdigest()
        if actual != expected:
            raise RuntimeError(f"qualified b6 body changed: {relative} {needle}")


def symbol_name(elf: Path, suffix: str) -> str:
    matches = [line.split()[-1] for line in run("llvm-nm", str(elf)).splitlines() if suffix in line]
    if len(matches) != 1:
        raise RuntimeError(f"expected one symbol containing {suffix!r}, found {matches!r}")
    return matches[0]


def check_hif_initializer(elf: Path) -> None:
    symbol = symbol_name(elf, "Transport10initialize")
    disassembly = run(
        "llvm-objdump",
        "-d",
        "--no-show-raw-insn",
        f"--disassemble-symbols={symbol}",
        str(elf),
    )
    required = ("cmp\tr3, #0x1e", "movs\tr0, #0x33", "lsls\tr0, r0, #0x5")
    missing = [instruction for instruction in required if instruction not in disassembly]
    if missing:
        raise RuntimeError(f"HIF initializer lost 30-buffer/0x660-stride shape: {missing}")


def check_no_relocations(elf: Path) -> None:
    relocations = run("llvm-readelf", "-r", str(elf))
    if "There are no relocations" not in relocations:
        raise RuntimeError("candidate contains relocations")
    undefined = run("llvm-nm", "-u", str(elf)).strip()
    if undefined:
        raise RuntimeError(f"candidate has undefined symbols:\n{undefined}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("baseline", type=Path)
    parser.add_argument("candidate", type=Path)
    args = parser.parse_args()

    packer = load_packer()
    baseline = text_bytes(args.baseline, packer)
    candidate = text_bytes(args.candidate, packer)

    # The non-packet MMIO literal ordering is identical to b6. Two 0x09c values
    # are normalized out because central address helpers fold the second TX-ring
    # base into arithmetic and share one extra packet-DMA control literal.
    for start, end in ((0x0A800000, 0x0AD00000),):
        if aligned_literal_sequence(baseline, start, end) != aligned_literal_sequence(candidate, start, end):
            raise RuntimeError("non-packet MMIO literal ordering differs from b6")
    excluded = {0x09C00600, 0x09C60100}
    if aligned_literal_sequence(baseline, 0x09C00000, 0x09D00000, excluded) != aligned_literal_sequence(
        candidate, 0x09C00000, 0x09D00000, excluded
    ):
        raise RuntimeError("normalized packet-controller MMIO ordering differs from b6")
    baseline_counts = collections.Counter(
        aligned_literal_sequence(baseline, 0x09C00000, 0x09D00000)
    )
    candidate_counts = collections.Counter(
        aligned_literal_sequence(candidate, 0x09C00000, 0x09D00000)
    )
    if candidate_counts[0x09C00600] != baseline_counts[0x09C00600] + 1:
        raise RuntimeError("unexpected packet-DMA control literal normalization")
    if candidate_counts[0x09C60100] + 1 != baseline_counts[0x09C60100]:
        raise RuntimeError("unexpected TX-ring base literal normalization")

    for pointer in MAC_POINTERS:
        if struct.pack("<I", pointer) not in candidate:
            raise RuntimeError(f"linker-derived MAC pointer {pointer:#x} is absent from .text")

    check_preserved_source()
    check_hif_initializer(args.candidate)
    check_no_relocations(args.candidate)

    combined_source = "\n".join(path.read_text() for path in (ROOT / "src").rglob("*.rs"))
    for forbidden in ("crosses_fifo_wrap", "hif_wrap_copy"):
        if forbidden in combined_source:
            raise RuntimeError(f"forbidden wrap-copy implementation present: {forbidden}")

    print("NORMALIZED PACKET-RAM TRANSITION GATES PASSED")


if __name__ == "__main__":
    try:
        main()
    except (OSError, RuntimeError, subprocess.CalledProcessError, StopIteration) as error:
        raise SystemExit(f"transition gate failed: {error}")
