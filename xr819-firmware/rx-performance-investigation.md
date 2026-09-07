# RX/performance investigation after key-selection closure

## Current vendor control

The stable Intel AX200 AP was tested with stock vendor firmware, the stock
vendor boot image, and the previously qualified vendor host module. Tests
isolate XR819 Wi-Fi in `wifi-test`, leaving Ethernet for management. Forward
iperf means AP-to-board RX; reverse means board-to-AP TX. Each phase lasts
20 seconds; TCP uses the default window and UDP requests `-b 20M` (iperf2
reports about 21.0 decimal Mbit/s offered).

| Direction | TCP | UDP received | UDP loss |
| --- | ---: | ---: | ---: |
| AP to XR819 | 19.6 Mbit/s | 21.0 Mbit/s | 0/35668 |
| XR819 to AP | 11.3 Mbit/s | 12.4 Mbit/s | 129/21338 (0.6%) |

Initial ping was 49/50; final ping was 50/50. Each phase ended with the BH
alive, WSM idle and zero used buffers. Rate adaptation was unrestricted.
This control rules out a general inability of the current AP/board/RF setup
to achieve the historical throughput; it does not yet isolate firmware from
host-driver differences.

Inputs:

- firmware SHA-256: `3e2462d476c9dfcb907cda1ba81d0a6d1bbee5e3207bdc1911ec5042d96fdfca`;
- boot SHA-256: `6583350b3eb12f70fc6d6081426717bd0019b55c6558ffe820c1548f0702bb8c`;
- module SHA-256: `758a28dcfbe9a9167937834bf2fd9570147702a61f8cc364811edd712decddff`;
- host kernel/AP: 7.2.1, iwlwifi, AX200 firmware `77.aa2dd297.0`;
- log: `/tmp/xr819-vendor-current-bidirectional.log`;
- harness: `/tmp/xr819-intel-bidirectional-run.sh`.

## Discriminators

The current open image ran the identical directional matrix with its existing
aggregate-feedback host module and fixed MCS5. It reached 21.2 Mbit/s in the
last scheduled five-second AP-to-board TCP interval, then stalled with three
TX confirmations outstanding. The host logged repeated missed interrupts,
`Timeout waiting for TX confirm`, and a fatal BH error. The final TCP total
includes another roughly twenty seconds of stalled shutdown and is not a
healthy throughput measurement. All subsequent phases are invalid.

The status text misleadingly still says BH alive, but includes `BH errcode: 1`
and three used buffers. The original harness did not fail its shell exit code
on this state. The revised harness now aborts at the first failed phase and
captures firmware RAM before recovery, rather than running more traffic into
a dead datapath. A reproduction is running with that postmortem capture.

The earlier key-selection validation exercised primarily board TX, not bulk
board RX. Do not call its reverse TCP result an RX throughput baseline.
Current failing log: `/tmp/xr819-open-current-bidirectional.log`.

The open TX result also needs accounting reconciliation: previous runs
reported almost every aggregate member acknowledged, but the AP accepted far
fewer data packets. An on-air BlockAck is not proof of delivery past
CCMP/replay/reorder validation. The source's RX BA consumer matches member
sequence numbers, while Intel's receive path additionally validates packet
numbers before delivering data. Whether those checks explain these losses is
still unproven.

If the current open image underperforms, test an exact saved open image that
previously delivered about 15 Mbit/s UDP before changing scheduling or RF
code. This distinguishes a recent diagnostic/layout regression from behavior
shared by the older firmware. Keep host-module differences as a separate
control, rather than attributing the complete vendor/open delta to firmware.

## Follow-up controls

A repeat of the current open image did not stall. It received 19.9 Mbit/s UDP
with zero loss, but delivered only 3.47 Mbit/s reverse TCP and 7.84 Mbit/s
reverse UDP with 56% loss. All firmware decrypt-drop counters remained zero.
This distinguishes good bulk RX capacity from a substantial outbound loss
problem, without dismissing the separately observed intermittent stall.

The exact saved auth-signature image that previously delivered 14.9 Mbit/s UDP
also performs poorly now: 3.86 Mbit/s reverse TCP, 6.70 Mbit/s reverse UDP and
58% loss. It stayed alive and drained. Thus the severe outbound loss is not
introduced by the RX key-selection fix or the final auth-class counter edit.
Its forward TCP/UDP were 6.59/8.24 Mbit/s, showing additional run variability.
Log: `/tmp/xr819-open-historical-bidirectional.log`.

