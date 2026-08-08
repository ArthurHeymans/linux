# XR819 HIF startup flow: disassembly notebook and implementation audit

This file is the focused handoff document for reconstructing XR819 firmware
startup through the first `WSM_STARTUP_IND_ID`. It records the authoritative
vendor order, the Radare2 evidence behind that order, the current Rust delta,
and failed experiments that must not be repeated without new evidence.

The broader PHY/calibration investigation remains in
[`xr819-firmware-reverse-engineering.md`](xr819-firmware-reverse-engineering.md).

## Source images and analysis setup

Primary 2018 images:

```text
/tmp/xr819-analysis/fw_xr819.bin   vendor firmware container
/tmp/xr819-analysis/fw-first.bin   0xfff00000 high bootstrap, 0x1a34 bytes
/tmp/fwcode.bin                    low Thumb code loaded at 0x00000000
```

Radare2 commands used for the excerpts below:

```sh
# Low Thumb firmware
r2 -q -a arm -b 16 -m 0 /tmp/fwcode.bin

# High bootstrap, mostly ARM initially and Thumb near 0xfff019aa
r2 -q -a arm -b 16 -m 0xfff00000 /tmp/xr819-analysis/fw-first.bin
```

Use `pdf`/`pd` for exact literals, mode, offsets, and call order. Use r2ghidra
`pdg` only as a control-flow aid. The routine at `0x00016550` is ARM code
embedded after Thumb code and must be decoded in ARM mode.

## Authoritative top-level order

The vendor startup order is fixed by `0x000164bc`:

```text
wait until initialized-SRAM word 0x04001428 is zero
0x00015926                         no-op in this image
main control update at 0x0ac800bc
0x000009e0                         remap/platform preparation
0x00000ba8 -> 0x00000a74           high-bootstrap platform/IRQ setup
0x00015b60                         diagnostic-console setup
0x00000c58 -> 0x00000bb0           DMA/clock setup and IRQ 4 registration
0x00000ab4                         HIF activation/wait
0x00000c38                         IRQ 6 registration
0x00000a88                         IRQ 18 and IRQ 20 registration
0x00000b88                         IRQ 21 registration
0x000009ac                         complete subsystem initialization
routing table at 0x0abb0010..30
0x00016550                         write-buffer drain; IRQ/FIQ enable
0x0000f140                         scheduler/runtime entry
```

The critical implication is that startup construction occurs *inside*
`0x000009ac`. Packet-DMA and MAC initialization precede it. The routing table,
global barrier, CPU interrupt enable, and scheduler follow it.

### Radare2 evidence: `0x000164bc`

```asm
0x000164bc  ldr r1, [0x00016544]       ; 0x04001428
0x000164be  ldr r0, [r1]
0x000164c0  cmp r0, 0
0x000164c2  bne 0x164be
0x000164c4  bl  0x15926
0x000164c8  ldr r2, [0x00016548]       ; 0x0ac80080
...
0x000164d4  str r0, [r2, 0x3c]         ; update 0x0ac800bc
0x000164d6  bl  0x9e0
0x000164da  bl  0xba8
0x000164de  bl  0x15b60
0x000164e2  bl  0xc58
0x000164e6  bl  0xab4
0x000164ea  bl  0xc38
0x000164ee  bl  0xa88
0x000164f2  bl  0xb88
0x000164f6  bl  0x9ac
0x000164fa  ldr r0, [0x0001654c]       ; 0x0abb0000
0x000164fe  str r1, [r0, 0x10]         ; route 0 = 0
...
0x00016524  str r1, [r0, 0x30]         ; final route = 0x12c
0x00016526  blx 0x16550
0x0001652a  bl  0xf140
```

Routing values derived from the exact instructions:

```text
0x0abb0010 = 0x000
0x0abb0014 = 0x3fd
0x0abb0018 = 0x3fa
0x0abb001c = 0x3ff
0x0abb0020 = 0x01b
0x0abb0024 = 0x05b
0x0abb0028 = 0x0b6
0x0abb002c = 0x10a
0x0abb0030 = 0x12c
```

## Complete `0x000009ac` order

