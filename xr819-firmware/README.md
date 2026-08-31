# XR819 open firmware experiment

This directory is an experimental `no_std` Rust implementation of firmware for
the XRadio XR819. It is expected to become a separate repository if hardware
bring-up succeeds.

The host protocol is the XR819-native extension of the CW1200 WSM ABI used by
the Linux driver. XR819-specific PHY, calibration and host-interface code stays
behind that protocol boundary.

## Current state

The current `hif-startup` binary delivers a CW1200 startup indication, completes
Linux probe, performs calibrated multi-channel active scans, transmits retained
probe-request templates, returns real beacon/probe-response indications, joins
a WPA2 network, completes the four-way handshake, and carries sustained TCP/UDP
traffic with hardware CCMP.

The intermittent terminal RX/MAC collapse was traced to Rust RX ownership logic,
not AES or a required FIQ path. A corrupt RX release-head ownership word was
marked pending and then returned without advancing the head; no later owner
could revisit it. Continuing through normal head reclamation fixes the terminal
stall. RX resynchronization also defers its hardware-consumer update while
zero-copy HIF slots remain host-owned.

The command-extracted production image qualified healthy in three fresh boots
against the `Hoeve Luitenant Halleux` AP at 3.28-3.63 Mbit/s TCP and
4.81-5.47 Mbit/s UDP delivered, with 20/20 final ping, every request buffer
released, BH alive, WSM idle, and no fatal error or exception. The exact packed
image is `4533b546e3147f787e4583bf177783cd4ea2b4bac1e9914289692fc4212b858d`.
The preceding qualified image
`88298e828ef298af0644a4afe0dd0c206849061bc6b501f800bfdb5ca29f4a53`
reached 4.87-5.09 Mbit/s TCP and 7.92-7.93 Mbit/s UDP on the former AP; those
figures are retained as a different-radio-environment baseline rather than a
direct throughput comparison.
The complete synchronous FIQ experiment is preserved separately on Jujutsu
bookmark `feature/mac-fiq`; FIQ is not required for the production RX fix.
The exact vendor call order, Radare2 excerpts, current implementation delta,
and experiment ledger are in
[`../xr819-hif-startup-flow.md`](../xr819-hif-startup-flow.md). Salvaged
semantic names, structure layouts, HIF/IRQ behavior, and RF algorithms from an
external annotated Ghidra archive are summarized in
[`../xr819-annotated-re-code.md`](../xr819-annotated-re-code.md). Complete
address-labelled decompiler exports are available under
[`../xr819-decompilation/`](../xr819-decompilation/).

## TX publication execution bisect

Set `XR819_TX_BISECT_STAGE` while building `hif-startup` to stop normal
management-frame publication at one controlled boundary and return a failed
WSM TX confirmation instead of touching later hardware state. The confirmation
retains the WSM packet ID and reports the selected stage in `ack_failures`, so
Linux's ordinary TX-confirm debug logging is sufficient to observe it.

The boundaries are:

1. TX request parsed, before context preparation;
2. context and command storage prepared, before publication;
3. pipe and hardware-ring validation complete;
4. frame ownership and timestamp initialized;
5. PAS command built, software slot selected, and hardware GO cleared;
6. EDCA timing published;
7. quantum published, immediately before `PIPE_IRQ_TRIGGER`;
8. immediately after `PIPE_IRQ_TRIGGER`.

For example:

```sh
XR819_TX_BISECT_STAGE=1 cargo build --release --bin hif-startup \
  --target thumbv5te-none-eabi -Z build-std=core
```

Stage zero or an unset variable preserves normal behavior. Start at stage 1
and advance until the confirmation disappears; the first missing confirmation
identifies the operation range that stops ordinary HIF progress.

`XR819_TX_BISECT_SUBTYPE` selects the frame class: 0 through 15 select an IEEE
802.11 subtype, 253 selects protected data, 254 selects non-authentication
frames, and 255 selects all frames. Protected-data stage 8 returns a clean
failed confirmation, proving that its current stall begins only after final
hardware-ring activation.

## WPA, protected data, and crypto status

The STA path now performs authentication, association, immediate ACK response,
EAPOL RX/TX, and WPA2 PTK/GTK negotiation with the unmodified Linux `cw1200`
driver. Reinstalling immediate-response descriptors during JOIN fixed the ACK
slot-21 pointer, and host TX accepts payload-bearing unicast data while rejecting
null/QoS-null frames that previously killed the TX path.