The non-aggregating control retained fast-loop and RX diagnostics but disabled
the existing aggregation features. It stayed alive, received 17.1 Mbit/s UDP
with zero loss, but transmitted only 1.93 Mbit/s UDP with 76% loss (reverse
TCP 3.80 Mbit/s). Thus loss is not exclusive to A-MPDU assembly. Common TX
ordering, encryption, hardware completion interpretation and PHY behavior
remain candidates. Log: `/tmp/xr819-no-aggregation-bidirectional.log`.

## PAS-head ordering hypothesis

The four-slot host scheduler initially chooses the lowest eligible context
index. Its existing PAS-head override was restricted to contexts that already
had a retry attempt. Context slots are recycled independently of PAS order,
so a new low-index context can start an aggregate ahead of an older high-index
context. The list-first publisher searches actual ring slots and detaches the
selected members, but does not itself require the selected head to be the
oldest queued frame. On-air acknowledgement can precede receiver replay or
reorder rejection, so clean completion counts do not rule out ordering loss.

A narrow candidate applies the existing PAS-head override to first attempts
as well as retries. It preserves the existing candidate-prefix exclusion and
ordinary-plan preconditions. The candidate passed build/layout checks and
stayed alive, but still lost 66% of outbound UDP (4.65 Mbit/s); reverse TCP
was 3.56 Mbit/s. Forward UDP was 17.2 Mbit/s with zero loss. It did not fix
the measured failure and was reverted rather than retained as an unqualified
scheduler change. Log: `/tmp/xr819-pas-head-order-bidirectional.log`.

The vendor control with the exact open host-driver module and fixed MCS5
remained healthy: forward TCP 23.0 Mbit/s, forward UDP 21.0 Mbit/s with one
lost datagram out of 35668, reverse TCP 11.8 Mbit/s, reverse UDP 12.1 Mbit/s
with 1.2% loss. All phases drained. This closes the host-module/rate-mask gap;
the severe outbound loss remains specific to the open firmware.
Log: `/tmp/xr819-vendor-matched-driver-mcs5.log`.

AP-side kernel drop tracing has been requested: local sudo requires a password.
The helper `/tmp/xr819-ap-drop-trace.sh` records drop location/reason counts,
not packet payloads or key material.

A temporary `experimental-software-tx-ccmp` discriminator routed network TX
through software CCM, leaving RX and the hardware AES known-answer test
unchanged. The first build exceeded the stack gate (7344/6912 bytes) and was
not deployed. An encryption-only AES schedule did not reduce that bound.
A temporary foreground-only static schedule with separate initialization
lowered it to 6512/6912 bytes; this image passed packing/layout checks.

Software TX still lost 38% at 2M offered, 69% at 5M, 61% at 10M and 65% at
20M; delivered throughput stayed between 0.820 and 1.29 Mbit/s. This does not
support an accelerator-only encryption defect. Software encryption also
slows admission/transmission, so throughput differences are not evidence of
a fix. The temporary feature, static schedule and AES alias were all removed.
Log: `/tmp/xr819-software-tx-load-sweep.log`.

The unchanged hardware-encryption image's outbound offered-load sweep shows
a sharp load dependence: 2M offered delivers 2.10 Mbit/s with 1/1786 lost;
5M offered delivers only 0.459 Mbit/s with 91% loss; 10M and 20M offered lose
87%. Both images used the same offered-load sweep to avoid mistaking
software-induced throttling for corrected encryption.
Log: `/tmp/xr819-hardware-tx-load-sweep.log`.

The read-only review found ordinary completion gating and retained-request
lifetime consistent with vendor. Two concrete investigation targets remain:

- `service_pipe_tx_success` can complete through expected status `0xff`
  without receiving a response. That is vendor behavior, but counting it as
  an actual ACK would overstate the delivery evidence.
- Rust watchdog recovery retires only the current slot before making the pipe
  inactive, without proving hardware quiescence or walking the remaining
  batch. Vendor `txp_fn_4155` (0x101f4) walks the batch. This is a plausible
  three-confirmation stall mechanism, not yet an explanation of sustained loss.

The next temporary probe uses the existing diagnostic feature, with no policy
change. MIB words 0..10 now contain: matched-response completions, no-response
success events, complete-BA retirements, watchdog expiries, publications, cursor
mismatches, last mismatched packed cursor, last no-response control flags,
last no-response FC, last mismatched raw hardware ring word, and last publication
packed cursor. Words 11..21 retain RX/authentication diagnostics. Completion
counts are event counts, not uniformly frame counts (BA retirement covers a
batch). Cursor mismatch records include pipe state; an active append must not
be mistaken for the vendor's inactive first-GO invariant.

