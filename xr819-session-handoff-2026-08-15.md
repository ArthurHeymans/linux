# XR819 firmware session handoff — 2026-08-15

Read this first, then
[`xr819-class0-tx-status-findings.md`](xr819-class0-tx-status-findings.md) for
the detailed evidence, then
[`xr819-hif-startup-flow.md`](xr819-hif-startup-flow.md).

Supersedes [`xr819-session-handoff-2026-08-10.md`](xr819-session-handoff-2026-08-10.md).

## Current resolved state — read before the historical ledger

The intermittent terminal RX/MAC collapse is resolved by a Rust RX ownership
fix. Under `corruption-non-fatal`, a corrupt RX release-head ownership word was
changed to pending and the function returned. Because that slot was already the
head, no later owner could revisit it; `DMA_CONSUMER` remained pinned and RX
stopped permanently. The release path now continues through normal head
reclamation after normalization.

A second correctness fix prevents RX resynchronization from advancing the
hardware consumer across outstanding zero-copy HIF slots. It was not the
terminal-collapse trigger in isolation, but it closes a real ownership
violation. Strict TX status ownership, independent pipe-watchdog configuration,
atomic scan scheduler-bit updates, and vendor ACK-list initialization order are
also retained.

Validated results:

- combined FIQ/RX fixes: 3/3 healthy, TCP 4.34-4.80 Mbit/s, UDP delivered
  7.91-7.93 Mbit/s;
- the same RX fixes without FIQ: 3/3 healthy, TCP 3.85-4.93 Mbit/s, UDP
  delivered 7.92-7.93 Mbit/s;
- restoring only the old corrupt-head return: 1/3 healthy, with two terminal
  0/20-ping collapses;
- restoring only the old resync jump: 3/3 reachable.

FIQ is not required for the production fix. Its complete implementation and
failure history are preserved on Jujutsu bookmark `feature/mac-fiq` at commit
`0d4c3ee4`; do not delete or conflate that branch with the clean production
line based on validated commit `12025526`. Final production image
`88298e828ef298af0644a4afe0dd0c206849061bc6b501f800bfdb5ca29f4a53`
qualified healthy 3/3 at 4.87-5.09 Mbit/s TCP and 7.92-7.93 Mbit/s UDP, with
20/20 final ping in every run.

Everything below is an evidence ledger. Sections labelled rejected, superseded,
or historical are not active implementation directions.

## Verdicts contaminated by the rx_rate defect — treat as UNTESTED

Everything measured between the `rx_rate` raw-byte change and its clamp ran with
a broken RX path that dropped frames and killed association. Dead runs were that
defect's signature, and they were read as experiment verdicts. Re-test before
citing any of these:

- **`wide-service-budget` "catastrophic"** — wrong. Re-tested on the fixed
  baseline it is simply *neutral*: UDP delivered 1.25/1.28 against baseline 1.29,
  `TXed` 7447/6670 against 6617. One 2.33M TCP reading looked like a 2x win and
  was an outlier; TCP spans 989K-2.33M inside a single arm, so UDP delivered and
  `TXed` are the metrics to use.
- **`loop-rate-counter` "breaks association"** — same window, same signature.
- **The original "TX pipelining is broken" verdict** came from this window and
  was invalid. It has since been re-tested on the fixed baseline with corrected
  vendor publication order and is genuinely broken for the independently
  measured completion/retry ownership reasons below.

## Cleared as causes of the publish -> start delay

- **EDCA contention window.** The mask at PAS `+0x4bc` is written only at join
  from CWmin at `+0x4cc`, and the host's defaults are 0x0003..0x000f
  (`sta.c:65-69`), which is tens of microseconds. We never grow the window, so
  vendor's post-success `pas_backoff_reset` (`annotated-main.c:10005-10026`) has
  nothing to restore.
- **PHY command 2.** Dispatched while handling the already-raised start event
  (`annotated-main.c:11911-11925`), so it cannot be inside the measured interval.
- **A periodic gate.** None found on the GO path; ring fields carry command
  pointer, stride, GO and cursors, not a scheduled time.

## CORRECTION: the proposed AIFS fix was backwards

Vendor copies the raw 44-byte WSM EDCA payload into PAS `+0x4cc`
(`annotated-main.c:24040`). The host serialises each field in q3, q2, q1, q0
order (`wsm.c:607-628`), so PAS `+0x4dc..+0x4df` contain q3..q0. Vendor's
packing formula at `annotated-main.c:24041-24043` therefore uses q2, q0, q1,
q3, exactly matching the original Rust `wire[1], wire[3], wire[2], wire[0]`
expression. Stock defaults produce `0x1162`; the short-lived `0x6211` change
was wrong and was reverted before any image containing it reached hardware.

## Measured correctly: iperf latency is queueing plus an 8.4 ms service cycle

The old `1837/2845/6/11` split was one final frame. A temporary `C0LS` layout now
accumulates four full-width sums and an exact sample count, and the decoder
selects the last complete marker-bearing MIB block instead of the first block or
a later failed all-zero read. It also latches the **first** start after GO;
previously every retry overwrote that timestamp, hiding retry time inside the
apparent pickup delay.

One healthy PHY-command-1 run gave the iperf-only delta over 4367 frames:

```text
admission -> publication                 39.50 ms  (queueing behind other frames)
publication -> first start                6.50 ms  (initial MAC acquisition)
first start -> final completion           1.91 ms  (retries/backoff included)
completion -> confirmation-ready          0.014 ms
```

The 39.5 ms is not inverse throughput: it is several host frames waiting behind
the single hardware owner. The serialized service cycle is ~8.42 ms, matching
~110 frames/s during the 40 seconds of iperf.

Forcing both random backoff fields to zero reduced first-start time only
6.50 -> 5.48 ms, retry time 1.91 -> 1.79 ms, TCP 1.07 -> 1.16 Mbit/s and UDP
delivered 1.39 -> 1.43 Mbit/s. The contention field participates but does not
explain the gap. The diagnostic feature was removed after measurement.

## PHY command 1: real reliability lead, throughput-neutral so far

Vendor calls `pac_phy_start_op(1)` once before every non-empty scheduler pass
(`txq_build_aggregate_lists`, `annotated-main.c:12400-12406`). Our ordinary
scheduler omitted it despite already carrying an exact Rust implementation.
The feature `phy-start-before-scheduler` now calls it once before the first
reservation attempt in a pass and never restarts it within that pass.

Interleaved result:

```text
control                       0/2 valid, TXed 175 / 669, fell to 6 Mbit/s
PHY command 1                 2/2 valid, TXed 6936 / 6863, stayed at 48 Mbit/s
PHY1 publish -> start         7.36 / 8.13 ms (control healthy run: 8.05 ms)
PHY1 TCP / UDP delivered      1.21-1.23 / 1.36-1.37 Mbit/s
```

This is promising for association/link stability but does not improve steady
throughput. Two runs per arm are not enough to make it normal behavior yet.

## Depth-2 ordinary batching now has a closed lifecycle

The earlier statement that bulk BE has zero TXOP was wrong. Live status proves
queue 2 carries the traffic (6706/6708 frames; other data queues zero). Host
queue 2 maps through AC 2 and pipe 2 to wire-ordered EDCA selector 2, whose live
TXOP is **3008 us**. Vendor's mode-1 branch can therefore batch ordinary legacy
frames even though station dump reports 54 Mbit/s; A-MPDU is not required.

The publication path is now vendor-ordered and host-tested:

1. build all descriptors while GO is zero;
2. one pipe trigger;
3. publish every staged slot and duration;
4. arm once;
5. GO once.

That exposed two independent depth-1 assumptions. Both are now repaired for the
ordinary legacy case.

### Completion ownership

A context pointer alone is not authority: routing by context previously freed a
buffer still referenced by a live MAC slot, reducing normal completions
~2440 -> 866 and producing 1949 give-ups. The backend now registers the exact
`(context, frame_node, pipe, slot)` before GO, queues up to four completions from
one vendor completion-ring drain, and the host driver confirms only a Scheduled
owner whose complete identity matches. MAC service runs once per driver pass,
then every completion produced by that pass is routed; it is no longer drained
from whichever per-context service happened to run first. Retry counts are read
from each frame rather than one global batch counter.

### Ordinary later-slot retry

Vendor permits `producer != current`. For an ordinary kind-0 slot with PAS flag
bit 15 clear, the mapped companion is another distinct ordinary PAS, so the
vendor branch clears only the current hardware command-mask bit and acknowledges
without forcing producer to follow current (`annotated-main.c:11540-11566`). The
Rust rearm now implements that bounded branch. Status-6/A-MPDU cursor coupling
remains explicitly unsupported.

### Hardware closure

Safe depth-2 image `d1b37e8e672a1edf781a8299ab9e2bb0e283ce473def9f8413f5adeab189c5bf`
completed the eight-second flood with:

```text
ping                              719 / 740, 2.84% loss
admitted / published              742 / 742
normal completions / give-ups     740 / 2
confirmed                         742
TX starts                         1039 flood delta (retries included)
queued / pending / used buffers   0 / 0 / 0
```

The strict status gate refused 1391 of 2131 flood-delta status events, yet every
published frame still reached normal completion or bounded give-up. This proves
those refusals are routine intermediate statuses rather than the prior batch
blocker.

Two earlier diagnostic runs must be qualified:

- ownership image `6646a9a7...` delivered 629/645 pings and closed all driver
  queues, then hit the intentional `xr819-rx-resync-owned` halt after the
  measurement;
- counter image `edd121d9...` showed an exact coherent prefix of 273 admitted,
  published, completed and confirmed with zero give-ups, then halted mid-flood
  on `xr819-hif-tx-boundary` because `tools/build-ota-image.sh` had accidentally
  omitted the established `corruption-non-fatal` feature.

The OTA builder now includes `pipe-watchdog`, `corruption-non-fatal`, and
`host-lane-independent` in its canonical base, preventing that self-inflicted
invalid image. `tools/decode-lifecycle-counters.py` also recovers the last
complete pushed C0LC burst when the post-traffic MIB returns zeros.

`tx-pipelining` remains off by default. Its depth-2 ownership/status lifecycle
is repaired, but **ordinary batching is now ruled out as the missing 10x
throughput mechanism**.

The first implementation armed too early: 746 frames produced 745 GO operations
and only one real two-slot batch. Separating pending/PAS progression from the
scheduler made batches form whenever two PAS owners were ready. Capacity
instrumentation then proved the scheduler itself is not dropping opportunities:
1081 passes began with at least two PAS owners, all 1081 staged a second slot,
and none failed reservation.

Bulk throughput remained baseline-like as batch coverage rose:

```text
configuration                    frames in 2-slot batches   TCP       UDP delivered
ordinary depth 2, natural load              13.3%           1.18M      1.33M
ordinary depth 2, another run               33.3%           1.09M      1.35M
4 ms PAS collection window                  71.9%           0.99M      1.39M
matched healthy depth-1 control               0%            1.04M      1.33M
```

The 4 ms window reduced GO operations from one per frame to 3902 arms for 6094
frames, yet did not raise throughput and slightly hurt TCP. It is reverted. Do
not spend runs on depth 4: one-GO ordinary slots do not remove the dominant
per-frame cost in this implementation.

