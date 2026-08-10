#!/usr/bin/env python3
"""Combine the validated low XR819 image with a high-SRAM extension.

The low call at 0x532 is redirected to the loader-installed Thumb/ARM veneer at
0x9720. The veneer dispatches to the extension loaded at 0xfff00000 and the
extension returns to the original low caller after invoking stable publish.
"""

from __future__ import annotations

import argparse
import hashlib
from pathlib import Path

LOW_IMAGE_SIZE = 0x7500
PUBLISH_CALL_OFFSET = 0x532
CHANNEL_DIVIDE_CALL_OFFSET = 0x63C6
EXPECTED_STABLE_SHA256 = (
    "96714ce0fee7c947ed512df54e49ec9edac32233c3446eba0cb7308818cfb765"
)
EXPECTED_PUBLISH_CALL = bytes.fromhex("01f07bf8")
VENEER_PUBLISH_CALL = bytes.fromhex("09f0f5f8")
EXPECTED_CHANNEL_DIVIDE_CALL = bytes.fromhex("00f05bfe")
VENEER_CHANNEL_DIVIDE_CALL = bytes.fromhex("03f0b3f9")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("stable", type=Path)
    parser.add_argument("extension", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()

    low = bytearray(args.stable.read_bytes())
    extension = args.extension.read_bytes()
    digest = hashlib.sha256(low).hexdigest()

    if len(low) != LOW_IMAGE_SIZE:
        raise SystemExit(f"stable image is {len(low):#x} bytes, expected {LOW_IMAGE_SIZE:#x}")
    if digest != EXPECTED_STABLE_SHA256:
        raise SystemExit(f"unexpected stable image SHA-256: {digest}")
    if low[PUBLISH_CALL_OFFSET : PUBLISH_CALL_OFFSET + 4] != EXPECTED_PUBLISH_CALL:
        raise SystemExit("stable startup publish call no longer matches the validated instruction")
    if (
        low[CHANNEL_DIVIDE_CALL_OFFSET : CHANNEL_DIVIDE_CALL_OFFSET + 4]
        != EXPECTED_CHANNEL_DIVIDE_CALL
    ):
        raise SystemExit("stable channel divide call no longer matches the validated instruction")
    if not extension:
        raise SystemExit("extension image is empty")

    low[PUBLISH_CALL_OFFSET : PUBLISH_CALL_OFFSET + 4] = VENEER_PUBLISH_CALL
    low[CHANNEL_DIVIDE_CALL_OFFSET : CHANNEL_DIVIDE_CALL_OFFSET + 4] = (
        VENEER_CHANNEL_DIVIDE_CALL
    )
    args.output.write_bytes(low + extension)

    print(f"low={len(low):#x} extension={len(extension):#x} total={len(low) + len(extension):#x}")
    print(f"sha256={hashlib.sha256(low + extension).hexdigest()}")


if __name__ == "__main__":
    main()
