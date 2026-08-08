# Annotated XR819 Ghidra decompiler exports

These files are generated reference pseudocode from the annotated Ghidra
programs supplied in `xr819.tar.gz`:

- `annotated-main.c`: `xr819-annotated-main / ghidra-fw-main.bin`
- `annotated-tcm.c`: `xr819-annotated / xr819-tcm.bin`

They are not buildable source code. Ghidra-inferred types, control flow, and
function boundaries must still be checked against disassembly. The matching
main firmware payload begins at offset `0x1ab8` in the `fw_xr819.bin` supplied
in `xr819-fw.tar.gz`.

The exports contain 757 main-image functions and 25 TCM/bootstrap functions.
Each function has an address/name separator, so normal tools such as `rg` work
well on the combined files.

Regenerate them through the already-running `ghidra-cli` bridge with:

```sh
python3 xr819-decompilation/export_all.py \
  xr819-annotated-main ghidra-fw-main.bin \
  xr819-decompilation/annotated-main.c

python3 xr819-decompilation/export_all.py \
  xr819-annotated xr819-tcm.bin \
  xr819-decompilation/annotated-tcm.c
```
