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

The TX-first run completed with 4.73 Mbit/s TX TCP, followed by 18.2 Mbit/s
RX TCP and zero-loss RX UDP at 16.3 Mbit/s. Buffers drained and final ping
was 50/50. Prior bulk RX is therefore not required for the TX slowdown.
Log: `/tmp/xr819-clean-fifo-tx-first.log`.

Next, `/tmp/xr819-clean-tcp-sender-observation.log` runs one fresh 30-second
TX TCP test while sampling the board sender's `ss -tin` once per second over
Ethernet. Only port-5001 socket metadata is recorded: windows, RTT, retries,
bytes and timing, not payloads or keys. This is intended to distinguish
congestion/retransmission/ACK stalls from steady pipeline throughput limits.

The first sampler run saw cwnd 5–10 segments, increasing retransmissions
(148 by the last shown sample), RTT excursions, a persistently nonempty send
queue, and roughly 434–438 KiB advertised send window. Receiver-window-limited
time was negligible. But the actual sample spacing was 3–4 seconds because
every sample established another SSH connection. The 1.53 Mbit/s throughput
is potentially observer-disturbed and is not an unbiased performance result.

The sampler was replaced with one persistent SSH connection established
before starting traffic, with the one-second sampling loop on the board.
The otherwise identical repeat is running at
`/tmp/xr819-clean-tcp-persistent-observation.log`. No firmware change.

The persistent sampler collected 30 socket samples. TX throughput was
2.52 Mbit/s. By the last sample, 157 retransmitted segments were reported
among 6783 data segments sent, with cwnd repeatedly around 5–11 after its
initial growth. RTT excursions reached tens of milliseconds and RTO reached
520 ms; the send queue stayed backlogged and advertised receive capacity
remained hundreds of KiB. Repeated SSH handshakes are therefore not required
for these TCP symptoms.

The next diagnostic uses identical clean firmware, patched AP, MCS5 and
persistent sampler with the previously isolated no-TX-BA host module:
`/tmp/xr819-clean-tcp-no-ba-observation.log`. This tests whether operational
TX aggregation is needed for the residual retransmission pattern; it is not
a proposal to permanently disable aggregation.

The no-TX-BA control completed with aggregate TX zero and only three TCP
retransmitted segments among 11734 data segments at the final sample. Cwnd
grew to roughly 61, versus repeated collapse with BA. Throughput was
4.46 Mbit/s. This strongly associates the residual loss with the aggregation
path rather than a receive-window limit, although it does not locate the drop.

Prepared a new bounded AP observer for the patched module:
`/tmp/xr819-ap-patched-pn-trace.sh`. It checks both candidate SHA256 and the
live module build ID. The release instructions match stock (only a diagnostic
string relocation differs), and both PN-comparison offsets/register layouts
were revalidated in the patched disassembly. The observer retains direct and
reorder PN checks plus slot-mismatch metadata; no payloads or keys. It will
expire after 420 seconds. This is not an unguarded reuse of stock-module probes.

The correlated TCP run completed at 1.32 Mbit/s with 155 sender retransmissions
by the last shown sample. AP probes counted 3495 PN comparisons during traffic
across direct and reorder paths, with zero PN rejects or slot mismatches.
The host TX-failure counter increased only from 1 to 3. These counters have
different scopes and are not a one-to-one packet proof, but they justify
investigating silent loss before PN validation and aggregate success accounting.
Logs: `/tmp/xr819-clean-tcp-pn-correlated.log` and
`/tmp/xr819-ap-patched-pn-trace.log`. The observer then expired normally
(exit 124 from its deliberate 420-second timeout).

A read-only child is reviewing aggregate acknowledgment freshness/identity
and member retirement; parent remains sole writer/hardware operator. A new
bounded AP observer additionally counts existing DROP-level diagnostic format
strings (radio-wide, without formatting arguments) and lab packets reaching
the common skb drop site after their headers were copied. Empty-skb drops
remain explicitly unattributed. It retains the guarded PN/slot probes.
Scripts: `/tmp/xr819-ap-patched-drop-trace.{sh,bt}`. No payload/key capture.

The read-only review identified a concrete possible false-success route:
`plan_ordinary_tx_pipe_status` permits kind-1 completion when state/status gates
match; `service_txp_pipe_tx_status` then calls `complete_tx_pipe_slot(..., 0)`.
With the active depth-two feature, that function enqueues every host aggregate
member and returns without consulting the retained BA bitmap. Parent verified
these bodies. Occurrence during the failing workload is not yet established;
counting kind-1/status-zero generic completions is the minimal discriminator.

Other review findings: accepted BAs do not validate transmitter address or
compressed/single-TID format, and retained observations lack an enforced
receive/start freshness boundary. These are potential correctness gaps, not
measured causes. The sequence-minus-SSN bitmap arithmetic and selective
requeue interception did not reveal a polarity or false-success error.
Do not reverse BA bitmap polarity based on unreliable vendor decompilation
condition-code comments.

The correlated AP-drop run completed at 2.44 Mbit/s with 129 TCP
retransmissions by the last shown sample. No PN rejects, slot mismatches or
lab-attributed post-copy common drops were recorded. Radio-wide diagnostic
formats reported duplicate drops and BAR-release notifications, not crypto
errors; the empty-skb drop counts matched the duplicate-format counts in the
traffic bins. These scoped counters do not prove absence of every receive
loss. Logs: `/tmp/xr819-clean-tcp-drop-correlated.log` and
`/tmp/xr819-ap-patched-drop-trace.log`. The observer expired normally after
its bounded run.

A temporary aggregate-completion counter build is being qualified without
changing completion behavior. MIB words: 0 generic host kind-1/status-zero
calls; 1 successful host-member enqueues through that path; 2 nonzero-status
generic host aggregate calls; 3 ordinary-status kind-1 caller entries; 4 last
incoming/expected MAC status packed in low/next byte; 5 last generic status;
6 last first-frame-node address. Remaining probe words are zero; RX diagnostic
words 11–21 remain intact. Logs/artifacts: `/tmp/xr819-aggregate-completion*`.
The counter build passed 300/301 tests and stack/layout checks. Its TCP run
completed at 2.40 Mbit/s with 145 retransmissions by the last shown sample.
During traffic, generic successful aggregate calls increased by 1911,
successful member enqueues by 5577, and ordinary-status aggregate completions
by 1515. The last incoming/expected status was 0x0c/0x0c. Thus the suspected
ordinary-status route is heavily exercised; this alone does not prove which
individual completions lacked valid acknowledgment.

A candidate now returns `AwaitBlockAck` for matched host aggregate statuses,
before the slot-state write or global-busy suppression. It leaves slot state,
MAC ownership and cursors unchanged for existing BA and bounded retry/timeout
handling. Non-host and ordinary-frame policy is retained. A focused planner
test covers host/non-host, busy and unmatched-status cases. Probe word 7 counts
these deferrals; word 3 should no longer grow for host aggregates.

The candidate passed 301 default / 302 diagnostic tests, unchanged stack
6736/6912, packing and DTCM layout. ELF/BIN:
`/tmp/xr819-ba-owned-status.{elf,bin}`; exception 0x1a4e0, size 0x58. Updated
postmortem installed before testing. Hardware run:
`/tmp/xr819-ba-owned-status-tcp.log`. No improvement or checkpoint is claimed
until the TCP and retirement behavior is measured.

Rejected: the deferral run delivered only 58.7 kbit/s, with 36% initial and
62% final ping loss. During traffic, deferrals increased by 38, ordinary-status
aggregate completions stayed zero, generic successes increased by 19 and
nonzero-status completions by 29. Buffers eventually drained, but this is not
acceptable retirement behavior. The deferral and its test were removed;
source archive: `/tmp/xr819-ba-owned-status-rejected.patch`. Original completion
policy is restored with temporary observation counters retained.

Vendor `rxfifo_find_frame_by_subtype` at 0x8d80 scans from the TX-start cursor
to the RX producer for subtype 0x94; the Rust backend method is a `None` stub.
A new non-consuming probe tests whether a same-TID BA is present at ordinary
aggregate completion. It scans at most 16 slots, checking magic, stride,
producer bounds and packet-RAM bounds. It does not establish peer identity or
acknowledgment validity, and changes neither FIFO cursors nor completion policy.
MIB word 7 now counts scan hits; words 8–10 contain the last BA control/SSN
and bitmap. No payload or key capture. Artifacts: `/tmp/xr819-status-ba-scan*`.

## Completion-time BA lookup probes

The non-consuming scan passed 300/301 tests, stack 6744/6912, packing and
layout. Exception record 0x1a5b8/0x58 was installed in the postmortem script
before deployment. `/tmp/xr819-status-ba-scan-tcp.log` completed at 1.32 Mbit/s
with 165 sender retransmissions. Both ping checks were 50/50 and buffers
drained. During traffic, 702 ordinary aggregate completions included 281
same-TID scan hits. This establishes availability in some completions, not
identity or acknowledgment of the retiring members.

The next observational build additionally requires matching BA RA/interface
and TA/current TX receiver, plus single-TID compressed format. Word 7 counts
these stricter scan hits. Words 8/9/10 now count current aggregate members
classified acknowledged/missing/outside-window by that BA, rather than storing
the last BA header. Member walking is bounded and detects repeated nodes.
Completion policy and RX FIFO ownership remain unchanged. This probe still
does not prove BA freshness or account for acknowledgments in earlier BAs.
Artifacts: `/tmp/xr819-status-ba-members*`.

The stricter probe passed 300/301 tests, stack 6776/6912, packing and layout.
Postmortem exception address was updated to 0x1a6f8/0x58 before deployment;
the live patched AP build ID was verified. Hardware run completed at
1.26 Mbit/s with 156 sender retransmissions. Both ping checks were 50/50 and
buffers drained. During traffic, 649 ordinary aggregate completions included
245 matching BA scan hits; member classifications were 641 acknowledged,
16 missing and 22 outside-window. Those completions still took generic success.
This is an accounting discrepancy worth resolving, not proof that all 38
non-acknowledged members were falsely confirmed: earlier retained/sticky
acknowledgments and freshness are not included in this probe.

A read-only adversarial review now checks safe completion-time BA integration,
including early all-ACK observations cleared before slot state becomes 3/4,
selective member requeue and no-BA recovery. Parent remains sole writer and
hardware operator. No new behavior or checkpoint yet; do not fabricate retry
IRQ ownership to reuse the retry dispatcher.

The review recommends a terminal ordinary-completion hook: preserve state-5
and existing cursor retirement, but merge admissible BA evidence and resolve
members individually, software-requeueing unresolved members under bounded
per-member retry policy. Do not import state-4 command-mask cleanup or retry
IRQ writes. The selective planner currently excludes depth two; integration
must cover both two- and deeper-member batches. The global scan cursor does
not yet prove freshness for the current command, so the diagnostic scan must
not simply be promoted to acknowledgment authority.

Parent verified the early all-ACK evidence loss. A prerequisite candidate now
clears the retained observation only after actual completion, not before the
slot-state check. It does not change ordinary status disposition or use FIFO
scan results for completion. Probe word 6 now counts all-ACK observations that
cannot yet retire (instead of recording the last frame-node address). This
counts observations, not necessarily distinct aggregates. Build qualification:
`/tmp/xr819-retain-early-ba*`. No hardware improvement or checkpoint claimed.

The retention candidate passed 300/301 tests, stack 6776/6912, packing and
layout; postmortem exception 0x1a708/0x58 was installed before testing and
patched AP identity verified. `/tmp/xr819-retain-early-ba-tcp.log` completed
at 3.02 Mbit/s with 128 sender retransmissions, both ping checks 50/50 and
buffers drained. During traffic, word 6 increased by 609: the early all-ACK
retention case is exercised. There were 1840 ordinary aggregate completions,
1223 strict BA scan hits, and 3926/35/27 acknowledged/missing/outside-window
member observations. These counters are not a one-to-one packet correlation.

Throughput variability prevents attributing this run's improvement yet.
An immutable pre-retention control is running with otherwise the same harness:
`/tmp/xr819-early-ba-retention-control.log`. Its original 0x1a6f8 postmortem
is restored before deployment. Initial SCP met the harness's asynchronous
recovery reboot (connection refused); the managed control has bounded SSH
readiness retries and will not proceed without uploading the correct script.
The two images differ in early retention and word-6 instrumentation; this is
not a perfectly instrumentation-identical comparison. No checkpoint yet.

The control completed at 2.27 Mbit/s with 139 retransmissions, both ping checks
50/50 and buffers drained. During traffic: 1405 ordinary completions, 863 BA
scan hits and 2518/32/26 acknowledged/missing/outside-window members. The
candidate's 3.02 Mbit/s and 128 retransmissions remain inconclusive against
this variable baseline. A repeat of the immutable retention image is running:
`/tmp/xr819-retain-early-ba-repeat.log`, with its correct postmortem restored
and patched AP identity checked before deployment.

The retention repeat delivered 1.78 Mbit/s with 152 retransmissions; both ping
checks were 50/50 and buffers drained. Early all-ACK observations increased
by 429. There were 1054 ordinary completions, 573 scan hits, and 1608/39/21
acknowledged/missing/outside-window member observations. The candidate/control/
candidate sequence does not establish a TCP improvement from retention alone.

## Retained-BA ordinary completion candidate

The next candidate adds a terminal `TxStatusPolicy::complete_ordinary_status`
hook while preserving the ordinary state-5 transition and existing cursor
retirement. With an already-retained exact-batch BA, it checks current slot,
command and member publication identities, preflights software requeue capacity,
and uses the existing selective finisher. That planner now accepts depth two
as well as deeper aggregates; a focused policy test covers wrapped two/four
member batches with allowed/exhausted retries. Reverse retry insertion remains
unchanged. Failed integrity checks are terminal, never an all-success fallback.
No retry IRQ is fabricated or acknowledged, and no in-place rearm is added.

This is intentionally partial: without a retained observation, existing generic
completion remains unchanged. The FIFO scanner is still diagnostic-only, and
the RX consumer's existing BA identity/freshness limitations remain unresolved.
This candidate does not claim to eliminate all false-success routes.

Probe words 4/5 now count members software-requeued by this hook / hook batch
resolutions. Words 0–3 and 6–10 retain their previous meanings (word 6 counts
early all-ACK observations). Artifacts: `/tmp/xr819-retained-status*`. No
checkpoint until demonstrated improvement and probe-free qualification.

