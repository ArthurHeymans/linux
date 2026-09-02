#!/usr/bin/env python3
"""Decode experimental depth-three/four A-MPDU telemetry from a counters log.

Build the image with `experimental-depth-four-ampdu` and
`vendor-host-tx-diagnostics`. The firmware reuses the final four fields of the
hardware-CCMP counters snapshot. Normal depth telemetry stores two saturating
16-bit counts per word. Images built with `experimental-ampdu-outcome-telemetry`
reuse those same four words for sixteen packed saturating 8-bit outcomes; pass
`--outcomes` to decode that layout.
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
            try:
                values[label] = int(match.group(1)) & 0xFFFFFFFF
            except ValueError:
                continue
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
    outcome_mode = "--outcomes" in sys.argv[1:]
    paths = [argument for argument in sys.argv[1:] if argument != "--outcomes"]
    try:
        text = sys.stdin.read() if not paths else open(paths[0]).read()
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

    outcome_names = [
        "ba_all_ack_rx",
        "ba_partial_rx",
        "deep_plan_rearm",
        "deep_plan_empty",
        "deep_plan_invalid",
        "deep_whole_rearm",
        "deep_give_up",
        "retry_event_rearm",
        "retry_event_give_up",
        "retry_event_complete",
        "watchdog_rearm",
        "watchdog_no_rearm",
        "retired_unmatched",
        "partial_give_up",
        "depth2_selective",
        "depth2_whole",
    ]
    if outcome_mode:
        words = [values.get(stage, 0) for stage in ("attempted", "published", "completed", "retried")]
        print("  outcomes:")
        for index, name in enumerate(outcome_names):
            count = (words[index >> 2] >> ((index & 3) * 8)) & 0xFF
            print(f"    {name:<24} {count:3d}")
    else:
        for stage in ("attempted", "published", "completed"):
            word = values.get(stage, 0)
            print(
                f"  {stage:<9} depth3={word & 0xFFFF:5d} "
                f"depth4={(word >> 16) & 0xFFFF:5d}"
            )
        retried = values.get("retried", 0)
        print(
            f"  retried   single={retried & 0xFFFF:5d} "
            f"multi={(retried >> 16) & 0xFFFF:5d}"
        )
    return 0


if __name__ == "__main__":
    sys.exit(main())
