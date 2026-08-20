# XR819 DTCM native-layout migration plan

**Status:** read-only reverse-engineering plan; no firmware or linker change is proposed by this document.  
**Firmware lineage studied:** `ad989e0a5b1a` (`Let the linker pack packet RAM objects`), the parent of the empty working-copy change.  
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

The likely final form is **several independently typed native Rust family objects**, mostly in ITCM, followed by one atomic removal of all legacy pointer roots and DTCM loader assumptions. A single packed `DtcmLayout` spanning `0x04000000..0x0400a000` is not sound today because it would claim unknown and overlaid bytes and would falsely imply one ownership domain.

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

The current linker partition in `xr819-firmware/link-main-low.x` is:

```text
0x04000000..0x04009080    DTCM_LEGACY_LOW
0x04009080..0x040094d4    DTCM_CONTEXT_POOL
0x040094d4..0x0400a000    DTCM_LEGACY_HIGH
0x0400a000..0x0400c000    DTCM_STACKS
```

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
.dtcm.context_pool    0x04009080 size 0x454, SHT_NOBITS, no PT_LOAD
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
| `0x04002078..0x0400218c` | `0x114` | BSS-like register-context/backoff/debug state | Medium; selected fields translated, remainder mixed |
| `0x0400218c..0x040021b4` | `0x28` | `ClockParameters` record | High shape from `register_structs!`; shared/timer-owned |
| `0x040021b4..0x04002234` | `0x80` | 32-entry scheduler handler table | High, `32 * 4`; contains code pointers |
| `0x04002234..0x040034b0` | `0x127c` | PHY/template/beacon/filter tables and opaque BSS | Medium islands, unknown aggregate extent |
| `0x040034b0..0x040035e0` | `0x130` | SDD-derived channel/gain/profile tables | Medium-high fields; layout populated by `configuration.rs` |
| `0x040035e0..0x04003670` | `0x90` | wake/context state and unknown | Low-medium |
| `0x04003670..0x04003674` | `0x4` | retained duration-source halfwords | High addresses; semantics incomplete; fixed quarantine |
| `0x04003674..0x04003678` | `0x4` | unknown | Unknown, not allocatable |
| `0x04003678..0x04003e78` | `0x800` | shared low-MAC/PAS/rate/link/queue state with many computed overlays | High family root, incomplete fields; all-or-nothing group |
| `0x04003e78..0x04003e98` | `0x20` | pre-VIF/link/aggregation header | Medium; includes link bitmap at `+0x18` |
| `0x04003e98..0x040049a8` | `0xb10` | three VIF records, stride `0x3b0` | High stride/count; internal records remain mixed |
| `0x040049a8..0x04005a24` | `0x107c` | unknown/possibly VIF-adjacent tables and gaps | Unknown, not allocatable |
| `0x04005a24..0x04008544` | `0x2b20` | 30 host WSM TX contexts, stride `0x170` | High exact range; mixed Rust/vendor mutation |
| `0x04008544..0x04008594` | `0x50` | unknown | Unknown, not allocatable |
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
| `0x04008f6c..0x04008f80` | `0x14` | context/completion accounting root and counters | High anchors, mixed ownership; includes fixed writer at `0x04008f71` |
| `0x04008f80..0x0400906c` | `0xec` | unknown | Unknown, not allocatable |
| `0x0400906c..0x04009080` | `0x14` | internal-context global/header prefix | Medium; free head is reached at base `+0x14` |
| `0x04009080..0x040094d4` | `0x454` | typed internal TX context pool: head + three `0x170` records | High exact linker-owned fixed quarantine |
| `0x040094d4..0x040096dc` | `0x208` | two power-save records, stride `0x104` | High base/stride/count; many retained consumers |
| `0x040096dc..0x04009720` | `0x44` | unknown/PS-HIF boundary | Unknown, not allocatable |
| `0x04009720..0x04009754` | `0x34` | HIF buffer/free-list and deferred-transfer roots | Medium |
| `0x04009754..0x04009928` | `0x1d4` | historical vendor HIF software/ring state; Rust owners now live in ITCM | High historical shape; fixed bytes remain quarantine for untranslated code |
| `0x04009928..0x0400993c` | `0x14` | MIC/HIF completion queue state | Medium; untranslated MIC path |
| `0x0400993c..0x04009a0c` | `0xd0` | PHY/RF/calibration/channel/gain state | High family root, incomplete typed layout; mixed fixed/native history |
| `0x04009a0c..0x04009c44` | `0x238` | PHY tail and unknown BSS | Low-medium; not allocatable |
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
| `0x04000260..0x04000288` | 10 `u32` visible | completion class callback table; class 6 at `0x04000278` contains Thumb `0x00015427` |
| `0x040002d8` | `u32` | hardware ring-cursor to software-slot translation |
| `0x040002dc..0x040002e4` | two 4-byte maps | pipe/queue status maps |
| `0x04000710..0x040007a4` | 37 `u32` | WSM command dispatch table |
| `0x04000804...` | table | AES transfer-class descriptors; class 6 TX CCMP, class 7 RX CCMP |
| `0x04000830..0x040009de` | `0x1ae` bytes | recovered AES mode-1 microcode, SHA-256 `211ad6ec...d881b4c` |
| `0x040010d4..0x040010e4` | 4 `u32` | per-pipe duration-quantum MMIO pointers |
| `0x040011ac..0x040011b4` | 8 bytes | HIF/control shadow and adjacent initialized state |
| `0x040011bc..0x0400123c` | 32 `u32` | IRQ callback table, reverse-indexed by IRQ |
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