Probe image: `/tmp/xr819-tx-completion.bin`, SHA256
`c1a2a01f2ba85d2a0a8bb92d872fc1647c1a57072c971dcdbe611ba3dc788b7e`.
The sweep returns to 2M after the high-load phases to test persistent versus
load-local damage. Its postmortem harness detects BH failure before further
traffic and uses this ELF's exception address 0x1a240, foreground dump
0x16000..0x1a400, and DTCM 0x04000000..0x0400c000.
The first two probe runs did not complete WPA authentication within 60 seconds,
including after restarting the AP; neither yielded throughput results. The BH
remained alive and all request buffers drained. Logs:
`/tmp/xr819-tx-completion-source-sweep{,-retry}.log`.

The unchanged baseline then passed the same harness, but itself needed 30 seconds
and one authentication timeout before connecting. Loss at 2/5/10/20M was
0.11%/23%/29%/42%; returning to 2M lost 133 packets in the first five seconds,
then zero in the last five. Thus the high-load failure need not persist at low
load, and absolute loss still varies substantially between runs. Log:
`/tmp/xr819-baseline-association-control.log`.

The unchanged probe is being retried with a 120-second authentication window,
WPA debug logging (without key logging), and MIB capture on authentication
failure. This is to distinguish a probe regression from variable setup, not to
count failed authentication attempts as performance measurements.
Log: `/tmp/xr819-tx-completion-wpa-debug.log`.

That unchanged probe connected in 8 seconds and completed the sweep. Loss was
0%/1.5%/41%/48%/0% at 2/5/10/20/2M. During all traffic phases the dedicated
watchdog and cursor-mismatch counters stayed zero, and no-response successes
stayed at their idle count of 12 (last FC 0x0040, a probe request). Response
completion deltas were 1786/4124/3936/2947/1786; complete-BA retirement deltas
were 0/69/643/494/0. This excludes those observed watchdog/cursor/no-response
paths as explanations for this sustained-loss run. It does not establish
which encrypted payloads passed the AP's replay/authentication checks.

The completion probe was removed after this comparison. Its replacement
checks unicast protected data on interface 0, separately for each QoS TID and
non-QoS traffic. Immediately before publication it follows the actual aggregate
member chain and compares each CCMP PN to the highest previously published PN.
It only observes software publication order, not actual on-air order or hidden
hardware retransmissions. MIB words 0..10 now mean: protected frames observed,
PNs at/below high-water, such violations without retry, such violations with
retry, previous PN low32, offending PN low32, previous/current sequence numbers
packed high/low16, offending control flags, TID plus CCMP key byte shifted 8,
equal-PN violations, and last PN low32. RX counters 11..21 are unchanged.
Build artifacts: `/tmp/xr819-tx-pn-order.{elf,bin}`.

The PN probe completed: loss 0%/1.4%/21%/32%/0% at 2/5/10/20/2M.
First-publication PN high-water violations increased by 0/104/2797/6584/6;
retry violations by 0/0/32/191/0. Two violations already existed at idle.
Example: PN 29456 published after 29458, with increasing-PN reference sequence
785 versus offending sequence 783, same TID 0, and no retry marker. This proves
software publication reordering, but not the AP drop reason or exact loss count.
Log: `/tmp/xr819-tx-pn-order-sweep.log`.

Source inspection found two independent ordering hazards: round-robin context
service can remove any pending member and append it to PAS, and publication
starts from the lowest reusable context index. Even the ordinary batch planner
walks context indices, not FIFO order. The earlier PAS-head-only experiment
could not correct ordering already lost during pending-to-PAS transfer, nor
necessarily order the entire ordinary batch.

A two-stage probe now measures PAS insertion and publication separately before
choosing the fix. Words 0..2 count PAS frames/nonretry violations/retry violations;
3..5 count the corresponding publication events; 6/7 are previous/current PN;
8 packs previous/current sequence; 9 packs TID, key byte, and stage (0 PAS,
1 publication) in successive bytes; 10 is offending control flags. All PNs in
the exported diagnostics are low32. This remains instrumentation only.
Image `/tmp/xr819-tx-order-stages.bin`; log
`/tmp/xr819-tx-order-stages-sweep.log`. Postmortem exception: 0x1a5a8.

The two-stage run localized most reordering to publication. At 2/5/10/20/2M,
nonretry inversion deltas entering PAS were 0/0/8/116/5, versus 1/82/2954/6429/171
at publication. Loss was 0%/2.9%/20%/34%/20% (the last five seconds at 2M
were loss-free). Pending service is a real but much smaller contributor.

