#!/usr/bin/env bash
# Run the checks that actually gate this firmware.
#
# Host tests alone are insufficient because the HIF/TX driver is ARM-only. The
# production ELF is also packed here so linker segments, packet-RAM NOBITS
# sections, stack bounds, and packed copy/fill destinations are all exercised.
set -euo pipefail

cd "$(dirname "$0")/.."

TARGET=thumbv5te-none-eabi
BUILD_STD=(-Z build-std=core)
ELF="target/$TARGET/release/hif-startup"
PACKED=$(mktemp)
BOOTSTRAP=$(mktemp)
trap 'rm -f "$PACKED" "$BOOTSTRAP"' EXIT

echo "== default process-local host tests (do not execute ARM driver/HIF behavior) =="
cargo +nightly test

echo "== diagnostic process-local host tests/recorders (do not execute ARM driver/HIF behavior) =="
cargo +nightly test --features vendor-host-tx-diagnostics

echo "== source and packer gates =="
python3 tools/check-address-literals.py
python3 tools/check-scheduler-event-layout.py
python3 tools/check-scheduler-support-layout.py
python3 tools/check-low-mac-pas-layout.py
python3 tools/check-vif-layout.py
python3 tools/check-vif-timer-layout.py
python3 tools/check-host-context-layout.py
python3 tools/check-peer-pipe-layout.py
python3 tools/check-command-channel-overlay.py
python3 tools/check-lmc-control-layout.py
python3 tools/check-link-sequence-layout.py
python3 tools/check-join-scan-layout.py
python3 tools/check-ba-lmc-pending-layout.py
python3 tools/check-ba-session-layout.py
python3 tools/check-ba-link-event-layout.py
python3 tools/test-pack-sectioned-elf.py

echo "== arm build and stack check: feature-free firmware =="
cargo +nightly build --release --bin hif-startup --target "$TARGET" "${BUILD_STD[@]}"
python3 tools/check-rust-main-stack.py "$ELF"
python3 tools/check-packet-ram-layout.py "$ELF"
python3 tools/pack-sectioned-elf.py "$ELF" "$PACKED"
python3 tools/check-dtcm-layout.py "$ELF" "$PACKED"
python3 tools/check-scheduler-event-layout.py "$ELF"
python3 tools/check-scheduler-support-layout.py "$ELF"
python3 tools/check-low-mac-pas-layout.py "$ELF"
python3 tools/check-vif-layout.py "$ELF"
python3 tools/check-vif-timer-layout.py "$ELF"
python3 tools/check-host-context-layout.py "$ELF"
python3 tools/check-peer-pipe-layout.py "$ELF"
python3 tools/check-command-channel-overlay.py "$ELF"
python3 tools/check-lmc-control-layout.py "$ELF"
python3 tools/check-link-sequence-layout.py "$ELF"
python3 tools/check-join-scan-layout.py "$ELF"
python3 tools/check-ba-lmc-pending-layout.py "$ELF"
python3 tools/check-ba-session-layout.py "$ELF"
python3 tools/check-ba-link-event-layout.py "$ELF"

