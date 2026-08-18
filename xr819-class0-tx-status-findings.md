# XR819 class-0 TX: from "the MAC refuses data" to a working link

Hardware-proven findings from the 2026-08-15 session. Start with
[`xr819-session-handoff-2026-08-15.md`](xr819-session-handoff-2026-08-15.md)
for the current state; this file is the detailed evidence trail, including the
hypotheses that were wrong.

This supersedes the
"stale ring ownership" theories in
[`xr819-host-tx-concurrency-architecture.md`](xr819-host-tx-concurrency-architecture.md)
and the falsified producer-advancement experiment recorded in
[`xr819-session-handoff-2026-08-10.md`](xr819-session-handoff-2026-08-10.md).

## Final correction: terminal collapse was an RX release-head logic bug

The earlier description below that corruption was “rare and survivable” is true
only for the initial low-rate run and is superseded for sustained hardware-CCMP
traffic. Exact TX command words can corrupt an RX slot's ownership word. In
`corruption-non-fatal`, Rust normalized that word to the vendor pending-release
marker and immediately returned. If the corrupt slot was already
`RELEASE_OFFSET`, no later callback could revisit it, so the release cursor and
packet-DMA consumer remained pinned until terminal RX death.

The fixed release path normalizes the word and then continues through ordinary
head release and the consecutive pending-successor walk. Three no-FIQ
qualifications with this correction were healthy at 3.85-4.93 Mbit/s TCP and
7.92-7.93 Mbit/s UDP delivered, all ending 20/20 ping. Restoring only the old
early return reproduced terminal failure in two of three runs. This isolates
the terminal-collapse trigger from the still-open question of why packet
controller command content reaches RX RAM in the first place.

RX resynchronization now also defers its hardware-consumer jump while zero-copy
HIF transfers remain outstanding. The old resync behavior stayed reachable 3/3
in isolation, so it was not the terminal-collapse trigger, but it violated RX
buffer ownership and remains corrected.

FIQ is not required for this fix. The complete FIQ experiment is preserved on
Jujutsu bookmark `feature/mac-fiq` at commit `0d4c3ee4`.

## Historical result: the link first worked

**The firmware carries real traffic.** Measured over the air, board to a wired
server, with the halting diagnostics disabled (`corruption-non-fatal`):

```text
ping  2643 sent, 2584 received, 2.23% loss   (second pass 20/20, 0% loss)
iperf TCP uplink, 25 s sustained:
  0-5 s 1.17 Mbit/s | 5-10 s 1.19 | 10-15 s 1.15 | 15-20 s 1.15 | 20-25 s 1.65
TXed 5015   RXed 6506   Pending TX 0   Used bufs 0
funnel: admitted 2646 -> published 2646 -> completed 2629 -> confirmed 2646
```

**The single largest cause of "the driver does not work" was our own
diagnostics.** `validate_tx_boundary`, the RX resynchronisation check and the
RX release check each *halt the firmware* on first sight of corruption. Every
run of this session stopped itself seconds in, and each halt was read as the
firmware failing. Making the three report-and-continue instead:

```text
                       TXed   ping loss
halting (all session)   ~100   89-100%
non-fatal, 2 detectors  1015    48.8%
non-fatal, 3 detectors  5015     2.2%
```

The corruption is real but **rare and survivable**: 21 RX resynchronisations
across 5400 published frames. The RX path's own resynchronisation absorbs it.

Counters stayed readable in every sample, so the host command lane no longer
dies — the observability collapse was downstream of the halts as well.

### What did not contribute

- **The pipe watchdog fired zero times** (`watchdog_recovered 0`) in the working
  run. The port is vendor-faithful and worth keeping as a safety net, but it
  did not produce this result.
- **`confirmed` equals `published` exactly** (5400/5400), so no buffer is lost.
  The `Used bufs` "leak" chased earlier does not exist.

### Remaining performance gaps

1. ~1.2 Mbit/s is far below what the PHY supports. Rate index 7 is OFDM
   9 Mbit/s and `AGG TXed` is 0, so neither rate scaling nor aggregation is
   working.
2. `retired 401` of 5400 (7.4%) under TCP load, versus 21 of 2646 (0.8%) under
   ping flood. Something about sustained bidirectional load raises the failure
   rate.
3. The hardware/controller reason TX command content first reaches RX RAM is
   still unexplained. The Rust release-head bug that converted that corruption
   into a terminal FIFO stall is fixed and isolated above.