WSM `ADD_KEY` and `REMOVE_KEY` are implemented for AES pairwise and group keys.
ARM firmware uses the XR819 AES engine to fill host-reserved CCMP IV/MIC space
on TX and authenticate/decrypt packet-DMA frames on RX. Allocation-free
RustCrypto AES-CCM remains the host-test backend and independent oracle. The
host round-trip test and Python `cryptography` AESCCM known-answer vector pass,
including exact ciphertext/MIC and
corrupted-ciphertext/MIC rejection. Linux no longer reports failed `0x000c`
key installation. The bounded vendor host-TX path now completes protected
station traffic end to end: DHCP obtains a lease, gateway and Internet pings
succeed, and an HTTP request completes with every queue and HIF buffer returned.
Host-selected rate control is hardware-validated across CCK, OFDM, and HT. The
vendor no-protection `bHwRateCode = 0xff` rule fixed legacy OFDM publication,
and the mixed-mode HT PHY control word now carries the vendor airtime field.
Forced endpoint tests passed at rates 0–3, 6, 13, 14 (MCS0), and 21 (MCS7),
followed by a successful host-selected connectivity run. The firmware now also accepts the XR819 host driver's 24-nibble MIB `0x1016`
retry policies and walks them per frame. A rate change recomputes PAS timing and
rebuilds the PHY descriptor before rearm; firmware does not run a competing
adaptive rate-selection algorithm. Hardware validation observed an MCS7 failure followed by successful MCS2
completion: `rate_try[2] = 0x00100000`, final rate 16, and one ACK failure. This
confirms that the host series drives hardware fallback and that the extended
confirmation reports the failed MCS7 attempt.

Firmware always advertises `XR819 open Rust native`, accepts the native
four-byte operational-mode MIB, uses synchronous JOIN semantics, and emits
XR819 confirmations with three packed per-rate failure words. There is no
compile-time wire-profile selection.

The ordinary vendor path is now specified end-to-end in
[`../xr819-vendor-host-tx-lifecycle.md`](../xr819-vendor-host-tx-lifecycle.md).
Implementation has moved away from class-6 context copying: `vendor_host_tx.rs`
models exact host-context initialization, mode-0 pending-list append, PAS-ring
compaction/insertion, and pending-task outcomes. HIF requests now carry an
explicit packet-RAM release token. The normal feature-free firmware admits
ordinary non-EAPOL data into a real host-pool context and retains the original
request token. It now applies vendor-shaped header
classification, per-link/TID sequence assignment, target hardware CCMP, VIF-slot
selection, PAS timing, descriptor construction, ownership bit `0x20`, and
mode-0 pending-list insertion. RESET now unlinks a queued context before freeing
it and returning the retained HIF request. Live pending-task service now applies
VIF/link, expiry, TBTT, and power-save gates; rejected class-0 frames are
confirmed before their HIF token is returned, while eligible frames enter the
compacting global PAS ring with ownership bit `0x40`. The implementation performs reversible non-aggregate scheduler selection,
AC-to-pipe mapping, PAS-slot removal, pipe-slot reservation, and kind-0 descriptor
generation before crossing into hardware ownership. It starts the PHY, triggers
the MAC, services retries/completion, emits the class-0 WSM confirmation, and
frees the context/HIF request only after confirmation publication. Management
and class-0 servicing are serialized while either owns the shared MAC runtime.
`HostTxDriver` now represents idle, retained, scheduler-reserved, and confirming
ownership as mutually exclusive states, including RESET cancellation. Detached
HIF requests use an owning `RequestBuffer`, so payload borrows cannot outlive
the packet-RAM owner, and a single non-copyable `MacEventQueue` capability is
passed to every MAC-event consumer. This remains a bounded single-outstanding
non-aggregate implementation, not yet the full production scheduler.

The optional `vendor-host-tx-diagnostics` feature retains the bring-up
observability without burdening the normal station image. It enables retained stage/frame/descriptor snapshots, HIF request counters,
bounded WSM debug events, and the latest native retry-feedback words through
the existing counters MIB. Without the feature, trace writers
compile to no-ops and the normal counters layout is preserved. Fatal MAC
exceptions remain available independently because they are part of terminal
recovery diagnostics rather than the verbose host-TX trace stream.

The vendor AES accelerator is mapped at `0x09c5_0000`. Ordinary target CCMP
uses transfer classes 6/7, commands `0x1100`, `0x1240`, `0x1402/0x1403`, and
`0x3008_1008/0x3008_1009`, with completion through registered IRQ 18 or 20
callbacks. RustCrypto is retained only for host execution and cross-checking.
Exact engine findings and known-answer/IRQ evidence are in
[`../xr819-aes-engine.md`](../xr819-aes-engine.md).

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

