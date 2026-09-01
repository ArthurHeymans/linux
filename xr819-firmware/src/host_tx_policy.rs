//! Host TX decisions that carry no hardware access.
//!
//! `host_tx_driver` is `#[cfg(target_arch = "arm")]`, so nothing in it is
//! compiled or tested by a host `cargo test` run. Decisions that can be
//! expressed without touching hardware live here instead, where they are
//! compiled and tested on every build, in the same spirit as
//! `tx::plan_ordinary_tx_pipe_status`.

/// First retained class-0 runtime owner observed for each MAC pipe.
///
/// Multiple contexts may legitimately share one pipe owner (a staged batch or
/// depth-two aggregate), so later observations of that pipe are coalesced. An
/// out-of-range pipe is rejected instead of being masked into another owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PipeRuntimeOwners {
    first: [Option<u8>; 4],
}

impl PipeRuntimeOwners {
    pub const fn new() -> Self {
        Self { first: [None; 4] }
    }

    pub fn observe(&mut self, index: usize, pipe: u8) -> bool {
        if pipe >= 4 || index > usize::from(u8::MAX) {
            return false;
        }
        let entry = &mut self.first[usize::from(pipe)];
        if entry.is_none() {
            *entry = Some(index as u8);
        }
        true
    }

    pub const fn contains(self, pipe: u8) -> bool {
        pipe < 4 && self.first[pipe as usize].is_some()
    }

    pub const fn first(self, pipe: usize) -> Option<usize> {
        if pipe < 4 {
            match self.first[pipe] {
                Some(index) => Some(index as usize),
                None => None,
            }
        } else {
            None
        }
    }

    pub fn is_empty(self) -> bool {
        self.first.iter().all(Option::is_none)
    }
}

impl Default for PipeRuntimeOwners {
    fn default() -> Self {
        Self::new()
    }
}

/// Match completion-ring evidence against the exact slot recorded before GO.
///
/// Context-only routing was measured and reverted because it could release a
/// buffer still referenced by the MAC. The completion drain now carries the
/// publication's pipe, slot, and frame-node identity; all four fields must
/// agree before host ownership can be released.
pub const fn completion_matches_owner(
    owner_context: u32,
    owner_frame_node: u32,
    owner_pipe: u8,
    owner_slot: u8,
    completion_context: u32,
    completion_frame_node: u32,
    completion_pipe: u8,
    completion_slot: u8,
) -> bool {
    owner_context == completion_context
        && owner_frame_node == completion_frame_node
        && owner_pipe == completion_pipe
        && owner_slot == completion_slot
}

/// Per-rate attempt nibbles for a frame transmitted at a single rate.
///
/// Mirrors `HostTxDriver::rate_try_for_single_rate`, kept here so the encoding
/// is testable on the host: `host_tx_driver` is ARM-only and invisible to
/// `cargo test`. The driver decodes 24 nibbles across 3 words as
/// `word = rate >> 3`, `nibble = rate & 7`.
pub const fn rate_try_for_single_rate(rate: u8, ack_failures: u8) -> [u32; 3] {
    let mut rate_try = [0_u32; 3];
    if rate < 24 {
        let attempts = {
            let raw = ack_failures as u32 + 1;
            if raw > 0xf { 0xf } else { raw }
        };
        rate_try[(rate >> 3) as usize] = attempts << ((rate as u32 & 7) * 4);
    }
    rate_try
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_try_encodes_one_attempt_at_the_transmitted_rate() {
        // Index 7 (OFDM 9 Mbit/s), transmitted once with no retries.
        assert_eq!(rate_try_for_single_rate(7, 0), [0x1000_0000, 0, 0]);
        // Index 8 starts the second word.
        assert_eq!(rate_try_for_single_rate(8, 0), [0, 0x0000_0001, 0]);
        // Index 13 (OFDM 54) with three retries: four attempts total.
        assert_eq!(rate_try_for_single_rate(13, 3), [0, 0x0040_0000, 0]);
    }

    #[test]
    fn rate_try_never_reports_zero_attempts_for_a_valid_rate() {
        // An all-zero set makes the driver record no attempts at all, which
        // starves minstrel_ht. Every valid rate must produce a nibble.
        for rate in 0..24_u8 {
            assert_ne!(rate_try_for_single_rate(rate, 0), [0; 3], "rate {rate}");
        }
        // The nibble saturates rather than wrapping to zero.
        assert_eq!(rate_try_for_single_rate(0, 200), [0xf, 0, 0]);
        // Out-of-range indices report nothing rather than corrupting a word.
        assert_eq!(rate_try_for_single_rate(24, 0), [0; 3]);
    }

    #[test]
    fn pipe_runtime_owners_coalesce_batches_but_keep_pipes_independent() {
        let mut owners = PipeRuntimeOwners::new();
        assert!(owners.observe(7, 2));
        assert!(owners.observe(9, 2));
        assert!(owners.observe(11, 0));
        assert_eq!(owners.first(0), Some(11));
        assert_eq!(owners.first(2), Some(7));
        assert_eq!(owners.first(1), None);
        assert!(owners.contains(0));
        assert!(!owners.contains(1));
        assert!(!owners.contains(4));
        assert!(!owners.is_empty());
        assert!(!owners.observe(12, 4));
        assert_eq!(PipeRuntimeOwners::new().first(4), None);
        assert!(PipeRuntimeOwners::new().is_empty());
    }

    #[test]
    fn completion_requires_context_pipe_slot_and_frame_node() {
        let owner = (0x0901_2340, 0x0901_2394, 2, 1);
        assert!(completion_matches_owner(
            owner.0, owner.1, owner.2, owner.3, owner.0, owner.1, owner.2, owner.3,
        ));
        assert!(!completion_matches_owner(
            owner.0,
            owner.1,
            owner.2,
            owner.3,
            0x0901_5580,
            owner.1,
            owner.2,
            owner.3,
        ));
        assert!(!completion_matches_owner(
            owner.0, owner.1, owner.2, owner.3, owner.0, owner.1, owner.2, 2,
        ));
        assert!(!completion_matches_owner(
            owner.0, owner.1, owner.2, owner.3, owner.0, owner.1, 3, owner.3,
        ));
        assert!(!completion_matches_owner(
            owner.0,
            owner.1,
            owner.2,
            owner.3,
            owner.0,
            0x0901_5594,
            owner.2,
            owner.3,
        ));
    }
}
