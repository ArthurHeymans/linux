# XR819 open firmware experiment

This directory is an experimental `no_std` Rust implementation of firmware for
the XRadio XR819. It is expected to become a separate repository if hardware
bring-up succeeds.

The initial compatibility target is the CW1200 WSM ABI used by Linux's
`cw1200` driver. XR819-specific PHY, calibration and host-interface code stays
behind that protocol boundary.

## Current state

The current `hif-startup` binary now delivers a CW1200 startup indication,
receives the initial host commands, and completes Linux probe. It remains an
instrumented bring-up image rather than a complete vendor-order implementation.
The exact vendor call order, Radare2 excerpts, current implementation delta,
and experiment ledger are in
[`../xr819-hif-startup-flow.md`](../xr819-hif-startup-flow.md). Salvaged
semantic names, structure layouts, HIF/IRQ behavior, and RF algorithms from an
external annotated Ghidra archive are summarized in
[`../xr819-annotated-re-code.md`](../xr819-annotated-re-code.md).

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
- CW1200 startup delivery accepted by Linux;
- polled host-to-firmware RX descriptors;
- generic write-MIB confirmation and structured configuration confirmation;
- validated and retained SDD/DPD calibration TLVs;
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
- allocation-free twelve-gain `0x17c20` arithmetic/publication series;
- initial candidate and signed-12 correction packing from dynamic IQ/DC calibration;
- detached bounded 64-word ADC capture from dynamic IQ/DC calibration;
- pure fixed-point three-correlation DFT and candidate normalization from `0x18480`;
- typed allocation-free translation of every `rf_op_dispatch2` search stage,
  including polynomial case 6 and bounded parabolic case 12 refinement;
- byte verification of all loader MMIO pairs and MAC/PHY copies against the
  annotated `xr819-fw.tar.gz` firmware container;
- vendor channel-timing validation and event-bit-10 scan activation semantics;
- verified completion path `0x13fac -> 0x111ba -> 0xed4c`;
- vendor-aligned 12-byte asynchronous empty scan completion while the real PHY scan path is incomplete.

Not yet implemented:

- interrupt vectors and exception reporting;
- real IRQ-driven HIF receive/completion handling;
- complete HIF queue/scheduler accounting;
- WSM dispatcher/state machine beyond initial probe requests;
- PHY initialization and calibration;
- TX/RX data path.

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
