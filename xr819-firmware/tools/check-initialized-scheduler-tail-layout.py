#!/usr/bin/env python3
"""Source/linked drift evidence for exactly [0x04002018, 0x04002078).

The initialized tail island behind the scheduler event island. Sub-record
evidence: the hardware-timer programming guard word at 0x04002028 is read by
vendor timer_start (pool DAT_0000f310 + 0x14) right after the armed timer
became list head; the thirty-two RF calibration payload bytes at
0x0400202c..0x0400204c are copied to MMIO 0x0abb8300 by dbg_expand_byte_table
(DAT_000168d0) before reg_write_list_apply(0x04000e90), and rebuilt from TLV
type 0x22 payloads by the tlv_dispatch_table handler tail at 0x177d6; the ten
u32 error-event counters at 0x04002050..0x04002078 accumulate byte lanes of
an error event payload in event_send_error_0x34's inlined tail
(pool DAT_00010d8).

The opaque islands 0x2018..0x2028 and 0x204c..0x2050 have no observed
accessor. Both multisets are pinned empty: no linked literal inside this
interval exists. The guard word has no pool entry of its own -- vendor
timer_start reaches it as *(scheduler_timer_list_head_root + 0x14), while
the retained Rust translation uses its exact field-derived address. The timer
list root 0x04002014 remains owned by the adjacent scheduler-event checker.
Empty sets are drift evidence, never writer
closure: computed, indirect, generic HIF/debug, vendor, IRQ, and FIQ
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
RANGE = (0x04002018, 0x04002078)
OFFSETS = range(0x2018, 0x2078)
LITERAL = re.compile(r"0x[0-9a-fA-F_]+")
SOURCE_EXTENSIONS = {
    ".rs", ".py", ".sh", ".c", ".h", ".hh", ".hpp", ".hxx", ".cc",
    ".cpp", ".cxx", ".s", ".S", ".asm", ".inc", ".ld", ".lds",
    ".ldh", ".x", ".toml", ".mk",
}
SOURCE_FILENAMES = {"Makefile", "Kconfig"}
OWNER_FILES = {
    "src/dtcm.rs",
    "tools/check-initialized-scheduler-tail-layout.py",
}
ADJACENT_DECLARATIONS = {
    "tools/check-scheduler-event-layout.py": "SCHEDULER_EVENT_RANGE = (0x04001FCC, 0x04002018)",
}
SANCTIONED_CONSUMER_LINES: dict[str, set[str]] = {
    "src/tx.rs": {"if became_head && read_u32(crate::dtcm::SCHEDULER_HARDWARE_TIMER_GUARD.get()) == 0 {"},
}
SANCTIONED_CONSUMER_FUNCTIONS: dict[str, set[str]] = {
    "src/tx.rs": {"start_scheduler_timer"},
}
# No linked literals fall inside this interval: the guard word is reached
# relative to the scheduler timer list head root (owned by the adjacent
# checker), so both multisets are pinned empty.
ALLOWED_LINKED_LITERALS: collections.Counter[int] = collections.Counter()
ALLOWED_DECODED_XREFS: collections.Counter[tuple[str, int]] = collections.Counter()
STRUCT = "#[repr(C, align(4))] struct SchedulerTail { opaque_2018: OpaqueBytes<0x10>, hardware_timer_guard: SharedU32, rf_calibration_bytes: [SharedU8; 32], opaque_204c: OpaqueBytes<0x04>, error_event_counts: [SharedU32; 10] }"
REQUIRED = (
    STRUCT,
    "initialized_tail: SchedulerTail,",
    "scheduler_event_island: SchedulerEventIsland",
    "pub(crate) const SCHEDULER_HARDWARE_TIMER_GUARD: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, initialized_tail) + core::mem::offset_of!(SchedulerTail, hardware_timer_guard));",
    "assert_type_layout!(SharedU32, 0x04, 4)",
    "assert_type_layout!(SharedU8, 0x01, 1)",
    "assert_type_layout!(SchedulerTail, 0x60, 4)",
    "offset_of!(SchedulerTail, hardware_timer_guard) == 0x10",
    "offset_of!(SchedulerTail, rf_calibration_bytes) == 0x14",
    "offset_of!(SchedulerTail, error_event_counts) == 0x38",
    "size_of::<[SharedU8; 32]>() == 0x20",
    "size_of::<[SharedU32; 10]>() == 0x28",
    "offset_of!(InitializedVendorImage, initialized_tail) == 0x2018",
    "assert_type_layout!(InitializedVendorImage, 0x2078, 4)",
    "assert_type_layout!(DtcmLayout, DTCM_STATE_SIZE, 4)",
    "assert_type_layout!(SharedDtcmState, DTCM_STATE_SIZE, 4)",
    "fn initialized_scheduler_tail_is_exact()",
    "size_of::<SchedulerTail>(), 0x60",
    "align_of::<SchedulerTail>(), 4",
    "hardware_timer_guard), 0x10",
    "hardware_timer_guard), 0x0400_2028",
    "rf_calibration_bytes), 0x14",
    "rf_calibration_bytes), 0x0400_202c",
    "error_event_counts), 0x38",
    "error_event_counts), 0x0400_2050",
    "error_event_counts) + core::mem::size_of::<[SharedU32; 10]>(), 0x0400_2078",
    "scheduler_timer_list_head().get(), 0x0400_2014",
    "size_of::<InitializedVendorImage>(), 0x2078",
    "align_of::<InitializedVendorImage>(), 4",
    "size_of::<DtcmLayout>(), DTCM_STATE_SIZE",
    "align_of::<DtcmLayout>(), 4",
    "size_of::<SharedDtcmState>(), DTCM_STATE_SIZE",
    "align_of::<SharedDtcmState>(), 4",
)
PHYSICAL = (0x04002018, 0x04002028, 0x0400202C, 0x0400204C, 0x04002050, 0x04002078)
COMPILE_TIME_PHYSICAL_INVENTORY = (
    "assert!(core::mem::size_of::<SchedulerTail>() == 0x60);",
    "assert!(core::mem::align_of::<SchedulerTail>() == 4);",
    "assert!(core::mem::offset_of!(SchedulerTail, hardware_timer_guard) == 0x10);",
    "assert!(core::mem::offset_of!(SchedulerTail, rf_calibration_bytes) == 0x14);",
    "assert!(core::mem::size_of::<[SharedU8; 32]>() == 0x20);",
    "assert!(core::mem::offset_of!(SchedulerTail, opaque_204c) == 0x34);",
    "assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, initialized_tail) + core::mem::offset_of!(SchedulerTail, opaque_204c) == 0x0400_204c);",
    "assert!(core::mem::offset_of!(SchedulerTail, error_event_counts) == 0x38);",
    "assert!(core::mem::size_of::<[SharedU32; 10]>() == 0x28);",
)
FOCUSED_TEST_INVENTORY = (
    "let image = DTCM_STATE_BASE;",
    "let tail = image + core::mem::offset_of!(InitializedVendorImage, initialized_tail);",
    "assert_eq!(core::mem::size_of::<SchedulerTail>(), 0x60);",
    "assert_eq!(core::mem::align_of::<SchedulerTail>(), 4);",
    "assert_eq!(core::mem::offset_of!(SchedulerTail, hardware_timer_guard), 0x10);",
    "assert_eq!(tail + core::mem::offset_of!(SchedulerTail, hardware_timer_guard), 0x0400_2028);",
    "assert_eq!(SCHEDULER_HARDWARE_TIMER_GUARD.get(), 0x0400_2028);",
    "assert_eq!(core::mem::offset_of!(SchedulerTail, rf_calibration_bytes), 0x14);",
    "assert_eq!(core::mem::size_of::<[SharedU8; 32]>(), 0x20);",
    "assert_eq!(tail + core::mem::offset_of!(SchedulerTail, rf_calibration_bytes), 0x0400_202c);",
    "assert_eq!(core::mem::offset_of!(SchedulerTail, error_event_counts), 0x38);",
    "assert_eq!(core::mem::size_of::<[SharedU32; 10]>(), 0x28);",
    "assert_eq!(tail + core::mem::offset_of!(SchedulerTail, error_event_counts), 0x0400_2050);",
    "assert_eq!(tail + core::mem::offset_of!(SchedulerTail, error_event_counts) + core::mem::size_of::<[SharedU32; 10]>(), 0x0400_2078);",
    "assert_eq!(tail + core::mem::size_of::<SchedulerTail>(), 0x0400_2078);",
    "assert_eq!(scheduler_timer_list_head().get(), 0x0400_2014);",
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
    sentinel = "__SCHEDULER_TAIL_SWAP__"
    return source.replace(left, sentinel, 1).replace(right, left, 1).replace(sentinel, right, 1)


def check_exact_inventory_regression(source: str) -> None:
    compile_scope = mask_test_module(source)
    test_scope = named_function(source, "initialized_scheduler_tail_is_exact")
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
        compile_item, compile_item.replace("== 0x38", "== 0x3c"), 1
    )
    test_address_item = normalized(FOCUSED_TEST_INVENTORY[12])
    test_address_swap = normalized(test_scope).replace(
        test_address_item,
        swapped_once(test_address_item, "0x0400_2050", "0x0400_204c"),
        1,
    )
    test_extent_item = normalized(FOCUSED_TEST_INVENTORY[14])
    test_extent_swap = normalized(test_scope).replace(
        test_extent_item,
        swapped_once(test_extent_item, "0x0400_2078", "0x0400_2074"),
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
            ("const", r"\bconst\s+(?!fn\b)([A-Za-z_][A-Za-z0-9_]*)\s*:[^=;]+\s*=\s*([^;]*);"),
            ("static", r"\bstatic\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:[^=;]+\s*=\s*([^;]*);"),
            ("type", r"\btype\s+([A-Za-z_][A-Za-z0-9_]*)\s*=\s*([^;]*);"),
        )
        for name, initializer in re.findall(pattern, code)
    ]
    imported = rust_use_aliases(code)
    views = {"MacPipeTail", "SchedulerTail", "scheduler_tail",
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
        ("const ROOT: usize = scheduler_tail; const NEXT: usize = ROOT; fn leak() -> *mut u32 { NEXT as *mut u32 }", {"ROOT", "NEXT"}, ["leak"]),
        ("static ROOT: usize = 0x0400_2028; static NEXT: usize = ROOT; fn leak() -> usize { NEXT }", {"ROOT", "NEXT"}, ["leak"]),
        ("const fn unrelated() -> usize { 0 } const ROOT: usize = 0x0400_2028; fn leak() -> usize { ROOT }", {"ROOT"}, ["leak"]),
        ("type Hidden = SchedulerTail; type Again = Hidden; fn leak(_: Again) {}", {"Hidden", "Again"}, ["leak"]),
        ("use crate::dtcm::SchedulerTail as Hidden; const ROOT: usize = scheduler_tail; fn leak(_: Hidden) -> usize { ROOT }", {"Hidden", "ROOT"}, ["leak"]),
        ("use crate::dtcm::{SchedulerTail as Hidden, scheduler_tail as Root}; const NEXT: usize = Root; fn leak(_: Hidden) -> usize { NEXT }", {"Hidden", "Root", "NEXT"}, ["leak"]),
        ("use crate::{dtcm::{SchedulerTail as Hidden}}; fn leak(_: Hidden) {}", {"Hidden"}, ["leak"]),
        ("fn leak() -> *mut u32 { (DTCM_STATE_BASE + 0x2054) as *mut u32 }", set(), ["leak"]),
        ("const ENTRY: usize = DTCM_STATE_BASE + 0x2054; const NEXT: usize = ENTRY; fn leak() -> *mut u32 { NEXT as *mut u32 }", {"ENTRY", "NEXT"}, ["leak"]),
        ("fn leak() -> *mut u32 { DtcmAddress::from_offset(0x2028).cast_mut() }", set(), ["leak"]),
        ("const ENTRY: DtcmAddress = DtcmAddress::from_offset(0x2054); const NEXT: DtcmAddress = ENTRY; fn leak() -> usize { NEXT.get() }", {"ENTRY", "NEXT"}, ["leak"]),
        ("fn leak() -> usize { DtcmAddress::from_offset_unchecked(0x2054).get() }", set(), ["leak"]),
        ("use crate::{dtcm::{DtcmAddress as HiddenAddress}}; const ENTRY: HiddenAddress = HiddenAddress::from_offset(0x2028); const NEXT: HiddenAddress = ENTRY; fn leak() -> usize { NEXT.get() }", {"ENTRY", "NEXT"}, ["leak"]),
        ("fn reset() { unsafe { core::ptr::write_volatile(0x0400_2028 as *mut u32, 0) } }", set(), ["reset"]),
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
    focused_test = named_function(source, "initialized_scheduler_tail_is_exact")
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
    declaration = re.search(r"#\[repr\(C, align\(4\)\)\] struct SchedulerTail \{[^}]*\}", source)
    if declaration is None or normalized(declaration.group()) != normalized(STRUCT) or "derive" in declaration.group():
        failures.append("src/dtcm.rs: SchedulerTail must be the exact private non-derived inventory")
    if source.count("struct SchedulerTail") != 1 or "pre_mac_phy_command_state: OpaqueBytes<0x208>" in source:
        failures.append("src/dtcm.rs: removed opaque split or duplicate scheduler tails remain")
    for relative, exact in ADJACENT_DECLARATIONS.items():
        if exact not in (ROOT / relative).read_text():
            failures.append(f"{relative}: adjacent checker range changed from {exact}")

    paths = source_paths()
    rust = {path.relative_to(ROOT).as_posix(): mask_test_module(code_only(path.read_text(errors="replace")))
            for path in paths if path.suffix == ".rs"}
    rust["src/dtcm.rs"] = rust["src/dtcm.rs"].replace(STRUCT, " " * len(STRUCT))
    production = "\n".join(rust.values())
    views, declarations = family_aliases(production)
    sanctioned_roots = {"SCHEDULER_HARDWARE_TIMER_GUARD"}
    for kind, name, _ in declarations:
        if name in views and name not in sanctioned_roots:
            failures.append(f"additional scheduler tail {kind} alias is forbidden: {name}")
    view_pattern = re.compile(rf"\b(?:{'|'.join(sorted(map(re.escape, views)))})\b")
    if re.search(r"\bimpl(?:\s*<[^>]*>)?\s+[^\{]*MacPipeTail", production):
        failures.append("production impl for SchedulerTail is forbidden")
    for match in re.finditer(r"\b(?:const\s+(?!fn\b)|static\s+)(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)[^;]*;", production):
        if (view_pattern.search(match.group())
                and match.group(1) not in sanctioned_roots
                and "RF_MODE_HALFWORD_TABLE:" not in normalized(match.group())):
            failures.append(f"direct scheduler tail const/static alias is forbidden: {match.group(1)}")
    for match in re.finditer(r"\b(?:unsafe\s+)?(?:const\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*scheduler_tail_entry_marker[A-Za-z0-9_]*)", production, re.I):
        failures.append(f"named scheduler tail production API is forbidden: {match.group(1)}")
    forbidden_operation = re.compile(r"\*(?:const|mut)|&(?:mut\s+)?|\b(?:read|write)(?:_volatile)?\s*\(|\b(?:value|init(?:ialize)?|reset|unchecked|generic_offset|slice|iter(?:ator)?)\b(?!\s*:)", re.I)
    for relative, code in rust.items():
        # start_scheduler_timer reads exactly the field-derived guard word
        # used by the retained vendor timer-start translation.
        sanctioned_functions = SANCTIONED_CONSUMER_FUNCTIONS.get(relative, set())
        excess = [f for f in functions_consuming_family(code, views) if f not in sanctioned_functions]
        for function in excess:
            failures.append(f"{relative}: unsanctioned production function over scheduler tail storage: {function}")
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
    print(f"INITIALIZED SCHEDULER TAIL SOURCE DRIFT-EVIDENCE GATE PASSED files={len(paths)}")


def linked_literals(path: Path) -> collections.Counter[int]:
    # Deliberately derived from decoded PC-relative literal loads, not from an
    # aligned-word scan of .text: the retained code at 0x6418 contains the
    # instruction pair `movs r0, #0x67; lsls r0, #0x10`, whose bytes form the
    # word 0x04002067 inside this interval -- a pure data/instruction
    # coincidence (same precedent as check-vif-layout.py).
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
        raise SystemExit(f"INITIALIZED SCHEDULER TAIL LINKED DRIFT GATE FAILED\nliterals={dict(literals)!r}\nxrefs={dict(xrefs)!r}")
    residual_literals, residual_xrefs = literals - ALLOWED_LINKED_LITERALS, xrefs - ALLOWED_DECODED_XREFS
    print(f"INITIALIZED SCHEDULER TAIL LINKED DRIFT-EVIDENCE GATE PASSED allowed_literals={sum(ALLOWED_LINKED_LITERALS.values())} residual_literals={sum(residual_literals.values())} residual_xrefs={sum(residual_xrefs.values())}")


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
        raise SystemExit(f"initialized-scheduler-tail drift gate failed: {error}")
