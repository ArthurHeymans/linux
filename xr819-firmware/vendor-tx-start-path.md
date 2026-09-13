# XR819 vendor TX start path versus the Rust firmware

## Scope and conclusion

This note follows the fixed-MCS5, list-first depth-4 A-MPDU path from a non-empty
software TX queue to the first MPDU reaching the air. Addresses refer to
`xr819-decompilation/annotated-main.c`; Rust references are current file:line
locations.

**Conclusion:** the vendor does issue `pac_phy_start_op(1)` early, before queue
selection and descriptor construction, but this is **not itself a PHY hardware
start or a rate-specific transmitter ramp**. The address-proven wrapper plus
structurally decoded command case show that command 1 only publishes PHY state
`3` (unless the retained PHY state is already `5`), stores a 10,000,000-tick
timeout, and starts/restarts a software timer.
The rate-specific PHY register sequence is command 2, issued from
`txp_pipe_tx_start()` only after the MAC reports phase 2 for event type `0x37`.
The vendor uses the same post-GO command-2 ordering.

Our current list-first aggregate path already starts command 1 before descriptor
construction, but then starts it **a second time** immediately before the final
EDCA/quantum/GO sequence. That duplicate is a real vendor delta and contradicts
the no-restart intent documented for the ordinary publisher. It is cheap to
remove, but command 1's actual effects make it an unlikely explanation for a
~1 ms post-GO delay by itself.

No vendor-only early PHY or pipe hardware start was found in the anchored path.
The strongest remaining candidate is therefore a **MAC/packet-controller
GO-to-start latency** (channel access, command fetch, or an internal start-event
handshake), not descriptor construction performed too late. The clearest real
firmware delta inside that window is narrower: vendor handles the `0x37` phase-2
event immediately in FIQ context, while Rust can handle it only in the next
cooperative service pass. That can add roughly one measured 190-265 us pass and
could postpone command-2 settling, but by itself it is not evidence for the full
~1 ms. GO-to-phase-2 timing has not yet been measured.

Evidence labels used below:

- **Address-proven:** direct instruction/decompilation evidence at the cited
  vendor address.
- **Structural:** inferred from control flow, translated state-machine
  semantics, or the absence of a software-visible first-air boundary; not a
  timestamped vendor measurement.
- **Measured:** board/monitor evidence recorded in the cited investigation.

## What “PHY operation 1” actually is

### Vendor wrapper: address-proven; command-case meaning: structural

`pac_phy_start_op()` at `0x00007f10` writes the command byte, clears a secondary
byte, dispatches the command, copies the output state, and starts a timer if the
returned timeout is nonzero (`annotated-main.c:8987-9003`):

```c
/* 0x00007f10 */
*(u8 *)(op + 0x10) = command;
*(u8 *)(op + 0x21) = 0;
phy_state_cmd_dispatch(op + 0x10, op + 0x18); /* 0x00016f6c */
*(u32 *)(op + 0x0c) = *(u8 *)(op + 0x18);
if (*(u32 *)(op + 0x1c) != 0)
    timer_start(op - 8);                     /* 0x0000f2aa */
```

The decompiler hides the switch body behind its `switch8_r3` injection. The
inline table resolves command 1 to `0x00016f8c`; its first instructions compare
the retained state with 5. The shared-case tail is not represented as a normal
Ghidra function, so the exact case meaning is **structural**, corroborated by the
translated state machine at `src/tx.rs:7076-7098`:

```text
command 1 @ 0x00016f8c:
    if retained_phy_state != 5: retained_phy_state = 3
    output_state = retained_phy_state
    timeout = 0x00989680
```

There is no PHY MMIO write and no wait in this command-1 case. Its concrete
operation is: **arm the PHY state machine for a possible command 2 and
start/restart its maintenance timer**.

The rate-specific command-2 case is separate:

```text
0x00016f92  cmp   r1, #5
0x00016f94  beq   0x00017018      ; skip programming when already in state 5
0x00016f96  ldrb  r0, [r0,#1]     ; selected rate/secondary byte
0x00016f98  bl    0x00019fc4
0x00016f9c  movs  r0, #4
0x00016f9e  b     0x00017016      ; retained state := 4
```