`0x000009ac` is not optional scaffolding around HIF. It is the vendor sequence
that makes packet memory usable and only then constructs startup.

```asm
0x000009ac  push {r4, lr}
0x000009ae  bl 0x94c
0x000009b2  bl 0xf6
0x000009b6  bl 0x16d24
0x000009ba  bl 0x14c
0x000009be  bl 0x5a8
0x000009c2  bl 0x774
0x000009c6  bl 0xfff0104c
0x000009ca  bl 0x13a40
0x000009ce  bl 0x594
0x000009d2  bl 0x560
0x000009d6  bl 0xc80
0x000009da  bl 0x158a8
0x000009de  pop {r4, pc}
```

Working names and status:

| Address | Role | Rust status |
|---|---|---|
| `0x94c` | Allocate four TX buffers and 30 RX buffers; call HIF init | Partial/manual |
| `0xf6` | Packet/DMA peripheral setup | Partial; destructive/incomplete |
| `0x16d24` | PHY/MAC subsystem setup | Partial fixed writes only |
| `0x14c` | Larger MAC/packet-memory state initialization | Not translated |
| `0x5a8` | Timer/state initialization | Not translated |
| `0x774` | Fixed state/register initialization | Not translated |
| `0xfff0104c` | High-bootstrap support routine | Not translated |
| `0x13a40` | WSM/MAC initialization | Not translated |
| `0x594` | Allocate 30 objects | Not translated |
| `0x560` | Initialize 16 records | Not translated |
| `0xc80` | High-level subsystem initializer | Not translated |
| `0x158a8` | Construct and enqueue startup indication | Wire format translated |

## HIF buffer allocation and initialization

### `0x0000094c`: exact buffer geometry

```asm
0x0000094e  ldr r4, [0x000009a4]       ; 0x09000000
0x00000950  ldr r1, [0x0000099c]       ; 0x04009720
0x00000952  ldr r2, [0x000009a0]       ; 0x000149a8
...
0x00000958  lsls r3, r0, 1
0x0000095a  adds r3, r3, r0
0x0000095c  lsls r3, r3, 7             ; index * 384
...
0x0000096a  str r3, [r5, 8]            ; four TX buffer pointers
...
0x00000970  ldr r1, [0x000009a8]       ; 0x00008a68
...
0x00000976  movs r3, 0x33
0x00000978  lsls r3, r3, 5             ; 0x660 = 1632
0x0000097a  muls r3, r0, r3
...
0x00000984  cmp r0, 0x1e               ; 30 RX buffers
0x00000986  str r3, [r2, r5]
...
0x0000098a  movs r2, 0x33
0x0000098c  lsls r2, r2, 5             ; r2 = 1632
0x0000098e  movs r1, 0x1e              ; r1 = 30
0x00000990  add r0, sp, 4               ; RX pointer array
0x00000992  bl 0xaca
```

Vendor addresses:

```text
TX buffers: 0x090149a8 + index * 384, index 0..3
RX buffers: 0x09008a68 + index * 1632, index 0..29
```

Early diagnostic builds used `0x0900fe00`. After correcting the main stack to
the vendor low-SRAM region, the Rust transport now uses the four vendor TX
buffers beginning at `0x090149a8`.

### `0x00000aca`: important hardware/software effects

Established effects:

```text
RX descriptor ring                 0x0ab00000, 32 descriptors
TX descriptor ring                 0x0ab00100, 4 descriptors
HIF software state                 0x04009754
software TX queue producer         0x040098fc
software TX queue consumer         0x04009900
descriptor TX producer             0x04009914
descriptor TX consumer             0x04009918
TX mask                            0x0400991c = 3
TX descriptor base                 0x04009920 = 0x0ab00100
TX length mask                     0x04009924 = control & 0x7f8
HIF interrupt acknowledgement      0x0ab00128 = 0x7ff
HIF control                        0x0ab00120 = 0x7ff or 0x7f7
IRQ 13                             callback registration, not only enable bit
```

The `0x7f7` choice is conditional on software mode already being one and bit 10
of the pre-existing HIF control being set. It is not a generic reset value.

## Rust runtime BSS initialization

