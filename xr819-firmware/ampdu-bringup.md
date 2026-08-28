# TX A-MPDU bring-up

## Qualified prerequisites

The ordinary host-TX scheduler can publish two non-aggregate frames through one
trigger/GO transaction. A diagnostic build measured 859 depth-two batches in a
10-second TCP run, but throughput remained near 2 Mbit/s. Multi-slot publication
alone is therefore not the vendor performance mechanism.

The aggregation grouping gate now uses typed PAS fields and requires:

- QoS data frames;
- the same interface, internal link, TID, and rate;
- an HT rate;
- an enabled host BlockAck policy; and
- a TX BlockAck session reported operational by the host.

A diagnostic run measured 777 eligible depth-two pairs in 10 seconds at TID 0
and rate index 19. Candidate supply is not the limiting factor.

The vendor `txp_desc_emit` opcode stream and its eight-by-eight negotiated
spacing/rate table are translated by the depth-two descriptor builder. Aggregate
length includes delimiter alignment, spacing bytes, the second MPDU, and FCS
allowance.

## BlockAck negotiation

The upstream-style cw1200 path assumed firmware-owned negotiation. It set
`TX_AMPDU_SETUP_IN_HW`, rejected every `ampdu_action`, and dropped all BlockAck
category action frames in `cw1200_tx_h_action()`.

The open-firmware path instead delegates negotiation to mac80211:

1. BlockAck action frames pass through normal WSM TX.
2. `IEEE80211_AMPDU_TX_START` returns immediate acceptance.
3. Operational and stopped TIDs are reported through private MIB `0xff48`.
4. Firmware does not admit an aggregate before the operational notification.

Monitor capture confirmed successful ADDBA exchange and the
`IEEE80211_AMPDU_TX_OPERATIONAL` callback. With aggregation still disabled, the
qualified WPA2 HT image sustained 3.95 Mbit/s TCP, 20/20 ping, an idle BH/WSM,
and zero used buffers. This is a useful prerequisite independently of the
kind-1 descriptor work.

## Rejected kind-1 experiment

A feature-gated depth-two experiment folded two reversible scheduler
reservations into one kind-1 pipe slot, used the typed software-record free
list, linked PAS `dwNextInAmpdu`, and emitted one command stream.

Every tested version stalled at the first aggregate after roughly 47-71 KiB of
ordinary traffic. Symptoms were consistent:

- no aggregate appeared in monitor capture;
- 10-14 host buffers remained owned;
- the BH stayed alive initially, then reported missed interrupts / TX-confirm
  timeout;
- no hardware `AGG TXed` or `MULTI TXed` accounting appeared.

The following corrections did not make the descriptor start:

- replacing ordinary batch bits 26/27 with aggregate member/head flags
  `0x20`/`0x40`;
- including negotiated spacing bytes in aggregate length;
- preparing each member's `dwParentQ` descriptor and invoking it from word 2;
- reproducing the vendor special-ACK branch (`slot+0xd = 0x0c`, `slot+0xe = 1`,
  and the additional duration/flags).

The failed hardware-publication code was removed rather than committed. Static
reconstruction later identified the decisive omission: vendor mode 1 resets its
cursor after building the auxiliary MPDU stream and emits shared PHY opcodes
`0x51`, `0x50`, and `0x52` at top-level command offsets `+0x0c..+0x14`, before
the opcode-0 transfer at `+0x18`. The rejected builder had incorrectly appended
those words to the auxiliary stream, leaving the MAC command entry zeroed.

## Qualified depth-two transmission

The corrected feature-gated path now keeps the two streams distinct:

- the top-level pipe command contains duration words, the three shared PHY
  words, and the transfer to the software record;
- the software record contains the two MPDU transfers, delimiter, optional
  negotiated-spacing trampoline, and terminal opcode;
- a kind-1 retry event retains slot state 4 while the joined RX lane waits for a
  matching compressed BlockAck;
- the BlockAck starting sequence and 64-bit bitmap retire both members only
  after both sequence bits are present;
- repeated growing bitmaps remain low-MAC-owned, while the existing one-second
  pipe watchdog provides a bounded give-up path if a member never appears; and
- successful members carry `WSM_TX_STATUS_AGGREGATION` in their host confirms.

