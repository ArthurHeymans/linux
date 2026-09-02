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

This closed the initial depth-two on-air publication and successful BlockAck
retirement slice. Later work below translated and qualified per-member partial-
BA retry in both directions. The code remains behind
`experimental-depth-two-ampdu` pending natural first-member-loss qualification
and larger aggregate depths.

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

WPA3-SAE/PMF+HT qualification found the SAE BSSID
`a2:41:74:2c:f3:9f` for the same SSID. Association completed with SAE, required
PMF, CCMP PTK/GTK, and BIP software fallback. The first HT data attempt exposed
that protected unicast management still used the legacy class-6 path without a
qualified management-CCMP implementation: the first protected ADDBA request
remained pending and blocked 13 data buffers. XR819 CCMP keys now request
mac80211's `SW_MGMT_TX` and `RX_MGMT` paths, leaving ordinary data CCMP on the
qualified firmware/hardware engine while protected management uses mac80211's
standard CCMP implementation. This allowed ping traffic to drain normally and
made depth-two aggregation operational.

The security path is improved but not yet fully qualified under sustained load.
High-rate traffic stops after roughly five to ten seconds with one management
packet and 13 data packets pending. A bounded diagnostic captured the management
packet as FC `0x40d0`, length 49, request flags `0x0a`, HT parameters `1`, and
plaintext prefix `03 01 01 00`: a BlockAck DELBA action. Direction counters
remained zero, so no selective first-member, second-member, or other
non-unanimous BA action precedes the stop. An atomic ownership snapshot then
found 12 software-owned `PasQueued` contexts, one class-0 hardware owner, and an
active management runtime at the same instant. The loop admitted management
requests without consulting the all-empty class-0 gate, but serviced an already
published management owner only while that same gate was true. This created a
closed cycle: management blocked class-0 publication while retained class-0
state blocked management completion.

New management requests now fail cleanly while any class-0 owner remains, and
an already active management owner is serviced regardless of later queued
class-0 software state. A 60-second WPA3 UDP run no longer stopped: it sent 34.6
MiB at 4.83 Mbit/s, completed the following 20/20 ping, kept the BH alive, and
continued through 18,716 TX confirmations. A subsequent 30-second TCP run held
2.69 Mbit/s. The 5 Mbit/s offered-load run left one ordinary data buffer while
receiver loss was high, so it is not the drain qualification point. Two
separate 30-second 3 Mbit/s UDP bursts each delivered 3.15 Mbit/s with zero
reported datagram loss and zero used buffers. Aggregation reached 388 frames in
the first burst; after the idle boundary mac80211 restarted the BA session and
the second burst raised the count to 678. This qualifies WPA3 BA teardown,
zero-buffer drain, and live restart at sustainable offered load. All temporary
ownership snapshots, direction counters, firmware-side software-CCMP, watchdog,
and unprotected-frame code were removed.

The final shared-executor image also passed WPA2 regression qualification.
A 60-second MCS1 TCP run reached 5.47 Mbit/s and a 60-second UDP run delivered
5.24 Mbit/s offered and 5.21 Mbit/s received, followed in both cases by 20/20
ping, BH alive, WSM idle, and zero used buffers. Forcing a legacy-only rate held
the aggregate count effectively flat (four already-in-flight completions across
a 20-second burst); restoring MCS1 and starting a new 30-second burst increased
the count by 290. The restart burst delivered 3.15 Mbit/s with zero reported
datagram loss and zero buffers. Clean image
`6963474b44dbe0165fafc4b881a99eb25e590bce1c4ad8304288fa37a46b9a6c`
is deployed.

Natural first-member loss still requires independent qualification before depth
two can leave its experimental feature gate. A direction-counter image explored
higher fixed rates without altering descriptors or BA parsing. MCS3 completed a
60-second run at 6.77 Mbit/s with 34,618 aggregates and 22 failed packets; MCS4
completed 60 seconds at 4.02 Mbit/s with 19,784 aggregates and 163 failed
packets; MCS5 completed 30 seconds at 1.63 Mbit/s with 3,368 aggregates. None
produced a usable per-member BA action in either direction: all four selective
direction counters remained zero. MCS7 failed before aggregation became
operational and is not a valid loss regime. A subsequent 600-second MCS4 TCP
soak broadened the natural search to 241,594 aggregates. It averaged 4.97
Mbit/s, completed 20/20 ping, kept the BH alive, and drained to zero buffers,
but first-member retry, second-member retry, and other non-unanimous BA-plan
counters all remained zero. The diagnostic image was removed and production
image `00887f499e35976d441cc164db75520b8b9828ba7a4315f0f20eaed41fd1f39f`
restored. Natural first-member qualification therefore still needs controlled
RF attenuation or interference rather than another ordinary fixed-rate soak.

