# XR819 firmware reverse engineering notes

This document records findings from static and differential analysis of the
vendor-provided XR819 firmware. Function names are provisional until enough
callers and data structures have been identified to assign semantic names.

The authoritative HIF startup call order, Radare2 excerpts, implementation
matrix, and experiment ledger are maintained separately in
[`xr819-hif-startup-flow.md`](xr819-hif-startup-flow.md). Future startup work
should begin there rather than reconstructing sequence from session history.

## Scope and goals

The immediate questions are:

1. Does the downloadable firmware directly control the PHY, or does it merely
   communicate with another opaque processor?
2. Which parts of channel selection, calibration, RX and TX must replacement
   firmware implement?
3. Can the vendor initialization be separated from the runtime low-MAC, making
   an incremental open firmware possible?

The current conclusion is that the downloadable firmware directly programs
substantial PHY state. The modulation/baseband datapath is presumably hardware,
but channel selection, calibration and coefficient programming are performed by
ARM firmware through memory-mapped registers.

## Analysed images

The principal image currently loaded in Ghidra is the 2018 XR819 firmware:

| File | Size | SHA-256 |
| --- | ---: | --- |
| `boot_xr819.bin` | 2,308 | `6583350b3eb12f70fc6d6081426717bd0019b55c6558ffe820c1548f0702bb8c` |
| `fw_xr819.bin` | 130,472 | `fb81436ad7cc0876614a2a9c2a54c5a93a75315aee164e3a3afe3db80842a9e1` |
| `sdd_xr819.bin` | 744 | `84d3fb3ca8e5d25a0c113a5063bccbeb5b53da230a0afa236b5b625f37db5161` |

Embedded version strings:

```text
XR_C01.08.52.58 Jul 19 2018 18:53:57
XRadio_FW Jul 19 2018 18:54:04
```

A commonly distributed older image is 126,416 bytes and identifies itself as a
June 2016 build. A first comparison shows the same 102 unique
`0x0ab8xxxx`-through-`0x0abcxxxx` address-like literals in both images, although
their code/data offsets differ. This suggests the underlying PHY register map
remained stable between the 2016 and 2018 firmware.

The firmware is little-endian mixed ARM/Thumb code. The analysed payload is
imported at address zero after removing its container header.

## Firmware-side source modules

Source filename strings retained in the image include:

```text
pac_phy.c
phy_mgt.c
pas_rates.c
rx_fifo.c
rx_handler.c
tx_ptcs.c
wsmlmac.c
syn_scan.c
syn_start.c
```

The filenames alone are not proof of direct PHY control, but the code near
`phy_mgt.c` contains direct accesses to several hardware register banks.

## Preliminary address-space map

Observed addresses fall into at least two broad classes:

| Range | Preliminary interpretation |
| --- | --- |
| `0x0400xxxx` | Firmware RAM/global state |
| `0x0ab8xxxx` | Hardware register bank, PHY-related |
| `0x0abaxxxx` | Hardware register bank, PHY-related |
| `0x0abbxxxx` | Hardware register bank, PHY/calibration-related |
| `0x0abcxxxx` | Hardware register bank, PHY/calibration-related |
| `0xfff0xxxx` | Shared/ROM/platform region; not yet fully classified |

The exact peripheral names and register widths are not yet known. The
`0x0ab8` through `0x0abc` classification is based on direct volatile-style
accesses, save/restore operations, channel-dependent programming and
calibration sequences.

## Direct PHY register programming

### Coefficient programming: `FUN_0001b868`

The function at `0x1b868` writes two indexed register arrays:

```c
*(u32 *)(0x0abb801c + index * 4) =
        ((coefficient_b & 0x3ff) << 10) |
        (coefficient_a & 0x3ff);

*(u32 *)(0x0abb8400 + index * 4) = transformed_coefficient;
```

The second value is derived from a division-like helper and represented around
`0x800`/`0x1000`. This looks like signed or phase/frequency coefficient
programming rather than a host-interface mailbox.

Confidence: **high** that this programs PHY hardware; **low** on the precise
meaning of the coefficients.

### PHY register save/modify: `FUN_0001bc5c`

The function at `0x1bc5c` saves several registers and installs a temporary
configuration. Its literal pool identifies bases including:

```text
0x0abb8000
0x0abc0080
```

It changes fields using masks/values including:

```text
0x00000800
0x07000200
0x00000101
0x00000002
```

The saved values are kept in a structure in firmware RAM around `0x0400994c`.
This function is called at the start of the measurement routine below.

Confidence: **high** that this temporarily reconfigures PHY hardware.

### Measurement/calibration sequence: `FUN_0001b9ee`

The function at `0x1b9ee`:

1. Calls `FUN_0001bc5c` to save and modify PHY registers.
2. Selects one or two measurement modes.
3. Writes a control value to a register near `0x0abb800c`.
4. Waits in 10-unit intervals.
5. Reads a status/result bit.
6. Reads a measurement value from another hardware register.
7. Computes a correction using firmware calibration state.
8. Stores the resulting correction.
9. Restores the saved register state.

The arithmetic combines a measured value, a signed calibration offset and a
scale factor of 1000. The exact physical quantity is not yet established, but a
frequency/clock or gain calibration is plausible.

Confidence: **high** that this is active PHY calibration; **medium-low** on the
specific calibration type.

## Channel/PHY configuration path correction

Earlier analysis incorrectly promoted interior Thumb offsets to standalone
functions. Radare2 `af` at `0x1856a` and `0x1814a` created artificial function
boundaries and therefore a false call chain.

Cross-checking with uninterrupted `pdf` output and Ghidra shows:

- `0x1856a` is inside the large routine beginning at `0x18480`; it is part of a
  64-iteration transform accumulating signed calibration outputs.
- `0x1814a` is an interior arithmetic block reached by branches in a larger
  calibration routine, not a callable channel-setup entry.