Monitor capture showed on-air QoS A-MPDU traffic followed by compressed BlockAck
frames addressed to the XR819 interface, including repeated growing bitmaps.
The packed image
`29a6fa103de333e319e210f55615ddac7537a5ef16e31efb42048a12176d5b14`
then completed:

- a 10-second TCP run at 5.00 Mbit/s, with 4,583 TX confirms, 4,436 aggregate
  confirms, 20/20 ping, an alive BH, idle WSM, and zero used buffers; and
- a 60-second TCP soak at 3.39 Mbit/s, with 17,844 TX confirms, 17,206 aggregate
  confirms, 20/20 ping, an alive BH, idle WSM, and zero used buffers.

This closes depth-two on-air publication and successful BlockAck retirement, but
not the vendor performance gap. The code remains behind
`experimental-depth-two-ampdu` while per-member partial-BA retry and larger
aggregate depths are still untranslated.

## Partial-BlockAck retry investigation

The first per-member retry attempt exposed an important limitation in the
qualified path: matching aggregate terminal status `0x0c` currently reaches
`complete_tx_pipe_slot()` and confirms both members before software interprets
the BlockAck bitmap. Deferring that status and waiting for the ordinary joined
RX lane stalled the first aggregate, because that lane never observed subtype
`0x94`.

Static disassembly confirms vendor `rxfifo_find_frame_by_subtype(0x94)` scans
packet RAM at `0x09400000` from the independent cursor at DTCM `0x040016c0`
toward MAC producer register `0x09c00604`. Approximate scans from either the
ordinary host-RX claim cursor or a separately captured TX-start producer did not
find the BlockAck: diagnostic runs remained at match stage zero and wedged with
7-11 host buffers owned. The captured TX-start cursor addressed a one-byte
sentinel whose apparent frame control was `0x0080`, so walking it as an ordinary
RX slot was not a faithful translation of vendor cursor/slot-validity semantics.

Those experiments were removed. The four cursor helpers are now translated as
read-only typed operations, but a diagnostic invocation at matching status
`0x0c` observed cursor equal to producer and therefore no immediately queued BA
frame. Deferring the aggregate and rescanning later also found no subtype
`0x94`, so cursor arithmetic alone is not the missing completion contract.

Further static reconstruction found two adjacent omissions:

- Ghidra dropped the fourth argument to `txp_build_ba_desc()`. Vendor
  `txp_build_resp_descs(3, 0)` passes selector `0x1c`, producing descriptor
  words `0x69000004`, `0x68000004`, and `0x6000001c`; the Rust translation had
  emitted zero selectors.
- Vendor TX BlockAck link records retain peer MAC, interface, TID, starting
  sequence, current sequence, and the active-link bitmap before state 6. The
  depth-two publisher now initializes those typed fields from the live VIF and
  PAS grouping key.

The corrected metadata remains compatible with the qualified fallback path:
a 10-second TCP run completed at 3.22 Mbit/s with 20/20 ping, an alive BH, idle
WSM, and zero used buffers.

Follow-up visibility probes closed the remaining timing ambiguity:

- 1,006 matching aggregate completions searched the vendor cursor immediately;
  none found subtype `0x94`;
- a bounded 256-pass delayed scan also found none;
- the ordinary joined-RX path observed zero matching BA frames while monitor
  capture continued to show the AP transmitting compressed BlockAck frames;
- the per-link hardware bitmap words remained zero at terminal status.

Therefore state 11 cannot yet be translated as a software timing/retry loop:
the XR819 is not publishing the received BA control frame or bitmap to the
firmware-visible structures used by vendor `bab_process_ba_bitmap()`.

A follow-up response-pipe audit rejected another false lead. Vendor literal
`DAT_00010928` is `0x09007000`, the packet-RAM response-pointer table, not the
MAC register block at `0x09c00a00`. `txp_program_pipe_slot()` already publishes
slots 2, 3, 11, and 12 there. Writing the same values to MAC registers broke
association traffic, while replacing the open retained aliases with only the
packet-RAM writes also regressed operation. Likewise, forcing descriptor mode
1 (`0x94`) stalled traffic. These descriptors construct immediate transmitted
responses; they do not establish received-BA visibility.

The vendor raw words around `txp_build_ba_desc()` also cannot be copied without
relocation: its `0x07002000` operand names vendor packet-RAM state, whereas the
open image's corresponding object is at a different packet-RAM offset. Tests
that substituted the raw word or zeroed the currently qualified selector words
stalled traffic. All such experiments were removed and packed image
`ba6ff60e...` restored.

