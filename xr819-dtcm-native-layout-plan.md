# XR819 DTCM native-layout migration plan

**Status:** fixed initialized multi-VIF beacon timer structurally decoded at `0x04001250..0x04001264`; default/diagnostic process-local host tests, source/linked drift-evidence gates, stack checks, complete exact-parent text-symbol delta gating, and normalized clean-B6 checks pass. No hardware run was performed or permitted. Fixed ABI identity and mixed volatile ownership remain; address movement and exclusive ownership are out of scope.  
**Firmware lineage:** candidate based directly on `8940467e9cdc` (`Model VIF state in Rust`), itself atop qualified low-MAC/PAS. The exact parent was rebuilt from revision files for code-generation comparison; no rejected patch was applied.  
**Vendor container:** `/tmp/fw_xr819.bin`, SHA-256 `3e2462d476c9dfcb907cda1ba81d0a6d1bbee5e3207bdc1911ec5042d96fdfca`, size `0x1fe44`.  
**Primary local evidence:** `xr819-decompilation/annotated-main.c`, `xr819-decompilation/annotated-tcm.c`, the container above, `xr819/ghidra-fw-main.bin.gzf`, `xr819/xr819-tcm.bin.gzf`, current Rust source and ELF, revision history, and rejected patches in `/tmp`. No web sources were used.

## Executive verdict

A one-shot retirement of the remaining DTCM ABI is **not implementable safely from the present map**. It is a reasonable final target, but only after translating a much larger closure than “all visible `0x0400xxxx` literals.” The blockers are structural:

* major families are reached through base-plus-index and negative-offset pointer arithmetic;
* the scheduler, low-MAC/PAS, VIF, scan/JOIN, timer, BA, power-save, HIF/MIC, context, and PHY families form cycles;
* some addresses are roots for several apparent subsystems or are addressed through another family’s base (for example, link state is reached as `VIF_BASE + 0x4920`);
* the fixed-address TALA experiment failed even when its exact `0x24`-byte shape and pointer arithmetic were preserved, proving that source-level layout equivalence is not sufficient evidence that an address may move;
* untranslated ARM/Thumb code still mutates many of the candidate fields. Such storage cannot be described as sound Rust-owned state.

What can be done now is narrower:

1. keep already translated CPU-only state in ordinary ITCM Rust globals;
2. retain the existing typed internal TX context pool at its fixed DTCM identity `0x04009080..0x040094d4` as **typed quarantine**, not as proof of exclusive ownership;
3. add read-only maps, generated reference reports, exact-layout tests, and family-specific volatile access layers;
4. translate whole dependency closures until a final atomic address-changing migration becomes possible.

The likely final native form is **several independently typed Rust family objects**, mostly in ITCM, followed by one atomic removal of all legacy pointer roots and DTCM loader assumptions. The previously rejected design was a movable, exclusively/native-owned packed `DtcmLayout`. The implemented candidate is materially different: one fixed-address, private, `MaybeUninit`/`UnsafeCell`-backed object is used only as a shared quarantine ABI view. It allocates no holes, exposes no complete-record safe references, and does not claim one ownership or synchronization domain.

---

## 1. Physical and loader constraints

### 1.1 Usable DTCM

The only usable DTCM address window is:

```text
0x04000000..0x0400c000    48 KiB uniquely addressable
```

Although CP15 `c0,c0,2` reports `0x001c0200` (nominally 128 KiB ITCM and 64 KiB DTCM), a destructive save/write/read/restore probe showed that `0x0400c000` aliases `0x04000000`. The current documentation records that writing `0xc0dec000` at `0x0400c000` changed the word at `0x04000000`, while `0x04004000` and `0x04008000` did not alias. Therefore:

```text
0x0400c000..0x04010000    NOT independently allocatable; aliases live low DTCM
```

This rules out “using the missing upper 16 KiB” for native state.

### 1.2 Stacks are not layout fields

The implemented candidate's linker partition in `xr819-firmware/link-main-low.x` is:

```text
0x04000000..0x0400a000    DTCM_STATE shared quarantine ABI object
0x0400a000..0x0400c000    DTCM_STACKS
```

The internal context pool remains a linker-exported member view at `0x04009080..0x040094d4`; it is no longer a standalone output section.

The stack region has a fixed internal partition:

```text
0x0400a000..0x0400a500    five 0x100-byte exception-mode stacks
0x0400a500..0x0400c000    0x1b00-byte (6,912-byte) system stack
```

These bytes must never be fields of a DTCM software-state struct. They are a physical call-stack allocation, not retained state. The linker already asserts the floor and top at `0x0400a000` and `0x0400c000`.

### 1.3 Vendor container initialization

The matching vendor container performs:

```text
COPY 0x04000000 length 0x10f8
COPY 0x040010f8 length 0x0f80
FILL 0x04002078 length 0x7bcc with zero
```

Thus:

```text
0x04000000..0x04002078    initialized DTCM image (COPY), total 0x2078
0x04002078..0x04009c44    vendor BSS-like zero region (FILL), total 0x7bcc
0x04009c44..0x0400a000    0x3bc-byte uninitialized/research margin
```

The current Rust sectioned image has no DTCM `PT_LOAD`. The release ELF inspected for this lineage has:

```text
.text                 0x00000000 size 0x106b4
.data                 0x000106b4 size 0x04c0
.bss                  0x00010b78 size 0x3728
.noinit.exception     0x000142a0 size 0x58
.dtcm.state           0x04000000 size 0xa000, SHT_NOBITS, no PT_LOAD
  internal pool view  0x04009080..0x040094d4 via linker-exported symbols
```

The packer emits only two ITCM load segments; it does not copy or fill the fixed context pool. `platform::initialize_runtime_state()` explicitly zeroes `0x04002078..0x04009c44`, and runtime startup reconstructs selected low-DTCM values and free lists. Low initialized DTCM below `0x04002078` is otherwise retained from an earlier vendor/boot state unless a specific Rust initializer rewrites it. This distinction matters: “zero at boot,” “zero on every Rust reload,” and “retained vendor COPY data” are not interchangeable.

### 1.4 Current good Rust state belongs in ITCM

At `ad989e0a5b1a`, ordinary Rust CPU state is in ITCM `.data`/`.bss`. Examples include:

* `Transport`, `HifRingState`, and `HifQueues`;
* response scratch and HIF sequence state;
* Rust scan storage;
* completion FIFO;
* probe-context sequence;
* PAS active-context accounting;
* non-class-0 internal-context count;
* retry PRNG state;
* channel PLL cache;
* channel power limits.

These are already outside the legacy DTCM ABI. A DTCM migration must not move them back merely to make one large `DtcmLayout` aesthetically complete.

Hardware-visible packet RAM (`0x090...`, `0x094...`), shared HIF SRAM (`0x0ab...`), high support SRAM (`0xfff...`), and MMIO (`0x09c...`, `0x0a...`) are separate ownership domains and must not become DTCM fields.

---

## 2. Evidence and local analysis method

### 2.1 Sources read

The investigation used:

* `xr819-firmware/link-main-low.x`, `src/download.rs`, `src/loader.rs`, `tools/pack-sectioned-elf.py`, `tools/check.sh`, and the current release ELF;
* `src/tx.rs`, `src/vendor_host_tx.rs`, `src/mac.rs`, `src/vif.rs`, `src/scan.rs`, `src/hif.rs`, `src/phy.rs`, `src/platform.rs`, and `src/configuration.rs`;
* `xr819-decompilation/annotated-main.c` and `annotated-tcm.c`;
* `xr819-firmware-reverse-engineering.md`, `xr819-hif-startup-flow.md`, `xr819-vendor-host-tx-lifecycle.md`, `xr819-aes-engine.md`, and `xr819-firmware/vendor-container-layout.md`;
* accepted migration history from `021fe0da9886`, `49d692f63662`, `42d7f0695e15`, `0d7bba6a09a1`, `15550c738a1a`, `5ece4273d44b`, `4699aef946de`, `b6d3d7dd84f5`, and the fixed-pool change in the current lineage;
* rejected patches `/tmp/xr819-native-tala-rejected.patch`, `/tmp/xr819-native-tala-exact-layout-rejected.patch`, and `/tmp/xr819-native-internal-context-pool-rejected.patch`.

The Ghidra project files were treated as provenance for the exported decompilation. The currently broken bridge was not required.

### 2.2 Recovering the vendor images

The local container grammar is documented and can be parsed without Ghidra:

```sh
python3 xr819-firmware/tools/inspect-vendor-container.py /tmp/fw_xr819.bin
```

For additional analysis, extract the destination-zero ITCM copy, the `0xfff00000` ARM support copy, and the two DTCM copies. The address relationship is direct: a word at a literal-pool address in the extracted ITCM/high image can be decoded as little-endian `u32`.

This was used to resolve, for example, the high-image TCM dispatcher literals:

```text
DAT_fff004b0 = 0x04003e98    VIF array
DAT_fff004b8 = 0x04003678    PAS/global low-MAC root
DAT_fff00d2c = 0x04005a24    30-entry host TX context pool
DAT_fff00d34 = 0x04001680    low-MAC pipe/global root
DAT_fff00d40 = 0x04008f6c    completion/context accounting root
DAT_fff011bc = 0x04001fd4    scheduler event structure
DAT_fff01358 = 0x04003e98    VIF array used by global initialization
```

### 2.3 Finding computed references that literal grep misses

A source grep for `0x04003e98` finds Rust literals and prose, but not every vendor use. The vendor main image often loads a DTCM base from an ITCM literal pool and then computes fields by stride and offset. A useful local report can be produced by:

1. parse every `DAT_000xxxxx`/`PTR_DAT_000xxxxx` token in `annotated-main.c`;
2. interpret the numeric suffix as an address in the extracted ITCM image;
3. read the little-endian word at that address;
4. retain values in `0x04000000..0x0400a000`;
5. associate each token with the enclosing decompiled function heading.

This method found, among many others:

```text
0x04003678 referenced by dozens of PAS, low-MAC, RX, TX, BA, scan, wake,
           power-save, rate, and scheduler functions.
0x04003e98 referenced by VIF, JOIN, scan, BA, RX/TX, power-save, key,
           measurement, template, and WSM dispatch functions.
0x04001fd4 referenced by the scheduler, IRQ paths, HIF/MIC, scan/JOIN,
           measurement, BA, PHY, timers, and TX completion.
0x040094d4 referenced as base + if_id * 0x104 by power-save initialization
           and many runtime power-save/JOIN/scan consumers.
```

Representative computed forms in the decompilation are:

```c
vif = 0x04003e98 + if_id * 0x3b0;
pas_global = 0x04003678;
pas_vif = 0x04003678 + if_id * 0x98 + 0x470;
pipe = 0x04001680 + pipe_id * 0x6c + 0xa0;
host_ctx = 0x04005a24 + index * 0x170;
ps_vif = 0x040094d4 + if_id * 0x104;
sequence = 0x04008890 + internal_link * 0x20 + tid * 2;
link_entry = 0x04003e98 + 0x4920 + index * 0x0c;
```

Exact source examples include:

* `fw_global_state_init` in `annotated-tcm.c` around lines 999–1167: two VIF records at stride `0x3b0`, three PAS records at stride `0x98`, large shared arrays, and cross-family initialization;
* `fw_timers_and_tasks_init` in `annotated-main.c` around lines 727–820: VIF timer objects and scheduler task registration;
* `task_22bc` around lines 2707–2775: pending TX, pipe scheduler, VIF counters, radio ownership, scan abort, and timer state in one function;
* `pas_build_rate_tables` around lines 9704–9822: PAS/VIF-relative tables and global airtime/class tables;
* `txp_program_pipe_hw` around lines 11112–11228: PAS stride `0x98`, VIF stride `0x3b0`, global scheduler mask, and link table reached through a computed offset;
* `mac_irq_handler` around lines 11950–12100: live pipe state, event publication, retry/completion dispatch, and scheduler writes;
* `tx_complete_tala_adapt` around lines 15497–15778: negative offsets from `0x04008f6c`, VIF stride `0x3b0`, PAS stride `0x98`, power-save calls, class callbacks, completion FIFO, and scheduler events;
* `phy_select_rate_tables` around lines 31542–31568: writes at offsets from the PHY root, including `0x040099d8`, `0x040099ec`, `0x040099f0`, and `0x040099f4`;
* `phy_apply_cfg_if_channel_match` around lines 30745–30783: the former PLL cache tuple and shared force state;
* `phy_program_gain_for_channel` and `phy_txpower_from_rate_table` around lines 31912–32220: power limits, threshold, selected table pointer, and PHY state all accessed relative to common roots.

This report should become a generated artifact before migration. Literal-only gates are insufficient.

---

## 3. Byte/range map of retained state below `0x0400a000`

### 3.1 Confidence labels

* **High:** exact base, size/stride, and role are supported by current Rust, decompilation, binary data, or linker assertions.
* **Medium:** base and family are supported, but internal extent or complete reader/writer set is incomplete.
* **Low/hypothesis:** useful grouping inferred from proximity or partial offsets; not safe for allocation or movement.
* **Unknown:** no coherent semantic claim. Unknown bytes are not free space.

### 3.2 Non-overlapping coarse map

This table is intentionally conservative. Subranges below refine known islands without implying that surrounding bytes are free.

| Range | Size | Classification | Evidence/confidence |
| --- | ---: | --- | --- |
| `0x04000000..0x04002078` | `0x2078` | Vendor initialized image: constant tables, dispatch tables, callbacks, timing/rate data, boot/scheduler parameters, AES microcode, and opaque data | High initialization; mixed semantic confidence |
| `0x04002078..0x04002094` | `0x1c` | typed saved register context and PAS backoff overrides | High structural confidence; shared quarantine |
| `0x04002094..0x0400218c` | `0xf8` | typed retained debug-console state | High structural confidence; untranslated quarantine |
| `0x0400218c..0x040021b4` | `0x28` | `ClockParameters` record | High shape from `register_structs!`; shared/timer-owned |
| `0x040021b4..0x04002234` | `0x80` | 32-entry scheduler handler table | High, `32 * 4`; contains code pointers |
| `0x04002234..0x040022b8` | `0x84` | typed 22-record PHY gain-source table | High structural confidence from retained builder loop |
| `0x040022b8..0x04002984` | `0x6cc` | typed retained beacon storage, selector, and IE indexes | High structural confidence; RF overlap remains address-only |
| `0x04002984..0x04003050` | `0x6cc` | beacon/template-adjacent opaque BSS | Low-medium; not allocatable |
| `0x04003050..0x040030d0` | `0x80` | typed two-record template descriptor table | High structural confidence from retained initializer |
| `0x040030d0..0x040034b0` | `0x3e0` | typed three-class template backing buffers | High structural confidence from retained descriptor initializer |
| `0x040034b0..0x040035e0` | `0x130` | typed SDD-derived channel/gain/profile tables | High structural confidence; shared quarantine |
| `0x040035e0..0x04003670` | `0x90` | typed wake clock words and 32 response pointers | High structural confidence; shared quarantine |
| `0x04003670..0x04003674` | `0x4` | typed duration-source halfwords | High |
| `0x04003674..0x04003678` | `0x4` | unknown | Unknown, not allocatable |
| `0x04003678..0x04003e78` | `0x800` | shared low-MAC/PAS/rate/link/queue state with many computed overlays | High family root, incomplete fields; all-or-nothing group |
| `0x04003e78..0x04003e98` | `0x20` | typed pre-VIF/link/aggregation header | High link-bitmap field at `+0x18`; remaining bytes opaque |
| `0x04003e98..0x040049a8` | `0xb10` | three VIF records, stride `0x3b0` | High stride/count; internal records remain mixed |
| `0x040049a8..0x04005a24` | `0x107c` | unknown/possibly VIF-adjacent tables and gaps | Unknown, not allocatable |
| `0x04005a24..0x04008544` | `0x2b20` | 30 host WSM TX contexts, stride `0x170` | High exact range; mixed Rust/vendor mutation |
| `0x04008544..0x04008594` | `0x50` | eight peer-pipe records plus management/scan tail | High count/stride and field shape |
| `0x04008594..0x040085fc` | `0x68` | uploaded WSM command-15 blob | High size; `0x040085f8` overlaps/anchors channel-switch control |
| `0x040085f8..0x04008618` | `0x20` | channel-switch/scan control overlay | Medium; overlaps blob tail, proving a simple field partition is unsafe |
| `0x04008618..0x04008798` | `0x180` | LMC/encryption/free-list roots and mixed control | Medium; several roots, incomplete extent |
| `0x04008798..0x040087b0` | `0x18` | host-context/duplicate-cache accounting | Medium |
| `0x040087b0..0x040087b8` | `0x8` | host TX free-list head plus adjacent state | High head at `0x040087b0`; adjacent word unresolved |
| `0x040087b8..0x040089d8` | `0x220` | link-state root, link-map entries, per-link/TID sequence table | High anchors/strides; mixed semantics |
| `0x040089d8..0x04008a18` | `0x40` | JOIN/scan timer/control objects | Medium |
| `0x04008a18..0x04008ab8` | `0xa0` | WSM response/scan/indication scratch and control | Medium |
| `0x04008ab8..0x04008ad8` | `0x20` | BA/LMC global header | Medium |
| `0x04008ad8..0x04008bb8` | `0xe0` | heavily shared pending-list, BA, LMC, scheduler/radio state | High root use, incomplete overlays; must migrate as a closure |
| `0x04008bb8..0x04008e78` | `0x2c0` | 16 LMC message records, stride `0x2c` | High base/stride/count inferred from `0..15`; callbacks still vendor-owned |
| `0x04008e78..0x04008f18` | `0xa0` | unknown | Unknown, not allocatable |
| `0x04008f18..0x04008f48` | `0x30` | BA/link/event/timer state | Medium |
| `0x04008f48..0x04008f6c` | `0x24` | TALA accounting, exact qualified layout | High shape; address-changing migration rejected |
| `0x04008f6c..0x04008f80` | `0x14` | typed completion/context accounting anchor | High structural confidence; byte/halfword shared quarantine |
| `0x04008f80..0x04009080` | `0x100` logical | typed 64-entry completion-ring view crossing the physical prefix boundary | High shape; overlapping shared quarantine |
| `0x04008f80..0x0400906c` | `0xec` physical | first 59 completion-ring words and overlapping completion state | Shared opaque backing |
| `0x0400906c..0x04009080` | `0x14` physical | final five completion-ring words plus internal-context prefix overlay | Shared opaque backing |
| `0x04009080..0x040094d4` | `0x454` | typed internal TX context pool with initializer fields in three `0x170` records | High exact linker-owned fixed quarantine |
| `0x040094d4..0x040096dc` | `0x208` | opaque power-save family; observed address stride `0x104` | High base/family span and visible stride; record extent/count/overlap semantics remain uncertain |
| `0x040096dc..0x04009720` | `0x44` | typed power-save extension and beacon/TIM boundary state | High for decoded fields; small interior gaps remain opaque |
| `0x04009720..0x04009754` | `0x34` | HIF buffer/free-list and deferred-transfer roots | Medium |
| `0x04009754..0x04009928` | `0x1d4` | historical vendor HIF software/ring state; Rust owners now live in ITCM | High historical shape; fixed bytes remain quarantine for untranslated code |
| `0x04009928..0x0400993c` | `0x14` | MIC/HIF completion queue state | Medium; untranslated MIC path |
| `0x0400993c..0x04009a0c` | `0xd0` | typed PHY/RF/calibration/channel/gain state | High structural confidence; shared quarantine |
| `0x04009a0c..0x04009c44` | `0x238` | typed two-page IQ-calibration workspace and result words | High structural confidence from vendor loop; semantics remain shared quarantine |
| `0x04009c44..0x0400a000` | `0x3bc` | outside vendor fill; research margin only | Unknown, not allocatable |
| `0x0400a000..0x0400a500` | `0x500` | exception stacks | Outside proposed layout |
| `0x0400a500..0x0400c000` | `0x1b00` | system stack | Outside proposed layout |

### 3.3 Important initialized-data islands

Within `0x04000000..0x04002078`:

| Range/address | Shape | Meaning/status |
| --- | ---: | --- |
| `0x04000138..0x0400014c` | 10 `u16` | TX duration timing table; rebuilt by Rust startup; read through the rate map |
| `0x04000194..0x040001aa` | 22 bytes | rate encoding table |
| `0x040001aa..0x040001c0` | 22 bytes | rate attribute table |
| `0x04000260..0x04000288` | 10 visible `u32` words | completion-related initialized island; the class-6-visible word at `0x04000278` contains Thumb `0x00015427`, but not every word is proven to be a complete callback entry |
| `0x040002d8` | `u32` | hardware ring-cursor to software-slot translation |
| `0x040002dc..0x040002e4` | two 4-byte maps | pipe/queue status maps |
| `0x04000710..0x040007a4` | 37 `u32` | WSM command dispatch table |
| `0x04000804...` | table | AES transfer-class descriptors; class 6 TX CCMP, class 7 RX CCMP |
| `0x04000830..0x040009de` | `0x1ae` bytes | recovered AES mode-1 microcode, SHA-256 `211ad6ec...d881b4c` |
| `0x04000b60..0x04000c10` | two `0x58`-byte lists | initialized PHY gain register writes: ten ordered `{u32 address, u32 value}` pairs and a physical `{0xffffffff, 0xffffffff}` terminator pair per list |
| `0x04000c60..0x04000ca6` | `0x46` bytes | opaque initialized prefix containing the unresolved sentinel-walk list |
| `0x04000ca6..0x04000da8` | 43 records, stride `0x06` | initialized PHY gain sources: two overlapping 22-record address views; shared quarantine |
| `0x04000da8..0x04000e18` | `0x70` bytes | opaque initialized suffix prefix; the separate root at its start is not decoded |
| `0x04000e18..0x04000e48` | 12 `u32` | initialized IQ-calibration gain indices `1a,19,18,16,15,14,12,11,10,02,01,00`; shared quarantine |
| `0x04000e48..0x040010d4` | `0x28c` bytes | opaque initialized suffix; `0x04000e48` is an excluded unchecked lookahead and distinct register-list root |
| `0x040010d4..0x040010e4` | 4 `u32` | per-pipe duration-quantum MMIO pointers |
| `0x04001160..0x04001164` | one shared `u32` | fixed TX aggregate expiration delta; no known Rust consumer or writer |
| `0x04001164..0x040011ac` | six `0x0c` records | initialized debug-command descriptors: raw command-name, help-text, and handler words; shared quarantine |
| `0x040011ac..0x040011b4` | 8 bytes | HIF/control shadow and adjacent initialized state |
| `0x040011bc..0x0400123c` | 32 `u32` | IRQ callback table, reverse-indexed by IRQ |
| `0x0400123c..0x04001240` | one shared `u32` | fixed PHY watchdog count; wrapping increment and reset observed |
| `0x04001250..0x04001264` | five shared `u32` words | fixed multi-VIF beacon `TimerEntry`; surrounding `0x04001240..0x04001250` and `0x04001264..0x040012a0` remain opaque |
| `0x040012a0..0x040012c8` | `0x28` | exported AMPDU counters table |
| `0x04001420...` | mixed | control words; `0x04001428` host-download state, former PRNG at `0x0400142c`, timer offset at `0x0400143c` |
| `0x04001680...` | mixed | low-MAC shared root and four pipe families |
| `0x04001e6c` | `u32` within larger root | retry/drain control; known untranslated writers |
| `0x04001fcc` | `u32` | global scheduler/radio exclusion mask, extremely high fan-out |
| `0x04001fd4...` | event/timer root | scheduler pending/events and timing state |
| `0x04001fc0..0x04001fc8` | two `u32` | TALA defaults `0x15020210`, `0x19140f0a` |

The dense initialized image also contains opaque tables and code pointers. It must not be replaced by a blanket zeroed Rust struct.

---

## 4. Family-by-family ownership and translation audit

“Direct” below means a fixed literal or a resolved literal-pool pointer. “Computed” means indexing/offset arithmetic from a family root.

### 4.1 Scheduler, timers, and global event state

**Ranges:** primarily `0x04001fcc..0x04002018`, `0x0400218c..0x04002234`, plus timer objects embedded in VIF/PS/scan records.

**Known fields/shape:** the shared `SchedulerExclusionState` at `0x04001fcc` contains two exclusion words. `SchedulerEventIsland` spans `0x04001fd4..0x04002018` and names pending events, runtime flags, startup/analog/remap observations, three analog words, and the intrusive timer-list head while leaving unresolved bytes opaque. `ClockParameterIsland` is an exact `0x28`-byte TSF snapshot/conversion record at `0x0400218c`. The scheduler handler table is 32 bounded code-pointer words at `0x040021b4`. Retained timer objects use the common `0x14`-byte `TimerEntry` shape: next, previous-link, deadline, callback, and context.

**Readers/writers:** `sched_main_loop`, `evt_flags_set/clear`, `timer_start`, `timer_cancel`, `sched_arm_next_timer`, `task_22bc`, `mac_irq_handler`, HIF send/defer paths, MIC completion, scan/JOIN, channel switch, measurement, BA, power save, PHY tasks, and current Rust scheduler helpers. The literal-pool report found dozens of independent pointers resolving to `0x04001fd4`.

**Initialization:** low words around `0x04001fcc` are vendor COPY data and are explicitly reconstructed/cleared by Rust startup. The timer-list root at `0x04002014` is in vendor FILL/BSS and is explicitly reset before timer insertion. Handler entries are installed at runtime.

**Status:** typed mixed/shared quarantine. Rust startup, event claiming/publication, timer-list operations, host-TX gates, and PHY observations now derive their addresses from `dtcm.rs`, but untranslated tasks read and write the same words and embedded timer objects retain vendor callbacks. This family requires volatile access under IRQ/FIQ exclusion. It is not sound to expose `&mut SchedulerState` while interrupt code can mutate it.

**Cross-family pointers:** timer objects point to VIF, PS, scan, BA, and PHY contexts; the handler table contains code pointers; scheduler bits are the publication mechanism for HIF, TX completion, scan, JOIN/channel switch, measurement, and BA.

**Migration condition:** translate the scheduler loop, event-bit producers/consumers, timer intrusive list, callback registration, and every retained timer callback before moving the root or embedded timer entries.

### 4.2 Low-MAC, pipe, PAS, retry, and completion family

**Ranges:** roots at `0x04001680`, `0x04001d00..0x04001e74`, `0x04001f78..0x04001f90`, `0x04003678..0x04003e78`, `0x04008ad8`, and `0x04008f48..0x04008f80`.

**Exact computed records:** four pipe records are reached as:

```text
0x04001680 + pipe * 0x6c + 0xa0
```

Each contains a four-entry software ring and pointers to hardware TX rings at `0x09c60000 + pipe * 0x80`. PAS per-interface state is reached from `0x04003678` with stride `0x98` and large shared offsets such as `+0x470`, `+0x494`, `+0x4ac`, `+0x4bc`, `+0x4cc`, and `+0x4e0`.

**Readers/writers:** `txp_scheduler_run`, `txp_build_pipe_descriptor`, `txp_submit_to_pipe`, `txp_pipe_tx_done_retry`, `txp_pipe_tx_success`, `txp_pipe_tx_status`, `mac_irq_handler`, `txp_fn_2441`, `txp_fn_4155`, `txp_prepare_all_pipes_idle`, `txp_dequeue_pending`, `task_22bc`, `txp_program_pipe_hw`, PAS rate/retry/backoff routines, BA handlers, RX handlers, scan/JOIN/wake reprogramming, and translated Rust TX code.

**Known fixed writers:** `0x04001e6c` is written by untranslated `mac_irq_handler`, `txp_scheduler_run`, `txp_prepare_all_pipes_idle`, `txp_fn_4155`, `task_22bc`, and `txp_dequeue_pending`, in addition to Rust startup/translated retry code. Moving only Rust accesses split one state machine into two copies and stranded HIF response/completion progress.

**Initialization:** a mixture of vendor COPY defaults, vendor FILL zero, and runtime reconstruction. Pipe roots and maps are rebuilt in `mac::initialize_tx_pipe_state()` and `initialize_vendor_startup_state()`. Retained DTCM means startup must explicitly clear state that vendor cold boot would have initialized.

**Status:** all-or-nothing mixed family. Access must be volatile and generally protected by the MAC-domain/IRQ/FIQ guard. Plain atomics are not a substitute for hardware ordering or for code that runs with interrupts disabled; ARMv5 also constrains available atomic widths. Use critical sections plus volatile loads/stores and explicit write-buffer drains at publication points.

**Cross-family pointers:** pipe slots point to DTCM contexts and packet-RAM commands; pending lists point into host/internal contexts; completion records invoke class callbacks; PAS reads VIF fields and link state; TX completion calls power-save and BA routines; scheduler events initiate drains.

### 4.3 TALA accounting

**Range:** exactly `0x04008f48..0x04008f6c`, size `0x24`, alignment 4.

A faithful raw shape is:

```rust
#[repr(C)]
struct TalaAccountingRaw {
    growth_streaks: [u8; 2],       // +0x00
    reserved_02: [u8; 2],          // +0x02
    successes: [u32; 2],           // +0x04
    failures: [u32; 2],            // +0x0c
    cumulative_tries: [u32; 2],    // +0x14
    weighted_penalties: [u32; 2],  // +0x1c
}

const _: () = {
    assert!(core::mem::size_of::<TalaAccountingRaw>() == 0x24);
    assert!(core::mem::offset_of!(TalaAccountingRaw, successes) == 0x04);
    assert!(core::mem::offset_of!(TalaAccountingRaw, failures) == 0x0c);
    assert!(core::mem::offset_of!(TalaAccountingRaw, cumulative_tries) == 0x14);
    assert!(core::mem::offset_of!(TalaAccountingRaw, weighted_penalties) == 0x1c);
};
```

The field names above are supported by the current `tx_complete_tala_adapt` translation and the decompiled algorithm. They describe behavior, not ownership.

**Readers/writers:** visible static references center on `tx_complete_tala_adapt`, which addresses the arrays as negative offsets from `DAT_0000d40c = 0x04008f6c`. It also reads/writes VIF `+0x128`, PAS retry policy, global TALA parameters, PHY/pipe export state, scheduler flags, and completion records. An unresolved indirect or cross-program consumer remains plausible.

**Initialization:** vendor FILL zero; Rust startup also zeroes the region.

**Status:** fixed-address quarantine. Both a semantic Rust rewrite and an exact-layout, exact-pointer-arithmetic relocation to ITCM failed on the controlled ath9k workload. No sound ownership claim is possible.

### 4.4 Context pools and context accounting

#### Host WSM TX context pool

```text
base   0x04005a24
stride 0x170
count  30
end    0x04008544
head   0x040087b0
```

Startup helper `0x11d68(base, 30)` clears the in-flight byte at `0x04003e9e`, clears the free-list head, sets `ctx+4` to the previous head, initializes `ctx+0x70` to `0xff`, initializes the frame node at `ctx+0x54`, and points `ctx+0xa0` at packet RAM `0x09003678 + index * 0x54`.

**Readers/writers:** WSM TX allocation/free, `tx_abort_frames_for_vif`, encryption/MIC callbacks, pending-list insertion/removal, PAS scheduling, retry/completion, diagnostics, arbitrary MIB memory access, and current Rust host-TX translation.

**Status:** typed raw quarantine is possible now, but moving it is blocked by the entire host TX lifecycle and retained abort/diagnostic paths. Context bytes contain intrusive links, DTCM and packet-RAM pointers, flags, timers, frame metadata, and overlaid substructures. A semantic struct with independent ordinary fields would be premature.

A safe current representation would be deliberately raw:

```rust
#[repr(C, align(4))]
struct HostTxContextRaw {
    bytes: UnsafeCell<[u8; 0x170]>,
}

impl HostTxContextRaw {
    unsafe fn read_u32(&self, offset: usize) -> u32 {
        debug_assert!(offset <= 0x16c && offset % 4 == 0);
        unsafe { self.bytes.get().cast::<u8>().add(offset).cast::<u32>().read_volatile() }
    }

    unsafe fn write_u32(&self, offset: usize, value: u32) {
        debug_assert!(offset <= 0x16c && offset % 4 == 0);
        unsafe { self.bytes.get().cast::<u8>().add(offset).cast::<u32>().write_volatile(value) }
    }
}

const _: () = {
    assert!(core::mem::size_of::<HostTxContextRaw>() == 0x170);
    assert!(core::mem::align_of::<HostTxContextRaw>() == 4);
};
```

Typed newtypes such as `ContextAddress(u32)`, `FrameNodeAddress(u32)`, `PacketRamAddress(u32)`, and `HostBufferAddress(u32)` should be used at API boundaries, but the backing record remains volatile raw storage until overlays and all writers are retired.

#### Internal TX context pool

