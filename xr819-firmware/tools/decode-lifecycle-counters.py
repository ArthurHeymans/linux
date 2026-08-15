#!/usr/bin/env python3
"""Decode class-0 lifecycle counters from a soak-run log.

The firmware reports these through the counters MIB, so the cw1200 driver
prints them under the MIB's own field names. This maps them back.

Build the image with `class0-lifecycle-counters`; the marker distinguishes this
layout from the TX identity snapshot the same MIB otherwise carries.
"""

import re
import sys

MARKER = 0x43304C43  # "C0LC"

# MIB field name -> lifecycle counter name, in slot order.
FIELDS = [
    ("plcp_errors", "marker"),
    ("fcs_errors", "admitted"),
    ("tx_packets", "published"),
    ("rx_packets", "tx_start"),
    ("rx_packet_errors", "status_delivered"),
    ("rx_decryption_failures", "status_ineligible"),
    ("rx_mic_failures", "completed"),
    ("rx_no_key_failures", "retired"),
    ("tx_multicast_frames", "confirmed"),
    ("tx_frames_success", "last_status"),
    ("tx_frame_failures", "last_expected"),
    ("tx_frames_retried", "watchdog_recovered"),
    ("tx_frames_multi_retried", "rx_resync"),
    ("rx_frame_duplicates", "latency_last_us"),
    ("rts_success", "latency_max_us"),
    ("rts_failures", "retired_unstarted"),
    ("ack_failures", "retired_started"),
    ("rx_multicast_frames", "retired_tx_success"),
    ("rx_frames_success", "retired_last_status"),
    ("rx_cmac_icv_errors", "retirement_deferred"),
    ("rx_cmac_replays", "output_corruption"),
    ("rx_mgmt_ccmp_replays", "suppressed_exception"),
]


def parse(text):
    values = {}
    for mib_name, label in FIELDS:
        match = re.search(rf"^{mib_name}:\s*(-?\d+)$", text, re.MULTILINE)
        if match:
            try:
                values[label] = int(match.group(1)) & 0xFFFFFFFF
            except ValueError:
                continue
    return values


def main():
    try:
        if len(sys.argv) < 2:
            text = sys.stdin.read()
        else:
            with open(sys.argv[1]) as handle:
                text = handle.read()
    except OSError as error:
        print(f"cannot read input: {error}")
        return 1

    values = parse(text)
    if not values:
        print("no counters block found")
        return 1
    if values.get("marker") != MARKER:
        print(f"marker 0x{values.get('marker', 0):08x} is not C0LC:")
        print("  image was not built with class0-lifecycle-counters,")
        print("  or the MIB carried the identity snapshot instead")
        return 1

    order = [label for _, label in FIELDS[1:]]
    width = max(len(label) for label in order)
    for label in order:
        value = values.get(label, 0)
        if label.startswith("last_"):
            print(f"  {label:<{width}}  0x{value:02x}")
        else:
            print(f"  {label:<{width}}  {value}")

    # The class-0 path is a funnel; each stage should be reached by everything
    # the previous stage passed on.
    #
    # `status_ineligible` is deliberately NOT a fault signal. A healthy
    # pre-traffic sample measured 45 ineligible of 49 delivered while
    # completing normally: the gate refuses every status that does not belong
    # to the slot currently under service, which is routine. Only movement
    # through the funnel carries signal.
    published = values.get("published", 0)
    started = values.get("tx_start", 0)
    completed = values.get("completed", 0)
    print()
    if published and not started:
        print("  -> published but never started: the MAC refuses at start")
    elif started and not completed:
        print("  -> started but never completed: no matching status delivered")
    if published and not values.get("confirmed", 0):
        print("  -> published but nothing confirmed back to the host")
    return 0


if __name__ == "__main__":
    sys.exit(main())