## Summary

**Corrected 2026-08-15 (late session). The earlier headline claim in this file
was wrong and is retained below only as a superseded hypothesis.**

Ordinary class-0 data frames are **not** categorically refused. Measured with
the lifecycle counters 500 ms after the first class-0 publication:

```text
admitted 16   published 16   tx_start 21   completed 17   confirmed 12
retired 3     last_status 0x11   last_expected 0x11
```

The MAC accepts the great majority of data frames and delivers exactly the
expected status `0x11`. A minority fail from the start (`retired 3`). The path
therefore **works and then degrades**, rather than being structurally broken.

The live defect is the one originally reported: **TX command-list bytes reach
the RX FIFO**. Everything downstream of it is now understood and mitigated.

### Fact vs assumption

This file previously stated the refusal as fact. It was an inference from a
single capture taken at an already-wedged moment, generalised into a permanent
property and never re-tested. Several turns of work were spent looking for a
structural reason the MAC would reject every data frame; there is none, because
it does not.

When reading the sections below, treat as **fact** only what is accompanied by
a hardware capture or a quoted vendor instruction, and treat every "therefore"
as an assumption until it has its own measurement.

### Current model of the failure chain

Each link is measured; the ordering is inference.

1. **(open, root)** TX command-storage bytes are written into the RX FIFO.
   Fact: 7 words from command storage `0x0900717c` found at RX FIFO address
   `0x09403f54` inside the producer delta `0x3dac -> 0x402c`
   (`validate_tx_boundary` phase `0x20`).
2. The RX path detects the inconsistency; unguarded it resynchronises, and with
   diagnostics built in **our own detector halts the firmware**. Fact.
3. Frames stop completing, so a pipe stays armed. Fact (pipe cursor capture).
4. **(fixed)** Nothing decremented the pipe watchdog, so an armed pipe was
   unrecoverable. Fact: vendor `FUN_00003bac` decrements `+0xa5`; our arm path
   already reloaded it to 5. Ported.
5. **(fixed)** A refused slot was never retired. Ported as defect B.
6. A stuck host slot in `Owned { Scheduled }` makes
   `management_runtime_available()` false forever, which gates `poll_request()`
   and kills the host command lane, blinding all readout. Fact (code path plus
   reproduced signature).

## What was tried and did not work

Recorded so none of it is re-litigated. Each entry is a hypothesis that was
tested and failed, not an untried idea.

| attempt | outcome |
| --- | --- |
| `pac_phy_start_op(6)` missing from the scheduler prologue | **Dead code in vendor.** Gate `0x04001d39` has no writer in either image. Reads `0x99` on our firmware only because we never initialise that vendor SRAM. |
| Scheduler bit 21 as the cause | Symptom, not cause. It is the ordinary-TX doorbell; the MAC cannot read an SRAM word. Later became relevant for a different reason (the pump). |
| Pipe asymmetry (class-6 on pipe 3, class-0 on pipe 0) | Explained, not a defect. `WSM_QUEUE_BEST_EFFORT` is 0; both vendor tables map queue -> ac -> pipe identically. |
| Missing sequence number assignment | Already implemented and faithful (`assign_sequence_number`). The captured `0x32000000` is correct because the capture halts on the **first** frame, where the counter is legitimately zero. |
| Wrong expected status | `frame+0x56` is `0x11` on both the accepted class-6 and the refused class-0 path. Retired. |
| Routing completions to the slot whose context they name | **Regression.** TXed 95 -> 27, ping 51/448 -> 0/561, `Used bufs` unchanged. Reverted; pinned by `host_tx_policy::route_completion` and its tests. |
| The `Used bufs` "leak" | **Probably never existed.** 7 vs 6 with and without the fix; 3-8 across runs, tracking traffic rather than climbing. Never controlled before being "fixed". |
| Removing the invented `0x040099a9` 5 -> 3 normalisation | 5x worse. Kept off by default. |
| Shared-slot exhaustion as the cause of readout death | Falsified: a build halting after 3 s of blocked output never fired. |
| Pushing counters as debug events to survive the wedge | Channel does not exist in practice: WSM id `0x0805` appears 19 times without `vendor-host-tx-foundation` and **zero** times with it. |
| Plaintext / clear-protected A/B on defect A | **Not run.** Its premise (first frame refused) was falsified before it started. |

