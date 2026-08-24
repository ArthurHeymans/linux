#!/usr/bin/env python3
"""Source/linked drift evidence for exactly [0x04000F88, 0x04001088).

Two adjacent 64-entry i16 RF scale tables read by vendor rf_dft_correlate_
samples during measurement DFT correlation:

- rf_scale_table_a [0x04000f88, 0x04001008): rooted at DAT_00019614, read as
  (value * sample * 2^-(signed)) >> 5 by rf_scale_by_tbl_a.
- rf_scale_table_b [0x04001008, 0x04001088): same shape at DAT_00019614 +
  0x80, read by rf_scale_by_tbl_b.

The sole caller masks every index with & 0x3f after each increment, proving
the 64-entry extent of both tables; the interval below (pre_rf_scale_tables)
and above (post_rf_scale_tables, next direct root rf_init_stage_a DATA at
0x040010a8 region / phy_select_rate_tables pointer targets at 0x04001088)
bounds them exactly. Vendor rf_init_stage_a stores the field-derived address of
table-A entry 60 into RF SRAM state in the non-primary mode branch. That opaque
pointer consumer changes no byte typing here.

The initial COPY values are loader-owned; no writer closure is claimed:
computed, indirect, generic HIF/debug, vendor, IRQ, and FIQ mutation remain
possible. The linked-literal and decoded PC-relative-xref multisets are
derived from decoded loads only and pinned empty.
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
RANGE = (0x04000F88, 0x04001088)
OFFSETS = range(0xF88, 0x1088)
LITERAL = re.compile(r"0x[0-9a-fA-F_]+")
SOURCE_EXTENSIONS = {
    ".rs", ".py", ".sh", ".c", ".h", ".hh", ".hpp", ".hxx", ".cc",
    ".cpp", ".cxx", ".s", ".S", ".asm", ".inc", ".ld", ".lds",
    ".ldh", ".x", ".toml", ".mk",
}
SOURCE_FILENAMES = {"Makefile", "Kconfig"}
OWNER_FILES = {
    "src/dtcm.rs",
    "tools/check-initialized-rf-scale-tables-layout.py",
}
ADJACENT_DECLARATIONS = {
    "tools/check-initialized-register-write-lists-layout.py": "(0x04000E48, 0x04000EC0)",
}
SANCTIONED_CONSUMER_LINES: dict[str, set[str]] = {}
SANCTIONED_CONSUMER_FUNCTIONS: dict[str, set[str]] = {
    "src/phy.rs": {"initialize_mac_software_state", "rf_init_stage_a_mode0"},
}
# No linked literals fall inside this interval: the guard word is reached
# relative to the scheduler timer list head root (owned by the adjacent
# checker), so both multisets are pinned empty.
ALLOWED_LINKED_LITERALS: collections.Counter[int] = collections.Counter({0x04001000: 1})
ALLOWED_DECODED_XREFS: collections.Counter[tuple[str, int]] = collections.Counter({
    ("_RNvNtCsiHlLB2CErfM_14xr819_firmware3phy22prepare_rf_mode0_stage", 0x04001000): 1,
})
STRUCT = '#[repr(C, align(2))] struct RfScaleHalfwordTable { entries: [SharedU16; 64] }'
REQUIRED = (
    STRUCT,
    "rf_scale_table_a: RfScaleHalfwordTable",
    "rf_scale_table_b: RfScaleHalfwordTable",
    "pre_rf_scale_tables: OpaqueBytes<0xc8>",
    "rate_pointer_targets: OpaqueBytes<0x20>",
    "pub(crate) const RF_SCALE_TABLE_A_MODE0_TARGET: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, rf_scale_table_a) + 60 * core::mem::size_of::<SharedU16>());",
    "pub(crate) const PHY_RATE_POINTER_TARGET_A: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, rate_pointer_targets));",
    "pub(crate) const PHY_RATE_POINTER_TARGET_B: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, rate_pointer_targets) + core::mem::size_of::<OpaqueBytes<0x20>>() / 2);",
    "tx_gain_rssi_halfword_table: TxGainRssiHalfwordTable",
    "assert_type_layout!(SharedU16, 0x02, 2)",
    "assert_type_layout!(RfScaleHalfwordTable, 0x80, 2)",
    "offset_of!(RfScaleHalfwordTable, entries) == 0",
    "offset_of!(InitializedVendorImage, pre_rf_scale_tables) == 0x0ec0",
    "size_of::<OpaqueBytes<0xc8>>() == 0xc8",
    "offset_of!(InitializedVendorImage, rf_scale_table_a) == 0x0f88",
    "offset_of!(InitializedVendorImage, rf_scale_table_b) == 0x1008",
    "offset_of!(InitializedVendorImage, rate_pointer_targets) == 0x1088",
    "size_of::<OpaqueBytes<0x20>>() == 0x20",
    "offset_of!(InitializedVendorImage, tx_gain_rssi_halfword_table) == 0x10a8",
    "assert_type_layout!(InitializedVendorImage, 0x2078, 4)",
    "assert_type_layout!(DtcmLayout, DTCM_STATE_SIZE, 4)",
    "assert_type_layout!(SharedDtcmState, DTCM_STATE_SIZE, 4)",
    "fn register_write_lists_after_iq_gain_indices_are_exact()",
    "size_of::<RfScaleHalfwordTable>(), 0x80",
    "align_of::<RfScaleHalfwordTable>(), 2",
    "rf_scale_table_a), 0x0400_0f88",
    "size_of::<RfScaleHalfwordTable>(), 0x0400_1008",
    "rf_scale_table_b), 0x0400_1008",
    "size_of::<RfScaleHalfwordTable>(), 0x0400_1088",
    "size_of::<InitializedVendorImage>(), 0x2078",
    "align_of::<InitializedVendorImage>(), 4",
    "size_of::<DtcmLayout>(), DTCM_STATE_SIZE",
    "align_of::<DtcmLayout>(), 4",
    "size_of::<SharedDtcmState>(), DTCM_STATE_SIZE",
    "align_of::<SharedDtcmState>(), 4",
)
PHYSICAL = (0x04000F88, 0x04001008, 0x04001088)
COMPILE_TIME_PHYSICAL_INVENTORY = (
    "assert!(core::mem::offset_of!(InitializedVendorImage, pre_rf_scale_tables) == 0x0ec0);",
    "assert!(core::mem::size_of::<OpaqueBytes<0xc8>>() == 0xc8);",
    "assert_type_layout!(RfScaleHalfwordTable, 0x80, 2);",
    "assert!(core::mem::offset_of!(RfScaleHalfwordTable, entries) == 0);",
    "assert!(core::mem::offset_of!(InitializedVendorImage, rf_scale_table_a) == 0x0f88);",
    "assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, rf_scale_table_a) + core::mem::size_of::<RfScaleHalfwordTable>() == 0x0400_1008);",
    "assert!(core::mem::offset_of!(InitializedVendorImage, rf_scale_table_b) == 0x1008);",
    "assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, rf_scale_table_b) + core::mem::size_of::<RfScaleHalfwordTable>() == 0x0400_1088);",
    "assert!(core::mem::offset_of!(InitializedVendorImage, rate_pointer_targets) == 0x1088);",
    "assert!(core::mem::size_of::<OpaqueBytes<0x20>>() == 0x20);",
)
FOCUSED_TEST_INVENTORY = (
    'let image = DTCM_STATE_BASE;',
    'let cal = image + core::mem::offset_of!(InitializedVendorImage, phy_cal_substate_register_write_list);',
    'let dbg = image + core::mem::offset_of!(InitializedVendorImage, dbg_expand_register_write_list);',
    'assert_eq!(core::mem::size_of::<PhyCalSubstateRegisterWriteList>(), 0x48);',
    'assert_eq!(core::mem::align_of::<PhyCalSubstateRegisterWriteList>(), 4);',
    'assert_eq!(core::mem::offset_of!(PhyCalSubstateRegisterWriteList, writes), 0);',
    'assert_eq!(core::mem::size_of::<[RegisterWrite; 8]>(), 0x40);',
    'assert_eq!(core::mem::offset_of!(PhyCalSubstateRegisterWriteList, terminator_address), 0x40);',
    'assert_eq!(cal, 0x0400_0e48);',
    'assert_eq!(cal + core::mem::size_of::<PhyCalSubstateRegisterWriteList>(), 0x0400_0e90);',
    'assert_eq!(core::mem::size_of::<DbgExpandRegisterWriteList>(), 0x30);',
    'assert_eq!(core::mem::align_of::<DbgExpandRegisterWriteList>(), 4);',
    'assert_eq!(core::mem::offset_of!(DbgExpandRegisterWriteList, writes), 0);',
    'assert_eq!(core::mem::size_of::<[RegisterWrite; 5]>(), 0x28);',
    'assert_eq!(core::mem::offset_of!(DbgExpandRegisterWriteList, terminator_address), 0x28);',
    'assert_eq!(dbg, 0x0400_0e90);',
    'assert_eq!(dbg + core::mem::size_of::<DbgExpandRegisterWriteList>(), 0x0400_0ec0);',
    'assert_eq!(image + core::mem::offset_of!(InitializedVendorImage, pre_rf_scale_tables), 0x0400_0ec0);',
    'assert_eq!(core::mem::size_of::<OpaqueBytes<0xc8>>(), 0xc8);',
    'assert_eq!(core::mem::size_of::<RfScaleHalfwordTable>(), 0x80);',
    'assert_eq!(core::mem::align_of::<RfScaleHalfwordTable>(), 2);',
    'assert_eq!(image + core::mem::offset_of!(InitializedVendorImage, rf_scale_table_a), 0x0400_0f88);',
    'assert_eq!(image + core::mem::offset_of!(InitializedVendorImage, rf_scale_table_a) + core::mem::size_of::<RfScaleHalfwordTable>(), 0x0400_1008);',
    'assert_eq!(RF_SCALE_TABLE_A_MODE0_TARGET.get(), 0x0400_1000);',
    'assert_eq!(image + core::mem::offset_of!(InitializedVendorImage, rf_scale_table_b), 0x0400_1008);',
    'assert_eq!(image + core::mem::offset_of!(InitializedVendorImage, rf_scale_table_b) + core::mem::size_of::<RfScaleHalfwordTable>(), 0x0400_1088);',
    'assert_eq!(image + core::mem::offset_of!(InitializedVendorImage, rate_pointer_targets), 0x0400_1088);',
    'assert_eq!(PHY_RATE_POINTER_TARGET_A.get(), 0x0400_1088);',
    'assert_eq!(PHY_RATE_POINTER_TARGET_B.get(), 0x0400_1098);',
    'assert_eq!(core::mem::size_of::<OpaqueBytes<0x20>>(), 0x20);',
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
    test_scope = named_function(source, "register_write_lists_after_iq_gain_indices_are_exact")
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
        compile_item, compile_item.replace("== 0x0400_1008", "== 0x0400_1006"), 1
    )
    test_address_item = normalized(FOCUSED_TEST_INVENTORY[21])
    test_address_swap = normalized(test_scope).replace(
        test_address_item,
        swapped_once(test_address_item, "0x0400_0f88", "0x0400_0f86"),
        1,
    )
    test_extent_item = normalized(FOCUSED_TEST_INVENTORY[25])
    test_extent_swap = normalized(test_scope).replace(
        test_extent_item,
        swapped_once(test_extent_item, "0x0400_1088", "0x0400_1086"),
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
    views = {"MacPipeTail", "RfScaleHalfwordTable", "rf_scale_table_a", "rf_scale_table_b",
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
        ("const ROOT: usize = rf_scale_table_a; const NEXT: usize = ROOT; fn leak() -> *mut u32 { NEXT as *mut u32 }", {"ROOT", "NEXT"}, ["leak"]),
        ("static ROOT: usize = 0x0400_1008; static NEXT: usize = ROOT; fn leak() -> usize { NEXT }", {"ROOT", "NEXT"}, ["leak"]),
        ("type Hidden = RfScaleHalfwordTable; type Again = Hidden; fn leak(_: Again) {}", {"Hidden", "Again"}, ["leak"]),
        ("use crate::dtcm::RfScaleHalfwordTable as Hidden; const ROOT: usize = rf_scale_table_a; fn leak(_: Hidden) -> usize { ROOT }", {"Hidden", "ROOT"}, ["leak"]),
        ("use crate::dtcm::{RfScaleHalfwordTable as Hidden, rf_scale_table_a as Root}; const NEXT: usize = Root; fn leak(_: Hidden) -> usize { NEXT }", {"Hidden", "Root", "NEXT"}, ["leak"]),
        ("use crate::{dtcm::{RfScaleHalfwordTable as Hidden}}; fn leak(_: Hidden) {}", {"Hidden"}, ["leak"]),
        ("fn leak() -> *mut u32 { (DTCM_STATE_BASE + 0xfa8) as *mut u32 }", set(), ["leak"]),
        ("const ENTRY: usize = DTCM_STATE_BASE + 0xfa8; const NEXT: usize = ENTRY; fn leak() -> *mut u32 { NEXT as *mut u32 }", {"ENTRY", "NEXT"}, ["leak"]),
        ("fn leak() -> *mut u32 { DtcmAddress::from_offset(0xfa8).cast_mut() }", set(), ["leak"]),
        ("const ENTRY: DtcmAddress = DtcmAddress::from_offset(0xfa8); const NEXT: DtcmAddress = ENTRY; fn leak() -> usize { NEXT.get() }", {"ENTRY", "NEXT"}, ["leak"]),
        ("fn leak() -> usize { DtcmAddress::from_offset_unchecked(0xfa8).get() }", set(), ["leak"]),
        ("use crate::{dtcm::{DtcmAddress as HiddenAddress}}; const ENTRY: HiddenAddress = HiddenAddress::from_offset(0xfa8); const NEXT: HiddenAddress = ENTRY; fn leak() -> usize { NEXT.get() }", {"ENTRY", "NEXT"}, ["leak"]),
        ("fn reset() { unsafe { core::ptr::write_volatile(0x0400_1008 as *mut u32, 0) } }", set(), ["reset"]),
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
    focused_test = named_function(source, "register_write_lists_after_iq_gain_indices_are_exact")
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
    declaration = re.search(r"#\[repr\(C, align\(2\)\)\] struct RfScaleHalfwordTable \{[^}]*\}", source)
    if declaration is None or normalized(declaration.group()) != normalized(STRUCT) or "derive" in declaration.group():
        failures.append("src/dtcm.rs: RfScaleHalfwordTable must be the exact private non-derived inventory")
    if (source.count("struct RfScaleHalfwordTable") != 1
            or "register_write_lists_suffix" in source):
        failures.append("src/dtcm.rs: removed opaque split or duplicate rf scale tables remain")
    for relative, exact in ADJACENT_DECLARATIONS.items():
        if exact not in (ROOT / relative).read_text():
            failures.append(f"{relative}: adjacent checker range changed from {exact}")

    paths = source_paths()
    rust = {path.relative_to(ROOT).as_posix(): mask_test_module(code_only(path.read_text(errors="replace")))
            for path in paths if path.suffix == ".rs"}
    rust["src/dtcm.rs"] = rust["src/dtcm.rs"].replace(STRUCT, " " * len(STRUCT))
    production = "\n".join(rust.values())
    views, declarations = family_aliases(production)
    sanctioned_roots = {
        "PHY_RATE_POINTER_TARGET_A",
        "PHY_RATE_POINTER_TARGET_B",
        "RF_SCALE_TABLE_A_MODE0_TARGET",
    }
    for kind, name, _ in declarations:
        if name in views and name not in sanctioned_roots:
            failures.append(f"additional scheduler tail {kind} alias is forbidden: {name}")
    view_pattern = re.compile(rf"\b(?:{'|'.join(sorted(map(re.escape, views)))})\b")
    if re.search(r"\bimpl(?:\s*<[^>]*>)?\s+[^\{]*MacPipeTail", production):
        failures.append("production impl for RfScaleHalfwordTable is forbidden")
    for match in re.finditer(r"\b(?:const|static)\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)[^;]*;", production):
        if (view_pattern.search(match.group())
                and match.group(1) not in sanctioned_roots
                and "RF_MODE_HALFWORD_TABLE:" not in normalized(match.group())):
            failures.append(f"direct scheduler tail const/static alias is forbidden: {match.group(1)}")
    for match in re.finditer(r"\b(?:unsafe\s+)?(?:const\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*register_write_lists_entry_marker[A-Za-z0-9_]*)", production, re.I):
        failures.append(f"named scheduler tail production API is forbidden: {match.group(1)}")
    forbidden_operation = re.compile(r"\*(?:const|mut)|&(?:mut\s+)?|\b(?:read|write)(?:_volatile)?\s*\(|\b(?:value|init(?:ialize)?|reset|unchecked|generic_offset|slice|iter(?:ator)?)\b(?!\s*:)", re.I)
    for relative, code in rust.items():
        # The retained PHY initialization paths publish only the exact
        # field-derived RF scale and rate-target pointers pinned above.
        sanctioned_functions = SANCTIONED_CONSUMER_FUNCTIONS.get(relative, set())
        excess = [f for f in functions_consuming_family(code, views) if f not in sanctioned_functions]
        for function in excess:
            failures.append(f"{relative}: unsanctioned production function over rf scale table storage: {function}")
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
    print(f"RF SCALE TABLES SOURCE DRIFT-EVIDENCE GATE PASSED files={len(paths)}")


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
        raise SystemExit(f"RF SCALE TABLES LINKED DRIFT GATE FAILED\nliterals={dict(literals)!r}\nxrefs={dict(xrefs)!r}")
    residual_literals, residual_xrefs = literals - ALLOWED_LINKED_LITERALS, xrefs - ALLOWED_DECODED_XREFS
    print(f"RF SCALE TABLES LINKED DRIFT-EVIDENCE GATE PASSED allowed_literals={sum(ALLOWED_LINKED_LITERALS.values())} residual_literals={sum(residual_literals.values())} residual_xrefs={sum(residual_xrefs.values())}")


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
        raise SystemExit(f"initialized-rf-scale-tables drift gate failed: {error}")