Build qualification passed 301/302 tests, stack 6800/6912, packing and layout;
postmortem exception is 0x1a8c8/0x58. The first hardware run initially sustained
5–6.64 Mbit/s, but collapsed near the end: overall 4.63 Mbit/s, 94 sender
retransmissions, final ping 0/50. Initial ping was 50/50. BH stayed alive,
station remained associated and buffers drained; these facts do not establish
link health. Hook counters recorded 2161 batch resolutions and 3066 software
member retries. This candidate is NOT qualified and must not be checkpointed.

The harness incorrectly returned success because `ping | tail` hid ping's
exit status. It now checks ping explicitly and captures metadata plus the
existing postmortem before recovery on total connectivity failure. The prior
run had already rebooted, so its failed-state RAM was not captured. Verbose
supplicant dumps were also inconsistent with metadata-only logging: debug
verbosity was removed, recovery output now selects state/event metadata, and
hexdump lines in `/tmp/xr819-retained-status-tcp.log` were redacted. SSH now
uses passwordless batch mode rather than an unnecessary legacy password.

The same immutable candidate is being reproduced with the corrected harness:
`/tmp/xr819-retained-status-failure-capture.log`. AP identity is guarded and
the matching postmortem is uploaded before deployment. No further firmware
behavior changes were made for this reproduction.

The reproduction completed at 5.13 Mbit/s with 105 retransmissions among
13640 sender data segments at the final sample. Both ping checks were 50/50
and buffers drained; the late connectivity failure did not recur, so no RAM
postmortem was triggered. Hook counters recorded 2147 batch resolutions and
2788 software member retries. This supports further testing, not qualification
or a claim that the earlier connectivity loss is resolved.

The same image is now running a 120-second transfer with failure capture:
`/tmp/xr819-retained-status-long-tcp.log`. The persistent metadata sampler now
accepts bounded `XR819_TCP_SECONDS` (default 30) and runs five extra samples;
no firmware changes. Patched AP identity and ELF-specific postmortem remain
guarded. No checkpoint.

The 120-second candidate run completed at 4.35 Mbit/s, with 443 retransmissions
among 45361 sender data segments at the final sample (approximately 0.98%).
Both ping checks were 50/50 and buffers drained. The prior total connectivity
loss did not recur; no postmortem was triggered. The hook resolved 7252 batches
and software-requeued 8701 members. These firmware counters have a different
scope from TCP segment statistics.

A duration-matched control now uses the immutable early-retention image without
the ordinary completion hook, with the same corrected harness and 120-second
sampler: `/tmp/xr819-retained-status-long-control.log`. Its postmortem address
0x1a708 is restored before deployment. This separates the hook from early
retention, though their diagnostic words 4/5 differ. The earlier unexplained
connectivity failure remains a qualification blocker; no checkpoint.

The duration-matched pre-hook control completed at 2.42 Mbit/s, with 604
retransmissions among 25819 sender data segments (approximately 2.34%). Both
ping checks were 50/50 and buffers drained. Compared with the candidate's
4.35 Mbit/s and approximately 0.98% retransmitted/sent ratio, this supports a
sustained benefit from retained-BA member resolution in this pair of runs.
The ratio is a TCP segment statistic, not measured radio packet loss, and a
single matched pair is not a reliability qualification. Control counters:
5837 ordinary completions, 2323 early all-ACK observations, 3367 scan hits,
and 9854/122/121 acknowledged/missing/outside-window member observations.

No performance-only repetition can explain the earlier 0/50 connectivity
failure. That failure still needs attribution/capture; the no-retained-BA
fallback and BA identity/freshness gaps also remain open. Temporary probes
and candidate changes remain uncheckpointed.

## Live TCP-stall capture

The metadata sampler now triggers after ten seconds of unchanged `bytes_acked`
while the sampled sender has unacked or unsent data and local iperf remains
running. Progress, idle queues and counter resets reset the timer. Ten synthetic
checks cover these cases without hardware access; Python and shell syntax
checks passed. This detects a backlogged TCP stall, not necessarily a radio
failure, and does not cover stalls before a sender socket is observed.

On trigger, `/tmp/xr819-capture-live-failure.sh` gathers AP neighbor/route and
station metadata, board neighbor/station/driver status and kernel messages,
then a short diagnostic ping and the existing ELF-specific RAM postmortem.
Each stage is bounded. Capture occurs before terminating traffic or allowing
harness recovery; the sampler returns 22, which the harness propagates. Recovery
skips further HIF counter queries after this destructive capture attempt.
No new packet capture or verbose supplicant logging is enabled.

A five-minute run is active at `/tmp/xr819-live-stall-capture-soak.log`, using
the unchanged retained-status image, SHA256
`5dc105d7d1b5ab78ff385c800324cb6175a598e29df42f08cf2da250f124557f`.
Patched AP identity and postmortem exception 0x1a8c8/0x58 are guarded before
deployment. No firmware edits or checkpoint for this diagnostic step.

The trigger fired around 97 seconds into traffic, after more than ten seconds
without ACK progress. Sender had 16 unacked/lost segments and 249056 unsent
bytes; diagnostic ping failed 3/3 before destructive capture. AP neighbor was
REACHABLE and the route correct. Both stations remained authorized/associated;
board BH was alive with empty driver queues and zero used buffers. These are
live observations, not proof that firmware or the radio remained functional.

Capture completed and the harness deliberately exited 22, then recovered.
However, memory-read success was NOT valid TCM capture: both exception reads
and the entire foreground range were all 0xff. The purported DTCM dump did
not match the ELF's pipe/cursor layout and contained apparent network-frame
data. Do not interpret those values as firmware corruption, and do not print
or broadly analyze that data as metadata. Copies are restricted to mode 600
inside `/tmp/xr819-live-stall-capture-ram` (directory mode 700).

The debug driver's first read sets CPU reset/access mode, then forwards raw
addresses to AHB/APB. CPU-visible ELF addresses have not been demonstrated to
be host-visible TCM addresses under that mapping. The next prerequisite is
validating a marker-backed capture/export path on healthy firmware, not another
broad blind memory dump. The live failure metadata is useful; the intended
foreground/exception/DTCM state was not reliably obtained. No checkpoint.

## CPU-assisted control-state export

Direct host access to CPU TCM is no longer assumed. A temporary diagnostic
schema makes the firmware read selected typed control fields itself and return
them through existing MIB 0x100c. No memory copying to guessed bus addresses,
CPU reset, packet bytes or key bytes. Reads are sequential, not an atomic
snapshot with respect to hardware/IRQs.

Words 0–10: STM1 marker 0x53544d31, hardware timer, pending scheduler events,
packed busy/active-TX/current-pipe/PHY-state bytes, RX release cursor, RX claim
cursor, RX producer, then four pipe headers (producer/last/current/state).
Words 11–21 remain RX diagnostics. Earlier aggregate-probe meanings for the
first eleven words DO NOT apply to this image.

The build passed 301/302 tests, stack 6800/6912, packing/layout, and primary LSP
on command.rs. `/tmp/xr819-cpu-state-export.bin` SHA256:
`86988c90c1801dc2c019abe336d688d4fca5b381ac2d268c2786c88c3d122eae`.
Healthy validation reads two MIB responses, checks the marker and pipe-header
bounds, and requires the timer to advance. Decoder checks reject all-ff,
zero-marker and invalid-index samples. Hardware validation is running at
`/tmp/xr819-cpu-state-healthy-validation.log`; no traffic soak until validated.

Automatic failure capture now uses this cooperative export instead of direct
TCM reads. It preserves passive metadata and a pre-HIF ping first, then records
a post-HIF ping because sending a command can wake the device or disturb the
failure. If HIF does not respond, that is recorded as a capture failure; there
is no blind-address fallback. This cannot capture a CPU that cannot service
commands. Retained-BA completion policy is otherwise unchanged; no checkpoint.

Healthy validation succeeded: both STM1 markers matched, timer advanced from
0x01a14106 to 0x01b0dd65, busy/active bytes were zero, all pipes inactive, and
RX release/claim/producer agreed while advancing from 0x3724 to 0x43f4.
BH was alive and buffers empty. This validates the CPU-assisted path, not any
host-bus mapping for TCM.

The validated image is now running a five-minute transfer with live stall
capture: `/tmp/xr819-cpu-state-stall-soak.log`. Healthy snapshot validation is
repeated before traffic. On a failure, passive metadata and pre-HIF ping come
before the cooperative request; post-HIF ping tests for an intervention effect.
No new firmware changes for this run.

The five-minute run completed at 4.28 Mbit/s, with 1113 retransmissions among
111964 sender data segments (approximately 0.99%). Both ping checks were
50/50, buffers drained, and no sustained-stall trigger fired. Healthy STM1
validation passed before traffic. This does not explain or resolve the prior
intermittent connectivity loss; no failed-state CPU snapshot was obtained.

One bounded ten-minute repeat is running with unchanged firmware/export and
the same ten-second progress trigger:
`/tmp/xr819-cpu-state-ten-minute-soak.log`. No checkpoint or new behavior.

The repeat stalled at about 60 seconds and exited intentionally with 22.
Pre-HIF ping failed 3/3, but both cooperative STM1 responses were valid: timer
0x07de2f1f -> 0x07edbe26; RX release/claim/producer remained equal and advanced
0x1f9c -> 0x2c6c; busy/active bytes zero and all TX pipes inactive. PHY operation
state was 3. Post-HIF ping still failed 3/3: the request did not restore observed
connectivity. Driver had one pending/used TX buffer, BH alive, device awake;
association and AP neighbor resolution remained intact. This rules out a
completely unresponsive firmware command path at capture time, not every
scheduler or radio failure.

Scheduler 0x002c0000 is NOT independently diagnostic: the healthy post-TCP
snapshot also had that value (PHY state 2 then, versus 5 before traffic).
Bit 21 is also raised by completion/task handling. Do not infer the cause from
a comparison to pre-traffic idle state alone.

The next diagnostic schema STM2 (0x53544d32) retains words 1–10 and adds:
11 internal-context count [7:0], PAS-context count [23:8], published/host-published/
completion-pending flags [26:24]; 12 publication/copy-completion counts in low/high
16 bits; 13–16 validated per-pipe hardware cursor/mask words (ffffffff if ring
validation fails); 17 filtered RX frames; 18 RX indications; 19 valid RX slots;
20 PHY command/output/dispatch-output/dispatch-flags bytes; 21 wake mode.
It exports metadata only and changes no scheduling/completion behavior.
Artifacts: `/tmp/xr819-cpu-ownership-export*`. Healthy validation must precede
traffic. No checkpoint.

STM2 passed 301/302 tests, stack 6800/6912, packing/layout and primary LSP.
Image SHA256: `8e75a5ad04e7a176296b50076d7ca580cdafd0bd8a7a50ec749abc0cacb16d69`.
Healthy validation passed in `/tmp/xr819-cpu-ownership-healthy-validation.log`:
internal/PAS counts, publication flags/count and copied completions were zero.
Hardware cursor/mask baseline was 0000000f/0000000f/0000000f/000f0f0f, stable
across both reads. Nonzero mask words alone are therefore not a stuck-owner
indicator. RX slots/indications advanced, with filtered count unchanged at 15.
PHY command/output/dispatch/flags was 02050501; wake-mode raw value 07ebbadd
is recorded without assuming an enum interpretation.

A bounded ten-minute reproduction is running on this image:
`/tmp/xr819-cpu-ownership-stall-soak.log`, with healthy validation before
traffic and the same live progress trigger. No behavior changes or checkpoint.

STM2 reproduced the stall at approximately 283 seconds, exiting intentionally
22. Both pre/post-HIF pings failed 3/3. Firmware responses remained live, with
all software context/ownership/publication/completion counts zero and all pipes
inactive. Driver queues and used buffers were also zero. This capture does not
support a stranded software TX owner. Hardware words were stable at
c00f0000/0000000f/0000000f/090f0f0f. RX slots and indications each advanced by
10 between requests, while filtered count stayed 1292. PHY bytes were 02050301,
wake-mode unchanged at 07ebbadd. Do not equate a difference from startup ring
words with corruption; healthy post-aggregation baseline is still needed.

The next unchanged-image run adds a single CPU snapshot after 15 seconds of
recently progressing traffic, then leaves only the existing stall trigger.
Failure metadata additionally includes mac80211 agg_status and AP station
counters after diagnostic pings. This tests loaded ring state and host BA
state without speculative register writes. HIF sampling is an intervention
and will be explicit in the log. Harness syntax checks passed.

The loaded comparison stalled around 515 seconds. Progressing snapshots showed
pipe-0 hardware words 090f0f0f (inactive) and 410f0e0e (active with four
publications). Failed snapshots showed db0f0000, stable while a single PAS/
publication retired between reads. All other owners were empty afterward.
Thus this run does not show a permanently stranded software owner either.

AP neighbor was FAILED in this capture, unlike earlier REACHABLE captures.
AP station TX/RX counters did not advance between pre- and post-HIF diagnostic
pings, so those pings do not establish a failed over-air unicast receive path.
No agg_status files were returned: host BA session state remains unavailable,
not proven inactive. The kernel log contained four MMC data errors; their
presence alone does not establish causality or alignment with stall onset.

Next, the same firmware will receive a reverse-direction probe after the
existing passive/HIF capture. The board has a permanent AP neighbor, so its
ICMP requests do not depend on AP neighbor resolution. AP RX counter deltas
can distinguish those requests reaching the AP even if replies cannot return.
The capture includes before/after AP counters and board status. This generates
only diagnostic ping traffic after failure and changes no firmware/registers.
The outer capture timeout is now 150 seconds to accommodate bounded stages.

Directional capture reproduced the failure (`/tmp/xr819-directional-failure-capture.log`).
AP RX stayed 219978 packets through both ping directions; TX stayed 61518.
Board station TX increased 220970 -> 220979 and failed 24 -> 32; these deltas
include concurrent TCP attempts, not just the three ICMP requests. The board's
permanent AP neighbor therefore did not restore delivery. AP neighbor failure
is not a sufficient explanation. STM2 stayed live with empty software owners,
pipe-0 db0f0000 and RX cursor/indication progress. This does not distinguish
failure to transmit from over-air rejection before AP station accounting.

