#!/usr/bin/env python3
"""Decode class-0 lifecycle counters from a soak-run log.

The firmware reports these through the counters MIB, so the cw1200 driver
prints them under the MIB's own field names. This maps them back. It also
recovers the last complete five-event C0LC/C0LS burst when a post-traffic MIB
read fails and returns an all-zero block.

Build the image with `class0-lifecycle-counters`; the marker distinguishes this
layout from the TX identity snapshot the same MIB otherwise carries. Pass
`--batch` for a `tx-pipelining` image whose final three diagnostic slots carry
batch-arm and retry-cursor metrics instead of single-frame timings.
"""

import re
import sys

MARKER = 0x43304C43  # "C0LC"
SUM_MARKER = 0x43304C53  # "C0LS"

# MIB field name -> lifecycle counter name, in slot order.
FIELDS = [
    ("plcp_errors", "marker"),
    ("fcs_errors", "admitted"),
    ("tx_packets", "published"),
    ("rx_packets", "tx_start"),
    ("rx_packet_errors", "status_delivered"),
    ("rx_decryption_failures", "status_ineligible"),
    ("rx_mic_failures", "completed"),
    ("rx_no_key_failures", "rx_valid_slots"),
    ("tx_multicast_frames", "confirmed"),
    ("tx_frames_success", "rx_filtered"),
    ("tx_frame_failures", "rx_indications"),
    ("tx_frames_retried", "stage_pending_split"),
    ("tx_frames_multi_retried", "rx_resync"),
    ("rx_frame_duplicates", "give_up"),
    ("rts_success", "latency_max_us"),
    ("rts_failures", "rx_released"),
    ("ack_failures", "stage_admit_publish"),
    ("rx_multicast_frames", "stage_start_confirm"),
    ("rx_frames_success", "pending_gate"),
    ("rx_cmac_icv_errors", "retirement_deferred"),
    ("rx_cmac_replays", "output_corruption"),
    ("rx_mgmt_ccmp_replays", "suppressed_exception"),
]


def unpack_stage_pair(value):
    """Two 16-bit microsecond fields packed into one counter."""
    return value & 0xFFFF, (value >> 16) & 0xFFFF


def parse_block(text):
    values = {}
    for mib_name, label in FIELDS:
        match = re.search(rf"^{mib_name}:\s*(-?\d+)$", text, re.MULTILINE)
        if match:
            try:
                values[label] = int(match.group(1)) & 0xFFFFFFFF
            except ValueError:
                continue
    return values


def parse(text):
    # Soak logs contain pre-traffic, post-flood and post-iperf blocks. A MIB
    # read can fail by returning a complete-looking block of zeros, so neither
    # the first nor the final textual match is authoritative. Select the last
    # self-contained block carrying one of our layout markers.
    sections = text.split("COUNTERS:")
    blocks = [parse_block(section) for section in sections]
    marked = [block for block in blocks if block.get("marker") in (MARKER, SUM_MARKER)]
    if marked:
        return marked[-1]
    nonempty = [block for block in blocks if block]
    return nonempty[-1] if nonempty else {}


def apply_layout(values):
    """Rename reused slots when the firmware reports cumulative latency sums."""
    if values.get("marker") != SUM_MARKER:
        return values
    remapped = dict(values)
    replacements = {
        "stage_pending_split": "latency_sum_admit_publish",
        "latency_max_us": "latency_sum_publish_start",
        "stage_admit_publish": "latency_sum_start_complete",
        "stage_start_confirm": "latency_sum_complete_confirm",
        "pending_gate": "latency_sample_count",
    }
    for old, new in replacements.items():
        remapped[new] = remapped.pop(old, 0)
    return remapped


def parse_pushed_lifecycle(text):
    """Recover the last complete five-event C0LC counter burst."""
    labels = [label for _, label in FIELDS[1:11]]
    current = {}
    complete = {}
    seen = set()
    for match in re.finditer(r"\b([0-9a-fA-F]{24,})$", text, re.MULTILINE):
        try:
            payload = bytes.fromhex(match.group(1))
        except ValueError:
            continue
        if len(payload) < 12:
            continue
        event_id = int.from_bytes(payload[4:8], "little")
        if event_id & 0xFFFF0000 != 0x4C430000:
            continue
        burst_index = event_id & 0xFFFF
        if burst_index >= 5:
            continue
        if burst_index == 0:
            current = {}
            seen = set()
        packed = int.from_bytes(payload[8:12], "little")
        low = burst_index * 2
        current[labels[low]] = packed & 0xFFFF
        current[labels[low + 1]] = packed >> 16
        seen.add(burst_index)
        if len(seen) == 5:
            complete = dict(current)
    return complete