```text
0x04009080..0x040094d4
free head       +0x000
contexts        +0x004, three records
record stride   0x170
size            0x454
alignment       4
```

The current accepted Rust shape is accurate:

```rust
#[derive(Clone, Copy)]
#[repr(C, align(4))]
struct InternalTxContext([u8; 0x170]);

#[repr(C)]
struct InternalContextPoolState {
    free_head: u32,
    contexts: [InternalTxContext; 3],
}
```

The linker asserts the exact start and end. Runtime initializes the free list because the section is NOLOAD and has no fill record.

**Readers/writers:** translated internal TX code, retained `tx_abort_frames_for_vif`, high-image `mib_read_dispatch`, and possibly diagnostics. Therefore this is **typed but fixed-address quarantine**, not sound exclusive Rust ownership. Its address cannot move until those consumers are retired or bridged.

#### Accounting root

`0x04008f6c` is the completion/context accounting anchor. Historical migrations moved the completion ring, probe sequence, PAS count, and non-class-0 count into native Rust state. The class-0 counter remains fixed at `0x04008f71`. Nearby bytes and negative offsets are still used by TALA and completion code. Do not define one sound Rust struct over `0x04008f48..0x04008f80` while vendor code remains.

### 4.5 VIF records

```text
base   0x04003e98
stride 0x3b0
count  3
end    0x040049a8
```

Known offsets from `src/vif.rs`:

```text
+0x18 mode                 u8
+0x19 active               u8
+0x1a interface            u8
+0x1b role/state           u8
+0x1c flags                u32
+0x20 rate config          u32
+0x28 basic rates          u32
+0x2c/+0x2e link masks     u16/u16
+0x30 TX busy/count        u16 observed
+0x34 own MAC              [u8; 6]
+0x3c BSSID                [u8; 6]
+0x42 channel              u16
+0x44 radio-owner object   embedded/cross-linked
+0xb0,+0xc4,+0xd8 timers   embedded timer entries
+0xec SSID length          u32
+0xf0 SSID                 [u8; 32]
+0x110 DTIM                u8
+0x116 ATIM                u16
+0x118 beacon interval     u32
+0x124 RTS threshold       u32
+0x128 A-MPDU length       u16
+0x12a internal link       u16
+0x15c..+0x160 link masks  u16 fields
+0x184,+0x198 timers       embedded timer entries
```

**Readers/writers:** effectively every high-level firmware subsystem: WSM handlers, JOIN, scan restore, beacon/TBTT, RX key/PN/reorder, TX classification/scheduling/completion, BA, power save, P2P, measurements, channel switch, templates, link allocation, and both TCM dispatcher and main image.

**Initialization:** `fw_global_state_init` initializes two operating VIFs; `fw_timers_and_tasks_init` writes interface IDs for all three and timer objects for VIFs 0 and 1. Rust startup initializes a translated subset. Record 2 is a synthetic/P2P-device/scan slot and is not semantically identical to records 0 and 1.

**Status:** shared/mixed. The five proven embedded timers now use the common structural `TimerEntry` layout, while surrounding JOIN/link state remains opaque. The complete record still preserves intrusive ownership and fields mutated by interrupts and untranslated tasks. It needs volatile access behind a VIF/MAC-domain guard; no safe complete-record reference is exposed.

The count/stride can be asserted now:

```rust
#[repr(C, align(4))]
struct VifRecordRaw(UnsafeCell<[u8; 0x3b0]>);

#[repr(C)]
struct VifArrayRaw([VifRecordRaw; 3]);

const _: () = {
    assert!(core::mem::size_of::<VifRecordRaw>() == 0x3b0);
    assert!(core::mem::size_of::<VifArrayRaw>() == 0xb10);
};
```

Do not give safe references to fields until all writers for that field are translated.

### 4.6 Scan/JOIN/channel-switch family

**Anchors:** VIF records, PAS state, scheduler bits, `0x04008594..0x04008618`, `0x040089d8..`, `0x04008a18..`, `0x04008b58`, `0x04008b78`, and PHY state.

**Readers/writers:** `syn_scan_begin_request`, `syn_scan_set_state`, `syn_scan_restore_channel`, `syn_scan_finish_and_confirm`, `syn_scan_dwell_next`, `syn_scan_build_probe_req`, `syn_scan_program_channel`, scan stop/abort/probe completion, JOIN apply/retry/timeout/teardown, `task_22bc`, channel-switch tasks, and current Rust `scan.rs`, `vif.rs`, `join.rs`, `mac.rs`, and `phy.rs`.

**Initialization:** vendor FILL zero plus `fw_global_state_init`, timer initialization, WSM request copies, and runtime setup. `0x04008594..0x040085fc` is a copied WSM blob and `0x040085f8` is simultaneously the base of channel-switch control. This is direct evidence of semantic overlay or tail reuse.

**Status:** blocked all-or-nothing group. Rust scan plan storage is already native ITCM, but it deliberately still publishes selected vendor-visible flags (`0x0400860c`, scheduler bit 10, VIF/PAS fields). The remaining fixed bytes cannot move until retained scan/JOIN/channel-switch tasks are gone.

### 4.7 Link, BA, sequence, and LMC families

**Anchors:** `0x04003cc0` BA/pipe table, `0x04003e78` pre-VIF header, `0x040087b8` link state, `0x040087cc` link-map count, `0x04008890` sequence table, `0x040089d0` link bitmap, `0x04008ab8`, `0x04008ad8`, `0x04008bb8`, and `0x04008f18`.

**Computed access:** sequence numbers use:

```text
0x04008890 + internal_link * 0x20 + tid * 2
```

Link-map entries are reached by vendor/Rust code as:

```text
0x04003e98 + 0x4920 + index * 0x0c == 0x040087b8 + index * 0x0c
```

This cross-family expression is precisely why a source grep scoped to “link state” or “VIF fields” misses consumers.

**Readers/writers:** link allocation/free/state transitions, TX sequence assignment, RX lookup/reorder, BA add/del/session timeout, beacon/TIM, power save, HIF MIB reporting, LMC request/message pools, scan/JOIN, and TX completion.

**Initialization:** `lmc_init_link_tables`, `lmc_msg_pool_init`, `bab_init`, `link_state_init_all`, timer initialization, and runtime resets.

**Status:** mixed and cyclic. BA sessions point into contexts/VIFs; contexts point into packet RAM; link state affects PAS scheduling; completion updates BA and power save. Moving one table independently is unsafe.

### 4.8 Power-save family

```text
base   0x040094d4
stride 0x104
count  2
end    0x040096dc
```

`ps_per_vif_timers_init` initializes seven timer objects per record at offsets `+0x70`, `+0x84`, `+0x98`, `+0xac`, `+0xc0`, `+0xd4`, and `+0xe8`, stores the interface at `+0x43`, and initializes fields beyond the nominal `0x104` view through related roots. Runtime functions use this family from JOIN, scan, TX completion, beacon handling, UAPSD, listen interval, PS-Poll, and doze/wake paths.

**Status:** typed overlapping address schema, shared/mixed, volatile, and timer-cyclic. The `0x138` logical view now records proven controls, seven timer entries, and extension fields through `+0x136`, while the physical view starts remain `0x104` apart. Thus view 0 overlaps view 1, and view 1 extends into `PowerSaveHifBoundary` through `0x04009710`. This is deliberately not represented as two owned Rust records.

### 4.9 HIF, deferred transfer, MIC, and crypto control

**Ranges:** `0x04009720..0x0400993c`, with AES transfer-class tables in initialized low DTCM around `0x04000804`.

The historical HIF software record is well supported:

```text
0x04009754 mode
0x04009758 pending/credit count
0x04009774 rx buffer array start (historical +0x20)
0x040097f4/+0x97f? release/accounting area
0x040097fc software TX pointer queue, 64 entries
0x040098fc software TX queue producer
0x04009900 software TX queue consumer
0x04009914 descriptor TX producer
0x04009918 descriptor TX consumer
0x0400991c TX mask = 3
0x04009920 descriptor base = 0x0ab00100
0x04009924 TX length mask
```

The previous Rust `register_structs!` definitions gave `HifSoftwareState` size `0x1a8` at `0x04009754` and `HifState` size `0x2c` at `0x040098fc`, overlapping in the vendor model. The current Rust implementation correctly moved CPU-only queue/ring ownership to ITCM and retained only hardware descriptors and shared-SRAM addresses.

**Readers/writers remaining:** vendor HIF init/send/receive/defer/confirm functions, high-TCM MIB/debug code, AES/deferred transfer completion, and MIC queue functions around `0x04009928`. Current Rust HIF no longer needs these fixed CPU bookkeeping records, but untranslated code may still touch them.

**Status:** exact fixed-address shared quarantine. The retained HIF/MIC queues are structurally described only to preserve pointer/ring contracts and prevent false free-space claims; they are not restored as native Rust ownership. The final migration should delete retained consumers rather than relocate these historical bookkeeping records.

### 4.10 PHY/RF/calibration family

**Range/root:** `0x0400993c..0x04009a0c` is the densest known core; related initialized tables are at `0x04000dd0`, `0x04000de8`, `0x04000e18`, `0x04001088`, `0x04001098`, `0x04002730`, and `0x040034b0..`.

**Known fields:** many byte/halfword/word fields in `src/phy.rs`; selected examples:

```text
0x0400993c..0x0400994c    four IQ reference coefficient/scale words; byte 0x9945 is independently observed
0x0400994c..0x04009974    typed PHY profile/channel/transition state
0x04009974..0x040099ac    typed frequency, calibration, measurement, and retained-state block
0x040099ce                calibrated/target channel field
0x040099d4,+0x99d6        former channel power limits; now native ITCM
0x040099d8                selected rate-table pointer, still fixed
0x040099ec,+0x99f0        selected auxiliary table pointers, still fixed
0x040099f4                shared frequency-offset/scale word, still fixed
0x040099f8,+0x99fc        former PLL divider cache; now native ITCM
0x04009a04                channel/gain threshold, fixed mixed consumer
0x04009a06                former cached channel, now native ITCM
0x04009a08                force/calibration flag, fixed mixed consumer
```

**Readers/writers:** `phy_select_rate_tables`, `phy_apply_cfg_if_channel_match`, RSSI and TX-power helpers, frequency-offset calculation, channel switch/scan restore, wake, temperature compensation, RF initialization and calibration, `task_11ecc`, and translated Rust PHY routines.

**Known fixed writers:** `0x04009a04` and `0x04009a08` remain fixed. `0x04009a04` is read as the threshold at root `0x040099d4 + 0x30` by gain programming and has additional retained consumers. `0x04009a08` is adjacent force/calibration state consulted outside the translated PLL owner.

**Initialization:** vendor COPY tables, vendor FILL zero, SDD-derived configuration, and substantial runtime PHY setup. The initialized COPY image also contains two adjacent gain register-write lists at `0x04000b60..0x04000c10`. Each has ten ordered 32-bit address/value pairs followed physically by `{0xffffffff, 0xffffffff}`. `phy_build_gain_tables` selects `0x04000b60` by default and `0x04000bb8` only for profile 1, then `reg_write_list_apply` loads each address and value as separate 32-bit words, emits one ordered 32-bit MMIO write, and advances eight bytes until the address sentinel. The second terminator word is not read.

The default ordered pairs are `(0x0ab80400, 0x55f4282b)`, `(0x0ab80410, 0x0000003a)`, `(0x0ab80c20, 0x00000000)`, `(0x0aba803c, 0x77871f1b)`, `(0x0ab80404, 0x0010137a)`, `(0x0ab80414, 0x000000ab)`, `(0x0ab80418, 0x00000000)`, `(0x0ab8041c, 0x00080ea0)`, `(0x0abc8004, 0x00000109)`, and `(0x0aba8048, 0x031fd6f0)`, followed by `(0xffffffff, 0xffffffff)`. The profile-1 list is identical except its first value is `0x55f4292b` and its second value is `0x0000003b`, followed by the same remaining pairs and terminator. These data support only default versus profile-1 selection and ordered register writes.

The initialized source island `0x04000ca6..0x04000da8` is the exact physical union of 43 six-byte records. Each record has a shared `u8` selector at `+0`, an opaque shared `u8` at `+1`, and shared `u16` storage at `+2` and `+4` that retained code interprets as signed 16-bit lower and upper values. `phy_build_gain_tables` makes exactly 22 copies from either source view: view 0 begins at `0x04000ca6`, view 1 begins 21 records later at `0x04000d24`, and therefore view 0 record 21 and view 1 record 0 share the same physical record. View numbers have no assigned profile semantics. Vendor COPY initializes the interval but does not establish immutability against HIF/debug/vendor/IRQ/FIQ mutation.

The Rust representation leaves `0x04000c60..0x04000ca6` opaque, including the unresolved sentinel-walk list, and leaves `0x04000da8..0x040010d4` opaque. It exposes only checked addresses for 43 physical records and two overlapping 22-record views; it exposes no values, pointers, references, slices, iteration, writes, or profile meaning. The dedicated source/linked checker owns only the half-open initialized-source interval. Focused tests pin the record layout, overlap, boundaries, rejection cases, image size, and duration-pointer root. No production consumer or `phy.rs` constant changed.

**Status:** mixed. The accepted migrations prove that small tuples can move only when all references are confined to translated code. They do not imply that the surrounding root can be moved. The initialized gain lists have no known runtime writer, but container COPY as the only evidenced initialization writer does not prove immutability: vendor, IRQ/FIQ, and debug paths can still observe or mutate shared DTCM quarantine.

---

## 5. Ownership classifications

### 5.1 Sound native Rust-owned fields

These already live in ITCM and have translated ownership APIs:

* HIF queue/ring ownership behind `Transport`;
* Rust scan storage;
* completion ring;
* probe sequence;
* PAS active count;
* non-class-0 internal-context count;
* retry PRNG state;
* channel PLL cache;
* channel power limits;
* other ordinary Rust state and diagnostics.

They may still use `UnsafeCell` because interrupts, terminal exception publication, or cooperative subsystems share access, but they are no longer part of the fixed DTCM ABI.

### 5.2 Typed but fixed-address quarantine

* `0x04009080..0x040094d4` internal TX context pool;
* a possible raw `HostTxContextRaw[30]` view at `0x04005a24`;
* raw `VifRecordRaw[3]` at `0x04003e98`;
* exact `TalaAccountingRaw` at `0x04008f48`;
* partial HIF/PHY/PAS views used only for volatile access and layout assertions.

Typing these ranges can prevent wrong strides and offsets. It does not grant ownership or permission to move them.

### 5.3 Shared/mixed fields requiring `UnsafeCell` and volatile access

All scheduler/event/timer words, pipe/PAS state, VIF records, context records, BA/link/LMC state, power-save records, HIF/MIC legacy records, and fixed PHY roots while untranslated code exists.

Required discipline:

* no `&mut T` to storage concurrently visible to IRQ/FIQ/vendor code;
* `UnsafeCell<T>` or raw pointers for interior mutation;
* volatile access for every inter-language/shared-state load/store;
* IRQ/FIQ exclusion or a proven single-owner guard for multiword invariants;
* explicit hardware barriers only at hardware publication boundaries;
* no claim that Rust atomics alone make device-visible or vendor-shared records coherent.

### 5.4 Opaque reserved ranges

Known occupied but semantically incomplete ranges may be represented as opaque bytes only when their exact extent is supported, for example a raw `0x170` context record or the exact TALA padding bytes. The type must expose no safe field API.

### 5.5 Unknown ranges that must not be allocated

Every coarse-map region marked unknown, especially:

```text
0x040049a8..0x04005a24
0x04008544..0x04008594
0x04008e78..0x04008f18
0x04008f80..0x0400906c
0x040096dc..0x04009720
0x04009a0c..0x0400a000
```

and smaller gaps inside otherwise known families. Linker “holes” are not free space unless binary references and runtime snapshots prove them unused.

### 5.6 Stack regions outside the proposed layout

`0x0400a000..0x0400c000` must remain a separate linker region with hard assertions. It must never be represented as padding in a top-level software-state struct.

---

## 6. Concrete Rust semantics and APIs

### 6.1 Address newtypes

Do not pass arbitrary `u32` values between families:

```rust
#[repr(transparent)]
#[derive(Clone, Copy, Eq, PartialEq)]
struct DtcmAddress(u32);

#[repr(transparent)]
#[derive(Clone, Copy, Eq, PartialEq)]
struct ContextAddress(u32);

#[repr(transparent)]
#[derive(Clone, Copy, Eq, PartialEq)]
struct FrameNodeAddress(u32);

#[repr(transparent)]
#[derive(Clone, Copy, Eq, PartialEq)]
struct PacketRamAddress(u32);

#[repr(transparent)]
#[derive(Clone, Copy, Eq, PartialEq)]
struct ThumbFnAddress(u32);
```

Constructors should validate range, alignment, stride, and Thumb bit where applicable. Cross-family pointer fields should use these newtypes only after confirming the stored representation; until then retain raw `u32` accessors.

### 6.2 Volatile shared wrapper

A minimal wrapper may clarify intent:

```rust
#[repr(transparent)]
struct Volatile<T>(UnsafeCell<T>);

unsafe impl<T> Sync for Volatile<T> {}

impl<T: Copy> Volatile<T> {
    unsafe fn read(&self) -> T {
        unsafe { self.0.get().read_volatile() }
    }

    unsafe fn write(&self, value: T) {
        unsafe { self.0.get().write_volatile(value) }
    }
}
```

This wrapper does not supply mutual exclusion. APIs that update related fields must require a guard:

```rust
struct MacDomainGuard<'a> { /* IRQ/FIQ exclusion proof */ }

impl<'a> MacDomainGuard<'a> {
    unsafe fn pipe_state(&mut self, pipe: usize) -> PipeStateView<'_> { /* ... */ }
}
```

### 6.3 Bitfields and enums

Use transparent integer newtypes rather than Rust enums for values that may contain unknown bits:

```rust
#[repr(transparent)]
#[derive(Clone, Copy, Eq, PartialEq)]
struct SchedulerEvents(u32);

impl SchedulerEvents {
    const SCAN: Self = Self(1 << 10);
    const CHANNEL_SWITCH: Self = Self(1 << 12);
    const MEASUREMENT: Self = Self(1 << 13);
    const HIF_CONFIRM_FLUSH: Self = Self(1 << 20);
    const TX_COMPLETION: Self = Self(1 << 21);
    const HOST_MESSAGE: Self = Self(1 << 22);
    const TIMER_DUE: Self = Self(1 << 27);
}

#[repr(transparent)]
struct VifFlags(u32);

#[repr(transparent)]
struct ContextFlags(u32);
```

For state bytes with incomplete discriminants, avoid a closed `enum`. Use a newtype plus named constants and preserve unknown values.

### 6.4 Exact layout assertions

Every typed fixed range needs compile-time and linked-address checks:

```rust
const _: () = {
    assert!(core::mem::size_of::<InternalContextPoolState>() == 0x454);
    assert!(core::mem::align_of::<InternalTxContext>() == 4);
    assert!(core::mem::offset_of!(InternalContextPoolState, free_head) == 0);
    assert!(core::mem::offset_of!(InternalContextPoolState, contexts) == 4);
};
```

Linker assertions must independently check start, end, stack floor/top, and any preserved hardware-visible order. Unit tests alone do not verify placement.

### 6.5 Opaque storage

For exact occupied ranges with unknown semantics:

```rust
#[repr(C, align(4))]
struct OpaqueWords<const N: usize> {
    bytes: UnsafeCell<[u8; N]>,
}
```

Do not make this a safe `Deref<[u8]>`; all access must remain family-private and volatile. Unknown ranges without a proven exact extent should not receive even an opaque static because doing so would allocate/claim them.

---

## 7. Candidate top-level layouts

### 7.1 Candidate A: one packed `DtcmLayout`

Illustrative only:

```rust
#[repr(C, align(8))]
struct DtcmLayout {
    initialized_vendor_tables: OpaqueWords<0x2078>,
    runtime_02078: OpaqueWords<0x1618>,
    low_mac_pas: OpaqueWords<0x800>,
    pre_vif: OpaqueWords<0x20>,
    vifs: [VifRecordRaw; 3],
    // ... many exact paddings/families ...
    internal_context_pool: InternalContextPoolState,
    power_save: OpaqueWords<0x208>,
    // ... PHY tail ...
}
```

**Assessment:** not acceptable now.

1. It would claim unknown ranges as owned padding.
2. It cannot honestly express overlays such as the command-15 blob/channel-switch overlap.
3. It suggests one synchronization domain where several interrupt/task domains exist.
4. It would tempt safe field references into storage still written by untranslated code.
5. If relocated, all internal DTCM pointers, negative-offset roots, callback contexts, and cross-family calculations must change atomically.
6. If left at `0x04000000`, it does not retire the ABI; it merely describes it.

A single packed layout becomes possible only after every byte below `0x0400a000` is either decoded and owned, proven dead, or deliberately preserved as a complete immutable vendor table. At that point it may still be inferior to separate native family objects.

### 7.2 Candidate B: independently anchored quarantine structs

```rust
#[unsafe(link_section = ".dtcm.fixed.host_contexts")]
static HOST_CONTEXTS: HostContextPoolRaw = /* zero/NOLOAD runtime init */;

#[unsafe(link_section = ".dtcm.fixed.internal_contexts")]
static INTERNAL_CONTEXTS: SharedInternalContextPool = /* current shape */;

#[unsafe(link_section = ".dtcm.fixed.vifs")]
static VIFS: VifArrayRaw = /* quarantine */;
```

Each section has an exact linker address and a family-private volatile API. Unknown ranges remain outside all sections.

**Assessment:** feasible as an investigative typing strategy, but it is not the final migration. It improves stride/address checking without moving bytes. Every anchor must be justified by a complete range and must not overlap an existing fixed section.

### 7.3 Candidate C: native family objects in ITCM plus fixed bridge views

This matches the successful migration direction:

```rust
static NATIVE_TX_COMPLETION: SharedCompletionRing = /* ITCM BSS */;
static NATIVE_SCAN: SharedScan = /* ITCM BSS */;
static NATIVE_PLL_CACHE: SharedChannelPllCache = /* ITCM BSS */;

struct LegacyPasView { base: DtcmAddress }
struct LegacyVifView { base: DtcmAddress }
```

A family moves only after all producers and consumers use its native object. During translation, a narrowly defined bridge may mirror a value if and only if ordering and bidirectional ownership are explicit. Bridges are temporary and dangerous: the failed `0x04001e6c` experiment demonstrates that two writable copies without one authoritative owner deadlock progress.

**Assessment:** best staging architecture. The final atomic step removes the remaining bridge views and fixed linker sections together.

### 7.4 Recommended final target

* ordinary CPU-only family structs in ITCM;
* no general DTCM `.data`/`.bss`;
* no fixed DTCM pointers in generated Rust or retained vendor code;
* context pools moved to ITCM only after all context users are translated, unless measurement proves a true TCM timing requirement;
* immutable vendor tables either compiled as Rust `const` data in ITCM/rodata or copied to a documented native table object;
* DTCM below `0x0400a000` either unused or reserved only for a demonstrated architecture-specific purpose;
* stacks remain fixed at `0x0400a000..0x0400c000`.

---

## 8. Required vendor-code retirement and dependency graph

### 8.1 Core graph

```text
scheduler/event/timer core
  -> HIF deferred send/confirm
  -> MIC/AES completion
  -> TX pending and completion
  -> scan/JOIN/channel switch/measurement
  -> BA and power save
  -> PHY calibration tasks

VIF records <-> PAS/low-MAC <-> link/BA/LMC
     ^              |              |
     |              v              v
scan/JOIN <-> scheduler <-> TX contexts/completion
     |                               |
     v                               v
PHY/RF <------------------------- power save

host contexts <-> HIF request lifetime <-> crypto/MIC
      |                |                    |
      v                v                    v
pending list -> PAS scheduler -> pipe/FIQ completion -> TALA/BA/PS -> callback/free

internal contexts -> same pending/PAS/pipe/completion path -> class callback/free
```

### 8.2 All-or-nothing groups

#### Group 1: scheduler/timer/event kernel

Translate:

* `sched_main_loop`, task registration, `evt_flags_set/clear`;
* `timer_start`, `timer_cancel`, timer-list root, timer IRQ/arming;
* all retained callback targets stored in DTCM timer objects or handler tables;
* IRQ/FIQ publication rules for scheduler bits.

Until this group is native, nearly every other family remains externally mutable.

#### Group 2: host/internal TX lifecycle

Translate as one closure:

* WSM buffer allocation/free and borrowed HIF request lifetime;
* context initialization and both free lists;
* crypto/MIC completion callbacks;
* `txq_list_insert/remove` and pending-list task `task_b88e`;
* `txp_program_pipe_hw`, PAS gates, `txp_scheduler_run`;
* descriptor construction/publication;
* FIQ `mac_irq_handler`, retry/give-up/success paths;
* completion FIFO drain, class callback dispatch, context return;
* abort/flush/teardown/diagnostic consumers.

Moving the pools before this closure is complete is not sufficient.

#### Group 3: VIF/PAS/link/BA/power-save

Translate:

* VIF init/JOIN/RESET/START and all embedded timers;
* PAS rate tables, backoff, queue state, per-link masks;
* link allocation/state/sequence tables;
* BA session allocation, timers, RX reorder and TX completion hooks;
* power-save, UAPSD, PS-Poll, beacon/TBTT, wake/doze logic.

These form a cycle through TX legality and completion. Partial movement risks frames that are queued under one copy of state and completed under another.

#### Group 4: scan/JOIN/channel switch/measurement/PHY

Translate:

* all scan state-machine tasks and timers;
* JOIN retry/timeout/teardown and scan restoration;
* channel-switch tasks and radio-owner arbitration;
* measurement state at `0x04001fc8`;
* retained PHY channel/rate-table/RSSI/frequency/temperature/calibration paths;
* `task_11ecc` and wake/restore callers.

Only then can fixed PHY roots `0x0400994c...` and fields `0x04009a04`/`0x04009a08` move.

#### Group 5: HIF/MIC/deferred engine

The CPU-only Rust HIF state is already native, but retained code still references fixed HIF/MIC/deferred-transfer records. Translate or remove:

* vendor HIF send/receive/confirm/defer routines that may still run;
* MIC queue and completion;
* AES serialized queue if hardware crypto is retained;
* high-TCM MIB/debug consumers of context/HIF state.

### 8.3 Proposed translation order

1. **Generate complete binary reference maps and runtime snapshots.** No address movement.
2. **Own scheduler and timer mechanics.** Replace code-pointer tables and embedded callbacks.
3. **Close the MAC FIQ and TX completion path.** This makes context and completion ownership tractable.
4. **Translate host/internal context lifecycle and pending/PAS scheduler.** Keep fixed addresses until closure passes.
5. **Translate VIF/PAS/link/BA/power-save together.** Resolve cycles under one MAC-domain owner.
6. **Translate scan/JOIN/channel-switch/measurement.** Remove remaining scheduler/VIF/PHY cross-writers.
7. **Translate retained PHY/RF consumers.** Then move `0x04009a04`, `0x04009a08`, and rate-table pointer/scale state.
8. **Retire high-TCM MIB/debug fixed consumers and arbitrary memory diagnostics in production.**
9. **Perform one final atomic relocation/removal:** switch all roots to native Rust objects, remove DTCM COPY/FILL compatibility assumptions, remove fixed linker anchors, and enable hard binary gates.

Steps 2–8 may be committed as translations while preserving addresses. The final address change should remain one atomic migration.

---

## 9. Linker, loader, and build-tool implications

### 9.1 Current linker facts

`link-main-low.x` currently:

* bounds ITCM to observed `0x1c000`;
* places one `.dtcm.state` shared-quarantine object at `0x04000000..0x0400a000` with no program header;
* exports actual ELF symbols for the DTCM object and internal-pool member boundaries at `0x04009080`, `0x04009084`, and `0x040094d4`;
* asserts stack floor/top;
* keeps packet RAM as NOLOAD, no-program-header ownership objects;
* puts ordinary Rust `.data`/`.bss` in ITCM.

The context pool is NOLOAD and absent from `PT_LOAD`; startup must initialize every required field. Any future fixed quarantine section must make COPY/FILL/NOLOAD intent explicit rather than relying on linker side effects.

### 9.2 Loader COPY/FILL policy

`download.rs` accepts section destinations only in:

```text
ITCM       0x00000000..0x0001c000
DTCM       0x04000000..0x0400a000
high SRAM  0xfff00000..0xfff14000
```

This correctly excludes stacks and aliased high DTCM. The packer converts `PT_LOAD` file bytes to COPY and memory tails to zero FILL. NOLOAD fixed sections outside `PT_LOAD` receive neither operation.

For a final migration:

* native ITCM structs naturally become ordinary `.data`/`.bss` in existing load segments;
* no new DTCM FILL should be introduced for CPU-only state;
* immutable tables moved from vendor DTCM need deterministic COPY/rodata semantics;
* any retained fixed quarantine during transition must be initialized explicitly and tested after warm reload;
* bootstrap stack use at `0x0400c000` must remain non-overlapping with accepted destinations below `0x0400a000`.

### 9.3 Bootstrap/runtime overlays

The sectioned bootstrap starts at `0x08000000`, relocates to packet RAM, and uses the DTCM system stack with top `0x0400c000`. The main-image stack and exception stacks occupy the same physical top region after transfer of control. They are temporal users of the stack partition, not fields to copy.

The command-15 blob/channel-switch overlap and retained packet/HIF overlays show that apparent adjacent ranges can have phase-dependent roles. A final map must record lifetime as well as address.

### 9.4 Required deterministic gates

The current `check-address-literals.py` is focused on packet RAM/MMIO. Add a DTCM-specific gate before migration:

1. **Rust source gate:** reject every `0x0400xxxx` literal outside a small owner manifest, including low-23-bit or offset forms where applicable.
2. **Resolved literal-pool gate:** scan the linked Rust ELF and any retained vendor blobs for words in `0x04000000..0x0400a000`.
3. **Instruction gate:** detect synthesized constants (`mov`/`orr`, `add base,#offset`) and PC-relative loads whose resolved value enters DTCM.
4. **Computed-root gate:** reject known legacy bases and derived forms, including `VIF_BASE + 0x4920`, negative offsets from `0x04008f6c`, and context base/stride arithmetic.
5. **Section gate:** reject all allocatable DTCM sections except explicitly approved fixed quarantine; after final migration reject all lower-DTCM sections.
6. **Packer gate:** reject COPY/FILL destinations in DTCM after final migration, except a deliberately documented immutable table if one remains.
7. **Stack gate:** retain exact assertions for `0x0400a000` and `0x0400c000`; reject any section or fill intersecting stacks.
8. **Pointer-value gate:** inspect native initialized data for stored legacy DTCM pointers, not only code literals.
9. **Callback gate:** reject handler/timer callback tables containing vendor code addresses after their subsystem is declared native.
10. **Warm-reload gate:** verify deterministic initialization with DTCM prefilled with a nonzero pattern in an emulator/model or diagnostic image.

A manifest should classify every surviving reference as `table`, `fixed-quarantine`, `stack`, or `forbidden`; an unclassified hit fails the build.

---

## 10. Failed experiments as evidence

### 10.1 `0x04001e6c` retry/drain split

Moving the Rust-visible retry/drain word while retained code continued writing the fixed address caused a panic in HIF response publication with 11 host TX buffers occupied and one pending TX. This is a direct demonstration that the word is a synchronization node, not an isolated flag. The failure mode was loss of progress rather than an immediate exception, which is exactly what a split producer/consumer state machine predicts.

Implication: a field is movable only when every reader and writer—including IRQ/FIQ paths and computed references—has one authoritative storage location.

### 10.2 `0x04003a6d` scheduler/RX gate

The migration was withdrawn because `task_22bc`, `tx_flush_all_queues`, channel-switch tasks, and other retained paths write it. Its neighborhood is heavily shared MAC state rooted at `0x04003a58`; a byte-sized type does not make it an independent ownership unit.

Implication: migration boundaries must follow state-machine closure, not scalar field width.

### 10.3 TALA semantic migration

The first patch replaced fixed words with a two-interface Rust structure. It associated quickly, but when the evaluation window activated, TCP fell to zero, UDP dropped to 449 Kbit/s, ath9k recorded 1,354 TX failures, and final ping was 0/20 without panic or exception.

Possible causes at that stage included incorrect interface count, changed padding, changed access order/code generation, a hidden consumer, or address/timing sensitivity.

### 10.4 TALA exact-layout migration

The second patch preserved:

* exact `0x24` size;
* exact offsets, including the two unused bytes;
* exact raw pointer arithmetic;
* the same volatile helper access pattern;

