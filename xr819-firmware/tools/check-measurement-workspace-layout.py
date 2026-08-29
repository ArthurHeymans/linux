#!/usr/bin/env python3
"""Source/linked drift evidence for the fixed measurement workspace.

This checker covers [0x040010f8, 0x04001160). It is evidence against source,
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
RANGE = (0x040010F8, 0x040010F8 + 0x68)
LITERAL = re.compile(r"0x[0-9a-fA-F_]+")
SOURCE_EXTENSIONS = {
    ".rs", ".py", ".sh", ".c", ".h", ".hh", ".hpp", ".hxx", ".cc",
    ".cpp", ".cxx", ".s", ".S", ".asm", ".inc", ".ld", ".lds",
    ".ldh", ".x", ".toml", ".mk",
}
SOURCE_FILENAMES = {"Makefile", "Kconfig"}
OWNER_FILES = {"src/dtcm.rs", "tools/check-measurement-workspace-layout.py", "tools/check-initialized-ps-wake-guard-layout.py"}
ALLOWED_LINKED_LITERALS: collections.Counter[int] = collections.Counter()
ALLOWED_DECODED_XREFS: collections.Counter[tuple[str, int]] = collections.Counter()

REQUIRED_INVENTORY = (
    "#[repr(C, align(4))] struct MeasurementWorkspace",
    "dwell_bound: SharedU16",
    "opaque_02: OpaqueBytes<0x03>",
    "measurement_type: SharedU8",
    "opaque_06: OpaqueBytes<0x02>",
    "completion_status: SharedU32",
    "opaque_0c: OpaqueBytes<0x05>",
    "dispatch_state: SharedU8",
    "dispatch_argument: SharedU16",
    "opaque_14: OpaqueBytes<0x04>",
    "start_timestamp_words: [SharedU32; 2]",
    "elapsed_timestamp_words: [SharedU32; 2]",
    "scan_request_prefix: OpaqueBytes<0x01>",
    "scan_request_mode: SharedU8",
    "opaque_2a: OpaqueBytes<0x3e>",
    "ps_wake_guard_words: PsWakeGuardWords",
    "measurement_workspace: MeasurementWorkspace",
    "tx_aggregate_expiration_delta: TxAggregateExpirationDelta",
    "debug_command_descriptors: InitializedDebugCommandDescriptors",
    "pub(crate) const MEASUREMENT_WORKSPACE: DtcmAddress",
    "pub(crate) const fn measurement_dwell_bound() -> DtcmAddress",
    "pub(crate) const fn measurement_type() -> DtcmAddress",
    "pub(crate) const fn measurement_completion_status() -> DtcmAddress",
    "pub(crate) const fn measurement_dispatch_state() -> DtcmAddress",
    "pub(crate) const fn measurement_dispatch_argument() -> DtcmAddress",
    "pub(crate) const fn measurement_start_timestamp_word(index: usize) -> Option<DtcmAddress>",
    "pub(crate) const fn measurement_elapsed_timestamp_word(index: usize) -> Option<DtcmAddress>",
    "pub(crate) const fn measurement_scan_request() -> DtcmAddress",
    "pub(crate) const fn measurement_scan_request_mode() -> DtcmAddress",
    "assert_type_layout!(MeasurementWorkspace, 0x68, 4)",
    "offset_of!(MeasurementWorkspace, dwell_bound) == 0x00",
    "offset_of!(MeasurementWorkspace, measurement_type) == 0x05",
    "offset_of!(MeasurementWorkspace, completion_status) == 0x08",
    "offset_of!(MeasurementWorkspace, dispatch_state) == 0x11",
    "offset_of!(MeasurementWorkspace, dispatch_argument) == 0x12",
    "offset_of!(MeasurementWorkspace, start_timestamp_words) == 0x18",
    "offset_of!(MeasurementWorkspace, elapsed_timestamp_words) == 0x20",
    "offset_of!(MeasurementWorkspace, scan_request_prefix) == 0x28",
    "offset_of!(MeasurementWorkspace, scan_request_mode) == 0x29",
    "offset_of!(InitializedDtcmPrefix, ps_wake_guard_words) == 0x10e4",
    "offset_of!(InitializedDtcmPrefix, measurement_workspace) == 0x10f8",
    "offset_of!(InitializedDtcmPrefix, tx_aggregate_expiration_delta) == 0x1160",
    "offset_of!(InitializedDtcmPrefix, debug_command_descriptors) == 0x1164",
    "offset_of!(InitializedDtcmPrefix, hif_control) == 0x11ac",
    "assert_type_layout!(InitializedDtcmPrefix, 0x2078, 4)",
    "assert_type_layout!(DtcmLayout, DTCM_STATE_SIZE, 4)",
    "assert_type_layout!(SharedDtcmState, DTCM_STATE_SIZE, 4)",
)
FORBIDDEN_API = (
    "measurement_workspace_ptr", "measurement_workspace_ref",
    "measurement_workspace_mut", "measurement_workspace_offset",
    "measurement_opaque", "measurement_workspace_bytes",
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
                    f"{relative}:{line}: measurement-workspace literal {match.group()} "
                    "is outside dtcm.rs"
                )
    if failures:
        raise SystemExit("\n".join(failures))
    print(
        "MEASUREMENT WORKSPACE SOURCE/LAYOUT DRIFT-EVIDENCE GATE PASSED "
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
            "MEASUREMENT WORKSPACE LINKED DRIFT GATE FAILED\n"
            f"literals expected={dict(ALLOWED_LINKED_LITERALS)!r} actual={dict(literals)!r}\n"
            f"xrefs expected={dict(ALLOWED_DECODED_XREFS)!r} actual={dict(xrefs)!r}"
        )
    print(
        "MEASUREMENT WORKSPACE LINKED-XREF DRIFT-EVIDENCE GATE PASSED "
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
        raise SystemExit(f"measurement-workspace drift gate failed: {error}")
