# XR819 firmware session handoff — 2026-08-15

Read this first, then
[`xr819-class0-tx-status-findings.md`](xr819-class0-tx-status-findings.md) for
the detailed evidence, then
[`xr819-hif-startup-flow.md`](xr819-hif-startup-flow.md).

Supersedes [`xr819-session-handoff-2026-08-10.md`](xr819-session-handoff-2026-08-10.md).

## Milestone reached: the link carries real traffic

The previous milestone was scan-owned probe TX. The firmware now associates and
passes ordinary data traffic over the air, board to a wired server:

```text
ping  2643 sent, 2584 received, 2.23% loss   (second pass 20/20, 0% loss)
iperf TCP uplink, 25 s sustained, ~1.2 Mbit/s
TXed 5015   RXed 6506   Pending TX 0   Used bufs 0
funnel: admitted 2646 -> published 2646 -> completed 2629 -> confirmed 2646
```

Image: `cd813a4dfc64ed276c8fcb62faad482bd2ebac42675948b7aa1b1caa52f47089`,
built by `tools/build-ota-image.sh` with
`class0-lifecycle-counters,pipe-watchdog,corruption-non-fatal`.

## The one thing to understand before continuing

**Our own diagnostics were the main reason the driver appeared broken.** Three
RX/TX boundary detectors halt the firmware the instant they see a TX command
signature in the RX FIFO. Every run halted seconds in, and each halt was read as
the firmware failing. Disabling them progressively:

```text
                        TXed   ping loss
halting (whole session)  ~100   89-100%
non-fatal, 2 detectors   1015    48.8%
non-fatal, 3 detectors   5015     2.2%
```

The corruption they detect is real but rare and survivable: 21 RX
resynchronisations across 5400 published frames, absorbed by the RX path's own
resynchronisation.

`corruption-non-fatal` is therefore required for any performance measurement.
Leave it **off** when investigating the corruption itself, since the halted
records carry the full state.

## What was fixed this session

| change | status |
| --- | --- |
| Retire class-0 slots the MAC refuses (defect B), keyed on the hardware ring cursor | committed, effective |
| Port vendor's 200 ms TX pipe watchdog (`FUN_00003bac`) | committed; **fired 0 times** in the working run |
| Make the three corruption detectors report-and-continue | committed, **this is what unblocked traffic** |
| `tools/check.sh`, `tools/build-ota-image.sh`, ARM-aware feature/type checking | committed |
| Class-0 lifecycle counters + decoder | committed |

## Where the throughput actually goes (measured)

Admission-to-confirmation latency, from the lifecycle counters:

```text
latency_last  ~3.4-4.0 ms      latency_max  20-42 ms
ping RTT avg  ~10.5 ms         airtime, 512 B at 9 Mbit/s  ~0.5 ms
```

**About 4 ms of each frame's ~10 ms round trip is spent inside our own TX
pipeline**, against roughly 0.5 ms of airtime. Raising the PHY rate therefore
cannot help much: it shrinks a part of the budget that is already small.

We publish one frame and wait for its completion, so throughput is pinned at
1/latency, i.e. ~250 frames/s at best. That matches the observed ~1.2 Mbit/s.

### The hardware already pipelines; we do not use it

From disassembly of `txp_scheduler_run` and the tx_start/tx_success handlers:
the MAC advances `pipe_state + 0xa2` (current) on tx_start whenever it differs
from `+0xa1` (last), so **multiple descriptors run back to back with no
firmware involvement**. On tx_success, `current == last` means the whole batch
is done. Vendor arms a *range* of slots in one shot; each pipe has four.

Our `execute_single_probe_publication` mirrors the single-frame path exactly, so
its publish loop runs once (next slot equals current). The minimal change is:
build N slot records with their own command storage, set `pipe_state + 1` to the
last slot, loop the GO writes from producer to last, then arm. **No aggregation
and no BlockAck session are required for this.**

A-MPDU is a separate and larger job: it needs QoS Data (not Null), unicast,
firmware-assigned sequence, ack policy 0, and a BA session, which in turn
requires the host to set a non-zero TX TID mask via the block-ack policy MIB.

## Open work, highest value first

1. **Multi-slot publication (pipelining).** Up to 4 frames in flight per pipe
   instead of 1, using hardware chaining that already exists. Evidence-backed
   recipe above. Largest win available and does not depend on rates,
   aggregation, or the corruption.
