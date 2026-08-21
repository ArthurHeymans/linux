#!/usr/bin/env bash
# Build and pack the feature-free production firmware.
#
# Run tools/check.sh separately when qualification checks are required.
set -euo pipefail

if [ "$#" -ne 1 ]; then
  echo "usage: $0 OUTPUT.bin" >&2
  exit 2
fi
OUT=$1

cd "$(dirname "$0")/.."

cargo +nightly build --release --bin hif-startup --target thumbv5te-none-eabi \
  -Z build-std=core
ELF=target/thumbv5te-none-eabi/release/hif-startup
python3 tools/check-rust-main-stack.py "$ELF"
python3 tools/check-packet-ram-layout.py "$ELF"
python3 tools/check-address-literals.py
python3 tools/check-low-mac-pas-layout.py
python3 tools/check-link-sequence-layout.py
python3 tools/check-join-scan-layout.py
python3 tools/check-ba-lmc-pending-layout.py
python3 tools/check-ba-session-layout.py
python3 tools/check-ba-link-event-layout.py
python3 tools/pack-sectioned-elf.py "$ELF" "$OUT"
python3 tools/check-dtcm-layout.py "$ELF" "$OUT"
python3 tools/check-low-mac-pas-layout.py "$ELF"
python3 tools/check-link-sequence-layout.py "$ELF"
python3 tools/check-join-scan-layout.py "$ELF"
python3 tools/check-ba-lmc-pending-layout.py "$ELF"
python3 tools/check-ba-session-layout.py "$ELF"
python3 tools/check-ba-link-event-layout.py "$ELF"

echo "features=none"
