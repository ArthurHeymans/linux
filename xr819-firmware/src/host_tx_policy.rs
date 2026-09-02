//! Host TX decisions that carry no hardware access.
//!
//! `host_tx_driver` is `#[cfg(target_arch = "arm")]`, so nothing in it is
//! compiled or tested by a host `cargo test` run. Decisions that can be
//! expressed without touching hardware live here instead, where they are
//! compiled and tested on every build, in the same spirit as
//! `tx::plan_ordinary_tx_pipe_status`.

/// Retained class-0 runtime owners observed at exact MAC pipe-slot identities.
///
/// Multiple contexts may legitimately share one slot owner for a depth-two
/// aggregate. Distinct staged batch slots remain independently visible;
/// `slot_mask` and `next_slot_owner` provide bounded round-robin service, while
/// `first_in_pipe` preserves the pipe-level publication gate. Out-of-range
/// identities are rejected instead of being masked.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Class0RuntimeOwners {
    first_by_pipe: [Option<u8>; 4],
    by_slot: [Option<u8>; 16],
}

impl Class0RuntimeOwners {
    pub const fn new() -> Self {
        Self {
            first_by_pipe: [None; 4],
            by_slot: [None; 16],
        }
    }

    pub fn observe(&mut self, index: usize, pipe: u8, slot: u8) -> bool {
        if pipe >= 4 || slot >= 4 || index > usize::from(u8::MAX) {
            return false;
        }
        let index = index as u8;
        let pipe_entry = &mut self.first_by_pipe[usize::from(pipe)];
        if pipe_entry.is_none() {
            *pipe_entry = Some(index);
        }
        let slot_entry = &mut self.by_slot[usize::from(pipe) * 4 + usize::from(slot)];
        if slot_entry.is_none() {
            *slot_entry = Some(index);
        }
        true
    }

    pub const fn contains_pipe(self, pipe: u8) -> bool {
        pipe < 4 && self.first_by_pipe[pipe as usize].is_some()
    }

    pub const fn first_in_pipe(self, pipe: usize) -> Option<usize> {
        if pipe < 4 {
            match self.first_by_pipe[pipe] {
                Some(index) => Some(index as usize),
                None => None,
            }
        } else {
            None
        }
    }

    pub const fn slot_owner(self, pipe: u8, slot: u8) -> Option<usize> {
        if pipe < 4 && slot < 4 {
            match self.by_slot[pipe as usize * 4 + slot as usize] {
                Some(index) => Some(index as usize),
                None => None,
            }
        } else {
            None
        }
    }

    pub fn slot_mask(self) -> u16 {
        self.by_slot
            .iter()
            .enumerate()
            .fold(0_u16, |mask, (index, owner)| {
                mask | if owner.is_some() { 1_u16 << index } else { 0 }
            })
    }

    pub fn next_slot_owner(self, eligible: u16, cursor: u8) -> Option<(u8, usize)> {
        (0..16_u8).find_map(|offset| {
            let slot_index = cursor.wrapping_add(offset) & 15;
            if eligible & (1_u16 << slot_index) == 0 {
                return None;
            }
            self.by_slot[usize::from(slot_index)]
                .map(|owner| (slot_index, usize::from(owner)))
        })
    }

    pub fn is_empty(self) -> bool {
        self.first_by_pipe.iter().all(Option::is_none)
    }
}

impl Default for Class0RuntimeOwners {
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
    fn runtime_owners_coalesce_aggregate_members_but_retain_distinct_slots() {
        let mut owners = Class0RuntimeOwners::new();
        assert!(owners.observe(7, 2, 1));
        assert!(owners.observe(9, 2, 1));
        assert!(owners.observe(10, 2, 3));
        assert!(owners.observe(11, 0, 2));
        assert_eq!(owners.first_in_pipe(0), Some(11));
        assert_eq!(owners.first_in_pipe(2), Some(7));
        assert_eq!(owners.first_in_pipe(1), None);
        assert_eq!(owners.slot_owner(2, 1), Some(7));
        assert_eq!(owners.slot_owner(2, 3), Some(10));
        assert_eq!(owners.slot_owner(2, 0), None);
        let mut eligible = owners.slot_mask();
        assert_eq!(eligible, (1 << 2) | (1 << 9) | (1 << 11));
        assert_eq!(owners.next_slot_owner(eligible, 10), Some((11, 10)));
        eligible &= !(1 << 11);
        assert_eq!(owners.next_slot_owner(eligible, 12), Some((2, 11)));
        eligible &= !(1 << 2);
        assert_eq!(owners.next_slot_owner(eligible, 3), Some((9, 7)));
        assert_eq!(owners.next_slot_owner(0, 0), None);
        assert!(owners.contains_pipe(0));
        assert!(!owners.contains_pipe(1));
        assert!(!owners.contains_pipe(4));
        assert!(!owners.is_empty());
        assert!(!owners.observe(12, 4, 0));
        assert!(!owners.observe(12, 0, 4));
        assert_eq!(Class0RuntimeOwners::new().first_in_pipe(4), None);
        assert_eq!(Class0RuntimeOwners::new().slot_owner(0, 4), None);
        assert!(Class0RuntimeOwners::new().is_empty());
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
