# XR819 TX timing capture design

## Question and current evidence

Locate the service-time or delivery difference between clean Rust and vendor
under the same automatic-rate, no-host-TX-BA configuration. Do not assume a
scheduler defect or use MCS0 as a throughput fix.

The latest serial pair completed approximately600s without a10s stall:
vendor5.37 Mbit/s, Rust3.24 Mbit/s. Firmware retry-exhaustion fractions were
0.404% and0.3145%, respectively. These are not equivalent to TCP loss rates.
A single pair cannot establish a stable40% deficit.

Existing counters count every parsed event, but timestamps retain the first
8 events/type/interval, not a random sample:

- Vendor:4621/280643 confirmation timestamps (1.65%);
  4648/564588 transport timestamps (0.82%).
- Rust:4708/168528 confirmation timestamps (2.79%);
  4730/339748 transport timestamps (1.39%).

Those samples diagnose sparse stalls, not unbiased latency distributions.
Do not reconstruct missing histories, estimate percentiles from them, or
compare firmware `media`/`queued` delay fields across images as qualified
wall-clock durations.

## First implementation: keep the existing diagnostic module

Use the existing three static driver events and numeric netdev/MMC projection.
No new firmware instrumentation, dynamic ARM probes, policy changes, or AP
module replacements. Retain the current completion/ownership/replay behavior.

Replace first-eight sampling with a versioned full-event numeric stream over
Ethernet SSH, saved on the workstation. The board performs schema validation
and projection only; matching, histograms and analysis run off-board.

Each record carries event type, CPU, exact kernel monotonic timestamp in integer
nanoseconds (parse decimal text without floating-point conversion), and a
collector sequence number. Preserve the currently allowed numeric fields:
queue cookie/previous cookie/length/queue/requeue; transport data/command,
message ID, length, before/after/result; confirmation cookie/status/rate/ACK
failures/flags/opaque firmware delays. Netdev has length only. MMC includes
opcode/errors/transferred bytes, with direction explicitly unknown.

Use fixed-width binary records with a schema-version header and bounded batch
framing. The format must specify byte order, record sizes and field widths;
include a batch checksum and sequence range so truncation is distinguishable
from valid EOF. No raw trace text, pointers, response words, payloads, keys,
credentials, unbounded strings or packet-content capture. Collector sequence
numbers detect downstream omissions, not events already lost in kernel rings.

At the observed vendor load, four driver records per WSM request mean roughly
1.12 million records per600s; at64 bytes each this is about72MB, plus commands,
netdev records and framing. Set an explicit512MiB per-run disk budget and a
bounded64KiB read/batch size. Reaching any cap fails capture visibly; never
silently fall back to sampling. Record actual byte rate and CPU cost rather
than assuming this estimate proves neutrality.

Keep drain separate from `ss`: the current collector blocks its trace reader
while running a subprocess with a3s timeout. Run the1Hz socket sampler in a
separate process with bounded output and timeout, timestamping before/after
each query. Similarly, collect cumulative per-CPU ring statistics separately
without pausing event reads. Record collector/socket-sampler CPU time and
system CPU/load at low frequency, not a new high-frequency probe.

On SSH failure or sustained output backpressure, mark capture invalid and stop
the test with bounded cleanup. Do not promise losslessness under unbounded
stalling or accumulate unbounded RAM. Keep the private workstation log mode600.

## Lifecycle matching: episodes, not globally unique cookies

Offline matching processes the whole stream. Keep capture order and timestamps;
do not conceal unexpected ordering by blindly sorting all records.

- A non-requeue admission creates an episode `(run, cookie, occurrence)`.
  Numeric cookies can repeat after completion. A collision with an unresolved
  episode is an ambiguity, not permission to overwrite it.
- A transport start creates an attempt within its admission episode. Match its
  end to that attempt, including data flag and message ID. Command writes are
  analyzed separately; do not associate their placeholder cookie with data.
- A confirmation closes a submission's response interval, not necessarily the
  packet's entire lifetime. Preserve status and flags. WSM_REQUEUE followed by
  the existing requeue event links previous/new cookie episodes; it is not
  final delivery or a new TCP packet. Support repeated attempts explicitly.
- Permit completion processing to overlap a write return if observed; retain
  signed timing and classify overlap. Do not clamp negative values or assume
  a cross-CPU causal ordering without verifying it. Use bounded pending joins
  and report ambiguous/unmatched events and CPU/order anomalies separately.
- Beginning/end-of-run fragments, parse errors, ring loss, cancellation without
  a terminal event, collisions, and unmatched retries are censored records.
  Exclude ambiguous joins from latency distributions, with counts and bounds.
  An unresolved episode is not proof of stranded firmware ownership.

Existing events do not cover every queue cancellation/removal. Therefore
matched-cohort queue occupancy is only a lower bound: positive occupancy can
show known work waiting, but zero does not prove an empty driver queue. If
this limitation blocks diagnosis, add a narrowly scoped static terminal-event
probe covering all remove/clear/expiry paths in a second phase, with a fresh
actual-hit qualification. Do not invent exact ownership from partial hooks.

## Measurements and decision rules

For every unambiguous data attempt compute:

1. Admission → write start: host queue/service wait, including any unobserved
   selection/credit work. It does not separately identify those sub-stages.
2. Write start → return: host-observed SDIO call duration.
3. Write return → confirmation: response latency, not pure airtime or firmware
   CPU execution. It includes firmware work, delivery/retries and host receive
   processing. Also report write-start → confirmation for overlap cases.
