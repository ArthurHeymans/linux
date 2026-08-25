# DTCM runtime contract

The lower DTCM range `0x04000000..0x0400a000` is one fixed shared-quarantine
ABI. `src/dtcm.rs` owns its physical layout and address construction. This
document records initialization and retention obligations; it does not grant
exclusive Rust ownership or permit address movement.

## Current closure

- Direct production DTCM literals outside `src/dtcm.rs`: **zero**.
- Direct `dtcm::...get()` arithmetic spellings outside `src/dtcm.rs`: **zero**,
  including arithmetic hidden behind an immediate `u32`/`usize` cast. Older
  production DTCM root/interior arithmetic outside `src/dtcm.rs` is now zero.
  The gate rejects direct arithmetic, integer-root constants, and arithmetic
  routed through local integer aliases; consumers use owner-derived fields.
- `.dtcm.*` section construction outside `src/dtcm.rs`: **zero**.
- The main Rust image has no DTCM `PT_LOAD`, COPY, or FILL payload.
- `InitializedVendorImage` remains exactly `0x2078` bytes at
  `0x04000000..0x04002078`.

The initialized-image transition contract is qualified on target for both cold
startup and warm SDIO rebind. The custom Rust image begins with the bytes left
by the preceding loader/vendor phase. Startup then reconstructs selected mutable
records. Retained bytes, zeroed bytes, and explicitly rebuilt bytes are different
contracts and must not be conflated. Rows still marked open remain family-level
closure requirements before those records can be relocated or claimed as
exclusively Rust-owned; they do not invalidate the qualified fixed-layout startup
contract.

## Initialized-image startup writer matrix

This matrix covers translated Rust startup writes into
`0x04000000..0x04002078`. “Retained” means that the Rust image currently relies
on the value already present at entry. It is an explicit audit state, not proof
that warm reload is deterministic.

