# XR819 vendor host-TX lifecycle

This document reconstructs the ordinary WSM `0x0004` host-data path from the
vendor firmware. It is a specification for a fresh implementation, not a list
of fields to transplant into the internal class-6 publisher.

The central rule is that one host `xr_tx_ctx` remains the identity of a frame
from WSM request admission through class-0 confirmation. The vendor path does
not allocate an internal class-6 context and later copy it.

## Function chain

```text
HIF/LMC request slot
  -> task_wsmlmac_11df0
  -> wsm_h_04_tx_req                 0x0000b600
  -> tx_wsm_buf_alloc                0x0000b6d0
  -> tx_lmac_req_submit              0x0000dec4
  -> tx_classify_hdr_len             0x0000e3c6
  -> tx_select_key_and_cipher        0x0000e1f0
  -> cipher/MIC queue
  -> post-crypto callback            0x0000dcb4
  -> txq_list_insert(ctx, queue, 0)  0x0000dc40
  -> event 0x00200000
  -> task_b88e                       0x0000b88e
  -> tx_frame_done_release           0x0000d1c4
  -> pas_retime_and_kick             0x000081ac
  -> pas_txq_push_global             0x000081a6
  -> txp_scheduler_run               0x0000aa5e
  -> txq_build_aggregate_lists       0x0000a2c0
  -> txp_build_pipe_descriptor       0x0000a712
  -> MAC pipe events
  -> txp_pipe_tx_success/retry       0x00009cdc / 0x00009550
  -> txp_fn_2441                     0x000091ec
  -> tx_frame_complete               0x0000b74c
  -> class-0 callback
  -> tx_frame_complete_stats         0x0000b9ba
  -> tx_confirm_build_and_send       0x0000b3dc
  -> tx_wsm_buf_free                 0x0000b702
```

## 1. HIF request lifetime

The HIF/LMC layer stores the original request pointer in a 30-slot ring at
`g_lmc + 0x4b8c + slot*4`. `task_wsmlmac_11df0()` passes that same pointer to
`wsm_h_04_tx_req()`.

`wsm_h_04_tx_req()` does not copy the MPDU into a firmware-owned packet buffer.
It stores:

- `ctx+0x00`: original WSM message pointer
- `ctx+0x1c`: borrowed 802.11 frame pointer, normally `msg+0x18`
- `ctx+0x18`: borrowed frame length, normally `MsgLen-0x18`
- alignment/QoS flag set: pointer `msg+0x1a`, length `MsgLen-0x1a`

The original HIF request therefore remains borrowed until the class-0
confirmation path. `lmc_req_confirm_and_release()` eventually clears or
requeues the matching LMC slot. A correct replacement must provide equivalent
stable ownership; immediate HIF buffer recycling is invalid.

## 2. Host context allocation

Host pool:

- base `0x04005a24`
- count 30
- stride `0x170`
- free-list head `0x040087b0`
- in-flight byte `0x04003e9e`

`tx_wsm_buf_alloc()` performs, under IRQ masking:

```text
ctx = free_head
free_head = ctx+0x04
ctx+0x0f = 0
ctx+0x20 = 0x000000fe
ctx+0x70 = 0x00fe
ctx+0x80 = 0
in_flight++
```

`ctx+0xa0` is pool-owned and points to a distinct 0x54-byte packet-SRAM
per-frame descriptor/work area:

```text
0x09003678 + host_index * 0x54
```

It is not merely a completion cookie. The post-crypto callback calls
`txp_submit_to_pipe(ctx+0xa0, ctx+0x54, ctx+0x8a)`, which writes a reusable
per-frame descriptor image into this area. Copying an internal context's
`ctx+0xa0` aliases another frame's descriptor storage.

## 3. WSM request fields copied before submit

`wsm_h_04_tx_req()` copies the fixed request metadata:

| Context | Source |
|---|---|
| `+0x08` | WSM packet ID |
| `+0x0c` | max TX rate |
| `+0x0d` | queue ID |
| `+0x0e` | `more` |
| `+0x0f` | WSM flags |
| `+0x10` | expiry time |
| `+0x14` | HT TX parameters |
| `+0x24` | max TX rate again |
| `+0x18` | borrowed MPDU length |
| `+0x1c` | borrowed MPDU pointer |
| `+0x00` | original WSM message pointer |