The empty `pas_backoff_reset` callbacks were investigated and are **not the
throughput gap**. A temporary read-only C0BO diagnostic sampled the retry count,
current CW and configured CWmin immediately before both callback sites. A
healthy eight-second flood produced 745 observations, only two expanded-window
samples, maximum CW 15 and retry count zero throughout. Per-queue follow-up
showed the actual callback mapping clearly:

```text
interface 0, queue 1 data observations   1056
queue 1 expanded observations               0
queue 3 setup/management observations       5 (2 expanded)
current / expected data CW               15 / 15
maximum retry-CW update count                0
```

Thus the earlier host queue number 2 is not the queue argument passed to
`pas_backoff_reset`; the vendor queue table maps this traffic to callback queue

1. More importantly, data CW is never enlarged at the reset point, so restoring
CWmin cannot remove milliseconds from the service cycle. Two write-enabled reset
runs and the higher-volume read-only run became invalid under traffic; discard
their throughput data. Both temporary reset/snapshot features and the C0BO
decoder were removed after extracting the state result.

## SELF-INFLICTED: unvalidated rx_rate was dropping our own RX frames

The `rx_rate` "fix" earlier in this session passed a raw descriptor byte
(`trailer+6`) straight into the WSM indication. `txrx.c:1241-1247` treats any
value >= 14 as HT with `rate_idx = rx_rate - 14`, and mac80211 at v6.18
**drops the frame** with `WARN(status->rate_idx > 76, "Rate marked as an HT rate
but passed status->rate_idx is not an MCS index [0-76]")`. A stray byte therefore
destroyed a received frame, and during association that meant losing the unicast
auth response: runs sat at AUTHENTICATING, then `BH status: terminated`,
`Pending TX: 28`, `TXed 0`.

```text
                       mac80211 WARNs   associated   TCP          TXed
raw byte (before)                  31       2 of 4   dead         0-251
clamped to 0..=21 (after)           0       4 of 4   1.05-1.23M   6910-7091
```

The old hard-coded 0 was wrong but harmless; the replacement was more accurate
and destructive. Clamping keeps real rates (0..=3 legacy direct, 4..=13 legacy
`-2`, 14..=21 HT MCS0..7 for this 1x1 chip) and reports unknown instead of
discarding the frame.

**This explains, and retracts, a lot:**

- "The environment moved" — asserted twice, wrong both times. We were dropping
  our own RX frames.
- The 6-of-6 regress failure "including the known-good control": `resync` was
  built after the `rx_rate` change and carries the same defect, which is why the
  control died too and the bisect looked like pure noise.
- Chronic association flakiness across the session, depending on whether a
  garbage byte landed during the auth exchange.

## The throughput gap, measured on a healthy baseline

Three valid runs, counters de-aliased, `give_up` now counted:

```text
admitted 6889-7070 == published == confirmed, completed 6892-7073, give_up 0-1
```

Nothing fails and nothing is stuck: every accepted frame publishes and completes.
So **"71% of frames fail on air" is retracted** — it came from a run degraded by
the `rx_rate` bug. The gap is *rate*: ~7000 frames per run against vendor's
~59000, with ~100% per-frame success. The 62-65% UDP loss is simply the host
offering more than we accept.

Per-frame budget:

```text
admit -> publish (our driver)      1837 us   39.1%   (pending gating 1805 us)
publish -> start (MAC pickup)      2845 us   60.5%
start -> complete (AIRTIME)            6 us    0.1%
complete -> confirm                   11 us    0.2%
total                              4699 us   -> ~210 frames/s
```

Airtime is **6 us**. Essentially the entire budget is our own overhead. Vendor's
~1300 frames/s implies ~770 us per frame. The two targets are the 2845 us MAC
pickup delay after publication and the 1805 us pending gate, in that order.

## CORRECTED: the gap is failed TX, not stalled TX (RETRACTED - see above)

A subagent's static analysis overturned the "watchdog never fires" reading, and
the load-bearing claims were verified by hand afterwards.

`COMPLETED` is bumped in exactly one place, the matched-success path
(`tx.rs:2132`). The retry give-up path bumps nothing: it calls
`complete_give_up(..., 0x0b)` (`tx.rs:3346`) with no counter at all. The driver's
`TXed` likewise counts only successes, since `cw1200_debug_txed()` sits inside
`if (!arg->status)` (`txrx.c:1071-1074`).

So `published - completed = 1833` was never "frames stuck where the watchdog
cannot see them". It is **failed TX**, reported honestly to the host as give-up
status `0x0b`, and the watchdog's zero recoveries are correct behaviour rather
than a defect. 1833 of 2578 is a **71% failure rate**, which is what the 60-75%
UDP loss measured all session has been saying.

Retracted: "the watchdog is inert and nothing gives up on abandoned frames".
The give-up path does nearly all the work; we simply never counted it, and the
absence of that one counter produced a confident wrong diagnosis.

Also corrected: our watchdog's status-`0x0b` retirement is **not** established as
vendor-exact. Vendor's visible recovery pass assigns internal status `0x18`
(`annotated-main.c:19877-19978`), and `FUN_00003bac` is absent from the
decompilation (the export jumps `0x3b54` -> `0x3d38`), so its expiry branch
cannot be cited. The comment claiming vendor-exactness overstates what is known.

Vendor mechanisms that do exist, for the record: retry exhaustion reporting
`0x0b` (`annotated-main.c:11384-11416`), software-owned pending expiry completing
status 10 (`13695-13780`, `12404-12417`), ~205ms frame lifetime (`15392-15449`),
and an 8000-tick HIF confirmation-coalescing timer that only releases existing
completions (`17423-17580`). No per-published-frame host-confirm deadline was
found.

### Measurement noise makes small A/Bs worthless

An interleaved 6-run bisect of resync / fatal-fix / post-cleanup was
inconclusive because the *control* spanned `TXed` 0 to 3069 across two runs, and
6827 to 16 earlier in the session on identical bytes. The spread within one arm
exceeds the spread between arms. Any comparison here needs 5-10 runs per arm.
Association itself succeeds about 85% of the time, so dead runs must be discarded
rather than averaged.

### Next

`GIVE_UP` now counts `0x0b` at `tx.rs:3346` under `watchdog-visibility`, so the
dominant outcome is visible for the first time. Then the question is why ~70% of
transmissions are unacknowledged while vendor sits at 0-3% loss on the same
signal: rate selection, ACK reception, or frames the AP drops. The earlier
station dump showed our TX bitrate at 36-48 against vendor's 54.

## ESTABLISHED: concurrent cw1200 driver activity wrecks us, vendor shrugs

Same binary `179e8b5c`, one probe every 2s across the traffic phases, identical
subshell/sleep/fork scaffolding in every arm. Ping loss is the discriminator;
UDP validation is unreliable in these runs so TCP, ping loss and `TXed` are the
signals.

```text
probe                            ping loss     TXed
none (clean baseline)                 2.4%     6827 / 3095
/proc/uptime (unrelated file)     1.7-2.0%     2995 / 3641 / 2553
cw1200 counters (WSM command)    11% / 76%     2337 / 293
cw1200 status  (no device I/O)       99.6%     66
vendor firmware + status probe       0.06%     60173, TCP 21.6M, 0% loss
```

Ruled out, each by its own arm: CPU contention (a busy loop costs ~4% and
nothing else), the periodic wake/fork/IO of the probe itself, and the WSM
command specifically, since the no-device-access `status` read is the most
destructive arm of all.

What remains is that *touching cw1200 driver state concurrently with traffic*
degrades us by one to two orders of magnitude while vendor firmware is untouched
by the identical probe. `cw1200_status_show` reaches no device: it prints cached
state under `spin_lock_bh(&priv->tx_policy_cache.lock)` and
`spin_lock(&priv->wsm_cmd.lock)`. The firmware cannot see a host lock, so the
only channel is delayed SDIO/BH servicing, and `spin_lock_bh` blocks the softirq
path directly where a busy loop merely competes for CPU.

**Lead hypothesis for the throughput gap:** we are intolerant of short stalls in
host BH servicing where vendor rides through them. The BH is the path all
traffic flows through at load, which is where our 10x gap lives.

Note the variance: within one arm `TXed` ranged 293 to 2337 and ping loss 11% to
76%. Damage is graded and probabilistic, so single runs cannot separate arms
here; use ping loss across several runs.

## VOID: everything measured with the mid-traffic sampler

A sampler that read the driver's debugfs `status` every 2s during traffic
**destroys our firmware**, while vendor is unaffected. Same binary `179e8b5c`:

```text
with sampler       TXed    16   no association, dead link
without sampler    TXed  6827   TCP 1.22M, UDP 1.40 Mbit delivered, VALID
vendor + sampler   TXed 60173   TCP 21.6M, UDP 8.64 Mbit, 0% loss
```

Void, and not to be cited: the "silent wedge" (`TXed` frozen at 23 with the
firmware alive), the funnel snapshot showing `completed` frozen at 1417 against
2931 published with 89131 rejected statuses, and the verdicts on two attempted
fixes (`release()` early return, `publish_host_class0_slot` halt removal) that
were judged against this poisoned baseline. Their code-level reasoning may still
hold, but no measurement supports it.

This firmware was *already documented* as instrumentation-fragile: one MMIO read
per main-loop iteration killed association. That warning was about firmware-side
probes, and I did not extend it to a host-side probe. Any new instrument must be
A/B'd against the same binary without it before its output is trusted.

## The sampler is a reliable reproducer of a real defect

A control-path read concurrent with TX load reduces us from 1.22 Mbit/s to no
link at all, reproducibly, and does nothing to vendor at 21.6 Mbit/s. Normal
driver activity issues such commands, so this is a genuine defect and not merely
a measurement artifact.

It is the cheapest failure reproducer found all session: deterministic, fast, and
with a vendor control that stays healthy. It also suggests the command lane is
not independent of the TX lane under load, which the `host-lane-independent`
feature was supposed to address.

## THE STALL: completions stop, not the firmware (VOID - see above)

Sampling the driver's queue state *during* traffic, instead of after a phase
when queues have drained, changes the diagnosis completely.

```text
ours    txed = 23 23 23 23 23 23 23 23      pending=6   used=4
vendor  txed = 10666 -> 51772 climbing      pending=0-2 used=0-9
```

We do not transmit slowly, we stop. But the firmware is **not** hung: counters
stay readable and keep advancing.

```text
admitted             2935
published            2931
completed            1417   <- equals the driver's TXed exactly
confirmed            2930
status_delivered    89131   <- ~30 delivered statuses per published frame
retirement_deferred 23747
suppressed_exception   84   <- MAC fatal events, survived rather than hung
```

**Frames publish and never complete.** The MAC keeps delivering statuses, we keep
finding them ineligible, `completed` freezes, the host receives no confirmations
for those frames, its queue fills, and throughput collapses. The driver's `TXed`
tracks `completed` exactly, which is how this stayed invisible: every earlier
sample was taken after a phase, when the queue had already drained.

84 MAC fatal events occurred in that run. They are almost certainly what leaves
the pipe in a state where no further status matches, since completion stops and
never resumes.

### The silent-hang bug, and two bad fixes

`enter_mac_fatal_quiescence` ends in `loop { nop }` and was not gated by
`corruption-non-fatal`, so a report-and-continue build suppressed the exception
publish and then span forever. The host saw no error at all. `PoppedMacEventEffects::fatal`
now returns `()` instead of `!`, so `execute_popped_single_outstanding_event`
keeps processing the event; halting builds still diverge.