Read-only review found matching hardware cursors, not demonstrated corruption,
and no missing ordinary-retirement PHY write relative to generic completion.
Selective completion omits bit 21, but its already-set failed-state value
weakens causality. Direct BA retirement can bypass the TX-success PHY tail;
we will measure transitions rather than force command 3 or clear registers.

STM3 (53544d33) retains control words 1–10. Tail words 11–20 are five
(sequence, packed-transition) pairs: PHY operation 1, TX-start handler,
TX-success handler, actual ordinary completion including cursor retirement,
and actual direct BA retirement. Word 21 is the global wrapping sequence.
Each category retains its last completed invocation independently, not a full
history. Packed nibbles 0/4/8 and 12/16/20 are before/after operation state,
retained PHY state and dispatch output; bits 24–25 identify the pipe,
26/27 are before/after pipe-active flags, bit 30 marks unknown pipe (op1),
and bit 31 rejects truncated out-of-range states. Zero sequence means unseen
unless the sequence has wrapped. No packet/key data, event-policy changes,
or new Cargo features. Typed 44-byte BSS storage; foreground-serialized writes.

`/tmp/xr819-phy-transition-export.bin` SHA256:
`eb6b838db0d2f0399e05ebab77c27bc75c19fc835d6db771692f79a37b759e8d`.
Build passed 301/302 tests, stack 6808/6912, packing/layout and primary LSP.
Reader recognizes STM1/2/3 and rejects unknown markers and truncated states.
Healthy validation must precede the next soak. All probes remain temporary;
no behavioral fix or checkpoint is claimed.

STM3 healthy validation passed, with op1/start/success/ordinary records in
sequence and timer/RX progress. The next soak failed before the requested
15-second progressing baseline. Failed records were TX-start 8219, TX-success
8220 and direct BA retirement 8221, all preserving [operation, retained,
dispatch] = [5,5,5]. Ordinary retirement was last seen at 8216. Op1 reached
8233 with [3,3,5] before and after; no newer start/success handlers appeared.
Thus repeated PHY requests followed the last successful sequence, while the
retained state had changed elsewhere. Direct BA retirement is temporally
adjacent, not thereby proven causal. Log: `/tmp/xr819-phy-transition-stall-soak.log`.

A concrete source mismatch exists in `phy::advance_awake_station_tx`: it
explicitly writes retained state 5 -> 3 despite its comment prohibiting that
write. Vendor `phy_state_advance` (annotated-main.c, 0x820a) reads this retained
state in its awake branch but does not directly change it. Rust calls this
from `claim_pas_accounting` when active PAS ownership is zero, during
`release_pending_to_pas`. The next candidate removes only this three-line
write, retaining the same STM3 probe and completion/retry policies. It is
not yet a demonstrated reliability fix.

`/tmp/xr819-preserve-phy-state.bin` SHA256:
`ab194714a85b16589c4f5870792060cbfb6a9e1d416965143209f5efb0df0b31`.
Build passed 301/302 tests, stack 6808/6912, packing/layout and primary PHY LSP.
A bounded ten-minute soak will validate STM3 before traffic, sample progressing
traffic at 15 seconds if available, and retain the same directional failure
capture. No checkpoint.

The preserve-state candidate also stalled (`/tmp/xr819-preserve-phy-state-soak.log`).
Both healthy loaded and failed transitions remained [5,5,5]. This falsifies
state 3 as a necessary condition for the observed failure. The last start and
success were 213298/213299, direct BA retirement 213300, and op1 then reached
213313 without another start/success. AP RX did not advance through the
reverse probe. Removing the forced state write is insufficient; no reliability
claim or checkpoint.

Next candidate: RX BA handling only updates retained member evidence; it no
longer triggers TX IRQ acknowledgement, writes the hardware completion word,
recycles cursors or directly completes contexts. Existing ordinary status and
retry completion remain enabled, including the retained-BA ordinary hook.
This is NOT the rejected global status-deferral experiment. Relative to the
preserve-state image, only RX-side retirement is removed; PHY preservation
and STM3 instrumentation remain. The STM3 direct-BA-retirement category should
now remain unseen. Other BA identity/freshness limitations remain unresolved.
Artifacts: `/tmp/xr819-ba-evidence-only.{elf,bin}`, build log with the same prefix.

Evidence-only candidate passed 301/302 tests, stack 6808/6912 and packing/layout.
SHA256: `a22e2748b4fd5ecff4a59d4c634dc65be340b9af3df3015f07f5eee326c43ce3`.
The first ten-minute soak completed at 6.13 Mbit/s (439 MiB reported), with
50/50 initial/final pings and no stall trigger. Last sender sample: 1649
retransmissions / 318765 data segments (~0.52%, not a radio-loss measure).
Loaded STM3 showed advancing start/success/ordinary sequences and no direct BA
retirement, as intended. Log: `/tmp/xr819-ba-evidence-only-soak.log`.
This supports the RX-retirement removal but does not yet qualify reliability.
An unchanged ten-minute repeat precedes probe cleanup and broader validation.

The repeat passed at 5.84 Mbit/s (418 MiB reported), again with 50/50 initial
and final pings and no stall. Last sender sample: 1752 retransmissions among
304456 data segments (~0.58%). Log: `/tmp/xr819-ba-evidence-only-repeat.log`.
Two consecutive passes support broader qualification, not universal reliability.

Temporary aggregate/PHY probes, CPU MIB schemas and the non-consuming BA scan
were removed. Full probed patch archived at
`/tmp/xr819-ba-evidence-only-with-probes.patch`; diagnostic binaries remain.
Normal MIB RX diagnostics are restored. Only behavioral TX/PHY changes and
this ledger remain modified; no new Cargo features. Clean build passed
301/302 tests, stack 6752/6912, packing/layout and primary TX/PHY LSP.
`/tmp/xr819-clean-ba-evidence.bin` SHA256:
`df9596e00a6d44a8b984c0cc58bab44d559f08493a40afac9081a782d23f4cc3`.

Next clean-image run combines 600s board TX TCP, 60s board RX TCP and 20s
UDP phases at 5/10 Mbit/s in each direction. Pings and status checks separate
phases. STM validation/loaded sampling are disabled for this image; failure
capture explicitly skips CPU-export decoding, with no blind TCM fallback.
No checkpoint; BA peer/format/freshness and no-evidence fallback remain open.

Clean-image bidirectional validation passed (`/tmp/xr819-clean-ba-bidirectional.log`):
600s board TX TCP 6.83 Mbit/s; 60s board RX TCP 20.9 Mbit/s. Last TX sender
sample: 1337 retransmissions / 355111 data segments (~0.38%, not radio loss).
Board TX UDP received 5.24 Mbit/s with 1/8919 lost (0.011%), then 10.5 Mbit/s
with 49/17835 lost (0.27%). Board RX UDP achieved only 3.89/5.99 Mbit/s at
requested 5/10 rates, with 0/6615 and 1/10242 lost: these phases do not prove
reception at the requested offered rates. All seven 50-ping checks passed;
no progress-stall trigger, and recovery was installed at exit.

Three consecutive ten-minute TX passes now include a probe-free build,
followed by bidirectional traffic. This is evidence of improvement after
removing RX-side retirement, not proof of the precise hardware race or of
unbounded reliability. Changes remain uncheckpointed pending the outstanding
BA evidence-validity and no-evidence-policy review.

## Matched vendor/Rust comparison after RX-retirement removal

Four serial runs used the same patched Intel AP, board TX MCS5, 120s TX TCP,
60s RX TCP and 20s UDP phases at requested 5/10 Mbit/s in each direction.
Each firmware pair shared the exact host module; enabled module adds BA action
logging, disabled module rejects host AMPDU_TX_START. Source differences were
checked against the same base driver. Manifest/hashes:
`/tmp/xr819-matched-comparison-plan.md`. Logs:
`/tmp/xr819-matched-{vendor,rust}-{ba-enabled,no-ba}.log`.

| Firmware / host TX BA | TX TCP Mbit/s | RX TCP Mbit/s | Last TX retrans/data segments |
| --- | ---: | ---: | ---: |
| Vendor / enabled | 11.5 | 22.0 | 304/120407 |
| Rust / enabled | 8.53 | 18.0 | 178/88299 |
| Vendor / disabled | 15.0 | 30.9 | 31/155301 |
| Rust / disabled | 5.38 | 16.5 | 13/55670 |

Vendor repeatedly started/stopped host TX BA without TX_OPERATIONAL. Rust
logged TX_OPERATIONAL tid0, window64, result0. Disabled Rust recorded zero
host aggregate TX/metadata/reports throughout. Disabling host negotiation does
not prove vendor firmware performs no internal aggregation; no over-air
aggregation-parity claim is made. These are individual serial runs, not
randomized repeated estimates of the BA-toggle effect.

TX UDP at requested 10M: vendor enabled 10.4 Mbit/s, 21/17835 lost (0.12%);
Rust enabled 10.5, 35/17835 (0.20%); vendor disabled 10.5, 6/17835 (0.034%);
Rust disabled 6.02, 5745/16022 (36%). Rust RX UDP achieved only 3.89/7.35
(enabled) and 3.71/6.74 (disabled) Mbit/s with zero reported loss, versus
vendor 5.24/10.5 in both runs. Therefore zero RX UDP loss does not establish
Rust reception at requested offered rates. All 28 sets of 50 pings passed,
with no sustained-stall capture and recovery installed after each run.

The TX performance deficit is reproducible in this comparison: about 26%
below vendor with host BA enabled, 64% below with it disabled. Rust aggregation
helps, rather than being the sole source of the remaining deficit. Very low
TCP retransmission counts in the disabled run plus its high-offer UDP loss
make service capacity/latency a stronger next investigation than simply
retrying more aggregate members. This is a hypothesis, not localization to
CPU, HIF, MAC or air time. Next discriminator: bounded admission-to-TX-start,
completion and host-credit timing under no-BA traffic, alongside packet/rate
metadata to resolve the vendor internal-aggregation caveat. No firmware code
was changed for these comparisons; BA correctness review remains necessary.

## Temporary SVC1 service-capacity discriminator

Three independent read-only reviews favor admission/refill granularity,
request-priority blocking of confirmations/RX, and synchronous crypto payload
waits as hypotheses, not proven bottlenecks. The historical 5 ms/pass comment
cannot describe current steady throughput: one 1632-byte input per 5 ms is
at most 2.61 Mbit/s even before protocol overhead. No-BA already supports
four ordinary members per publication; vendor also selects idle pipes.
Removing pipe ownership or request priority is not an established safe fix.

A separate disposable working change adds observation-only `service_probe.rs`
under the existing RX diagnostics feature. No new feature, timer read,
descriptor read, scheduling change, or hardware ownership change is added.
The earlier clean candidate remains byte-for-byte unchanged. Ordinary
publication sizes and PAS eligibility/busy observations are counted at the
existing decision sites. Once per 64 foreground passes, software-only reads
sample pending confirmations, request priority and shared-response capacity.
These are systematic samples, potentially phase-aliased, not time-weighted
probabilities. Busy observations count owners seen, not unique frames or
MAC idle duration. Publication counts cover successful ready-batch paths,
not all possible retry/reserved fallbacks. No latency claim follows yet.

MIB 0x100c carries schema SVC1 (`0x53564331`) in word 0, not real Linux named
statistics: 1 loops; 2 successful class-0 admissions; 3 samples; 4 pending
confirmation samples; 5 pending+request-priority samples; 6 those also having
response capacity; 7..10 ordinary publication depths 1..4. Words 11..16 keep
RX pending passes, host-request-blocked passes, lifetime max pending bytes,
lifetime max host transfers, decrypt drops and authentication drops. Words
17..21 are PAS busy-owner observations, no-eligible-PAS publisher passes,
eligible-PAS publisher passes, request-priority samples and response-capacity
samples. Counters wrap modulo 2^32; RX maxima must not be differenced.
Export runs cooperatively, with no interrupt writers. No packet/key bytes.

Validation: 301 default / 303 feature tests pass (one focused probe test),
primary LSP clean, stack 6768/6912, packing and DTCM layout pass. Probe image
`/tmp/xr819-service-probe.bin`, SHA256
`32e89bf1736b798b5af8cfdcaf76d31abe116c7658d7fc64c4c21d43cc906fd8`.
Build log `/tmp/xr819-service-probe-build.log`. Saved-log decoder
`/tmp/xr819-decode-service-probe.py` requires SVC1 and all 22 fields, rejects
invalid pass/sample relationships, and reports snapshot intervals including
setup/pings rather than pretending to measure exact traffic durations.

Planned serial no-host-BA control/probe comparison repeats the prior 120s TX
TCP, 60s RX TCP and 20s UDP 5/10M phases using the unchanged no-BA module,
patched Intel AP, fixed board MCS5 and recovery harness. All CPU-state/PHY
capture modes stay disabled; only normal metadata counters are read between
phases. Stop on failure. First establish whether observation perturbs traffic;
then use occupancy/blocking evidence to decide if sparse qualified timing is
needed. Do not checkpoint or retain temporary probes as a production fix.

First serial control/probe pair completed (`/tmp/xr819-service-no-ba-{control,probe}.log`,
1050s harness exit 0, recovery requested after each). It does NOT establish
probe neutrality. Clean TX/RX TCP was only 1.00/5.07 Mbit/s versus probe
3.66/15.0; the clean run degraded to very slow but nonzero TCP progress.
Last sender retrans/data segments were 319/10855 clean and 381/38582 probe,
versus 13/55670 in the earlier clean no-BA run. Board station TX failures
over primary TCP grew by 323/397 respectively. This comparison no longer
has the earlier near-zero retransmission conditions. No RF or firmware
cause is established. One MMC data error was logged in the probe run,
without proven causal relation. Clean had one 49/50 ping set, all others
50/50; probe all seven sets 50/50. Exit 0 does not mean zero packet loss.
Both retained zero host aggregate TX counters.

At requested UDP 10M board TX, clean received 3.03 Mbit/s, loss 6406/11597
(55%); probe 5.19 Mbit/s, loss 6258/15111 (41%). At requested 5M, clean
3.18 Mbit/s with 3004/8471 loss (35%), probe 4.76 with 814/8919 (9.1%).
RX UDP achieved only 3.89/7.28 clean and 2.76/6.59 probe at requested 5/10M,
zero reported loss: still not requested-rate RX qualification.