Not yet implemented or production-complete:

- active-VIF channel restoration and complete power-save resumption;
- an explicit, pressure-safe HIF publication and credit-ordering contract for
  the existing 64-entry software queue and four hardware descriptors;
- deterministic initialization and retention contracts for every live DTCM field
  (direct literals and direct field-root expressions are already closed; older
  family-local alias arithmetic remains under audit);
- controlled degraded-RF fallback and long-duration soak qualification;
- multiple hardware-owned TX frames, then aggregation;
- CCMP replay protection;
- production recovery policy for architected CPU exceptions and unrecoverable
  packet-controller faults.

## Historical bring-up order (completed)

This sequence records the original bring-up plan; it is not the current backlog.

1. Replace the mailbox main image with a minimal HIF/WSM transport loop.
2. Send a CW1200-compatible startup indication.
3. Accept configuration and reset commands.
4. Apply the SDD and static PHY initialization tables.
5. Implement channel setup and read-only RX.
6. Implement TX and confirmations.

Run the host tests, ARM build, call-chain stack check, and sectioned-bootstrap
check with:

```sh
./tools/check.sh
```

Operational station behavior is feature-free; Cargo features are reserved for
diagnostics. Build the packed production image with:

```sh
./tools/build-ota-image.sh firmware.bin
```

### Vendor-style sectioned images

The matching vendor firmware is a compact section stream, not a flat image: it
loads ARM support code at `0xfff00000`, Thumb code at `0x00000000`, initialized
data at `0x04000000`, and explicitly zero-fills later DTCM ranges. The exact
matching-container map and corrected boundaries are documented in
[`vendor-container-layout.md`](vendor-container-layout.md).

Inspect a vendor container and package one coherent Rust ELF's `PT_LOAD`
segments with:

```sh
python3 tools/inspect-vendor-container.py fw_xr819.bin
python3 tools/pack-sectioned-elf.py firmware.elf firmware.sections
```

`download-boot-sectioned` loads the packed copy/fill stream. Build its raw ARM
bootstrap with:

```sh
./tools/build-sectioned-bootloader.sh boot_xr819.bin
```

The bootstrap starts at `0x08000000`, relocates its body to `0x09010000`, and
uses the reserved DTCM stack at `0x0400c000`. The main image contains no native
DTCM payload, so section loading cannot overlap active bootstrap frames. Moving
this stack into staging SRAM was tested separately and rejected after repeated
TX failures and a lost final ping.

### Split high-SRAM extensions

The validated low Thumb startup occupies exactly `0x7500` downloaded bytes and
is timing/layout sensitive. `download-boot-low` preserves that low image while
copying later payload bytes to executable SRAM at `0xfff00000`. It installs a
low Thumb/ARM veneer at `0x00009720` after vendor SRAM initialization.

Build and package an extension with:

```sh
cargo +nightly build --release --bin hif-extension-probe \
    --target armv5te-none-eabi -Z build-std=core
llvm-objcopy -O binary \
    target/armv5te-none-eabi/release/hif-extension-probe extension.bin
./tools/build-split-extension.py stable.bin extension.bin combined.bin
```

The packaging tool verifies the stable image hash and original startup call
before applying the four-byte call redirection. It must not be used with an
arbitrary low image.

### Packet-RAM ownership

The production image places ordinary Rust-owned packet objects sequentially in
one `.packet_ram.runtime` NOBITS linker region. Ordered input subsections make
the linker, rather than per-object address directives, assign each object's
address. Retained LMC state remains outside the region. The RX FIFO stays in its
separate `0x094` hardware window and keeps a `0x7000` logical cursor over
`0x8000` of linear backing so crossing slots remain directly publishable without
a copy buffer.

`src/packet_ram.rs` is the sole Rust ownership map for runtime `0x090`/`0x094`
storage. Consumers derive raw device addresses directly from its aligned Rust
statics; only retained LMC objects remain linker anchors. Genuine
packet-controller MMIO identities are centralized in `platform.rs`. The exact
pre-transition control is clean b6 image
`39b516dc65c1330ee67a12f8d17059e72aea7b07adf5aaac7043f080e0fdc47b`.

The normal gate builds and packs a real production ELF, verifies every packet
section is `SHT_NOBITS | SHF_ALLOC` and outside all `PT_LOAD` segments, rejects
packed destinations anywhere in the `0x09` window, checks full and low-23-bit
source-literal ownership, retains ordinary BSS fill records, and links the
sectioned bootstrap with its relocated image and BSS bounded to the documented
HIF-input overlay:

```sh
XR819_B6_ELF=/path/to/archived-b6-hif-startup.elf ./tools/check.sh
```

The baseline argument enables the normalized disassembly/MMIO-order gate. The
standalone layout and packer regressions are:

```sh
python3 tools/check-packet-ram-layout.py target/thumbv5te-none-eabi/release/hif-startup
python3 tools/test-pack-sectioned-elf.py
```

### TCM layout checks

The low Thumb image keeps `.text`, `.rodata`, ordinary `.data`, and ordinary
`.bss` in the writable ITCM mapping. The `0x1c000` linker bound is a conservative
observed envelope, not a proven physical capacity: matching vendor ITCM content
ends at the file/address split `0x1b3dc`, rounded up to the next 4 KiB boundary.

`link-main-low.x` now divides observed DTCM ownership explicitly:

```text
0x04000000..0x04009c44  .dtcm.bss    explicitly zeroed shared state
0x04009c44..0x0400a000  .dtcm.noinit retained research quarantine
0x0400a000..0x0400a500  five 256-byte exception-mode stacks
0x0400a500..0x0400c000  6,912-byte system stack
```

Both DTCM sections remain `NOLOAD`. Rust startup clears the initialized prefix
and runtime suffix of the combined `.dtcm.bss` allocation through
linker-exported boundaries in the qualified ascending volatile-word order, then
reconstructs every initialized value it consumes. There is no remaining vendor
COPY dependency and no DTCM `PT_LOAD`, COPY, or FILL record. Every top-level
field of `InitializedDtcmPrefix` is represented by an ordered `.dtcm.bss.initialized.*`
input object, every top-level runtime family is its own `.dtcm.bss` input object,
and the final
956-byte research margin has an explicit typed `.dtcm.noinit` retention object.
There is no remaining catch-all target byte allocation. Individual family types
still contain deliberate `OpaqueBytes` fields where semantics or ownership
remain unknown. The complete `InitializedDtcmPrefix` and `DtcmLayout` remain
host layout oracles rather than target allocations.

The linker concatenates runtime, scheduler, configuration, PAS, VIF, host-TX,
command, LMC, BA, completion, internal-context, power-save, HIF, MIC, and PHY
objects into the single exact `.dtcm.bss` output section. This creates ordinary
Rust allocation identities without changing any external address or implying
exclusive ownership. Existing internal-pool linker symbols derive from the typed
`InternalContextPoolState`; no family creates a standalone output section.

CP15 `c0,c0,2` reports `0x001c0200`, whose standard fields describe 128 KiB
ITCM and 64 KiB DTCM. That physical-size report does not provide another
independently addressable region here: a save/write/read/restore probe at
`0x0400c000` read its pattern back but also changed `0x04000000`, proving that
the address aliases live legacy DTCM. The stack top therefore remains the hard
usable-DTCM boundary; linker or loader sections must not target
`0x0400c000..0x04010000`.

Rust-owned CPU state now uses ordinary writable ITCM `.data` and `.bss`. The
new DTCM section names describe fixed external startup contracts, not permission
to move ordinary CPU-only state back into DTCM. This
includes HIF queue/ring ownership, `Transport`, response scratch, HIF sequence
state, the completed-frame FIFO, probe-context sequence, PAS accounting,
internal-context count, retry PRNG state, channel PLL cache, and channel power
limits. The native `.dtcm.bss` section deliberately remains `NOLOAD`; its qualified
explicit zeroing paths walk `__dtcm_data_start..__dtcm_data_end` and
`__dtcm_bss_start..__dtcm_bss_end` directly in ascending volatile words rather
than reconstructing either subrange from integer addresses. There is still no
main-image DTCM fill record.
Hardware descriptors, packet buffers, and MMIO identities remain in shared
packet RAM or MMIO rather than TCM. CPU-form packet-RAM pointers, MAC offsets,
DMA bus addresses, and command encodings have separate ownership rules recorded
in [`packet-ram-address-contract.md`](packet-ram-address-contract.md). The
internal TX context pool is an exact
typed member view of the shared quarantine object, not a standalone allocation
or exclusive ownership claim; retained teardown and diagnostic code still
addresses that fixed identity.