Two attempted fixes made things worse and were reverted. Do not repeat them:

- **Returning instead of halting in `release()`** when a slot falls outside the
  FIFO. Skipping the release stalls the release head permanently, the FIFO never
  drains, and 3 of 4 runs then failed to associate at all.
- **Removing the halt in `publish_host_class0_slot`** and the HIF `raw_id` fatal.
  Continuing past an unusable pipe state stopped frame acceptance entirely:
  `pending` climbed to 20 with `used=0` and `TXed` frozen at 4.

"Halting is bad for measurement" is true, but these halts guard different things.
Each site needs its own reasoning about what continuing would corrupt.

### Next

Correlate the first suppressed fatal against the frame where `completed` stops.
If they coincide, the work is to recover pipe state after a fatal — re-arm or
resynchronise so status matching resumes — rather than merely surviving it.
Vendor never raises these events at all, so the deeper question of why we
provoke them remains open.

## VENDOR BASELINE: the gap is ours, and the fault is not inherent

Vendor firmware was run on the same board, same position, same AP, same driver
and the same harness. All four runs validate (server report present, traffic
confirmed):

```text
run   TCP          UDP delivered   UDP loss   exceptions   diag   fatal   ping loss
1     11.5 Mbit     8.83 Mbit        0%           0          0      0      0.52%
2     15.9 Mbit     9.05 Mbit        0%           0          0      0      0.04%
3      7.76 Mbit    8.65 Mbit        0%           0          0      0      0.31%
4      7.34 Mbit    7.12 Mbit        3%           0          0      0      0.65%
```

Against our firmware's validated numbers (TCP 1.05-1.26, UDP 1.28-1.41 delivered
with 60-75% loss, ping 2-5%, exceptions and corruption throughout):

- **~10x on TCP, ~7x on UDP, ~50x on ping loss.**
- **Vendor raises zero firmware exceptions and zero BH diag records.**

Two conclusions follow, and both overturn earlier reasoning in this document.

**The environment is not the limit.** Vendor reaches 15.9 Mbit/s TCP in exactly
our setup, so AP distance and rate selection do not explain our 1.3 Mbit/s.

**The corruption is not an inherent hardware fault.** Static analysis concluded
that TX command-fetch data reaching the RX write path was internal to the packet
controller and "probably not fixable from firmware". Vendor runs the same
silicon in the same slot without triggering it once. Whatever causes it is
something *our* firmware does or fails to do.

### Running vendor firmware (needed twice already)

The firmware and bootloader are a matched pair. Vendor's `boot_xr819.bin` is
2308 bytes, ours is 5224. Swapping only `fw_xr819.bin` fails with
`Wait for download completion failed: 0x00000006`, and swapping only the
bootloader fails too. `tools/ota-soak-vendorfw-run.sh` swaps both and restores
both, which matters because leaving vendor's bootloader behind breaks our
firmware as well.

After a failed load the chip stays wedged (`Bootloader is not ready`, or
`CMD req stuck in firmware`) until a reboot. Those messages are the wedge, not
evidence about the firmware under test; reboot before concluding anything.

### Where to look next

Vendor avoids the fault, so compare initialisation and programming order rather
than recovery. First candidates, in order:

1. Packet-DMA and MAC configuration registers written at cold init. We write
   `0x09c00600 = 0x01020418`, then zero `0x0604`/`0x0608`, plus `0x060c`,
   `0x061c`, `0x0620` (`platform.rs:591-602`). Diff every one against vendor's
   init path.
2. Pipe and ring programming: `ring+0x0c` command base, `+0x10` stride `0x54`,
   `+0x14` GO (`mac.rs:578-591` against `annotated-main.c:18557-18571`).
3. Ordering and any wait/handshake between publication and GO that we skip.

### Why we are 10x slower: narrowed, not solved

Station statistics during traffic, ours against vendor:

```text
                 ours              vendor
signal           -64 dBm           -64 dBm
tx bitrate       36-48 Mbit/s      54 Mbit/s
rx bitrate       1.0 Mbit/s        48 Mbit/s     <- ours was a firmware bug, see below
tx retries       3088              2730
tx failed        0                 6
tx bytes         2.46 MB           25.1 MB
```

**The air is not the problem.** Signal is identical, our TX rate is 36-48
Mbit/s, and `tx failed` is zero. Every airtime argument in earlier revisions of
this document assumed we were pinned at OFDM 9 Mbit/s; that was never measured
and is wrong.

**We simply do not feed the radio.** 2.46 MB against 25.1 MB is roughly 84
frames/s against 850. The medium is idle waiting for us.

Where the time goes, measured under iperf load rather than ping flood:

```text
admission -> publication   37.4 ms     (1.2 ms under ping load)
tx_start -> confirmation      317 us
main loop period               72 us   (13,900 iterations/s)
pending_gate                    0      (nothing ever refuses the frame)
```

**Eliminated:**

- Loop cadence. At 72 us per pass a frame among 30 contexts should wait about
  600 us, not 37 ms.
- Service budget. Raising `SERVICE_BUDGET` from 4 to 30 (`wide-service-budget`)
  was catastrophic: every run dead, `TXed` 51-861 against 4214-6779. The cost is
  per pass, not per context.
- The corruption scan. `validate_tx_boundary` compiles out under
  `corruption-non-fatal`, so it is not in the hot path of these builds.
- Publishing into an armed pipe. `non_aggregate_scheduler_decision` already
  refuses a pipe whose `idle_pipe_mask` bit is clear.

**Still open:** what serialises frames to ~84/s when the loop runs at 13.9 kHz,
pipe occupancy is ~317 us, and nothing refuses the frame. The next measurement
should be on the admission side: how fast the host hands us TX requests, whether
we ever run out of the 30 contexts, and how long a context sits between arriving
and being admitted.

### The firmware is fragile to instrumentation

An MMIO read of `0x0ac00004` plus a counter update per main-loop iteration broke
association outright, and even a bare counter increment left 2 of 3 runs unable
to associate. Both are now behind `loop-rate-counter`, off by default. Budget for
this when planning measurements: adding work to the main loop changes the
behaviour under study.

### RX rate reporting was a firmware bug

The WSM receive indication carries `rx_rate` at byte +10, and
`drivers/net/wireless/st/cw1200/txrx.c:1241-1247` maps it straight to
`rate_idx`. We wrote a hard-coded 0, which is rate index 0, so **every frame we
ever received was reported to mac80211 as 1 Mbit/s**. The AP was not slowing
down; we were mislabelling. Now sourced from `trailer+6`, mirroring vendor's
adjacent rate/RCPI descriptor pair (`annotated-main.c:13085-13125` copies
`param_1+0xe` and `+0xf` into indication bytes +10 and +11).

Every RX rate figure recorded before this fix is meaningless.

## READ THIS BEFORE TRUSTING ANY THROUGHPUT NUMBER

**A UDP iperf client with no reachable server still reports the offered rate.**
UDP has no handshake and nothing pushes back, so `-b 8M` prints "8.39 Mbit/s"
(8 Mbit plus headers) whether or not a single datagram arrives. Several
conclusions in earlier revisions of this document were built on exactly that
artifact, including a claimed capacity of 8.39 Mbit/s at 93% PHY efficiency.

A run is only valid if **both** hold:

- iperf printed a **server report** — the line carrying jitter and `lost/total`.
  Its absence means nothing arrived.
- the firmware's **`TXed` counter moved** across the iperf phase. In the fake
  runs it did not move at all, while iperf claimed tens of thousands of
  datagrams.

`tools/validate-iperf-runs.py` applies both checks. `tools/ota-soak-udp-run.sh`
now records `UDP_VALIDATION txed_before/after` inline.

### The validated numbers

```text
run   TCP        UDP delivered   UDP loss
ce0   1.18 Mbit   1.33 Mbit       63.9%
ce1   1.07 Mbit   1.33 Mbit       59.5%
ce2   1.05 Mbit   1.28 Mbit       75.0%
rs4   1.19 Mbit   1.31 Mbit       63.5%
rs5   1.18 Mbit   1.30 Mbit       62.2%
rx0   1.22 Mbit   1.33 Mbit       70.7%
rx2   1.26 Mbit   1.41 Mbit       66.9%
```

**The link carries ~1.3 Mbit/s, and every valid run agrees.** TCP sits at
1.05-1.26 Mbit/s, essentially at the UDP figure, so TCP is not collapsing below
capacity: it is at capacity. At OFDM 9 Mbit/s that is roughly 14% efficiency, so
there is a large amount of headroom, and offering more than about 1.3 Mbit/s
simply produces 60-75% loss.

### Claims retracted

- "Capacity is 8.39 Mbit/s, about 93% efficiency." Artifact.
- "TCP collapses under loss and the link is much faster than TCP shows." Wrong;
  TCP and UDP agree.
- "Throughput tracks the resync count, 1 resync giving 8.39 and 40+ giving 1.30."
  Spurious: the low-resync runs were dead links, which have no traffic and
  therefore neither corruption nor throughput.
- "We are at the PHY ceiling so pipelining cannot help." The ceiling argument
  was based on the artifact. At 14% efficiency the question is open again.

## Milestone reached: the link carries real traffic

The previous milestone was scan-owned probe TX. The firmware now associates and
passes ordinary data traffic over the air, board to a wired server:

```text
ping  2643 sent, 2584 received, 2.23% loss   (second pass 20/20, 0% loss)
iperf TCP uplink, 25 s sustained, ~1.2 Mbit/s
UDP   ~1.3 Mbit/s delivered, 60-75% loss when offered more
TXed 5015   RXed 6506   Pending TX 0   Used bufs 0
funnel: admitted 2646 -> published 2646 -> completed 2629 -> confirmed 2646
```

**TCP and UDP agree at ~1.2-1.3 Mbit/s**, so this is the link's throughput, not
a TCP artefact. See the validation section at the top before quoting any higher
figure.

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

**This list was reordered after measurement. Pipelining, previously first, is
now last and should not be attempted: the entire firmware TX path is 1.35 ms of
a ~10.8 ms per-frame budget and airtime is under 200 us, so there is almost
nothing for extra frames in flight to hide.**

1. **Find where the missing 85% of the medium goes.** Valid runs deliver
   ~1.3 Mbit/s at OFDM 9 Mbit/s, about 14% efficiency, and offering more just
   produces 60-75% loss. The firmware TX path accounts for 1.35 ms of a ~9 ms
   per-frame budget at that rate, so most of the gap is still unexplained and
   unmeasured. Start by instrumenting where a frame waits, not by adding
   capacity: the earlier "we are at the PHY ceiling" conclusion was based on a
   measurement artifact and is withdrawn.
2. **Port vendor's resync repair branch.** When the current slot is invalid but
   the computed next offset equals the producer and the length fits, vendor
   adopts that offset, clears the slot state and writes the magic at the new
   consumer, repairing the FIFO head instead of discarding. We only scan or
   flush. See `annotated-main.c:10364-10371`.
3. **Rate selection and aggregation.** Rate index 7 is OFDM 9 Mbit/s and
   `AGG TXed` is 0. The "~0.5 ms of a ~10 ms budget" estimate this entry used to
   carry was superseded by the latency decomposition: actual airtime is 96-194
   us. Rate work raises the ceiling but cannot help while loss dominates, so it
   belongs after the resync work above.

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
4. **Failure rate under TCP.** `retired` is 0.8% under ping flood but 7.4%
   under sustained TCP. Understand what bidirectional load changes.