- The previously listed `0x18836` and `0x111ec` relationships must therefore
  not be used to drive the Rust scan implementation without fresh caller and
  boundary analysis.

The region remains clearly PHY/calibration-related and contains direct hardware
references including `0x0abb8300` and `0x0abb8040`, but it is not the WSM entry.
The real start-scan dispatch is now established independently: the ID-masked
jump table at `0x04000710` sends command ID 7 to `0x10cfa`, which calls the
valid function `0x13d44`. That function validates channel timing, copies scan
state, marks `0x0400860c`, and raises platform event bit 10. The event callback
registration at `0x698..0x6e4` maps bit 10 to `0x14352`; its scan state machine
calls per-channel setup `0x14304`, which calls `0xfdfa` and then the large MAC
programmer `0xf802`. `0xfdfa`'s compact channel-control encoding is now
translated and tested in Rust. Retained scan channels produce the exact
`zerocopy` eight-byte ABI assembled by `0x14304`, including operation, option,
control, channel, link count, and link ID. The `0x124c0` control-gate
classifier immediately preceding `0xf802` is also translated as pure logic.
`0xf802`, rather than the withdrawn interior labels, is the next real
channel-programming boundary. The terminal path is also established:
`0x13fac` performs cleanup and calls `0x111ba`, which constructs the 12-byte
`0x0806` scan-complete indication and queues it through `0xed4c`.

`0x1a98c`, `0x18c84`, and `0x1b388` still contain channel-dependent hardware
and signed-correction behavior, but their callers need to be re-established
from valid function boundaries.

## Additional PHY register banks

Literal pools used by functions in the same subsystem expose further hardware
addresses:

```text
0x0ab80800
0x0ab80c00
0x0ab80040
0x0ab88000
0x0aba2000
0x0aba8040
0x0abb8000
0x0abb801c
0x0abb8040
0x0abb8300
0x0abb8400
0x0abc0080
```

This is a broad hardware interface, not a single mailbox to another firmware.
It suggests the ARM firmware owns configuration of multiple baseband/PHY blocks.

An aligned literal scan currently finds **102 unique addresses** between
`0x0ab80000` and `0x0abc8004`. Some could be data that merely resembles an
address, but the majority occur in coherent register tables or literal pools
used by hardware-access routines.

### Embedded register initialization table

The data at `0x1ee80` begins with a length of `0x148` bytes, followed by exactly
41 address/value pairs. The first entries are:

```text
address       value
0x0ab80004    0x00000023
0x0ab80008    0x0000003f
0x0ab80014    0x0000001c
0x0ab80018    0x0000002c
0x0ab8001c    0x00000044
0x0ab80020    0x00000041
0x0ab80024    0x0000003e
0x0ab80064    0x00210140
0x0ab80070    0x00000006
0x0ab80074    0x00004040
0x0ab80080    0x0190007e
0x0ab80108    0x00200300
```

Later entries cover `0x0ab80cxx`, `0x0ab88xxx`, `0x0ab9xxxx`, `0x0abaxxxx`
and `0x0abbxxxx`. This is strong evidence of table-driven hardware
initialization inside the downloadable firmware. The table does not occur in
the 744-byte SDD and therefore appears to be a firmware-supplied default or
silicon initialization table rather than board calibration data.

The identical 41 address/value pairs occur in the 2016 firmware at `0x1dea8`.
Thus this initialization table and its register values were preserved exactly
across the two firmware releases.

A byte-level comparison of identified calibration routines gives:

| Routine | 2016/2018 equal bytes |
| --- | ---: |
| PHY register save/measurement setup | 100.0% |
| coefficient-pair writer | 100.0% |
| measurement read-pair primitive | 98.5% |
| multipoint calibration | 97.2% |
| small measurement calibration | 96.6% |
| 64-sample correlation | 95.6% |

Most differing bytes are consistent with relocated branches or literals after
other firmware code changed size. This indicates that the core measurement and
calibration machinery was exceptionally stable between 2016 and 2018, making a
mechanical compatibility translation attractive.

The large I/Q-like iterative routine did change more substantially. Its 2016
entry begins near `0x19474`, while the 2018 version begins at `0x1a3a8`. Both
have the same broad shape--large stack workspace, repeated measurement setup,
component accumulation and final correction application--but structure sizes
and portions of the algorithm differ. The open implementation should therefore
follow the newer 2018 behavior and use the 2016 routine only as supporting
evidence.

The consumer is now identified: this is firmware-container section type 2, not
a low-firmware data object. The vendor download bootloader reads a `0x148`-byte
section as 41 address/value pairs and performs each MMIO write directly before
jumping to firmware. The Rust downloader now reproduces all 41 writes. This
explains why no low-code reference to file offset `0x1ee80` existed.

### Additional static PHY profiles

The firmware contains several more address/value tables delimited by a pair of
`0xffffffff` words. Examples include:

- two ten-register profiles at `0x1d948` and `0x1d9a0`;
- a four-register profile at `0x1da08`;
- a seven-register profile at `0x1da48`;
- a larger `0x0abbxxxx` profile at `0x1dc30`.

The two ten-register profiles are nearly identical. They differ only in:

```text
profile A                       profile B
0x0ab80400 = 0x55f4282b        0x0ab80400 = 0x55f4292b
0x0ab80410 = 0x0000003a        0x0ab80410 = 0x0000003b
```

All remaining register values match. These are likely adjacent channel,
bandwidth, silicon-mode or calibration profiles. Their consumers still need to
be located before assigning a precise interpretation.

The `0xffffffff, 0xffffffff` terminator makes these tables particularly suitable
for an initial open implementation: once their selection conditions are known,
they can be represented directly as Rust slices of register/value pairs.

## Runtime calibration routines

The channel path conditionally invokes two substantial calibration families
when the firmware's PHY-type selector equals 2.