The duplicate max-rate byte at `+0x24` is later used by confirmation/rate
accounting and must not be omitted.

## 4. `tx_lmac_req_submit()` initialization

The submit function initializes the same host context in place:

| Offset | Value/meaning |
|---|---|
| `+0x80` | `1` initial ownership state |
| `+0xbd` | interface ID |
| `+0xbf` | link slot from original queue bits 2..5 |
| `+0x0d` | queue reduced to low two bits |
| `+0x40` | submit timer |
| `+0x53` | completion class `0` |
| `+0x50` | zero |
| `+0x52` | `1` |
| `+0x4c` | zero |
| `+0x58` | zero, then policy flags are added |
| `+0x74..+0x7c` | zero |
| `+0x68`, `+0x6c` | submit timer minus one |
| `+0x38`, `+0x3c` | zero |
| `+0x64` | request expiry time |
| `+0x54` | borrowed 802.11 header pointer |
| `+0x60` | AC from queue-to-AC map |
| `+0x61` | PTA priority from WSM flags bits 1..3 |
| `+0x62` | retry-policy index from WSM flags bits 4..6 |
| `+0x5c` | low 16 bits of MPDU length |
| `+0x70` | `0xfe` nonterminal status |
| `+0x72` | zero try count |
| `+0xa4` | zero |
| `+0xa7` | copied from `+0x52`, therefore `1` |
| `+0x90` | zero |
| `+0x63` | requested max rate |

It also:

1. validates the interface/link against VIF `+0x2c`;
2. sets base flag bit 23;
3. calls `txq_set_frame_lifetime(ctx+0x54, 0)`;
4. folds HT parameters into flags;
5. prepares the retry policy with `pas_tx_policy_prepare()`;
6. calls `tx_classify_hdr_len()`;
7. dispatches key/cipher processing.

A fresh Rust host initializer should implement this sequence directly. It
should not call `prepare_probe_context()` and overwrite selected fields later.

## 5. Header classification and crypto handoff

`tx_classify_hdr_len()` is responsible for substantially more than splitting
lengths:

- sets direct-frame bit `0x1000`;
- records frame control at `ctx+0x5e`;
- marks multicast/no-ACK state;
- calculates 3-address/4-address and QoS header length;
- writes header bytes to `ctx+0x44`;
- writes payload bytes to `ctx+0x48`;
- records QoS TID and ACK policy;
- may assign a sequence number;
- sets VIF-slot state used by the compatibility descriptor;
- ends by calling `tx_select_key_and_cipher()`.

The sequence helper is now fully resolved at `0x00000ee4`. For unicast QoS
frames selected for firmware sequence assignment it:

- uses `ctx+0xbd` as the interface and `ctx+0xbf` as the host link;
- starts from `vif+0x12a`, optionally resolving a nonzero host link through
  the 12-byte link-map records rooted at `0x040087b8`;
- indexes the per-link/TID sequence table at
  `0x04008890 + internal_link*0x20 + tid*2`;
- writes the sequence control field at MPDU `+0x16`;
- stores `sequence >> 4` at `ctx+0xa8`;
- advances the table entry by `0x10`, masked with `0xfff0`.

For software CCMP, the replacement for the hardware crypto/MIC pipeline must
enter the same post-crypto callback state described below. It must not jump
directly to pipe publication.

## 6. Post-crypto callback at `0xdcb4`

The recovered callback takes a pointer equivalent to `ctx+0x110`. Translated
back to the outer context, it performs:

```text
if cipher is TKIP-class 2 or 3:
    copy two generated halfwords into frame tail

if ctx+0x70 < 0xfe:
    tx_frame_complete(ctx, existing_status)
    return

ctx+0x4c = 0
choose/allocate optional pipe object from flags/header
ctx+0x4c = result
if result != 0:
    result[7]++

if ctx+0xa0 != 0:
    tx_submit_wrapper(ctx+0x54)
    # exact call: txp_submit_to_pipe(ctx+0xa0, ctx+0x54, ctx+0x8a)

ctx+0x80 |= 0x20
txq_list_insert(ctx, ctx+0x0d, 0)
if ctx+0x0e == 0:
    evt_flags_set(..., 0x00200000)
```