and changed only the backing address to Rust-owned ITCM. It passed one router workload but failed the controlled ath9k workload with 1,384 TX failures and no completed throughput report.

This rules out the obvious struct-shape and interface-count explanations. Remaining hypotheses are:

1. an indirect consumer or cross-program consumer still uses the fixed DTCM address;
2. another field aliases or is addressed relative to `0x04008f6c` in a way the audit missed;
3. DTCM versus ITCM data-access timing changes a high-rate completion race;
4. compiler placement changed instruction scheduling or critical-section timing enough to expose an existing race;
5. memory aliasing/ordering assumptions differ between the fixed and native paths;
6. the family has incomplete semantics even though the visible arithmetic matches.

The right response is not to select a more elaborate Rust type. It is to instrument all accesses and isolate whether fixed identity or timing is causal.

### 10.5 Internal context pool address experiments

Moving the pool to `0x0400c000` stalled scan because that address aliases `0x04000000`. Moving it to ITCM produced one healthy run and one stall, but controls had the same distribution, so it did not prove an address dependency. The accepted improvement was therefore only typing and linker-checking at the original address.

Implication: separate “shape/ownership improvement” from “address movement,” and demand controlled repeated hardware evidence for the latter.

---

## 11. Adversarial feasibility assessment

### 11.1 What can be typed now

With high confidence:

* internal context pool (`0x454` bytes);
* host context records as raw `0x170`-byte aligned cells and a 30-entry array;
* VIF records as raw `0x3b0`-byte cells and a three-entry array;
* TALA exact `0x24` raw shape;
* scheduler callback table `[u32; 32]` as code-address quarantine;
* completion callback table around `0x04000260` as a bounded table once its exact count is confirmed;
* HIF historical records for forensic comparison only;
* selected fixed tables with exact lengths (rate/timing/AES microcode);
* two power-save stride views, but not yet a trustworthy semantic record;
* PHY selected fields as volatile accessors, not one semantic struct.

### 11.2 What remains blocked

* moving either context pool;
* moving or safely owning VIF/PAS/pipe state;
* TALA relocation;
* scheduler/event/timer relocation;
* fixed class-0 accounting at `0x04008f71`;
* HIF/MIC/deferred-transfer fixed roots while vendor paths may run;
* scan/JOIN/channel-switch fixed control;
* BA/link/LMC state;
* power-save records;
* PHY root and fixed writers at `0x04009a04`/`0x04009a08`;
* unknown ranges and all apparent gaps.

### 11.3 Minimum translation closure for a credible one-shot migration

At minimum, the final atomic change requires native implementations of:

1. scheduler event loop and timers;
2. IRQ/FIQ dispatch and complete MAC TX completion/retry path;
3. host/internal context allocation, pending, PAS, pipe, callback, abort, and free lifecycle;
4. VIF/JOIN/RESET/START plus PAS/link/BA/power-save state machines;
5. scan/channel-switch/measurement and their timers;
6. remaining HIF/MIC/deferred engine consumers;
7. PHY/RF functions that read/write the fixed root;
8. production MIB/debug paths that can inspect or mutate fixed state.

Anything less leaves a fixed pointer or hidden writer that can invalidate native ownership.

### 11.4 Likely failure modes

* silent queue stall rather than crash;
* host credits leaked because request lifetime and context lifetime diverge;
* retry and completion consumers observe different copies;
* timer intrusive lists follow stale DTCM pointers after warm reload;
* callback tables retain vendor addresses absent from the Rust image;
* VIF active publication occurs before dependent fields are initialized;
* BA/power-save completion updates race context reclamation;
* DTCM/ITCM timing changes expose high-rate completion races;
* hidden base-plus-offset reference writes an old address;
* immutable initialized tables are accidentally zeroed;
* unknown “gap” allocation overwrites an overlaid or phase-dependent object;
* a section or loader fill enters the stack partition;
* use of `0x0400c000` corrupts `0x04000000` through aliasing;
* safe Rust references are created while untranslated code mutates the same bytes, causing language-level undefined behavior even if hardware behavior appears correct.

### 11.5 Investigation plan that still targets one final atomic migration

1. Produce a versioned CSV/Markdown reference inventory from literal pools and function bodies.
2. Add boot-time checksums/snapshots for DTCM in bounded coarse blocks, then narrower hot families.
3. Instrument TALA fixed and relocated variants to compare every load/store sequence and completion timing; use external capture/driver evidence to avoid layout-changing firmware diagnostics where possible.
4. Decode exact record extents and overlays for scheduler, PAS, VIF, PS, and PHY roots.
5. Translate closures while preserving fixed addresses and exact runtime initialization.
6. For each family, require a “no retained reader/writer” proof generated from both main and high-TCM images.
7. Add native APIs and ownership guards only after the proof; keep raw volatile quarantine before it.
8. Build a final migration branch that moves all remaining families at once, removes fixed literals/anchors, and changes loader/linker policy atomically.
9. Qualify exact parent and candidate under repeated cold resets and controlled APs before accepting.

---

## 12. Implementation review checklist

### Reverse-engineering closure

* [ ] Every DTCM literal-pool pointer in the main image is resolved and assigned to a family.
* [ ] Every DTCM pointer in the high-TCM image is resolved and assigned.
* [ ] Known synthesized/computed roots and negative-offset accesses are included.
* [ ] Each moved field has a complete reader/writer list, including IRQ/FIQ and callbacks.
* [ ] Cross-family pointers and intrusive-list links are documented.
* [ ] Unknown ranges remain unallocated.
* [ ] Overlays and phase-dependent reuse are explicitly represented.
* [ ] Arbitrary MIB memory read/write and diagnostic consumers are retired or excluded from production.

### Rust layout and ownership

* [ ] Every `#[repr(C)]` type has `size_of`, `align_of`, and `offset_of!` assertions.
* [ ] Array counts and strides match vendor evidence.
* [ ] Closed enums are used only where every discriminant is known; otherwise integer newtypes preserve unknown values.
* [ ] Shared fields use `UnsafeCell`/volatile access.
* [ ] Multiword invariants require an IRQ/FIQ or domain guard.
* [ ] No safe `&mut` exists for storage still visible to untranslated code.
* [ ] Packet RAM/shared SRAM/MMIO addresses use separate newtypes and are not folded into DTCM ownership.
* [ ] Native CPU-only state remains in ITCM.

### Linker and loader

* [ ] No output section intersects `0x0400a000..0x0400c000`.
* [ ] Stack floor/top assertions remain exact.
* [ ] No loader destination reaches `0x0400c000` or above.
* [ ] COPY/FILL/NOLOAD intent is explicit for every remaining DTCM section.
* [ ] Runtime initialization is deterministic after warm reload and nonzero retained contents.
* [ ] The internal context pool remains exactly `0x04009080..0x040094d4` until all fixed consumers are retired.
* [ ] Final migration removes obsolete DTCM COPY/FILL compatibility assumptions and fixed anchors together.

### Generated gates

* [ ] Source-level `0x0400xxxx` ownership check passes.
* [ ] Linked-image literal-pool scan passes.
* [ ] Computed-root/synthesized-constant scan passes.
* [ ] Initialized native data contains no legacy DTCM pointers.
* [ ] Callback/timer tables contain only valid native code pointers.
* [ ] No new DTCM PT_LOAD or fill record appears unexpectedly.
* [ ] Packet-RAM and MMIO ownership gates still pass unchanged.
* [ ] Stack-depth checks still pass for normal and exception chains.

### Hardware qualification

* [ ] Use a bounded cold SDIO reset after every fatal experiment.
* [ ] Run exact parent controls interleaved with candidates.
* [ ] Test at least two AP/peer profiles, including the controlled ath9k case that exposed TALA.
* [ ] Verify association latency, TCP, UDP loss/throughput, and final ping.
* [ ] Verify zero leaked host buffers, zero pending contexts, and empty completion/pending lists.
* [ ] Verify scan, JOIN, RESET, channel switch, wake, and repeated traffic cycles.
* [ ] Exercise management/internal TX and ordinary class-0 data TX.
* [ ] Exercise retry/give-up, BA, power-save/UAPSD, and teardown paths.
* [ ] Check exception/fatal diagnostics and external MAC captures.
* [ ] Repeat after warm firmware reload to expose retained-state bugs.
* [ ] Repeat after true cold reset to distinguish retained-state from deterministic logic failures.
* [ ] Do not accept a migration based on one clean run or on absence of a panic.

---

## Final recommendation

Retain the implemented monolithic object only as a **fixed shared-quarantine ABI view** of `0x04000000..0x0400a000`; do not reinterpret it as a movable native state object or expose safe complete-record references. Its value is structural: one linker allocation, opaque occupied families, exact boundary checks, and narrow raw views. Continue translating complete state-machine closures while preserving fixed addresses. When the scheduler/timer core, TX lifecycle, VIF/PAS/link/BA/power-save cycle, scan/JOIN/channel-switch paths, remaining HIF/MIC consumers, and PHY/RF fixed-root users are all native, replace the quarantine with independently owned native families and remove every legacy DTCM root and loader assumption atomically.

The internal context member view remains the model for what is safe today: improve type shape, direct symbol/addend addressing, and linker verification without claiming exclusive ownership or address mobility. The failed TALA exact-layout experiment remains the model for what must not be inferred: exact bytes and source arithmetic do not prove that a black-box ABI address is irrelevant.

---

## Implementation appendix: first linker-owned structural candidate

The first candidate is implemented in `xr819-firmware/src/dtcm.rs` and deliberately changes the conclusion of Candidate A only in one narrow respect: a monolithic type is now used as a **fixed-address quarantine ABI map**, not as a movable or exclusively owned Rust object. It preserves every retained address and does not translate vendor behavior. This makes the linker allocation and the Rust structural assertions agree while retaining the ownership caveats established above.

### A.1 Linker object and initialization contract

`DTCM_STATE` is a `#[repr(C, align(4))]` object of exactly `0xa000` bytes in the single `.dtcm.state` output section:

```text
0x04000000..0x0400a000  .dtcm.state, SHT_NOBITS | SHF_ALLOC, NOLOAD, :NONE
0x0400a000..0x0400c000  unchanged exception/system stack partition
```

`link-main-low.x` now has one `DTCM_STATE` memory region rather than lower-legacy/context-pool/upper-legacy regions. The output section is explicitly outside every program header. The linker still asserts the exact stack floor and top and additionally asserts the DTCM object size, the internal free head at `0x04009080`, first internal context at `0x04009084`, and pool end at `0x040094d4`.

The Rust static stores `MaybeUninit<DtcmLayout>` inside `UnsafeCell`. Consequently, the Rust initializer does not provide COPY or zero-fill semantics. Retained vendor COPY bytes below `0x04002078` are left untouched. `platform::initialize_runtime_state()` continues to clear exactly `0x04002078..0x04009c44` and reconstruct the same selected low-DTCM startup words. No loader record was added.

### A.2 Exact candidate field sequence

Every entry below is a private field with compile-time `size_of!`, `align_of!`, and top-level `offset_of!` assertions. Named opaque families contain private `UnsafeCell<MaybeUninit<[u8; N]>>` storage and expose no byte slice or allocation API.

| Offset | Size | Candidate type | Meaning |
| ---: | ---: | --- | --- |
| `0x0000` | `0x2078` | `InitializedVendorImage` | retained vendor COPY image, including tables, callbacks, AES data, and opaque initialized words |
| `0x2078` | `0x114` | `RuntimePrefix` | early vendor-zeroed context/backoff/debug state |
| `0x218c` | `0x28` | `ClockParameterIsland` | typed TSF snapshots, counter cache, conversion controls, and correction offset |
| `0x21b4` | `0x80` | `SchedulerHandlerTable` | 32 bounded shared raw callback words |
| `0x2234` | `0x127c` | `PreConfigurationTables` | PHY/template/beacon/filter state |
| `0x34b0` | `0x130` | `SddConfigurationTables` | SDD-derived channel/gain/profile tables |
| `0x35e0` | `0x90` | `WakeContextState` | mixed wake/context state |
| `0x3670` | `0x4` | `DurationSources` | retained duration-source halfwords |
| `0x3674` | `0x4` | `PreLowMacWord` | occupied undecoded word |
| `0x3678` | `0x800` | `LowMacPasFamily` | shared low-MAC/PAS/rate/link/pipe/queue family |
| `0x3e78` | `0x20` | `PreVifHeader` | pre-VIF link/aggregation header |
| `0x3e98` | `0xb10` | `VifRecords` | three opaque `VifRecord` values at stride `0x3b0` |
| `0x49a8` | `0x107c` | `PostVifQuarantine` | occupied VIF-adjacent unknown state |
| `0x5a24` | `0x2b20` | `HostTxContexts` | 30 opaque host contexts at stride `0x170` |
| `0x8544` | `0x50` | `PreCommandQuarantine` | eight peer-pipe records, four management counters, scan channel, and pending root |
| `0x8594` | `0x84` | `CommandChannelSwitchOverlay` | one deliberately opaque overlay: command blob at `+0x00` and channel-switch view at `+0x64` overlap |
| `0x8618` | `0x180` | `LmcControlRoots` | encryption free-list header plus first 31 duplicate-cache records; record 31 crosses into `0x8798` |
| `0x8798` | `0x18` | `HostContextAccounting` | host-context/duplicate-cache accounting |
| `0x87b0` | `0x8` | `HostContextFreeList` | shared free head plus unresolved adjacent word |
| `0x87b8` | `0x220` | `LinkAndSequenceState` | link map/state and per-link/TID sequences |
| `0x89d8` | `0x40` | `JoinScanControl` | JOIN/scan timers and controls |
| `0x8a18` | `0xa0` | `WsmResponseScratch` | WSM response/scan/indication scratch |
| `0x8ab8` | `0x20` | `BaLmcHeader` | BA/LMC global header |
| `0x8ad8` | `0xe0` | `PendingBaLmcState` | shared pending-list/BA/LMC/scheduler/radio state |
| `0x8bb8` | `0x2c0` | `LmcMessages` | 16 opaque records at stride `0x2c` |
| `0x8e78` | `0xa0` | `BaSessions` | four BA session records, stride `0x28` |
| `0x8f18` | `0x30` | `BaLinkEventState` | BA/link/event/timer state |
| `0x8f48` | `0x24` | `TalaAccounting` | exact semantic arrays and reserved bytes; still shared quarantine |
| `0x8f6c` | `0x14` | `ContextCompletionPrefix` | completion/context accounting anchor, including class-0 count at `+5` |
| `0x8f80` | `0xec` | `PreInternalContextQuarantine` | occupied undecoded bytes |
| `0x906c` | `0x14` | `InternalContextPrefix` | internal-context global/header prefix |
| `0x9080` | `0x454` | `InternalContextPoolState` | free head plus three typed opaque `0x170` contexts |
| `0x94d4` | `0x208` | `PowerSaveFamily` | one opaque family; `0x104` is retained only as an observed address/view stride with uncertain extent and overlap semantics |
| `0x96dc` | `0x44` | `PowerSaveHifBoundary` | occupied PS/HIF boundary |
| `0x9720` | `0x34` | `HifBufferState` | host-message free ring plus pending/completed transfer queue |
| `0x9754` | `0x1d4` | `LegacyHifSoftwareState` | coalesce timer, RX buffers, 64-entry TX queue, and transport roots |
| `0x9928` | `0x14` | `MicCompletionState` | MIC pending/completed transfer queue |
| `0x993c` | `0xd0` | `PhyCoreState` | typed four-word calibration-reference prefix plus retained PHY/RF core |
| `0x9a0c` | `0x238` | `PhyTail` | remaining PHY and unknown vendor-zeroed tail |
| `0x9c44` | `0x3bc` | `ResearchMargin` | occupied quarantine beyond the vendor zero-fill endpoint |

### A.3 Semantic fields versus opaque storage

High-confidence count/stride semantics are encoded for VIF records, host contexts, internal contexts, LMC messages, the scheduler table, and the exact TALA arrays. The power-save area is intentionally different: only the `0x208` family span and observed `0x104` address stride are retained, without asserting two owned records. `InitializedVendorImage` also asserts the documented initialized islands: duration timing at `0x0138`, rate encoding/attributes at `0x0194`/`0x01aa`, ten visible completion-related words at `0x0260`, ring/status maps at `0x02d8`/`0x02dc`, command dispatch at `0x0710`, AES descriptors/microcode at `0x0804`/`0x0830`, two PHY gain register-write lists at `0x0b60`/`0x0bb8`, duration-quantum pointers at `0x10d4`, the TX aggregate expiration delta at `0x1160`, six initialized debug-command descriptors at `0x1164`, HIF shadow at `0x11ac`, IRQ callbacks at `0x11bc`, AMPDU counters at `0x12a0`, control words at `0x1420`, the low-MAC initialized root at `0x1680`, retry/TALA anchors within that root, scheduler exclusion words at `0x1fcc`, and the event island at `0x1fd4`. TALA uses shared scalar wrappers at offsets `0x00`, `0x04`, `0x0c`, `0x14`, and `0x1c`. The internal pool retains its exact `free_head`/`contexts` split at offsets `0` and `4`.

Everything with unresolved internal extent or ownership is a private opaque family. In particular, the command/channel-switch overlap is one opaque overlay rather than two fields, and the historically overlapping HIF views are represented as one quarantine family. The power-save stride is asserted but no field API is provided because the decompilation's larger relative offsets remain ambiguous. Unknown ranges are represented only because the complete fixed ABI object necessarily spans them; no API names them as padding, free space, or allocatable capacity.

### A.4 Pointer and ownership discipline

`DtcmAddress` validates addresses against the state interval and preserves separate address-domain semantics. The module exposes no `&mut DtcmLayout`, no safe references to family records, and no dereference/slice API for opaque bytes. The only live typed view currently required is the internal TX pool, exposed crate-privately as raw pointers. `tx.rs` derives the free-head and context addresses from the `DTCM_STATE` symbol and then performs the same volatile initialization writes as before.

This is therefore a **shared quarantine ABI view**. `#[repr(C)]` and semantic field names document byte identity and improve deterministic checking; they do not assert that Rust is the sole writer, that ordinary references are sound, or that any family can move independently.

### A.5 Deterministic gates

`tools/check-dtcm-layout.py` verifies the exact section set, `SHT_NOBITS | SHF_ALLOC` type, address, size, absence from every alias-inclusive DTCM `PT_LOAD`, actual ELF linker symbols for state/context boundaries, exact stack exclusion, and separation from packet RAM. When given a packed image, it also parses COPY/FILL records, rejects destinations anywhere in `0x04000000..0x04010000`, and requires ENTRY to be the terminal record. `pack-sectioned-elf.py` rejects every allocatable section in that alias-inclusive range regardless of name except the exact approved `.dtcm.state` policy. Both `tools/check.sh` and `tools/build-ota-image.sh` run the new gate.

### A.6 ARM code-generation comparison

The first monolithic formulation derived internal-pool fields through the top-level Rust object and caused a blanket `+0x50` shift at the first affected TX/probe functions. The final formulation exports `__dtcm_context_pool_start` from the actual `DTCM_STATE` object and uses raw symbol/addend pointer arithmetic on ARM. This restores direct absolute-address materialization while keeping the pool physically inside `.dtcm.state`; no separate storage or safe complete-record reference was introduced.

Whole-function bytes were compared against `/tmp/xr819-b6-hif-startup.elf`. Exact identity is not practical under the required ownership/API changes and whole-crate size-LTO: seven streams retain differences. Their final sizes and SHA-256 hashes are recorded so the residual is explicit rather than normalized away:

| Function | Parent size/hash | Candidate size/hash |
| --- | --- | --- |
| `HostSchedulerReservation::publish_in_batch` | `0x1e8` / `1529ccc8b90d86a71509298d11f4e751a07fb12b3938fd80c574967e63dd4dcd` | `0x1f0` / `3c6ec23111d57f7fee391cea0fbe415b5d68b9c2ec0b37bb8c510aba66bf54eb` |
| `emit_prepared_probe_descriptor` | `0x2d0` / `15bcfc31fb3b7dd8cf259b2dbd535419f8423c81614da39d209662fc339d2ef4` | `0x2f0` / `a6eb2121eb2577c6ae661e23a6c3a1151116b609af0f175c9b5d10bcbf06e924` |
| `enter_mac_fatal_quiescence` | `0xe0` / `b43c679f2dda323e43118646827cd17aa4dca1df3833f957ad7953c96994c9fc` | `0xdc` / `388de11ec9238faf0300352b370a9a29cbc0ad468f7e5f73f4c48198cfa1c3a2` |
| `initialize_internal_pool` | `0x6c` / `4fcadf5c2116793227d27ced6dfad226339014d78780323b1a89065df592f15a` | `0x5c` / `0f62bebcc495df356df976b7da294ebc5676574e1b813b4ee2901a01143792c0` |
| `prepare_probe_context` | `0x284` / `137dbd2e516a633ec650e376939c3b68df9ae784b0941300864cd2b91e817a5f` | `0x290` / `36be4e1c275eaf9a9b46e9070db7fd38507ad7f8f09e5ae936873a1f2a6e2589` |
| `release_wsm_context_address` | `0xac` / `76a57832ba5479a2a9bc13683e91b3cc9fb8d8614bb63c7260bfb3078bcb9185` | `0xb4` / `de94a20a43922c17852d88e29e6503fad070a155a3f5ff9d86d84837ffb91a7d` |
| `service_single_probe_runtime_inactive` | `0x1648` / `11ab52244cb1b6f4b8ac8077beb4fd7373044098e00bc31c7516332915bafbb2` | `0x1654` / `af31991d1e8b5a8cca0d213e3fdcb8e9b9575e14db7847b69ec039e4c5ba1469` |

The differences are code-generation residuals, not hidden by the deterministic gates. Address identities and operation order remain separately checked.

### A.7 Hardware qualification

The reviewed candidate image
`783a264ca91661928ddc812047e709629ed38e1fcb9ebfaae5f5c59d314a43ee`
completed three channel-11 qualifications against the already qualified
linker-packed packet-RAM parent. Associations completed in 8, 8, and 19
seconds. TCP measured 12.8, 13.3, and 12.4 Mbit/s. Every UDP run transferred
30 MiB at 8.39 Mbit/s with zero of 21,402 datagrams lost, and every final ping
was 20/20. TX failure counters were 5, 6, and 0. No malformed WSM message,
handler failure, exception, or fatal diagnostic occurred.

These runs qualify the fixed-address structural representation and its residual
code-generation differences. They do not prove exclusive ownership of any
shared field or justify relocating DTCM families independently.

### A.8 Native VIF ownership candidate

The first semantic consumer conversion replaces the opaque `VifRecord` body
with decoded ordinary Rust fields for the JOIN/scan-owned subset. Every decoded
field has a compile-time offset assertion; embedded timers and unresolved bytes
remain private `MaybeUninit` quarantine. The overlapping rate word/byte view is
represented by one byte array with semantic setters rather than competing
fields.

`dtcm::with_vifs()` masks IRQ and FIQ on ARM, rejects synchronous re-entry with
a borrow flag, and lends one temporary `&mut [VifRecord; 3]`. Because the target
is single-core, interrupt masking plus the no-foreign-call closure boundary is
the mutex. It does not use atomics and does not treat DTCM as MMIO.

`vif.rs` now uses regular field reads and assignments for activity selection,
JOIN conflict checks, STA initialization, active-link publication, teardown,
and snapshots. STA defaults are collected in `VifRecord::initialize_sta()`.
Raw volatile accesses remain only for the still-untyped PAS, radio-owner-global,
and PHY families; those are outside this VIF conversion and must be converted
with their own owners before their volatility can be removed.

No hardware qualification was run for this semantic VIF conversion.

### A.9 Complete native VIF consumer pass

All Rust VIF-root consumers now enter through the typed owner rather than
reconstructing `base + interface * 0x3b0 + field`:

* `mac.rs` uses typed scan-prefix fields and models the wake-time `+0x3c6`
  access as the following VIF record's decoded `wake_reinit_flag`, making the
  cross-record relationship explicit;
* `platform.rs` initializes each record's `own_mac` array directly;
* `tx.rs` uses typed snapshots/mutations for TALA, completion accounting,
  management-frame address/rate selection, and diagnostic helper addresses;
* `vendor_host_tx.rs` uses typed VIF snapshots and mutation methods for link
  eligibility, power-save link masks, sequence selection, pending decisions,
  live diagnostics, and TX-busy accounting.

The source gate now rejects the fixed VIF root outside `dtcm.rs` and unit-test
layout expectations. Remaining volatile arithmetic in those consumers belongs
to other not-yet-typed families, principally PAS, link/LMC, contexts, and PHY.
No hardware qualification was run for this pass.

### A.10 Independent typed-VIF blocker closure

The independent review follow-up narrowed the semantic owner rather than
expanding `SharedVifState` snapshots:

* power-save now has two exact mutations matching `txp_program_pipe_hw`
  (`annotated-main.c:11128-11243`): the QoS branch clears only `buffered_links`;
  the tail always clears `awake_links`, reads the old effective mask after that
  write, and recomputes only when the old mask was nonzero, using the earlier
  sleeping-mask observation;
* VIF reads are operation-specific (pipe flags, power-save policy/masks,
  allowed links, TALA threshold/AMPDU, probe fields, diagnostics, wake
  selection), preserving the TALA AMPDU read at the reduction decision rather
  than at function entry;
* direct/interior Rust accesses at scan completion, host-context accounting,
  probe construction, synthetic scan publication, and wake restoration now use
  typed VIF APIs. Source scanning covers Rust and tools, rejects every literal
  in `0x04003e98..0x040049a8` plus synthesized record offsets outside the owner
  and test expectations, and the linked-image gate pins the exact owner-emitted
  interior literal set;
* operating interfaces are explicitly `0..2`; record 2 is explicitly the
  synthetic scan record. RESET eligibility is restricted to operating VIFs, so
  record-2 activity cannot select teardown;
* JOIN keeps preliminary record/PAS preparation but checks and publishes the
  radio owner before final VIF `active`, effective-link, operating-state, and
  activity-state publication. Busy therefore leaves no newly final VIF state;
* `with_vifs` returns an explicit re-entry error, production mutations either
  propagate it or halt instead of silently discarding writes, and closures are
  field-only. Host tests use zeroed process-local backing plus a mutex/re-entry
  guard and never form references at target DTCM addresses.

Focused host tests cover both power-save branches, synthetic-record RESET
eligibility, JOIN Busy/final publication, the vendor next-record wake flag,
nested borrow rejection/write retention, host backing identity, and TALA read
timing. All 158 host tests pass. LSP reports no errors in the touched Rust and
Python files. Source, packer, stack, packet-RAM, DTCM layout, packed-image, and
bootstrap checks pass.

A fresh artifact was built at
`/tmp/xr819-typed-vif-review-20260718-112411/thumbv5te-none-eabi/release/hif-startup`:

```text
ELF SHA-256     1b471b325cf577cac21954640229e3102ad209795edf47241663b5791b371e2a
packed SHA-256  c8576c7ec2255eae08de8f4c46c493222af4ebc45a3735f6097513e7466c6c21
bootstrap SHA-256 48d858b8785220aa7c9a9c0898164da1ecc71b4b6218283b2687b3d2f821d197
.text size/hash 0x10e60 / 5d04c43d0249cce92ba48678e925c63b266060bc8ee9ca7c71f65ce18fc667b8
```

The explicit comparison baseline is `/tmp/xr819-b6-hif-startup.elf` (ELF
SHA-256 `3fb63f89d154025f62d55501513169bc3b2d37bf04584793e51222b24968d0b2`,
`.text` `0x10638`, SHA-256
`7463d0dd4163322c534f32062a5cea2b9c795859038afa128ecb467356ad533c`).
The candidate adds 13 sized text symbols, principally narrow VIF operations,
and changes 127 of 195 common sized text-symbol byte streams under size-LTO;
`.text` grows by `0x828`. Relevant normalized instruction-count changes include
`program_pipe_eligible` 232 -> 256, `vif::activate_sta` 315 -> 327,
`vif::teardown` 52 -> 56, `vif::snapshot` 44 -> 59,
`join::activate_sta` 142 -> 123, `mac::reinitialize_after_wake` 338 -> 382,
`prepare_probe_context` 296 -> 373, and `release_wsm_context_address` 72 -> 77.
The original clean-b6 gate compared literal-pool byte order, so unrelated
size-LTO changes caused a false failure even though the exact MMIO literal
multiset retained only the qualified additions and removals. A follow-up gate
compared that exact multiset while retaining source hashes for qualified
packet/HIF routines; all deterministic checks then passed. The broad VIF
code-generation changes above remain explicit.

Hardware qualification did not match the exact parent distribution. Candidate
runs 1 and 2 associated in 14 seconds and completed at 13.1 and 13.0 Mbit/s TCP,
8.39 Mbit/s UDP with zero of 21,402 datagrams lost, 20/20 ping, and no failures.
Run 3 remained scanning for the full 35-second association window. The exact
fixed-layout parent then associated in 19 seconds and completed at 13.5 Mbit/s
TCP, 8.39 Mbit/s UDP with zero loss, and 20/20 ping. The observed distribution
was therefore candidate 2/3 versus exact parent 4/4. The typed-VIF conversion is
rejected and does not supersede the hardware-qualified fixed-layout image.

### A.11 Low-MAC/PAS semantic-layout candidate

This candidate decodes the fixed family at `0x04003678..0x04003e78` without
moving it and without converting VIF records. It remains a **shared ABI and raw
address view**, not exclusive Rust ownership: translated foreground code,
cooperative MAC service, and retained IRQ/FIQ-shaped code can still mutate the
same bytes.

#### Corrected overlap and semantic confidence

`LowMacPasFamily` remains an exact `#[repr(C, align(4))]` `0x800`-byte type, but
it no longer embeds three independent `PasRecord` fields. The complete
`+0x470..+0x800` tail is one opaque overlapping region. `PasStrideLayout` is
only an offset schema for three observed starts at `+0x470 + n*0x98`; the
`0x98` stride is qualified, but three disjoint semantic records are not.

The alternate raw root at family `+0x620` (`0x04003c98`) is explicit as
`AlternatePasRootAddress`. Its root begins inside the final `0x18` bytes of the
third PAS stride view. Its observed `+0x19` and `+0x1a` byte accesses land at
`0x04003cb1` and `0x04003cb2`, immediately beyond that view's nominal end
`0x04003cb0`. The same opaque tail also contains the BA root at `+0x648`, whose
`0x38`-stride view crosses the family boundary for pipe seven. No ordinary
field, shared reference, or non-overlapping ownership claim is made for these
overlays.

Names were weakened to match the evidence. Examples include `slot_bits`,
`mode_byte`, `path_selector_byte`, `nonzero_block_byte`,
`tbtt_window_control_byte`, `receive_gate_bits`, `receive_state_byte`,
`optional_pipe_object_word`, `control_bits`, and `vif_mode_byte`. Documentation
only names values or bits exercised by translated/vendor paths: PAS activity
values 1/2, observed mode bytes 0/1/2/5/6/`0x0f`, slot bit 0 plus the JOIN write
of 4, receive-gate bits 0/1, receive-state values 0/1/4, JOIN VIF control bit 0
plus bit 10 or bit 11, and the individually tested VIF control bits 29/30/26.
Unknown values remain representable.

#### Layout-derived address APIs and production helpers

Production family constants and PAS field accessors derive offsets from
`offset_of!`, `size_of!`, and the declared `repr(C)` layouts. Checked APIs bound
interfaces, queues, rates, address bytes, policies, and BA pipes where the
production caller has those semantics. Compatibility `*_unchecked` accessors
state explicitly that they preserve unbounded vendor base-plus-index arithmetic;
the policy-sentinel and crossing BA views remain intentionally address-only.
No API returns a complete family snapshot or a safe shared reference.

JOIN and teardown now use operation-specific PAS helpers in production. The
JOIN helper preserves the synthetic-view clear, selected-view control writes,
copied path-byte read/write, no-inline backoff reset, own/BSSID copies, and
family publications in their original order. Teardown preserves its three PAS
control writes before the last-active band clear. The no-inline backoff helper
was retained because inlining it grew both `apply_edca` and `activate_sta`.

The disconnected `PasJoinPublication`, process-local re-entry boolean, and
test-only band-mask branch were removed. Host tests run the same operation
helpers through a recorder that captures operation kind, address, width, value,
branch, and order. Coverage now includes the complete JOIN control prefix,
backoff reads/writes, address-copy phase, family publications, and both teardown
last-active branches. Layout tests separately cover checked bounds, every named
PAS field offset, the `+0x620` crossing, and the BA cross-family extent. Default
host tests pass 153/153; diagnostic-feature host tests pass 154/154.

#### Drift gates and their limits

`tools/check-low-mac-pas-layout.py` is deliberately described as a drift and
known-access manifest, not complete computed-reference proof.

