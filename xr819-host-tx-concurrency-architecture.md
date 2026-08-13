# XR819 host-TX concurrency architecture

## Status

This document is the implementation plan following the vendor-firmware audit of
`wsm_dispatch_cmd`, `wsm_h_04_tx_req`, `tx_wsm_buf_alloc/free`,
`tx_confirm_build_and_send`, `task_b88e`, PAS scheduling, and RESET handling.
It supersedes attempts to add a FIFO above the single-outstanding executor or to
reduce the advertised HIF input-buffer count.

The current validated baseline remains commit `9f0b0896` with native XR819 WSM
and one outstanding ordinary class-0 frame. The request arena and single-owner
executor are implemented, but burst validation exposed an incomplete HIF output
model. The next implementation must preserve the executor while reproducing the
vendor's atomic request-credit and 64-entry output-staging lifecycle.

## Vendor facts that constrain the design

- There are 30 host HIF input buffers and 30 hardware-defined host TX contexts.
- A TX context retains the original 1632-byte packet-RAM request; crypto and TX
  operate directly on its MPDU.
- The original request buffer remains owned until its TX confirmation is built.
  Completion selects the owner by context identity, but completed request-buffer
  credits are appended through one ordered HIF publication path.
- Synchronous WSM commands are dispatched independently while TX contexts remain
  pending. Command progress must never depend on class-0 completion.
- TX completion and confirmation may be out of order. `packet_id` and context
  identity, not FIFO order, select the owner.
- The vendor pending list supports mid-list removal and both head/tail insertion.
- Rate-policy uploads mutate a global table. Contexts retain the policy index and
  consult current policy contents later; the open firmware should preserve this
  behavior initially.
- RESET is a barrier: stop admission for the affected VIF, cancel reversible
  owners, allow irreversible hardware owners to complete, publish their
  confirmations, and confirm RESET only after the affected ownership set drains.
- Command responses and TX confirmations overwrite their original 1632-byte
  request buffers. Independently allocated `0x08xx` indications use the four
  384-byte firmware buffers.
- Vendor publication atomically appends a confirm-class request buffer to the
  input-credit FIFO and to a 64-entry firmware-to-host software queue. Up to four
  queued messages are staged in hardware descriptors.
- Descriptor reclaim advances the software output FIFO, refills free hardware
  descriptors, and releases indication buffers. It does not control confirm
  request-credit return.

## Rejected designs

### FIFO of retained requests above `HostTxDriver`

This did not model vendor concurrency. It coupled command progress and
confirmation publication to a software FIFO and eventually caused a mismatched
synchronous WSM response and fatal BH exit.

### Withholding HIF descriptors while class-0 is active

This starved synchronous commands behind data traffic. Reducing the startup
credit count to one did not solve the underlying command-lane dependency.

### Immediate busy failure for the second TX request

This preserved ownership but converted ordinary bursts into artificial TX
failures and made sustained TCP unusable.

## Target architecture

### Fixed host-context arena

Use exactly 30 slots. A slot index maps directly to the vendor hardware context:

```text
context_address = 0x04005a24 + index * 0x170
```

The hardware address is authoritative. A debug-only generation may detect stale
software handles in host tests, but release correctness must never depend on a
generation counter.

Each occupied slot owns one non-copyable `RetainedHostTx`, including its original
HIF request release token.

### Separate ownership and lifecycle

The implementation must make revocability explicit:

```text
HardwareOwnership = Reversible | HardwareOwned
Stage = Retained | Pending | PasQueued | Reserved | Scheduled |
        ConfirmationReady
```

The irreversible boundary remains scheduler publication. No hardware-owned slot
may be cancelled, freed, or reused.

### Independent lanes

The cooperative loop has independent responsibilities:

1. HIF request dispatcher
2. synchronous command lane
3. host-context arena and pending/PAS policy executor
4. single-owner class-0 MAC executor
5. management MAC executor
6. command-response and TX-confirmation publication
7. RX indication publication

A bounded number of requests and events is serviced per pass, but a pending TX
context must not gate command dispatch.

### Initial concurrency bound

Many contexts may be retained, pending, or PAS-ready. Initially only one
class-0 context may be scheduler-reserved or hardware-owned. This preserves the
validated MAC executor while removing the admission bottleneck.

Management work may prevent publication of the next data frame, but it may not
revoke an already hardware-owned frame.

### Confirmation handling

- Select confirmation owners by explicit hardware completion order and context
  identity, never by submission order or numerical minimum `packet_id`.
- Keep the context and release token until the confirmation bytes have been
  written into the original request buffer and atomic publication can succeed.
- Atomic publication returns that identity exactly once to the ordered input-
  credit FIFO and admits the same buffer to the 64-entry output queue.
- Four hardware descriptors stage the head of that queue; descriptor pressure
  must not directly block command execution or confirmation construction.
- A full 64-entry output queue retains `ConfirmationReady` ownership; it never
  drops or frees it.
- Command responses have admission priority over ordinary TX confirmations.

### Command handling

Synchronous commands execute independently of TX occupancy. Their responses are
encoded into the original request buffer and use the same atomic publication
primitive as TX confirmations. Hardware-descriptor exhaustion is absorbed by
the 64-entry output queue. If that queue is full, retain the command request and
response state in the command-lane owner until admission succeeds.

### Fixed-record flight recorder

Diagnostic builds keep a 256-entry, 24-byte fixed-record ring in firmware BSS.
Each record contains a commit sequence, hardware timestamp, event/flags, two
arguments, and one packed state word. Writers fill the payload first and commit
the sequence last. The ring overwrites old records during normal operation but
freezes on the first firmware-visible invariant failure. It records request
observation/detach/credit, output enqueue, descriptor stage/reclaim, backing
message release, RX FIFO claim/release, and TX lifecycle boundaries. The
hardware lifecycle never depends on the recorder. Recorder metadata and the
newest three complete records are temporarily exposed through the existing
counters MIB; paginated draining can be added separately.