The next slice must instead reconstruct ownership of the normal RX FIFO around
`rxfifo_next_frame()`/`rxfifo_release_slot()`, particularly why vendor cursor
`+0x40` remains on a live slot until `rxfifo_find_frame_by_subtype(0x94)` while
the open consumer observes only a released sentinel. The shared DTCM words are
now represented by `RxFifoStateAddress` rather than unrelated low-MAC producer
aliases: release cursor `+0x10`, claim cursor `+0x14`, deferred consumer `+0x18`,
and BA scan cursor `+0x40`. This is a typed view over the existing fixed-layout
record, not a relocation or claim over its adjacent MAC timing fields.

A hardware experiment then initialized the BA scan cursor with the live producer
at RX synchronization and advanced it whenever release would otherwise leave it
on the cleared head slot. The path remained healthy (3.16 Mbit/s, 10/10 ping,
zero used buffers), but 746 matching aggregate completions still found zero
subtype-`0x94` frames. The speculative cursor coupling was removed. This rules
out a merely stale `+0x40` value: the BA frame is absent from the firmware-visible
normal RX stream before cursor policy can recover it. Keep the qualified
all-members-success fallback until the earlier hardware admission/filter stage
is identified.

The first admission-register comparison also rejected the apparent packet-DMA
control mismatch. Both images initialize `0x09c00600` to `0x01020418`, and
previous same-state no-HT captures show both open and vendor firmware at
`0x010e0419`. Vendor reaches `0x013e0419` only during active aggregate/retry
operation, so bits `0x00300000` are runtime activity state rather than a static
control-frame admission policy. Forcing them would copy status, not configure
routing.

The aggregate publication-side controls are likewise present: vendor sets the
per-link state to 6 immediately before its kind-1 descriptor build; the open
publisher does the same. Vendor's special-ACK duration descriptor passes mode
1 and ORs `0x80 + 0x0c` into command word `+4`; the open builder emits the same
`0x8c` flag and expected terminal status `0x0c`. The remaining missing contract
is therefore later than aggregate admission but earlier than normal packet-RAM
visibility. A new target capture is required to compare the live RX slot stream
at that boundary; the target became unreachable before the diagnostic image
could be installed.

The vendor aggregate-member ownership table is now translated independently of
BA visibility. `MacAggregateSlotTablesAddress` derives the fixed eight-by-16
slot bank at `0x04001b10..0x04001d10` without introducing an in-range linked
literal. Depth-two publication writes the two PAS/frame-node identities to
slots 0 and 1 and clears slots 2 through 15; the matching qualified fallback
completion clears all 16 words. The drift gate permits only the typed accessor,
publisher, and matching completion consumer.

Hardware qualification of packed image `18e616a194f9e5dccdfd3cbe4d1c9b9d7f013344d5809bc425dd73b397a722f9`
completed at 5.10 Mbit/s TCP with 4,602 aggregate confirmations, 20/20 ping,
an alive BH, idle WSM, and zero used buffers. This does not prove partial-BA
retirement, but it establishes the exact vendor member-tracking ownership that
`bab_process_ba_bitmap()` will consume once the BA bitmap becomes visible.

The depth-two subset of the BA window decision is now pure and explicit:
`classify_depth_two_block_ack()` masks 12-bit sequence numbers, handles wrap at
4095, distinguishes acknowledged and missing members inside the 64-bit window,
and separately marks members outside that window. The existing qualified
all-ack consumer now uses this classifier without changing its ownership rule;
missing or outside-window members remain owned for the bounded watchdog rather
than being falsely confirmed. Packed image
`18dcc3bb45ce4776f2b394a865a30cc56ca8c7c8e570dfee80123eb6c5c2caaa`
completed a 5-second TCP run at 3.14 Mbit/s with 1,386 aggregate confirmations,
10/10 ping, and zero used buffers after drain.

A second pure layer, `plan_depth_two_block_ack_actions()`, now maps those member
states to confirm, retry, or give-up actions using explicit per-member retry
eligibility and the operational-session state. Acknowledged members always
confirm; missing and outside-window members retry only while both policy and
the BA session permit it. This remains unconnected to runtime ownership until
a real bitmap is visible, but fixes the decision contract needed by the next
per-member requeue slice.