A follow-up used the existing output-power MIB at channel gain programming as a
controlled RF attenuation mechanism, without changing descriptor identity,
aggregate shape, sequence numbers, or member order. At 0 dBm, a 60-second MCS1
run produced only 80 aggregates and no non-unanimous BA action. At 10 dBm, a
120-second MCS4 run produced 2,546 aggregates and no action. At 15 dBm, a
180-second MCS6 run produced 3,768 aggregates and again no action. Fixed-rate
MCS5--7 runs at the ordinary power setting also crossed regimes ranging from
near-clean delivery to more than 85% UDP loss without producing a selective
member action.

To bias physical loss toward member one, a valid traffic-shaping run alternated
a 1,470-byte first datagram with a 64-byte second datagram while preserving both
original packet descriptors. At 15 dBm and MCS7 it added roughly 68,000
aggregates; at 10 dBm and MCS7 it produced 10,262 aggregates. Both direction
counters remained zero. The severe 10 dBm/MCS7 endpoint eventually left nine
queue-2 buffers locked after traffic stopped, with BH alive and WSM idle; a
quiesced reload recovered it. The diagnostic attenuation and counters were
removed and clean image
`6963474b44dbe0165fafc4b881a99eb25e590bce1c4ad8304288fa37a46b9a6c`
was restored. Software-controlled power and frame-length bias therefore do not
close the first-member gate; the next valid attempt needs an external RF
attenuator or independent in-channel interferer.

The descriptor ISA audit did not find a safe software first-member-loss control.
Vendor `txp_desc_emit()` case 0 emits exactly `0x65000000 | (address &
0x001ffffc)` with no spare per-transfer flag bits; case 1 emits the fixed
`0x66000000` inter-member delimiter command and case 2 emits the fixed
`0xe4000000` terminal command. Setting an ignored low delimiter bit
(`0x66000001`) preserved ordinary operation. Changing the delimiter opcode to
`0x67000000` produced only second-member loss (52 selective second retries and
eight second terminals in five seconds), confirming that it controls the
boundary before member two. Changing only the first transfer opcode to
`0x64000000` prevented aggregation from becoming operational and left eight
buffers when the BH failed. These images were removed. The available command
stream can invalidate member two or the whole aggregate, but cannot invalidate
member one while preserving both transfer operands, member order, and the
qualified aggregate shape; first-member proof now requires controlled RF loss
or external attenuation/interference.

After closing the retained-address safety audit, retry accounting and publication
cleanup were split from one global backend value into four pipe-local states.
This does not yet permit simultaneous hardware owners: the host scheduler still
publishes only when no class-0 runtime owner exists. It removes the first
software-state collision that would otherwise reset another pipe's retry count
or publication identities when multi-pipe outstanding work is enabled.

The host driver now also discovers retained class-0 owners by pipe, coalescing
multiple contexts that legitimately share one staged batch or aggregate owner.
Each occupied pipe receives its own bounded service step after the shared MAC
completion pass. Missing scheduled-owner metadata and out-of-range pipes halt
before publication or completion routing. Publication remains globally
serialized, so this is an ownership-model change rather than a concurrency
switch.

Multi-pipe publication is now enabled one bounded step beyond that prerequisite.
The scheduler scans ready PAS contexts for a queue-mapped pipe absent from the
retained owner set, skips contexts whose pipe is still active, and admits at
most one new batch per service pass. The reservation boundary still rechecks
live pipe idleness and all existing slot, frame, command, and hardware-ring
identities before publication. Same-pipe batches and depth-two aggregates remain
coalesced under one pipe owner; no pipe may acquire a second retained owner.
The qualified image sustained three simultaneous one-Mbit UDP streams split
across Linux TX queues 0 and 2, followed by 20/20 ping with the BH alive, WSM
idle, and no used buffers. Cold and ordinary warm depth-two traffic remained
healthy as well.