def parse_pushed_latency(text):
    """Recover full-width C0LS counters from WSM event trace payloads."""
    labels = {
        10: "latency_sum_admit_publish",
        13: "latency_sum_publish_start",
        15: "latency_sum_start_complete",
        16: "latency_sum_complete_confirm",
        17: "latency_sample_count",
    }
    values = {}
    for match in re.finditer(r"\b([0-9a-fA-F]{24,})$", text, re.MULTILINE):
        try:
            payload = bytes.fromhex(match.group(1))
        except ValueError:
            continue
        if len(payload) < 12:
            continue
        event_id = int.from_bytes(payload[4:8], "little")
        if event_id & 0xFFFF0000 != 0x4C530000:
            continue
        label = labels.get(event_id & 0xFFFF)
        if label:
            values[label] = int.from_bytes(payload[8:12], "little")
    return values


def main():
    batch_layout = "--batch" in sys.argv[1:]
    paths = [argument for argument in sys.argv[1:] if argument != "--batch"]
    try:
        if not paths:
            text = sys.stdin.read()
        else:
            with open(paths[0]) as handle:
                text = handle.read()
    except OSError as error:
        print(f"cannot read input: {error}")
        return 1

    values = apply_layout(parse(text))
    pushed_lifecycle = parse_pushed_lifecycle(text)
    pushed = parse_pushed_latency(text)
    if pushed_lifecycle:
        values["marker"] = MARKER
        values.update(pushed_lifecycle)
    if pushed:
        values["marker"] = SUM_MARKER
        values.update(pushed)
    if batch_layout and values.get("marker") == MARKER:
        values["batch_arms"] = values.pop("stage_admit_publish", 0)
        values["multi_slot_arms"] = values.pop("stage_start_confirm", 0)
        values["scheduler_capacity"] = values.pop("pending_gate", 0)
    if not values:
        print("no counters block found")
        return 1
    if values.get("marker") not in (MARKER, SUM_MARKER):
        print(f"marker 0x{values.get('marker', 0):08x} is not C0LC/C0LS:")
        print("  image was not built with class0-lifecycle-counters,")
        print("  or the MIB carried the identity snapshot instead")
        return 1

    order = [label for _, label in FIELDS[1:]]
    if batch_layout and values.get("marker") == MARKER:
        replacements = {
            "stage_admit_publish": "batch_arms",
            "stage_start_confirm": "multi_slot_arms",
            "pending_gate": "scheduler_capacity",
        }
        order = [replacements.get(label, label) for label in order]
    if values.get("marker") == SUM_MARKER:
        replacements = {
            "stage_pending_split": "latency_sum_admit_publish",
            "latency_max_us": "latency_sum_publish_start",
            "stage_admit_publish": "latency_sum_start_complete",
            "stage_start_confirm": "latency_sum_complete_confirm",
            "pending_gate": "latency_sample_count",
        }
        order = [replacements.get(label, label) for label in order]
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
    if batch_layout and "scheduler_capacity" in values:
        packed = values["scheduler_capacity"]
        print("\n  scheduler capacity:")
        print(f"    passes starting with >=2 PAS  {packed & 0xffff:8d}")
        print(f"    failed to stage second slot    {(packed >> 16) & 0xffff:8d}")
    print_stage_breakdown(values)
    return 0


