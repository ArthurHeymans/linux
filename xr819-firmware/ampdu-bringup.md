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
