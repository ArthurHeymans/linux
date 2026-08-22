#!/usr/bin/env python3
"""Source/linked drift evidence for [0x0400123c, 0x04001240).

This pins source ownership plus empty aligned linked-literal and decoded
PC-relative-xref multisets. It is drift evidence, not writer closure:
register-computed, indirect, generic HIF/debug, vendor, IRQ, and FIQ mutation
remain possible.
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
RANGE = (0x0400123C, 0x04001240)
LITERAL = re.compile(r"0x[0-9a-fA-F_]+")
SOURCE_EXTENSIONS = {
    ".rs", ".py", ".sh", ".c", ".h", ".hh", ".hpp", ".hxx", ".cc",
    ".cpp", ".cxx", ".s", ".S", ".asm", ".inc", ".ld", ".lds",
    ".ldh", ".x", ".toml", ".mk",
}
SOURCE_FILENAMES = {"Makefile", "Kconfig"}
OWNER_FILES = {
    "src/dtcm.rs",
    "tools/check-initialized-phy-watchdog-counter-layout.py",
    # Reviewed exclusive end boundary of the adjacent callback table.
    "tools/check-initialized-irq-callbacks-layout.py",
}
ALLOWED_LINKED_LITERALS: collections.Counter[int] = collections.Counter()
ALLOWED_DECODED_XREFS: collections.Counter[tuple[str, int]] = collections.Counter()

REQUIRED = (
    "#[repr(C, align(4))] struct PhyWatchdogCounter { count: SharedU32 }",
    "irq_callbacks: [SharedU32; 32]",
    "phy_watchdog_counter: PhyWatchdogCounter",
    "multi_vif_beacon_timer_tail: InitializedMultiVifBeaconTimerTail",
    "ampdu_counters: AmpduTelemetryCounters",
    "pub(crate) const PHY_WATCHDOG_COUNTER: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, phy_watchdog_counter) + core::mem::offset_of!(PhyWatchdogCounter, count));",
    "assert_type_layout!(PhyWatchdogCounter, 0x04, 4)",
    "offset_of!(PhyWatchdogCounter, count) == 0",
    "offset_of!(InitializedVendorImage, phy_watchdog_counter) == 0x123c",
    "offset_of!(InitializedVendorImage, multi_vif_beacon_timer_tail) == 0x1240",
    "offset_of!(InitializedVendorImage, ampdu_counters) == 0x12a0",
    "assert_type_layout!(InitializedVendorImage, 0x2078, 4)",
    "assert_type_layout!(DtcmLayout, DTCM_STATE_SIZE, 4)",
    "assert_type_layout!(SharedDtcmState, DTCM_STATE_SIZE, 4)",
    "fn initialized_phy_watchdog_counter_address_is_exact()",
    "IRQ_CALLBACK_TABLE.get() + core::mem::size_of::<[SharedU32; 32]>()",
    "PHY_WATCHDOG_COUNTER.get(), 0x0400_123c",
    "PHY_WATCHDOG_COUNTER.get() + core::mem::size_of::<PhyWatchdogCounter>()",
    "offset_of!(InitializedVendorImage, multi_vif_beacon_timer_tail), 0x1240",
    "size_of::<InitializedMultiVifBeaconTimerTail>(), 0x60",
    "AMPDU_TELEMETRY_COUNTERS.get(), 0x0400_12a0",
    "size_of::<InitializedVendorImage>(), 0x2078",
    "align_of::<InitializedVendorImage>(), 4",
    "size_of::<DtcmLayout>(), DTCM_STATE_SIZE",
    "align_of::<DtcmLayout>(), 4",
    "size_of::<SharedDtcmState>(), DTCM_STATE_SIZE",
    "align_of::<SharedDtcmState>(), 4",
)
FORBIDDEN_IDENTIFIERS = (
    "phy_watchdog_counter_ptr", "phy_watchdog_counter_pointer",
    "phy_watchdog_counter_ref", "phy_watchdog_counter_mut",
    "phy_watchdog_counter_value", "phy_watchdog_counter_read",
    "phy_watchdog_counter_write", "phy_watchdog_counter_offset",
    "phy_watchdog_counter_generic_offset", "phy_watchdog_counter_unchecked",
    "phy_watchdog_counter_slice", "phy_watchdog_counter_iter",
    "phy_watchdog_counter_get", "phy_watchdog_counter_set",
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
            elif source[index] == '"' or (source[index] == "'" and (single_quote_strings or re.match(r"'(?:\\.|[^\\'\n])'", source[index:]))):
                quote = source[index]; output.append(" "); index += 1; state = "string"
            else:
                output.append(source[index]); index += 1
        elif state == "line":
            if source[index] == "\n": output.append("\n"); state = "code"
            else: output.append(" ")
            index += 1
        elif state == "block":
            if source.startswith("/*", index): output.extend("  "); index += 2; depth += 1
            elif source.startswith("*/", index):
                output.extend("  "); index += 2; depth -= 1
                if depth == 0: state = "code"
            else: output.append("\n" if source[index] == "\n" else " "); index += 1
        elif source[index] == "\\" and index + 1 < len(source):
            output.extend("  "); index += 2
        elif source[index] == quote:
            output.append(" "); index += 1; state = "code"
        else:
            output.append("\n" if source[index] == "\n" else " "); index += 1
    return "".join(output)


def mask_test_module(code: str) -> str:
    marker = re.search(r"#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]\s*mod\s+tests\s*\{", code)
    if marker is None:
        return code
    return code[:marker.start()] + re.sub(r"[^\n]", " ", code[marker.start():])


def normalized(source: str) -> str:
    return " ".join(source.split())


def source_paths() -> list[Path]:
    return sorted(path for path in ROOT.rglob("*") if path.is_file()
                  and (path.suffix in SOURCE_EXTENSIONS or path.name in SOURCE_FILENAMES)
                  and "target" not in path.parts and ".git" not in path.parts)


def in_range(value: int) -> bool:
    return RANGE[0] <= value < RANGE[1]


def check_source() -> None:
    source = (ROOT / "src/dtcm.rs").read_text()
    compact = normalized(source)
    failures = [f"src/dtcm.rs: missing exact inventory: {item}" for item in REQUIRED if normalized(item) not in compact]
    if source.count("struct PhyWatchdogCounter") != 1 or source.count("const PHY_WATCHDOG_COUNTER:") != 1:
        failures.append("src/dtcm.rs: watchdog struct and sole address constant must each occur exactly once")
    declaration = re.search(r"(?:#\[[^\n]*\]\s*)*#\[repr\(C, align\(4\)\)\] struct PhyWatchdogCounter \{ count: SharedU32 \}", source)
    if declaration is None or "derive" in declaration.group():
        failures.append("src/dtcm.rs: PhyWatchdogCounter must be the exact non-derived quarantine struct")
    production = mask_test_module(code_only(source, False, False))
    for identifier in FORBIDDEN_IDENTIFIERS:
        if re.search(rf"\b{identifier}\b", production):
            failures.append(f"src/dtcm.rs: forbidden watchdog API {identifier}")
    for match in re.finditer(r"(?:pub(?:\(crate\))?\s+)?(?:const\s+)?(?:unsafe\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*watchdog[A-Za-z0-9_]*)", production, re.I):
        failures.append(f"src/dtcm.rs: watchdog function API is forbidden: {match.group(1)}")
    # No second constant, pointer/reference/value operation, or generic offset API may mention this view.
    for line_number, line in enumerate(production.splitlines(), 1):
        if "PhyWatchdogCounter" not in line and "PHY_WATCHDOG_COUNTER" not in line and "phy_watchdog_counter" not in line:
            continue
        if re.search(r"\*(?:const|mut)|&(?:mut\s+)?|read(?:_volatile)?\s*\(|write(?:_volatile)?\s*\(|unchecked|\b(?:slice|iter)\b", line):
            failures.append(f"src/dtcm.rs:{line_number}: pointer/reference/value/read/write/unchecked watchdog API")
    for path in source_paths():
        relative = path.relative_to(ROOT).as_posix()
        if relative in OWNER_FILES:
            continue
        code = code_only(path.read_text(errors="replace"), path.suffix in {".py", ".sh", ".toml"}, path.suffix in {".py", ".sh", ".toml"})
        if relative.endswith(".rs"):
            code = mask_test_module(code)
        for match in LITERAL.finditer(code):
            value = int(match.group().replace("_", ""), 16)
            if in_range(value):
                line = code.count("\n", 0, match.start()) + 1
                failures.append(f"{relative}:{line}: PHY-watchdog physical literal {match.group()} is outside reviewed owners")
    if failures:
        raise SystemExit("\n".join(failures))
    print(f"INITIALIZED PHY WATCHDOG COUNTER SOURCE DRIFT-EVIDENCE GATE PASSED files={len(source_paths())}")


def linked_literals(path: Path) -> collections.Counter[int]:
    with tempfile.NamedTemporaryFile() as output:
        subprocess.run(["llvm-objcopy", "--dump-section", f".text={output.name}", str(path), "/dev/null"], check=True)
        data = Path(output.name).read_bytes()
    result: collections.Counter[int] = collections.Counter()
    for offset in range(0, len(data) - 3, 4):
        value = struct.unpack_from("<I", data, offset)[0]
        if in_range(value): result[value] += 1
    return result


def decoded_literal_xrefs(path: Path) -> collections.Counter[tuple[str, int]]:
    disassembly = subprocess.run(["llvm-objdump", "-d", "--print-imm-hex", str(path)], check=True, text=True, stdout=subprocess.PIPE).stdout
    words = {int(a, 16): int(v, 16) for a, v in re.findall(r"^\s*([0-9a-fA-F]+):.*?\.word\s+0x([0-9a-fA-F]+)\s*$", disassembly, re.MULTILINE)}
    symbol = "<outside-symbol>"
    result: collections.Counter[tuple[str, int]] = collections.Counter()
    for line in disassembly.splitlines():
        if header := re.match(r"^[0-9a-fA-F]+ <(.+)>:$", line): symbol = header.group(1); continue
        if ".word" in line: continue
        reference = re.search(r"@\s+0x([0-9a-fA-F]+)(?:\s|<|$)", line)
        if reference is not None and (value := words.get(int(reference.group(1), 16))) is not None and in_range(value):
            result[(symbol, value)] += 1
    return result


def check_elf(path: Path, dump: bool) -> None:
    literals, xrefs = linked_literals(path), decoded_literal_xrefs(path)
    if dump:
        print(f"ALLOWED_LINKED_LITERALS={dict(sorted(literals.items()))!r}")
        print(f"ALLOWED_DECODED_XREFS={dict(sorted(xrefs.items()))!r}")
        return
    if literals != ALLOWED_LINKED_LITERALS or xrefs != ALLOWED_DECODED_XREFS:
        raise SystemExit(f"INITIALIZED PHY WATCHDOG COUNTER LINKED DRIFT GATE FAILED\nliterals={dict(literals)!r}\nxrefs={dict(xrefs)!r}")
    print("INITIALIZED PHY WATCHDOG COUNTER LINKED DRIFT-EVIDENCE GATE PASSED literals=0 decoded_xrefs=0")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("elf", nargs="?", type=Path)
    parser.add_argument("--dump", action="store_true")
    args = parser.parse_args()
    check_source()
    if args.elf is not None: check_elf(args.elf, args.dump)


if __name__ == "__main__":
    try: main()
    except (OSError, subprocess.CalledProcessError) as error:
        raise SystemExit(f"initialized-PHY-watchdog-counter drift gate failed: {error}")
