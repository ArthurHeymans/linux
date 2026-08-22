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
python3 tools/check-initialized-retry-path-counter-layout.py
python3 tools/check-initialized-per-tid-telemetry-bank-layout.py
python3 tools/check-initialized-tx-confirm-aggregation-state-layout.py
python3 tools/check-initialized-configuration-apply-flags-layout.py
python3 tools/check-initialized-control-words-layout.py
python3 tools/check-queue-pipe-mappings-layout.py
python3 tools/check-duration-quantum-pointers-layout.py
python3 tools/check-initialized-tx-rate-tables-layout.py
python3 tools/check-initialized-completion-words-layout.py
python3 tools/check-tkip-sbox-layout.py
python3 tools/check-aes-transfer-class-layout.py
python3 tools/check-phy-gain-register-write-lists-layout.py
python3 tools/check-phy-init-register-write-lists-layout.py
python3 tools/check-initialized-phy-gain-source-layout.py
python3 tools/check-initialized-iq-calibration-gain-indices-layout.py
python3 tools/check-measurement-workspace-layout.py
python3 tools/check-tx-aggregate-expiration-delta-layout.py
python3 tools/check-initialized-debug-command-descriptors-layout.py
python3 tools/check-initialized-irq-callbacks-layout.py
python3 tools/check-initialized-phy-watchdog-counter-layout.py
python3 tools/check-initialized-mac-aggregate-slot-tables-layout.py
python3 tools/check-initialized-mac-pipe-tails-layout.py
python3 tools/check-initialized-rf-mode-halfword-table-layout.py
python3 tools/check-initialized-register-write-lists-layout.py
python3 tools/check-initialized-rf-scale-tables-layout.py
python3 tools/check-initialized-tx-gain-rssi-table-layout.py
python3 tools/check-initialized-rate-pair-table-layout.py
python3 tools/check-initialized-scheduler-tail-layout.py
python3 tools/check-initialized-interface-2-radio-latch-layout.py
python3 tools/check-initialized-multi-vif-beacon-timer-layout.py
python3 tools/check-initialized-measurement-dwell-timer-layout.py
python3 tools/check-initialized-dtim-capture-latch-layout.py
python3 tools/check-initialized-measurement-control-reset-words-layout.py
python3 tools/check-initialized-pre-host-pas-radio-stop-word-layout.py
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
python3 tools/check-initialized-debug-platform-local-tail-layout.py
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
python3 tools/check-initialized-retry-path-counter-layout.py "$ELF"
python3 tools/check-initialized-per-tid-telemetry-bank-layout.py "$ELF"
python3 tools/check-initialized-tx-confirm-aggregation-state-layout.py "$ELF"
python3 tools/check-initialized-configuration-apply-flags-layout.py "$ELF"
python3 tools/check-initialized-control-words-layout.py "$ELF"
python3 tools/check-queue-pipe-mappings-layout.py "$ELF"
python3 tools/check-duration-quantum-pointers-layout.py "$ELF"
python3 tools/check-initialized-tx-rate-tables-layout.py "$ELF"
python3 tools/check-initialized-completion-words-layout.py "$ELF"
python3 tools/check-tkip-sbox-layout.py "$ELF"
python3 tools/check-aes-transfer-class-layout.py "$ELF"
python3 tools/check-phy-gain-register-write-lists-layout.py "$ELF"
python3 tools/check-phy-init-register-write-lists-layout.py "$ELF"
python3 tools/check-initialized-phy-gain-source-layout.py "$ELF"
python3 tools/check-initialized-iq-calibration-gain-indices-layout.py "$ELF"
python3 tools/check-measurement-workspace-layout.py "$ELF"
python3 tools/check-tx-aggregate-expiration-delta-layout.py "$ELF"
python3 tools/check-initialized-debug-command-descriptors-layout.py "$ELF"
python3 tools/check-initialized-irq-callbacks-layout.py "$ELF"
python3 tools/check-initialized-phy-watchdog-counter-layout.py "$ELF"
python3 tools/check-initialized-mac-aggregate-slot-tables-layout.py "$ELF"
python3 tools/check-initialized-mac-pipe-tails-layout.py "$ELF"
python3 tools/check-initialized-rf-mode-halfword-table-layout.py "$ELF"
python3 tools/check-initialized-register-write-lists-layout.py "$ELF"
python3 tools/check-initialized-rf-scale-tables-layout.py "$ELF"
python3 tools/check-initialized-tx-gain-rssi-table-layout.py "$ELF"
python3 tools/check-initialized-rate-pair-table-layout.py "$ELF"
python3 tools/check-initialized-scheduler-tail-layout.py "$ELF"
python3 tools/check-initialized-interface-2-radio-latch-layout.py "$ELF"
python3 tools/check-initialized-multi-vif-beacon-timer-layout.py "$ELF"
python3 tools/check-initialized-measurement-dwell-timer-layout.py "$ELF"
python3 tools/check-initialized-dtim-capture-latch-layout.py "$ELF"
python3 tools/check-initialized-measurement-control-reset-words-layout.py "$ELF"
python3 tools/check-initialized-pre-host-pas-radio-stop-word-layout.py "$ELF"
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
python3 tools/check-initialized-debug-platform-local-tail-layout.py "$ELF"
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

