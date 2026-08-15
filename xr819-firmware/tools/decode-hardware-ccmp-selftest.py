#!/usr/bin/env python3
"""Decode the boot-time XR819 AES-engine CCMP known-answer result."""

import re
import sys

FIELDS = [
    ("plcp_errors", "marker"),
    ("fcs_errors", "status"),
    ("tx_packets", "irq_mask"),
    ("rx_packets", "aes_status"),
    ("rx_packet_errors", "elapsed_us"),
    ("rx_decryption_failures", "mismatch"),
    ("rx_mic_failures", "checksum"),
    ("rx_no_key_failures", "pre_microcode_status"),
    ("tx_multicast_frames", "post_microcode_status"),
    ("tx_frames_success", "post_key_status"),
    ("tx_frame_failures", "post_context_status"),
    ("tx_frames_retried", "post_aad_first_status"),
    ("tx_frames_multi_retried", "post_aad_tail_status"),
    ("rx_frame_duplicates", "start_readback"),
    ("rts_success", "source_readback"),
    ("rts_failures", "destination_readback"),
    ("ack_failures", "length_readback"),
    ("rx_multicast_frames", "microcode_checksum"),
    ("rx_frames_success", "final_aes_status"),
    ("rx_cmac_icv_errors", "matrix_cases_passed"),
    ("rx_cmac_replays", "matrix_failed_length_or_live_mismatches"),
    ("rx_cmac_key_id_errors", "first_live_mismatch"),
]
MARKER = 0x4857434B  # "HWCK"
STATUS = {
    1: "pass",
    2: "ciphertext/MIC pass; latched AES status bit 0 clear",
    0xFFFF0000: "running",
    0xE001: "timeout",
    0xE002: "engine rejected/authentication status clear",
    0xE003: "ciphertext or MIC mismatch",
    0xE004: "runtime AAD construction failed",
    0xE005: "AES microcode initialization timed out",
    0xE006: "boundary/large-payload KAT failed",
    0xE007: "RX decrypt/authentication KAT failed",
}


def main() -> int:
    try:
        text = open(sys.argv[1]).read() if len(sys.argv) > 1 else sys.stdin.read()
    except OSError as error:
        print(f"cannot read input: {error}")
        return 1

    blocks = []
    for section in text.split("COUNTERS:"):
        values = {}
        for field, label in FIELDS:
            match = re.search(rf"^{field}:\s*(-?\d+)$", section, re.MULTILINE)
            if match:
                try:
                    values[label] = int(match.group(1)) & 0xFFFFFFFF
                except ValueError:
                    continue
        if values.get("marker") == MARKER:
            blocks.append(values)
    if not blocks:
        print("no HWCK counters block found")
        return 1

    values = blocks[-1]
    status = values.get("status", 0)
    irq_mask = values.get("irq_mask", 0)
    mismatch = values.get("mismatch", 0)
    print(f"status:       {STATUS.get(status, f'unknown 0x{status:08x}')}")
    print(f"IRQ 18:       {'yes' if irq_mask & (1 << 18) else 'no'}")
    print(f"IRQ 20:       {'yes' if irq_mask & (1 << 20) else 'no'}")
    print(f"AES latched:  0x{values.get('aes_status', 0):08x}")
    print(f"AES final:    0x{values.get('final_aes_status', 0):08x}")
    print(f"elapsed:      {values.get('elapsed_us', 0)} us")
    print(f"checksum:     0x{values.get('checksum', 0):08x}")
    print("command sequence:")
    for label in (
        "pre_microcode_status",
        "post_microcode_status",
        "post_key_status",
        "post_context_status",
        "post_aad_first_status",
        "post_aad_tail_status",
        "start_readback",
    ):
        print(f"  {label:<24} 0x{values.get(label, 0):08x}")
    print(f"DMA source:   0x{values.get('source_readback', 0):08x}")
    print(f"DMA dest:     0x{values.get('destination_readback', 0):08x}")
    print(f"DMA length:   {values.get('length_readback', 0)}")
    print(f"microcode:    0x{values.get('microcode_checksum', 0):08x}")
    matrix_count = values.get("matrix_cases_passed", 0)
    print(
        f"KAT/live:     {min(matrix_count, 5)}/5 TX matrix, "
        f"{'pass' if matrix_count >= 6 else 'fail'} RX decrypt, "
        f"{'pass' if matrix_count >= 7 else 'fail'} RX bad-MIC, "
        f"{max(matrix_count - 7, 0)} verified frames"
    )
    failure = values.get("matrix_failed_length_or_live_mismatches", 0)
    if values.get("status") == 0xE006:
        print(f"failed length: {failure}")
    elif failure:
        print(f"live mismatch: {failure}, first=0x{values.get('first_live_mismatch', 0):08x}")
    if mismatch:
        print(f"mismatch:     index {(mismatch >> 16) & 0xffff}, expected 0x{(mismatch >> 8) & 0xff:02x}, actual 0x{mismatch & 0xff:02x}")
    return 0 if status == 1 else 2


if __name__ == "__main__":
    raise SystemExit(main())
