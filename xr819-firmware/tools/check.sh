#!/usr/bin/env bash
# Run the checks that actually gate this firmware.
#
# `cargo test` alone is not sufficient and is actively misleading. The host TX
# driver is ARM-only, so a host test run compiles none of it. A change to that
# module that does not even type-check for ARM can still produce green tests,
# which is exactly how a broken image reached the board. Any ARM-only code is
# only checked by an ARM build.
#
# This script runs the host tests and then type-checks the shipped firmware.
set -euo pipefail

cd "$(dirname "$0")/.."

TARGET=thumbv5te-none-eabi
BUILD_STD=(-Z build-std=core)

echo "== host tests =="
cargo +nightly test --features vendor-host-tx-diagnostics

echo "== arm build and stack check: feature-free firmware =="
cargo +nightly build --release --bin hif-startup --target "$TARGET" "${BUILD_STD[@]}"
tools/check-rust-main-stack.py "target/$TARGET/release/hif-startup"

echo "== arm check: sectioned-image bootstrap =="
cargo +nightly check --release --bin download-boot-sectioned \
  --target armv5te-none-eabi "${BUILD_STD[@]}"

echo "ALL CHECKS PASSED"