5. **The corruption itself — now diagnosed.** It is the *live* TX command
   fetch: the leaked bytes come from the command storage of the slot the MAC is
   fetching at that instant (pipe armed, ring cursor on that slot). AES is
   eliminated, there is no command-list destination register, no software copy
   path, and the two address windows are not aliases. The mechanism is internal
   to the packet controller and probably not fixable from firmware, so treat
   recovery quality as the deliverable. See the dedicated section below.
6. **Multi-slot publication (pipelining). Do not pursue.** Kept only so the
   next reader does not rediscover it.

   **Status attribution is ordinal and already batch-safe.** Vendor's
   `txp_pipe_tx_status` (`0x9a32`) selects the slot from the pipe's `current`
   pointer (`+0xa2`), not from anything in the status, and advances `current` by
   one per accepted status. The MAC walks slots in order, so N armed slots
   produce N statuses matched against successive `current` values. Depth 1 is
   the degenerate case of the same scheme, so pipelining does not need a new
   completion-matching design.
Note there are **four** halting corruption detectors, three in `radio.rs` and
one in `hif.rs`; `corruption-non-fatal` neutralises all of them. Do not count
corruption inside `hif::validate_tx_boundary`: it runs several times per
main-loop pass, so a counter there measures call frequency (it read 2,162,040)
rather than corruption events.

## The corruption is the throughput bottleneck

(An earlier revision titled this "...and it starts with a MAC fatal". That was
wrong: three capture campaigns found no anomaly in any state at the fatal, and
the corruption is the live TX command fetch, described below.)

Two runs of the same image, one good and one bad:

```text
              UDP        rx_resync   latency_last   RX frames dropped
"good" (rx1)  INVALID            1        6.3 ms                 358
valid  (rx0)  1.33 Mbit/s       21      152.0 ms                 655
```

**This comparison does not hold.** rx1 has no server report and its `TXed` did
not move, so it carried no traffic at all; a run with no traffic trivially has
no corruption and no resyncs. The apparent correlation between resync count and
throughput was an artifact of comparing a dead link against a working one.

**Capacity is 8.39 Mbit/s**, about 93% efficiency at rate index 7 (OFDM
9 Mbit/s). The hardware is fine. What varies between a good run and a bad one is
how often the corruption fires, so the run-to-run spread that has made every
measurement in this document noisy was never harness variance: it is the
corruption count.

The chain is: corruption -> RX consumer resynchronisation -> the resync discards
FIFO content -> received frames are lost -> echo replies and TCP ACKs vanish ->
retransmission -> TCP collapses to a stall. So the corruption is not a quality
issue to fix later, it is the throughput bottleneck.

### What the halting build caught

Building without `corruption-non-fatal` halts at the first fault.

**Sample size is 1.** The record below was read three times, but all three reads
are the same latched record inside run 1; run 2 produced no record at all while
still dying (99.79% loss, BH fatal at 39.8 s). An earlier draft of this document
called the fault deterministic on the strength of those three identical reads,
which was wrong. Run 2 also shows there is a second failure mode that kills the
link without raising a MAC fatal.

The fault fired at 460 frames during the ping flood, not under iperf load.

```text
[0]  event.raw   0x41403993   type 0x39, status 0x13, fatal bit 30, completion
[1]  trace_count 0x154a0d72
[2]  prior       0x01403903   type 0x39, status 0x03, not fatal
[3]  prior       0x0140b711   type 0x37, status 0x11  (ordinary TX success)
[4]  prior       0x0243b71a   type 0x37, status 0x1a, phase 3
[5]  prior       0x0242b71a   type 0x37, status 0x1a, phase 2
[6]  command     0x09007080   TX command base, pipe 0 slot 0
[7]  cmd+0x0c    0x51040809
[8]  cmd+0x10    0x50000002
[9]  cmd+0x14    0x520c0252
[10] cmd+0x18    0x31004188
[14] pipe state  0x04001720
[15] ring base   0x09c60000
[16]             0x090f0f0f
```

The fatal interpretation is right, and vendor agrees: `mac_event_dispatch`
(`0x9c4e`) tests the same bit and panics on it.

```c
trace_push_word(uVar8);
if ((int)(uVar8 << 1) < 0) {          /* bit 30 */
    fw_assert(DAT_0000a0ec, 0xde, 0x29);
}
```

Vendor otherwise treats type 0x39 as an ordinary TX status, special-casing only
status 6, and `mac_irq_tx_status_dispatch` counts statuses 6..0x18 (which
includes 0x13) into a stat at `+0x2c`.

**So the MAC is genuinely raising a fatal condition, deterministically, and the
open question is why.** It fires in normal builds too; `corruption-non-fatal`
only suppresses the report, while `enter_mac_fatal_quiescence` still runs, which
is the likely source of the degradation and the resyncs that follow it.

Start here: what makes the MAC raise type 0x39 status 0x13 after an ordinary
0x37/0x11 success. The record's command storage and pipe state are captured
above and are reproducible in two runs.

## The corruption: TX command fetch lands in the RX FIFO

Probing the RX FIFO corruption detector with the TX pipe state at the moment of
detection answers what it is:

```text
pointer      0x09007080   leaked bytes come from pipe 0 slot 0 command storage
pipe_bytes   0x01000000   producer=0 last=0 current=0 armed=1
ring+0x20    0x410f0e0f   hardware cursor = slot 0
ring+0x0c    0x00007080   ring command base, consistent with slot 0
aes src/dst/len  0 0 0    AES engine idle
match        7 words from command offset +0x0c, into the live RX producer delta
```

**The leaked slot is the slot the MAC is actively fetching.** The pipe is armed,
the ring cursor sits on slot 0, and slot 0's command body is what turns up in the
RX FIFO. Earlier captures leaked slot 3 (`0x0900717c`) with the cursor there.
The leak follows the live fetch rather than any fixed address, which is why the
`0x0900717c` coincidence with an old stale-slot capture was misleading.

AES is eliminated as the writer: source, destination and length all read zero.

Static analysis independently established that the MAC fetches command lists
autonomously from `ring+0x0c`, that there is no command-list *destination*
register, that no software copy path exists, and that `0x09403f00` is not an
alias of `0x0900717c` (the DMA mask `0xf6ffffff` clears bits 27 and 24 but
preserves bit 22). It also explained `0x09403fc4`, previously treated as
suspicious, as a legitimate zero-copy HIF *source* pointer.

So the remaining mechanism is internal to the packet controller: data fetched
for TX command execution reaches the RX packet-DMA write path. That may not be
fixable from firmware, which makes the recovery path the thing worth improving.

### RX consumer desynchronisation

The other record type shows the consumer positioned past the producer:

```text
producer 0x35e8   CLAIM 0x3dd4   RELEASE 0x3b04   dma_consumer 0x3b04
slot at CLAIM: word0 0x4000f6f8 (not the magic), +0x18 length 298
words found:   0x00236002  0x07004600  0xf0000000   <- command list + terminator
```

Our normal consume path already validates exactly as vendor does: length >= 4,
length <= MAX_FRAME_LEN+4, and available >= length (`radio.rs:1143-1148`). The
problem is that `available_bytes(0x3dd4, 0x35e8)` wraps to 0x6814, about 26 KB,
so once the consumer passes the producer every check becomes vacuous and stale
slots beyond the producer still carry a valid magic and a plausible length.

### Vendor's resync, and what we were missing

`rxfifo_next_frame` (`annotated-main.c:10331`) uses the same magic `0x00aa55ff`
and the same total flush when the span is exhausted, so the destructive fallback
is vendor behaviour and should stay. Two things we did not have:

- After finding a candidate carrying the magic, vendor also requires the slot
  length to fit the remaining span AND the *following* slot to carry the magic,
  and keeps scanning otherwise. Without that lookahead any payload word equal to
  `0x00aa55ff` resynchronises us onto garbage.
- A repair branch: when the current slot is invalid but the computed next offset
  equals the producer and the length fits, vendor adopts that offset, clears the
  slot state and writes the magic at the new consumer, repairing the FIFO head
  instead of discarding anything. Not yet ported.

### Measured: the lookahead is correct but not sufficient

Six runs with the vendor lookahead ported:

```text
run  accepted/flushed/rejected   UDP          ping loss
rs0        1 / 0 / 0             8.39 Mbit/s   74.6%
rs1        -                     8.39 Mbit/s    2.16%
rs3        1 / 0 / 0             8.39 Mbit/s   97.5%
rs4       40 / 1 / 5             1.31 Mbit/s    2.41%
rs5       45 / 2 / 15            1.30 Mbit/s    2.28%
```

**False magic matches are real**: 5 and 15 of them in single runs, each one a
payload word equal to `0x00aa55ff` that the old code would have resynchronised
onto. That bug is fixed and the fix is vendor-exact. It is a correctness fix.

**It does not change throughput.** rs4 and rs5, the only valid runs in that
batch, deliver 1.31 and 1.30 Mbit/s, matching every other valid run. The rs0 and
rs3 rows that appeared to show 8.39 Mbit/s with a single resync were dead links
carrying no traffic.

### The stale-slot hypothesis, and why it is wrong

Vendor asserts that the software slot pointer equals the hardware ring cursor
(`annotated-main.c:657`, in `tx_ptcs.c`), and an old capture showed producer 2
against cursor 3 with the cursor slot holding a null frame. That, plus seven
words of command storage `0x0900717c` turning up in the RX FIFO at `0x09403f54`
and `0x0900717c` also being the never-published slot in that capture, suggested
one mechanism behind everything: the cursor runs ahead into an unpublished slot,
executes stale commands, and DMAs them over RX memory.

**Measured, and it is wrong.** Recording producer and cursor side by side at the
fault gives:

```text
type 0x39 status 0x0f, pipe 3
producer=1 last=0 current=0 armed=0 | hardware cursor=1   -> equal
cursor cmd 0x090074c4  word[0]=0x00000000  word[+0x14]=0x520b0022
```

The cursor equals the producer, so vendor's invariant holds. The `word[0]==0`
that looked like a smoking gun in an earlier capture is just the producer slot:
the next slot to be written is legitimately empty, with stale payload behind it
from the previous frame that used it. `0x0900717c` matching the RX FIFO
corruption was coincidence; it is one of only four slot addresses.

An earlier revision of this section claimed the fatal arrives on an idle pipe.
That was one capture, and eight further runs contradict it:

```text
type 0x13 status 0x1a  pipe 0 | prod=0 last=0 cur=0 armed=1 hwcur=1  invariant OK
type 0x39 status 0x0f  pipe 0 | prod=0 last=0 cur=0 armed=1 hwcur=1  invariant OK
```

The pipe is armed with slot 0 in flight and the hardware cursor has correctly
advanced to slot 1. Note the invariant vendor actually asserts is
`(last+1) & 3 == cursor`, not `producer == cursor`; both readings hold here.

**So pipe and cursor state at the fault look entirely normal.** Stale-slot
execution, idle-pipe delivery, and cursor divergence are all eliminated. Three
capture campaigns against the fatal event have produced no anomaly in the state
we can see, while bit 30 has now appeared with five distinct type/status pairs:
0x39 with 0x04, 0x0f and 0x13, 0x37 with 0x1a, and 0x13 with 0x1a.

