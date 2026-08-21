#!/usr/bin/env python3
"""Source/linked drift evidence for the fixed TX aggregate expiration delta.

This checker covers [0x04001160, 0x04001164). It is evidence against source,
layout-inventory, aligned linked-literal, and decoded PC-relative-xref drift;
it is not proof of ownership or consumer closure. Register-computed, indirect,
generic HIF/debug, vendor, IRQ, and FIQ accesses remain outside its proof.
"""

from __future__ import annotations

import argparse
import collections
import re
import struct
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RANGE = (0x04001160, 0x04001160 + 0x04)
LITERAL = re.compile(r"0x[0-9a-fA-F_]+")
SOURCE_EXTENSIONS = {
    ".rs", ".py", ".sh", ".c", ".h", ".hh", ".hpp", ".hxx", ".cc",
    ".cpp", ".cxx", ".s", ".S", ".asm", ".inc", ".ld", ".lds",
    ".ldh", ".x", ".toml", ".mk",
}
SOURCE_FILENAMES = {"Makefile", "Kconfig"}
OWNER_FILES = {"src/dtcm.rs", "tools/check-tx-aggregate-expiration-delta-layout.py"}
ALLOWED_LINKED_LITERALS: collections.Counter[int] = collections.Counter()
ALLOWED_DECODED_XREFS: collections.Counter[tuple[str, int]] = collections.Counter()

REQUIRED_INVENTORY = (
    "#[repr(C, align(4))] struct TxAggregateExpirationDelta { value: SharedU32 }",
    "measurement_workspace: MeasurementWorkspace",
    "tx_aggregate_expiration_delta: TxAggregateExpirationDelta",
    "debug_command_descriptors: InitializedDebugCommandDescriptors",
    "pub(crate) const TX_AGGREGATE_EXPIRATION_DELTA: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, tx_aggregate_expiration_delta));",
    "assert_type_layout!(TxAggregateExpirationDelta, 0x04, 4)",
    "offset_of!(TxAggregateExpirationDelta, value) == 0",
    "offset_of!(InitializedVendorImage, measurement_workspace) == 0x10f8",
    "offset_of!(InitializedVendorImage, tx_aggregate_expiration_delta) == 0x1160",
    "offset_of!(InitializedVendorImage, debug_command_descriptors) == 0x1164",
    "offset_of!(InitializedVendorImage, hif_control) == 0x11ac",
    "assert_type_layout!(InitializedVendorImage, 0x2078, 4)",
    "assert_type_layout!(DtcmLayout, DTCM_STATE_SIZE, 4)",
    "assert_type_layout!(SharedDtcmState, DTCM_STATE_SIZE, 4)",
)
FORBIDDEN_API = (
    "tx_aggregate_expiration_delta_ptr", "tx_aggregate_expiration_delta_ref",
    "tx_aggregate_expiration_delta_mut", "tx_aggregate_expiration_delta_offset",
    "tx_aggregate_expiration_delta_value", "tx_aggregate_expiration_delta_read",
    "tx_aggregate_expiration_delta_write", "tx_aggregate_expiration_delta_bytes",
    "tx_aggregate_expiration_delta_slice", "tx_aggregate_expiration_delta_iter",
    "tx_aggregate_expiration_delta_field", "tx_aggregate_expiration_delta_writer",
    "tx_aggregate_expiration_delta_generic_offset",
)


def code_only(source: str, hash_comments: bool, single_quote_strings: bool) -> str:
    output: list[str] = []
    index = 0
    state = "code"
    depth = 0
    quote = ""
    while index < len(source):
        if state == "code":
            if source.startswith("//", index):
                output.extend("  "); index += 2; state = "line"
            elif source.startswith("/*", index):
                output.extend("  "); index += 2; depth = 1; state = "block"
            elif hash_comments and source[index] == "#":
                output.append(" "); index += 1; state = "line"
            elif source[index] == '"' or (
                source[index] == "'" and (
                    single_quote_strings
                    or re.match(r"'(?:\\\\.|[^\\\\'\n])'", source[index:]) is not None
                )
            ):
                quote = source[index]; output.append(" "); index += 1; state = "string"
            else:
                output.append(source[index]); index += 1
        elif state == "line":
            if source[index] == "\n": output.append("\n"); state = "code"
            else: output.append(" ")
            index += 1
        elif state == "block":
            if source.startswith("/*", index):
                output.extend("  "); index += 2; depth += 1
            elif source.startswith("*/", index):
                output.extend("  "); index += 2; depth -= 1
                if depth == 0: state = "code"
            else:
                output.append("\n" if source[index] == "\n" else " "); index += 1
        elif source[index] == "\\" and index + 1 < len(source):
            output.extend("  "); index += 2
        elif source[index] == quote:
            output.append(" "); index += 1; state = "code"
        else:
            output.append("\n" if source[index] == "\n" else " "); index += 1
    return "".join(output)


def source_paths() -> list[Path]:
    return sorted(
        path for path in ROOT.rglob("*")
        if path.is_file()
        and (path.suffix in SOURCE_EXTENSIONS or path.name in SOURCE_FILENAMES)
        and "target" not in path.parts and ".git" not in path.parts
    )


def in_range(value: int) -> bool:
    return RANGE[0] <= value < RANGE[1]