The first candidate fix orders all publication candidates by forward distance
from the PAS head, not reusable context index. This selects the aggregate head
and supplies an explicit FIFO index order to the ordinary-batch planner; the
planner no longer silently reorders remaining members by their arena indices.
A focused test covers reused indices [3,1,2,0,4], interleaved pipes, physical
slot wrap, and invalid duplicate/out-of-range order entries. The earlier
head-only attempt did not enforce this whole-batch invariant. The two-stage
probe remains temporarily enabled to measure whether publication now preserves
PAS order. Upstream pending reordering is deliberately not changed in this
candidate so its effect can be measured separately. Artifacts:
`/tmp/xr819-pas-fifo.{elf,bin}`.

The candidate removed publication-added first-transmission PN inversions:
PAS/publication inversion deltas were both 0/0/0/72/0 at 2/5/10/20/2M.
However, loss remained 0%/7.4%/42%/58%/0.95%. The 10M phase therefore lost
42% despite zero nonretry PN inversions at either stage. This corrects the
ordering invariant but does not validate it as the throughput/reliability fix.
Log: `/tmp/xr819-pas-fifo-publication-sweep.log`; connection took 66 seconds.

The candidate and all temporary order probes were archived together in
`/tmp/xr819-pas-fifo-with-order-probes.patch` and reverted from the working
source rather than retained as an unsuccessful performance change. The
qualified RX key-selection fix remains untouched. No new checkpoint was made.
The immutable candidate ELF/image remain available for controlled AP-side
comparison.

## AP-side drop trace: receiver reorder-release path

The user started `/tmp/xr819-ap-drop-trace.sh`. A timestamped sweep with
20-second quiet intervals compared the immutable FIFO candidate with stock
vendor, using the same diagnostic host module and fixed MCS5:

| Offered TX load | Open loss | Vendor loss | Open reorder-release drops | Vendor reorder-release drops |
| --- | --- | --- | --- | --- |
| 10M | 72% (6444/8918) | 0.34% (30/8919) | 5246 | 0 |
| 20M | 53% (5644/10617) | 0.35% (40/11548) | 5191 | 0 |

Open windows: 11:34:07–11:34:20 and 11:34:46–11:34:57. Vendor windows:
11:40:32–11:40:43 and 11:41:08–11:41:19. Counts sum the overlapping 10-second
trace bins, not exact per-packet attribution. Both images also generate drops
inside `iwl_mvm_rx_mpdu_mq`; vendor produces many of those while delivering
nearly all datagrams, so generic Intel-drop totals alone are misleading.
The strong discriminator is `iwl_mvm_release_frames`.

Verified Linux v7.2.1 `drivers/net/wireless/intel/iwlwifi/mvm/rxmq.c` against
the running workstation version. Its reorder release calls an inlined
`iwl_mvm_check_pn` before passing the packet to mac80211. The actual loaded
module object has one drop call in this function, reached by invalid station,
missing key, invalid TID, or rejected PN; the exact PN comparison still needs
measurement before claiming which condition caused the burst.

Read-only disassembly `/tmp/xr819-ap-release-disassembly.log` identifies
`iwl_mvm_release_frames+0x201` as its `memcmp` call: RDI/RSI point at current
and stored six-byte big-endian PNs. The narrower helper
`/tmp/xr819-ap-pn-trace.sh` checks kernel 7.2.1, x86_64, and exact compressed
module SHA256 before attaching there. It filters the lab transmitter and
samples only PN, sequence, queue, TID and flags, not payloads or keys. Root is
needed even to compile it on this system. The user subsequently started it;
it attached successfully. The histogram signedness warning is benign for the
nonnegative differences of these 48-bit PNs.

Logs: `/tmp/xr819-ap-trace-open-fifo.log`,
`/tmp/xr819-ap-trace-vendor.log`, `/tmp/xr819-ap-drop-trace.log`.
Source consulted: <https://raw.githubusercontent.com/gregkh/linux/v7.2.1/drivers/net/wireless/intel/iwlwifi/mvm/rxmq.c>

