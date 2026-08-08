# Salvage notes from `xr819.tar.gz` and `xr819-fw.tar.gz`

The first archive contains two Ghidra packed program files:

```text
ghidra-fw-main.bin.gzf
xr819-tcm.bin.gzf
```

They were restored locally as:

```text
project xr819-annotated-main / program ghidra-fw-main.bin
project xr819-annotated      / program xr819-tcm.bin
```

The main program has 757 functions, extensive semantic names, 63 detailed
plate comments, global symbols, and useful WSM/TX/RX structure definitions.
The TCM program has 25 named functions covering MIB dispatch, global-state
initialization, debug UART, and debug-message helpers.

## Compatibility and binary authority

The annotated main image is byte-identical to the main payload beginning at
container offset `0x1ab8` in the supplied `xr819-fw.tar.gz` `fw_xr819.bin`.
That archive is therefore authoritative for interpreting the annotations. It
still differs from the older image used for the currently deployed target
build, and function drift between those builds is not globally constant.
Addresses and hardware literals must consequently be cross-checked before
transferring a reconstructed routine to the deployed image.

Examples of locally verified corresponding functions:

| Annotated image | Target 2018 image | Meaning |
| --- | --- | --- |
| `0x13d98` | `0x13d44` | `syn_scan_begin_request` |
| `0x14000` | `0x13fac` | `syn_scan_finish_and_confirm` |
| `0x14358` | `0x14304` | `syn_scan_program_channel` |
| `0x17c74` | `0x17c20` | `rf_calibrate_iq_dc` |
| `0x1843e` | `0x183ea` | `rf_load_band_regs` |
| `0x184d4` | `0x18480` | `rf_dft_correlate_samples` |
| `0x18610` | `0x185bc` | `rf_capture_adc_samples` |
| `0x18666` | `0x18612` | `phy_program_iq_corr` |
| `0x187f4` | `0x187a0` | `rf_save_band_regs` |
| `0x1899c` | `0x18948` | `rf_apply_channel_settings` |
| `0xeda4` | `0xed4c` | `hif_send_msg_to_host` |
| `0xeebc` | `0xee64` | `mic_finish_entry` |

## RF and calibration findings

The annotations confirm the high-level roles of the current calibration chain:

```text
rf_cal_gate                 target 0x168b8
rf_set_test_tone            target 0x17884
rf_select_band_regs         target 0x178be
rf_cal_path_setup           target 0x178ce
rf_clear_iq_dac             target 0x179c0
rf_compute_iq_gain_corr     target 0x179ea
rf_write_iq_corr_regs       target 0x17ac8
rf_measure_iq               target 0x17b70
rf_set_iq_dac               target 0x17bf2
rf_calibrate_iq_dc          target 0x17c20
```

The decompiled `rf_calibrate_iq_dc` confirms:

- twelve gain iterations;
- baseline DAC `(0x11, 0x11)` and target DAC `(1, 1)`;
- per-axis scale `(target - baseline) * -256`;
- zero scales replaced with one before division;
- coefficient `((-baseline << 14) / scale) + 0x44`;
- final iteration coefficients and scales saved by the four-word reference
  helper;
- optional secondary measurement uses the saved final-gain values;
- primary register publication precedes normalized gain publication;
- cleanup always clears the DAC fields, restores the path and band, disables
  the test tone, and clears the calibration gate.

### Dynamic IQ/DC routine

Target `0x18948` is named `rf_apply_channel_settings` in the annotated image.
The decompiler exposes the whole algorithm rather than only its MMIO envelope:

1. Reject re-entry using a global active/state byte.
2. Allocate and clear the large workspace.
3. Select one of two four-value initial candidates.
4. Save and override RF/MAC registers through `rf_save_band_regs`.
5. Decode two existing packed correction words into four signed 12-bit values.
6. Run averaging passes. Each pass executes thirteen measurement/search stages.
7. Program candidate corrections, trigger capture, collect 64 ADC words, run a
   fixed-point DFT/correlation, and dispatch a stage-specific update.
8. Reject candidate components outside `+-0x200` and `+-0x800` bounds.
9. Average accepted corrections and add the mode-dependent seed.
10. Publish the final signed 12-bit correction pairs.
11. Replicate two selected correction words across sixteen table slots.
12. Run a final verification capture and set per-band validity/failure flags.
13. Restore all saved registers through `rf_load_band_regs`.

Initial candidates:

```text
modes 1/2:  ( 7, -7, -5, 1)
other:      (-4,-11, -4, 0)
```