### `FUN_00019680`: multi-point measurement and correction

`FUN_00019680` saves hardware state and performs repeated measurements over
roughly twelve settings. For each setting it:

1. selects a hardware measurement configuration;
2. obtains two pairs of readings;
3. computes signed differences scaled by `0x100`;
4. derives correction slopes/intercepts;
5. stores per-setting correction values;
6. restores the previous hardware state at completion.

It directly uses register banks including:

```text
0x0abb8180
0x0abb8600
0x0abc0000
```

A second phase computes two further correction terms scaled by `0x4000`.
This is consistent with gain, I/Q or DC-offset calibration, but the exact
quantity remains unconfirmed.

The core measurement primitive is `FUN_000195d0`. It exposes a concrete
hardware measurement interface:

```text
0x0abb80f0  command/status
0x0abb810c  signed result 0
0x0abb8110  signed result 1
0x0abb8114  measurement configuration
0x0abb81a4  setting/select value
```

`FUN_000195d0` writes the command word, polls command/status bit 4 until the
hardware reports completion, and reads two signed 23-bit results. This is
strong evidence for a dedicated PHY measurement engine in hardware controlled
directly by firmware.

### `FUN_0001a3a8`: iterative four-component calibration

`FUN_0001a3a8` is a large iterative routine. It:

- runs up to thirteen measurement configurations per iteration;
- accumulates four signed components;
- rejects components outside `0x200` and `0x800` bounds;
- averages accepted measurements;
- writes resulting pairs across two sixteen-entry arrays;
- performs a final verification measurement;
- records separate validity flags for the resulting corrections.

Its helper chain accesses `0x0abb8000`, `0x0abb8180` and `0x0abc0000`.
The four-component structure and separate correction arrays make transmit or
receive I/Q/DC calibration a plausible interpretation.

A helper in this path, `FUN_00019ee0`, processes 64 captured values. It
multiplies each value by two phase-dependent lookup functions and accumulates
real/imaginary-style component pairs before writing six signed outputs. The
lookup functions are consistent with sine/cosine table operations. This raises
confidence that the routine estimates amplitude/phase or I/Q error from a
captured test waveform rather than performing generic control arithmetic.

`FUN_0001a8bc` either reuses cached correction pairs for mode 5 or invokes this
full calibration with constant `0x07ff0110`.

### Calibration dispatch

`FUN_0001810e` dispatches both calibration families according to a mode value.
It calls `FUN_00018308`, which invokes `FUN_00019680` for PHY type 2, followed
by `FUN_00018388`, which invokes `FUN_0001a8bc` for the same PHY type.

Therefore channel configuration can trigger active measurement loops, not just
static register-table replay. A replacement will probably need either:

1. to reproduce these algorithms;
2. to run them once and preserve their outputs; or
3. initially to retain vendor-assisted PHY initialization/calibration and take
   control afterward.

## Connection to MAC state transitions

The channel/PHY path has now been traced upward to `FUN_00011262`.
`FUN_00011262` accepts a variable-length state description containing interface
indices, mode fields and a 16-bit value at offset 4. It passes that offset-4
value to `FUN_000111ec`, which ultimately reaches
`FUN_00018836 -> FUN_0001856a -> FUN_0001814a`.

`FUN_00011262` is called from three major state-machine areas:

```text
0x159cc
0x15d64
0x16eec
```

The callers can now be characterized more closely:

- `0x15d64` constructs a synchronization description from scan state and is
  reached from `0x14006`, which stores the requested scan channel. This is the
  clearest scan-channel path.
- `0x159cc` constructs mode 3 state and is used while a scan/synchronization
  operation is restarted or resumed.
- `0x16eec` walks up to three active firmware interfaces, selects those whose
  stored channel matches the requested value, and constructs a shared
  synchronization state. It is associated with normal start/join/interface
  transitions rather than a one-time boot path.

This means PHY channel programming is not merely boot-time initialization: it
is invoked by scan and normal runtime MAC state changes.

`FUN_000111ec` also quiesces or reconfigures surrounding TX/RX state before the
PHY call and restores it afterward. This is consistent with a real channel
transition requiring the low MAC and PHY to be coordinated.

### Low-MAC scheduler connection

A separate path reaches `FUN_00016eec` through:

```text
FUN_00003c4e
  -> FUN_00016f66
       -> FUN_00016eec
            -> FUN_00011262
                 -> PHY channel configuration
```

`FUN_00003c4e` manages a current state and a linked queue of pending states. It
compares the 16-bit value at state offset `0x0e` between the current and incoming
states. When the values differ, it invokes `FUN_00016f66` before scheduling the
new state; when they match, it can switch state without that reconfiguration.

This strongly identifies the field as a channel/frequency selector and proves
that firmware scheduler transitions actively invoke the PHY channel path. The
PHY operation is therefore part of timing-sensitive low-MAC scheduling, not
just host-command handling.

## SDD relationship

The host treats `sdd_xr819.bin` as TLV data. Known element IDs include:

```text
0xc5  reference frequency
0xeb  PTA configuration
```

The host extracts the reference frequency for DPLL setup, then sends the full
744-byte SDD to firmware as `dpdData` during operational-mode configuration.
Therefore firmware-side PHY setup almost certainly consumes additional
board/RF tables from the SDD.

The current replacement strategy should retain the vendor SDD as opaque board
calibration input until each element is understood. Replacing executable
firmware does not require inventing new calibration values.

## CW1200 hardware comparison

A literal-address comparison against CW1200 `wsm_22.bin` provides strong
evidence that XR819 retained the same underlying PHY architecture.

For the `0x0ab80000` through `0x0abcffff` hardware ranges:

```text
XR819 unique address literals:     102
CW1200 unique address literals:     63
shared address literals:            60
CW1200-only address literals:        3
```

