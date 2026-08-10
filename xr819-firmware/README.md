# XR819 open firmware experiment

This directory is an experimental `no_std` Rust implementation of firmware for
the XRadio XR819. It is expected to become a separate repository if hardware
bring-up succeeds.

The initial compatibility target is the CW1200 WSM ABI used by Linux's
`cw1200` driver. XR819-specific PHY, calibration and host-interface code stays
behind that protocol boundary.

## Current state

The current `hif-startup` binary delivers a CW1200 startup indication, completes
Linux probe, performs calibrated multi-channel active scans, transmits retained
probe-request templates, and returns real beacon/probe-response indications.
It remains an instrumented bring-up image rather than a complete vendor-order
implementation: JOIN/VIF activation, association, and normal data traffic are
not implemented.
The exact vendor call order, Radare2 excerpts, current implementation delta,
and experiment ledger are in
[`../xr819-hif-startup-flow.md`](../xr819-hif-startup-flow.md). Salvaged
semantic names, structure layouts, HIF/IRQ behavior, and RF algorithms from an
external annotated Ghidra archive are summarized in
[`../xr819-annotated-re-code.md`](../xr819-annotated-re-code.md). Complete
address-labelled decompiler exports are available under
[`../xr819-decompilation/`](../xr819-decompilation/).

Implemented:

- allocation-free WSM header parser/encoder;
- CW1200 startup-indication encoder;
- configuration-request parser with borrowed SDD/DPD data;
- reset-request parser;
- generic status-response encoder;
- host unit tests for wire layouts;
- ARMv5TE linker layout at `0x08000000`;
- naked reset entry that initializes the stack;
- a 68-byte Rust mailbox payload proven to execute on XR819 hardware;
- a Rust download bootloader implementing the host FIFO handshake;
- high-address ARM and low-address Thumb main-image layouts;
- translated XR819 platform, interrupt-controller, HIF activation, descriptor,
  and runtime-state initialization;
- bounded firmware-side HIF interrupt servicing for bring-up;
- the exact `0x0aa80004 = 0x200` clock/remap transition;
- vendor loader section-type-2 MMIO initialization (41 PHY/MAC pairs);
- four vendor type-zero MAC/PHY copies totaling 3472 bytes;
- extracted `0x16ac6` MAC calibration anchors and register lists in `src/phy.rs`;
- complete pure-Rust translation of the `0x17008` 22-to-80 MAC table generator;
- active vendor `0x16a38`/`0x198f2`/`0x16ca4` MAC software state;
- active, target-verified mode-zero MAC hardware initialization and bit-11 enable;
- faithful IRQ 6 platform-event callback replacing its diagnostic stub;
- vendor packet-RAM TX buffers and translated packet-DMA list setup;
- CW1200 startup delivery accepted by Linux, advertising only the real 2.4 GHz band;
- polled host-to-firmware RX descriptors;
- structured configuration confirmation with vendor-compatible -16.0 dBm
  minima, zero stepping, and SDD-derived `0xe3`/`0xe4` maxima;
- length-correct failure confirmations for unsupported read-MIB and join;
- retained TX-queue, EDCA, U-APSD, RCPI/RSSI, RX-filter, and template-frame
  configuration needed for interface bring-up and scan requests;
- honest failure status for other requests whose state effects are unimplemented;
- WSM command dispatch with sequence/link fields stripped and explicit `if_id`;
- modulo-eight sequence stamping on every normal host-bound WSM message;
- validated and retained SDD/DPD calibration TLVs, with live publication of
  reference, power-profile, channel-table, and temperature fallback state into
  the vendor DTCM layout;
