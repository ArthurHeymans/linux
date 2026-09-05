#!/usr/bin/env python3
"""Decode experimental depth-three/four A-MPDU telemetry from a counters log.

Build the image with `experimental-depth-four-ampdu` and
`vendor-host-tx-diagnostics`. The firmware reuses the final four fields of the
hardware-CCMP counters snapshot. Normal depth telemetry stores two saturating
16-bit counts per word. Images built with `experimental-ampdu-outcome-telemetry`
reuse those same four words for sixteen packed saturating 8-bit outcomes; pass
`--outcomes` to decode that layout. Pass `--feedback` for images that reuse
four outcome bytes for the raw retry-feedback census. Pass `--depth-five` for
the isolated member-five image, where the low halfword is repurposed from
depth three to depth five. Pass `--depth-eight` for the corresponding isolated
depth-eight layout.
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
    feedback_mode = "--feedback" in sys.argv[1:]
    depth_five_mode = "--depth-five" in sys.argv[1:]
    depth_eight_mode = "--depth-eight" in sys.argv[1:]
    outcome_mode = feedback_mode or "--outcomes" in sys.argv[1:]
    paths = [
        argument
        for argument in sys.argv[1:]
        if argument not in ("--outcomes", "--feedback", "--depth-five", "--depth-eight")
    ]
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
        "deep_plan_no_session",
        "retry_event_rearm",
        "retry_event_give_up",
        "retry_event_complete",
        "watchdog_rearm",
        "deep_plan_mixed_rate",
        "retired_unmatched",
        "deep_plan_outside_window",
        "deep_plan_no_rate",
        "depth2_whole",
    ]
    if feedback_mode:
        outcome_names[6] = "feedback_ack_failures"
        outcome_names[11] = "feedback_rate_try"
        outcome_names[13] = "feedback_ack_without_try"
        outcome_names[14] = "feedback_count_mismatch"
    if outcome_mode:
        words = [values.get(stage, 0) for stage in ("attempted", "published", "completed", "retried")]
        print("  feedback:" if feedback_mode else "  outcomes:")
        for index, name in enumerate(outcome_names):
            count = (words[index >> 2] >> ((index & 3) * 8)) & 0xFF
            print(f"    {name:<24} {count:3d}")
    else:
        low_depth = 8 if depth_eight_mode else 5 if depth_five_mode else 3
        for stage in ("attempted", "published", "completed"):
            word = values.get(stage, 0)
            print(
                f"  {stage:<9} depth{low_depth}={word & 0xFFFF:5d} "
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