Thus **60 of 63 (95.2%)** CW1200 PHY register addresses also occur in XR819
firmware. XR819 contains 42 additional addresses, consistent with an extended
or revised implementation rather than a completely different PHY.

More significantly, CW1200 firmware contains the same `0x148`-byte,
41-register initialization table found in both XR819 firmware versions. All 41
address/value pairs are identical. This table was therefore stable across:

- CW1200 `WSM_A30.02.0395`, built in 2012;
- XR819 firmware built in 2016;
- XR819 firmware built in 2018.

This makes it plausible that a replacement XR819 firmware can adopt CW1200-like
WSM semantics without fighting a fundamentally different radio. The remaining
differences are likely concentrated in boot/host-interface integration,
extensions, calibration details and firmware policy/state-machine behavior.

## Current architectural conclusion

The evidence currently supports this split:

```text
ARM firmware
  - channel configuration
  - PHY register programming
  - measurement/calibration loops
  - rate-dependent PHY parameters
  - RX/TX descriptor handling
  - timing-sensitive low MAC

PHY/baseband hardware
  - modulation/demodulation datapath
  - coding/decoding and filtering
  - timing primitives and likely portions of ACK handling
```

No evidence yet indicates that channel setup is delegated to a second hidden
processor. This is encouraging: the required operations should be observable
and reproducible from the existing firmware.

## Live hardware access

A guarded debugfs interface has now been added to the development `cw1200`
driver and tested on the Orange Pi Zero target.

Files exposed for XR819 are:

```text
/sys/kernel/debug/ieee80211/phy0/cw1200/halt
/sys/kernel/debug/ieee80211/phy0/cw1200/ahb
/sys/kernel/debug/ieee80211/phy0/cw1200/apb
```

Raw control is disabled unless the `cw1200_core.unsafe_debugfs` module parameter
is enabled and the caller has `CAP_SYS_RAWIO`.

The operational firmware normally leaves the host interface in queue mode. AHB
or APB prefetch does not complete in this mode, so raw reads cannot safely be
performed while the firmware and bottom half continue running. Writing `1` to
`halt` now:

1. locks host TX;
2. suspends the driver bottom half;
3. asserts the XR819 CPU reset bit;
4. switches the host interface back to access mode.

With the device halted, direct AHB reads returned meaningful data:

```text
0xfff00000 = 0xe3877000
0xfff00010 = 0xe12fff10

0x0abb80f0 = 0x01004018  measurement command/status
0x0abb810c = 0x007f30de  measurement result 0
0x0abb8110 = 0x00001653  measurement result 1
0x0abb8114 = 0x00000000  measurement configuration
0x0abb81a4 = 0x00000000  measurement selector
```

This validates the statically recovered PHY addresses and confirms that the
host can inspect the measurement engine directly.

A test write/readback at `0x08000000` also succeeded while halted, proving host
AHB writes work. However, resuming the vendor firmware after this experiment
triggered an assertion at `hif.c:674`. The original word was restored before
resume, so either this address has side effects/aliases runtime state or repeated
CPU reset/access-mode transitions require more HIF restoration than currently
performed.

Consequently:

- read-only halted snapshots are the current safe operation;
- arbitrary writes should be treated as destructive experiments;
- reboot is the reliable recovery path after writes;
- firmware resume after a halt is not yet considered generally safe.

### First open Rust code execution

The debug interface now also supports:

```text
upload  binary writes into SRAM beginning at 0x08000000
run     releases CPU reset while retaining host access mode
```

A `no_std` Rust payload was linked for ARMv5TE at `0x08000000`. Its naked entry
sets `sp` to `0x0800ff00`, branches into Rust, writes `0x58523831` (`"XR81"`) to
`0x0900ff80`, and continuously increments `0x0900ff8c`.

The 68-byte raw image was uploaded and executed successfully. Live reads showed:

```text
SRAM 0x08000000 = 0xe59fd000  first payload instruction
APB  0x0900ff80 = 0x58523831  mailbox magic
APB  0x0900ff8c = changing     heartbeat counter
```

This proves:

- XR819 executes ordinary ARMv5TE Rust-generated code;
- the CPU reset vector for the downloaded boot payload is `0x08000000`;
- a stack near the top of the 64 KiB SRAM window is usable;
- host APB communication remains available while custom code runs;
- custom firmware can be iterated without replacing the system kernel image.

### Open Rust download bootloader

A Rust replacement for the 2.3 KiB vendor bootloader is now operational. The
current binary is approximately 228 bytes and implements:

1. the `0x12345678` bootloader-ready handshake;
2. download-control initialization at `0x0900ff80`;
3. the 32 KiB ring FIFO at `0x09004000`;
4. host `put` and firmware `get` producer/consumer counters;
5. copying the streamed image to `0xfff00000`;
6. download-success status;
7. jumping to the streamed main image.

A separately linked 68-byte Rust main image was streamed through this protocol,
copied to `0xfff00000`, and executed successfully. Observed values were:

```text
0x0900ff80 = 0x12345678  bootloader ready
0x0900ff94 = 0x00000000  download success
0x0900ff98 = 0x57534d31  main-image magic "WSM1"
0x0900ff9c = changing     main-image heartbeat
0xfff00000 = 0xe59fd000  first main-image instruction
```

This validates the complete host-to-bootloader-to-main-image execution chain.
The next firmware boundary is no longer image loading; it is the runtime HIF
queue and WSM startup indication.

### Reset-to-HIF startup flow

> Focused handoff: see
> [`xr819-hif-startup-flow.md`](xr819-hif-startup-flow.md) for the exact call
> order, primary disassembly excerpts, current Rust mismatches, remap values,
> and negative-test ledger.

A separate Ghidra project, `xr819-fw-thumb`, now contains the raw code section
loaded at address zero and imported explicitly as `ARM:LE:32:v8T`. This fixes
the missing/merged early Thumb functions seen in the mixed ARM/Thumb ELF
project and produces a clean decompilation of the HIF initializer at `0x0aca`.
Radare2 is used alongside it for precise mixed Thumb/ARM boundaries such as the
ARM routine embedded at `0x00016550`.