| Physical field | Semantic owner | Cold source / Rust initializer | Warm-reload behavior | Readers and writers | Exclusion | Hardware-visible | State |
|---|---|---|---|---|---|---|---|
| `tx_duration_timing[10]` | MAC rate timing | vendor COPY, then `mac::initialize_tx_pipe_state` rewrites every entry | rebuilt | MAC/TX descriptor builders; startup writer | startup single-threaded | descriptor input | closed |
| `rate_encoding[22]` | PHY rate encoding | vendor COPY, then complete startup rewrite | rebuilt | TX/PHY descriptor builders; startup writer | startup single-threaded | descriptor input | closed |
| `rate_attributes[22]` | PHY rate attributes | vendor COPY, then complete startup rewrite | rebuilt | TX/PHY descriptor builders; startup writer | startup single-threaded | descriptor input | closed |
| `queue_pipe_mappings` | queue/pipe policy | vendor COPY; startup rewrites the mapping word and both four-byte direction maps | rebuilt | MAC/TX scheduling; startup writer | startup single-threaded | indirectly | closed for translated maps |
| `duration_quantum_pointers[4]` | MAC pipe timing | vendor COPY, then complete startup rewrite from MAC register identities | rebuilt | TX descriptor publication; startup writer | startup single-threaded | contains MMIO pointers | closed |
| `initialized_rate_policies[2][5]` | retained PAS policy template | vendor COPY | retained and copied into runtime PAS policy records | startup reader; retained PAS readers are possible | startup single-threaded for copy | no | retained COPY dependency |
| `irq_callbacks[32]` | interrupt dispatch | vendor COPY; registration replaces selected entries with Thumb callback pointers | partially rebuilt | interrupt dispatch and registration | IRQ/FIQ masked while routing changes | yes, callback publication | open: enumerate selected and retained entries |
| `scheduler_exclusion_state` | scheduler/MAC exclusion | vendor COPY, then both words explicitly zeroed by `platform::initialize_runtime_state` | rebuilt | scheduler foreground/IRQ/FIQ paths; platform writer | IRQ/FIQ contract required after startup | no | closed at startup |
| `control_words.tsf_resync_state` | TSF control | vendor COPY, then explicit zero | rebuilt | MAC/TSF paths; platform writer | startup single-threaded | indirectly | closed at startup |
| `control_words.random_lfsr` | retained vendor retry state | vendor COPY; translated Rust retry logic uses a separate ITCM `RETRY_RANDOM_STATE` | retained | retained vendor consumers only; no translated Rust writer | family-specific | no | open: prove whether retained code still consumes it |
| `control_words.tsf_accumulator_low` | TSF accumulation | vendor COPY, then explicit zero | rebuilt | TSF paths; platform writer | startup single-threaded | indirectly | closed for low word |
| remaining `control_words` | MAC/HIF timing state | vendor COPY | retained unless a later family initializer writes it | mixed translated and retained consumers | family-specific | mixed | open |
| `host_pas_ring.head/tail` | host PAS scheduler | vendor COPY, then both words zeroed | rebuilt roots | host-TX scheduler; MAC startup writer | MAC domain / IRQ+FIQ exclusion | no | roots closed |
| `host_pas_ring.slots[64]` | host PAS scheduler | vendor COPY | retained; logical emptiness currently comes from zero head/tail | PAS scheduler and diagnostics | MAC domain / IRQ+FIQ exclusion | no | open: stale-slot policy must be explicit |
| `mac_tx_queue_state` | MAC TX queue | vendor COPY, then head and tail zeroed | rebuilt | MAC/TX paths; MAC startup writer | MAC domain / IRQ+FIQ exclusion | no | closed at startup |
| `mac_pipe_records[4]` | packet-controller pipe ownership | vendor COPY, then `mac::rebuild_pipe_state` reconstructs pipe records | rebuilt by family initializer | MAC/TX/FIQ completion paths | MAC domain / IRQ+FIQ exclusion | yes | audit exact per-field publication order |
| `mac_retry_hardware_state` | retry/drain gate | vendor COPY, then complete control word zero | rebuilt | MAC/TX completion; MAC startup writer | MAC domain / IRQ+FIQ exclusion | indirectly | closed at startup |
| `mac_beacon_state.mode` and control words | beacon/response state | vendor COPY; mode becomes 2 and four control words become zero | partially rebuilt | beacon/TBTT paths; MAC startup writer | MAC domain | yes | open: response and completion fields retained |
| `mac_wake_runtime_state.timer` | wake scheduler timer | vendor COPY; startup publishes callback/context and clears previous-link | inactive timer contract | scheduler and wake paths; MAC startup writer | scheduler timer exclusion | callback is executable | open: verify inactive `next`/deadline semantics |
| `mac_wake_runtime_state.phy_state` | wake/PHY state | vendor COPY, then set to 2 | rebuilt scalar | wake and PHY paths | MAC domain | indirectly | closed scalar |
| selected `initialized_low_mac_prefix` bytes | low-MAC runtime | vendor COPY; receive gate/state and related controls are rewritten | partially rebuilt | MAC/TX/FIQ paths | MAC domain / IRQ+FIQ exclusion | yes | open at record level |
| `mac_runtime_accounting.software_records` | packet-DMA software free list | vendor COPY, then four nodes and free head are reconstructed | rebuilt | packet-DMA/MAC paths; platform writer | startup single-threaded, MAC domain once live | packet-RAM pointers | closed at startup |
| `scheduler_event_island.timer_list_head` | scheduler timer list | vendor COPY, then explicit zero | rebuilt | timer insertion/removal; MAC startup writer | scheduler IRQ/FIQ exclusion | no | closed at startup |
| PHY tables, gain records, calibration tables | PHY/RF | vendor COPY; selected records are rewritten during channel activation | retained until family activation | PHY foreground/tasks and retained routines | PHY/MAC domain as documented per routine | MMIO programming inputs | open by PHY subfamily |
| telemetry, debug, aggregation, HIF-control, opaque islands | mixed retained ABI | vendor COPY | retained or partially mutated | mixed translated/retained/debug consumers | family-specific | mixed | open; classify before relocation |

## Access rules

1. New and migrated production accesses receive a complete field or
   indexed-record address from `src/dtcm.rs`. Existing family-local aliases and
   record-internal arithmetic are migration debt, not precedent for new code.
2. Checked accessors are used at trust boundaries. `_unchecked` accessors retain
   intentional vendor-compatible arithmetic where callers already establish or
   deliberately omit bounds.
3. Volatile access preserves representation ordering but is not synchronization.
   Compound MAC, scheduler, pipe, PAS, and completion transitions require their
   existing IRQ/FIQ or `MacDomainGuard` exclusion.
