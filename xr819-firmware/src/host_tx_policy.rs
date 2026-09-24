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

    pub fn occupied_slots(self, pipe: u8) -> Option<[bool; 4]> {
        if pipe >= 4 {
            return None;
        }
        Some(core::array::from_fn(|slot| {
            self.by_slot[usize::from(pipe) * 4 + slot].is_some()
        }))
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

pub const MAX_ORDINARY_BATCH_DEPTH: usize = 4;

/// Pure vendor-shaped plan for one ordinary pre-GO pipe transaction.
///
/// The vendor excludes armed pipes, then fills at most four consecutive slots
/// for one idle pipe. Candidate entries are retained-context indices annotated
/// with their queue-mapped pipe; `None` means that the context is not eligible
/// during this service pass. Any retained slot owner rejects the whole plan
/// rather than treating an active pipe as appendable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OrdinaryBatchPlan {
    pipe: u8,
    len: u8,
    indices: [usize; MAX_ORDINARY_BATCH_DEPTH],
    slots: [u8; MAX_ORDINARY_BATCH_DEPTH],
}

impl OrdinaryBatchPlan {

    pub const fn len(self) -> usize {
        self.len as usize
    }

    pub const fn index(self, position: usize) -> Option<usize> {
        if position < self.len as usize {
            Some(self.indices[position])
        } else {
            None
        }
    }

    pub const fn slot(self, position: usize) -> Option<u8> {
        if position < self.len as usize {
            Some(self.slots[position])
        } else {
            None
        }
    }
}

/// Select the oldest live admission, even across ticket wrap and arena reuse.
/// Callers exclude owners already visited in this bounded service pass.
/// Pending lifetimes must remain shorter than a full u32 admission cycle.
pub fn oldest_pending_context(
    candidates: impl IntoIterator<Item = (usize, u32)>,
    next_order: u32,
) -> Option<usize> {
    candidates.into_iter()
        .max_by_key(|(_, order)| next_order.wrapping_sub(*order))
        .map(|(index, _)| index)
}

pub fn plan_ordinary_batch(
    candidates: &[Option<u8>],
    candidate_order: &[usize],
    occupied_slots: [bool; 4],
    first_index: usize,
    pipe: u8,
    first_slot: u8,
) -> Option<OrdinaryBatchPlan> {
    if pipe >= 4
        || first_slot >= 4
        || candidates.get(first_index).copied().flatten() != Some(pipe)
        || candidate_order.first().copied() != Some(first_index)
        || candidate_order.iter().enumerate().any(|(position, index)| {
            candidates.get(*index).copied().flatten().is_none()
                || candidate_order[..position].contains(index)
        })
        || occupied_slots.into_iter().any(|occupied| occupied)
    {
        return None;
    }

    let mut indices = [0; MAX_ORDINARY_BATCH_DEPTH];
    let mut slots = [0; MAX_ORDINARY_BATCH_DEPTH];
    indices[0] = first_index;
    slots[0] = first_slot;
    let mut len = 1;
    for &index in candidate_order.iter().skip(1) {
        if candidates[index] != Some(pipe) {
            continue;
        }
        if len == MAX_ORDINARY_BATCH_DEPTH {
            break;
        }
        indices[len] = index;
        slots[len] = first_slot.wrapping_add(len as u8) & 3;
        len += 1;
    }

    Some(OrdinaryBatchPlan {
        pipe,
        len: len as u8,
        indices,
        slots,
    })
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

/// Per-rate failure nibbles for a frame transmitted at a single rate.
///
/// XR819 TX confirmations report failed attempts in these 24 nibbles. The
/// cw1200 driver adds the final successful attempt at `tx_rate`; including it
/// here would make every first-try success look like one retry to minstrel.
/// The words use `word = rate >> 3`, `nibble = rate & 7`.
pub const fn rate_try_for_single_rate(rate: u8, ack_failures: u8) -> [u32; 3] {
    let mut rate_try = [0_u32; 3];
    if rate < 24 {
        let failures = if ack_failures > 0xf { 0xf } else { ack_failures as u32 };
        rate_try[(rate >> 3) as usize] = failures << ((rate as u32 & 7) * 4);
    }
    rate_try
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_try_encodes_only_failed_attempts() {
        // The driver adds the successful attempt itself, so a first-try
        // success must leave all failure nibbles clear.
        assert_eq!(rate_try_for_single_rate(7, 0), [0; 3]);
        assert_eq!(rate_try_for_single_rate(8, 0), [0; 3]);
        // Index 13 (OFDM 54) with three failed attempts before success.
        assert_eq!(rate_try_for_single_rate(13, 3), [0, 0x0030_0000, 0]);
    }

    #[test]
    fn rate_try_saturates_failure_counts() {
        assert_eq!(rate_try_for_single_rate(0, 200), [0xf, 0, 0]);
        // Out-of-range indices report nothing rather than corrupting a word.
        assert_eq!(rate_try_for_single_rate(24, 1), [0; 3]);
    }

    #[test]
    fn ordinary_batch_plan_rejects_occupied_or_stale_ownership() {
        let candidates = [None, Some(1), Some(1)];
        assert!(plan_ordinary_batch(&candidates, &[1, 2], [false, true, false, false], 1, 1, 0).is_none());
        assert!(plan_ordinary_batch(&candidates, &[1, 2], [false; 4], 1, 4, 0).is_none());
        assert!(plan_ordinary_batch(&candidates, &[1, 2], [false; 4], 1, 1, 4).is_none());
        assert!(plan_ordinary_batch(&candidates, &[1, 2], [false; 4], 2, 1, 0).is_none());
        assert!(plan_ordinary_batch(&candidates, &[1, 2], [false; 4], 1, 0, 0).is_none());
    }

    #[test]
    fn ordinary_batch_plan_requires_the_first_eligible_context() {
        let candidates = [Some(0), Some(1), Some(1)];
        assert!(plan_ordinary_batch(&candidates, &[0, 1, 2], [false; 4], 1, 1, 0).is_none());
    }

    #[test]
    fn pending_order_survives_arena_reuse_and_ticket_wrap() {
        let candidates = [(0, 0), (3, u32::MAX - 1), (1, u32::MAX)];
        assert_eq!(oldest_pending_context(candidates, 1), Some(3));
        assert_eq!(oldest_pending_context(candidates.into_iter().filter(|(i, _)| *i != 3), 1), Some(1));
        assert_eq!(oldest_pending_context([(0, 0)], 1), Some(0));
        assert_eq!(oldest_pending_context([], 1), None);
    }

    #[test]
    fn ordinary_batch_preserves_fifo_across_reused_context_indices() {
        let candidates = [Some(2), Some(2), Some(1), Some(2), Some(2)];
        let plan = plan_ordinary_batch(&candidates, &[3, 1, 2, 0, 4], [false; 4], 3, 2, 2)
            .expect("FIFO head need not have the lowest context index");
        assert_eq!(core::array::from_fn(|i| plan.index(i)), [Some(3), Some(1), Some(0), Some(4)]);
        assert_eq!(core::array::from_fn(|i| plan.slot(i)), [Some(2), Some(3), Some(0), Some(1)]);
        assert!(plan_ordinary_batch(&candidates, &[3, 3], [false; 4], 3, 2, 2).is_none());
        assert!(plan_ordinary_batch(&candidates, &[3, 99], [false; 4], 3, 2, 2).is_none());
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