## Proven on hardware

Terminal-exception capture `xr819-pipe-cursor-divergence` (image
`5500db794e8f08ce`), taken at the instant the vendor cursor invariant broke:

```text
expected_status (frame+0x56)        = 0x11
statuses the MAC actually delivered = 0x0f, 0x13   (0x11 never once)
slot state at refusal               = 1  (published, GO written, not started)
refused count                       = 4  while TXed = 6
producer=2 last=2 current=2 armed=1 | hardware ring cursor = 3
producer slot 2 -> cmd 0x09007128 frame 0x04008428   (live)
hw slot      3 -> cmd 0x0900717c frame 0x00000000   (never published)
ring+0x18 sentinel = 0                              (advance_slot never ran)
```

`0x0f` is the same status carried by the `0x4140398f` fatal event seen in the
previous session (`event_type 0x39`, `status 0x0f`, bit30 fatal).
`wsm_status_from_internal` has no entry for `0x0f`; it is not a completion code.

## The two defects

### A. The MAC refuses class-0 data at submission (SUPERSEDED)

**This section's premise is false and is kept only to document the error.** See
the Summary: 16 of 16 frames published and 17 completed with status `0x11` in
the first 500 ms.

What the capture below actually shows is a **wedged moment**, not the steady
state. The refusal is real when it happens and a minority of frames do fail
this way from the start (`retired 3` in 500 ms), but it is not the general
behaviour of the data path. Every "eliminated" subsection that follows is still
valid as evidence about *that* capture; none of it supports the claim that data
frames are categorically refused.

The original text follows: the refusal arrives at `slot_state == 1`, the
descriptor is published and GO is written, but `txp_pipe_tx_start` never fires.
Scan, JOIN, EAPOL and association all work. Under load only 7 of 562 attempted
frames were transmitted.

#### The descriptor is not the fault

Image `8e321c0bb1f26264` (`class0-descriptor-dump`) halts immediately after GO
and publishes the assembled command list. Captured for a 590-byte encrypted
ping:

```text
+0x0c 0x51040803   phy rate word
+0x10 0x50000002   phy control
+0x14 0x520f0252   hw rate code 0x0f, len+4 = 0x252 (590 bytes)
+0x18 0x31004188   fc low  -> FC 0x88 0x41 = QoS data, ToDS, Protected
+0x1c 0x47000041   fc high
+0x20 0x20808008   metadata byte, if_id 0
+0x24 0x3200003c   duration
+0x28 0x290136a6   header source = frame + 4
+0x2c 0x32000000   secondary (flags bit0 set -> header+0x16 branch)
+0x30 0x400136b8   payload source = (frame + 0x18) & 0x7ffffc
+0x34 0x00236002   ((590-24) & 0xfff) << 12 | (payload & 3)
+0x38 0x07004600   terminal
+0x3c 0xf0000000   end
dur[1] = 0x00003091 -> frame-kind marker 0x91 = 0x11 + 0x80
frame+4 flags = 0x24c01001
```

Every word matches vendor `txp_submit_to_pipe` (`0xadd0`) for these exact
flags: bit 0 set selects the `0x32 | header+0x16` secondary branch, bit 4 clear
suppresses the retry marker, and `frame+0x0a & 0xf = 8` selects the
`0x29`/`0x40` branch. Descriptor construction is therefore **eliminated**.

#### The rate is not the fault

`pas_rate_to_hw_code` (`0x8348`) is `RATE_ATTRIBUTE[idx]`, and
`RATE_ATTRIBUTE[7] = 15`. The observed hw code `0x0f` is rate index 7, i.e.
legacy OFDM 9 Mbit/s — not an HT MCS. This matters because the association is
explicitly non-HT (`disable_ht=1`, driver reports `HT: off`, rate mask
`0x00003FC0` = OFDM indices 6..13), so an HT rate would have been a plausible
refusal cause. It is not one. **Eliminated.**

#### Eliminated: radio/PHY ownership, VIF masks, PAS accounting, PHY state

Image `5b900ac50aea8b15` publishes one record differencing the class-6
publication the MAC **accepts** against the class-0 publication it **refuses**:

```text
                     class6 (accepted)      class0 (refused)
radio owner          0x04003edc             0x04003edc          same
vif +0x66/+0x2c/+0x2e 3 / 0x8101 / 0x8081   3 / 0x8101 / 0x8081 same
phy cmd 0x04001d2c   5                      5                   same
PHY state 0x04003a6e 4                      4                   same
0x040099a9 reprog    5                      3                   DIFFERS
0x0400994f wake      2                      0                   DIFFERS
0x04009945 arg       0x97                   0                   DIFFERS
sched 0x04001fd4     0x984d7a97             0x986d7a97          bit 21 set
```

This eliminates in one measurement: radio ownership (`lmc_sched_request_radio`
is already performed by JOIN at `vif.rs:315`), VIF active/effective masks, the
global active count `0x04008f76` `0 -> 1` transition and its
`phy_state_advance(0)` call (both implemented at `vendor_host_tx.rs:667` and
`phy.rs:1837`), and the PHY state machine itself — state is 4 on both sides, so
the `state == 1` wake branch is never reached.

The three differing bytes are exactly what `advance_awake_station_tx` writes.

#### Falsified: removing the invented `0x040099a9` normalisation

`advance_awake_station_tx` contains a write with no vendor basis — vendor
`phy_state_advance` (`0x820a`) only *reads* `0x040099a9`:

```rust
if retained_state == 5 { write_u8(0x0400_99a9, 3); }
```

Since the accepted class-6 path publishes with retained state 5, removing this
looked like the fix. It is not. Measured over the same 25s flood:

| image | TXed | loss |
| --- | ---: | ---: |
| `unmatched-tx-status-recovery` only | 54 | 94.5% |
| plus `phy-advance-vendor-exact` (normalisation removed) | 10 | 99.65% |

Removing it makes things markedly worse, so the byte is load-bearing even though
it is not vendor-shaped. Kept behind `phy-advance-vendor-exact`, **off by
default**. Do not remove it again without new evidence.

#### One stuck class-0 frame makes the firmware stop answering the host

The observability collapse is not a separate load bug. It follows directly from
defect A:

1. A class-0 frame is published and the MAC never completes it, so its slot
   stays `Owned { phase: Scheduled }`.
2. `HostTxDriver::management_runtime_available()` returns false for any slot in
   `Reserved` or `Owned { Scheduled }`, so it is now permanently false.
3. `poll_request()` is gated on it, so **host requests are never polled again**
   and MIB reads go unanswered (driver reports zeros, `BH errcode 1`).
4. `host_request_waiting` therefore stays true, which starves both the debug
   event publish and the radio indication publish, since both are gated on
   `!host_request_waiting`. The trace ordinal freezes.

Measured signature, reproduced on every run:

```text
sample        trace ordinal   counters MIB
pre                     172   readable
after-flood             188   zeros
after-iperf             188   zeros
```

A `hif-stall-dump` build that halts once `output_available()` has been false for
three seconds **never fired**, which rules out shared-slot exhaustion and was
what pointed at the request gate instead.

The defect-B retirement does not rescue this, because retirement runs from
`service_txp_pipe_tx_status`: it needs a *further* delivered status to trigger.
When the MAC stops delivering statuses for the wedged pipe, nothing drives
retirement, so the slot stays `Owned { Scheduled }` indefinitely.

The design consequence is worth stating separately from defect A: a stuck TX
frame should never be able to stop the firmware answering host commands. The
command lane is gated on TX progress, so one refused frame takes out all host
visibility, which is why every measurement after t+25s this session was blind.

#### Readout stops before the interesting part of every run

Both host-visible readout paths fail at the same moment, roughly when the ping
flood ends:

```text
sample        trace ordinal   counters MIB
pre                     158   readable (marker present)
after-flood             191   all zeros, BH errcode 1
after-iperf             191   all zeros
```

The driver still reports `BH status: alive` and `WSM retval: 0` throughout, so
this is not a detected crash. The firmware simply stops answering MIB reads and
stops emitting indications.

Pushing counters as debug events was tried as a way around the MIB failure and
does **not** work: WSM id `0x0805` debug events appear 19 times in a build
without `vendor-host-tx-foundation` and **zero** times in every build with it,
including runs that predate the push emitter. Debug events are never published
at all in foundation builds, so that channel cannot carry diagnostics for the
host TX path.

The consequence is that nothing can currently be measured during or after the
flood, which is exactly when defect A occurs. Adding more instrumentation does
not help while both channels are dead; the reason the firmware stops responding
under load is the blocking issue.

#### Class-0 lifecycle counters, and which of them mean anything

