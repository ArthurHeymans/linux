#!/usr/bin/env python3
"""Source/linked drift evidence for [0x04001250, 0x04001264).

This pins source ownership plus empty aligned linked-literal and decoded
PC-relative-xref multisets. Empty linked sets are drift evidence, not writer
closure: register-computed, indirect, generic HIF/debug, vendor, IRQ, and FIQ
mutation remain possible.
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
RANGE = (0x04001250, 0x04001264)
LITERAL = re.compile(r"0x[0-9a-fA-F_]+")
SOURCE_EXTENSIONS = {
    ".rs", ".py", ".sh", ".c", ".h", ".hh", ".hpp", ".hxx", ".cc",
    ".cpp", ".cxx", ".s", ".S", ".asm", ".inc", ".ld", ".lds",
    ".ldh", ".x", ".toml", ".mk",
}
SOURCE_FILENAMES = {"Makefile", "Kconfig"}
OWNER_FILES = {
    "src/dtcm.rs",
    "tools/check-initialized-multi-vif-beacon-timer-layout.py",
}
ALLOWED_LINKED_LITERALS: collections.Counter[int] = collections.Counter()
ALLOWED_DECODED_XREFS: collections.Counter[tuple[str, int]] = collections.Counter()

REQUIRED = (
    "#[repr(C, align(4))] struct InitializedMultiVifBeaconTimerTail { opaque_00: OpaqueBytes<0x10>, timer: TimerEntry, opaque_24: OpaqueBytes<0x3c> }",
    "struct TimerEntry { next: SharedU32, previous_link: SharedU32, deadline: SharedU32, callback: SharedU32, context: SharedU32 }",
    "phy_watchdog_counter: PhyWatchdogCounter, multi_vif_beacon_timer_tail: InitializedMultiVifBeaconTimerTail",
    "ampdu_counters: AmpduTelemetryCounters",
    "pub(crate) const MULTI_VIF_BEACON_TIMER: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, multi_vif_beacon_timer_tail) + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, timer));",
    "assert_type_layout!(TimerEntry, 0x14, 4)",
    "offset_of!(TimerEntry, next) == 0x00",
    "offset_of!(TimerEntry, previous_link) == 0x04",
    "offset_of!(TimerEntry, deadline) == 0x08",
    "offset_of!(TimerEntry, callback) == 0x0c",
    "offset_of!(TimerEntry, context) == 0x10",
    "assert_type_layout!(InitializedMultiVifBeaconTimerTail, 0x60, 4)",
    "offset_of!(InitializedMultiVifBeaconTimerTail, opaque_00) == 0",
    "offset_of!(InitializedMultiVifBeaconTimerTail, timer) == 0x10",
    "offset_of!(InitializedMultiVifBeaconTimerTail, opaque_24) == 0x24",
    "offset_of!(InitializedVendorImage, multi_vif_beacon_timer_tail) == 0x1240",
    "offset_of!(InitializedVendorImage, ampdu_counters) == 0x12a0",
    "assert_type_layout!(InitializedVendorImage, 0x2078, 4)",
    "assert_type_layout!(DtcmLayout, DTCM_STATE_SIZE, 4)",
    "assert_type_layout!(SharedDtcmState, DTCM_STATE_SIZE, 4)",
    "fn initialized_multi_vif_beacon_timer_address_is_exact()",
    "size_of::<TimerEntry>(), 0x14",
    "align_of::<TimerEntry>(), 4",
    "MULTI_VIF_BEACON_TIMER.get(), 0x0400_1250",
    "MULTI_VIF_BEACON_TIMER.get() + core::mem::size_of::<TimerEntry>(), 0x0400_1264",
    "size_of::<OpaqueBytes<0x3c>>()",
    "AMPDU_TELEMETRY_COUNTERS.get(), 0x0400_12a0",
    "size_of::<InitializedVendorImage>(), 0x2078",
    "align_of::<InitializedVendorImage>(), 4",
    "size_of::<DtcmLayout>(), DTCM_STATE_SIZE",
    "align_of::<DtcmLayout>(), 4",
    "size_of::<SharedDtcmState>(), DTCM_STATE_SIZE",
    "align_of::<SharedDtcmState>(), 4",
)
FORBIDDEN_IDENTIFIERS = (
    "multi_vif_beacon_timer_ptr", "multi_vif_beacon_timer_pointer",
    "multi_vif_beacon_timer_ref", "multi_vif_beacon_timer_mut",
    "multi_vif_beacon_timer_value", "multi_vif_beacon_timer_read",
    "multi_vif_beacon_timer_write", "multi_vif_beacon_timer_offset",
    "multi_vif_beacon_timer_generic_offset", "multi_vif_beacon_timer_unchecked",
    "multi_vif_beacon_timer_slice", "multi_vif_beacon_timer_iter",
    "multi_vif_beacon_timer_get", "multi_vif_beacon_timer_set",
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


def rust_use_aliases(code: str) -> list[tuple[str, str]]:
    """Return local-name -> imported-leaf aliases for plain and grouped uses."""
    aliases: list[tuple[str, str]] = []
    for match in re.finditer(
        r"\buse\s+(?:[A-Za-z_][A-Za-z0-9_]*::)*"
        r"([A-Za-z_][A-Za-z0-9_]*)\s+as\s+([A-Za-z_][A-Za-z0-9_]*)\s*;",
        code,
    ):
        imported, local = match.groups()
        aliases.append((local, imported))
    for match in re.finditer(
        r"\buse\s+(?:[A-Za-z_][A-Za-z0-9_]*::)*\{([^{};]*)\}\s*;",
        code,
    ):
        for item in match.group(1).split(","):
            item = item.strip()
            alias = re.fullmatch(
                r"(?:[A-Za-z_][A-Za-z0-9_]*::)*"
                r"([A-Za-z_][A-Za-z0-9_]*)\s+as\s+([A-Za-z_][A-Za-z0-9_]*)",
                item,
            )
            if alias is not None:
                imported, local = alias.groups()
                aliases.append((local, imported))
    return aliases


def timer_view_aliases(code: str) -> tuple[set[str], list[tuple[str, str]]]:
    declarations = [
        match.groups()
        for match in re.finditer(
            r"\bconst\s+([A-Za-z_][A-Za-z0-9_]*)\s*:[^=;]+\s*=\s*([^;]*);",
            code,
        )
    ]
    imported_aliases = rust_use_aliases(code)
    views = {
        "InitializedMultiVifBeaconTimerTail",
        "MULTI_VIF_BEACON_TIMER",
        "multi_vif_beacon_timer_tail",
    }
    while True:
        aliases = {
            name
            for name, initializer in declarations
            if set(re.findall(r"\b[A-Za-z_][A-Za-z0-9_]*\b", initializer)) & views
        }
        aliases |= {
            local for local, imported in imported_aliases if imported in views
        }
        expanded = views | aliases
        if expanded == views:
            return views, declarations
        views = expanded


def functions_consuming_timer_view(code: str, views: set[str]) -> list[str]:
    view_pattern = re.compile(
        rf"\b(?:{'|'.join(sorted(map(re.escape, views)))})\b"
    )
    consumers: list[str] = []
    function_starts = re.finditer(
        r"\b(?:unsafe\s+)?(?:const\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)[^\{;]*\{",
        code,
    )
    for start in function_starts:
        depth = 1
        index = start.end()
        while index < len(code) and depth:
            depth += (code[index] == "{") - (code[index] == "}")
            index += 1
        if view_pattern.search(code[start.start():index]):
            consumers.append(start.group(1))
    return consumers


def check_alias_tracking_regression() -> None:
    fixture = """
        const MULTI_VIF_BEACON_TIMER: DtcmAddress =
            InitializedMultiVifBeaconTimerTail::timer;
        const TIMER_ALIAS: DtcmAddress = MULTI_VIF_BEACON_TIMER;
        const SECOND_ALIAS: DtcmAddress = TIMER_ALIAS;
        fn generic_view() -> *mut u32 { SECOND_ALIAS.get() as *mut u32 }
    """
    views, _ = timer_view_aliases(fixture)
    if not {"TIMER_ALIAS", "SECOND_ALIAS"} <= views or functions_consuming_timer_view(
        fixture, views
    ) != ["generic_view"]:
        raise SystemExit(
            "checker self-test failed: aliased multi-VIF timer API was accepted"
        )
    renamed_import = """
        use crate::dtcm::MULTI_VIF_BEACON_TIMER as TIMER_IMPORT;
        const IMPORT_ALIAS: DtcmAddress = TIMER_IMPORT;
        fn imported_view() -> usize { IMPORT_ALIAS.get() }
    """
    views, _ = timer_view_aliases(renamed_import)
    if not {"TIMER_IMPORT", "IMPORT_ALIAS"} <= views or functions_consuming_timer_view(
        renamed_import, views
    ) != ["imported_view"]:
        raise SystemExit(
            "checker self-test failed: renamed imported timer alias was accepted"
        )
    grouped_import = """
        use crate::dtcm::{
            MULTI_VIF_BEACON_TIMER as GROUP_TIMER,
            InitializedMultiVifBeaconTimerTail as GROUP_TAIL,
        };
        const TRANSITIVE_GROUP_TIMER: DtcmAddress = GROUP_TIMER;
        fn grouped_view() -> usize { TRANSITIVE_GROUP_TIMER.get() }
        fn grouped_type_view(_: GROUP_TAIL) {}
    """
    views, _ = timer_view_aliases(grouped_import)
    if not {"GROUP_TIMER", "GROUP_TAIL", "TRANSITIVE_GROUP_TIMER"} <= views or functions_consuming_timer_view(
        grouped_import, views
    ) != ["grouped_view", "grouped_type_view"]:
        raise SystemExit(
            "checker self-test failed: grouped imported timer aliases were accepted"
        )


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
    if source.count("struct InitializedMultiVifBeaconTimerTail") != 1 or source.count("const MULTI_VIF_BEACON_TIMER:") != 1:
        failures.append("src/dtcm.rs: timer-tail struct and sole address constant must each occur exactly once")
    declaration = re.search(r"(?:#\[[^\n]*\]\s*)*#\[repr\(C, align\(4\)\)\] struct InitializedMultiVifBeaconTimerTail \{ opaque_00: OpaqueBytes<0x10>, timer: TimerEntry, opaque_24: OpaqueBytes<0x3c> \}", source)
    if declaration is None or "derive" in declaration.group():
        failures.append("src/dtcm.rs: InitializedMultiVifBeaconTimerTail must be the exact non-derived quarantine struct")
    paths = source_paths()
    rust_production = {
        path.relative_to(ROOT).as_posix(): mask_test_module(
            code_only(path.read_text(errors="replace"), False, False)
        )
        for path in paths
        if path.suffix == ".rs"
    }
    production = rust_production["src/dtcm.rs"]
    for relative, code in rust_production.items():
        for identifier in FORBIDDEN_IDENTIFIERS:
            if re.search(rf"\b{identifier}\b", code):
                failures.append(f"{relative}: forbidden multi-VIF timer API {identifier}")
        for match in re.finditer(r"(?:pub(?:\(crate\))?\s+)?(?:const\s+)?(?:unsafe\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*multi_vif_beacon_timer[A-Za-z0-9_]*)", code, re.I):
            failures.append(f"{relative}: multi-VIF timer function API is forbidden: {match.group(1)}")
    # Reject renamed constants, including transitive aliases of the sole address
    # constant, and generic functions that hide this view behind an
    # evidence-free API name anywhere in production Rust. Compile-time
    # assertions are unnamed constants.
    timer_views, constant_declarations = timer_view_aliases(
        "\n".join(rust_production.values())
    )
    for name, _ in constant_declarations:
        if name in timer_views and name != "MULTI_VIF_BEACON_TIMER":
            failures.append(f"additional multi-VIF timer constant is forbidden: {name}")
    timer_view_pattern = re.compile(
        rf"\b(?:{'|'.join(sorted(map(re.escape, timer_views)))})\b"
    )
    for relative, code in rust_production.items():
        for function in functions_consuming_timer_view(code, timer_views):
            failures.append(f"{relative}: function API over the multi-VIF timer is forbidden: {function}")
        for line_number, line in enumerate(code.splitlines(), 1):
            if timer_view_pattern.search(line) is None:
                continue
            if re.search(r"\*(?:const|mut)|&(?:mut\s+)?|read(?:_volatile)?\s*\(|write(?:_volatile)?\s*\(|unchecked|\b(?:slice|iter)\b", line):
                failures.append(f"{relative}:{line_number}: pointer/reference/value/read/write/unchecked multi-VIF timer API")
    for path in paths:
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
                failures.append(f"{relative}:{line}: multi-VIF beacon timer physical literal {match.group()} is outside reviewed owners")
    if failures:
        raise SystemExit("\n".join(failures))
    print(f"INITIALIZED MULTI-VIF BEACON TIMER SOURCE DRIFT-EVIDENCE GATE PASSED files={len(paths)}")


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
        raise SystemExit(f"INITIALIZED MULTI-VIF BEACON TIMER LINKED DRIFT GATE FAILED\nliterals={dict(literals)!r}\nxrefs={dict(xrefs)!r}")
    print("INITIALIZED MULTI-VIF BEACON TIMER LINKED DRIFT-EVIDENCE GATE PASSED literals=0 decoded_xrefs=0")


def main() -> None:
    check_alias_tracking_regression()
    parser = argparse.ArgumentParser()
    parser.add_argument("elf", nargs="?", type=Path)
    parser.add_argument("--dump", action="store_true")
    args = parser.parse_args()
    check_source()
    if args.elf is not None: check_elf(args.elf, args.dump)


if __name__ == "__main__":
    try: main()
    except (OSError, subprocess.CalledProcessError) as error:
        raise SystemExit(f"initialized-multi-VIF-beacon-timer drift gate failed: {error}")
