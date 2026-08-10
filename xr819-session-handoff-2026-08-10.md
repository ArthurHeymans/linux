# XR819 firmware session handoff — 2026-08-10

Handoff summary for continuing after the pi-web session of 2026-08-07..10 was
interrupted mid-run. Read this first, then
[`xr819-hif-startup-flow.md`](xr819-hif-startup-flow.md) (the authoritative
startup/TX handoff) and
[`xr819-firmware-reverse-engineering.md`](xr819-firmware-reverse-engineering.md).

## Project in one paragraph

Incremental open reimplementation of the XRadio XR819 (cw1200-family) Wi-Fi
firmware in Rust (`#![no_std]`, `thumbv5te-none-eabi`), developed against the
vendor 2018 firmware (`XR_C01.08.52.58`). Strategy: translate vendor MMIO
semantics *exactly* from disassembly, prove each boundary on real hardware
behind compile-time/inactive guards, keep the normal (non-experimental) image
byte-identical to the known-good build, and replace vendor policy machinery
with simpler Rust design only after the hardware contract is proven.

Current milestone: **scan-owned probe TX**. Passive multi-channel scan RX is
hardware-proven and committed. The `probe-tx-experiment` feature publishes
wildcard probes during channel-1 dwell through a scan-state-owned controller
and a boot-wide publication budget. **Reliability of repeated publication is
the open problem** — that is where the session was interrupted.

## Repository state

- jj repository (colocated with git). Working copy `@` = `lzzlkprr` (no
  description). Parent `ykkxsxvn` "xr819-open-firmware | Add inactive XR819 TX
  service executors".
- **Uncommitted working-copy changes** (the reliability work, all from the
  last session):
  - `xr819-firmware/src/scan.rs` (+286): scan-owned probe state machine
    (`WaitingForTune → WaitingForDelay → InFlight → WaitingForDelay/Done`),
    `probe_publication_budget` (currently **8**), deadline/transition
    helpers, host tests.
  - `xr819-firmware/src/tx.rs` (+454): TX service executors, event
    instrumentation (event loop max raised 4 → 32 → 256 in experiments;
    final images used 32).
  - `xr819-firmware/src/bin/hif_startup.rs` (+76): experiment hook, max
    events per HIF call now **32** (was 256 in one experiment, 4 before).
  - `xr819-firmware/Cargo.toml` (+4): `probe-tx-experiment` feature.
  - Both `.md` docs updated with the audit narrative through "68 tests".
  - Untracked: `xr819/ghidra-fw-main.bin.gzf` (2.2 MiB — exceeds the 1 MiB
    jj snapshot limit; see quirks below).
- Host coverage: **70 tests pass**; thumbv5te release build passes; LSP
  clean. Non-feature build has expected dead-code warnings for the
  feature-gated helpers (`deadline_reached`, `transition_error_code`,
  `ActiveProbePhase::{WaitingForDelay, Done}`).

## Environment quirks (learned the hard way)

- **jj snapshotting fails with a GPG signing error** in this environment
  when it has to write a new working-copy commit. Workaround:
  `jj --config signing.behavior=drop <cmd>`.
- The untracked `xr819/ghidra-fw-main.bin.gzf` trips the 1 MiB new-file
  snapshot limit. Workaround: add `--config snapshot.max-new-file-size=3000000`
  (or gitignore it).
- Use `nix-shell -p <pkg> --run '...'`, **not** `nix shell` (user corrected
  this twice).
- Rust toolchain is not on PATH by default. Builds use:
  ```bash
  export PATH=/home/arthur/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:$PATH
  OBJCOPY=/nix/store/akhr6dy289x36fwlhak6kman61dj2blg-gcc-arm-embedded-14.2.rel1/bin/arm-none-eabi-objcopy
  cargo fmt --manifest-path xr819-firmware/Cargo.toml
  cargo test --manifest-path xr819-firmware/Cargo.toml --features probe-tx-experiment
  cargo build --release --manifest-path xr819-firmware/Cargo.toml --bin hif-startup \
      --target thumbv5te-none-eabi -Z build-std=core                      # normal image
  cargo build --release ... --features probe-tx-experiment                # experiment image
  $OBJCOPY -O binary xr819-firmware/target/thumbv5te-none-eabi/release/hif-startup /tmp/<name>.bin
  ```
  Always record the sha256 of every image and verify it on-target after
  install (`sha256sum /lib/firmware/xr819/*.bin`).

## Target hardware

- Board at **192.168.0.104**, `root` / password `1234`, via
  `/nix/store/nkkj35yh0rmj71bwyz8wn7jg6mkm15bx-sshpass-1.10/bin/sshpass -p 1234 ssh -o StrictHostKeyChecking=no`.
