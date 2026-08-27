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