`phy_program_iq_corr` packs values as:

```text
0x0abb8068 = (first & 0xfff) | ((second & 0xfff) << 16)
0x0abb80a8 = (third & 0xfff) | ((fourth & 0xfff) << 16)
```

`rf_capture_adc_samples` has exact bounded behavior:

```text
poll 0x0abb81ac bit 15
maximum 10000 polls
one vendor delay unit per poll
copy 64 words from 0x0abb81c4 after completion or timeout
```

The Rust firmware now contains a detached translation of this capture behavior.
It returns readiness separately because the vendor still copies the 64 words
after timeout.

`rf_dft_correlate_samples` uses two 64-entry signed sine/cosine tables. The
complete 256-byte table at annotated SRAM `0x04000f88` occurs byte-for-byte in
both local vendor firmware files. Target `0x18480` was cross-checked instruction
by instruction and is now translated as pure Rust, including sample sign
recovery, bit-length scaling, mode-dependent selection of one or three complex
correlations, phase stepping modulo 64, wrapping fixed-point accumulation, and
signed high-16-bit output. Candidate normalization and rejection at target
`0x18c3e..0x18cb8` is translated as well.

Forced-case Ghidra imports and Radare2 `pd:g` output recovered all cases of
annotated `rf_op_dispatch2` at `0x17f34`:

```text
case 0   select third - common_step
case 1   retain all three correlations; select third + common_step
case 2   retain common sample 1; select third/fourth + common_step
case 3   retain common sample 2; select fourth - common_step
case 4   retain common sample 5; select fourth + common_step
case 5   retain common sample 3
case 6   retain common sample 4; polynomially refine third/fourth
case 7   select second - second_step
case 8   refresh three correlations; select second + second_step
case 9   retain axis sample 3; select first - first_step
case 10  retain axis sample 4; select first + first_step
case 11  retain axis sample 1
case 12  retain axis sample 2; bounded parabolic refinement of first/second
case 13  no operation
```

The complex-pair helpers are now identified exactly: `0x19598` arithmetic-shifts
both signed 16-bit components, `0x195aa` finds their maximum absolute component,
ARM helper `0xf07c` returns `real*real + imaginary*imaginary`, and `0x1af88`
performs signed division. The complete switch is represented by the allocation-
free Rust `DynamicIqSearchState` and `apply_dynamic_iq_search_stage`; cases 6
and 12 use separately tested pure refinement functions.

The outer `rf_apply_channel_settings` arithmetic is now translated as well:

- configuration bit 1 doubles the requested low-byte pass count;
- pass zero supplies baseline metrics and is excluded from accumulation;
- passes after pass one begin from `accumulated / (pass_index - 1)`;
- accepted passes use wrapping four-component accumulation;
- final values are `accumulated / accepted_count + profile seed`;
- the primary profile seed is `(-2, 5, -1, -1)`;
- the alternate profile uses the mode-selected initial candidate;
- verification restores either rejected packed pair and sets two `x1024`
  quality flags.

Ghidra, Radare2 `pd:g`, and direct Thumb disassembly all expose an asymmetric
vendor publication test: rejection of the second packed pair suppresses final
publication, while the independently calculated first-pair rejection flag does
not. The Rust `DynamicIqVerification` preserves and documents this behavior
rather than silently correcting it.

The complete `rf_save_band_regs`/`rf_load_band_regs` envelope is now represented
by a typed 20-register snapshot plus detached acquisition/restoration routines.
The implementation preserves:

- the extra one-unit delay in the alternate profile;
- profile-specific MAC/PHY override constants;
- the derived `0x0abb801c` and `0x0abc0030` values;
- the `0x04001ff4` bit-26 contribution to `0x0abc0020`;
- the alternate-profile quirk that restores zero to `0x0abb81a4` instead of
  saving its live value;
- conditional restoration of the two packed IQ-correction registers;
- calibration stop before final control restoration;
- PLL restart and final six-unit delay.

Both save and load paths were independently cross-checked with Radare2 `pd:g`.
Forced cases 0 and 1 of `phy_cal_cmd_dispatch` were then recovered separately:
stop clears `0x0abb805c..0x0abb8064`, while start publishes
`(0x12, 0x88000000 | (command & 0xff), 0)`. These writes are now integrated
into the envelope.

