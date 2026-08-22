#!/usr/bin/env python3
"""Source/linked drift evidence for exactly [0x04000DA8, 0x04000DD0).

The TLV dispatch handler table consumed by vendor tlv_dispatch_table: rooted
at literal-pool word DAT_0001752c = 0x04000da8, loaded exactly once (PC
0x17522) inside tlv_dispatch_default and passed as the handler table. The
dispatch loop walks 8-byte entries {key, handler} until handler == 0 and
calls the matching handler with the record payload. The recovered
initialization snapshot shows exactly four populated entries (keys 0, 1, 3,
4; handlers 0x000175d3/0x00017565/0x000177f5/0x00017535 -- all inside the
tlv_dispatch_default code blob) followed by the zero terminator entry,
bounding the table at [0x04000da8, 0x04000dd0). A whole-binary scan found no
other accessor of the interval (one incidental never-loaded word 0x04000db5).

No production operation touches this interval. Initial COPY values are
loader-owned; no writer closure is claimed: computed, indirect, generic
HIF/debug, vendor, IRQ, and FIQ mutation remain possible. The linked-literal
and decoded PC-relative-xref multisets are derived from decoded loads only
and pinned empty.
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
RANGE = (0x04000DA8, 0x04000DD0)
OFFSETS = range(0xDA8, 0xDD0)
LITERAL = re.compile(r"0x[0-9a-fA-F_]+")
SOURCE_EXTENSIONS = {
    ".rs", ".py", ".sh", ".c", ".h", ".hh", ".hpp", ".hxx", ".cc",
    ".cpp", ".cxx", ".s", ".S", ".asm", ".inc", ".ld", ".lds",
    ".ldh", ".x", ".toml", ".mk",
}
SOURCE_FILENAMES = {"Makefile", "Kconfig"}
OWNER_FILES = {
    "src/dtcm.rs",
    "tools/check-initialized-tlv-handler-table-layout.py",
    "tools/check-initialized-phy-gain-source-layout.py",
    "tools/check-initialized-rf-mode-halfword-table-layout.py",
}
ADJACENT_DECLARATIONS = {
    "tools/check-initialized-rf-mode-halfword-table-layout.py": 'STRUCT = "#[repr(C, align(2))] struct RfModeHalfwordTable { entries: [SharedU16; 36] }"',
    "tools/check-initialized-iq-calibration-gain-indices-layout.py": "INITIALIZED_IQ_CALIBRATION_GAIN_INDICES_RANGE = (0x04000E18, 0x04000E48)",
}
SANCTIONED_CONSUMER_LINES: dict[str, set[str]] = {}
SANCTIONED_CONSUMER_FUNCTIONS: dict[str, set[str]] = {}
# No linked literals fall inside this interval: the guard word is reached
# relative to the scheduler timer list head root (owned by the adjacent
# checker), so both multisets are pinned empty.
# One sanctioned linked literal: mac::initialize_tx_pipe_state writes
# DURATION_QUANTUM_POINTERS + pipe * 4 for pipes 0..4; LLVM emits the loop
# with base 0x040010e4 and negative register offsets (-16..-4), so the pool
# word VALUE lands inside this interval while every store targets the
# duration-quantum-pointers interval below. No byte of this record is
# written through it.
# No linked literals fall inside this interval: the handler-table root is
# consumed only by tlv_dispatch_default (vendor side), and no retained Rust
# code touches these bytes.
ALLOWED_LINKED_LITERALS: collections.Counter[int] = collections.Counter()
ALLOWED_DECODED_XREFS: collections.Counter[tuple[str, int]] = collections.Counter()
STRUCT = '#[repr(C, align(4))] struct TlvDispatchHandlerEntry { key: SharedU32, handler: SharedU32 } #[repr(C, align(4))] struct TlvDispatchHandlerTable { entries: [TlvDispatchHandlerEntry; 4], terminator_key: SharedU32, terminator_handler: SharedU32 }'
REQUIRED = (
    STRUCT,
    "tlv_dispatch_handler_table: TlvDispatchHandlerTable",
    "rf_mode_halfword_table: RfModeHalfwordTable",
    "assert_type_layout!(SharedU32, 0x04, 4)",
    "assert_type_layout!(TlvDispatchHandlerTable, 0x28, 4)",
    "offset_of!(TlvDispatchHandlerTable, entries) == 0",
    "size_of::<[TlvDispatchHandlerEntry; 4]>() == 0x20",
    "offset_of!(TlvDispatchHandlerTable, terminator_key) == 0x20",
    "offset_of!(TlvDispatchHandlerTable, terminator_handler) == 0x24",
    "offset_of!(InitializedVendorImage, tlv_dispatch_handler_table) == 0x0da8",
    "offset_of!(InitializedVendorImage, rf_mode_halfword_table) == 0x0dd0",
    "assert_type_layout!(InitializedVendorImage, 0x2078, 4)",
    "assert_type_layout!(DtcmLayout, DTCM_STATE_SIZE, 4)",
    "assert_type_layout!(SharedDtcmState, DTCM_STATE_SIZE, 4)",
    "fn tlv_dispatch_handler_table_is_exact()",
    "size_of::<TlvDispatchHandlerTable>(), 0x28",
    "align_of::<TlvDispatchHandlerTable>(), 4",
    "terminator_handler)], [0x20, 0x24]",
    "tlv, 0x0400_0da8",
)
PHYSICAL = (0x04000DA8, 0x04000DC8, 0x04000DD0)
COMPILE_TIME_PHYSICAL_INVENTORY = (
    "assert!(core::mem::offset_of!(InitializedVendorImage, tlv_dispatch_handler_table) == 0x0da8);",
    "assert_type_layout!(TlvDispatchHandlerTable, 0x28, 4);",
    "assert!(core::mem::offset_of!(TlvDispatchHandlerTable, entries) == 0);",
    "assert!(core::mem::size_of::<[TlvDispatchHandlerEntry; 4]>() == 0x20);",
    "assert!(core::mem::offset_of!(TlvDispatchHandlerTable, terminator_key) == 0x20);",
    "assert!(core::mem::offset_of!(TlvDispatchHandlerTable, terminator_handler) == 0x24);",
)
FOCUSED_TEST_INVENTORY = (
    'let image = DTCM_STATE_BASE;',
    'let tlv = image + core::mem::offset_of!(InitializedVendorImage, tlv_dispatch_handler_table);',
    'assert_eq!(core::mem::size_of::<TlvDispatchHandlerTable>(), 0x28);',
    'assert_eq!(core::mem::align_of::<TlvDispatchHandlerTable>(), 4);',
    'assert_eq!(core::mem::offset_of!(TlvDispatchHandlerTable, entries), 0);',
    'assert_eq!(core::mem::size_of::<[TlvDispatchHandlerEntry; 4]>(), 0x20);',
    'assert_eq!([core::mem::offset_of!(TlvDispatchHandlerTable, terminator_key), core::mem::offset_of!(TlvDispatchHandlerTable, terminator_handler)], [0x20, 0x24]);',
    'assert_eq!(tlv, 0x0400_0da8);',
    'assert_eq!(tlv + core::mem::offset_of!(TlvDispatchHandlerTable, terminator_key), 0x0400_0dc8);',
    'assert_eq!(tlv + core::mem::size_of::<TlvDispatchHandlerTable>(), 0x0400_0dd0);',
    'assert_eq!(RF_MODE_HALFWORD_TABLE.get(), 0x0400_0dd0);',
    'assert_eq!(INITIALIZED_IQ_CALIBRATION_GAIN_INDICES.get(), 0x0400_0e18);',
    'assert_eq!(core::mem::size_of::<InitializedVendorImage>(), 0x2078);',
    'assert_eq!(core::mem::align_of::<InitializedVendorImage>(), 4);',
    'assert_eq!(core::mem::size_of::<DtcmLayout>(), DTCM_STATE_SIZE);',
    'assert_eq!(core::mem::align_of::<DtcmLayout>(), 4);',
    'assert_eq!(core::mem::size_of::<SharedDtcmState>(), DTCM_STATE_SIZE);',
    'assert_eq!(core::mem::align_of::<SharedDtcmState>(), 4);',
)


def code_only(source: str, hash_comments: bool = False, single_quote_strings: bool = False) -> str:
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
    return code if marker is None else code[:marker.start()] + re.sub(r"[^\n]", " ", code[marker.start():])


def normalized(source: str) -> str:
    return " ".join(source.split())


def named_function(source: str, name: str) -> str | None:
    start = re.search(rf"\bfn\s+{re.escape(name)}\s*\([^)]*\)\s*\{{", source)
    if start is None:
        return None
    depth = 1
    index = start.end()
    while index < len(source) and depth:
        depth += (source[index] == "{") - (source[index] == "}")
        index += 1
    return None if depth else source[start.start():index]


def missing_inventory(scope: str, inventory: tuple[str, ...]) -> list[str]:
    compact = normalized(scope)
    return [item for item in inventory if normalized(item) not in compact]


def exact_inventory_present(scope: str, inventory: tuple[str, ...]) -> bool:
    return normalized(" ".join(inventory)) in normalized(scope)


def swapped_once(source: str, left: str, right: str) -> str:
    sentinel = "__REGISTER_WRITE_LISTS_SWAP__"
    return source.replace(left, sentinel, 1).replace(right, left, 1).replace(sentinel, right, 1)


def check_exact_inventory_regression(source: str) -> None:
    compile_scope = mask_test_module(source)
    test_scope = named_function(source, "tlv_dispatch_handler_table_is_exact")
    if test_scope is None:
        raise SystemExit("checker self-test failed: focused test extraction failed")
    for label, scope, inventory in (
        ("compile-time", compile_scope, COMPILE_TIME_PHYSICAL_INVENTORY),
        ("focused-test", test_scope, FOCUSED_TEST_INVENTORY),
    ):
        if not exact_inventory_present(scope, inventory):
            raise SystemExit(f"checker self-test failed: canonical {label} inventory is absent")
        canonical = normalized(" ".join(inventory))
        for item in inventory:
            mutated = canonical.replace(normalized(item), "", 1)
            if exact_inventory_present(mutated, inventory):
                raise SystemExit(f"checker self-test failed: deleted {label} mapping was accepted: {item}")

    compile_item = normalized(COMPILE_TIME_PHYSICAL_INVENTORY[5])
    compile_swap = normalized(compile_scope).replace(
        compile_item, compile_item.replace("== 0x24", "== 0x22"), 1
    )
    test_address_item = normalized(FOCUSED_TEST_INVENTORY[7])
    test_address_swap = normalized(test_scope).replace(
        test_address_item,
        swapped_once(test_address_item, "0x0400_0da8", "0x0400_0da6"),
        1,
    )
    test_extent_item = normalized(FOCUSED_TEST_INVENTORY[9])
    test_extent_swap = normalized(test_scope).replace(
        test_extent_item,
        swapped_once(test_extent_item, "0x0400_0dd0", "0x0400_0dce"),
        1,
    )


def source_paths() -> list[Path]:
    return sorted(path for path in ROOT.rglob("*") if path.is_file()
                  and (path.suffix in SOURCE_EXTENSIONS or path.name in SOURCE_FILENAMES)
                  and "target" not in path.parts and ".git" not in path.parts)


def in_range(value: int) -> bool:
    return RANGE[0] <= value < RANGE[1]


def owned_literals(code: str) -> list[int]:
    values = [int(match.group().replace("_", ""), 16) for match in LITERAL.finditer(code)]
    physical = [value for value in values if in_range(value)]
    relative_context = re.search(
        r"\bDTCM_STATE_BASE\b|"
        r"\b(?:[A-Za-z_][A-Za-z0-9_]*::)*from_offset(?:_unchecked)?\s*\(", code,
    )
    relative = [value for value in values if value in OFFSETS] if relative_context else []
    return physical + relative


def rust_use_aliases(code: str) -> list[tuple[str, str]]:
    aliases: list[tuple[str, str]] = []
    for statement in re.finditer(r"\buse\s+([^;]*);", code):
        aliases.extend((local, imported) for imported, local in re.findall(
            r"\b([A-Za-z_][A-Za-z0-9_]*)\s+as\s+([A-Za-z_][A-Za-z0-9_]*)\b",
            statement.group(1),
        ))
    return aliases


def family_aliases(code: str) -> tuple[set[str], list[tuple[str, str, str]]]:
    declarations = [
        (kind, name, initializer)
        for kind, pattern in (
            ("const", r"\bconst\s+([A-Za-z_][A-Za-z0-9_]*)\s*:[^=;]+\s*=\s*([^;]*);"),
            ("static", r"\bstatic\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:[^=;]+\s*=\s*([^;]*);"),
            ("type", r"\btype\s+([A-Za-z_][A-Za-z0-9_]*)\s*=\s*([^;]*);"),
        )
        for name, initializer in re.findall(pattern, code)
    ]
    imported = rust_use_aliases(code)
    views = {"MacPipeTail", "TlvDispatchHandlerEntry", "TlvDispatchHandlerTable", "tlv_dispatch_handler_table",
             *(name for _, name, initializer in declarations if owned_literals(initializer))}
    while True:
        aliases = {name for _, name, initializer in declarations
                   if set(re.findall(r"\b[A-Za-z_][A-Za-z0-9_]*\b", initializer)) & views}
        aliases |= {local for local, source in imported if source in views}
        expanded = views | aliases
        if expanded == views: return views, declarations
        views = expanded


def functions_consuming_family(code: str, views: set[str]) -> list[str]:
    pattern = re.compile(rf"\b(?:{'|'.join(sorted(map(re.escape, views)))})\b")
    consumers: list[str] = []
    for start in re.finditer(r"\b(?:unsafe\s+)?(?:const\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)[^\{;]*\{", code):
        depth = 1; index = start.end()
        while index < len(code) and depth:
            depth += (code[index] == "{") - (code[index] == "}"); index += 1
        body = code[start.start():index]
        if pattern.search(body) or owned_literals(body): consumers.append(start.group(1))
    return consumers


def check_alias_tracking_regression() -> None:
    fixtures = (
        ("type A = TlvDispatchHandlerTable; type B = A; fn leak(_: B) {}", {"A", "B"}, ["leak"]),
        ("static ROOT: usize = 0x0400_0dac; static NEXT: usize = ROOT; fn leak() -> usize { NEXT }", {"ROOT", "NEXT"}, ["leak"]),
        ("type Hidden = TlvDispatchHandlerTable; type Again = Hidden; fn leak(_: Again) {}", {"Hidden", "Again"}, ["leak"]),
        ("use crate::dtcm::TlvDispatchHandlerTable as Hidden; type Again = Hidden; fn leak(_: Again) {}", {"Hidden", "Again"}, ["leak"]),
        ("use crate::dtcm::{TlvDispatchHandlerTable as H1}; type H2 = H1; const N: usize = core::mem::size_of::<H2>(); fn leak(_: H2) -> usize { N }", {"H1", "H2", "N"}, ["leak"]),
        ("use crate::{dtcm::{TlvDispatchHandlerTable as Hidden}}; fn leak(_: Hidden) {}", {"Hidden"}, ["leak"]),
        ("fn leak() -> *mut u32 { (DTCM_STATE_BASE + 0xdb4) as *mut u32 }", set(), ["leak"]),
        ("const ENTRY: usize = DTCM_STATE_BASE + 0xdb4; const NEXT: usize = ENTRY; fn leak() -> *mut u32 { NEXT as *mut u32 }", {"ENTRY", "NEXT"}, ["leak"]),
        ("fn leak() -> *mut u32 { DtcmAddress::from_offset(0xdb4).cast_mut() }", set(), ["leak"]),
        ("const ENTRY: DtcmAddress = DtcmAddress::from_offset(0xdb4); const NEXT: DtcmAddress = ENTRY; fn leak() -> usize { NEXT.get() }", {"ENTRY", "NEXT"}, ["leak"]),
        ("fn leak() -> usize { DtcmAddress::from_offset_unchecked(0xdb4).get() }", set(), ["leak"]),
        ("use crate::{dtcm::{DtcmAddress as HiddenAddress}}; const ENTRY: HiddenAddress = HiddenAddress::from_offset(0xdb4); const NEXT: HiddenAddress = ENTRY; fn leak() -> usize { NEXT.get() }", {"ENTRY", "NEXT"}, ["leak"]),
        ("fn reset() { unsafe { core::ptr::write_volatile(0x0400_0dac as *mut u32, 0) } }", set(), ["reset"]),
    )
    for fixture, expected_aliases, expected_functions in fixtures:
        views, _ = family_aliases(fixture)
        if not expected_aliases <= views or functions_consuming_family(fixture, views) != expected_functions:
            raise SystemExit("checker self-test failed: alias, constructor-offset, or raw-pointer family API was accepted")


def check_source() -> None:
    source = (ROOT / "src/dtcm.rs").read_text()
    compact = normalized(source)
    failures = [f"src/dtcm.rs: missing exact inventory: {item}" for item in REQUIRED if normalized(item) not in compact]
    compile_scope = mask_test_module(source)
    focused_test = named_function(source, "tlv_dispatch_handler_table_is_exact")
    failures.extend(
        f"src/dtcm.rs: missing exact compile-time physical mapping: {item}"
        for item in missing_inventory(compile_scope, COMPILE_TIME_PHYSICAL_INVENTORY)
    )
    if not exact_inventory_present(compile_scope, COMPILE_TIME_PHYSICAL_INVENTORY):
        failures.append("src/dtcm.rs: exact contiguous compile-time scheduler tail assertion inventory changed")
    if focused_test is None:
        failures.append("src/dtcm.rs: focused scheduler tails layout test is missing")
    else:
        failures.extend(
            f"src/dtcm.rs: focused test missing exact mapping/extent: {item}"
            for item in missing_inventory(focused_test, FOCUSED_TEST_INVENTORY)
        )
        if not exact_inventory_present(focused_test, FOCUSED_TEST_INVENTORY):
            failures.append("src/dtcm.rs: exact contiguous focused scheduler tail test inventory changed")
    check_exact_inventory_regression(source)
    for physical in PHYSICAL:
        spelling = f"0x{physical >> 16:04x}_{physical & 0xffff:04x}"
        if spelling not in source:
            failures.append(f"src/dtcm.rs: missing physical address/boundary {spelling}")
    entry_declaration = re.search(r"#\[repr\(C, align\(4\)\)\] struct TlvDispatchHandlerEntry \{[^}]*\}", source)
    table_declaration = re.search(r"#\[repr\(C, align\(4\)\)\] struct TlvDispatchHandlerTable \{[^}]*\}", source)
    expected_declarations = (
        "#[repr(C, align(4))] struct TlvDispatchHandlerEntry { key: SharedU32, handler: SharedU32 }",
        "#[repr(C, align(4))] struct TlvDispatchHandlerTable { entries: [TlvDispatchHandlerEntry; 4], terminator_key: SharedU32, terminator_handler: SharedU32 }",
    )
    if (entry_declaration is None or table_declaration is None
            or normalized(entry_declaration.group()) != normalized(expected_declarations[0])
            or normalized(table_declaration.group()) != normalized(expected_declarations[1])):
        failures.append("src/dtcm.rs: TlvDispatchHandlerTable must be the exact private non-derived inventory")
    if (source.count("struct TlvDispatchHandlerEntry") != 1
            or source.count("struct TlvDispatchHandlerTable") != 1
            or "post_initialized_phy_gain_source_records" in source):
        failures.append("src/dtcm.rs: removed opaque split or duplicate tlv handler tables remain")
    for relative, exact in ADJACENT_DECLARATIONS.items():
        if exact not in (ROOT / relative).read_text():
            failures.append(f"{relative}: adjacent checker range changed from {exact}")

    paths = source_paths()
    rust = {path.relative_to(ROOT).as_posix(): mask_test_module(code_only(path.read_text(errors="replace")))
            for path in paths if path.suffix == ".rs"}
    rust["src/dtcm.rs"] = rust["src/dtcm.rs"].replace(STRUCT, " " * len(STRUCT))
    production = "\n".join(rust.values())
    views, declarations = family_aliases(production)
    sanctioned_roots = set()
    for kind, name, _ in declarations:
        if name in views and name not in sanctioned_roots:
            failures.append(f"additional scheduler tail {kind} alias is forbidden: {name}")
    view_pattern = re.compile(rf"\b(?:{'|'.join(sorted(map(re.escape, views)))})\b")
    if re.search(r"\bimpl(?:\s*<[^>]*>)?\s+[^\{]*MacPipeTail", production):
        failures.append("production impl for RfScaleHalfwordTable is forbidden")
    for match in re.finditer(r"\b(?:const|static)\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)[^;]*;", production):
        if view_pattern.search(match.group()) and "RF_MODE_HALFWORD_TABLE:" not in normalized(match.group()):
            failures.append(f"direct scheduler tail const/static alias is forbidden: {match.group(1)}")
    for match in re.finditer(r"\b(?:unsafe\s+)?(?:const\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*register_write_lists_entry_marker[A-Za-z0-9_]*)", production, re.I):
        failures.append(f"named scheduler tail production API is forbidden: {match.group(1)}")
    forbidden_operation = re.compile(r"\*(?:const|mut)|&(?:mut\s+)?|\b(?:read|write)(?:_volatile)?\s*\(|\b(?:value|init(?:ialize)?|reset|unchecked|generic_offset|slice|iter(?:ator)?)\b(?!\s*:)", re.I)
    for relative, code in rust.items():
        # start_scheduler_timer (tx.rs) is the retained Rust translation of
        # vendor timer_start; it reads exactly the guard word 0x0400_0dac that
        # timer_start itself reads, via the single pinned consumer line below.
        sanctioned_functions = SANCTIONED_CONSUMER_FUNCTIONS.get(relative, set())
        excess = [f for f in functions_consuming_family(code, views) if f not in sanctioned_functions]
        for function in excess:
            failures.append(f"{relative}: unsanctioned production function over tlv handler table storage: {function}")
        sanctioned_lines = SANCTIONED_CONSUMER_LINES.get(relative, set())
        for line_number, line in enumerate(code.splitlines(), 1):
            if normalized(line) in sanctioned_lines:
                continue
            if view_pattern.search(line) and forbidden_operation.search(line):
                failures.append(f"{relative}:{line_number}: pointer/reference/read/write/value/init/offset/slice API is forbidden")
            for value in owned_literals(line):
                if value in OFFSETS and relative != "src/dtcm.rs":
                    failures.append(f"{relative}:{line_number}: raw scheduler tail DTCM offset 0x{value:x} is forbidden")

    for path in paths:
        relative = path.relative_to(ROOT).as_posix()
        if relative in OWNER_FILES: continue
        code = rust[relative] if relative.endswith(".rs") else code_only(
            path.read_text(errors="replace"), path.suffix in {".py", ".sh", ".toml"}, path.suffix in {".py", ".sh", ".toml"})
        adjacent = ADJACENT_DECLARATIONS.get(relative)
        if adjacent is not None:
            code = code.replace(adjacent, " " * len(adjacent), 1)
        sanctioned_lines = SANCTIONED_CONSUMER_LINES.get(relative, set())
        for match in LITERAL.finditer(code):
            value = int(match.group().replace("_", ""), 16)
            if in_range(value):
                line_number = code.count("\n", 0, match.start()) + 1
                if normalized(code.splitlines()[line_number - 1]) in sanctioned_lines:
                    continue
                failures.append(f"{relative}:{line_number}: scheduler tail physical literal {match.group()} is outside reviewed owners")
    if failures: raise SystemExit("\n".join(failures))
    print(f"TLV DISPATCH HANDLER TABLE SOURCE DRIFT-EVIDENCE GATE PASSED files={len(paths)}")


def linked_literals(path: Path) -> collections.Counter[int]:
    # Deliberately derived from decoded PC-relative literal loads, not from an
    # aligned-word scan of .text (same precedent as check-vif-layout.py and
    # check-initialized-scheduler-tail-layout.py): Thumb instruction pairs can
    # form words that land inside a reviewed interval.
    xrefs = decoded_literal_xrefs(path)
    return collections.Counter(value for _, value in xrefs.elements())


def decoded_literal_xrefs(path: Path) -> collections.Counter[tuple[str, int]]:
    disassembly = subprocess.run(["llvm-objdump", "-d", "--print-imm-hex", str(path)], check=True, text=True, stdout=subprocess.PIPE).stdout
    words = {int(address, 16): int(value, 16) for address, value in re.findall(r"^\s*([0-9a-fA-F]+):.*?\.word\s+0x([0-9a-fA-F]+)\s*$", disassembly, re.MULTILINE)}
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
        raise SystemExit(f"TLV DISPATCH HANDLER TABLE LINKED DRIFT GATE FAILED\nliterals={dict(literals)!r}\nxrefs={dict(xrefs)!r}")
    residual_literals, residual_xrefs = literals - ALLOWED_LINKED_LITERALS, xrefs - ALLOWED_DECODED_XREFS
    print(f"TLV DISPATCH HANDLER TABLE LINKED DRIFT-EVIDENCE GATE PASSED allowed_literals={sum(ALLOWED_LINKED_LITERALS.values())} residual_literals={sum(residual_literals.values())} residual_xrefs={sum(residual_xrefs.values())}")


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
        raise SystemExit(f"initialized-tlv-handler-table drift gate failed: {error}")
