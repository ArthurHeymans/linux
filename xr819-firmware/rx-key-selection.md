# RX CCMP key selection

## Source finding

The original `rx_key` mixed pairwise and group matches in one table search.
Key ID zero could match both a GTK and the AP's PTK, so insertion order chose
the cipher key. A unicast frame could also fall back to a GTK, and a group
frame to a PTK. This is a confirmed selection bug, not yet an explanation for
the previously observed roughly 400 authentication failures per traffic run.

RX now selects the class from Address 1's group bit before searching:

- group receiver: AES group key matching interface and CCMP Key ID;
- unicast receiver: AES pairwise key matching interface and transmitter,
  retaining the existing Key ID zero requirement;
- no fallback between those classes.

The local vendor decompilation agrees on the relevant CCMP distinction:
`annotated-main.c`, `key_lookup_for_frame` at `0x000012d8`, distinguishes
pairwise selector `0xf` from group Key ID lookup. The RX data branch at
`LAB_0000cefc` tests Address 1's group bit before choosing the selector. Its
unicast fallback accepts only key type zero (WEP), not AES group keys. This
change does not add WEP support or alter TX lookup.

## Focused tests

A host-only helper seals From-DS CCMP frames with an explicitly supplied key,
without calling the production TX key lookup. Tests cover unicast, broadcast,
and multicast, both PTK/GTK slot orders, and group Key IDs zero through three.
A second test rejects frames sealed with the wrong key class even if their
MIC would authenticate under that key.

Before the fix, the class/order test fails with `Authentication`; the
wrong-class test incorrectly returns success instead of `MissingKey`. After
the fix, 298 default-feature and 299 hardware-diagnostic-feature library tests
pass. The ARM diagnostic image passes stack, packing and DTCM layout checks:

- image SHA-256: `163775e7577b8b51df389e241d43fdc22cb9dd8f87802d057e871ea556c5d897`;
- compiled maximum main call chain: 6408/6912 bytes, unchanged;
- features: `experimental-list-first-depth-four-ampdu`,
  `experimental-fast-loop`, `experimental-aggregate-rate-feedback`,
  `experimental-rx-path-diagnostics`.

## Hardware results

The unchanged auth-class discriminator image
`7ff36b7acaff34ac4859f7a5516ad07f43383648d558ea7592466a36f26272fe` was tried
first on the Intel AP. It saw the correct beacon and repeatedly reached
ASSOCIATED, but never completed the WPA handshake within the harness timeout.
The host bottom half stayed alive, WSM idle, and buffers drained. No CCMP
decryption failures were reported in that failed association attempt.
This is not a throughput result and cannot qualify or disqualify the fix.

The unchanged repeat completed association in two seconds, passed both 50-ping
checks, and finished alive/idle with zero buffers and zero invalid aggregate
reports. It recorded zero authentication failures (both group and unicast),
one missing-key drop, and no retry-marked authentication failure. Thus the
previous session's roughly 400-failure signature was not reproduced.

This repeat delivered only 1.73 Mbit/s default-window board-TX TCP and
4.08 Mbit/s UDP with 73% loss, well below the historical Intel baseline.
The RX pending-byte peak was 12060 and the maximum host-transfer count was 8.
These results must not be compared directly with the old best throughput
numbers or attributed to the key-selection patch, which was not installed.

The first attempt lost its volatile supplicant log on recovery reboot; its
previous-boot kernel journal confirms repeated local deauthentication about
ten seconds after association. The repeat preserved that log and confirms
successful PTK/GTK negotiation.

The first changed-image attempt stopped before target association because the
USB ath9k beacon observer disappeared. The workstation kernel records a USB
disconnect and ath9k teardown; the Intel AP remained active. A further attempt
uses the same traffic test but explicitly records the unavailable independent
observer and requires the target's completed association BSSID to equal the
Intel AP. This is a harness/topology change, not a firmware outcome.

The first attempt with target-side BSSID checking again stalled in ASSOCIATED
with an authentication timeout and zero CCMP drops, matching the initial
unchanged-image failure. The next run enabled supplicant debug logging and
completed in eight seconds. The trace records EAPOL messages 1/4 through 4/4,
PTK installation, and GTK installation with Key ID **1**. This AP run therefore
does not exercise the Key ID zero ambiguity covered by the host tests.

That fixed-image run passed both 50-ping checks, recorded zero decryption drops,
and finished alive/idle with zero used buffers and exact 32398/32398 aggregate
length/ACK accounting (zero invalid reports). Default-window TCP was
2.54 Mbit/s; UDP was 5.51 Mbit/s with 67% loss. Both the current unchanged and
changed images remain far below the historical throughput, so this is evidence
of functional compatibility, not a demonstrated throughput improvement or a
resolution of the broader RX investigation.

A second successful run of the exact fixed image associated in two seconds,
passed both 50-ping checks, and received **99/100 multicast and 100/100 broadcast**
UDP datagrams sent from the Intel AP into the board's Wi-Fi namespace. The
supplicant again installed GTK Key ID 1. After traffic, all decryption-drop
counters were zero, the BH was alive, WSM idle, buffers zero, and aggregate
accounting exact at 33487/33487 with zero invalid reports. TCP was 2.29 Mbit/s;
UDP was 4.91 Mbit/s with 72% loss. Logs are saved locally as
`/tmp/xr819-rx-key-handshake.log` and `/tmp/xr819-rx-key-group.log`; the unchanged
control is `/tmp/xr819-rx-key-baseline-repeat.log`. The harness restored its
recovery image set after each run.

The narrow key-selection fix is hardware-compatible and closes a reproduced
correctness bug. The broader RX/throughput issue remains open: the old
400-authentication-failure signature did not occur in the unchanged control,
and substantial UDP loss persists despite clean firmware aggregate accounting.
Do not attribute the low throughput or the missing historical signature to this
fix. Next, reproduce the historical baseline inputs and reconcile packets
reported acknowledged by the firmware with packets actually received by the AP.

The feature-free ARM build also passes stack, packer and DTCM layout checks;
the AES DMA-address source gate passes. The existing RX diagnostic feature
remains useful while the original failure signature is unresolved; no new
feature was added.