The PN comparison trace confirmed replay rejection, not merely an ambiguous
Intel drop site. For example, queue 7/TID 0 released sequence 1970/PN 1971
while stored PN was already 2034, then rejected the successive older PNs.
Other bursts start 63 below stored PN as well (4608 vs 4671; 6305 vs 6368;
15205 vs 15268). Samples have decrypted status and largely no Retry bit.
The recorded sequence/PN relationship remains consistent across sequence wrap.
At 10M, 6163 of 6404 reorder-release PN comparisons failed; application loss
was 6419/8919 (72%). This points to interaction with a 64-entry receive reorder
window, but does not yet prove whether transmitter behavior or the AP's window
advancement produces the inversion at release.
Saved trace: `/tmp/xr819-ap-pn-trace-open-fifo.log`; traffic:
`/tmp/xr819-ap-pn-open-fifo.log`.

A new control refuses `IEEE80211_AMPDU_TX_START` in a copied diagnostic host
driver (`/tmp/xr819-no-tx-ba-driver/sta.c`) rather than changing firmware
aggregation features. It retains HT, MCS5, CCMP and RX aggregation, but prevents
negotiation of a TX BA session with the AP. This is distinct from the earlier
no-aggregation firmware test, which did not prevent host-driven BA negotiation.
Only that return value changes; the original diagnostic module is untouched.
The rebuilt module SHA256 is
`d7d9f42188e40942314c2e19ee8befbf479315d94f099834530e519079b9b9c2`.
It is running with the same immutable FIFO candidate and includes board
`agg_status` snapshots to verify session state. This is diagnostic, not a
proposal to disable aggregation permanently or bypass replay protection.
Log: `/tmp/xr819-no-tx-ba-session.log`.

That control completed with aggregate TX count zero throughout and no hits on
the reorder-release PN probe during traffic. `agg_status` files were not
present in the snapshots, so no session-table evidence was obtained. Loss
nevertheless remained 0%/6.7%/37%/43%/3% at 2/5/10/20/2M. This does not support
removing TX BA as the fix. It also does not establish that the remaining loss
has the same drop reason: the first PN probe covers only reorder release.

The exact module has a second inlined PN comparison at
`iwl_mvm_rx_mpdu_mq+0xc74`, verified in
`/tmp/xr819-ap-mpdu-disassembly.log`. A combined tracer is prepared at
`/tmp/xr819-ap-pn-both-trace.sh`: path 0 is reorder release, path 1 direct RX.
It samples forward PN jumps as well as rejected PNs, with separate counters
for each path/queue/TID and the same exact-module guard. The user started it
successfully, and the no-TX-BA control was repeated.

That repeat confirms direct-path PN rejection as well: PN 1851 was accepted
before PN 1850, then PN 1853 before 1852. Most sampled rejected frames have no
Retry bit, with PN and sequence both one behind the accepted successor. In
the 5M phase PAS and publication counters each recorded 173 first-transmission
inversions; the AP recorded 172 direct-path PN rejects. The total application
loss was larger (926/4461), so these inversions do not account for every loss.
Logs: `/tmp/xr819-direct-pn-no-tx-ba.log` and preserved
`/tmp/xr819-ap-pn-both-no-ba-control.log`.

The FIFO publication candidate and two-stage probes were restored from the
archive for an upstream ordering fix. Pending service now uses explicit
admission tickets rather than a rotating arena index. Each eligible pending
owner is visited at most once per bounded pass, preserving hardware-first
service and existing pending eligibility decisions. Already-PAS-queued owners
no longer consume no-op software-service budget. Tickets stay with retained
owners through requeue; wrap/reused-index selection has a focused unit test.
This does not yet guarantee ordering when an older owner remains ineligible
while a younger one becomes eligible; that remains subject to measured gates.

The candidate passed 300 default / 301 diagnostic tests, stack 6736/6912,
packing and layout checks. It is first being compared with the same no-TX-BA
control and AP tracer before restoring ordinary BA negotiation for validation.
Artifacts `/tmp/xr819-pending-fifo.{elf,bin}`; exception 0x1a8d8; log
`/tmp/xr819-pending-fifo-no-ba.log`. That attempt failed authentication before
traffic. After resetting the AP, the unchanged candidate connected and completed
`/tmp/xr819-pending-fifo-no-ba-retry.log`.

The completed no-TX-BA run had zero nonretry inversions at both firmware stages
and zero PN rejects in either traced AP path. The 2M and 5M phases were
loss-free. At 10M, iperf sent 6013 datagrams, exactly 3589 reached PAS and AP
PN checking, and 2424 were missing at iperf. At 20M, 6008 were sent and 3559
reached PAS, versus 2450 reported lost (one-packet difference). Thus the large
remaining overload loss in this control is before PAS, not receiver PN rejection
of published packets. Saved trace:
`/tmp/xr819-ap-pn-pending-fifo-no-ba.log` (traffic 13:34:33–13:37:07).

