#!/usr/bin/env python3
"""Decode the experimental TX-status lifecycle counters-MIB layout."""

import re
import sys

MAGIC = 0x54584C43  # "TXLC"
MIB_FIELDS = [
    "plcp_errors",
    "fcs_errors",
    "tx_packets",
    "rx_packets",
    "rx_packet_errors",
    "rx_decryption_failures",
    "rx_mic_failures",
    "rx_no_key_failures",
    "tx_multicast_frames",
    "tx_frames_success",
    "tx_frame_failures",
    "tx_frames_retried",
    "tx_frames_multi_retried",
    "rx_frame_duplicates",
    "rts_success",
    "rts_failures",
    "ack_failures",
    "rx_multicast_frames",
    "rx_frames_success",
    "rx_cmac_icv_errors",
    "rx_cmac_replays",
    "rx_mgmt_ccmp_replays",
]
STAGES = [
    "published",
    "started",
    "pipe_success",
    "status_accepted",
    "completed",
    "confirmed",
]


def parse_last_marked_block(text):
    marked = None
    # Harnesses either prefix a block with "COUNTERS:" or print the 22 fields
    # directly below a timestamped MIB heading. A positive lookahead keeps the
    # leading plcp_errors line in every candidate section.
    sections = re.split(r"(?=^plcp_errors:)", text, flags=re.MULTILINE)
    for section in sections:
        words = []
        for field in MIB_FIELDS:
            match = re.search(rf"^{field}:\s*(-?\d+)$", section, re.MULTILINE)
            if not match:
                break
            try:
                words.append(int(match.group(1)) & 0xFFFFFFFF)
            except ValueError:
                break
        if len(words) == len(MIB_FIELDS) and words[0] == MAGIC:
            marked = words
    return marked


def elapsed(start, end):
    return (end - start) & 0xFFFFFFFF if start and end else None


def main():
    try:
        if len(sys.argv) > 1:
            with open(sys.argv[1]) as handle:
                text = handle.read()
        else:
            text = sys.stdin.read()
    except OSError as error:
        print(f"cannot read input: {error}")
        return 1

    words = parse_last_marked_block(text)
    if words is None:
        print("no TXLC counters block found")
        return 1

    labels = [
        "published",
        "started",
        "pipe_success",
        "status_accepted",
        "completed",
        "confirmed",
        "identity_mismatch",
        "out_of_order",
    ]
    print("observations:")
    for label, value in zip(labels, words[1:9]):
        print(f"  {label:<18} {value}")

    packet_id, context, sequence = words[9:12]
    pipe_slot_generation, stage_bits, status_detail, terminal = words[12:16]
    pipe = pipe_slot_generation & 0xFF
    slot = (pipe_slot_generation >> 8) & 0xFF
    generation = pipe_slot_generation >> 16
    delivered = status_detail & 0xFF
    expected = (status_detail >> 8) & 0xFF
    slot_state = (status_detail >> 16) & 0xFF
    slot_kind = status_detail >> 24
    terminal_status = terminal & 0xFFFF
    retries = terminal >> 16
    present_stages = [name for bit, name in enumerate(STAGES) if stage_bits & (1 << bit)]

    print("\nlatest accepted ordinary completion:")
    print(f"  packet_id          0x{packet_id:08x}")
    print(f"  context            0x{context:08x}")
    print(f"  sequence_number    {sequence & 0x0fff} (802.11 SC 0x{(sequence & 0x0fff) << 4:04x})")
    print(f"  pipe/slot/gen       {pipe}/{slot}/{generation}")
    print(f"  stages             0x{stage_bits:02x} ({', '.join(present_stages) or 'none'})")
    print(f"  status             delivered=0x{delivered:02x} expected=0x{expected:02x}")
    print(f"  slot               kind={slot_kind} state={slot_state}")
    print(f"  terminal/retries   0x{terminal_status:04x}/{retries}")

    timestamps = words[16:22]
    print("\ntimestamps and deltas:")
    for index, (name, timestamp) in enumerate(zip(STAGES, timestamps)):
        delta = elapsed(timestamps[index - 1], timestamp) if index else None
        suffix = "" if delta is None else f" (+{delta})"
        print(f"  {name:<18} 0x{timestamp:08x}{suffix}")

    if words[7]:
        print("\n  -> slot identity changed before a later lifecycle event")
    if words[8]:
        print("\n  -> one or more stages arrived without all required predecessors")
    if stage_bits == 0x3F:
        print("\n  -> software observed the complete chain; compare sequence_number with the air capture")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
