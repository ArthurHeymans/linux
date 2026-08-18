#!/usr/bin/env bash
# Build and pack the canonical over-the-air test image.
#
# Feature sets are easy to get wrong by hand, and a wrong one fails silently.
# The canonical profile uses strict vendor-shaped status ownership: unmatched
# retirement is an explicit comparison feature and must not be pulled into
# normal watchdog images. `vendor-host-tx-foundation` also gates the STA JOIN
# path, so omitting it produces an image that boots but never associates.
# The base also keeps the established non-fatal corruption mode and independent
# host lane: omitting them made a healthy batch run halt on the known
# `xr819-hif-tx-boundary` detector halfway through the diagnostic flood.
#
# `cargo test` does not catch this: the host TX service path is
# `#[cfg(target_arch = "arm")]`, so the host test build never compiles it. Only
# an ARM build does. Always go through this script before flashing.
#
#   $1 = output image path
#   $2... = extra features to append to the canonical set
set -euo pipefail

OUT=${1:?usage: build-ota-image.sh OUT [extra,features]}
shift
EXTRA=${1:-}

BASE=probe-tx-experiment,wsm-xr819-native,vendor-host-tx-diagnostics,pipe-watchdog,corruption-non-fatal,host-lane-independent
FEATURES=$BASE${EXTRA:+,$EXTRA}

cd "$(dirname "$0")/.."

cargo test --features vendor-host-tx-diagnostics >/dev/null
cargo build --release --bin hif-startup --target thumbv5te-none-eabi \
  -Z build-std=core --no-default-features --features "$FEATURES"
python3 tools/pack-sectioned-elf.py \
  target/thumbv5te-none-eabi/release/hif-startup "$OUT"

echo "features=$FEATURES"