- Driver: `cw1200` (mainline), MMC device `1c10000.mmc` on
  `sunxi-mmc`. Firmware files: `/lib/firmware/xr819/boot_xr819.bin` (boot
  block, currently the low-current variant) and `fw_xr819.bin` (the image
  under test). Reload procedure:
  ```sh
  systemctl stop netplan-wpa-wlan0.service   # keep hostapd/wpa away during experiments
  ip link set wlan0 down
  echo 1c10000.mmc > /sys/bus/platform/drivers/sunxi-mmc/unbind; sleep 1
  echo 1c10000.mmc > /sys/bus/platform/drivers/sunxi-mmc/bind;   sleep 5
  ip link set wlan0 up
  ```
- Test: `timeout 35 iw dev wlan0 scan freq 2412` (channel 1). Success =
  `rc=0`, ~363/1222 result lines (varies per image), BH alive.
- Health check after each scan batch (note: `phyNNN` changes every bind):
  ```sh
  phy=$(basename /sys/class/ieee80211/phy*)
  dir=/sys/kernel/debug/ieee80211/$phy/cw1200
  tail -1 $dir/counters          # last field abused as experiment diagnostic
  cat $dir/status  (BH status/Pending TX/Used bufs/WSM/Scan)
  ```
- **Healthy state**: `BH status: alive`, `Pending TX: 0`, `Used bufs: 0`,
  `WSM: idle`, `Scan: idle`.
- **Failure signature** (the bug under investigation): `BH: terminated`,
  `Pending TX: 2–4`, `Used bufs: 1`, debugfs `counters` reads return
  `Connection timed out`, subsequent scans return 1 line then `rc=146`
  (timeout, -110).
- Postmortem via debugfs is currently **blocked**: `unsafe_debugfs`
  module param is `N` and echoing `Y` fails (`Operation not permitted`,
  then `Connection timed out` once BH is dead); `halt` write fails after BH
  death; `apb` reads return `Device or resource busy`. A kernel-side change
  (enable `unsafe_debugfs` at boot, or capture before death) is needed for
  AHB postmortem.

## Current target state (important)

The target is **still running the experimental image**, not the stable one:

```text
/lib/firmware/xr819/boot_xr819.bin = 81acc379b2c5…  (download-boot-low-current)
/lib/firmware/xr819/fw_xr819.bin   = ecea4bc9600a…  (hif-startup-reliable-final-8, budget 8, events 32)
```

Stable known-good firmware is `/tmp/xr819-normal-after-hw.bin` locally,
sha256 `96714ce0fee7c947ed512df54e49ec9edac32233c3446eba0cb7308818cfb765`.
Restore procedure: install it + `download-boot-low-current.bin`, unbind/bind,
run one verification scan, confirm alive BH / zero used buffers.

## Where the session was interrupted

Last user instruction: **"Ok try to make it reliable."** The session:

1. Ran a reliability matrix on repeated channel-1 scans with varying budget
   and event-loop sizes (results below).
2. Built final candidates `hif-startup-reliable-final-{normal,8}.bin`
   (tests passed, 70/70) and started the final 3-boot × 5-scan test.
3. **boot 1 passed** (5 scans, 4 publications, then budget consumed, BH
   alive). **boot 2 failed immediately** (scan 1 → 1 line, scan 2 →
   rc=146): the BH dies across the unbind/bind rebind.
4. Rebooted the target to recover (came up after ~9×3 s), then spawned a
   subagent which was aborted when pi-web died. No final summary was
   written; the working-copy edits listed above are all present on disk.

### Reliability test matrix (2026-08-10, all with the scan-owned state machine)

| Image | Budget | Events/call | Result |
| --- | ---: | ---: | --- |
| `reliability-8` (`ad1c0c23…`) | 8 | 32 | 6 scans OK; 4 publications then budget consumed; BH alive |
| `reliability-32-v2` (`8818701b…`) | 32 | 32 | died after 2 publications (scan 3) |
| `reliability-32-events` (`c73f7e76…`) | 32 | 32 | died after 6 publications (scan 7) |
| `reliability-32-events256` (`5e769b4e…`) | 32 | 256 | died after 2 publications (scan 3) |
| `reliable-final-8` (`ecea4bc9…`) | 8 | 32 | boot 1: 5 scans OK (4 pubs); **boot 2 (after rebind): dead on scan 1–2** |

Conclusions so far:

- Death is **non-deterministic** (2–6 publications) — not a fixed budget
  accounting bug. One buffer is left used and 2–4 TXs pending at death.
- Death also happens **across driver unbind/bind** even within budget —
  fresh-boot state is not the trigger.
- Normal-image scans after restore always pass; the normal image is
  byte-identical and unaffected.
