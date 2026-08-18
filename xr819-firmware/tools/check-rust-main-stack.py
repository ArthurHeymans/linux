#!/usr/bin/env python3
"""Reject a firmware image whose rust_main frame reaches mode stacks."""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

SYSTEM_STACK_BYTES = 0x0400_C000 - 0x0400_B500


def rust_main_frame(elf: Path) -> int:
    disassembly = subprocess.run(
        ["llvm-objdump", "-d", str(elf)],
        check=True,
        stdout=subprocess.PIPE,
        text=True,
    ).stdout.splitlines()

    try:
        start = next(index for index, line in enumerate(disassembly) if "<rust_main>:" in line)
    except StopIteration as error:
        raise RuntimeError("rust_main symbol not found") from error

    load = re.compile(
        r"^\s*[0-9a-f]+:\s+.*\bldr\s+(r\d+),\s*\[pc,.*@\s*0x([0-9a-f]+)"
    )
    add_sp = re.compile(r"^\s*[0-9a-f]+:\s+.*\badd\s+sp,\s*(r\d+)\b")
    literal_address: int | None = None
    register: str | None = None

    for line in disassembly[start + 1 : start + 12]:
        if literal_address is None:
            if match := load.search(line):
                register = match.group(1)
                literal_address = int(match.group(2), 16)
        elif match := add_sp.search(line):
            if match.group(1) == register:
                break
    else:
        raise RuntimeError("rust_main stack-allocation sequence not found")

    literal = re.compile(
        rf"^\s*{literal_address:x}:\s+.*\.word\s+0x([0-9a-f]+)\s*$"
    )
    try:
        encoded = int(
            next(match.group(1) for line in disassembly if (match := literal.match(line))),
            16,
        )
    except StopIteration as error:
        raise RuntimeError("rust_main stack-allocation literal not found") from error

    signed = encoded - (1 << 32) if encoded & 0x8000_0000 else encoded
    if signed >= 0:
        raise RuntimeError(f"unexpected non-negative stack adjustment {signed}")
    return -signed


def main() -> int:
    if len(sys.argv) != 2:
        print(f"usage: {Path(sys.argv[0]).name} ELF", file=sys.stderr)
        return 2

    try:
        frame = rust_main_frame(Path(sys.argv[1]))
    except (OSError, RuntimeError, subprocess.CalledProcessError) as error:
        print(f"stack check failed: {error}", file=sys.stderr)
        return 1

    print(f"rust_main_stack_frame={frame}")
    print(f"system_stack_capacity={SYSTEM_STACK_BYTES}")
    if frame > SYSTEM_STACK_BYTES:
        print(
            f"rust_main frame exceeds the system stack by {frame - SYSTEM_STACK_BYTES} bytes",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
