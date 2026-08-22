#!/usr/bin/env python3
"""Source/linked drift evidence for [0x04001410, 0x0400141c).

The linked aligned-literal and decoded PC-relative-xref multisets are pinned
empty. Empty linked sets are drift evidence, not writer closure: computed,
indirect, generic HIF/debug, vendor, IRQ, and FIQ mutation remain possible.
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
RANGE = (0x04001410, 0x0400141C)
OFFSETS = range(0x1410, 0x141C)
LITERAL = re.compile(r"0x[0-9a-fA-F_]+")
SOURCE_EXTENSIONS = {
    ".rs", ".py", ".sh", ".c", ".h", ".hh", ".hpp", ".hxx", ".cc",
    ".cpp", ".cxx", ".s", ".S", ".asm", ".inc", ".ld", ".lds",
    ".ldh", ".x", ".toml", ".mk",
}
SOURCE_FILENAMES = {"Makefile", "Kconfig"}
OWNER_FILES = {
    "src/dtcm.rs",
    "tools/check-initialized-tx-confirm-aggregation-state-layout.py",
}
ADJACENT_DECLARATIONS = {
    "tools/check-ampdu-completion-control-layout.py": "(0x0400140C, 0x04001410)",
    "tools/check-initialized-control-words-layout.py": "(0x04001420, 0x04001440)",
}
ALLOWED_LINKED_LITERALS: collections.Counter[int] = collections.Counter()
ALLOWED_DECODED_XREFS: collections.Counter[tuple[str, int]] = collections.Counter()
FIELDS = ("state", "pending_message_raw", "append_cursor_raw")
STRUCT = (
    "#[repr(C, align(4))] struct TxConfirmAggregationState { "
    "state: SharedU32, pending_message_raw: SharedU32, append_cursor_raw: SharedU32 }"
)
REQUIRED = (
    STRUCT,
    "tx_confirm_aggregation_state: TxConfirmAggregationState",
    "pre_control_words: OpaqueBytes<0x04>",
    "assert_type_layout!(SharedU32, 0x04, 4)",
    "assert_type_layout!(TxConfirmAggregationState, 0x0c, 4)",
    "offset_of!(TxConfirmAggregationState, state) == 0x00",
    "offset_of!(TxConfirmAggregationState, pending_message_raw) == 0x04",
    "offset_of!(TxConfirmAggregationState, append_cursor_raw) == 0x08",
    "offset_of!(InitializedVendorImage, tx_confirm_aggregation_state) == 0x1410",
    "offset_of!(InitializedVendorImage, pre_control_words) == 0x141c",
    "size_of::<OpaqueBytes<0x04>>() == 0x04",
    "offset_of!(InitializedVendorImage, control_words) == 0x1420",
    "assert_type_layout!(InitializedVendorImage, 0x2078, 4)",
    "assert_type_layout!(DtcmLayout, DTCM_STATE_SIZE, 4)",
    "assert_type_layout!(SharedDtcmState, DTCM_STATE_SIZE, 4)",
    "fn initialized_tx_confirm_aggregation_state_addresses_are_exact()",
    "size_of::<TxConfirmAggregationState>(), 0x0c",
    "align_of::<TxConfirmAggregationState>(), 4",
    "fields.windows(2)",
    "AMPDU_COMPLETION_CONTROL.get(), 0x0400_140c",
    "AMPDU_COMPLETION_CONTROL.get() + core::mem::size_of::<AmpduCompletionControl>(), 0x0400_1410",
    "offset_of!(InitializedVendorImage, pre_control_words), 0x0400_141c",
    "size_of::<OpaqueBytes<0x04>>(), 4",
    "offset_of!(InitializedVendorImage, control_words), 0x0400_1420",
    "size_of::<InitializedVendorImage>(), 0x2078",
    "align_of::<InitializedVendorImage>(), 4",
    "size_of::<DtcmLayout>(), DTCM_STATE_SIZE",
    "align_of::<DtcmLayout>(), 4",
    "size_of::<SharedDtcmState>(), DTCM_STATE_SIZE",
    "align_of::<SharedDtcmState>(), 4",
)
PHYSICAL = (0x0400140C, 0x04001410, 0x04001414, 0x04001418, 0x0400141C, 0x04001420)
COMPILE_TIME_PHYSICAL_INVENTORY = (
    "assert_type_layout!(SharedU32, 0x04, 4);",
    "assert_type_layout!(TxConfirmAggregationState, 0x0c, 4);",
    "assert!(core::mem::offset_of!(TxConfirmAggregationState, state) == 0x00);",
    "assert!(core::mem::offset_of!(TxConfirmAggregationState, pending_message_raw) == 0x04);",
    "assert!(core::mem::offset_of!(TxConfirmAggregationState, append_cursor_raw) == 0x08);",
    "assert!(core::mem::offset_of!(InitializedVendorImage, tx_confirm_aggregation_state) == 0x1410);",
    "assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, tx_confirm_aggregation_state) + core::mem::offset_of!(TxConfirmAggregationState, state) == 0x0400_1410);",
    "assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, tx_confirm_aggregation_state) + core::mem::offset_of!(TxConfirmAggregationState, pending_message_raw) == 0x0400_1414);",
    "assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, tx_confirm_aggregation_state) + core::mem::offset_of!(TxConfirmAggregationState, append_cursor_raw) == 0x0400_1418);",
    "assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, tx_confirm_aggregation_state) + core::mem::size_of::<TxConfirmAggregationState>() == 0x0400_141c);",
    "assert!(core::mem::offset_of!(InitializedVendorImage, pre_control_words) == 0x141c);",
    "assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, pre_control_words) == 0x0400_141c);",
    "assert!(core::mem::size_of::<OpaqueBytes<0x04>>() == 0x04);",
    "assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, pre_control_words) + core::mem::size_of::<OpaqueBytes<0x04>>() == 0x0400_1420);",
    "assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, control_words) == 0x0400_1420);",
    "assert_type_layout!(InitializedVendorImage, 0x2078, 4);",
    "assert_type_layout!(DtcmLayout, DTCM_STATE_SIZE, 4);",
    "assert_type_layout!(SharedDtcmState, DTCM_STATE_SIZE, 4);",
)
FOCUSED_TEST_INVENTORY = (
    "let image = DTCM_STATE_BASE;",
    "let record = image + core::mem::offset_of!(InitializedVendorImage, tx_confirm_aggregation_state);",
    "let fields = [record + core::mem::offset_of!(TxConfirmAggregationState, state), record + core::mem::offset_of!(TxConfirmAggregationState, pending_message_raw), record + core::mem::offset_of!(TxConfirmAggregationState, append_cursor_raw)];",
    "assert_eq!(core::mem::size_of::<SharedU32>(), 4);",
    "assert_eq!(core::mem::align_of::<SharedU32>(), 4);",
    "assert_eq!(core::mem::size_of::<TxConfirmAggregationState>(), 0x0c);",
    "assert_eq!(core::mem::align_of::<TxConfirmAggregationState>(), 4);",
    "assert_eq!([core::mem::offset_of!(TxConfirmAggregationState, state), core::mem::offset_of!(TxConfirmAggregationState, pending_message_raw), core::mem::offset_of!(TxConfirmAggregationState, append_cursor_raw)], [0x00, 0x04, 0x08]);",
    "assert_eq!(fields, [0x0400_1410, 0x0400_1414, 0x0400_1418]);",
    "for pair in fields.windows(2) { assert_eq!(pair[0] + core::mem::size_of::<SharedU32>(), pair[1]); }",
    "assert_eq!(record, 0x0400_1410);",
    "assert_eq!(record + core::mem::size_of::<TxConfirmAggregationState>(), 0x0400_141c);",
    "assert_eq!(AMPDU_COMPLETION_CONTROL.get(), 0x0400_140c);",
    "assert_eq!(AMPDU_COMPLETION_CONTROL.get() + core::mem::size_of::<AmpduCompletionControl>(), 0x0400_1410);",
    "assert_eq!(image + core::mem::offset_of!(InitializedVendorImage, pre_control_words), 0x0400_141c);",
    "assert_eq!(core::mem::size_of::<OpaqueBytes<0x04>>(), 4);",
    "assert_eq!(image + core::mem::offset_of!(InitializedVendorImage, pre_control_words) + core::mem::size_of::<OpaqueBytes<0x04>>(), 0x0400_1420);",
    "assert_eq!(image + core::mem::offset_of!(InitializedVendorImage, control_words), 0x0400_1420);",
    "assert_eq!(core::mem::size_of::<InitializedVendorImage>(), 0x2078);",
    "assert_eq!(core::mem::align_of::<InitializedVendorImage>(), 4);",
    "assert_eq!(core::mem::size_of::<DtcmLayout>(), DTCM_STATE_SIZE);",
    "assert_eq!(core::mem::align_of::<DtcmLayout>(), 4);",
    "assert_eq!(core::mem::size_of::<SharedDtcmState>(), DTCM_STATE_SIZE);",
    "assert_eq!(core::mem::align_of::<SharedDtcmState>(), 4);",
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
    sentinel = "__TX_CONFIRM_LAYOUT_SWAP__"
    return source.replace(left, sentinel, 1).replace(right, left, 1).replace(sentinel, right, 1)


def check_exact_inventory_regression(source: str) -> None:
    compile_scope = mask_test_module(source)
    test_scope = named_function(source, "initialized_tx_confirm_aggregation_state_addresses_are_exact")
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

    compile_item = normalized(COMPILE_TIME_PHYSICAL_INVENTORY[6])
    compile_swap = normalized(compile_scope).replace(
        compile_item, compile_item.replace("0x0400_1410", "0x0400_1414"), 1
    )
    test_address_item = normalized(FOCUSED_TEST_INVENTORY[8])
    test_address_swap = normalized(test_scope).replace(
        test_address_item,
        swapped_once(test_address_item, "0x0400_1410", "0x0400_1414"),
        1,
    )
    test_field_item = normalized(FOCUSED_TEST_INVENTORY[2])
    test_field_swap = normalized(test_scope).replace(
        test_field_item,
        swapped_once(
            test_field_item,
            "core::mem::offset_of!(TxConfirmAggregationState, state)",
            "core::mem::offset_of!(TxConfirmAggregationState, pending_message_raw)",
        ),
        1,
    )
    if exact_inventory_present(compile_swap, COMPILE_TIME_PHYSICAL_INVENTORY):
        raise SystemExit("checker self-test failed: swapped compile-time field/address mappings were accepted")
    if exact_inventory_present(test_address_swap, FOCUSED_TEST_INVENTORY):
        raise SystemExit("checker self-test failed: swapped focused-test addresses were accepted")
    if exact_inventory_present(test_field_swap, FOCUSED_TEST_INVENTORY):
        raise SystemExit("checker self-test failed: swapped focused-test field mappings were accepted")


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
    views = {"TxConfirmAggregationState", "tx_confirm_aggregation_state",
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
        ("const ROOT: usize = tx_confirm_aggregation_state; const NEXT: usize = ROOT; fn leak() -> *mut u32 { NEXT as *mut u32 }", {"ROOT", "NEXT"}, ["leak"]),
        ("static ROOT: usize = 0x0400_1410; static NEXT: usize = ROOT; fn leak() -> usize { NEXT }", {"ROOT", "NEXT"}, ["leak"]),
        ("type Hidden = TxConfirmAggregationState; type Again = Hidden; fn leak(_: Again) {}", {"Hidden", "Again"}, ["leak"]),
        ("use crate::dtcm::TxConfirmAggregationState as Hidden; const ROOT: usize = tx_confirm_aggregation_state; fn leak(_: Hidden) -> usize { ROOT }", {"Hidden", "ROOT"}, ["leak"]),
        ("use crate::dtcm::{TxConfirmAggregationState as Hidden, tx_confirm_aggregation_state as Root}; const NEXT: usize = Root; fn leak(_: Hidden) -> usize { NEXT }", {"Hidden", "Root", "NEXT"}, ["leak"]),
        ("use crate::{dtcm::{TxConfirmAggregationState as Hidden}}; fn leak(_: Hidden) {}", {"Hidden"}, ["leak"]),
        ("fn leak() -> *mut u32 { (DTCM_STATE_BASE + 0x1414) as *mut u32 }", set(), ["leak"]),
        ("const ENTRY: usize = DTCM_STATE_BASE + 0x1414; const NEXT: usize = ENTRY; fn leak() -> *mut u32 { NEXT as *mut u32 }", {"ENTRY", "NEXT"}, ["leak"]),
        ("fn leak() -> *mut u32 { DtcmAddress::from_offset(0x1418).cast_mut() }", set(), ["leak"]),
        ("const ENTRY: DtcmAddress = DtcmAddress::from_offset(0x1418); const NEXT: DtcmAddress = ENTRY; fn leak() -> usize { NEXT.get() }", {"ENTRY", "NEXT"}, ["leak"]),
        ("fn leak() -> usize { DtcmAddress::from_offset_unchecked(0x1418).get() }", set(), ["leak"]),
        ("use crate::{dtcm::{DtcmAddress as HiddenAddress}}; const ENTRY: HiddenAddress = HiddenAddress::from_offset(0x1414); const NEXT: HiddenAddress = ENTRY; fn leak() -> usize { NEXT.get() }", {"ENTRY", "NEXT"}, ["leak"]),
        ("fn reset() { unsafe { core::ptr::write_volatile(0x0400_1410 as *mut u32, 0) } }", set(), ["reset"]),
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
    focused_test = named_function(source, "initialized_tx_confirm_aggregation_state_addresses_are_exact")
    failures.extend(
        f"src/dtcm.rs: missing exact compile-time physical mapping: {item}"
        for item in missing_inventory(compile_scope, COMPILE_TIME_PHYSICAL_INVENTORY)
    )
    if not exact_inventory_present(compile_scope, COMPILE_TIME_PHYSICAL_INVENTORY):
        failures.append("src/dtcm.rs: exact contiguous compile-time TX-confirm layout assertion inventory changed")
    if focused_test is None:
        failures.append("src/dtcm.rs: focused TX-confirm aggregation layout test is missing")
    else:
        failures.extend(
            f"src/dtcm.rs: focused test missing exact mapping/extent: {item}"
            for item in missing_inventory(focused_test, FOCUSED_TEST_INVENTORY)
        )
        if not exact_inventory_present(focused_test, FOCUSED_TEST_INVENTORY):
            failures.append("src/dtcm.rs: exact contiguous focused TX-confirm layout test inventory changed")
    check_exact_inventory_regression(source)
    for physical in PHYSICAL:
        spelling = f"0x{physical >> 16:04x}_{physical & 0xffff:04x}"
        if spelling not in source:
            failures.append(f"src/dtcm.rs: missing physical address/boundary {spelling}")
    declaration = re.search(r"(?:#\[[^\n]*\]\s*)*#\[repr\(C, align\(4\)\)\]\s*struct\s+TxConfirmAggregationState\s*\{[^}]*\}", source)
    if declaration is None or normalized(declaration.group()) != normalized(STRUCT) or "derive" in declaration.group():
        failures.append("src/dtcm.rs: TxConfirmAggregationState must be the exact private non-derived inventory")
    if source.count("struct TxConfirmAggregationState") != 1 or "pre_control_words: OpaqueBytes<0x10>" in source:
        failures.append("src/dtcm.rs: old opaque extent or duplicate TX-confirm state remains")
    for relative, exact in ADJACENT_DECLARATIONS.items():
        if exact not in (ROOT / relative).read_text():
            failures.append(f"{relative}: adjacent checker range changed from {exact}")

    paths = source_paths()
    rust = {path.relative_to(ROOT).as_posix(): mask_test_module(code_only(path.read_text(errors="replace")))
            for path in paths if path.suffix == ".rs"}
    rust["src/dtcm.rs"] = rust["src/dtcm.rs"].replace(STRUCT, " " * len(STRUCT))
    production = "\n".join(rust.values())
    views, declarations = family_aliases(production)
    for kind, name, _ in declarations:
        if name in views:
            failures.append(f"additional TX-confirm aggregation {kind} alias is forbidden: {name}")
    view_pattern = re.compile(rf"\b(?:{'|'.join(sorted(map(re.escape, views)))})\b")
    if re.search(r"\bimpl(?:\s*<[^>]*>)?\s+[^\{]*TxConfirmAggregationState", production):
        failures.append("production impl for TxConfirmAggregationState is forbidden")
    for match in re.finditer(r"\b(?:const|static)\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)[^;]*;", production):
        if view_pattern.search(match.group()):
            failures.append(f"direct TX-confirm aggregation const/static alias is forbidden: {match.group(1)}")
    for match in re.finditer(r"\b(?:unsafe\s+)?(?:const\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*aggregation_state[A-Za-z0-9_]*)", production, re.I):
        failures.append(f"named TX-confirm aggregation production API is forbidden: {match.group(1)}")
    forbidden_operation = re.compile(r"\*(?:const|mut)|&(?:mut\s+)?|\b(?:read|write)(?:_volatile)?\s*\(|\b(?:value|init(?:ialize)?|reset|unchecked|generic_offset|slice|iter(?:ator)?)\b", re.I)
    for relative, code in rust.items():
        for function in functions_consuming_family(code, views):
            failures.append(f"{relative}: production function API over TX-confirm aggregation storage is forbidden: {function}")
        for line_number, line in enumerate(code.splitlines(), 1):
            if view_pattern.search(line) and forbidden_operation.search(line):
                failures.append(f"{relative}:{line_number}: pointer/reference/read/write/value/init/offset/slice API is forbidden")
            for value in owned_literals(line):
                if value in OFFSETS and relative != "src/dtcm.rs":
                    failures.append(f"{relative}:{line_number}: raw TX-confirm DTCM offset 0x{value:x} is forbidden")

    for path in paths:
        relative = path.relative_to(ROOT).as_posix()
        if relative in OWNER_FILES: continue
        code = rust[relative] if relative.endswith(".rs") else code_only(
            path.read_text(errors="replace"), path.suffix in {".py", ".sh", ".toml"}, path.suffix in {".py", ".sh", ".toml"})
        adjacent = ADJACENT_DECLARATIONS.get(relative)
        if adjacent is not None:
            code = code.replace(adjacent, " " * len(adjacent), 1)
        for match in LITERAL.finditer(code):
            value = int(match.group().replace("_", ""), 16)
            if in_range(value):
                line = code.count("\n", 0, match.start()) + 1
                failures.append(f"{relative}:{line}: TX-confirm aggregation physical literal {match.group()} is outside reviewed owners")
    if failures: raise SystemExit("\n".join(failures))
    print(f"INITIALIZED TX-CONFIRM AGGREGATION STATE SOURCE DRIFT-EVIDENCE GATE PASSED files={len(paths)}")


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
        raise SystemExit(f"INITIALIZED TX-CONFIRM AGGREGATION STATE LINKED DRIFT GATE FAILED\nliterals={dict(literals)!r}\nxrefs={dict(xrefs)!r}")
    print("INITIALIZED TX-CONFIRM AGGREGATION STATE LINKED DRIFT-EVIDENCE GATE PASSED literals=0 decoded_xrefs=0")


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
        raise SystemExit(f"initialized-TX-confirm-aggregation-state drift gate failed: {error}")
