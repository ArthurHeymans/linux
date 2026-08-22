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
python3 tools/check-low-mac-pas-layout.py
python3 tools/check-pre-vif-header-layout.py
python3 tools/check-vif-layout.py
python3 tools/check-vif-timer-layout.py
python3 tools/check-power-save-layout.py
python3 tools/check-hif-mic-layout.py
python3 tools/check-initialized-hif-control-layout.py
python3 tools/check-ampdu-telemetry-layout.py
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
python3 tools/check-initialized-multi-vif-beacon-timer-layout.py
python3 tools/check-initialized-measurement-dwell-timer-layout.py
python3 tools/check-initialized-measurement-control-reset-words-layout.py
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
python3 tools/check-host-context-layout.py
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
python3 tools/test-pack-sectioned-elf.py

echo "== arm build and stack check: feature-free firmware =="
cargo +nightly build --release --bin hif-startup --target "$TARGET" "${BUILD_STD[@]}"
python3 tools/check-rust-main-stack.py "$ELF"
python3 tools/check-packet-ram-layout.py "$ELF"
python3 tools/pack-sectioned-elf.py "$ELF" "$PACKED"
python3 tools/check-dtcm-layout.py "$ELF" "$PACKED"
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
python3 tools/check-low-mac-pas-layout.py "$ELF"
python3 tools/check-pre-vif-header-layout.py "$ELF"
python3 tools/check-vif-layout.py "$ELF"
python3 tools/check-vif-timer-layout.py "$ELF"
python3 tools/check-power-save-layout.py "$ELF"
python3 tools/check-hif-mic-layout.py "$ELF"
python3 tools/check-initialized-hif-control-layout.py "$ELF"
python3 tools/check-ampdu-telemetry-layout.py "$ELF"
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
python3 tools/check-initialized-multi-vif-beacon-timer-layout.py "$ELF"
python3 tools/check-initialized-measurement-dwell-timer-layout.py "$ELF"
python3 tools/check-initialized-measurement-control-reset-words-layout.py "$ELF"
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
python3 tools/check-host-context-layout.py "$ELF"
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

