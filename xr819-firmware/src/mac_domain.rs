//! Exclusive capability for mutation of shared MAC scheduler and pipe state.
//!
//! The firmware is cooperative, but the MAC domain is also visible to IRQ/FIQ
//! handlers. Entering the domain preserves the current CPU interrupt state and
//! masks both classes until the guard is dropped.

/// Root owner for shared MAC mutation.
///
/// Keep this as an ordinary value owned by `rust_main`; a mutable borrow proves
/// that only one guard can exist at a time.
pub struct MacDomain {
    _private: (),
}

impl Default for MacDomain {
    fn default() -> Self {
        Self::new()
    }
}

impl MacDomain {
    pub const fn new() -> Self {
        Self { _private: () }
    }

    pub fn enter(&mut self) -> MacDomainGuard<'_> {
        let interrupt_state = unsafe { crate::tx::disable_irq_fiq_save() };
        MacDomainGuard {
            _domain: self,
            interrupt_state,
        }
    }
}

/// Proof that the caller exclusively owns the shared MAC mutation domain.
///
/// The guard may not escape the borrow of `MacDomain`. Dropping it restores the
/// IRQ/FIQ mask bits that were present on entry.
pub struct MacDomainGuard<'a> {
    _domain: &'a mut MacDomain,
    interrupt_state: u32,
}

impl Drop for MacDomainGuard<'_> {
    fn drop(&mut self) {
        unsafe { crate::tx::restore_irq_fiq_saved(self.interrupt_state) };
    }
}

/// Pure model of the CPSR mask-bit restoration performed by the ARM backend.
const fn restored_interrupt_state(current: u32, previous: u32) -> u32 {
    (current & !0xc0) | (previous & 0xc0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restore_preserves_non_interrupt_cpsr_bits() {
        assert_eq!(
            restored_interrupt_state(0x6000_00d3, 0x0000_0013),
            0x6000_0013
        );
        assert_eq!(
            restored_interrupt_state(0x6000_0013, 0x0000_00d3),
            0x6000_00d3
        );
    }

    #[test]
    fn restore_uses_only_the_saved_irq_fiq_bits() {
        assert_eq!(restored_interrupt_state(0x1234_5678, 0xffff_ff00), 0x1234_5638);
        assert_eq!(restored_interrupt_state(0x1234_5678, 0xffff_ffff), 0x1234_56f8);
    }
}