def check_source() -> None:
    dtcm = (ROOT / "src/dtcm.rs").read_text()
    missing = [item for item in REQUIRED_INVENTORY if item not in dtcm]
    forbidden = [item for item in FORBIDDEN_API if item in dtcm]
    failures = [f"src/dtcm.rs: missing reviewed inventory: {item}" for item in missing]
    failures += [f"src/dtcm.rs: forbidden broad/reference API: {item}" for item in forbidden]
    address_declaration = REQUIRED_INVENTORY[4]
    if dtcm.count(address_declaration) != 1:
        failures.append("src/dtcm.rs: sole address declaration is not unique")
    public_names = re.findall(
        r"pub(?:\(crate\))?\s+(?:const\s+fn|const|fn|static|type|struct)\s+([A-Za-z0-9_]+)",
        dtcm,
    )
    related_public_names = [
        name for name in public_names if "tx_aggregate_expiration_delta" in name.lower()
    ]
    if related_public_names != ["TX_AGGREGATE_EXPIRATION_DELTA"]:
        failures.append(
            "src/dtcm.rs: unexpected TX aggregate expiration delta public API: "
            f"{related_public_names!r}"
        )
    for path in source_paths():
        relative = path.relative_to(ROOT).as_posix()
        if relative in OWNER_FILES:
            continue
        code = code_only(
            path.read_text(errors="replace"),
            path.suffix in {".py", ".sh", ".toml"},
            path.suffix in {".py", ".sh", ".toml"},
        )
        for match in LITERAL.finditer(code):
            token = match.group().replace("_", "")
            value = int(token, 16)
            # Only full physical-address literals are gated. Short values such
            # as an MMIO payload 0x1100 are intentionally not synthesized.
            if len(token) >= 10 and in_range(value):
                line = code.count("\n", 0, match.start()) + 1
                failures.append(
                    f"{relative}:{line}: TX-aggregate-expiration-delta literal {match.group()} "
                    "is outside dtcm.rs"
                )
    if failures:
        raise SystemExit("\n".join(failures))
    print(
        "TX AGGREGATE EXPIRATION DELTA SOURCE/LAYOUT DRIFT-EVIDENCE GATE PASSED "
        f"files={len(source_paths())} inventory={len(REQUIRED_INVENTORY)}"
    )


def linked_literals(path: Path) -> collections.Counter[int]:
    with tempfile.NamedTemporaryFile() as output:
        subprocess.run(
            ["llvm-objcopy", "--dump-section", f".text={output.name}", str(path), "/dev/null"],
            check=True,
        )
        data = Path(output.name).read_bytes()
    result: collections.Counter[int] = collections.Counter()
    for offset in range(0, len(data) - 3, 4):
        value = struct.unpack_from("<I", data, offset)[0]
        if in_range(value): result[value] += 1
    return result


def decoded_literal_xrefs(path: Path) -> collections.Counter[tuple[str, int]]:
    disassembly = subprocess.run(
        ["llvm-objdump", "-d", "--print-imm-hex", str(path)],
        check=True, text=True, stdout=subprocess.PIPE,
    ).stdout
    words = {
        int(address, 16): int(value, 16)
        for address, value in re.findall(
            r"^\s*([0-9a-fA-F]+):.*?\.word\s+0x([0-9a-fA-F]+)\s*$",
            disassembly, flags=re.MULTILINE,
        )
    }
    current_symbol = "<outside-symbol>"
    result: collections.Counter[tuple[str, int]] = collections.Counter()
    for line in disassembly.splitlines():
        if header := re.match(r"^[0-9a-fA-F]+ <(.+)>:$", line):
            current_symbol = header.group(1); continue
        if ".word" in line: continue
        reference = re.search(r"@\s+0x([0-9a-fA-F]+)(?:\s|<|$)", line)
        if reference is None: continue
        value = words.get(int(reference.group(1), 16))
        if value is not None and in_range(value): result[(current_symbol, value)] += 1
    return result


def check_elf(path: Path, dump: bool) -> None:
    literals = linked_literals(path)
    xrefs = decoded_literal_xrefs(path)
    if dump:
        print(f"ALLOWED_LINKED_LITERALS={dict(sorted(literals.items()))!r}")
        print(f"ALLOWED_DECODED_XREFS={dict(sorted(xrefs.items()))!r}")
        return
    if literals != ALLOWED_LINKED_LITERALS or xrefs != ALLOWED_DECODED_XREFS:
        raise SystemExit(
            "TX AGGREGATE EXPIRATION DELTA LINKED DRIFT GATE FAILED\n"
            f"literals expected={dict(ALLOWED_LINKED_LITERALS)!r} actual={dict(literals)!r}\n"
            f"xrefs expected={dict(ALLOWED_DECODED_XREFS)!r} actual={dict(xrefs)!r}"
        )
    print(
        "TX AGGREGATE EXPIRATION DELTA LINKED-XREF DRIFT-EVIDENCE GATE PASSED "
        f"literals={sum(literals.values())} decoded_xrefs={sum(xrefs.values())}; "
        "this is not ownership proof"
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("elf", nargs="?", type=Path)
    parser.add_argument("--dump", action="store_true")
    args = parser.parse_args()
    check_source()
    if args.elf is not None: check_elf(args.elf, args.dump)


if __name__ == "__main__":
    try:
        main()
    except (OSError, subprocess.CalledProcessError) as error:
        raise SystemExit(f"TX-aggregate-expiration-delta drift gate failed: {error}")
