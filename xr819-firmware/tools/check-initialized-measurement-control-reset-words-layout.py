#!/usr/bin/env python3
"""Source/linked drift evidence for [0x04001294, 0x040012A0).

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
RANGE = (0x04001294, 0x040012A0)
OWNED_OFFSETS = {0x1294, 0x1298, 0x129C}
LITERAL = re.compile(r"0x[0-9a-fA-F_]+")
SOURCE_EXTENSIONS = {
    ".rs", ".py", ".sh", ".c", ".h", ".hh", ".hpp", ".hxx", ".cc",
    ".cpp", ".cxx", ".s", ".S", ".asm", ".inc", ".ld", ".lds",
    ".ldh", ".x", ".toml", ".mk",
}
SOURCE_FILENAMES = {"Makefile", "Kconfig"}
OWNER_FILES = {
    "tools/check-initialized-measurement-control-reset-words-layout.py",
}
ALLOWED_LINKED_LITERALS: collections.Counter[int] = collections.Counter()
ALLOWED_DECODED_XREFS: collections.Counter[tuple[str, int]] = collections.Counter()

REQUIRED = (
    "#[repr(C, align(4))] struct InitializedMultiVifBeaconTimerTail { opaque_00: OpaqueBytes<0x0f>, interface_2_radio_latch: SharedU8, timer: TimerEntry, opaque_24: OpaqueBytes<0x18>, measurement_dwell_timer: TimerEntry, dtim_capture_latch: SharedU8, opaque_51: OpaqueBytes<0x03>, measurement_control_word_0: SharedU32, measurement_control_word_1: SharedU32, measurement_control_word_2: SharedU32 }",
    "phy_watchdog_counter: PhyWatchdogCounter, multi_vif_beacon_timer_tail: InitializedMultiVifBeaconTimerTail",
    "ampdu_counters: AmpduTelemetryCounters",
    "assert_type_layout!(SharedU32, 0x04, 4)",
    "assert_type_layout!(InitializedMultiVifBeaconTimerTail, 0x60, 4)",
    "offset_of!(InitializedMultiVifBeaconTimerTail, interface_2_radio_latch) == 0x0f",
    "offset_of!(InitializedMultiVifBeaconTimerTail, dtim_capture_latch) == 0x50",
    "offset_of!(InitializedMultiVifBeaconTimerTail, opaque_51) == 0x51",
    "offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_0) == 0x54",
    "offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_1) == 0x58",
    "offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_2) == 0x5c",
    "offset_of!(InitializedVendorImage, multi_vif_beacon_timer_tail) == 0x1240",
    "offset_of!(InitializedVendorImage, ampdu_counters) == 0x12a0",
    "assert_type_layout!(InitializedVendorImage, 0x2078, 4)",
    "assert_type_layout!(DtcmLayout, DTCM_STATE_SIZE, 4)",
    "assert_type_layout!(SharedDtcmState, DTCM_STATE_SIZE, 4)",
    "fn initialized_measurement_control_reset_words_are_exact()",
    "let tail = DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, multi_vif_beacon_timer_tail)",
    "assert_eq!(tail, 0x0400_1240)",
    "MEASUREMENT_DWELL_TIMER.get() + core::mem::size_of::<TimerEntry>(), 0x0400_1290",
    "offset_of!(InitializedMultiVifBeaconTimerTail, dtim_capture_latch), 0x0400_1290",
    "offset_of!(InitializedMultiVifBeaconTimerTail, opaque_51) + core::mem::size_of::<OpaqueBytes<0x03>>(), 0x0400_1294",
    "[0x0400_1294, 0x0400_1298, 0x0400_129c]",
    "size_of::<SharedU32>(), 0x04",
    "offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_2) + core::mem::size_of::<SharedU32>(), 0x0400_12a0",
    "AMPDU_TELEMETRY_COUNTERS.get(), 0x0400_12a0",
    "size_of::<InitializedMultiVifBeaconTimerTail>(), 0x60",
    "align_of::<InitializedMultiVifBeaconTimerTail>(), 4",
    "size_of::<InitializedVendorImage>(), 0x2078",
    "align_of::<InitializedVendorImage>(), 4",
    "size_of::<DtcmLayout>(), DTCM_STATE_SIZE",
    "align_of::<DtcmLayout>(), 4",
    "size_of::<SharedDtcmState>(), DTCM_STATE_SIZE",
    "align_of::<SharedDtcmState>(), 4",
)
ALLOWED_COMPILE_ASSERTIONS = (
    "assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, multi_vif_beacon_timer_tail) + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, opaque_51) + core::mem::size_of::<OpaqueBytes<0x03>>() == 0x0400_1294);",
    "assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, multi_vif_beacon_timer_tail) + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_0) + core::mem::size_of::<SharedU32>() == 0x0400_1298);",
    "assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, multi_vif_beacon_timer_tail) + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_1) + core::mem::size_of::<SharedU32>() == 0x0400_129c);",
)
FORBIDDEN_IDENTIFIERS = tuple(
    f"measurement_control_word_{index}_{suffix}"
    for index in range(3)
    for suffix in (
        "ptr", "pointer", "ref", "mut", "value", "read", "write", "reset",
        "init", "offset", "generic_offset", "unchecked", "slice", "iter",
        "get", "set",
    )
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


def owned_address_literals(code: str) -> list[int]:
    values = [int(match.group().replace("_", ""), 16) for match in LITERAL.finditer(code)]
    return [value for value in values if in_range(value) or value in OWNED_OFFSETS]


def timer_view_aliases(code: str) -> tuple[set[str], list[tuple[str, str, str]]]:
    declarations = [
        (kind, name, initializer)
        for kind, pattern in (
            ("const", r"\bconst\s+([A-Za-z_][A-Za-z0-9_]*)\s*:[^=;]+\s*=\s*([^;]*);"),
            ("static", r"\bstatic\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:[^=;]+\s*=\s*([^;]*);"),
            ("type", r"\btype\s+([A-Za-z_][A-Za-z0-9_]*)\s*=\s*([^;]*);"),
        )
        for name, initializer in re.findall(pattern, code)
    ]
    imported_aliases = rust_use_aliases(code)
    views = {
        "InitializedMultiVifBeaconTimerTail",
        "measurement_control_word_0",
        "measurement_control_word_1",
        "measurement_control_word_2",
        *(name for _, name, initializer in declarations if owned_address_literals(initializer)),
    }
    while True:
        aliases = {
            name
            for _, name, initializer in declarations
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
        body = code[start.start():index]
        if view_pattern.search(body) or owned_address_literals(body):
            consumers.append(start.group(1))
    return consumers


def check_alias_tracking_regression() -> None:
    fixture = """
        const WORD_ALIAS: usize = measurement_control_word_0;
        const SECOND_ALIAS: usize = WORD_ALIAS;
        fn generic_view() -> *mut u32 { SECOND_ALIAS as *mut u32 }
    """
    views, _ = timer_view_aliases(fixture)
    if not {"WORD_ALIAS", "SECOND_ALIAS"} <= views or functions_consuming_timer_view(
        fixture, views
    ) != ["generic_view"]:
        raise SystemExit("checker self-test failed: transitive word alias was accepted")
    renamed_import = """
        use crate::dtcm::measurement_control_word_1 as WORD_IMPORT;
        const IMPORT_ALIAS: usize = WORD_IMPORT;
        fn imported_view() -> usize { IMPORT_ALIAS }
    """
    views, _ = timer_view_aliases(renamed_import)
    if not {"WORD_IMPORT", "IMPORT_ALIAS"} <= views or functions_consuming_timer_view(
        renamed_import, views
    ) != ["imported_view"]:
        raise SystemExit("checker self-test failed: renamed word import was accepted")
    grouped_import = """
        use crate::dtcm::{
            measurement_control_word_2 as GROUP_WORD,
            InitializedMultiVifBeaconTimerTail as GROUP_TAIL,
        };
        const TRANSITIVE_GROUP_WORD: usize = GROUP_WORD;
        fn grouped_view() -> usize { TRANSITIVE_GROUP_WORD }
        fn grouped_type_view(_: GROUP_TAIL) {}
    """
    views, _ = timer_view_aliases(grouped_import)
    if not {"GROUP_WORD", "GROUP_TAIL", "TRANSITIVE_GROUP_WORD"} <= views or functions_consuming_timer_view(
        grouped_import, views
    ) != ["grouped_view", "grouped_type_view"]:
        raise SystemExit("checker self-test failed: grouped aliases were accepted")
    static_alias = """
        static HIDDEN_WORD: usize = measurement_control_word_0;
        static SECOND_HIDDEN_WORD: usize = HIDDEN_WORD;
        fn hidden_address() -> usize { SECOND_HIDDEN_WORD }
    """
    views, declarations = timer_view_aliases(static_alias)
    if not {"HIDDEN_WORD", "SECOND_HIDDEN_WORD"} <= views or not any(
        kind == "static" and name == "HIDDEN_WORD" for kind, name, _ in declarations
    ) or functions_consuming_timer_view(static_alias, views) != ["hidden_address"]:
        raise SystemExit("checker self-test failed: static word alias was accepted")
    type_alias = """
        type HiddenTail = InitializedMultiVifBeaconTimerTail;
        type SecondHiddenTail = HiddenTail;
        fn hidden_tail(_: SecondHiddenTail) {}
    """
    views, declarations = timer_view_aliases(type_alias)
    if not {"HiddenTail", "SecondHiddenTail"} <= views or not any(
        kind == "type" and name == "HiddenTail" for kind, name, _ in declarations
    ) or functions_consuming_timer_view(type_alias, views) != ["hidden_tail"]:
        raise SystemExit("checker self-test failed: type alias was accepted")
    raw_physical_write = """
        fn reset() { unsafe { core::ptr::write_volatile(0x0400_1294 as *mut u32, 0) } }
    """
    views, _ = timer_view_aliases(raw_physical_write)
    if functions_consuming_timer_view(raw_physical_write, views) != ["reset"]:
        raise SystemExit("checker self-test failed: raw physical pointer/write API was accepted")
    renamed_offset = """
        const RAW_WORD: DtcmAddress = DtcmAddress::from_offset(0x1294);
        const RENAMED_WORD: DtcmAddress = RAW_WORD;
        const TRANSITIVE_WORD: DtcmAddress = RENAMED_WORD;
        fn hidden_offset_api() -> DtcmAddress { TRANSITIVE_WORD }
    """
    views, declarations = timer_view_aliases(renamed_offset)
    if not {"RAW_WORD", "RENAMED_WORD", "TRANSITIVE_WORD"} <= views or not any(
        kind == "const" and name == "RAW_WORD" for kind, name, _ in declarations
    ) or functions_consuming_timer_view(renamed_offset, views) != ["hidden_offset_api"]:
        raise SystemExit("checker self-test failed: renamed/transitive DTCM offset API was accepted")


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
    if source.count("struct InitializedMultiVifBeaconTimerTail") != 1:
        failures.append("src/dtcm.rs: measurement-control words require exactly one enclosing tail struct")
    declaration = re.search(r"(?:#\[[^\n]*\]\s*)*#\[repr\(C, align\(4\)\)\] struct InitializedMultiVifBeaconTimerTail \{ opaque_00: OpaqueBytes<0x0f>, interface_2_radio_latch: SharedU8, timer: TimerEntry, opaque_24: OpaqueBytes<0x18>, measurement_dwell_timer: TimerEntry, dtim_capture_latch: SharedU8, opaque_51: OpaqueBytes<0x03>, measurement_control_word_0: SharedU32, measurement_control_word_1: SharedU32, measurement_control_word_2: SharedU32 \}", source)
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
    checked_rust = dict(rust_production)
    for assertion in ALLOWED_COMPILE_ASSERTIONS:
        if production.count(assertion) != 1:
            failures.append(f"src/dtcm.rs: reviewed compile-time boundary assertion is not unique: {assertion}")
        checked_rust["src/dtcm.rs"] = checked_rust["src/dtcm.rs"].replace(
            assertion, re.sub(r"[^\n]", " ", assertion)
        )
    for relative, code in rust_production.items():
        for identifier in FORBIDDEN_IDENTIFIERS:
            if re.search(rf"\b{identifier}\b", code):
                failures.append(f"{relative}: forbidden measurement-control word API {identifier}")
        for match in re.finditer(r"(?:pub(?:\(crate\))?\s+)?(?:const\s+)?(?:unsafe\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*measurement_control_(?:word|reset)[A-Za-z0-9_]*)", code, re.I):
            failures.append(f"{relative}: measurement-control function API is forbidden: {match.group(1)}")
    # Reject renamed constants, statics, and type aliases, including transitive
    # aliases of the sole address/type views, plus generic functions that hide
    # these views behind an evidence-free API name anywhere in production Rust.
    # Compile-time assertions are unnamed constants.
    timer_views, constant_declarations = timer_view_aliases(
        "\n".join(checked_rust.values())
    )
    for kind, name, _ in constant_declarations:
        if name in timer_views and not (
            kind == "const"
            and name in {"MEASUREMENT_DWELL_TIMER", "MULTI_VIF_BEACON_TIMER"}
        ):
            failures.append(f"additional measurement-control word {kind} alias is forbidden: {name}")
    timer_view_pattern = re.compile(
        rf"\b(?:{'|'.join(sorted(map(re.escape, timer_views)))})\b"
    )
    for relative, code in checked_rust.items():
        for function in functions_consuming_timer_view(code, timer_views):
            failures.append(f"{relative}: function API over measurement-control storage is forbidden: {function}")
        for line_number, line in enumerate(code.splitlines(), 1):
            if timer_view_pattern.search(line) is not None and re.search(r"\*(?:const|mut)|&(?:mut\s+)?|read(?:_volatile)?\s*\(|write(?:_volatile)?\s*\(|unchecked|\b(?:slice|iter)\b", line):
                failures.append(f"{relative}:{line_number}: pointer/reference/value/read/write/unchecked measurement-control API")
            for value in owned_address_literals(line):
                if value in OWNED_OFFSETS:
                    failures.append(
                        f"{relative}:{line_number}: raw measurement-control DTCM offset literal 0x{value:x} is forbidden"
                    )
    for path in paths:
        relative = path.relative_to(ROOT).as_posix()
        if relative in OWNER_FILES:
            continue
        code = checked_rust[relative] if relative.endswith(".rs") else code_only(
            path.read_text(errors="replace"),
            path.suffix in {".py", ".sh", ".toml"},
            path.suffix in {".py", ".sh", ".toml"},
        )
        for match in LITERAL.finditer(code):
            value = int(match.group().replace("_", ""), 16)
            if in_range(value):
                line = code.count("\n", 0, match.start()) + 1
                failures.append(f"{relative}:{line}: measurement-control physical literal {match.group()} is outside reviewed owners")
    if failures:
        raise SystemExit("\n".join(failures))
    print(f"INITIALIZED MEASUREMENT CONTROL RESET WORDS SOURCE DRIFT-EVIDENCE GATE PASSED files={len(paths)}")


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
        raise SystemExit(f"INITIALIZED MEASUREMENT CONTROL RESET WORDS LINKED DRIFT GATE FAILED\nliterals={dict(literals)!r}\nxrefs={dict(xrefs)!r}")
    print("INITIALIZED MEASUREMENT CONTROL RESET WORDS LINKED DRIFT-EVIDENCE GATE PASSED literals=0 decoded_xrefs=0")


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
        raise SystemExit(f"initialized-measurement-control-reset-words drift gate failed: {error}")