- Diagnostic encoding: the last `counters` field (host driver labels it
  `rx_mgmt_ccmp_replays`) carries the experiment diagnostic; each
  publication adds `0x20200` (e.g. `0x23200 → 0x83800` for 4 pubs with
  budget 8 — budget is consumed 2 units/publication).

## Next steps (in order)

1. **Restore the stable image** on the target and verify (see above).
2. Decide how to observe the death: either boot the target kernel with
   `cw1200_core.unsafe_debugfs=Y` (or set it before experiments and halt
   the BH *before* it dies), or extend firmware-side postmortem capture
   (the fatal path already publishes a versioned record; the reliability
   death does not go through that path — it looks like a silent stall, not
   the fatal-quiesce).
3. Instrument **publication/completion identity per attempt** (pipe, slot,
   generation, ring cursors, scheduler word) so the stall can be attributed
   to publication N vs completion N-1. The previous session's stated plan:
   "instrument every publication/completion identity to determine why the
   later repeated publication stalls before safely increasing the budget".
4. Investigate the **rebind death** (boot 2) separately — it may be the
   same bug with zero successful publications, which would make it the
   cheaper repro: one bind, one scan batch, dead.
5. Only after the stall is understood: raise the budget beyond 8 and
   re-run the 3-boot × 5-scan matrix.

## Key technical facts (do not re-derive)

- Command storage ≠ hardware ring: per-pipe command storage at
  `0x09007080 + pipe*0x150`; hardware ring at `0x09c60000 + pipe*0x80`;
  completion-ring state at `0x04008f6c`; event FIFO at `0x09c00a20`
  (readiness at `0x09c00a24`, destructive pop); scheduler pending word
  `0x09c00e84` (captured once per event pass); completion bit 20 claimed
  atomically from `0x04001fd4`.
- Publication ordering is trigger-before-GO; final GO is hardware-ring
  `+0x14 = 1`. Class-6 callback must report `Returned` before completion
  is recorded; context identity is checked on return.
- Vendor `txp_pipe_tx_done_retry` (`0x9550`) receives only the saved
  `0x100 << pipe` pending mask (r6 at `0x9f98`), **not** the whole
  scheduler word. Inactive-pipe command sentinel is `0xff00ffff`
  (corrected from the r2 guess `0xc7ff00ff`); matching-payload kind-2
  status counter is `0xfff01aa4` (not the shifted r2-image address).
  Give-up completion status is `0x0b`, which is **not** automatically
  terminal.
- `ProbeTxTracker` has monotonic generation identity; retry resolution is
  explicit; fatal-quiesced state exits only via reset.
- Timer translation: `timer_start`/`timer_cancel`, IRQ/FIQ-protected sorted
  intrusive list at `0x04002014`, deadline `0x0ac00004 + 0x0400143c`,
  compare at `0x0ac00014/1c`. CPSR restore helper (`0xf018`) replaces only
  I/F bits.
- The experiment hook records publication/completion/failure at
  `0x0900ffa0..a8`; with the guard false LLVM removes it and the release
  image stays byte-identical. `STARTUP_DEBUG_STAGE`/`REMAP_DEBUG_INDEX`
  are patchable diagnostics, not production design.
- Hardware-proven images so far: one-shot `c82366b1…` (diagnostic
  `0x3000`), bounded-2 `24b069f4…` (`0x3200`, 2 completions).
  Known-bad: unbounded `76a11f0d…` (≥15 pubs then lost completion),
  `8596595f…` (timeout, 1 used buffer, terminated BH).
- `ieee80211` crate (0.5.9, no_std, forbids unsafe) is a candidate for
  frame build/parse **above** the XR819 hardware boundary only.

## Failed experiments — do not repeat without new evidence

- Unbounded republication (see `76a11f0d…` above).
- Treating status `0x0b` as automatically terminal.
- Postmortem through debugfs `halt`/`apb` after BH death (busy/timed out).
- Kimi K3 workflow branches: provider returns 401 (invalid/expired API
  key). Use Terra/Luna branches instead.
- Routing-table installation before `0x9ac` completes (too early; removed
  from the minimal path).

## pi/session notes

- The interrupted session file:
  `~/.pi/agent/sessions/--home-arthur-src-linux--/2026-08-07T08-17-46-338Z_019fdb4c-b962-7f25-806a-4c1ef7483777.jsonl`
  (36 MB; last entry is the aborted subagent).
- Models in use: `openai-codex/gpt-5.6-sol` (main), Terra/Luna workflow
  branches for parallel read-only audits. Parallelize **read-only
  reconstruction only** — the translated functions share ownership state,
  so parallel edits create conflicting assumptions.