Normal TX BA negotiation is now restored by using the original diagnostic
host module with the same candidate image. This validation must succeed before
considering the intended aggregation configuration fixed.
Log: `/tmp/xr819-pending-fifo-normal-ba.log`.

## Remaining normal-BA failure after first-transmission ordering fixes

Normal BA still loses 67% at 5M and 84% at both 10M and 20M. Firmware
first-transmission PN inversion counts remain zero at both stages. The AP
records no direct-path PN rejects during these phases, but thousands in
reorder release. One particularly useful trace sequence is:

- release accepts sequence 1889 / PN 1890;
- release next accepts sequence 1954 / PN 1955 (a jump of 65);
- release immediately rejects sequence 1891 / PN 1892 and the following
  older buffered run against the new high-water mark 1955.

Both acceptance and rejection occur in path 0, not an unobserved direct-path
update. This is consistent with a newly enqueued frame aliasing an old hole
in the AP's modulo-64 reorder storage before that storage is released, but
receiver head/NSSN values and the transmitter's recovery behavior still need
verification. Do not call the overall issue fixed or bypass replay checks.
Saved trace: `/tmp/xr819-ap-pn-pending-fifo-normal-ba.log`.

A read-only delegated review is tracing vendor TX BA window management, retry
fencing and BAR generation after missing/abandoned frames. Parent remains sole
writer and hardware operator. No checkpoint has been made.

The review found vendor retry fencing (link states 6/7/8/10 refuse ordinary
admission) and reverse traversal/front insertion of surviving retries. Rust
already intends reverse/front reinsertion. Outgoing BAR recovery was not
established from the available vendor bodies; incoming BAR handling is not
proof of transmission. Its suggested outside-window retry discriminator is
already enabled for deeper aggregates in this build: aggregate-rate-feedback
pulls in member-requeue, which pulls in outside-window-retry. This must not be
presented as a newly discovered missing feature.

Prepared `/tmp/xr819-ap-pn-window-trace.sh` and `.bt` for the next discriminator.
It retains both PN hooks and exact-module guards, adding reorder head, target
NSSN, buffer size, stored count and the actual release-loop slot sequence.
All offsets were checked in the loaded module disassembly: r12 is the reorder
buffer, stack+0x30 holds BA data, stack+4 the target NSSN, and stack+0x38 the
incremented loop sequence. Mismatches between the packet sequence and logical
release slot are counted with bounded samples. This can distinguish a modulo
ring alias at release from merely seeing a jump in packet PNs. No payload/key
capture or runtime mutation is added. Shell syntax checked; BPF compilation
and attachment require user root launch. No new firmware image is needed.

## Receiver slot alias confirmed

The user attached the window tracer successfully. The unchanged candidate's
normal-BA sweep completed with 37%/59%/77% loss at 5M/10M/20M and zero
first-transmission PN inversions at either firmware stage.

The trace establishes the slot mismatch directly, not just by inference from
PN distances: head=1877, NSSN=1878, buffer size=64, logical release slot=1877,
but the dequeued packet is sequence 1941 / PN 1942. Subsequent releases also
consume packets 1942 and 1943 from slots 1878 and 1879. Older buffered packets
1880 onward are then rejected against the prematurely advanced PN high-water
mark. The first alias has 58 stored packets. This proves the modulo-ring
alias at the receiver's release site; it does not yet explain what vendor
transmit behavior avoids exposing it.

Traffic: `/tmp/xr819-pending-fifo-window-trace.log` (14:15:56–14:18:31).
Saved trace: `/tmp/xr819-ap-pn-window-open-fifo.log`.
The matched stock-vendor sweep with the same tracer and original diagnostic
module completed: `/tmp/xr819-vendor-window-trace.log`. Vendor loss was 0.35%
at 10M and 1.4% at 20M, with no slot aliases or reorder-release PN checks.
All observed vendor PN comparisons were direct-path. There were direct-path
rejects, largely sampled retransmissions, despite healthy application delivery.
Thus total PN rejects are not equivalent to lost application datagrams, and
this control does not establish vendor handling of the same buffered BA-window
conditions. Saved trace: `/tmp/xr819-ap-pn-window-vendor.log`.

