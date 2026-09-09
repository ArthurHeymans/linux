#![no_std]

#[cfg(test)]
extern crate std;

pub(crate) mod backoff;
#[cfg(target_arch = "arm")]
pub mod command;
pub mod configuration;
pub mod crypto;
pub mod download;
pub mod dtcm;
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

/// Records a recoverable corruption report and allows execution to continue.
#[macro_export]
macro_rules! halt_unless_reporting {
    () => {};
}

pub mod host_tx_diagnostics;
#[cfg(target_arch = "arm")]
pub mod host_tx_driver;
// Deliberately not ARM-gated: `host_tx_driver` is invisible to host test runs,
// so its hardware-free decisions live here where they are always compiled.
pub mod host_tx_policy;
pub mod join;
pub mod loader;
pub mod mac;
pub mod mac_domain;
pub mod packet_ram;
pub mod phy;
pub mod platform;
pub mod radio;
pub mod rate_policy;
pub(crate) mod rx_model;
pub mod scan;
pub mod tcm;
pub mod tx;
pub mod vendor_host_tx;
pub mod vif;
pub mod wsm;