SVC1 decoded successfully (`/tmp/xr819-service-no-ba-probe-summary.jsonl`).
Primary TCP snapshot interval (142s, including setup/reads) counted 808750
passes, 38587 admissions (4.77% of passes), 10758 ordinary publications,
mean depth 3.587, and 3243539 busy-PAS-owner observations. Of 12637 sparse
samples, 825 had pending confirmations and 240 were request-blocked; all
240 also had response capacity. Thus 29.1% of pending-confirmation samples,
but only 1.90% of all samples, were blocked. RX pending passes 44332,
request-blocked 9026 (20.4%), no decrypt/authentication drops.
High-offer TX UDP's snapshot interval counted 176786 passes, 8965 admissions,
2314 ordinary publications (2192 full depth four), mean depth 3.874, 707511
busy-PAS-owner observations. Confirmation overlap was 48/159 (30.2%), or
48/2762 (1.74%) of all samples; response capacity existed in all 48. RX
request-blocking was only 16/722 pending passes. These are not time shares.
The global admission-per-pass ceiling is not saturated over either full
interval, and high-offer UDP batches are already nearly full. Neither fact
excludes expensive active passes, completion/refill latency, or airtime.
Busy PAS counts cannot identify MAC idle time or unique stalled owners.

Next: reverse the order (probe then unchanged clean), using identical image
hashes and configuration, before attributing rate differences to observation
or changing scheduling. Original runner archived as
`/tmp/xr819-service-probe-comparison-first.sh`; repeat logs receive `-repeat`.

Reversed pair completed, exit 0 after 1031s; recovery requested and all 14
50-ping sets passed. Probe TX/RX TCP 2.95/11.5 Mbit/s, clean 4.18/14.3.
Last sender retrans/data segments 489/31129 probe, 151/43782 clean. Thus
neither pair establishes a consistent probe advantage or neutrality.
At requested TX UDP 5M: probe 4.80 Mbit/s with 720/8919 lost (8.1%), clean
4.71 with 868/8919 (9.7%). At requested 10M: probe 4.50 with 7977/15663
(51%), clean 4.79 with 6275/14448 (43%). Probe RX UDP achieved 4.16/5.71
Mbit/s, zero reported loss. These current runs do not recreate the earlier
near-zero no-BA TCP retransmission conditions.

Repeat SVC1 primary-TCP interval: 845371 loops, 31213 admissions (3.69%),
mean ordinary depth 3.619; confirmation blocked/capacity-present 183/646
pending samples (28.3%), or 183/13209 total samples (1.39%). High-offer TX
UDP: 179264 loops, 7808 admissions (4.36%), mean depth 3.888 (1916/2008
batches full); confirmation overlap 27/162 (16.7%) pending samples, or
27/2801 (0.96%) total. No exported decrypt/authentication drops. Occupancy
findings repeat, but request-priority overlap is variable and not a duration.
Do not infer throughput limits or remove gates from these counts alone.

SVC1 source/diff archived in `/tmp/xr819-service-probe-svc1.{rs,patch}`.
Next temporary schema SVC2 (`0x53564332`) reuses existing AES timeout-loop
timer deltas to count setup-status and TX/RX payload waits. No new timer
MMIO read, interrupt change, or crypto/replay behavior change is introduced.
For each completed or timed-out wait, record call count, last timeout-loop
elapsed sum, lifetime elapsed maximum and total polling iterations. A wait
already satisfied at its first condition check records zero elapsed. Setup
figures exclude FIFO writes and initial timer reads; payload figures exclude
the tail after the last timer read and DMA setup before the timeout origin.
These are observed polling spans, not total function latency. Iteration
increments and end-of-wait accumulation may perturb execution, so this too
requires qualification; do not assume no added MMIO implies no timing effect.

SVC2 words 0 marker, 1 loop count, 2 admissions; 3..6 TX payload calls/sum/
max/iterations; 7..10 RX payload same; 11..16 unchanged RX diagnostics;
17..20 AES setup-status waits (both directions) calls/sum/max/iterations;
21 sparse service sample count (for loop/schema consistency validation).
All sums/counts wrap modulo 2^32; maxima are lifetime, not interval maxima.
No raw packet or key bytes are exported.

SVC2 validation: 301 default / 303 feature tests, stack 6768/6912, primary
LSP and edited-source diagnostics clean, packing/DTCM checks pass. Image
`/tmp/xr819-service-crypto-probe.bin`, SHA256
`20c6445e82e3c9b348971e86e548c3175cecf92a7c09bab14c70b3815af8e5e3`.
`/tmp/xr819-decode-service-crypto-probe.py` validates the distinct schema,
pass/sample consistency and wait sums/maxima; synthetic valid/invalid tests
passed. Runner `/tmp/xr819-service-crypto-run.sh` uses the same no-BA driver,
fixed MCS5 and bidirectional phases. Its purpose is polling-span localization,
not a claim of performance improvement or instrument neutrality.

SVC2 run completed, exit 0 after 511s; recovery requested. TX/RX TCP
1.70/17.5 Mbit/s, last sender retrans/data 719/18392. TX UDP at requested
5/10M received 2.07/1.97 Mbit/s, lost 4912/8468 (58%) and 7116/10490 (68%).
One ping set was 49/50; another took 24s despite 50/50 replies. RX UDP 5M
had a long receiver tail: sender 1.30 Mbit/s over 20.61s, receiver 535 kbit/s
over 49.11s, lost 45/2280 = 1.97% (iperf incorrectly printed 0%). RX UDP
10M received 4.93 Mbit/s, 0/8411 loss. This is not a neutral instrumentation
qualification, nor requested-rate RX reception.

`/tmp/xr819-service-crypto-no-ba-summary.jsonl` validates SVC2. Using the
existing vendor timer's microsecond convention: primary-TCP interval 144s
had 18528 TX payload waits averaging 130.424 ticks (335.85 iterations),
13794 RX waits averaging 7.166 ticks, and 258576 setup waits averaging
0.108 ticks. Recorded sums: TX 2416499, RX 98843, setup 27957 ticks,
total 2.543299 seconds (1.77% of the whole snapshot interval). RX TCP's
69s interval had 96145 RX payload waits averaging 130.669 ticks, TX ACK
waits averaging 6.884 ticks, total recorded waits 12.739956 seconds
(18.46%). Setup wait lifetime maximum 1 tick, payload maximum 131 ticks;
no 50000-tick timeout spans or exported RX decrypt/authentication drops.
Setup calls equal eight times total TX+RX payload calls in every interval,
consistent with the eight status waits per successful AES transaction.
High-offer TX UDP payload wait mean 129.421 ticks, total 0.553908 seconds
of recorded crypto waits over its 31s snapshot interval. No full-function
CPU utilization follows from these spans; programming/memory work and
unobserved tails remain excluded. The result weakens synchronous waiting
as the dominant TX bottleneck, while showing a meaningful RX wait cost.

All SVC1/SVC2 code was archived then removed from the working source by
restoring `src/` from the preserved clean parent. SVC2 archive:
`/tmp/xr819-service-probe-svc2.{patch,rs}`. No diagnostic feature was added,
no scheduling/crypto ownership fix or checkpoint is claimed. The working
change now contains the investigation ledger only. Next discriminator is a
fresh stock-vendor no-host-BA run under the same current AP conditions:
today's clean and probed Rust runs have worse TX retransmissions than the
earlier matched pair, so a stale vendor rate is an insufficient control.

Post-removal validation: 301 default / 302 feature tests, stack 6752/6912,
LSP and packing/layout pass. `/tmp/xr819-post-service-clean.bin` matches
`/tmp/xr819-clean-ba-evidence.bin` byte-for-byte (SHA256 `df9596e0...`);
full build log `/tmp/xr819-post-service-clean-build.log`.

## Fresh vendor control also loses TCP progress

The fresh stock-vendor no-host-BA run failed the deliberate backlog-progress
guard with exit 22 after 157s of harness time. This is a detected TCP stall,
not evidence that the test process or firmware crashed. Exact firmware/boot/
module hashes and patched AP build ID were checked before deployment.
Log `/tmp/xr819-service-fresh-vendor-no-ba.log`; runner
`/tmp/xr819-service-fresh-vendor-control.sh`. No CPU-state export or direct
memory capture was attempted. Recovery files and reboot were subsequently
verified over Ethernet SSH with byte comparisons to the recovery artifacts.

At the trigger (about 30s into TCP), bytes_acked remained 1390080 for more
than 10s with 75296 bytes notsent, three unacked segments and growing RTO.
Last sender data/retrans segments 1063/100. The aborted run's receiver
summary was only 295 kbit/s over 37.81s, not a completed 120s benchmark.
Pre-capture AP station had 1020 RX packets, 663 TX packets, 18 TX retries,
zero TX failures and 11.4s inactivity. Board reported 1120 TX packets,
1133 retries, 102 TX failures, zero beacon loss. Driver BH was alive,
used input buffers zero, no pending TX/RX, WSM idle. These counters do not
localize where the TCP retransmissions stopped succeeding.

Crucially, all three forward pings before HIF capture, all three afterward,
and all three reverse pings succeeded. The link was not globally dead.
AP RX/TX packet counters advanced to 1029/672 after those probes. The
no-host-BA vendor still reported AGG TXed 451 (members without host head
metadata); this reinforces the distinction between disabling host BA and
proving absence of firmware-side aggregation, without establishing over-air
aggregation behavior. No claim that this is the same race as Rust's former
RX-side retirement stall is justified.

This control demonstrates that the current TCP-progress failure does not
require Rust firmware or the temporary probes. It does NOT erase the earlier
matched Rust/vendor performance deficit, prove an AP defect, or exonerate
Rust's remaining service latency. Pause speculative Rust scheduling changes:
the next useful discriminator is synchronized AP receive/drop and station
retry/confirmation metadata during the reproducible vendor TCP failure,
including its aggregation state, before interpreting throughput comparisons.

## AP-observed stock-vendor TCP repeat

After local sudo authentication was found to be terminal-specific, the user
started the bounded existing AP observer in their terminal. No credentials
were copied into commands or files. Six probes attached successfully before
the parent started `/tmp/xr819-vendor-ap-observed-run.sh`, a TCP-only 120s
repeat with the exact stock vendor/no-host-BA inputs and patched AP. Module
SHA256 `8139310b...`, live build ID `ac263b65...`, and disassembly offsets
were checked: release+0x201 and MPDU+0xc74 are the two PN memcmp calls;
MPDU+0x358 is the common skb drop path. No module swap, replay change,
firmware modification, or packet/key dump. The observer prints bounded
header/reorder/PN metadata and radio-wide debug format strings, not their
payload arguments. It expires automatically after seven minutes.

Run completed after 266s with exit 0, no 10s TCP-progress stall reproduced.
TX TCP 1.89 Mbit/s over 121.76s; last sender retrans/data segments
593/20355. Initial ping 49/50, final 50/50. Three MMC data errors were logged
at board uptime 139.397676, 151.929042 and 169.972158 seconds; timing and
causality relative to individual failed transmissions remain unresolved.
Recovery files were subsequently byte-verified over Ethernet SSH. Log
`/tmp/xr819-vendor-ap-observed.log`. Observer expiry was confirmed and its
full output saved as `/tmp/xr819-vendor-ap-observed-drop-full.log`; the
counts below remain scoped to the earlier preserved prefix.

Trace prefix copied at harness completion to
`/tmp/xr819-vendor-ap-observed-drop-prefix.log`, before observer expiry.
Its printed intervals include association and pings, not exclusively TCP;
a final partial 10s interval may be absent. Total lab-transmitter-filtered
PN comparisons: 2679 reorder-release and 17260 direct-path. No recorded PN
reject, slot mismatch, or lab-attributed common drop. These scoped negatives
do not establish absence of all AP drops or explain the earlier stall.

Radio-wide debug-format counters reported 415 BAR notifications and 210
duplicate notifications; 210 common drops had an empty skb and could not be
attributed by transmitter. Do NOT relabel these as 210 proven board drops.
Lab-filtered reorder samples show TID0/window16 active despite host TX BA
being disabled: AP reorder/BA state for this peer exists. This strengthens
the warning that the host toggle is not aggregation parity, without proving
that every vendor data frame was aggregated. PN gaps alone are not replay
rejects or proof of where packet loss occurred.

The traced low-throughput repeat weakens PN-replay rejection/modulo-slot
alias as explanations for THIS run, not for the unobserved prior stall.
Remaining discriminators include pre-allocation duplicate classification,
BAR/reorder release behavior and synchronized board MMC/confirmation timing.
No Rust scheduling optimization follows from this capture.

## Prepared board-specific flow discriminator

Next observation scripts are isolated in `/tmp`; firmware/module images and
normal harness are unchanged. `/tmp/xr819-ap-flow.bt` probes the duplicate
branch at MPDU+0x34e, where rax still addresses the original 802.11 header,
before the skb becomes attributable. It filters the board TA and records
TID/sequence-control/Retry metadata. Release-function entry filters the
board's ieee80211_sta address; BAR entry/return CPU markers classify nested
releases and expose entry/exit balance so missing returns invalidate that
classification. Verified release ABI/disassembly and raw private offsets:
BA tid at +20, window at +24, buffer head/stored/queue at +0/+2/+4.

Additional header-only probes count board-port-5001 TCP entry at the AP and
AP ACKs queued toward the board. Neither means application acceptance or
successful over-air ACK delivery. Sequence/ACK samples are bounded per 5s;
event counts are cumulative, avoiding print/clear count races. Wall/monotonic
pairs provide clock correlation. This focused observer does not repeat the
previous PN-check probes, so their scoped negatives cannot be carried over.
Module/image guards remain mandatory; root compilation/attachment of this
new AP script remains pending (unprivileged codegen requires CAP_DAC_READ_SEARCH).
User launcher `/tmp/xr819-ap-flow.sh` expires after 600s and rejects another
active bpftrace observer. Trace log is pre-created user-owned mode 600.

Board kernel supports dynamic kprobes but NOT CONFIG_HIST_TRIGGERS. A failed
histogram preflight was cleaned up; no triggers or instance remain. The
replacement `/tmp/xr819-board-flow.sh` preflight successfully registered ARM
EABI probes for wsm_txed headers and caller-decoded cw1200_tx_confirm_cb
fields, then removed them. It uses an isolated trace instance and only
CMD53 error events, not unsafe bh_rx_trace (whose last column contains data).
`/tmp/xr819-board-flow-monitor.py` continuously drains the bounded trace ring,
aggregates submissions by raw WSM id and confirmations by status/rate/ACK
failures/flags, and reports firmware delay sums/log2 bins without assuming
their timing semantics are qualified. It emits projected MMC error fields,
not raw trace lines, register-response words, or packet bytes. It reports
parse failures and ring overruns; either weakens completeness claims.