`0x00019fc4` stores the secondary byte and tail-calls `0x00019f90`; the latter
performs a short, synchronous PHY register sequence and returns. The translated
writes are visible in Rust at `src/tx.rs:7041-7065`: set bit `0x800` in
`0x0abb8004`, write `0x001400c8` to `0x0abb8014`, `0x0abb8010`, and
`0x0abb8440..0x0abb844c`, then rewrite control `0x0abb800c`. There is no polling
loop in the vendor instruction sequence, although the writes may initiate
asynchronous hardware settling.

### Rust translation

`start_phy_operation_1()` is an exact translation of the command-1 state and
timer publication (`src/tx.rs:7076-7100`):

```rust
write_u8(MAC_PHY_OPERATION_COMMAND, 1);
if read_u8(global_state) != 5 {
    write_u8(global_state, 3);
}
write_u8(output, read_u8(global_state));
write_u32(MAC_PHY_OPERATION_TIMEOUT, 0x0098_9680);
write_u32(MAC_PHY_OPERATION_STATE, u32::from(read_u8(output)));
start_scheduler_timer(MAC_PHY_OPERATION_TIMER, timeout)
```

The conditional command-2 translation is in `service_pipe_tx_start()`
(`src/tx.rs:7580-7623`), with the register writes in
`dispatch_phy_command_2()` (`src/tx.rs:7041-7065`). Thus our actual PHY order is
also:

```text
command 1 before GO -> MAC phase-2 event after GO -> command 2 if operation state == 3
```

It is not “start the PHY only at GO.”

## Vendor ordered sequence: staged software batch to first air

The sequence below begins when PAS frames are present in the 64-entry global
ring (`head != tail`). “First air” is not observable in the vendor code; the
last step is therefore structural, constrained by the monitor capture.

1. **Scheduler establishes idle-pipe mask and performs exceptional repair only
   when requested.** `txp_scheduler_run()` (`0x0000aa5e`) marks pipe bits idle
   when pipe state `+0xa3` is zero; control bit 3 can invoke `txp_fn_4425()`
   (`0x0000038c`) before normal scheduling (`annotated-main.c:12760-12780`).
   `txp_fn_4425()` advances/synchronizes slots, clears abandoned ownership, and
   asserts that the software producer equals hardware cursor
   (`annotated-main.c:580-650`). This is a repair/rearm path, not an unconditional
   normal-batch start.

2. **Command 1 is issued before queue scan and descriptor construction.** In
   `txq_build_aggregate_lists()` (`0x0000a2c0`), the vendor tests global-ring
   `head != tail`, then executes `mov r0,#1` at `0x0000a4f2` and calls
   `pac_phy_start_op()` at `0x0000a4f4`, before iterating any PAS entry
   (`annotated-main.c:12400-12408`):

   ```c
   if (head != tail) {
       scan_head = head;
       pac_phy_start_op(1);              /* 0x00007f10 */
       for (; head != tail; head = (head + 1) & 0x3f) {
           frame = ring[head];
           ...
       }
   }
   ```

3. **Queue entries are checked, grouped, and detached.** The scan applies
   expiry and `txp_program_pipe_hw()` gates, maps AC to an idle physical pipe,
   checks BA/link/rate compatibility, calls `txq_try_append_to_aggregate()`,
   clears selected global-ring slots, and records aggregate ownership
   (`annotated-main.c:12408-12467`).

4. **The scheduler claims the pipe and builds the complete A-MPDU command.** For
   aggregate state 2, `txp_scheduler_run()` sets the link state to 6 and calls
   `txp_build_pipe_descriptor(..., kind=1)`
   (`annotated-main.c:12782-12800`). `txp_build_pipe_descriptor()`
   (`0x0000a712`) allocates the packet-list descriptor, walks the MPDU chain,
   emits delimiters/spacing and PHY words, constructs the duration command, and
   finally advances the 4-slot producer (`annotated-main.c:12555-12650,
   12677-12679`).

5. **GO is explicitly held low; EDCA and TXOP quantum are programmed.** The
   common scheduler tail writes `ring+0x14 = 0` at `0x0000ac84`, updates the
   cached interface EDCA word if needed (`0x0000acbe-0x0000acc6`), and writes
   the pipe duration quantum at `0x0000acf6`
   (`annotated-main.c:12801-12823`).