`class0-lifecycle-counters` reports funnel counts through the counters MIB
(`tools/decode-lifecycle-counters.py`). Two runs of the same image show which
counters are instruments and which are weather:

```text
counter             run A  run B   stable
tx_start                9      9   yes, exact
completed               4      5   yes, +-1
status_delivered       49     93   no, ~2x
status_ineligible      45     88   no, ~2x
TXed (throughput)       9     26   no, ~2.9x
```

The funnel stages (`admitted`, `published`, `tx_start`, `completed`,
`confirmed`) are stable because they count transitions of our own frames.
`status_delivered` and `status_ineligible` count MAC status deliveries driven by
ambient radio activity and are as noisy as throughput.

`status_ineligible` is **not** a fault signal. A healthy pre-traffic sample
measured 45 ineligible of 49 delivered while completing normally: the gate
refuses every status that does not belong to the slot under service.

#### The OTA harness cannot resolve anything smaller than a ~3x effect

The same image, flashed and run twice through `xr819-cursor-run.sh`, produces:

```text
image        run  TXed  ping received
2233cccc       A    95  51/448 (88.6% loss)
2233cccc       B    36  21/524 (96.0% loss)
```

plus an earlier run of near-identical code at TXed 54. Identical firmware
therefore spans **36-95 TXed**, a 2.6x spread, and the ping flood adapts its
rate (448 vs 524 vs 1437 packets sent) so loss percentages are not comparable
across runs either.

Consequences for how this work is measured:

- A single run cannot qualify or disqualify a change. Any conclusion of the
  form "TXed dropped, therefore my change regressed it" needs the control run
  of the unmodified image *in the same session*.
- Only effects larger than roughly 3x are detectable at all. The defect-B fix
  (Pending TX 4 -> 0, a state change rather than a rate) was measurable; small
  throughput changes are not.
- Prefer static evidence when it is available. Comparing the packed images of a
  pure refactor (4 differing bytes out of 75864, identical size) settled a
  suspected regression in seconds where the harness could not have.

#### Do not route completions to the slot whose context they name

`service_host_class0_runtime` drains one global completion queue but is called
from inside per-slot servicing, so a completion often names a slot other than
the one being serviced. That looks like a leak: the completion is traced
(`0x48543f00`) and dropped, apparently stranding a slot in `Owned` and never
returning its HIF request buffer.

Routing the completion to the slot that owns the context was measured over the
air against the same harness and environment, and is a **clear regression**:

```text
                TXed  RXed  ping received      Pending TX  Used bufs
unmodified        95   138  51/448 (88.6% loss)         2          7
routed            27    43   0/561 ( 100% loss)         2          6
```

It also fails on its own terms: `Used bufs` is unchanged (7 vs 6, noise), so
the discard is **not** the buffer-accounting problem. `Used bufs` sitting at
6-7 of 30 with `Pending TX` 2 appears to be ordinary in-flight state rather
than an unbounded leak, and the earlier "climbs" observation was never
controlled against an unmodified run.

A completion naming another slot must therefore not be treated as that slot's
completion. The reverted code is kept described here so the same fix is not
reattempted.

#### Build hazard: the OTA feature set

`unmatched-tx-status-recovery = []` enables nothing on its own. Building with
only that feature yields an image with no host TX driver and, because
`vendor-host-tx-foundation` also gates `join-sta-experiment`, no STA JOIN path.
The image flashes and boots but never associates, which at harness level is
indistinguishable from a firmware regression.

`cargo test` does not catch this. The host TX service path is
`#[cfg(target_arch = "arm")]`, so the host test build never compiles it; a
change that does not even type-check for ARM can still report a fully green
suite. Use `tools/build-ota-image.sh`, which pins the canonical feature set and
always performs the ARM build.

#### Vendor SRAM is left uninitialised by the open firmware

The `txp_scheduler_run` prologue gates `pac_phy_start_op(6)` on
`0x04001d39 & 1`. No code in either the main or TCM image writes that byte, so
the branch is dead in vendor, where startup BSS init zeroes it. On our firmware
it reads **0x99**: `clear_rust_bss()` clears only the Rust BSS, and our startup
initialises the vendor PHY command block at `0x04001d20` only at `+0x0c`,
`+0x10`, `+0x18`, `+0x1c` and `+0x21`.

The practical consequence is general, not specific to this branch: any vendor
logic translated later that reads vendor SRAM we never initialise will branch on
leftover garbage. Before translating a gated vendor path, check both that
something writes the gate and that our startup initialises it.

