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
python3 tools/check-scheduler-event-layout.py
python3 tools/check-runtime-register-backoff-layout.py
python3 tools/check-debug-console-layout.py
python3 tools/check-scheduler-support-layout.py
python3 tools/check-phy-gain-source-layout.py
python3 tools/check-template-descriptor-layout.py
python3 tools/check-template-backing-layout.py
python3 tools/check-rf-initialization-view.py
python3 tools/check-beacon-ie-index-view.py
python3 tools/check-beacon-filter-storage-layout.py
python3 tools/check-sdd-profile-layout.py
python3 tools/check-wake-context-layout.py
python3 tools/check-vif-timer-layout.py
python3 tools/check-power-save-layout.py
python3 tools/check-hif-mic-layout.py
python3 tools/check-initialized-hif-control-layout.py
python3 tools/check-ampdu-telemetry-layout.py
python3 tools/check-initialized-control-words-layout.py
python3 tools/check-queue-pipe-mappings-layout.py
python3 tools/check-duration-quantum-pointers-layout.py
python3 tools/check-initialized-tx-rate-tables-layout.py
python3 tools/check-initialized-completion-words-layout.py
python3 tools/check-initialized-irq-callbacks-layout.py
python3 tools/check-host-pas-ring-layout.py
python3 tools/check-mac-pipe-records-layout.py
python3 tools/check-low-mac-global-prefix-layout.py
python3 tools/check-mac-beacon-state-layout.py
python3 tools/check-mac-wake-runtime-layout.py
python3 tools/check-mac-phy-command-state-layout.py
python3 tools/check-mac-runtime-accounting-layout.py
python3 tools/check-mac-retry-hardware-state-layout.py
python3 tools/check-mac-tx-queue-state-layout.py
python3 tools/check-initialized-rate-policies-layout.py
python3 tools/check-phy-descriptor-gain-records-layout.py
python3 tools/check-ampdu-completion-control-layout.py
python3 tools/check-phy-reference-layout.py
python3 tools/check-phy-profile-layout.py
python3 tools/check-phy-measurement-layout.py
python3 tools/check-phy-channel-cache-layout.py
python3 tools/check-phy-table-control-layout.py
python3 tools/check-phy-iq-calibration-layout.py
python3 tools/check-low-mac-pas-layout.py
python3 tools/check-pre-vif-header-layout.py
python3 tools/check-peer-pipe-layout.py
python3 tools/check-command-channel-overlay.py
python3 tools/check-lmc-control-layout.py
python3 tools/check-link-sequence-layout.py
python3 tools/check-join-scan-layout.py
python3 tools/check-ba-lmc-pending-layout.py
python3 tools/check-ba-session-layout.py
python3 tools/check-ba-link-event-layout.py
python3 tools/check-context-completion-layout.py
python3 tools/check-completion-ring-view.py
python3 tools/check-internal-context-prefix-layout.py
python3 tools/check-internal-context-layout.py
python3 tools/pack-sectioned-elf.py "$ELF" "$OUT"
python3 tools/check-dtcm-layout.py "$ELF" "$OUT"
python3 tools/check-scheduler-event-layout.py "$ELF"
python3 tools/check-runtime-register-backoff-layout.py "$ELF"
python3 tools/check-debug-console-layout.py "$ELF"
python3 tools/check-scheduler-support-layout.py "$ELF"
python3 tools/check-phy-gain-source-layout.py "$ELF"
python3 tools/check-template-descriptor-layout.py "$ELF"
python3 tools/check-template-backing-layout.py "$ELF"
python3 tools/check-rf-initialization-view.py "$ELF"
python3 tools/check-beacon-ie-index-view.py "$ELF"
python3 tools/check-beacon-filter-storage-layout.py "$ELF"
python3 tools/check-sdd-profile-layout.py "$ELF"
python3 tools/check-wake-context-layout.py "$ELF"
python3 tools/check-vif-timer-layout.py "$ELF"
python3 tools/check-power-save-layout.py "$ELF"
python3 tools/check-hif-mic-layout.py "$ELF"
python3 tools/check-initialized-hif-control-layout.py "$ELF"
python3 tools/check-ampdu-telemetry-layout.py "$ELF"
python3 tools/check-initialized-control-words-layout.py "$ELF"
python3 tools/check-queue-pipe-mappings-layout.py "$ELF"
python3 tools/check-duration-quantum-pointers-layout.py "$ELF"
python3 tools/check-initialized-tx-rate-tables-layout.py "$ELF"
python3 tools/check-initialized-completion-words-layout.py "$ELF"
python3 tools/check-initialized-irq-callbacks-layout.py "$ELF"
python3 tools/check-host-pas-ring-layout.py "$ELF"
python3 tools/check-mac-pipe-records-layout.py "$ELF"
python3 tools/check-low-mac-global-prefix-layout.py "$ELF"
python3 tools/check-mac-beacon-state-layout.py "$ELF"
python3 tools/check-mac-wake-runtime-layout.py "$ELF"
python3 tools/check-mac-phy-command-state-layout.py "$ELF"
python3 tools/check-mac-runtime-accounting-layout.py "$ELF"
python3 tools/check-mac-retry-hardware-state-layout.py "$ELF"
python3 tools/check-mac-tx-queue-state-layout.py "$ELF"
python3 tools/check-initialized-rate-policies-layout.py "$ELF"
python3 tools/check-phy-descriptor-gain-records-layout.py "$ELF"
python3 tools/check-ampdu-completion-control-layout.py "$ELF"
python3 tools/check-phy-reference-layout.py "$ELF"
python3 tools/check-phy-profile-layout.py "$ELF"
python3 tools/check-phy-measurement-layout.py "$ELF"
python3 tools/check-phy-channel-cache-layout.py "$ELF"
python3 tools/check-phy-table-control-layout.py "$ELF"
python3 tools/check-phy-iq-calibration-layout.py "$ELF"
python3 tools/check-low-mac-pas-layout.py "$ELF"
python3 tools/check-pre-vif-header-layout.py "$ELF"
python3 tools/check-peer-pipe-layout.py "$ELF"
python3 tools/check-command-channel-overlay.py "$ELF"
python3 tools/check-lmc-control-layout.py "$ELF"
python3 tools/check-link-sequence-layout.py "$ELF"
python3 tools/check-join-scan-layout.py "$ELF"
python3 tools/check-ba-lmc-pending-layout.py "$ELF"
python3 tools/check-ba-session-layout.py "$ELF"
python3 tools/check-ba-link-event-layout.py "$ELF"
python3 tools/check-context-completion-layout.py "$ELF"
python3 tools/check-completion-ring-view.py "$ELF"
python3 tools/check-internal-context-prefix-layout.py "$ELF"
python3 tools/check-internal-context-layout.py "$ELF"

echo "features=none"