6. **The MAC pipe is triggered and every staged slot is published before one
   GO.** The vendor writes `(1 << pipe) << 25` at `0x0000ad04`, then for
   producer through last: increments active count, marks slot state 1 at
   `0x0000ad2c`, and pushes the slot duration into the hardware ring at
   `0x0000ad32`. It sets pipe state 1, `control |= 1`, watchdog 5, then writes
   `ring+0x14 = 1` at `0x0000ad4c`, `0x0000ad52`, `0x0000ad56`, and
   `0x0000ad5a`, respectively (`annotated-main.c:12823-12837`):

   ```c
   MAC_PIPE_TRIGGER = (1 << pipe) << 25;
   do {
       active_tx_count++;
       slot->state = 1;
       *ring = slot->duration;
   } while (slot != last);
   pipe->state = 1;
   pipe->control |= 1;
   pipe->watchdog = 5;
   ring->go = 1;
   ```

7. **A MAC phase-2 event invokes the TX-start handler in FIQ context.** The
   vendor `mac_irq_handler()` (`0x00009eb4`) decodes event type `0x37` and phase
   `0x20000`, then directly calls `txp_pipe_tx_start(pipe)` at `0x00009f06`
   (`annotated-main.c:11984-11999`).

8. **TX-start marks the slot and conditionally programs the PHY.** At
   `txp_pipe_tx_start()` (`0x00009dea`), slot state becomes 2 at `0x00009e1a`.
   The operation-state comparison is at `0x00009e20-0x00009e24`; when it is 3,
   command 2 is prepared at `0x00009e2c-0x00009e36` and dispatched at
   `0x00009e3a`. Output 4 advances operation state to 4; otherwise scheduler bit
   18 is raised (`annotated-main.c:11902-11926`). It may also assign a sequence value
   and advance `current` toward `last` (`annotated-main.c:11927-11941`).

9. **Hardware performs channel access, fetches the command/data, and emits the
   A-MPDU.** No firmware write between `txp_pipe_tx_start()` and first air has
   been identified. The monitor result constrains this stage: four MPDUs are
   back-to-back, p50 215 us apart, with one BA 2 us after the last member
   (`rx-performance-investigation.md:3478-3508`).

## Rust ordered sequence for the measured list-first depth-4 path

1. **Wait until the target pipe has no runtime owner.** `publish_ready_batch()`
   excludes every candidate whose pipe is present in `owners`
   (`src/host_tx_driver.rs:591-624`, especially `:615-616`). Therefore the
   current code does not begin the same-pipe aggregate transaction while the
   previous batch is still hardware-owned.

2. **Collect two to four compatible PAS contexts, then enter
   `publish_list_first_depth_four()`.** Candidate gathering and invocation are
   at `src/host_tx_driver.rs:750-843`; the call is `:820-823`.

3. **Perform eligibility/ring/descriptor-resource checks, then issue command 1.**
   `publish_list_first_depth_four()` validates all members, requires pipe state
   zero, locates the PAS slots and command storage, takes a software-record
   descriptor, then calls `start_phy_operation_1()`
   (`src/vendor_host_tx.rs:1469-1569`; call at `:1566-1568`). This is before
   aggregate-chain mutation and before command construction.

4. **Detach members and construct the aggregate descriptor.** The code clears
   PAS-ring slots, writes the member table/link state, and calls
   `prepare_host_ampdu()` (`src/vendor_host_tx.rs:1577-1643`). The builder emits
   per-member transfer words, spacing commands and PHY words, fills the pipe
   slot, and builds duration state (`src/tx.rs:5865-6036`).

5. **Issue command 1 a second time.** `publish_list_first_depth_four()` calls
   `publish_planned_host_ampdu()` at `src/vendor_host_tx.rs:1644-1648`.
   That function unconditionally calls `start_phy_operation_1()` again at
   `src/tx.rs:6222-6225`, after descriptor construction and shortly before GO.
   Since `timer_start()` restarts an active timer, this discards the timer age
   established by the earlier call. Normally it simply republishes the same
   state 3 (or preserves state 5).

6. **Set current/last, hold GO low, and program EDCA/quantum.** This is
   `src/tx.rs:6226-6267`; `ring.go() = 0` is `:6238`.

7. **Publish/arm/GO in vendor order.** `finalize_staged_pipe()` writes the pipe
   trigger, marks each slot state 1, pushes durations, sets pipe state 1,
   `control |= 1`, reloads the watchdog, and writes GO 1
   (`src/tx.rs:9010-9056`; trigger `:9028`, GO `:9053`). The measured list-first
   A-MPDU reaches this staged-batch GO, not the single-slot GO at
   `src/tx.rs:8990`.