`tools/check-dtcm-access.py` freezes direct production DTCM literals outside
`src/dtcm.rs` in an exact reviewed manifest. The manifest is now empty: live
CPU accesses and hardware-published DTCM addresses use field-derived owners,
while unrelated control bit 26 is expressed as a bit rather than an
address-looking value. The same gate rejects direct arithmetic on a
`dtcm::...get()` expression, including arithmetic after a numeric cast; new
indexed and interior accesses must be derived by `src/dtcm.rs`. Production
DTCM root/interior arithmetic outside the owner is eliminated; the gate also
rejects integer-root constants and arithmetic through local aliases. The
current startup writer matrix,
access rules, and
cold/warm snapshot procedure are
recorded in [`dtcm-runtime-contract.md`](dtcm-runtime-contract.md). The optional
`dtcm-contract-diagnostics` image captures entry, post-platform, and
post-startup copies of the initialized `0x2078` bytes and publishes them through
paginated private read-MIB responses. It also reports the firmware-internal warm
contract comparison through the ordinary startup indication, avoiding MMIO and
post-startup WSM diagnostics. The entry snapshot is now taken after Rust zeroes the initialized prefix, so
diagnostic transitions validate reconstruction from the production baseline
rather than preserving loader bytes. Feature-free firmware has no snapshot
storage, diagnostic MIB path, or startup-label report.

The current linked ITCM image ends at `0x000142e8`, leaving about 31 KiB below
the conservative `0x0001c000` observed envelope. Further decoded CPU-only state
should use Rust globals in ITCM while the lower DTCM quarantine is reduced one
coherent family at a time.

ARM builds emit LLVM stack-size metadata. `tools/check-rust-main-stack.py`
combines it with direct-call edges from the linked disassembly. The deepest
normal chain runs through channel activation and dynamic IQ calibration and
uses 2,892 of 6,912 bytes. The checker also includes the 56-byte exception
veneer and validates the terminal exception chain at 224 of each 256-byte mode
stack. Handwritten assembly helpers use a reported disassembly-prologue
fallback; reachable recursion or indirect calls fail analysis rather than being
silently ignored.

An explicit `tcm-size-diagnostic` feature adds ARM interworking helpers for the
CP15 TCM type and region registers. The registers are read only when the host
requests diagnostic MIB `0x100c`; normal startup remains unchanged. This is a
destructive research feature: the XR819 core may not implement the newer TCM
type-register format, so use it only before a planned power cycle.

## Fast hardware iteration

A failed experimental startup leaves XR819 in queue mode. The driver can
recover ordinary failures by asserting embedded CPU reset and restoring direct
access mode. Embedded CPU reset does not reset XR819's retained HIF engine or
descriptor ownership, so the SDIO probe now calls `mmc_hw_reset()` before every
XR819 firmware load. This power-cycles and reinitializes the single-function
SDIO card without rebooting the board, making quiesced function-level
`mmc1:0001:1` unbind/bind cycles equivalent to cold HIF startup. Stop
`wpa_supplicant` and remove the interface from active association before
unbinding: forcing driver removal while mac80211 is still issuing teardown WSM
commands can leave the remove path in uninterruptible sleep and still requires a
board power cycle.

Use `tools/reload-target.sh BOOT_IMAGE MAIN_IMAGE` for routine replacement. It
stops association owners, finds and lowers the XR819 interface even when its PHY
was moved into a network namespace, and only then performs the SDIO unbind/bind.
For an already-unbound failed probe, a direct retry remains:

```sh
echo mmc1:0001:1 > /sys/bus/sdio/drivers/cw1200_wlan_sdio/bind
```

The bind write returns the probe error when experimental firmware times out, but the
new image was still downloaded and executed. Replace the firmware and repeat.
This fast path is intended for firmware-only iterations after a failed probe.
A reboot remains required after experiments that alter persistent CP15/cache
state or initialize the packet-DMA/platform engines, and is recommended after
replacing kernel modules or a fatal BH/IRQ state.

Warm rebind qualification completed three consecutive quiesced function-level
unbind/bind cycles, followed by WPA2 association, a 20-second 4.86 Mbit/s TCP
run, 20/20 ping, and zero used buffers. The reload helper was then qualified
both from an idle interface and from an active WPA2 association whose PHY had
been moved into a network namespace; it stopped association userspace, lowered
the interface, and returned with fresh `wlan0`/`phy3` registration without a
stuck remove task. Post-reload WPA2 traffic reached 5.67 Mbit/s over ten seconds,
completed 10/10 ping, and drained to zero used buffers. Postmortem halts and experiments that
change MMC host or persistent CP15 state may still require a board power cycle;
avoid debugfs halt during active bring-up.