A read-only kind-1 completion probe then closed the remaining TX-record
possibilities. At matching terminal status the packed slot header was
`0x01030c0c` (kind 1, slot state 3, expected `0x0c`, delivered `0x0c`). The
hardware ring retained its ordinary descriptor values (`+0x0c = 0x7080`,
`+0x10 = 0x54`), completion word zero, and cursor/pending word `0x000f0f0f`;
none encoded a per-member result. The auxiliary packet stream still contained
the two MPDU transfers, negotiated-spacing transfer, and terminal
`0xe4000000`. Words after that terminal were unowned scratch data, not a result
record. The apparent extra transfer seen in the first probe was the legitimate
spacing trampoline, not a hardware-written completion pointer.

All completion probes were removed and the normal feature image restored. The
bitmap is therefore absent from both normal RX ownership and every translated
kind-1 TX completion record. The next evidence source must be an untyped MAC
sideband/event path or a receive-control capture mode that vendor enables
outside these records.

The MAC sideband path is not that source. Matching kind-1 status events are type
`0x39`; direct reads of `0x0ab80c50` across 713 status-`0x0c` completions varied
only within the low ten bits (last `0x290`, accumulated OR `0x2ff`). Vendor uses
the same capture as measurement/trace metadata through DTCM `+0x1d14`; it
cannot contain a 64-bit BA window or a pointer to one.

A bounded forced-loss experiment then made the second aggregate transfer invoke
the first member descriptor again, deliberately withholding the second member's
sequence from the receiver. Traffic collapsed as expected (66.8 Kbit/s, no
ping replies, 13 buffers retained), proving the current all-success fallback is
unsafe under loss. Nevertheless, 15 matching aggregate completions exposed no
subtype-`0x94` frame either immediately or through joined RX, and the sideband
OR remained only `0x2bf`. The experiment was removed immediately and packed
image `1958067d4e12c44c73bccc8c3b0409d42f2865e673cf258ad7cc3cfac2704e3a`
restored.

This leaves no observed per-member hardware result source. The implemented
state-11 fallback now treats a kind-1 retry event as a missing result for both
members. It preserves both sequence numbers, advances both ordinary rate-policy
try counters together, requires an active TX BA session and a common next rate,
and allows at most two whole-aggregate retries. Rearm follows the vendor retry
ownership sequence: rebuild the command in place, retain slot state 4, trigger
the pipe, clear only the current hardware command-mask bit, then acknowledge the
owned retry event. Re-running GO was rejected because it stalled after eight
aggregates and left active ownership behind.

A forced-loss qualification repeated the first descriptor in place of the
second. The bounded command-mask rearm sustained 304 Kbit/s and 3/3 ping under
that permanent second-member loss, then drained to zero used buffers with the
BH alive and WSM idle. The normal image
`ea856dfc52287542f5baf02747b09bc6531a88b78db1b364cf999b292d16cbd9`
then reached 5.45 Mbit/s over ten seconds, 4,882 aggregate confirmations, 20/20
ping, and zero used buffers.

The stopped-session edge initially exposed a distinct ownership bug: giving up
before any rearm left the aggregate's current hardware command-mask bit set, so
the host eventually killed the BH with 14 TX buffers stuck. Aggregate give-up
now releases that bit before member completion and clears global busy ownership.
With the BA session forced inactive at the first retry event and the second
member permanently withheld, the corrected path sustained 520 Kbit/s and 3/3
ping, then drained to zero used buffers with the BH alive and WSM idle.

The restored normal image
`dff39906ac610da24205ce9167fc1ab50827f68ffd36497adee3f9eecf49c14d`
reached 4.55 Mbit/s in a five-second smoke test and 5.21 Mbit/s over 60 seconds.
The long run completed 26,442 aggregate confirmations and 20/20 follow-up pings
with the BH alive, WSM idle, and zero used buffers.

The generic pipe watchdog already covered event silence, but its expiry path
called `complete_tx_pipe_slot()` directly and therefore bypassed the aggregate
command-mask and busy-owner release above. Watchdog retirement now invokes the
same kind-1 command-mask release before completing both members. The first
version released every kind-1 watchdog slot, including normally started slots,
and reproducibly reduced a 60-second TCP run from 5.21 to 3.44 Mbit/s. The
release gate now requires exact retry ownership: kind 1 and slot state 4.

