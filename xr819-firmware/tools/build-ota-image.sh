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
python3 tools/pack-sectioned-elf.py \
  target/thumbv5te-none-eabi/release/hif-startup "$OUT"

echo "features=none"
