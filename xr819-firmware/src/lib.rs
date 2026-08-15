#![no_std]

pub mod configuration;
pub mod crypto;
pub mod download;
pub mod exception;
pub mod hif;
pub mod host_tx_arena;
/// Halts in **every** build, including report-and-continue ones.
///
/// Use where continuing past the detected condition would corrupt state the
/// firmware cannot repair. Two measured examples: returning instead of halting
/// in `radio::release()` when a slot falls outside the FIFO stalls the release
/// head permanently and left 3 of 4 runs unable to associate; skipping the halt
/// in `publish_host_class0_slot` let execution proceed with an unusable pipe and
/// stopped frame acceptance outright (`pending` 20, `used` 0, `TXed` 4).
#[macro_export]
macro_rules! halt_always {
    () => {
        loop {
            core::hint::spin_loop();
        }
    };
}

/// Halts in halting builds; under `corruption-non-fatal` records and continues.
///
/// Use only where the firmware can genuinely carry on. Note the failure mode
/// this exists to prevent: `publish_terminal_exception` returns immediately
/// under `corruption-non-fatal`, so a site that publishes and *then* spins
/// unconditionally goes silent — the host receives no exception while the
/// firmware stops, `TXed` freezes and the driver waits on buffers that never
/// come back. That is strictly worse than halting loudly.
#[macro_export]
macro_rules! halt_unless_reporting {
    () => {
        #[cfg(not(feature = "corruption-non-fatal"))]
        $crate::halt_always!();
    };
}

pub mod host_tx_diagnostics;
#[cfg(all(feature = "vendor-host-tx-foundation", target_arch = "arm"))]
pub mod host_tx_driver;
// Deliberately not ARM-gated: `host_tx_driver` is invisible to host test runs,
// so its hardware-free decisions live here where they are always compiled.
pub mod host_tx_policy;
pub mod join;
pub mod loader;
pub mod mac;
pub mod phy;
pub mod platform;
pub mod radio;
pub mod rate_policy;
pub mod scan;
pub mod tcm;
pub mod tx;
pub mod vendor_host_tx;
pub mod vif;
pub mod wsm;
pub mod wsm_profile;
