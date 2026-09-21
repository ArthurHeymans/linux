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

## Downlink collapse: the AP stops being acknowledged

### Method: this measurement needs a load gate and interleaving

The host is simultaneously the AP and the TCP receiver, so unrelated host load
corrupts every number below. A first serial bisect (clean Sep 7 image, HIF-only,
tip) read 6.62 / 4.51 / 3.48 Mbit/s board-TX TCP while host load climbed
0.46 / 1.85 / 5.38 from an unrelated `btrfs-endio`/`kcryptd` job. The same tip
image measured 6.00 Mbit/s at load 1.4 and 3.48 at load 5.4. That series is void:
its apparent monotonic regression was host load, not firmware. Every phase below
waits for two consecutive host samples under 3.0, phases alternate
vendor/Rust/vendor/Rust, and host load plus the AP's per-station rate are sampled
inside every phase.

### Matched, load-guarded pair (host load <= 0.89 in all four phases)

Identical boot, host module, AP, fixed board MCS5 and harness; only firmware
differs. 90s board-TX TCP, 30s board-RX TCP, then a 5/10/20/30 Mbit/s UDP
offered-load sweep in both directions.

| Phase | TX TCP | RX TCP | TX UDP 20/30M | RX UDP 20/30M |
| --- | ---: | ---: | ---: | ---: |
| vendor-a | 11.88 | 27.00 | 14.20 / 15.40 | 21.00 / 31.50 |
| Rust tip-a | 7.70 | 21.10 | 13.70 / 13.30 | 14.90 / 23.70 |
| vendor-b | 11.74 | 29.20 | 15.10 / 15.40 | 21.00 / 31.50 |
| Rust tip-b | 6.53 | 19.90 | 13.10 / 13.20 | 11.70 / 23.10 |

Host load was <= 0.89, 0.68, 0.51 and 0.81 respectively, so this pair is
comparable. Uplink capacity is close (Rust loses 4-5% at 20/30M offered where the
vendor loses 0.2%); the downlink is 26-45% short and the vendor is still not
saturated at 31.5 Mbit/s.

### The AP's rate collapses against the Rust board

Per-2s `iw dev wlp4s0 station dump` during each phase:

| AP tx bitrate | vendor-a | Rust-a | vendor-b | Rust-b |
| --- | ---: | ---: | ---: | ---: |
| MCS 7 (65 M) | 187/196 | 6 | 181/197 | 0 |
| MCS 0 (6.5 M) | 0 | 27 | 0 | 7 |
| AP tx retries (phase delta) | +2379 | +6178 | +2558 | +7251 |
| AP tx failed (phase delta) | **+0** | **+96** | **+0** | **+67** |

The AP holds MCS 7 and records **zero** failed transmissions against the vendor
board, twice. Against the Rust board it retries ~2.5x more, records real
failures, and falls back to MCS 0-5. This is the downlink deficit: MCS 0 is
6.5 Mbit/s PHY, which is what the 3.5-5 Mbit/s low-offer RX UDP phases deliver.
The board is not being acknowledged.

### BlockAck action census

`cw1200_ampdu_action` logging over a full 9-minute phase (mac80211 action
numbers: 0=RX_START, 1=RX_STOP, 2=TX_START, 3=TX_STOP_CONT, 6=TX_OPERATIONAL):

| | RX_START | RX_STOP | TX_START | TX_OPERATIONAL |
| --- | ---: | ---: | ---: | ---: |
| vendor-a | **0** | **0** | 164 | 0 |
| Rust-a | **96** | **84** | 12 | 12 |

The board's downlink BA session is started and stopped roughly every five seconds
with the Rust firmware, and never exists at all with the vendor firmware.

### Static finding: there is no downlink BA responder

- `cw1200_ampdu_action` in both the lab module and the stock tree driver
  (`drivers/net/wireless/st/cw1200/sta.c`) returns 0 for
  `IEEE80211_AMPDU_RX_START` and `IEEE80211_AMPDU_RX_STOP` without telling the
  firmware anything. The host has never programmed a downlink BA session.
- The vendor firmware therefore handles the ADDBA exchange internally: mac80211
  never sees an RX BA session with vendor firmware (zero RX_START events above).
  The Rust firmware passes protected management up through mac80211's
  `SW_MGMT_TX`/`RX_MGMT` path, so mac80211 negotiates the session and the
  firmware/MAC is never told about it.
- The vendor keeps four DTCM BA session records at `0x8e78` (`activity`,
  `peer_mac`, `tid`, `interface`, `timeout_1024us`, `timer`). The Rust firmware
  models that layout but **never writes it**: the only references are the type,
  the linker region and layout assertions. No code populates a session.

Hypothesis, not yet proven: with no BA responder programmed, the MAC cannot
answer the AP's A-MPDUs with a BlockAck, so the AP retries, DELBAs and
renegotiates, and its rate control collapses to MCS 0. That single defect would
account for the AP failures, the rate collapse, the 96/84 session churn, and both
the low-offer RX UDP collapse and the 23-vs-31.5 Mbit/s ceiling. Confirmation
would need a monitor capture, or programming the responder and re-measuring.

### Temporary service probe

`experimental-service-probe` exports counters MIB 0x100c words 0..=10 (marker
`RXP1`): passes, passes with a hardware-owned class-0 slot, completions,
published RX indications, drops at the 24 host-transfer cap, passes blocked on
output descriptors, passes within 4 KiB of the 0x7000 RX FIFO limit, and lifetime
maxima for outstanding host transfers and pending bytes. It changes no
scheduling, ownership or lifecycle behaviour, and is removed once read.

### Probe result: the RX data path is not the bottleneck

First probe phase (`/tmp/xr819-probe-probe-a.log`, image SHA256
`e6516d12619ac2b323ed81a9a0507cc957bf544aa693e434a2d9c70109dd213e`), per
snapshot interval:

| interval | passes/s | TX busy% | completions | rx indications | drops at 24 | near-full passes | max pending bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| board TX TCP (142s) | 4848 | 54.6 | 115422 | 40082 | 43 | 0 | 22304 |
| board RX TCP (40s) | 4461 | 65.0 | 13922 | 51913 | 0 | 0 | 22304 |
| TX UDP 5/10/20/30M | 4765-5789 | 27-49 | 6746-22869 | 385-924 | 0 | 0 | 22304 |
| RX UDP 5/10M | 6263-6807 | 0.3-1.3 | 59-61 | 7624-13015 | 0 | 0 | 22304 |

Both RX admission hypotheses are eliminated: drops at the 24 host-transfer cap
are effectively zero (43 in the whole TX phase, none elsewhere), and the RX FIFO
never came within 4 KiB of its 0x7000 limit. Frames that arrive are delivered;
what is missing is the BlockAck signalling above.

The probe also shows the TX pipe is hardware-owned only 27-65% of passes during
TX traffic and ~0% during RX traffic, so the TX path is not air- or
MAC-saturated: the uplink deficit is software/latency-bound.

Known limitation of this first build: `blocked_by_descriptor` was incremented at
the call site that hardcodes `publication_available = false` because a host
request or control indication is already pending, so it duplicated the
host-request count (37491 in the TX phase). The attribution now mirrors the
existing rx-path-diagnostics else-if chain, and words 0..=12 are the probe schema
with 13..=21 left to rx-path-diagnostics.

Probe repeat (`/tmp/xr819-probe-probe-b.log`) reproduces the admission result and
adds a second signal. Drops at the 24 host-transfer cap stay near zero (66 in the
143s TX phase, none elsewhere). But during the board-TX phase the RX FIFO reaches
**28296 of its 28672-byte limit** with 473 near-full passes, and RX publication is
blocked on output-queue capacity for **37491 of 107579** pending passes in the
first run and **44929 of 115053** in the second - 35-40% of the time. Probe-a
peaked lower (22304 bytes) with no near-full pass, so this pressure varies.

That is a second, independent defect: while the board transmits, the firmware-to-
host output queue (64 entries) holds TX confirmations and RX indications together,
`publication_available()` stays false, RX publication stalls behind it, and the
receive FIFO fills. A board that cannot receive while transmitting also misses the
AP's ACKs, BlockAcks and aggregates, which is consistent with the oscillating
4.4-11.3 Mbit/s uplink and with the AP's failed transmissions.

Two mechanisms are therefore live, and they are not mutually exclusive:

1. missing downlink BlockAck responder (static finding above) - the AP's
   aggregates are never properly acknowledged;
2. RX starvation behind the TX confirmation output queue - the board misses
   frames while it is transmitting.

The probe's RX admission counters were correct for both runs; only the printed
schema changed mid-series, so `blocked req/ctl/desc` in
`/tmp/xr819-probe-run.log` is misaligned and the raw words must be read with the
words 0..=10 mapping.

## Downlink BA responder: attempted, falsified in this form

The static finding above was tested directly by programming the receive block-ack
session from received ADDBA requests, in three increasing levels of fidelity, each
in an interleaved load-gated A/B against the unmodified tip.

| build | what it writes | AP tx failed | board RX BA starts/stops |
| --- | --- | ---: | ---: |
| tip (baseline) | - | +93, +95 | 44/30, 43/32 |
| shadow only | DTCM `BaSessions` + `MacPipeTail` | +96, +67 | 52/41, 52/40 |
| + registers | + peer MAC/TID, enable bit, scoreboard clears | **+297**, +99 | **51/41**, **40/27** |

**The downlink BA session churn is invariant at ~40-52 starts per phase in every
phase, with and without the responder.** Programming the MAC receive pipe from
the ADDBA request therefore does not make the AP's aggregates acknowledged, and
the hypothesis "the board is not answering A-MPDUs because the MAC was never
programmed" is falsified in this form. The shadow-only build additionally
regressed board TX (2.46 Mbit/s against 8.98), which is why it was abandoned
rather than iterated.

What survives is the *difference* in session ownership. The vendor firmware
consumes the ADDBA exchange itself (`bab_event_dispatch` builds the ADDBA/DELBA
signalling), so mac80211 never sees an RX BA session and the churn is zero. The
open firmware leaves the exchange to mac80211's `SW_MGMT_TX`/`RX_MGMT` path and
only added MAC programming on the side, which is a split-brain arrangement: two
owners, one agreement. Any future attempt must move the whole exchange into the
firmware - session ownership, the reorder buffer, and the signalling - rather
than adding a writer beside mac80211.

### Register map recovered for the vendor receive BA pipe

Vendor `pipe_setup_entry` writes MAC register space, not only the DTCM shadow.
The pointers resolve only with the container offset from
`xr819-decompilation/README.md` (main payload at `0x1ab8` in `fw_xr819.bin`);
`DAT_00002828` reads `0x04001680` (`initialized_low_mac_prefix`),
`DAT_0000282c` `0x09c01200`, `DAT_00002830` `0x09c00040`, `DAT_0000283c`
`0x04001d40`, and `DAT_00005dd4` `0x04008ad8`, which is the session table base
minus its `+0x3a0` field offset.

```text
0x09c00060 + pipe*0xc   peer MAC[0..4]        (stats_export_pipe_counters clears
0x09c00064 + pipe*0xc   peer MAC[4..6]         these to ff:ff:ff:ff:ff:ff, which
0x09c00068 + pipe*0xc   TID                    confirms the meaning)
0x09c01204 + pipe*0x20  (win_size << 4) | 0x80400000   block-ack window
0x09c01210 + pipe*0x20  four cleared scoreboard words
0x09c0011c              per-pipe enable, one of two bits at 0x10 + pipe*2,
                        mirrored to DTCM mac_phy_command_state + 0x44
```

The enable bit's two-way choice comes from comparing a per-interface byte at
`0x04003678 + if_id*0x98 + 0x481` with a global byte at `0x04003ab8 + 0x19`.

The responder implementation is archived as `/tmp/xr819-rx-ba-responder.patch`
and removed from the working source; the feature gate and its three parse tests
go with it.

## Real-AP rig: the Hoeve AP exposes an uplink defect the lab AP hid

The Intel AX200 lab AP is not a neutral instrument: it is also the host, its load
corrupts the numbers, and it pins the board's rate to MCS5. Testing against a
real-world AP with the host as a plain wired peer removes all three problems.

### Setup

- AP: `Hoeve Luitenant Halleux`, BSSID `94:83:c4:ba:5b:0e`, 2.4 GHz channel 1,
  WPA2/WPA3 mixed. Its BSSIDs changed since the older configs in this tree, so
  `/root/xr819-hlh-new.conf` on the board carries the current BSSID and PSK.
- The board associates with its wifi in the `wifi-test` netns and takes a DHCP
  address on the house LAN, so all board traffic is forced across the real AP.
- The **board** runs the iperf servers and the **host** opens every connection
  outbound: the host firewall blocks inbound iperf (an inbound attempt hangs
  indefinitely), but outbound connections pass. Measured 91.4 Mbit/s host-to-board
  over Ethernet to confirm the path.
- Same `cw1200_core.ko` for both firmwares, so only the firmware differs.

### Result: downlink is identical, uplink is not

Offered-load UDP sweep, 15 s per point, board as server, host as client:

| Direction / offered | Rust tip | vendor stock |
| --- | ---: | ---: |
| board RX (downlink) 5/10/20/30M | 5.24 / 10.50 / 21.00 / 31.50 | 5.24 / 10.50 / 21.00 / 31.40 |
| board TX (uplink) 5M | 1.08 (72% lost) | 5.28 (0%) |
| board TX 10M | 3.09 (49%) | 10.60 (0%) |
| board TX 20M | 3.74 (47%) | 19.60 (0%) |
| board TX 30M | 3.90 (48%) | **29.50** (0%) |

TCP, 60 s each: board TX 1.44 Mbit/s against vendor 28.5; board RX 0.107 Mbit/s
against vendor 13.5. Ping 20/20 in both.

Three conclusions:

1. **The downlink radio path is fine.** Downlink UDP is identical between
   firmwares at every offered rate, so the 23-vs-31.5 downlink deficit chased
   earlier is specific to the Intel lab AP, not a general firmware defect.
2. **The downlink TCP collapse is a consequence of the uplink.** Downlink UDP
   carries 31.5 Mbit/s at 0% loss while downlink TCP manages 101 Kbit/s; the only
   difference is that TCP needs the board's ACKs to travel upstream. So there is
   one root defect, not several: the firmware's TX path saturates near 8.5 Mbit/s
   with ~18% loss where the vendor reaches 29.5 Mbit/s at 0%.
3. **The lab AP masked it by pinning the rate.** The lab harness always ran
   `iw dev wlan0 set bitrates ht-mcs-2.4 5`, so it never exercised rate
   adaptation. The board's own `iw link` against the real AP reports
   `tx bitrate: 6.0 MBit/s` - a legacy rate - which is consistent with the
   ~8 Mbit/s uplink ceiling and with queue overflow producing the flat ~18% loss.

### Power-save is a real, separate factor

The lab harness also always ran `iw dev wlan0 set power_save off`; the first real-AP
runs did not, and the difference is large:

| | Rust, PS default | Rust, PS off | vendor |
| --- | ---: | ---: | ---: |
| board TX UDP 5M | 1.08 (72%) | 3.80 (4.6%) | 5.28 (0%) |
| board TX UDP 10M | 3.09 (49%) | 8.17 (17%) | 10.60 (0%) |
| board TX UDP 20M | 3.74 (47%) | 9.25 (19%) | 19.60 (0%) |
| board TX UDP 30M | 3.90 (48%) | 8.34 (18%) | 29.50 (0%) |

Forcing PS off roughly halves the loss and doubles the uplink, and the reported
state is then `Power save: off`. But it does not fix TCP (uplink 1.44 -> 1.83,
downlink 0.107 -> 0.101), so power-save is a contributor, not the root cause.
This is consistent with the architecture ledger, which already lists "power-save
behavior" as outstanding vendor-parity work.

### The Intel AX200 cannot be used as the observer

The plan to use the AX200 as a monitor while the Hoeve AP served as the device
under test failed: `iw` reported `type monitor, channel 1` and tcpdump ran, but
`rx_packets` never advanced and the pcap stayed at its 102-byte header. Reloading
iwlwifi and creating a fresh monitor interface made it receive - for a few
seconds: 105 frames in 14 minutes against roughly 8400 expected beacons, then it
went deaf again. iwlwifi monitor mode on this card delivers a short burst after
the driver reload and then stops. A reception self-check sampled over the first
6 s therefore passes while the capture is already dying. Air visibility needs
different hardware; an `ath9k_htc`-style USB dongle would do, and the module is
already loaded on this host waiting for one.

## RX output-queue reserve: Hoeve A/B rejects it

The probe-b signal (RX publication blocked on the shared 64-entry
firmware-to-host output queue for 35-40% of pending passes during board TX,
RX FIFO reaching 28296/28672 bytes) motivated `experimental-rx-output-reserve`:
8 output entries kept for receive indications, with TX confirmations and async
events gated through `async_*` capacity checks. Both A/B images also carried
the `RXP1` service probe; the verdict is throughput, not probe words.

Four Hoeve phases, same open boot `51cbe9ec…`, same BA-enabled diagnostic host
module `a237a79e…`, PS off, auto-rate, 60s TX TCP + 30s RX TCP + 5/10/20/30M TX
UDP sweep per phase. Logs `/tmp/xr819-hlh-res-{control-a,reserve-a,control-b,
reserve-b}.log`; images `/tmp/xr819-reserve-{control,reserve}.bin`
(`9cbc9863…` / `f1b44852…`, 318 tests each, rebuilt bit-identically on the day).

| Phase | TX TCP | TX UDP 10M | TX UDP 20M | TX UDP 30M | Board tx-rate view |
| --- | ---: | ---: | ---: | ---: | --- |
| control-a | 2.31 | 9.18 (12%) | 9.22 (21%) | 9.32 (21%) | MCS0/2, touches MCS5 |
| reserve-a | 2.28 | 7.10 (26%) | 6.94 (32%) | 6.90 (34%) | pinned MCS1, 1x MCS6 |
| control-b | 2.28 | 8.78 (16%) | 8.56 (19%) | 8.57 (21%) | MCS2, touches MCS6/3 |
| reserve-b | 1.30 | 4.06 (48%) | 4.07 (46%) | 3.70 (50%) | pinned MCS0/1 all run |

RX UDP ran at full offered rate to 31.5 Mbit/s in all four phases; downlink TCP
stayed collapsed (~50-400 kbit/s) in all four (wash: starved of upstream ACKs).
Pings 20/20 throughout; `RUN_EXIT=0` everywhere. The TX UDP 5M point is void in
reserve-a and control-b (board port-5002 UDP server dead before that section,
`0 bytes, 0/0` over a stretched 17.9s); valid only in control-a (5.24, 0.03%)
and reserve-b (3.59, 31%).

Both reserve phases are worse than both controls on every comparable point.
Withholding 8/64 output entries from confirmations throttles TX-confirmation
drain into host-input-credit starvation: fewer admissions, more loss, deeper
minstrel fallback. The probe pressure was real but this partitioning cures
nothing; the root loss (vendor 0% vs Rust 12-21% at equal offered loads) is
untouched by output-queue policy.

Orchestration notes: the first attempt died after control-a when its EXIT-trap
recovery reboot landed between the next phase's `wait_ready` and `scp`
(`Connection refused`, exit 2). The wrapper now settles on post-phase recovery
file state before starting the next phase; the resume's wrapper record expired
over the hour, so reserve-b ran standalone with identical arguments.

Archived: `/tmp/xr819-rx-output-reserve-rejected.patch` (hif.rs + hif_startup.rs
hunks, Cargo feature context), both binaries and their SHAs in
`/tmp/xr819-reserve-rejected-hashes.txt`. Reverted from the working source:
hif.rs/hif_startup.rs have zero diff vs the committed parent, Cargo.toml keeps
only the service-probe feature lines. The reverted tree passes 318 tests; its
probe-image build differs from the archived control binary only by codegen
(the archived control called the semantically-identical `async_*` wrappers with
reserve 0), which is expected and not a behavior change. The `RXP1` probe code
is retained for the next discriminator, not as a fix.

Next: rate control. Nothing on Hoeve sustains high MCS (controls briefly touch
MCS5/6 and deliver ~9 Mbit/s; vendor holds 25+ Mbit/s TCP). Compare TX
retry/fallback policy and per-rate attempt histories between the firmwares,
not just confirmation-rate distributions.

## Rate-feedback capture: the air is flawless, the service rate is the ceiling

A Hoeve run with an extended static host driver (confirmation events now carry
`rate_try[3]` alongside status/rate/ack_failures) settles tries-high-and-burns
versus never-tries-high in one measurement. Firmware
`8ef940a1…` (reserve-reverted probe control), module `a45b7beb…`, PS off,
auto-rate, no-host-BA driver (per-frame confirmations). Log
`/tmp/xr819-hlh-ratetry.log`, per-second board metadata
`/tmp/xr819-hlh-ratetry-metadata.log`, analyzer `/tmp/analyze-ratetry3.py`.

Method correction: the board monitor *resets* its counters after every
per-second emit (`counts.emit(); counts = Counts()`), so each
`BOARD_STATIC_FLOW` line is a one-second delta, not a cumulative snapshot.
Past and future analyses must sum lines within a phase window, never
difference endpoints. Phase windows map host `PHASE_START` times to board
`mono` through the `TCP_MONITOR_READY` anchor (first snapshot mono 48.67).