The host driver's OPERATIONAL callback returns the private TX-BA MIB write's
status. A copied driver in `/tmp/xr819-ba-session-diag-driver` now logs each BA
action, TID, SSN, buffer size, and that command's result without changing policy.
Its module build is underway; use it to verify actual setup/teardown rather
than assuming identical modules imply identical firmware/session behavior.
The standalone LSP lacks kernel include configuration; the external kernel
module build is the authoritative compile check. That build passed without
compiler warnings. Module SHA256:
`a237a79ed4c141ec0a3d66a961aba1cef183c4e3e8a89f110bf672b78b6f0626`.

## Vendor host-managed TX BA does not become operational

The short 5M vendor check completed with 2/4461 datagrams lost (0.045%).
Its BA callback log contains only action 2 (TX_START) followed about one second
later by action 3 (TX_STOP_CONT), repeatedly. There is no action 6
(TX_OPERATIONAL) or private operational-MIB call. Enum values were verified
against the board module's kernel headers. Therefore the earlier healthy
vendor controls do not establish healthy *host-managed operational TX BA*;
identical host modules did not produce equivalent session states. This also
means private-MIB rejection is not established as the cause: that command was
never reached in this vendor run. Autonomous vendor aggregation is a separate
question not settled by these callbacks alone.

Log: `/tmp/xr819-vendor-ba-session.log`. The identical short check completed
on the pending+publication FIFO Rust candidate with the same diagnostic module:
`/tmp/xr819-open-ba-session.log`. Rust logs TX_START, then TX_OPERATIONAL with
buf_size=64 and private-MIB result=0. It loses 1500/4461 datagrams (34%) at 5M.
This confirms the host-managed session-state difference rather than inferring
it from absence of receiver buffering. The AP window tracer remains active;
saved session-check trace: `/tmp/xr819-ap-window-ba-session-checks.log`.

A synthetic, hardware-free reproducer at
`/tmp/xr819-ap-reorder-alias-repro.py` models the observed first three holes
and a buffered run. Existing enqueue-before-release order accepts four newer
frames and PN-rejects 61 older ones. Releasing slots evicted by an incoming
out-of-window sequence before enqueue accepts all 65 with the *same* PN
validation and arrivals. This supports a controlled AP-driver preflush test,
not a claim that a kernel fix has been compiled or validated. Reloading a
patched workstation Wi-Fi module requires user approval/root assistance.
No workstation kernel/module changes had been made at that point.

The user approved a controlled AP-module test. A temporary `iwlmvm.ko` was
built from Linux 7.2.1 source against the running kernel's exact Nix dev output
(`/nix/store/80i6v9rsm4qasls9g0fiywc2nigv43xj-linux-7.2.1-dev`) using its GCC
15.3.0 compiler. The patch preflushes evicted slots before enqueue and clamps
an older local NSSN so it cannot undo that advance. PN validation is unchanged.
Patch: `/tmp/xr819-ap-reorder-preflush.patch`; build log:
`/tmp/xr819-ap-preflush-build.log`. Kernel vermagic matches. Optional module
BTF was skipped because vmlinux was unavailable at the build's expected path;
there is no claim of symbol-CRC validation (this kernel has no modversions).
Candidate SHA256:
`8139310b16907c687c2585856ef0f0f0f5588478a80cf3ae01ce65764d05b957`.

`/tmp/xr819-ap-preflush-control.sh apply` is prepared for user root launch.
It validates kernel/module hashes, stages root-only copies, stops known
bpftrace instances before unloading the module, and refuses unknown tracers.
It reloads only iwlmvm, restores the lab AP, and attempts stock recovery on
failure/interruption. `restore` explicitly returns to the stock module.
Installed Nix kernel files remain untouched; reboot also restores stock.
The script passed bash syntax and shellcheck checks. Initial application
encountered a NetworkManager readiness race after interface recreation and
returned to stock. A bounded activation retry fixed the swap procedure. The
patched AP module is now loaded, verified by its live sysfs build ID
`ac263b6586ba1a801696b8764f74fda11ebb2ac6` (stock:
`0b66948ba9965f0fd3140c95b0e686aad9f95750`). Old offset probes are stopped and
must not be restarted on this module.

The full 2M/5M/10M/20M/2M Rust sweep is running against the patched AP with
unchanged pending+publication FIFO firmware, MCS5, and the policy-preserving
BA-session diagnostic host module. Log: `/tmp/xr819-open-patched-ap.log`.
The first patched-AP sweep completed with TX BA operational, window 64 and
private-MIB result=0. At 5M, loss fell to 3/4461 (0.067%); at 10M it was
158/7984 (2.0%); at 20M it was 223/13516 (1.6%), delivering 15.2 Mbit/s.
Both firmware first-transmission inversion counters stayed zero. This is a
large improvement over the stock-AP BA-session check (34% loss at 5M), but the
old offset tracer was absent on the patched module.

