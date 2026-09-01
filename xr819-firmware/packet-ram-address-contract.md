# Packet-RAM address-space contract

XR819 software and hardware use multiple representations for the same packet-RAM
storage. They are not interchangeable.

## Address classes

| Class | Shape | Valid use |
|---|---|---|
| CPU-form runtime address | `0x09007000..0x09016630` | Rust volatile loads/stores and software ownership records |
| CPU-form RX FIFO address | `0x09400000..0x09408000` | Rust RX FIFO loads/stores after bounded offset normalization |
| MAC 23-bit offset | CPU address masked with `0x007f_ffff` | MAC registers and opcode payloads only |
| DMA/HIF bus address | CPU address masked with `0xf6ff_ffff` | HIF, AES, MIC, and packet-DMA descriptors only |
| A-MPDU opcode-0 transfer | `0x65000000 | (cpu & 0x001f_fffc)` | MAC command stream only |
| RX FIFO logical offset | wrapping offset below `0x7000` | RX producer/consumer accounting only |

A bus encoding is never a CPU pointer. Decoding a hardware word by OR-ing a CPU
base or by using the masked value directly is forbidden unless an independently
owned CPU-form identity proves the target object.

## Ownership paths

### A-MPDU command streams

`packet_ram::ampdu_transfer_word` is the owner of opcode `0x65` encoding. It
accepts only aligned addresses in the linker-owned runtime pool. Host tests also
accept process-local linker objects so descriptor construction can be tested
without manufacturing target pointers.

The command word is write-only from the CPU-address perspective. Whole-aggregate
rearm does not recover an address from `command + 0x18`; it validates the exact
DTCM software-record node, reads that node's CPU-form packet-record address, and
requires the DTCM-node and packet-record indices to match.

Initial and retry publication additionally require:

- the command address to be the exact linker-owned `(pipe, slot)` command base;
- both host contexts to retain their exact linker-owned frame-state addresses;
- the DTCM descriptor node to be an exact free-list node base;
- the packet record to be the matching exact linker-owned software-record base.

Interior command addresses, interior DTCM nodes, masked bus aliases, unaligned
packet addresses, and mismatched node/record pairs are rejected before they are
dereferenced or written.

### Ordinary MAC command pointers

`mac.rs` derives its 23-bit offsets through
`packet_ram::mac_packet_offset_unchecked`; every caller supplies a linker-owned
root or object base. Response-pointer publication separately accepts only exact
response-command bases before applying the DMA bus encoding, and rejects invalid
pipe slots before any volatile write. Platform startup uses the same owned
23-bit encoder for TX-ring and automatic-response roots.

Ordinary TX descriptor publication first constructs a
`packet_ram::RuntimePacketAddress`, which proves that the complete frame or
command-list destination fits inside one linker-owned runtime object. Distinct
`MacPacketOffset` and `TxPayloadBusAddress` values can then be derived from that
CPU-form identity; their representations cannot be supplied where a CPU pointer
is required. TX start, timing, completion-message, and completion-dispatch paths
also revalidate each retained context frame address together with its full frame
length before dereferencing header fields. The publication registry retains the
exact DTCM context/frame-node and packet-RAM command base for each `(pipe, slot)`;
depth-two BlockAck handling requires the live slot's frame and command words to
still match that identity before consuming retained state. Ordinary TX start
and successful-completion drains independently construct a `LiveTxSlot`,
requiring bounded pipe/slot cursors, an exact DTCM frame node, and the matching
packet-RAM command base before dereferencing either retained word. Watchdog
retirement preserves the vendor's legitimate empty-frame recovery case, but a
non-empty slot must satisfy the same live identity; retry-owned aggregate slots
must validate that identity, the pipe's exact hardware-ring MMIO root, and a
bounded current cursor before their command-mask bit is released. Ordinary and
depth-two retry/rearm paths use the same mockable identity validator before
reading retained slot metadata, rebuilding descriptors, or touching the ring.
Every remaining production hardware-ring consumer now accepts only the exact
per-pipe MMIO root; advance, resynchronization, diagnostics, retry completion,
BlockAck completion, and single/batched publication reject mismatched retained
roots before MMIO access. The first candidate (`ca809215…`) also repeated
unrelated slot/frame/command validation inside the infallible publication tail;
it was rejected after leaving nine host buffers stuck. Keeping those identities
at their existing ownership boundary while validating only the retained ring
restored cold and warm traffic. Interface metadata and duration-table selectors
are range-checked before their addresses
are formed. Source gates
reject open-coded MAC, platform, and production TX packet-RAM masks and freeze
the checked TX boundary calls.

### HIF and crypto DMA

HIF RX initialization and TX staging pass CPU-form addresses through
`packet_ram::packet_dma_bus_address`, which accepts only linker-owned runtime or
RX FIFO storage. `HifQueues` retains the CPU-form identity, and completion
reclamation uses that software queue entry rather than reconstructing a pointer
from a descriptor. Fixed linker-owned emergency and startup roots use the
explicit unsafe encoder without adding a fallible call to the exception stack.

AES/CCMP validates the live payload slice through the same checked encoder
before publishing either source or destination. It returns
`CcmpError::InvalidDmaAddress` before starting hardware when a slice is not in
owned packet RAM. Source gates reject open-coded HIF and crypto DMA masks.

### RX FIFO

`radio.rs` treats producer and consumer values as logical offsets. Every CPU
access normalizes the offset modulo the qualified `0x7000` logical extent and
adds it to `packet_ram::rx_fifo_base()`. No HIF or MAC descriptor encoding is
used as an RX FIFO CPU address.

### Diagnostics

Diagnostic pointer snapshots are non-owning. They now require the complete
requested word to fit inside one linker-owned object, preventing a valid final
byte from authorizing an out-of-range multi-byte read.

## Remaining audit boundary

Production `0x007f_ffff` and `0xf6ff_ffff` packet-address masks are now confined
to `packet_ram.rs`; numeric copies in tests remain independent encoding oracles.
The TX `0x007f_fffc` value is an alignment/field mask passed to the typed payload
encoder, not a recoverable pointer representation. Remaining raw command words
combine opcodes with already validated typed address values; no translated path
was found that converts a masked hardware form back into a CPU pointer.