The RX-side corruption is the better target: its signature is byte-identical
across runs, while the fatal event is neither reproducible in its code nor
correlated with any state we have managed to record.

### Reading the record

The driver's diag buffer truncates at 80 bytes (8 header + 18 words), so
anything past word 17 never reaches dmesg regardless of what the firmware
writes. `CW1200_BH_RX_DIAG_DATA_SIZE` reads 32 in this tree, which matches
neither, so the board runs a module built from a different revision. Put new
fields in low word indices and verify the transport before spending board time.

`current_pipe_record` is already `PIPE_RECORDS + pipe*0x6c + 0xa0`, so the slot
bytes are at `+0..+3`. Adding `0xa0` again reads another pipe's record and
yields values like 166 and 223, which cannot be slot indices at all.

### Fatal event variety

Across runs, bit 30 has been seen with type 0x39 statuses 0x04, 0x0f, and 0x13,
and with type 0x37 status 0x1a. Four distinct pairs from one image fits bit 30
being a severity flag over a shared upstream fault rather than four separate
conditions.

## What actually limits throughput: loss, not the TX pipeline

Measured with the same image in one run, UDP directly after TCP:

```text
UDP, 8 Mbit/s offered     3.83 - 3.91 Mbit/s delivered
TCP                       964 -> 755 -> 0.000 -> 0.000 Kbit/s
ping                      2-5% loss, with +20 and +41 duplicates
```

The link carries ~3.9 Mbit/s. TCP collapses to a stall, which is RTO backoff
from losing the same segment repeatedly, so every "1.1-1.2 Mbit/s" figure in
this document is TCP's reaction to loss and not a capacity measurement. At rate
index 7 (OFDM 9 Mbit/s), 3.9 Mbit/s of goodput is about 43% efficiency, which is
ordinary for 802.11 overhead. **The transmit path was never the bottleneck.**

The duplicates are the strongest lead: the AP is retransmitting frames it had
already delivered, so our 802.11 ACKs are not reaching it. Chase RX-side ACK
behaviour before anything else.

### The latency budget, fully decomposed

```text
admission -> PAS release      1203 us   waiting to be serviced, NOT gated
PAS release -> publication      45 us
publication -> tx_start         45 us
tx_start -> completion          96 us   <- actual airtime
completion -> confirmation      11 us
firmware total                1355 us   of a ~10.8 ms per-frame budget
```

`pending_gate` measured 0 across 5327 frames, so `service_pending` never
refuses; frames simply wait for a service pass. Even eliminating that entirely
recovers ~1.2 ms of ~10.8 ms.

### Consequences for the plan

- **TX pipelining is not worth pursuing.** Airtime is under 200 us, so there is
  almost nothing for extra frames in flight to hide. Depth 2 is implemented,
  measured, and off by default; see the section below.
- **Power save is not involved.** `iw set power_save off` changed nothing:
  1.18 Mbit/s and 9.997 ms RTT against 1.1-1.2 Mbit/s and ~10 ms with it on.
- **Rate selection matters more than it appeared**, but only after loss is
  fixed: TCP cannot use extra capacity it keeps losing.
- The RX path has never been instrumented. Every measurement in this document is
  TX-side, yet ping RTT covers both directions and the duplicates implicate RX.

## Depth 2 pipelining: implemented, measured, does not work yet

`tx-pipelining` (stage 2 slots, arm once) is in the tree and off by default.
Four runs at depth 2 against three at depth 1:

| | depth 1 | depth 2 |
|---|---|---|
| runs with traffic | 2 of 3 | 1 of 4 |
| ping loss (good run) | 2.1-2.3% | 2.5% |
| final ping recovered | 20/20, 19/20 | 0/20 in every run |
| `latency_last` | 1.9-4.0 ms | 9.5 ms |
| `latency_max` | 131-197 ms | 33.5 ms |

It engages: the latency distribution changes shape, with the tail cut sharply
and the typical case tripled. But throughput does not survive, so something in
the staged path is wrong. Do not enable it without finding that first.

Untested guesses, in the order worth checking:

1. The per-slot ring duration write (`hardware_ring + 0`) may not be a FIFO
   push. Vendor's loop writes it once per slot; if the register is a plain
   latch, only the last slot's duration survives and the earlier slot runs with
   the wrong airtime.
2. `staged_slots_total()` resets to 0 at arm, so `staged_headroom` is true again
   while frames are still in flight. The reservation is rejected by the idle
   check, so no state is corrupted, but every service pass now attempts a
   reservation it cannot complete.
3. Slot records staged before the arm sit unarmed for the rest of the service
   pass. Nothing else in the driver expects a published-but-unarmed slot.

Depth 1 (`BatchPosition::Only`) reproduces the previous sequence and is what all
the healthy measurements above use.

## Vendor ignores unmatched TX statuses; do not retire on them

`txp_pipe_tx_status` gates on three things and does nothing whatsoever if any
fails:

```c
iVar6 = *(byte *)(iVar5 + 0xa2) * 0x18 + iVar5 + 0xa0;   // slot from `current`
if ((*(char *)(iVar5 + 0xa3) == '\x01') &&        // pipe armed
    (*(byte *)(iVar6 + 0xd) == param_1) &&        // delivered == expected
    (*(char *)(iVar6 + 0xf) == '\x03'))           // slot state 3
```

There is no else branch. A status that does not match is **normal** and is
discarded; a pipe that genuinely stops is recovered by the 200 ms watchdog
(`FUN_00003bac`), which is the only recovery mechanism vendor has.

Our cursor-based retirement had no vendor counterpart. Because the slot comes
from `current`, a status for a finished frame is evaluated against the frame
published behind it, and retirement destroyed that frame before it transmitted:
all 297 retirements in an iperf run were slot state 1, ~10% of published frames
under load against ~1% under ping flood. Retiring nothing there is what vendor
does.

The watchdog reporting zero recoveries in healthy runs is **correct**, not a
sign it is broken: `armed` is cleared on every normal completion, so the gate is
false whenever traffic flows. The single run that wedged lost its MIB lane
before the counters could be read, so the watchdog has never actually been
observed under the condition it exists for.

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

## Hardware CCMP and the intermittent fast-path collapse

Hardware AES-CCM is functionally validated. The vendor mode-1 AES program was
recovered from DTCM `0x04000830..0x040009de` and loaded through the vendor
`0x80000006`, byte-stream, `0x9000` sequence. CPU IRQ/FIQ remain masked in this
firmware, so completion is reproduced in the foreground: wait for controller
pending IRQ 18/20, acknowledge controller `+0x04`, then invoke the matching
callback. Polling AES bit 29 was wrong; callback-only waiting also timed out
because the CPU interrupt dispatcher cannot run.

The RustCrypto-generated boot KAT now covers exact TX ciphertext/MIC, repeated
payload lengths 1/15/16/17/1506, exact RX plaintext, and corrupted-MIC rejection.
IRQ 20 completes the tested TX and RX operations. The seven-operation KAT takes
about 947 us.

Correct hardware completion demonstrated the expected acceleration. One TX-only
run reached TCP 3.01 Mbit/s and UDP sender 7.86 Mbit/s at 54 Mbit/s with zero TX
failures before RX later stopped. Full TX+RX hardware runs have delivered:

```text
TCP          UDP delivered   final ping
3.46 Mbit/s  5.50 Mbit/s     19/20
2.90 Mbit/s  3.89 Mbit/s     20/20
3.12 Mbit/s  4.49 Mbit/s     20/20
```

The transform itself is not the intermittent failure: live shadow comparison
verified 6513 and 6660 hardware-encrypted frames against RustCrypto with zero
mismatches. An 8 ms post-crypto delay is healthy, while 1/2/4 ms are not. This
establishes a load/timing threshold but is not an acceptable fix; vendor queues
hardware crypto asynchronously and does not deliberately pace it.

### Collapse signature

Five byte-identical full-hardware runs produced two valid and three dead runs.
A later three-run set was entirely dead, while another vendor-order set produced
one valid and two dead runs. Dead runs usually carry traffic initially, then RX
and TX counters freeze, TX failures rise to about 2300-2600, the rate falls to
6-24 Mbit/s, BH remains alive, queues drain, and final ping is 0/20. A corrected
TX-only run also showed a second phenotype: TX stayed strong at 54 Mbit/s with
zero failures while RX stopped. Do not assume every terminal signature has one
cause.

### Failed or neutral investigations

- **AES bit-29 completion:** real bug; replaced with actual pending IRQ 20.
  Peak throughput improved, but the later collapse remains.
- **Packet-DMA IRQ 26 foreground servicing:** neutral.
- **Hardware RX crypto:** valid decrypt and bad-MIC KATs pass; enabling it live
  did not remove collapse.
- **Vendor RX tail repair:** ported and neutral.
- **Host-request suppression of RX:** removing the absolute gate was neutral.
- **Vendor-style RX drain-until-empty:** bounded 32-slot draining regressed
  initial ping loss to 10.9% and collapsed earlier; reverted.
- **PN publication ordering:** thousands of strict PN decreases occur in both
  valid and failed runs. The first diagnostic incorrectly counted equality;
  after correction, decrease count still did not correlate with validity.
- **RX FIFO corruption/resynchronisation at terminal failure:** three failed
  runs recorded zero accepted/flushed/rejected resyncs and ended with DMA
  producer exactly equal to claim. They did not terminate with unread or
  desynchronised FIFO data.
- **Vendor pending-list/PAS-ring scheduling order:** replacing context-array
  rotation with vendor queue order produced one valid and two dead runs, matching
  existing variability. Reverted; scheduling order is not established as the
  cause.

These negative results are important: do not repeat RX fairness, drain depth,
PN-order-count, IRQ 26, tail-repair, or vendor-order experiments as if they were
new leads.

### Version-control boundary

Validated ownership repair, AES microcode/KATs, pending-IRQ completion, hardware
TX/RX, IRQ 26 foreground service, and RX tail repair are committed as Jujutsu
change `yzpzlxuk`, commit `12025526`, titled
`Add vendor-shaped TX ownership and hardware CCMP`. The failed scheduling and
cold-read experiments were reverted from the following working-copy change.

### Masked IRQ 6/21 audit

Raw vendor disassembly corrects and narrows the remaining masked-interrupt lead:

- Startup `0x00000c38` registers IRQ 6 callback `0x0000f25e`, not `0x0000f1fe`
  as an old Rust comment claimed, and scheduler task `0x0000f204` on event bit
  27. The callback only raises the event. The task disables/reprograms the timer
  compare, walks the sorted timer list at `0x04002014`, unlinks expired entries,
  and invokes their callbacks. Servicing the IRQ without translating that timer
  queue would therefore be incomplete and unsafe.
- Startup `0x00000b88` registers IRQ 21 callback `mic_complete` at `0x0000ef1c`
  and deferred task `0x0000ee90` on event bit 29. This is the vendor MIC-engine
  request/completion queue. The open firmware does not submit that queue, but
  this does **not** prove the hardware source is inactive: the open startup also
  does not explicitly reset the `0x09c40000` MIC register block, and it enables
  IRQ 21 with no way to acknowledge a stale or spontaneous completion while CPU
  IRQ/FIQ are masked. Treat IRQ 21 as an enabled-but-unowned source until the
  pending-bit experiment resolves it.