* The lexical source gate scans 65 project source files across Rust, Python,
  shell, C/C++, assembly, linker-script, and TOML extensions. Outside `dtcm.rs`
  it rejects in-family literals and known synthesized legacy forms after
  removing comments and strings.
* The linked gate pins the exact 47-word aligned in-family literal multiset.
* It also decodes PC-relative literal loads and pins 60 xrefs across 42
  `(symbol, value)` keys, so literal-pool reuse and containing-symbol drift are
  reviewed rather than inferred from raw word presence alone.

These checks do not recover register-computed addresses that have no literal,
prove indirect vendor consumers complete, or prove semantic ownership.

The clean-b6 transition gate no longer scans every aligned `.text` word or
special-cases individual literal counts. That approach confused Thumb
instruction bytes and linker literal-pool reuse with changed MMIO operations.
The revised gate combines hashes of qualified packet-transition source bodies
with decoded PC-relative literal-value sets for `0x09c...` packet-controller
and `0x0a8...0x0ac...` shared/MMIO ranges. It detects a real new or removed
MMIO value and is insensitive to duplicate pool words. It does not claim to
count repeated operations outside the source-hashed functions.

#### Generated code and residuals

The final feature-free `.text` is `0x106c0`: `0x88` above clean b6 and four
bytes below the exact `aebbc728` parent (`0x106c4`). The prior review candidate
was `0x1073c`, so the review fixes remove `0x7c` bytes. VIF activation returns
from stack `120` to `96`, its exact b6/parent frame, and shrinks below both
reference bodies. `apply_edca`, probe publication, duration building, and the
large probe service body return exactly to parent size/instruction/stack shape.
Startup retains the only notable family-adjacent growth versus the parent:
`+0x0c` bytes, six instructions, and stack `80 -> 88`; its source-level
volatile/MMIO ordering is unchanged. Further reduction would require reverting
layout-derived address materialization or otherwise changing qualified source,
so it remains an explicit residual rather than being hidden.

Sizes below are `b6 -> exact parent -> final candidate`; instruction counts
exclude literal `.word` entries.

| Function | Size | Instructions | Stack | Load/store mnemonic order vs parent |
| --- | --- | --- | --- | --- |
| `HostTxDriver::admit` | `0x5b4 -> 0x5dc -> 0x5dc` | `665 -> 679 -> 679` | `160 -> 160 -> 160` | preserved |
| `ChannelTransitionScheduler::start` | `0x120 -> 0x120 -> 0x120` | `128 -> 128 -> 128` | `112 -> 112 -> 112` | preserved |
| `HostSchedulerReservation::publish_in_batch` | `0x1e8 -> 0x1f0 -> 0x1f0` | `222 -> 224 -> 224` | `136 -> 144 -> 144` | preserved |
| `join::activate_sta` | `0x154 -> 0x154 -> 0x154` | `142 -> 142 -> 142` | `80 -> 80 -> 80` | preserved |
| `mac::initialize_vendor_startup_state` | `0x258 -> 0x268 -> 0x274` | `241 -> 253 -> 259` | `88 -> 80 -> 88` | changed address materialization |
| `mac::program_rate_tables` | `0x2d8 -> 0x2cc -> 0x2cc` | `329 -> 323 -> 323` | `200 -> 200 -> 200` | preserved |
| `mac::reinitialize_after_wake` | `0x34c -> 0x364 -> 0x364` | `338 -> 352 -> 352` | `88 -> 88 -> 88` | preserved |
| `phy::begin_channel_transition` | `0x77c -> 0x790 -> 0x790` | `809 -> 819 -> 819` | `160 -> 160 -> 160` | preserved |
| `platform::program_station_address` | `0x80 -> 0x80 -> 0x80` | `57 -> 57 -> 57` | `48 -> 48 -> 48` | preserved |
| `tx::build_single_frame_duration` | `0xe4 -> 0xe4 -> 0xe4` | `96 -> 96 -> 96` | `24 -> 24 -> 24` | preserved |
| `tx::complete_tx_pipe_slot` | `0x15c -> 0x15c -> 0x15c` | `155 -> 155 -> 155` | `56 -> 56 -> 56` | preserved |
| `tx::execute_single_probe_publication` | `0x144 -> 0x144 -> 0x144` | `145 -> 145 -> 145` | `56 -> 56 -> 56` | preserved |
| `tx::prepare_context_publication` | `0x180 -> 0x180 -> 0x180` | `183 -> 183 -> 183` | `216 -> 216 -> 216` | preserved |
| `tx::prepare_probe_context` | `0x284 -> 0x290 -> 0x290` | `296 -> 304 -> 304` | `80 -> 80 -> 80` | preserved |
| `tx::prepare_single_frame_pas_timing` | `0x12c -> 0x12c -> 0x12c` | `143 -> 143 -> 143` | `48 -> 48 -> 48` | preserved |
| `tx::service_single_probe_runtime_inactive` | `0x1648 -> 0x1654 -> 0x1654` | `2521 -> 2528 -> 2528` | `208 -> 208 -> 208` | preserved |
| `vendor_host_tx::release_pending_to_pas` | `0x170 -> 0x170 -> 0x170` | `165 -> 165 -> 165` | `48 -> 48 -> 48` | preserved |
| `vif::activate_sta` | `0x2c4 -> 0x2c4 -> 0x2b8` | `315 -> 315 -> 309` | `96 -> 96 -> 96` | changed, same volatile order |
| `vif::apply_edca` | `0x10c -> 0x10c -> 0x10c` | `123 -> 123 -> 123` | `112 -> 112 -> 112` | preserved |
| `vif::teardown` | `0x7c -> 0x7c -> 0x78` | `52 -> 52 -> 50` | `24 -> 24 -> 20` | changed, same volatile order |

The deepest normal stack chain remains 2892 bytes. Exception use remains
224/256 bytes.

#### Final deterministic qualification

LSP reports no errors on the modified Rust/Python files; host-target inactive
`cfg` hints remain expected. Default and diagnostic host tests pass. The full
`XR819_B6_ELF=/tmp/xr819-b6-hif-startup.elf ./tools/check.sh` run passes source,
packer, ARM build, stack, packet-RAM, DTCM, low-MAC literal/xref, revised
clean-b6 MMIO, and sectioned-bootstrap checks.

Fresh review artifacts:

```text
ELF       /tmp/xr819-low-mac-pas-review-20260820T114759Z.elf
          438fc7b0037995396740c09a85ca51685f43ff1d09cf6f9bc4c0ec9387b0f048
.text     size 0x106c0
          c85183891b68dfbf558eb565912884e37c33971096c4ab507333018a3a8db389
packed    /tmp/xr819-low-mac-pas-review-20260820T114759Z.bin
          da7dae3888ef170b15336c7a065935ff08b8dcfb49b66f21858e6c6e066bcb5c
bootstrap /tmp/xr819-low-mac-pas-bootstrap-20260820T114759Z.bin
          48d858b8785220aa7c9a9c0898164da1ecc71b4b6218283b2687b3d2f821d197
```

Hardware qualification completed cleanly in three channel-11 runs. Association
completed in 14, 14, and 3 seconds. TCP measured 15.4, 13.9, and 16.4 Mbit/s.
Every UDP run transferred 30 MiB at 8.39 Mbit/s with zero, one, and zero of
21,402 datagrams lost; every final ping was 20/20; and TX failures were zero in
all runs. No malformed WSM message, handler failure, exception, or fatal
diagnostic occurred.

The candidate is therefore qualified as a fixed-address semantic layout and
known-access conversion. It still claims neither exclusive movable ownership
nor complete computed-reference closure; the unresolved VIF/link/BA/power-save
and retained-vendor cycles remain migration blockers.


### A.10 Fresh typed VIF conversion atop qualified low-MAC/PAS

This candidate starts directly from `319d653982e4`. The old final typed-VIF
snapshot and rejected patch were used only to recover field evidence and review
findings; they were not applied. The design is intentionally narrower than the
rejected attempt:

* `VifRecord` is a complete `#[repr(C, align(4))]` ABI description with exact
  `0x3b0` size and three physical records at `0x04003e98`, `0x04004248`, and
  `0x040045f8`;
* every named, reserved, overlay, and tail field has a compile-time offset,
  size, and alignment assertion;
* the rate word/byte overlap is one explicit eight-byte overlay, and the wake
  access historically written as previous-record `+0x3c6` is documented and
  addressed as the following record's `wake_reinit_flag` at `+0x16`;
* operating VIFs remain exactly records 0 and 1. Record 2 is the synthetic scan
  record and its activity byte is never sufficient for RESET or teardown.

#### Ownership decision and unresolved vendor writers

Complete exclusive Rust ownership is still unsound. Production-reachable
retained vendor routines, timer callbacks, completion paths, and IRQ/FIQ code
can mutate decoded VIF bytes. The candidate therefore does **not** construct a
safe `&VifRecord`, `&mut VifRecord`, or array reference on target. Decoded
scalars use shared ABI wrappers and operation helpers perform volatile accesses
through field-derived raw addresses. This is the exact blocker requested by the
review: translating the Rust consumer closure removes duplicate Rust address
arithmetic, but does not remove retained vendor writers.

No target IRQ/FIQ owner is asserted for the VIF records: that would imply an
exclusive ordinary-reference domain which retained vendor writers disprove.
Production operations remain narrow volatile accesses in the qualified parent
order. Host tests alone use a process mutex/thread-local guard with explicit
re-entry failure around the zeroed process-local `DTCM_STATE` backing; target
addresses are never dereferenced on the host.

#### Consumer closure and PAS integration

Production consumers in VIF, JOIN, scan, MAC, platform, TX, vendor-host-TX,
diagnostics, and tools now enter through typed VIF addresses or narrow VIF
operations. Covered forms include direct interior literals, record stride
arithmetic, mode-interior roots, link-map alternate roots, TALA AMPDU reads,
probe MAC/rate fields, activity and link decisions, host-context counters,
power-save masks, and the next-record wake selection.

The low-MAC/PAS implementation from `319d6539` remains the only PAS address
owner. JOIN and teardown retain its operation recorder, typed PAS stride views,
backoff helper, and exact publication branches. No old `PAS_BASE` arithmetic or
parallel power-save mutation implementation was reintroduced. VIF power-save
fields remain volatile and their read-modify-write points preserve the qualified
branch behavior: clearing `awake_links` always occurs, while effective links are
recomputed only when the old effective mask is nonzero.

JOIN preparation writes non-final defaults before owner acquisition, then runs
the qualified PAS publication sequence. The radio-owner conflict check precedes
all final VIF publication. Only owner success publishes `active = 2`, effective
links, operating state `0x30`, and activity state `3`; Busy leaves none of those
final values.

#### Drift gates and focused tests

`tools/check-vif-layout.py` rejects production source literals in the complete
`0x04003e98..0x040049a8` physical range, all three record starts/end, historical
low-16-bit synthesized forms, and legacy VIF root/stride identifiers outside the
owners. Its linked-image gate decodes PC-relative literal loads and pins exact
`(containing symbol, loaded value)` xrefs. It deliberately does not scan aligned
words, because Thumb instructions are not address literals. The gate runs beside
the low-MAC/PAS source and decoded-xref gates in `tools/check.sh`.

Focused host tests cover exact field offsets, host-local backing, nested
mutation rejection, operating-versus-synthetic activity semantics, record-2
teardown exclusion, Busy/final JOIN publication, teardown fields, rate overlay,
power-save branch interaction, following-record wake selection, TALA read timing,
probe snapshots, and PAS access width/value/branch/order.

#### Qualification and code generation

No hardware testing was run. LSP reports no errors. Default host tests pass
162/162 and diagnostic-feature tests pass 163/163. The full deterministic run
with `XR819_B6_ELF=/tmp/xr819-b6-hif-startup.elf` passes source gates, host tests,
ARM build, stack analysis, packet-RAM/DTCM layout, low-MAC/PAS and VIF decoded
xref manifests, normalized clean-b6 transition checks, packing, and bootstrap.
The deepest normal call chain remains 2892 bytes and exception use remains
224/256 bytes.

The exact `319d6539` parent was rebuilt from revision files in `/tmp` for a
separate comparison. Parent `.text` is 67264 bytes (`0x106c0`); this candidate
is 67792 bytes (`0x108d0`), a localized `+0x210` typed-VIF consumer delta. The
largest changed bodies are VIF/host-TX consumers. No MMIO literal or barrier-set
change is present, but several hot functions have changed decoded load/store
mnemonic sequences from address materialization and inlining. Source-level
volatile read/write points and ordering were reviewed and focused tests pass,
but the broad `HostTxDriver::service_index` growth (`+0x90`) means exact-parent
code-generation qualification is **not clean** and remains a blocker.

| Function | Parent -> candidate size | Instructions | Stack | load/store/barrier order |
| --- | ---: | ---: | ---: | --- |
| `vif::activate_sta` | `0x2b8 -> 0x2e4` | `309 -> 329` | `96 -> 128` | changed |
| `vif::teardown` | `0x78 -> 0xa4` | `50 -> 74` | `20 -> 20` | changed |
| `vif::snapshot` | `0x5c -> 0x5c` | `44 -> 44` | `28 -> 28` | exact |
| `join::activate_sta` | `0x154 -> 0x154` | `142 -> 142` | `80 -> 80` | exact |
| `join::reset` | `0x90 -> 0xbc` | `52 -> 74` | `16 -> 24` | mnemonic order exact |
| `tx::prepare_probe_context` | `0x290 -> 0x298` | `304 -> 308` | `80 -> 88` | changed; VIF reads remain at original decisions |
| `tx::service_single_probe_runtime_inactive` | `0x1654 -> 0x1680` | `2528 -> 2551` | `208 -> 184` | changed |
| `vendor_host_tx::program_pipe_eligible` | `0x1e8 -> 0x1dc` | `232 -> 226` | `56 -> 56` | changed |
| `mac::reinitialize_after_wake` | `0x364 -> 0x364` | `352 -> 352` | `88 -> 88` | changed mnemonic sequence, exact size/stack |
| `platform::program_station_address` | `0x80 -> 0xa0` | `57 -> 72` | `48 -> 48` | changed |
| `HostTxDriver::admit` | `0x5dc -> 0x5e8` | `679 -> 685` | `160 -> 160` | changed |
| `HostTxDriver::service_index` | `0x820 -> 0x8b0` | `963 -> 1030` | `712 -> 712` | changed; blocker |

Fresh artifacts:

```text
ELF       /tmp/xr819-typed-vif-pas-20260820T130233Z.elf
          a13c39e9a067b14c665f4a38e90d5972cedcaba0f9e494735c430881bb8ee233
.text     size 0x108d0
packed    /tmp/xr819-typed-vif-pas-20260820T130233Z.bin
          f1e28897bdbe4dfdbec7b968e33990badf22223bde24c354f48a72b9ccfc1e9a
bootstrap /tmp/xr819-typed-vif-pas-bootstrap-20260820T130233Z.bin
          48d858b8785220aa7c9a9c0898164da1ecc71b4b6218283b2687b3d2f821d197
```

Qualification status is host/static and codegen-blocked, not clean. The exact
remaining ownership blocker is retained vendor/IRQ mutation of decoded VIF
fields; therefore this candidate is a typed volatile ABI conversion, not
exclusive native VIF ownership. The separate clean-qualification blocker is the
hot-code/LTO divergence listed above.

### A.12 Independent-review blocker closure for typed VIF on PAS

This follow-up fixes the independent-review blockers without hardware testing.
It remains a fixed-address typed volatile ABI conversion, not native/exclusive
VIF ownership and not proof of whole-parent equivalence.

Power-save observation again follows the parent short-circuit timing.
`sleeping_links` and `link_gate` are separate narrow volatile reads;
`link_gate` is evaluated only after the frame-control read/classification and
only when the preceding release predicate and link-zero/sleeping tests require
it. A recorder test pins the ordered two-byte sleeping read, two-byte frame
control read, and conditional one-byte link-gate read, including the branch in
which no link-gate observation is permitted.

Station-address publication again performs, for each source byte, low-MAC 0,
low-MAC 1, VIF 0, VIF 1, and VIF 2 byte writes before advancing the loop. The
VIF publication is one typed per-byte operation. A host recorder pins all 30
write addresses, one-byte widths, values, and order. The target body is still
four bytes larger than the exact parent because typed address materialization
differs, despite having two fewer decoded instructions and the same 48-byte
stack frame.

`PendingVifState` and `prepare_pending_state` are removed. The pending TX path
now derives one `VifRecordAddress` from the retained interface byte and keeps
the parent's scalar `active_mask`, `effective_mask`, `effective_link`, control,
radio-state, mode, and publication flow with narrow volatile field operations.
`HostTxDriver::service_index` is reduced from the previous candidate's
`0x8b0`/1030 instructions back to `0x834`/968, versus the exact parent's
`0x820`/963, while retaining the exact 712-byte stack frame. The residual is
`+0x14` bytes and five instructions, bounded by the new gate rather than called
exact.

`any_active`, `active_interface`, and `is_active` now use explicit record-0 then
record-1 reads with no iterator/`Option` pipeline. Teardown derives one record,
reuses it for all deactivation and owner operations, and excludes synthetic
record 2 before mutation. Target hot paths no longer use the removed pending
snapshot or a generic closure/`Result` abstraction for pending-state control.
The host-only VIF mutation guard remains under `cfg(test)`.

JOIN Busy coverage now models the complete relevant sequence: preliminary VIF
preparation, the complete 40-operation PAS publication recorder, the radio-owner
read, suppression of all three owner writes on conflict, and rejection of final
VIF publication. The test verifies that `active`, effective links, and activity
state remain unpublished and that operating state remains at its preliminary
`0x23` value. This supersedes the earlier helper-only Busy outcome test as the
sequencing evidence, while retaining that smaller focused test.

`tools/check-hot-codegen.py` adds an exact-parent focused gate. It compares
symbol size, decoded instruction count, LLVM stack metadata, exact decoded
memory-mnemonic order for the unchanged JOIN wrapper, and the complete global
IRQ/barrier mnemonic/operand sequence. It bounds service-index, pipe
eligibility, large probe service, probe preparation, scan, JOIN/RESET, VIF
activation/teardown, station programming, and wake reinitialization. The gate
prints explicitly that it is bounded focused evidence, not whole-parent
equivalence. `tools/check.sh` now runs default and diagnostic host suites and
runs this comparison when `XR819_PARENT_ELF` is supplied. The VIF decoded-xref
manifest remains a known-access drift gate only: it does not prove computed
reference closure, indirect vendor-consumer completeness, semantic ownership,
or full parent equivalence.

Exact fixed-layout parent to final candidate comparison:

| Function | Size | Instructions | Stack | Qualification |
| --- | ---: | ---: | ---: | --- |
| `HostTxDriver::service_index` | `0x820 -> 0x834` | `963 -> 968` | `712 -> 712` | bounded residual `+0x14`/`+5` |
| `vendor_host_tx::program_pipe_eligible` | `0x1e8 -> 0x1e0` | `232 -> 228` | `56 -> 56` | smaller; ordered power-save recorder added |
| `tx::service_single_probe_runtime_inactive` | `0x1654 -> 0x1680` | `2528 -> 2551` | `208 -> 184` | bounded typed-VIF residual |
| `tx::prepare_probe_context` | `0x290 -> 0x298` | `304 -> 308` | `80 -> 88` | bounded typed-VIF residual |
| `scan::service` | `0x504 -> 0x528` | `530 -> 548` | `48 -> 48` | bounded explicit-record residual |
| `join::activate_sta` | `0x154 -> 0x154` | `142 -> 142` | `80 -> 80` | exact size/instruction/stack and memory-mnemonic order |
| `join::reset` | `0x90 -> 0xa8` | `52 -> 62` | `16 -> 24` | bounded explicit-record residual |
| `vif::activate_sta` | `0x2b8 -> 0x2e4` | `309 -> 329` | `96 -> 128` | bounded typed-address residual |
| `vif::teardown` | `0x78 -> 0x80` | `50 -> 54` | `20 -> 16` | bounded; one record reused |
| `platform::program_station_address` | `0x80 -> 0x84` | `57 -> 55` | `48 -> 48` | exact recorder order; address materialization residual |
| `mac::reinitialize_after_wake` | `0x364 -> 0x364` | `352 -> 352` | `88 -> 88` | exact size/instruction/stack |

The global decoded IRQ/barrier sequence is exact at 17 operations. The deepest
normal call chain remains 2892 bytes and exception use remains 224/256 bytes.
Final `.text` is `0x107bc`, `0xfc` above the exact fixed-layout parent
`0x106c0` and `0x114` below the previous typed-VIF candidate `0x108d0`.

Default host tests pass 165/165 and diagnostic-feature tests pass 166/166. LSP
reports no errors; host-target inactive-code hints remain expected. The complete
run with both
`XR819_B6_ELF=/tmp/xr819-b6-hif-startup.elf` and
`XR819_PARENT_ELF=/tmp/xr819-parent-319d6539/xr819-firmware/target/thumbv5te-none-eabi/release/hif-startup`
passes source, packer, default/diagnostic host, ARM build, stack, packet-RAM,
DTCM, low-MAC/PAS, VIF decoded-xref, focused exact-parent, normalized clean-b6
MMIO/transition, packing, and bootstrap checks.

Fresh uniquely named artifacts:

```text
ELF       /tmp/xr819-typed-vif-final-20260820T133802Z/hif-startup.elf
          97f88bcee783f6ac0d3eefbdf938e0b1d8b0c6dae3677ce776ff52511954ec78
packed    /tmp/xr819-typed-vif-final-20260820T133802Z/hif-startup.bin
          39b3d010f506d886b7a6f768af27762a302cf870ef6a1ac002d56216ec927ae8
bootstrap /tmp/xr819-typed-vif-final-20260820T133802Z/bootstrap.bin
          48d858b8785220aa7c9a9c0898164da1ecc71b4b6218283b2687b3d2f821d197
.text     size 0x107bc
```

A final adversarial comparison found one remaining parent mismatch in
`program_pipe_eligible`: the typed helpers reloaded shared `buffered_links` and
`awake_links` after the eligibility decision, while the qualified parent
mutated the values already observed. In the hybrid execution model those late
reloads can lose a concurrent wake or buffered-bit publication because the
vendor scheduler serialization is no longer present. The helpers now receive
the saved observations and perform no late reload. Tests explicitly mutate the
backing values between observation and helper invocation and verify that the
saved values remain authoritative.

The corrected packed image is
`2761563f78c4af6d84e8e7d446a5a7ab9528537e730060af8d2af96c794ef644`.
Its first channel-11 qualification associated in two seconds, completed TCP at
15.7 Mbit/s, transferred 30 MiB UDP at 8.39 Mbit/s with zero of 21,402
datagrams lost, and finished with 20/20 ping, zero TX failures, and no fatal
diagnostic. Additional qualification was still running when this checkpoint
was committed. Remaining limitations are the bounded hot code-generation
residuals above, retained vendor/IRQ writers, and lack of complete
register-computed/indirect VIF-reference closure.

### A.13 Native host WSM TX contexts and coupled free-list lifecycle

This candidate starts directly from committed typed VIF `8940467e9cdc`. It
models the 30 host class-0 contexts at their fixed ABI identity and converts the
known production Rust closure without relocating any byte or claiming exclusive
Rust ownership. No hardware run was performed.

#### Exact record and coupled roots

`HostTxContext` is now a complete `#[repr(C, align(4))` layout of exactly
`0x170` bytes. The 30-record `HostTxContexts` array remains exactly
`0x04005a24..0x04008544`; in this historical host-context slice the following
`0x50` bytes at `0x8544` were not consumed. They are decoded later in A.21 as
the peer-pipe table and management/scan tail. Every named field, opaque region, nested
PAS overlay, size, alignment, and offset has a compile-time assertion.

The semantic prefix covers the original HIF request pointer, intrusive link,
packet identity, WSM request metadata, borrowed MPDU pointer/length, completion
status/rate words, submit timing, header/payload split, optional pipe object,
completion class, and the decoded `ctx+0x54` PAS/frame-node overlay. The PAS
view names only supported fields: control bits, AC/rate/policy, timestamps,
status/tries, ownership, duration, descriptor state, airtime, dedicated packet-
RAM frame-state pointer, TID/insertion mode/sequence, interface/duration-slot/
host-link, completion link byte, QoS/cipher state, and their exact widths.
`ctx+0xd4..+0x170` remains private opaque storage; it includes the retained
crypto callback interior root at `ctx+0x110`. Smaller unresolved PAS words and
bytes remain private opaque fields even where initialization writes a known
zero.

Address domains are explicit: `HostContextAddress`, `HostFrameNodeAddress`,
`HostPasAddress`, `HifRequestAddress`, and `PacketRamAddress`. Context/index and
frame-node/PAS round trips are checked; interior, pre-range, end, pre-command,
and free-head pointers are rejected. No API yields `&HostTxContext`,
`&mut HostTxContext`, or a record slice.

The mixed root at `0x04008798..0x040087b0` is decoded only as far as direct
vendor evidence supports:

```text
+0x00..+0x0c  opaque accounting/cache state
+0x0c         duplicate-cache cursor
+0x10         deferred event owner/context
+0x14         auxiliary TX-buffer free-list head (not the host WSM list)
```

The host WSM free head remains the following word at `0x040087b0`. The adjacent
word at `0x040087b4` remains unresolved and has no production mutation API.
`0x040087b8` is still the separate qualified link/sequence root; this pass does
not absorb link, LMC, BA, TALA, or power-save storage.

#### Consumer closure and ownership

The Rust closure now derives host-field addresses from the layout in:

* `vendor_host_tx.rs`: pool rebuild/pop/push, request initialization, header
  classification and sequence publication, post-crypto pending insertion,
  pending removal and eligibility, PAS accounting/ring handoff, scheduler
  reservation/cancellation/publication, and live diagnostics;
* `host_tx_driver.rs`: admission identity, frame-node ownership, bounded service,
  completion routing, confirmation reads, RESET cancellation, and final release;
* `tx.rs`: host pointer validation, transformed-host compatibility ownership,
  descriptor preparation/publication, retry/rearm, pipe start/success, completion
  enqueue/drain/callback, power-save/TALA reads, and class-0 free integration;
* `host_tx_diagnostics.rs`: submission, retry, completion, and frame snapshots.

Generic completion code that can receive either internal or host contexts uses
operation-specific scalar address helpers. The host branch is layout-derived;
the retained internal branch keeps its existing raw compatibility offset until
the internal family is decoded further. Mixed pending-list links likewise use a
typed host link when the node is in the host range and preserve the raw internal
link for internal nodes.

The context/request-credit-before-confirmation ordering is inherited unchanged
from exact parent `8940467e9cdc`; it is not vendor-lifetime equivalence. The
executable statements in parent and candidate `hif_startup.rs:518-548` call
`finish_confirmation()` before `publish_request_in_place()`, and
`hif.rs:1103-1125` appends the request credit before `enqueue_output()`. The
conversion did not alter either file's executable ordering, so this structural
migration deliberately leaves it alone. Packet-RAM MPDU and per-context
`0x54` frame-state identities, service budget, scheduling, batching, retry
policy, and confirmation ordering remain inherited from the parent.

**Separate future correctness issue — HIF request-buffer lifetime.** The vendor
ordinary non-coalesced confirmation reuses the original request pointer held at
`ctx+0x00`: `tx_confirm_build_and_send` frees the host context at
`annotated-main.c:13412`, then calls `hif_send_msg_to_host(puVar9)` at
`annotated-main.c:13417-13418`. `hif_send_msg_to_host` publishes that same
message pointer into the host-facing TX ring (`annotated-main.c:17583-17617`),
and `hif_tx_confirm_drain` calls `hi_msg_release()` only after descriptor
completion (`annotated-main.c:17541-17578`). The Rust parent/candidate instead
copy confirmation bytes to independent output storage, append the original RX
request credit at `hif.rs:1124`, and only then enqueue the output at
`hif.rs:1125`. Matching the vendor request-buffer lifetime and publication
dependency therefore needs a separate HIF redesign; it is explicitly out of
scope for this candidate.

Ownership remains mixed shared ABI. Retained vendor callbacks, the translated
IRQ/FIQ-shaped completion path, and diagnostics can access live fields, so all
production scalar accesses are raw volatile. Multiword/list/free-list
transitions remain under IRQ/FIQ masking or `MacDomainGuard`; the process-local
recorder's re-entry test models that exclusion. No ordinary safe reference is
formed over target DTCM.

#### Retained vendor and diagnostic evidence

The local decoded main image resolves the host pool root in
`tx_abort_frames_for_vif`, the `0x04008798` root in `txbuf_freelist_pop/push`,
`tx_wsm_buf_alloc/free`, `rx_dup_cache_check`, `dup_cache_init`,
`lmc_post_event_200`, and `task_13b58`. The exact root arithmetic establishes
host head `root+0x18`, auxiliary head `root+0x14`, duplicate cursor `root+0x0c`,
and deferred owner `root+0x10`.

The high-TCM diagnostic dispatcher starts at `0x04005a24`, walks at stride
`0x170` for at most 30 records, matches packet ID at `+0x08`, and reads `+0x60`,
`+0x70`, `+0x72`, `+0x58`, and `+0x80`. The negative-root post-crypto callback
continues to prove the `ctx+0x110` interior identity. The vendor abort routine's
known non-advancing loop still checks only record zero despite a bound of 30;
this implementation does not copy that bug and RESET uses the native 30-owner
driver state.

Arbitrary MIB memory access and unknown indirect vendor consumers remain a
limitation. The gates below do not prove register-computed references or
exclusive ownership, and no address relocation is proposed.

#### Free-list and lifecycle operations

Pool operations are explicit and preserve vendor order and widths:

* rebuild clears the head, links records in ascending-index push order, writes
  terminal `0x00ff`, restores each dedicated packet-RAM frame-state pointer, and
  publishes the final head;
* pop reads/validates the exact head, validates the frame-state identity, reads
  the next link, publishes the new head, then writes request flags (8-bit),
  completion status (32-bit), terminal status (16-bit), and ownership (32-bit);
* push writes completion and terminal sentinels, ORs host return bit
  `0x00040000`, reads the old head, stores the context link, publishes the new
  head, then decrements typed VIF in-flight accounting;
* pending/PAS/reservation/scheduled/completing phases still prevent publication
  skips and prevent hardware-owned cancellation; free occurs only after
  confirmation admission or a reversible pre-hardware abort.

The transformed class-0 compatibility path now reuses the same allocator/free
implementation instead of maintaining a second host-head arithmetic copy.

Pool rebuild is a deliberate broader recovery policy inherited from the exact
parent, not vendor-equivalent allocation. With zero typed in-flight contexts,
the parent rebuilds after an empty head, invalid head, or frame-state mismatch;
with a nonzero count it returns the allocation error. Vendor
`tx_wsm_buf_alloc` (`annotated-main.c:13531-13552`) simply pops a nonzero head
and never rebuilds. There is no source evidence that only one zeroed word is a
unique safe "uninitialized head" signature, and narrowing would change parent
recovery behavior, so this candidate retains the broader policy.

#### Gates and tests

`tools/check-host-context-layout.py` scans production source across Rust,
Python, shell, C/C++, assembly, linker/build scripts, and TOML extensions. It
rejects literals in the full context range and `0x04008798..0x040087b8`, known
low-16-bit synthesized forms, and legacy root/stride identifiers outside
`dtcm.rs`. One generic packer test literal at `0x04008000` is explicitly
classified as a non-context DTCM fixture. Complete items that are exclusively
`cfg(test)` are masked while later production items continue to be scanned;
the former first-`cfg(test)` tail truncation is gone.

The linked manifests pin reviewed aligned literal words and decoded PC-relative
literal xrefs by containing symbol. They are drift evidence, not complete
closure. The gate deliberately does not classify arbitrary
aligned Thumb words as xrefs, recover register-only computed addresses, inspect
arbitrary MIB accesses, or prove retained vendor closure. The existing VIF xref
manifest was narrowed by one entry because transformed host release now calls
the typed host free operation instead of duplicating the VIF in-flight write.

Process-local host tests and recorders cover every named field offset,
size/alignment, context/index
and frame-node/PAS round trips, malformed/interior/end pointer rejection,
packet-RAM identity, accounting/free-head/link-root boundaries, exact pool
rebuild/pop/push address-width-value order, empty/corrupt head and frame-state
rejection, initialization address-width-value order, nested guard rejection,
phase skips, pending append/removal, PAS compaction, scheduler ownership, and the
existing arena confirmation/RESET invariants. These tests execute process-local
models and recorders; they do not execute ARM driver, packet-RAM, interrupt, or
HIF publication behavior.

#### Exact-parent and B6 code generation