### HIF output publication

Do not put a global FIFO above request dispatch or the TX executor. Vendor
`hif_rx_process()` still dispatches exactly one completed input descriptor per
invocation and reschedules itself when another is ready; this cooperative service
boundary does not serialize the retained request owners. The vendor FIFO is below
execution and contains only complete firmware-to-host messages:

```text
command/TX/RX completion
        -> atomic publication
        -> 64-entry software output FIFO
        -> four hardware descriptors
        -> host consumption and descriptor reclaim
```

For a `0x04xx` response or confirmation, atomic publication performs these
operations as one ownership transition:

1. rewrite the original 1632-byte request buffer;
2. append its identity to the host input-credit FIFO;
3. append the same identity and length to the output FIFO;
4. stage FIFO heads into any free hardware descriptors;
5. issue one write-buffer drain after the grouped credit, queue, and descriptor
   stores, with no visibility barrier between credit return and output admission.

The four 384-byte buffers serve small allocated `0x08xx` indications, but a
receive indication may be much larger. Vendor receive publication writes the
16-byte WSM header into the packet-DMA slot's existing headroom and retains that
slot through HIF publication. Descriptor reclaim immediately calls
`hi_msg_release()`, which dispatches `0x0804` to `rx_buf_release(message + 0x10,
flags & 0x40)`. The RX FIFO keeps independent claim and release cursors; an
out-of-order release is marked `0xffffff00`, and releasing the head walks and
reclaims any consecutive marked successors. On HIF completion, vendor advances
the hardware consumer and stages exactly one descriptor successor *before*
releasing that completed message's backing storage, then repeats for the next
cleared descriptor. It does not bulk-fill every newly free descriptor before
performing releases. There is no timer or
subsequent-completion grace. Descriptor staging reads
`MsgLen` from the queued buffer at staging time and
preserves descriptor bits 13..14 from the pre-sequenced WSM header; the newly
assigned WSM sequence belongs only in the header. No descriptor-reclaim grace
period is permitted. Credit return order is the publication FIFO order, while TX
completion ownership remains identity-based.

### RESET and unjoin

Track a per-VIF epoch for association/key ownership. RESET/unjoin:

1. blocks new TX admission for the affected VIF/epoch;
2. removes and fails retained, pending, PAS-ready, and cancelled reservations;
3. marks scheduled contexts to finish normally;
4. continues publishing every affected TX confirmation;
5. confirms RESET only when no affected arena slot remains.

Do not copy the vendor's defective pool-walk or incomplete drain counters.

## Implementation structure

### Custom target code

The following remain custom because they encode chipset identity and ownership:

- `HostContextArena<[Slot; 30]>`
- fixed free-slot bitmap
- context-address/index conversion
- index-based intrusive pending list corresponding to `ctx+0x04`
- checked lifecycle transitions
- completion lookup by context/packet ID
- HIF identity-return and publication ownership
- volatile hardware boundary
- RESET epoch barrier

### Crate-backed support

Potential target dependencies, only when they simplify code:

- `heapless`: small bounded deferred-response/confirmation queues
- `bitflags`: named hardware flag values
- `zerocopy`: WSM and descriptor byte layouts, never MMIO semantics
- const assertions (or `static_assertions` if needed): context count, strides,
  and wire layout checks

Host-only verification:

- `proptest`: randomized request/command/completion/RESET interleavings
- Kani: exhaustive transition invariants for a reduced arena

Do not use `generational-arena`, `slotmap`, `intrusive-collections`, `bbqueue`,
or a state-machine macro for the central model. Their software identity,
pointer-link, ring, or homogeneous typestate models do not match the hardware.

## Required invariants

1. Every detached HIF request has exactly one owner.
2. Every release token is consumed exactly once.
3. No hardware-owned slot becomes free or reversible.
4. Initially at most one class-0 slot is reserved/scheduled.
5. A context appears in at most one pending/PAS/reservation structure.
6. Every TX confirmation matches exactly one context/packet ID.
7. Commands progress independently when all TX contexts are occupied.
8. Hardware-descriptor saturation is absorbed by the software output FIFO.
9. Output-FIFO saturation cannot lose a command response or TX confirmation.
10. Every confirm-class publication appends one input credit and one output
    entry for the same request-buffer identity.
11. Input credits and output entries preserve one atomic publication order.
12. RESET confirms only after all affected contexts drain.
13. Rate-policy lookup remains index-based and observes the vendor-compatible
    mutable global policy table.
14. Each cooperative pass has bounded work and cannot starve MAC events or RX.
15. Management and class-0 never consume the shared MAC backend concurrently.

## Staged implementation

1. Add a pure no-I/O arena/transition model with exhaustive focused tests.
2. Replace the single `HostTxDriver` slot with 30 independently owned entries.
3. Admit each TX directly into the vendor pending list.
4. Service pending/PAS work across entries with a bounded round-robin cursor.
5. Permit only one scheduler reservation/hardware owner.
6. Keep command dispatch active regardless of arena occupancy.
7. Implement atomic original-buffer publication and the 64-entry output FIFO.
8. Publish command responses before TX confirmations; retain both when the
   software output FIFO is full.
9. Implement multi-entry RESET cancellation and drain.
10. Validate WPA/DHCP/ping/HTTP and exact ownership drain.
11. Re-run sustained 1,400-byte ICMP and the 1 MiB TCP upload.
12. Only after this is stable, consider multiple hardware-owned frames and
    per-pipe credits.