- explicit clearing of linker-defined Rust `.bss` before using retained state;
- successful registration of a Linux `phy` and `wlan0`;
- borrowed parsing and fixed-storage retention of scan channels/SSIDs;
- corrected false channel-function boundaries at `0x1856a`/`0x1814a`;
- verified vendor start-scan dispatch `0x10cfa -> 0x13d44`;
- traced event bit 10 through `0x14352 -> 0x14304 -> 0xfdfa -> 0xf802`;
- translated the compact `0xfdfa` channel-control encoding;
- exact `zerocopy` eight-byte `0x14304 -> 0xf802` channel request ABI;
- translated pure `0x124c0` channel-control gate classification;
- traced `0xf802 -> 0xf78c -> 0x16dd6 -> 0x16b0a -> 0x166ea` radio tuning;
- translated PHY-mode, recalibration, and same-mode transition decisions;
- verified `0x1682a` 2.4 GHz channel-to-frequency mapping;
- translated `0x17224` measurement timing and `0x19928` channel offsets;
- exact `0x18f2c -> 0x18ef0` fractional PLL synthesis;
- detached exact `0x1838c` PLL latch and `0x17e92` measurement-path MMIO;
- translated `0x1a1fc`/`0x19f8e` measurement register save/restore envelope;
- detached complete mode-zero `0x19f8e` trigger/poll/read/restore routine;
- translated mode-zero counter scaling, limits, fallback, and fatal early return;
- allocation-free SDD `0x30/0x31` threshold-table parsing for `0x19dd0`;
- allocation-free SDD `0xec` channel-step parsing for `0x1a112`;
- combined mode-zero SDD channel calibration, verified against the target's 744-byte SDD;
- translated primary and refinement arithmetic from `0x17c20(1,1)`;
- detached `0x168b8`, `0x17884`, and `0x178be` calibration MMIO helpers;
- translated `0x17b70` 23-bit I/Q accumulator decoding;
- detached complete `0x178ce` calibration register envelope and timer waits;
- translated `0x179ea` normalization, shift-state update, and gain-indexed publication plan;
- translated `0x17ac8` signed-8 primary coefficient packing;
- detached bounded `0x17bf2 -> 0x17b70` sample command and accumulator read;
- allocation-free twelve-gain `0x17c20` arithmetic/publication series and
  complete bounded primary/optional-secondary hardware acquisition envelope
  with nested timeout cleanup and exact publication ordering;
- initial candidate and signed-12 correction packing from dynamic IQ/DC calibration;
- detached bounded 64-word ADC capture from dynamic IQ/DC calibration;
- pure fixed-point three-correlation DFT and candidate normalization from `0x18480`;
- typed allocation-free translation of every `rf_op_dispatch2` search stage,
  including polynomial case 6 and bounded parabolic case 12 refinement;
- translated dynamic-IQ pass doubling, averaging, profile-seed addition,
  pair-restoration checks, quality flags, and asymmetric vendor publication rule;
- detached typed `rf_save_band_regs`/`rf_load_band_regs` snapshot envelope with
  exact profile overrides, delays, calibration start/stop words, conditional IQ
  restores, and PLL restart;
- pure translations of both `rf_program_synth_freq` branches, including the
  21-step restoring divider and the larger wrapping 64-bit fixed-point chain;
- allocation-free 13-stage dynamic-IQ pass orchestration with exact capture,
  candidate-publication, DFT, first-pass control, and failure-stop scheduling;
- pass-level dynamic-IQ baseline capture, doubled pass count, normalized
  accumulation, accepted-pass averaging, and final seed addition;
- final dynamic-IQ verification, sixteen-entry correction-bank replication,
  conditional pair restoration, quality flags, and asymmetric DTCM-state
  publication;
- complete detached dynamic-IQ hardware wrapper with the vendor profile
  shortcut, live synth preparation, exact control-word staging,
  timeout-preserving capture, candidate publication, and guaranteed restoration
  on every normal post-acquisition return;
- complete detached `phy_set_channel_full` normal path with profile-zero/one
  PLL calculation, timing, temperature conversion, calibration, AGC,
  SDD-derived threshold/TX-power publication, frequency offset, configuration
  slots, correction-cache initialization, and channel recording;
- byte verification of all loader MMIO pairs and MAC/PHY copies against the
  annotated `xr819-fw.tar.gz` firmware container;
- vendor channel-timing validation and event-bit-10 scan activation semantics;
- verified completion path `0x13fac -> 0x111ba -> 0xed4c`;
- vendor-aligned 12-byte asynchronous empty scan completion after successfully
  running the live channel transition for the first retained scan channel;
