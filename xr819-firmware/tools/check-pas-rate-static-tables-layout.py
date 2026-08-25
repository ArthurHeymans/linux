#!/usr/bin/env python3
"""Source/linked drift evidence for exactly [0x04000000, 0x04000138).

The initialized image prefix, three records:

- PasRateStaticTables [0x04000000, 0x04000058): eleven {pointer, value}
  MMIO patch pairs applied by vendor mac_program_slot_timings' tail loop
  (DAT_0000f7ac, static count 0xb): *pointer = value; every recovered
  pointer targets 0x09c00xxx MMIO. mac_program_timing_regs additionally
  stores the pointer VALUE 0x04000004 into a config structure field --
  value-only, never dereferenced in-binary.
- PacDurationQuanta [0x04000058, 0x0400078): eight u32 duration quanta read
  by vendor pac_phy_calc_duration as *(u32*)(0x04000020 + mode * 4) under a
  fw_assert bound 14 <= mode < 22, proving both extent and index domain.
- prefix_suffix [0x0400078, 0x04000138): no observed accessor (whole-binary
  scan found no other loaded pool word in the interval); stays opaque.

Ghidra DATA/PARAM references at pas_reprogram_all_vif_rate_tables (PC
0x7f82: movs/lsls producing 0x04000000 as a bitfield value) and
mac_pipe_irq_service (PC 0x9c12) are value coincidences, not accesses.

Retained Rust pins one sanctioned source literal: mac::program_before_
scan_channel stores the pointer VALUE 0x0400_0000 into MMIO register
0x0270 (mirroring vendor hardware table pointing). Initial COPY values are
loader-owned; no writer closure is claimed: computed, indirect, generic
HIF/debug, vendor, IRQ, and FIQ mutation remain possible. The linked-literal
and decoded PC-relative-xref multisets are derived from decoded loads only;
sanctioned entries are pinned below.
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
RANGE = (0x0400014c, 0x04000194)
OFFSETS = range(0x0, 0x48)
LITERAL = re.compile(r"0x[0-9a-fA-F_]+")
SOURCE_EXTENSIONS = {
    ".rs", ".py", ".sh", ".c", ".h", ".hh", ".hpp", ".hxx", ".cc",
    ".cpp", ".cxx", ".s", ".S", ".asm", ".inc", ".ld", ".lds",
    ".ldh", ".x", ".toml", ".mk",
}
SOURCE_FILENAMES = {"Makefile", "Kconfig"}
OWNER_FILES = {
    "src/dtcm.rs",
    "tools/check-pas-rate-static-tables-layout.py",
    # Adjacent-interval owner: pins the shared rate_encoding boundary
    # 0x04000194 in its linked-literal manifest.
    "tools/check-initialized-tx-rate-tables-layout.py",
}
ADJACENT_DECLARATIONS = {
    "tools/check-pas-rate-static-tables-layout.py": "(0x0400014C, 0x04000194)",
}
SANCTIONED_CONSUMER_LINES: dict[str, set[str]] = {}
SANCTIONED_CONSUMER_FUNCTIONS: dict[str, set[str]] = {}
# Vendor-only tables: no retained Rust code references any address inside
# [0x0400014c, 0x04000194), so both multisets start pinned empty and any
# appearance of an in-interval literal or decoded xref fails the gate.
ALLOWED_LINKED_LITERALS: collections.Counter[int] = collections.Counter()
ALLOWED_DECODED_XREFS: collections.Counter[tuple[str, int]] = collections.Counter()
STRUCT = '#[repr(C, align(4))] struct PasRateStaticTables { mcs_fallback_rates: [SharedU8; 8], pas_fallback_suffix: OpaqueBytes<0x8>, ofdm_airtime_durations: [SharedU16; 16], rate_group_records: [SharedU32; 6] }'
REQUIRED = (
    "pas_rate_static_tables: PasRateStaticTables",
    "rate_encoding: [SharedU8; 22]",
    "assert_type_layout!(SharedU16, 0x02, 2)",
    "assert_type_layout!(SharedU32, 0x04, 4)",
    "assert_type_layout!(PasRateStaticTables, 0x48, 4);",
    "assert!(core::mem::offset_of!(PasRateStaticTables, mcs_fallback_rates) == 0x00);",
    "assert!(core::mem::offset_of!(PasRateStaticTables, pas_fallback_suffix) == 0x08);",
    "assert!(core::mem::size_of::<OpaqueBytes<0x8>>() == 0x8);",
    "assert!(core::mem::offset_of!(PasRateStaticTables, ofdm_airtime_durations) == 0x10);",
    "assert!(core::mem::size_of::<[SharedU16; 16]>() == 0x20);",
    "assert!(core::mem::offset_of!(PasRateStaticTables, rate_group_records) == 0x30);",
    "assert!(core::mem::size_of::<[SharedU32; 6]>() == 0x18);",
    "assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, pas_rate_static_tables) + core::mem::size_of::<PasRateStaticTables>() == 0x0400_0194);",
    "offset_of!(InitializedVendorImage, rate_encoding) == 0x0194",
    "offset_of!(InitializedVendorImage, rate_attributes) == 0x01aa",
    "initialized_prefix_tables_are_exact",
)
PHYSICAL = (0x0400014c, 0x04000154, 0x0400015c, 0x0400017c, 0x04000194)
COMPILE_TIME_PHYSICAL_INVENTORY = (
    'assert!(core::mem::offset_of!(InitializedVendorImage, pas_rate_static_tables) == 0x014c);',
    'assert_type_layout!(PasRateStaticTables, 0x48, 4);',
    'assert!(core::mem::offset_of!(PasRateStaticTables, mcs_fallback_rates) == 0x00);',
    'assert!(core::mem::offset_of!(PasRateStaticTables, pas_fallback_suffix) == 0x08);',
    'assert!(core::mem::size_of::<OpaqueBytes<0x8>>() == 0x8);',
    'assert!(core::mem::offset_of!(PasRateStaticTables, ofdm_airtime_durations) == 0x10);',
    'assert!(core::mem::size_of::<[SharedU16;',
    '16]>() == 0x20);',
    'assert!(core::mem::offset_of!(PasRateStaticTables, rate_group_records) == 0x30);',
    'assert!(core::mem::size_of::<[SharedU32;',
    '6]>() == 0x18);',
    'assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, pas_rate_static_tables) + core::mem::size_of::<PasRateStaticTables>() == 0x0400_0194);',
)
FOCUSED_TEST_INVENTORY = (
    'let image = DTCM_STATE_BASE;',
    'let list = image + core::mem::offset_of!(InitializedVendorImage, mac_slot_timing_patch_list);',
    'let quanta = image + core::mem::offset_of!(InitializedVendorImage, pac_duration_quanta);',
    'let suffix = image + core::mem::offset_of!(InitializedVendorImage, prefix_suffix);',
    'assert_eq!(list, DTCM_STATE_BASE);',
    'assert_eq!(core::mem::size_of::<MacSlotTimingPatchEntry>(), 0x08);',
    'assert_eq!([core::mem::offset_of!(MacSlotTimingPatchEntry, pointer), core::mem::offset_of!(MacSlotTimingPatchEntry, patch_word)], [0x00, 0x04]);',
    'assert_eq!(core::mem::size_of::<MacSlotTimingPatchList>(), 0x58);',
    'assert_eq!(core::mem::size_of::<[MacSlotTimingPatchEntry; 11]>(), 0x58);',
    'assert_eq!(list + core::mem::size_of::<MacSlotTimingPatchList>(), 0x0400_0058);',
    'assert_eq!(quanta, 0x0400_0058);',
    'assert_eq!(core::mem::size_of::<PacDurationQuanta>(), 0x20);',
    'assert_eq!(quanta + core::mem::size_of::<PacDurationQuanta>(), 0x0400_0078);',
    'assert_eq!(suffix, 0x0400_0078);',
    'assert_eq!(core::mem::size_of::<OpaqueBytes<0xc0>>(), 0xc0);',
    'assert_eq!(suffix + core::mem::size_of::<OpaqueBytes<0xc0>>(), 0x0400_0138);',
    'assert_eq!(TX_DURATION_TIMING_TABLE.get(), 0x0400_0138);',
    'let pas_tables = image + core::mem::offset_of!(InitializedVendorImage, pas_rate_static_tables);',
    'assert_eq!(pas_tables, 0x0400_014c);',
    'assert_eq!(core::mem::size_of::<PasRateStaticTables>(), 0x48);',
    'assert_eq!([core::mem::offset_of!(PasRateStaticTables, mcs_fallback_rates), core::mem::offset_of!(PasRateStaticTables, pas_fallback_suffix), core::mem::offset_of!(PasRateStaticTables, ofdm_airtime_durations), core::mem::offset_of!(PasRateStaticTables, rate_group_records)], [0x00, 0x08, 0x10, 0x30]);',
    'assert_eq!(pas_tables + core::mem::size_of::<[SharedU8; 8]>(), 0x0400_0154);',
    'assert_eq!(pas_tables + core::mem::offset_of!(PasRateStaticTables, ofdm_airtime_durations), 0x0400_015c);',
    'assert_eq!(pas_tables + core::mem::offset_of!(PasRateStaticTables, rate_group_records), 0x0400_017c);',
    'assert_eq!(pas_tables + core::mem::size_of::<PasRateStaticTables>(), 0x0400_0194);',
    'let ccb = image + core::mem::offset_of!(InitializedVendorImage, completion_callback_prefix);',
    'assert_eq!(ccb, 0x0400_0228);',
    'assert_eq!(core::mem::size_of::<CompletionCallbackPrefix>(), 0x38);',
    'assert_eq!([core::mem::offset_of!(CompletionCallbackPrefix, completion_prefix_suffix), core::mem::offset_of!(CompletionCallbackPrefix, p2p_action_offsets)], [0x00, 0x30]);',
    'assert_eq!(ccb + core::mem::size_of::<OpaqueBytes<0x30>>(), 0x0400_0258);',
    'assert_eq!(ccb + core::mem::size_of::<CompletionCallbackPrefix>(), 0x0400_0260);',
    'let rcm = image + core::mem::offset_of!(InitializedVendorImage, ring_cursor_map_tables);',
    'assert_eq!(rcm, 0x0400_0288);',
    'assert_eq!(core::mem::size_of::<RingCursorMapTables>(), 0x50);',
    'assert_eq!([core::mem::offset_of!(RingCursorMapTables, beacon_mask_words), core::mem::offset_of!(RingCursorMapTables, mib_defaults_template)], [0x00, 0x20]);',
    'assert_eq!(rcm + core::mem::size_of::<[SharedU32; 8]>(), 0x0400_02a8);',
    'assert_eq!(rcm + core::mem::size_of::<RingCursorMapTables>(), 0x0400_02d8);',
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
    test_scope = named_function(source, "initialized_prefix_tables_are_exact")
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

    compile_item = normalized(COMPILE_TIME_PHYSICAL_INVENTORY[0])
    compile_swap = normalized(compile_scope).replace(
        compile_item, compile_item.replace("== 0x014c", "== 0x014a"), 1
    )
    test_address_item = normalized(FOCUSED_TEST_INVENTORY[17])
    test_address_swap = normalized(test_scope).replace(
        test_address_item,
        swapped_once(test_address_item, "0x0400_014c", "0x0400_014a"),
        1,
    )
    test_extent_item = normalized(FOCUSED_TEST_INVENTORY[24])
    test_extent_swap = normalized(test_scope).replace(
        test_extent_item,
        swapped_once(test_extent_item, "0x0400_0194", "0x0400_0192"),
        1,
    )


def source_paths() -> list[Path]:
    return sorted(path for path in ROOT.rglob("*") if path.is_file()
                  and (path.suffix in SOURCE_EXTENSIONS or path.name in SOURCE_FILENAMES)
                  and "target" not in path.parts and ".git" not in path.parts)


def in_range(value: int) -> bool:
    return RANGE[0] <= value < RANGE[1]


# Interval-specific: offsets 0x0..0x47 are indistinguishable from arbitrary
# small integers, so the template's relative-offset evidence arm is disabled
# entirely (see module docstring).
def owned_literals(code: str) -> list[int]:
    values = [int(match.group().replace("_", ""), 16) for match in LITERAL.finditer(code)]
    return [value for value in values if in_range(value)]



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
    views = {"PasRateStaticTables", "PasRateStaticEntry", "PacDurationQuanta",
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
    # NOTE: relative-offset traps (DTCM_STATE_BASE + 0x2c,
    # DtcmAddress::from_offset(0x2c)) are NOT detectable for this interval:
    # offsets 0x0..0x137 are indistinguishable from arbitrary small integers,
    # so the relative arm is disabled (see owned_literals). Only family-name
    # aliases and in-range physical literals other than the base are tracked.
    fixtures = (
        ("type R = PasRateStaticTables; const ROOT: usize = core::mem::size_of::<R>(); const NEXT: usize = ROOT; fn leak(_: R) -> usize { NEXT }", {"R", "ROOT", "NEXT"}, ["leak"]),
        ("type A = PasRateStaticTables; type B = A; fn leak(_: B) {}", {"A", "B"}, ["leak"]),
        ("use crate::dtcm::PasRateStaticTables as Hidden; type Again = Hidden; fn leak(_: Again) {}", {"Hidden", "Again"}, ["leak"]),
        ("use crate::dtcm::{PasRateStaticTables as H1}; type H2 = H1; const N: usize = core::mem::size_of::<H2>(); fn leak(_: H2) -> usize { N }", {"H1", "H2", "N"}, ["leak"]),
        ("fn reset() { unsafe { core::ptr::write_volatile(0x0400_017c as *mut u32, 0) } }", set(), ["reset"]),
        ("const ENTRY: usize = 0x0400_017c; const NEXT: usize = ENTRY; fn leak() -> usize { NEXT }", {"ENTRY", "NEXT"}, ["leak"]),
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
    focused_test = named_function(source, "initialized_prefix_tables_are_exact")
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
    # The struct is declared across multiple lines with doc comments; strip
    # comment lines and whitespace before comparing.
    stripped = re.sub(r"\s*///[^\n]*", "", source)
    tables_declaration = re.search(
        r"#\[repr\(C, align\(4\)\)\]\s*struct PasRateStaticTables \{[^}]*\}", stripped
    )
    expected_declarations = (
        "#[repr(C, align(4))] struct PasRateStaticTables { mcs_fallback_rates: [SharedU8; 8], pas_fallback_suffix: OpaqueBytes<0x8>, ofdm_airtime_durations: [SharedU16; 16], rate_group_records: [SharedU32; 6] }",
    )
    # Tolerate the trailing field comma before the closing brace.
    def canon(text: str) -> str:
        return normalized(re.sub(r",?\s*\}", "}", text))
    if (tables_declaration is None
            or canon(tables_declaration.group()) != canon(expected_declarations[0])):
        failures.append("src/dtcm.rs: PasRateStaticTables must be the exact private non-derived inventory")
    if (source.count("struct PasRateStaticTables") != 1
            or "pre_rate_tables" in source):
        failures.append("src/dtcm.rs: removed opaque split or duplicate pas rate static tables remain")
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
    if re.search(r"\bimpl(?:\s*<[^>]*>)?\s+[^\{]*(?:PasRateStaticTables|PasRateStaticEntry|PacDurationQuanta)", production):
        failures.append("production impl for PAS rate static tables is forbidden")
    for match in re.finditer(r"\b(?:const|static)\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)[^;]*;", production):
        if view_pattern.search(match.group()) and "RF_MODE_HALFWORD_TABLE:" not in normalized(match.group()):
            failures.append(f"direct scheduler tail const/static alias is forbidden: {match.group(1)}")
    for match in re.finditer(r"\b(?:unsafe\s+)?(?:const\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*register_write_lists_entry_marker[A-Za-z0-9_]*)", production, re.I):
        failures.append(f"named scheduler tail production API is forbidden: {match.group(1)}")
    forbidden_operation = re.compile(r"\*(?:const|mut)|&(?:mut\s+)?|\b(?:read|write)(?:_volatile)?\s*\(|\b(?:value|init(?:ialize)?|reset|unchecked|generic_offset|slice|iter(?:ator)?)\b(?!\s*:)", re.I)
    for relative, code in rust.items():
        # start_scheduler_timer (tx.rs) is the retained Rust translation of
        # vendor timer_start; it reads exactly the guard word 0x0400_017c that
        # timer_start itself reads, via the single pinned consumer line below.
        sanctioned_functions = SANCTIONED_CONSUMER_FUNCTIONS.get(relative, set())
        excess = [f for f in functions_consuming_family(code, views) if f not in sanctioned_functions]
        for function in excess:
            failures.append(f"{relative}: unsanctioned production function over pas rate static table storage: {function}")
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
    print(f"PAS RATE STATIC TABLES SOURCE DRIFT-EVIDENCE GATE PASSED files={len(paths)}")


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
        raise SystemExit(f"PAS RATE STATIC TABLES LINKED DRIFT GATE FAILED\nliterals={dict(literals)!r}\nxrefs={dict(xrefs)!r}")
    residual_literals, residual_xrefs = literals - ALLOWED_LINKED_LITERALS, xrefs - ALLOWED_DECODED_XREFS
    print(f"PAS RATE STATIC TABLES LINKED DRIFT-EVIDENCE GATE PASSED allowed_literals={sum(ALLOWED_LINKED_LITERALS.values())} residual_literals={sum(residual_literals.values())} residual_xrefs={sum(residual_xrefs.values())}")


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
        raise SystemExit(f"initialized-prefix-tables drift gate failed: {error}")
