#!/usr/bin/env python3
"""Source/linked drift evidence for exactly [0x04001D5C, 0x04001E6C).

Four per-MAC-pipe tail records with stride 0x44 rooted at g_fw_ctx
(0x04001680) + pipe * 0x44 + 0x6dc: vendor pipe_setup_entry writes the setup
halfwords/type/cleared word and clears frame_count, pipe_clear_entry clears
frame_count, stats_export_pipe_counters reads the setup words, type byte,
counter words 0x708..0x71c, and zero-tests frame_count, and the tail copy
loop refills 0x708..0x71c from MMIO scratch. The absolute root 0x04001e60
(= pipe 3 + 0x714) reads/writes the scheduler flag byte inside
counter_word_714 and the byte just past this range; those remain vendor
operations, not Rust semantics.

The translated `mac::export_pipe_counters` reader now reaches each reviewed
field through exact `src/dtcm.rs` accessors. The single pinned aligned literal /
decoded xref (0x04001d9c inside `mac::reinitialize_after_wake`) is its inlined
pipe-0 scratch-word read. This is reader closure only: computed, indirect,
generic HIF/debug, vendor, IRQ, and FIQ mutation remain possible.
"""

from __future__ import annotations

import argparse
import collections
import hashlib
import re
import struct
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RANGE = (0x04001D5C, 0x04001E6C)
OFFSETS = range(0x1D5C, 0x1E6C)
LITERAL = re.compile(r"0x[0-9a-fA-F_]+")
SOURCE_EXTENSIONS = {
    ".rs", ".py", ".sh", ".c", ".h", ".hh", ".hpp", ".hxx", ".cc",
    ".cpp", ".cxx", ".s", ".S", ".asm", ".inc", ".ld", ".lds",
    ".ldh", ".x", ".toml", ".mk",
}
SOURCE_FILENAMES = {"Makefile", "Kconfig"}
OWNER_FILES = {
    "src/dtcm.rs",
    "tools/check-initialized-mac-pipe-tails-layout.py",
}
ADJACENT_DECLARATIONS = {
    "tools/check-mac-phy-command-state-layout.py": "(0x04001D10, 0x04001D5C)",
    "tools/check-mac-retry-hardware-state-layout.py": "MAC_RETRY_HARDWARE_STATE_RANGE = (0x04001E6C, 0x04001E70)",
}
SANCTIONED_ROOTS = {"MAC_PIPE_TAILS"}
SANCTIONED_APIS = {
    "mac_pipe_tail_field_unchecked",
    "mac_pipe_tail_setup_words_unchecked",
    "mac_pipe_tail_setup_word_6e0_unchecked",
    "mac_pipe_tail_type_byte_unchecked",
    "mac_pipe_tail_counter_708_unchecked",
    "mac_pipe_tail_counter_70c_unchecked",
    "mac_pipe_tail_counter_710_unchecked",
    "mac_pipe_tail_counter_714_unchecked",
    "mac_pipe_tail_frame_count_unchecked",
    "mac_pipe_tail_scratch_unchecked",
}
SANCTIONED_CONSUMER_FUNCTIONS = {
    "src/dtcm.rs": SANCTIONED_APIS,
    "src/mac.rs": {"export_pipe_counters"},
}
SANCTIONED_EXPORT_BODY_SHA256 = "64f5223d81e112027a32b47f2b8282da12bca58001be5b27eb0a1f4bed46a9bd"
SANCTIONED_READER_PATTERNS = {
    "mac_pipe_tail_setup_words_unchecked": ("read_u32", 1),
    "mac_pipe_tail_setup_word_6e0_unchecked": ("read_u32", 1),
    "mac_pipe_tail_type_byte_unchecked": ("read_u8", 1),
    "mac_pipe_tail_counter_708_unchecked": ("read_u32", 1),
    "mac_pipe_tail_counter_70c_unchecked": ("read_u32", 1),
    "mac_pipe_tail_counter_710_unchecked": ("read_u32", 1),
    "mac_pipe_tail_counter_714_unchecked": ("read_u32", 1),
    "mac_pipe_tail_frame_count_unchecked": ("read_u32", 2),
    "mac_pipe_tail_scratch_unchecked": ("read_u32", 1),
}
# Retained translated consumer: mac::export_pipe_counters reads the reviewed
# per-pipe tail fields through exact owner APIs. LLVM folds the pipe-0 scratch
# word into this literal inside the inlined reinitialize_after_wake.
ALLOWED_LINKED_LITERALS: collections.Counter[int] = collections.Counter({0x04001D9C: 1})
ALLOWED_DECODED_XREFS: collections.Counter[tuple[str, int]] = collections.Counter({
    ("_RNvNtCsiHlLB2CErfM_14xr819_firmware3mac23reinitialize_after_wake", 0x04001D9C): 1,
})
STRUCT = (
    "#[repr(C, align(4))] struct MacPipeTail { setup_word_6dc: SharedU16, setup_word_6de: SharedU16, "
    "setup_word_6e0: SharedU16, opaque_6e2: OpaqueBytes<0x02>, opaque_6e4: OpaqueBytes<0x1c>, "
    "type_byte_700: SharedU8, opaque_701: OpaqueBytes<0x03>, cleared_word_704: SharedU32, "
    "counter_word_708: SharedU32, counter_word_70c: SharedU32, counter_word_710: SharedU32, "
    "counter_word_714: SharedU32, frame_count_718: SharedU32, scratch_word_71c: SharedU32 }",
    "#[repr(C, align(4))] struct MacPipeTails { records: [MacPipeTail; 4] }",
)
REQUIRED = (
    STRUCT[0],
    STRUCT[1],
    "mac_pipe_tails: MacPipeTails,",
    "mac_retry_hardware_state: MacRetryHardwareState",
    "assert_type_layout!(SharedU16, 0x02, 2)",
    "assert_type_layout!(MacPipeTail, 0x44, 4)",
    "assert_type_layout!(MacPipeTails, 0x110, 4)",
    "offset_of!(MacPipeTails, records) == 0",
    "offset_of!(MacPipeTail, setup_word_6dc) == 0x00",
    "offset_of!(MacPipeTail, setup_word_6de) == 0x02",
    "offset_of!(MacPipeTail, setup_word_6e0) == 0x04",
    "offset_of!(MacPipeTail, opaque_6e2) == 0x06",
    "offset_of!(MacPipeTail, opaque_6e4) == 0x08",
    "offset_of!(MacPipeTail, type_byte_700) == 0x24",
    "offset_of!(MacPipeTail, opaque_701) == 0x25",
    "offset_of!(MacPipeTail, cleared_word_704) == 0x28",
    "offset_of!(MacPipeTail, counter_word_708) == 0x2c",
    "offset_of!(MacPipeTail, counter_word_70c) == 0x30",
    "offset_of!(MacPipeTail, counter_word_710) == 0x34",
    "offset_of!(MacPipeTail, counter_word_714) == 0x38",
    "offset_of!(MacPipeTail, frame_count_718) == 0x3c",
    "offset_of!(MacPipeTail, scratch_word_71c) == 0x40",
    "offset_of!(InitializedVendorImage, mac_pipe_tails) == 0x1d5c",
    "offset_of!(InitializedVendorImage, mac_retry_hardware_state) == 0x1e6c",
    "assert_type_layout!(InitializedVendorImage, 0x2078, 4)",
    "assert_type_layout!(DtcmLayout, DTCM_STATE_SIZE, 4)",
    "assert_type_layout!(SharedDtcmState, DTCM_STATE_SIZE, 4)",
    "MAC_PIPE_TAILS: DtcmAddress",
    "fn mac_pipe_tail_field_unchecked(pipe: usize, offset: usize) -> DtcmAddress",
    "fn mac_pipe_tail_setup_words_unchecked(pipe: usize) -> DtcmAddress",
    "fn mac_pipe_tail_setup_word_6e0_unchecked(pipe: usize) -> DtcmAddress",
    "fn mac_pipe_tail_type_byte_unchecked(pipe: usize) -> DtcmAddress",
    "fn mac_pipe_tail_counter_708_unchecked(pipe: usize) -> DtcmAddress",
    "fn mac_pipe_tail_counter_70c_unchecked(pipe: usize) -> DtcmAddress",
    "fn mac_pipe_tail_counter_710_unchecked(pipe: usize) -> DtcmAddress",
    "fn mac_pipe_tail_counter_714_unchecked(pipe: usize) -> DtcmAddress",
    "fn mac_pipe_tail_frame_count_unchecked(pipe: usize) -> DtcmAddress",
    "fn mac_pipe_tail_scratch_unchecked(pipe: usize) -> DtcmAddress",
    "fn initialized_mac_pipe_tail_accessors_are_exact()",
    "fn initialized_mac_pipe_tails_are_exact()",
    "size_of::<MacPipeTail>(), 0x44",
    "align_of::<MacPipeTail>(), 4",
    "size_of::<MacPipeTails>(), 0x110",
    "align_of::<MacPipeTails>(), 4",
    "MAC_PHY_COMMAND_STATE.get() + core::mem::size_of::<MacPhyCommandState>(), 0x0400_1d5c",
    "MAC_RETRY_HARDWARE_STATE.get(), 0x0400_1e6c",
    "size_of::<InitializedVendorImage>(), 0x2078",
    "align_of::<InitializedVendorImage>(), 4",
    "size_of::<DtcmLayout>(), DTCM_STATE_SIZE",
    "align_of::<DtcmLayout>(), 4",
    "size_of::<SharedDtcmState>(), DTCM_STATE_SIZE",
    "align_of::<SharedDtcmState>(), 4",
)
PHYSICAL = (0x04001D5C, 0x04001DA0, 0x04001E28, 0x04001E6C)
COMPILE_TIME_PHYSICAL_INVENTORY = (
    "assert_type_layout!(MacPipeTail, 0x44, 4);",
    "assert_type_layout!(MacPipeTails, 0x110, 4);",
    "assert!(core::mem::offset_of!(MacPipeTails, records) == 0);",
    "assert!(core::mem::offset_of!(MacPipeTail, setup_word_6dc) == 0x00);",
    "assert!(core::mem::offset_of!(MacPipeTail, setup_word_6de) == 0x02);",
    "assert!(core::mem::offset_of!(MacPipeTail, setup_word_6e0) == 0x04);",
    "assert!(core::mem::offset_of!(MacPipeTail, opaque_6e2) == 0x06);",
    "assert!(core::mem::offset_of!(MacPipeTail, opaque_6e4) == 0x08);",
    "assert!(core::mem::offset_of!(MacPipeTail, type_byte_700) == 0x24);",
    "assert!(core::mem::offset_of!(MacPipeTail, opaque_701) == 0x25);",
    "assert!(core::mem::offset_of!(MacPipeTail, cleared_word_704) == 0x28);",
    "assert!(core::mem::offset_of!(MacPipeTail, counter_word_708) == 0x2c);",
    "assert!(core::mem::offset_of!(MacPipeTail, counter_word_70c) == 0x30);",
    "assert!(core::mem::offset_of!(MacPipeTail, counter_word_710) == 0x34);",
    "assert!(core::mem::offset_of!(MacPipeTail, counter_word_714) == 0x38);",
    "assert!(core::mem::offset_of!(MacPipeTail, frame_count_718) == 0x3c);",
    "assert!(core::mem::offset_of!(MacPipeTail, scratch_word_71c) == 0x40);",
    "assert!(core::mem::offset_of!(InitializedVendorImage, mac_pipe_tails) == 0x1d5c);",
    "assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, mac_pipe_tails) + core::mem::size_of::<MacPipeTails>() == 0x0400_1e6c);",
    "assert!(core::mem::offset_of!(InitializedVendorImage, mac_retry_hardware_state) == 0x1e6c);",
)
FOCUSED_TEST_INVENTORY = (
    "let image = DTCM_STATE_BASE;",
    "let tails = image + core::mem::offset_of!(InitializedVendorImage, mac_pipe_tails);",
    "assert_eq!(core::mem::size_of::<MacPipeTail>(), 0x44);",
    "assert_eq!(core::mem::align_of::<MacPipeTail>(), 4);",
    "assert_eq!(core::mem::size_of::<MacPipeTails>(), 0x110);",
    "assert_eq!(core::mem::align_of::<MacPipeTails>(), 4);",
    "assert_eq!([core::mem::offset_of!(MacPipeTail, setup_word_6dc), core::mem::offset_of!(MacPipeTail, setup_word_6de), core::mem::offset_of!(MacPipeTail, setup_word_6e0), core::mem::offset_of!(MacPipeTail, opaque_6e2), core::mem::offset_of!(MacPipeTail, opaque_6e4), core::mem::offset_of!(MacPipeTail, type_byte_700), core::mem::offset_of!(MacPipeTail, opaque_701), core::mem::offset_of!(MacPipeTail, cleared_word_704), core::mem::offset_of!(MacPipeTail, counter_word_708), core::mem::offset_of!(MacPipeTail, counter_word_70c), core::mem::offset_of!(MacPipeTail, counter_word_710), core::mem::offset_of!(MacPipeTail, counter_word_714), core::mem::offset_of!(MacPipeTail, frame_count_718), core::mem::offset_of!(MacPipeTail, scratch_word_71c)], [0x00, 0x02, 0x04, 0x06, 0x08, 0x24, 0x25, 0x28, 0x2c, 0x30, 0x34, 0x38, 0x3c, 0x40]);",
    "assert_eq!(tails, 0x0400_1d5c);",
    "assert_eq!(tails + core::mem::size_of::<MacPipeTail>(), 0x0400_1da0);",
    "assert_eq!(tails + 3 * core::mem::size_of::<MacPipeTail>(), 0x0400_1e28);",
    "assert_eq!(tails + core::mem::size_of::<MacPipeTails>(), 0x0400_1e6c);",
    "assert_eq!(MAC_PHY_COMMAND_STATE.get() + core::mem::size_of::<MacPhyCommandState>(), 0x0400_1d5c);",
    "assert_eq!(MAC_RETRY_HARDWARE_STATE.get(), 0x0400_1e6c);",
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


def macro_definitions(source: str) -> list[str]:
    definitions: list[str] = []
    for start in re.finditer(r"\bmacro_rules\s*!\s*[A-Za-z_][A-Za-z0-9_]*\s*\{", source):
        depth = 1
        index = start.end()
        while index < len(source) and depth:
            depth += (source[index] == "{") - (source[index] == "}")
            index += 1
        if depth == 0:
            definitions.append(source[start.start():index])
    return definitions


def missing_inventory(scope: str, inventory: tuple[str, ...]) -> list[str]:
    compact = normalized(scope)
    return [item for item in inventory if normalized(item) not in compact]


def exact_inventory_present(scope: str, inventory: tuple[str, ...]) -> bool:
    return normalized(" ".join(inventory)) in normalized(scope)


def swapped_once(source: str, left: str, right: str) -> str:
    sentinel = "__MAC_PIPE_TAILS_SWAP__"
    return source.replace(left, sentinel, 1).replace(right, left, 1).replace(sentinel, right, 1)


def check_exact_inventory_regression(source: str) -> None:
    compile_scope = mask_test_module(source)
    test_scope = named_function(source, "initialized_mac_pipe_tails_are_exact")
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

    compile_item = normalized(COMPILE_TIME_PHYSICAL_INVENTORY[17])
    compile_swap = normalized(compile_scope).replace(
        compile_item, compile_item.replace("== 0x1d5c", "== 0x1da0"), 1
    )
    test_address_item = normalized(FOCUSED_TEST_INVENTORY[7])
    test_address_swap = normalized(test_scope).replace(
        test_address_item,
        swapped_once(test_address_item, "0x0400_1d5c", "0x0400_1da0"),
        1,
    )
    test_extent_item = normalized(FOCUSED_TEST_INVENTORY[9])
    test_extent_swap = normalized(test_scope).replace(
        test_extent_item,
        swapped_once(test_extent_item, "0x0400_1e28", "0x0400_1e6c"),
        1,
    )
    if exact_inventory_present(compile_swap, COMPILE_TIME_PHYSICAL_INVENTORY):
        raise SystemExit("checker self-test failed: swapped compile-time extent mappings were accepted")
    if exact_inventory_present(test_address_swap, FOCUSED_TEST_INVENTORY):
        raise SystemExit("checker self-test failed: swapped focused-test base addresses were accepted")
    if exact_inventory_present(test_extent_swap, FOCUSED_TEST_INVENTORY):
        raise SystemExit("checker self-test failed: swapped focused-test queue extents were accepted")


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
    views = {"MacPipeTail", "MacPipeTails", "mac_pipe_tails", *SANCTIONED_APIS,
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
        ("const ROOT: usize = mac_pipe_tails; const NEXT: usize = ROOT; fn leak() -> *mut u32 { NEXT as *mut u32 }", {"ROOT", "NEXT"}, ["leak"]),
        ("static ROOT: usize = 0x0400_1d5c; static NEXT: usize = ROOT; fn leak() -> usize { NEXT }", {"ROOT", "NEXT"}, ["leak"]),
        ("type Hidden = MacPipeTails; type Again = Hidden; fn leak(_: Again) {}", {"Hidden", "Again"}, ["leak"]),
        ("use crate::dtcm::MacPipeTails as Hidden; const ROOT: usize = mac_pipe_tails; fn leak(_: Hidden) -> usize { ROOT }", {"Hidden", "ROOT"}, ["leak"]),
        ("use crate::dtcm::{MacPipeTails as Hidden, mac_pipe_tails as Root}; const NEXT: usize = Root; fn leak(_: Hidden) -> usize { NEXT }", {"Hidden", "Root", "NEXT"}, ["leak"]),
        ("use crate::{dtcm::{MacPipeTails as Hidden}}; fn leak(_: Hidden) {}", {"Hidden"}, ["leak"]),
        ("fn leak() -> *mut u32 { (DTCM_STATE_BASE + 0x1e28) as *mut u32 }", set(), ["leak"]),
        ("const ENTRY: usize = DTCM_STATE_BASE + 0x1e28; const NEXT: usize = ENTRY; fn leak() -> *mut u32 { NEXT as *mut u32 }", {"ENTRY", "NEXT"}, ["leak"]),
        ("fn leak() -> *mut u32 { DtcmAddress::from_offset(0x1d5c).cast_mut() }", set(), ["leak"]),
        ("const ENTRY: DtcmAddress = DtcmAddress::from_offset(0x1e28); const NEXT: DtcmAddress = ENTRY; fn leak() -> usize { NEXT.get() }", {"ENTRY", "NEXT"}, ["leak"]),
        ("fn leak() -> usize { DtcmAddress::from_offset_unchecked(0x1e28).get() }", set(), ["leak"]),
        ("use crate::{dtcm::{DtcmAddress as HiddenAddress}}; const ENTRY: HiddenAddress = HiddenAddress::from_offset(0x1d5c); const NEXT: HiddenAddress = ENTRY; fn leak() -> usize { NEXT.get() }", {"ENTRY", "NEXT"}, ["leak"]),
        ("fn reset() { unsafe { core::ptr::write_volatile(0x0400_1d5c as *mut u32, 0) } }", set(), ["reset"]),
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
    focused_test = named_function(source, "initialized_mac_pipe_tails_are_exact")
    failures.extend(
        f"src/dtcm.rs: missing exact compile-time physical mapping: {item}"
        for item in missing_inventory(compile_scope, COMPILE_TIME_PHYSICAL_INVENTORY)
    )
    if not exact_inventory_present(compile_scope, COMPILE_TIME_PHYSICAL_INVENTORY):
        failures.append("src/dtcm.rs: exact contiguous compile-time MAC pipe tail assertion inventory changed")
    if focused_test is None:
        failures.append("src/dtcm.rs: focused MAC pipe tails layout test is missing")
    else:
        failures.extend(
            f"src/dtcm.rs: focused test missing exact mapping/extent: {item}"
            for item in missing_inventory(focused_test, FOCUSED_TEST_INVENTORY)
        )
        if not exact_inventory_present(focused_test, FOCUSED_TEST_INVENTORY):
            failures.append("src/dtcm.rs: exact contiguous focused MAC pipe tail test inventory changed")
    check_exact_inventory_regression(source)
    for physical in PHYSICAL:
        spelling = f"0x{physical >> 16:04x}_{physical & 0xffff:04x}"
        if spelling not in source:
            failures.append(f"src/dtcm.rs: missing physical address/boundary {spelling}")
    declaration = re.search(r"#\[repr\(C, align\(4\)\)\] struct MacPipeTail \{[^}]*\} #\[repr\(C, align\(4\)\)\] struct MacPipeTails \{[^}]*\}", source)
    if declaration is None or normalized(declaration.group()) != normalized(" ".join(STRUCT)) or "derive" in declaration.group():
        failures.append("src/dtcm.rs: MacPipeTail/MacPipeTails must be the exact private non-derived inventory")
    if source.count("struct MacPipeTails") != 1 or "pre_mac_phy_command_state: OpaqueBytes<0x208>" in source:
        failures.append("src/dtcm.rs: removed opaque split or duplicate MAC pipe tails remain")
    for relative, exact in ADJACENT_DECLARATIONS.items():
        if exact not in (ROOT / relative).read_text():
            failures.append(f"{relative}: adjacent checker range changed from {exact}")

    paths = source_paths()
    rust = {path.relative_to(ROOT).as_posix(): mask_test_module(code_only(path.read_text(errors="replace")))
            for path in paths if path.suffix == ".rs"}
    for struct_text in STRUCT:
        rust["src/dtcm.rs"] = rust["src/dtcm.rs"].replace(struct_text, " " * len(struct_text))
    production = "\n".join(rust.values())
    views, declarations = family_aliases(production)
    for kind, name, _ in declarations:
        if name in views and name not in SANCTIONED_ROOTS:
            failures.append(f"additional MAC pipe tail {kind} alias is forbidden: {name}")
    view_pattern = re.compile(rf"\b(?:{'|'.join(sorted(map(re.escape, views)))})\b")
    if re.search(r"\bimpl(?:\s*<[^>]*>)?\s+[^\{]*MacPipeTail", production):
        failures.append("production impl for MacPipeTails is forbidden")
    for match in re.finditer(r"\b(?:const(?!\s+fn\b)|static)\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)[^;]*;", production):
        if view_pattern.search(match.group()) and match.group(1) not in SANCTIONED_ROOTS:
            failures.append(f"direct MAC pipe tail const/static alias is forbidden: {match.group(1)}")
    for match in re.finditer(r"\b(?:unsafe\s+)?(?:const\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*pipe_tail[A-Za-z0-9_]*)", production, re.I):
        if match.group(1) not in SANCTIONED_APIS:
            failures.append(f"named MAC pipe tail production API is forbidden: {match.group(1)}")
    export_body = named_function(rust["src/mac.rs"], "export_pipe_counters")
    if export_body is None:
        failures.append("src/mac.rs: sanctioned export_pipe_counters reader is missing")
    else:
        export_hash = hashlib.sha256(normalized(export_body).encode()).hexdigest()
        if export_hash != SANCTIONED_EXPORT_BODY_SHA256:
            failures.append(
                "src/mac.rs: export_pipe_counters body changed outside the exact reviewed reader inventory"
            )
        total_calls = sum(len(re.findall(rf"\b{re.escape(api)}\b", export_body)) for api in SANCTIONED_READER_PATTERNS)
        expected_calls = sum(count for _, count in SANCTIONED_READER_PATTERNS.values())
        if total_calls != expected_calls:
            failures.append(
                f"src/mac.rs: MAC pipe tail accessor count changed from {expected_calls} to {total_calls}"
            )
        reader_stripped = export_body
        for api, (reader, count) in SANCTIONED_READER_PATTERNS.items():
            pattern = re.compile(
                rf"\b{reader}\s*\(\s*crate::dtcm::{re.escape(api)}\s*\(\s*index\s*\)\s*\.\s*get\s*\(\s*\)\s*\)"
            )
            actual = len(pattern.findall(export_body))
            if actual != count:
                failures.append(
                    f"src/mac.rs: {api} must occur exactly {count} time(s) as a {reader} source, found {actual}"
                )
            reader_stripped = pattern.sub(" ", reader_stripped)
        if view_pattern.search(reader_stripped):
            failures.append(
                "src/mac.rs: export_pipe_counters contains an unreviewed MAC pipe tail root or API use"
            )

    forbidden_operation = re.compile(r"\*(?:const|mut)|&(?:mut\s+)?|\b(?:read|write)(?:_[A-Za-z0-9]+)?\s*\(|\b(?:value|init(?:ialize)?|reset|unchecked|generic_offset|slice|iter(?:ator)?)\b", re.I)

    def mask_sanctioned_bodies(relative: str, code: str) -> str:
        names: set[str] = set()
        if relative == "src/dtcm.rs":
            names = SANCTIONED_APIS
        elif relative == "src/mac.rs" and export_body is not None:
            names = {"export_pipe_counters"}
        masked = code
        for name in names:
            body = named_function(masked, name)
            if body is None:
                continue
            masked = masked.replace(
                body,
                "".join("\n" if char == "\n" else " " for char in body),
                1,
            )
        return masked

    def forbidden_line_numbers(relative: str, code: str) -> list[int]:
        masked = mask_sanctioned_bodies(relative, code)
        return [
            line_number
            for line_number, line in enumerate(masked.splitlines(), 1)
            if view_pattern.search(line) and forbidden_operation.search(line)
        ]

    regression_fixtures = (
        "macro_rules! leak { () => { write_u32(MAC_PIPE_TAILS.get(), 0) } }\nfn invoke() { leak!() }",
        "macro_rules! leak { () => { write_u32(mac_pipe_tail_frame_count_unchecked(0).get(), 0) } }\nfn invoke() { leak!() }",
    )
    if any(not forbidden_line_numbers("src/dtcm.rs", fixture) for fixture in regression_fixtures):
        raise SystemExit("checker self-test failed: macro-hidden MAC pipe tail write was accepted")
    split_macro_fixture = (
        "macro_rules! tails { () => { MAC_PIPE_TAILS } }\n"
        "fn export_pipe_counters() { read_u32(tails!().get()) }"
    )
    if not any(view_pattern.search(definition) for definition in macro_definitions(split_macro_fixture)):
        raise SystemExit("checker self-test failed: split macro MAC pipe tail alias was accepted")

    for relative, code in rust.items():
        for definition in macro_definitions(code):
            if view_pattern.search(definition):
                failures.append(f"{relative}: macros over MAC pipe tail roots or APIs are forbidden")
        sanctioned_functions = SANCTIONED_CONSUMER_FUNCTIONS.get(relative, set())
        for function in functions_consuming_family(code, views):
            if function not in sanctioned_functions:
                failures.append(f"{relative}: production function API over MAC pipe tail storage is forbidden: {function}")
        for line_number in forbidden_line_numbers(relative, code):
            failures.append(f"{relative}:{line_number}: pointer/reference/read/write/value/init/offset/slice API is forbidden")
        for line_number, line in enumerate(code.splitlines(), 1):
            for value in owned_literals(line):
                if value in OFFSETS and relative != "src/dtcm.rs":
                    failures.append(f"{relative}:{line_number}: raw MAC pipe tail DTCM offset 0x{value:x} is forbidden")

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
                failures.append(f"{relative}:{line}: MAC pipe tail physical literal {match.group()} is outside reviewed owners")
    if failures: raise SystemExit("\n".join(failures))
    print(f"INITIALIZED MAC PIPE TAILS SOURCE DRIFT-EVIDENCE GATE PASSED files={len(paths)}")


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
        raise SystemExit(f"INITIALIZED MAC PIPE TAILS LINKED DRIFT GATE FAILED\nliterals={dict(literals)!r}\nxrefs={dict(xrefs)!r}")
    residual_literals, residual_xrefs = literals - ALLOWED_LINKED_LITERALS, xrefs - ALLOWED_DECODED_XREFS
    print(f"INITIALIZED MAC PIPE TAILS LINKED DRIFT-EVIDENCE GATE PASSED allowed_literals={sum(ALLOWED_LINKED_LITERALS.values())} residual_literals={sum(residual_literals.values())} residual_xrefs={sum(residual_xrefs.values())}")


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
        raise SystemExit(f"initialized-MAC-pipe-tails drift gate failed: {error}")