2. **Rate selection and aggregation.** Rate index 7 is OFDM 9 Mbit/s and
   `AGG TXed` is 0. Worth doing, but airtime is ~0.5 ms of a ~10 ms budget, so
   expect roughly 10% rather than a multiple until pipelining lands.

   The firmware itself **cannot** raise a rate. With the driver's policy flags
   (`TERMINATE_WHEN_FINISHED | COUNT_INITIAL_TRANSMIT` = `0x0C`), vendor's only
   upward path `pas_rate_recovery_on_success` (0x876a) is gated off by
   `(flags & 3) == 2` and `rate_recoveries != 0`. Rate control belongs entirely
   to minstrel_ht on the host; the firmware transmits `wsm_tx.max_tx_rate` and
   reports back what happened.

   We were reporting all-zero `rate_try` nibbles, and `cw1200_xr819_tx_status`
   derives `rate_num` from exactly those: an all-zero set makes it write
   `idx = -1, count = 0` into every `tx->status.rates[]`, so mac80211 recorded
   **no attempts at all** and minstrel never accumulated samples to promote a
   rate. Fixed (`host_tx_policy::rate_try_for_single_rate`).

   **Measured: this changed nothing.** 1.11 Mbit/s before, 1.06 Mbit/s after,
   within run-to-run noise. It is a correctness fix that restores the host's
   feedback loop, not a throughput win, which is what the latency budget
   predicted. Note also that with `AGG TXed == 0` the MCS indices (14-21) are
   practically unreachable, so minstrel's ceiling is legacy OFDM regardless.
3. **Failure rate under TCP.** `retired` is 0.8% under ping flood but 7.4%
   under sustained TCP. Understand what bidirectional load changes.
4. **The corruption itself.** TX command-storage bytes reach the RX FIFO:
   7 words from `0x0900717c` observed at `0x09403f54` inside the producer
   delta, and an RX slot whose ownership word was `0x07004600` (the class-0
   command list `+0x38` word). Now a quality issue rather than a blocker.
   Suggested approach: disassembly investigation of how vendor arbitrates the
   packet controller between TX command fetch and RX FIFO writes.

Note there are **four** halting corruption detectors, three in `radio.rs` and
one in `hif.rs`; `corruption-non-fatal` neutralises all of them. Do not count
corruption inside `hif::validate_tx_boundary`: it runs several times per
main-loop pass, so a counter there measures call frequency (it read 2,162,040)
rather than corruption events.

## Reading the decompilation: the raw image is incomplete

`/tmp/xr819-ann-main.bin` is zero-padded from `0x1b400` and does **not** contain
the DTCM data image, so `DAT_*` constants pointing at `0x0400_0000+` cannot be
resolved from it. The full payload is `fw_xr819.bin` from offset `0x1ab8`; its
`XR01` load records put DTCM `.data` at payload offset `0x1b3dc`, so
**DTCM address A = payload offset 0x1b3dc + A**. Cross-check: `0x040002dc` and
`0x040002e0` both read `01 00 02 03`, matching the queue->AC and AC->pipe tables
already recorded here.

Several functions are also absent from `annotated-main.c` because Ghidra never
created them (the 200 ms TX watchdog at `0x3bac` is one). When a path leads into
a gap, disassemble the raw image rather than concluding the code does not exist.

## Working practices that proved necessary

- **`tools/check.sh` before flashing, always.** `host_tx_driver` is
  `#[cfg(target_arch = "arm")]`, so `cargo test` compiles none of it and stays
  green on code that does not build for ARM. Three images were flashed this
  session containing neither the fix under test nor the STA JOIN path.
- **`tools/build-ota-image.sh` for images.** `unmatched-tx-status-recovery`
  alone pulls in nothing; the resulting image boots but cannot associate.
- **One run proves nothing.** Identical firmware spans 36-95 TXed. Always run
  the unmodified control in the same session before concluding a change
  regressed or improved anything.
- **Prefer static evidence when a change should be semantically neutral.**
  Comparing two packed images (4 differing bytes of 75864) settled a suspected
  regression in seconds where the harness could not have.
- **Hardware-free decisions belong in host-testable modules**
  (`host_tx_policy`, `plan_pipe_watchdog`), because the ARM paths are invisible
  to the test suite.

## Harness

`xr819-firmware/tools/ota-soak-run.sh IMAGE LABEL` flashes, reboots, waits for
association, then runs a 25 s ping flood and a 25 s iperf TCP uplink in the
wifi netns against a server on the board's wired `end0`. It waits for the board
to answer ssh before touching it, so a reboot race cannot masquerade as a
firmware failure. Decode counters with
`tools/decode-lifecycle-counters.py < run.log`.