To control for tracing overhead, the stock AP module has been restored and
its live build ID verified. No bpftrace process is running. A short otherwise
matched 5M Rust check is running at `/tmp/xr819-stock-ap-no-trace.log`.
The tracer-free stock check completed with TX BA operational at window 64
and 704/4461 datagrams lost (16%) at 5M. Thus tracer overhead is not required
for the severe loss. The patched AP has been restored and its live build ID
verified again. A full repeat is running at
`/tmp/xr819-open-patched-ap-repeat.log`, with the same firmware, diagnostic
host module and MCS5, still without tracing. That repeat completed with loss
2.0%/3.5%/2.5% at 5M/10M/20M and 14.1 Mbit/s delivered at 20M. TX BA was
operational with a 64-frame window, and all snapshots had zero used buffers.
The large improvement repeats, but remaining loss is not claimed solved.

## Probe-free qualification

The temporary PAS/publication PN probes and repurposed MIB words were removed.
The remaining firmware source changes are FIFO pending service and FIFO PAS
publication, including complete ordinary-batch ordering. The qualified probed
source diff is archived at `/tmp/xr819-pending-fifo-qualified-with-probes.patch`.
The AP patch is now tracked as `iwlwifi-reorder-preflush.patch` beside this
ledger (explicitly tracked because the repository ignores patch files).

The clean firmware passed 300 default / 301 diagnostic tests, stack 6736/6912,
packing and DTCM layout checks. ELF/BIN: `/tmp/xr819-clean-fifo.{elf,bin}`;
exception record 0x1a440, size 0x58. The board postmortem script was updated
before deployment. Primary LSP diagnostics on the three changed Rust files
reported no errors. No new experiment feature was added.

Hardware qualification is running with the original diagnostic host module,
patched AP and MCS5: `/tmp/xr819-clean-fifo-hardware.log`. It performs the full
TX UDP sweep, then RX TCP, RX UDP at 20M, and TX TCP, checking buffer/BH health
between phases and capturing RAM before recovery on a fatal BH condition.

That run completed with zero used buffers at all snapshots and final ping
50/50. TX UDP loss at 5M/10M/20M was 0.045%/0.75%/3.9%, delivering 12.0 Mbit/s
at 20M offered load. RX UDP delivered 15.8 Mbit/s with 0/28753 lost. However,
TCP averaged only 7.76 Mbit/s RX and 5.35 Mbit/s TX. AP snapshots show rate
fallback and increasing retries; this does not by itself explain the TCP
result or establish a cleanup regression.

Before clean-image signoff, the same full protocol is running on the immutable
probe-bearing FIFO image with the same original diagnostic host module and
patched AP: `/tmp/xr819-probed-fifo-bidirectional-control.log`. Its original
exception capture address was restored before deployment. No source probes
were reintroduced and no checkpoint has been made yet.

The probe-bearing control completed without a BH failure, with buffers drained
and final ping 50/50. TCP was also low: 3.16 Mbit/s RX and 2.78 Mbit/s TX;
RX UDP was 14.9 Mbit/s with zero loss. At 20M offered TX UDP it delivered
7.94 Mbit/s with 11% loss. These results do not support a regression caused
by removing the probes, but do show substantial residual variability.

A clean-image run now tests RX TCP immediately after association, omitting
the preceding TX UDP sweep, followed by RX UDP and TX TCP:
`/tmp/xr819-clean-fifo-fresh-tcp.log`. This checks load-history dependence;
it must not be conflated with proof that all TCP limitations are fixed.

The fresh-order run completed: RX TCP averaged 19.4 Mbit/s (reaching 25.6 in
one five-second interval), RX UDP delivered 16.4 Mbit/s without loss, but TX
TCP measured afterward remained 2.37 Mbit/s. Buffers drained and final ping
was 50/50. This shows that low RX TCP is not persistent in the clean image;
it does not yet separate TX TCP's own limitation from prior RX load.

The FIFO source changes and the receiver patch are being checkpointed as
separate, demonstrated improvements, not as a claim of complete performance
resolution. The clean firmware SHA256 is
`2e4437d388453f77a907834895ce7735b3ac11e535042d121e78e0c18838ff32`.
Temporary order probes are absent. TCP variability, sporadic association
delays, and the previously observed TX-confirmation stall remain open.
Next test: TX TCP first after association, before either bulk RX phase.
The DTCM redesign remains deferred while that performance work continues.
