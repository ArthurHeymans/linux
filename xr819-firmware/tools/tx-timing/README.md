# Full XR819 TX timing capture

Diagnostic tools for the separately built static `cw1200_flow` module. No
firmware changes or ARM function probes. Hardware qualification is pending.

## Files

- `flow_format.py`: numeric allowlist projection,64-byte event records,
  bounded32-byte frame headers, CRC32, sequences and final accounting.
- `collect_flow.py`: board collector with nonblocking bounded output, a separate
  socket/ring sampler, STOP handshake and quiescent final drain.
- `project_flow.c`: allocation/libc/syscall-free native numeric projection hot
  path; `build-native.sh` builds both architectures and runs differential tests
  plus host ASan/UBSan boundary exercises. Python projection remains a reference
  implementation for fixtures, not the hardware default.
- `run_flow.py`: workstation receiver, private artifact creation, TCP progress
  guard and bounded shutdown. Uploads only the collector and format module.
- `analyze_flow.py`: episode matcher and unconditional/stratified host-timestamp
  latency distributions. Quantiles are explicit histogram bounds, not exact
  percentiles. Cookie reuse, requeues, overlap and censoring are explicit.
- `test_flow.py`: pure local fixtures; no hardware access.

Run pure Python tests from the repository root (native tests explicitly skip
unless `XR819_FLOW_NATIVE_TEST_LIB` is provided):

```sh
python3 -m unittest discover -s xr819-firmware/tools/tx-timing -p 'test_*.py' -v
```

Build and qualify the native helper, including all tests:

```sh
nix-shell -p pkgsCross.armv7l-hf-multiplatform.stdenv.cc --run \
  'bash xr819-firmware/tools/tx-timing/build-native.sh'
```

Outputs live in private `/tmp/xr819-flow-native`, not the repository. Deployment
requires a matching source-hash qualification stamp and verifies uploaded hashes.

Analyze a saved capture without touching the board:

```sh
python3 xr819-firmware/tools/tx-timing/analyze_flow.py /tmp/xr819-full-flow-….xtf
```

Do not invoke the hardware runner casually: it assumes the guarded external
harness has installed the exact module/firmware, associated the station, set
rate policy, started the namespace iperf server and owns recovery. The current
integration is `/tmp/xr819-tcp-full-flow-run.sh`; its only change from the
qualified static harness is the observer invocation. The short qualification
wrapper is `/tmp/xr819-full-flow-smoke.sh`, using vendor firmware and automatic
rates. It does not require AP probes for its initial collector-safety check.

## Protocol and artifacts

All integers in binary event records are little endian. Layout is
`<QQHH10II`: collector sequence, exact kernel monotonic ns, CPU, event kind,
ten32-bit value slots and one reserved zero word. Unused slots must be zero.
Only signed result/error fields use two's-complement decoding. Schema IDs
and field order are defined in `SCHEMAS`; no pointers, raw trace lines or
packet contents are serialized. MMC completion direction remains unknown.

Frame layout `<4sBBHQQII`: magic `XTF1`, version1, kind, reserved0,
first sequence/current total, record count, payload length, CRC32.
CRC covers the header except its CRC field followed by payload. DATA payloads
contain whole records; controls contain bounded ASCII JSON produced from
selected numeric metadata and fixed protocol fields. Payload limit64KiB.
Socket records contain numeric fields only, not raw `ss` output.

The receiver saves an exclusive mode600 `.xtf` artifact plus a manifest and
analysis report. It validates records while draining independently of live
failure capture; only the latest socket sample is exposed to the main loop.
Skipped console samples remain in the binary file and reset the live progress
inference rather than inventing continuous backlog. The full file remains
available for later offline time-bin/gap analysis; the current report focuses
on per-request latency distributions, not exact driver occupancy.

Collection fails on malformed trace text, ring loss, dead/stalled sampling,
1MiB output backpressure,512MiB byte budget, signal/disconnect or deadline.
Normal shutdown stops traffic, requests STOP, disables producers, drains until
quiet for100ms (bounded2s), checks remaining entries/loss counters, emits final
accounting, drains output and removes the instance. Forced shutdown is not a
successful capture. A complete binary file is separate from successful traffic
and verified recovery; the harness must independently check the latter.

Collector START and END monotonic stamps plus workstation launch/ready/STOP/
END receipt stamps in the manifest provide coarse clock brackets. They do not
establish sub-millisecond cross-host synchronization. Use board-local timing
for service measurements; retain existing AP counters only for coarse windows.

## Qualification still required

Passing local fixtures does not prove ARM runtime compatibility, actual-hit
safety, neutral overhead or correct firmware semantics. First qualify a short
hardware run, including final drain and recovery. Then compare old versus full
collection in both orders and a tracing-disabled reference before interpreting
vendor/Rust timing differences. Keep the existing images and static module
unchanged. Extend terminal-event coverage only if censored histories prevent
the next inference; unresolved cookies are not proof of firmware ownership.