Current names in `xr819-fw-thumb` include:

```text
0x0000094c  hif_allocate_buffers_and_initialize
0x00000ab4  hif_activate_and_wait_ready
0x00000aca  hif_initialize
0x00000bb0  platform_dma_interrupt_initialize
0x00000c38  register_irq6_task
0x00000b88  register_irq21_task
0x000058b6  platform_clock_parameters_initialize
0x00016004  hif_publish_exception_descriptor
0x000164bc  firmware_main_initialize
0x00016550  drain_write_buffer_and_enable_interrupts
```

The 2018 firmware startup path is:

```text
0xfff00000  ARM reset entry
    write CP15 control = 0x00001f74
    branch to 0xfff00014
    initialize IRQ/SVC/ABT/UND/SYS stacks below 0x0400c000
    check/write warm-start marker 0xe3877000 at 0xfff00000
    clear 4 KiB below 0x0400c000
    BLX 0x000164bc

0x000164bc  firmware main initialization
    wait for 0x04001428 to become zero
    call 0x00015926                    currently a no-op
    update the 0x0ac80000 control block
    call 0x000009e0                    platform/RF preparation
    call 0x00000ba8 -> 0x00000a74      ROM/platform handshake
    call 0x00015b60                    diagnostic console setup
    call 0x00000c58 -> 0x00000bb0      interrupt/timer platform setup
    call 0x00000ab4                    activate HIF and wait ready
    call 0x00000c38                    register IRQ 6 task
    call 0x00000a88                    register IRQs 18 and 20
    call 0x00000b88                    register IRQ 21 task
    call 0x000009ac                    subsystem initialization
    initialize a fixed table at 0x0abb0010..0x0abb0030
    BLX  0x00016550                    drain write buffer; enable IRQ/FIQ
    call 0x0000f140                    enter scheduler/runtime
```

`0x00000c58` calls `0x00000bb0` immediately before HIF activation. Raw Thumb
analysis shows that it configures several platform blocks before registering
IRQ 4:

```text
0x0a980000[0x00]  clear 0x80, set 0x10
0x0a980000[0x08]  clear 0x10
0x0aa80040[0x08]  set 0x000000f0
0x0aa80040[0x00]  set 0x00f00000
0x0ac80040[0x18]  = 0x00040090
0x0aa80000[0x04]  = 0x00000200
0x04001fd8         = 1
0x0ac80080[0x20]  = 31
0x0ac80080[0x1c]  = 6
IRQ 4 handler      = 0x0000f039
```

It also conditionally sets `0x00120000` in `0x0ac800bc` based on a halfword at
`0x04001ff0`, then calls `0x000058b6`. The meanings are not fully named yet,
but this is a concrete platform/DMA/interrupt prerequisite and must be
understood before translating HIF activation.

`0x00000ab4` then performs another prerequisite which the first Rust HIF
experiment omitted:

```text
read  0x0ab00140
write 1 to 0x04009754                  HIF software mode
wait while 0x0ab00134 bit 10 is set    hardware startup busy
```

`0x000009ac` then calls the following initializers in order:

```text
0x0000094c  allocate four 384-byte output buffers and initialize HIF
0x000000f6  platform interrupt/peripheral initialization
0x00016d24  unknown subsystem initialization
0x0000014c  larger MAC/platform state initialization
0x000005a8  timer/state initialization
0x00000774  small fixed table/register initialization
0xfff0104c  high-memory firmware routine
0x00013a40  WSM/MAC initialization
0x00000594  allocate 30 objects
0x00000560  initialize 16 object records
0x00000c80  call high-level subsystem initializer
0x000158a8  construct and queue WSM_STARTUP_IND_ID
```

`0x000158a8` is the exact startup-indication constructor. It allocates 168
bytes with `0x0000e5f4`, constructs WSM ID `0x0801`, fills the 30 x 1632-byte
input-buffer geometry and firmware identity fields, copies the 128-byte build
label, and passes the message to `0x0000ed4c`.

The output path after construction is now bounded precisely:

```text
0x0000ed4c  enqueue message pointer
    preserve/set WSM transport bits
    increment pending-message count at 0x04009758
    set scheduler event bit 1 at 0x04001fd8 for the first pending message
    place the pointer in the 64-entry software queue at 0x040097fc
    increment software queue producer at 0x040098fc
    call 0x0000ec82 while fewer than four descriptors are outstanding

0x0000ec82  publish one hardware descriptor
    read the message pointer from the software queue
    insert the two-bit sequence at WSM ID bits 13..14
    write buffer_address & 0xf6ffffff to 0x0ab00100 + slot * 8
    write ((length + 1) & 0x1fff) | transport bits | 1 to descriptor control
    increment descriptor producer at 0x04009914
```

The Rust path already performs the same first-message descriptor publication.
One accounting error was found and fixed: it had advanced the software queue
consumer at `0x04009900` while publishing; the vendor code leaves that counter
for the later completion/reclaim path. This error was incorrect but did not
explain the first startup failure, because the descriptor itself was already
written before the bad counter update.

The clean decompilation of `0x00000aca` establishes the exact HIF ring setup:

- four firmware-to-host descriptors at `0x0ab00100`;
- 32 host-to-firmware descriptors at `0x0ab00000`;
- 30 host input buffers of 1632 bytes beginning at `0x09008a68`;
- four 384-byte output buffers beginning at `0x090149a8`;
- firmware counters and masks around `0x040098fc`;
- software HIF state beginning at `0x04009754`;
- IRQ 13 registered as the HIF interrupt;
- `0x0ab00128 = 0x7ff` to acknowledge/clear HIF interrupt state;
- `0x0ab00120 = 0x7ff`, or `0x7f7` only when software mode is already one
  and the pre-existing hardware control has bit 10 set;