Do not blindly foreground-dispatch IRQ 6 or 21. A zero-extra-MMIO diagnostic
used the interrupt-controller pending word already read every main-loop pass for
IRQ 26 service and latched IRQ 6/21 presence and rising edges only when state
changed. Three runs of image
`8f095332051b388e94bdccd515a4375ecc8e775ca547e3ec85ca71bd33026847`
produced one collapsed and two valid links. All three recorded exactly:

```text
observed pending mask  0x00000000
IRQ 6 rising edges     0
IRQ 21 rising edges    0
```

Thus neither enabled-but-unowned source asserted during these runs, including
the collapse. IRQ 6/21 pending service is ruled out for the observed failure and
the temporary diagnostic was reverted. IRQ 21 was worth checking despite being
the MIC engine; its subsystem identity alone was not evidence that the hardware
source stayed inactive.

### Content-corruption blind spot and packet-DMA IRQ 27

A read-only adversarial review exposed an overstatement in the zero-resync
conclusion. Canonical `corruption-non-fatal` builds return immediately from
`radio::validate_tx_boundary`, so they do **not** scan the RX producer delta for
the historical TX-command signature. The cumulative resync fields prove only
that framing, lengths, magic and ownership stayed coherent. They do not exclude
TX command bytes inside an otherwise valid payload, data overwritten before the
consumer sees it, or the packet-DMA producer engine stopping entirely.

Vendor `fw_subsystem_init` registers IRQ 27 as `event_send_error_0x34`, alongside
packet-DMA IRQ 26. A no-extra-MMIO experiment enabled IRQ 27, acknowledged it
from the pending word already read for IRQ 26, cold-latched its count, and split
RX decrypt/authentication errors from generic filtering. Image
`31d933859df43933a59753571de1f3459c40ae30e2d9ddd6cceddf7beafc9279`
produced one valid and two collapsed runs:

```text
run        valid   IRQ27   RX decrypt errors   final TX failures
1          yes          0       1436                 15
2          no           0       1049               2525
3          no           0        845               2318
```

IRQ 27 did not assert, including in either collapse, so the vendor packet-DMA
error line does not report this failure. Decrypt-error count also does not
correlate: the valid run had the most errors in both absolute terms and relative
to RXed. The temporary IRQ27/error diagnostic was reverted. This does not clear
a silent producer-engine wedge; it only proves that vendor's explicit error
source stays quiet.

### Vendor single-initialisation experiment

Static comparison found that normal open-firmware startup programmed RX/MAC
register tables before `initialize_vendor_startup_state`, then rebuilt pipe
hardware later, including live writes to ring GO and a `0x09c00e8c` cycle.
Vendor performs those initialisations once. The existing `vendor-single-init`
feature removes the earlier RX-table copy and the later hardware pipe rewrite
while retaining software pipe-record reconstruction.

Three runs of image
`97f71ce32fcb641dc433738a20b4ace34c078efcd98a60cb7da5db4ca59b8e6f`
produced one scan-only association failure, one healthy run (TCP 2.74 Mbit/s,
UDP delivered 5.05 Mbit/s, zero TX failures), and one ordinary collapse (TCP
965 Kbit/s, no UDP server report, 2470 TX failures, final ping 0/20). This
matches baseline variability. Duplicate live pipe/RX initialisation is not
established as the collapse cause; leave the feature off.

### Fresh-boot downlink-only isolation

Three runs removed the initial ping flood and all uplink iperf load. Each fresh
boot received a 30-second TCP stream from the wired `end0` address to the Wi-Fi
netns, followed by downlink UDP. All three collapsed during TCP downlink:

```text
run   first intervals                    cumulative TCP   TX failures   final ping
1     2.83M, then zero                    226 Kbit/s           9          0/20
2     3.77M, 54.8K, then zero             315 Kbit/s           9          0/20
3     4.40M, 3.36M, 96.8K, then zero      644 Kbit/s          10          0/20
```

The UDP client continued to print the offered 8.39 Mbit/s but received no final
server acknowledgement; those UDP figures are invalid. Image:
`6f01365e7ff0221947ddeccfdd39116c6b5a4aa0a3701d584785b4371f5b3604`.

This is a major discriminator: high-rate uplink publication is not required.
Downlink still produces TCP ACK frames (713/986/2002 TXed at terminal state),
but at far lower TX volume than the ordinary uplink harness. The terminal state
has only 9-10 TX failures rather than ~2500, so this is the RX-dominant phenotype.
Prioritise RX slot retention/HIF release, RX producer/MAC receive state, and
shared RX/TX turnaround over host-TX scheduler ordering.

### RX ownership/HIF pressure is not the collapse

A zero-extra-MMIO image cold-latched current/high-water ownership at existing
RX/HIF transitions. Standard uplink-harness collapses reached only RX host
high-water 4 and HIF software-output high-water 4-7, then drained to zero.
Downlink-only qualification was even clearer:

```text
run       result      RX host high-water   HIF output high-water
1         collapsed          4                     4
2         healthy           16                    16
3         collapsed          4                     5
limits                       24                    64
```

The healthy run carried TCP 3.79 Mbit/s and downlink UDP 7.80 Mbit/s with zero
TX failures and final ping 20/20. The collapsed runs stopped during TCP and
ended at 0/20. Higher ownership pressure correlates with successful traffic,
not failure. Neither the 24-slot RX host limit nor 64-entry HIF output queue is
the cause. The temporary pressure instrumentation should be removed after the
RX-restart experiment that reuses its cold snapshot words.

### Vendor RX restart recovery experiment

Vendor `mac_rx_restart` (`0x000002f4`) saves and clears RX-related bits 2..5 in
`0x09c00e8c`, disables packet DMA by clearing `0x09c00600` bit 0, waits for busy
bit 23 to drain, re-enables RX, and restores the saved mask. The open experiment
ports that sequence behind `rx-progress-restart`, waits for class-0 and
management MAC ownership to become idle, and cold-latches attempts/successes.
It uses the already-maintained RX indication counter and existing 200 ms pipe
watchdog timer read.

The first trigger armed after >=16 indications per tick and required five fully
empty ticks. It never fired in any run (`attempts=0`), including one collapsed
run, so its two healthy downlink results cannot be credited to recovery. More
importantly, this is not vendor policy: vendor reaches `mac_rx_restart` only
inside `tx_flush_all_queues` after repeated pipe-idle timeout, abort-all, and a
forced cleanup timeout. The proposed low-progress trigger was rejected and its
follow-up run was cancelled. Do not present invented periodic RX restart as a
vendor-shaped fix.

### New lead: missing RX FIFO low-space resume handshake

Independent foreground and subagent reads of raw vendor code converged on the
same omitted RX lifecycle. At the tail of `rxfifo_next_frame` (`0x00008bda`),
vendor computes free FIFO space and, below `0x1000`, snapshots the live mode word
from `0x09c00200` into RX FIFO state `0x04001698`. At the tail of
`rxfifo_release_slot` (`0x00008cfc`), once free space rises above `0x0fff`, it
writes that saved word back to `0x09c00200` and clears the latch. The write-back
is the only visible resume/kick operation.

Vendor is interrupt-driven: an HIF completion/release can preempt after
`rxfifo_next_frame` returns and before `rx_handler_main_loop` clears the temporary
latch. The open firmware keeps CPU IRQ/FIQ masked and serializes HIF reclaim and
RX polling, eliminating that release window entirely. This is not ownership
saturation: healthy downlink reached host/HIF high-water 16, while collapsed
runs reached only 4-5. It is a missing low-space transition that can leave the
producer silently stopped after the consumer later drains, exactly matching
producer==claim, zero resync and IRQ27 silence.

Feature `rx-low-space-resume` adapts the vendor handshake to cooperative service:
when claim leaves less than 4 KiB free, snapshot `0x09c00200`; when release
restores more than 4 KiB, write the same word back. Reads/writes occur only on
that vendor threshold transition, not as recurring instrumentation. Cold fields
count entries, resumes and a pending latch. Image
`3226392a8e6584d391ee2b0f664df8fa25a29e20c18ca71d636d87ef4aabf06a`
produced one healthy and two collapsed downlink runs. Both collapsed runs
entered low-space once, while the healthy run did not, but all three reported
zero resume writes. That exposed a porting error rather than falsifying the
lead: the first implementation normalized ring free space. Vendor literally
computes unsigned `(release - claim) + 0x7000`; after release wraps ahead of
claim, a value above `0x7000` is intentional and makes the resume test succeed.
The normalized helper remained below threshold and suppressed the write-back.
Image `fb51f8fd740e57a920c5800710935623a59e44fb4334d92fb99d177d802bc42a`
used the exact vendor wrapping expression. All three fresh-boot downlink runs
collapsed: cumulative TCP was 1.37 Mbit/s, 1.78 Mbit/s and 546 Kbit/s; each ended
with 0/20 ping and only 7, 7 and 8 TX failures. Crucially, all three recorded
zero low-space entries and zero resume writes. The earlier one-entry correlation
was entirely an artifact of normalizing the ring expression. Vendor's actual
less-than-4-KiB condition did not occur in these collapses, so the low-space
resume handshake is ruled out as their cause. The experimental feature and
snapshot decoder fields were reverted.

### New lead: missing packet-DMA self-write after every RX drain

Vendor `rx_handler_main_loop` (`0x00008e3e`) clears FIFO state `+0x18` after
`rxfifo_next_frame`; when no next frame exists it executes the literal MMIO
self-write `*0x09c00600 = *0x09c00600` before returning. Rust
`radio::poll_indication` returned immediately when claim equalled producer and
omitted this action. In vendor's event-driven task the write occurs once after a
drain cycle, not as a continuous empty-FIFO poll. It is therefore plausibly a
packet-DMA producer re-arm or shared TX-fetch/RX-write turnaround action and is
a better match for silent producer death than the disproved low-space path.

Feature `rx-empty-rearm` tracks nonempty-to-empty drain transitions and performs
that exact control read/self-write only once per transition. Cold fields count
drain cycles, writes, and an active unfinished drain. It adds no MMIO to the
nonempty hot path beyond vendor's required empty-tail action. Image
`dda46bb26b2cc7e9e9c58d36c2351000c097e6b1a0109eb9625d27eaa287b0b4`
produced one healthy and two collapsed downlink runs, matching baseline
variability. The healthy run delivered 4.26 Mbit/s TCP and 7.92 Mbit/s UDP with
20/20 final ping. The collapsed runs delivered cumulative 1.85 and 1.49 Mbit/s
TCP and ended 0/20 with only 6 and 7 TX failures. Every observed drain transition
executed its write: 36,890/36,890, 12,082/12,082 and 9,759/9,759 cycles/rearms,
with no pending drain. The write therefore executed heavily before both healthy
and collapsed outcomes and did not prevent terminal RX death. The feature and
snapshot decoder fields were reverted.

### PHY timer audit correction

The PAC timer at `0x04001d18` is real. Startup registers timer callback
`0x00002aa3`, which raises scheduler bit 18; task callback `0x00002ab0` either
finishes a state-2 PHY transition or calls `pac_phy_start_op(4)`. Command 4
checks/latches temperature/calibration state through `0x000167ec` and
`0x0001a140`; deferred work is later serviced by scheduler command 6. Rust
replaces this timer callback with `inactive_startup_task` and never claims
scheduler bit 18, so the lifecycle is incomplete.

