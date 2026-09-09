//! Shared PAS contention-window (backoff) bank.
//!
//! One production implementation of the vendor bank arithmetic, used by every
//! qualified mutation site: the physical-retry growth at the retry entry
//! (`0x967e` -> `0xff62`), the two vendor-mapped reset hooks (status `0x9a8c`,
//! TX-success `0x9d06..0x9d2c`), the head policy-exhaustion reset
//! (`0x8a20..0x8a26`) and the VIF JOIN reset-all.
//!
//! Instruction-verified layout (`0xff64..0xffc8`, `0x8838..0x8864`). For
//! interface `i` and access category `a` the bank lives at
//! `0x04003678 + i * 0x98` with retry counter `+0x4ac + 4a`, current window
//! `+0x4bc + 4a`, CWmin `+0x4cc + 2a` and CWmax `+0x4d4 + 2a`. Those exact
//! addresses are produced by [`crate::dtcm::pas_stride_view`].
//!
//! Growth: when the old counter is even the window becomes `2 * W + 1` with u32
//! wrapping; the clamp against the maximum applies on **both** parities and the
//! counter always advances by one (wrapping). Reset: counter `0`, window
//! `CWmin`.
//!
//! The override block at `0x04002088` selects alternative sources: `+4` is the
//! reset window and `+8` the growth clamp. Nothing here enables override mode;
//! it only reproduces the arithmetic when the mode is already on.

/// Byte-level access to the PAS bank and the override control block.
///
/// Production supplies volatile MMIO (through [`VolatileBackoffBankIo`] or the
/// TX executor's pipe MMIO); tests supply a recording mock, so every qualified
/// mutation site is exercised without duplicating the arithmetic.
pub(crate) trait BackoffBankIo {
    fn read_u16(&mut self, address: usize) -> u16;
    fn read_u32(&mut self, address: usize) -> u32;
    fn write_u32(&mut self, address: usize, value: u32);
}

/// Rejected bank selection. PAS has three banks (interface 2 is a legal
/// internal bank) and four access categories per bank. A rejected selection
/// performs no write and never halts or spins: the callers are all past the
/// destructive MAC event read.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BackoffBankError {
    Interface,
    AccessCategory,
}

#[cfg(target_arch = "arm")]
pub(crate) struct VolatileBackoffBankIo;

#[cfg(target_arch = "arm")]
impl BackoffBankIo for VolatileBackoffBankIo {
    #[inline(always)]
    fn read_u16(&mut self, address: usize) -> u16 {
        unsafe { (address as *const u16).read_volatile() }
    }

    #[inline(always)]
    fn read_u32(&mut self, address: usize) -> u32 {
        unsafe { (address as *const u32).read_volatile() }
    }

    #[inline(always)]
    fn write_u32(&mut self, address: usize, value: u32) {
        unsafe { (address as *mut u32).write_volatile(value) };
    }
}

fn override_enabled<I: BackoffBankIo>(io: &mut I) -> bool {
    io.read_u32(crate::dtcm::pas_backoff_override_enabled().get()) != 0
}

fn override_reset_window<I: BackoffBankIo>(io: &mut I) -> u32 {
    io.read_u32(crate::dtcm::pas_backoff_override_window().get())
}

fn override_maximum_window<I: BackoffBankIo>(io: &mut I) -> u32 {
    io.read_u32(crate::dtcm::pas_backoff_override_maximum_window().get())
}

struct BankEntry {
    retry_count: usize,
    window: usize,
    cw_min: usize,
    cw_max: usize,
}

fn entry(interface: u8, queue: u8) -> Result<BankEntry, BackoffBankError> {
    let bank = crate::dtcm::pas_stride_view(usize::from(interface))
        .ok_or(BackoffBankError::Interface)?;
    let queue = usize::from(queue);
    Ok(BankEntry {
        retry_count: bank
            .retry_count(queue)
            .ok_or(BackoffBankError::AccessCategory)?
            .get(),
        window: bank
            .contention_window(queue)
            .ok_or(BackoffBankError::AccessCategory)?
            .get(),
        cw_min: bank
            .cw_min(queue)
            .ok_or(BackoffBankError::AccessCategory)?
            .get(),
        cw_max: bank
            .cw_max(queue)
            .ok_or(BackoffBankError::AccessCategory)?
            .get(),
    })
}

/// Vendor `0xff62`: one growth per qualified physical transmission retry.
pub(crate) fn grow<I: BackoffBankIo>(
    io: &mut I,
    interface: u8,
    queue: u8,
) -> Result<(), BackoffBankError> {
    let entry = entry(interface, queue)?;
    let maximum = if override_enabled(io) {
        override_maximum_window(io)
    } else {
        u32::from(io.read_u16(entry.cw_max))
    };
    let count = io.read_u32(entry.retry_count);
    let window = io.read_u32(entry.window);
    let grown = if count & 1 == 0 {
        window.wrapping_mul(2).wrapping_add(1)
    } else {
        window
    };
    io.write_u32(entry.window, grown.min(maximum));
    io.write_u32(entry.retry_count, count.wrapping_add(1));
    Ok(())
}