- the firmware TX length mask is the selected control value AND `0x7f8`.

The `0x7f7` branch is therefore not a generic XR819 reset rule. It is a
state-dependent path. `0x00000ab4` sets software mode to one before
`0x00000aca`, so the value already present in `0x0ab00120` bit 10 determines
whether clean startup selects `0x7f7` or `0x7ff`.

Radare2 `pdf` and r2ghidra `pdg` are now used together: `pdf` is authoritative
for instruction mode, literal-pool values, and exact offsets, while `pdg`
provides control-flow pseudocode. This caught an earlier literal transcription
error: the value written to `0x0ac80058` is `0x00040090`, not `0x04009000`.
Standalone Ghidra remains useful for cross-checking functions whose mode and
boundaries were recognized correctly.

Radare2 analysis of the mixed-mode target identified `0x00016550` as five ARM
instructions embedded after the Thumb startup code:

```asm
mcr p15, 0, r0, c7, c10, 4    ; drain ARM write buffer
mrs r0, cpsr
bic r1, r0, #0xc0             ; clear IRQ and FIQ mask bits
msr cpsr_c, r1
bx  lr
```

This is the first concrete explanation for why correctly shaped descriptors
could remain invisible to the HIF engine. The reference firmware initializes
all packet buffers, descriptors, counters, and HIF controls, then explicitly
drains the ARM write buffer before enabling interrupts. The initial Rust code
looped immediately after publishing and omitted this synchronization point.
The Rust translation now includes the same CP15 write-buffer drain, but it does
not yet enable CPU interrupts because the HIF ISR has not been translated.

The normal output path queues a buffer in firmware RAM and writes an
address/control pair into one of the four descriptors. A separate emergency
path at `0x00016004` writes directly to `0x0ab0012c/0x0ab00130`; vendor firmware
uses this for WSM exception indication `0x0800`, not normal WSM traffic.

One halted working-vendor snapshot established these runtime values:

```text
0x0ab00100 = 0x0000a3e8   current output descriptor address
0x0ab00104 = 0x00000024   consumed/current descriptor control
0x0ab00120 = 0x000007ff   HIF control
0x0ab00124 = 0x00000000   interrupt status at capture
0x0ab00128 = 0x00000000   interrupt acknowledge
0x0ab0012c = 0x00000000   emergency address
0x0ab00130 = 0x00000000   emergency control
0x0ab00134 = 0x04034e00   runtime startup/status word
0x0ab00140 = 0x00000002   runtime activation state
0x0ab00000 = 0x0000ca28   current input descriptor address
```

This confirms `0x7ff` is a valid normal runtime control value and that the
activation register eventually reaches two. Halting and resuming vendor
firmware caused a fatal BH/scan state even without writes, so further live
vendor snapshots are not justified without a reboot-based plan.

The Rust translation now has a separate `platform` module covering the stable
hardware portions of:

```text
0x00015944  sample and relocate eight remap-window values
0x000009e0  reset/remap and interrupt-controller preparation
0x0000572a  0x0ac00000 peripheral initialization
0x000056d8  0x0a880000 interrupt-controller initialization
0x00000bb0  platform/DMA/clock setup, excluding untranslated IRQ callback
0x000058b6  clock parameter selection
0x00000ab4  bounded HIF activation and ready wait
0x000164c8  pre-platform update of 0x0ac800bc
0x000164fa  fixed 0x0abb0010..0x0abb0030 routing table
```

`pdf`/`pdg` comparison also corrected two register-map mistakes during the
translation: the remap-window array begins at `0x04001ff4` (state offset
`0x20`), and `0x00000bb0` writes `0x00f00000` to `0x0aa80040` while writing
`0x200` separately to `0x0aa80004`.

A short settling delay was added between the Rust downloader publishing
success and jumping to the main image. Without it, the main image changes the
platform/remap registers before Linux completes its first status poll, and the
host observes spurious bootloader status 8 instead of success.

With all translated setup above, the firmware downloads and runs but host
CONTROL still remains `0x3000`, with a zero next-message length. Selecting the
reference `0x7f7` control path, populating the software buffer-pointer tables,
and polling/acknowledging HIF status like IRQ 13 do not change the result. A
bounded experiment using the dedicated exception descriptor at
`0x0ab0012c/0x0ab00130` also produces no host-visible length. This separates
the failure from the normal four-entry output ring: the HIF engine itself has
not reached the vendor runtime state.

The packet-memory/DMA setup called immediately after HIF initialization was
also translated from `0x000000bc`, `0x0000f608`, `0x0000f4f4`, and
`0x0000fcfe`. It initializes the `0x09c0xxxx` engines, four `0x09c6xxxx`
channel records, and the `0x09016a28` hardware list before publishing startup.
A clean reboot test still produced `CONTROL = 0x3000`. Packet-DMA omission was
a reasonable hypothesis but is now a recorded negative result, not a path to
repeat unchanged. An early isolated experiment with the final `0x00016d24`
write setting bit 11 at `0x0ab80c00` also left CONTROL at `0x3000`; it is not
part of the current minimal path because the vendor first executes `0x16ac6`.