The exact `8940467e9cdc` parent ELF is
`xr819-firmware/target/8940467e-exact-parent.elf`, SHA-256
`9109a91163f65cfdf691d0a72c609731df371b4a756cff44d9486cef3b7d53d7`.
Its `.text` is `0x107bc`, SHA-256
`db5a4f5bb914d7a3118390acf867c45872e29482f7f39db7a666aed60f7da3ed`.
The qualified candidate `.text` is `0x10770`, net `-0x4c`, SHA-256
`18c7dab420e7c79ee6065ec826026be4b578906e8d5954b61355fbe76c1dc839`.
The final clean-B6 gate input ELF is SHA-256
`0d9fb3b923b5db128bb255b22b68ea2b52a21f13dc3623fb24019af8d463471c`
(the pre-normalization archived ELF identity was
`3fb63f89d154025f62d55501513169bc3b2d37bf04584793e51222b24968d0b2`);
its unchanged `.text` is `0x10638`, SHA-256
`7463d0dd4163322c534f32062a5cea2b9c795859038afa128ecb467356ad533c`.

`tools/check-hot-codegen.py` now gates the complete sized text-symbol inventory
through `tools/host-context-codegen-manifest.json`: 196 parent symbols, 197
candidate symbols, 195 common, 81 exact-byte-identical common symbols, 114
changed common byte streams, one parent-only symbol, and two candidate-only
symbols. Any new, removed, or changed stream fails until the complete manifest
is reviewed and regenerated. It records address, size, instruction count, LLVM
stack size, exact byte hash, normalized instruction hash, and load/store
signature for every residual. This remains drift evidence, not behavioral or
ownership closure.

The parent-only symbol is the parent's
`HostSchedulerReservation::publish_in_batch` monomorphization (`...Ms2...`).
Candidate-only symbols are the corresponding candidate monomorphization
(`...Ms1...`) and `rate_policy::rate_for_try_count`. The fixed
`tx::host_prepared_context` is present in both images.

Required exact-parent rows are reported even when their bytes are unchanged:

| Function | Size | Instructions | Stack | Memory ops |
| --- | ---: | ---: | ---: | ---: |
| `rust_main` | `0x1b24 -> 0x1b6c` | `2986 -> 3012` | `1552 -> 1552` | `1470 -> 1479` |
| `HostTxDriver::admit` | `0x5e8 -> 0x5c0` | `685 -> 668` | `160 -> 160` | `338 -> 327` |
| `HostTxDriver::service_index` | `0x834 -> 0x834` | `968 -> 968` | `712 -> 712` | `496 -> 496` |
| `HostTxDriver::confirmation_state` | `0xa8 -> 0xa4` | `81 -> 79` | `72 -> 72` | `45 -> 43` |
| `HostTxDriver::finish_confirmation` | `0x58 -> 0x58` | `37 -> 37` | `16 -> 16` | `9 -> 9` |
| `HostTxDriver::cancelled_confirmation` | `0xc0 -> 0xc0` | `82 -> 82` | `48 -> 48` | `43 -> 43` |
| `free_host_context` | `0x3c -> 0x3c` | `24 -> 24` | `16 -> 16` | `13 -> 13` |
| `HostSchedulerReservation::publish_in_batch` | `0x1f0 -> 0x1d8` | `224 -> 217` | `144 -> 144` | `122 -> 118` |
| `release_pending_to_pas` | `0x170 -> 0x170` | `165 -> 165` | `48 -> 48` | `69 -> 69` |
| `program_pipe_eligible` | `0x1e0 -> 0x1e0` | `228 -> 228` | `56 -> 56` | `87 -> 87` |
| `prepare_host_frame_timing` | `0x38 -> 0x38` | `25 -> 25` | `48 -> 48` | `6 -> 6` |
| transformed publication `cancel` | `0xd4 -> 0xb4` | `91 -> 80` | `112 -> 112` | `45 -> 42` |
| `release_wsm_context_address` | `0xb4 -> 0x8c` | `74 -> 56` | `32 -> 24` | `35 -> 20` |
| `prepare_context_publication` | `0x180 -> 0x180` | `183 -> 183` | `216 -> 216` | `116 -> 116` |
| `emit_host_frame_descriptor_at` | `0x46 -> 0x46` | `33 -> 33` | `56 -> 56` | `12 -> 12` |
| `emit_prepared_probe_descriptor` | `0x2f0 -> 0x2f0` | `355 -> 355` | `112 -> 112` | `150 -> 150` |
| `release_unpublished_probe_context` | `0x70 -> 0x5c` | `41 -> 36` | `8 -> 16` | `18 -> 15` |
| `prepare_probe_context` | `0x298 -> 0x298` | `308 -> 308` | `88 -> 88` | `159 -> 159` |
| `service_single_probe_runtime_inactive` | `0x1680 -> 0x160c` | `2551 -> 2498` | `184 -> 184` | `1175 -> 1150` |
| `Transport::enqueue_output` | `0x78 -> 0x78` | `53 -> 53` | `40 -> 40` | `26 -> 26` |
| `Transport::release_request` | `0x48 -> 0x48` | `31 -> 31` | `16 -> 16` | `11 -> 11` |
| `Transport::publish_request_in_place` | `0x110 -> 0x110` | `113 -> 113` | `56 -> 56` | `38 -> 38` |
| `Transport::publish` | `0x6c -> 0x6c` | `39 -> 39` | `32 -> 32` | `13 -> 13` |

The global 17-entry IRQ/barrier sequence is exact. Decoded load/store mnemonic
order is exact for `free_host_context` and `publish_request_in_place`; the
process-local free-list recorder separately pins address, width, value, and
operation order. The normalized clean-B6 packet/MMIO transition gate passes.
The full 114-stream residual with exact per-symbol hashes is in the checked
manifest and the uniquely named residual log below.

Hardware qualification was subsequently run with the exact archived images and
strict receiver/ping/TX-failure accounting. The original candidate completed
two of three runs: the complete runs measured 15.5 and 14.9 Mbit/s TCP, zero of
21,402 UDP datagrams lost, 20/20 final pings, and zero TX failures. The third
measured 13.9 Mbit/s TCP but produced no final UDP receiver report and only
19/20 pings, despite zero TX failures. The interleaved exact typed-VIF parent
completed its paired run at 14.6 Mbit/s TCP, zero UDP loss, 20/20 pings, and
zero TX failures; the established parent sample remained complete.

The first two timing experiments were inconclusive when applied directly to
the full candidate. Forcing `rate_policy::rate_for_try_count` inline produced
one clean run and one 19/20-ping run. An initial parent-shaped validator build
completed traffic but reported five and three TX failures in two of three
runs. Both experiments were removed before constructing narrower boundaries.

A hardware-guided source bisect then isolated the failure:

* retaining the typed DTCM layout while restoring all runtime files produced
  the exact parent ELF byte-for-byte;
* typed allocator/free-list, host-driver, and lifecycle code with parent
  `tx.rs` completed 3/3 runs, as did its three interleaved parents;
* adding the early `tx.rs` context/completion/retry access conversion completed
  3/3 runs, as did its three interleaved parents;
* the complementary late publication/descriptor/release partition lost 1,628
  of 21,402 UDP datagrams in its first run, while its paired parent lost three;
* descriptor-only runs lost zero and 6,105 datagrams respectively;
* the final validator-only boundary lost 5,998 of 21,401 datagrams, while its
  paired parent lost 20.

The isolated validator used `HostContextAddress::from_raw`, whose stride check
compiled to `__aeabi_uidivmod` in both per-frame wrappers. The final fix keeps
typed field ownership, validates the bounded range/stride without division,
constructs the already-validated address through a documented unsafe
constructor, and keeps `host_prepared_context` out of line. The resulting
`prepare_host_frame_timing` and `emit_host_frame_descriptor_at` streams exactly
match the parent's normalized instruction hashes and restore the parent counts
of 25/6 and 33/12 instructions/memory operations.

The fixed candidate completed 3/3 interleaved runs:

| Run | TCP | UDP loss | Final ping | TX failures |
| --- | ---: | ---: | ---: | ---: |
| candidate 1 | 14.2 Mbit/s | 0/21,402 | 20/20 | 0 |
| candidate 2 | 12.4 Mbit/s | 3/21,402 | 20/20 | 0 |
| candidate 3 | 16.0 Mbit/s | 0/21,402 | 20/20 | 0 |

The corresponding parent controls measured 14.5, 14.5, and 13.2 Mbit/s TCP.
The first two lost 15 and one UDP datagrams with 20/20 pings and zero TX
failures. The third parent was itself degraded, losing 2,833 datagrams and
reporting one TX failure, while the immediately preceding candidate remained
clean. The fixed candidate therefore meets or exceeds the interleaved parent
distribution.

Final uniquely named artifacts were emitted after the complete deterministic
run with both exact-parent and B6 gates enabled:

```text
ELF       /tmp/xr819-dtcm-layout/xr819-firmware/target/thumbv5te-none-eabi/release/hif-startup
          cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
.text     size 0x10770
          18c7dab420e7c79ee6065ec826026be4b578906e8d5954b61355fbe76c1dc839
packed    /tmp/xr819-host-context-fast-validation-fix.bin
          711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
bootstrap
          48d858b8785220aa7c9a9c0898164da1ecc71b4b6218283b2687b3d2f821d197
checks    /tmp/xr819-fast-validation-final-check.log
          32743129fa33fdc6d25856c772e938c7a560ede823238f481c5165bf61941e8d
manifest  tools/host-context-codegen-manifest.json
          e7b15480a63bd41d58d9084f97d72781664362a3f617886500b63c8b3e70de56
```

The host-context migration is hardware-qualified.

### A.14 Link-map and sequence-state semantic layout

The fixed-DTCM range `0x040087b8..0x040089d8` is now represented by one
`LinkAndSequenceState` quarantine layout rather than an opaque byte family. The
layout follows the decoded vendor capacities and exact computed accesses:

```text
0x040087b8..0x040087d0  LinkMapHeader
  +0x14 u16 entry_count
  +0x16 u16 release_blocked_links
0x040087d0..0x04008890  16 LinkMapEntry records, stride 0x0c
  +0x00 host_link
  +0x01 interface
  +0x02 internal_link
  +0x03 inactivity
  +0x04 release_flags
  +0x05 auxiliary_flags
  +0x06 six-byte peer MAC
0x04008890..0x040089d0  10 x 16 volatile u16 sequence counters
0x040089d0..0x040089d2  internal-link allocation bitmap
0x040089d2..0x040089d8  unresolved tail
```

The ten internal-link rows and sixteen TIDs exactly preserve
`0x04008890 + internal_link * 0x20 + tid * 2`. Vendor `link_slot_alloc`
iterates slots `0..9`, treats slot 9 as its exhausted/reserved result, clears
sixteen counters for allocated slots other than 9, and stores allocation state
in the bitmap at `0x040089d0`. The host-link map separately allows sixteen
records and host link IDs below 15; these remain distinct ID spaces.

Current Rust sequence assignment and PAS release gating now derive count,
record fields, sequence counters, blocked-link state, and record flags from the
typed layout. Count-driven loops deliberately use unchecked address views to
preserve the parent's behavior if shared state is corrupt; no safe references
are created to retained-vendor state. BA/LMC and pending-list state beginning at
`0x04008ab8` remains opaque and outside this slice.

`tools/check-link-sequence-layout.py` rejects direct or synthesized family
addresses outside `dtcm.rs` and pins reviewed linked literals/xrefs. The
complete parent/candidate text-symbol inventory is pinned by
`tools/link-sequence-codegen-manifest.json`. All 197 sized text symbols retain identical bytes and instruction streams
relative to the qualified host-context parent. The complete ELF and packed
firmware image are also byte-for-byte identical, so `.data`, stack bounds,
IRQ/barrier ordering, packet/MMIO behavior, and all anonymous metadata remain
unchanged. No new hardware run is required for an image already qualified under
the host-context campaign.

Final deterministic artifacts:

```text
ELF       /tmp/xr819-link-state/xr819-firmware/target/thumbv5te-none-eabi/release/hif-startup
          cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    /tmp/xr819-link-sequence-layout.bin
          711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-link-final-check.log
          2a6ed95f0c79219baa61a483f09e43d80d8c6846a254283611c69cc7769fbccd
manifest  tools/link-sequence-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.15 BA/LMC/pending structural layout

The coupled fixed-DTCM range `0x04008ab8..0x04008e78` is now represented by
three documented shared-quarantine layouts rather than opaque byte families:

```text
0x04008ab8..0x04008ad8  BaLmcHeader
  +0x02 request-slot index
  +0x03 pending-request count
  +0x04/+0x08/+0x0c request words
  +0x14 TIM flags
  +0x16 request flags
  +0x18 JOIN retry state
  +0x19 scan state
  +0x1b BA policy enable
0x04008ad8..0x04008bb8  PendingBaLmcState
  +0x00/+0x04 pending TX head/tail
  +0x08 MAC BSSID mode
  +0x0b pending-service gate
  +0x48 current radio owner
  +0x4c radio wait-list head
  +0x54 deferred radio owner
  +0xbc..+0xc7 radio role/timer state
  +0xd0 LMC message controls
  +0xd1/+0xd2 BA counters
  +0xd3/+0xd4 LMC producer/consumer
0x04008bb8..0x04008e78  16 polymorphic LMC messages, stride 0x2c
  +0x00 kind
  +0x01 flags
  +0x04..+0x27 kind-specific payload
  +0x28 interface
  +0x29 completion state
```

The pending-list insertion/removal paths, JOIN radio-owner publication,
inter-VIF radio handoff, MAC BSSID publication, LMC ring allocation, type-7
completion message construction, completion-return wakeup, and radio timer
transition now derive addresses from `dtcm.rs`. Retained vendor and IRQ/FIQ code
still owns the same bytes, so APIs return volatile addresses and deliberately
preserve the parent's list ordering, cursor arithmetic, and unchecked ring
index calculation. No references to complete shared records are created.

`0x04008e78..0x04008f18` remains opaque. Decompiled BA session consumers reach
that following area, so the message-record boundary does not imply that the BA
state machine is exclusively owned or movable.

`tools/check-ba-lmc-pending-layout.py` rejects direct and synthesized family
addresses outside `dtcm.rs` and pins eight linked literals and ten decoded
literal xrefs. `tools/ba-lmc-pending-codegen-manifest.json` pins the complete
197-symbol parent/candidate inventory. The ELF and packed image are
byte-for-byte identical to the hardware-qualified link-state parent, so no
additional hardware run is required.

Final deterministic artifacts:

```text
ELF       /tmp/xr819-link-state/xr819-firmware/target/thumbv5te-none-eabi/release/hif-startup
          cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    /tmp/xr819-ba-lmc-pending-layout.bin
          711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-ba-lmc-final-check.log
          55c07ff9441f4af46f4a7733a9d524bf4d7167b2824bc24a00cec15a4caad895
manifest  tools/ba-lmc-pending-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.16 BA session records

The former opaque range `0x04008e78..0x04008f18` is four fixed-DTCM BA
session records with stride `0x28`:

```text
+0x00 u32 activity/in-use state
+0x04 six-byte peer MAC
+0x0a u8 TID
+0x0b u8 interface
+0x0c six unresolved bytes
+0x12 u16 timeout in 1024-us units
+0x14 0x14-byte scheduler timer object
```

The shape follows `bab_init`, which initializes four timers at
`base + index * 0x28 + 0x14`, and the allocation/find/teardown paths that use
activity, peer address, TID, interface, and timeout at the decoded offsets.
These remain retained-vendor-owned records; structural typing does not create
safe references or move the table.

`tools/check-ba-session-layout.py` rejects production literals and synthesized
base/stride forms outside `dtcm.rs`. The linked firmware contains no Rust
literal or decoded literal-load xrefs into this range. The complete 197-symbol
inventory, ELF, and packed image remain byte-for-byte identical to the
qualified BA/LMC parent, so no hardware rerun is required.

Final deterministic artifacts:

```text
ELF       /tmp/xr819-link-state/xr819-firmware/target/thumbv5te-none-eabi/release/hif-startup
          cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    /tmp/xr819-ba-session-layout.bin
          711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-ba-session-final-check.log
          5a5486765fb9fcb5f8684e2404915884f76ffbfc973815d1776c2b8a5b40b971
manifest  tools/ba-session-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.17 BA/link timers and network flag state

The fixed-DTCM range `0x04008f18..0x04008f48` now has the decoded shared
layout:

```text
+0x00 u8 deferred BA action
+0x01 u8 deferred BA interface
+0x02 unresolved byte
+0x03 u8 periodic-timer enable
+0x04 0x14-byte periodic timer
+0x18 0x14-byte transition timer
+0x2c u8 current ERP/HT network flags
+0x2d u8 accumulated beacon flags
+0x2e u8 changed/edge mask
+0x2f unresolved byte
```

`bab_link_state_check` publishes the deferred action/interface pair. Firmware
initialization enables and initializes the first timer at `0x04008f1c`; JOIN
and start-state paths use the second timer at `0x04008f30`.
`rx_beacon_update_erp_ht_flags` accumulates ERP/HT observations and
`event_flag_edge_detect` consumes the three bytes at `0x04008f44..0x04008f46`
without changing their volatile ordering.

The state remains vendor/timer owned and no safe complete-record references are
created. `tools/check-ba-link-event-layout.py` rejects production literals and
synthesized roots outside `dtcm.rs`; the linked Rust firmware contains no
literal or decoded literal-load xrefs into the range. The complete ELF remains
byte-identical to the qualified parent, so no hardware rerun is required.

Final deterministic artifacts:

```text
ELF       /tmp/xr819-link-state/xr819-firmware/target/thumbv5te-none-eabi/release/hif-startup
          cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    /tmp/xr819-ba-link-event-layout.bin
          711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-ba-link-event-final-check.log
          f6fd1347a8a609386abc0e38571704dd3e0cd7af5ed92dd5ed6cc75706233162
manifest  tools/ba-link-event-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.18 JOIN/scan control and LMC request ring

The adjacent range `0x040089d8..0x04008ab8` now has two structural shared
layouts.

`JoinScanControl` at `0x040089d8..0x04008a18` contains:

```text
+0x00/+0x04 scheduler word/deadline
+0x08/+0x09 beacon timer active/interface
+0x0c channel owner
+0x14 channel-use state
+0x18 alternate channel owner
+0x20 0x14-byte JOIN timer
+0x34 JOIN status
+0x36 start state
+0x37 interface state
+0x38 response status
+0x3c request word
```

`WsmResponseScratch` at `0x04008a18..0x04008ab8` contains a `0x0c` scan/control
prefix followed by thirty request pointers at `0x04008a24..0x04008a9c` and the
first twenty-eight request-status bytes at `0x04008a9c..0x04008ab8`.
The logical thirty-byte status ring crosses the old family boundary: status
bytes 28 and 29 are the first two bytes at `0x04008ab8..0x04008aba`, before the
producer and consumer cursors at `0x04008aba` and `0x04008abb`. This overlap is
now documented explicitly rather than treating the boundary as ownership.

The decoded shape follows beacon timer selection, channel-use registration,
JOIN/start/scan transitions, and the LMC request enqueue/collect/confirm paths.
All state remains retained-vendor/timer owned and no safe complete-record
references are created.

`tools/check-join-scan-layout.py` rejects production literals and synthesized
JOIN timer or LMC ring roots outside `dtcm.rs`. The linked Rust image contains
no literal or decoded literal-load xrefs into this range. The complete ELF is
byte-identical to the qualified parent, so no hardware rerun is required.

Final deterministic artifacts:

```text
ELF       /tmp/xr819-link-state/xr819-firmware/target/thumbv5te-none-eabi/release/hif-startup
          cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    /tmp/xr819-join-scan-layout.bin
          711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-join-scan-final-check.log
          a656b9089d29788967ee7e0e28e282e7bdcff625a07a544ebe743268e30d9b03
manifest  tools/join-scan-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.19 Command upload and channel/JOIN/scan overlay

The range `0x04008594..0x04008618` is now represented as one explicit overlay
rather than an undifferentiated byte array. The WSM command-15 handler uploads
`0x68` bytes at `0x04008594`; the final four uploaded bytes simultaneously form
the head of the channel-switch control view at `0x040085f8`.

The decoded tail is:

```text
+0x64 channel-switch overlay head / uploaded blob tail
+0x68 channel-switch active
+0x69 interface
+0x6c mode
+0x6d countdown
+0x6e channel
+0x70 JOIN mode
+0x71 JOIN flags
+0x72 saved register context
+0x74 rate configuration
+0x78 scan state
+0x79 scan flags
+0x7b TX-buffer free-count overlay
+0x7c scan word
+0x80 tail word
```

The translated MAC register-save path and scan activity publication now derive
`0x04008606` and `0x0400860c` from the typed layout. Their volatile widths and
observation order remain unchanged. The whole object remains shared with
retained command upload, channel-switch, JOIN, scan, register-save, and
TX-buffer code; no safe complete-record reference is exposed.

`tools/check-command-channel-overlay.py` rejects production literals and
synthesized command/channel roots outside `dtcm.rs`, and pins five linked
literals plus six decoded xrefs. The full ELF remains byte-identical to the
qualified JOIN/scan parent, so no hardware rerun is required.

Final deterministic artifacts:

```text
ELF       /tmp/xr819-link-state/xr819-firmware/target/thumbv5te-none-eabi/release/hif-startup
          cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    /tmp/xr819-command-channel-layout.bin
          711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-command-channel-final-check.log
          6c43760f1b4fe15d34957055c0e5398b096511dc8ae265b44bf59c81497bcf7b
manifest  tools/command-channel-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.20 LMC encryption roots and duplicate cache

The range `0x04008618..0x04008798` now contains a typed shared header followed
by the first 31 records of the receive duplicate cache:

```text
0x04008618 +0x00 u32 JOIN/beacon match state
           +0x04 u32 encryption-context free-list head
           +0x08 u32 encryption allocation generation
0x04008624        duplicate-cache record 0
record stride     0x0c
record fields     +0x00 six-byte peer MAC
                  +0x06 u16 identity/interface-sequence key
                  +0x08 u32 context/discriminator
```

The logical duplicate cache has 32 records. Records 0 through 30 occupy
`0x04008624..0x04008798`; record 31 begins at `0x04008798` and ends at
`0x040087a4`, crossing into the following `HostContextAccounting` family. This
computed overlap explains why the old `0x180` boundary could not imply
exclusive ownership.

The translated LMC pool reset now derives the encryption free-list head and
generation addresses from `dtcm.rs` while preserving the exact publication
sequence. Retained encryption allocation/free, JOIN/beacon matching, RX
duplicate suppression, and invalidation paths continue to share the state.

`tools/check-lmc-control-layout.py` covers the logical extent through
`0x040087a4`, rejects production literals and synthesized base/stride forms,
and pins one linked literal/xref. The complete ELF remains byte-identical to
the qualified command/channel parent, so no hardware rerun is required.

Final deterministic artifacts:

```text
ELF       /tmp/xr819-link-state/xr819-firmware/target/thumbv5te-none-eabi/release/hif-startup
          cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    /tmp/xr819-lmc-control-layout.bin
          711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-lmc-control-final-check.log
          0069ec6ee9825091de1d1a6d6aac7771d3704347133524e8291b626681728a30
manifest  tools/lmc-control-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.21 Peer-pipe table and pre-command tail

The former unknown range `0x04008544..0x04008594` is now decoded as:

```text
0x04008544  eight peer-pipe records, stride 0x08
  +0x00 six-byte peer MAC
  +0x06 state flags
  +0x07 signed aging/replacement counter
0x04008584  four u16 management counters
0x0400858c  u16 scan/channel observation
0x0400858e  two unresolved bytes
0x04008590  u32 pending/control root
```

`pipe_find_or_alloc_lower` scans records 0 through 3 and
`pipe_find_or_alloc_upper`/`pipe_find_by_mac_upper` scan records 4 through 7.
The allocation paths compare all six peer-address bytes, use state bit 0 as the
occupied marker, and maintain the signed replacement counter at `+0x07`.
Beacon processing applies the same aging transition across all eight records.

The tail fields are independently used by management RX, scan channel
validation, and pending-list reset. They remain retained-vendor owned, and no
safe record references are created.

`tools/check-peer-pipe-layout.py` rejects production literals and synthesized
base/stride forms outside `dtcm.rs`. The linked Rust image has no literal or
decoded literal-load xrefs into the range. The complete ELF remains
byte-identical to the qualified LMC-control parent, so no hardware rerun is
required.

Final deterministic artifacts:

```text
ELF       /tmp/xr819-link-state/xr819-firmware/target/thumbv5te-none-eabi/release/hif-startup
          cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    /tmp/xr819-peer-pipe-layout.bin
          711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-peer-pipe-final-check.log
          e15a37189ae0652e18fa9f9a8a1f70fe55e4601df7ddef185d611ee6c7689260
manifest  tools/peer-pipe-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.22 Scheduler/event/timer roots

The initialized scheduler root is now represented by two exact shared layouts:

```text
0x04001fcc  u32 global scheduler/radio exclusion mask
0x04001fd0  u32 secondary exclusion/task state
0x04001fd4  u32 pending scheduler events
0x04001fd8  u32 scheduler/runtime flags
0x04001fe6  u16 PHY startup mode
0x04001ff0  u16 analog/gain enable observation
0x04001ff4  u32 primary retained remap word
0x04001ffc  u32 secondary retained remap word
0x04002000  three u32 analog/calibration words
0x04002014  u32 intrusive timer-list head
0x04002018  end
```

Unresolved bytes remain opaque. The mixed scheduler and PHY naming reflects
actual retained consumers rather than exclusive subsystem ownership. The event
word and timer root remain live publication structures shared with vendor task,
IRQ, HIF, MIC, scan/JOIN, BA, power-save, and PHY paths.

Translated startup, scheduler event claiming/publication, timer insertion and
cancellation, host-TX gating, and PHY observation paths now derive these
addresses from `dtcm.rs`. Exact volatile operations, IRQ/FIQ masking, event-bit
ordering, and intrusive-list writes are unchanged.

`tools/check-scheduler-event-layout.py` covers `0x04001fcc..0x04002018`, permits
only the explicit diagnostic probe and packer-test fixtures, and pins 16 linked
literal words plus 25 decoded literal-load xrefs. The complete ELF remains
byte-identical to the qualified peer-pipe parent, so no hardware rerun is
required.

Final deterministic artifacts:

```text
ELF       /tmp/xr819-link-state/xr819-firmware/target/thumbv5te-none-eabi/release/hif-startup
          cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    /tmp/xr819-scheduler-event-layout.bin
          711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-scheduler-event-final-check.log
          91917c4f24d452b2d2597a767ed0250185adf883c9cc801910b1e3ab4586f3d8
manifest  tools/scheduler-event-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.23 Clock/TSF state, scheduler handlers, and timer schema

The scheduler support range `0x0400218c..0x04002234` now has exact typed
layouts:

```text
ClockParameterIsland 0x0400218c..0x040021b4
  +0x04 u32 MAC clock snapshot
  +0x08 u32 beacon/counter snapshot
  +0x0c u32 hardware counter cache
  +0x14 u32 conversion factor
  +0x1c u8  conversion mode
  +0x24 u32 correction offset

SchedulerHandlerTable 0x040021b4..0x04002234
  32 shared raw callback words, stride 4
```

`sched_main_loop` consumes the handler table in reverse mask-bit order using
`clz`, and `sched_register_task` writes the selected entry. Rust startup's five
callback installations now use a bounded typed address constructor while
preserving their exact indices and write order. Callback values remain raw code
addresses because retained vendor tasks are still production reachable.

The common retained timer object is now encoded as `TimerEntry`:

```text
+0x00 u32 next
+0x04 u32 previous-link pointer
+0x08 u32 deadline
+0x0c u32 callback
+0x10 u32 callback context
size 0x14
```

JOIN, BA-session, BA-periodic, and BA-transition timer fields use this structural
type. This does not grant safe references or exclusive ownership; scheduler and
interrupt paths mutate the intrusive links and callbacks remain mixed native
and vendor addresses.

`tools/check-scheduler-support-layout.py` covers the full clock/handler range,
rejects production literals and synthesized base/stride forms, and pins two
linked literal words plus two decoded literal-load xrefs. The complete ELF
remains byte-identical to the qualified scheduler-event parent, so no hardware
rerun is required.

Final deterministic artifacts:

```text
ELF       /tmp/xr819-link-state/xr819-firmware/target/thumbv5te-none-eabi/release/hif-startup
          cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    /tmp/xr819-scheduler-support-layout.bin
          711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-scheduler-support-final-check.log
          9e6754f2846170cde03642025f3bd063d688dbbce8c747ea346bd1012adbe83c
manifest  tools/scheduler-support-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.24 Embedded VIF timers

Each physical `0x3b0`-byte VIF record now exposes five proven `TimerEntry`
layouts:

```text
+0x0b0 timer 0
+0x0c4 timer 1
+0x0d8 timer 2
+0x184 link timer 0
+0x198 link timer 1
```

The first group exactly fills `+0x0b0..+0x0ec`; the second fills
`+0x184..+0x1ac`. Surrounding bytes remain opaque. Every timer retains the
common `next`, `previous_link`, `deadline`, `callback`, and `context` fields,
without creating references to state concurrently mutated by retained timer,
task, IRQ, or FIQ paths.

All three physical records retain the exact `0x3b0` stride. Record 2 remains a
synthetic/P2P-device/scan slot and is not assigned the operating semantics of
records 0 and 1 merely because its bytes share the same physical layout.

`tools/check-vif-timer-layout.py` covers the fifteen disjoint timer extents,
rejects production literals and synthesized timer-base forms, and records one
aligned in-range word with no decoded literal-load xref. The complete ELF
remains byte-identical to the qualified scheduler-support parent, so no
hardware rerun is required.

Final deterministic artifacts:

```text
ELF       /tmp/xr819-link-state/xr819-firmware/target/thumbv5te-none-eabi/release/hif-startup
          cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    /tmp/xr819-vif-timer-layout.bin
          711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-vif-timer-final-check.log
          ca6b1b828a41bafaba3533ba9761586388a85a0f592d401c86a03eafa1cf3727
manifest  tools/vif-timer-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.25 Overlapping power-save timer views

The power-save family retains two observed view starts at stride `0x104`, but
the proven logical schema extends to `+0x138`:

```text
base 0x040094d4 + interface * 0x104
+0x02a u8  global sleep/transition state
+0x030 u32 global timer duration
+0x040 u8  mode
+0x044 u16 flags
+0x054 u32 pending/accounting word
+0x05a u16 queue mask
+0x070 seven TimerEntry objects, contiguous through +0x0fc
+0x0fc u8  state
+0x118/+0x11c/+0x120 u32 timer durations/control words
+0x128 u32 interval
+0x134/+0x136 u16 counter/threshold
logical extent 0x138
```

View 0 begins at `0x040094d4`; view 1 begins at `0x040095d8`. Therefore view 0's
extension fields overlap view 1, while view 1's extension reaches
`0x04009710`, inside the existing `PowerSaveHifBoundary` quarantine. The seven
timers themselves remain within the `0x208` power-save family and end at
`0x040096d4` for interface 1.

The translated TX completion path now derives the two former fixed literals at
`0x040094fe` and `0x04009504` from the schema. Existing interface-relative
arithmetic and volatile ordering remain unchanged.

`tools/check-power-save-layout.py` covers the complete overlapping logical
extent, permits only the linker/layout boundary fixtures, and pins five linked
literal words and decoded literal-load xrefs. The complete ELF remains
byte-identical to the qualified VIF-timer parent, so no hardware rerun is
required.

Final deterministic artifacts:

```text
ELF       /tmp/xr819-link-state/xr819-firmware/target/thumbv5te-none-eabi/release/hif-startup
          cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    /tmp/xr819-power-save-layout.bin
          711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-power-save-final-check.log
          faa980a9d55e460c64c8b987a7102e4cac4c3c4bf711750c86f96e2fda9b16b6
manifest  tools/power-save-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.26 Retained HIF and MIC queue state

The range `0x04009720..0x0400993c` now has exact shared layouts:

```text
HifBufferState 0x04009720..0x04009754
  +0x00 producer / +0x04 consumer
  +0x08 four host-message buffer pointers
  +0x1c active transfer state
  +0x20/+0x24 pending head/tail
  +0x28/+0x2c completed/deferred head/tail
  +0x30 control word

LegacyHifSoftwareState 0x04009754..0x04009928
  +0x00 mode
  +0x04 pending/credit count
  +0x08 coalesce count
  +0x0c TimerEntry
  +0x20 32 RX buffer pointers
  +0xa0 RX consumer / +0xa4 RX state
  +0xa8 64-entry software TX pointer queue
  +0x1a8/+0x1ac TX producer/consumer
  +0x1cc/+0x1d0 sequence/transport state

MicCompletionState 0x04009928..0x0400993c
  active, pending head/tail, completed head/tail
```

