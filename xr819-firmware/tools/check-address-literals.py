#!/usr/bin/env python3
"""Enforce ownership of XR819 packet-RAM and packet-controller literals."""

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LITERAL = re.compile(r"0x[0-9a-fA-F_]+")
KNOWN_DATA = {
    ("src/loader.rs", 0x090A0F0F),
    ("src/packet_ram.rs", 0x7000),
    ("src/packet_ram.rs", 0x09007000),
    ("src/phy.rs", 0x09001F01),
    ("src/platform.rs", 0x09090000),
}
# Hardware and retained software records often store only the low 23 bits of a
# packet-RAM pointer. These literals are just as layout-coupled as full 0x090
# addresses and must be derived from packet_ram objects too.
def object_starts(base: int, count: int, stride: int) -> set[int]:
    return {base + index * stride for index in range(count)}


PACKET_POINTER_OFFSETS = set().union(
    object_starts(0x3678, 30, 0x54),
    object_starts(0x7000, 32, 4),
    object_starts(0x7080, 16, 0x54),
    object_starts(0x75C0, 80, 0x10),
    object_starts(0x7BC0, 2, 2),
    object_starts(0x7BC4, 13, 0x54),
    object_starts(0x8008, 4, 1),
    object_starts(0x8A68, 30, 0x660),
    object_starts(0x149A8, 4, 0x180),
    object_starts(0x14FA8, 3, 0x400),
    object_starts(0x15FA8, 4, 0x2A0),
    {0x15BA8, 0x16A28, 0x16AB4, 0x17500},
)
TEST_LITERAL_FILES = {
    "src/download.rs",
    "src/host_tx_policy.rs",
    "src/radio.rs",
    "src/tx.rs",
    "src/vendor_host_tx.rs",
}


def code_only(source: str) -> str:
    output: list[str] = []
    index = 0
    state = "code"
    block_depth = 0
    while index < len(source):
        if state == "code":
            if source.startswith("//", index):
                output.extend("  ")
                index += 2
                state = "line-comment"
            elif source.startswith("/*", index):
                output.extend("  ")
                index += 2
                block_depth = 1
                state = "block-comment"
            elif source[index] == '"':
                output.append(" ")
                index += 1
                state = "string"
            else:
                output.append(source[index])
                index += 1
        elif state == "line-comment":
            if source[index] == "\n":
                output.append("\n")
                state = "code"
            else:
                output.append(" ")
            index += 1
        elif state == "block-comment":
            if source.startswith("/*", index):
                output.extend("  ")
                index += 2
                block_depth += 1
            elif source.startswith("*/", index):
                output.extend("  ")
                index += 2
                block_depth -= 1
                if block_depth == 0:
                    state = "code"
            else:
                output.append("\n" if source[index] == "\n" else " ")
                index += 1
        else:
            if source[index] == "\\" and index + 1 < len(source):
                output.extend("  ")
                index += 2
            elif source[index] == '"':
                output.append(" ")
                index += 1
                state = "code"
            else:
                output.append("\n" if source[index] == "\n" else " ")
                index += 1
    return "".join(output)


def is_test_literal(path: str, offset: int, code: str) -> bool:
    if path not in TEST_LITERAL_FILES:
        return False
    marker = code.find("#[cfg(test)]")
    return marker >= 0 and offset > marker


def main() -> None:
    failures: list[str] = []
    for source_path in sorted((ROOT / "src").rglob("*.rs")):
        relative = source_path.relative_to(ROOT).as_posix()
        code = code_only(source_path.read_text())
        for match in LITERAL.finditer(code):
            value = int(match.group().replace("_", ""), 16)
            packet_address = 0x09000000 <= value < 0x09500000
            packet_pointer_offset = value in PACKET_POINTER_OFFSETS
            controller_address = 0x09C00000 <= value < 0x09D00000
            if not packet_address and not packet_pointer_offset and not controller_address:
                continue
            if (relative, value) in KNOWN_DATA or is_test_literal(relative, match.start(), code):
                continue
            if controller_address and relative == "src/platform.rs":
                continue
            line = code.count("\n", 0, match.start()) + 1
            family = (
                "packet RAM"
                if packet_address
                else "packet RAM pointer offset"
                if packet_pointer_offset
                else "packet-controller MMIO"
            )
            failures.append(
                f"{relative}:{line}: {family} literal {match.group()} is outside its owner"
            )
    if failures:
        raise SystemExit("\n".join(failures))
    print("ADDRESS LITERAL OWNERSHIP PASSED")


if __name__ == "__main__":
    main()