That prerequisite is now bounded more precisely. `0x16a38` initializes MAC
software state, `0x16ca4` derives timing values, and `0x171ce` performs the
hardware-producing tail. `0x171ce` initializes `0x0aba2000`, applies four
address/value lists via `0x171a6`, and invokes `0x17008`. The latter derives 22
calibration anchors from initialized-SRAM tables at `0x04000ca0` or
`0x04000d24`, expands them into the 80-word hardware table at `0x0ab80800`,
and updates `0x0ab80400/0x0ab80410`. Radare2 `pdf` and Ghidra decompilation
now agree on the full selection and packing algorithm. It has been translated
as a pure Rust generator with golden values and as an unsafe mode-zero hardware
routine applying the exact `0x171ce` constants and five register lists. The
routine is now called after packet-DMA preparation and sets MAC bit 11 only
after those effects. Probe and repeated empty scans remain stable. Halted AHB
verification observed `0x2b202b20` at `0x0ab80800`, boundary 55 at
`0x0ab80410`, and bit 11 set at `0x0ab80c00`. The fixed software-state portion of `0x16a38`, mode-zero pointer setup from
`0x198f2`, and signed remap timing derivation from `0x16ca4` are now active
before the hardware routine. The signed second field derived from remap window
`0x04118000` is `-852`, stored as wrapped `u16`; treating its sign bit as an
unsigned field would incorrectly produce `9940`. Repeated target scans remain
stable with this state enabled.

Software channel policy remains inactive. IRQ 6 now faithfully sets
platform-event bit 27 at `0x04001fd4`,
matching vendor callback `0x0000f1fe`; IRQs 18, 20, and 21 remain diagnostic
stubs pending their encoder/MIC queue completion translations.

Parsing the vendor firmware container clarified its memory layout:

```text
copy  0xfff00000  0x001a34   ARM bootstrap and high support routines
fill  0xfff01a34  0x001d24   zero
fill  0xfff03758  0x0108a8   zero
copy  0x00000000  0x01b37c   main Thumb firmware
copy  0x04000000  0x0010f8   initialized SRAM data
copy  0x040010f8  0x000f80   initialized SRAM data
fill  0x04002078  0x007bcc   firmware BSS
```

The Rust loader had previously copied one flat image only to `0xfff00000`.
Low-address Thumb linker/loader variants now reproduce the main-code placement,
and Rust explicitly clears the vendor BSS range plus initializes the platform
state used by translated startup. The flat binary also omits Rust's own
`.bss`; linker-defined `__bss_start..__bss_end` clearing is now performed at
main entry. Without it, retained SRAM made the first parsed scan appear busy
and could silently corrupt configuration and HIF scratch statics. This is
architecturally closer to the vendor image, although it does not yet make HIF
visible to the host. Both the original
high-address ARM main and the low-address Thumb main have now produced the same
zero next-message length; changing execution address or instruction mode alone
is not the missing HIF step.

Live indirect APB and AHB reads while the device remains in queue mode still
fail with `Prefetch bit is not cleared`. A debugfs halt permits bounded reads,
but resuming the new MAC-initialized path lost a later HIF command interrupt
and killed the BH; MMC rebind recovered. Halt should therefore be treated as a
postmortem operation, not a transparent pause/resume mechanism. This was reconfirmed for HIF MMIO
addresses `0x0ab00104`, `0x0ab00120`, `0x0ab00134`, and `0x0ab00140`; their
returned zeros are unusable.

A safe postmortem variant now works for SRAM postcodes. After startup timeout,
the driver first stops the BH, asserts XR819 CPU reset, restores ACCESS mode,
waits 30 ms, and only then uses the AHB bridge. CPU reset preserves packet
SRAM, so values written by Rust at `0x0900ff98` can be recovered without a live
halt. This converts the previously unreliable timeout diagnostics into a
bounded execution trace while also leaving the CPU stopped.

The first traces established that execution is not merely reaching the custom
main entry:

```text
PLT2/DMC3  reached translated platform/clock setup on cold boots
HIF0       reached the call to Transport::initialize on a warm retry
HIN6       completed HIF ring/control initialization
PDM0       entered translated packet-DMA setup and faulted inside it
RTE1       after removing the out-of-order packet-DMA/MAC calls, completed
           HIF initialization and interrupt routing before message encoding
```

This also exposed a sequencing error in the experiment: packet-DMA and MAC
initialization belong later in `0x000009ac`; calling them before startup
publication can fault and was removed from the minimal HIF image. Cold and
warm traces currently diverge around the write sequence following DMC3, so the
remaining pre-activation platform setup must be made deterministic before a
missing postcode is interpreted as an HIF failure.

The reference high bootstrap's exact CP15 control value `0x1f74` originally
appeared ineffective when the only observation was `CONTROL = 0x3000`, so it
was reverted. SRAM postcodes supplied new binary-grounded evidence: without
that value, cold execution repeatedly stopped immediately after DMC3, at the
vendor write `0x0aa80004 = 0x200`. Restoring `0x1f74` allowed a reset/rebind run
to continue through HIF initialization and routing (`RTE1`). The CP15 value is
therefore required for faithful execution across the clock/reset transition,
even though it is not by itself sufficient to activate HIF.

Granular packet-DMA postcodes then narrowed the next fault:

```text
PDM1  completed the register writes translated from 0x000000bc
PDM2  completed the 0x09c00e00 setup at 0x0000f608
PDM22 completed the four channel records at 0x09c60000/0x09c60100
      next CPU write to the vendor list at 0x09016a28 did not complete
```

The `0x09016a28` list and the normal `0x090149a8` startup buffer are both in the
same currently inaccessible high packet-memory window. Granular traces reached
`PDM22` after all four `0x09c60000/0x09c60100` channel records, then stopped on
the first `0x09016a28` write. Repeated partial packet-DMA runs can destabilize
SDIO and are not comparable without resetting the SDIO host/power sequence.

A patchable one-word diagnostic captured all eight cold hardware remap windows
through the only consistently retained postcode word:

```text
window 0 = 0x00b140d4
window 1 = 0x04100000
window 2 = 0x04118000
window 3 = 0xf08b400d
window 4 = 0xd08f4196
window 5 = 0x00000081
window 6 = 0x0ec2e284
window 7 = 0x00000827
```

`FUN_00015944` stores windows 0..2 and 6..7 unchanged, while adding `0x166`
4-KiB pages to the three packed address fields in windows 3..5. The resulting
stored values are `0xf0a1a00d`, `0x80a5a196`, and `0x00000083`.