A temporary host debug control sent the real private MIB `0xff48` while TCP was
active. Clearing TID 0 stopped new aggregation immediately: the aggregate count
moved only from 1,246 to 1,248 while ordinary TX advanced by 1,029 frames over
five seconds. Re-enabling the same MIB resumed aggregation without reassociation.
The same stop/restart was repeated with the second member permanently withheld;
aggregation stopped at 44, ordinary TX advanced by 1,171 frames, aggregation
resumed after restart, 10/10 ping completed, and all buffers drained. The host
control and forced descriptor were removed after qualification.

The aggregation-adjacent raw accesses in `mark_ba_session_state_5()` are now
bounded by typed views. Context `+0x54` uses the existing frame-address/PAS
accessor, while `BaPipeObjectAddress` validates the returned DTCM identity and
owns only its translated state byte at `+0x06`; the rest of that object remains
opaque. The production backend still returns no object, so this is confinement
rather than new runtime policy. Packed image
`cf8fe039bfde580d9630ac94cb405effdfc2f9c01a088cea5c7f8ff3b69aebf8`
reached 5.12 Mbit/s over ten seconds with 4,508 aggregate confirmations, 20/20
ping, and zero used buffers.

A 60-second WPA2 UDP soak offered 5.24 Mbit/s and delivered 5.15 Mbit/s. The
receiver reported 497 lost of 26,752 datagrams (1.86%). Follow-up rate-controlled
runs showed that this was saturation rather than silent aggregate corruption:
the link had fallen to MCS 0 and sustained about 3.94 Mbit/s, so a 4.19 Mbit/s
offer lost about 6%; at 3.15 Mbit/s it delivered 3.13 Mbit/s with 61 of 16,052
datagrams lost (0.38%).

Those lower-rate tests did expose a real teardown defect. Under sustained MCS-0
UDP, exactly five host buffers remained owned after traffic stopped. Diagnostics
showed five contexts in `PasQueued`, all MAC pipes idle, no retry/receive gate,
and a healthy PAS ring containing those contexts. Their admission timestamps
had exceeded the scheduler lifetime while waiting behind the saturated pipe.
`service_index()` already completed this `Expired` result, but
`publish_ready_batch()` restored it to `PasQueued` forever. The batch publisher
now rejects the expired PAS and emits the same truthful failure confirmation.
A 30-second forced-MCS-0 regression run then drained immediately and after ten
idle seconds, with 20/20 ping, BH alive, WSM idle, and zero used buffers.

The WPA3-SAE/PMF AP was not visible during this qualification attempt; the
client remained in `SCANNING`, so security-path aggregation still requires a
later run when that BSSID is available.

In a forced silence test, the second member was withheld and the retry path
acknowledged its event without re-triggering hardware. Watchdog expiry recovered
at 411 Kbit/s, 5/5 ping, BH alive, WSM idle, and zero used buffers. The corrected
normal image restored 5.16 Mbit/s over 60 seconds with 26,228 aggregate
confirmations, 20/20 ping, and zero used buffers.

Warm SDIO unbind/rebind remains a separate pre-existing failure: firmware
download completes, but startup times out with HIF `0x0ab00100 = 0x00013f4c`,
`0x0ab00104 = 0x000000a9`, and `0x0ab00000 = 0x0000800c`. A cold reboot restores
normal operation. The retry/session-stop changes neither fix nor worsen that
post-download startup contract.

The missing bitmap source is now identified. A live vendor/open register
comparison found that the joined open path left MAC receive-mode register
`0x09c00204` at the synthetic station value `0x00198000`, while the vendor
joined runtime selected the generic active-VIF value `0x0279fe00`. The open
mode word at `0x09c00200` was already the generic joined value, so the two
registers described inconsistent modes. Publishing the generic selector
`0x00180783` and filter `0x0279fe00` made the received compressed BlockAck
visible in the normal packet-DMA FIFO without changing packet-DMA status bits.

A raw boundary probe captured cursor `0x32e8`, producer `0x3334`, frame control
`0x0094`, BA control/start word `0x12e00004`, and bitmap
`0xffffffffffffffff`. This is the exact vendor source consumed by
`rxfifo_find_frame_by_subtype(0x94)` and `bab_process_ba_bitmap()`: normal RX
packet RAM, previously excluded by the wrong joined receive mode.