Counts are emitted beside socket samples over one passwordless SSH session;
the flow-specific observer/harness are `/tmp/xr819-tcp-observe-flow.py` and
`/tmp/xr819-tcp-flow-run.sh`. Tracing setup occurs only after deployment and
association, before iperf starts. Exit traps remove the board trace instance
and its probes; recovery reboot remains the final guard. This is more work
than the earlier counters and may perturb performance. Submission/confirmation
counts are not packet-by-packet matching, and AP samples are not full traces.
Synthetic aggregation, malformed-event and metadata-projection checks passed;
board preflight cleanup verified. Hardware traffic test awaits AP attachment.
Runner ready: `/tmp/xr819-vendor-flow-run.sh` (stock vendor, unchanged no-host-
BA driver, fixed board MCS5, TCP-only 120s, existing stall capture/recovery).

### Setup corrections and rejected ARM kprobe experiment

Initial AP launch failed before attachment: this bpftrace rejects `nsecs(wall)`
and cannot resolve the split-BTF ieee80211_sta type automatically. Also,
root tee could not create/open the user-owned log directly in sticky /tmp
(fs.protected_regular). Corrected to explicit `nsecs(monotonic)` with
userspace bracketing wall/monotonic pairs; decoded loaded mac80211 split BTF
confirmed ieee80211_sta.addr offset 0 (type size 600). BTF SHA256
`2ef07a82bca14893bbad75195ede2fa8e1fb055dc53e2fedb73b84a7166e72a8`
is now guarded. Log moved into private, non-sticky
`/tmp/xr819-flow-capture/ap-flow.log`. Corrected AP source SHA256
`b41fbb31a852248682c6a6bf153d3786148ae7088bff01705a36d888e9f6d2d9`.
Eight probes attached and emitted readiness successfully.

The subsequent synchronized run is DISQUALIFIED. At the very first traced
wsm_txed call, board uptime about 57.65s, the ARM Thumb kprobe single-step
path caused a kernel execute fault. Stack attribution: wsm_txed ->
t16_emulate_push -> thumb16_singlestep -> kprobe_trap_handler. The wireless
BH worker terminated. Only one submission and zero confirmation events
were observed; socket stayed SYN-RECV. Thus there was no valid steady-state
TCP experiment, and this failure must NOT be attributed to vendor/Rust
firmware, AP reorder, or missing TX confirmations. Registration-only preflight
was insufficient to establish probe-hit safety on this kernel.

Harness eventually exited 21 after final ping 0/50; initial ping had passed
50/50. The final snapshot showed BH `terminated`, which the ordinary harness's
`dead`-only snapshot check did not catch; its sender progress detector also
does not cover SYN-RECV/no bytes_acked. These are diagnostic-guard limitations,
not evidence of a second hardware failure. Log `/tmp/xr819-vendor-flow.log`
contains an unexpected kernel Oops register/stack dump from the generic
failure dmesg capture; restricted to mode 600, not packet-data evidence and
not for public upload or further raw-memory interpretation.

Recovery boot/firmware/module files, absence of the flow instance and removal
of both kprobes were verified over Ethernet SSH after reboot. Archived
rejected scripts have `-rejected` names. `/tmp/xr819-board-flow.sh` and
`/tmp/xr819-tcp-flow-run.sh` now refuse execution; no more ARM function-kprobe
runs. Firmware source/images remain unchanged. Any future board-side detailed
instrumentation needs safe static tracepoints or a separately qualified driver
build, not another arbitrary instruction-offset probe.

AP observer expired normally; full log preserved privately as
`/tmp/xr819-vendor-flow-rejected-ap-full.log`. Disabled bpftrace `-k` for the
next run: it floods normal missing-map-element lookups with warnings and can
itself perturb timing. Cumulative counts and BAR-entry/exit balance remain.
`/tmp/xr819-vendor-flow-run.sh` now invokes ONLY the unchanged ordinary harness,
with new `vendor-ap-flow-only` log names, fresh-observer age <=90s guard,
stock vendor/no-host-BA inputs and 120s TCP. This next AP-only run cannot
provide per-second board submission/confirmation timing; do not imply that
it can. Await a fresh AP observer before starting it.

### AP-only flow run reproduces vendor stall without board probes

Fresh AP observer attached eight probes without `-k`. Ordinary vendor TCP
run exited 22 after 218s harness time on backlogged no-progress detection;
no host Oops or board kprobes. Recovery files and probe absence subsequently
verified over Ethernet. Log `/tmp/xr819-vendor-ap-flow-only.log`; AP prefix
`/tmp/xr819-vendor-ap-flow-only-ap-prefix.log`. This is a valid observation
of the stall under AP instrumentation, not a completed throughput benchmark.
Aborted TCP summary 2.72 Mbit/s over 97.29s; last sender retrans/data
370/23202, bytes_acked 33033224, notsent 286704, 19 unacked, RTO 5792ms.
Initial pings 47/50. During failure forward pings 0/3 before and 0/3 after
capture; reverse board->AP pings 3/3. Link state was therefore asymmetric,
not uniformly dead. Driver BH alive, zero used input buffers.

Board-filtered AP duplicate count settled at 30 and did not increase during
the sustained stall. At AP monotonic 653989005943983 ns (08:10:01.352 UTC),
cumulative TCP-entry count was 17814, ACK-queued 11548, BAR entries/exits
180/180; these remained flat through 08:10:16.352. Thus the AP was not
continuing to process TCP data or emit TCP ACKs during that interval, nor
was the traced duplicate branch repeatedly rejecting the board's retries.
Counts are skb/handler invocations, not TCP segment counts (GRO and batching
prevent direct equality with sender segment counters). Bounded sequence
samples cannot reconstruct every frame/ACK or prove absence of earlier drops.

BAR-triggered releases were on RX queue0/TID0/window16; ordinary releases
predominantly used queue12, reaching 2656. This is a real queue distinction,
not by itself a bug: release_frames_from_notif selects the notification's
queue, and firmware may separately notify other queues. At 08:10:17.925,
a further board BAR release on queue0 moved head2779 toward NSSN2795 with
one stored frame. Later one TCP ACK was queued, without another TCP-entry
hit. Do not treat these late capture-era events as recovered bulk traffic.
All observed BAR entry/exit counts balanced, with no persistent nesting marker.

Pre-capture AP station RX22895/TX11606, 370 TX retries, zero TX failures,
about 11s inactivity; board TX23272, retries11214, failures381, no beacon
loss. These counters cannot distinguish failure to transmit, failure before
AP accounting, or earlier receiver drops inducing retransmission/backoff.
MMC errors at board uptime62.460915 and91.817575 map approximately to
08:08:48.675 and08:09:18.032 UTC using the board association wall/uptime
pair, roughly70s and41s before the final loss of TCP progress near08:09:59.
They are not time-coincident evidence for that stop, but delayed effects
are not excluded. Mapping is approximate, not hardware-clock synchronization.

This run narrows the next capture toward early AP MPDU admission/drop
classification versus board transmission; it does not identify which side
lost the frames. No firmware/replay/scheduling fix follows. Any board-side
confirmation timing must avoid the rejected ARM dynamic probes; use safe
static tracepoints or a separately qualified diagnostic host driver.

### Early AP receive trace: short run passes with a slow tail

Added `/tmp/xr819-ap-early-flow.bt`, SHA256
`52cf569dce3a52e706dd15d395f7313478d81e8e3b8fae65eb72bcc00fdf14ef`.
Exact patched MPDU+0x118 is after descriptor/MPDU length validation, before
skb allocation and crypto/duplicate handling: rbx is the RX packet, r9 the
48/64-byte descriptor size, saved len at sp+0x10 and queue at sp+0xc.
Header at packet+8+size is accessed only with sufficient declared length;
only board TA, frame type/subtype, QoS TID, sequence-control, Retry, length
and descriptor status are exported, with bounded samples. Status is at
packet+0x14; cumulative status keys mask 0xf63 (CRC/FIFO, ICV/MIC, cipher,
decrypted). Frames discarded by AP firmware or before this length check
remain invisible; bad-CRC transmitter matching is necessarily tentative.

MPDU entry/return depth-one per-CPU context attributes the existing common
drop at +0x358 even before skb population; nested calls are excluded and
entry/exit/nesting counters qualify attribution. Synchronous __iwl_dbg
DROP calls with function name iwl_mvm_rx_crypto record only static format
strings, never payload arguments. Source shows these crypto messages before
rejection. Existing duplicate/BAR/TCP-entry/ACK-queue probes remain. No PN
probe or board kprobe added. Thirteen AP probes attached successfully.

`/tmp/xr819-vendor-early-flow.log` completed in 267s harness time, exit0:
TCP receiver 6.89 Mbit/s over 123.18s, sender last retrans/data525/73796;
initial/final pings both50/50. This did not reproduce the >=10s backlogged
stall. It did end badly: 115–120s only62.6 kbit/s and the trailing3.18s
40.1 kbit/s. Do not turn completion into a sustained-health qualification.
MMC error uptime169.386458 maps approximately to08:52:10.201 UTC from the
board association clock pair, near the late slowdown, not proof of cause.
Recovery images/module and absence of board probes verified after reboot.

AP prefix `/tmp/xr819-vendor-early-flow-ap-prefix.log` ends with main queue6
73718 board QoS-data indications, all masked status0xa43: CRC/FIFO OK,
CCMP/CCM cipher, MIC OK, hardware-decrypted. Common-drop73 matches the
board duplicate count73; no crypto-rejection-format hits. Radio-wide MPDU
entries/exits73905/73905, no nested calls; BAR entries/exits182/182.
TCP-entry39695 and ACK-queued27999 are handler counts, not directly
comparable to wire segments or RX descriptors. This run supports working
attribution and no observed crypto-status failure, not a negative finding
for an unobserved stall. RX queue6 versus prior queue12 is not itself an
error. Prior flow's completed full AP log was archived as
`/tmp/xr819-vendor-ap-flow-only-ap-full.log`; this short early observer also
expired and its full log is `/tmp/xr819-vendor-early-flow-ap-full.log`.

Prepared a longer repeat using exactly the same BPF source: launcher
`/tmp/xr819-ap-early-flow-long.sh` expires after1200s; ordinary-harness
wrapper `/tmp/xr819-vendor-early-flow-long-run.sh` requests600s TCP with the
same10s stall guard, vendor/no-host-BA/fixed-MCS5 inputs, fresh readiness
<=90s, no board probes or CPU reads. Separate `early-flow-long` logs preserve
short-run evidence. Shell syntax checked; fresh privileged AP launch needed.
Firmware and replay/ownership behavior remain unchanged.

### Long early-AP trace catches a recoverable progress gap

`/tmp/xr819-vendor-early-flow-long.log` exited22 after466s harness time,
not a kernel crash. Requested600s TCP was aborted on the10s no-progress
guard; receiver summary4.58 Mbit/s over333.56s is not a completed benchmark.
Last sender bytes_acked189676416, notsent686352, three unacked, cwnd1,
RTO8576ms/backoff4, retrans/data1287/132282. Driver BH alive, used buffers0;
board TX132382, retries48368, failures1175, no beacon loss. All three
failure-capture ping sets passed3/3. Recovery files and absence of board
probes subsequently verified over Ethernet.

The distinguishing evidence is in
`/tmp/xr819-vendor-early-flow-long-ap-prefix.log`, correlated with its
bracketing AP wall/monotonic pair (times below UTC):

- First observed TCP payload sequence3744390578. At09:11:59.957875 the AP
  queued ACK3934066994; their difference is exactly189676416, the board's
  frozen bytes_acked. The receiver and sender therefore agreed on progress
  immediately before the gap; this is not evidence of that ACK going missing.
- During09:12:00–09:12:11, sender data_segs_out increased132278→132282 and
  bytes_retrans1857784→1863576 while acknowledged bytes stayed fixed. These
  are TCP transmission attempts, NOT proof of SDIO delivery or radio emission.
- AP main-queue8 early indications remained133742 from09:12:04 through
  09:12:14; TCP-entry86545, ACK-queued60614 and duplicate1032 were also flat.
  Radio-wide MPDU entries/exits moved only133893/133893→133894/133894, then
  remained flat. This points upstream of the AP MPDU handler, rather than
  active rejection by its traced crypto/duplicate paths. AP transport or
  firmware discards remain possible; absence at this hook is not absence
  on the air.
- Capture pings produced queue1 indications starting09:12:14.379. TCP resumed
  at09:12:17.230041, BEFORE recovery/reboot: first TCP sequence3934066994 was
  precisely the missing next byte; ACK advanced immediately. The interval
  without TCP entries was about17.27s. This recovery is also compatible with
  the next exponentially backed-off retransmission. Do not claim that pings
  unwedged firmware, or label the guard trigger a permanent firmware stall.

Final prefix main-queue data indications134753 all masked status0xa43;
common drops1048 (1047 queue8, one queue1) equal duplicate count1048.
No crypto-rejection-format hits, nested calls or lost-event warnings.
MPDU entries/exits134920/134920; BAR entries/exits471/471. These final counts
include resumed traffic and capture pings and must not replace the frozen
stall-window counts. No rejection by other untraced stages is ruled out.

Numerous MMC errors occurred, but the last, uptime334.861021, maps roughly
to09:11:11.469—about48s before the gap. Thus there is again no time-coincident
reported MMC error establishing its cause. Next useful boundary is board
submission/transport/confirmation timing using static tracepoints or a
separately qualified host driver; do not reintroduce ARM dynamic probes or
make another speculative firmware/AP change.

### Static host TX tracing builds and passes an actual-hit short test

Created separate `/tmp/xr819-static-flow-driver` from the preserved no-host-BA
host driver. Original module remains SHA256
`d7d9f42188e40942314c2e19ee8befbf479315d94f099834530e519079b9b9c2`.
Diagnostic module SHA256
`5e41292b44d3813ea71a6ad93d56fbe71111b2aa5bfa5264974a7951b47f7cb4`;
patch `/tmp/xr819-static-flow-driver.patch`. Kernel6.18.0-xr819-test+ ARMv7
Thumb2 cross-build with its GCC15.3 toolchain succeeded without warnings.
Standalone LSP lacks the kernel include/build configuration and is not a
valid type-check here; the actual kernel module build is the validation.