The HIF and MIC queue controls share the same five-word
`DeferredTransferQueue` shape. This describes intrusive pointer publication and
completion ordering without creating references or claiming ownership over
request-embedded links mutated by retained scheduler and interrupt paths.

`tools/check-hif-mic-layout.py` covers the complete retained range. The linked
image contains three aligned in-range words but no decoded literal-load xrefs;
these are pinned as drift evidence rather than treated as proven consumers. The
complete ELF remains byte-identical to the qualified power-save parent, so no
hardware rerun is required.

Final deterministic artifacts:

```text
ELF       /tmp/xr819-link-state/xr819-firmware/target/thumbv5te-none-eabi/release/hif-startup
          cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    /tmp/xr819-hif-mic-layout.bin
          711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-hif-mic-final-check.log
          6aebbf9735d677d4c6167a7d10d31bd0ab9dcd36f95b9925bdcd7e66acfb3447
manifest  tools/hif-mic-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.27 PHY calibration reference prefix

The first `0x10` bytes of `PhyCoreState` are now an exact shared layout:

```text
0x0400993c u32 coefficient I
0x04009940 u32 coefficient Q
0x04009944 u32 scale I
0x04009948 u32 scale Q
0x0400994c end
```

The translated IQ-calibration path now derives all four word addresses from
`dtcm.rs`. Byte `0x04009945`, inside the scale-I word, is also independently
written by retained PHY reset logic, so the typed API exposes a bounded byte
overlay rather than pretending the word is observed only atomically.

The words remain shared with retained RF calibration consumers and cannot move
independently. No references are created over the volatile state.

`tools/check-phy-reference-layout.py` covers the exact prefix, rejects
production literals and synthesized base/stride forms, and pins two linked
literal words plus three decoded literal-load xrefs. The complete ELF remains
byte-identical to the qualified HIF/MIC parent, so no hardware rerun is
required.

Final deterministic artifacts:

```text
ELF       /tmp/xr819-link-state/xr819-firmware/target/thumbv5te-none-eabi/release/hif-startup
          cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    /tmp/xr819-phy-reference-layout.bin
          711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-phy-reference-final-check.log
          08774d2e2a39a15976b0e6e31f0af39b5e346d80f6daf95f75a91227df25ba6c
manifest  tools/phy-reference-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.28 PHY profile and channel state

The next `0x28` bytes of `PhyCoreState` are now an exact shared layout:

```text
0x0400994c +0x00 u8 primary state
           +0x02 u8 active profile
           +0x03 u8 transition phase/status
           +0x06 u16 selected channel
           +0x0d u8 profile-0 readiness
           +0x10 u8 auxiliary observation
           +0x11 u8 transition gate
           +0x13 u8 calibration stage
           +0x14 u8 profile-0 state
           +0x15 u8 profile-1 readiness
           +0x16 u16 profile-1 channel
           +0x20 u32 retained reference word
0x04009974 end
```

Production PHY, configuration, scan, and startup-observation paths now derive
these addresses from `dtcm.rs`. The standalone extension diagnostic retains one
explicit profile literal and is recorded as an allowed diagnostic-only source.
All volatile widths, branch order, and state-transition writes remain exact.

`tools/check-phy-profile-layout.py` covers the complete block, rejects other
production literals and synthesized base/stride forms, and pins 17 linked
literal words plus 37 decoded literal-load xrefs. The complete ELF remains
byte-identical to the qualified PHY-reference parent, so no hardware rerun is
required.

Final deterministic artifacts:

```text
ELF       /tmp/xr819-link-state/xr819-firmware/target/thumbv5te-none-eabi/release/hif-startup
          cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    /tmp/xr819-phy-profile-layout.bin
          711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-phy-profile-final-check.log
          2bdbd126bbaf833092e5f7a22fdcb297f538437eba6b56bce03f45c53a4f6819
manifest  tools/phy-profile-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.29 PHY measurement and calibration state

The next `0x38` bytes of `PhyCoreState` are now an exact shared layout:

```text
0x04009974 +0x00 u32 channel frequency kHz
           +0x08 u8 measurement/control byte
           +0x0e u16 sample-width control
           +0x10 u32 retained offset/table word
           +0x14 i16 correction
           +0x17 u8 override value
           +0x18 u8 silicon variant
           +0x1c i16 denominator
           +0x1e i16 correction offset
           +0x20 i32 measured value A
           +0x24 i32 measured value B
           +0x35 u8 retained PHY state
           +0x37 u8 zero-selection flag
0x040099ac end
```

Production PHY, TX, VIF, configuration, and startup-observation paths now derive
these addresses from `dtcm.rs`. The extension diagnostic retains explicit reads
of the two measured values and is recorded as diagnostic-only. Volatile widths,
calibration ordering, and state-machine observations remain unchanged.

`tools/check-phy-measurement-layout.py` covers the complete block, rejects other
production literals and synthesized base/stride forms, and pins 15 linked
literal words plus 22 decoded literal-load xrefs. The complete ELF remains
byte-identical to the qualified PHY-profile parent, so no hardware rerun is
required.

Final deterministic artifacts:

```text
ELF       /tmp/xr819-link-state/xr819-firmware/target/thumbv5te-none-eabi/release/hif-startup
          cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    /tmp/xr819-phy-measurement-layout.bin
          711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-phy-measurement-final-check.log
          9507a275c0917ab25b1efcd92853b647fdfa4f29eb3cc573ff863e66b895a62b
manifest  tools/phy-measurement-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.30 PHY channel cache and calibration controls

The next `0x30` bytes of `PhyCoreState` are now an exact shared layout:

```text
0x040099ac +0x00 two u32 channel-configuration cache words
           +0x18 u8 startup/diagnostic observation
           +0x22 u16 retained channel
           +0x24 u8 calibration state
           +0x25 u8 calibration auxiliary state
           +0x28 u32 retained control word
           +0x2c u32 selected rate/configuration table pointer
0x040099dc end
```

The two cache slots remain bounded, and the translated copy helper preserves its
existing `slot <= 1` validation before using the unchecked typed constructor.
Production PHY, configuration, scan, and startup-observation paths now derive
these addresses from `dtcm.rs`. The extension diagnostic retains one explicit
control-word read at `0x040099d4`.

`tools/check-phy-channel-cache-layout.py` covers the complete block, rejects
other production literals and synthesized base/stride forms, and pins eight
linked literal words plus fourteen decoded literal-load xrefs. The complete ELF
remains byte-identical to the qualified PHY-measurement parent, so no hardware
rerun is required.

Final deterministic artifacts:

```text
ELF       /tmp/xr819-link-state/xr819-firmware/target/thumbv5te-none-eabi/release/hif-startup
          cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    /tmp/xr819-phy-channel-cache-layout.bin
          711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-phy-channel-cache-final-check.log
          f5f6ff96d8bdaff10a417ba2b28ad40dd6a65323e9641f4527a573c1fbf4a19c
manifest  tools/phy-channel-cache-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.31 PHY table pointers and calibration controls

The final `0x30` bytes of `PhyCoreState` are now an exact shared layout:

```text
0x040099dc +0x00 opaque retained state
           +0x10 u32 calibration table A pointer
           +0x14 u32 calibration table B pointer
           +0x18 i32 retained state scale
           +0x28 i16 calibration threshold
           +0x2c u8 extended-settle flag
           +0x2d u8 table/control flag
0x04009a0c end
```

Production PHY initialization, calibration, transition, and observation paths
now derive these addresses from `dtcm.rs`. The extension diagnostic retains one
explicit state-scale read at `0x040099f4`. Volatile widths and calibration/MMIO
ordering remain unchanged. Together with A.27 through A.30, this removes the
remaining opaque bytes from the complete `0x0400993c..0x04009a0c`
`PhyCoreState` quarantine layout while preserving unknown fields as explicit
opaque subranges.

`tools/check-phy-table-control-layout.py` covers the complete block, rejects
other production literals and synthesized base/stride forms, and pins four
linked literal words plus four decoded literal-load xrefs. The complete ELF
remains byte-identical to the qualified PHY-channel-cache parent, so no hardware
rerun is required.

Final deterministic artifacts:

```text
ELF       /tmp/xr819-link-state/xr819-firmware/target/thumbv5te-none-eabi/release/hif-startup
          cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    /tmp/xr819-phy-table-control-layout.bin
          711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-phy-table-control-final-check.log
          fbdd98c82f377b443c66696764c97abe0fc986c229e7cb85e79bfe3e7cd9a276
manifest  tools/phy-table-control-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.32 PHY IQ-calibration workspace

The former opaque `PhyTail` at `0x04009a0c..0x04009c44` is now an exact
structural workspace derived from the retained `rf_calibrate_iq_dc` loop:

```text
0x04009a0c page 0, size 0x100
  +0x14       12 slots, stride 0x10
    slot +0x00 u32 axis-A value
         +0x04 u32 axis-B value
         +0x08 8 opaque bytes
  +0xd4       0x2c opaque page suffix
0x04009b0c page 1, identical size and slot layout
0x04009c0c result block, size 0x38
  +0x14       nine u32 result/control words
0x04009c44 end
```

The vendor loop advances its working index by two, multiplies it by eight, and
therefore addresses twelve slots at a `0x10` stride. Its second coefficient bank
uses the same offsets at `base + 0x100`; final writes occupy `base + 0x214`
through `base + 0x234`. Bytes not covered by those proven accesses remain opaque
inside each page rather than being treated as spare capacity.

No production Rust code currently accesses these slots directly, so this change
adds only bounded structural address constructors and layout assertions. The
archived vendor reference report remains the semantic evidence; the linked Rust
image contains only one in-range endpoint literal in `rust_main`.

`tools/check-phy-iq-calibration-layout.py` covers the complete block, rejects
production literals and synthesized base/stride forms outside `dtcm.rs`, and
pins that linked endpoint literal and decoded xref. The complete ELF remains
byte-identical to the qualified PHY-table-control parent, so no hardware rerun
is required.

Final deterministic artifacts:

```text
ELF       /tmp/xr819-link-state/xr819-firmware/target/thumbv5te-none-eabi/release/hif-startup
          cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    /tmp/xr819-phy-iq-calibration-layout.bin
          711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-phy-iq-calibration-final-check.log
          3539bf7e7f74d4877b6aae9ea38a92a627c5c24a22eded0c0716e6f6c6b75595
manifest  tools/phy-iq-calibration-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.33 Runtime register context and PAS backoff overrides

The first `0x1c` bytes of the former opaque `RuntimePrefix` are now an exact
shared layout:

```text
0x04002078 +0x00 four u32 saved MAC register-context words
           +0x10 u32 PAS backoff override-enable word
           +0x14 u32 PAS backoff override window
           +0x18 u32 retained backoff word, semantics unresolved
0x04002094 end
```

The remaining `0xf8` bytes through `0x0400218c` stay opaque debug-console state.
Translated MAC channel reprogramming and PAS reset/update paths now derive the
proven addresses from `dtcm.rs`; volatile widths and operation ordering remain
unchanged. Test fixtures retain explicit addresses to verify the vendor-visible
access trace.

`tools/check-runtime-register-backoff-layout.py` covers the complete typed
prefix, rejects other production literals and synthesized base forms, and pins
two linked literal words plus two decoded literal-load xrefs. The complete ELF
remains byte-identical to the qualified PHY-IQ-calibration parent, so no
hardware rerun is required.

Final deterministic artifacts:

```text
ELF       /tmp/xr819-link-state/xr819-firmware/target/thumbv5te-none-eabi/release/hif-startup
          cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    /tmp/xr819-runtime-register-backoff-layout.bin
          711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-runtime-register-backoff-final-check.log
          e5fcdbdc522a67ee66dd160cdd6ccb91374e4b3575a6d30d3a2320c2576e1556
manifest  tools/runtime-register-backoff-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.34 SDD profile and gain tables

The complete `0x130`-byte SDD configuration family is now structurally typed as
two `0x92`-byte profile banks plus a retained `0x0c`-byte tail:

```text
profile +0x00 11 u16 rate limits
        +0x16 16 three-byte channel records
        +0x46 u8 channel-record count
        +0x48 i16 AGC correction
        +0x4a i16 calibration coefficient
        +0x4c two i16 conversion values
        +0x50 two i16 RSSI coefficients
        +0x54 11 i16 per-rate RSSI scales
        +0x6a 0x28 retained bytes
profile 0: 0x040034b0
profile 1: 0x04003542
family end: 0x040035e0
```

Four gain coefficients at `0x040035ac..0x040035b4` are represented as a narrow
address view into the proven retained suffix of profile one rather than as a
second overlapping owner. Production SDD retention and PHY gain computation
now derive profile, channel-record, coefficient, and rate-scale addresses from
`dtcm.rs`. The unchecked constructors preserve the original validated profile,
rate, count, and record arithmetic with wrapping operations, avoiding checked
arithmetic or division in the generated firmware path.

`tools/check-sdd-profile-layout.py` covers the whole family, retains reviewed
extension-probe fixtures, rejects other production literals and synthesized
base/stride forms, and pins six linked literal words plus nine decoded
literal-load xrefs. The complete ELF remains byte-identical to the qualified
runtime-register/backoff parent, so no hardware rerun is required.

Final deterministic artifacts:

```text
ELF       /tmp/xr819-link-state/xr819-firmware/target/thumbv5te-none-eabi/release/hif-startup
          cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    /tmp/xr819-sdd-profile-layout.bin
          711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-sdd-profile-final-check.log
          3ceed21dce86ef3025df4352891ca61775cc457597c8f0a9221621595680255b
manifest  tools/sdd-profile-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.35 Wake context and duration sources

The complete wake-context family and its adjacent duration sources are now
structurally typed:

```text
0x040035e0 +0x00 four u32 retained clock/context words
           +0x10 32 u32 packet-RAM response pointers
0x04003670 +0x00 two u16 duration-source values
0x04003674 end
```

The retained vendor wake path writes the four leading words and response-pointer
array; translated wake reinitialization consumes the same 32-entry array and
two duration halfwords after RX, pipe, and register synchronization. The Rust
path now derives those addresses from `dtcm.rs` while preserving loop bounds,
volatile widths, and publication order.

`tools/check-wake-context-layout.py` covers both adjacent families, rejects
production literals and synthesized bases outside `dtcm.rs`, and pins one
linked literal word plus two decoded literal-load xrefs. The complete ELF
remains byte-identical to the qualified SDD-profile parent, so no hardware
rerun is required.

Final deterministic artifacts:

```text
ELF       /tmp/xr819-link-state/xr819-firmware/target/thumbv5te-none-eabi/release/hif-startup
          cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    /tmp/xr819-wake-context-layout.bin
          711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-wake-context-final-check.log
          a78efe826b161f7228d1de3628b43ba53316dc8281750a2af769f3073aa935fc
manifest  tools/wake-context-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.36 PHY gain-source records

The first `0x84` bytes of `PreConfigurationTables` are now an exact 22-record
source table used by retained `phy_build_gain_tables`:

```text
0x04002234 22 records, stride 0x06
  +0x00 u8 selector
  +0x01 u8 retained byte
  +0x02 i16 lower value
  +0x04 i16 upper value
0x040022b8 end
```

The retained builder copies and adjusts all 22 records before deriving its
80-entry hardware gain table. No production Rust code currently reads this
source table directly, so the migration adds only bounded structural addresses
and compile-time layout checks. The remainder of `PreConfigurationTables`
through `0x040034b0` stays opaque.

`tools/check-phy-gain-source-layout.py` covers the complete record table and
rejects production literals or synthesized base/stride forms outside
`dtcm.rs`. The current Rust ELF has no linked in-range literal or decoded xref;
the archived vendor decompilation and DTCM reference report remain the evidence
for the record count and stride. The complete ELF remains byte-identical to the
qualified wake-context parent, so no hardware rerun is required.

Final deterministic artifacts:

```text
ELF       /tmp/xr819-link-state/xr819-firmware/target/thumbv5te-none-eabi/release/hif-startup
          cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    /tmp/xr819-phy-gain-source-layout.bin
          711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-phy-gain-source-final-check.log
          2cb74b000a41cbb0c97c825e3a4f8a9e86914f1cfe7f8e962fc3575250de1c38
manifest  tools/phy-gain-source-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.37 Template-frame descriptors

The retained template initializer proves two `0x40`-byte descriptors at
`0x04003050..0x040030d0`. Each descriptor contains seven typed template-kind
bytes, associated flags and lengths, and five raw pointer words at offsets
`+0x04`, `+0x0c`, `+0x2c`, `+0x34`, and `+0x3c`. Bytes not written by the
initializer remain explicit opaque fields within the record.

The two records are structurally identical and selected with a bounded index.
Their pointer targets remain raw shared addresses because several backing areas
and beacon/filter consumers overlap or are not yet decoded. No production Rust
path currently accesses the descriptor records directly.

`tools/check-template-descriptor-layout.py` covers the complete table and
rejects production literals or synthesized base/stride forms outside
`dtcm.rs`. The current Rust ELF has no linked in-range literal or decoded xref;
the retained initializer remains the layout evidence. The complete ELF remains
byte-identical to the qualified PHY-gain-source parent, so no hardware rerun is
required.

Final deterministic artifacts:

```text
ELF       /tmp/xr819-link-state/xr819-firmware/target/thumbv5te-none-eabi/release/hif-startup
          cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    /tmp/xr819-template-descriptor-layout.bin
          711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-template-descriptor-final-check.log
          f7e038e473e021960d6fa07999a25eaf11e17d7badf779ed3a1e0347072fb7c4
manifest  tools/template-descriptor-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.38 Debug-console state

The remaining `0xf8` bytes of `RuntimePrefix` are now an exact retained console
layout:

```text
0x04002094 +0x00 u32 input length
           +0x04 u32 flags/halt state
           +0x08 TimerEntry
           +0x1c u32 memory command address
           +0x20 u32 memory command value
           +0x24 u32 registered-command count
           +0x28 32 u32 raw command-descriptor pointers
           +0xa8 80-byte input line buffer
0x0400218c end
```

The `dbg_console_readline` bound of `0x4f`, command registration ceiling of 32,
and timer operations prove the array sizes and offsets. Command pointers and
memory-command values remain raw shared words; no safe callback or memory API is
exposed. No production Rust path currently invokes this retained console.

`tools/check-debug-console-layout.py` covers the complete family and rejects
production literals or synthesized bases outside `dtcm.rs`. The current Rust
ELF has no linked in-range literal or decoded xref; retained console routines
remain the structural evidence. The complete ELF remains byte-identical to the
qualified template-descriptor parent, so no hardware rerun is required.

Final deterministic artifacts:

```text
ELF       /tmp/xr819-link-state/xr819-firmware/target/thumbv5te-none-eabi/release/hif-startup
          cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    /tmp/xr819-debug-console-layout.bin
          711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-debug-console-final-check.log
          f05e41df5ba3f8492fb7d0f4992363393fc38461f0e23fbef62be1c689e89f73
manifest  tools/debug-console-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.39 Context-completion accounting prefix

The `0x14` bytes immediately following TALA are now an exact shared layout:

```text
0x04008f6c +0x00 u32 root word
           +0x04 u8 external-context count
           +0x05 u8 class-0 internal-context count
           +0x06 u16 retained state
           +0x08 u16 allocation state
           +0x0a u16 pending/completion count
           +0x0c u32 coalescing state
           +0x10 u32 free/teardown state
0x04008f80 end
```

The retained allocation, free, completion, power-save, and HIF coalescing paths
prove the mixed byte, halfword, and word accesses. Translated TX now derives the
class-0 count byte from the field layout rather than adding five to the family
base. No safe aggregate reference is exposed because retained writers remain
reachable.

`tools/check-context-completion-layout.py` covers the whole prefix and rejects
production literals or synthesized aliases outside `dtcm.rs`. The current Rust
ELF has no standalone in-range linked literal or decoded xref because the byte
address is synthesized from adjacent roots. The complete ELF remains
byte-identical to the qualified debug-console parent, so no hardware rerun is
required.

Final deterministic artifacts:

```text
ELF       /tmp/xr819-link-state/xr819-firmware/target/thumbv5te-none-eabi/release/hif-startup
          cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    /tmp/xr819-context-completion-layout.bin
          711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-context-completion-final-check.log
          ac6797d6135d82193deb7bb48f8bc72dc34996091d579d4078cab4a1f1a58b39
manifest  tools/context-completion-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.40 Rejected disjoint beacon-IE index ownership

A candidate attempted to partition `0x04002578..0x04002984` as one selector
word followed by two `0x204`-byte IE-offset indexes. This shape follows
`ie_index_build`: each index has a `u32` count and up to 256 `u16` offsets, with
observed starts at `0x0400257c` and `0x04002780`.

That ownership interpretation is not sound. The first proposed index spans
`0x0400257c..0x04002780` and therefore contains `0x04002730`, which is also a
retained RF-initialization root and is published by translated PHY startup.
The candidate source gate exposed this cross-family overlap immediately. The
candidate was reverted before commit; no ELF or packed image was produced from
it as an accepted endpoint.

Future work must represent the beacon indexes as overlapping address views,
like the power-save schema. The RF side of that overlap is now represented by
the accepted address-only view in A.41. The bytes remain occupied quarantine
and are not free space.

### A.41 Overlapping RF-initialization view

Retained `rf_init_stage_a` proves one logical RF view rooted at `0x04002730`,
with accesses from `root - 0xac` through `root + 0x38`. The resulting observed
extent is `0x04002684..0x0400276c`. It contains several table/control pointers,
seven consecutive negative-offset words, and positive fields at `+0x14`,
`+0x18`, `+0x1c`, `+0x20`, `+0x24`, `+0x30`, and `+0x38`.

`RfInitializationObservedLayout` is deliberately not embedded in
`PreConfigurationTables`: its extent overlaps beacon IE-index storage and does
not confer physical ownership. Translated PHY startup now publishes
`RF_INITIALIZATION_ROOT` rather than a raw `0x04002730` literal, without
changing the surrounding MMIO or calibration sequence.

`tools/check-rf-initialization-view.py` covers the full logical extent, rejects
other production literals and synthesized aliases outside `dtcm.rs`, and pins
one linked root literal plus one decoded xref. The complete ELF remains
byte-identical to the qualified context-completion parent, so no hardware rerun
is required.

Final deterministic artifacts:

```text
ELF       /tmp/xr819-link-state/xr819-firmware/target/thumbv5te-none-eabi/release/hif-startup
          cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    /tmp/xr819-rf-initialization-layout.bin
          711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-rf-initialization-final-check.log
          4e099e8302bc876252ec3820c0e7819e822c9d96cd779bfc5e84d948ef3954d8
manifest  tools/rf-initialization-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.42 Overlapping beacon IE-index views

With the RF overlap represented separately, the two retained beacon IE indexes
are now modeled as address-only views:

```text
0x04002578 u32 active-index selector
0x0400257c index 0: u32 count plus 256 u16 offsets, logical size 0x204
0x04002780 index 1: u32 count plus 256 u16 offsets, logical size 0x204
0x04002984 logical end of index 1
```

`ie_index_build` bounds the count at 256 and alternates the current and prior
indexes using the selector. These are not embedded owners: index 0 overlaps the
accepted RF-initialization view, including its root at `0x04002730`. The APIs
therefore expose only bounded addresses and preserve the underlying
`PreConfigurationTables` bytes as opaque quarantine.

`tools/check-beacon-ie-index-view.py` covers both logical indexes, treats the RF
checker as a reviewed overlapping owner, and rejects other production literals
or synthesized aliases. Its one linked literal and decoded xref are the shared
RF root already reviewed in A.41. The complete ELF remains byte-identical to the
qualified RF-initialization parent, so no hardware rerun is required.

Final deterministic artifacts:

```text
ELF       /tmp/xr819-link-state/xr819-firmware/target/thumbv5te-none-eabi/release/hif-startup
          cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    /tmp/xr819-beacon-ie-index-layout.bin
          711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-beacon-ie-index-final-check.log
          66ff81148d8a1186b46026b6576f5d666d9da7720b5cf07598d8184e10d6a6e6
manifest  tools/beacon-ie-index-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.43 Beacon-filter physical storage

The complete `0x6cc`-byte beacon-filter storage rooted at `0x040022b8` is now
represented in the physical `PreConfigurationTables` layout:

```text
0x040022b8 +0x000 u32 stored beacon length
           +0x004 700-byte stored beacon image
           +0x2c0 u32 active IE-index selector
           +0x2c4 IE index 0, size 0x204
           +0x4c8 IE index 1, size 0x204
0x04002984 end
```

The 700-byte cap comes from both `ie_index_build` and
`beacon_filter_check_and_store`; the latter copies the complete beacon after its
length word and flips the selector after publication. The index count and 256
halfword offsets retain the A.42 layout. This physical quarantine does not
invalidate the overlapping RF address view: no safe aggregate reference or
exclusive owner is exposed.

`tools/check-beacon-filter-storage-layout.py` covers the complete physical
family, recognizes the adjacent gain-table boundary and reviewed overlapping
RF/IE checkers, and rejects other production literals or synthesized aliases.
Its one linked literal and decoded xref remain the shared RF root. The complete
ELF remains byte-identical to the qualified beacon-IE-index parent, so no
hardware rerun is required.

Final deterministic artifacts:

```text
ELF       /tmp/xr819-link-state/xr819-firmware/target/thumbv5te-none-eabi/release/hif-startup
          cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    /tmp/xr819-beacon-filter-storage-layout.bin
          711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-beacon-filter-storage-final-check.log
          ba5113373d5457a9d4a1162d8d4aeedf60dbebc5bac98281b25a6cdc0f0d057a
manifest  tools/beacon-filter-storage-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.44 Template backing buffers

The retained descriptor initializer partitions the complete
`0x040030d0..0x040034b0` tail into three two-buffer classes:

```text
0x040030d0 two 0x100-byte primary buffers
0x040032d0 two 0x060-byte secondary buffers
0x04003390 two 0x090-byte tertiary buffers
0x040034b0 end
```

For descriptor index `i`, the initializer publishes primary at
`0x040030d0 + i * 0x100`, secondary at `0x040032d0 + i * 0x60`, and tertiary at
`0x04003390 + i * 0x90`. The three arrays exactly fill the prior opaque `0x3e0`
bytes without gaps. Their contents remain raw shared bytes because frame-type
consumers retain heterogeneous payload semantics.

`tools/check-template-backing-layout.py` covers the complete tail, recognizes
the adjacent descriptor and SDD checker boundaries, and rejects production
literals or synthesized base/stride aliases outside `dtcm.rs`. The current Rust
ELF has no linked in-range literal or decoded xref; the retained initializer is
the layout evidence. The complete ELF remains byte-identical to the qualified
beacon-filter-storage parent, so no hardware rerun is required.

Final deterministic artifacts:

```text
ELF       /tmp/xr819-link-state/xr819-firmware/target/thumbv5te-none-eabi/release/hif-startup
          cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    /tmp/xr819-template-backing-layout.bin
          711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-template-backing-final-check.log
          2f67cf4e8740e28ec9a02d3851a406cc602476241115652233f31873d622d7a3
manifest  tools/template-backing-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.45 Internal TX context initializer fields

Each of the three fixed `0x170`-byte internal TX contexts now names the fields
proven by `tx_ctx_pool_init`:

```text
context +0x04 u32 next-free link
        +0x1c u32 802.11 header pointer
        +0x70 u16 result/status value
        +0xc4 u32 cipher-buffer pointer
```

All intervening and trailing bytes remain opaque within the exact record size.
Translated pool initialization derives the four offsets with `offset_of!`
constants while preserving the original four source lines, unchecked pointer
arithmetic, volatile widths, write order, and free-list construction. The pool
still consists of one free head followed by exactly three records.

`tools/check-internal-context-layout.py` covers
`0x04009080..0x040094d4`, retains the reviewed linker and DTCM-check ownership
fixtures, and pins eight linked base literals plus sixteen decoded xrefs. The
complete ELF remains byte-identical to the qualified template-backing parent,
so no hardware rerun is required.

Final deterministic artifacts:

```text
ELF       /tmp/xr819-link-state/xr819-firmware/target/thumbv5te-none-eabi/release/hif-startup
          cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    /tmp/xr819-internal-context-layout.bin
          711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-internal-context-final-check.log
          459ad69e761d55064bcb3cc64e1b499623b6cb9af87670fbf11c3fe7fd0c66a2
manifest  tools/internal-context-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.46 Internal TX PAS and completion fields

The internal context records now encode the same translated completion/PAS
field contract previously expressed only by raw offsets in `ContextAddress`.
The outer record names completion state through `+0x53`; an exact `0x80`-byte
`InternalPasContext` begins at `+0x54` and names frame/control/rate fields,
timestamps, terminal status, retry count, ownership, duration, descriptor
state, frame-state address, sequence metadata, interface/link bytes, the
internal cipher-buffer pointer at outer `+0xc4`, and the retained crypto tail.

This is structural shared quarantine, not a claim that host and internal
contexts have identical ownership. In particular, the internal cipher pointer
occupies bytes that remain opaque in the host PAS schema. The final
`+0xd4..+0x170` context tail remains opaque for unresolved encryption and
aggregation overlays.

All compile-time offsets match the existing translated raw-address contract.
No generated symbol, ELF byte, or packed-image byte changes, so the existing
internal-context source/linked/codegen gates and hardware qualification remain
valid.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-internal-context-pas-final-check.log
          99fd1e1eda6222d08bc7e34fe56e7f7fc2b2266c69abe14da4f46027355a8899
```

### A.47 Overlapping completion-ring view

Retained `tx_complete_tala_adapt` treats `0x04008f80..0x04009080` as a
64-entry ring of raw context pointers. The ring base is
`ContextCompletionPrefix + 0x14`; consumer and producer cursors remain at
prefix `+0x0c` and `+0x10`. Each drain clears the selected pointer before
advancing the consumer modulo 64.

The logical ring crosses the old physical family boundary: entries 0 through 58
occupy `PreInternalContextQuarantine`, while entries 59 through 63 occupy
`InternalContextPrefix`. It is therefore represented as an address-only
`CompletionRingObservedLayout`, not as a second embedded owner. This also
explains why `0x0400906c` simultaneously appears as the internal-context prefix
root in retained initialization evidence.

`tools/check-completion-ring-view.py` covers the complete logical ring,
recognizes both adjacent family checkers, and rejects production literals or
synthesized base/stride aliases outside `dtcm.rs`. The current Rust ELF has no
linked in-range literal or decoded xref because translated completion ownership
now lives in native state. The complete ELF remains byte-identical to the
qualified internal-PAS parent, so no hardware rerun is required.

Final deterministic artifacts:

```text
ELF       /tmp/xr819-link-state/xr819-firmware/target/thumbv5te-none-eabi/release/hif-startup
          cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    /tmp/xr819-completion-ring-layout.bin
          711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-completion-ring-final-check.log
          8306fdbc560629d8d507bb7d4dc6e0546b74ec62cf12c52eb18df7e258911190
manifest  tools/completion-ring-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.48 Internal-context prefix overlay

The physical `0x14`-byte prefix at `0x0400906c..0x04009080` now names its
proven first word as the retained 24-bit IV/PN seed initialized by
`tx_ctx_pool_init`. The remaining `0x10` bytes stay opaque.

This field deliberately aliases completion-ring entry 59 from A.47. The
physical prefix type records the internal-context initialization meaning, while
the completion-ring type remains an address-only view of the retained drain
contract. Neither representation grants exclusive ownership or a safe shared
reference.

`tools/check-internal-context-prefix-layout.py` covers the physical prefix,
recognizes both overlapping completion-ring and adjacent pool checkers, and
rejects production literals or synthesized aliases outside `dtcm.rs`. The
current Rust ELF has no linked in-range literal or decoded xref. The complete
ELF remains byte-identical to the qualified completion-ring parent, so no
hardware rerun is required.

Final deterministic artifacts:

```text
ELF       /tmp/xr819-link-state/xr819-firmware/target/thumbv5te-none-eabi/release/hif-startup
          cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    /tmp/xr819-internal-prefix-layout.bin
          711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-internal-prefix-final-check.log
          de8561a400d6f357fdbb0858b5854cd117e296909be020f56635ebb86b4bf0b2
manifest  tools/internal-context-prefix-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.49 Typed internal-context address schema

`InternalContextAddress` now derives all translated internal TX field addresses
from `InternalTxContext` and `InternalPasContext` offsets rather than repeating
an undocumented numeric schema. It provides bounded construction by pool index
and an explicitly unsafe raw constructor for call sites that have already
established internal-context identity.

