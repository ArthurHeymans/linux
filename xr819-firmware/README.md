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
[`../xr819-hif-startup-flow.md`](../xr819-hif-startup-flow.md).

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
- vendor packet-RAM TX buffers and translated packet-DMA list setup;
- CW1200 startup delivery accepted by Linux;
- polled host-to-firmware RX descriptors;
- generic write-MIB confirmation and structured configuration confirmation;
- successful registration of a Linux `phy` and `wlan0`.

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