if [[ -n "${XR819_INITIALIZED_INTERFACE_2_RADIO_LATCH_PARENT_ELF:-}" ]]; then
  echo "== supplemental initialized-interface-2-radio-latch exact-parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/initialized-interface-2-radio-latch-codegen-manifest.json \
    "$XR819_INITIALIZED_INTERFACE_2_RADIO_LATCH_PARENT_ELF" "$ELF"
else
  echo "== initialized-interface-2-radio-latch codegen gate skipped: set XR819_INITIALIZED_INTERFACE_2_RADIO_LATCH_PARENT_ELF to the exact parent ELF; this supplemental symbol gate does not replace complete-file identity =="
fi

if [[ -n "${XR819_INITIALIZED_SCHEDULER_TAIL_PARENT_ELF:-}" ]]; then
  echo "== supplemental initialized-scheduler-tail exact-parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/initialized-scheduler-tail-codegen-manifest.json \
    "$XR819_INITIALIZED_SCHEDULER_TAIL_PARENT_ELF" "$ELF"
else
  echo "== initialized-scheduler-tail codegen gate skipped: set XR819_INITIALIZED_SCHEDULER_TAIL_PARENT_ELF to the exact parent ELF; this supplemental symbol gate does not replace complete-file identity =="
fi

if [[ -n "${XR819_INITIALIZED_RATE_PAIR_TABLE_PARENT_ELF:-}" ]]; then
  echo "== supplemental initialized-rate-pair-table exact-parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/initialized-rate-pair-table-codegen-manifest.json \
    "$XR819_INITIALIZED_RATE_PAIR_TABLE_PARENT_ELF" "$ELF"
else
  echo "== initialized-rate-pair-table codegen gate skipped: set XR819_INITIALIZED_RATE_PAIR_TABLE_PARENT_ELF to the exact parent ELF; this supplemental symbol gate does not replace complete-file identity =="
fi

if [[ -n "${XR819_TX_GAIN_RSSI_TABLE_PARENT_ELF:-}" ]]; then
  echo "== supplemental initialized-tx-gain-rssi-table exact-parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/initialized-tx-gain-rssi-table-codegen-manifest.json \
    "$XR819_TX_GAIN_RSSI_TABLE_PARENT_ELF" "$ELF"
else
  echo "== initialized-tx-gain-rssi-table codegen gate skipped: set XR819_TX_GAIN_RSSI_TABLE_PARENT_ELF to the exact parent ELF; this supplemental symbol gate does not replace complete-file identity =="
fi

if [[ -n "${XR819_RF_SCALE_TABLES_PARENT_ELF:-}" ]]; then
  echo "== supplemental initialized-rf-scale-tables exact-parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/initialized-rf-scale-tables-codegen-manifest.json \
    "$XR819_RF_SCALE_TABLES_PARENT_ELF" "$ELF"
else
  echo "== initialized-rf-scale-tables codegen gate skipped: set XR819_RF_SCALE_TABLES_PARENT_ELF to the exact parent ELF; this supplemental symbol gate does not replace complete-file identity =="