`pac_phy_start_op(6)` is therefore **eliminated** as a cause of the class-0
refusal, and so is its "all four pipes idle" precondition — the measured idle
mask never reaches `0xf` on either path.

#### Open lead: class-6 and class-0 use different pipes

The idle-pipe masks from the same capture decode as:

```text
class6 (accepted) idle mask 0x7  -> pipe 3 armed
class0 (refused)  idle mask 0xe  -> pipe 0 armed
```

The accepted and refused paths do not share a pipe, and pipe 0 is the pipe that
wedged in every failure recorded here.

This is most likely **not** a defect. `WSM_QUEUE_BEST_EFFORT` is 0, so a ping
correctly arrives as queue 0. The firmware maps it queue -> ac -> pipe through
two tables that vendor uses identically:

- `ctx + 0x60 = [0x040002dc][queue_id]` (vendor `0x4949`, our admission path)
- pipe `= [0x040002e0][ac]` (vendor `0x4813`, our reservation path)

With both tables `[1, 0, 2, 3]`, BE gives ac 1 and pipe 0, while internal
management lands on pipe 3. Hardware pipe order simply does not match WSM queue
order. The `EDCA(0) = 3, 7, 2, 1504` line in driver status is printed in
**mac80211** queue order (0 = VO), not WSM order, so it does not indicate that
data is on the voice queue.

Still worth confirming: that vendor initialises both tables to `[1, 0, 2, 3]`
rather than those values having been inferred.

#### Frame-node differential (record "C0FN")

Diffing the accepted class-6 frame node against the refused class-0 frame node
is mostly uninformative **by construction**: the two are genuinely different
frames, so differing length, flags, rate and selector fields carry no signal.

```text
field                 class6      class0
header address     0x090157e8  0x09013042
flags              0x00801008  0x24c01001
len | fc           133/0x0188  590/0x4188
sel|kind|rate|kind 3|6|6|0x11  1|4|7|0x11
ownership +0x2c    0x00000103  0x00000161
policy|link|class  0|0|6       0|0|0
```

Two results do matter:

- **`+0x56` is `0x11` on both paths.** The frame the MAC accepts expects the
  same completion status the refused frame never receives, so the expected
  status is not miscomputed. This retires the "wrong expected status" family of
  explanations.
- **Ownership `+0x2c` differs**: class-0 sets bits 5 and 6 (crypto-complete,
  ready-frame) which class-6 lacks, and lacks bit 1 which class-6 has.

**False alarm, recorded so it is not re-chased.** The descriptor's secondary
command is `0x32000000` for class-0, i.e. the sequence-control immediate read
from `header + 0x16` (vendor `txp_submit_to_pipe` line `12974`) is zero. This is
*not* a missing sequence assignment: `assign_sequence_number`
(`vendor_host_tx.rs:289`) already implements vendor `tx_assign_seq_num`
(`0x0ee4`) faithfully, under the same flags bit-29 guard, and the capture halts
at the **first** class-0 publication, where a counter starting at zero correctly
yields sequence zero.

#### Remaining unexplained: scheduler bit 21

Scheduler word `0x04001fd4` bit 21 (`0x00200000`) is **set** at class-0
publication and clear at class-6. The task handler table resolves that bit to
`0xb88f`, i.e. **`task_b88e`**, the ordinary TX task — not a completion sweep as
the surrounding annotation guesses. The class-6 path consumes the bit through
`claim_scheduler_mask_atomic(1 << 21)`; the class-0 path raises it and nothing
services it, because `task_b88e` is not implemented.

This is a symptom rather than a cause: the bit lives in SRAM and the MAC cannot
read it. It does document a real architectural divergence — the open firmware
publishes class-0 directly through the probe publisher instead of through the
vendor scheduler task.

#### Superseded hypothesis: radio/PHY ownership is never established for data

What remains is the state *around* publication rather than the descriptor
itself. Management, probe and EAPOL frames reach the MAC through the scan/probe
path, which performs the PHY operation and timer setup before publication;
ordinary joined data does not. The vendor establishes radio ownership through
`lmc_sched_request_radio`, `task_22bc`, `lmc_sched_radio_release` and
`phy_state_advance` before ordinary class-0 `txp_scheduler_run()`. None of that
is translated, which would explain a MAC that accepts management descriptors and
refuses data descriptors built by the same code.

