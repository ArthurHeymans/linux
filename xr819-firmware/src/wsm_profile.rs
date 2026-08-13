//! Compile-time WSM wire-profile selection.

#[cfg(all(feature = "wsm-cw1200-compat", feature = "wsm-xr819-native"))]
compile_error!("select only one WSM wire profile");

/// Native mode is opt-in. Builds without an explicit profile retain the
/// established CW1200-compatible behavior for existing diagnostic commands.
pub const CW1200_COMPATIBLE: bool = !cfg!(feature = "wsm-xr819-native");

pub const STARTUP_LABEL: &[u8] = if CW1200_COMPATIBLE {
    b"XR819 open Rust WSM"
} else {
    b"XR819 open Rust native"
};