if [[ -n "${XR819_POWER_SAVE_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed power-save parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/power-save-codegen-manifest.json \
    "$XR819_POWER_SAVE_PARENT_ELF" "$ELF"
else
  echo "== power-save codegen gate skipped: set XR819_POWER_SAVE_PARENT_ELF to the qualified VIF-timer ELF =="
fi

if [[ -n "${XR819_HIF_MIC_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed HIF/MIC parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/hif-mic-codegen-manifest.json \
    "$XR819_HIF_MIC_PARENT_ELF" "$ELF"
else
  echo "== HIF/MIC codegen gate skipped: set XR819_HIF_MIC_PARENT_ELF to the qualified power-save ELF =="
fi

if [[ -n "${XR819_PHY_REFERENCE_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed PHY-reference parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/phy-reference-codegen-manifest.json \
    "$XR819_PHY_REFERENCE_PARENT_ELF" "$ELF"
else
  echo "== PHY-reference codegen gate skipped: set XR819_PHY_REFERENCE_PARENT_ELF to the qualified HIF/MIC ELF =="
fi

if [[ -n "${XR819_PHY_PROFILE_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed PHY-profile parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/phy-profile-codegen-manifest.json \
    "$XR819_PHY_PROFILE_PARENT_ELF" "$ELF"
else
  echo "== PHY-profile codegen gate skipped: set XR819_PHY_PROFILE_PARENT_ELF to the qualified PHY-reference ELF =="
fi

if [[ -n "${XR819_PHY_MEASUREMENT_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed PHY-measurement parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/phy-measurement-codegen-manifest.json \
    "$XR819_PHY_MEASUREMENT_PARENT_ELF" "$ELF"
else
  echo "== PHY-measurement codegen gate skipped: set XR819_PHY_MEASUREMENT_PARENT_ELF to the qualified PHY-profile ELF =="
fi

if [[ -n "${XR819_PHY_CHANNEL_CACHE_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed PHY-channel-cache parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/phy-channel-cache-codegen-manifest.json \
    "$XR819_PHY_CHANNEL_CACHE_PARENT_ELF" "$ELF"
else
  echo "== PHY-channel-cache codegen gate skipped: set XR819_PHY_CHANNEL_CACHE_PARENT_ELF to the qualified PHY-measurement ELF =="
fi

if [[ -n "${XR819_PHY_TABLE_CONTROL_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed PHY-table-control parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/phy-table-control-codegen-manifest.json \
    "$XR819_PHY_TABLE_CONTROL_PARENT_ELF" "$ELF"
else
  echo "== PHY-table-control codegen gate skipped: set XR819_PHY_TABLE_CONTROL_PARENT_ELF to the qualified PHY-channel-cache ELF =="
fi

if [[ -n "${XR819_PHY_IQ_CALIBRATION_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed PHY-IQ-calibration parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/phy-iq-calibration-codegen-manifest.json \
    "$XR819_PHY_IQ_CALIBRATION_PARENT_ELF" "$ELF"
else
  echo "== PHY-IQ-calibration codegen gate skipped: set XR819_PHY_IQ_CALIBRATION_PARENT_ELF to the qualified PHY-table-control ELF =="
fi

if [[ -n "${XR819_RUNTIME_REGISTER_BACKOFF_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed runtime-register/backoff parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/runtime-register-backoff-codegen-manifest.json \
    "$XR819_RUNTIME_REGISTER_BACKOFF_PARENT_ELF" "$ELF"
else
  echo "== runtime-register/backoff codegen gate skipped: set XR819_RUNTIME_REGISTER_BACKOFF_PARENT_ELF to the qualified PHY-IQ-calibration ELF =="
fi

if [[ -n "${XR819_SDD_PROFILE_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed SDD-profile parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/sdd-profile-codegen-manifest.json \
    "$XR819_SDD_PROFILE_PARENT_ELF" "$ELF"
else
  echo "== SDD-profile codegen gate skipped: set XR819_SDD_PROFILE_PARENT_ELF to the qualified runtime-register/backoff ELF =="
fi

if [[ -n "${XR819_WAKE_CONTEXT_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed wake-context parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/wake-context-codegen-manifest.json \
    "$XR819_WAKE_CONTEXT_PARENT_ELF" "$ELF"
else
  echo "== wake-context codegen gate skipped: set XR819_WAKE_CONTEXT_PARENT_ELF to the qualified SDD-profile ELF =="
fi

if [[ -n "${XR819_PHY_GAIN_SOURCE_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed PHY-gain-source parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/phy-gain-source-codegen-manifest.json \
    "$XR819_PHY_GAIN_SOURCE_PARENT_ELF" "$ELF"
else
  echo "== PHY-gain-source codegen gate skipped: set XR819_PHY_GAIN_SOURCE_PARENT_ELF to the qualified wake-context ELF =="
fi

if [[ -n "${XR819_TEMPLATE_DESCRIPTOR_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed template-descriptor parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/template-descriptor-codegen-manifest.json \
    "$XR819_TEMPLATE_DESCRIPTOR_PARENT_ELF" "$ELF"
else
  echo "== template-descriptor codegen gate skipped: set XR819_TEMPLATE_DESCRIPTOR_PARENT_ELF to the qualified PHY-gain-source ELF =="
fi

if [[ -n "${XR819_DEBUG_CONSOLE_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed debug-console parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/debug-console-codegen-manifest.json \
    "$XR819_DEBUG_CONSOLE_PARENT_ELF" "$ELF"
else
  echo "== debug-console codegen gate skipped: set XR819_DEBUG_CONSOLE_PARENT_ELF to the qualified template-descriptor ELF =="
fi

if [[ -n "${XR819_CONTEXT_COMPLETION_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed context-completion parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/context-completion-codegen-manifest.json \
    "$XR819_CONTEXT_COMPLETION_PARENT_ELF" "$ELF"
else
  echo "== context-completion codegen gate skipped: set XR819_CONTEXT_COMPLETION_PARENT_ELF to the qualified debug-console ELF =="
fi

if [[ -n "${XR819_RF_INITIALIZATION_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed RF-initialization parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/rf-initialization-codegen-manifest.json \
    "$XR819_RF_INITIALIZATION_PARENT_ELF" "$ELF"
else
  echo "== RF-initialization codegen gate skipped: set XR819_RF_INITIALIZATION_PARENT_ELF to the qualified context-completion ELF =="
fi

if [[ -n "${XR819_BEACON_IE_INDEX_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed beacon-IE-index parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/beacon-ie-index-codegen-manifest.json \
    "$XR819_BEACON_IE_INDEX_PARENT_ELF" "$ELF"
else
  echo "== beacon-IE-index codegen gate skipped: set XR819_BEACON_IE_INDEX_PARENT_ELF to the qualified RF-initialization ELF =="
fi

if [[ -n "${XR819_BEACON_FILTER_STORAGE_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed beacon-filter-storage parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/beacon-filter-storage-codegen-manifest.json \
    "$XR819_BEACON_FILTER_STORAGE_PARENT_ELF" "$ELF"
else
  echo "== beacon-filter-storage codegen gate skipped: set XR819_BEACON_FILTER_STORAGE_PARENT_ELF to the qualified beacon-IE-index ELF =="
fi

if [[ -n "${XR819_TEMPLATE_BACKING_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed template-backing parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/template-backing-codegen-manifest.json \
    "$XR819_TEMPLATE_BACKING_PARENT_ELF" "$ELF"
else
  echo "== template-backing codegen gate skipped: set XR819_TEMPLATE_BACKING_PARENT_ELF to the qualified beacon-filter-storage ELF =="
fi

if [[ -n "${XR819_INTERNAL_CONTEXT_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed internal-context parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/internal-context-codegen-manifest.json \
    "$XR819_INTERNAL_CONTEXT_PARENT_ELF" "$ELF"
else
  echo "== internal-context codegen gate skipped: set XR819_INTERNAL_CONTEXT_PARENT_ELF to the qualified template-backing ELF =="
fi

if [[ -n "${XR819_COMPLETION_RING_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed completion-ring parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/completion-ring-codegen-manifest.json \
    "$XR819_COMPLETION_RING_PARENT_ELF" "$ELF"
else
  echo "== completion-ring codegen gate skipped: set XR819_COMPLETION_RING_PARENT_ELF to the qualified internal-PAS ELF =="
fi

if [[ -n "${XR819_INTERNAL_CONTEXT_PREFIX_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed internal-context-prefix parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/internal-context-prefix-codegen-manifest.json \
    "$XR819_INTERNAL_CONTEXT_PREFIX_PARENT_ELF" "$ELF"
else
  echo "== internal-context-prefix codegen gate skipped: set XR819_INTERNAL_CONTEXT_PREFIX_PARENT_ELF to the qualified completion-ring ELF =="
fi

if [[ -n "${XR819_PRE_VIF_HEADER_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed pre-VIF-header parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/pre-vif-header-codegen-manifest.json \
    "$XR819_PRE_VIF_HEADER_PARENT_ELF" "$ELF"
else
  echo "== pre-VIF-header codegen gate skipped: set XR819_PRE_VIF_HEADER_PARENT_ELF to the qualified power-save-boundary ELF =="
fi

if [[ -n "${XR819_INITIALIZED_HIF_CONTROL_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed initialized-HIF-control parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/initialized-hif-control-codegen-manifest.json \
    "$XR819_INITIALIZED_HIF_CONTROL_PARENT_ELF" "$ELF"
else
  echo "== initialized-HIF-control codegen gate skipped: set XR819_INITIALIZED_HIF_CONTROL_PARENT_ELF to the qualified power-save-tail ELF =="
fi

if [[ -n "${XR819_AMPDU_TELEMETRY_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed A-MPDU-telemetry parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/ampdu-telemetry-codegen-manifest.json \
    "$XR819_AMPDU_TELEMETRY_PARENT_ELF" "$ELF"
else
  echo "== A-MPDU-telemetry codegen gate skipped: set XR819_AMPDU_TELEMETRY_PARENT_ELF to the qualified initialized-HIF-control ELF =="
fi

if [[ -n "${XR819_INITIALIZED_PER_TID_TELEMETRY_BANK_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed initialized-per-TID-telemetry-bank parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/initialized-per-tid-telemetry-bank-codegen-manifest.json \
    "$XR819_INITIALIZED_PER_TID_TELEMETRY_BANK_PARENT_ELF" "$ELF"
else
  echo "== initialized-per-TID-telemetry-bank codegen gate skipped: set XR819_INITIALIZED_PER_TID_TELEMETRY_BANK_PARENT_ELF to the exact parent ELF =="
fi

if [[ -n "${XR819_INITIALIZED_TX_CONFIRM_AGGREGATION_STATE_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed initialized-TX-confirm-aggregation-state parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/initialized-tx-confirm-aggregation-state-codegen-manifest.json \
    "$XR819_INITIALIZED_TX_CONFIRM_AGGREGATION_STATE_PARENT_ELF" "$ELF"
else
  echo "== initialized-TX-confirm-aggregation-state codegen gate skipped: set XR819_INITIALIZED_TX_CONFIRM_AGGREGATION_STATE_PARENT_ELF to the exact parent ELF =="
fi

if [[ -n "${XR819_INITIALIZED_CONFIGURATION_APPLY_FLAGS_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed initialized-configuration-apply-flags parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/initialized-configuration-apply-flags-codegen-manifest.json \
    "$XR819_INITIALIZED_CONFIGURATION_APPLY_FLAGS_PARENT_ELF" "$ELF"
else
  echo "== initialized-configuration-apply-flags codegen gate skipped: set XR819_INITIALIZED_CONFIGURATION_APPLY_FLAGS_PARENT_ELF to the exact parent ELF =="
fi

if [[ -n "${XR819_INITIALIZED_CONTROL_WORDS_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed initialized-control-words parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/initialized-control-words-codegen-manifest.json \
    "$XR819_INITIALIZED_CONTROL_WORDS_PARENT_ELF" "$ELF"
else
  echo "== initialized-control-words codegen gate skipped: set XR819_INITIALIZED_CONTROL_WORDS_PARENT_ELF to the qualified A-MPDU-telemetry ELF =="
fi

if [[ -n "${XR819_QUEUE_PIPE_MAPPINGS_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed queue/pipe-mappings parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/queue-pipe-mappings-codegen-manifest.json \
    "$XR819_QUEUE_PIPE_MAPPINGS_PARENT_ELF" "$ELF"
else
  echo "== queue/pipe-mappings codegen gate skipped: set XR819_QUEUE_PIPE_MAPPINGS_PARENT_ELF to the qualified initialized-control-words ELF =="
fi

if [[ -n "${XR819_DURATION_QUANTUM_POINTERS_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed duration-quantum-pointers parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/duration-quantum-pointers-codegen-manifest.json \
    "$XR819_DURATION_QUANTUM_POINTERS_PARENT_ELF" "$ELF"
else
  echo "== duration-quantum-pointers codegen gate skipped: set XR819_DURATION_QUANTUM_POINTERS_PARENT_ELF to the qualified queue/pipe-mappings ELF =="
fi

if [[ -n "${XR819_INITIALIZED_TX_RATE_TABLES_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed initialized-TX-rate-tables parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/initialized-tx-rate-tables-codegen-manifest.json \
    "$XR819_INITIALIZED_TX_RATE_TABLES_PARENT_ELF" "$ELF"
else
  echo "== initialized-TX-rate-tables codegen gate skipped: set XR819_INITIALIZED_TX_RATE_TABLES_PARENT_ELF to the qualified duration-quantum-pointers ELF =="
fi

if [[ -n "${XR819_INITIALIZED_COMPLETION_WORDS_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed initialized-completion-words parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/initialized-completion-words-codegen-manifest.json \
    "$XR819_INITIALIZED_COMPLETION_WORDS_PARENT_ELF" "$ELF"
else
  echo "== initialized-completion-words codegen gate skipped: set XR819_INITIALIZED_COMPLETION_WORDS_PARENT_ELF to the qualified initialized-TX-rate-tables ELF =="
fi

if [[ -n "${XR819_INITIALIZED_IRQ_CALLBACKS_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed initialized-IRQ-callbacks parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/initialized-irq-callbacks-codegen-manifest.json \
    "$XR819_INITIALIZED_IRQ_CALLBACKS_PARENT_ELF" "$ELF"
else
  echo "== initialized-IRQ-callbacks codegen gate skipped: set XR819_INITIALIZED_IRQ_CALLBACKS_PARENT_ELF to the qualified initialized-completion-words ELF =="
fi

if [[ -n "${XR819_INITIALIZED_PHY_WATCHDOG_COUNTER_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed initialized-PHY-watchdog-counter parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/initialized-phy-watchdog-counter-codegen-manifest.json \
    "$XR819_INITIALIZED_PHY_WATCHDOG_COUNTER_PARENT_ELF" "$ELF"
else
  echo "== initialized-PHY-watchdog-counter codegen gate skipped: set XR819_INITIALIZED_PHY_WATCHDOG_COUNTER_PARENT_ELF to the qualified current parent ELF =="
fi

if [[ -n "${XR819_INITIALIZED_MULTI_VIF_BEACON_TIMER_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed initialized-multi-VIF-beacon-timer parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/initialized-multi-vif-beacon-timer-codegen-manifest.json \
    "$XR819_INITIALIZED_MULTI_VIF_BEACON_TIMER_PARENT_ELF" "$ELF"
else
  echo "== initialized-multi-VIF-beacon-timer codegen gate skipped: set XR819_INITIALIZED_MULTI_VIF_BEACON_TIMER_PARENT_ELF to the qualified current parent ELF =="
fi

if [[ -n "${XR819_INITIALIZED_MEASUREMENT_DWELL_TIMER_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed initialized-measurement-dwell-timer parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/initialized-measurement-dwell-timer-codegen-manifest.json \
    "$XR819_INITIALIZED_MEASUREMENT_DWELL_TIMER_PARENT_ELF" "$ELF"
else
  echo "== initialized-measurement-dwell-timer codegen gate skipped: set XR819_INITIALIZED_MEASUREMENT_DWELL_TIMER_PARENT_ELF to the qualified current parent ELF =="
fi

if [[ -n "${XR819_INITIALIZED_MEASUREMENT_CONTROL_RESET_WORDS_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed initialized-measurement-control-reset-words parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/initialized-measurement-control-reset-words-codegen-manifest.json \
    "$XR819_INITIALIZED_MEASUREMENT_CONTROL_RESET_WORDS_PARENT_ELF" "$ELF"
else
  echo "== initialized-measurement-control-reset-words codegen gate skipped: set XR819_INITIALIZED_MEASUREMENT_CONTROL_RESET_WORDS_PARENT_ELF to the qualified current parent ELF =="
fi

if [[ -n "${XR819_HOST_PAS_RING_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed host-PAS-ring parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/host-pas-ring-codegen-manifest.json \
    "$XR819_HOST_PAS_RING_PARENT_ELF" "$ELF"
else
  echo "== host-PAS-ring codegen gate skipped: set XR819_HOST_PAS_RING_PARENT_ELF to the qualified initialized-IRQ-callbacks ELF =="
fi

if [[ -n "${XR819_MAC_PIPE_RECORDS_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed MAC-pipe-records parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/mac-pipe-records-codegen-manifest.json \
    "$XR819_MAC_PIPE_RECORDS_PARENT_ELF" "$ELF"
else
  echo "== MAC-pipe-records codegen gate skipped: set XR819_MAC_PIPE_RECORDS_PARENT_ELF to the qualified host-PAS-ring ELF =="
fi

if [[ -n "${XR819_LOW_MAC_GLOBAL_PREFIX_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed low-MAC-global-prefix parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/low-mac-global-prefix-codegen-manifest.json \
    "$XR819_LOW_MAC_GLOBAL_PREFIX_PARENT_ELF" "$ELF"
else
  echo "== low-MAC-global-prefix codegen gate skipped: set XR819_LOW_MAC_GLOBAL_PREFIX_PARENT_ELF to the qualified MAC-pipe-records ELF =="
fi

if [[ -n "${XR819_MAC_BEACON_STATE_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed MAC-beacon-state parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/mac-beacon-state-codegen-manifest.json \
    "$XR819_MAC_BEACON_STATE_PARENT_ELF" "$ELF"
else
  echo "== MAC-beacon-state codegen gate skipped: set XR819_MAC_BEACON_STATE_PARENT_ELF to the qualified low-MAC-global-prefix ELF =="
fi

if [[ -n "${XR819_MAC_WAKE_RUNTIME_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed MAC-wake-runtime parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/mac-wake-runtime-codegen-manifest.json \
    "$XR819_MAC_WAKE_RUNTIME_PARENT_ELF" "$ELF"
else
  echo "== MAC-wake-runtime codegen gate skipped: set XR819_MAC_WAKE_RUNTIME_PARENT_ELF to the qualified MAC-beacon-state ELF =="
fi

if [[ -n "${XR819_MAC_PHY_COMMAND_STATE_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed MAC-PHY-command-state parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/mac-phy-command-state-codegen-manifest.json \
    "$XR819_MAC_PHY_COMMAND_STATE_PARENT_ELF" "$ELF"
else
  echo "== MAC-PHY-command-state codegen gate skipped: set XR819_MAC_PHY_COMMAND_STATE_PARENT_ELF to the qualified MAC-wake-runtime ELF =="
fi

if [[ -n "${XR819_MAC_RUNTIME_ACCOUNTING_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed MAC-runtime-accounting parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/mac-runtime-accounting-codegen-manifest.json \
    "$XR819_MAC_RUNTIME_ACCOUNTING_PARENT_ELF" "$ELF"
else
  echo "== MAC-runtime-accounting codegen gate skipped: set XR819_MAC_RUNTIME_ACCOUNTING_PARENT_ELF to the qualified MAC-PHY-command-state ELF =="
fi

if [[ -n "${XR819_MAC_RETRY_HARDWARE_STATE_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed MAC-retry-hardware-state parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/mac-retry-hardware-state-codegen-manifest.json \
    "$XR819_MAC_RETRY_HARDWARE_STATE_PARENT_ELF" "$ELF"
else
  echo "== MAC-retry-hardware-state codegen gate skipped: set XR819_MAC_RETRY_HARDWARE_STATE_PARENT_ELF to the qualified MAC-runtime-accounting ELF =="
fi

if [[ -n "${XR819_MAC_TX_QUEUE_STATE_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed MAC-TX-queue-state parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/mac-tx-queue-state-codegen-manifest.json \
    "$XR819_MAC_TX_QUEUE_STATE_PARENT_ELF" "$ELF"
else
  echo "== MAC-TX-queue-state codegen gate skipped: set XR819_MAC_TX_QUEUE_STATE_PARENT_ELF to the qualified MAC-retry-hardware-state ELF =="
fi

if [[ -n "${XR819_INITIALIZED_RATE_POLICIES_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed initialized-rate-policies parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/initialized-rate-policies-codegen-manifest.json \
    "$XR819_INITIALIZED_RATE_POLICIES_PARENT_ELF" "$ELF"
else
  echo "== initialized-rate-policies codegen gate skipped: set XR819_INITIALIZED_RATE_POLICIES_PARENT_ELF to the qualified MAC-TX-queue-state ELF =="
fi

if [[ -n "${XR819_PHY_DESCRIPTOR_GAIN_RECORDS_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed PHY-descriptor-gain-records parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/phy-descriptor-gain-records-codegen-manifest.json \
    "$XR819_PHY_DESCRIPTOR_GAIN_RECORDS_PARENT_ELF" "$ELF"
else
  echo "== PHY-descriptor-gain-records codegen gate skipped: set XR819_PHY_DESCRIPTOR_GAIN_RECORDS_PARENT_ELF to the qualified initialized-rate-policies ELF =="
fi

if [[ -n "${XR819_AMPDU_COMPLETION_CONTROL_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed A-MPDU-completion-control parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/ampdu-completion-control-codegen-manifest.json \
    "$XR819_AMPDU_COMPLETION_CONTROL_PARENT_ELF" "$ELF"
else
  echo "== A-MPDU-completion-control codegen gate skipped: set XR819_AMPDU_COMPLETION_CONTROL_PARENT_ELF to the qualified PHY-descriptor-gain-records ELF =="
fi

if [[ -n "${XR819_PHY_INIT_REGISTER_WRITE_LISTS_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed PHY-init-register-write-lists parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/phy-init-register-write-lists-codegen-manifest.json \
    "$XR819_PHY_INIT_REGISTER_WRITE_LISTS_PARENT_ELF" "$ELF"
else
  echo "== PHY-init-register-write-lists codegen gate skipped: set XR819_PHY_INIT_REGISTER_WRITE_LISTS_PARENT_ELF to the qualified PHY-gain-register-write-lists ELF =="
fi

if [[ -n "${XR819_INITIALIZED_PHY_GAIN_SOURCE_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed initialized-PHY-gain-source parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/initialized-phy-gain-source-codegen-manifest.json \
    "$XR819_INITIALIZED_PHY_GAIN_SOURCE_PARENT_ELF" "$ELF"
else
  echo "== initialized-PHY-gain-source codegen gate skipped: set XR819_INITIALIZED_PHY_GAIN_SOURCE_PARENT_ELF to the qualified PHY-init-register-write-lists ELF =="
fi

if [[ -n "${XR819_INITIALIZED_IQ_CALIBRATION_GAIN_INDICES_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed initialized-IQ-calibration-gain-indices parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/initialized-iq-calibration-gain-indices-codegen-manifest.json \
    "$XR819_INITIALIZED_IQ_CALIBRATION_GAIN_INDICES_PARENT_ELF" "$ELF"
else
  echo "== initialized-IQ-calibration-gain-indices codegen gate skipped: set XR819_INITIALIZED_IQ_CALIBRATION_GAIN_INDICES_PARENT_ELF to the qualified initialized-PHY-gain-source ELF =="
fi

if [[ -n "${XR819_MEASUREMENT_WORKSPACE_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed measurement-workspace parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/measurement-workspace-codegen-manifest.json \
    "$XR819_MEASUREMENT_WORKSPACE_PARENT_ELF" "$ELF"
else
  echo "== measurement-workspace codegen gate skipped: set XR819_MEASUREMENT_WORKSPACE_PARENT_ELF to the qualified initialized-IQ-gain-index ELF =="
fi

if [[ -n "${XR819_TX_AGGREGATE_EXPIRATION_DELTA_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed TX-aggregate-expiration-delta parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/tx-aggregate-expiration-delta-codegen-manifest.json \
    "$XR819_TX_AGGREGATE_EXPIRATION_DELTA_PARENT_ELF" "$ELF"
else
  echo "== TX-aggregate-expiration-delta codegen gate skipped: set XR819_TX_AGGREGATE_EXPIRATION_DELTA_PARENT_ELF to the qualified measurement-workspace ELF =="
fi

if [[ -n "${XR819_INITIALIZED_DEBUG_COMMAND_DESCRIPTORS_PARENT_ELF:-}" ]]; then
  echo "== complete reviewed initialized-debug-command-descriptors parent text-symbol gate =="
  python3 tools/check-hot-codegen.py \
    --manifest tools/initialized-debug-command-descriptors-codegen-manifest.json \
    "$XR819_INITIALIZED_DEBUG_COMMAND_DESCRIPTORS_PARENT_ELF" "$ELF"
else
  echo "== initialized-debug-command-descriptors codegen gate skipped: set XR819_INITIALIZED_DEBUG_COMMAND_DESCRIPTORS_PARENT_ELF to the qualified TX-aggregate-expiration-delta ELF =="
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