fi

if [[ -n "${XR819_REGISTER_WRITE_LISTS_PARENT_ELF:-}" ]]; then
  echo "== supplemental initialized-register-write-lists exact-parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/initialized-register-write-lists-codegen-manifest.json \
    "$XR819_REGISTER_WRITE_LISTS_PARENT_ELF" "$ELF"
else
  echo "== initialized-register-write-lists codegen gate skipped: set XR819_REGISTER_WRITE_LISTS_PARENT_ELF to the exact parent ELF; this supplemental symbol gate does not replace complete-file identity =="
fi

if [[ -n "${XR819_INITIALIZED_RF_MODE_HALFWORD_TABLE_PARENT_ELF:-}" ]]; then
  echo "== supplemental initialized-RF-mode-halfword-table exact-parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/initialized-rf-mode-halfword-table-codegen-manifest.json \
    "$XR819_INITIALIZED_RF_MODE_HALFWORD_TABLE_PARENT_ELF" "$ELF"
else
  echo "== initialized-RF-mode-halfword-table codegen gate skipped: set XR819_INITIALIZED_RF_MODE_HALFWORD_TABLE_PARENT_ELF to the exact parent ELF; this supplemental symbol gate does not replace complete-file identity =="
fi

if [[ -n "${XR819_INITIALIZED_MAC_PIPE_TAILS_PARENT_ELF:-}" ]]; then
  echo "== supplemental initialized-MAC-pipe-tails exact-parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/initialized-mac-pipe-tails-codegen-manifest.json \
    "$XR819_INITIALIZED_MAC_PIPE_TAILS_PARENT_ELF" "$ELF"
else
  echo "== initialized-MAC-pipe-tails codegen gate skipped: set XR819_INITIALIZED_MAC_PIPE_TAILS_PARENT_ELF to the exact parent ELF; this supplemental symbol gate does not replace complete-file identity =="
fi

if [[ -n "${XR819_INITIALIZED_MAC_AGGREGATE_SLOT_TABLES_PARENT_ELF:-}" ]]; then
  echo "== supplemental initialized-MAC-aggregate-slot-tables exact-parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/initialized-mac-aggregate-slot-tables-codegen-manifest.json \
    "$XR819_INITIALIZED_MAC_AGGREGATE_SLOT_TABLES_PARENT_ELF" "$ELF"
else
  echo "== initialized-MAC-aggregate-slot-tables codegen gate skipped: set XR819_INITIALIZED_MAC_AGGREGATE_SLOT_TABLES_PARENT_ELF to the exact parent ELF; this supplemental symbol gate does not replace complete-file identity =="
fi

if [[ -n "${XR819_INITIALIZED_DTIM_CAPTURE_LATCH_PARENT_ELF:-}" ]]; then
  echo "== supplemental initialized-DTIM-capture-latch exact-parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/initialized-dtim-capture-latch-codegen-manifest.json \
    "$XR819_INITIALIZED_DTIM_CAPTURE_LATCH_PARENT_ELF" "$ELF"
else
  echo "== initialized-DTIM-capture-latch codegen gate skipped: set XR819_INITIALIZED_DTIM_CAPTURE_LATCH_PARENT_ELF to the exact parent ELF; this supplemental symbol gate does not replace complete-file identity =="
fi

if [[ -n "${XR819_INITIALIZED_PRE_HOST_PAS_RADIO_STOP_WORD_PARENT_ELF:-}" ]]; then
  echo "== supplemental initialized-pre-host-PAS-radio-stop-word exact-parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/initialized-pre-host-pas-radio-stop-word-codegen-manifest.json \
    "$XR819_INITIALIZED_PRE_HOST_PAS_RADIO_STOP_WORD_PARENT_ELF" "$ELF"
else
  echo "== initialized-pre-host-PAS-radio-stop-word codegen gate skipped: set XR819_INITIALIZED_PRE_HOST_PAS_RADIO_STOP_WORD_PARENT_ELF to the exact parent ELF; this supplemental symbol gate does not replace complete-file identity =="
fi

echo "features=none"