- repeated hardware scans complete with BH alive, WSM idle, and no outstanding
  firmware buffers;
- continuously serviced packet-DMA RX outside scan state, matching the vendor
  receive task and preventing idle-era frames from contaminating later dwells;
- active scan requests publish directed or wildcard probe templates through a
  bounded one-context TX/completion path and retain the host-requested dwell;
- hardware-validated cold channel-1 scans returning 2–3 BSS records around
  -66 dBm, repeated channel-1 scans returning up to 3 BSS records, and full
  scans returning 4–6 BSS records without FIFO leaks or BH failure;
- a passive-RX path with multi-channel dwell, packet-DMA FIFO recycling,
  beacon/probe-response and on-air DS-channel filtering, WSM receive
  indications, and fixed diagnostic counters;
- vendor-style zero-copy RX indications that use the FIFO slot's 16-byte
  headroom and defer slot recycling until HIF TX descriptor reclamation, so the
  full advertised 1600-byte frame size is supported instead of the temporary
  368-byte copied-frame limit. The newest MAC/RX activation and zero-copy
  changes remain hardware-unvalidated pending a target reset;
- a cooperative PHY transition scheduler matching the vendor's two-phase
  `phy_cal_run_step_timed` flow: hardware work enters state 1, waits 120 vendor
  timer ticks without blocking the HIF loop, then publishes terminal state 2
  and enables RX before channel dwell begins;
- translated wake restoration for the represented MAC state: static MAC/RX
  register banks, packet-DMA pipe records, producer/consumer synchronization,
  TBTT and pipe descriptor images, bounded controller readiness, LMC pool reset,
  PAS fallback/rate tables, IFS timing, ACK/CTS control descriptors, mode/BSSID
  restoration, and register-context save ordering.

Not yet implemented:

- active-VIF channel restoration, power-save resumption, and scheduler-bit-21
  work beyond the returned single-probe domain;
- IRQ 18/20/21 completion consumers and faithful IRQ-driven HIF scheduling;
- complete HIF queue/scheduler accounting;
- JOIN/VIF state effects, association, and normal TX/RX data traffic;
- production exception reporting and recovery behavior.

## Intended bring-up order

1. Replace the mailbox main image with a minimal HIF/WSM transport loop.
2. Send a CW1200-compatible startup indication.
3. Accept configuration and reset commands.
4. Apply the SDD and static PHY initialization tables.
5. Implement channel setup and read-only RX.
6. Implement TX and confirmations.

Run host-side protocol tests with:

```sh
cargo test
```

Active probe scanning is enabled by the default Cargo feature set. Build with
`--no-default-features` for the retained passive rollback image.

Build the hardware mailbox payload with:

```sh
cargo +nightly build --release --bin mailbox \
    --target armv5te-none-eabi -Z build-std=core
```

The raw payload is produced from the ELF with `arm-none-eabi-objcopy -O binary`.

## Fast hardware iteration

A failed experimental startup leaves XR819 in queue mode. The driver can
recover ordinary failures by asserting embedded CPU reset and restoring direct
access mode. For a cleaner XR819 reset, unbind and rebind the `1c10000.mmc`
platform device; partial packet-DMA initialization can still require a physical
power cycle.

After copying new firmware to the target, reprobe with:

```sh
echo mmc1:0001:1 > /sys/bus/sdio/drivers/cw1200_wlan_sdio/bind
```

The write returns the probe error when experimental firmware times out, but the
new image was still downloaded and executed. Replace the firmware and repeat.
This fast path is intended for firmware-only iterations after a failed probe.
A reboot remains required after experiments that alter persistent CP15/cache
state or initialize the packet-DMA/platform engines, and is recommended after
replacing kernel modules or a fatal BH/IRQ state.

A postmortem halt followed by MMC unbind once left the target in uninterruptible
sleep and required a physical power cycle. The corrected ordered downloader is
now deployed and stable on `phy1`; avoid debugfs halt during active bring-up.
