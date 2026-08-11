# XR819 vendor container and Rust sectioned-image layout

## Matching container

The matching vendor container used for this analysis is:

```text
/tmp/fw_xr819.bin
size   0x1fe44 (130628)
SHA256 3e2462d476c9dfcb907cda1ba81d0a6d1bbee5e3207bdc1911ec5042d96fdfca
```

Parse it with:

```sh
python3 tools/inspect-vendor-container.py /tmp/fw_xr819.bin
```

The exact section stream is:

| # | Kind | File header | File data | Destination | Length | Other |
|---:|---|---:|---:|---:|---:|---|
| 0 | copy | `0x000004` | `0x000010` | `0xfff00000` | `0x001a7c` | high ARM bootstrap/support |
| 1 | fill | `0x001a8c` | - | `0xfff01a7c` | `0x001d24` | zero |
| 2 | fill | `0x001a9c` | - | `0xfff037a0` | `0x010860` | zero, ending at `0xfff14000` |
| 3 | copy | `0x001aac` | `0x001ab8` | `0x00000000` | `0x01b3d0` | main Thumb/ITCM image |
| 4 | copy | `0x01ce88` | `0x01ce94` | `0x04000000` | `0x0010f8` | initialized DTCM |
| 5 | copy | `0x01df8c` | `0x01df98` | `0x040010f8` | `0x000f80` | initialized DTCM |
| 6 | fill | `0x01ef18` | - | `0x04002078` | `0x007bcc` | zero, ending at `0x04009c44` |
| 7 | MMIO | `0x01ef28` | `0x01ef30` | address/value pairs | `0x000148` | 41 pairs |
| 8 | copy | `0x01f078` | `0x01f084` | `0x0ab81000` | `0x000400` | MAC/PHY table |
| 9 | copy | `0x01f484` | `0x01f490` | `0x0ab88400` | `0x000330` | MAC/PHY table |
| 10 | copy | `0x01f7c0` | `0x01f7cc` | `0x0ab88800` | `0x000330` | MAC/PHY table |
| 11 | copy | `0x01fafc` | `0x01fb08` | `0x0ab88c00` | `0x000330` | MAC/PHY table |
| 12 | entry | `0x01fe38` | - | `0xfff00000` | - | trailer `0x3994defa` |

This resolves earlier approximate `0x1b37c`, `0x1b3dc`, and `0x2fb0`
boundaries. Those values came from treating the tail beginning at file offset
`0x1ab8` as a flat main payload. That tail still contains later section headers;
the matching container's actual ITCM copy is exactly `0x1b3d0`, and its two
initialized-DTCM copies total `0x2078`.

## Container grammar

All values are little-endian `u32`.

```text
magic:  "XR01"
copy:   type=0, destination, length, data[length]
fill:   type=1, destination, value, length
MMIO:   type=2, byte_length, (address, value)[byte_length / 8]
entry:  type=4, entry_address, trailer
```

The final trailer is not a normal CRC32 of either the whole preceding stream or
the section payloads. It remains opaque; the open sectioned downloader validates
stream structure and destinations instead.

## Rust build direction

A coherent Rust ELF may contain several `PT_LOAD` segments selected by its
linker script:

```text
ITCM code/rodata       0x00000000...
initialized DTCM       0x04000000...
zero-filled DTCM BSS   selected non-overlapping DTCM regions
optional high support  0xfff00000...
```

Do not use `objcopy -O binary` for this address space: the holes between these
regions are enormous. `tools/pack-sectioned-elf.py` converts the ELF program
headers into compact copy/fill records while retaining one build's linked
addresses and entry point.

`download-boot-sectioned` accepts writes only inside the observed ITCM, DTCM,
and vendor high-support windows. The existing stable and split-extension
download paths remain unchanged.

## Hardware validation

The sectioned downloader was cold-booted with the exact stable ELF repackaged
as three records:

```text
copy 0x00000000 length 0x7500
fill 0x00007500 length 0x2218
entry 0x00000001
```

It reached healthy WSM idle state, leaked no buffers, received normal scan
traffic, and retained seven APs. This proves that the host-visible firmware file
may have a different size/hash while the loader reconstructs the intended
runtime image.

A single coherent JOIN-enabled Rust ELF was then packaged without binary
splicing:

```text
copy 0x00000000 length 0xacd0
copy 0x0000acd0 length 0x037c
fill 0x0000b04c length 0x1f38
entry 0x00000001
```

Linux received its startup indication and reported `CW1200 WSM init done` with
`XR819 open Rust WSM`. This removes the previous HIF-startup blocker. With
active-probe scanning enabled, its first automatic start-scan request (`0x0008`)
timed out, leaving one host buffer outstanding. A passive sectioned build then
completed normal scanning, received 49 frames, and retained nine APs, localizing
the stall to active-probe preparation rather than common scan/channel handling.

JOIN and host management TX were subsequently built from the same coherent ELF
while active-probe scanning remained disabled. JOIN completed far enough for
Linux to issue repeated authentication frames without `wsm_join failed`, and
all firmware/host buffers returned. An ath9k capture nevertheless found zero
frames from `12:42:2a:37:70:07` among 1826 valid channel-1 frames:

```text
/tmp/xr819-coherent-join.pcap
SHA256 b583b69d4ce6a8e178afda64456a5dfd373669342702a92282594d8c78865a21
```

The old split downloader's unconditional `0x7500` boundary redirected every
later byte of a larger rebuilt image to `0xfff00000`. For the JOIN build that
misplaced code above `0x7500` and all following initialized data. `0x7500` was
therefore a split-loader policy boundary, not an ITCM capacity requirement.

Before moving ordinary Rust statics into DTCM, produce an ownership map for all
fixed `0x0400xxxx` state. General `.data`/`.bss` must not overlap translated HIF,
scheduler, VIF, TX, scan, PHY/gain, callback-table, or stack state.