However, `pac_phy_start_op(1)` starts the same 10,000,000-tick timer for each
nonempty TX queue batch, and vendor `timer_start` (`0x0000f2aa`) cancels and
restarts an already-active timer. TCP downlink ACK traffic therefore continually
postpones expiry just as vendor does. This missing maintenance may remove a
post-idle recovery path, but it is not currently the strongest initiator for a
5-15-second collapse during active downlink. Do not describe it as periodic
under sustained TX without accounting for the restart semantics.

### Completed evidence: nonfatal valid-payload TX-command content watch

Feature `rx-content-watch` restores visibility that `corruption-non-fatal`
previously compiled out, without restoring a halting producer-delta scan. At the
final class-0 publication boundary it snapshots five 96-bit anchors from the
fully built TX command into a bounded 128-entry direct-mapped ordinary-SRAM
table. The RX parser hashes each four-byte-aligned 96-bit window of a valid,
contiguous frame payload and performs an exact three-word comparison on a table
hit. It records only match count, first command pointer, and first command/RX
offsets. Thus it detects recent historical TX-command content inside an
otherwise valid RX slot, has a negligible 96-bit false-match probability, does
not read producer/control MMIO, and never emits a host event or changes recovery.

Host tests (137), ARM release check, Python compile and source diagnostics pass.
Image `42a833e2c61365116bee92dff80d311b8f63ed25f00de647c361d87c54854729`
produced two collapsed runs with zero matches and one healthy run with nine
matches, all at command offset 0 and RX offset 0. The healthy-only offset-zero
matches are low-information/common frame-leading content, not evidence of
corruption. The detector was tightened to reject triples with fewer than two
nonzero words or three identical words, and now snapshots every distinctive
96-bit window spanning command offsets `0x0c..0x50`, the same region used by the
historical halting detector. The direct table grew to 256 entries; RX lookup cost
remains one hash and one exact comparison only on a table hit. Image
`be5bce8b09bf862f9f98afa04bb79191c25e297dd5fd40e0c5f75a9e71450d01`
produced three collapsed runs. Match counts were 9, 0 and 5, so content matches
are not required for collapse but may identify one corruption phenotype. The
host driver's MIB printout omits `rx_cmac_key_id_errors`, so the displayed zero
command/RX offsets were missing telemetry rather than real offsets. The first
match is now packed entirely into printed `rx_cmac_replays`: command slot,
command offset and RX offset. Image
`8328e3b48db227e3187b79bacb934bf947386023efb435ef38d28767a56f0d8b`
produced three collapsed runs, all with nonfatal exact 96-bit matches to recent
TX commands. Counts were 5, 3 and 10. Every first match began at command offset
`0x0c`; the matching triples appeared at unrelated valid-RX payload offsets
`0x1b4`, `0x5e0` and `0x078`, from command slots `0x09007080`, `0x09007128` and
`0x0900717c`. Final pings were 0/20; cumulative TCP was 886, 499 and 1130 Kbit/s.
This is not the earlier command-offset-zero/common-header artifact. It reproduces
historical TX-command leakage without halting and proves that valid FIFO framing
can conceal packet-controller content corruption. It does not yet prove all
collapses require leakage: the preceding refined image had one collapsed run
with zero retained matches, possibly due direct-table replacement or a separate
collapse phenotype.

The cold record now also includes the exact matched words, hash, RX indication
index and TX command generation, packed into host-visible MIB words 13-20. Image
`38a1733681aa984ac2cf9dc32b9a62c5839634e46a3d73d24107e34106ff4afa`
was cancelled when investigation pivoted from further confirmation to a
corrective sequencing experiment. A mistakenly overlapping launch briefly
contended for the board; both affected runs were terminated and their logs must
be discarded.

### Rejected experiment: post-publication MAC FIQ service

The leaked command starts at `command+0x0c`, which is the first packet-controller
opcode emitted by `emit_prepared_probe_descriptor`: `0x51...`, followed by
`0x50...` and `0x52...`. This is command-fetch content, not common 802.11 frame
data. A concrete vendor/open ordering mismatch surrounds it. Vendor source
`0x16` runs `mac_irq_handler` by immediate FIQ preemption. Open firmware masks
CPU FIQ. `HostTxDriver::service` first drains old MAC events, then may publish a
new class-0 pipe trigger; the main loop subsequently consumes RX packet RAM
without servicing the phase/start event created by that new command fetch. The
new event waits until the next cooperative pass. Vendor never exposes this
TX-fetch-to-RX-consumer interval.

Feature `mac-post-publish-service` adds a hardware-only MAC drain after class-0
publication and immediately before RX FIFO polling. It cannot publish another
TX command or advance software-only contexts; it only executes newly pending
MAC events and routes completions. This is a corrective scheduling change, not
telemetry. Host tests (137), ARM release check and LSP diagnostics pass. Image
`bd3cb892beeca586cfada900b7114d3ab50e6845cd979b3eb0f66f392533dfa7`
was promising but incomplete. One run terminally collapsed (884 Kbit/s TCP,
0/20 final ping). Two runs remained reachable through the end: one delivered
2.59 Mbit/s TCP and 6.82 Mbit/s UDP; the other was degraded at 1.47 Mbit/s TCP
and 877 Kbit/s UDP but finished 20/20 ping. Baseline fresh-boot downlink had
collapsed terminally 3/3, so servicing once before RX appears to reduce terminal
failure but does not close the race. A single immediate readiness sample can
occur before the phase/start event becomes visible.

The corrective path now tracks whether this service pass actually armed a TX
command. Only then it waits up to 64 short readiness polls for the source-0x16
MAC event and drains it before RX packet-RAM access. There is no wait on ordinary
passes and no diagnostic state. Image
`98820e109faa9619b5215f60acfab1ab43ada5aa325c53f72f0b04c706871be1`
terminally collapsed in two of three runs (1.84 Mbit/s and 18.3 Kbit/s
cumulative TCP, both 0/20 final ping). The third delivered 3.98 Mbit/s TCP and
4.95 Mbit/s UDP with 19/20 final ping. Bounded waiting for the MAC phase/start
event is not sufficient; delayed FIQ servicing influences timing but is not the
hardware exclusion mechanism.

### Rejected experiment: TX-fetch/RX-DMA exclusion

Feature `tx-fetch-rx-dma-guard` directly serializes the confirmed shared
packet-RAM collision. Immediately before an armed class-0 publication it saves
`0x09c00600`, clears bit 0 to stop RX packet-DMA writes, and waits up to 1024
short polls for busy bit 23 to drain. It then publishes the TX command. After the
new source-0x16 event is serviced (or the bounded event wait expires), it restores
the exact saved control word. The guard executes only on passes that actually
arm a TX command. This uses the same stop/drain primitive present in vendor
`phy_rx_disable_and_drain`/`mac_rx_restart`, but applies it narrowly to compensate
for the open firmware's missing hardware/FIQ exclusion around command fetch.
Host tests (137), ARM release check and LSP diagnostics pass. Image
`ce175037087b373b2581fbfd3084d11bdeef872bf8540079fc7754c85a054c94`
kept two of three runs healthy (TCP 2.99/2.50 Mbit/s, UDP 7.92/6.37 Mbit/s,
20/20 final ping) but one terminally collapsed at 711 Kbit/s and 0/20. The
guard therefore changes the failure probability substantially but restoring RX
DMA at the MAC phase/start event is too early: that event proves execution has
started, not that command fetch ownership has retired.

Feature `tx-fetch-rx-dma-until-completion` retains the same RX-DMA stop/drain
through the strict matching `(context, frame_node, pipe, slot)` TX completion,
then restores the exact saved control word before confirmation processing. This
is the first software-visible point that proves the command slot has retired.
Image `12c26ad5302c77d1c5d8000a99eca69cfb9f183171af87b119a780a1f11ee752`
failed decisively: all runs fell to about 9.6-9.9 Kbit/s, TXed stayed at 4-5,
and final ping was 0/20. TX completion depends on the receive path, so holding
RX packet DMA disabled through completion deadlocks ordinary TX. All DMA-guard
and cooperative post-publication experiments were reverted.

### Preserved experimental branch: real source-0x16 MAC FIQ

Rust has no ARM FIQ ABI. `cortex-a-rt` explicitly leaves FIQ as a raw assembly
hook because FIQ banks `r8-r14`; its normal IRQ trampoline also uses ARMv7
`SRS/RFE`, unavailable on ARM9/ARMv5. `aarch32-cpu` supplies ARMv4-compatible
FIQ mask/unmask helpers but no context wrapper. Therefore feature `mac-fiq`
installs a minimal ARM veneer into the existing FIQ vector literal: preserve
shared `r0-r7` plus `r12/lr` on the preinitialized 8-byte-aligned FIQ stack,
call a Thumb `extern "C"` Rust handler, restore, and return with
`subs pc, lr, #4` using hardware `SPSR_fiq` restoration.

The Rust FIQ body exclusively drains the destructive MAC event FIFO with the
existing complete bounded vendor dispatcher. It does not drain scheduler tasks
or route host completions; foreground performs those while briefly masking FIQ.
Foreground no longer reads/destructively consumes the MAC FIFO under this
feature. Startup patches only vector literal `__xr819_vectors+0x3c`, enables
source `0x16`, and clears only CPSR F while leaving IRQ masked. Final disassembly
confirms ARM veneer `0x1c4`, odd Thumb handler target, and the shared static MAC
backend (not a promoted stack copy). Host tests (137), ARM release build and LSP
diagnostics pass. Image
`51f417f62e1f3505f24d7aba934ae086c60b4b5c1d0d0a2a3db1c89d577aa5f5`
failed to leave scanning in all three runs, without an exception record. The
veneer returned correctly, but scan/management foreground paths still called
`service_single_probe_runtime_inactive`, creating a second destructive consumer
of `0x09c00a20/24` alongside FIQ. The shared credit path then failed.

Under `mac-fiq`, `service_single_probe_runtime_inactive` now masks FIQ briefly
and drains only scheduler/completion state; it never reads the MAC FIFO. This
covers class-0, scan probe and management callers, making source `0x16` the sole
destructive FIFO owner. The exported FIQ handler uses `black_box` around the
shared static backend pointer; release disassembly confirms it passes static
address `0x00010a58` rather than an optimizer-promoted stack temporary. Image
`648e386a3822079a491671ac66313aeb9a7ed9cbe91255429fa91a9f44885cb8`
still failed scan credits. Several narrower FIQ ownership variants were then
qualified:

- routing without ordinary source enable: no FIQ arrived after join, so TX
  completion stopped;
- enabling source 22 only after key install: FIQ arrived, but direct Rust backend
  execution still broke credits;
- minimal FIQ raw-word queue with foreground event execution: scan/auth was kept
  cooperative until key install, and the hardware drain tail was moved after
  foreground effects, but joined class-0 TX still stopped at 4-5 frames and
  credits failed.

The ARM veneer and banked-register handling were valid and no exception record
was produced. The unresolved part is interrupt-controller/FIFO acknowledgement
and exact source-0x16 ownership semantics, not Rust's basic exception return.
A real FIQ cannot be introduced safely by approximating those semantics. A
later minimal variant enabled FIQ only after key installation, drained raw words
into a 64-entry SRAM ring, executed them in foreground, and moved the drain tail
after event effects. With controller source 22 disabled no FIQ arrived; with it
enabled the first joined class-0 TX stopped at 4-5 frames and host credits
failed. This held even after cooperative scan/auth ownership was preserved until
the post-key transition. All `mac-fiq` code and features were reverted. Do not
resume this path without raw vendor FIQ entry/exit and controller
acknowledgement disassembly beyond `mac_irq_handler` itself.

