//! Hardware-free ownership model for the packet-DMA RX FIFO.
//!
//! Hardware access remains in `radio`; this module owns the cursor and token
//! invariants that must hold around those accesses.

const RELEASED_OWNERSHIP: u32 = 0xcccc_cc00;
const PENDING_OWNERSHIP: u32 = 0xffff_ff00;
const OWNERSHIP_MASK: u32 = 0xffff_ff00;

/// Unique ownership proof for one claimed zero-copy RX slot.
///
/// Tokens are deliberately neither `Copy` nor `Clone`. Only the RX ring may
/// construct one, and releasing a slot consumes it.
#[derive(Debug, Eq, PartialEq)]
pub struct RxToken {
    slot_offset: u32,
    next: u32,
}

impl RxToken {
    const fn new(slot_offset: u32, next: u32) -> Self {
        Self { slot_offset, next }
    }

    pub(crate) const fn slot_offset(&self) -> u32 {
        self.slot_offset
    }

    pub(crate) const fn next(&self) -> u32 {
        self.next
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReleaseAction {
    AlreadyReleased,
    Pending,
    MarkPending { low_state: u32, corrupt: bool },
    ReleaseHead {
        low_state: u32,
        normalize_first: bool,
        corrupt: bool,
    },
}

/// Software ownership state for the RX ring.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct RxRing {
    release_offset: u32,
    claim_offset: u32,
    host_transfer_count: u32,
    resync_release_deferred: bool,
}

impl RxRing {
    pub(crate) const fn new() -> Self {
        Self {
            release_offset: 0,
            claim_offset: 0,
            host_transfer_count: 0,
            resync_release_deferred: false,
        }
    }

    pub(crate) fn synchronize(&mut self, offset: u32) {
        self.release_offset = offset;
        self.claim_offset = offset;
        self.host_transfer_count = 0;
        self.resync_release_deferred = false;
    }

    pub(crate) const fn release_offset(&self) -> u32 {
        self.release_offset
    }

    pub(crate) const fn claim_offset(&self) -> u32 {
        self.claim_offset
    }

    pub(crate) const fn host_transfer_count(&self) -> u32 {
        self.host_transfer_count
    }

    /// Claims the current ring head and advances to `next`.
    ///
    /// The caller cannot choose the token identity independently of the ring's
    /// current claim cursor.
    pub(crate) fn claim(&mut self, next: u32) -> RxToken {
        let token = RxToken::new(self.claim_offset, next);
        self.claim_offset = next;
        token
    }

    pub(crate) fn publish_host_transfer(&mut self, limit: u32) -> bool {
        if self.host_transfer_count >= limit {
            return false;
        }
        self.host_transfer_count += 1;
        true
    }

    pub(crate) fn complete_host_transfer(&mut self) -> bool {
        if self.host_transfer_count == 0 {
            return false;
        }
        self.host_transfer_count -= 1;
        true
    }

    pub(crate) fn set_claim_offset(&mut self, next: u32) {
        self.claim_offset = next;
    }

    /// Records a parser resynchronization and returns the release cursor update
    /// that may be applied after the claim-cursor hardware write.
    pub(crate) fn resynchronize(&mut self, previous_claim: u32, target: u32) -> Option<u32> {
        self.claim_offset = target;
        if self.host_transfer_count == 0 && self.release_offset == previous_claim {
            self.resync_release_deferred = false;
            Some(target)
        } else {
            self.resync_release_deferred = true;
            None
        }
    }

    pub(crate) fn set_release_offset(&mut self, next: u32) {
        self.release_offset = next;
    }

    /// Finishes a deferred resynchronization after the last host transfer is
    /// returned. The caller performs the matching hardware-consumer write.
    pub(crate) fn finish_deferred_resync(&mut self) -> Option<u32> {
        if self.resync_release_deferred && self.host_transfer_count == 0 {
            self.release_offset = self.claim_offset;
            self.resync_release_deferred = false;
            Some(self.release_offset)
        } else {
            None
        }
    }

    pub(crate) const fn classify_release(
        &self,
        token: &RxToken,
        state: u32,
    ) -> ReleaseAction {
        let ownership = state & OWNERSHIP_MASK;
        let low_state = state & 0xff;
        if ownership == RELEASED_OWNERSHIP {
            return ReleaseAction::AlreadyReleased;
        }

        let pending = ownership == PENDING_OWNERSHIP;
        let corrupt = ownership != 0 && !pending;
        if token.slot_offset != self.release_offset {
            if pending {
                ReleaseAction::Pending
            } else {
                ReleaseAction::MarkPending { low_state, corrupt }
            }
        } else {
            ReleaseAction::ReleaseHead {
                low_state,
                normalize_first: !pending,
                corrupt,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claim_token_identity_comes_from_the_current_head() {
        let mut ring = RxRing::new();
        ring.synchronize(0x100);

        let first = ring.claim(0x180);
        let second = ring.claim(0x240);

        assert_eq!(first.slot_offset(), 0x100);
        assert_eq!(first.next(), 0x180);
        assert_eq!(second.slot_offset(), 0x180);
        assert_eq!(second.next(), 0x240);
        assert_eq!(ring.claim_offset(), 0x240);
    }

    #[test]
    fn corrupt_release_head_is_reclaimed_in_the_same_call() {
        let mut ring = RxRing::new();
        ring.synchronize(0x100);
        let token = ring.claim(0x180);

        assert_eq!(
            ring.classify_release(&token, 0x1234_56a5),
            ReleaseAction::ReleaseHead {
                low_state: 0xa5,
                normalize_first: true,
                corrupt: true,
            }
        );
    }

    #[test]
    fn out_of_order_release_is_marked_pending() {
        let mut ring = RxRing::new();
        ring.synchronize(0x100);
        ring.set_claim_offset(0x180);
        let token = ring.claim(0x200);

        assert_eq!(
            ring.classify_release(&token, 0x0000_00a5),
            ReleaseAction::MarkPending {
                low_state: 0xa5,
                corrupt: false,
            }
        );
    }

    #[test]
    fn resync_never_releases_across_host_owned_slots() {
        let mut ring = RxRing::new();
        ring.synchronize(0x100);
        assert!(ring.publish_host_transfer(24));

        assert_eq!(ring.resynchronize(0x100, 0x240), None);
        assert_eq!(ring.release_offset(), 0x100);
        assert_eq!(ring.claim_offset(), 0x240);
        assert!(ring.complete_host_transfer());
        assert_eq!(ring.finish_deferred_resync(), Some(0x240));
    }

    #[test]
    fn resync_advances_immediately_when_the_ring_is_unowned() {
        let mut ring = RxRing::new();
        ring.synchronize(0x100);

        assert_eq!(ring.resynchronize(0x100, 0x240), Some(0x240));
        ring.set_release_offset(0x240);
        assert_eq!(ring.release_offset(), 0x240);
        assert_eq!(ring.claim_offset(), 0x240);
    }

    #[test]
    fn host_transfer_limit_is_exact() {
        let mut ring = RxRing::new();
        for _ in 0..24 {
            assert!(ring.publish_host_transfer(24));
        }
        assert!(!ring.publish_host_transfer(24));
        for _ in 0..24 {
            assert!(ring.complete_host_transfer());
        }
        assert!(!ring.complete_host_transfer());
    }
}