/// Vendor reset: counter cleared, window returned to CWmin (or the override
/// reset window). Used by the status hook, the TX-success marker and head
/// policy exhaustion.
pub(crate) fn reset<I: BackoffBankIo>(
    io: &mut I,
    interface: u8,
    queue: u8,
) -> Result<(), BackoffBankError> {
    let entry = entry(interface, queue)?;
    let enabled = override_enabled(io);
    let override_window = override_reset_window(io);
    io.write_u32(entry.retry_count, 0);
    let window = if enabled {
        override_window
    } else {
        u32::from(io.read_u16(entry.cw_min))
    };
    io.write_u32(entry.window, window);
    Ok(())
}

/// VIF reset-all: every access category of one interface, as performed by JOIN
/// and by the channel-0 EDCA AIFS change.
pub(crate) fn reset_all_queues<I: BackoffBankIo>(
    io: &mut I,
    interface: u8,
) -> Result<(), BackoffBankError> {
    // Reject the interface before any write, exactly like the per-queue path.
    entry(interface, 0)?;
    let enabled = override_enabled(io);
    let override_window = override_reset_window(io);
    for queue in 0..4_u8 {
        let entry = entry(interface, queue)?;
        io.write_u32(entry.retry_count, 0);
        let window = if enabled {
            override_window
        } else {
            u32::from(io.read_u16(entry.cw_min))
        };
        io.write_u32(entry.window, window);
    }
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// Sparse recording memory. Reads return the last written or seeded value.
    pub(crate) struct MockBankIo {
        cells: [(usize, u32); 64],
        count: usize,
        pub writes: [(usize, u32); 32],
        pub write_count: usize,
    }

    impl MockBankIo {
        pub(crate) fn new() -> Self {
            Self {
                cells: [(0, 0); 64],
                count: 0,
                writes: [(0, 0); 32],
                write_count: 0,
            }
        }

        pub(crate) fn set(&mut self, address: usize, value: u32) {
            self.cells[self.count] = (address, value);
            self.count += 1;
        }

        pub(crate) fn get(&self, address: usize) -> u32 {
            self.cells[..self.count]
                .iter()
                .rev()
                .find_map(|(stored, value)| (*stored == address).then_some(*value))
                .unwrap_or(0)
        }

        pub(crate) fn bank(&mut self, interface: u8, queue: u8, window: u32, count: u32) {
            let entry = entry(interface, queue).unwrap();
            self.set(entry.window, window);
            self.set(entry.retry_count, count);
        }

        pub(crate) fn limits(&mut self, interface: u8, queue: u8, minimum: u16, maximum: u16) {
            let entry = entry(interface, queue).unwrap();
            self.set(entry.cw_min, u32::from(minimum));
            self.set(entry.cw_max, u32::from(maximum));
        }

        pub(crate) fn state(&self, interface: u8, queue: u8) -> (u32, u32) {
            let entry = entry(interface, queue).unwrap();
            (self.get(entry.retry_count), self.get(entry.window))
        }
    }

    impl BackoffBankIo for MockBankIo {
        fn read_u16(&mut self, address: usize) -> u16 {
            self.get(address) as u16
        }

        fn read_u32(&mut self, address: usize) -> u32 {
            self.get(address)
        }

        fn write_u32(&mut self, address: usize, value: u32) {
            self.writes[self.write_count] = (address, value);
            self.write_count += 1;
            self.set(address, value);
        }
    }

    fn seeded(interface: u8, queue: u8, window: u32, count: u32) -> MockBankIo {
        let mut io = MockBankIo::new();
        io.limits(interface, queue, 15, 1023);
        io.bank(interface, queue, window, count);
        io
    }

    #[test]
    fn bank_addresses_match_the_instruction_verified_layout() {
        let entry = entry(1, 2).unwrap();
        let base = 0x0400_3678 + 0x98;
        assert_eq!(entry.retry_count, base + 0x4ac + 4 * 2);
        assert_eq!(entry.window, base + 0x4bc + 4 * 2);
        assert_eq!(entry.cw_min, base + 0x4cc + 2 * 2);
        assert_eq!(entry.cw_max, base + 0x4d4 + 2 * 2);
    }

    #[test]
    fn growth_doubles_on_even_counters_and_always_advances() {
        let mut io = seeded(0, 1, 15, 0);
        grow(&mut io, 0, 1).unwrap();
        assert_eq!(io.state(0, 1), (1, 31));
        grow(&mut io, 0, 1).unwrap();
        assert_eq!(io.state(0, 1), (2, 31));
        grow(&mut io, 0, 1).unwrap();
        assert_eq!(io.state(0, 1), (3, 63));
    }

    #[test]
    fn growth_clamps_on_both_parities_and_wraps_the_counter() {
        let mut io = seeded(0, 0, 1023, 0);
        grow(&mut io, 0, 0).unwrap();
        assert_eq!(io.state(0, 0), (1, 1023));

        // An odd counter still clamps a window left above the maximum.
        let mut odd = MockBankIo::new();
        odd.limits(0, 0, 15, 31);
        odd.bank(0, 0, 63, 3);
        grow(&mut odd, 0, 0).unwrap();
        assert_eq!(odd.state(0, 0), (4, 31));

        let mut wrapping = seeded(0, 3, 15, u32::MAX);
        grow(&mut wrapping, 0, 3).unwrap();
        assert_eq!(wrapping.state(0, 3), (0, 15));
    }

    #[test]
    fn reset_restores_the_configured_minimum() {
        let mut io = seeded(2, 3, 511, 7);
        reset(&mut io, 2, 3).unwrap();
        assert_eq!(io.state(2, 3), (0, 15));
    }

    #[test]
    fn override_mode_supplies_its_own_reset_and_clamp_windows() {
        let mut io = seeded(0, 0, 15, 0);
        io.set(crate::dtcm::pas_backoff_override_enabled().get(), 1);
        io.set(crate::dtcm::pas_backoff_override_window().get(), 7);
        io.set(crate::dtcm::pas_backoff_override_maximum_window().get(), 63);

        grow(&mut io, 0, 0).unwrap();
        assert_eq!(io.state(0, 0), (1, 31));
        grow(&mut io, 0, 0).unwrap();
        grow(&mut io, 0, 0).unwrap();
        // 31 -> 63 -> clamped at the override maximum instead of CWmax.
        assert_eq!(io.state(0, 0), (3, 63));

        reset(&mut io, 0, 0).unwrap();
        assert_eq!(io.state(0, 0), (0, 7));
    }

    #[test]
    fn every_pas_bank_is_addressable_and_isolated() {
        let mut io = MockBankIo::new();
        for interface in 0..3_u8 {
            io.limits(interface, 0, 15, 1023);
            io.bank(interface, 0, 15, 0);
        }
        grow(&mut io, 1, 0).unwrap();
        assert_eq!(io.state(0, 0), (0, 15));
        assert_eq!(io.state(1, 0), (1, 31));
        assert_eq!(io.state(2, 0), (0, 15));
    }

    #[test]
    fn out_of_range_selections_are_rejected_without_writes() {
        let mut io = MockBankIo::new();
        assert_eq!(grow(&mut io, 3, 0), Err(BackoffBankError::Interface));
        assert_eq!(grow(&mut io, 0, 4), Err(BackoffBankError::AccessCategory));
        assert_eq!(reset(&mut io, 3, 0), Err(BackoffBankError::Interface));
        assert_eq!(reset(&mut io, 0, 4), Err(BackoffBankError::AccessCategory));
        assert_eq!(
            reset_all_queues(&mut io, 3),
            Err(BackoffBankError::Interface)
        );
        assert_eq!(io.write_count, 0);
    }

    #[test]
    fn parsed_edca_minima_reach_the_production_reset_path() {
        let mut payload = [0_u8; 44];
        // Wire order is BK, BE, VI, VO; the parser reverses it into host order.
        for (wire, cwmin) in [15_u16, 15, 7, 3].into_iter().enumerate() {
            payload[wire * 2..wire * 2 + 2].copy_from_slice(&cwmin.to_le_bytes());
            payload[8 + wire * 2..8 + wire * 2 + 2].copy_from_slice(&1023_u16.to_le_bytes());
            payload[16 + wire] = 3;
        }
        let parameters = crate::wsm::EdcaParameters::parse(&payload).unwrap();

        // `apply_edca` writes the host table in wire order [3, 2, 1, 0].
        let wire = [
            parameters.queues[3],
            parameters.queues[2],
            parameters.queues[1],
            parameters.queues[0],
        ];
        let mut io = MockBankIo::new();
        for (queue, entry) in wire.into_iter().enumerate() {
            io.limits(0, queue as u8, entry.cwmin, entry.cwmax);
            io.bank(0, queue as u8, 511, 5);
        }

        reset_all_queues(&mut io, 0).unwrap();

        for (queue, entry) in wire.into_iter().enumerate() {
            assert_eq!(io.state(0, queue as u8), (0, u32::from(entry.cwmin)));
        }
    }
}
