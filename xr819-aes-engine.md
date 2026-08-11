# XR819 AES/CCMP engine

This note records the locally recovered AES accelerator interface and the plan
for validating it against the RustCrypto CCMP backend. No XR819 information was
sourced from the web.

## Current role

The accelerator is not required for initial correctness. The open firmware
currently uses no-std RustCrypto AES-CCM for WSM AES pairwise/group keys. A host
round-trip test and an independent Python `cryptography` AESCCM vector pass, but
live protected payload TX still stalls only after MAC hardware-ring activation.
That makes MAC publication/completion reliability
the immediate blocker; hardware AES is retained as an optimization target and
an independent known-answer oracle.

Do not replace RustCrypto until software and hardware produce identical
ciphertext/MIC results for the same key, nonce, AAD, and payload, and corrupted
ciphertext/MIC tests reliably fail authentication.

## Register map

The engine is rooted at `0x09c5_0000`:

| Offset | Purpose |
|---:|---|
| `0x00` | command write / status read |
| `0x04` | 32-bit input FIFO |
| `0x08` | separate bulk/debug port; unused by ordinary CCMP |
| `0x10` | DMA source address |
| `0x14` | DMA destination address |
| `0x18` | payload length |

Observed status bits:

- bit 13: FIFO ready;
- bit 12: command busy/not accepted;
- bit 0: authenticated operation succeeded, sampled after the completion IRQ.

DMA addresses are programmed as `address & 0xf6ff_ffff`.

## Ordinary CCMP transfers

The vendor transfer-class table at DTCM `0x0400_0804` identifies:

- class 6, flags `0x84`: CCMP TX encryption;
- class 7, flags `0x80`: CCMP RX decryption;
- classes 8/9/10: protected-management/BIP stages, not ordinary data CCMP.

The function previously named `crypto_hw_program_ccmp` at `0xe794` belongs to
the protected-management path. Ordinary data CCMP is programmed by
`hif_hw_program_xfer` at `0xe6fc`, reached through the serialized engine queue.

### Key loading

Wait for command-ready, write four little-endian 32-bit words containing the
16 key bytes to `AES+0x04`, then issue:

```text
0x1100
```

The key bytes are not reversed relative to their WSM representation.

### CCM context

Write one 16-byte context block and issue `0x1240`:

```text
byte 0       0x01
byte 1       QoS TID/priority
bytes 2..7   transmitter address A2
bytes 8..13  PN5 PN4 PN3 PN2 PN1 PN0
bytes 14..15 payload length, big endian
```

### AAD

The staged AAD stream starts with a two-byte big-endian AAD length followed by:

```text
masked frame control
address 1
address 2
address 3
masked sequence control
optional address 4
optional QoS control
```

For normal data, `masked_fc = (fc & 0xc78f) | 0x4000`.

AAD lengths and tail commands are:

| AAD | Shape | TX | RX |
|---:|---|---:|---:|
| 22 | 3-address non-QoS | `0x148b` | `0x148a` |
| 24 | 3-address QoS | `0x14ab` | `0x14aa` |
| 28 | 4-address non-QoS | `0x14eb` | `0x14ea` |
| 30 | 4-address QoS | `0x140b` | `0x140a` |

The first AAD block uses `0x1403` for TX or `0x1402` for RX.

### DMA start

After programming source, destination, and plaintext length:

```text
TX encrypt: 0x3008_1009
RX decrypt: 0x3008_1008
```

TX encrypts in place. RX supports the vendor overlap:

```text
src = frame + header_length + 8
dst = frame + header_length
```

This decrypts while removing the eight-byte CCMP header. The logical DMA length
excludes both the CCMP header and eight-byte MIC; hardware consumes or writes
the tag adjacent to the payload.

## Interrupt and queue behavior

The vendor registers `enc_xfer_complete` (`0xea08`) for IRQ 18 and IRQ 20. The
handler clears the global owner, checks AES status bit 0 for transfer classes
6 through 9, moves the descriptor to a deferred-completion list, starts the
next queued operation, and schedules callbacks outside interrupt context.

The exact direction-to-IRQ mapping is not yet known. The replacement must not
assume status bit 0 itself signals completion.

Required invariants:

1. serialize all TX and RX AES operations globally;
2. mutate the queue under IRQ exclusion;
3. retain key/AAD/source/destination storage until completion;
4. use DMA-visible uncached memory initially;
5. issue a write barrier before the start command;
6. use bounded FIFO/command waits;
7. treat timeout as fatal until an engine reset sequence is known.

## Validation plan

### 1. RustCrypto independent known-answer tests — initial vector passed

The first fixed vector was generated independently with Python
`cryptography.hazmat.primitives.ciphers.aead.AESCCM` and is now hard-coded in
the Rust tests:

```text
key        000102030405060708090a0b0c0d0e0f
nonce      0312422a3770070123456789ab
AAD        88412005b6ff014312422a377007ffffffffffff00000300
plaintext  101112131415161718191a1b1c1d1e1f
ciphertext c740959bf59a9631a8c2c9d9910d114b
MIC        1185855bce30f3e9
```

The test also verifies the CCMP header
`ab 89 00 20 67 45 23 01`, successful decryption, and rejection after corrupting
either ciphertext or MIC. Remaining coverage should add non-QoS and four-address
vectors, but the live failure is no longer plausibly explained by a basic
QoS CCMP nonce/AAD/byte-order error.

### 2. Idle status survey

Read `AES+0x00` without writes. Confirm bit 13 is ready and bit 12 is clear.

### 3. IRQ identity

Install separate bounded counters for IRQ 18 and IRQ 20. Submit one TX and one
RX known-answer operation and record:

- which IRQ fires for each direction;
- whether both ever fire for one operation;
- status bit 0 and interrupt pending/acknowledgement order.

### 4. Hardware TX known-answer

Use uncached scratch memory and compare hardware output byte-for-byte with the
fixed RustCrypto vector. For QoS AAD length 24, issue:

```text
0x1100, 0x1240, 0x1403, 0x14ab, 0x3008_1009
```

### 5. Hardware RX authentication

Decrypt the preceding ciphertext using:

```text
0x1100, 0x1240, 0x1402, 0x14aa, 0x3008_1008
```

Then flip one ciphertext bit and one MIC bit in separate runs; both must clear
authentication success.

### 6. Boundary matrix

After the basic vector passes, test AAD lengths 22/24/28/30, payload lengths
1/15/16/17, overlapping RX destination, and the largest supported Wi-Fi frame.

## Integration boundary

Expose a CCMP-level Rust API rather than arbitrary command words. The public
operation should accept an 802.11 frame and selected key, then construct the
AAD, nonce, PN, context, and DMA ranges internally. Keep RustCrypto as the
fallback/reference backend until repeated live traffic proves hardware AES and
its IRQ completion path stable.
