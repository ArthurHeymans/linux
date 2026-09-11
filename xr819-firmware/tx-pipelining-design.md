# TX pipelining design (staged-plan step 12)

## Measured target

Hoeve stage-latency capture (`/tmp/xr819-hlh-stages-metadata.log`,
analyzer `/tmp/analyze-stages3.py`, 27,239 matched timelines):

| Stage | p50 |
| --- | --- |
| driver queue → SDIO write done | 0.9 ms |
| write done → confirmation (firmware + ~0.25 ms air + host RX) | 21.2 ms |

Delivered throughput equals confirmations × frame size one-to-one, and the
air is flawless (all-success MCS7). The ~8 ms publish→complete→confirm→credit
cycle is fully serialized by the single-owner gate, and each frame waits ~2-3
cycles in pending/PAS. Hiding the cycle with a second outstanding batch per
pipe should roughly double throughput at the same offered load
(~5 → ~10 Mbit/s no-BA; BA runs scale by depth on the identical cycle bound).

## Established hardware/software contract

- Pipe state: 0 = idle, 1 = active. Set to 1 at GO (`finalize_staged_pipe`:
  slots→1, durations to FIFO, state 1, control\|=1, watchdog 5, `ring.go()=1`,
  `PIPE_IRQ_TRIGGER`), cleared to 0 only on full drain (`service_pipe_tx_success`
  when `current == last`: walks `current_slot..=last` through
  `complete_tx_pipe_slot`, advances past last, state 0).
- Active window is `[current_slot .. last_slot]` (mod 4); the MAC advances
  `current` (mirror 02) per slot completion. Completion routing is already
  per-`(pipe, slot)` identity, and `Class0RuntimeOwners` already tracks
  `by_slot[16]` plus `occupied_slots`/`slot_owner` — only the `contains_pipe`
  publication veto enforces single-ownership.
- Reservation bakes in the idle requirement twice: `idle_pipe_mask` feeds
  `non_aggregate_scheduler_decision`, and first-member slot selection reads the
  pipe's current-slot cursor (correct only when idle). In-batch members already
  stage consecutive ring slots (`reserve_non_aggregate_scheduler_in_batch`).

## Vendor counter-evidence (2026-09-12): vendor likely does NOT pipeline either

`txp_scheduler_run` (annotated-main 0xaa5e) feeds only pipes whose +0xa3 byte
is clear, and `txp_pipe_tx_success` (0x9cdc) clears +0xa3/+0xa4 only on full
drain (`current == last`: walks slots through `txp_fn_2441`, advances past
`last`, state 0). `txp_fn_4155` (0x101f4) walks every outstanding slot on
watchdog expiry — confirming our watchdog's current-slot-only retirement is
weaker than vendor, independent of pipelining. Three table bases
(DAT_0000ab0c/ab0b8/10520) share the 0x6c stride, so the +0xa3 comparison
across scheduler/completion/watchdog is structural, not address-proven — but
the shape matches our single-owner gate exactly.

If vendor runs one outstanding batch per pipe like us, its 29.5 Mbit/s comes
from a faster cycle and/or deeper batches, not from multi-batch pipelining.
The observed scaling (no-BA ~5, depth-4 BA ~9) already shows throughput =
depth x cycle-rate with a fixed ~8 ms cycle. Therefore:

1. **Depth first**: depth-8 A/B on Hoeve (feature stack already in-tree).
   ~2x at constant cycle confirms fixed-cost cycle and buys throughput with
   zero hardware-contract risk.
   *Status 2026-09-12: BLOCKED on the ARM stack gate (6992/6912, +240B in
   retry-planning frames from 4→8 arrays). Ledger records the confirmation
   of the fixed-cycle model from existing data instead; depth-8 hardware
   waits for a stack trim as part of depth productionization.*
2. **Cycle surgery second**: firmware-internal GO/completion/confirm
   timestamps to split the firmware cycle from host turnaround, then cut the
dominant hops.
3. **Multi-outstanding demoted to fallback**: only if the cycle cannot
   shrink and depth caps out. The window protocol in this doc stays valid for
   that case; the second-GO question stays open.

## Cycle surgery (2026-09-12, replaces pipelining as the active plan)

CYC3 measured the 8.6 ms cycle: GO->drain 1.8 ms (~1 ms air + ~0.8 ms
drain), drain->confirm 0.37 ms (healthy), confirm->GO 3.5 ms (the target).
The single GO-on-idle-pipe hardware contract is preserved; the work is
removing passes and round trips inside confirm->GO:

1. HIF multi-dispatch (admit N ready requests per pass, vendor behavior).
2. Pro-active staging + same-pass GO on pipe retire.
3. Deep retained backlog across all 30 contexts.

Measured outcome (CYC4, 2026-09-12): multi-dispatch is null in A/B, and the
pipe-0 pass partition reports **zero** idle-blocked passes across 310k passes
in a full run — every idle pass was work-free, so items 1 and 2 have nothing
to act on and the frames simply are not in the firmware when the pipe
retires. The 3.1 ms confirm->GO is a host request-response round trip
(confirm read, mac80211 wake, SDIO write), with the pipe idle for roughly a
third of wall time waiting on it. Item 3 only helps if the host really keeps
the firmware supplied; at one batch per round trip the remaining lever is
depth (amortize one round trip over more frames), still blocked on the stack
gate. A transport-pending clause in the same pass hook is the follow-up that
makes the starvation verdict airtight. See the CYC4 entry in
`rx-performance-investigation.md`.

## Original pipelining sketch (fallback case)

1. **Publication gate** (`host_tx_driver.rs`, two `publish_ready_batch`
   variants): replace the `owners.contains_pipe(pipe)` veto with a free-slot
   check against the MAC window — allow staging while `staged_count(pipe) < 2`
   (MVP: one active + one staged batch) and the new window
   `[(last+1)&3 .. (last+len)&3]` does not intersect `[current..last]`.
2. **Reservation** (`reserve_non_aggregate_scheduler_at` + decision fn): new
   "pipe active with staged capacity" branch that skips the idle-pipe
   requirement and selects base slot `(last+1)&3` instead of the current
   cursor. Window-full (`(last+1)&3 == current`) refuses, as today.
3. **GO extension**: after staging batch 2, update `last_slot`, refresh the
   watchdog, and re-trigger (`ring.go()=1` + `PIPE_IRQ_TRIGGER`). Pipe state
   stays 1.
4. **Watchdog** (`service_pipe_watchdog_tick`, `Expired` arm): walk *all*
   outstanding slots on the pipe (vendor `txp_fn_4155` behavior at 0x101f4)
   instead of retiring only the current slot.
5. **Per-slot audit (no change expected, must verify)**: `route_hardware_completion`
   / `route_hardware_requeue` (identity-routed ✓), BA evidence handling
   (currently evidence-only, no direct retirement ✓ — keep it that way during
   this work), selective/whole retry re-arm (slot-indexed ✓ — confirm no
   `first_in_pipe` assumption), `complete_ordinary_status` hook (per-slot ✓).

## Safety invariants

- Staged window ∩ MAC active window = ∅, always; window-full refuses.
- Every staged slot has exactly one software owner (`HardwareOwner` as today).
- No second GO while `mac_retry_hardware_state` / receive-gate busy (existing
  reservation guards stay).
- Completion identity `(context, frame_node, pipe, slot)` unchanged;
  confirmations keep `completion_order` publication.

## Open hardware questions (answer before/with first code)

1. **Second-GO semantics on an active pipe.** Does re-writing `ring.go()=1` +
   IRQ trigger while state==1 extend the window cleanly, or does the MAC latch
   the window at the first GO? Static lead: check whether vendor `txp` code
   re-GOs active pipes (annotated-main refs around `txp_fn_4155`/GO writes);
   hardware fallback: stage-then-GO with a completion-identity assertion and
   the existing postmortem harness on first failure.
2. **Who consumes `last_slot` updates** — DTCM-read per slot or latched window?
   Determines whether updating `last_slot` before re-GO is sufficient.
3. **Watchdog walk shape** — mirror vendor `txp_fn_4155` exactly rather than
   inventing a loop (the current single-slot retirement is already flagged as
   a latent stall mechanism).

## Validation

- MIB cycle counters first: completions/s at fixed offered load should ~2x
  with the feature vs without (same firmware otherwise).
- Hoeve A/B throughput (the reserve A/B harness pattern, minus its reboot
  race — settle on recovery state between phases).
- Re-run the stage-latency capture: queue→confirm p50 should fall toward the
  ~8 ms single-cycle floor while throughput doubles (Little's law in reverse).
- Regression: BA-session runs, watchdog-expiry runs, and the FIFO-order
  probes stay clean; any postmortem capture is a stop-ship signal, not data.

## Explicitly out of scope

- 3-4 outstanding batches (MVP is 2; revisit after it measures clean).
- Changing the retry/fallback policy, BA handling, or rate feedback in the
  same change. Those subsystems are exonerated by the capture and stay frozen
  so the A/B attributes only to pipelining.