The copied class-0 completion queue is sized for the resulting worst case: two
completed contexts on each of four simultaneously owned pipes before the host
driver drains the backend. The bounded eight-entry queue retains exact copied
context, frame-node, pipe, slot, status, and retry identities and still halts on
an impossible ninth completion rather than dropping ownership evidence.

Repeated compressed BlockAck handling now retains the classified member states
per pipe and exact aggregate frame-node pair instead of rescanning released RX
FIFO storage later. Acknowledgements are sticky across repeated or shifted
bitmaps, so first-only followed by second-only evidence completes both members;
total misses still enter whole-aggregate retry, and stale observations whose
member identities no longer match are rejected before selective retry planning.

Retry terminal completion now carries the executor's already-latched pipe into
success and give-up handling instead of rereading the shared
`MAC_CURRENT_PIPE` byte after policy and descriptor work. This keeps command-mask
release, partial member conversion, and copied completion identities tied to the
same pipe even while another pipe is active.

Publication registration now independently enforces that ownership boundary.
Single and first descriptors require an empty per-pipe identity slice, the last
ordinary batch member requires exactly one staged identity, and depth-two
registration rejects any existing identity. An accidental second owner can no
longer erase the retained context, frame-node, or command identity needed to
route the first owner's completion.

Retry attempt accounting is now indexed by the exact `(pipe, slot)` identity
rather than by pipe alone. Registration resets only the published slot, retry
decisions reject interior or cross-pipe retained slot addresses, and completion
reporting reads attempts from the publication's exact slot. Same-pipe
concurrency remains disabled, but a later second outstanding slot can no longer
share or reset the first slot's retry budget.

Selective-retry member ownership, partial give-up pairs, and retained BlockAck
observations now use the same exact pipe-slot index. Every ordinary batch member
resets its own slot state, including the last staged member, while a depth-two
pair shares only its published aggregate slot. Same-pipe outstanding work can no
longer consume another slot's accumulated bitmap or deferred member action.