The existing `joined-data-no-phy-start-diagnostic` feature is evidence the two
paths already differ here.

Still unexamined: data lands on **pipe 0** rather than BE pipe 2, which looks
anomalous given `0x040002dc = [1, 0, 2, 3]`.

Ruled out by direct comparison against the vendor decompilation:

- the command list itself is word-for-word vendor-shaped
  (`txp_submit_to_pipe` `0xadd0`; masks, `0x7fffff` operands and the
  `0x07004600`/`0xf0000000` tail all match)
- `frame + 0x56 = 0x11` is the correct expectation
  (`pas_compute_tx_timing` `0x7fa6`)
- the frame kind is programmed to hardware correctly
  (`desc_or_flags` `0x90ba` is `*(cmd+4) |= kind + 0x80`)
- packet-DMA and pipe initialisation match `mac_hw_reset_regs` `0xbc`,
  `mac_hw_init_pipes` `0xf564`, `mac_program_base_regs` `0x10a2c`,
  `mac_program_timing_regs` `0x10982`

### B. A refused frame leaked the slot forever (fixed)

`plan_ordinary_tx_pipe_status` returns `Ineligible` and returns — no retire, no
fail, no release. Vendor has the same structure but never observes `0x0f`, so
there is no vendor behaviour to copy.

Consequence chain, each step observed:

1. slot stays `armed = 1`, software producer frozen at 2
2. hardware ring cursor advances to 3 (frame node NULL)
3. the controller re-executes a stale command list still pointing at a recycled
   HIF request buffer (`0x0900b722` = RX buffer #7 + 0x1a)
4. that stream — setup commands, `0x29` header source, `0x40` payload, the
   `0x07004600` tail, then the MPDU — lands at the RX producer

Fix: `plan_unmatched_tx_status_retirement` (feature
`unmatched-tx-status-recovery`). It uses the **hardware ring cursor as the
completion authority** rather than a timeout: once the cursor has moved past
`current`, the hardware is demonstrably finished with the slot, so the frame is
completed with vendor give-up status `0x0b` and the cursors retire through the
same writes the vendor success path uses. This restores the invariant asserted
at the end of `txp_fn_4425` (`0x38c`):

```text
pipe_state[0xa0] == (ring[0x20] & 0x3fffffff) >> 27
```

## Measured effect of the B fix

| metric | before | after |
| --- | ---: | ---: |
| TXed | 7 | 54 |
| ping received | 2 / 562 | 28 / 509 |
| Pending TX at end | 4 | 0 |
| Used bufs at end | 4 | 8 |

The pipe cycles instead of pinning. Two residuals remain:

1. `Used bufs` still climbs — some retired frames produce no host confirmation,
   so HIF request buffers are not returned.
2. The flight-recorder cursor goes `938 -> 0` across the flood, i.e. the
   firmware restarts under sustained load.

## Instrumentation notes

- The counters MIB is **useless after a wedge**: it answers all-zero. Never
  diagnose a wedge by polling it. Use the terminal-exception path
  (`hif::publish_terminal_exception`, surfaced in debugfs `bh_rx_trace`), which
  reports at the transition itself.
- `host_tx_diagnostics::populate_counters` returns the TX identity snapshot
  whenever `identity[0] == 0x58544944` ("XTID"), which hides the flight recorder
  during traffic. Identity slot 21 now carries the packed pipe-cursor word.
- Useful features: `pipe-cursor-assert` (halt + 18-word report at first
  divergence), `unmatched-tx-status-recovery` (the B fix),
  `pipe-cursor-resync` (**do not use** — writes the hardware cursor backwards to
  the stale software producer; kept only as a record of a wrong turn).

## Test harness

All prior `xr819-*-cold-iperf.sh` scripts ran `iperf -c 127.0.0.1`, i.e. pure
loopback inside the netns. They never exercised the radio; their throughput
numbers are meaningless. `/tmp/xr819-cursor-run.sh` drives real over-the-air
traffic: ping flood plus an `iperf` client in the wifi netns against a server
bound to the board's wired `end0`, so the wifi leg is XR819 -> AP -> switch.
The development host cannot serve (NixOS firewall active, no passwordless
sudo, and the host must not be modified).

Reproduction: the fault needs roughly 30 class-0 frames. A 10 s ping burst can
pass by luck; use the 25 s flood.