The flat main image does not contain `.bss`, and the translated vendor runtime
clear only covers the vendor range `0x04002078..0x04009c44`. Rust statics at
`__bss_start..__bss_end` therefore retained arbitrary SRAM contents across
reloads. This was exposed when the new scan state started in `Busy` before its
first request.

`hif-startup::_start` now clears its linker-defined Rust `.bss` before calling
`initialize_runtime_state()`. This is required for all `UnsafeCell`-backed
configuration, HIF scratch, and scan state. With the fix, repeated scans retain
and release their plans correctly across ordinary MMC rebinds.

## Vendor loader register section

The 41-entry PHY/MAC register table is not consumed by low firmware code. It is
container section type 2 and is applied directly by the vendor download
bootloader before firmware entry. Radare2 at bootloader `0x080004c4` shows:

```asm
0x080004c4  ldr r0, [section_type]
0x080004c8  cmp r0, 2
0x080004d0  mov r2, 4
0x080004dc  bl read_stream             ; read byte count
...
0x080004f0  ldr r4, [remaining]
0x08000510  add r1, sp, 4
0x08000514  bl read_stream             ; chunks of address/value pairs
...
0x0800052c  ldr r1, [r1, 4]            ; value
0x08000530  ldr r2, [r6, r0, lsl 3]    ; address
0x08000534  str r1, [r2]                ; direct MMIO write
```

The section begins with byte count `0x148`, meaning 41 eight-byte pairs. The
custom Rust downloader now applies all 41 writes before jumping to the low
image. This includes defaults in `0x0ab80000..0x0abb0078` and reproduces a
previously omitted vendor loader side effect. Startup, probe, configuration,
and empty-scan completion continue to work with the table enabled.

## Real vendor start-scan dispatcher

The WSM dispatcher at `0x0000e558` masks the command ID with `0x0c3f`, clamps
IDs above `0x24`, and jumps through the initialized-SRAM table at
`0x04000710`. Table index 7 contains Thumb pointer `0x00010cfb`, establishing
the real start-scan request path:

```text
WSM ID 7
  -> 0x00010cfa  request wrapper / confirmation builder
       -> 0x00013d44  start-scan validation and scheduler activation
```

`0x10cfa` passes the payload at request offset four to `0x13d44`, then rewrites
the request buffer as an eight-byte status confirmation and queues it through
`0xed4c`.

`0x13d44` validates:

- no scan already active (`0x0400860c`);
- at most 34 channels and 16 SSIDs in the vendor ABI;
- every channel has nonzero maximum dwell;
- maximum dwell is not below minimum dwell;
- probe delay does not exceed `minimum_dwell * 1024`.

On success it copies retained scan state, marks `0x0400860c = 1`, and sets
platform event bit 10 at `0x04001fd4` to start scheduler processing. The Rust
scan engine now reproduces these channel validation rules, vendor busy status
4, the active byte, and event-bit lifecycle while preserving CW1200's public
48-channel parser/storage limit. Linux currently submits 11-channel batches,
which remain valid.

The event callback registration block at `0x00000698..0x000006e4` maps event
bit 10 to Thumb callback `0x00014353`. This establishes the next intact path:

```text
0x13d44 sets event bit 10
  -> scheduler 0xf140
  -> callback table 0x040021b4[10]
  -> 0x14352 scan state machine
       -> 0x14304 per-channel setup
            -> 0xfdfa channel-control encoding
            -> 0xf802 full MAC/channel programming
```

`0x14304` obtains the current retained channel and calls `0xfdfa`. The latter is
fully translated as the pure Rust `channel_control_word()`: band zero starts
with `0x17`, band one with `0x26`, bandwidth modes add `0x100/0x200/0x400`,
and channel flag bit 8 adds `0x40`. A normal 2.4 GHz scan channel in mode zero
therefore produces `0x0117`. Retained Rust scan state now exposes this exact
control word per channel, forming the first tested connection from the parsed
WSM request into the real vendor channel-setup ABI.

`0x14304` then assembles an exact eight-byte call ABI for `0xf802`:

```text
byte 0    operation = 0
byte 1    option = scan flags bit 2
u16  2    `0xfdfa` channel-control word
u16  4    channel number / flags
byte 6    link count = 1
byte 7    link ID = 2
```

This is represented with a `zerocopy` wire type and is produced directly from
retained scan state. For band zero, flags bit 2 set, and channel 6, the bytes
are `00 01 17 01 06 00 01 02`.

The following `0x124c0` prerequisite is also translated as a pure classifier.
It stores gate value `0x40` when control bits `0x22` are both present or the low
seven bits equal `0x12`; otherwise it stores `1`.

`0xf802` is the true large channel-programming boundary. It configures per-link
state, packet engines, gain/rate tables, MAC state, and downstream operations;
its side effects are not yet enabled.

The real scan-complete producer is now identified. Terminal scan state calls
`0x13fac`, which clears scan state and calls `0x111ba`. `0x111ba` allocates a
12-byte message, assigns WSM ID `0x0806`, fills status, PSM, channel count, and
a trailing 16-bit vendor field, then queues it through `0xed4c`:

```text
scan terminal state
  -> 0x13fac  cleanup and completion argument assembly
       -> 0x111ba  12-byte WSM_SCAN_COMPLETE_IND
            -> 0xed4c
```

The Rust indication now also uses the vendor/CW1200-aligned 12-byte layout,
with its currently unused trailing field zeroed. Repeated Linux scans accept
this corrected length. The previously examined `0x11208` constructs a
`0x0809` indication, not scan-complete.

## MAC table generation at `0x00017008`

Radare2 `pdf` and Ghidra decompilation now agree on the complete algorithm:

1. Select the 22 six-byte anchors at `0x04000ca0` for mode zero or
   `0x04000d24` for mode one.
2. Read a signed correction from `0x040034f8` or `0x0400358a`.
3. Normalize each signed pair as `(upper - correction + 8) >> 4` and
   `(lower + correction + 8) >> 4`.
4. For each of 80 target positions, search anchors from index 21 down to zero,
   retaining the qualifying entry with the lowest signed lower value.
5. Pack the selected selector and low seven upper bits into both halfwords of
   one `0x0ab80800` table word.
6. Apply the mode-specific address/value list, copy table bits into
   `0x0ab80400`, and place the first one-based entry whose upper field is below
   11 into the low seven bits of `0x0ab80410`.

`src/phy.rs` contains both extracted anchor arrays, all five relevant register
lists, the pure table generator, golden-value tests, and an unsafe exact
mode-zero hardware function. With zero correction the generated table begins
with `0x2b202b20`, has boundary entry 55, and ends with `0x001a001a`.

The translated `0x16a38`, `0x198f2`, and `0x16ca4` software-state writes now
run immediately before the hardware function. They initialize the vendor MAC
state at `0x0400994c`, mode-zero helper pointers/state at `0x040099d4`, and the
two signed remap-derived timing values at `0x04009990/0x04009992`. For remap
window two `0x04118000`, the exact signed decoding produces `1032` and `-852`
(stored as a wrapped `u16`), not an unsigned `9940`.

The hardware function then runs after packet-DMA preparation, matching the
`0x16d24` position in `0x9ac`. Probe and repeated empty scans remain stable.
A bounded halted read verified the live values:

```text
0x0ab80c00 = 0x00b43fdb   bit 11 enabled
0x0ab80800 = 0x2b202b20   first generated table word
0x0ab80400 = 0x55f42b2b   table field merged
0x0ab80410 = 0x00000037   one-based boundary 55
0x0aba2000 = 0x00ed00ed   initialized work table
0x0aba805c = 0x27082026   vendor fixed value
```

IRQ 6 is no longer a diagnostic stub. The vendor callback at `0x0000f1fe`
sets bit 27 in the platform event word at `0x04001fd4`; the Rust callback now
reproduces that operation. IRQs 18, 20, and 21 remain diagnostic stubs, so no
channel operation is started yet. Also, resuming after a debugfs halt lost a
subsequent HIF command
interrupt and killed the BH; rebind recovered normally. Treat debugfs halt as a
postmortem operation for this path rather than expecting a live resume.