Two corrections to earlier experiments follow directly:

1. The vendor insertion mode is **0 (append at tail)**, not mode 2/prepend.
2. The event is raised only when WSM `more == 0`; a burst's earlier frames rely
   on the final request to kick the task.

The callback also builds the per-frame descriptor at `ctx+0xa0` before queue
insertion. Earlier bounded queue diagnostics omitted this coherent transition.

## 7. Pending-list task `task_b88e`

The list root is `0x04008ad8`, represented as `{head, tail}`. The task walks the
list without blindly removing every frame.

For each context it evaluates:

- global scheduler/radio mask;
- VIF active-link bitmap `vif+0x2c` using `ctx+0xbf`;
- VIF suspended-link bitmap `vif+0x2e`;
- action-frame exception;
- VIF operating state (`vif+0x18` values 4 or 6);
- completion class;
- `txp_program_pipe_hw(ctx+0x54, 0)` eligibility;
- class-0 expiry relative to `ctx+0x40`.

Only then does it remove the node. Removal leads to either:

- `tx_frame_complete(ctx, 0x14/10)` for invalid/expired frames; or
- `tx_frame_done_release(ctx)` for an eligible frame.

A replacement must preserve the distinction between **leave queued**, **reject
and confirm**, and **release to PAS**. Synthetic immediate insertion/removal
bypasses the most important gates in the task.

## 8. PAS release and global ring

`tx_frame_done_release()` performs accounting and power-save follow-up. For a
normal host context (`ctx+0x70 == 0xfe`) it then:

```text
ctx+0x80 |= 0x40
pas_retime_and_kick(ctx+0x54)
```

`pas_retime_and_kick()` recomputes the complete timing image and calls
`pas_txq_push_global()`.

The 64-entry global ring is not a conventional always-tail FIFO. Insertion
uses the PAS byte at `pas+0x53` (`ctx+0xa7`):

- value `1`: append at tail;
- value `0`: prepend at head;
- other values: assert.

Host submission initializes `ctx+0xa7 = 1`, so normal host frames append at
the tail. The ring helper first compacts holes; direct head/tail mutation that
assumes an empty ring is not equivalent.

## 9. Scheduler selection

`txp_scheduler_run()` first derives an idle-pipe mask and calls
`txq_build_aggregate_lists(mask, ...)`.

For each ring entry, `txq_build_aggregate_lists()`:

1. drops expired or invalid frames with status 10;
2. checks `txp_program_pipe_hw()`;
3. maps AC to a hardware pipe and requires that pipe in the idle mask;
4. optionally determines a BA/TID pipe;
5. calls `txq_try_append_to_aggregate()` in aggregate or non-aggregate mode;
6. clears the selected ring slot;
7. marks selection state in the frame.

The minimum non-aggregate scheduler path is therefore not just
`ctx+0x58 |= 0x04000000`. It includes ring scan, expiry, pipe eligibility,
AC-to-pipe mapping, TXOP budget accounting, aggregate-list state, and ring-slot
ownership.

After list construction, `txp_scheduler_run()` calls
`txp_build_pipe_descriptor(..., kind=0)` for a single frame. That function:

- reserves the pipe's current 4-entry slot;
- stores the PAS pointer in the slot;
- sets PAS selected/owned state;
- emits the pipe descriptor with `txp_submit_to_pipe()`;
- builds duration/backoff commands;
- advances the pipe producer index;
- records the new tail index.

The existing direct publisher duplicates only part of this terminal operation
and does not reproduce the scheduler's ownership records.

## 10. Completion and confirmation

On successful pipe completion, `txp_pipe_tx_success()` walks completed pipe
slots and calls `txp_fn_2441()` for every PAS frame.

`txp_fn_2441()` writes:

- completion status at PAS `+0x1c` (`ctx+0x70`);
- completion timestamp at PAS `+0x14` (`ctx+0x68`);
- retry/aggregation flags;
- completion event `0x00200000` for successful class-0-style slots.

The normal completion drain eventually calls `tx_frame_complete()`. Class 0's
callback is `tx_frame_complete_stats()`, which snapshots status/rate/retry and
delay fields, then calls `tx_confirm_build_and_send()`.

