# Full-capture qualification ledger

## Local validation

25 focused tests passed with the native helper enabled, including all-schema
random differential projection against Python, cookie reuse/requeues, signed
errors, overlap, lost/truncated streams, output caps and graceful tail drain.
The C helper passed20000 host ASan/UBSan boundary exercises. Host/ARM shared
objects had no undefined symbols or external library dependencies. Standalone
Pyright reported0 errors/warnings. Persistent editor import diagnostics for
new sibling modules did not reproduce in the standalone checker or runtime.

## Rejected Python-projection smoke

`/tmp/xr819-full-flow-smoke.log`, capture
`/tmp/xr819-full-flow-1788891146331930309.xtf`, exit24. Initial ping50/50;
traffic was interrupted after24.24s. CPU2 ring overrun130, with queued unread
events at termination. Collector CPU16.90s during roughly25s collection.
This is instrumentation saturation, not firmware failure or a timing baseline.

The failed batch had five validated but unemitted records; the footer's count
therefore disagreed with the stream. The decoder correctly rejected it, but
failure reporting hid the original reason when STOP hit a closed SSH pipe.
Native projection now preserves its valid prefix before reporting an error;
receiver manifests retain the failure even if shutdown fails, and unbuffered
stdin avoids a deferred BufferedWriter BrokenPipe traceback. No lost run was
reclassified as complete. Recovery files/trace absence were verified.

## Native-projection smoke: complete capture, overhead unqualified

`/tmp/xr819-full-flow-native-smoke.log`, capture
`/tmp/xr819-full-flow-1788892244367665443.xtf`, exit0. Vendor, automatic rates,
same static host module; no AP observer for this initial collector qualification.
TCP8.62 Mbit/s over30.29s; both ping sets50/50. This is not an overhead benchmark.

93308 events:22635 admissions,45542 transport events (including136 command
write pairs),22635 confirmations,2493 netdev events and3 MMC errors. All22635
data lifecycles matched, zero censored live episodes. CRC/sequence/final counts
validated, zero ring overruns/drops, final entries0 on all four CPUs, no partial
line. Artifact7000880 bytes. Recovery independently verified after a transient
post-reboot SSH timeout.

Observed vendor latency histogram median bounds: admission→write start
2.10–2.36ms; SDIO call0.131–0.147ms; write return→confirmation4.72–5.24ms.
These are host-observed intervals, not pure firmware CPU/airtime, and are NOT
yet an unperturbed performance baseline. Timestamps preserve text trace precision
(rescaled to integer ns), not extra precision invented by the binary encoding.

Collector CPU18.29s remains substantial. Next separate user/system CPU,
count trace reads/empty wakeups/input bytes and bound empty-read polling with
1ms backoff. Do not enlarge rings to hide saturation or call the collector
neutral merely because one full capture passed. Then qualify old/full collection
in opposite orders and retain a tracing-disabled reference before comparing
vendor/Rust latency distributions.

## Polling and batching investigation

Poll-instrumented capture `/tmp/xr819-full-flow-1788892824038620738.xtf`
matched21646 requests with no loss/censoring; both pings50/50. Collector
18.00CPU seconds over31.75s:13.34user/4.66system. Empty reads0/26611, so
empty polling was not responsible. The1ms empty-read backoff was not exercised.

Synthetic-only board profiling (`/tmp/xr819-projection-offline-profile.log`)
used30000 numeric fixture events, no tracing or Wi-Fi load. Tiny-batch Python
call/framing overhead was substantial. Batching up to16KiB or10ms reduced
profiled elapsed time2.61→0.365s for the same event count; this is an instrumented
synthetic workload, not a live throughput/neutrality claim.26 tests then passed.

Batched hardware capture `/tmp/xr819-full-flow-1788893741363336746.xtf`
matched24700 requests,101476 events, zero loss/censoring and empty final rings.
Both pings50/50. Collector13.87CPU seconds over31.74s:8.88user/4.99system;
2675 projection batches,60093 reads, zero empty reads. Recovery verified.
Full-event recording works, but CPU cost remains significant.

## Frozen overhead calibration: neutrality not established

`/tmp/xr819-full-flow-calibration.sh` completed all five120s traffic phases
in1616s including setup/recovery. Frozen collector source in
`/tmp/xr819-full-flow-calibration-tools`, input hashes in
`/tmp/xr819-full-flow-calibration-inputs.sha256`. Same stock vendor firmware,
static module, automatic rates, patched AP; AP probes off throughout. Recovery
files and trace absence verified independently after each run. All ten50-ping
sets passed. No guard fired.

- Old A:4.45 Mbit/s; final TCP389 retransmissions/46534 data segments.
- Full A:3.56 Mbit/s;407/37167. Capture
  `/tmp/xr819-full-flow-1788894590672585190.xtf`:161331 events,37454 matched
  requests, zero censored,23.96collector CPU seconds.
- Full B:3.98 Mbit/s;422/41751. Capture
  `/tmp/xr819-full-flow-1788894917119118327.xtf`:179519 events,42040 matched
  requests, zero censored,28.40collector CPU seconds.
- Old B:4.03 Mbit/s;415/42139.
- Tracing disabled reference:5.04 Mbit/s;364/52480.

Both old collections reported zero parser/loss/ring errors and complete local
drain. Both full captures passed sequence/CRC/footer/lifecycle validation.
Full A was20% slower than adjacent Old A; Full B was1.24% slower than Old B.
The single untraced reference was faster than either collector. Changing loss
and run-to-run variation prevent attributing all differences to instrumentation,
but these results DO NOT establish neutrality. Do not treat120 one-second bins
as120 independent experiment replicates or claim a fixed overhead percentage.

Keep the captures as qualified event histories, not unperturbed performance
baselines. Before a quantitative vendor/Rust timing comparison, the next design
candidate is native projection directly from kernel binary trace pages, using
an established trace-event decoder rather than another handwritten page parser.
Only allowlisted fields may leave the process; do not save raw pages/pointers.
This would remove kernel text formatting and much remaining userspace work,
but is not implemented or qualified by the above results.