Kind-1 retry handling now validates receiver address and TID, reads the 12-bit
starting sequence plus 64-bit compressed bitmap, and feeds the existing
classifier/action planner. An all-ack retry event completes normally. When one
member is acknowledged and the other is retryable, the acknowledged context is
completed once, the missing context advances only its own retry/rate-policy
state, and the kind-1 slot is converted in place to an ordinary single-frame
retry. Aggregate descriptor and member-table ownership are released during that
conversion; the remaining context keeps the existing slot/command owner until
its ordinary completion.

A forced permanent second-member omission exercised this split: the aggregate
sent the first descriptor twice, the receiver acknowledged only the first
sequence, and the missing second member was republished as an ordinary frame.
The run sustained 350 Kbit/s, completed 20/20 ping, and drained to zero used
buffers; 300 aggregate attempts were followed by ordinary TX rather than whole-
aggregate rearm. A normal 60-second MCS-1 TCP run then reached 4.99 Mbit/s with
25,296 aggregate confirmations, 20/20 ping, BH alive, WSM idle, and zero used
buffers.

The reverse diagnostic that substituted the second descriptor for the first
stalled after eight aggregates and is not accepted as a valid first-member-loss
injection: it changes the first transfer's descriptor identity as well as its
sequence and does not preserve the qualified aggregate shape. That image was
removed.

A safer synthetic first-member-action test retained the qualified duplicate-
first loss injection, which produces a real retry event, but reversed only the
software-visible action bitmap: the first member was classified missing and the
second acknowledged. This exercised the opposite ownership split without
changing the first hardware descriptor. It sustained 342 Kbit/s, completed
20/20 ping, converted 299 aggregates into ordinary retries, and drained to zero
used buffers. Clearing the first bit during normal all-ack traffic alone did not
exercise this path because the bitmap classifier is intentionally entered only
for a hardware retry event.

Stopped-session partial termination initially exposed a second shared-executor
race. Direct dual completion, reentrant completion draining, and an early
kind-0 terminal split all retained host buffers or stalled the BH. Kernel
tracing showed no stale or unknown confirmation IDs, while packet-ID lifecycle
accounting showed submissions stopping before MAC completion.

An atomic stalled-state snapshot identified the actual gate: all 13 retained
class-0 contexts were `PasQueued`, none owned hardware, the HIF output queue was
fully reclaimed, and the host-input ring still had 16 live credits. Management
publication was nevertheless considered available while class-0 contexts were
still owned. Serializing the shared management executor until every
`HostTxDriver` state is empty removes that interleaving; allowing management
while only confirmations remained reduced the regression but still retained one
buffer, so the all-empty gate is intentional.

The descriptor-preserving stopped-session injector converts an otherwise
successful kind-1 status into retry ownership, clears only the first member's
software-visible BA bit, and marks the BA session stopped. Both original
descriptors and the on-air aggregate remain unchanged. With the corrected gate,
a 20-second forced run reached 4.93 Mbit/s, reported 491 failed members across
8,027 aggregate transmissions, completed 20/20 ping, and drained to zero used
buffers. This qualifies the `Confirm + GiveUp` ownership split: the acknowledged
member completes successfully and the missing member reuses ordinary kind-0
terminal completion.

The production image then completed two consecutive 60-second MCS1 TCP soaks at
5.26 and 5.31 Mbit/s. Both completed 20/20 ping, kept the BH alive and WSM idle,
and drained to zero used buffers.

Selective exhaustion used the same descriptor-preserving retry injection while
setting only the missing member's retry count beyond its active policy. A
20-second run reached 5.18 Mbit/s, reported 513 failed members across 8,501
aggregates, completed 20/20 ping, and drained to zero. This qualifies exactly-
once give-up after `prepare_selective_member_retry()` reaches policy exhaustion.

Outside-window classification initially reused the active-session retry action.
Forcing one member outside the 64-frame BA window proved that unsafe: throughput
collapsed to 122 Kbit/s, ping loss reached 100%, and four buffers remained when
the BH failed. An outside-window result cannot establish a retryable missing
member, so it now always maps to `GiveUp`. The corrected forced run reached 5.07
Mbit/s, reported 571 failed members across 8,149 aggregates, completed 20/20
ping, and drained to zero. The final production image then completed a
60-second MCS1 TCP soak at 5.35 Mbit/s with 20/20 ping and zero used buffers.
Natural first-member loss still requires independent qualification before depth
two can leave its experimental feature gate.