**Known fields/shape:** `BootState` at `0x04001fd4` is currently described by a `register_structs!` type, but this type is only a partial view. `ClockParameters` is `0x28` bytes at `0x0400218c`. The scheduler handler table is 32 code pointers at `0x040021b4`.

**Readers/writers:** `sched_main_loop`, `evt_flags_set/clear`, `timer_start`, `timer_cancel`, `sched_arm_next_timer`, `task_22bc`, `mac_irq_handler`, HIF send/defer paths, MIC completion, scan/JOIN, channel switch, measurement, BA, power save, PHY tasks, and current Rust scheduler helpers. The literal-pool report found dozens of independent pointers resolving to `0x04001fd4`.

**Initialization:** low words around `0x04001fcc` are vendor COPY data and are explicitly reconstructed/cleared by Rust startup. The timer-list root at `0x04002014` is in vendor FILL/BSS and is explicitly reset before timer insertion. Handler entries are installed at runtime.

**Status:** mixed/shared. Even when a particular event bit is only set by Rust, untranslated tasks read the same event word and embedded timer objects retain vendor callbacks. This family requires `UnsafeCell` or volatile access under IRQ/FIQ exclusion. It is not sound to expose `&mut SchedulerState` while interrupt code can mutate it.

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

**Status:** shared/mixed. A partial typed prefix is useful for documentation, but any full `VifRecord` must preserve embedded timer layouts, intrusive ownership, and fields mutated by interrupts and untranslated tasks. It needs `UnsafeCell`/volatile access behind a VIF/MAC-domain guard.

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

**Status:** shared/mixed, volatile, timer-cyclic. The apparent record extent must be validated carefully because the decompiler shows offsets such as `+0x118..+0x120` from the per-interface calculation during initialization; either the logical structure is larger than `0x104`, the base denotes an interior view, or adjacent storage is intentionally shared. This ambiguity alone blocks a semantic Rust struct.

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

**Status:** historical typed quarantine. Do not reintroduce these into a native DTCM layout. The final migration should delete their consumers, not recreate them in a new DTCM struct.

### 4.10 PHY/RF/calibration family

**Range/root:** `0x0400993c..0x04009a0c` is the densest known core; related initialized tables are at `0x04000dd0`, `0x04000de8`, `0x04000e18`, `0x04001088`, `0x04001098`, `0x04002730`, and `0x040034b0..`.

**Known fields:** many byte/halfword/word fields in `src/phy.rs`; selected examples:

```text
0x0400993c..0x04009948    IQ reference coefficients/scales
0x0400994c...             shared PHY state root
0x04009974                channel frequency kHz
0x0400998c                silicon variant / calibration family
0x04009990,+0x9992        derived timing halfwords
0x04009994,+0x9998        measurements
0x040099a9                PHY state byte
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

**Initialization:** vendor COPY tables, vendor FILL zero, SDD-derived configuration, and substantial runtime PHY setup.

**Status:** mixed. The accepted migrations prove that small tuples can move only when all references are confined to translated code. They do not imply that the surrounding root can be moved.

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

`link-main-low.x` correctly:

* bounds ITCM to observed `0x1c000`;
* splits lower legacy DTCM around the fixed internal pool;
* anchors `.dtcm.context_pool` at `0x04009080` and asserts end `0x040094d4`;
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

Do not implement a monolithic DTCM struct now. Treat `0x04000000..0x0400a000` as a shrinking quarantine, with only exact, evidence-backed anchored views. Continue translating complete state-machine closures while preserving fixed addresses. When the scheduler/timer core, TX lifecycle, VIF/PAS/link/BA/power-save cycle, scan/JOIN/channel-switch paths, remaining HIF/MIC consumers, and PHY/RF fixed-root users are all native, perform one final atomic migration that removes every legacy DTCM root and loader assumption together.

The existing fixed internal context pool is the model for what is safe today: improve type shape and linker verification without claiming exclusive ownership or address mobility. The failed TALA exact-layout experiment is the model for what must not be inferred: exact bytes and source arithmetic do not prove that a black-box ABI address is irrelevant.
