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

HIF RX initialization and TX staging publish masked bus addresses to hardware,
but retain CPU-form addresses in `HifQueues`. Completion reclamation uses the
software queue entry; it never reconstructs a CPU pointer from a descriptor.
AES/MIC command setup similarly masks a pointer derived from the live Rust slice
and continues to use the slice pointer for CPU access.

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

Open-coded `0x007f_ffff` and `0xf6ff_ffff` masks remain hardware encoders, not
decoders. They should be moved behind named packet-RAM encoding helpers as each
MAC/HIF/crypto family is independently reviewed, without changing command word
layout or volatile publication order. No remaining translated path was found
that converts either masked form back into a CPU pointer.