4. No safe reference or slice may be formed over shared quarantine records.
5. Physical addresses remain fixed until every reader, writer, callback,
   computed root, IRQ/FIQ path, and warm-reload obligation for a whole family is
   closed.

## Snapshot qualification evidence

Target qualification covers both cold entry and warm Rust reload. Capture the
raw `0x2078` initialized image at these checkpoints when extending or reviewing
the contract:

1. after the qualified vendor/loader COPY and before Rust reconstruction;
2. after `platform::initialize_runtime_state`;
3. after MAC/pipe/internal-pool startup completes;
4. immediately before and after a warm Rust reload.

Compare by contract, not by whole-image equality. The reviewed transition ranges and the selected canonical values already
proved from startup source live in
`tools/dtcm-initialized-snapshot-contract.json`; validate raw target dumps with:

```sh
python3 tools/compare-dtcm-initialized-snapshots.py \
  before.bin after.bin --transition copy-to-platform
python3 tools/compare-dtcm-initialized-snapshots.py \
  before.bin after.bin --transition platform-to-startup
python3 tools/compare-dtcm-initialized-snapshots.py \
  before.bin after.bin --transition warm-entry-to-startup
```

The `dtcm-contract-diagnostics` feature now captures all three checkpoints into
ordinary ITCM immediately at their defined startup boundaries. It exposes the
captures through private read-MIB IDs `0xff00..0xff47`: entry uses
`0xff00..0xff17`, platform uses `0xff18..0xff2f`, and startup uses
`0xff30..0xff47`. Each stage has 24 pages with at most 352 snapshot bytes per response so
the complete WSM confirmation fits the 384-byte HIF output slot. Feature-free
firmware contains neither the buffers nor the MIB path.

Build the diagnostic image with:

```sh
cargo +nightly build --release --bin hif-startup \
  --features dtcm-contract-diagnostics \
  --target thumbv5te-none-eabi -Z build-std=core
```

Save the 72 raw read-MIB confirmations (or their diagnostic data portions), then
assemble and compare them with:

```sh
python3 tools/assemble-dtcm-initialized-snapshots.py responses/*.bin \
  --output-dir snapshots
python3 tools/compare-dtcm-initialized-snapshots.py \
  snapshots/entry.bin snapshots/platform.bin --transition copy-to-platform
python3 tools/compare-dtcm-initialized-snapshots.py \
  snapshots/platform.bin snapshots/startup.bin --transition platform-to-startup
```

On a warm firmware reload, use that run's `entry.bin` and `startup.bin` with the
`warm-entry-to-startup` transition. Because the target's pre-existing rebind
failure can stall the first post-startup WSM command, the diagnostic firmware
also compares the warm entry image internally before reconstruction and places a
bounded result in the ordinary startup indication label. `u` is the count of
changed bytes outside reviewed writer ranges, `f` lists their first offsets, `c`
is the count of canonical startup-value mismatches, and `e` lists their first
offsets. This path requires no extra WSM command or MMIO read.

Qualified target evidence:

- cold COPY-to-platform: 43 changed bytes, all in reviewed writer ranges;
- cold platform-to-startup: 398 changed bytes, all in reviewed writer ranges;
- cold startup label: `XR819 DTCM u=0000 f=none c=0000 e=none`;
- warm SDIO unbind/rebind startup label:
  `XR819 DTCM u=0000 f=none c=0000 e=none`.

The warm run subsequently reproduced the known command-channel failure
(`0x0006` timeout followed by BH termination). The zero warm report classifies
that failure outside initialized DTCM reconstruction: all warm-entry differences
were writer-owned and all selected startup fields reached their canonical values.
The normal software gate builds the ARM diagnostic image, verifies its stack and
linker envelope, checks that the generated firmware contract matches the reviewed
JSON, and runs synthetic capture/assembly/comparison regressions.

- COPY-stable table ranges must match the qualified reference bytes;
- rebuilt fields must eventually gain canonical startup values rather than only
  an allowed-change range;
- callback words must eventually resolve to expected current-image targets;
- inactive intrusive records must eventually gain explicit list-invariant checks;
- retained mutable ranges must have an explicit preservation or reset rule;
- every changed byte must belong to a named writer in the matrix.

A diagnostic prefill test must restore COPY-stable tables before execution and
prefill only ranges declared reconstructible. It must never overwrite live
stacks or use `0x0400c000..0x04010000`, which aliases lower DTCM.
