#!/usr/bin/env bash
# Run the checks that actually gate this firmware.
#
# `cargo test` alone is not sufficient and is actively misleading. The host TX
# driver is declared as
#
#     #[cfg(all(feature = "vendor-host-tx-foundation", target_arch = "arm"))]
#     pub mod host_tx_driver;
#
# so a host test run compiles *none* of it. A change to that module that does
# not even type-check for ARM can still produce a fully green `cargo test`,
# which is exactly how a broken image reached the board. Any ARM-only code is
# only checked by an ARM build.
#
# This script runs the host tests and then type-checks every feature set that
# is actually shipped or flashed.
set -euo pipefail

cd "$(dirname "$0")/.."

TARGET=thumbv5te-none-eabi
BUILD_STD=(-Z build-std=core)

# Feature sets that must always compile for ARM: the normal image, the strict
# canonical OTA profile, and the qualified hardware-CCMP profile.
NORMAL=probe-tx-experiment,wsm-cw1200-compat
OTA=probe-tx-experiment,wsm-xr819-native,vendor-host-tx-diagnostics,pipe-watchdog,corruption-non-fatal,host-lane-independent
HARDWARE=probe-tx-experiment,wsm-xr819-native,vendor-host-tx-foundation,pipe-watchdog,corruption-non-fatal,host-lane-independent,phy-start-before-scheduler,hardware-ccmp

echo "== host tests =="
cargo test --features vendor-host-tx-diagnostics

for features in "$NORMAL" "$OTA" "$HARDWARE"; do
  echo "== arm check: $features =="
  cargo check --release --bin hif-startup --target "$TARGET" "${BUILD_STD[@]}" \
    --no-default-features --features "$features"
done

echo "ALL CHECKS PASSED"