Result: every phase's confirmations are ~100% `status=0, tx_rate=21 (MCS7),
ack_failures 0-2, rate_try all zero` — first-try MCS7 success, with only a
1-3% tail of 1-4 retries at the same rate. One frame in ~30k at rate 20. Zero
`RETRY_EXCEEDED`, zero failures at any other rate. Minstrel held MCS7 for the
entire run (every confirmation's policy-head rate is 21), and it was right to.

Delivered throughput equals confirmations times frame size, one-to-one:

| Phase | Confirmations | Rate | Implied @1500B | iperf delivered |
| --- | ---: | ---: | ---: | --- |
| TCP-TX 60s | 8420 | 137/s | 1.65 Mbit/s | 1.61 |
| UDP-TX-5M | 1722 | 115/s | 1.38 | 1.34 (1714/5346 datagrams) |
| UDP-TX-10M | 5546 | 370/s | 4.44 | 5.86-7.10 (window-clipped) |
| UDP-TX-20M | 6868 | 458/s | 5.49 | 5.98 |
| UDP-TX-30M | 7238 | 483/s | 5.79 | 5.84 (7140 datagrams) |

(`queued ≈ confirmed` in every phase, and `net_dev_queue` exceeds admissions
by exactly the iperf-reported loss: e.g. 10M phase 9338 skbs in vs 5559
admitted.) So iperf "loss" is driver-queue overflow *before* firmware
admission, never air loss: frames that reach the MAC fly first-try at MCS7,
everything above the completion rate is dropped upstream and counted lost.
Only 4 MMC error events in the whole run.

The invariant is the publication cycle rate, not the frame rate: ~115
cycles/s at thin 5M batches rising only to ~120-480 frames/s as batches fill
toward depth 4 — i.e. roughly **8 ms per publish→air→complete→confirm→credit
cycle** against ~1 ms of MCS7 airtime for a full batch. Per-cookie trace
samples show ~2-20 ms admission-to-confirmation latency. The cooperative loop
round-trip is the ceiling; rate control, crypto waits (1.77%), and the
output-queue reserve are all exonerated as primary causes, and minstrel's
MCS7 stance is vindicated. The MCS0-2 parking seen in other runs' trajectory
is a secondary fallback under burstier air, not the throughput mechanism.

This also reframes the rejected reserve: throttling confirmation drain
directly throttles the only cycle that matters. The fix direction is the
staged-plan step 12 that the investigation has been circling since the start
— multiple hardware-owned frames / per-pipe credits so the MAC has work while
a completion is being serviced (the vendor PAS ring does exactly this) —
and/or cutting per-cycle loop hops. Next: break the ~8 ms into staged
admission→GO→completion→confirmation→credit latencies from the existing
per-cookie trace samples, then design the pipelining change against that
breakdown. With BA + depth the same cycle rate carries 2-4x the frames,
which is why BA-enabled runs reach ~9 Mbit/s under the identical cycle bound.

## Stage-latency capture: 1 ms host, 21 ms firmware queueing, 0.25 ms air

Full-capture static run (all trace rows, ~500/s over SSH without loss) of one
60s Hoeve UDP-TX-20M phase: 5.33 Mbit/s delivered, 47% queue-overflow loss
(23727/50958), firmware `8ef940a1…`, module `a45b7beb…`. Log
`/tmp/xr819-hlh-stages.log`, 13.8 MB metadata
`/tmp/xr819-hlh-stages-metadata.log`, analyzer `/tmp/analyze-stages3.py`.
(An earlier attempt of the same script ran the phase without `-R` and measured
a perfect 21 Mbit/s downlink instead — same radio and firmware receiving
flawlessly while TX stays capped.)

Sequential per-cookie matching (13 cookie ids, order-matched within each id,
5 s generation guard) yields 27,239 complete queue→write→confirm timelines:

| Stage | p50 | p90 | max |
| --- | ---: | ---: | --- |
| queued → SDIO write done (driver/handoff) | 0.9 ms | 1.5 ms | 4.4 ms |
| write done → confirmation (firmware + air + host RX) | 21.2 ms | 31.2 ms | 178 ms |
| queue → confirm total | 22.2 ms | 32.3 ms | 182 ms |

All 27,239 confirmations are status 0 (27143 at MCS7, 95 at MCS6, 1 at MCS5);
airtime per frame is ~0.25 ms. So ~21 of 22 ms is firmware-side queueing and
handling plus host indication delivery — against ~1 ms of host handoff. By
Little's law, 454 completions/s at 22 ms mean ~10 frames live in the pipeline
while the single-owner gate admits one batch per pipe: each frame waits ~2-3
serialized publish→complete→confirm→credit cycles (~8 ms each) in
pending/PAS before its batch GOes. Queueing amplifies the cycle; the cycle
itself is the disease.

(The first sparse-sampling attempt at this analysis produced garbage p50s —
first-8-per-second samples per event kind are independent subsets, so
cross-kind pairing is coincidental. Full capture was required. A side
correction: the `0/0` UDP-5M sections in the A/B were a dead board UDP
server, not a firmware behavior.)

Fix direction is now fully determined: staged-plan step 12, multiple
hardware-owned batches per pipe. It hides latency wherever inside the
firmware/host-RX path it lives, and depth measurements already show the
mechanism (fuller batches → more frames per identical cycle). The design
constraint found so far: publication currently refuses unless
`mac_pipe_state == 0` and uses the pipe's current-slot cursor, so pipelining
needs the real MAC multi-slot contract (which slot a second GO may use, how
pipe state tracks 2+ in-flight batches), plus per-slot watchdog/BA/retry
audits. Completion routing by (pipe, slot) and `occupied_slots`/`slot_owner`
already exist; the `contains_pipe` gate is the only publication blocker.

## Pipelining design written; implementation not started

The stage split (1 ms host, ~21 ms firmware queueing, 0.25 ms air) plus the
MAC window protocol (state 0→1 at GO in `finalize_staged_pipe`, 1→0 only on
full drain in `service_pipe_tx_success` walking `current..=last`, per-slot
completion identity already routed, `occupied_slots` already tracked) fully
determine the fix shape. It is recorded in
`xr819-firmware/tx-pipelining-design.md`: allow a second staged batch per pipe
behind `experimental-pipelined-publish`, stage after `last`, extend the window
and re-trigger GO, walk all outstanding slots on watchdog expiry, and audit
(not change) the per-slot completion/retry/BA paths.

The single blocking unknown is second-GO semantics on an active pipe
(latched window vs extended window) — a vendor-decompilation read
(`txp_fn_4155` at 0x101f4 and surrounding GO writes) first, then a hardware
probe if static evidence is inconclusive. No implementation exists yet, and
the retry/BA/rate subsystems stay frozen so the eventual A/B attributes only
to pipelining.

## Vendor scheduler read: one outstanding batch per pipe, like us

`txp_scheduler_run` (annotated-main 0xaa5e) builds its candidate mask only
from pipes with the +0xa3 byte clear, stages slots from the pipe cursor to the
new last, marks +0xa3/+0xa4, reloads the watchdog (5), and GOes
(`ring+0x14 = 1`) per staged pipe. `txp_pipe_tx_success` (0x9cdc) advances
`current` past each completed slot and clears +0xa3/+0xa4 plus pipe state only
on full drain (`current == last`, walking slots through `txp_fn_2441`).
`txp_fn_4155` (0x101f4) walks *every* outstanding slot on watchdog expiry.
Caveat: the three +0xa3 observations live in different table bases
(DAT_0000ab0c/ab0b8/10520) with a shared 0x6c stride, so cross-function
identity is structural, not address-proven.

If that reading holds, vendor's 29.5 Mbit/s is a faster cycle and/or deeper
batches — not multi-batch pipelining. Our own scaling already shows
throughput = depth x cycle-rate at a fixed ~8 ms cycle (no-BA ~5, depth-4
BA ~9). Consequence, recorded in `tx-pipelining-design.md`: depth-8 A/B
first (in-tree, no contract risk), firmware-internal cycle timestamps second,
multi-outstanding demoted to fallback. Our watchdog should still learn the
4155 full-window walk regardless — current-slot-only retirement is weaker
than vendor on any depth.

## Depth-8 blocked on stack; fixed-cycle model confirmed from existing data

A depth-8 candidate (base + list-first-depth-five/eight + depth-eight-ampdu,
319 tests pass) fails the ARM stack gate: 6992/6912, 80 bytes over, while the
depth-4 control is 6752/6912. Delta frames are all in the retry-planning path
with arrays scaled by `MAX_EXPERIMENTAL_AMPDU_DEPTH` 4→8:
`service_single_probe_runtime_inactive` 432→616,
`decide_retry` 144→216, `depth_four_selective_plan_for` 152→232 (plus the
fixed ~208 panic tail). Trimming 240B there means reworking by-value
`RetainedAmpduBlockAck` moves, the `[u32; 8]` return/error paths and small
array merges across inlined frames — multi-iteration surgery on the retry
path for a discriminator run. Images archived (`/tmp/xr819-depth8-candidate.bin`
`18f26262…`, `/tmp/xr819-depth4-control.bin` `8ef940a1…`, ELFs beside them);
no depth-8 hardware run until the stack fits.

The discriminator question (does throughput scale with depth at fixed cycle?)
answers from existing BA-enabled lab reports without new hardware: baseline
62114 heads / 230986 length (mean depth 3.72) delivering 5.84 Mbit/s
(487 1500B-frames/s) implies 131 cycles/s ≈ 7.6 ms; the command-first
candidate (60246/222682, depth 3.70, 5.69 Mbit/s = 474/s) implies 128/s ≈
7.8 ms. Same cycle across a scheduling change, throughput tracking depth —
the fixed-cost cycle model holds. Depth would buy throughput, but only after
the stack trim; the bigger lever is the cycle itself.

Next highest value is firmware-internal cycle timestamps (GO vs completion
IRQ vs confirmation publish vs credit return, reusing the SVC2 AES-timer
technique — no new MMIO) to find what fills the ~7 ms, then cut it. That
doubles throughput at every depth including depth-4-today.

## CYC3 itemizes the cycle: confirm->GO 3.5 ms dominates

Cycle-probe build `68514f0b…` (324 tests, stack 6760/6912) hooks both GO
triggers, per-pipe drain, and all three confirm publish sites behind
`experimental-cycle-probe` (CYC3 schema; never combined with RXP1). Hoeve
UDP-20M 60s, no-BA static driver: 5.36 Mbit/s, 47% queue-overflow loss.
MIB snapshots bracket the phase via board `cw1200/counters`; decoder
`/tmp/xr819-decode-cycle-probe.py`. Log `/tmp/xr819-hlh-cycle.log`.

Means (vendor µs ticks, pipe 0 — the only active pipe):

| Span | Mean | n | Meaning |
| --- | ---: | ---: | --- |
| GO -> drain | 1820 | 6976 | ~1 ms airtime (depth-4) + ~0.8 ms drain latency |
| drain -> confirm (global) | 371 | 27391 | routing/encode/publish is healthy |
| confirm -> GO | 3519 | 6976 | credit/admission/reservation — the target |

6976 GOs / ~60 s = 116 batches/s ≈ 8.6 ms cycle; 27391 drains = 3.93
members/GO. Host-side trace from the same run: driver handoff
queue->SDIO-write 0.9 ms, but mac80211 queues in instantaneous bursts
(inter-queue p50 0.01 ms) while firmware completes steadily — the 3.5 ms
confirm->GO contains a host supply round trip (confirm read, queue wake,
SDIO write) plus firmware staging, in unknown proportion.

Surgery plan (no hardware-contract risk — single GO on idle pipe preserved):

1. **HIF multi-dispatch**: `command::service_one` admits one request per
   pass (~0.2 ms/frame, ~0.8 ms per 4-frame refill). Vendor reschedules
   immediately while descriptors are ready; ours waits for the next pass.
   Drain N ready requests per pass when backlogged.
2. **Pro-active staging + immediate GO on idle**: reserve/stage the next
   batch from already-arrived frames while the current batch flies, so GO
   fires the same pass the pipe retires instead of ~17 passes later.
3. **Deep retained backlog**: keep all 30 contexts filled so host supply
   latency overlaps airtime instead of serializing into every cycle.

A one-line MIB lesson for future probes: the board monitor resets counters
after every per-second emit (deltas, not cumulative) — difference phase
windows, and validate the schema magic before trusting any number (an early
dry run decoded an idle phase correctly as all-zeros).

## Multi-dispatch A/B interim: no throughput effect, per-phase CYC3 works

`experimental-multi-dispatch` (up to 4 TX admissions per command-lane pass;
sync semantics identical) images qualified: control `22827d6c…` (refactored
single-shot) vs candidate `c652da52…`, both CYC3, stack ≤6768/6912. Hoeve,
no-BA static driver, per-phase MIB snapshots (11 per run).

multidispatch-a (candidate, raw log later clobbered by an accidental phase
rerun — numbers preserved here): TX TCP 1.37 Mbit/s; UDP 5.07 (3.7%) /
5.41 (41%) / 5.16 (41%) / 4.61 (43%) at 5/10/20/30M. Per-phase CYC3:
UDP-TX go_drain ~1.8-2.2 ms, confirm_go 4.3-8.4 ms, drain_confirm ~0.29 ms,
3.4-4.0 members/GO. Against control-a (TX TCP 1.29; UDP 5.14/5.45/5.49) the
candidate is indistinguishable to slightly worse — draining more admissions
per pass did not move throughput.

Two methodology notes. First, RX-phase windows correctly show ~idle spans
(single-digit GOs, multi-second confirm_go), confirming the full-run means
are idle-polluted and only per-phase windows compare. Second, wrappers keep
dying between phases with no record (third occurrence); phase logs are now
the only durable state, so archive a phase log before any rerun touches its
name — the rerun that produced this note clobbered the good
multidispatch-a raw log (its numbers above survive in this ledger).

## CYC4 pass partition: zero blocked passes, confirm->GO is host supply

Cycle-probe build `efefbee2…` (CYC4 schema; 324 tests, stack 6760/6912) with
the added pipe-0 end-of-pass partition. Hoeve UDP-20M 60 s, no-BA static
driver: 5.09 Mbit/s, 47% queue-overflow loss (23509/49501) — control parity.
Log `/tmp/xr819-hlh-cycle4.log`, MIB windows
`/tmp/xr819-cycle4-mib-{before,after}.txt`, decoder
`/tmp/xr819-decode-cycle4.py`, harness `/tmp/xr819-hlh-cycle4-run.sh`.

Differenced means (vendor µs ticks, pipe 0 — the only active pipe):

| Span | Mean | n | Meaning |
| --- | ---: | ---: | --- |
| GO -> first drain | 1974 | 6612 | first member's airtime + drain latency |
| drain -> confirm (global) | 383 | 25996 | routing/encode/publish is healthy |
| confirm -> GO | 3097 | 6612 | idle window before the next batch GOes |

310310 loops over the 61 s phase = 5087 passes/s; 6612 GOs = 108 batches/s
≈ 9.2 ms cycle; 25996 confirmations = 3.93 members/GO.

Pass partition for pipe 0: busy 288253 (92.9%), **idle-blocked 0 (0.0%)**,
idle-starved 22057 (7.1%). Busy is derived as `loops - starved - blocked`, so
the split is exhaustive by construction.

Over 310k passes and the whole run there was not one pass where pipe 0 was
idle with admitted-but-unpublished work (a PasQueued candidate with no
hardware owner, or a reservation waiting to trigger). Every idle pass was
genuinely work-free. Together with multi-dispatch's null result, the 3.1 ms
confirm->GO is host supply latency — confirm read, mac80211 queue wake, SDIO
write — not firmware admission, reservation or publication mechanics. Pass
counts are not wall-time shares (the idle path sleeps; ~2.8 ms per idle pass
against ~0.14 ms busy), but the zero is a count, so it does not depend on
that.

Caveat recorded for the next refinement: the classifier only sees
`host_tx_driver` state, so a request still sitting in the HIF input queue
reads as starved. `HostTxState` is Owned/Reserved/Confirming; the two counted
waiting states are the only admission-visible ones (mid-pass phases
Submitted..PendingEligible do not survive a pass). The cheap closure is a
transport-pending clause in the same pass hook — if it also reads zero, the
starvation verdict is airtight. Multi-dispatch's null result already argues
those frames are absent: admitting up to four requests per pass changed
nothing, and frames already in the firmware would have been published within
a pass or two of the pipe retiring, not 3.1 ms later.

Consequence for the active plan: neither multi-dispatch (measured null) nor
pro-active staging (nothing to stage) can remove the idle window, because the
frames have not arrived. At a fixed host request-response latency the round
trip is paid once per batch, so depth is the only lever that amortizes it —
consistent with the depth-first priority, still blocked on the ARM stack gate.
The remaining firmware-side alternative is to make the round trip overlap the
airtime (host-side queue depth / earlier confirmation publication), not to
re-order firmware publication.

Build recipe (reproduced byte-exact, correcting the earlier note that the
cycle probe was never combined with RXP1): the probe images were built with
the standard board-diagnostic set
`experimental-list-first-depth-four-ampdu,experimental-fast-loop,experimental-aggregate-rate-feedback,experimental-rx-path-diagnostics,experimental-cycle-probe`
— RXP1 *is* present in both CYC3 and CYC4 images. It is harmless:
`cycle_probe::populate` runs last in `encode_read_mib_data_response` and
overwrites all 22 words, so the CYC4 schema wins. Rebuilding the working copy
with that feature set and `tools/pack-sectioned-elf.py` yields exactly
`efefbee21fbd42a4aa867e81b55b81463675f34fe380616d4d047a66e4f1c69f` (ELF
`f9ace96f…`).

## Depth-eight stack gate cleared by dropping retry-path array copies (reverted, see the depth A/B below)

The depth-8 candidate had been parked at 6992/6912 on the ARM stack gate. The
whole excess was in the retry-planning path, whose frames scale with
`MAX_EXPERIMENTAL_AMPDU_DEPTH` 4 -> 8. The trim keeps the depth-sized member
array in the retained BA observation instead of copying it around:

- `depth_four_selective_plan_for` validates the walked chain against
  `observation.members` one member at a time and returns only the plan and
  reason mask (the returned `[u32; MAX]` was live in the two deepest frames);
- `apply_depth_four_selective_retry` and `finish_depth_four_selective_actions`
  take that array by reference, and `prepare_whole_ampdu_retry` returns a
  member count rather than its context array;
- the remaining indexed reads that could open a bounds-check panic edge on the
  deepest frame use `get`/`iter().take()`, and
  `rewrite_depth_four_member_table` takes a slice so the clearing call passes
  the empty slice instead of a stack temporary.

All of it is inside `experimental-depth-two/four-ampdu` functions, so the
feature-free image keeps every symbol byte-identical (only panic line-number
metadata in `.text` moves, which is the reviewed-manifest drift case).

Measured `rust_main` call chains: depth-8 A/B candidate 6824/6912 (depth-4
base 6712, cycle-probe build 6720, worst deeper combination 6864). Rebuilt
images: control `6a42cb37…` (standard diagnostic base + list-first-depth-four)
and candidate `de77947e…` (that base + list-first-depth-eight + depth-eight),
so the Hoeve depth A/B is unblocked.

## Depth A/B on the self-hosted Intel AP: depth buys nothing, trim reverted

The Hoeve A/B never produced a valid comparison because its AP died mid-session.
That AP is the home Flint 3 (OpenWrt, `192.168.0.145`) serving `Hoeve Luitenant
Halleux` as an MLO AP whose configured MAC is the `94:83:c4:ba:5b:0e` we pin.
`ath12k` failed to submit beacon templates, ran a firmware recovery that
`limit[ed] service ready radios to 2`, and only pdev 1/2 came back: MLD link 0
(the 2.4 GHz link, `94:83:c4:ba:5b:0e`) never returned, leaving `ap-mld0`
NO-CARRIER with Channel 0, 0 dBm and 0 stations. hostapd logged 15x `Failed to
set beacon parameters` first. The board therefore only saw neighbouring APs on
the same SSID. The `depth8-a` phase had already logged two authentication
timeouts before it associated, so its 12-outstanding-frames confirm timeout is
not attributable to the firmware, and the second pre-refactor attempt failed
outright at association (`ASSOCIATION_FAILED state=SCANNING`).

Reran on the host Intel AX200 AP (`nmcli connection up xr819-lab-intel`;
`xr819-lab`, ch6, fixed MCS5, BA-forcing driver `a237a79e`), which removes the
peer as a variable:

| run | mean members/aggregate | TCP board-TX | UDP board-TX 20M |
| --- | --- | --- | --- |
| depth-4 control | 2.93 (9292 aggregates / 27182 members) | 9.86 Mbit/s | 11.3 Mbit/s (8.9% loss) |
| depth-8, pre-trim | 6.98 (3003 / 20972) | 10.2 | 10.5 (9.7%) |
| depth-8, trimmed | 6.99 (3013 / 21073) | 9.46 | 12.0 (6.6%) |

All three runs were clean: 0% ping loss, every aggregate acknowledged, 0
invalid aggregate reports, 4-5 TX misses. Aggregate depth rose 2.4x while
delivered throughput stayed flat, so the cycle scales with aggregate depth and
the "fixed ~8 ms cycle -> ~2x at depth-8" model is falsified. The depth-first
plan is dead, and it explains why the earlier no-BA -> depth-4 gain (5 -> 9
Mbit/s) did not continue: it saturates by depth 4. The remaining ceiling is
per-member cost (~1 ms per member, roughly 4x the ~250 us airtime of a
1500-byte frame at MCS5) on top of the CYC4 host round trip. (Corrected by the
CYC5 rate sweep below: consecutive drain gaps measure 13-16 us and do not move
with rate, so there is no per-member cost. The surviving term is a fixed
~1.6-2.2 ms GO->first-completion latency.)

The trimmed image also failed to associate in 2 of 3 Intel runs
(`STATE=SCANNING`, exit 10) against 0 of 5 for the untrimmed image, the control
and the historical Intel runs. Three runs cannot prove a regression, but the
trim existed only to fit a depth-8 stack budget that is now worthless, so it is
reverted: `src/tx.rs` is back to the `tputyvot` content, the feature-free image
is byte-identical again (`4dbd65e4…`, stack 6592), and both the depth-4 base
(6760) and the cycle-probe build (6768) stay inside the 6912 budget.

Harness lesson from the same session: the A/B wrapper's `settle_recovery` used
`ssh` with only `ConnectTimeout`, so a boot window left it blocked on a
half-open connection for 83 minutes. Wrap that probe in `timeout` and add
`ServerAliveInterval`/`ServerAliveCountMax` before the next board session.

## CYC5 rate sweep: the cycle is airtime plus a fixed ~1.8 ms completion latency

CYC4 could not decompose its own cycle: the three spans summed to 5.45 ms
against a measured 9.22 ms cycle, and the missing 3.8 ms was read as an
intra-batch drain spread. CYC5 measures every boundary instead: GO -> next GO
(the cycle), GO -> first drain, consecutive drain gaps inside one batch, last
drain -> confirmation, confirmation -> first admission (host round trip) and
last admission -> GO (staging). It replaces the CYC4 words and adds an
admission hook in `dispatch_single_request`; pipe 0 carries the traffic and
other pipes only increment one witness word.

Method: one depth-4 probe image (`6adeb80d…`, stack 6760/6912) at four fixed
2.4 GHz MCS values on the host Intel AP with the BA-forcing driver, 30 s of TCP
board-TX traffic per point, MIB snapshots bracketing the traffic window. Every
point's rate is confirmed by the AP's own receive report, not by the setting.

| MCS | rate | members/batch | cycle | GO -> first drain | air/batch | residual | drain -> drain | drain -> confirm | confirm -> admit | admit -> GO | TCP |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 6.5 | 3.99 | 11712 us | 9926 us | 7710 us | 2216 us | 16.4 us | 167 us | 3211 us | 7130 us | 4.43 Mbit/s |
| 2 | 19.5 | 3.99 | 5423 | 4514 | 2570 | 1944 | 16.3 | 169 | 1855 | 1969 | 9.26 |
| 5 | 52.0 | 3.93 | 3449 | 2686 | 949 | 1737 | 13.7 | 174 | 1702 | 1171 | 11.40 |
| 7 | 65.0 | 3.13 | 8299 | 2223 | 605 | 1618 | 13.6 | 170 | 11380 | 984 | 3.87 |

Two conclusions, both correcting earlier ones:

1. **There is no per-member cost.** Consecutive drains inside a batch arrive
   13-16 us apart and that gap is flat across a 10x rate change, so the members
   leave the MAC back to back and the earlier ~1 ms/member figure and the
   3.8 ms intra-batch spread inference are both wrong. The drains arrive as one
   burst after the batch's whole airtime has elapsed, which is why the first
   drain is late while the rest follow immediately.
2. **The cycle is aggregate airtime plus a fixed ~1.6-2.2 ms latency between GO
   and the first completion.** That residual is nearly rate-independent
   (2216/1944/1737/1618 us) and dominates as the rate rises: airtime is 66% of
   the cycle at MCS0 but 28% at MCS5, while the fixed term is 50% at MCS5. It
   also explains the depth A/B: doubling depth adds airtime but not this fixed
   term, so throughput stays flat. Depth is deprioritised, not disproven.

Caveats recorded with the numbers: the per-event spans overlap batches, so
confirmation -> admission and admission -> GO do not sum into the cycle (the
host round trip is largely hidden behind airtime at low rates and only surfaces
at MCS5); drain -> confirmation undercounts (36-541 samples) because the span is
only recorded while the batch's drain window is still open; the MCS7 point is
host-starved rather than clean (79% idle-starved passes, confirm -> admit 11.4 ms,
members/batch 3.13) so it is a bound, not a rate point; and the earlier BA A/B's
UDP arm is offered-load-limited, so only its TCP arm carries the depth result.

Next: attribute the fixed GO -> first-completion latency. The cheapest
discriminator is to count cooperative passes and MAC-event services between a GO
and its first drain: many passes with no completion visible means the MAC
signals the batch late, few passes means the firmware is not being scheduled.

## The GO -> first-completion latency is MAC-side, not firmware scheduling

CYC5 now also counts, inside each GO -> first-drain window, the cooperative
passes, the MAC event-FIFO services, and (new) how many passes elapsed at the
first drain since the last MAC event service. Same depth-4 probe, same rig,
30 s TCP board-TX per point, join retried up to three times.

| MCS | rate | passes in window | per pass | MAC services/batch window | passes since last MAC service at the first drain | GO -> first drain |
| --- | --- | --- | --- | --- | --- | --- |
| 5 | 52.0 | 11.03 | 239 us | 3.2 | **0.00** (n=9620) | 2633 us |
| 0 | 6.5 | 52.18 | 190 us | 4.2 | **0.00** (n=2943) | 9926 us |
| 7 | 65.0 | 10.11 | 229 us | 3.3 | **0.00** (n=4338) | 2315 us |

The firmware is not starved and does not sit on a visible completion: it runs
10-52 passes across the window (190-239 us each), MAC events are serviced
throughout, and the first drain lands in the *same pass* as an event service for
every batch measured (mean 0.00, not a rounded near-zero). So the completion is
being delivered by a MAC event and drained immediately; the fixed ~1.7-2.2 ms
residual sits upstream of the firmware, between the GO trigger and the MAC
raising that event. With airtime, channel access and MAC aggregation machinery
all inside that span, the next discriminators are (a) a non-aggregate build at
the same rate, which separates aggregate-completion machinery from a per-GO
fixed cost, and (b) varying the per-pipe watchdog value the firmware writes at
GO, which moves if the completion becomes visible only on a MAC poll tick.

Also corrected here: the board's join path flakes about one run in three on
*any* image (`wpa_state=ASSOCIATED` without the four-way handshake). The same
untrimmed probe image failed twice in three attempts on one point and twice in
three on another in this session, so the 2-of-3 association failures that the
trimmed depth-8 image showed earlier were not evidence against that trim; the
trim was reverted for having no benefit, and that reasoning stands on its own.
The sweep harness now retries each point up to three times.

## CYC6 distributions: the excess is tail-heavy, and host supply is marginal

CYC5's means were tail-dominated, so CYC6 replaces the spans that cannot
compose (the reviewer is right that per-event confirm/admit spans sample a
different population than batches, so they are event latencies, not cycle
budget components) with histograms of the two gaps that matter, and drops the
"passes since last MAC service" word, which is zero by construction given the
lane order. Words: GO->first-drain histogram at 250/500/1000/2000 us,
confirm->admit histogram at 500/2000 us, plus the aggregates. Randomised MCS
order, 20 s cooldown, up to three attempts per point.

| MCS | TCP | mem/batch | GO->GO | GO->first drain | passes | MAC svc | <=500us | <=1000 | <=2000 | >2000 | confirm->admit >2000 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 4.45 | 3.99 | 11482 | 9942 | 52.4 | 4.12 | 0.1% | 0.0% | 0.0% | 99.9% | 2.5% |
| 5 | 4.45 | 3.52 | 8060 | 2611 | 11.6 | 3.32 | 1.7% | 2.6% | 32.4% | 63.3% | 9.8% |
| 2 | 4.38 | 3.62 | 9983 | 4471 | 21.7 | 3.71 | 0.0% | 2.0% | 3.2% | 94.8% | 9.2% |

Three corrections to the previous entry:

1. **The excess is not a fixed delay.** GO->first drain is broad and
   tail-heavy: at MCS5 it spans 250 us to >2 ms with 63% above 2 ms, and even at
   MCS0 it is 99.9% above 2 ms against ~7.7 ms of airtime. The mean of ~1.7-2.2 ms
   I reported is a tail average, not a constant. Report quantiles, not means.
2. **Host supply is marginal at MCS5 and run-dependent.** The same image and
   setting produced 11.1 Mbit/s with 21% idle-starved passes in one run and
   4.45 Mbit/s with 75% idle-starved passes in another (this sweep's third
   attempt after two join flakes). So the "cycle" at high rate is a mixture of
   MAC completion latency and host starvation, and any depth or rate law derived
   from single runs is unsafe. This also means the flaky-join retry can select
   the starved cell, exactly as the review warned: results must be reported by
   attempt index.
3. **No early-visibility signal exists to sample.** A class-0 completion is
   produced only by processing a MAC event: `service_single_probe_runtime_inactive`
   pops the event, the handler enqueues into the firmware completion ring and the
   scheduler lane drains it. The pipe words a pass might poll instead (cursor,
   `completion_word`) are firmware-written, not MAC-written. The firmware
   therefore cannot observe a completion before the event that carries it, and
   its own contribution is bounded by one `mac_service_pending` poll (190-240 us
   measured). The review's alternative hypothesis is not merely unproven here,
   it is not expressible in this firmware's data model.

Next: remove host starvation before characterizing the MAC term at all — a
supply that saturates the pipe independent of TCP (a UDP blast or a kernel-side
queue test) — and read the MAC event payload itself, because if the completion
event carries a hardware timestamp that is the only clock that can split air and
channel access from MAC report delay.

## CYC7 supply classification: supply is a real lever, but the MAC term dominates

CYC7 classifies every batch by whether the refill window before its GO contained
any idle-starved pass (pipe 0 idle with no admitted work at all), and reports
cycle and GO->first-drain separately for supply-clean and supply-starved
batches. One boot, two windows: a TCP reverse phase (window-limited supply) and
then a UDP blast (`-u -R -b 30M -l 1200`, higher offered frame rate), each with
its own MIB window. MCS5, depth-4 probe image `35a98659…`.

| window | TCP served | members/batch | idle-starved passes | clean batches | clean cycle | clean GO->first drain | starved cycle | throughput |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| TCP reverse | 5107 | 3.59 | 67.6% | 96.1% | 3773 us | 2655 us | 74446 us (n=201) | 5.63 Mbit/s |
| UDP blast | 11853 | 3.86 | 16.4% | 99.7% | 2538 us | 2151 us | 85010 us (n=35) | 8.49 Mbit/s |

Three conclusions:

1. **The cycle means were tail-dominated by starvation, as the review suspected.**
   Starved batches take 74-85 ms against 2.5-3.8 ms for clean ones, so a mean
   cycle mostly measures how often the host stalled.
2. **Supply is a genuine lever, worth about half the throughput.** The blast
   cuts idle-starved passes from 67.6% to 16.4%, the clean-batch cycle from 3773
   to 2538 us, and lifts TCP-measured throughput 5.63 -> 8.49 Mbit/s. Attacking
   the host path is therefore worthwhile, not cosmetic.
3. **But a hard floor remains below the host.** With 99.7% of batches
   supply-clean and only 16.4% idle-starved passes, the clean cycle is still
   2538 us and GO->first drain alone is 2151 us (85% of it), against ~0.9-1.0 ms
   of ideal airtime for 3.9 members at MCS5. So the MAC/air term dominates the
   supply-clean cycle and carries ~1.2 ms of excess over ideal serialization.
   Perfect supply alone cannot reach vendor class.

Harness lessons from this round, both worth keeping: the run script started only
a TCP iperf server, so the first UDP blast died instantly and produced a
three-second "window"; and a silently failing ARM build left a stale ELF whose
packed hash matched the previous image exactly, which is how it was caught.
Host `cargo test` passing does not prove the ARM firmware builds, because the
probe hooks guarded by `target_arch = "arm"` are not compiled on the host, so
the build result must be checked rather than the artifact hash.

## CYC9 depth buckets: GO -> first drain is a fixed per-batch cost plus airtime

CYC8 also refuted the loop-stall explanation: during traffic no cooperative pass
interval exceeds 1 ms (mean 192-265 us, 0% slow in both windows), so the ~1.2 ms
excess a supply-clean batch carries is not firmware scheduling. CYC9 separates
the remaining two candidates by bucketing GO -> first drain by the batch's own
depth (1..4), which yields the per-batch intercept and the per-member slope from
within-run depth spread rather than a rate sweep. Same two-window rig (TCP then
UDP blast), MCS5, image `e11036ff…`.

| window | depth 1 | depth 2 | depth 3 | depth 4 | fit |
| --- | --- | --- | --- | --- | --- |
| TCP reverse | 1825 us (n=210) | 2402 (n=229) | 2563 (n=435) | 2682 (n=6255) | 1788 us + 225 us x depth |
| UDP blast | 1334 us (n=29) | 1851 (n=373) | 1927 (n=813) | 2090 (n=11052) | 1503 us + 147 us x depth |

Ideal airtime per member at MCS5 with 1570 bytes is ~242 us, so the per-member
slope is essentially airtime (the low-end 147 us reflects few small-depth
samples and smaller frames), while the intercept is a **fixed ~1.5-1.8 ms per
batch that is not airtime and does not depend on depth**. That single term
explains most of the supply-clean cycle (2450 us at 3.87 members under the
blast), and it is consistent across the two windows to within ~285 us.

Combined with the earlier negatives - no per-member drain cost (13-16 us gaps),
no loop stall (0 intervals > 1 ms), no firmware poll delay (the completion
arrives with an event and the firmware drains it in the same pass), and no
visible early completion to poll - the remaining unexplained term is a fixed
MAC-side latency between the GO trigger and the first completion that is about
seven cooperative polls long at MCS5.

Next discriminators, in order: (1) a load sweep, because a fixed report delay
should be load-independent while a PHY ramp or channel-access cost should grow
with the idle gap before the GO - the depth-4 intercept is the comparable
quantity; (2) a monitor capture of the board's own bursts, which shows whether
the air actually starts ~1.5 ms after the GO or the burst is simply short,
requiring a spare monitor-capable radio (the host's ath9k_htc module is loaded
but no adapter is attached).

## The fixed per-batch term survives idle gaps and the watchdog knob

Two more probes against the fixed ~1.5-2 ms per-batch GO -> first-drain term.

**Duty-cycled supply (CYC10).** Same CYC9 image, three regimes in one boot: TCP,
continuous UDP blast, and a blast interrupted 1 s on / 1 s off so many batches
follow a one-second idle gap. Depth-4 GO -> first drain: 2989 us (TCP), 2227 us
(continuous blast), **2107 us (duty)**. A PHY ramp or recalibration cost would
grow after a long idle and a contention cost would fall in the quieter regime;
neither happened, so the term is not idle-dependent.

**Watchdog reload A/B.** The per-pipe watchdog is a countdown the firmware
reloads at GO, and 5 counts is suspiciously close to the term, so the probe
gained a build-time override (`XR819_PROBE_WATCHDOG`, production still reloads
5; symbol-level production identity verified unchanged - text hash moves only
through panic line metadata). Same rig, UDP window:

| reload | depth-1 | depth-2 | depth-3 | depth-4 | fit | clean cycle | throughput |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 2 | 1282 us | 1971 | 2041 | 2114 | 1693 + 106 x depth | 2490 us | 9.7 Mbit/s |
| 5 | 1638 | 1853 | 1947 | 2134 | 1505 + 157 x depth | 2500 | 9.7 |
| 10 | 2241 | 1876 | 2000 | 2100 | 1729 + 93 x depth | 2466 | 10.4 |

Depth-4 moves by 34 us across a 5x change in the reload, so the countdown does
not gate completion reporting.

The term has now survived every lever available without new hardware: it is not
per-member (the slope is airtime), not firmware scheduling (no loop interval
above 1 ms), not firmware polling (the completion arrives with an event and is
drained in that pass), not host supply (present in supply-clean batches), not
idle-dependent, and not the watchdog countdown. What remains is a MAC-internal
or on-air cost that is insensitive to our own load, and the decisive measurement
is a monitor capture of the board's own bursts: it shows whether the air starts
~1.5 ms after the GO or the burst occupies ~2 ms on air. That needs a spare
monitor-capable radio - adding a monitor interface to the AX200 while it runs
the AP is refused (`Operation not permitted`), and the host's ath9k_htc module
is loaded with no adapter attached.

Process note: the probe tests share one global counter block and cargo runs them
in parallel, so they now take a spin-lock guard; without it two tests that
reset and read the same counters produce intermittent false failures.

### Open contradiction: the depth model predicts a gain the depth A/B did not show

The CYC9 fit is linear and predicts that depth-8 would amortise the fixed
per-batch term over twice the members: 1500 + 8 x 157 = 2756 us for 8 members
against 2134 us for 4, i.e. roughly 1.3-1.5x throughput. The measured depth A/B
was flat (depth-4 control 9.86 TCP / 11.3 UDP; pre-refactor depth-8 10.2 / 10.5;
refactored depth-8 9.46 / 12.0). Both cannot be right, and the discrepancy is
unresolved. Candidate explanations, in the order I would test them: the depth
A/B was partly supply-masked (one arm ran 75% idle-starved, and we now know
supply swings run to run); the linear fit does not extend past depth 4 (the
depth-8 arms may carry mixed-rate members or more retries, which would raise the
slope); or the "intercept" is not a true per-batch cost but something that grows
with members in a saturating way. Re-running the depth A/B with the CYC9 probe
and the supply classification in the same window would settle it, but depth-8
does not fit the ARM stack budget without the reverted trim, so it needs either
the trim back or a smaller diagnostic base.

## Monitor capture: the air is efficient, and ~1 ms elapses between GO and first air

A spare AR9271 (ath9k_htc) was put in monitor mode on the lab channel and the
board's own bursts captured while it ran the probe image at MCS5 (`/tmp/xr819-mon.pcap`,
analyser `/tmp/xr819-analyse-monitor.py`). 47,246 board data frames grouped into
13,526 bursts, 73% of them exactly four members.

| quantity | measurement |
| --- | --- |
| members per burst | p50 4 (histogram 1:1243, 2:671, 3:1787, 4:9825) |
| burst span, first -> last member | p50 630 us, p90 750 us |
| member gap inside a burst | p50 215 us, p90 262 us |
| AP block-ack frames | 14,032 for 13,526 bursts (~1.04 per burst) |
| burst end -> block-ack | p50 2 us |
| burst end -> next burst start | p50 2141 us |

Three things follow.

1. **The aggregate is really one packed A-MPDU.** ~1.04 block-acks per four-member
   burst means one BA for the whole burst, not one per member, and the 215 us
   member spacing equals one frame's airtime at MCS5 (~1250-byte frames), i.e.
   the members are transmitted back to back with no dead air. The air is
   efficient and the AP acknowledges 2 us after the last member.
2. **Air occupancy is ~0.85 ms for four members** (span 630 us plus the first
   member's own ~215 us), so the 630-750 us span is airtime, not a stall.
3. **Therefore ~0.8-1.0 ms elapses between the firmware's GO register write and
   the first frame appearing on air**: the firmware's depth-4 GO -> first drain
   in the same window is 2073 us, minus ~845 us of measured air and ~200 us of
   completion/confirm latency. That is the fixed per-batch term localised at
   last - it is inside the MAC's TX-start path after the GO trigger, not host
   supply, not scheduling, not the watchdog, and not airtime.

Implication for the depth contradiction: if the fixed term really is ~1 ms per
GO, then amortising it over eight members should give ~1.5x, which is what CYC9
predicted and what the flat depth A/B did not show. The capture can adjudicate
without any firmware change, because it measures members per burst and air
occupancy independently of the firmware's own bookkeeping: run depth-4 and
depth-8 images under the same capture and compare members per burst and burst
spans. That needs a depth-8 image inside the ARM stack budget, so it needs
either the reverted trim or a smaller diagnostic base.

Caveat recorded with the derivation: the ~1 ms is a difference between a
capture-measured duration and a firmware-measured duration, which assumes both
describe the same batch population. Only durations are compared, so no clock
synchronisation is required.

## CYC11: the ~1.3 ms is MAC latency before its own start event

The static comparison pointed at the MAC's start path, so CYC11 timestamps the
boundaries the firmware can see after the GO: the type-`0x37` phase-2 start
event dequeue, `txp_pipe_tx_start` entry (with the retained PHY operation state),
PHY command 2 entry/exit, and the first completion. MCS5, whole-phase window,
16,971 batches, 3.79 members each, 8.5-10.4 Mbit/s:

| span | mean | n |
| --- | --- | --- |
| GO -> first drain | 2394 us | 16971 |
| **GO -> phase-2 dequeue** | **1312 us** | 16971 (100% of batches) |
| phase-2 -> first drain | 1082 us | 16971 |
| TX-start -> first drain | 993 us | 16971 |
| PHY command 2 | **0 dispatches** | — |

Two candidates die here:

1. **PHY command 2 never runs.** The retained operation state is 5 at every one
   of the 17,379 TX-start events (`state3=0 state5=17379`), so the rate-specific
   command-2 branch is skipped exactly as the vendor skips it in steady state.
   "Repeated command 2" and "our PHY state machine differs" are both refuted.
2. **The delay is before the MAC tells us anything.** The MAC raises its own
   start event 1312 us after our GO write, on every batch; the whole post-event
   path (handler, TX-start, air, completion report) is 1082 us, of which ~845 us
   is measured air, so only ~240 us - about one cooperative pass - is ours.

So the fixed term is a **per-GO MAC start latency of ~1.3 ms**, and nothing the
firmware does after the GO can change it.

Correction recorded while preparing the second experiment: the command-1 A/B was
dropped before spending board time. The vendor calls `pac_phy_start_op(1)` inside
`txq_build_aggregate_lists`, which `txp_scheduler_run` invokes on every pass with
a non-empty ring (`annotated-main.c:12780`), and its wrapper also restarts the
timer whenever the timeout is non-zero - so the vendor arms command 1 far more
often than our twice per batch, and our double call is not a delta at all.

### New lead: the MAC latency is per GO and per pipe, and we use one pipe

Measured `other-pipe events` has been 0-1 per window across every probe in this
series, so all our traffic is serialised on pipe 0. The vendor scheduler
computes an *idle-pipe mask* and maps access categories onto idle pipes
(`annotated-main.c:12760-12780`), i.e. it can have several pipes in flight at
once. If the ~1.3 ms is paid per GO per pipe, then publishing successive batches
on different pipes - the next batch started while the current one is still in
flight - overlaps that latency instead of amortising it through depth, which is
the one lever that does not depend on batch shape. That is the next experiment:
a two-pipe alternating publication A/B measuring throughput and the pipe
distribution.

## Dual-access-category test: a second pipe does not overlap the start latency

The CYC11 result says the ~1.3 ms is paid per GO, and every earlier probe showed
all traffic serialised on pipe 0 while the vendor scheduler can map access
categories onto idle pipes. That is testable without touching the firmware: run
two concurrent board->host flows with different TOS values so mac80211 routes
them to different access categories, and count GOs per pipe in the probe
(words 18..=21 now carry the pipe-0..3 GO counts).

BE (`-S 0x00`) and VO (`-S 0xB8`) UDP flows, 20 Mbit/s each, 30 s, MCS5:

| quantity | value |
| --- | --- |
| GOs per pipe 0..3 | **2825, 0, 0, 13835** |
| pipe-0 batches | 2825 (members/batch 3.65) |
| pipe-0 GO -> first drain | **10389 us** |
| pipe-0 GO -> phase-2 dequeue | **9156 us** (was 1312 us single-flow) |
| phase-2 -> first drain | 1233 us |
| TX-start events | 44266, all with PHY state 5, command 2 never dispatched |
| idle-starved passes | 37.4% |

So the second access category really does put traffic on a **second pipe**
(pipe 3), confirming the AC-to-pipe mapping is live and that our per-pipe
ownership gate permits two pipes in flight. But the two flows do **not** overlap
their start latencies: pipe 0's GO -> phase-2 rose from 1312 us to 9156 us, about
seven times worse, while its own post-event handling stayed at 1233 us. That is
the signature of a MAC start path that serialises across pipes rather than one
that runs per pipe in parallel, and it argues against the multi-pipe lever for
this bottleneck.

Caveats recorded, because this run is suggestive rather than clean: the harness
did not capture the two per-flow iperf summaries (the ssh tail printed empty
lines), so the delivered split is unknown; the probe decomposes the batch
timeline for pipe 0 only, so pipe 3's latency is unmeasured; and the two flows
run at equal offered load with VO enjoying EDCA priority, so some of pipe 0's
regression is ordinary voice-over-best-effort contention rather than proof of a
shared start engine. A clean re-run should capture both flow summaries and
extend the per-pipe decomposition to a second pipe; if the start latency is
shared, a single-pipe depth strategy remains the only lever, and the vendor's
29.5 Mbit/s would then need a different explanation than pipe parallelism.

One process note for the ledger: an inline python summary in the run wrapper
computed `after - before` with the operands swapped and printed negative deltas
(4294964471 for 2825); the numbers above come from the decoder, which diffs
correctly.

### Low-rate second pipe: the regression was contention, not a shared start engine

The dual-flow run above left the intended discriminator ambiguous, because a
20 Mbit/s VO flow both occupies a second pipe and hogs the medium. Repeat with
the second pipe present but lightly loaded: BE at 20 Mbit/s plus VO at
**1 Mbit/s**, 30 s, MCS5.

| quantity | single flow | VO at 20 Mbit/s | VO at 1 Mbit/s |
| --- | --- | --- | --- |
| GOs per pipe 0..3 | 16971 (pipe 0 only) | 2825 / 0 / 0 / 13835 | 8649 / 0 / 0 / 2877 |
| pipe-0 GO -> phase-2 | 1312 us | 9156 us | **1708 us** |
| pipe-0 phase-2 -> first drain | 1082 us | 1233 us | 1423 us |

The second access category drives a second pipe at both loads (2877 GOs on pipe
3 at 1 Mbit/s), so multi-pipe in flight is real. But pipe 0's start latency
tracks the second flow's *load*, not the mere presence of a second pipe: +30% at
1 Mbit/s against +600% at 20 Mbit/s. That identifies the earlier 9156 us as
medium contention rather than an inherently shared MAC start engine, and it
means the ~1.3 ms is substantially channel-access queueing - time the MAC spends
waiting for the medium before it raises its start event - on top of its own
preparation.

That reopens the multi-pipe lever in a different form: if the latency is mostly
waiting for the medium, then a second batch already staged on another pipe can
be transmitted as soon as the first finishes, without paying a fresh GO-to-start
round trip. What is still missing is the measurement that decides it - the
delivered throughput of each flow - because the dual-flow harness did not capture
either per-flow summary (both the host-side server logs and the board-side client
tail came back empty), so the split between the two flows is unknown.

## Multi-pipe control: the apparent gain was the second host flow, not the pipe

The dual-TOS run above delivered 12.6 Mbit/s in total (BE 7.35 at 34% loss plus
VO 5.24 at 0.14% loss), above any single-flow result, which looked like a
multi-pipe win. It is not: a second flow in the *same* access category is the
control, and it produces the same total.

Host-side reverse clients so both flows are board TX and the delivered rates are
reported on the measuring side, 30 s, MCS5:

| run | flows | delivered | total | GOs per pipe 0..3 | pipe-0 GO -> phase-2 |
| --- | --- | --- | --- | --- | --- |
| dual pipe | BE 20M + VO(EF) 5M | 7.35 + 5.24 | **12.6 Mbit/s** | 8649 / 0 / 0 / 2877 | 2856 us |
| **same pipe (control)** | BE 20M + BE 5M | 6.93 + 4.89 | **11.8 Mbit/s** | 11099 / 0 / 0 / **0** | 1126 us |
| single flow | BE 20M (UDP blast) | 8.5-10.4 | 8.5-10.4 | pipe 0 only | 1312 us |

The control confirms the routing (all 11,099 GOs on pipe 0, none on pipe 3) and
totals within 6.6% of the dual-pipe run - inside the run-to-run spread this
series has already shown - so **using a second pipe adds no throughput**. Two
host flows are enough to lift the total; whether they share a pipe or not is
irrelevant.

That also re-reads the two earlier dual-flow runs: with a heavily loaded VO flow
the BE flow's GO -> phase-2 latency rose to 9156 us because EDCA priority had
the VO flow occupying the medium and BE deferring to it, not because two pipes
contend inside the MAC. In the same-pipe control, where both flows are BE and
the medium is not pre-empted, the latency is 1126 us - the single-flow value.

So both structural levers are now measured flat: depth (the A/B) and pipe
parallelism (this control). The attention moves to what caps the *aggregate* near
10-12 Mbit/s at MCS5 when neither batch shape nor pipe count helps - the station's
share of the medium as scheduled by the AP, the per-GO MAC start latency floor,
or host supply - and the same caveat applies to this control as to the rest of
the series: one run per arm, so it rejects a large effect, not a small one.

## Unpinned-rate interleaved arms: the gap is loss, not rate

The 29.5 Mbit/s table above was measured on a real AP at unrestricted rate with
the same host module for both firmwares, while the lab rig pins MCS5. To find
which factor that table isolates, the lab rig was run with the rate *unpinned*
(`iw set bitrates` skipped, so rate control chooses freely) and the two firmwares
interleaved, two arms each:

| arm | delivered (UDP, 30M offered) | loss | rate the AP observed |
| --- | --- | --- | --- |
| vendor-a | 12.2 Mbit/s | 0.25% | 52.0 Mbit/s MCS 5 |
| ours-a | 8.95 | 22% | MCS 5 |
| ours-b | 5.58 (last window 1.43) | 18% | MCS 5 |
| vendor-b | 12.0 | 0.38% | MCS 5 |

Both firmwares sit at MCS5 even unpinned, so rate selection is not what
separates them here, and the per-GO MAC start latency cannot explain it because
the vendor pays the same one. What separates them is **loss at the same rate and
the same host module**: ~0.3% against ~20%, with one of our runs stalling for a
whole 10-second window. That is now the concrete, reproducible gap, and it also
retracts the "parity" reading briefly recorded after the matched fixed-MCS5
control: that control shows the open host module costs the vendor too (12.1
rather than 29.5), not that we are level with it.

The next run must capture the board-side driver counters (TXed / AGG TXed /
MULTI TXed / TX miss) alongside the host-side loss, because the open harness
filters them out today and they are what decides whether the 20% is board-side
admission loss or air loss.

## BA versus no-TX-BA with per-datagram sequence logging

To attribute the ~20% board-TX loss, the iperf flow was replaced with a paced
python sender on the board (1200-byte payload carrying a 32-bit sequence) and a
receiving logger on the host that reports loss and the run lengths of missing
sequence numbers. Two host modules were interleaved, same firmware and rate:
the BA-session driver and the no-TX-BA driver.

| arm | module | offered | received | loss | missing-run profile |
| --- | --- | --- | --- | --- | --- |
| ba-a | BA session | 30737 | 22333 | 27.3% | 1518 runs, mean 5.5; 1365 singles plus clusters at 64-69 |
| ba-b | BA session | 32555 | 21510 | 33.9% | 774 runs, mean 14.3; 576 singles plus clusters at 64-69 |
| noba-a | no TX BA | 23042 | 12336 | **46.5%** | 2220 runs, mean 4.8; runs of 3-6 dominate |
| noba-b | no TX BA | 21760 | 11628 | **46.6%** | 2139 runs, mean 4.7; runs of 3-6 dominate |

Findings:

1. **TX BA helps substantially; it is not the cause of the loss.** Losing the BA
   session nearly doubles the loss (27-34% -> 46.5%) and cuts delivered
   datagrams by ~45%. The static comparison's best-supported hypothesis, that
   the BA/window path is responsible, is not supported by this A/B.
2. **The module decides the transmission shape, and the probe sees it.** With BA
   there is ~1.04 TX-start events per batch (one A-MPDU per batch); without BA
   it is ~4.2 per batch, i.e. one per member, which is exactly the extra airtime
   overhead the throughput difference shows. TX-start -> first drain is also
   shorter without BA (310 us against ~955 us) because each start covers a
   single frame.
3. **The loss profile is bimodal.** The BA arms show many isolated single
   datagrams (1365 and 576) plus a handful of long runs clustered at **64-69**
   consecutive datagrams, which is the BA session/reorder window size; the no-BA
   arms show short runs of 3-6 instead. That is a real structural signature, but
   it cannot yet be read as air loss.

Confound recorded: the paced python sender sleeps between datagrams, so bursts
of loss can equally be board-side queue drops rather than air loss, and the
board-side driver counters are still filtered out of the harness. Separating
"never transmitted" from "transmitted and not delivered" needs the monitor
capture running alongside this flow, which is the next run. Note also that the
host firewall drops unsolicited inbound UDP, so the receiver must send a hello
first to open the return path; without it every arm reads zero received.

## Loss appears with load, in 64-datagram chunks

The sequence tooling was swept over offered load (BA-session driver, MCS5, same
firmware). Loss is essentially absent at low load and window-shaped above it:

| offered | sent | received | loss by sender/receiver counts | in-range gaps | missing-run profile |
| --- | --- | --- | --- | --- | --- |
| 2 Mbit/s | 6242 | 5830 | 6.6% (all of it a contiguous tail) | **0** | none |
| 5 Mbit/s | — | — | — | — | three consecutive join failures |
| 10 Mbit/s | 31191 | 22182 | 28.9% | 20.9% | 137 runs, mean 42.8, **60 of them exactly 64**, 17 singles |
| 20 Mbit/s | 34532 | 18625 | 46.1% | 36.4% | 910 runs, **692 singles**, ~94 runs of 64-68 |

Two readings matter here.

1. **At 2 Mbit/s the received sequence range has zero gaps**, so the per-frame
   path is clean when the pipe is not loaded; the 413-datagram discrepancy
   between the sender's count and the highest received sequence is a contiguous
   tail, which is a metric artefact to fix (the receiver derives "offered" from
   the highest sequence it saw, so a lost tail is invisible).
2. **Above capacity the loss arrives in exactly-64 chunks.** At 10 Mbit/s, 60 of
   137 runs are exactly 64 consecutive missing datagrams and they carry most of
   the loss; at 20 Mbit/s the same mode persists (~94 runs of 64-68) alongside a
   new population of 692 isolated losses, which is what ordinary queue overflow
   above capacity looks like.

Two host-side numbers found while chasing that 64: the driver creates four TX
queues of capacity **16** each (64 slots total, `main.c` `cw1200_queue_init(...,
16, ...)`), and our firmware advertises `input_buffers: 30` in its WSM caps
(`src/wsm.rs`), which caps how many frames the host may keep outstanding. Neither
is 64 by itself, but the AP's reorder buffer is 64, and the earlier AP-side trace
already showed open-firmware runs producing thousands of
`iwl_mvm_release_frames` drops against zero for matched vendor runs. That makes
"our PN/sequence handling under retries and requeues makes the AP drop whole
reorder windows" the leading hypothesis, and the AP PN/reorder trace the next
measurement rather than another blind A/B.

## The 64-datagram loss run, reproduced on today's build with the mechanism visible

The window tracer was re-attached and a single 10 Mbit/s sequence flow was driven
through it on the current build (boot `/tmp/xr819-open-boot.bin`, firmware
`/tmp/xr819-pipe-probe.bin`, BA-session diag module, MCS5). Both halves agree.

Sequence receiver: `SENT=31244`, `RECEIVED=20872`, in-range loss 25.4%, 181
missing runs, and **69 of them exactly 64 datagrams**.

Tracer, same window: every sampled slot mismatch has `delta=64` (16 of 16), and
the reorder buffer is effectively full when it fires (occupancy 60-64, with seven
events at 63 and seven at 64). The chain is explicit in one event:

```
SLOT_MISMATCH seq=1489 slot=1425 delta=64 head=1425 nssn=1426 size=64 count=63
PN_ADVANCE   seq=1489 pn=30170 stored=30105 head=1425 nssn=1426
SLOT_MISMATCH seq=1490 slot=1426 delta=64 head=1426 nssn=1491 size=64 count=63
PN_REJECT    seq=1427 pn=30108 stored=30171 ... slot=1427 count=62
PN_REJECT    seq=1428 pn=30109 stored=30171 ... count=61
   ... 20 consecutive rejects, seq 1427-1446, PNs 30108-30127 ...