8. **Return from the service pass.** MAC servicing occurs near the beginning of
   a later `HostTxDriver::service()` call, only when a hardware owner exists
   (`src/host_tx_driver.rs:241-278`; executor call `:262-264`). Because batch
   publication is at the end of the pass (`src/host_tx_driver.rs:334-339`), an
   event produced after GO cannot be handled in the same pass.

9. **Cooperatively pop phase 2, run TX-start, and conditionally dispatch command
   2.** The event adapter maps type `0x37`, phase 2 to
   `service_pipe_tx_start()` (`src/tx.rs:3566-3585`). MAC FIFO admission is
   cooperative rather than FIQ-driven (`src/tx.rs:5710-5729`). TX-start and
   command 2 are `src/tx.rs:7580-7623`.

10. **Hardware emits the A-MPDU and later reports success.** Measured GO to
    first drain is 2073 us for the matched depth-4 population; subtracting
    ~845 us air and ~200 us completion/confirm latency leaves about 0.8-1.0 ms
    before first air (`rx-performance-investigation.md:3501-3508`).

## Concrete deltas

| Delta | Vendor | Rust | Significance |
| --- | --- | --- | --- |
| Command-1 count per selected aggregate | Once, before global-ring scan (`0xa2c0`; `annotated-main.c:12400-12406`) | Twice: before descriptor construction (`vendor_host_tx.rs:1566-1568`) and again after construction (`tx.rs:6222-6225`) | Real ordering difference. Cheap to remove, but command 1 has no PHY MMIO and no wait. |
| Start-event execution | Direct FIQ call from `mac_irq_handler()` (`0x9eb4 -> 0x9dea`; `annotated-main.c:11984-11999`) | Cooperative event pop in a later service pass (`tx.rs:3566-3585,5710-5729`; `host_tx_driver.rs:241-264,334-339`) | Strongest plausible firmware-created post-GO bubble. Magnitude unmeasured. |
| Ability to arm command 1 while another pipe/batch is active | `txq_build_aggregate_lists()` calls command 1 whenever the global ring is nonempty, before per-frame idle-mask acceptance (`annotated-main.c:12400-12420`) | Same-pipe candidates are excluded while owned (`host_tx_driver.rs:615-616`); command 1 is not called until publication is attempted | Structural overlap delta, but command 1 is only state/timer publication. No evidence it hides a PHY ramp. |
| Descriptor/trigger/GO order | Descriptor, GO=0, EDCA/quantum, trigger, slot durations, arm, GO=1 (`annotated-main.c:12789-12837`) | Same order (`tx.rs:5865-6036,6226-6276,9010-9056`) | No missing early hardware pipe-start write was found. |
| Rate-specific PHY command | Command 2 in post-GO `txp_pipe_tx_start()` when operation state is 3 (`annotated-main.c:11915-11925`) | Same conditional order (`tx.rs:7610-7622`) and same register sequence (`tx.rs:7041-7065`) | Not an early vendor-only operation. A state-value mismatch remains possible. |
| Exceptional pipe repair | `txp_fn_4425()` only when scheduler control bit 3 is present (`annotated-main.c:12767-12770`) | No equivalent unconditional call in the measured publish path; cursor diagnostics exist around ordinary publication (`tx.rs:5832-5842`) | Low-probability unless the control bit or cursor is commonly divergent. |

The comment at `src/tx.rs:5821-5823` correctly says the ordinary
`publish_host_class0_slot()` must not restart command 1. It does **not** describe
the current list-first aggregate path, which does restart command 1 at
`src/tx.rs:6223`.

## Ranked candidate mechanisms and discriminators

### 1. MAC/packet-controller per-GO start latency or hidden start handshake

**Why ranked first.** The monitor capture places 0.8-1.0 ms before first air
while the A-MPDU itself is compact (`rx-performance-investigation.md:3478-3508`).
The vendor and Rust normal publication orders match: both construct the complete
descriptor, hold GO low, program EDCA/quantum, write the pipe trigger, publish
all duration entries, arm the pipe, and write GO once
(`annotated-main.c:12782-12837`; `src/tx.rs:5865-6036,6226-6276,9010-9056`).
The term survives supply-clean batching, one-second idle gaps, and watchdog
reload 2/5/10 (`rx-performance-investigation.md:3382-3449`). Command 1 is
already issued before construction in Rust and is not a PHY MMIO launch. No
missing vendor pre-GO hardware write has been identified.