4. Admission → confirmation: total observed request response latency.

Report count, sum, median, p90/p99, maxima and matched/censored fractions, split
by queue, frame-size band, confirmation rate/status/flags and requeue behavior.
Keep unconditional distributions too: conditioning only on successful or
high-rate frames can hide the cause. Compare common strata with adequate
sample counts; completion rate is not a full retry-rate chain.

Measure idle gaps, concurrent submitted-not-yet-confirmed requests, completed
requests/bytes per second and overlapping waits. Never add overlapping packet
latencies and call the sum elapsed firmware busy time. Use interval unions
and matched-cohort occupancy, with explicit incomplete-ownership caveats.
Use event-time100ms bins plus1s/whole-phase summaries; large TCP gaps and typical
service distributions are different outcomes. Keep first/last traffic periods
separate from the steady-state middle without silently dropping slow tails.

Join the separately sampled TCP sender backlog, bytes_acked, cwnd, RTT and
retransmission counters by time windows, not imagined packet identity:

| Observation | Next boundary to inspect |
| --- | --- |
| Backlogged TCP, sparse admissions | Host stack/qdisc/mac80211/driver-entry path;1Hz backlog alone does not prove continuous packet availability |
| Known admitted work waits before write | Host selection/credit/service boundary |
| Long write calls/errors | SDIO/host transport |
| Prompt writes, long confirmation waits | Firmware/radio/host RX-confirmation boundary |
| Fast lifecycle but TCP/ACK progress gaps | Delivery/reordering/ACK behavior; success confirmation is not end-to-end delivery |

Netdev SKB counts and WSM frame counts differ under GSO. Existing netdev length
alone cannot join a TCP segment to a queue cookie. Do not use it as an exact
per-packet latency boundary. If that boundary becomes the lead, separately
design a scalar driver-entry token with complete drop/segmentation semantics;
do not export kernel addresses to approximate an identity.

## AP clocks and scope

First extract board-local timing with kernel monotonic timestamps: these need
no workstation/board clock subtraction. Keep the existing AP observer fixed
through matched tests; restart only between runs to preserve coverage.

For cross-host plots, collect repeated bounded clock-exchange brackets over
Ethernet at start/end and infrequently during capture. Retain RTT and clock
uncertainty/drift bounds. Board wall time alone is not synchronization proof.
AP probes and workstation use the AP host's monotonic domain. Compare broad
activity windows only when wider than clock uncertainty plus aggregation.

The current AP trace has cumulative counters every5s and bounded packet
samples. It cannot support sub-millisecond board→AP latency joins. Use it for
coarse receive/TCP/ACK silence and drop attribution only. If board-local
measurements leave this boundary unresolved, design an independently qualified
AP capture of permitted sequence/TID/retry and TCP range metadata, preserving
retransmission, aggregation and TCP-coalescing ambiguity. Do not imply the
initial implementation measures application-to-air or ACK round-trip timing.

## Completeness and shutdown are acceptance gates

Start the trace reader and sampler before enabling events; take initial ring
stats and explicit READY, then launch traffic. At normal stop: end traffic,
allow a bounded tail, disable event production, drain the trace pipe until
empty, emit the final partial interval, final ring stats, record totals,
checksum/sequence footer, and only then close SSH/remove the instance.
Use an explicit graceful-stop handshake rather than SIGTERM as normal EOF.
Recovery follows capture shutdown, not the reverse.

If forced interruption is necessary, label the tail incomplete. A prior
fully covered window can remain useful if its loss boundaries are known;
missing final stats do not certify the entire run. Recovery requires actual
file comparisons and tracing absence after reboot, not the recovery marker.

Acceptance requires: no kernel drops/overruns, no unexplained parse or stream
sequence errors, complete footer, reconciled emitted/received totals and
explicit lifecycle coverage. Tolerated censoring must be classified, not
hidden inside a successful exit code. Also verify intended image/module
hashes, rate mode, AP identity and observer coverage before deployment.

## Qualification and bounded experiment

1. Fixtures for exact deployed trace formats, all event types and signed MMC
   errors; ensure arbitrary extra/raw fields never reach output.
2. Matcher fixtures: cookie reuse, requeue old→new, concurrent requests,
   confirmation/write-return overlap, unmatched starts/ends, cancellation
   ambiguity, cross-CPU ordering, loss boundaries and truncated batches.
   Use known latency distributions to verify every observation is represented.
3. Replay synthetic peak-rate streams through framing/drain and analysis;
   exercise slow consumers, output caps, corrupt batches and graceful/forced
   shutdown. No hardware needed for these tests.
4. Short vendor hardware actual-hit qualification, including final drain and
   recovery. Check observed rates/records against existing driver totals.
5. Establish capture overhead using adjacent matched repeats with old versus
   full-event collection (same static module and AP probes), reverse the order,
   and compare throughput, CPU use, socket behavior and failure fractions.
   Also retain a tracing-disabled reference to measure common static/AP
   observer overhead. Do not call a collector neutral based on one fast run.
6. Once qualified, two matched vendor/Rust pairs in opposite order, same auto
   rate settings and600s guard. Read the full timing report before expanding
   instrumentation. If variability dominates, report that rather than endlessly
   sweeping new firmware candidates.

Deliverables: versioned projection collector, separate socket sampler,
graceful-stop integration, offline episode analyzer with focused fixtures,
private full-event artifacts and one comparison report with coverage, timing
strata and justified next boundary. These are diagnostic tools, not firmware
fixes. No live deployment is part of this design-only step.
