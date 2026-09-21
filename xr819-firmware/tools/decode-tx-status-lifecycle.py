#!/usr/bin/env python3
"""Decode the experimental TX-status lifecycle counters-MIB layout."""

import re
import sys

MAGIC = 0x54584C43  # "TXLC"
PN_MAGIC = 0x5458504E  # "TXPN"
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
        if len(words) == len(MIB_FIELDS) and words[0] in (MAGIC, PN_MAGIC):
            marked = words
    return marked


def decode_event(raw):
    markers = []
    for bit, name in [
        (25, "pipe"),
        (24, "completion"),
        (23, "pipe_service"),
        (26, "beacon"),
        (30, "fatal"),
    ]:
        if raw & (1 << bit):
            markers.append(name)
    return {
        "type": (raw >> 8) & 0x3F,
        "pipe": (raw >> 18) & 3,
        "phase": (raw >> 16) & 3,
        "status": raw & 0x3F,
        "markers": markers,
    }


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

    if words[0] == PN_MAGIC:
        cursor = words[9]
        available = min(cursor, 4)
        print("\nperiodic accepted ordinary completions (newest first):")
        for age in range(available):
            record = (cursor - age - 1) & 3
            base = 10 + record * 3
            identity, pn_low, pn_generation = words[base:base + 3]
            sequence = identity & 0xFFF
            start_to_success = (identity >> 12) & 0x3FF
            success_to_status = (identity >> 22) & 0x3FF
            packet_number = pn_low | ((pn_generation & 0xFFFF) << 32)
            completion_ordinal = pn_generation >> 16
            print(
                f"  age={age} sequence={sequence:4d} SC=0x{sequence << 4:04x} "
                f"PN=0x{packet_number:012x} start->success={start_to_success} "
                f"success->status={success_to_status} completion={completion_ordinal}"
            )
        print("\n  -> match each sequence+PN pair against the monitor capture")
        return 0

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

    publish_to_start = words[16]
    start_to_success = words[17] & 0xFFFF
    success_to_status = words[17] >> 16
    status_to_completion = words[18] & 0xFFFF
    completion_to_confirmation = words[18] >> 16
    print("\nstage deltas:")
    print(f"  published -> started       {publish_to_start}")
    print(f"  started -> pipe_success    {start_to_success}")
    print(f"  pipe_success -> status     {success_to_status}")
    print(f"  status -> completed        {status_to_completion}")
    print(f"  completed -> confirmed     {completion_to_confirmation}")

    event_names = ["start", "pipe_success", "status"]
    event_words = words[19:22]
    print("\nraw MAC FIFO events:")
    for name, raw in zip(event_names, event_words):
        event = decode_event(raw)
        markers = ",".join(event["markers"]) or "none"
        print(
            f"  {name:<12} 0x{raw:08x} type=0x{event['type']:02x} "
            f"pipe={event['pipe']} phase={event['phase']} status=0x{event['status']:02x} "
            f"markers={markers}"
        )
    if event_words[1] == event_words[2]:
        print("  -> pipe success and accepted status came from the same FIFO word")

    if words[7]:
        print("\n  -> slot identity changed before a later lifecycle event")
    if words[8]:
        print("\n  -> one or more stages arrived without all required predecessors")
    if stage_bits == 0x3F:
        print("\n  -> software observed the complete chain; compare sequence_number with the air capture")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