Three static events in group cw1200_flow, defined in flow_trace.{c,h}:

- cw1200_queued records numeric queue cookie/previous cookie, length, queue,
  requeue marker at admission and requeue. Cookies are not kernel pointers.
- cw1200_transport records before/after cw1200_data_write, preserving its
  single call and existing error handling. Records cookie only for guarded
  data requests, raw WSM header ID, aligned length, data/command classification,
  phase and return code. A successful call does not prove radio transmission.
- cw1200_confirmed copies caller-decoded cookie/status/rate/ACK-failures/flags
  and firmware delay fields before existing completion handling. No payload,
  key, hardware register read, arbitrary instruction offset or kprobe.

`/tmp/xr819-board-static-flow.sh` owns isolated xr819_static_flow trace
instance, mono clock,256KiB per CPU. Enables those events plus static
wlan0-filtered net_dev_queue and errored CMD53 completion events. Cleanup
stops/disables events and removes instance before normal recovery reboot.
`/tmp/xr819-board-static-monitor.py` emits one-second numeric aggregates,
eight metadata samples per event kind (64 for MMC errors), parser/loss
counters, and periodic/final ring stats. Samples are bounded, not complete
packet histories. Netdev skb counts and WSM frame counts differ with GSO
and segmentation; do not interpret their difference as packet loss.

`/tmp/xr819-tcp-observe-static.py` continuously drains the projected remote
stream into an exclusive mode600 file, even while the main thread captures
a stall. Bounded console-forwarding queue overflow does NOT stop the full
metadata log; overflow/failure/end status is reported. Synthetic projection,
malformed/lost events, sample limits and forwarding-overflow preservation
passed. The dedicated harness also recognizes BH terminated, not just dead.

Actual-hit test `/tmp/xr819-static-flow-smoke.log` exited0 after171s; receiver
6.48 Mbit/s over32.80s, both50-ping sets50/50. Complete projected log:
`/tmp/xr819-static-metadata-1788861158185027630.log` (36 intervals).
Counts18433 queue admissions,18433 data write attempts,18433 successful write
returns,18433 confirmations;1155 netdev queue events, one MMC error. Status6
(WSM_STATUS_RETRY_EXCEEDED) appeared68 times; other confirmations status0.
No parser errors, lost notices, ring overruns/commit overruns/dropped events,
or console-forwarding losses. Drain finished successfully. This qualifies
actual static event hits and collection, NOT probe neutrality or reliability.

Recovery SSH initially timed out during reboot; subsequent boot/firmware/
module comparisons and static-instance/kprobe absence passed. No permanent
installation or firmware change. Added and synthetically checked only CMD53
argument bit31 as a future error sample's write/read indicator; raw argument
and address remain excluded. The smoke log cannot retroactively identify
the direction of its one MMC error.

Prepared `/tmp/xr819-static-flow-long-run.sh`: same vendor firmware, diagnostic
host module,600s TCP/10s progress guard, simultaneous existing early AP trace,
separate static-flow-long log names and continuous private board metadata.
Needs a fresh `/tmp/xr819-ap-early-flow-long.sh` observer. Prior vendor-only
long AP full log preserved as `/tmp/xr819-vendor-early-flow-long-ap-full.log`.
No performance/ownership fix inferred from this instrumentation.

### Synchronized static run interrupted by loss of board reachability

`/tmp/xr819-static-flow-long.log` exited1 after560s. TCP was progressing
(12.8 Mbit/s in415–420s) when Ethernet SSH closed remotely near board
uptime476s, wall10:50:34 UTC. No10s progress-guard capture fired. Observer
then timed out waiting15s for still-running iperf; this exception masked
reporting the SSH monitor's exit status. Aborted TCP summary10.2 Mbit/s over
438.35s is not a completed600s result or evidence of improved reliability.

Board became unreachable on both wireless and Ethernet. Subsequent SSH
returned No route to host; br0 neighbor192.168.0.104 FAILED. Recovery trap
printed INSTALLING_RECOVERY but its SSH command could not be verified; that
marker does not establish recovery. Diagnostic module/stock firmware may
remain installed. Need power/network/serial-console inspection before
another hardware run; preserve panic output before resetting if available.
No board panic/Oops was captured, so do not infer one from loss of SSH alone.

Private log `/tmp/xr819-static-metadata-1788864214520542879.log` has418
intervals,385777 queue events,771554 transport phase events,385768
confirmations (384508 status0,1260 status6),30069 netdev events. It ends
abruptly without BOARD_STATIC_PARTIAL_TAIL_BYTES or final ring stats.
No reported ring overruns/commit overruns/dropped events through the last
periodic report; no lost-event notices. Local drain reached EOF with no
forwarding drops, but that does not establish completeness of the remote
trace at disappearance. End-count differences are not stranded-owner proof.
AP prefix is `/tmp/xr819-static-flow-long-ap-prefix.log`.

There are21 parser errors. Source inspection identified a collector defect
introduced after the smoke test: mmc_request_done's TP_printk does NOT
include cmd_arg (only mmc_request_start does). Requiring it to infer CMD53
direction rejected otherwise valid completion errors. Their details were
not persisted and cannot be recovered from this projected log; zero decoded
MMC errors is NOT zero actual errors. Removed the requirement, explicitly
reporting unknown direction, and tested a fixture matching the real kernel
completion format while excluding response words. Earlier fabricated
argument-bearing fixtures were insufficient. Driver module is unchanged.
Also moved the SSH-exit-status check before waiting for iperf, so future
observer failures report their primary cause rather than a secondary wait
timeout. Neither collector fix explains the board's disappearance.

**Subsequent user clarification:** power was accidentally cut during that
run. The board disappearance is therefore an externally interrupted test,
not evidence of a kernel/firmware/network failure. No serial-console capture
is required for the repeat. After power returned, recovery files initially
did not match; `/tmp/xr819-restore-after-disconnect.sh` restored all three,
rebooted, and verified a changed boot ID, matching recovery files and no
static trace instance. Later nmap/SSH confirmed orangepizero remains at
192.168.0.104, Ethernet interface end0. Full interrupted AP log archived as
`/tmp/xr819-static-flow-power-cut-ap-full.log`. Repeat wrapper
`/tmp/xr819-static-flow-long-repeat-run.sh` uses separate logs and the
corrected collector, same static module and firmware. Fresh AP observer
needed because the previous bounded observer expired.

### Synchronized repeat resolves the late TX requests into retry failures

First repeat exited10 during authentication, before TCP/static board tracing;
excluded from performance evidence. Recovery verified. Unchanged retry2
(`/tmp/xr819-static-flow-long-repeat2.log`) exited22 after273s harness time.
Initial ping50/50; failure pings3/3,1/3,2/3. BH alive, used buffers0.
Aborted TCP29.6 kbit/s over61.51s; not a600s benchmark. Final sender
bytes_acked224440, notsent123080, three unacked, cwnd1, RTO10000ms,
53 retransmissions/211 data segments. Recovery files/trace-instance absence
verified over Ethernet afterward.

Private stream `/tmp/xr819-static-metadata-1788874753369477186.log` contains
60 intervals,228 queue events,228 successful data-write attempts/results,
228 confirmations and167 netdev events. Firmware reported status6
(RETRY_EXCEEDED)57 times;171 confirmations status0. Zero parser errors or
lost notices; all available periodic ring-overrun/drop counters zero. No MMC
error events observed with the corrected completion parser. No final remote
footer after intentional observer termination, so do not assert complete
kernel drain at teardown. The sparse failure window itself is covered.

At board mono187.628195, cookie33685508 entered queue2; write started at
187.628527 and returned0 at187.628668; confirmation at187.631847 reported
status6/rate19/ack_failures4/flags0. At192.732301 the same numeric cookie
was reused for another admission, write returned0 at192.732812, and another
status6/ack4 confirmation arrived at192.736326. Cookie reuse is explicit:
these are separate completed requests, not one owner persisting across time.
Admission→write-return about0.47/0.51ms; write-return→confirmation3.18/3.51ms.
Two preceding failures also had successful writes and confirmations within
about4.14/6.70ms. Counts in these sparse intervals fit the sample caps.

Synchronized AP prefix `/tmp/xr819-static-flow-long-repeat2-ap-prefix.log`:
main queue11 indications161, TCP-entry128, duplicate1 remained flat from
13:39:54 through13:40:09 UTC, including the final retries. All observed main
queue status bits were0xa43; no crypto-rejection hits. This localizes these
requests past host admission/SDIO acceptance into firmware-reported transmit
retry exhaustion, not missing host submissions or indefinite confirmations.
It does NOT prove actual radio emission or exclude AP firmware/transport
loss before host RX. Status6 is firmware's report, not independent air capture.
Strong reported signals (~-37dBm AP, -42dBm board) are not proof of a reliable
radio link. Do not weaken replay checks or change Rust scheduling from this.

Next controlled run `/tmp/xr819-static-flow-mcs0-run.sh` changes only board
fixed TX rate5→0, retaining600s request/10s guard, same vendor firmware,
static host module and AP trace. Compare retry failures and progress before
making a rate-dependent claim; ideally repeat MCS5 afterward to distinguish
run-to-run variation. Separate mcs0 logs preserve prior evidence.

### MCS0 completes; MCS5 return control pending

Initial MCS0 launch never passed Ethernet wait_ready (exit3, empty traffic
log); no deployment or rate change occurred. SSH and ARP discovery could not
find the board then. User subsequently reported the network fixed; SSH and
all recovery-file comparisons passed. Do not count that failed launch as a
wireless result.

Resumed `/tmp/xr819-static-flow-mcs0.log` exited0 after746s harness time:
TCP3.06 Mbit/s over602.21s, initial/final pings50/50, no progress stall.
Private metadata `/tmp/xr819-static-metadata-1788881400104585837.log`:
159769 admissions, successful data writes and confirmations, plus one
successful command write. All confirmations rate14 (MCS0); status0=159695,
status6=74 (0.0463%). No parsed MMC errors, parser errors, loss notices,
reported ring overruns or forwarding losses; final remote footer present.
Last TCP sample770 retransmissions/159818 data segments, cwnd319 and RTT
about1.70s: completion does not imply low latency or no TCP loss. Firmware
retry exhaustion and TCP retransmissions remain different measurements.
Recovery files and trace-instance removal verified after reboot.

This is healthier than the prior MCS5 stall, but the intervening network
repair and historical run variability prevent attributing it solely to rate.
Prepared same600s/10s-guard MCS5 return control in
`/tmp/xr819-static-flow-mcs5-return-run.sh`, using identical vendor firmware,
static driver and tracing with new log names. No firmware behavior change.

### MCS5 return reproduces retry-exhaustion stall

`/tmp/xr819-static-flow-mcs5-return.log` exited22 after525s harness time,
with backlogged TCP stopping around385s. Aborted receiver2.85 Mbit/s over
397.51s is not a completed600s result. Initial pings50/50; failure sets3/3,
2/3,2/3. Last sender1316 retransmissions/99153 data segments, cwnd1,
RTO7360ms, bytes_acked141662184, notsent590784 and three unacked.
Recovery files and static-instance absence verified over Ethernet.

Private metadata `/tmp/xr819-static-metadata-1788882295020516473.log`:
99270 admissions, successful data-write calls and confirmations. Firmware
status6=1293/99270 (1.3025%), versus74/159769 (0.0463%) in the preceding
MCS0 run. All remaining confirmations status0. Sparse late requests again
received status6/rate19/ACK-failures4/flags0, rather than being stranded in
host queues or waiting indefinitely for completion. Zero parser errors,
lost notices or reported periodic ring overruns/drops; continuous local
metadata drain ended without forwarding losses.

AP queue8 indications99154, TCP-entry71257, ACK-queued48399 and duplicate689
remained flat from15:51:10 through15:51:30 UTC; capture pings appeared on
queue1. One CMD53 timeout (cmd_err/data_err=-110, zero bytes) was observed
near15:47:28 UTC, roughly224s before the final pause. Direction unknown
by design. Every traced data-write call still returned0, so that isolated
completion error cannot establish a failed outgoing submission at the stall.

MCS0 success followed by MCS5 failure after the network repair strengthens
rate-dependent delivery/retry behavior as the lead. It does not establish a
universal MCS5 defect, an RF-only cause, or a firmware fix; earlier MCS5 runs
were much faster. Next inspect the actual configured retry/fallback policy
and test automatic rate selection rather than treating forced-MCS5 failures
as a Rust scheduler defect or leaving MCS0 as the final performance solution.

### Automatic-rate vendor run completes; matched Rust comparison prepared

Static host-driver review: tx_policy_build_xr819 constructs 24-rate nibble
policies from mac80211 control rates; tx_policy_upload supplies the policy
and retry limits to firmware; max_tx_rate comes from the sorted leading rate.
MCS0–7 map to hardware IDs14–21. This verifies the configuration path,
not every runtime fallback attempt. Existing empty FIXED_MCS harness mode
skips the forced-rate command after fresh boot/association. Driver unchanged.

`/tmp/xr819-static-flow-auto-rate.log` exited0 after748s harness time:
5.37 Mbit/s over602.20s, no10s progress stall; pings49/50 then50/50.
Private metadata `/tmp/xr819-static-metadata-1788886130537630677.log`:
280643 admissions/data-write successes/confirmations,1651 command-write
successes; statuses0=279510,6=1133 (0.404%). Completion-rate distribution:
15:2,16:27,17:869,18:23833,19:31212,20:164251,21:60449. Mostly MCS6/7,
not simply selection of MCS0. These are confirmation rates, not per-attempt
fallback histories.26 MMC error events parsed; all traced data and command
write returns0. Direction is unknown; do not equate MMC events to TX failures.
Zero parser errors/lost notices, periodic overruns/drops and forwarding losses;
final remote footer present. Last sender1259 retransmissions/280164 data
segments, cwnd1,21 lost/unacked: successful bounded run is not loss-free or
proof of a permanent fix. Recovery comparisons/trace absence passed after
one transient post-reboot SSH timeout.

