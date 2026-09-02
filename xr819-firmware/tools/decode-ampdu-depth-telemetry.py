#!/usr/bin/env python3
"""Decode experimental depth-three/four A-MPDU telemetry from a counters log.

Build the image with `experimental-depth-four-ampdu` and
`vendor-host-tx-diagnostics`. The firmware reuses the final four fields of the
hardware-CCMP counters snapshot. Each word stores saturating 16-bit depth-three
and depth-four observation counts in its low and high halves.
"""

import re
import sys

MARKER = 0x4857434B  # "HWCK"
FIELDS = [
    ("plcp_errors", "marker"),
    ("rx_frames_success", "attempted"),
    ("rx_cmac_icv_errors", "published"),
    ("rx_cmac_replays", "completed"),
    ("rx_mgmt_ccmp_replays", "retried"),
]


def parse_block(text):
    values = {}
    for field, label in FIELDS:
        match = re.search(rf"^{field}:\s*(-?\d+)$", text, re.MULTILINE)
        if match:
            values[label] = int(match.group(1)) & 0xFFFFFFFF
    return values


def parse(text):
    sections = text.split("COUNTERS:")
    blocks = [parse_block(section) for section in sections]
    marked = [block for block in blocks if block.get("marker") == MARKER]
    if marked:
        return marked[-1]
    nonempty = [block for block in blocks if block]
    return nonempty[-1] if nonempty else {}


def main():
    try:
        text = sys.stdin.read() if len(sys.argv) == 1 else open(sys.argv[1]).read()
    except OSError as error:
        print(f"cannot read input: {error}")
        return 1

    values = parse(text)
    if not values:
        print("no counters block found")
        return 1
    if values.get("marker") != MARKER:
        print(f"marker 0x{values.get('marker', 0):08x} is not HWCK")
        print("  image lacks the depth-four diagnostic layout, or the MIB read failed")
        return 1

    for stage in ("attempted", "published", "completed", "retried"):
        word = values.get(stage, 0)
        print(
            f"  {stage:<9} depth3={word & 0xFFFF:5d} "
            f"depth4={(word >> 16) & 0xFFFF:5d}"
        )
    return 0


if __name__ == "__main__":
    sys.exit(main())