`tx_confirm_build_and_send()` builds WSM `0x0404` or coalesced `0x041e`, sends
it through HIF, and finally calls `tx_wsm_buf_free()`.

`tx_wsm_buf_free()` performs the host-pool return under IRQ masking:

```text
ctx+0x20 = 0xff
ctx+0x70 = 0xff
ctx+0x80 |= 0x00040000
ctx+0x04 = free_head
free_head = ctx
in_flight--
```

The internal pool uses a different return bit (`0x00020000`) and different free
list. They must remain separate.

## Implementation boundary

The next firmware implementation should introduce an ordinary host-TX module
with these explicit phases:

1. allocate host context;
2. retain/copy stable WSM frame ownership;
3. initialize exactly as `wsm_h_04_tx_req()` and `tx_lmac_req_submit()`;
4. classify header and perform software CCMP;
5. run the exact post-crypto callback, including `ctx+0xa0` descriptor build;
6. append to the pending list and raise the batched event;
7. run `task_b88e` eligibility/removal;
8. run the complete minimum non-aggregate global-ring scheduler;
9. use the existing MAC event machinery only after vendor-shaped pipe-slot
   ownership has been established;
10. execute class-0 stats/confirmation and host-pool return.

The implementation foundation now exists in
`xr819-firmware/src/vendor_host_tx.rs`: a pure host-context initializer,
mode-0 pending-list append plan, exact PAS-ring compaction/insertion model, and
pending-task decision model with focused tests. It now also owns the exact
30-entry host-pool geometry, lazy-safe allocation, class-0 free transition,
per-index `ctx+0xa0` validation, and an explicit phase sequence that cannot jump
from submission directly to scheduling.

HIF request handling keeps each detached packet-RAM input under an owning
`RequestBuffer`; payload slices borrow that owner, and consuming it is the only
way to obtain the non-copyable `RequestReleaseToken`. Ordinary non-EAPOL data
moves the buffer into `RetainedHostTx`, while management and EAPOL remain on
class 6. `HostTxDriver` makes idle, retained, scheduler-reserved, scheduled, and
confirming ownership mutually exclusive and centralizes reversible RESET
cancellation. A single non-copyable `MacEventQueue` capability is passed to
class-0, class-6, and probe servicing so the shared event FIFO has one explicit
consumer. The ordinary-data
`tx_classify_hdr_len()` path now applies 3/4-address headers, QoS and HT-control
lengths, TID, QoS ACK policy, multicast/no-ACK flags, payload splitting, exact
per-link/TID sequence assignment, VIF-slot selection, and software CCMP directly
to the retained packet-RAM frame. It also recomputes PAS timing afterward.

The dispatcher now calls `enqueue_post_crypto()`: it builds the dedicated
`ctx+0xa0` descriptor, sets ownership bit `0x20`, appends to the pending list
with mode 0, and applies the WSM `more` event gate. It refuses the unresolved
optional per-peer pipe-object branch instead of silently skipping it. RESET can
scan and unlink this queued context before returning the class-0 context and HIF
token, so the new ownership boundary is reversible.

Live `task_b88e` service is now wired for the retained ordinary context. It
reads the vendor global mask, VIF active/effective link masks, operating state,
class-0 lifetime, and the translated `txp_program_pipe_hw()` power-save/TBTT
gate. It preserves leave-queued behavior, unlinks rejected contexts for a WSM
confirmation, and routes eligible contexts through ownership bit `0x40`, PAS
retiming, compaction, and insertion into the real ring at `0x04001578`.

Class-0 rejection confirmations now retain the HIF token until the confirmation
is actually published, then free the host context and recycle the request.
RESET can also remove a context from either the pending list or an unscheduled
PAS-ring slot.

Minimum non-aggregate scheduler reservation is now wired. It derives the idle
pipe mask, maps PAS AC through `0x040002e0`, rechecks lifetime and
`txp_program_pipe_hw()`, removes only the selected PAS ring slot, sets selected
flag `0x04000000`, reserves the current 4-entry pipe slot, and emits the kind-0
descriptor into that slot's command storage. This is a reversible
`SchedulerReserved` phase: RESET restores the PAS ring head/slot, frame flags,
pipe-slot metadata, and command words before freeing the context. Scheduler
expiry is also converted into a class-0 status-10 confirmation.