This leaves channel access, hardware command/data fetch, or a controller
handshake around the phase-2 start event as the best-supported location. It is
not yet proven that vendor firmware avoids this cost; vendor throughput may
amortise or overlap it through behavior outside this one anchored idle-pipe
transaction.

**Cheapest discriminator.** Timestamp GO (`src/tx.rs:9053`) and dequeue of the
first `0x37/phase 2` event (`src/tx.rs:3577-3585`), then retain the existing
first-completion timestamp. If GO -> phase 2 consumes most of the residual, the
delay is inside the MAC before its start event. If phase 2 is early while PHY
state is already 5, the remainder is after the event in fetch/channel access.
Also correlate the descriptor's random-backoff value with GO -> phase 2.

**Cheapest falsifying test.** GO -> phase 2 is no more than one cooperative pass,
and phase 2 -> first completion is fully accounted for by measured A-MPDU
airtime plus the known completion margin, with no remaining ~0.8-1.0 ms excess.
That would falsify a hidden MAC start handshake in this interval and instead
challenge the GO/air population match or the subtraction used to derive it.

**Vendor difference.** None established in the anchored scheduler path. A
continuously armed or differently overlapped multi-pipe vendor schedule remains
structurally possible but is not address-proven here.

### 2. Delayed phase-2 TX-start handling, possibly followed by command-2 settling

**Evidence.** This is the clearest normal-path execution difference after GO.
Vendor processes event `0x37/phase 2` in FIQ and immediately enters
`txp_pipe_tx_start()` (`0x9eb4 -> 0x9dea`;
`annotated-main.c:11984-11999`). Rust writes GO at the end of its service pass
(`host_tx_driver.rs:334-339`, `tx.rs:9053`) and services MAC events near the
beginning of a subsequent pass (`host_tx_driver.rs:252-264`). The handler
contains the only identified post-GO rate-specific PHY programming
(`tx.rs:7610-7622`).

**Why ranked second.** This is structural, not timing-proven. Cooperative passes
are roughly 190-265 us with no >1 ms stalls, so next-pass service alone cannot
explain the full ~1 ms. It matters only if the MAC waits for the handler, or if
command 2 launches a longer asynchronous settle. If steady-state PHY operation
state is 5, command 2 is skipped and this candidate narrows to at most one pass.

**Cheapest discriminator.** In the same probe as candidate 1, timestamp TX-start
entry and command-2 entry/exit (`src/tx.rs:7041-7065,7580-7623`), and count
phase-2 events with `MAC_PHY_OPERATION_STATE` 3 versus 5.

**Falsifier.** GO -> phase-2 dequeue is below one pass and a bounded fast-service
A/B does not reduce GO -> first air; or steady-state operation state is 5 and
command 2 is almost never executed.

**Vendor difference.** Immediate FIQ execution rather than cooperative
next-pass execution.

### 3. Our aggregate path's second command-1 call destroys useful early state/timer overlap

**Evidence.** The first call is at `src/vendor_host_tx.rs:1566-1568`, before
chain/descriptor construction. The second is at `src/tx.rs:6222-6225`, after
construction. Vendor calls once at `0xa2c0` (`annotated-main.c:12403-12406`).
Vendor `timer_start()` restarts an already-active timer, so the first timer age
is lost. This is the exact “issued early versus issued at GO” shape in the
question.

**Why not ranked higher.** Address-proven command-1 code performs no PHY MMIO,
no wait, and no asynchronous completion launch; it only changes software state
and starts/restarts a long timer. Actual command 2 remains post-GO in both
firmwares. The earlier command-1 A/B improved reliability but not steady
throughput (`xr819-session-handoff-2026-08-15.md:114-132`).

**Cheapest experiment.** Remove/guard only the call at `src/tx.rs:6223` when the
list-first caller has already issued command 1. Keep the call for callers that
do not pre-arm. Re-run the same CYC/monitor window.

**Falsifier.** Unchanged GO -> first-air residual. Also decisive: timestamped
operation state and timer show no transition between the two calls.

**Vendor difference.** One command-1 call, not two.

### 4. Retained PHY state differs, causing command 2 on every Rust batch while vendor stays ready in state 5