The host driver's retained-owner inventory now records every exact `(pipe,
slot)` while preserving the qualified one-service-step-per-pipe schedule.
Depth-two contexts sharing one aggregate slot are coalesced, ordinary staged
slots remain distinct, and completion routing rejects a pipe-slot with no live
owner before searching exact context and frame-node identities. Publication is
still limited to one active batch per pipe.

Hardware-owner servicing now walks the exact slot mask with a persistent
round-robin cursor and the existing four-context pass budget. Distinct ordinary
batch slots each receive a bounded service step, aggregate members sharing one
slot remain coalesced, and owners beyond the current pass resume fairly on the
next pass. The eight-entry copied-completion bound remains exact while only one
two-context batch may be active on each of four pipes.

## Vendor scheduler audit: one transaction per idle pipe

The vendor scheduler does **not** append a second batch to an already armed
pipe. `txp_scheduler_run()` (`0x0000aa5e`) builds its eligible mask only from
pipe records whose `+0xa3` armed byte is zero. Active pipes are excluded before
`txq_build_aggregate_lists()` (`0x0000a2c0`) sees queued PAS contexts.

Within that single scheduler transaction, however, the vendor fills the pipe
more deeply than the current Rust subset:

- ordinary mode in `txq_try_append_to_aggregate()` (`0x0000a078`) admits up to
  four contexts for one pipe;
- aggregate mode chains same-link, same-rate PAS records up to the literal
  sixteen-member cap, additionally bounded by TXOP/airtime policy;
- `txp_build_pipe_descriptor()` (`0x0000a712`) records the old producer as the
  last slot and advances the local producer modulo four for every ordinary
  descriptor, while one aggregate chain occupies one kind-1 slot;
- only after every descriptor is built does `txp_scheduler_run()` clear the
  ring GO word, publish every slot command from the first through the last,
  set the pipe armed/control/watchdog bytes, and write GO once.

Therefore active-pipe append and a second GO are the wrong next model. The
vendor-shaped expansion path is to grow the existing pre-GO transaction: first
from two to four ordinary slots, with copied-completion capacity raised from
8 to 16, and separately from depth-two toward a deeper single-slot A-MPDU.
The per-slot owner, retry, BlockAck, and fair-service work above remains required
for both expansions.

`host_tx_policy::plan_ordinary_batch()` now captures that ordinary transaction
shape without touching MMIO. It requires an entirely unowned pipe, validates the
first eligible context and producer slot, selects at most four queue-ordered
contexts mapped to the same pipe, and assigns consecutive modulo-four slots.
Host tests cover wraparound, the vendor depth cap, other-pipe interleaving,
stale first-owner selection, invalid cursors, and attempted active-pipe reuse.
The `experimental-four-slot-ordinary` hardware feature now carries this plan
through reversible reservation and one pre-GO publication transaction. It
admits `Middle` registrations only for consecutive planner-approved slots,
retains the qualified two-slot path when the feature is disabled, and expands
copied completion capacity from 8 to 16 only in the experiment. A four-slot
wraparound test proves all duration words are published before one final GO.
The feature waits at most one additional nonempty service pass for four ready
contexts, then publishes whatever depth is available so queue filling cannot
create an unbounded latency regression.

The first ordinary-only hardware run (`7a5a7f6d...028311`) was healthy through
18,264 TX frames, 3.71 Mbit/s TCP, final 20/20 ping, BH alive, WSM idle, and zero
used buffers. It did **not** qualify four-slot publication: the diagnostic batch
word ended at `0x02000d70`, proving that every observed transaction still had
depth two because publication ran as soon as two contexts became ready. That
negative result motivated the bounded one-pass fill rule above.

The follow-up ordinary-only image (`3bbbd0f7...a16a43e2`) observed batch words
`0x03000005` after ping flood and `0x04000d7a` after iperf, proving real depth-
three and depth-four publications. It completed 18,766 TX frames, sustained
3.81 Mbit/s TCP, ended with 20/20 ping, and restored BH alive, WSM idle, and
zero used buffers. The exit trap restored the recovery image. This qualifies
the feature-gated four-slot ordinary transaction; the normal image remains at
the prior two-slot limit.

The combined depth-two plus four-slot image
(`af8bf622...a9e14e9d1`) also passed the same OTA qualification: 23,788 TX
frames, 18,650 aggregates, 4.49 Mbit/s TCP, final 20/20 ping, BH alive, WSM
idle, and zero used buffers. This is the qualified image for continued
aggregation work; recovery firmware was restored afterwards.

## Depth-four aggregate planning

The next aggregate step is pure and does not alter MMIO publication.
`host_tx_policy::plan_ampdu_group()` now selects a contiguous same-interface,
link, TID, and rate QoS-data prefix, requires both configured and operational BA
policy, applies the vendor zero-means-unbounded airtime budget, and caps the
first experiment at four members. A same-pipe incompatibility closes the chain
rather than being skipped.

`tx::build_planned_ampdu_descriptor()` models the corresponding vendor member
loop for two through four contiguous frame-state records. Every non-final
member emits the delimiter command and optional spacing transfer, the final
member emits the terminal command, and aggregate length uses the vendor's
`(length + 0x0b) & !3` arithmetic. Host tests cover depth caps, rate changes,
BA-policy gates, airtime limits, holes, spacing, exact opcode order, and PHY
length publication. The qualified depth-two publisher, BlockAck handling, and
retry paths remain unchanged until ownership and completion policy are expanded
for every planned member.

The host-only BlockAck model now covers the same four-member shape. It validates
one contiguous sequence prefix, classifies 12-bit wraparound against the
compressed 64-bit window, accumulates repeated observations with sticky
acknowledgements, rejects mismatched aggregate depths, and produces per-member
confirm/retry/give-up actions. Whole-aggregate rearm is allowed only for a total
miss when every member may retry at the same next rate and the BA session is
still active. This closes the pure decision layer; retained identities and MMIO
publication still remain depth-two.

The first production ownership boundary is now generalized independently of
MMIO construction. `SingleProbeMacBackend::register_ampdu_publications()`
validates a contiguous two-to-four context prefix, reserves the physical slot's
registry entry for the aggregate head, places later identities in the remaining
per-pipe entries, validates every context before mutation, and resets retry/BA
state once. The existing depth-two publisher now uses this path, so hardware
regression qualification is required even though on-air aggregate depth remains
two. Image `3b124888...a263eb8c` passed that regression with 20,018 TX frames,
14,446 aggregates, 3.40 Mbit/s TCP, final 20/20 ping, BH alive, WSM idle, and
zero used buffers. Recovery firmware was restored after the run.

Descriptor preparation now accepts the same bounded two-to-four context prefix.
It validates every linker-owned host context and frame-state identity, requires
one rate, emits each ordinary per-MPDU subdescriptor, builds the tested bounded
vendor opcode stream, and links every PAS through `next_in_ampdu` with an exact
terminal zero. The depth-two entry point is now a wrapper over this generic
constructor, while the publisher still supplies exactly two members. ARM uses
an explicitly unsafe unchecked A-MPDU transfer encoder only after these exact
packet-RAM identities have been construction-proven; host planning retains the
fallible encoder. Image `4c6518c4...f18b0a1e` passed follow-up hardware
qualification with 20,879 TX frames, 15,128 aggregates, 3.22 Mbit/s TCP, final
20/20 ping, BH alive, WSM idle, and zero used buffers. An immediately preceding
run completed 21,246 TX frames with the same clean firmware state but lost its
final ping check; the clean follow-up classifies that as transient RF/path
noise. Recovery firmware was restored after both runs.

Production BlockAck retention now walks the exact registered PAS chain, rejects
cycles, invalid frame-node identities, unregistered members, and chains beyond
four, and records the full two-to-four member sequence set against one exact
pipe/slot. Repeated compressed BA observations use the generic sticky merge,
and successful completion now requires acknowledgement of every retained
member. The existing missed-completion retry planner still deliberately accepts
only retained depth-two observations, so deeper publication remains blocked
until selective and whole-aggregate retry are generalized. Image
`3b888693...1098def` passed hardware qualification with 21,082 TX frames,
15,502 aggregates, 3.38 Mbit/s TCP, final 20/20 ping, BH alive, WSM idle, and
zero used buffers. Two preceding runs completed comparable traffic with the
same clean firmware state but lost only the final ping replies; the third clean
run, including a low reported RX bitrate, confirms the documented transient
RF/path behavior. Recovery firmware was restored after every run.

Whole-aggregate retry is now bounded over the same exact two-to-four member PAS
chain. It validates the complete chain before mutation, requires an active BA
session, available retry policy and status storage for every member, the
existing two-attempt aggregate limit, and one shared next rate. Only a retained
total miss may take the depth-three/four whole-rearm path; any partial deeper
observation gives up safely until selective retry is generalized. Rearm rebuilds
the generic descriptor in place from the retained CPU-form software-record
identity, preserves slot state 4, releases only that slot's command-mask bit,
and never issues a second GO. Image `26805e09...5a4d84ff` passed hardware
qualification with 19,174 TX frames, 14,338 aggregates, 2.63 Mbit/s TCP, final
20/20 ping, BH alive, WSM idle, and zero used buffers. Recovery firmware was
restored after the run.

Two attempts to replace the qualified depth-two selective tail with one generic
production tail were rejected. Images `71f2fe0e...086b6026` and
`e87d0c4b...ea6d9a67` both stopped at the first aggregate-era transition with
zero completed aggregates, eight host buffers outstanding, and a BH TX-confirm
timeout. Increasing stack headroom did not change the signature, so the generic
tail must not replace the exact depth-two sequence while deeper handling is
under development. `experimental-depth-four-ampdu` now provides a separate
feature boundary, and the host-only selective planner exhaustively covers all
16 four-member acknowledgement subsets plus unavailable and mixed-rate retry
groups. The qualified depth-two image remains byte-identical to
`26805e09...5a4d84ff`.

The production selective tail is now present only under
`experimental-depth-four-ampdu` and is entered only for retained chains deeper
than two. Depth-two observations continue through the previously qualified
selective and partial-give-up code. The deeper path recomputes its bounded plan
from declared retained identities, detaches terminal members once, rewrites a
same-rate retry subset in original order, converts a one-member subset to an
ordinary retry, and rebuilds a multi-member subset without a second GO. To fit
this opt-in diagnostic image inside the observed ITCM envelope, its optional
flight recorder is reduced from 256 to 192 records; normal diagnostics retain
256. The combined depth-four feature build has 52 bytes of qualified system
stack headroom. Deeper publication is still disconnected, so this stage can
regression-test feature isolation before any depth-three/four frame reaches the
MAC. The non-depth-four image `bbfa07f7...97397fe9` passed with 19,683 TX
frames, 14,494 aggregates, 2.70 Mbit/s TCP, final 20/20 ping, BH alive, WSM
idle, and zero used buffers. The isolated depth-four-feature image
`68962273...84435ae7` then passed the same depth-two workload with 22,139 TX
frames, 16,658 aggregates, 3.86 Mbit/s TCP, final 20/20 ping, BH alive, WSM
idle, and zero used buffers. Recovery firmware was restored after both runs.

Depth-three/four publication is now connected only under the same experimental
feature. The scheduler forms a contiguous policy-approved prefix, reserves all
members before mutation, builds one generic descriptor and member table, and
publishes the aggregate through one hardware trigger. Four compact telemetry
words report saturating depth-three and depth-four counts for attempted,
published, and completed aggregates. The retry word reports single-member and
multi-member selective retry groups so the common small retry subsets remain
visible; `tools/decode-ampdu-depth-telemetry.py` decodes the reused counters-MIB
fields.
The larger publication image reduces the optional flight recorder further to 64
records; normal diagnostics remain unchanged.

Image `2ecf814a...edd7552` reached 28,185 TX frames and 22,662 aggregates in its
first hardware qualification. A follow-up 20-second OTA TCP transfer sustained
2.20 Mbit/s, final ping was 20/20, and the firmware ended with BH alive, WSM
idle, no pending TX, and zero used buffers. Telemetry moved from a latest
attempted/published depth of three to depth four, recorded successful deeper
completion observations, and retained twelve observed depth-four retry events.
This proves real depth-four publication, completion, and retry activity rather
than only scheduler planning. The image is therefore qualified for continued
feature-gated depth-four work; the normal depth-two image remains unchanged.

The split depth histogram image `7036b5b8...bd248512` then quantified the
workload. Its clean repeat reached 18,146 TX frames and 13,489 aggregates,
sustained 2.43 Mbit/s TCP and 4.79 Mbit/s received UDP, completed final ping
20/20, and ended with BH alive, WSM idle, no pending TX, and zero used buffers.
It attempted and published 377 depth-three plus 2,944 depth-four aggregates,
fully acknowledged 317 depth-three plus 1,674 depth-four aggregates, and
observed thirteen depth-four retry events. An immediately preceding run had the
same clean firmware state but severe RF loss; the repeat restored normal traffic
without changing the image. Recovery firmware was restored after both runs.

A first attempt to act on partial BlockAck observations directly from the next
cooperative service pass was rejected. Accepting only slot state 3 left most
partial observations waiting and repeatedly ended with a transiently blocked
TX tail. Expanding that path to state 4 introduced a race with outstanding MAC
status ownership: image `a7d56f2a...fc51dbfb` stopped after 334 transmitted
frames with ten confirmations outstanding and the Linux BH reporting a fatal
TX-confirm timeout. The harness restored recovery firmware after the failure.

Partial BlockAck retry is instead deferred until the existing MAC-pipe watchdog
proves the transaction inactive. The watchdog-specific path validates the
retained observation, publication, live slot, and retry plan before rearming,
and deliberately does not acknowledge `PIPE_IRQ_PENDING`. Image
`79d20ec4...e7b885` completed 20,161 TX frames and 15,493 aggregates, sustained
2.62 Mbit/s TCP and 6.32 Mbit/s received UDP, and finished the final ping with
19/20 replies. It attempted and published 401 depth-three plus 3,309 depth-four
aggregates, fully acknowledged 341 depth-three plus 2,694 depth-four aggregates,
and executed seven single-member plus ten multi-member selective retries. The
firmware ended with BH alive, WSM idle, no pending TX, and zero used buffers.
Recovery firmware was restored and both recovery hashes were verified.

The unchanged image repeated with 19,821 TX frames and 15,829 aggregates,
2.12 Mbit/s TCP, and 5.46 Mbit/s received UDP. It attempted and published 372
depth-three plus 3,478 depth-four aggregates, fully acknowledged 303 plus 2,784,
and executed twelve single-member plus six multi-member selective retries. The
final ping path lost all twenty probes, but the firmware again ended with BH
alive, WSM idle, no pending TX, zero used buffers, and no driver errors; the
preceding flood delivered 3,586 of 3,605 probes. This matches the previously
observed transient post-traffic path loss rather than a firmware ownership
leak. Recovery hashes were verified again. The watchdog retry path is therefore
qualified for continued feature-gated development, while an earlier trigger
still requires explicit transaction-generation ownership.

The first outcome-census image `a08eb9f4...84a882f2` replaced the depth
histogram with sixteen packed saturating outcome counters under the separate
`experimental-ampdu-outcome-telemetry` feature. It completed 22,711 TX frames
and 17,870 aggregates, sustained 3.21 Mbit/s TCP and 7.22 Mbit/s received UDP,
finished ping 19/20, and ended with BH alive, WSM idle, no pending TX, and zero
used buffers. Of the first deeper BA observations, 80 were already complete and
54 were partial. Retry-event processing produced 15 non-empty selective plans,
18 empty plans, 13 rearms, 18 completion dispositions, and one aggregate
give-up. Crucially, neither watchdog rearm nor watchdog refusal occurred, and no
unmatched aggregate retired. This falsifies the working assumption that normal
partial-BA recovery waits 0.6-1.0 seconds for watchdog expiry: the qualified
watchdog path is a safety net, while the active bottleneck is the large fraction
of empty selective plans and the one-confirm/one-publication service cadence.
Recovery firmware was restored and both hashes were verified.

The policy-budget parity image `10498ade...8e17fe7` removed the artificial
two-attempt whole-aggregate limit and, like vendor firmware, applied the head
member's fallback rate to every member after each member's policy admitted a
retry. Its adaptive run completed 24,203 TX frames and 19,210 aggregates at
3.87 Mbit/s TCP and 6.20 Mbit/s received UDP, with 84 host-visible failures,
zero pending/used buffers, and no firmware errors. A fixed-MCS3 qualification
then sustained 3.50 Mbit/s over 30 seconds, completed final ping 20/20, and
ended clean. New telemetry proved that whole retry is common rather than
unexercised: the fixed-MCS3 run recorded 136 deeper and 153 depth-two whole
rearms. It still recorded 34 empty deeper selective plans and 153 host-visible
failures, and it did not meet the 4 Mbit/s fixed-MCS3 target. Policy parity is
therefore a qualified correctness and adaptive-throughput improvement, but the
outcome census falsifies it as the main remaining throughput fix. With zero
watchdog rearms in every census, the next experiment moves to confirmation
coalescing and service-loop latency before implementing a more invasive BA join.
Recovery firmware was restored and both hashes were verified again.

The feature-gated fast-loop image `1b07950e...f85b8ab` coalesces up to four
ready confirmations into WSM multi-confirm `0x041e` and skips the one-pass
batch-fill wait whenever the target pipe is idle. Its first equivalent build
sustained 5.78 Mbit/s TCP and 7.51 Mbit/s received UDP, completed final ping
20/20 at 2.71 ms average, and returned 28,319 confirmations through 11,160
multi-confirm messages. The cleaned final image repeated at 5.92 Mbit/s TCP
and 7.57 Mbit/s received UDP, completed final ping 20/20 at 4.34 ms average,
and returned 28,425 confirmations through 11,096 multi-confirm messages. Both
runs ended with BH alive, WSM idle, no pending TX, zero used buffers, and no
driver errors; recovery hashes were verified after each. This is the first
repeatable improvement larger than RF/run variance: TCP is 53% above the Stage
1 adaptive run and over twice the earlier qualified depth-four baseline. The
host `TX burst` counter remained near zero, proving it does not measure the
confirmation-credit bottleneck removed here. Multi-confirm and immediate
idle-pipe publication are qualified together under `experimental-fast-loop`;
separating their individual contributions is lower priority than fixing the
remaining empty selective plans and member requeue path.
