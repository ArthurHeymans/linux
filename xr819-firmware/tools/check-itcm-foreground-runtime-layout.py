#!/usr/bin/env python3
"""Reject movement of the hardware-qualified foreground ITCM runtime prefix."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

EXPECTED = {
    "__bss_start": 0x00014210,
    "__itcm_hif_queues_start": 0x00014210,
    "__itcm_hif_ring_state_start": 0x00014394,
    "__itcm_host_tx_driver_start": 0x000143AC,
    "__itcm_foreground_quarantine_end": 0x00015150,
}


def symbols(elf: Path) -> dict[str, int]:
    output = subprocess.run(
        ["llvm-nm", "--defined-only", str(elf)],
        check=True,
        stdout=subprocess.PIPE,
        text=True,
    ).stdout
    result: dict[str, int] = {}
    for line in output.splitlines():
        fields = line.split(maxsplit=2)
        if len(fields) == 3:
            address, _kind, name = fields
            if name in EXPECTED:
                result[name] = int(address, 16)
    return result


def main() -> int:
    if len(sys.argv) != 2:
        print(f"usage: {Path(sys.argv[0]).name} ELF", file=sys.stderr)
        return 2

    elf = Path(sys.argv[1])
    try:
        actual = symbols(elf)
    except (OSError, subprocess.CalledProcessError, ValueError) as error:
        print(f"foreground ITCM layout check failed: {error}", file=sys.stderr)
        return 1

    missing = EXPECTED.keys() - actual.keys()
    if missing:
        print(f"foreground ITCM layout symbols missing: {', '.join(sorted(missing))}", file=sys.stderr)
        return 1

    failures = [
        f"{name}: expected {expected:#010x}, got {actual[name]:#010x}"
        for name, expected in EXPECTED.items()
        if actual[name] != expected
    ]
    if failures:
        print("foreground ITCM runtime layout moved:", file=sys.stderr)
        for failure in failures:
            print(f"  {failure}", file=sys.stderr)
        return 1

    for name, expected in EXPECTED.items():
        print(f"{name}={expected:#010x}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
