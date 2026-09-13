# XR819 TX throughput: consolidated findings

Current state of the board-TX throughput investigation as of 2026-09-13. The
narrative and per-experiment detail live in `rx-performance-investigation.md`;
this file is the summary to read first: what is established, what was retracted,
what is still open, and how to reproduce it.

## Question and target

The XR819 (cxd) 802.11n chip is driven by a no-OS Rust firmware on an Orange Pi
Zero over SDIO, with a cw1200-derived host driver. Vendor firmware reaches
~29.5 Mbit/s in the board->host direction; we measure 8.5-12.6 Mbit/s at fixed
MCS5 with depth-4 aggregates. The goal is to find the difference.

## Rig and method

- **AP**: self-hosted on the workstation's Intel AX200 (`nmcli connection up
  xr819-lab-intel`, SSID `xr819-lab`, channel 6, 192.168.77.1/24). The board's
  radio is moved into a `wifi-test` netns and associated with a pinned BSSID.
- **Rate control**: fixed MCS via `iw dev <if> set bitrates ht-mcs-2.4 <n>`,
  **verified from the AP's own receive report** (`rx bitrate: 52.0 MBit/s MCS 5`)
  rather than trusting the setting.
- **Board firmware**: probe builds from `xr819-firmware/` with
  `experimental-list-first-depth-four-ampdu,experimental-fast-loop,experimental-aggregate-rate-feedback,experimental-cycle-probe`
  (deep recent image `acf6b46c…`, stack 6760/6912). Host driver:
  `/tmp/xr819-ba-session-diag-driver/cw1200_core.ko` (`a237a79e…`).
- **Instruments**: firmware MIB counters (the probe overwrites counter words
  0..21; schema below), a monitor capture on a spare AR9271
  (`/tmp/xr819-monitor-start.sh`, analyser `/tmp/xr819-analyse-monitor.py`), and
  the AP's station dump.
- **Traffic**: iperf2, board-TX, 30 s windows; delivered rates are read on the
  measuring side.

CYC11/pipe probe MIB words: 0 marker `0x43594342`, 1 loops, 2 batches,
3 members, 4 GO->first-drain sum, 5 GO->phase-2 sum, 6 its count,
7 phase-2->first-drain sum, 8 its count, 9 PHY-command-2 events,
10 command-2 duration sum, 11 its count, 12 TX-start events, 13 TX-starts with
state 3, 14 with state 5, 15 TX-start->first-drain sum, 16 its count,
17 pipe-0 idle-starved passes, 18..=21 GO counts for pipes 0..3.
Decoder `/tmp/xr819-decode-cyc11.py`.

## Established by measurement

| finding | evidence |
| --- | --- |
| **Per-GO MAC start latency ~1.3 ms** | 1312 us from our GO write to the MAC's own type-`0x37` phase-2 event on 100% of 16,971 batches |
| Post-event path is prompt | phase-2 -> first completion 1082 us, of which ~845 us is measured air; ~one cooperative pass |
| Air is efficient | monitor: 4-member bursts, ~1.04 BlockAcks per burst, members ~215 us apart, span ~630 us, BA 2 us after the last member |
| No per-member drain cost | drain gaps 13-16 us, flat across a 10x rate change |
| No loop stalls | 0 cooperative-pass intervals >1 ms during traffic (mean 190-266 us) |
| No firmware poll delay | the completion is drained in the same pass as the MAC event that carries it |
| Host supply is not the limit | 34-42% loss on the host side means the device is slower than the host |
| PHY command 2 never runs | state 5 at all 17,379 TX-start events; the rate-specific branch is skipped as the vendor skips it in steady state |
| Depth is flat | depth-4 vs depth-8 A/B: 9.86/11.3 vs 10.2/10.5 and 9.46/12.0 Mbit/s |
| Pipe parallelism is flat | BE+VO on two pipes 12.6 Mbit/s vs BE+BE on one pipe 11.8 Mbit/s (11,099 GOs, all pipe 0) |
| Idle gaps are irrelevant | depth-4 GO->first-drain 2107 us after 1 s idle against 2227 us continuous |
| Watchdog is irrelevant | reload 2/5/10 gives 2114/2134/2100 us |
| Duplicate command 1 is not a delta | the vendor arms it once per scheduler pass, not once per batch |

Vendor facts from the static comparison (`vendor-tx-start-path.md`): command 1
publishes PHY state 3 (or preserves 5) and restarts a timer with no PHY MMIO;
command 2 is post-GO in both firmwares; the vendor runs `txp_pipe_tx_start` for
the phase-2 event directly in FIQ while we use a cooperative pass; the vendor
scheduler maps access categories onto idle pipes.

## Retracted during the investigation

Listed so nobody re-derives them: "~1 ms per member" (the drain gaps are 13-16
us); "the cycle scales with depth" (a tail average of a starved distribution);
"the confirm->GO gap is host round trip and dominates" (it is not; the MAC start
latency does); "a fixed ~1.7-2.2 ms excess" (it was the mean of a tail-heavy
distribution); "the trimmed depth-8 image caused association failures" (the join
path flakes about one run in three on any image, including untrimmed ones); "our
duplicate command 1 is a vendor delta".

## Open questions

1. **What caps the aggregate near 10-12 Mbit/s at MCS5**, when neither batch
   shape (depth) nor pipe count changes it, the device is the slower side, and
   the per-GO latency plus airtime account for only part of each batch interval
   (roughly 1.8 ms per batch is unaccounted in the two-flow case).
2. **The depth contradiction**: the linear depth model predicts depth-8 should
   gain 1.3-1.5x; the A/B was flat. Either the model does not extend past depth 4
   (mixed-rate members or more retries raise the slope) or the A/B was
   supply-masked.
3. **How vendor firmware reaches 29.5 Mbit/s.** With the same post-GO PHY
   ordering and the same command-2 skip, the vendor must either pay a smaller
   per-GO latency or keep more work in flight, and no evidence for either has
   been found yet.

## Method notes and pitfalls hit

- **Host tests are not the ARM build.** A silently failing ARM build left a stale
  ELF whose packed hash matched the previous image; `cargo test` passed because
  the probe hooks are `target_arch = "arm"`-gated. Always check the build result,
  not the artifact hash.
- **MIB windows**: the board monitor emits deltas, not cumulative counters; and
  probe counters must be bracketed around the traffic window.
- **iperf2 output is buffered**: host-side server summaries are lost if the
  server is killed rather than allowed to finish; prefer clients on the
  measuring side (`-R` reverse) so the delivered rate and loss are captured
  directly.
- **Join flakiness**: `wpa_state=ASSOCIATED` without the four-way handshake
  recurs about one run in three on any image; retry, and report results by
  attempt index rather than silently replacing failed cells.
- **Single runs**: almost every A/B here is one run per arm, which rejects large
  effects only. Anything near a 10-20% difference is inside the observed spread.
- **Harness hygiene**: start the UDP server the window needs, and hand the
  monitor adapter back with `sudo bash /tmp/xr819-monitor-stop.sh`.

## Reproduction pointers

- Probe runs: `/tmp/xr819-cyc11-run.sh` (TCP + UDP windows), `/tmp/xr819-wd-run.sh`
  (TCP + UDP, used for most sweeps), `/tmp/xr819-tos3-run.sh` (dual access
  category, host-side reverse clients), `/tmp/xr819-tos4-run.sh` (same-pipe
  control).
- Monitor: `/tmp/xr819-monitor-start.sh`, `/tmp/xr819-monitor-session.sh`,
  `/tmp/xr819-analyse-monitor.py`, capture `/tmp/xr819-mon.pcap`.
- Key logs: `/tmp/xr819-cyc11-mcs5.log`, `/tmp/xr819-tos3.log`,
  `/tmp/xr819-tos4.log`, `/tmp/xr819-cyc8-mcs5.log`, `/tmp/xr819-mon.pcap`.