```

Read that as: the AP wants sequence 1425 and has 63 frames buffered behind it.
The frame that arrives and is released is **1489 = 1425 + 64**, so it lands in the
physical slot belonging to the missing 1425. Its PN is legitimate and higher, so
the AP accepts it and moves the stored high-water mark to 30170. Every older frame
that was buffered legitimately (1427-1446, PNs 30108-30127) now fails the replay
check and is rejected - the loss run. The path-0 reject counter reached 1946 over
the run with bursts of up to 20 consecutive rejects.

So the defect is not our PN assignment, which the trace shows advancing normally;
it is that **we transmit at, and one past, the negotiated 64-frame window relative
to the oldest unacknowledged sequence**. Frame `head+64` is outside a 64-frame
window, and because the AP's reorder ring has exactly 64 slots it aliases the head
slot itself. That single fact explains the quantisation (the delta is always 64),
the load dependence (64 frames must be in flight), the absence of loss at low
load, and vendor immunity, since the vendor never exposes a hole to 64 frames of
advance.

The firmware question is therefore specific: where do we account for outstanding
members and the window limit, and what lets publication continue to `head+64`
instead of holding? That is a code audit, not another hardware run.

## Why the hole is never filled: no BAR, no fencing

The trace says a missing member is left behind while we run a full window past it.
Two code facts explain that and neither involves PN handling.

First, the BA planner gives up rather than fencing. `plan_depth_two_block_ack_actions`
in `src/tx.rs` maps an acknowledged member to `Confirm`, a missing member to
`Retry` only when the session is active and a retry is still allowed, and
everything else - missing with no retry left, or `OutsideWindow` - to `GiveUp`.
`complete_give_up` then retires the pipe slot and clears `LOW_MAC_PIPE_BUSY`
immediately, so publication resumes with the hole still unacknowledged at the AP.
There is no admission fencing while a member is outstanding, which is exactly the
behaviour the vendor presents as link states 6/7/8/10 refusing ordinary admission.

Second, the hole is never announced to the AP. A search of the firmware finds no
BAR generation at all: no `block_ack_req`, no BAR template, nothing that would
advance the AP's reorder head past the hole. The monitor capture of our own TX
confirms it on air - 10,881 BlockAck frames from the AP and **zero** BlockAckReq
frames in 76,336 frames total. Since iwlwifi releases its reorder window on a BAR,
the AP's head simply stays where the hole is until the ring aliases 64 frames
later, which is the loss run we keep measuring.

The host side is consistent with that reading: `cw1200_tx_confirm_cb` sets
`IEEE80211_TX_STAT_ACK` only when the confirmation status is zero, so a given-up
member is reported to mac80211 as not delivered and the host's sequence space
moves on regardless, while the AP is still waiting.

So the defect is a missing recovery action, not a corrupted one. The fix is either
to send a BAR when a member is given up (moving the AP's head immediately) or to
fence publication so the sender cannot advance to `head+64` in the first place,
and the vendor's own choice between those is the next thing to establish from a
capture rather than from our decompilation.

## The matched capture: the vendor never runs a TX BA session

The same harness, module, channel and sequence flow were run against the vendor
firmware with the monitor attached, so the two can be put side by side.

| | TX BA session | on air | loss | missing-run profile |
| --- | --- | --- | --- | --- |
| vendor | `action=2` then `action=3`, five times, `buf_size=0`, **never** `action=6` | unaggregated single frames, 44,360 per-frame ACKs, 41 BlockAcks | 0.43% | 97 of 98 runs are **singles** |
| ours | `action=0`, `action=2`, **`action=6` OPERATIONAL**, `buf_size=64` | 4-member A-MPDUs, 10,881 BlockAcks | 25.4% | 69 runs of exactly 64 |

Two conclusions follow, and the first retracts the BAR hypothesis.

The vendor does not send BlockAckReq either - its capture contains **zero**
subtype-8 frames, exactly like ours. It does not need to, because it never reaches
`IEEE80211_AMPDU_TX_OPERATIONAL`: mac80211 starts a TX BA session and stops it
again every time, so the AP never holds a reorder window for our traffic and a
lost frame costs one datagram rather than a window. Copying a vendor BAR behaviour
is therefore impossible, because there is none to copy.

What the comparison does show is that the defect is **amplification, not loss
rate**. The window tracer counted 211 slot mismatches across its trace while we
sent tens of thousands of datagrams - on the order of 0.2-0.7% of frames are
abandoned, which is the same order as the vendor's whole 0.43% loss figure. Our
air and retry path is roughly as good as the vendor's; what differs is that each
abandoned frame is multiplied by 64 because we keep a BA session open, leave the
hole, and run a full window past it. Recovering the hole - by announcing it with a
BAR, or by fencing admission so the sender cannot reach `head+64` - should take
our loss from 25% to roughly the vendor's figure without touching the RF path at
all. That is now the single highest-value change available, and it is a firmware
behaviour we owe the AP rather than a vendor behaviour we can copy.

## The recovery mac80211 already owns, and the one guard that blocks it

Tracing the hole back through the stack found the recovery mechanism already
written, in mac80211, and gated on a flag our driver never set.

`net/mac80211/status.c` sends a BlockAckReq by itself when a reported subframe
carries `IEEE80211_TX_STAT_AMPDU_NO_BACK`: it reads the failed frame's sequence
and calls `ieee80211_send_bar()`, which builds a compressed BAR at `seq + 1` and
sends it through the ordinary TX path. iwlwifi sets that flag with the comment
"single frame failure in an AMPDU queue => send BAR". Our driver's
`cw1200_tx_confirm_cb` set nothing of the sort on a failed member, which is why
both of our captures contain zero BlockAckReq frames even though the window
tracer counted 211 abandoned holes.

The driver now sets it for a failed QoS data member, with a `tx_ampdu_no_back`
counter in the debugfs output so the effect is observable. Building and running
that change on the board proved the mechanism and exposed the missing half: the
BA session came up operational, the sender emitted 31,250 datagrams, and the
host received **nothing at all**, with the radio's `tx_packets` moving only from
25 to 85 in thirty seconds. That is a wedged TX path, not a slow one, and it is
what happens when mac80211 now emits BlockAckReqs that the firmware refuses: the
request is dropped, the driver's queue entry never completes, and the host's
credit never returns.

The refusal is a length guard. `vendor_host_tx::classify_header` opens with

```
if frame.len() < 24 || frame.len() > u16::MAX as usize { return Err(Truncated); }
```

and a BlockAckReq is a 20-octet control frame (2 frame control, 2 duration, 6 RA,
6 TA, 2 BAR control, 2 start sequence), so it is rejected before any of its
fields are read. Everything downstream then assumes the ordinary-data shape too:
`assign_sequence` and the QoS branch are keyed off type bits that a control frame
does not have, and the payload length would have to be zero rather than
`len - 24`.

So the remaining work is a control-frame shape in the firmware's host TX
admission: accept type 1, set the 20-octet header length and a zero payload, skip
QoS/sequence assignment, and leave the frame unencrypted. That is the only thing
standing between the driver change and a real BAR on air, and until it lands the
driver change must not be deployed on its own, because refusing the BAR wedges
the link rather than degrading it.

## BlockAckReqs on air, and the 64-run mode collapses

Three changes were needed before a single BlockAckReq could reach the air, and
each one was found by its own failure.

The driver had to ask mac80211 for the recovery. `cw1200_tx_confirm_cb` covers the
frames the firmware confirms, but the abandoned members of our aggregates are not
confirmed at all - they are reaped by the queue TTL and reported through
`cw1200_skb_dtor`, which set no recovery flag either. Setting
`IEEE80211_TX_STAT_AMPDU_NO_BACK` in `cw1200_skb_dtor` for QoS data is what finally
produced BlockAckReq frames: twelve logged transmissions, at the basic rate
(`rate=0x00`), queue 2, `hdrlen=16`, and `tid=8` because a control frame carries no
TID for the driver's classifier.

The firmware had to stop refusing them. `vendor_host_tx::classify_header` now
accepts the 20-octet control shape (type 1, subtype 8) with a 20-octet header, no
payload, no QoS control, no sequence assignment, and the TID taken from the BAR
control field, which is what keeps it on the same EDCA queue as the data it is
repairing. Before that change the same driver wedged the link completely: the
refused request never completed, so the host's credit never returned.

With both in place the loss profile changed shape rather than size. The missing
runs collapsed from 140 runs of exactly 64 down to three, and the population became
singles and pairs (164 singles and 96 pairs of 353 runs). That is what releasing the
peer's window looks like: the hole now costs one datagram instead of sixty-four.

Throughput collapsed at the same time - the sender managed 5,607 datagrams in its
thirty seconds against the usual 31,000 - so the loss percentage stayed near 50%.
The link is no longer losing whole windows, but it is no longer carrying load
either, and separating those two effects is the next measurement. The candidates
are a BAR retry storm from mac80211 (a failed BAR is stored and re-sent when the
next unicast on the TID succeeds), the discarded-frame reporting now stopping the
TID queue, and the 44-octet skb the driver logs for a 20-octet frame.

## Correction: no BlockAckReq ever reached the air, and the marking caused the collapse

Static analysis of both codebases, checked against the sources, retracts two claims
recorded above.

First, the firmware classifier change is dead code. `TxRequest::parse` rejects any
frame shorter than 24 octets (`src/wsm.rs`), so a 20-octet BAR never reaches
`classify_header` and the control-frame branch is unreachable. Even if it were
reached, `command.rs` routes the request by `is_unicast_data()` /
`is_unicast_eapol()`, so a control frame would be handed to the management
publisher, which rejects anything that is not management or data. The monitor
capture agrees with that reading: zero BlockAckReq frames on air, which is what we
should have concluded at the time instead of crediting the classifier change for
the wedge disappearing.

Second, the marking that produced the counter and the collapse is wrong in a
different way. `cw1200_skb_dtor` is the driver's normal TX finalizer, not a
drop-only path: `cw1200_queue_remove` calls it for *every* TX confirmation. Marking
every QoS frame there sets `IEEE80211_TX_STAT_AMPDU_NO_BACK` on frames that were
delivered successfully, and mac80211 does not consult `acked` before issuing the
BAR (`net/mac80211/status.c`). That is why the counter reached 2,873 while the
failure-branch log never fired, and why the BARs were generated and dropped: the
firmware refused every one. The measured harm is a self-inflicted storm rather
than a repair - 5,507 datagrams sent against ~31,000, loss up from 25% to 47%,
`Used bufs` never above single digits, `TX TTL` zero, no queue locked or overfull.
The disappearance of the 64-datagram runs is therefore a side effect of telling
mac80211 that every frame failed its BlockAck, not evidence that a BAR repaired
anything.

The remaining work, if BAR recovery is still the goal, is three separate pieces in
the firmware (accept a short control frame in `TxRequest::parse`, give control
frames a publication path that accepts them, and handle the compressed BlockAck
response rather than a plain ACK), plus correcting the classifier's TID decode,
which must read bits 12-15 of the little-endian BAR control rather than the low
nibble. None of that exists today, so the honest baseline is the pre-change
behaviour: full rate with the 64-datagram runs.

## Reverted, and the baseline verified

The driver's `cw1200_skb_dtor` marking was removed and the remaining marking in
`cw1200_tx_confirm_cb` was gated on `IEEE80211_TX_CTL_AMPDU`, which this
(non-txq) driver never sets, so nothing fires. The classifier's TID decode now
reads bits 12-15 of the little-endian BAR control, matching mac80211's
`CBMTID_COMPRESSED_BA (0x0004) | (tid << 12)`, and the unit test uses the real
encoding (`0x3004` for TID 3) rather than the `0x1003` we invented.

One verification run of the same 10 Mbit/s flow, same firmware path, confirms the
revert restores the baseline exactly:

| | storm (marking on) | reverted |
| --- | --- | --- |
| sent in 30 s | 5,507 | **31,243** |
| loss | 47% | **21.0%** |
| runs of ~64 | 4 | **75** |

So the pre-change behaviour is back: full rate, and the 64-datagram loss runs are
still there. The BAR episode is closed as a regression that has been undone; the
real problem is unchanged from where it started, and any future BAR attempt must
first solve the four blockers the static analysis identified (the `TxRequest::parse`
length gate, the dispatch path that only admits unicast data, the management
publisher that rejects control frames, and the aggregate-only response handling
that would need the BlockAck class rather than a plain ACK).

## The retry route refuted: a hole is not a transient delivery failure

The cheaper alternative to BlockAckReq recovery was to stop abandoning members.
`selective_member_retry_rate` returned `None` as soon as the rate policy stopped
offering a step, which made a missing member unretryable and turned it into the
hole. It now spends up to sixteen same-rate retries past the policy's own limit
before giving up, which should recover any transient failure.

It does not. Same flow, same host module, one run each:

| | baseline | bounded same-rate retry |
| --- | --- | --- |
| sent in 30 s | 31,243 | 36,104 |
| received | 22,196 | 25,155 |
| in-range loss | 21.0% | 25.6% |
| runs in the 64-68 band | 75 of 64, plus 4 of 54 | **96** (22 of 64, 27 of 65, 27 of 66, 14 of 67, 6 of 68) |
| single datagrams missing | 5 | **727** |

Retrying at the same rate made the window-shaped loss *more* frequent and added
hundreds of singles, so the hole is not a transient delivery failure that more
attempts recover. Either the retry never reaches the air, or the peer cannot accept
the frame even when it does. The change has been reverted; the verified baseline
remains `eb2fa79b`, and the board is on recovery firmware.

That leaves the durable route - a real BlockAckReq with the four firmware pieces the
static analysis identified - and one cheap diagnostic before it: capture the air
during a run and check whether the hole's sequence number is ever re-sent as a
single MPDU. If it is never re-sent, the retry machinery is the fault; if it is
re-sent and still not delivered, the peer is refusing it and only a BAR can recover.

## The BAR plumbing is in, but the host is never told about the hole

The four pieces the static analysis asked for are implemented: `TxRequest::parse`
accepts the 20-octet control shape, `is_unicast_control` routes it into the host TX
driver, `classify_header` recognises the control frame before applying any data
header minimum, and `validated_tx_frame` accepts a 20-octet frame. The BAR's
response class is deliberately the no-response one (flags bit 9, frame kind 0xff),
because a BlockAckReq is answered with a BlockAck rather than an ACK and waiting for
the ACK class is what turned every BAR into a failure and a re-send.

With that firmware and a driver that marks `NO_BACK` on firmware-reported failures,
the run generated **zero** BAR requests and **zero** BAR transmissions, and mac80211
therefore sent nothing. The reason is upstream of all of it: the host is never told
that a member was abandoned. The driver only marks a failure inside the branch that
has already matched the confirmation to a queue entry (`cw1200_queue_get_skb`), and
the firmware's failure confirmations for given-up members are not reaching that
branch - the 3,200 "failed" confirmations counted earlier are counted at the top of
`cw1200_tx_confirm_cb`, before the early returns and before that match, so they were
never matched to an skb and never became a BAR.

That makes the abandonment report the real blocker, not the transmission path: until
mac80211 is told that a member was abandoned, it has no reason to ask for a
BlockAckReq, and no amount of firmware BAR capability will be exercised. The next
step is therefore on the driver side - find out why the given-up member's
confirmation fails the queue lookup (packet-id lifetime, or an aggregate
confirmation that carries only the head member's id) - and only then re-test the
transmission path, which is now in place.

## The loss is invisible to the host: every frame is confirmed as delivered

With the BAR-capable firmware and a driver instrumented at every exit of
`cw1200_tx_confirm_cb`, one run gave the answer in a single line:

```
TX confirm: 31202 ok, 0 fail      Conf unmatched: 0 (0 failed)
Used bufs:  0                     TX TTL: 0
stream:     SENT 31192, loss 55.6%, 145 runs of exactly 64
```

Every host frame is confirmed **successfully**, including the members that were
never delivered, so mac80211 has no idea anything was lost: it neither retransmits
(that is the firmware's job inside a BA session) nor asks for a BlockAckReq - hence
zero BAR requests even with the transmission path now implemented. Nothing is
"unmatched"; there simply are no failures to report.

The firmware does have the notion: the give-up action passes status `0x0b` and bumps
the `GIVE_UP` counter. But no failure confirmation ever reaches the driver, so either
the plan never classifies a member as given up - every member comes back
`Acknowledged` - or the confirmation is built with a zero status regardless. The
driver's existing `AGG report: ctl, len, ack, invalid` counters sum the aggregate
lengths and acked lengths the firmware reports, and comparing `ack` against `len`
during a run is what will separate those two cases: if they match while half the
datagrams are lost, the BA-driven classification is claiming every member acked.

That is the root defect this whole investigation has been circling: the firmware
hides its own losses from the host. Until a missing member is reported as failed, no
retry, no BlockAckReq and no host-level recovery can happen, and the 64-datagram
runs are simply what those hidden losses look like at the receiver.

## The firmware's accounting is honest: the peer acks frames it later drops

One sampled run with the aggregate counters read alongside the stream settles what
the false successes were:

```
AGG report: 8685 ctl, 30690 len, 30059 ack, 0 invalid
TX confirm: 30971 ok, 3 fail
stream:     SENT 31101, loss 19.2%, 215 missing runs (56 of exactly 64)
```

Of 30,690 aggregate members the firmware reports **30,059 acked** - 98% - with 631
members genuinely missing, and only three failure confirmations reached the driver.
So the confirmations were not lying about the BlockAck: the peer really did ack those
frames.

That is the mechanism, seen from the sending side at last. The peer acknowledges
every frame it *buffers*, including the frames sitting behind a hole, so the sender's
only delivery signal says "acknowledged" while roughly a sixth of the datagrams
(19.2% of the stream, against 2% missing members) are later discarded when the peer
releases its reorder window. No sender-side status can report that: the frames were
acked, and their loss happens afterwards inside the peer's reorder buffer.

Which is why retrying the hole could not help. The frames behind the hole are
already acked and already doomed; filling the hole only stops *new* frames from being
rejected, it does not recover the buffered ones. The only thing that removes the loss
is preventing the hole from accumulating a full window of advance, and the quantity
to bound is therefore **how far the transmit sequence runs ahead of the peer's
reorder head**, which the BlockAck's starting sequence tells us directly. Capping that
advance below 64 - holding publication instead of transmitting the frame that would
alias the peer's ring - keeps the peer's window releasable and turns each hole back
into a single lost datagram instead of a run of sixty-four.

## The hole trigger, and a rig that will not sit still

The localiser's result says the only signal that a frame was lost is the aggregate
metadata the firmware already sends: `ampdu_len > ampdu_ack_len` means the peer did
not acknowledge a member, i.e. there is a hole in its reorder window. Nothing else in
the system can see the loss, because every frame buffered behind that hole is
reported acked and is discarded later inside the peer.

So the driver now sets `IEEE80211_TX_STAT_AMPDU_NO_BACK` when the metadata reports
`len > ack_len`, on the head member, which makes mac80211 build a compressed
BlockAckReq at that frame's next sequence. That start sequence sits just past the
frames the peer has already buffered, so the peer releases and delivers them instead
of holding them until its 64-slot ring aliases. Combined with the transmission path
committed as `57549ffa`, this is the first configuration in which the recovery can
actually reach the air, and it needs no advance cap: the frames behind a hole are
recovered rather than prevented.

It is not yet measured. The rig stopped cooperating at the point where it mattered:
four consecutive association failures (`state=SCANNING`, against a documented rate of
about one in three), and the single attempt that did associate delivered 104
datagrams in its sequence flow before the sender and receiver gave up. The AP itself
is healthy on the host side (`xr819-lab-intel` activated, channel 6), and each failed
attempt returns the board to recovery firmware, so the block is the radio environment
rather than either codebase.

## The BlockAckReq is now generated and handed over - and then never completes

With the hole trigger in place the recovery path ran end to end for the first time:
the driver reported one `AGG BAR req`, mac80211 built a BlockAckReq, and the driver
logged `XR819 BAR tx: len=44 hdrlen=16 q=2 tid=8 rate=0x00` handing it to the
firmware. No wedge from a refused request, which is what the transmission path was
for.

What came next is the new problem. The same run stalled: 261 datagrams sent in 313
seconds against ~31,000 in 30 seconds normally, `TX confirm: 212 ok, 1 fail` and
`Conf unmatched: 0` over that whole period, and no `XR819 BAR confirm` line at all.
The BA session had already been torn down eight seconds after it came up. So the
BlockAckReq was accepted and published but never completed, and the host's TID queue
sat behind it - the same stall signature as the refused request, reached from the
other side.

A frame published with the no-response class has to be completed by the MAC's
transmission report alone, and that completion evidently never reaches the host.
Until it does, every BlockAckReq costs a stalled queue instead of a repaired window,
which is worse than not sending one. The next change is therefore in the firmware's
completion path for a no-response control frame, not in the trigger or the
transmission path, both of which now demonstrably work.

## The BAR is what stalls the link, in both response classes

Two runs isolate it. With the hole trigger enabled, the flow crawls: 261 datagrams in
313 seconds (no-response class) and 283 datagrams in 35 seconds (BlockAck class).
With the trigger disabled - same firmware image, same module build, same harness,
same flow - the link carries **37,059 datagrams in 30 seconds** at 18.3% loss with
833 missing runs. So the environment is fine and the stall is caused by publishing a
BlockAckReq at all.

Changing the response class from no-response (0x0200) to the BlockAck class (0x4000)
was therefore not the fix, even though that class is the semantically correct one and
the same class the aggregate path retires frames on. In both cases the frame is
published and never completed, and the host's TID queue sits behind it.

The next question is where it stops, and it needs the air: either the MAC refuses to
transmit the control frame - so no status ever arrives and nothing can complete - or
it transmits and the completion never reaches the host. A monitor capture during one
run distinguishes those, because the first shows no BlockAckReq on air and the second
shows one.

## The capture: our BlockAckReq never reaches the air

A monitor capture during a run with the hole trigger enabled settles the question the
last run left open. Frames involving the board or the AP, by kind:

- BlockAckReq from the **board**: **0**
- BlockAckReq from the **AP**: 15 (its own downlink requests)
- data frames from the board: 232, BlockAck from the AP: 63

So the driver handed BlockAckReqs to the firmware and the MAC transmitted none of
them, which is why none of them completed and why the host's TID queue sat behind
them. The first hypothesis was aggregation: a control frame cannot ride an A-MPDU, and
the host TX path is an aggregation path. The BlockAckReq was therefore routed to the
single-frame publisher instead (`command.rs` no longer admits control frames to the
host TX driver; `prepare_host_management_publication` accepts the 20-octet shape and
sets the BlockAck response class) and the image was rebuilt.

That did not fix it either. The next run handed over twelve BlockAckReqs, received no
confirmation for any of them, and still crawled: 1,052 datagrams in 33 seconds against
roughly 37,000 when the trigger is off. Because neither publisher produced a
completion, the fault is in the completion path rather than in how the frame is
published: whatever path transmits a control frame, the MAC's transmission report is
not being turned into a host confirmation for it.

The next measurement must therefore watch the firmware's own accounting - host
contexts admitted versus completed, and whether a transmission status arrives for the
control frame's slot - rather than the air or the driver, both of which have now been
eliminated as the cause.

## The completion gate, and why fixing it changed nothing

Static analysis (medium effort, one agent, full agreement with the code) found the
precise reason a published BlockAckReq never retires: `service_pipe_tx_success`
completes frames only when the slot's marker byte is `0xff`, and a BlockAckReq is
published with the BlockAck response class `0x0c` there, because that class is what
selects the expected response. Nothing else retires it either - a received BlockAck
does not synthesize a matching status report - so the frame stays in slot state 3 and
the host's TID queue waits behind it.

The fix is in: the success path now also completes a frame whose control field is a
BlockAckReq (`frame_control & 0x00fc == 0x0084`), which is safe because that event
proves the hardware finished with the context. The image was rebuilt
(`be704730`).

It changed nothing observable, and the capture explains why. **The MAC never
transmits the BlockAckReq at all** - zero on air while the driver hands them over -
so the physical-success event that both the old marker check and the new condition
depend on never happens. The completion gate was real and worth fixing, but it sits
downstream of a transmission problem: a 20-octet control frame handed to the MAC as a
single frame does not come out.

The remaining suspects are mechanical: the descriptor or MPDU length for a 20-octet
control frame being rejected, the MAC requiring control frames through a different
pipe class, or the vendor firmware sending BlockAckReqs through its own BA-session
machinery instead of the host path. The unused DTCM structures left over from the
vendor layout - `DTCM_BA_SESSIONS` with a `timeout_1024us` field and a timer,
`DTCM_PENDING_BA_LMC` - are suggestive of the last one.

## The transmission gate was the length: a 24-octet MPDU goes out

The measurement probe answered it in one run. Publishing the BlockAckReq padded to 24
octets - four zero bytes past the frame, which is exactly where the FCS belongs -
produced **51 BlockAckReq frames from the board on air**, every one with
`TA=12:42:2a:37:70:07` (the board), `RA=98:5f:41:18:76:17` (the AP) and
`ctrl=0x0004` (compressed bitmap, TID 0). The unpadded 20-octet frame produced zero,
twice. The same run also shows 4,779 data frames from the board and 1,594 BlockAcks
from the AP.

So the MAC will not transmit a 20-octet control MPDU, and accepts the same frame at
24 octets. That is consistent with the descriptor counting the four FCS octets: the
transmitted MPDU is then a complete BlockAckReq rather than a truncated one, which is
why the peer answers it.

The probe is therefore the fix for transmission, and it explains the earlier
contradiction: the frame had been published correctly all along and was simply never
put on the air, so no completion event could ever fire.

Completion is still missing - twelve BlockAckReqs handed over, none confirmed - and
the code now says why: `prepare_host_management_publication` publishes through an
internal class-6 context rather than the host's class-0 pool, so its completion cannot
produce the host's WSM confirmation for the packet id mac80211 is waiting on. The next
change is to keep the 24-octet length while publishing through the host path, whose
contexts do confirm; the two halves are now known separately and just have to be
combined.

## Combining the two proven halves

The transmission requirement and the completion requirement are now in one path. The
BlockAckReq goes back through the host TX path, whose contexts produce the WSM
confirmation mac80211 waits on, and `classify_and_encrypt` extends a control frame to
24 octets in place before classifying it, so the MPDU the MAC is handed is the shape it
will actually transmit. `classify_header` reports the MPDU length as the retained frame
length rather than trimming it back to 20.

The verification is inconclusive because the rig is not cooperating. The one run that
produced numbers had an effectively dead link: the sender offered 53,587 datagrams and
the receiver took 40, only 35 aggregate members reached the firmware, and no
BlockAckReq was generated at all, so nothing under test was exercised. Two further
attempts failed to associate, and the board was not even reachable for a cleanup pass
in between.

The rig now fails association most of the time and occasionally brings up a link that
carries almost nothing, after having been healthy enough twenty minutes earlier to put
51 BlockAckReqs on air. That pattern - join failures alternating with dead links, and a
module reload or reboot fixing it only temporarily - points at the board or its radio
environment rather than either codebase.

## Both halves are provable, but no single path does both

The isolation is unambiguous, with the same firmware image (`60e411dd`) and flow:

| hole trigger | result |
| --- | --- |
| enabled (host path + 24-octet MPDU) | 175 datagrams delivered in 35 s, dead link |
| disabled | **35,612 datagrams in 30 s**, 19.7% loss, 1,067 runs (973 singles) |

So the environment is fine and the BlockAckReq path is what stops the link: the frame
never completes and the host's TID queue waits behind it.

Each publisher solves exactly one half, and neither solves both:

- the **management publisher** transmits (51 BlockAckReqs on air, measured) but
  publishes on an internal class-6 context, so its completion cannot produce the WSM
  confirmation mac80211 waits on;
- the **host TX path** has class-0 contexts whose completions do produce host
  confirmations, but it publishes by aggregating into A-MPDU batches, and a single
  control frame does not complete through that path either.

What is missing is a single-frame publication on a host context: transmit the
20-octet frame with the four FCS octets the MAC requires, from the context pool whose
completion the host confirmation comes from. Nothing else is left in the mechanism;
the trigger, the length, the response class and the completion gate are all in place
and individually verified.

## The loss mechanism moves: 64-datagram runs all but disappear

With the BlockAckReq routed through the management publisher - which transmits it as a
single 24-octet frame and whose class-6 completion carries the host packet id back -
the run finally changed shape rather than merely stalling:

| | baseline (trigger off) | BAR recovery enabled |
| --- | --- | --- |
| sent in 30 s | 35,612 | 8,787 |
| received | 23,811 | 7,144 |
| loss | 19.7% | **12.5%** |
| runs of ~64 | 45 (plus 973 singles) | **3** (plus 456 singles) |
| counters | - | 61 BAR requests, 7,667 confirmations, `Conf unmatched` 0, `Used bufs` 3 |

The window-shaped loss is essentially gone: three runs of 64-71 against forty-five
before, and total loss down from 19.7% to 12.5%. That is the mechanism working - a
BlockAckReq makes the peer release the frames it had buffered behind a hole, so they
are delivered instead of discarded, and a hole costs one datagram rather than sixty
to seventy.

What is left is throughput: 8,787 datagrams against 35,612 on the same environment, and
`XR819 BAR confirm` never appears in the driver even though twelve BlockAckReqs were
handed over and every confirmation that does arrive is matched (`Conf unmatched` is
zero). Each unconfirmed BlockAckReq still holds a TID queue entry, so the sender paces
at roughly a quarter speed. The class-6 completion is therefore still not reaching the
host for this frame, despite the path preserving the packet id, which is the next
thing to instrument: count `HostManagementTxReport::Completed` emissions and the
management runtime's published/completed handoff.

## Two hypotheses ruled out by reading, and the trace reader located

While chasing why the management publisher's completion does not reach the host, two
plausible mechanisms were disproved from the code rather than from another run.

The host TX path does **not** aggregate a control frame: `ampdu_candidate_matches`
(`host_tx_policy.rs`) requires `frame_control & 0x008c == 0x0088`, i.e. QoS data, so a
BlockAckReq can neither head nor join an A-MPDU batch and `plan_ampdu_group` returns
`None` for it, leaving `aggregate_len = 1` and the single-frame publication the earlier
design note asked for. The earlier conclusion that the host path "batches into A-MPDUs
and never completes a lone control frame" was wrong.

The class-6 completion does carry the host identity: `service_host_management_tx`
returns `Completed { packet_id: runtime.host_packet_id, .. }` once
`service_single_probe_runtime_inactive` yields a completion, and `hif_startup` turns
that into the asynchronous WSM confirmation. So the path is right on paper; the
completion itself is not arriving.

For the next measurement the firmware already has a readable sink: the TX execution
trace (`TX_EXEC_TRACE`, twelve words, magic "TXEX"), which the driver prints from
`debug.c` when the magic matches. Trace points at publish result, `host_published`
arming, `service_single_probe_runtime_inactive` completion/no-completion, and the
handoff should show which of those links is missing without another air capture.

## Instrumenting the management handoff: built, wired, and one address short

The management publication path has no live observable, so the firmware now keeps seven
words of counters next to the TX execution trace: publications armed, publications that
stopped at a bisect stage with the last stage seen, completions seen with the last
status, service polls that found no completion, and entries into the waiting state. The
driver can read arbitrary memory through its `ahb` debugfs file, which returns `-EBUSY`
while the firmware is live unless the module is loaded with `unsafe_debugfs=1`, and
which otherwise enters reset/access mode and freezes the snapshot - so the words survive
a read taken after the flow has finished.

That path works: with `options cw1200_core unsafe_debugfs=1` the reads succeed and return
words. They just are not our words. Computing the address from the trace's link-address
delta assumed the driver's `XR819_TRACE_ADDRESS` (0x0900fd20) still describes this
firmware, and it does not: the current layout keeps DTCM state at 0x04000000..0x04009c44
(`check-dtcm-layout.py`), so that constant is stale and the delta landed in uninitialised
memory. Resolving a DTCM symbol to the address the SDIO host reads it at - the convention
the working probe counter reads already use - is the remaining step before the counters
can answer whether the publication armed or stopped at a bisect stage.

Two runs were also lost to the recurring association failure, one of them after
restarting the lab AP profile, which is worth watching because it coincides with the new
modprobe option.

## Fire-and-forget BlockAckReq: the shortest route around the missing completion

The BlockAckReq's *transmission* is already proven - its effect on the peer is what
collapsed the 64-datagram runs - so the only reason its absent completion matters is
that the driver holds its TID queue entry and mac80211 then stops the queue, which costs
about three quarters of the throughput. That makes the completion measurable but not
necessary.

The driver now retires the entry itself: `cw1200_queue_packet_id_for_skb` returns the
packet id of a queued frame (the queue's item structure is private to `queue.c`, so the
lookup lives there), and the TX path calls `cw1200_tx_confirm_cb` with a synthetic
success confirmation as soon as the hardware has taken a BlockAckReq. That reuses the
real confirmation path, so queue removal, skb destruction and mac80211 status are
identical to a firmware confirmation - and it needs no firmware change, no DTCM address
and no live firmware read.

Its verification run is outstanding: three attempts stopped at `wait_ready`, after which
the board's SSH resets every connection while still answering ping. That is the wedged
state documented earlier, and it needs a physical power cycle rather than another run.

Should the synthetic confirmation interact with a later real one for the same packet id,
the unmatched counter will say so; `Conf unmatched` exceeding zero after this change is
expected and benign.

## The fire-and-forget retirement retires the frame before it is transmitted

The change that was to remove the throughput regression - retire a BlockAckReq's queue
entry once the hardware has taken it - is placed before the hand-over rather than after
it, so it does not do what its description says.

In `cw1200_tx` the frame is queued under `ps_state_lock`:

    spin_lock_bh(&priv->ps_state_lock);
    BUG_ON(cw1200_queue_put(&priv->tx_queue[t.queue], t.skb, &t.txpriv));
    spin_unlock_bh(&priv->ps_state_lock);

Queuing is not transmission. The frame becomes hardware work only at the
`cw1200_bh_wakeup(priv)` call further down, which is what makes the BH thread run
`wsm_get_tx` -> `cw1200_queue_get` on that queue. The synthetic confirmation sits
between the two. It reaches `cw1200_tx_confirm_cb`, which calls `cw1200_queue_remove`,
which moves the item off `queue->queue` into the free pool - before the wakeup that
would have transmitted it. The BlockAckReq is therefore retired, dropped, and never put
on air, and the reorder-window release this path was supposed to buy cannot happen at
all.

Two further defects follow from the same placement. `cw1200_queue_remove` finishes in
`stats->skb_dtor`, which is `cw1200_skb_dtor`, which ends in
`ieee80211_tx_status_skb()` - that consumes the skb. The item has already been published
to the queue, and `spin_unlock_bh` has re-enabled softirqs by then, so a BH thread that
had picked the item up is copying from that skb into the SDIO write while the
confirmation path frees it. That is a use-after-free on the transmit path. Reporting a
frame as delivered from inside the driver's `tx` op is also a reentrancy class mac80211
normally only sees from a bottom half: `ieee80211_tx_status_skb` can run rate control
and queue wake logic while mac80211 is still inside `ieee80211_tx`.

The verification run is consistent with the first defect and cannot distinguish the
second. The board booted with firmware `bar11` (`914e11a3`) and module `f599ac94`,
associated in two seconds, answered twenty pings, and then took 38 datagrams in thirty
seconds against roughly 62,000 offered. Two of the 38 were lost in one run of two, so
the sample contains no 64-datagram run, but 38 datagrams is far too few to say anything
about the loss mechanism. The sender then stopped inside `send()`, its SSH session never
returned, and the board came back from the test image's reboot answering ARP and ping
with every TCP port refused. That is the wedged state again, not a measurement.

None of this contradicts the BlockAckReq's effect on the peer: the management-publisher
measurement that moved loss from 19.7% to 12.5% and reduced the 64-datagram runs from
45 to 3 stands on its own evidence. This is only about where the host retires the entry.
Retiring it has to happen after the frame has been written to the device, and it cannot
be reported as a success the air never carried. Module `f599ac94` should not be run
again as it stands.

## The run harness assumed a fixed board address

The run harness pinned the board at `192.168.0.104`, but the board takes its address
from the lab router's DHCP and had moved to `192.168.0.121`; `.104` was held by a
different device with a locally-administered MAC, which answers ping and is why the
earlier failures read as a dead board. The harness now resolves the board by MAC, proves
a candidate answers before using it, re-resolves across the boot window, and sweeps the
subnet only when the neighbour table has no entry for that MAC. It also refuses to flash
unless the three recovery files are present, and reports a failed recovery instead of
silently doing nothing.

The first probe of that change exposed a bad failure mode of its own: it treated "the
board does not answer SSH" as "the board cannot be found" and aborted with
`STEP_FAILED find_board` while the board was simply still booting after its recovery
reboot. A known address now resolves even when sshd is not up yet, and waiting for
readiness is left to the readiness test. Two more changes make a wedge cheaper to
diagnose: the setup and flow stages are bounded, so a stalled board is reported as
`WEDGE_DETECTED` in about a hundred seconds instead of consuming the whole outer
timeout, and the BAR counters are now sampled from the wired management path every two
seconds during the flow, so a board that dies mid-flow still leaves the evidence behind.

## A matched pair: the BlockAckReq buys the loss figure and then stops the link

The Sep 14 numbers cannot be compared against today's link, so the comparison was taken
again, next to itself, on the same board, firmware `bar11` (`914e11a3`) and flow. Only
the two places that ask mac80211 for a BlockAckReq differ: the hole signal
(`tx->status.ampdu_len > tx->status.ampdu_ack_len`) and the failure path in
`cw1200_tx_confirm_cb`. With both disabled nothing sets `IEEE80211_TX_STAT_AMPDU_NO_BACK`
and `AGG BAR req` stays at zero for a whole run.

| | no BlockAckReq | BlockAckReq |
| --- | --- | --- |
| sent in 30 s | 34,200 | 2,394 |
| rate | 1139/s, the documented baseline was 1187/s | dies at ~8 s |
| loss | 39.05% | 26.13% |
| runs in the 64-68 band | 75 (22x64, 15x66, 14x68, 12x65, 12x67) | 1 |
| `TX confirm` failures | 0-2 for the whole run | 276 and climbing |
| AP `rx packets` at the end | 19,747, still climbing | 1,311, frozen |

Both arms started from a clean baseline (5.97 ms and 5.28 ms average ping, 0% loss), so
the difference is not a settling artifact. The BlockAckReq does what it was built for -
loss 39.05% to 26.13%, and the window-shaped runs collapse from 75 to 1 - and it also
ends the run.

The stall has the same shape both times it was caught, and the counters date it:

    successes  34 -> 1547 -> 1547 -> 1547 -> 1547 -> 1547      (frozen)
    failures    0 ->  159 ->  184 ->  216 ->  246 ->  276      (climbing)
    AP rx      28 ->  282 -> 1311 -> 1311 -> 1311 -> 1311      (frozen)
    AGG BAR req 0 ->  127 ->  132 ->  138 ->  144 ->  150

and again, with a lower Bar count: successes freeze at 385, AP reception at 306,
failures climb 31 to 147.

The no-BlockAckReq arm also measures the false-success defect directly. Its failure
count stays at 0-2 while 39% of the frames never reach the AP: 31,547 firmware
confirmations against 19,747 received frames. The firmware reports success for frames
the air never carried. That single number explains most of the loss in both arms, and it
is why the BlockAckReq arm showed 1,547 confirmations at a moment when the AP had
received 282.

## Why a BlockAckReq stops the link: a full TID queue that never drains

Each TID queue holds 16 entries (`cw1200_queue_init(..., i, 16, cw1200_ttl[i])`), and
every BlockAckReq in these runs is queued on queue 2 (`q=2 tid=8` in the tx log). The
firmware never confirms one, and that is provable from the code rather than inferred:
`cw1200_tx_confirm_cb` logs `XR819 BAR confirm` for any confirmation whose frame is a
BlockAckReq, whatever the status, and that line has never appeared in any run while
`Conf unmatched` stays at zero. So nothing removes a BlockAckReq's entry through the
confirmation path at all; entries leave only when the GC timer expires them.

That makes the failure self-reinforcing. Sixteen unretired entries fill queue 2,
`cw1200_queue_put` sets `overfull`, `__cw1200_queue_lock` calls
`ieee80211_stop_queue(2)`, and the unlock needs the queue to drain to half its capacity
while the failure path keeps asking for more BlockAckReqs. The queue stops, the TID's
traffic stops with it, the AP sees nothing more, and the failures being reported are for
the frames already inside. That is the frozen-counter signature above, and it is the
same defect the fire-and-forget change was reaching for, only placed correctly this time.

## The retirement belongs after the hand-over, in the BH

The change committed as `ad4125b1` retired the entry in `cw1200_tx`, between
`cw1200_queue_put` and `cw1200_bh_wakeup`, which is before the frame can be transmitted,
and it freed the skb from the xmit path. The BH is where a confirmation really arrives,
and it already has everything needed to retire the entry there: `cw1200_queue_get`
stamps the driver's packet id into the WSM TX buffer (`(*tx)->packet_id =
item->packet_id`), and the 802.11 frame follows the header at
`data + sizeof(struct wsm_tx)`.

The driver now records the BlockAckReq's packet id in `wsm_get_tx` and, in
`cw1200_bh_tx`, retires that entry through `cw1200_tx_confirm_cb` once
`cw1200_data_write` has returned. That is after the frame is on the device, in the same
context a firmware confirmation arrives in, so queue removal, skb destruction and the
mac80211 status are the ones the normal path produces. Module `d6081b94`; its
verification run is next.

## The arm order is not what stops the link

The pair above is open to one objection: the control ran first, in the best conditions,
and the BlockAckReq arm that followed started from a link already at MCS0 with 266
retries before the flow. If running second were what stops a run, the pair would show
exactly what it showed.

Running it the other way settles it, with an AP reset before every arm. The BlockAckReq
arm went first, the control second, the BlockAckReq arm third:

| arm | order | baseline | outcome |
| --- | --- | --- | --- |
| BlockAckReq | 1st | 189.1 ms | stalled, successes froze at 178 |
| no BlockAckReq | 2nd | 6.2 ms | ran the full 30 s at 1078/s |
| BlockAckReq | 3rd | 37.0 ms | stalled, successes froze at 384 |

Four BlockAckReq runs have now stalled and both control runs finished the window,
including the control that ran last. The stall follows the BlockAckReq, not the arm
order.

## The loss figures do not reproduce; only the stall does

The pair's loss numbers should not be read as the size of the BlockAckReq's benefit.
Today the same configuration gives very different loss. The control lost 39.05% with 75
runs in the 64-68 band in one run, and 79.63% with no 64-68 band and a longest run of 82
in the next. The BlockAckReq arms gave 26.13%, 22.13%, 43.54%, and a 3.70% that only
covers 135 datagrams because the link stopped almost immediately. Window-shaped loss
appears in some runs and not others on the same firmware.

What reproduces is the stall: four of four BlockAckReq runs stopped with the same
signature and both control runs did not. The loss benefit is real - the window shape
collapses and the peer does release buffered frames - but a single run's loss percentage
from this rig is not evidence on its own. The claim in the section above that the
BlockAckReq takes loss from 39.05% to 26.13% should be read as those two runs' numbers
rather than as the size of the effect, and the 45-to-3 runs figure recorded on Sep 14
should be treated the same way.

## The stall, measured: a TID queue pinned above its unlock threshold by unconfirmed BlockAckReqs

The whole chain is now visible instead of just the shape of its end. The status file
already carried what was needed - per-queue `queued`/`pending`/`sent`/`overfull`/`locked`
and the hardware input-buffer pool - so the lifecycle could be sampled every two seconds
during a run without touching the driver at all.

With firmware `bar11` and the BlockAckReq triggers on, the run stalls like this:

    Queue 2: queued 9-11   pending 9-11   sent 178 -> 294   locked: YES   overfull: YES
    TX bufs: 30 x 1632 bytes     Used bufs: 9-11
    TX confirm: 168 ok (frozen)  failures 31 -> 148 (climbing)
    AGG BAR req: 14 -> 37        AP rx: 97 (frozen)

    XR819 BAR tx: 30 lines       XR819 BAR confirm: 0 lines       Conf unmatched: 0

Four things follow, and three of them correct earlier claims in this file.

**The queue is the mechanism.** Queue 2 is locked and overfull while nine to eleven
entries sit in it for the whole run. The lock is set at `num_queued >= capacity -
(num_present_cpus() - 1)`, which is **13** on this quad-core board and not the 16 that
capacity alone suggests, and it is cleared only when `num_queued <= capacity >> 1`, which
is **8**. Nine to eleven entries therefore sit permanently above the unlock threshold and
`ieee80211_stop_queue(2)` is never undone. Note also that queue 2 is an AC queue carrying
the data traffic itself, not a per-TID queue, so the entries that pin it are competing
with the flow rather than sitting somewhere harmless.

**What keeps them there is that a BlockAckReq is never confirmed.** The driver hands one
over thirty times in this run and logs each hand-over, and it receives no confirmation at
all - not one, at any status - while `Conf unmatched` stays at zero, so they are not being
dropped by the lookup either. Entries leave only when the GC timer expires them, and BAR
transmit lines continue past the stall at 128-134 s, so mac80211 keeps producing
BlockAckReqs even with the AC stopped, replacing whatever the timer frees.

**The hardware input-buffer credits are not the binding constraint.** Thirty buffers are
reported and only nine to eleven are in use. A credit is taken in `wsm_alloc_tx_buffer()`
before every `wsm_get_tx` and is returned when a confirmation arrives, so the theory here
was that an unconfirmed frame leaks one for good and `tx_burst = input_buffers -
hw_bufs_used` would decay to nothing once all thirty were gone. The next section records
that this theory was wrong: the credit comes back, and returning it from the BH
double-counts.

**The failure path does drain.** An ordinary failure removes its entry
(`cw1200_queue_remove`, txrx.c:1198); only `WSM_REQUEUE` with its flag set requeues. So the
queue is not clogged by failures that never retire. It is clogged by the BlockAckReqs that
each failure asks for and that nothing retires.

## Retiring the BlockAckReq entry: the queue half works, the credit half was wrong

The fix retires each handed-over BlockAckReq in the BH, after `cw1200_data_write` has
returned, through `cw1200_tx_confirm_cb`. Measured on the first attempt, the queue half
does what it was built for: queue 2, which sat at `queued 9-11` with `locked: yes` for a
whole run before, instead dipped, drained to `queued: 1, locked: no, overfull: no`, and
stayed unlocked. With the queue free the sender also reached its full paced rate,
51,182 datagrams in 30 s (1706/s), against about 1000/s in every stalling run.

The same attempt falsified the other half. The first version also returned the hardware
input-buffer credit with `wsm_release_tx_buffer(priv, 1)`, on the theory recorded above
that an unconfirmed frame leaks one of the thirty buffers. The run says the credit does
come back, and releasing it there double-counts. Three warnings on the BH kworker, in
order: `WARNING: bh.c:166 at wsm_release_tx_buffer`, which is
`WARN_ON(priv->hw_bufs_used < 0)`; `WARNING: queue.c:482 at cw1200_queue_get_skb`, a
confirmation arriving for an entry the synthetic one had already retired; and
`WARNING: bh.c:501 at cw1200_bh_rx_helper`, the RX path's credit-failure path. After that
the radio was dead - frames handed over froze at 56, AP rx froze at 68, and the AP's own
retries ran 2 to 359 while it dropped to MCS0 - so the station's receive path had aborted.

The credit return has been removed. The theory in the section above that an unconfirmed
frame necessarily leaks a buffer is therefore wrong as stated: whatever returns it, it
comes back, and 9-11 of 30 in use at the stall is what a working pool looks like, not a
leak in progress.

This arm also cannot be read as anything about the air, and that is worth stating plainly:
the radio died from this bug, so the run says nothing about the loss or the transmission
problem that remains once the queue half is clean.

The retirement sits *after* `print_hex_dump_bytes(..., data, ...)`, `wsm_txed(priv, data)`
and the sequence update. The earlier placement, immediately after `cw1200_data_write`,
left those uses reading a buffer whose skb the confirmation path had already consumed.

The confirmation reports success deliberately, and mac80211 is why: in
`net/mac80211/status.c` a BlockAckReq that is *not* acked is treated as failed and goes
through `ieee80211_set_bar_pending()`, which sends the request again on the next
successful unicast on that TID. Since that next request would also go unconfirmed, a
non-acked report would turn one leaked entry into a feedback loop. The air result is not
knowable from the driver either way, so success is the only report that terminates.

## The board's instability was self-inflicted, and the spin was an arithmetic bug

Three separate things were taking the board down, and none of them was the radio. Two
were ours, one was a stock Armbian cron job.

**`apt-get` on every boot.** `/etc/cron.d/armbian-updates` runs
`/usr/lib/armbian/armbian-apt-updates` on `@reboot` and `@daily`, and that runs
`apt-get upgrade -s -qq`. Measured on an otherwise idle board: 72-93% of a core, load
1.7-2.4, the SoC at 88-91 C, competing for the same SD card the rootfs lives on, and
holding apt's locks. This board reboots on every harness attempt, so it was doing that
constantly. `top` on the "idle" board was the tell: `PID 669 root R 92.9 %CPU apt-get`.
Disabled, and the board's idle temperature went to 54-60 C.

**The unbounded retry, and its real cause.** A stalled run spun in `wsm_get_tx` when a
queue's link map claimed work for the link mask and the queue held nothing to deliver.
The kernel log gives the rate: `cw1200_queue_get: 3460173 callbacks suppressed` in a five
second window, roughly 700,000 iterations per second, each taking `spin_lock_bh`. That
burns a core with bottom halves disabled, which starves that CPU's softirqs - including
the SD-card path under the rootfs - so sshd stops answering while ping still works and
only a power cycle brings it back.

The root cause is one line in `cw1200_queue_get_num_queued()`:

    if (link_id_map == (u32)-1)
            ret = queue->num_queued - queue->num_pending;

Both are `size_t`. `num_pending` had drifted **negative** - the status file showed
`pending: 4294967286`, which is -10 - so the subtraction wraps and reports twenty
available frames when there are none. That is the `link mask 0xffffffff` in the log, and
it is why the queue kept being selected forever. The spin and the accounting bug are the
same defect, and no radio conclusion drawn while it was live was trustworthy.

**The warnings were also a brake.** Two `WARN_ON`s fired on expected outcomes - "nothing
for this link mask" in `cw1200_queue_get`, and "confirmation for an entry that is gone"
in `cw1200_queue_get_skb` - producing 513 and then 448 full stack traces per run, about
128 lines/s. Behind a saturated 115200 baud console that accidentally throttled the spin
to ~5 iterations/s, which is why the board survived ~100 s while getting very hot. It also
means removing the warning and quieting the console made the board die *faster*: the
brake had been the logging. That is worth remembering before "fixing" a symptom again.

**And the harness hid its own state.** It logged the module it *intended* to install,
never the one installed at boot, and `recover()` failed silently - which is how a test
module survived a dozen reboots while the board was blamed instead.

Fixed: the retry is bounded; `num_queued - num_pending` cannot invent work; the two
decrements of `num_pending` cannot take it below zero and warn ratelimited when they would
have; both `WARN_ON`s are ratelimited one-liners; the synthetic BlockAckReq confirmation
runs in the BH loop rather than nested inside a transmit; the harness records the board's
actual installed module, firmware, namespace and process state before touching it, and
verifies its own restore with retries instead of rebooting into a test image; and each run
now reports true per-run counts (BAR hand-overs, BAR confirmations, BAR requests,
stale-queue hits) rather than six-line tails, which is what let a "12 BAR hand-overs"
artifact nearly pass for evidence.

Result: three consecutive runs completed with the board surviving, temperature flat at
65-68 C, no wedges and no power cycles. The board is finally an instrument.

## What the instrument shows about the radio

With the driver and the board healthy, the radio problem is plain and the numbers are
stable enough to act on: loss 30.36%, 60.57% and 70.29% across three arms, with 89, 89
and ~16 runs in the 64-69 band - the window-shaped loss, which remains the dominant term.
BARs are requested in the hundreds (339, 503, 990) and roughly 90% of them come back as
*failed, unmatched* confirmations (309, 448, 893). The `XR819 BAR confirm` log line, which
fires for a matched BAR confirmation at any status, is zero in every run.

Two readings have to be corrected before conclusions follow from that. First, "the
firmware reports the failure late" is an inference, not a measurement, and it was drawn
while the accounting was broken. Second, 64 UDP datagrams is not 64 802.11 sequence
numbers: BlockAckReqs and management frames consume sequence numbers too, so the
64-slot-reorder-ring story does not follow from a 64-datagram run - and runs extending to
71 need explaining rather than rounding.

To answer it without a kernel change, the receiver now logs arrival timestamps and reports
late arrivals grouped into bursts. A datagram whose sequence is below everything already
received is one the peer was holding; a BlockAckReq that advances its reorder window
releases them as a burst of old sequence numbers arriving together. That is local to the
receiver, so it needs no clock sync with the board, and it distinguishes "the peer
buffered and discarded this" from "we never transmitted it" - which is the question the
64-run loss has never actually been asked.

## The air settles it: three-point correlation, 2026-09-20 (xr819-air-a attempt 2)

First run with all three points at once: board handover counts, a monitor
capture on the AR9271 (`wlp198s0f3u1`, ch6, `/tmp/xr819-mon.pcap`, 8.9 MB),
and the AP/UDP receiver. Flow window 12:09:53-12:10:29 local, module
`16246b954cb9` (accounting fix), clean 4.8 ms baseline, `RECOVERY_VERIFIED`
afterwards. Attempt 1 was correctly discarded by the baseline gate (26/99 ms).

Flow totals: `SENT=34307`, `RECEIVED=18640` of `OFFERED=31840`, `LOSS=41.46%`,
978 miss-runs, max 71, 119 runs in the 64-68 band (66x37, 68x27, 67x26,
65x16, 64x13). `LATE=0 LATE_BURSTS=0` again.

| point | value | source |
|---|---|---|
| firmware claims delivered | 33,283 | `TX confirm ok` delta |
| unique MPDUs on air | 21,498 | pcap CCMP IVs, SA=station, flow window |
| air transmission attempts | 25,269 | same (3,771 retries, ~15%) |
| AP received | ~19,055 | `AP rx` 27 -> 19082 |
| UDP delivered | 18,640 | receiver |
| BARs handed to firmware | 366 | `bar_tx` / `AGG BAR req` |
| BARs on air | **0** | pcap, whole capture |
| matched BAR confirmations | 0 | `bar_confirm` |
| unmatched (failed) BAR confirmations | 333 | `Conf unmatched` |
| BAs from AP | 8,828 | pcap, flow window (aggregation active) |

Three readings, all lower bounds, all in the same direction:

1. **At least ~11,785 claimed frames never went on air** (33,283 claimed vs
   21,498 unique MPDUs; monitor miss rate at -29 dBm same-room is ~0-1%, and
   air >= AP rx corroborates the capture). The false-success defect is now a
direct measurement, not an inference: ~35% of claimed deliveries.
2. **~2,443 lost on air** (21,498 unique vs ~19,055 at the AP, ~11%).
3. **~415 dropped at the AP/host** (19,055 vs 18,640). `LATE=0` says the
   host reorder buffer released nothing: frames arriving behind the stuck
   window are dropped on arrival, which is the 64-run signature.

And the asymmetry is now on the record from the air, not the driver logs:
the firmware reports *success* for data it never transmits, and *failure*
(late, unmatched) for the 366 BARs it also never transmits. Aggregation
itself works - 8,828 BAs from the AP prove it. The defect is localized to
the firmware's control/data TX-status path: BARs handed to it die inside it,
and data MPDUs are claimed without transmission.

Caveats kept: AP `rx packets` includes non-data frames, so the middle column
is approximate; per-frame proof (802.11 seq vs WSM packet id) still needs
the monitor + driver timestamp join. But no per-frame join can move 366 -> 0
or 33,283 -> 21,498.

What this promotes: Sol's (ii) - re-test management-publisher BARs with the
burst probe, since the host-path BAR demonstrably never reaches air - and
(iii), the firmware TX-status/BA accounting fix, as the durable target.
Sol's (i), the lone-control-frame host path, is deferred further: there is
nothing wrong on the host side to fix; the frame dies in firmware.

## Management-publisher retest: zero BARs, and why (2026-09-20, xr819-mgmt-b)

Firmware `78880a43ad0c` (current source, management-publisher BAR routing,
feature-free) + acct driver, clean 4.5 ms baseline, full 30 s flow, board
healthy throughout (temp 65-70 C flat, `PRINTK_LINES` 9, `RECOVERY_VERIFIED`).
Attempt 1 of 4; the long AP settle (connected + 15 s beacon stability) fixed
the assoc race that ate the previous loop 0/4.

| | bar11 + acct (host-path BARs) | mgmt-bar + acct |
|---|---|---|
| BARs requested | 323-366 | **0** |
| BARs on air | 0 | 0 (none requested) |
| firmware claims | 33,283 ok / 367 fail | 14,254 ok / **12 fail** |
| AP received | ~19,055 | 9,593 |
| UDP | 18,640/31,840, 41%, 64-runs | 9,568/21,499, **55.5%, scattered** |
| unmatched confirmations | 333 failed | 0 |

`SENT=24521 RECEIVED=9568 LOSS=55.50%`, 1898 miss-runs, max 1424 (one edge
gap; the rest are singles/small), `LATE=0`. Queue 2 locked mid-flow
(queued=pending=11-13, overfull) and unlocked at the end - retirement holds.

The zero is the finding, and it is driver-side logic, not firmware: both BAR
triggers (hole-signal and failure-driven `NO_BACK`) read the firmware's own
report. bar11 reports hundreds of failures, which fire the triggers, whose
BARs it then drops (333 failed-unmatched). mgmt-bar reports 12 failures in
14k, so neither trigger ever fires and no BAR is ever requested. The
false-success defect therefore suppresses its own remedy end to end: the
firmware lies about success, the driver believes it, and the window loss goes
unrepaired (55.5% scattered loss, no 64-runs because no BAR ever probes the
window either way).

Consequence for Sol's order: (ii) cannot be tested by waiting for the driver
to request BARs - that waits on firmware honesty, which is (iii). The way in
is the capped trigger-independent BAR Sol also specified: a few BARs per run
requested regardless of what the firmware claims, through the
management-publisher path that demonstrably transmits. That splits "BARs
can't release the window" from "BARs are never asked for" - which this run
shows is the actual state.

Side note: this firmware was flashed despite failing the
`RUNTIME REGISTER BACKOFF LINKED DRIFT` gate (two extra refs to 0x04002088,
pre-existing at HEAD, no source change by me). It associated, passed traffic
for the full window, and recovered cleanly - evidence the gate manifest is
stale, not that the image is bad. The drift still needs its own reckoning,
but it did not predict behavior.

## The confirm path misread its own frames (2026-09-20, found live)

The forced trigger fired zero times across two flows (13k+ confirmations)
with correct code in the right place - verified by brace-depth audit,
rebuild, and a boot marker proving the image loads. The cause was one level
down: in `cw1200_tx_confirm_cb`, the queued skb carries the WSM TX header
prepended (`skb_push` in xmit, tracked in `txpriv->offset`, undone by
`skb_pull` in the destructor). So `skb->data` there is WSM header bytes, not
the 802.11 header, and all three frame-type checks in that function
classified garbage and never matched: the BAR-confirm log (which is why
`XR819 BAR confirm` never appeared in *any* run ever), the failure trigger
(which is why the "323 BAR requests" in the air run came only from the
hole-signal site), and the forced trigger. Fix: `hdr = skb->data +
txpriv->offset`, the same restoration the destructor uses. This was a
pre-existing diagnostic bug I copied, not a regression.

With the fix, first flow (`c3a6be02`, mgmt-bar firmware): `bar_tx=15`,
`bar_confirm=15` matched, `bar_request=8`, `retired_confirm=28`,
`dmesg_lines=85`. Fifteen BARs requested (8 failure-driven + ~7 forced),
all fifteen handed over and all fifteen confirmed matched - the first run in
which the BAR lifecycle completes on the host path. Queue 2 locked mid-flow
and unlocked at the end; board healthy, recovery verified. Loss 72.6%
scattered (no 64-runs), `LATE=0` - but this run had no air capture, so
whether those 15 BARs reached air is still open. The AP side looked sick
(retries frozen at 263, fell back to MCS0), so the loss number itself is
suspect this hour.

## Air verdict: zero BARs on air, aggregation torn down by the AP (2026-09-20)

Flow `xr819-forced-d` attempt 3 (attempts 1-2 correctly gate-rejected on
32/54 ms baselines), module `c3a6be02`, mgmt-bar firmware, with a 1200 s
monitor capture fully overlapping the 18:48:41-18:49:18 flow. The capture is
proven sensitive: it sees the ADDBA/DELBA exchange and EAPOL in detail.

| | value |
|---|---|
| UDP | 9,203/22,508, loss 59.1% scattered, max run 2137 (edge), rest singles |
| `LATE` | 0, no bursts |
| BARs handed to firmware | 16 (`bar_tx`), 7 failure + ~9 forced |
| matched BAR confirmations | 16 |
| **BARs on air** | **0** |
| station QoS data on air | 7,059 attempts, 6,784 unique IVs |
| BAs from AP | **0**; 10,335 plain ACKs |
| firmware claims | 13,966 ok / 25 fail; AP rx ~9,361 |

Two firsts, both negative:

1. **The management publisher does not transmit either.** 16 BARs in, 0 on
air, with a capture that sees management frames fine. Combined with the
morning run (host-path BARs: 366 in, 0 out), neither firmware path puts a
BAR on air today. The Sep-14 "51 on air" is not reproduced by current
source - whether by source drift since or by link state is open, but the
firmware's real verdict (failed, late, unmatched) is consistent in every
run: it never intends these frames to fly.
2. **There is no aggregation to release.** Six seconds into the flow the AP
fired a burst of a dozen ADDBA Requests within 2 ms, then a burst of
DELBAs; the station answered one ADDBA Response. dmesg matches (`BA
action=0 buf_size=64` then `action=1 buf_size=0`). The rest of the flow is
plain per-frame ACKs. With no aggregate session, the reorder-window theory
is inapplicable to this run's loss - and indeed there are no 64-runs, only
scattered singles. The evening link is unaggregated; the morning link was
aggregated with BAs flowing. Same rig, same day, different link mode.

The ADDBA-then-instant-DELBA pattern (a dozen unanswered requests in 2 ms)
also says the link is lossy for robust 6 Mb/s management, not just for
data: the station isn't answering the AP's management frames. Together with
two gate-rejected baselines (32/54 ms) before the one clean one, the
picture is a degrading RF evening, not a stable instrument. Evening loss
numbers (55-72%) should not be compared against morning numbers (20-40%);
only within-evening matched arms count.

What stands after today, in decreasing order of confidence:

- The host side is now fully instrumented and honest: bounded spin,
  ratelimited WARNs, verified recovery, true RUN-COUNTS, matched BAR
  confirmations, synthetic-vs-real double-confirm decomposition.
- The firmware confirms data it never transmits (~35% gap measured on air)
  and fails BARs it never transmits (0 on air, failed-unmatched late).
  Both halves of Sol's asymmetry are now air-backed.
- BARs, forced or triggered, change nothing measurable while they never
  reach air. Sol (ii) is blocked on transmission, not on will.
- The durable targets are Sol (iii) - firmware TX-status/false-success -
  plus the new ADDBA-teardown mechanism: who gives up on aggregation and
  why, given the Sep-14 link sustained it.

## Firmware TX-status chain, mapped (2026-09-20, source reading)

Sol (iii) starts here. Traced end to end in current source, all verified:

- Host status = `wsm_status_from_internal(hw_status)` (`tx.rs:10428`) -
  a near-identity map of a HARDWARE status word (reverse-engineered vendor
  table at `0x0000aff8`). The Rust layer does not invent success.
- It flows from `HostClass0Completion.status` <- `complete_tx_pipe_slot`
  <- the hardware completion drain, via `complete_context` ->
  `dispatch_completed_context` -> confirmation states -> TX_CONFIRM_ID.
- The Rust retry layer (`decide_retry`) only diverts to Rearm/GiveUp
  (policy exhaustion via `resolve_head_retry_step`, watchdog, BA branches).
  `SingleProbeMacBackend` is the ONLY production `SingleTxRetryBackend`
  impl, and every `CompleteSuccess` return in it sits in an
  aggregate/BlockAck branch. Ordinary frames have no software success path
  - their success IS the hardware completion status, passed through.
- `ack_failures` comes from the context `try_count`, incremented on rearm.

So for ordinary frames the firmware believes whatever the completion drain
yields, and the false-success candidates order themselves by testability:

(a) completion attributed without transmission (stale/default status word
or slot reuse) - fits the 35% air gap and the scattered shape;
(b) success-on-submit (completion raised at GO, not at ACK);
(c) misread ACK evidence (least likely - retries genuinely happen, ~15% on
air, and GiveUp paths do fire).

The single most suspicious point for the next dive: `complete_tx_pipe_slot`
writes the passed-in `status` as `terminal_status` with no ACK/BA-evidence
check for ordinary frames - so the check, if any, lives upstream of it, in
whoever produces that `status` u16 from the pipe/slot/MMIO state. That is
where the 11.7k phantom successes are born.

Discriminating experiment, no hardware changes: export the per-confirmation
`(status, try_count, ack_failures)` distribution through the existing
`host_tx_diagnostics` counters or the MIB `0x100c` words. If claimed
successes cluster at try_count==0/ack_failures==0 while air shows retries,
they were never attempted (a). If they show attempts, correlate with pipe
slot KIND/state at completion (b vs c). One firmware field ends the
guessing.

### Correction and implemented discriminator (2026-09-21)

The `try_count == 0` discriminator above was wrong: `try_count` counts rearms,
so a legitimate first-attempt success also has zero. The experiment now uses
an explicit identity-preserving lifecycle instead.

Feature `experimental-tx-status-lifecycle` tracks every ordinary host frame by
`(packet_id, context, pipe, slot, slot_generation)` across:

1. hardware publication;
2. `service_pipe_tx_start` (slot state 2);
3. `service_pipe_tx_success` (slot state 3);
4. an accepted ordinary status, including delivered/expected status and the
   observed slot kind/state;
5. `complete_tx_pipe_slot` return;
6. host confirmation publication.

The counters MIB starts with `TXLC` (`0x54584c43`). Aggregate stage counts and
identity/order failures occupy words 1–8. Words 9–21 preserve the latest exact
ordinary completion, including the 802.11 sequence number, packed inter-stage
deltas, and the raw event-FIFO words that caused start, pipe-success, and
accepted status, so it can be matched directly against the monitor capture and
the hardware producer can be identified. A non-zero identity mismatch proves
stale status or slot reuse. A complete
`0x3f` stage bitmap with no matching air sequence instead localizes the lie
below the software lifecycle: GO/MAC event generation or hardware status
production. The feature compiles out of normal firmware.

First hardware attempt exposed two harness/diagnostic mistakes rather than a
radio result. `ota-soak-run.sh` still hard-coded the board's obsolete `.104`
lease while the board was healthy at `.121`; the dynamic-address harness is the
valid runner. That run then returned `HWCK` (`0x4857434b`) instead of `TXLC`:
the old AES self-test snapshot had higher MIB priority than every host-TX
layout. `experimental-tx-status-lifecycle` now explicitly takes precedence;
the same image still completed a 30-second MCS5 flow and automatic recovery
verified the board's resting module afterward. The radio result from that run
is discarded because no lifecycle words were observable.

After two correctly rejected degraded baselines (148 ms and 369 ms average
RTT), a third arm was healthy at 4.9 ms and carried 26,378 offered sequence
packets. `TXLC` was finally visible, and exposed another instrumentation gap:
publication stayed zero while start/pipe-success/status/completion reached
15,440/15,440/14,544/14,544, making every later event an expected identity
mismatch. The normal two-frame scheduler uses `publish_in_batch`, while the
original hook existed only in two single-publication host-driver branches.
The hook now lives at the common successful
`HostSchedulerReservation::publish_in_batch` boundary, covering both single
and batched ordinary frames without double-counting. No false-success
conclusion is taken from the broken identity run; it validated the later four
lifecycle taps and the MIB schema.

The corrected image (`6365043157f5`) passed on the first healthy arm: 8.6 ms
baseline, 21,767 offered application packets, and verified automatic recovery.
Across the measured flow the firmware reported:

```
published          12712
started            13856
pipe_success       13856
status_accepted    12712
completed          12712
confirmed          12712
identity_mismatch      0
out_of_order           0
```

The extra 1,144 starts/pipe-successes are physical retry attempts; every
published frame eventually received accepted status `0x11`, terminal status
zero, and a host confirmation. The latest exact frame carried packet ID
`0x02020005`, sequence 2673, pipe/slot/generation `0/1/3177`, and the complete
`0x3f` stage bitmap. Its measured chain was publication -> start 3539 ticks,
start -> pipe-success 188, success -> accepted status 68, completion 2, and
confirmation 78.

This rules out stale slot reuse and every translated software attribution
boundary: no identity or ordering fault occurred in 12,712 completions. Yet the
AP's station counter advanced only about 8.2k packets while firmware claimed
12.7k successes, reproducing the roughly 35% gap. The false success is therefore
below `execute_popped_single_outstanding_event`: the MAC emits a phase-3
pipe-success and matching `0x11` completion event for frames the AP does not
observe. The next discriminator must capture the raw MAC event word and status
producer registers around those two events, not add more host-confirmation
logging.

That raw-event discriminator ran in image `d99eea484439` after rejecting one
71 ms baseline. The accepted arm had a 5.4 ms baseline, offered 25,154 packets,
and recovered automatically. During the sequence flow, publication advanced
14,143 while the AP station RX counter advanced 9,436: another 4,707-frame
(33.3%) claimed-success gap with zero identity or ordering faults. The latest
exact completion had these FIFO words:

```
start         0x0242b71a  type=0x37 pipe=0 phase=2 pipe marker
pipe-success  0x0243b71a  type=0x37 pipe=0 phase=3 pipe marker
status        0x0140b711  type=0x37 pipe=0 phase=0 completion marker status=0x11
```

So pipe-success and status are not two interpretations of one FIFO word. The
MAC emits a distinct ACK-class completion event 68 timer ticks after the
phase-3 event; start to phase-3 was 247 ticks, consistent with real frame
airtime rather than success immediately at GO. This eliminates
success-on-submit and narrows the defect to the hardware response decision:
either the transmitter fails to reach air but synthesizes `0x11`, or ACK
evidence is accepted for a frame the AP did not receive. Resolving those two
requires the already-configured AR9271 monitor interface on channel 6 to capture
the exact TXLC sequence concurrently; further software lifecycle stages cannot
discriminate them.

The first concurrent capture was healthy (5.0 ms baseline), sensitive throughout
the sequence flow, and observed 10,777 station attempts at about -32 dBm. The
firmware again completed a coherent chain for its latest sequence 387. That
sequence was on air, but it appeared four times because 15,553 completions wrap
the 12-bit sequence space:

```
sequence 387  PN 0x00000000018c
sequence 387  PN 0x00000000118c
sequence 387  PN 0x00000000218c
sequence 387  PN 0x00000000318c
```

The last occurrence is consistent with the latest firmware completion, but a
sequence alone cannot prove which wrap the lifecycle record described. The MIB
report therefore now uses magic `TXPN` and retains a four-entry ring of exact
`(sequence, CCMP PN, terminal status, retries, slot generation)` completions.
The 48-bit PN is copied from the encrypted frame at publication and is identical
to Wireshark's `wlan.ccmp.extiv`; it removes both sequence-wrap and retry
ambiguity. Four terminal samples give an 80% chance of containing at least one
missing frame at the measured 33% gap, and can be repeated without adding
another software stage.

The first `TXPN` capture produced the first exact per-frame proof. With a 5.2 ms
baseline, zero sender errors, 15,060 measured publications, and no identity or
ordering faults, firmware reported terminal success with zero retries for four
exact MPDUs:

| sequence | CCMP PN | monitor result |
|---:|---:|---|
| 2797 | `0x3af5` | present once |
| 1631 | `0x3667` | **absent** |
| 1632 | `0x3668` | **absent** |
| 1755 | `0x36e3` | **absent** |

The capture was strong (~-32 dBm), contained 11,401 unique station PNs during
the flow, and saw neighbors around each hole (for example `0x3660`–`0x3670`
with the claimed `0x3667/0x3668` missing). Sequence-only matches from earlier
wraps exist, but the exact reported PNs do not. This is no longer an aggregate
counter inference: three specific encrypted MPDUs completed through status
`0x11` and host confirmation without appearing on air.

The remaining split is whether the MAC skipped those MPDUs before RF or ran a
full-length transaction and then synthesized/accepted false ACK evidence. The
same four-entry ring can pack start-to-phase-3 and phase-3-to-status deltas in
place of the now-redundant terminal-success/retry fields. A short first delta means skipped transmit; a normal airtime-sized first delta
moves the defect into RF/ACK/status production.

A tail-sampled timing arm was correctly treated as calibration only: all four
exact PNs were on air. It established normal full-size-frame start-to-phase-3
values of 218–271 ticks. Status followed either quickly (7–8 ticks) or after
70–79 ticks, so status delay alone is not an air-presence discriminator.

Sampling every 2,048th completion removed the drained-tail bias. The first
monitor arm had inadequate RF coverage and was rejected even though tcpdump was
running. After resetting the AR9271, the repeat had a 4.3 ms baseline, 0 kernel
capture drops, -27 dBm station frames, 14,287 measured completions, and no
identity/order faults. Exact results:

| completion | sequence / PN | start -> phase-3 | phase-3 -> status | air |
|---:|---|---:|---:|---|
| 6144 | 2046 / `0x1806` | 246 | 87 | **absent** |
| 8192 | 4088 / `0x2000` | 219 | 79 | **absent** |
| 10240 | 2014 / `0x27e6` | 320 | 7 | present |
| 12288 | 0 / `0x3008` | 254 | 8 | present |

The capture contains strong neighboring PNs around both holes (for example
`0x1800`, `0x1809`, `0x180d` and `0x1ffd`, `0x1ffe`, `0x2002`–`0x2004`,
`0x2008`). Combined with the earlier high-coverage three-of-four exact-PN
result, these are genuine phantom successes rather than sequence wrap or kernel
capture loss.

Most importantly, the two absent MPDUs spend 219/246 timer ticks between MAC
start and phase-3 success, squarely inside the 218–320 range of observed
full-size on-air frames. They are not completed at GO and do not take a short
software/hardware bypass. The command runs for a full frame-sized transaction,
then the MAC produces the same ACK-class `0x11` completion accepted by vendor
logic. The remaining defect is below descriptor scheduling: either the PHY does
not radiate a valid MPDU after consuming full airtime, or the response/status
producer accepts completion without valid ACK evidence. Descriptor construction
and response timing match the vendor `txp_build_pipe_descriptor` /
`tx_build_duration_desc` paths, so the next code audit belongs to MAC/PHY TX
state and ACK-response qualification registers rather than host completion.

A temporary `TXRG` image read those registers live after another healthy
false-success run. Own MAC (`12:42:2a:37:70:07`), BSSID
(`98:5f:41:18:76:17`), address-match modes, RX/event filters, event mask, and
pipe IRQ state all matched the translated constants. One concrete vendor drift
did appear: live mode was `0x07ebbadd`, missing bit `0x4000` from the expected
`0x07ebfadd`. `publish_join_pas_with_io` wrote PAS `mode_byte = 1`, while vendor
`mac_apply_channel_and_vif_config` treats `mode_byte == 2` as STA and sets that
bit before `mac_program_mode_regs`.

Changing the PAS byte to 2 produced that inferred mode register and kept
association/recovery healthy. It did **not** solve false success: on a 4.2 ms
baseline the changed image published about 12.5k frames while AP RX advanced
only about 6.8k. A later halted live-vendor MMIO dump disproved the inference:
joined vendor firmware itself has `0x09c00200 = 0x07ebbadd`, exactly the value
produced by PAS `mode_byte = 1`. The temporary mode-byte change is therefore
reverted; static branch naming was not sufficient evidence for the live JOIN
state.

A temporary whole-run phase-3-to-status census then separated that internal
path without relying on four sampled frames. Over 12,542 accepted ordinary
statuses, the delay distribution was sharply bimodal:

- 5,068 (40.41%) arrived within 8 timer ticks;
- none arrived in 9--32 ticks;
- 6 (0.05%) arrived in 33--64 ticks;
- 7,168 arrived in 65--128 ticks and 300 after 128 ticks (59.54% combined).

The same qualified arm's application loss was 59.40%. The near identity of the
late-status fraction and packet-loss fraction, together with the empty
9--32-tick gap and the earlier exact-PN holes at 68--79 ticks, identifies two
hardware outcomes that both produce accepted status `0x11`: prompt ACK and a
late timeout-like result. The translated vendor completion path currently maps
both to `complete_tx_pipe_slot(..., 0)` and therefore reports both as success.
The next discriminator is to classify the late mode as failed completion and
verify that host TX status and retry behavior track the on-air result.

Parallel open-source and vendor-decompilation audits found no omitted software
ACK-validity gate. Vendor `mac_irq_handler` also reduces the raw completion to
its six-bit status, and `txp_pipe_tx_status` passes matching `0x11` to
`txp_fn_2441(..., 0)` without reading elapsed time, a response-result register,
or the descriptor. The unexplained event bit 22 is ignored by vendor code.

A temporary prompt/late provenance image then compared every accepted event in
the two timing populations. The complete raw word was invariant within both
classes and identical across them: `0x0140b711`. The pre-service scheduler word
was invariant zero, and MAC `0x0a28` was invariant `0x19190000`. TX-ring
completion `+0x1c` was invariant zero. MAC `0x0e90`, `0x0ea0`, and TX-ring
cursor/pending `+0x20` varied within both classes rather than separating them.
Thus the visible event and status-time registers provide no ACK/timeout bit;
the late population is already misclassified when hardware emits the same
`0x11` with no retry-pending indication. A proper fix now requires finding the
response-timeout/descriptor or MAC/PHY initialization difference that should
produce the retry event. Timing-based failure classification remains a bounded
fallback, not the preferred root fix.

The remaining vendor-vs-Rust candidates were tested individually with the same
whole-run timing census and healthy baselines. None moved the late population:

- vendor JOIN response slots 8/16/19: 59.30% late;
- STA PAS/BSSID publication before PHY channel transition: 58.69% late;
- descriptor response-duration field +16: 58.38% late, with the same bucket
  boundary, so that field is not the active late-status timer;
- vendor-direction live-register context save: 61.09% late;
- remove the extra pre-vendor-order MAC/RX initialization pass: 58.97% late;
- vendor `mac_set_txop_limit(0)` JOIN tail: 59.03% late.

Ghidra resolved the TXOP block literal at `0x00007b58` to `0x09c00e00`, so the
last test exactly wrote `0x09c00e24 = 0` and reloaded `0x09c00e1c` from
`0x09c00e38`; its negative result is not based on a guessed address. The tested
fractions remain within the run-to-run RF spread around the 59.54% baseline.
Per-frame descriptor construction was also instruction-audited equivalent to
vendor, including duration sources, expected status `0x11`, command words, and
trigger/duration/arm/GO ordering.

At this point no exposed hardware result or known vendor initialization delta
separates the false-success mode. The measured 9--32-tick empty interval is the
only stable outcome discriminator, but two timing-based disposition tests show
that it is not itself a fix. Feeding late protected-data `0x11` into the normal
scheduler-pending retry handler raised idle ping to about 856 ms because that
handler expects real pending-bit ownership. Retiring the same late population
immediately with terminal failure `0x0b` kept a normal 4.7 ms baseline but
increased application loss to 69.50% and produced 87 lifecycle identity
mismatches. Both experiments were removed. Timing remains strong localization
evidence, not a safe replacement for the missing hardware retry indication.

A subsequent retry-engine audit found the first durable improvement. The
synthetic scan transition programs slot timing from scan PAS `+0x4f8`, which is
zero in the open runtime, and JOIN previously never restored the active
station's 2.4 GHz slot-time base. Restoring base 9 immediately after STA PAS
activation, before active rate tables and response descriptors, reproduced
across three qualified arms:

- offered traffic rose from roughly 18--24k to 39.8--46.6k packets per 30 s;
- received traffic rose to 21.8--26.2k;
- loss improved from roughly 59--69% to 41.99--49.02%;
- late statuses fell from 59.54% to 49.88--51.60%.

Repeating the test without the extra IFS write produced the same result (50,277
sender packets, 46,640 offered, 23,779 received), isolating the improvement to
slot timing. The restoration is retained as a real JOIN fix. It is not the full
false-success fix: approximately half of statuses remain in the late population,
and immediate MIB reads show 5.7--9.7k completed frames still awaiting host
confirmation under the doubled load.

The historical vendor control (`/tmp/xr819-vendorbarcap.log`) remains the
functional target: 31,783 of 31,919 offered packets received (0.43% loss), with
34,672 AP RX packets for 34,761 sender calls. Its MIB counters do not count
physical attempts directly: `tx_packets` counts host admissions and
`tx_frames_multi_retried` counts completed frames whose retry byte exceeds one.
Nevertheless, 31,895 such frames demonstrate that vendor completion carries
substantial retry accounting where the open late-`0x11` path carries none.

A fresh halted-vendor comparison made the timing and response-path differences
directly observable. Two vendor arms delivered 28,454/28,796 and
29,478/29,804 packets (1.19% and 1.09% loss), with AP RX close to sender count.
The live timing registers were `0x09c00e30 = 0x47`, `0x09c00e58 = 0x92`, and
`0x09c00e5c = 0xda`; all independently decode to slot-time base 9 and validate
the retained JOIN restoration exactly.

The full vendor/open MMIO diff then exposed a response-slot translation error.
Vendor `txp_program_pipe_slot_ex(slot, 1, 0, 0)` sets the slot bit in the
`+0x08` response bitmap, but the open `clear_pipe_slot_ex` translation cleared
it. Consequently vendor ended with `0x09c00a08 = 0x00180783`, while open had
`0x00180183`: missing bits `0x600` are exactly response slots 11 and 12 holding
the two BlockAck descriptors. The implementation now matches the vendor
instruction sequence by retaining those slots in `+0x08`, clearing them from
`+0x0c` and `+0x10`, and zeroing the associated field selector.

The first qualified corrected arm had a 5.8 ms baseline, offered 47,333 packets,
received 27,110, and lost 42.72%. A final arm with the live-vendor PAS mode
restored offered 47,703 packets, received 28,003, and lost 41.30%. These results
materially improve the slot-only 49.02% arm, but do not close the vendor gap:
periodic lifecycle samples remain in the late population and 12,265 completions
were still awaiting host confirmation in the final arm. The response-slot
correction is retained as a second independently verified vendor-fidelity fix,
while the remaining ordinary-ACK false-success path still requires another MAC
configuration difference.

A subsequent probe of another apparent dump delta showed why halted-state values
must not be copied blindly. Both vendor and open had `0x09c00228 = 0x20000002`;
a first experiment that zeroed it failed association twice and was discarded.
Writing only vendor's halted `0x09c00224 = 0x01000002` kept a clean 4.1 ms
baseline and delivered 27,650 of 46,445 offered packets (40.47% loss), which is
within the spread of the 41.30--42.72% corrected arms and does not solve false
success. Static analysis found no vendor firmware writer for `+0x24`; it is a
hardware-derived status/default rather than a JOIN configuration word. The
probe was removed.

A fresh halted dump of the current image then removed most of the old apparent
MMIO differences. Joined mode `0x0200`, hardware-derived `0x0208`, corrected BA
bitmap `0x0a08`, and timing register `0x0e48` all matched vendor. The arm itself
received 29,556 of 50,836 offered packets (41.86% loss). The only stable
response-routing delta left was `0x0a0c` bits 6 and 17, vendor slots 8 and 19.
Repeating those JOIN-tail publications with the exact vendor bitmap semantics
(clear `+0x08`, set `+0x0c`, clear `+0x10`) produced 26,970 of 48,693 packets
(44.61% loss) on a 4.2 ms baseline. Thus the residual TBTT slots are not the
ordinary-ACK false-success cause and the temporary code was removed.

Feature-gated retry-event provenance then separated genuine physical retries
from unrelated completion-class FIFO traffic. Across qualified runs, almost
every status `0x19` carried the required `0x09c00e84` pending ownership bit;
in the final arm 7,500 of 7,505 did, matching the 7,503 excess MAC starts above
publications. This is the working physical retry path. Status `0x04` behaved
differently: 595 were observed, only 5 had pending ownership, and 265 fell
through ordinary status dispatch. Crucially, **zero** retry-class fallthroughs
hit an active state-3 ordinary slot expecting `0x11`; the latest `0x04` saw no
active slot (`expected = 0`, state 0, kind 0). Thus queued `0x04` events are
background/unowned traffic, not suppressed retry requests for the phantom
MPDUs. Ordinary false-success frames receive only bare `0x11` with no pending
ownership. The remaining defect is definitively in hardware ACK/response
qualification before retry-event generation, not firmware retry routing.

A broad halted PHY/RF comparison next exposed 161 differing words across
`0x0ab8`, `0x0abb`, and `0x0abc`. Most are live calibration/sample windows, but
one static translation error was concrete: vendor `dbg_expand_byte_table` reads
32 bytes from DTCM `0x0400202c`, while the open hardcoded table corresponds to
`0x0400201c`, sixteen bytes early. Applying the exact vendor source in isolation
was not a fix. Two arms had unusable 0.8--1.2 second ping baselines; the qualified
10.1 ms arm offered 39,913 packets, received 20,166, and lost 49.48%. It also
produced 981 lifecycle identity mismatches and 13,290 more MAC starts than
publications, showing substantially changed retry/slot behavior without vendor
correctness. Vendor's halted `0x0abb8300` table no longer equals its initial
source bytes, so later hardware/calibration evolution is part of the contract.
The isolated source correction was removed; the next PHY work must reconstruct
the complete detector-table calibration path rather than replacing only the
initial table. A separate override with the complete stable live-vendor table
also had no effect: its qualified 3.72 ms arm offered 49,302 packets, received
28,205, and lost 42.79%, with 52,878 publications/completions, 56,049 physical
starts, and zero lifecycle identity faults. The temporary table was removed.
Thus neither the initial detector source nor its final halted values alone
explain false success.

A second vendor PHY dump classified 337 of 448 captured words as stable and
reduced the vendor/open comparison to 54 stable differences. The dominant
stable group is the sixteen-word dynamic-IQ correction banks: vendor repeatedly
held packed pairs `(-38, -310)` and `(2, -26)`, while open held `(-35, -262)`
and `(3, -27)`. An exact vendor-pair override initially yielded no qualified arm: one attempt
had a 2.0 second ping average, one narrowly missed the gate at 36/152 ms
average/maximum, and the third found the AP unavailable after `wpa_supplicant`
restarted without NetworkManager reacquiring the interface. Reactivating the AP
profile allowed a qualified repeat at 3.78 ms average ping. It offered 50,623
packets, received 29,038, and lost 42.64%; firmware published 54,338 frames,
started 57,147 physical attempts, and accepted 54,337 statuses, with one
identity mismatch. This is no improvement over the current 41--42% loss range,
so the temporary exact-pair override was removed. The stable dynamic-IQ delta
is not the remaining false-success cause. The duplicated stable RF-result pair
at `0x0abc00e8/00ec` and `0x0abc01e8/01ec` was likewise neutral when forced to
the exact vendor values after calibration: two qualified arms lost 39.85% and
45.48% (42.67% combined), spanning the normal open-firmware variation. The
first arm's apparent improvement did not repeat, so this override was removed.

An expanded 2,120-word AHB comparison added the digital PHY, detector, and
auxiliary RF regions omitted by the first snapshot. Two vendor runs agreed on
1,930 words; intersecting those with open firmware left 45 stable differences.
Forcing the clustered digital-control values at `0x0ab8010c`, `0x0ab80110`,
`0x0ab80128`, and `0x0ab80134` to the stable vendor state was neutral: the
qualified 4.11 ms arm offered 48,749 packets, received 28,666, and lost 41.20%,
with two lifecycle identity mismatches. The temporary override was removed.
The three stable repeated pattern entries at `0x0ab80a1c`, `0x0ab80b00`, and
`0x0ab80b38` were also not missing configuration: replacing open
`0x09110911` with vendor `0x26202620` worsened qualified loss to 46.77%
(47,243 offered, 25,149 received) and was removed. The stable detector/control
pair at `0x0aba8060/8064` was neutral at 41.28% loss (50,445 offered, 29,623
received) and was removed as well. Forcing the stable auxiliary result at
`0x0abc80a0` from open `0x2e` to vendor `0x9d` worsened qualified loss to
46.64% (49,294 offered, 26,305 received) and was removed. The remaining stable
calibration word at `0x0abb8018` was also negative: vendor `0x75a` produced
45.51% loss (49,528 offered, 26,989 received), so it was removed. All isolated
stable differences from the expanded PHY/RF capture are therefore neutral or
negative; the remaining divergence is an initialization/calibration sequence
or an uncaptured MAC/PHY interface state, not one final halted register value.

The conditional dispatcher-mode-3 branch inside vendor
`phy_cal_step_measure()` was also tested as a possible missing transient. A
control/phase-only translation lost 45.41%. Repeating with the complete
`phy_set_bandwidth_mode(0)` side effects and vendor ordering lost 45.38% on a
borderline 20.095 ms baseline. Since the vendor branch is guarded by profile
state and forcing it is consistently harmful, it is not active for this joined
mode-zero path; both temporary translations were removed.