**Evidence.** Command 1 preserves state 5; command 2 is skipped when retained
state is 5 (`0x16f8c-0x16f90`, `0x16f92-0x16f98`). Both vendor and Rust move
through state 3 -> command 2 -> state 4, and the success path can dispatch
command 3 toward state 5 (`src/tx.rs:7549-7566`). The Rust timer/scheduler-bit-18
maintenance lifecycle is incomplete: the vendor registers timer callback
`0x00002aa3`, while Rust replaces it and does not claim scheduler bit 18
(`xr819-session-handoff-2026-08-15.md:1513-1528`). A wrong retained state could
therefore turn a normally skipped command 2 into a per-batch operation.

**Cheapest experiment.** Count/log `(retained_state, operation_state)` at the
first and second command-1 calls and at every TX-start; count actual command-2
executions per GO.

**Falsifier.** Steady-state batches enter GO/start with retained/operation state
5 and command-2 count near zero.

**Vendor difference.** None in the intended state machine; the possible delta
is our incomplete maintenance allowing different live state.

### 5. Random-backoff/CW state or duration-command contents add the pre-air wait

**Evidence.** `tx_build_duration_desc()` (`0x000090c4`) draws a masked random
backoff and encodes it in descriptor word 1, then emits the duration opcode
(`annotated-main.c:10900-10975`). Rust does the same in
`build_single_frame_duration()` (`src/tx.rs:3036-3096`), and the A-MPDU builder
reuses that routine (`src/tx.rs:5981-5992`). Ordering is the same, but live CW,
RNG, or reset/growth state can differ. An older zero-backoff experiment moved a
much larger first-start measurement by about 1 ms but did not remove the gap;
it was not the current matched depth-4 monitor test.

**Cheapest experiment.** In the current depth-4 image, record the chosen random
value per GO and bucket GO -> phase-2/first-drain by it. Then A/B a current-path
`random = 0` build without changing EDCA or retry policy.

**Falsifier.** No correlation and no timing shift with forced zero.

**Vendor difference.** No code-order difference found; only live backoff-state
maintenance may differ.

### 6. Missing normal use of `txp_fn_4425()` / pipe cursor rearm

**Evidence.** Vendor can call `txp_fn_4425()` before scheduling when pipe control
bit 3 is set (`annotated-main.c:12767-12770`). The function advances hardware
slot state and asserts producer/cursor agreement (`0x0000038c`;
`annotated-main.c:603-650`). Rust does not call it in the measured normal
aggregate path. A stale cursor could make the controller consume an empty slot
or wait for a rearm boundary before reaching the published command.

**Cheapest experiment.** Record pipe control bit 3, producer/current/last, and
hardware cursor immediately before each aggregate publication. Invoke the exact
repair only when the vendor gate is present; do not make it unconditional.

**Falsifier.** Gate never set and cursors agree on all delayed batches.

**Vendor difference.** Conditional repair exists in vendor; no evidence it is
active in the normal measured path.

### 7. Command-1 timer expiry/command-4 maintenance

**Evidence.** The timer lifecycle is incomplete in Rust, but command 1 uses a
10,000,000-tick timeout and repeated TX batches restart it
(`annotated-main.c:8995-9002`; `xr819-session-handoff-2026-08-15.md:1523-1528`).
The measured delay is unchanged after one-second idle gaps
(`rx-performance-investigation.md:3423-3428`).

**Cheapest experiment.** Record timer active/deadline and scheduler bit 18 at
GO/start. If desired, port only the exact callback/task observation before any
behavioral change.

**Falsifier.** Timer remains far from expiry and bit 18 is absent throughout
delayed batches.

**Vendor difference.** Complete callback/task maintenance versus the Rust
placeholder, but it does not match a per-batch ~1 ms hot-path delay.

## Single most valuable next action

Add one compact CYC probe that timestamps **GO -> `0x37/phase-2` dequeue ->
TX-start entry -> command-2 entry/exit -> first completion**, while recording
retained/operation PHY state and the descriptor's random-backoff value. This one
measurement separates:

- cooperative event-service delay;
- repeated versus skipped PHY command 2;
- post-handler MAC/PHY settling;
- descriptor/channel-access delay.

Before the hardware run, remove or gate the duplicate command-1 call only if a
clean A/B image can be produced; otherwise leave behavior unchanged for the
instrumentation run. The timestamp probe is more valuable than immediately
moving command 1 earlier, because current code already issues it before
descriptor construction and address-proven vendor code shows it is not the
hardware ramp itself.
