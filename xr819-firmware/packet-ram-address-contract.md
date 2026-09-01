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

The `0x007f_ffff` forms in `mac.rs`, `platform.rs`, and ordinary descriptor
construction are encode-only values written to MAC registers or command words.
Software retains CPU-form command and buffer addresses separately in typed
contexts, pipe slots, and linker-derived roots. Publication now checks exact
command identity rather than mere membership in the command array.

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

Open-coded `0x007f_ffff` MAC masks and the remaining `0xf6ff_ffff` MAC/TX masks
are hardware encoders, not decoders. They should be moved behind named
packet-RAM encoding helpers as each family is independently reviewed, without
changing command word layout or volatile publication order. No remaining
translated path was found that converts either masked form back into a CPU
pointer.