## Pre-HIF platform flow

### `0x00000bb0`: do not reorder or omit the transition

Exact excerpt:

```asm
0x00000bb0  ldr r0, [0x00000c20]       ; 0x0a980000
...
0x00000bbe  str r1, [r0]               ; clear 0x80, set 0x10
0x00000bc4  str r1, [r0, 8]            ; clear 0x10
0x00000bc6  ldr r0, [0x00000c24]       ; 0x0aa80040
...
0x00000bce  str r1, [r0, 8]            ; set 0xf0
...
0x00000bd6  str r1, [r0]               ; set 0x00f00000
0x00000bd8  ldr r0, [0x00000c28]       ; 0x00040090
0x00000bda  ldr r1, [0x00000c2c]       ; 0x0ac80040
0x00000bdc  str r0, [r1, 0x18]         ; 0x0ac80058
0x00000be0  bl  0x16600                ; writes 0x10 to 0x0ac80064
...
0x00000bf8  str r0, [r2, 0x3c]         ; optional 0x00120000 at 0x0ac800bc
0x00000bfa  ldr r1, [0x00000c24]       ; 0x0aa80040
0x00000c00  subs r1, 0x40              ; 0x0aa80000
0x00000c02  str r0, [r1, 4]            ; 0x0aa80004 = 0x200
0x00000c06  str r0, [r3, 4]            ; 0x04001fd8 = 1
0x00000c0a  str r0, [r2, 0x20]         ; 0x0ac800a0 = 31
0x00000c0e  str r0, [r2, 0x1c]         ; 0x0ac8009c = 6
0x00000c14  bl  0x16148                ; register IRQ 4
0x00000c18  bl  0x58b6                 ; clock parameter setup
```

The current Rust diagnostic path defers `0x0aa80004 = 0x200`. That is useful for
localization but is not a candidate final implementation. The correct task is
to reproduce all prerequisites and downstream handling around this exact
transition.

### `0x00000a74 -> 0xfff019aa`

```asm
0x00000a74  ldr r1, [0x00000a84]       ; 0xfff032ec
0x00000a76  movs r0, 1
0x00000a7a  str r0, [r1]
0x00000a7c  bl 0xfff019aa
```

High-bootstrap routine:

```asm
0xfff019aa  movs r0, 0x2b
0xfff019ac  movs r1, 0xa9
0xfff019ae  lsls r1, r1, 0x14          ; 0x0a900000
0xfff019b2  str r0, [r1, 4]            ; 0x0a900004 = 0x2b
0xfff019b4  ldr r1, [0xfff01a28]       ; 0xfff01999
0xfff019b6  movs r0, 0
0xfff019b8  bl 0x16148                 ; register IRQ 0
0xfff019bc  ldr r1, [0xfff01a2c]       ; 0xfff01991
0xfff019be  movs r0, 2
0xfff019c0  bl 0x16148                 ; register IRQ 2
```

Merely setting interrupt-controller enable bits is not equivalent to installing
the callback table entries used by `0x16148`.

### `0x00000ab4`: activation

```asm
0x00000ab4  ldr r0, [0x00000b60]       ; 0x0ab00140
0x00000ab6  ldr r0, [r0]
0x00000ab8  ldr r1, [0x00000b64]       ; 0x04009754
0x00000aba  movs r2, 1
0x00000abc  str r2, [r1]               ; software mode = 1
0x00000abe  ldr r1, [0x00000b60]
0x00000ac0  subs r1, 0x40              ; 0x0ab00100
0x00000ac2  ldr r0, [r1, 0x34]         ; 0x0ab00134
0x00000ac4  lsls r0, r0, 0x15          ; test bit 10
0x00000ac6  bmi 0xac2
0x00000ac8  bx lr
```

`0x0ab00140` is read but not written by this routine. The working vendor value
`2` is hardware/runtime generated.

## Packet-DMA order

`0x000000f6` calls its initializers in this exact order:

```asm
0x000000f6  push {r4, lr}
0x000000f8  bl 0xbc
0x000000fc  bl 0xf608
0x00000100  bl 0xf4f4
0x00000104  bl 0x44
0x00000108  bl 0xfcfe
0x0000010c  bl 0x4c6
0x00000114  bl 0x16148                ; IRQ 27
0x0000011c  bl 0x16148                ; IRQ 26
0x00000120  pop {r4, pc}
```

Current granular observations:

```text
PDM1   completed the 0xbc register block
PDM2   completed 0xf608 / 0x09c00e00 setup
PDM22  completed four channel records at 0x09c60000 and 0x09c60100
next   first write to vendor list 0x09016a28 did not produce a later postcode
```

This does **not** justify skipping the list in a final implementation. It means
that an earlier packet-memory/remap prerequisite is missing.

## Exact startup indication construction

### `0x000158a8`

```asm
0x000158aa  movs r0, 0xa8
0x000158ac  bl 0xe5f4                 ; allocate 168 bytes
0x000158b2  strh r1, [r0]             ; length 0xa8
0x000158b4  ldr r1, [0x00015928]      ; 0x0801
0x000158b8  strh r1, [r0, 2]
0x000158ba  movs r1, 0x1e
0x000158bc  strh r1, [r0, 4]          ; 30 input buffers
0x000158be  movs r1, 0x33
0x000158c0  lsls r1, r1, 5
0x000158c2  strh r1, [r0, 6]          ; 1632-byte buffers
...
0x000158c4  movs r1, 2
0x000158c6  strh r1, [r0, 0x10]       ; firmware type
0x000158c8  ldr r1, [0x0001592c]      ; API 0x424
0x000158cc  strh r1, [r0, 0x12]
...
0x00015900  ldr r0, [0x0001593c]      ; build 0x148a
0x00015902  strh r0, [r4, 0x14]
0x00015904  movs r0, 8
0x00015906  strh r0, [r4, 0x16]       ; version 8
0x0001590c  bl 0xf0a4                 ; copy 128-byte label
0x00015914  bl 0xed4c                 ; enqueue
```

The Rust wire layout matches this shape, but several identity values currently
differ and the Rust path bypasses the vendor allocator and two-level queue.

## Exact enqueue and descriptor publication

### `0x0000ed4c`: software enqueue

Important effects from the exact instructions:

```asm
0x0000ed5e  ldrh r0, [r4, 2]
...
0x0000ed68  orrs r0, r1               ; add transport bit 0x400 if required
0x0000ed6a  strh r0, [r4, 2]
0x0000ed6c  ldr r7, [0x0000edec]      ; 0x04009754
0x0000ed6e  ldrh r0, [r7, 4]
0x0000ed70  adds r0, 1
0x0000ed78  strh r0, [r7, 4]          ; pending-message count
...
0x0000ed7c  ldr r0, [0x0000edd0]      ; 0x04001fd4
0x0000ed82  blx 0xefdc                ; set event bit 1 at 0x04001fd8
...
0x0000edb6  str r4, [r0, 0x28]        ; software TX pointer queue
0x0000edba  str r0, [r6, 0x28]        ; advance 0x040098fc
...
0x0000edc8  bl 0xec82                 ; publish if <4 outstanding
```

### `0x0000ec82`: hardware descriptor publication

```asm
0x0000ec84  ldr r5, [0x0000edd8]      ; 0x04009914
0x0000ec8a  ldr r0, [r0, 0x28]        ; software queue producer
0x0000ec8c  ldr r4, [r5]              ; descriptor producer
0x0000ec8e  cmp r0, r4
0x0000ec90  beq 0xecde
...
0x0000ec9c  ldr r6, [r0, 0x28]        ; message pointer
0x0000ec9e  ldrh r7, [r6, 2]          ; WSM ID
...
0x0000ecb0  lsls r1, r4, 0xd
0x0000ecb2  orrs r1, r7
0x0000ecba  strh r1, [r6, 2]          ; insert sequence bits
0x0000ecbc  bics r2, r3               ; buffer & 0xf6ffffff
0x0000ecc4  str r2, [r3, r1]          ; descriptor address
0x0000ecc6  ldrh r2, [r6]             ; message length
0x0000ecca  adds r2, 1
0x0000ecce  lsrs r2, r2, 0x13         ; mask to 13 bits
0x0000ecd0  orrs r2, r3               ; valid bit
0x0000ecd2  orrs r0, r2               ; preserve transport bits
0x0000ecd8  str r0, [r2, r1]          ; descriptor control
0x0000ecdc  str r0, [r5]              ; advance descriptor producer
```

