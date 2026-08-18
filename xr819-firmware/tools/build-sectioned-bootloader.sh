#!/usr/bin/env bash
# Build the ARM bootstrap that loads sectioned ITCM/DTCM firmware images.
set -euo pipefail

if [ "$#" -ne 1 ]; then
  echo "usage: $0 OUTPUT.bin" >&2
  exit 2
fi
OUT=$1

cd "$(dirname "$0")/.."

# This bootstrap starts in ordinary SRAM in ARM state, then relocates its body
# to packet RAM. Override the Thumb main-image linker configuration completely.
RUSTFLAGS='-C target-feature=+strict-align -C link-arg=-Tlink-download.x' \
  cargo +nightly build --release --bin download-boot-sectioned \
    --target armv5te-none-eabi -Z build-std=core

ELF=target/armv5te-none-eabi/release/download-boot-sectioned
llvm-objcopy -O binary "$ELF" "$OUT"

printf 'elf=%s\n' "$ELF"
printf 'entry=0x08000000\n'
printf 'bootstrap_stack=0x0400c000\n'
printf 'output=%s\n' "$OUT"
sha256sum "$OUT"