Prefix tests also corrected two ordering assumptions. The fixed routing table
at `0x0abb0010..0x0abb0030` is written only after `FUN_000009ac` returns, so it
must not precede startup construction; the first routing write itself blocked
the minimal path. Conversely, packet-DMA initialization really does occur
after HIF ring setup but before `FUN_000158a8`, so it cannot simply be removed
from a faithful final startup path even though it remains destructive in the
incomplete translation.

With the clock transition and packet-DMA disabled, staged execution reaches all
of `Transport::initialize`. The last retained code is `HIN6`, immediately after
writing `0x0ab00120`; later writes no longer appear in the original postcode
window. Removing the incorrectly early routing write lets execution progress
far enough to destabilize SDIO while accessing/publishing the temporary
`0x0900fe00` message buffer.

The apparent clock/remap and packet-memory failures were subsequently resolved.
The low Thumb main had incorrectly reset SP to `0xfff1ff00`. Vendor `0xbb0`
writes `0x0aa80004 = 0x200`, after which that stack window is unavailable; the
next Rust call therefore stopped even though the transition itself was correct.
Moving SP to the vendor low-SRAM stack region at `0x0400c000` produced these
results:

```text
STG6       exact clock transition and HIF initialization completed
0x504d0003 write/readback passed at both 0x090149a8 and 0x09016a28
STG8       translated packet-DMA setup and 0x09016a28 list completed
```

Using the vendor TX buffer at `0x090149a8` then delivered the first startup
indication to Linux:

```text
CW1200 WSM init done.
Input buffers: 30 x 1632 bytes
Hardware: 7.9
WSM firmware [XR819 open Rust WSM], ver: 1, build: 1, api: 1, cap: 0x0003
```

A polling translation of the RX descriptor ring plus generic write-MIB and
structured configuration confirmations now lets Linux complete probe and
register a PHY and `wlan0` without a stuck command. The full SDD/DPD payload is
validated and retained in firmware memory. Start-scan is now parsed into exact
fixed, channel, and SSID records and copied into persistent fixed storage before
the HIF RX descriptor is recycled. It also reproduces `0x13d44`'s 34-channel
hardware limit, nonzero/ordered dwell checks, probe-delay bound, active byte,
event bit 10, and vendor busy status. Firmware work is serviced before the next
host command, producing a response followed by an empty scan-complete
indication. Its encoder now matches vendor `0x111ba`'s 12-byte wire length,
including a zeroed trailing 16-bit vendor field. Two consecutive target scans complete without timeout while real
channel tuning and RX remain absent.

### Reboot-free firmware iteration

A failed probe leaves the chip in queue mode. Rebinding originally failed with
`Device is already in QUEUE mode!`. The XR819 firmware loader now handles this
state by setting `CPU_RESET` and `ACCESS_MODE` in CONFIG, waiting 30 ms, and
then running the normal bootloader download path. This was verified repeatedly
without rebooting Linux or the board.

After replacing the firmware files, another iteration is started with:

```sh
echo mmc1:0001:1 > /sys/bus/sdio/drivers/cw1200_wlan_sdio/bind
```

The sysfs write returns the probe timeout for a failing image, which is
expected. Restoring the vendor files and issuing the same bind successfully
loads vendor firmware after ordinary failed images. This is the preferred
firmware-only loop after a failed probe. Firmware that changes CP15/cache state
or initializes the `0x09c0xxxx` packet-DMA/platform engines can survive the
CONFIG CPU reset sufficiently to prevent the next bootloader handshake; those
experiments still require a board reboot. Reloading kernel
modules while an operational vendor firmware/BH is active can also leave stale
IRQ/work state, so module replacement and fatal BH states use a reboot.

The module-unload debugfs crash was also identified: the wiphy debugfs parent
was removed before the driver's child directory. The driver now removes its
child directory before `ieee80211_unregister_hw()`.

## Reverse-engineering priorities

1. Trace `FUN_00016eec` to the exact WSM START/JOIN entry points and assign its
   synchronization mode values.
2. Locate the consumers of the static address/value tables at `0x1d948`,
   `0x1dc30` and `0x1ee80`.
3. Identify the hardware measurement primitives called by `FUN_00019680` and
   `FUN_0001a3a8`.
4. Catalogue each access in the `0x0ab8xxxx` through `0x0abcxxxx` ranges and
   distinguish register addresses from false-positive data words.
5. Separate initialization-only writes from per-channel and per-packet writes.
6. Identify which runtime tables are copied or derived from SDD elements.
7. Map status bits and polling loops in `FUN_0001b9ee`.
8. Compare the 2018 and 2016 implementations of each calibration routine.
9. Build a host-side register snapshot facility before attempting replacement
   firmware.

## Ghidra names

The following names have now been applied in the `xr819-fw-diff` Ghidra project:

| Address | Name |
| --- | --- |
| `0x1814a` | `phy_configure_channel` |
| `0x1856a` | `phy_apply_channel_if_changed` |
| `0x18836` | `phy_mode_channel_request` |
| `0x192e4` | `phy_measure_set_selector` |
| `0x195d0` | `phy_measure_read_pair` |
| `0x19652` | `phy_measure_set_excitation` |
| `0x19680` | `phy_calibrate_multipoint` |
| `0x19ee0` | `phy_correlate_64_samples` |
| `0x1a3a8` | `phy_calibrate_iq_candidate` |
| `0x1b868` | `phy_write_coefficient_pair` |
| `0x1b9ee` | `phy_run_measurement_calibration` |
| `0x1bc5c` | `phy_save_and_enter_measurement_mode` |

The `phy_calibrate_iq_candidate` name deliberately retains a confidence marker:
the waveform correlation and four-component corrections strongly suggest I/Q
or related amplitude/phase calibration, but the exact analog impairment has not
yet been proven.