The API covers every internal field currently consumed through the shared
`ContextAddress` abstraction: list linkage, borrowed header/frame address,
completion fields, PAS frame/control/rate state, timestamps, retry and
ownership state, descriptor/frame-state pointers, sequence/interface/link
metadata, and the internal cipher buffer. Exact tests pin all three record
bases and representative fields through the final record.

This is address typing only. It creates no reference to vendor/IRQ/FIQ-mutated
storage and does not change the existing volatile access contract. The complete
ELF remains byte-identical to the qualified internal-prefix parent, so no
hardware rerun is required.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-internal-address-final-check.log
          57b46de63917ce28ec9319fc8090490d8502ebb9a42c1ea043b6ff5c2ebcebfc
```

### A.50 Derive translated internal-context offsets from layouts

The shared `ContextAddress` accessors in `tx.rs` no longer carry 31 numeric
internal-context offsets. `InternalContextAddress` exposes associated offset
constants derived with `offset_of!`, and the existing `host_or_offset` hot path
uses those constants while retaining the exact wrapping addition and host-range
branch generated by the qualified parent.

An attempted direct closure-based host/internal address dispatch changed the
complete ELF and was rejected before acceptance. Keeping the original runtime
arithmetic while deriving its constants from the typed schema restores exact
ELF identity and avoids adding validation, division, or branch work to the
per-frame path.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-internal-offset-use-final-check.log
          c0817c350d16c5e7957cbac93afe7498fd6a8ea59d76afb037a43e7aecf26a1d
```

### A.51 Physical power-save/HIF boundary tail

The physical `PowerSaveHifBoundary` at `0x040096dc..0x04009720` now encodes
the portion proven by logical power-save view 1. Its first `0x34` bytes are
that view's extension tail: three duration words, an interval word, and the
final counter/threshold halfwords at physical offsets `+0x30/+0x32`. The
remaining `0x10` bytes before the HIF roots stay opaque.

This does not partition the two power-save views into owned records. It merely
records the already-qualified overlap in the physical top-level quarantine,
with compile-time assertions tying the physical offsets to the logical schema.
The existing power-save source/linked/codegen gate covers the complete typed
portion through `0x04009710`. The complete ELF remains byte-identical to the
qualified internal-offset parent, so no hardware rerun is required.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-power-save-boundary-final-check.log
          bc8b179c7621a013064a98211ef0fb33ef025eec63694fad82157a5cc5291934
```

### A.52 Pre-VIF link bitmap

The `0x20`-byte pre-VIF header at `0x04003e78..0x04003e98` now names the
retained link/BA bitmap word at `+0x18` (`0x04003e90`). Retained BA,
aggregation, link-reset, and queue-building code all read or update this exact
word. The surrounding `0x1c` bytes remain explicit opaque quarantine.

`PRE_VIF_LINK_BITMAP` is derived from the top-level member and field offsets.
Exact tests pin the field eight bytes before the first VIF record.
`tools/check-pre-vif-header-layout.py` covers the complete header and recognizes
both the preceding low-MAC/PAS and following VIF family checkers. The translated
Rust ELF currently has no linked in-range literal or decoded xref. The complete
ELF remains byte-identical to the qualified power-save-boundary parent, so no
hardware rerun is required.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-pre-vif-final-check.log
          8ae06fa05b35df0bce345ff6138172919745064048af65f6dc54ab65eafc160f
manifest  tools/pre-vif-header-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.53 Physical power-save view prefixes

The physical `0x208`-byte `PowerSaveFamily` is now represented as two exact
`0x104`-byte prefixes at the observed per-interface view starts. Each prefix
names the proven sleep state, global timer duration, mode/flags, pending and
queue-mask fields, seven embedded timers, and the state byte at `+0xfc`.
Opaque gaps remain explicit.

These prefixes do not truncate the logical power-save views. Each logical view
continues through `+0x138`; view 0 therefore overlaps physical prefix 1, and
view 1 continues into the typed PS/HIF boundary from A.51. The physical type
records the backing layout without asserting exclusive ownership or independent
record lifetimes. Exact tests tie the second physical prefix to logical view 1.
The complete ELF remains byte-identical to the qualified pre-VIF parent, so no
hardware rerun is required.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-power-save-prefix-final-check.log
          c25015f4783b10a1da9c1a04fa2265af142ac9a1204e73fbf9213355afd72fb9
```

### A.54 Physical completion-ring backing

The physical `0xec` bytes at `0x04008f80..0x0400906c` now encode the first
59 raw completion-ring entries directly as `[SharedU32; 59]`. Entries 59
through 63 continue to overlap the internal-context prefix and remain covered
by the address-only 64-entry logical view from A.47.

This removes the false implication that the physical bytes were semantically
unknown while preserving their shared volatile quarantine status. It does not
create references or change the native Rust completion-ring owner. The existing
completion-ring source/linked/codegen gate covers the full logical extent. The
complete ELF remains byte-identical to the qualified power-save-prefix parent,
so no hardware rerun is required.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-completion-backing-final-check.log
          84791400ddd928fd13036dfacdebfadf71cc01f53f30be5070143d7ffa21c4d8
```

### A.55 Internal-prefix completion backing

`InternalContextPrefix` now records all five overlapping completion-ring words.
Its first word is explicitly named `iv_seed_or_completion_entry_59`, preserving
both the retained IV/PN initialization meaning and the completion drain's ring
meaning. The following four words are entries 60 through 63.

Together with A.54, the entire physical `0x100`-byte completion-ring backing is
now structurally typed. The logical address-only view remains authoritative for
ring indexing, while the prefix type records the cross-family alias without
claiming independent ownership. The complete ELF remains byte-identical to the
qualified completion-backing parent, so no hardware rerun is required.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-internal-prefix-backing-final-check.log
          66eacc129d474a6c71b031e3c346d611af0a71cad105a7319157ce5885557b52
```

### A.56 Complete power-save/HIF boundary

Retained `ps_try_enter_sleep_all` proves two additional halfwords at logical
power-save offsets `+0x138/+0x13a`. The first carries the scan-completion value
passed when all interfaces enter the terminal sleep state; the second is the
per-view sleep-vote count accumulated across active interfaces. For interface
1 these map to physical `0x04009710` and `0x04009712`.

The final boundary word at `0x0400971c` is retained beacon/TIM state: beacon
processing writes it and TBTT wake scheduling reads the surrounding state.
`PowerSaveObservedLayout` now extends through `+0x13c`, while the physical
`PowerSaveHifBoundary` names both halfwords and the final word with only the
interior `0x8` bytes left opaque. The power-save drift gate now covers through
the exact HIF root at `0x04009720`. The complete ELF remains byte-identical to
the qualified internal-prefix-backing parent, so no hardware rerun is required.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-power-save-final-tail-check.log
          9abf06c98c45731ee2676d5d6f61e85e632e780b778d0a164d05061ea40bea4e
```

### A.57 Retained HIF ring controls

The final previously opaque `0x1c` bytes of `LegacyHifSoftwareState` now name
seven retained HIF ring-control words at `0x04009904..0x04009920`:
queue depth, queued TX producer/consumer, input producer/consumer, input ring
mask, and input descriptor base. The existing sequence and transport words
remain at `0x04009920/+0x04`.

These names follow the retained queue, RX dispatch, TX-confirm drain, and
sequence-publication arithmetic. Some words participate in overlapping
negative-offset views from adjacent HIF roots; structural typing therefore
remains shared volatile quarantine rather than exclusive ownership. Exact
assertions and tests pin every word. The existing HIF/MIC source, linked-xref,
and codegen gate covers the complete range. The complete ELF remains
byte-identical to the qualified power-save-tail parent, so no hardware rerun is
required.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-hif-ring-controls-final-check.log
          c2e0dfed5d1f7c28e50c27bf44560ce57c0877ec200bb24d44d2835728d1d07d
```

### A.58 Power-save wake statistics and leading state

The leading `0x2a` bytes of each physical power-save prefix are now decoded.
They contain three wake-stat control bytes, the measured wake duration, minimum/
sum/maximum hardware-register samples, minimum/sum/maximum elapsed-time
samples, TX-completion state, next-TBTT state, doze state, and requested PM
mode.

The wake-stat accumulator proves the six words at `+0x08..+0x20`: on its first
sample it initializes min/sum/max triples, then updates extrema and totals on
subsequent wakes. `ps_wake_sequence`, beacon timing, TX completion, PM command,
and doze-selection paths independently prove the remaining leading fields.
Both the logical overlapping view and each physical `0x104` prefix carry the
same layout. Bounded address APIs and exact tests pin representative fields in
both interfaces. The complete ELF remains byte-identical to the qualified HIF
ring-control parent, so no hardware rerun is required.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-power-save-wake-stats-final-check.log
          900044b2c6dad7076a53fa4a39b9d4b3359fe1f17599f91a366f9b508b09142d
```

### A.59 Power-save timing and activity prefix

The next power-save prefix slice now names the wake-stat sample counter at
`+0x2b`, resume state at `+0x2d`, beacon timing reference at `+0x34`, joined
state word at `+0x38`, wake lead time at `+0x3c`, and the mode/activity/pending/
timer bytes at `+0x40..+0x43`.

Beacon processing updates the timing reference, join setup publishes the state
word, wake scheduling consumes the lead-time value, and the sleep/reevaluation
paths repeatedly test the activity byte. The retained timer initializer writes
the final byte. Both logical and physical schemas carry these exact offsets,
with bounded address APIs and cross-interface tests. The complete ELF remains
byte-identical to the qualified wake-statistics parent, so no hardware rerun is
required.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-power-save-timing-final-check.log
          a64b4c6bd8d57d91855162cce0c643e229da3699b5597633eb83a3b7e9b48bd5
```

### A.60 Power-save control and U-APSD state

The remaining bytes between the core flags at `+0x44` and the first timer at
`+0x70` are now typed. They include sleep-transition flags, the wake-stat
timestamp, signed backoff adjustment, reset/beacon/pending-control/U-APSD state
bytes, pending state and flags, wake reason, queue mask, and five TX-followup/
U-APSD timing/configuration words.

The retained sleep, beacon, TX-completion, poll/QoS-null, join-reset, and U-APSD
setup/restart paths independently exercise these addresses. The typed widths
preserve their observed byte, halfword, and word accesses; names do not imply
exclusive Rust ownership. Both logical and physical prefix schemas now remain
structured continuously through the seven embedded timers. The complete ELF
remains byte-identical to the qualified power-save-timing parent, so no hardware
rerun is required.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-power-save-control-final-check.log
          c32cdd4870f6945db345e71e6cb133e8c9b13b684496a8dd6c3df1dd085a08e4
```

### A.61 Power-save post-timer state

The final seven bytes of each physical `0x104` power-save prefix are now typed:
TX-completion pending, beacon RX state, beacon rate, and a 32-bit wake-timer
delay at `+0x100`. The logical view keeps an explicit opaque `+0x104..+0x118`
span before its previously decoded duration extension.

Retained TX completion, beacon processing, and TBTT wake scheduling separately
exercise these fields. Exact tests pin both interface-0 addresses and the
interface-1 wake-delay word at `0x040096d8`, immediately inside the physical
PS/HIF boundary. The complete ELF remains byte-identical to the qualified
power-save-control parent, so no hardware rerun is required.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-power-save-post-timer-final-check.log
          16801b8d9c3fec3c113ac27bc2c3979eee4e988e91f8b32fc0f25cbb00cfe2a9
```

### A.62 Power-save beacon wake scheduling state

The logical extension at `+0x104..+0x118` is now fully typed as beacon airtime,
pre-TBTT offset, beacon and backoff intervals, next-wake deadline, TX-completion
state, wake-timer active state, one unresolved byte, and the beacon-timing
adjustment flag.

Beacon RX recomputes airtime and timing, TBTT scheduling consumes these values
to publish the wake deadline, and reset/sleep paths clear or test the state
bytes. The second logical view places this extension inside the physical
PS/HIF boundary, as required by the established overlap. Exact bounded-address
tests cover both views. The complete ELF remains byte-identical to the qualified
post-timer parent, so no hardware rerun is required.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-power-save-wake-schedule-final-check.log
          7aac0a8c19cc7f720f49f92b212559b626719a68378a109c723279dff10b7480
```

### A.63 Power-save interval and error tail

The final unresolved words in the logical power-save tail are now typed. The
word at `+0x124` is retained PM/mode control; `+0x12c` is the maximum backoff
used by interval growth; `+0x130` is the last-beacon timestamp. The mode-error
report latch at `+0x13c` prevents duplicate event `0x0805` indications.

The logical view is now `0x140` bytes including alignment padding. For interface
1, the error latch maps to physical `0x04009714` inside the PS/HIF boundary,
which is represented there as the same shared byte. Exact tests pin both view
instances. The complete ELF remains byte-identical to the qualified wake-
scheduling parent, so no hardware rerun is required.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-power-save-interval-tail-final-check.log
          b2f6f5daa2a8c691e980c37b71b33b010acd080cb837b94a1305725a24f58911
```

### A.64 Initialized HIF coalescing controls

The initialized island at `0x040011ac..0x040011bc` is now an exact
`InitializedHifControl` record. Its first two words are the queued-depth and
pending-count shadows maintained by host-message publication. The tail contains
the coalescing enable byte, pending/ring-depth/count thresholds, and the 32-bit
coalescing delay.

These are the retained defaults consumed by `hif_confirm_coalesce_hold`:
enabled, thresholds 10/4/2, and delay 8000 timer ticks. The record ends exactly
at the 32-entry IRQ callback table. Narrow address APIs and tests pin every
field. `tools/check-initialized-hif-control-layout.py` adds source, linked-xref,
and exact-parent codegen gates. The complete ELF remains byte-identical to the
qualified power-save-interval parent, so no hardware rerun is required.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-initialized-hif-control-final-check.log
          4529d9ac7536a1cb66bdadfc98f72f33191fcf467f7c9165c04bc8b201399331
manifest  tools/initialized-hif-control-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.65 Initialized A-MPDU telemetry

The initialized `0x28`-byte island at `0x040012a0..0x040012c8` is now an exact
telemetry record. Its first four words hold TX error/count accounting and a
64-bit accumulated duration. Four management-RX counters follow. The word at
`+0x20` remains semantically unresolved but structurally shared, and the final
word is the retained TX retry counter.

The completion path updates the TX count/duration quartet, management RX updates
the four middle counters, and pipe retry handling increments the final word.
Bounded address APIs expose the four-counter management array without creating
references. `tools/check-ampdu-telemetry-layout.py` pins the source range, one
linked base literal, two decoded xrefs, and exact-parent codegen. The complete
ELF remains byte-identical to the qualified initialized-HIF-control parent, so
no hardware rerun is required.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-ampdu-telemetry-final-check.log
          fc5aef2925921581135857fbb8cfd03c1fd2a87fa1fc88078a973f68bb96a632
manifest  tools/ampdu-telemetry-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.66 Initialized beacon, TSF, random, and timer controls

The initialized words at `0x04001420..0x04001440` are now an exact eight-word
record. Proven fields include beacon state, RX-indication state, TSF resync
state, the firmware random LFSR, the low TSF accumulator word, and the retained
timer counter. The two words at `+0x14/+0x18` remain structurally typed but
semantically unresolved.

Production literals for `0x04001428`, `0x0400142c`, `0x04001430`, and
`0x0400143c` were migrated in platform, scan, TX, and host-TX code to narrow
`dtcm.rs` address APIs while preserving volatile access and exact arithmetic.
`tools/check-initialized-control-words-layout.py` pins the complete source range,
three linked TSF-state literals, thirteen timer-counter literals, their decoded
xref multiset, and exact-parent codegen. The complete ELF remains byte-identical
to the qualified A-MPDU-telemetry parent, so no hardware rerun is required.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-initialized-control-final-check.log
          324781b52c562d545b9cf104ea8eec2f76c38b654862a1a296b0d556bab1071b
manifest  tools/initialized-control-words-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.67 Initialized queue/pipe mappings

The initialized `0x0c` bytes at `0x040002d8..0x040002e4` are now an exact
`QueuePipeMappings` record: four pipe-order bytes, four WSM queue-to-access-
category bytes, and the reverse access-category-to-queue map.

All production literals in MAC initialization, TX preparation/publication, and
host-TX scheduling were replaced with bounded or base typed address APIs. The
hot runtime additions and volatile byte accesses remain unchanged. An initial
source rewrite shifted Rust panic-location line metadata and changed the full
ELF despite identical executable symbols; restoring original source line counts
recovered exact ELF identity. `tools/check-queue-pipe-mappings-layout.py` pins
three linked literals and the complete decoded xref multiset. The complete ELF
remains byte-identical to the qualified initialized-control-words parent, so no
hardware rerun is required.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-queue-pipe-mappings-final-check.log
          25d195dfdf4d2be9f6875980e2073d9c55e7d5b6723ee3e386fab94b5e784a5b
manifest  tools/queue-pipe-mappings-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.68 Duration-quantum register pointers

The four initialized words at `0x040010d4..0x040010e4` are now exposed through
a bounded duration-quantum pointer API. Retained startup fills them with the
four MAC pipe duration-register addresses; TX publication reads the table when
programming per-pipe timing.

The remaining production base literals in MAC initialization and TX were
replaced with the typed root while preserving the existing `pipe * 4`
arithmetic and volatile writes. `tools/check-duration-quantum-pointers-layout.py`
pins two linked base literals, their decoded xrefs, and exact-parent codegen.
The complete ELF remains byte-identical to the qualified queue/pipe-mappings
parent, so no hardware rerun is required.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-duration-quantum-final-check.log
          4117aaa7bf12e5ba084cb4df6da199dba7422adb70371f52fbcc38b1945c48bf
manifest  tools/duration-quantum-pointers-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.69 Initialized TX rate tables

The initialized duration table at `0x04000138..0x0400014c` is now exposed as
ten bounded `u16` entries. The two 22-byte tables at
`0x04000194..0x040001c0` are now exact byte arrays for rate encoding and rate
attributes instead of opaque storage.

MAC startup and all current Rust TX consumers now derive these addresses from
the `InitializedVendorImage` layout. The retained wrapping rate-index additions,
volatile widths, and initialization order are unchanged. The linked image folds
the attribute-table access into arithmetic from the encoding-table root, so the
drift manifest intentionally contains linked roots only at `0x04000138` and
`0x04000194`. `tools/check-initialized-tx-rate-tables-layout.py` pins that exact
literal and decoded-xref multiset. The complete ELF and packed image remain
byte-identical to the qualified duration-quantum-pointers parent, so no hardware
rerun is required.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-initialized-tx-rate-final-check.log
          2c1c34b35ccfabafbb4e49424b984d040176d2b2738a3ef97953cabe9ab5af0a
manifest  tools/initialized-tx-rate-tables-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.70 Initialized completion words

The ten initialized words at `0x04000260..0x04000288` now have a bounded typed
address API. They remain raw quarantined words rather than function pointers:
the qualified image and decompilation prove the visible extent, but not that
every entry is callable.

The translated TX completion-class lookup now derives its base from the
`InitializedVendorImage` layout while retaining the unchecked class-driven
`* 4` arithmetic and volatile load. The current linked scanner resolves no
standalone literal in this interval, so
`tools/check-initialized-completion-words-layout.py` intentionally pins an empty
linked-literal/xref set in addition to the source ownership gate and complete
exact-parent codegen manifest. The complete ELF and packed image remain
byte-identical to the qualified initialized-TX-rate-tables parent, so no
hardware rerun is required.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-completion-words-final-check.log
          6742a2e0bc7999d60e67abccc04a7470128f62fecdbf11db723dfcbef55762fb
manifest  tools/initialized-completion-words-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.71 Initialized IRQ callback words

The 32 initialized callback words at `0x040011bc..0x0400123c` now have a bounded
typed address API. Values remain quarantined raw words because callback targets
include retained code and are not Rust-owned function pointers.

The platform callback-table root now derives from the `InitializedVendorImage`
layout. Existing index arithmetic, initialization writes, and interrupt
registration ordering are unchanged. The linked image materializes only the
used interior addresses `0x040011d0` and `0x040011e4`;
`tools/check-initialized-irq-callbacks-layout.py` pins those literals and their
complete decoded xrefs. The complete ELF and packed image remain byte-identical
to the qualified initialized-completion-words parent, so no hardware rerun is
required.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-irq-callback-final-check.log
          360a9ecd8a9b02cd2e62c67d790291250bea206ada49aa94e7969008b83a9f25
manifest  tools/initialized-irq-callbacks-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.72 Host PAS scheduling ring

The retained host PAS ring at `0x04001578..0x04001680` is now an exact
`HostPasRing`: 32-bit head and tail cursors followed by 64 pointer words. The
logical cursors remain masked to six bits by consumers; slot arithmetic remains
unchecked after that equivalent validation.

MAC startup, the translated class-0 scheduler, cancellation rollback,
diagnostics, and host-driver ring maintenance now derive the root from the
`InitializedVendorImage` layout. All volatile widths, compaction order, hole
clearing, head/tail publication, and wrapping cursor arithmetic are unchanged.
`tools/check-host-pas-ring-layout.py` pins the single linked root and its complete
decoded xref multiset. The complete ELF and packed image remain byte-identical
to the qualified initialized-IRQ-callbacks parent, so no hardware rerun is
required.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-host-pas-ring-final-check.log
          c27e45999de02f6976ccc72c733ce233178728d370b9d4ecf952e2939bb1f0cb
manifest  tools/host-pas-ring-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.73 MAC pipe records

The four retained MAC pipe records at `0x04001720..0x040018d0` are now exact
`0x6c`-byte layouts. Each record contains its current-slot byte, state byte,
hardware-ring pointer, and four `0x18`-byte slot records. Each slot names the
frame, auxiliary, and command words while keeping its mixed-width state prefix
and unresolved bytes quarantined.

Current Rust TX and host-scheduler consumers now derive direct pipe-record roots
from the initialized-image layout. Existing `pipe * 0x6c`, `slot * 0x18`,
volatile byte/word operations, and publication ordering remain unchanged.
`tools/check-mac-pipe-records-layout.py` pins the linked root plus the two
materialized interior field addresses and their decoded xrefs. The complete ELF
and packed image remain byte-identical to the qualified host-PAS-ring parent, so
no hardware rerun is required.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-mac-pipe-record-final-check.log
          1a4e8f1935881b39d03f7a9e1a4485ffee20ee421250119d3b46b27ba8719575
manifest  tools/mac-pipe-records-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.74 Initialized low-MAC global prefix

The `0xa0`-byte low-MAC prefix at `0x04001680..0x04001720` is now an exact
`LowMacGlobalPrefix`. It names the FIFO controls, rate configuration, runtime
flags, producer words, slot-time values, IFS duration, and two 22-entry airtime
tables. Only the three bytes at `+0x0d` and the word at `+0x40` remain
semantically opaque.

MAC, radio, PHY, and TX roots now derive from this physical layout, including
the direct FIFO-status and legacy-mode bytes and short-airtime table. Existing
relative arithmetic and mixed-width volatile operations remain unchanged. The
linked image materializes eleven interior addresses rather than the base;
`tools/check-low-mac-global-prefix-layout.py` pins all 33 resolved literals and
59 decoded xrefs. The complete ELF and packed image remain byte-identical to the
qualified MAC-pipe-records parent, so no hardware rerun is required.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-low-mac-global-final-check.log
          6cd44eb8e30a1c32a2ba816b64dda9c08620cbe5cc6aac406afdac726da53bb3
manifest  tools/low-mac-global-prefix-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.75 MAC beacon control state

The `0x40` bytes at `0x04001a80..0x04001ac0` are now an exact
`MacBeaconState`. It names two response-command words, the event/control state,
secondary command, control and selector words, mode byte, and completion word.
Unresolved prefix/interior bytes remain explicit opaque quarantine.

MAC startup, station-mode programming, response-descriptor installation, TX
status handling, beacon-event handling, and completion publication now derive
these addresses from the physical layout. Existing volatile widths, event
ordering, and raw state arithmetic are unchanged.
`tools/check-mac-beacon-state-layout.py` pins four linked interior addresses and
their complete decoded xref multiset. The complete ELF and packed image remain
byte-identical to the qualified low-MAC-global-prefix parent, so no hardware
rerun is required.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-mac-beacon-state-final-check.log
          0721c7be2f2c085f2f8025ec2b6e7cf9ea81034883b9c6c39955fc1788232cff
manifest  tools/mac-beacon-state-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.76 MAC wake runtime state

The `0x48` bytes at `0x04001ac0..0x04001b08` are now an exact
`MacWakeRuntimeState`. It contains the retained timer at `+0x08`, PHY and wake
transition bytes, saved MAC mode, wake control, 22-byte retry-rate map, and EDCA
slot-timing cache. Four leading/interior words and bytes remain explicit opaque
quarantine where semantics are not proven.

MAC wake/reconfiguration, PHY transitions, TX duration/retry selection,
power-save event handling, EDCA publication, VIF configuration, and the startup
diagnostic now derive their addresses from this physical layout. Timer setup,
volatile widths, wrapping rate indexes, and wake/event ordering remain unchanged.
`tools/check-mac-wake-runtime-layout.py` pins seven linked interior addresses and
the complete decoded xref multiset. The complete ELF and packed image remain
byte-identical to the qualified MAC-beacon-state parent, so no hardware rerun is
required.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-mac-wake-runtime-final-check.log
          08b9edf0fe593e153d6419c558f3ae86169e42dcb395c6ea40b147a5056a13d7
manifest  tools/mac-wake-runtime-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.77 MAC PHY command state

The `0x4c` bytes at `0x04001d10..0x04001d5c` are now an exact
`MacPhyCommandState`. It covers the radio-stop byte, FIQ sideband capture,
retained operation timer, operation state/command/output/timeout, dispatch
command and output blocks, completion status, and interface byte. Unresolved
interior words and bytes remain explicit opaque quarantine.

MAC startup/radio-stop, PHY channel completion, command-1/2/3/7 dispatch,
operation timer management, pipe event handling, and sideband capture now derive
their addresses from the physical layout. The overlapping `0x04001d20` logical
operation root remains explicit while the backing record is modeled once.
Volatile widths, callback initialization, timer ordering, and command-state
transitions remain unchanged. `tools/check-mac-phy-command-state-layout.py` pins
five linked interior addresses and their complete decoded xrefs. The complete
ELF and packed image remain byte-identical to the qualified MAC-wake-runtime
parent, so no hardware rerun is required.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-mac-phy-command-final-check.log
          e83cbbeacefb6f55b698888a6ea56aca4e354c6e18acbcf413b836087b2b4b66
manifest  tools/mac-phy-command-state-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.78 MAC runtime accounting state

The final initialized-image bytes at `0x04001f78..0x04001fcc` are now an exact
`MacRuntimeAccountingState`. It names the current-pipe word and overlapping
sample flag, status/sample counters, current record and slot, four pipe-event
bytes, the packet software-record free list, rolling average, silicon-control
byte, and two accounting parameters.

The free list is modeled physically as one head word followed by four
`next`/packet-record node pairs, matching the retained overlapping initialization
loop. Platform DMA setup, TX completion/recycling, pipe events, TALA accounting,
and PHY silicon-mode setup now derive these addresses from the initialized-image
layout. Existing list publication, volatile widths, wrapping counters, and
unchecked arithmetic remain unchanged. `tools/check-mac-runtime-accounting-layout.py`
pins four linked addresses and the complete decoded xref multiset. The complete
ELF and packed image remain byte-identical to the qualified MAC-PHY-command-state
parent, so no hardware rerun is required.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-mac-runtime-accounting-final-check.log
          db227a43c82a125d4bf25d100afbba58e612927af4807fa1aac062d8b815cc05
manifest  tools/mac-runtime-accounting-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.79 MAC retry hardware state

The retained word at `0x04001e6c..0x04001e70` is now an exact
`MacRetryHardwareState`. The low byte is the scheduler/retry hardware gate; the
remaining three bytes stay opaque because only the startup word clear proves
their physical occupation.

MAC startup, host scheduling diagnostics and admission, event draining, and the
retry-control update now derive the address from the initialized-image layout.
The startup 32-bit clear and runtime byte accesses remain deliberately distinct,
preserving the retained mixed-width contract. `tools/check-mac-retry-hardware-state-layout.py`
pins the linked root and its complete decoded xrefs. The complete ELF and packed
image remain byte-identical to the qualified MAC-runtime-accounting parent, so
no hardware rerun is required.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-mac-retry-hardware-final-check.log
          6e5d37ed228726b1d51aaff4c06b7c95447079e76398914f96dff9b6b4d15249
manifest  tools/mac-retry-hardware-state-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.80 MAC TX queue state

The two retained queue pointers at `0x040018d0..0x040018d8` are now an exact
`MacTxQueueState` head/tail pair. Decompiled `txq_remove_frame_by_link_seq`
updates the same words when unlinking a matching frame; Rust startup clears both
before retained queue consumers can run.

The startup writes now derive from the initialized-image layout while preserving
the original two 32-bit stores and their order. The linked image folds them to a
single root literal; `tools/check-mac-tx-queue-state-layout.py` pins that root and
its decoded xref. The complete ELF and packed image remain byte-identical to the
qualified MAC-retry-hardware-state parent, so no hardware rerun is required.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-mac-tx-queue-final-check.log
          105cce2d9b88ad64853dc0da94a44924f4726c97f9715f767914c34850073d03
manifest  tools/mac-tx-queue-state-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.81 Initialized retained rate policies

The two five-word policy records at `0x04000200..0x04000228` are now an exact
`InitializedRatePolicies` layout. Startup copies these immutable initialized
words into the first two retained PAS policy records before queue scheduling is
allowed.

Both copy loops now derive their source from the initialized-image layout while
preserving the original five-word bounds, destination order, and volatile
32-bit reads/writes. The compiler folds both policy roots into one linked base;
`tools/check-initialized-rate-policies-layout.py` pins that root and both decoded
uses. The complete ELF and packed image remain byte-identical to the qualified
MAC-TX-queue-state parent, so no hardware rerun is required.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-initialized-rate-policy-final-check.log
          7f4ec38f25e327bfd1ca10712c96e1f4f51acd271496e98bf600e003d3dc90ee
manifest  tools/initialized-rate-policies-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.82 PHY threshold descriptors and gain records

The two `0x08`-byte channel-threshold descriptors at
`0x0400145c..0x0400146c` and sixteen `0x10`-byte TX gain records at
`0x0400146c..0x0400156c` are now exact initialized-image layouts. Descriptor
fields name count, default threshold, and record pointer. Gain records name rate,
requested offset, selected power, cleared/reserved fields, gain code, and RSSI
value.

Threshold lookup and all-slot gain programming now derive their roots from the
physical layout after retaining the same profile/slot validation. Pointer loads,
record strides, volatile widths, hardware publication order, and wrapping gain
arithmetic remain unchanged. The linked image materializes only the gain-loop
interior address `0x0400147a`; `tools/check-phy-descriptor-gain-records-layout.py`
pins that address and decoded xref. The complete ELF and packed image remain
byte-identical to the qualified initialized-rate-policies parent, so no hardware
rerun is required.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-phy-descriptor-gain-final-check.log
          fd18d2db810483c8e85faf51784467341488a37b6f3efc7be3d93c4aecbaf288
manifest  tools/phy-descriptor-gain-records-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.83 A-MPDU completion control

The initialized word at `0x0400140c..0x04001410` is now an exact
`AmpduCompletionControl`. Retained completion code and the translated drain path
use its nonzero value to decide whether to count pending class-zero completions
before draining the shared completion ring.

The translated completion path now derives both this gate and the existing
A-MPDU telemetry root from typed initialized-image layouts. The gate read,
completion-ring walk, counter widths, and telemetry update order remain
unchanged. `tools/check-ampdu-completion-control-layout.py` pins the linked word
and decoded xref; the existing telemetry gate continues to cover
`0x040012a0..0x040012c8`. The complete ELF and packed image remain byte-identical
to the qualified PHY-descriptor-gain-records parent, so no hardware rerun is
required.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-ampdu-completion-control-final-check.log
          49e84ea01b2338d4520d93e383f42513b822c87d0bfd95973ddfb0c300f8f65c
manifest  tools/ampdu-completion-control-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.84 Initialized TKIP S-box tables

The retained initialized interval `0x04000310..0x04000710` is now an exact
`TkipSboxTables` quarantine view containing two contiguous 256-entry tables of
shared `u16` scalars. `low_byte` occupies `0x04000310..0x04000510` and
`high_byte` occupies `0x04000510..0x04000710`; the latter ends exactly at the
existing command-dispatch table. The `0x040002e4..0x04000310` prefix remains
opaque. The complete range lies in the vendor COPY record, and no owned Rust
initializer was introduced.