def print_stage_breakdown(values):
    """Where a frame's admission-to-confirmation time goes.

    Only start->complete is airtime. Time in admit->publish or
    complete->confirm is our own driver, and deeper TX pipelining cannot
    help with it.
    """
    if values.get("marker") == SUM_MARKER:
        print_latency_sums(values)
        return

    loop_period = values.get("output_corruption")
    if loop_period and loop_period & 0x8000_0000:
        # Latched once, at the first suppressed MAC fatal.
        print("\n  Funnel at the FIRST MAC fatal:")
        print(f"    completed at first fatal  {loop_period & 0xFFFF:8d}")
        print(f"    published at first fatal  {(loop_period >> 16) & 0x7FFF:8d}")
        done = values.get("completed")
        if done is not None:
            after = done - (loop_period & 0xFFFF)
            verdict = "fatal stops completion" if after <= 2 else "completion continues past it"
            print(f"    completions after it      {after:8d}  <- {verdict}")
    elif loop_period:
        print("\n  Main loop:")
        print(f"    main loop iterations: {loop_period}")
    resync = values.get("rx_resync")
    if resync is not None:
        print("\n  RX resync paths (vendor rxfifo_next_frame scan):")
        print(f"    accepted (valid slot)   {resync & 0x3FF:8d}")
        print(f"    flushed (span exhausted){(resync >> 10) & 0x3FF:8d}   <- discards the whole pending region")
        print(f"    rejected by lookahead   {(resync >> 20) & 0xFFF:8d}   <- false magic matches vendor skips")
    released = values.get("rx_released")
    if released is not None:
        print("\n  RX path:")
        print(f"    valid slots seen        {values.get('rx_valid_slots', 0):8d}")
        print(f"    indications published   {values.get('rx_indications', 0):8d}")
        print(f"    frames dropped          {values.get('rx_filtered', 0):8d}")
        print(f"    slots released          {released & 0xFFFF:8d}")
        print(f"    host transfers in use   {(released >> 16) & 0xFFFF:8d} / 24")
    gate = values.get("pending_gate")
    if gate:
        passes = gate & 0xFFFF
        mask = (gate >> 16) & 0xFFFF
        names = [
            (1 << 0, "link inactive"),
            (1 << 1, "globally blocked (0x04001fcc & 0xa0)"),
            (1 << 2, "vif not operating"),
            (1 << 3, "pipe not eligible (program_pipe_eligible)"),
            (1 << 4, "expired"),
        ]
        refused = [label for bit, label in names if mask & bit] or ["none recorded"]
        print(f"\n  pending gate: {passes} LeaveQueued passes, refused by:")
        for label in refused:
            print(f"    - {label}")
    pending_split = values.get("stage_pending_split")
    if pending_split is not None:
        admit_to_pas, pas_to_publish = unpack_stage_pair(pending_split)
        print("\n  admit -> publish, split:")
        print(f"    admission -> PAS release (pending gating)  {admit_to_pas:6d} us")
        print(f"    PAS release -> publication (reservation)   {pas_to_publish:6d} us")
    admit_publish = values.get("stage_admit_publish")
    start_confirm = values.get("stage_start_confirm")
    if admit_publish is None or start_confirm is None:
        return
    admit_to_publish, publish_to_start = unpack_stage_pair(admit_publish)
    start_to_complete, complete_to_confirm = unpack_stage_pair(start_confirm)
    stages = [
        ("admit -> publish  (driver)", admit_to_publish),
        ("publish -> start  (MAC pickup)", publish_to_start),
        ("start -> complete (airtime)", start_to_complete),
        ("complete -> confirm (driver)", complete_to_confirm),
    ]
    total = sum(value for _, value in stages)
    print("\n  latency breakdown, last frame:")
    for label, value in stages:
        share = f"{100.0 * value / total:5.1f}%" if total else "    -"
        print(f"    {label:<34} {value:6d} us  {share}")
    print(f"    {'total':<34} {total:6d} us")


def print_latency_sums(values):
    samples = values.get("latency_sample_count", 0)
    print("\n  latency breakdown, run average:")
    if not samples:
        print("    no valid stage samples")
        return
    stages = [
        ("admit -> publish  (driver)", "latency_sum_admit_publish"),
        ("publish -> first start (pickup)", "latency_sum_publish_start"),
        ("first start -> complete (retries)", "latency_sum_start_complete"),
        ("complete -> confirm (driver)", "latency_sum_complete_confirm"),
    ]
    averages = [(label, values.get(key, 0) / samples) for label, key in stages]
    total = sum(value for _, value in averages)
    print(f"    valid samples                       {samples:8d}")
    for label, value in averages:
        share = f"{100.0 * value / total:5.1f}%" if total else "    -"
        print(f"    {label:<34} {value:8.1f} us  {share}")
    print(f"    {'total':<34} {total:8.1f} us")

    saturated = [key for _, key in stages if values.get(key) == 0xFFFFFFFF]
    if saturated:
        print("    WARNING: saturated sums: " + ", ".join(saturated))


if __name__ == "__main__":
    sys.exit(main())
