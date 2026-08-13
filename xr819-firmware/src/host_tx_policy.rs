//! Host TX decisions that carry no hardware access.
//!
//! `host_tx_driver` is `#[cfg(target_arch = "arm")]`, so nothing in it is
//! compiled or tested by a host `cargo test` run. Decisions that can be
//! expressed without touching hardware live here instead, where they are
//! compiled and tested on every build, in the same spirit as
//! `tx::plan_ordinary_tx_pipe_status`.

/// What to do with a TX completion observed while servicing one host slot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompletionRouting {
    /// The completion belongs to the slot being serviced; confirm it.
    ConfirmServicedSlot,
    /// The completion names some other context. Record it and do nothing else.
    Ignore,
}

/// Decides who may consume a completion drained while servicing one slot.
///
/// Completions come from a single global queue but are drained from inside
/// per-slot servicing, so a completion frequently names a context other than
/// the slot doing the draining. Only the serviced slot may consume one.
///
/// Routing the completion to whichever slot owns the named context looks
/// obviously correct and is not: it was measured over the air against an
/// otherwise identical image and regressed transmission badly, while leaving
/// the buffer accounting it was meant to fix unchanged.
///
/// ```text
///                 TXed  RXed  ping received        Pending TX  Used bufs
/// ignore (this)     95   138  51/448 (88.6% loss)           2          7
/// route to owner    27    43   0/561 ( 100% loss)           2          6
/// ```
///
/// See `xr819-class0-tx-status-findings.md`. Do not reintroduce routing
/// without new over-the-air evidence.
pub const fn route_completion(serviced_context: u32, completion_context: u32) -> CompletionRouting {
    if serviced_context == completion_context {
        CompletionRouting::ConfirmServicedSlot
    } else {
        CompletionRouting::Ignore
    }
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
    fn completion_for_the_serviced_slot_is_confirmed() {
        assert_eq!(
            route_completion(0x0901_2340, 0x0901_2340),
            CompletionRouting::ConfirmServicedSlot
        );
    }

    #[test]
    fn completion_naming_another_context_is_never_routed_to_it() {
        // Measured: routing these to their owner cost TXed 95 -> 27 and all
        // ping delivery. The only safe action is to ignore them here.
        assert_eq!(
            route_completion(0x0901_2340, 0x0901_5580),
            CompletionRouting::Ignore
        );
        assert_eq!(route_completion(0, 0x0901_5580), CompletionRouting::Ignore);
    }
}
