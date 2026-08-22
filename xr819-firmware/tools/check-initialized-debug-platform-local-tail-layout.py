#!/usr/bin/env python3
"""Source/linked drift evidence for [0x04001454, 0x0400145c).

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
RANGE = (0x04001454, 0x0400145C)
OFFSETS = range(0x1454, 0x145C)
LITERAL = re.compile(r"0x[0-9a-fA-F_]+")
SOURCE_EXTENSIONS = {
    ".rs", ".py", ".sh", ".c", ".h", ".hh", ".hpp", ".hxx", ".cc",
    ".cpp", ".cxx", ".s", ".S", ".asm", ".inc", ".ld", ".lds",
    ".ldh", ".x", ".toml", ".mk",
}
SOURCE_FILENAMES = {"Makefile", "Kconfig"}
OWNER_FILES = {
    "src/dtcm.rs",
    "tools/check-initialized-debug-platform-local-tail-layout.py",
    "tools/check-phy-descriptor-gain-records-layout.py",
}
ADJACENT_DECLARATIONS = {
    "tools/check-phy-descriptor-gain-records-layout.py": "PLATFORM_LOCAL_TAIL_RANGE = (0x04001454, 0x0400145C)",
}
ALLOWED_LINKED_LITERALS: collections.Counter[int] = collections.Counter()
ALLOWED_DECODED_XREFS: collections.Counter[tuple[str, int]] = collections.Counter()
STRUCT = "#[repr(C, align(4))] struct DebugPlatformLocalTail { platform_local_word_2c: SharedU32, platform_local_word_30: SharedU32 }"
IMAGE_FIELDS = "pre_phy_channel_threshold_descriptors: OpaqueBytes<0x14>, debug_platform_local_tail: DebugPlatformLocalTail"
REQUIRED = (
    STRUCT, IMAGE_FIELDS,
    "assert_type_layout!(SharedU32, 0x04, 4)",
    "assert_type_layout!(DebugPlatformLocalTail, 0x08, 4)",
    "offset_of!(DebugPlatformLocalTail, platform_local_word_2c) == 0x00",
    "offset_of!(DebugPlatformLocalTail, platform_local_word_30) == 0x04",
    "offset_of!(InitializedVendorImage, pre_phy_channel_threshold_descriptors) == 0x1440",
    "offset_of!(InitializedVendorImage, debug_platform_local_tail) == 0x1454",
    "offset_of!(InitializedVendorImage, phy_channel_threshold_descriptors) == 0x145c",
    "assert_type_layout!(InitializedVendorImage, 0x2078, 4)",
    "assert_type_layout!(DtcmLayout, DTCM_STATE_SIZE, 4)",
    "assert_type_layout!(SharedDtcmState, DTCM_STATE_SIZE, 4)",
    "fn initialized_debug_platform_local_tail_addresses_are_exact()",
    "size_of::<OpaqueBytes<0x14>>(), 0x14",
    "size_of::<DebugPlatformLocalTail>(), 0x08",
    "[core::mem::offset_of!(DebugPlatformLocalTail, platform_local_word_2c), core::mem::offset_of!(DebugPlatformLocalTail, platform_local_word_30)], [0x00, 0x04]",
    "fields, [0x0400_1454, 0x0400_1458]",
    "tail + core::mem::size_of::<DebugPlatformLocalTail>(), 0x0400_145c",
    "phy, 0x0400_145c",
    "size_of::<InitializedVendorImage>(), 0x2078",
    "align_of::<InitializedVendorImage>(), 4",
    "size_of::<DtcmLayout>(), DTCM_STATE_SIZE",
    "align_of::<DtcmLayout>(), 4",
    "size_of::<SharedDtcmState>(), DTCM_STATE_SIZE",
    "align_of::<SharedDtcmState>(), 4",
)
PHYSICAL = (0x04001440, 0x04001454, 0x04001458, 0x0400145C)
COMPILE_TIME_PHYSICAL_INVENTORY = (
    "assert_type_layout!(SharedU32, 0x04, 4);",
    "assert_type_layout!(DebugPlatformLocalTail, 0x08, 4);",
    "assert!(core::mem::offset_of!(DebugPlatformLocalTail, platform_local_word_2c) == 0x00);",
    "assert!(core::mem::offset_of!(DebugPlatformLocalTail, platform_local_word_30) == 0x04);",
    "assert!(core::mem::offset_of!(InitializedVendorImage, pre_phy_channel_threshold_descriptors) == 0x1440);",
    "assert!(core::mem::offset_of!(InitializedVendorImage, debug_platform_local_tail) == 0x1454);",
    "assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, debug_platform_local_tail) + core::mem::offset_of!(DebugPlatformLocalTail, platform_local_word_2c) == 0x0400_1454);",
    "assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, debug_platform_local_tail) + core::mem::offset_of!(DebugPlatformLocalTail, platform_local_word_30) == 0x0400_1458);",
    "assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, debug_platform_local_tail) + core::mem::size_of::<DebugPlatformLocalTail>() == 0x0400_145c);",
    "assert!(core::mem::offset_of!(InitializedVendorImage, phy_channel_threshold_descriptors) == 0x145c);",
    "assert_type_layout!(InitializedVendorImage, 0x2078, 4);",
    "assert_type_layout!(DtcmLayout, DTCM_STATE_SIZE, 4);",
    "assert_type_layout!(SharedDtcmState, DTCM_STATE_SIZE, 4);",
)
FOCUSED_TEST_INVENTORY = (
    "let image = DTCM_STATE_BASE;",
    "let opaque = image + core::mem::offset_of!(InitializedVendorImage, pre_phy_channel_threshold_descriptors);",
    "let tail = image + core::mem::offset_of!(InitializedVendorImage, debug_platform_local_tail);",
    "let fields = [tail + core::mem::offset_of!(DebugPlatformLocalTail, platform_local_word_2c), tail + core::mem::offset_of!(DebugPlatformLocalTail, platform_local_word_30)];",
    "let phy = image + core::mem::offset_of!(InitializedVendorImage, phy_channel_threshold_descriptors);",
    "assert_eq!(core::mem::size_of::<SharedU32>(), 0x04);",
    "assert_eq!(core::mem::align_of::<SharedU32>(), 4);",
    "assert_eq!(core::mem::size_of::<DebugPlatformLocalTail>(), 0x08);",
    "assert_eq!(core::mem::align_of::<DebugPlatformLocalTail>(), 4);",
    "assert_eq!([core::mem::offset_of!(DebugPlatformLocalTail, platform_local_word_2c), core::mem::offset_of!(DebugPlatformLocalTail, platform_local_word_30)], [0x00, 0x04]);",
    "assert_eq!(fields, [0x0400_1454, 0x0400_1458]);",
    "assert_eq!(fields[0] + core::mem::size_of::<SharedU32>(), fields[1]);",
    "assert_eq!(opaque, 0x0400_1440);",
    "assert_eq!(core::mem::size_of::<OpaqueBytes<0x14>>(), 0x14);",
    "assert_eq!(opaque + core::mem::size_of::<OpaqueBytes<0x14>>(), 0x0400_1454);",
    "assert_eq!(tail + core::mem::size_of::<DebugPlatformLocalTail>(), 0x0400_145c);",
    "assert_eq!(phy, 0x0400_145c);",
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
    sentinel = "__DEBUG_PLATFORM_LOCAL_TAIL_LAYOUT_SWAP__"
    return source.replace(left, sentinel, 1).replace(right, left, 1).replace(sentinel, right, 1)


def check_exact_inventory_regression(source: str) -> None:
    compile_scope = mask_test_module(source)
    test_scope = named_function(source, "initialized_debug_platform_local_tail_addresses_are_exact")
    if test_scope is None:
        raise SystemExit("checker self-test failed: focused test extraction failed")
    for label, scope, inventory in (("compile-time", compile_scope, COMPILE_TIME_PHYSICAL_INVENTORY), ("focused-test", test_scope, FOCUSED_TEST_INVENTORY)):
        if not exact_inventory_present(scope, inventory):
            raise SystemExit(f"checker self-test failed: canonical {label} inventory is absent")
        canonical = normalized(" ".join(inventory))
        for item in inventory:
            if exact_inventory_present(canonical.replace(normalized(item), "", 1), inventory):
                raise SystemExit(f"checker self-test failed: deleted {label} mapping was accepted: {item}")
    swapped = swapped_once(STRUCT, "platform_local_word_2c: SharedU32", "platform_local_word_30: SharedU32")
    if normalized(swapped) == normalized(STRUCT):
        raise SystemExit("checker self-test failed: swapped fields were accepted")
    wrong_address = normalized(test_scope).replace("[0x0400_1454, 0x0400_1458]", "[0x0400_1458, 0x0400_1454]", 1)
    if exact_inventory_present(wrong_address, FOCUSED_TEST_INVENTORY):
        raise SystemExit("checker self-test failed: swapped focused-test addresses were accepted")
    for bad in (STRUCT.replace(", platform_local_word_30: SharedU32", ""), "#[derive(Copy)] " + STRUCT, IMAGE_FIELDS.replace("0x14", "0x1c"), IMAGE_FIELDS + ", pre_phy_channel_threshold_descriptors: OpaqueBytes<0x1c>"):
        if normalized(bad) in {normalized(STRUCT), normalized(IMAGE_FIELDS)}:
            raise SystemExit("checker self-test failed: removed/derived/old/duplicate inventory was accepted")


def source_paths() -> list[Path]:
    return sorted(path for path in ROOT.rglob("*") if path.is_file()
                  and (path.suffix in SOURCE_EXTENSIONS or path.name in SOURCE_FILENAMES)
                  and "target" not in path.parts and ".git" not in path.parts)


def in_range(value: int) -> bool:
    return RANGE[0] <= value < RANGE[1]


def owned_literals(code: str) -> list[int]:
    values = [int(match.group().replace("_", ""), 16) for match in LITERAL.finditer(code)]
    physical = [value for value in values if in_range(value)]
    # Bare fixed offsets must seed alias tracking even when their DTCM address
    # constructor or raw-pointer consumer appears in a later declaration.
    relative = [value for value in values if value in OFFSETS]
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
    views = {"DebugPlatformLocalTail", "debug_platform_local_tail",
             *(name for _, name, initializer in declarations if owned_literals(initializer))}
    while True:
        aliases = {name for _, name, initializer in declarations
                   if set(re.findall(r"\b[A-Za-z_][A-Za-z0-9_]*\b", initializer)) & views}
        aliases |= {local for local, source in imported if source in views}
        expanded = views | aliases
        if expanded == views: return views, declarations
        views = expanded


def forbidden_family_declarations(
    declarations: list[tuple[str, str, str]], views: set[str]
) -> set[str]:
    return {name for _, name, _ in declarations if name in views}


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
        ("const ROOT: usize = debug_platform_local_tail; const NEXT: usize = ROOT; fn leak() -> *mut u32 { NEXT as *mut u32 }", {"ROOT", "NEXT"}, ["leak"]),
        ("static ROOT: usize = 0x0400_1454; static NEXT: usize = ROOT; fn leak() -> usize { NEXT }", {"ROOT", "NEXT"}, ["leak"]),
        ("const TAIL_OFFSET: usize = 0x1454;", {"TAIL_OFFSET"}, []),
        ("static TAIL_OFFSET: usize = 0x145b;", {"TAIL_OFFSET"}, []),
        ("const TAIL_OFFSET: usize = 0x1454; const NEXT: usize = TAIL_OFFSET; fn leak() -> usize { NEXT }", {"TAIL_OFFSET", "NEXT"}, ["leak"]),
        ("const TAIL_OFFSET: usize = 0x1454; const ADDRESS: DtcmAddress = DtcmAddress::from_offset(TAIL_OFFSET); fn leak() -> *mut u32 { ADDRESS.get() as *mut u32 }", {"TAIL_OFFSET", "ADDRESS"}, ["leak"]),
        ("type Hidden = DebugPlatformLocalTail; type Again = Hidden; fn leak(_: Again) {}", {"Hidden", "Again"}, ["leak"]),
        ("use crate::dtcm::DebugPlatformLocalTail as Hidden; const ROOT: usize = debug_platform_local_tail; fn leak(_: Hidden) -> usize { ROOT }", {"Hidden", "ROOT"}, ["leak"]),
        ("use crate::dtcm::{DebugPlatformLocalTail as Hidden, debug_platform_local_tail as Root}; const NEXT: usize = Root; fn leak(_: Hidden) -> usize { NEXT }", {"Hidden", "Root", "NEXT"}, ["leak"]),
        ("use crate::{dtcm::{DebugPlatformLocalTail as Hidden}}; fn leak(_: Hidden) {}", {"Hidden"}, ["leak"]),
        ("fn leak() -> *mut u32 { (DTCM_STATE_BASE + 0x1454) as *mut u32 }", set(), ["leak"]),
        ("const ENTRY: usize = DTCM_STATE_BASE + 0x1454; const NEXT: usize = ENTRY; fn leak() -> *mut u32 { NEXT as *mut u32 }", {"ENTRY", "NEXT"}, ["leak"]),
        ("fn leak() -> *mut u32 { DtcmAddress::from_offset(0x1454).cast_mut() }", set(), ["leak"]),
        ("const ENTRY: DtcmAddress = DtcmAddress::from_offset(0x1454); const NEXT: DtcmAddress = ENTRY; fn leak() -> usize { NEXT.get() }", {"ENTRY", "NEXT"}, ["leak"]),
        ("fn leak() -> usize { DtcmAddress::from_offset_unchecked(0x1454).get() }", set(), ["leak"]),
        ("use crate::{dtcm::{DtcmAddress as HiddenAddress}}; const ENTRY: HiddenAddress = HiddenAddress::from_offset(0x1454); const NEXT: HiddenAddress = ENTRY; fn leak() -> usize { NEXT.get() }", {"ENTRY", "NEXT"}, ["leak"]),
        ("fn reset() { unsafe { core::ptr::write_volatile(0x0400_1454 as *mut u32, 0) } }", set(), ["reset"]),
        ("fn read_tail(_: &DebugPlatformLocalTail) -> u32 { 0 }", set(), ["read_tail"]),
        ("fn write_tail(_: *mut DebugPlatformLocalTail) {}", set(), ["write_tail"]),
        ("fn clear_tail(_: DebugPlatformLocalTail) {}", set(), ["clear_tail"]),
        ("fn generic_offset<T>(_: DebugPlatformLocalTail) -> usize { 0 }", set(), ["generic_offset"]),
        ("fn tail_slice(_: DebugPlatformLocalTail) -> &'static [u8] { &[] }", set(), ["tail_slice"]),
        ("fn tail_iterator(_: DebugPlatformLocalTail) -> core::iter::Empty<u8> { core::iter::empty() }", set(), ["tail_iterator"]),
    )
    for fixture, expected_aliases, expected_functions in fixtures:
        views, _ = family_aliases(fixture)
        if not expected_aliases <= views or functions_consuming_family(fixture, views) != expected_functions:
            raise SystemExit("checker self-test failed: alias, constructor-offset, or raw-pointer family API was accepted")

    rejected_offsets = (
        ("const TAIL_OFFSET: usize = 0x1454;", {"TAIL_OFFSET"}),
        ("static TAIL_OFFSET: usize = 0x145b;", {"TAIL_OFFSET"}),
        ("const TAIL_OFFSET: usize = 0x1454; const NEXT: usize = TAIL_OFFSET;", {"TAIL_OFFSET", "NEXT"}),
        ("const TAIL_OFFSET: usize = 0x1454; const ADDRESS: DtcmAddress = DtcmAddress::from_offset(TAIL_OFFSET); fn raw() -> *mut u32 { ADDRESS.get() as *mut u32 }", {"TAIL_OFFSET", "ADDRESS"}),
    )
    for fixture, expected_rejections in rejected_offsets:
        views, declarations = family_aliases(fixture)
        if forbidden_family_declarations(declarations, views) != expected_rejections:
            raise SystemExit("checker self-test failed: bare or split fixed-offset alias was accepted")


def check_source() -> None:
    source = (ROOT / "src/dtcm.rs").read_text()
    compact = normalized(source)
    failures = [f"src/dtcm.rs: missing exact inventory: {item}" for item in REQUIRED if normalized(item) not in compact]
    compile_scope = mask_test_module(source)
    focused_test = named_function(source, "initialized_debug_platform_local_tail_addresses_are_exact")
    failures.extend(
        f"src/dtcm.rs: missing exact compile-time physical mapping: {item}"
        for item in missing_inventory(compile_scope, COMPILE_TIME_PHYSICAL_INVENTORY)
    )
    if not exact_inventory_present(compile_scope, COMPILE_TIME_PHYSICAL_INVENTORY):
        failures.append("src/dtcm.rs: exact contiguous initialized-debug-platform-local-tail compile-time assertion inventory changed")
    if focused_test is None:
        failures.append("src/dtcm.rs: focused debug platform-local tail layout test is missing")
    else:
        failures.extend(
            f"src/dtcm.rs: focused test missing exact mapping/extent: {item}"
            for item in missing_inventory(focused_test, FOCUSED_TEST_INVENTORY)
        )
        if not exact_inventory_present(focused_test, FOCUSED_TEST_INVENTORY):
            failures.append("src/dtcm.rs: exact contiguous focused initialized-debug-platform-local-tail test inventory changed")
    check_exact_inventory_regression(source)
    for physical in PHYSICAL:
        spelling = f"0x{physical >> 16:04x}_{physical & 0xffff:04x}"
        if spelling not in source:
            failures.append(f"src/dtcm.rs: missing physical address/boundary {spelling}")
    declaration = re.search(r"(?:#\[[^\n]*\]\s*)*#\[repr\(C, align\(4\)\)\]\s*struct\s+DebugPlatformLocalTail\s*\{[^}]*\}", source)
    if declaration is None or normalized(declaration.group()) != normalized(STRUCT) or "derive" in declaration.group():
        failures.append("src/dtcm.rs: DebugPlatformLocalTail must be the exact private non-derived inventory")
    if source.count("struct DebugPlatformLocalTail") != 1 or source.count(IMAGE_FIELDS) != 1 or "pre_phy_channel_threshold_descriptors: OpaqueBytes<0x1c>" in source:
        failures.append("src/dtcm.rs: duplicate record or old/duplicate opaque inventory remains")
    for relative, exact in ADJACENT_DECLARATIONS.items():
        if exact not in (ROOT / relative).read_text():
            failures.append(f"{relative}: adjacent checker range changed from {exact}")

    paths = source_paths()
    rust = {path.relative_to(ROOT).as_posix(): mask_test_module(code_only(path.read_text(errors="replace")))
            for path in paths if path.suffix == ".rs"}
    rust["src/dtcm.rs"] = rust["src/dtcm.rs"].replace(STRUCT, " " * len(STRUCT))
    production = "\n".join(rust.values())
    views, declarations = family_aliases(production)
    forbidden_declarations = forbidden_family_declarations(declarations, views)
    for kind, name, _ in declarations:
        if name in forbidden_declarations:
            failures.append(f"additional debug platform-local tail {kind} alias is forbidden: {name}")
    view_pattern = re.compile(rf"\b(?:{'|'.join(sorted(map(re.escape, views)))})\b")
    if re.search(r"\bimpl(?:\s*<[^>]*>)?\s+[^\{]*DebugPlatformLocalTail", production):
        failures.append("production impl for DebugPlatformLocalTail is forbidden")
    for match in re.finditer(r"\b(?:const|static)\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)[^;]*;", production):
        if view_pattern.search(match.group()):
            failures.append(f"direct debug platform-local tail const/static alias is forbidden: {match.group(1)}")
    for match in re.finditer(r"\b(?:unsafe\s+)?(?:const\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*(?:platform_local|debug_tail|word_2c|word_30)[A-Za-z0-9_]*)", production, re.I):
        failures.append(f"named debug platform-local tail production API is forbidden: {match.group(1)}")
    forbidden_operation = re.compile(r"\*(?:const|mut)|&(?:mut\s+)?|\b(?:read|write)(?:_volatile)?\s*\(|\b(?:value|init(?:ialize)?|reset|unchecked|generic_offset|slice|iter(?:ator)?)\b", re.I)
    for relative, code in rust.items():
        for function in functions_consuming_family(code, views):
            failures.append(f"{relative}: production function API over debug platform-local tail storage is forbidden: {function}")
        for line_number, line in enumerate(code.splitlines(), 1):
            if view_pattern.search(line) and forbidden_operation.search(line):
                failures.append(f"{relative}:{line_number}: pointer/reference/read/write/value/init/offset/slice API is forbidden")
            for value in owned_literals(line):
                if value in OFFSETS and relative != "src/dtcm.rs":
                    failures.append(f"{relative}:{line_number}: raw initialized-debug-platform-local-tail DTCM offset 0x{value:x} is forbidden")

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
                failures.append(f"{relative}:{line}: debug platform-local tail physical literal {match.group()} is outside reviewed owners")
    if failures: raise SystemExit("\n".join(failures))
    print(f"INITIALIZED DEBUG PLATFORM-LOCAL TAIL SOURCE DRIFT-EVIDENCE GATE PASSED files={len(paths)}")


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
        raise SystemExit(f"INITIALIZED DEBUG PLATFORM-LOCAL TAIL LINKED DRIFT GATE FAILED\nliterals={dict(literals)!r}\nxrefs={dict(xrefs)!r}")
    print("INITIALIZED DEBUG PLATFORM-LOCAL TAIL LINKED DRIFT-EVIDENCE GATE PASSED literals=0 decoded_xrefs=0")


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
        raise SystemExit(f"initialized-initialized-debug-platform-local-tail drift gate failed: {error}")