Retained `tkip_key_mix` performs six `u16` reads from each table using the low
byte (`& 0xff`) and high byte as indexes. `tkip_phase1_mix` performs five `u16`
reads from each table per iteration for exactly eight iterations. The exported
DTCM reference report records the corresponding low-table references at
`0x00001bde..0x00001e16` and high-table references at
`0x00001be0..0x00001e18`. These consumers provide exclusive evidence for
256-entry, two-byte-wide access; no retained runtime writer or Rust
reader/writer was found, and container COPY initialization is the only known
writer. This supports structural read-only naming, not Rust ownership or an
immutability claim: vendor and IRQ/FIQ-visible DTCM remains shared quarantine,
and no safe reference or slice is exposed.

The new API returns only the table root and bounded, layout-derived entry
addresses for indexes `0..256`; it provides no value read or write operation.
No production source literal was migrated, and the unrelated bare MAC MMIO
offset `0x0310` remains untouched. `tools/check-tkip-sbox-layout.py` scans
production sources while masking Rust `cfg(test)` items and pins the reviewed
ELF aligned-literal and decoded-PC-relative-xref counters at zero. Those empty
counters are drift evidence rather than consumer closure because
register-computed retained references are not recovered. The checker runs in
source-only and linked phases of both software build scripts. No production
callsite was created, so this layout-only slice has no codegen manifest.

The focused address test, source-only checker, complete `tools/check.sh` gate,
Thumb build, linked checker (`literals=0 decoded_xrefs=0`), and exact complete
artifact hashes passed. No hardware test was run.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-tkip-sbox-final-check.log
          b9f6eb49286b6be3497e80f3f112e9d3bd2cb235fb9d7cd0387f7098fe855c51
```

### A.85 Initialized AES transfer-class table

The retained vendor-COPY interval `0x04000804..0x04000830` is now an exact
`AesTransferClassTable` shared-quarantine view: 11 volatile-width `u32` entries,
stride four, size `0x2c`, and alignment four. It starts after the unchanged
opaque `0x040007a4..0x04000804` prefix and ends exactly at the existing AES
mode-1 microcode at `0x04000830`; neither adjacent range is absorbed or moved.
No Rust initializer, value accessor, safe reference, slice, writer, or
immutability/ownership claim was introduced.

Retained `hif_start_next_xfer` loads one 32-bit word from
`0x04000804 + (*(u8 *)(descriptor + 4) * 4)`. Its unchecked `u8` indexing is
unchanged and is not constrained by the bounded layout API. Existing evidence
identifies transfer classes 6 and 7 as TX and RX CCMP respectively, while the
observed class 10 use establishes that entry without supporting additional
crypto semantics. No known Rust reader/writer or runtime writer was found.
The DTCM reference report resolves the base only to `hif_start_next_xfer`; this
is drift evidence, not complete consumer closure.

The new private API exposes only the layout-derived table root and bounded
addresses for classes `0..11`. `tools/check-aes-transfer-class-layout.py` owns
the exact half-open range `[0x04000804, 0x04000830)`, masks Rust `cfg(test)`
items during its production-source scan, rejects raw in-range literals and
alternate base/stride forms outside the layout owner, and pins reviewed linked
aligned literals and decoded PC-relative xrefs by symbol at zero. Computed,
indirect, vendor, IRQ, and FIQ consumers remain outside closure. The checker
runs in source-only and linked phases of `tools/check.sh` and
`tools/build-ota-image.sh`. No production callsite exists, so no codegen
manifest was added.

The focused test pins the root; classes 0, 6, 7, and 10; the rejected class 11;
and exact adjacency to AES microcode. Focused host tests, the source checker,
the complete software gate, Thumb release build, linked checker, and packed
image gate passed without complete-artifact drift. No hardware test was run.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-aes-transfer-class-final-check.log
          31306106616d0fcfc13baa01db75b8ccf60438d389de1d334ad3f0488d2dbaab
```

### A.86 Initialized PHY gain register-write lists

The exact initialized interval `0x04000b60..0x04000c10` is now represented by
two adjacent private `RegisterWriteList` shared-quarantine layouts. Each list is
`0x58` bytes, alignment four: ten ordered `RegisterWrite` pairs of two `u32`
shared scalars at offsets `+0` and `+4`, followed by a sentinel address word at
`+0x50` and a semantically opaque physical word at `+0x54`. The default root is
`0x04000b60`, profile 1 begins at `0x04000bb8`, and the two lists end exactly at
`0x04000c10`; no byte in the adjacent register-list family is claimed.

The vendor COPY bytes contain the exact ordered pairs recorded in section 4.10.
`reg_write_list_apply` loads the address as one 32-bit word, checks it against
`0xffffffff`, then loads the following value as one 32-bit word, performs one
32-bit MMIO write, and advances eight bytes in order. It never reads the second
terminator word. `phy_build_gain_tables` chooses the first list by default and
the second only when the PHY profile byte equals one. The retained-reference
report contains only the two decoded roots, at PCs `0x000171c4` and
`0x00017192` respectively. No known runtime writer or Rust reader/writer was
found, but container COPY being the only evidenced initialization writer does
not establish immutability; vendor, IRQ/FIQ, and debug access remains possible.

The API is address-only: it derives the private list root with `offset_of!`,
returns list roots only for profiles 0 and 1, and returns normal pair roots only
for entries 0 through 9. It exposes no safe reference, slice, value read/write,
terminator-value accessor, or runtime iterator, and does not constrain the
retained unchecked vendor loop. No PHY/MMIO production consumer changed, so
volatile widths, wrapping/unchecked arithmetic, MMIO/barrier/interrupt order,
initialization order, request ownership, and all HIF behavior remain unchanged.
No production callsite exists and no codegen manifest was added.

`tools/check-phy-gain-register-write-lists-layout.py` owns the exact half-open
range `[0x04000b60, 0x04000c10)`, masks Rust `cfg(test)` items, rejects raw
in-range literals and alternate base/list-stride/entry-stride forms outside the
layout owner, and pins reviewed aligned linked literals and decoded
PC-relative xrefs to empty counters. Computed/indirect and vendor/IRQ/FIQ/debug
access remains outside closure. Source-only and linked invocations run in both
software build scripts. The focused exact-address test, source-only checker,
complete software gate, Thumb release build, linked checker, complete ELF hash,
and packed-image hash passed without drift. No hardware test was run.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-phy-gain-register-write-lists-final-check.log
          c8ba4944f2f4e38339be9c7505ad828063c68ff84e1470ab0fd820cc09730659
```

### A.87 Initialized PHY initialization register-write lists

The complete initialized interval `0x04000c10..0x04000c60` is represented by
three consecutive private, alignment-four shared-quarantine layouts. List 0 at
`0x04000c10` has one `RegisterWrite`, the address sentinel at `+0x08`, and a
semantically opaque physical `u32` at `+0x0c`, for size `0x10`. List 1 at
`0x04000c20` has four writes, sentinel at `+0x20`, and opaque word at `+0x24`,
for size `0x28`. List 2 at `0x04000c48` has two writes, sentinel at `+0x10`,
and opaque word at `+0x14`, for size `0x18`. Each write retains the exact
`{SharedU32 address, SharedU32 value}` representation and eight-byte stride.
At completion of this slice, the following `0x474` bytes remained opaque and
the duration-quantum-pointer boundary stayed at `0x040010d4`. Appendix A.88
subsequently decodes only `0x04000ca6..0x04000da8` within that interval.

The vendor COPY image establishes these ordered source literals: list 0 has
`0x0ab80108 = 0x00200300`; list 1 has `0x0ab8807c = 0x00000001`,
`0x0ab88058 = 0x000063d9`, `0x0ab8808c = 0x0000103f`, and
`0x0ab88090 = 0x1010103f`; list 2 has `0x0ab90000 = 0x00000000` and
`0x0ab90014 = 0x0fffffff`. Every list is followed by address sentinel
`0xffffffff`; the following physical `u32` has known width but no assigned
semantics. These values are evidence recorded here, not Rust constants or APIs.

`reg_write_list_apply` performs an unchecked eight-byte walk: one 32-bit
address load, the sentinel comparison, one 32-bit value load only for a normal
entry, one ordered 32-bit MMIO write, then pointer advance. It does not read the
word following the sentinel. `phy_apply_reg_init_lists` passes roots
`0x04000c10`, `0x04000c20`, `0x04000c48`, and the excluded unresolved root
`0x04000c60` in order, at retained PCs `0x00017208`, `0x0001720e`,
`0x00017214`, and `0x0001721a`. No other retained direct reference occurs in
the decoded interval. Vendor COPY initializes the bytes, but the unrestricted
HIF memory writer and possible vendor/IRQ/FIQ mutation require shared
quarantine; no immutability or exclusive ownership is claimed.

The Rust API exposes addresses only: the private crate root is derived with
`offset_of!`, list indices 0 through 2 map to the three exact roots, and entry
counts `[1, 4, 2]` retain eight-byte spacing. There are no safe references or
slices, value/sentinel/MMIO accessors, iterators, or validation APIs. No
production PHY/MMIO callsite changed. Consequently volatile widths,
wrapping/unchecked arithmetic, MMIO/barrier/interrupt and initialization order,
request ownership, and all 30 HIF inputs remain unchanged.

`tools/check-phy-init-register-write-lists-layout.py` owns exactly
`[0x04000c10, 0x04000c60)`. It masks Rust `cfg(test)` items, rejects raw
in-range and synthesized boundary literals plus alternate base/root/stride
forms outside the layout owner, and pins aligned linked literals and decoded
PC-relative xrefs by containing symbol. The reviewed linked manifests are both
empty (`literals=0`, `decoded_xrefs=0`). Source-only and linked invocations run
adjacent to the PHY gain-list checker in both build scripts. The exact-parent
complete codegen review is recorded in
`tools/phy-init-register-write-lists-codegen-manifest.json` and reports no text
symbol drift.

Focused host tests pin every root and normal entry, invalid list and entry
indices, list adjacency, final structural end, and the unchanged duration
pointer root. The focused test, source checker, complete software gate, Thumb
release/linked checker, exact-parent codegen gate, and OTA packing passed
without artifact drift. No hardware test was run. At this checkpoint `0x04000c60` onward remained
opaque; Appendix A.88 later refines only the evidence-backed initialized island.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-phy-init-register-write-lists-final-check.log
          c486edf9a0570d86f35c8ac7ad6acd0bc2bc74a62966f30b35dd5f25536da7f1
manifest  tools/phy-init-register-write-lists-codegen-manifest.json
```

### A.88 Initialized overlapping PHY gain-source records

The initialized COPY interval `0x04000ca6..0x04000da8` is represented as exactly
43 private `InitializedPhyGainSourceRecord` values (`0x102` bytes). Each
alignment-two, six-byte record contains `selector: SharedU8` at `+0x00`, an
opaque `SharedU8` at `+0x01`, and `lower: SharedU16` and `upper: SharedU16` at
`+0x02` and `+0x04`. Retained `phy_build_gain_tables` reads the latter two
fields as signed 16-bit values; the Rust quarantine does not expose values.

Two logical 22-record views overlap by one record: view 0 starts at
`0x04000ca6`, view 1 starts at `0x04000d24` (`21 * 6` bytes later), and view 0
record 21 is physically identical to view 1 record 0. Their final physical
record starts at `0x04000da2` and ends at `0x04000da8`. View numbers have no
assigned profile semantics. Vendor COPY is the evidenced initializer, not
proof of immutability; vendor, HIF/debug, IRQ, and FIQ mutation remain possible.

`InitializedVendorImage` preserves its `0x2078` size by splitting the former
`0x474` opaque region into an opaque `0x46`-byte prefix, the typed `0x102`-byte
union, and an opaque `0x32c`-byte suffix. Thus `0x04000c60..0x04000ca6` and
`0x04000da8..0x040010d4` remain opaque, and `duration_quantum_pointers` remains
at `0x040010d4`. The separate root at `0x04000da8` is not absorbed.

The crate-private API is address-only and derives its root with `offset_of!`.
It checks physical index `< 43` and logical `view < 2 && index < 22`, computing
the latter as physical index `view * 21 + index`. It provides no pointer,
reference, slice, iterator, value, write, opaque-byte, or profile accessor.
No production consumer changed.

`tools/check-initialized-phy-gain-source-layout.py` owns exactly the half-open
range `[0x04000ca6, 0x04000da8)`, masks Rust `cfg(test)` items, rejects raw and
synthesized roots/boundaries and alternate base/root/view-stride/record-stride
forms outside its two trusted owner files, and pins aligned linked literals and
decoded PC-relative xrefs by containing symbol. Both reviewed linked manifests
are empty. Source-only and linked checks are integrated into `check.sh` and
`build-ota-image.sh`. Focused tests pin physical records 0, 21, and 42, both
view endpoints and their shared record, all rejection boundaries, type layout,
image size, and the unchanged duration-pointer address. Exact-parent codegen is
recorded in `tools/initialized-phy-gain-source-codegen-manifest.json`; it
requires no text-symbol, memory-order, IRQ/barrier, stack, or symbol-set drift.
No hardware test was run.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-initialized-phy-gain-source-final-check.log
          a9832ae063265d6882ffcf2885d0863e20051ce791b3c509085571058289028b
manifest  tools/initialized-phy-gain-source-codegen-manifest.json
```

### A.89 Initialized IQ-calibration gain-index table

The vendor COPY image initializes exactly twelve contiguous aligned `u32` words
at `0x04000e18..0x04000e48`, represented by the private alignment-four
`InitializedIqCalibrationGainIndices { entries: [SharedU32; 12] }`. Its size is
`0x30`, `entries` starts at `+0x00`, and entry offsets are `+0x00`, `+0x04`,
`+0x08`, `+0x0c`, `+0x10`, `+0x14`, `+0x18`, `+0x1c`, `+0x20`, `+0x24`, `+0x28`,
and `+0x2c`. COPY values in order are `0x1a, 0x19, 0x18, 0x16, 0x15, 0x14,
0x12, 0x11, 0x10, 0x02, 0x01, 0x00`.

Retained `rf_compute_iq_gain_corr` indexes this root with four-byte stride for
exactly twelve iterations. `rf_write_iq_corr_regs` reads its `u32` entries but
performs an unchecked post-body lookahead at `0x04000e48`; that address is
excluded from the table and remains opaque because it is also the distinct
register-list root passed by `phy_cal_apply_substate`. `rf_calibrate_iq_dc`
retains the root as reader/orchestrator. Vendor COPY is the only exact known
writer, not evidence of immutability: generic HIF/debug memory mutation and
vendor/IRQ/FIQ mutation remain possible. The duplicate values in
`phy.rs::IQ_CALIBRATION_GAIN_INDICES` are evidence only; production users remain
unchanged and do not read this DTCM view.

`InitializedVendorImage` splits only the prior `0x32c` suffix as `0x70 + 0x30 +
0x28c`: opaque `0x04000da8..0x04000e18`, the table, then opaque
`0x04000e48..0x040010d4`. `duration_quantum_pointers` remains at
`0x040010d4`, and the image remains `0x2078` bytes. This is a shared quarantine
view with no immutability or ownership closure. The crate-private API derives
the root with `offset_of!` and returns only a checked `DtcmAddress` for indices
`0..12`; it exposes no pointer, reference, slice, iterator, value read/write,
or entry-12/lookahead accessor.

`tools/check-initialized-iq-calibration-gain-indices-layout.py` owns exactly
`[0x04000e18, 0x04000e48)`. It masks Rust `cfg(test)`, permits only `dtcm.rs`
and itself as source owners, rejects raw in-range or synthesized roots and
alternate named base/root/count/stride forms elsewhere, and pins aligned linked
literals plus decoded PC-relative xrefs by containing symbol. The reviewed
linked counters are empty because no production consumer was added. This is
drift evidence, not closure over computed, indirect, vendor, IRQ/FIQ, HIF, or
debug accesses.

Compile-time assertions and focused tests pin type size/alignment, entries
field offset, the exact root, entries 0, 1, and 11, rejection of index 12, table
end and opaque boundaries, the unchanged duration-pointer root, and the image
size. Source/linked checks run in `check.sh` and `build-ota-image.sh`; the exact
parent manifest requires no text-symbol, operation-order, IRQ/barrier, stack,
or symbol-set drift. No hardware test was run.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-initialized-iq-calibration-gain-indices-check.log
          b53a8f0151306a62218ea989c2f045d154495129e3110777956c7e0105a7c744
manifest  tools/initialized-iq-calibration-gain-indices-codegen-manifest.json
```

### A.90 Fixed measurement workspace

The measurement workspace is a fixed shared-quarantine view at
`0x040010f8..0x04001160`, not relocated or exclusively Rust-owned.
`measure_setup_by_type` forms its root as `0x0400110c - 0x14` and calls
`fw_memzero(root, 0x68)`, proving the half-open physical extent. It then writes
the request's measurement type as one byte at `+0x05` and completion status as
one 32-bit word at `+0x08`, in that order after the complete zero. This preserves
the observed zero-before-field-write initialization order without adding a Rust
initializer or production consumer.

| Offset | Width/shape | Decoded field |
| ---: | --- | --- |
| `+0x00` | volatile/shared `u16` | dwell bound |
| `+0x02` | `0x03` opaque bytes | unknown |
| `+0x05` | volatile/shared `u8` | measurement type |
| `+0x06` | `0x02` opaque bytes | unknown |
| `+0x08` | volatile/shared `u32` | completion status |
| `+0x0c` | `0x05` opaque bytes | unknown |
| `+0x11` | volatile/shared `u8` | dispatch state |
| `+0x12` | volatile/shared `u16` | dispatch argument |
| `+0x14` | `0x04` opaque bytes | unknown |
| `+0x18` | two volatile/shared `u32` words | start timestamp; deliberately not `u64` |
| `+0x20` | two volatile/shared `u32` words | elapsed timestamp; deliberately not `u64` |
| `+0x28` | one opaque byte | nested scan-request root/prefix |
| `+0x29` | volatile/shared `u8` | nested scan-request mode |
| `+0x2a` | `0x3e` opaque bytes | undecoded nested request tail |

The enclosing former `0xc8`-byte opaque field was first split as `0x14 + 0x68
+ 0x4c`; the adjacent fixed slice now refines the suffix as one typed `0x04` word
at `0x04001160..0x04001164` followed by opaque `0x04001164..0x040011ac`.
The independently observed `0x04001160` reference in
`txq_build_aggregate_lists` supports the workspace's exclusive endpoint; the
following initialized HIF/control field remains at `0x040011ac`.
`InitializedVendorImage`, `DtcmLayout`, and `SharedDtcmState` retain their exact
sizes.

Retained direct consumers include `measure_setup_by_type`,
`measure_state_dispatch_args`, `measure_arm_dwell_timer`, task completion, and
`measure_emit_complete`. Vendor COPY initialization of the containing DTCM image,
generic HIF bulk memory reads/writes, debug memory access, and untranslated
vendor/IRQ/FIQ mutation remain consumers or mutation avenues even where they do
not appear as direct literals. The nested scan-request tail therefore remains
opaque. This view grants no immutability, exclusive ownership, safe reference,
or request-ownership claim; MMIO/barrier/interrupt order, arithmetic behavior,
initialization routines, and all 30 HIF inputs remain unchanged.

The crate-private inventory is address-only: the workspace root; dwell, type,
completion, dispatch-state, and dispatch-argument addresses; checked start and
elapsed timestamp-word addresses for indices zero and one; the nested request
root; and its mode byte. It returns only `DtcmAddress`/`Option<DtcmAddress>` and
exposes no reference, pointer, value, slice, iterator, writer, generic offset,
or opaque-tail access.

`tools/check-measurement-workspace-layout.py` gates full 32-bit source literals
in `[0x040010f8, 0x04001160)` outside `dtcm.rs` and itself, verifies the reviewed
struct/accessor/layout inventory, and pins aligned linked literals and decoded
PC-relative xrefs. It intentionally does not treat short values such as
`0x1100` as physical addresses. Its empty linked manifests and source coverage
are drift evidence only, not proof against register-computed, indirect, generic
HIF/debug, vendor, IRQ, or FIQ accesses and not ownership proof. The checker is
integrated in both `check.sh` and `build-ota-image.sh`; focused host tests pin all
reviewed addresses and reject timestamp index two. The exact-parent codegen
manifest requires no text-symbol, operation-order, IRQ/barrier, stack, or symbol
set drift. No hardware test was run.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
manifest  tools/measurement-workspace-codegen-manifest.json
```

### A.91 Fixed TX aggregate expiration delta

The initialized vendor COPY island at `0x04001160..0x04001164` is decoded as
exactly one `#[repr(C, align(4))]` `TxAggregateExpirationDelta` containing one
shared-quarantine `SharedU32` at offset zero. The decompilation retains the sole
observed read in `txq_build_aggregate_lists` as
`(piVar13[-5] - iVar4) + DAT_0000a6e8 < 0`; this establishes one 32-bit operand
and its structural role. `/tmp/xr819-dtcm-refs.out:203` independently records
`04001160,0000a404,txq_build_aggregate_lists,READ`. There is no known Rust
consumer or writer, and no production code was added or translated.

The address-only crate-private inventory consists solely of
`TX_AGGREGATE_EXPIRATION_DELTA`, derived with `offset_of!` from the fixed
initialized image. It exposes no pointer, reference, value, read/write, slice,
iterator, count/stride, generic-offset, or interior-field API. `SharedU32`
already supplies the `UnsafeCell<MaybeUninit<u32>>` quarantine representation;
this does not establish immutable storage, exclusive Rust ownership, a larger
TX record, a second element, or any extent beyond the one observed word.
Vendor COPY initialization and register-computed, indirect, generic HIF/debug,
vendor, IRQ, and FIQ mutation remain within the containment model, so no safe
reference is created.

The next independently referenced root is `0x04001164`. This slice stopped at
that boundary; Appendix A.92 now decodes its exact six-record extent through
`0x040011ac`. Initialized HIF control remains fixed at `0x040011ac`, and the
containing `InitializedVendorImage` remains size `0x2078` and alignment four.
No bytes beyond this slice, TALA relocation, or `0x04002984..0x04003050` were
changed here.

The retained subtraction and addition remain the vendor's unchecked/wrapping
32-bit operations followed by the signed-negative comparison. MMIO, barrier,
IRQ/FIQ, initialization, and request-ownership order and all 30 HIF inputs are
unchanged because this is structural source-only decoding with no production
consumer.

`tools/check-tx-aggregate-expiration-delta-layout.py` covers exactly the
half-open range `[0x04001160, 0x04001164)`. It gates full physical-address source
literals outside `src/dtcm.rs` and itself, requires the exact struct, enclosing
split, sole address constant, assertions, and global sizes, forbids broad or
safe APIs, and pins empty aligned linked-literal and decoded PC-relative-xref
multisets. This is drift evidence only; register-computed, indirect, generic
HIF/debug, vendor, IRQ, and FIQ accesses remain outside its proof. Focused host
tests pin the address, type shape and end, the following descriptor-table
boundary, and complete initialized-image size. The checker runs in source and linked phases of both
build gates, and the exact-parent manifest permits no codegen drift. No hardware
test was run.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
manifest  tools/tx-aggregate-expiration-delta-codegen-manifest.json
```

### A.92 Fixed initialized debug-command descriptor table

The vendor COPY initializes `0x04001164..0x040011ac`. The resolved parameter at
PC `0x00015bbe` binds root `0x04001164` to the second argument of
`dbg_console_register_cmds(..., DAT_00015d60, 6)`, proving the fixed root and
hard count six. Registration reads separate aligned 32-bit words at record
`+0x00`, `+0x08`, and `+0x04`, advances by three `int` elements, and may stop
at the first null field. Execution identifies `+0x00` as the command-name word,
prints `+0x04` as help/display text, and indirectly dispatches through `+0x08`.
This proves three shared `u32` words at offsets 0, 4, and 8 with stride `0x0c`;
it does not prove pointee extents, valid strings, Thumb targets, or a callable
Rust type. Six records occupy exactly `0x48` bytes and meet the independent HIF
root at `0x040011ac`, leaving no unexplained bytes in the selected interval.

`DebugCommandDescriptor` and `InitializedDebugCommandDescriptors` preserve that
exact representation and remain shared quarantine views. Early null termination
does not transfer ownership or make later records immutable. Vendor COPY,
generic HIF/debug writes, retained vendor code, and IRQ/FIQ mutation remain
possible. The bounded crate-private API returns only `DtcmAddress` or
`Option<DtcmAddress>` for the table, six record roots, and the three fields; it
creates no references and exposes no pointers, values, readers/writers, slices,
iterators, generic offsets, callback conversions, validation, copying, or
invocation. No production consumer changed, so `dbg_console_init` still orders
task registration, descriptor registration, announcement, and timer
initialization exactly as before. Volatile widths, MMIO/barrier/IRQ/FIQ order,
wrapping arithmetic, request ownership, and all 30 HIF inputs are untouched.

`tools/check-initialized-debug-command-descriptors-layout.py` covers exactly
half-open range `[0x04001164, 0x040011ac)`. It requires the exact structs,
fields, count, offsets, enclosing split, bounded address-only API, bounds tests,
assertions, and global sizes; rejects unchecked/generic-offset, pointer,
reference, value, read/write, callable, slice, and iterator APIs; gates full
physical-address source literals outside `src/dtcm.rs` and itself; and pins the
empty aligned linked-literal and decoded PC-relative-xref multisets. Its
adversarial self-test verifies that a renamed generic function deriving a raw
pointer from `DTCM_STATE_BASE + slot * 4` is rejected while the bounded
address-only descriptor API remains accepted. This is drift evidence, not writer
closure or ownership proof. The checker runs in the
source and linked phases of `check.sh` and `build-ota-image.sh`, and the
exact-parent manifest permits no text-symbol drift. No TALA relocation or bytes
in `0x04002984..0x04003050` changed, and no hardware test was run.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-debug-command-descriptors-final-check.log
          b77f1eb1479983d4267398d506386618d09bd9626e330eca9029a5707cad46eb
manifest  tools/initialized-debug-command-descriptors-codegen-manifest.json
```


### A.93 Fixed initialized PHY watchdog counter

The initialized COPY interval `0x0400123c..0x04001240` is now represented by
exactly one `#[repr(C, align(4))] PhyWatchdogCounter { count: SharedU32 }`.
`/tmp/xr819-dtcm-refs.out` records exactly three direct references, all in
`phy_watchdog_check`: a 32-bit write at PC `0x00017366`, a 32-bit read at
`0x0001736a`, and a 32-bit write at `0x00017370`. The decompilation types the
literal root as `uint *`, reads the old value, writes the unchecked/wrapping
`old + 1`, compares that incremented value with 3, and eventually resets the
same word to zero. This supports only a shared 32-bit watchdog count.

The decoded IRQ callback table ends exactly at `0x0400123c`. The new counter
ends at `0x04001240`; `0x04001240..0x040012a0` remains an opaque `0x60`-byte
tail, and the independently decoded A-MPDU telemetry table still begins at
`0x040012a0`. No timer or adjacent byte was decoded. The counter remains in the
initialized vendor COPY placement and is only a shared quarantine view: vendor,
IRQ, FIQ, generic-memory, and HIF/debug mutation remain possible. This is not
writer closure or exclusive ownership.

The sole API is the crate-private address constant `PHY_WATCHDOG_COUNTER`,
derived from the two structural offsets. It exposes no pointer, reference,
value, reader/writer, generic offset, unchecked accessor, slice, or iterator,
and it has no production consumer. Therefore volatile operations, MMIO,
barrier and interrupt order, initialization order, wrapping arithmetic, request
ownership, and all 30 HIF inputs remain unchanged.

`tools/check-initialized-phy-watchdog-counter-layout.py` covers exactly the
half-open range `[0x0400123c, 0x04001240)`. It requires the exact struct,
enclosing split, sole address constant, compile-time assertions, focused test,
and global sizes; rejects unsafe API expansion; source-gates physical literals;
and pins empty aligned linked-literal and decoded PC-relative-xref multisets.
Those empty sets are drift evidence, not writer closure. The checker runs in
both source and linked-ELF phases of `check.sh` and `build-ota-image.sh`.
`tools/initialized-phy-watchdog-counter-codegen-manifest.json` provides the
exact-parent text-symbol gate through
`XR819_INITIALIZED_PHY_WATCHDOG_COUNTER_PARENT_ELF`; symbol checks remain
supplemental to complete-file identity.

The focused host test
`initialized_phy_watchdog_counter_address_is_exact` pins the type, count offset,
IRQ boundary, counter start/end, opaque-tail start/size/end, A-MPDU boundary,
and complete `InitializedVendorImage`, `DtcmLayout`, and `SharedDtcmState`
sizes and alignments. Software-only checks and the Thumb build were run; no
hardware test was run. No TALA relocation or bytes in
`0x04002984..0x04003050` changed.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-phy-watchdog-final-check.log
          3394026cdee1042e65d55ba66f478e534bce447ab2f91bc11b861cc76dcbac18
manifest  tools/initialized-phy-watchdog-counter-codegen-manifest.json
          5fd2fb1a12cd6444b610c7b253549b39776ddb59299682e5058cc17601cc4bf6
```

### A.94 Fixed initialized multi-VIF beacon timer

The fixed initialized interval `0x04001250..0x04001264` is now represented by
the already-qualified five-word `TimerEntry` inside the exact non-derived
`InitializedMultiVifBeaconTimerTail`. `/tmp/xr819-dtcm-refs.out` gives four
direct roots at `0x04001250`: initialization in `fw_timers_and_tasks_init`,
cancel and start in `rx_mgmt_frame_handler`, and start in
`beacon_arm_multi_vif_timer`. `timer_entry_init` writes 32-bit callback `+0x0c`,
then 32-bit context `+0x10`, then 32-bit previous-link `+0x04`; it does not
initialize next or deadline. Generic cancel/start retain raw 32-bit intrusive
links, IRQ/FIQ exclusion, wrapping/unchecked deadline arithmetic, and MMIO
publication order. No operation or production consumer was added.

The enclosing tail starts at `0x04001240`: opaque prefix `+0x00..+0x10`, timer
`+0x10..+0x24`, and opaque suffix `+0x24..+0x60`, meeting the independently
decoded A-MPDU counters at `0x040012a0`. Thus `0x04001240..0x04001250` and
`0x04001264..0x040012a0`, including the asynchronously written byte at
`0x0400124f`, remain opaque. The timer and its raw callback/context/link words
remain shared vendor/IRQ/FIQ quarantine state. No safe reference, pointer,
reader/writer, callback conversion, validation, initialization, arithmetic, or
pointee type is exposed; the sole API is the crate-private address-only
`MULTI_VIF_BEACON_TIMER`, derived exclusively from structural offsets.

`tools/check-initialized-multi-vif-beacon-timer-layout.py` covers exactly the
half-open range `[0x04001250, 0x04001264)`. It requires the exact type and field
inventory, enclosing split, sole constant, compile-time assertions, focused
test, and global sizes; rejects derives and pointer/reference/value/read/write,
unchecked, generic-offset, slice, iterator, and function APIs; source-gates
physical literals; and pins empty aligned linked-literal and decoded
PC-relative-xref multisets. Alias closure covers direct and transitive constants
plus renamed plain and grouped Rust `use` imports across production files;
adversarial self-tests pin imported and transitive generic-accessor rejection.
Empty linked sets are drift evidence, not writer closure. The checker runs in
source and linked phases of both build scripts.
The exact-parent manifest is gated by
`XR819_INITIALIZED_MULTI_VIF_BEACON_TIMER_PARENT_ELF`; symbol/codegen checks are
supplemental to complete-file identity.

The focused test pins all five `TimerEntry` offsets and its `0x14/4` layout,
tail offsets and `0x60/4` layout, physical boundaries `0x04001240`,
`0x04001250`, `0x04001264`, and `0x040012a0`, and all three enclosing global
sizes and alignments. Default and diagnostic process-local host tests, all
software-only gates, the Thumb release build, linked checker, exact-parent gate,
and OTA packing pass. Request ownership, all 30 HIF inputs, volatile widths,
MMIO/barrier/interrupt order, initialization order, and wrapping/unchecked
arithmetic remain unchanged because no production operation was added. No
hardware test was run. No TALA relocation or bytes in
`0x04002984..0x04003050` changed.

```text
ELF       cec5f4beeb89d23467bb84e2cec9ba77922c0fb80fe01ab802054b3e46d1640b
packed    711c7b9873bdd711d0f3f368e7129f27694622cf0ba5b627a800f7d17147a492
checks    /tmp/xr819-multi-vif-beacon-timer-final-check.log
          1f70acf5adc03cf41ce9156650380565a5e27c97698c9da5e97a201d04df2db1
manifest  tools/initialized-multi-vif-beacon-timer-codegen-manifest.json
```