Both branches of `rf_program_synth_freq` are now represented as pure Rust. The
mode-1 branch uses constant `0x4c4b4`, the runtime reference multiplied by
1000, and an exact 21-step restoring division before inserting the saved integer
field at bit 21. The larger branch preserves the original wrapping 64-bit
multiply/divide chain, remainder scaling, 21-bit packing, and zero-denominator
failure boundary. The live wrapper now reads the mode, reference clock and
current PLL word before touching the band registers, prepares the complete synth
word, and aborts cleanly if the fixed-point denominator is invalid. It later
publishes that prepared word at the original call position and performs the
vendor PLL restart. The complete detached hardware wrapper therefore has no
remaining synth callback and guarantees band-register restoration on every
normal post-acquisition return.

The annotated outer profile gate is now preserved too: a nonzero byte at
`0x0400994e` skips acquisition and marks `0x04009961` valid, while profile zero
runs the primary calibration path with fixed seed `(-2, 5, -1, -1)`.

The outer routine's complete 13-stage hardware schedule is now explicit and
allocation-free. Stages 0 and 7 publish without capture or DFT; stages 6 and 12
capture and correlate without publishing a new candidate; all remaining stages
capture, publish, and correlate in that order. First-pass control overrides at
stages 0 and 1 are represented separately. `run_dynamic_iq_search_pass()` now
drives this schedule through a hardware callback and applies all dispatcher
state transitions, stopping immediately on acquisition failure.

Pass-level orchestration is now explicit as well. The live wrapper now decodes
its starting four signed values directly from the two correction words captured
by `rf_save_band_regs`, rather than accepting a guessed initial candidate. Pass
zero records the baseline verification metrics but is excluded from
accumulation; later passes begin from
`accumulated / (pass_index - 1)`, retain their normalized final candidates with
wrapping addition, and finish with `accumulated / accepted_passes + seed`.
Configuration bit 1 doubles the requested pass count. Searches with fewer than
two total passes, missing baseline capture, or any failed stage return no result
rather than dividing by zero or publishing partial calibration.

Final verification is now combined with publication planning. The final packed
candidate is first written to hardware and replicated across both sixteen-word
correction banks. Rejected packed pairs are then restored independently by the
band-register loader, both `×1024` quality flags are retained, and the vendor's
asymmetric rule controls publication into persistent DTCM state: first-pair
rejection still allows the original packed candidate words to be recorded,
while second-pair rejection suppresses that software-state publication entirely.

The primary `rf_calibrate_iq_dc`/target `0x17c20` hardware envelope is now
assembled as well. It gates the calibration engine, preserves the selected band
register set, performs bounded baseline and target measurements at all twelve
vendor gain indices, publishes primary and normalized correction tables, stores
the final-gain coefficient/scale references at `0x0400993c`, and restores the
path, test tone, band selector, and engine gate on success or timeout. A direct
annotated-C review also corrected one subtlety: every gain derives its shift
word from the same profile value at `0x04009978`; shift output from one gain is
not fed into the next gain.

The supplied annotated container was also reparsed independently. Its ordered
41-register type-2 section and all four MAC/PHY type-0 copies are byte-identical
to `xr819-firmware/src/loader.rs` and `xr819-firmware/data/`.

### SDD-derived configuration power ranges

Annotated SDD callback `0x17626` copies eleven little-endian halfwords from each
of TLVs `0xe3` and `0xe4` into runtime profiles at `0x040034b0` and
`0x04003542`. `phy_get_tx_power_range` (`0x16e9c`) reads profile entry ten at
offsets `0x14` and `0xa6`, while supplying a fixed minimum of `-160` and the
configuration response supplies zero stepping. For the supplied 744-byte SDD:

```text
TLV e3 entry 10 = 272  (27.2 dBm)
TLV e4 entry 10 = 212  (21.2 dBm)
minimums        = -160 (-16.0 dBm)
stepping        = 0
```

The Rust configuration parser now derives these maxima from the retained SDD
instead of advertising a hard-coded 20.0 dBm maximum.

## Scan findings

The semantic scan names validate the previously reconstructed state machine:

```text
syn_scan_begin_request
syn_scan_set_state
syn_scan_restore_channel
syn_scan_finish_and_confirm
syn_scan_dwell_next
syn_scan_set_band_and_program
syn_scan_build_probe_req
syn_scan_maybe_send_probe
syn_scan_program_channel
syn_scan_complete_if_scanning
syn_scan_abort
syn_scan_probe_tx_done
syn_scan_filter_rx_frame
```

Important recovered behavior:

- start-scan rejects more than 34 channels;
- active scan returns status 4;
- malformed channel timing returns status 2;
- event bit `0x400` drives the cooperative scan task;
- channel dwell uses the selected channel's maximum dwell time multiplied by
  `0x400`, plus a small PHY-dependent adjustment;
- state 2 and state 3 distinguish two dwell/power-save paths;
- state 6 immediately requeues the scan event;
- completion restores a prior vif channel when one exists, otherwise stops the
  radio, then sends `ind_0806_scan_complete`;
- probe response and beacon frames are specially admitted during scan RX
  filtering.

These findings should guide real dwell and RX integration, but target literals
must still be checked against the 2018 image.

## HIF, IRQ, RX, and TX findings

The annotations provide useful names for nearly the complete data path:

```text
hif_rx_process
hif_send_msg_to_host
hif_tx_confirm_drain
hif_irq_demux
rx_indication_build_and_send
rx_handler_main_loop
wsm_h_04_tx_req
tx_frame_complete
tx_confirm_build_and_send
mac_irq_handler
enc_xfer_complete
mic_finish_entry
```

### Hardware IRQ demultiplexing

`hif_irq_demux`:

1. reads pending bits from interrupt-controller offset `0x20`;
2. acknowledges them by writing the same mask at offset `4`;
3. dispatches each set bit through a reverse-indexed 32-entry callback table;
4. keeps hardware IRQ dispatch separate from cooperative scheduler events.

### IRQ 18/20 encoder completion

Annotated `enc_xfer_complete` corresponds to the target routine beginning near
`0xe9c0`. It:

1. clears the encoder active word;
2. asserts if the current transfer pointer is null;
3. clears transfer byte `+5`;
4. conditionally sets byte `+5` according to transfer class and two hardware
   status bits;
5. queues the transfer for deferred completion;
6. starts the next queued encoder transfer.

This is enough to replace the diagnostic IRQ 18/20 callback only after the
encoder queue initialization and the deferred-transfer helpers are translated.

### IRQ 21 MIC completion

Target `0xee64` is `mic_finish_entry`. It receives the completed object pointer,
appends it to the queue rooted at `0x04009928`, and sets cooperative event bit
29 (`0x20000000`). Activation still requires preserving the IRQ callback's
incoming `r0` argument and translating the MIC consumer task.

### Host message publication

Annotated `hif_send_msg_to_host` corresponds to target `0xed4c`. It:

- validates the message pointer;
- sets transport bit `0x0400` when appropriate;
- increments pending-message accounting;
- sets event bit 1 when the count transitions from zero to one;
- checks the 64-entry software ring for overflow;
- queues transport-owned messages when their header requests it;
- inserts the message into the ring;
- starts publication when fewer than four hardware entries are outstanding.

This provides the missing semantic frame for implementing faithful `0xed4c`
pending/event accounting.

## Recovered data types

Useful annotated structure sizes include:

```text
wsm_tx                 24 bytes
xr_tx_pas             124 bytes
xr_tx_ctx             368 bytes
xr_txpipe_state        56 bytes
xr_vif                944 bytes
xr_bab_session         40 bytes
xr_key_slot           164 bytes
```

The types include partial field names for queue IDs, rate policy, retry state,
completion class, vif/link IDs, cipher state, packet IDs, WSM TX fields, and
TX durations. They should substantially accelerate real TX request and confirm
translation.

Detailed comments also establish:

```text
30 host TX contexts, each 0x170 bytes
3 smaller internal TX contexts
30 x 1632-byte host request buffers
4 x 384-byte firmware-to-host buffers
24 shared key slots
3 vif structures, with vif 2 acting as the P2P-device slot
```

## TCM findings

The TCM archive identifies:

```text
mib_write_dispatch
mib_read_dispatch
fw_global_state_init
hif_send_debug_msg
hif_send_debug_u32
UART/debug printf and hexdump routines
```

This should accelerate replacement of generic `WRITE_MIB` success responses
with real state effects and help recover remaining startup initialization.

## Recommended use

1. Use annotated names to identify a candidate routine.
2. Match it to the target 2018 binary by call shape, literals, and uninterrupted
   disassembly; never apply a global address offset.
3. Use the annotated decompiler output to understand structures and algorithmic
   intent.
4. Translate only behavior confirmed in the target image.
5. Prioritize:
   - dynamic IQ DFT/candidate search;
   - `0xed4c` HIF queue accounting;
   - IRQ 18/20 and IRQ 21 queue consumers;
   - scan dwell and RX filtering;
   - WSM TX request and TX confirmation structures.