Required software state missing from the current direct Rust publication:

```text
pending-message count at 0x04009758
first-message scheduler event at 0x04001fd8
complete transport/valid-bit handling
later completion/reclaim behavior
```

## Final global barrier and interrupt enable

`0x00016550` is ARM code. Decode it in ARM mode:

```asm
mcr p15, 0, r0, c7, c10, 4   ; drain write buffer
mrs r0, cpsr
bic r1, r0, #0xc0            ; clear IRQ and FIQ mask bits
msr cpsr_c, r1
bx  lr
```

A local barrier in Rust `Transport::publish()` is not equivalent to this global
post-`0x9ac`, post-routing transition.

## Remap-window evidence

Cold raw hardware values read through `0x0aa80008/0x0aa8002c`:

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

`0x00015944` adds `0x166` 4-KiB pages to three packed address fields before
saving windows 3..5 in firmware state:

```text
saved window 3 = 0xf0a1a00d
saved window 4 = 0x80a5a196
saved window 5 = 0x00000083
```

Do not infer CPU death solely from loss of the `0x0900ff98` postcode after a
clock/remap write. The postcode address itself may have moved or become owned
by another hardware view.

## Current Rust implementation delta

As of this note, `xr819-firmware/src/bin/hif_startup.rs` is still an
instrumented diagnostic path, not a faithful replacement for `0x000164bc`.

| Vendor behavior | Current Rust behavior |
|---|---|
| Observe zero at `0x04001428` | Flat custom loader must reproduce vendor initialized-SRAM zero before waiting |
| Complete high reset/bootstrap | Downloader sets TCM/CP15; main sets one stack only |
| Complete `0xa74/fff019aa` callbacks | Hardware write and IRQ enable bits only |
| Complete `0xbb0`, including `0xaa80004 = 0x200` | Transition restored and proven working |
| Register IRQ 4/6/18/20/21/13 callbacks | Table/source side effects reproduced with diagnostic stubs |
| Complete all of `0x9ac` | HIF and packet-DMA translated; intervening MAC/timer routines remain partial |
| Vendor four-buffer allocator | Four packet-RAM buffers at `0x090149a8`, selected by TX producer |
| `0xed4c -> 0xec82` two-level queue | Direct publication plus descriptor reclaim; scheduler accounting remains partial |
| Routing after `0x9ac` | Removed from minimal path after discovering it was too early |
| Global barrier and IRQ/FIQ enable | Local publish drain; CPU interrupts remain masked |
| Scheduler `0xf140` | Polling loop |

`STARTUP_DEBUG_STAGE` and `REMAP_DEBUG_INDEX` are patchable diagnostic words.
They must not be mistaken for production design.

## Experiment ledger

### Proven

- The vendor wait at `0x04001428` depends on the container's initialized-SRAM
  segment. Removing the custom zero initialization made the main image time out
  with `DL??` before stage 0. Therefore the zero write is a required
  loader-equivalent data effect, not evidence that the host changes this word
  during the custom downloader protocol.
- Rust downloader executes.
- Rust main image executes and can update a retained mailbox/heartbeat.
- The first `WSM_STARTUP_IND_ID` is delivered and parsed by Linux.
- Host-to-firmware RX descriptor polling works.
- `WRITE_MIB_REQ_ID` receives a successful generic confirmation.
- `CONFIGURATION_REQ_ID` receives a structured configuration confirmation.
- The complete SDD/DPD request is validated as TLV data and retained in static
  firmware storage; element `0xc5` resolves to the expected 24 MHz reference.
- Linux completes probe and registers a `phy` / `wlan0` without a stuck command.
- Start-scan is parsed exactly as Linux serializes it: a 12-byte fixed header,
  16-byte channel records, and 36-byte SSID records. The borrowed HIF payload is
  copied into a fixed retained scan plan before the RX descriptor is reused.