The irreversible boundary is now wired behind the opt-in feature. Publication
records the consumed pipe slot, advances the producer modulo four, increments
the active-completion count, builds duration/backoff state, programs EDCA
quantum, triggers the MAC pipe, and marks the slot hardware-owned. The ordinary
runtime then cooperatively services the existing MAC event/retry/completion
machinery.

Class 0 no longer returns through the internal class-6 free path. Completion is
retained until a WSM TX confirmation descriptor is available; the confirmation
uses the completed status, final rate, and observed retry count. Only after HIF
publication does the runtime perform `tx_wsm_buf_free()` semantics and recycle
the original borrowed request token. RESET refuses to unwind a hardware-owned
scheduled frame and lets it complete normally.

This is now a complete minimum single-outstanding, non-aggregate
allocation-to-confirmation path suitable for the next gated hardware candidate.
It is not yet a production scheduler: multiple simultaneous ordinary frames,
A-MPDU construction, the optional per-peer pipe object, and exact full
`tx_frame_complete_stats()` accounting remain outside the boundary.

Current gated candidate:

- `/tmp/xr819-vendor-host-tx-sectioned.bin`
- SHA256 `df6e5362864de50317492fe50a9f4c940fed4d3ef37280119354da84e11c6328`
- 58,544 bytes

The earlier flat binary `ee1b221e...` was rejected by the installed sectioned
bootloader with status 6 (`STATUS_BAD_FORMAT`) before firmware execution. It is
not evidence about host TX.

Hardware validation on 2026-08-12 re-established WPA with the known RustCrypto
baseline on full reboot attempt 6. The correctly sectioned candidate then
booted successfully on six full reboots, but every attempt stalled during
management authentication before WPA completion. Persistent logs consistently
show one to three management TX frames outstanding followed by `Missed
interrupt?`, `TX Frames (3) stuck in firmware` or `Timeout waiting for TX
confirm`, and BH termination. The ordinary class-0 path was therefore never
entered and no ordinary-frame result can be inferred. The baseline was restored
and verified at SHA256 `04d9f9a9...`.

A matched current-source control built without `vendor-host-tx-foundation`
(`--no-default-features --features join-sta-experiment`, sectioned SHA256
`bce49f10...`) subsequently failed all six full-reboot WPA attempts with the
same authentication-stage 1-to-3-frame TX stalls and BH termination. Therefore
the available evidence does **not** establish a candidate-specific regression.

A disassembly audit then identified two shared class-6 defects: unlike vendor
FIQ `mac_irq_handler()` at `0x9eb4`, the cooperative runtime drained the MAC
event FIFO only while a frame was outstanding; and the vendor's nonfatal trace
branch in `txp_pipe_tx_start()` at `0x9dea` had been translated into permanent
fatal quiescence. The same mistake existed in the dormant LMC-allocation failure
callback. The corrected class-6 image now drains events between transmissions,
records those two conditions nonfatally, and publishes a WSM exception before
true fatal quiescence. Sectioned image SHA256 `383368d9...` reached WPA
`COMPLETED` by second 3 on each of its first two full-reboot trials, with no TX
stall or BH fatal, and the RustCrypto baseline was restored afterward.

This strongly validates the shared event-service diagnosis. Three further
full-reboot trials also reached WPA `COMPLETED`, establishing six consecutive
clean boots for the corrected class-6 image. A forced TX watchdog remains
deferred because prematurely reclaiming hardware-owned state is less safe and
was not needed for these trials.

The corrected class-0 candidate (sectioned SHA256 `0218db0a...`) subsequently
reached WPA `COMPLETED` on all three boots, so enabling the host lifecycle no
longer regresses management TX. One raw ordinary frame was submitted per trial,
but the transmitter-filtered monitor captured zero ordinary MPDUs. The first
two runs remained free of logged TX/BH failures; the third later received a
mismatched WSM command response and the driver terminated BH. The existing
harness had two confounders: ordinary network traffic could run between WPA and
the explicit test frame, and its debugfs path was derived incorrectly after the
wiphy moved namespaces. The next run must disable IPv6, avoid address/DHCP
configuration, snapshot the correct XR819 debugfs state before and after exactly
one raw class-0 submission, and enable WSM dumps to identify the unexpected
response before changing scheduler code.