if [[ -n "${XR819_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed exact-parent text-symbol delta gate =="
  python3 tools/check-hot-codegen.py "$XR819_PARENT_ELF" "$ELF"
else
  echo "== exact-parent hot-code gate skipped: set XR819_PARENT_ELF to the fixed-layout parent ELF =="
fi

if [[ -n "${XR819_LINK_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed link/sequence parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/link-sequence-codegen-manifest.json \
    "$XR819_LINK_PARENT_ELF" "$ELF"
else
  echo "== link/sequence codegen gate skipped: set XR819_LINK_PARENT_ELF to the qualified host-context ELF =="
fi

if [[ -n "${XR819_BA_LMC_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed BA/LMC/pending parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/ba-lmc-pending-codegen-manifest.json \
    "$XR819_BA_LMC_PARENT_ELF" "$ELF"
else
  echo "== BA/LMC/pending codegen gate skipped: set XR819_BA_LMC_PARENT_ELF to the qualified link-state ELF =="
fi

if [[ -n "${XR819_BA_SESSION_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed BA-session parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/ba-session-codegen-manifest.json \
    "$XR819_BA_SESSION_PARENT_ELF" "$ELF"
else
  echo "== BA-session codegen gate skipped: set XR819_BA_SESSION_PARENT_ELF to the qualified BA/LMC parent ELF =="
fi

if [[ -n "${XR819_BA_LINK_EVENT_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed BA/link/event parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/ba-link-event-codegen-manifest.json \
    "$XR819_BA_LINK_EVENT_PARENT_ELF" "$ELF"
else
  echo "== BA/link/event codegen gate skipped: set XR819_BA_LINK_EVENT_PARENT_ELF to the qualified BA-session ELF =="
fi

if [[ -n "${XR819_JOIN_SCAN_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed JOIN/scan parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/join-scan-codegen-manifest.json \
    "$XR819_JOIN_SCAN_PARENT_ELF" "$ELF"
else
  echo "== JOIN/scan codegen gate skipped: set XR819_JOIN_SCAN_PARENT_ELF to the qualified BA/link/event ELF =="
fi

if [[ -n "${XR819_COMMAND_CHANNEL_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed command/channel parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/command-channel-codegen-manifest.json \
    "$XR819_COMMAND_CHANNEL_PARENT_ELF" "$ELF"
else
  echo "== command/channel codegen gate skipped: set XR819_COMMAND_CHANNEL_PARENT_ELF to the qualified JOIN/scan ELF =="
fi

if [[ -n "${XR819_LMC_CONTROL_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed LMC-control parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/lmc-control-codegen-manifest.json \
    "$XR819_LMC_CONTROL_PARENT_ELF" "$ELF"
else
  echo "== LMC-control codegen gate skipped: set XR819_LMC_CONTROL_PARENT_ELF to the qualified command/channel ELF =="
fi

if [[ -n "${XR819_PEER_PIPE_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed peer-pipe parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/peer-pipe-codegen-manifest.json \
    "$XR819_PEER_PIPE_PARENT_ELF" "$ELF"
else
  echo "== peer-pipe codegen gate skipped: set XR819_PEER_PIPE_PARENT_ELF to the qualified LMC-control ELF =="
fi

if [[ -n "${XR819_SCHEDULER_EVENT_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed scheduler-event parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/scheduler-event-codegen-manifest.json \
    "$XR819_SCHEDULER_EVENT_PARENT_ELF" "$ELF"
else
  echo "== scheduler-event codegen gate skipped: set XR819_SCHEDULER_EVENT_PARENT_ELF to the qualified peer-pipe ELF =="
fi

if [[ -n "${XR819_SCHEDULER_SUPPORT_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed scheduler-support parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/scheduler-support-codegen-manifest.json \
    "$XR819_SCHEDULER_SUPPORT_PARENT_ELF" "$ELF"
else
  echo "== scheduler-support codegen gate skipped: set XR819_SCHEDULER_SUPPORT_PARENT_ELF to the qualified scheduler-event ELF =="
fi

if [[ -n "${XR819_VIF_TIMER_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed VIF-timer parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/vif-timer-codegen-manifest.json \
    "$XR819_VIF_TIMER_PARENT_ELF" "$ELF"
else
  echo "== VIF-timer codegen gate skipped: set XR819_VIF_TIMER_PARENT_ELF to the qualified scheduler-support ELF =="
fi

if [[ -n "${XR819_B6_ELF:-}" ]]; then
  echo "== qualified source and decoded-MMIO drift gate against clean b6 =="
  python3 tools/check-packet-ram-transition.py "$XR819_B6_ELF" "$ELF"
else
  echo "== clean-b6 source/MMIO drift gate skipped: set XR819_B6_ELF to archived clean-b6 ELF =="
fi

echo "== arm build: sectioned-image bootstrap =="
./tools/build-sectioned-bootloader.sh "$BOOTSTRAP"

echo "ALL CHECKS PASSED"