Prepared `/tmp/xr819-static-flow-rust-auto-rate-run.sh`: same static no-host-BA
driver, AP observer,600s/10s guard, unrestricted rates; switches to qualified
clean Rust image df9596e0 and matching open boot51cbe9ec, both hash-guarded.
No Rust code or completion/replay behavior changed. Preserve vendor full AP
log before restarting the bounded observer for the matched comparison.

### Automatic-rate Rust comparison: stable, still slower in this pair

`/tmp/xr819-static-flow-rust-auto-rate.log` exited0 after748s harness time:
3.24 Mbit/s over601.34s versus vendor5.37 Mbit/s, about40% lower. Both
Rust ping sets50/50; no10s stall. Private metadata
`/tmp/xr819-static-metadata-1788887061312710043.log`:168528 admissions,
successful data writes and confirmations,1346 successful command writes.
Status0=167998, status6=530 (0.3145% versus vendor0.4037%). Completion
rates16:26,17:851,18:8126,19:16074,20:120516,21:22935, mostly MCS6;
these are not a per-attempt rate history. No parsed MMC error events.
Zero parser/loss notices, reported ring overruns/drops and forwarding losses;
final remote footer present. Last TCP sample585 retransmissions/168394 data
segments, cwnd66 and RTT61.3ms; compare distributions, not isolated final
samples, before attributing performance to latency or congestion behavior.

Recovery comparisons and trace-instance absence verified after a transient
post-reboot SSH timeout. Lower retry-exhaustion fraction does not explain
away the throughput gap or exclude different burst timing/ACK behavior.
This is one serial pair, not proof of a fixed40% deficit or probe neutrality.
Next use the existing synchronized timing evidence to compare service gaps,
then a bounded reverse-order confirmation if needed; avoid new firmware
instrumentation or speculative scheduling changes without a concrete lead.

### Negotiated-BA Rust, automatic rates, packet tracing disabled

Following the static review, reused the qualified clean Rust image (SHA256
`df9596e00a6d44a8b984c0cc58bab44d559f08493a40afac9081a782d23f4cc3`)
and previously tested BA-enabled diagnostic host module (SHA256
`a237a79ed4c141ec0a3d66a961aba1cef183c4e3e8a89f110bf672b78b6f0626`).
No firmware changes, forced BA flags, depth increases, packet tracing, or
CPU-state dumps. Automatic rates, existing patched Intel AP. Socket polling
and before/after diagnostic snapshots remain, so this is not observer-free.

The first launch stopped before deployment because global tracing_on was1;
events were disabled, tracer was nop, no instances existed, and recovery
files matched. Turned the idle global switch off and relaunched. Log:
`/tmp/xr819-rust-ba-enabled-untraced-retry.log`; wrapper:
`/tmp/xr819-rust-ba-enabled-untraced-run.sh`.

Completed601.7964s at4.70 Mbit/s, no10s stall guard, both50-ping sets50/50,
BH alive. TID0 reached TX_OPERATIONAL with MIB result0. Idle aggregate
counters were zero; after TCP:184012 AGG TXed,51153 aggregate heads,
132859 members without metadata, reports187819 length/183992 ACK/0 invalid.
These are host-reported aggregation/completion counters, not independent
on-air delivery or aggregate-depth measurements. Last TCP socket sample:
843 retransmissions/244852 data segments (about0.344%).

Recovery file comparisons and trace-instance absence independently verified;
wrapper exit0. The4.70 result is higher than the earlier traced no-BA Rust
3.24, but driver BA mode, tracing, and run time all differ: it does not isolate
aggregation benefit or establish vendor parity. It does establish that the
existing negotiated-BA path is operational and sustained this10-minute run.
A matched untraced control/repeat is needed before attributing the difference.

Independent follow-up also confirmed the watchdog only retires the current
ordinary slot before recycling the pipe. This was already recorded above,
including zero watchdog expiries during a sustained-loss run. Expiry alone
is not hardware-quiescence proof; do not implement a blind batch-retirement
loop or treat this latent recovery concern as the throughput explanation.

### Matched untraced vendor control after negotiated-BA Rust

`/tmp/xr819-vendor-ba-enabled-untraced.log` completed600.3473s at8.19 Mbit/s
versus Rust4.70 over601.7964s. Same BA-enabled diagnostic host module,
automatic rates, patched Intel AP,600s request and socket-only observer;
stock vendor boot/firmware hashes checked before deployment. No packet
traces or CPU-state dumps. Vendor baseline pings49/50, final50/50; BH alive,
no10s stall guard, wrapper exit0 and recovery file comparisons/absence of
trace instances verified.

Vendor again never reached host TX_OPERATIONAL: repeated TX_START/STOP_CONT
callbacks, no operational result. AGG TXed changed11->21, with no aggregate
head metadata or aggregate reports. These host counters cannot establish
vendor's internal/on-air aggregation mode or depth. Rust did reach
TX_OPERATIONAL and reported aggregation, so matched host configuration is
not equivalent negotiated/internal BA behavior.

Last socket sample vendor7410 retransmissions/431612 data segments (1.717%),
versus Rust843/244852 (0.344%). These are TCP attempts, not radio failures.
Vendor was1.74x faster (Rust42.6% lower) in this serial pair despite the higher
TCP retransmission fraction. The deficit therefore persists without packet
tracing and with Rust negotiated aggregation enabled; neither observation
localizes its cause or establishes a repeatable fixed magnitude. No firmware
fix or aggregate-depth change made.

### Pre-publication completion-drain discriminator: not retained

Tested a second bounded32-event MAC service and existing completion/requeue
routing pass immediately before publication, after software context service.
Recomputed runtime owners afterward; no FIQ enable, FIFO/replay changes, RX-BA
retirement, or new features. The deliberate underfill wait is already disabled
by experimental-fast-loop, so this tests completion-service placement instead.
302 library tests passed; ARM call-chain6680/6912, exception224/256, packet-RAM,
packing and DTCM checks passed. Build log `/tmp/xr819-prepublish-drain-build.log`;
candidate ELF/image `/tmp/xr819-prepublish-drain.{elf,bin}`, image SHA256
`a05ca29d4849afc7e198f231a279baea22f7fd02a84bdf584ab4bee900df5680`.

Same automatic-rate BA-enabled driver and untraced600s harness:

| Run | TCP Mbit/s | Duration s | Last TCP retrans/data |
| --- | ---: | ---: | ---: |
| Earlier clean baseline | 4.70 | 601.7964 | 843/244852 |
| Extra pre-publication drain | 3.73 | 600.7471 | 1020/194543 |
| Clean baseline return | 4.28 | 600.1239 | 1135/222761 |

Candidate and return both reached TID0 TX_OPERATIONAL, passed100/100 pings,
kept BH alive, avoided the10s stall guard, and exited0. Recovery file comparisons
and absence of trace instances verified after each. Candidate aggregate reports:
39476 heads,143274 length,140726 ACK,0 invalid; return46719 heads,171245 length,
166384 ACK,0 invalid. These are host reports, not on-air depth measurements.
Logs `/tmp/xr819-prepublish-drain-run.log` and
`/tmp/xr819-prepublish-baseline-return.log`.

Candidate was12.9% below the return baseline and20.6% below the earlier baseline.
One candidate run does not establish a fixed regression, but there is no benefit
supporting retention. Archived `/tmp/xr819-prepublish-drain-candidate.patch` and
restored host_tx_driver.rs to its exact parent; no behavioral fix retained.
This weakens the specific extra-drain placement hypothesis, not all cooperative
service/FIQ latency hypotheses: the extra pass also adds work and may alter
batch formation. Do not reinterpret this as proof FIQ cannot improve performance.

### Ordinary backoff discriminator exposed HIF response-availability panic

Narrow experiment grew the existing per-interface/AC contention window on
ordinary host retries (every other retry, clamped to CWmax), resetting on
ordinary success/retry-exhaustion. Aggregate retry policy and ownership were
unchanged; nonzero vendor backoff-override mode was deliberately untouched.
303 tests, ARM stack6752/6912 and exception224/256, packing and layout passed.
Image `/tmp/xr819-ordinary-backoff.bin` SHA256
`978df375d9d7d900524022d4c5947ce1882614d04068b4dc4b47d8083d31cfb6`;
ELF and build log share that prefix. Candidate patch archived as
`/tmp/xr819-ordinary-backoff-candidate.patch`; tx.rs restored to exact parent.

Untraced automatic-rate BA-enabled600s request failed guard exit22, log
`/tmp/xr819-ordinary-backoff-run.log`. Initial50/50 pings, TID0 operational,
then TCP stopped progressing around517s; aborted result4.65 Mbit/s/539.1713s
is not a successful throughput measurement. BH errcode1, five used buffers,
four pending in one queue; failure pings0/3. Recovery files and absence of
trace instances independently verified.

Kernel received a firmware exception indication: kind0x100, line1123,
column9, file hash0xf0ce8142. The firmware's FNV-1a file hash resolves exactly
to `src/hif.rs`; line1123 is `assert!(self.response_available())` immediately
after reclaim_tx in publish_request_in_place. This is a concrete HIF response
storage availability failure, not evidence of RF retry exhaustion or a
watchdog leak. One earlier MMC data error was not time-coincident. Last TCP
sample856/217416 retrans/data. No CPU-state dump was attempted. Candidate
not retained; traffic perturbation may have exposed a pre-existing HIF bug,
but causation by the backoff modification is not established. Next inspect
caller availability accounting and coalesced-confirmation publication.

### HIF response-capacity gate: first hardware qualification passed

Source inspection found a concrete admission mismatch: command::service_one
checks publication_available (output queue space only), whereas synchronous
response publication additionally requires a free shared output buffer and no
prepared shared slot. poll_request previously consumed the pending invocation
and detached the descriptor without checking those additional requirements.
The coalesced-confirmation caller already checks response_available; this is
not evidence that its batching caused the panic.

Candidate hif.rs reclaims completed output and checks response_available before
consuming an invocation/descriptor. A blocked request remains pending until
storage is available. Shared response-capacity logic has a focused test for
full buffers despite queue space, full queue, prepared storage, and wraparound.
No backoff, FIQ, FIFO, replay, or RX-BA policy changes.303 tests passed; ARM
stack6752/6912 and exception224/256; packing, packet-RAM and DTCM gates passed.
Image `/tmp/xr819-response-capacity.bin` SHA256
`ee76be8ff39b40e08374a71f6f1c37d1bb52ca06cb90d2d65e7bd12557a22fe2`;
ELF/build log share that prefix.

Untraced automatic-rate BA-enabled run completed600.7371s at5.01 Mbit/s,
100/100 pings, TID0 operational, BH alive, no stall guard or firmware assertion,
exit0. Aggregate reports54819 heads/203332 length/199384 ACK/0 invalid. Last
TCP sample822 retransmissions/260640 data segments. Recovery comparisons and
absence of trace instances verified. Log `/tmp/xr819-response-capacity-run.log`.

Retain as a concrete capacity-precondition correction with one successful
hardware qualification, not a proven throughput gain or proof all HIF stalls
are fixed. Earlier clean samples4.70 and4.28 are serial, variable conditions;
this run does not reproduce the same buffer pressure as the rejected backoff
candidate. No new firmware feature introduced.

### Host-request priority discriminator: static audit and build qualification

Started a new jj change on the retained response-capacity correction; hif.rs is
unchanged by this discriminator. Sole writer/hardware owner; no nested agents.

The suspected reversed airtime offsets are **not a discrepancy**. Read-only
Ghidra instruction inspection of xr819-annotated-main/ghidra-fw-main.bin:
`pas_compute_tx_timing` retains PAS in r4 at0x7fa8; at0x806a-0x8074 it passes
r1=PAS+0x38 and r0=r1+2=PAS+0x3a to airtime_compute(0x83e2). The callee saves
r0/r1; after its extra stack word these pointers are at sp+4/sp+8. It stores
the unextended result through the first pointer at0x8458-0x8460, then adds
0x20,0x0a,or0x10 and stores through the second at0x8462-0x8480. Thus base is
PAS+0x3a and extended is PAS+0x38, matching Rust mac::{base,extended}_airtime,
compute_single_frame_pas_timing and prepare_single_frame_pas_timing. The type
labels are wDurPayA at0x38 and wDurPayB at0x3a: their names do not imply which
is base. No airtime code or Ghidra database edits. Evidence files:
`/tmp/xr819-priority-airtime-{caller,compute}.json` and
`/tmp/xr819-priority-pas-type.json`. Function-disasm initially returned only ten
instructions; the final evidence uses explicit disasm -n350/-n80, not that
truncated result. This verifies offset ordering, not all timing mathematics.

Transport audit: request_available includes response_available, so its name
means IRQ-notified descriptor readiness AND current full admission capacity,
not raw pending work or a reservation. service_interrupt only records notified
RX/TX work; reclaim_tx releases shared storage or RX tokens only following the
TX notification, preserving stage-successor-before-release ordering. Output
queue capacity is64; shared output slots and prepared storage are additional
constraints. publish_radio uses zero-copy RX tokens; command responses and
coalesced confirmations copy into shared storage. poll_request reclaims/checks
full capacity before detaching one input and schedules a ready successor.
command::service_one finishes response publication before returning; admission
failure releases its request credit and encodes an immediate failure response.
HostTxDriver::admit does not publish asynchronous HIF output. Retained full
capacity gating is therefore required for both success and failure paths.

Candidate moves the single command service from loop end to immediately after
MAC/management service and before asynchronous output. Fresh successor admission
state is sampled afterward; all existing asynchronous capacity checks and
host-request gates remain. No extra command batching, publication-owner reorder,
BA bypass, RX-BA retirement, crypto/replay changes, FIQ, depth or feature changes.
The admitted command gets first use of reclaimed response capacity each pass;
a blocked input is not detached. A successor retains priority when admissible.
Freshly arriving hardware notifications wait for the next normal interrupt
service, as before. This does not claim an absolute latency bound while the
host stops draining output, nor fairness for asynchronous work under an
unbounded synchronous-command stream. It removes avoidable suppression by the
request already serviced, not all possible confirmation delay.