### Completed correction: genuinely strict TX-status ownership

A feature dependency error invalidated the earlier assumption that watchdog
images had tested strict ownership. `pipe-watchdog` implicitly enabled
`unmatched-tx-status-recovery`, so omitting the latter from the command line did
nothing and reproduced byte-identical image
`6f01365e7ff0221947ddeccfdd39116c6b5a4aa0a3701d584785b4371f5b3604`.
The watchdog also called a helper compiled only by the unmatched feature, which
made the hidden coupling structural rather than declarative.

`pipe-watchdog` is now independent, while its forced-retirement helper is built
for either recovery mechanism. Image
`eb968c54376a2c90025f687a26f3ba25a90afc6fd10ca8d85bf9ef07f84da384`
is the first real image that discards unmatched statuses like vendor and retains
only vendor's bounded pipe-watchdog retirement. It passes 137 host tests, ARM
release check, and clean LSP diagnostics. Its three serialized fresh-boot
results were: one terminal collapse (190 Kbit/s cumulative TCP, 10 TX failures,
0/20 final ping) and two healthy runs (3.51/3.52 Mbit/s TCP, 6.98/7.92 Mbit/s
UDP delivered, zero TX failures, 20/20 final ping). There were no credit-failed
reports. A same-session three-run control with unmatched retirement explicitly
enabled, image
`944a20fb8529cb1fb7ff525990c5fccf31cf5ab74e68f0e056ebfe586c42e0a7`,
terminally collapsed 3/3: 878 Kbit/s, 292 Kbit/s, and 1.39 Mbit/s cumulative
TCP; 8-9 TX failures; final ping 0/20 in every run. No credit-failed report was
present. The matched 2/3 healthy versus 0/3 healthy result establishes that the
non-vendor retirement materially raises collapse probability by retiring the
wrong active slot. Keep the feature decoupling, but do not claim it is the sole
cause: one strict run still collapsed.

Feature `ack-template-watch` performs a cold read only when the counters MIB is
explicitly requested. It verifies all 35 immutable vendor automatic-ACK command
words, records the first mismatch, and captures `0x09c00e8c/e90`. It adds no
recurring hot-path reads. Strict image
`1623c0f2fe009d72411e41d135749c399fc7e5dd214754c8498fa63b73dff879`
produced two terminal collapses (904/218 Kbit/s cumulative TCP, final ping 0/20)
and one healthy run (4.89 Mbit/s TCP, 5.39 Mbit/s UDP delivered, final ping
20/20). All three, including both collapsed runs, reported zero ACK-list
mismatches, `0x09c00e8c = 0xbf`, and `0x09c00e90 = 0x000f0002` on their final
available cold read. Static ACK-template/register corruption is therefore ruled
out; the missing ACKs are a live response-engine/packet-controller symptom.

A focused raw-disassembly audit recovered vendor source-0x16 entry at
`0x1c..0x28` and handler `0x9eb4..0xa05c`. Vendor performs no software
controller pending read, mask, acknowledge, EOI, or rearm in FIQ: it
unconditionally pops `0x09c00a20`, executes all effects synchronously, drains
until signed-empty using `0x09c00a24`, runs the empty tail, and returns through
`ldmia ... pc^`. The matching container's initialized DTCM proves
`*(u32 *)0x04001430 == 0`; ordinary controller bit 22 is not vendor behavior.
Vendor enables FIQ from startup through route `0x1600a037`. The failed late-FIQ
experiments therefore combined a non-vendor ordinary source enable with a
deferred handler contract vendor never uses.

The audit also found one concrete cold-order mismatch: vendor writes
`0x09c00e8c = 0xbf` before constructing `0x09016a28..0x09016ab0`, while open
firmware did the reverse. This is corrected in image
`3bb8a0294f4499f6610ee1bda5b4f2ab9fa0a5daeb940adbe1d418fd160e8c9f`,
which was serialized with strict status ownership. The first attempt
never associated and is discarded. The next two runs were healthy and are the
fastest stable open-firmware downlink samples so far: 5.01/4.72 Mbit/s TCP,
7.92/7.91 Mbit/s UDP delivered, final ping 20/20. The replacement third valid
run terminally collapsed after 15-20 seconds (1.22 Mbit/s cumulative TCP, eight
TX failures, final ping 0/20). The order is vendor-exact and worth keeping, but
its 2/3 healthy rate does not improve on strict ownership alone and it is not the
root cause.

The next exact-FIQ implementation found a concrete flaw in every earlier veneer
experiment: release `rust_main` reserves a `0x15bc`-byte frame from system SP
`0x0400c000` down to about `0x0400aa44`, so vendor FIQ SP `0x0400b500` lies
inside active Rust foreground locals. Each FIQ invocation overwrote the main
loop frame, directly explaining scan-credit corruption despite a correct
exception return. Feature `mac-fiq` now uses FIQ SP `0x0400a800`, in the unused
DTCM gap above vendor initialized state ending at `0x04009c44` and below the
Rust foreground frame. FIQ is enabled from startup through route `0x1600a037`
without ordinary bit 22, drains source 0x16 synchronously and unbounded to empty,
and foreground masks FIQ around every mutable/shared backend access. Image
`5f7b608928fd0de8504912f9fb1d8b8d8a499c7a1e5a4de9b859839d0e0aeeba`
passed 137 host tests, ARM release check, clean LSP, and release-disassembly
checks, but failed before startup because `mac-fiq` disabled cooperative FIFO
service before CPU FIQ was active, accumulating a boot-event backlog. A runtime
ownership transition now keeps foreground cooperative service through the first
valid configuration response, then marks FIQ active and unmasks it atomically.

The first transition image faulted at `0x18e`: Rust emitted a Thumb `bl` directly
into the ARM FIQ-unmask helper because the assembly symbol lacked `%function`
metadata. Marking it as an ARM function produces verified `blx`. The next image
ran but eventually returned to DTCM data address `0x04001720`. Preserving `r12`
and 8-byte AAPCS stack alignment did not change that signature. Release
disassembly then exposed the cause: LTO promoted the small shared MAC backend
into an FIQ-stack temporary and passed SP as the backend pointer, so event
effects overwrote the exception return frame. `service_mac_fiq` is now
`inline(never)` and passes a `black_box`-opaque pointer to static address
`0x00010888`; release disassembly verifies the real static pointer is supplied.
This removed exceptions and credit failures, but the first safe run still
stopped at 5 TX / 114 RX frames with final ping 0/20.

Image `cd1e8656d183cb548034f2992506b782d226ebc76b100aeb0d9837aac360b5eb`
adds only ordinary-SRAM transition counters: FIQ entries/events/empty entries,
maximum event batch, completion queued, foreground runtime calls, scheduler
drains, and completions taken. They are exposed only on explicit cold counters
MIB reads. The final three-run FIQ qualification was mechanically stable but
not curative: run 1 collapsed early, run 2 sustained 5.04 Mbit/s TCP and then
collapsed after UDP with final ping 0/20, and run 3 remained healthy at 4.59
Mbit/s TCP / 7.92 Mbit/s UDP with final ping 19/20. FIQ therefore finished 1/3
fully healthy versus strict cooperative service at 2/3. The complete experiment
is preserved on Jujutsu bookmark `feature/mac-fiq`, now at change `rnrlxtqz`,
commit `0d4c3ee4` (`Fix RX release ownership after corruption`). The original FIQ
checkpoint remains its parent at `88caff1b`.

An adversarial Rust review then found a critical remaining race: foreground
`reserve_non_aggregate_scheduler` removes PAS ownership, rewrites the slot
record, and clears/rebuilds the 16-word packet-controller command stream before
`publish_host_class0_slot` acquires its FIQ guard. FIQ can process the same pipe
while that stream is partial. The guard now covers reservation through
publication or complete rollback; failed publication no longer leaves a
partially built reservation exposed across main-loop passes. The review also
found two terminal RX ownership bugs: `corruption-non-fatal` normalized a corrupt
release-head word and returned before advancing it, and resynchronization moved
`DMA_CONSUMER` across outstanding zero-copy HIF slots. Corrupt heads now continue
through normal release, while a resync defers the hardware release cursor until
all older host transfers return. Scan scheduler-bit RMW now uses the common
IRQ/FIQ-masked set/clear primitive. Image
`7b094ae01bf19cca38ee597aeda6dafaf708d2ca029510d06236f097ca471fe7`
passes host tests, ARM release check and clean LSP. Its serialized three-run
qualification was the first completely healthy set: 4.34/4.80/4.46 Mbit/s TCP,
7.91/7.92/7.93 Mbit/s UDP delivered, zero TX/credit failures or exceptions, and
final ping 20/20, 20/20, 19/20. This is 3/3 healthy versus the mechanically
stable FIQ image at 1/3. Image
`79061908e8893aaa2a4c62ee74c4e10b4669fd813ebcde113b3ff3971ecc5201`
contains the same strict/RX logic corrections without `mac-fiq` and was also
healthy 3/3: 3.85/4.93/4.77 Mbit/s TCP, 7.93/7.92/7.92 Mbit/s UDP delivered,
final ping 20/20 in every run, with zero credit failures or exceptions. FIQ is
therefore not required to close the original collapse. The root correction is
in RX ownership handling. Two three-run destructive rollback variants were run:
image
`80a28edb1aba7b1a93c32652ff0a50471d4bdc4a1c5cadf87dec238be0d856f9`
restores only the old corrupt-release-head early return while keeping safe
resync; image
`d349a7fb1ac3601338d9780ec6c4e6c9aa8ef336d85af33a87814f022496665a`
restores only the old resync release jump while keeping corrupt-head recovery.
The isolation was decisive. Restoring the old corrupt-head early return produced
only 1/3 healthy runs; the other two terminally collapsed at 1.99 Mbit/s and
930 Kbit/s cumulative TCP with final ping 0/20. Restoring only the old resync
release jump remained reachable 3/3 at 5.32/4.96/5.30 Mbit/s TCP and final ping
20/20, although one UDP run degraded to 3.39 Mbit/s. Therefore the original
terminal-collapse trigger was the `corruption-non-fatal` release-head return:
TX command content changed the head ownership word, Rust marked it pending and
returned, and no later owner could revisit or advance the FIFO head. Continuing
through normal head reclamation closes the terminal failure. Deferred resync
release remains as an independent zero-copy ownership correctness fix. Temporary
legacy rollback features were removed after isolation. The FIQ implementation
and FIQ-specific full-transaction guard remain preserved separately on bookmark
`feature/mac-fiq`. A clean production line based directly on validated commit
`12025526` contains strict watchdog/status feature separation, the vendor ACK
initialization order, corrupt-head reclamation, deferred resync release, atomic
scan scheduler-bit updates, and corrected canonical build profiles without FIQ.
Exact production image
`88298e828ef298af0644a4afe0dd0c206849061bc6b501f800bfdb5ca29f4a53`
passes 138 host tests, all `tools/check.sh` ARM profiles, and clean LSP. Its
final serialized qualification was healthy 3/3: 5.09/4.87/5.08 Mbit/s TCP,
7.92/7.93/7.93 Mbit/s UDP delivered, final ping 20/20 in every run, and zero TX
failures, credit failures, or exceptions. This is the production endpoint.