- Firmware-owned work is serviced before the next host request. Start-scan
  receives `0x0407`, then an explicit empty `0x0806` on the next service pass,
  so repeated Linux scans finish cleanly while the real PHY scan is absent.
- Postmortem SRAM reads work after stopping BH, asserting CPU reset, restoring
  ACCESS mode, and waiting 30 ms.
- `CONTROL = 0x3000` means host WUP + hardware RDY + zero next-message length.
- Startup wire shape and first descriptor format are understood.
- Cold remap windows are captured exactly.

### Prefix observations

```text
STG0  initialized-SRAM startup wait completed
STG1  main-control update completed
STG2  translated 0x9e0 subset completed
STG3  high-platform subset completed
STG4  DMA/clock subset with destructive transition deferred completed
STG5  bounded HIF activation completed
HIN6  HIF ring/control initializer reached its final postcode
```

### Destructive/incomplete boundaries

```text
0x0aa80004 = 0x200
    RESOLVED: the Rust main stack was incorrectly at 0xfff1ff00. The transition
    invalidated that window, so the next call/stack access stopped execution.
    Moving SP to the vendor region below 0x0400c000 allows `0xbb0`, HIF setup,
    and later packet-DMA initialization to complete.

0x09016a28
    RESOLVED by the same stack correction. Both `0x090149a8` and `0x09016a28`
    pass write/readback after `0xbb0`; the complete list write reaches stage 8.

0x0abb0010
    routing was attempted before startup in an earlier Rust path; vendor writes
    it after 0x9ac, and the premature write blocked progress.

0x0900fe00 temporary TX buffer
    obsolete diagnostic workaround; production bring-up now uses the vendor
    buffer bank at `0x090149a8`.
```

### Closed hypotheses unless new evidence appears

- High ARM versus low Thumb entry alone does not fix HIF.
- Descriptor `0x7ff` versus `0x7f7` selection alone does not fix HIF.
- Emergency descriptor publication alone does not fix HIF.
- Repeating live queue-mode AHB/APB reads is not useful; use post-reset ACCESS
  mode for retained postmortem data.

## Next reverse-engineering tasks

Work from the vendor order, not by accumulating isolated writes:

1. Replace the IRQ 18/20 encoder callback and IRQ 21 MIC callback with
   translated queue completion behavior. IRQ 6, `0x16a38`/`0x198f2`/`0x16ca4`
   software state, mode-zero MAC hardware initialization, and bit 11 enable are
   now active and target-tested.
2. Translate the bounded leading portion of `0xf802`, now verified as the real
   channel-programming boundary reached through event bit 10, `0x14352`, and
   `0x14304`; preserve the verified `0x13fac -> 0x111ba -> 0xed4c` completion
   path when replacing synthetic completion. The earlier `0x18836 -> 0x1856a -> 0x1814a` chain
   remains withdrawn because those are interior calibration blocks.
3. Replace the empty scan completion with real channel tuning and receive
   indications, preserving the retained SDD for board-specific values.
4. Translate the remaining middle of `0x9ac`, then restore post-`0x9ac` routing,
   the global `0x16550` barrier, real IRQ handlers, and scheduler entry.
5. Implement faithful `0xed4c` pending/event accounting and RX/TX completion.

## Operational warnings

- Safe firmware-only reset can be performed by unbinding and rebinding the
  `1c10000.mmc` platform device when no fatal packet-DMA state is active.
- Partial `0x09c0xxxx` initialization can require a physical XR819 power cycle.
- The target runs `6.18.0-xr819-test+`.
- The current source tree reports Linux `7.2.0-rc5`. After regenerating local
  kernel build metadata, newly built modules have `7.2.0-rc5-xr819-test+`
  vermagic and must **not** be installed on the 6.18 target. The target still
  has the earlier matching diagnostic module. Recover a matching 6.18 build
  tree or update the target before further module deployment.
- The currently installed image delivers startup, handles initial write-MIB and
  configuration requests, and registers `phy44` / `wlan0`. It still lacks real
  PHY/channel/scan behavior.