Targeted primary LSP: no errors in hif.rs/hif_startup.rs.303 library tests pass;
nightly thumbv5te-none-eabi release hif-startup build-std=core passes. Stack6744
of6912, exception224 of256; packet-RAM, section packing and DTCM checks pass.
Features: experimental-list-first-depth-four-ampdu,experimental-fast-loop,
experimental-aggregate-rate-feedback,experimental-rx-path-diagnostics (defaults
and transitive features unchanged). Build script/log `/tmp/xr819-priority-build.*`.
Candidate `/tmp/xr819-priority-candidate.bin` SHA256
`94eafb9c709f32b61f2b94badbc24e5924f47f4463c6bd4e11db94a246d2d950`;
ELF SHA256 `e3890749a061eea76ef0de63edf5be328e4499b837d8ed82b24954bd0b6d3f2f`;
patch `/tmp/xr819-priority-candidate.patch` SHA256
`2455b984d815f57a5c399e96f4b6b686ba596c32e3fa9f00880eb74e185294ca`.
Baseline archived `/tmp/xr819-priority-baseline.{bin,elf}`: image SHA256
`ee76be8ff39b40e08374a71f6f1c37d1bb52ca06cb90d2d65e7bd12557a22fe2`,
ELF `88860a7e8748e9da48fc81cbb6aca1f951a87d2a10ab6db51f3e3ac27fa344b9`.

Read the response-capacity and TCP wrappers, observer and failure-capture paths
fully before reuse. CPU export/loaded captures, tracing and extra bidirectional
phases remain disabled; BA-enabled driver, auto-rate600s TX,10s stall guard,
recovery EXIT unchanged. Before starting, no managed runs or host observers,
board iperf/bpftrace absent, all three recovery files independently compared,
no trace instances. Started serial baseline-first/candidate/baseline-return via
`/tmp/xr819-priority-series.sh`, with separate file/trace verification after each
wrapper. Per-phase logs use `/tmp/xr819-priority-*-run.log` and `*-recovery.log`.

### Priority discriminator: no observed benefit; reverted, third run interrupted

| Run | TCP Mbit/s | Duration s | Last TCP retrans/data segments |
| --- | ---: | ---: | ---: |
| Retained HIF baseline first | 5.84 | 600.6334 | 294/303071 |
| Command-first candidate | 5.69 | 600.6721 | 201/294817 |
| Retained HIF baseline return | incomplete | ~47s logged | 30/14839 |

The first two runs exited0, passed100/100 pings, reached TID0 BA operational,
and reported BH alive without a recorded stall/assertion. Independent recovery
logs confirm all three files and no trace instances/events after each. Host
aggregate reports were baseline62114 heads/230986 length/228844 ACK/0 invalid,
candidate60246/222682/220435/0. These counts are not latency measurements or
on-air aggregation depth; TCP retransmissions are not radio retry counts.
Candidate was2.6% below the first baseline: no observed benefit supports
retention, but this incomplete serial pair does not establish a regression or
a repeatable effect. The planned return baseline did NOT finish and must not
be presented as a completed baseline-candidate-baseline comparison.

Lifecycle failure: series.log ends PHASE_START baseline-return at10:13:12+02;
return traffic log ends abruptly at08:16:18UTC (10:16:18+02), with TCP ACKs still
progressing around47.6s, no exit code, stall capture, or recovery footer. After
the parent escalated the lost completion notification, no local managed series
or matching wrapper/observer processes remained. Cause of their disappearance
is unknown; a bounded local kernel-journal OOM query around10:14-10:18 found
no entries. Do not infer an RF/firmware failure or successful recovery from this
truncated log. Waiting for the lost notification left hardware ownership open
and the EXIT trap did not restore the board: this was an orchestration/recovery
failure, not a completed test. No replacement benchmark was run.

At10:35:43UTC board SSH was responsive (uptime2h21m); test iperf daemons remained,
boot matched recovery but firmware/module did not. Immediately copied all three
recovery files, ran depmod/sync, compared them, then rebooted. Separate post-boot
verification succeeded at10:36:28UTC (uptime0min): boot cmp, host-event-drain
vendor firmware cmp, module cmp; zero trace instances, events disabled, nop
tracer, no iperf. Evidence `/tmp/xr819-priority-interruption-restore.log` and
`/tmp/xr819-priority-interruption-recovery.log`; managed verifier proc_f397
exited0. Recovery is the host-event-drain vendor image, not stock firmware.

Restored src/bin/hif_startup.rs exactly from the retained-HIF parent revision.
Scheduling candidate remains archived with ELF/image/patch, not retained in
source. hif.rs was never changed in this experiment; its full response-capacity
check including admission-failure protection remains. Final new-change diff is
this ledger only; no new feature, airtime correction, or improvement checkpoint.
303 tests and the6744/6912,224/256 stack/layout qualification above apply to the
archived candidate. They do not prove absence of every command/security/owner
regression; reverted source restores the prior qualified scheduling exactly.

### Vendor-faithful contention-window correction: implementation and gating

Started a new jj change on the retained response-capacity parent; hif.rs and
hif_startup.rs are untouched by this work. Sole writer and sole hardware owner.

Implements the instruction-verified spec (`/tmp/xr819-backoff-event-spec.md`),
not the archived `/tmp/xr819-ordinary-backoff-candidate.patch`, which grew only
on the ordinary tail, reset from `complete_tx_pipe_slot`, dropped vendor gates
and halted on `interface >= 2`.

New `src/backoff.rs` holds the single production bank implementation behind a
mockable `BackoffBankIo`: growth (even counter -> `2W+1`, clamp on **both**
parities, wrapping counter advance), per-AC reset (counter 0, window CWmin) and
the interface-wide reset-all. Bounds are checked against the three real PAS
banks and four ACs and rejected without a write - no halt and no spin, because
every caller is past the destructive MAC event read. Override arithmetic is
supported without enabling override mode: `0x04002088+4` supplies the reset
window and `+8` the growth clamp, so dtcm's `opaque_18` is now named
`override_maximum_window` (`0x04002090`) with offset and address asserts.

Placement follows event provenance, which `SingleTxRetryBackend::decide_retry`
now carries explicitly as `RetryEvent`:

- One growth per qualified physical retry event in
  `execute_single_outstanding_tx_retry`, after the mask/slot-state/inactive-pipe
  and status-6 gates and **before** the ordinary and aggregate branches, so a
  four-member aggregate, a BlockAck-driven early `CompleteSuccess` and an
  `AwaitBlockAck` all mutate exactly once. Scope is host mode-0: internal frames
  (vendor policy `0xf`, mode 2) are skipped. The bank is the head frame's
  interface/AC - the same entry `TxPolicy::program_random_backoff` draws the
  random backoff from.
- The head policy-exhaustion reset (`0x8a20`) sits in a shared
  `resolve_head_retry_step` used by the ordinary tail, and runs only for
  `RetryEvent::PhysicalRetry`. Structural rejections, member give-ups, session
  failures, watchdog `decide_retry` and generic status `0x0b` retirement do not
  reach it.
- The two existing vendor-mapped hooks (`reset_status_backoff` at `0x9a8c`,
  `reset_backoff` at `0x9d2c`) now call the shared reset. Their executor gates
  are unchanged: the status hook still only fires on the `Complete` plan (kind 2
  advances without completion), and the success marker keeps pipe state 1,
  retry rate `0xff` and PAS bit 15 clear, now expressed as the tested
  `success_marker_resets_backoff` predicate with no slot-kind exclusion.
- `prepare_selective_member_retry`, `prepare_whole_ampdu_retry`,
  `complete_tx_pipe_slot` and the watchdog remain free of bank mutations.

VIF JOIN/EDCA reset-all delegates to the same shared implementation, preserving
the existing access order.

Offline: 318 library tests pass (303 before). The new vectors drive the
production executor and hooks through mock I/O rather than duplicate models:
growth once per event including the BA-driven early decision, `(0,15)->(1,31)`
then `(1,31)->(2,31)`, zero bank writes for absent mask, wrong slot state,
inactive pipe, internal frames and out-of-range interface/AC, watchdog
provenance mutating nothing, exhaustion resetting only on a physical retry,
override reset 7 and clamp 63, odd-parity clamping, counter wraparound, bank
isolation across interfaces 0/1/2, and a real 44-byte EDCA payload fed through
the parser into the production reset path. Nightly thumbv5te-none-eabi release
build passes with stack 6752/6912 and exception 224/256; packet-RAM, packing and
DTCM checks pass. Features unchanged.

Candidate `/tmp/xr819-backoff-qualified-candidate.bin` SHA256
`40e4399d30c315bd33c15c38d9f6094234beee103104676c5c62c9b43a3777e2`, ELF
`4d8ef6ee93f3c4ee72af4e63634b8414f8edbfd7a0223dcae0e61d5aca2b4d9c`, patch
`/tmp/xr819-backoff-qualified-candidate.patch`. The parent revision rebuilt in a
separate jj workspace reproduces the baseline image bit-exactly
(`ee76be8ff39b40e08374a71f6f1c37d1bb52ca06cb90d2d65e7bd12557a22fe2`), so the
candidate differs from the baseline only by this correction.

Unresolved facts carried from the spec, not invented here: the runtime
W/CWmin/CWmax before the first data TX, whether vendor `txp_pipe_tx_success`
resets for aggregate pipes in the tested BA path, and whether
`g_backoff_ctrl[2]` is ever written by a WSM MIB in this driver.

### Independent board-side recovery timeout qualified before deployment

The previous series died with its local runner and its EXIT trap never restored
the board. A host-side trap cannot be the only recovery path, so recovery now
also runs from the board itself.

`/root/xr819-restore-recovery.sh` takes `/run/xr819-restore.lock` under flock,
requires the `/root/xr819-deadman-armed` flag, clears that flag first so it can
fire at most once per arm, copies the three recovery files, runs `depmod -a`,
syncs, re-compares all three, logs to `/root/xr819-restore.log` and reboots.
`/etc/systemd/system/xr819-deadman.{service,timer}` run it at `OnBootSec=1200`
with `AccuracySec=1s`; arming is `touch` plus `systemctl enable`, so the timer
survives the reboots each phase performs - the property a transient
`systemd-run --on-active` timer lacks.

Qualified independently, every step observed from the board's own state after
the host session was closed:

| Check | Evidence |
| --- | --- |
| Timer fires without the host | marker written by the board at 14:36:26Z after arming at 14:35:11Z |
| Real script restores and reboots | RESTORE_START 14:37:42Z, RESTORE_FILES_OK 14:37:53Z, boot_id changed, three-file cmp OK |
| Disarm suppresses the fire | armed then stopped: no restore entry, boot_id unchanged |
| Survives a reboot | armed and enabled, board rebooted; fired 90s into the new boot, restored, rebooted again |
| No reboot loop | next boot logged SKIP_NOT_ARMED with the flag cleared |

Logs `/tmp/xr819-deadman-qualify-{armed,fired,disarm,persistent}.log` and
`/tmp/xr819-deadman-qualified-final.log`. The 1200s production deadline leaves
roughly 400s of margin over the ~750-800s boot-to-phase-end path, so it fires
several minutes after a hung 600s phase rather than during a healthy one. The
series arms before each phase, verifies the armed state in its preflight, and
disarms only after the phase's independent three-file/trace verification.

### Backoff correction: hardware qualification not obtained; host unfit today

The first baseline phase of the series failed and the single approved retry never
started, so **this correction has no hardware qualification**. It is qualified on
code and tests only.

Baseline phase (image `ee76be…`, the retained-HIF parent with **no** backoff
change) exited 22 on the 10s stall guard at ~509s of a 600s board-TX run;
aborted result 200 MB over 523.33s is not a throughput measurement. Ping at
failure 0/3. Board side showed no firmware fault: BH alive, datapath unlocked,
all four TX queues queued 0 / pending 0, TX miss 636, no assertion or exception.
Board dmesg carried `sunxi-mmc 1c10000.mmc: data error, sending stop command` at
t=387s and BA add/stop churn every ~2s between t=103s and t=155s. Intervals were
2-6 Mbit/s throughout, against 5.84 and 5.01 Mbit/s on the same image earlier the
same day. Logs `/tmp/xr819-backoff-attempt1-{series,baseline-first-run,baseline-first-recovery}.log`.

**A ~500s-class stall therefore reproduces on the unmodified baseline image.**
That is the direct reason the earlier 517s abort must not be attributed to the
backoff work, independent of the HIF fix having changed that failure surface.

Environment audit before the retry found the board and AP in exactly the morning
configuration - channel 6, EDCA 3/7/2/1504/200, 7/15/2/3008/200, 15/1023/3,
15/1023/7, Rates 0x3FCF, basic 0xF, HT on, AMPDU dens/spcn 5, long 4 / short 7,
powersave off, AP module build-id `ac263b6586ba1a801696b8764f74fda11ebb2ac6`,
20 MHz, 22 dBm, no observers, no trace instances, no iperf - but the **host** at
load 20.9/21.1/15.4 on 16 cores from an unrelated coreboot build. The host is
simultaneously the AP and the TCP receiver for a board-TX (`-R`) run, and the
failure signature fits a starved receiver rather than a firmware fault: the board
had empty queues while the socket sat at unacked 139, notsent 385168, rtt 10.4s,
rto 23s. The owner's build was left alone.

The retry was therefore gated on host quiescence instead of being run blind:
25 one-minute samples between 17:08 and 17:33 never produced two consecutive
one-minute loads below 4.0 (values ranged 2.95 to 27.43), so the gate aborted
with `HOST_NEVER_QUIET_ABORT` and no further attempt was made. Today's outcome is
**host/link fitness, not a firmware regression**; nothing about the candidate's
on-air behaviour was measured. Log `/tmp/xr819-backoff-series.log`.

Board left recovered and idle: three-file cmp OK, zero trace instances, events 0,
nop tracer, no iperf, deadman disabled and unarmed (`/tmp/xr819-backoff-final-board-state.log`).
The deadman never had to fire during the phases; its last board-log entries are
still from its own qualification. The units stay installed but disabled for the
deferred qualification run.

Archived: `/tmp/xr819-backoff-qualified-{candidate,baseline}.{bin,elf}`,
`/tmp/xr819-backoff-qualified-candidate.patch`,
`/tmp/xr819-backoff-qualified-build.log`, hashes in
`/tmp/xr819-backoff-qualified-hashes.txt`, phase and deadman logs as listed above.

Decision: the change is **kept in the working revision on code/test merit only**,
explicitly labelled not hardware-qualified. It must not be treated as a verified
improvement checkpoint, and the deferred qualification (baseline -> candidate ->
baseline, deadman armed per phase, host quiet) is still owed. Reverting is one
`jj abandon` of this revision; the archived patch reproduces it exactly.
