#!/usr/bin/env python3
"""Source/linked drift evidence for [0x040012cc, 0x0400140c).

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
RANGE = (0x040012CC, 0x0400140C)
OFFSETS = range(0x12CC, 0x140C)
ROOTS = tuple(0x040012CC + index * 0x20 for index in range(10))
LITERAL = re.compile(r"0x[0-9a-fA-F_]+")
SOURCE_EXTENSIONS = {
    ".rs", ".py", ".sh", ".c", ".h", ".hh", ".hpp", ".hxx", ".cc",
    ".cpp", ".cxx", ".s", ".S", ".asm", ".inc", ".ld", ".lds",
    ".ldh", ".x", ".toml", ".mk",
}
SOURCE_FILENAMES = {"Makefile", "Kconfig"}
OWNER_FILES = {
    "src/dtcm.rs",
    "tools/check-initialized-per-tid-telemetry-bank-layout.py",
}
ALLOWED_LINKED_LITERALS: collections.Counter[int] = collections.Counter()
ALLOWED_DECODED_XREFS: collections.Counter[tuple[str, int]] = collections.Counter()
FIELDS = tuple(f"table_{index:02d}" for index in range(10))
STRUCT = (
    "#[repr(C, align(4))] struct PerTidTelemetryBank { "
    + ", ".join(f"{field}: [SharedU32; 8]" for field in FIELDS)
    + " }"
)
REQUIRED = (
    STRUCT,
    "pre_per_tid_telemetry_bank: OpaqueBytes<0x04>, per_tid_telemetry_bank: PerTidTelemetryBank",
    "assert_type_layout!(SharedU32, 0x04, 4)",
    "assert_type_layout!(PerTidTelemetryBank, 0x140, 4)",
    "size_of::<[SharedU32; 8]>() == 0x20",
    "offset_of!(InitializedVendorImage, pre_per_tid_telemetry_bank) == 0x12c8",
    "offset_of!(InitializedVendorImage, per_tid_telemetry_bank) == 0x12cc",
    "offset_of!(InitializedVendorImage, ampdu_completion_control) == 0x140c",
    "assert_type_layout!(InitializedVendorImage, 0x2078, 4)",
    "assert_type_layout!(DtcmLayout, DTCM_STATE_SIZE, 4)",
    "assert_type_layout!(SharedDtcmState, DTCM_STATE_SIZE, 4)",
    "fn initialized_per_tid_telemetry_bank_addresses_are_exact()",
    "size_of::<PerTidTelemetryBank>(), 0x140",
    "align_of::<PerTidTelemetryBank>(), 4",
    "size_of::<[SharedU32; 8]>() / core::mem::size_of::<SharedU32>(), 8",
    "roots.windows(2)",
    "pre_per_tid_telemetry_bank), 0x0400_12c8",
    "size_of::<OpaqueBytes<0x04>>(), 4",
    "let bank = image + core::mem::offset_of!(InitializedVendorImage, per_tid_telemetry_bank)",
    "AMPDU_TELEMETRY_COUNTERS.get() + core::mem::size_of::<AmpduTelemetryCounters>(), 0x0400_12c8",
    "AMPDU_COMPLETION_CONTROL.get(), 0x0400_140c",
    "size_of::<InitializedVendorImage>(), 0x2078",
    "align_of::<InitializedVendorImage>(), 4",
    "size_of::<DtcmLayout>(), DTCM_STATE_SIZE",
    "align_of::<DtcmLayout>(), 4",
    "size_of::<SharedDtcmState>(), DTCM_STATE_SIZE",
    "align_of::<SharedDtcmState>(), 4",
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


def source_paths() -> list[Path]:
    return sorted(path for path in ROOT.rglob("*") if path.is_file()
                  and (path.suffix in SOURCE_EXTENSIONS or path.name in SOURCE_FILENAMES)
                  and "target" not in path.parts and ".git" not in path.parts)


def in_range(value: int) -> bool:
    return RANGE[0] <= value < RANGE[1]


def owned_literals(code: str) -> list[int]:
    values = [int(match.group().replace("_", ""), 16) for match in LITERAL.finditer(code)]
    physical = [value for value in values if in_range(value)]
    # Small integer literals in this interval also occur as unrelated masks and
    # register values. They identify this bank only in a DTCM-relative
    # expression; inspect the whole declaration/function so multiline pointer
    # arithmetic cannot evade the gate.
    relative_context = re.search(
        r"\bDTCM_STATE_BASE\b|"
        r"\b(?:[A-Za-z_][A-Za-z0-9_]*::)*from_offset(?:_unchecked)?\s*\(",
        code,
    )
    relative = [value for value in values if value in OFFSETS] if relative_context else []
    return physical + relative


def rust_use_aliases(code: str) -> list[tuple[str, str]]:
    aliases: list[tuple[str, str]] = []
    # Search the complete use tree rather than only its outermost group. This
    # deliberately tracks aliases conservatively through arbitrarily nested
    # groups such as `use crate::{dtcm::{View as Hidden}};`.
    for statement in re.finditer(r"\buse\s+([^;]*);", code):
        for imported, local in re.findall(
            r"\b([A-Za-z_][A-Za-z0-9_]*)\s+as\s+([A-Za-z_][A-Za-z0-9_]*)\b",
            statement.group(1),
        ):
            aliases.append((local, imported))
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
    views = {"PerTidTelemetryBank", "per_tid_telemetry_bank", *FIELDS,
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
        ("const ROOT: usize = table_00; const NEXT: usize = ROOT; fn leak() -> *mut u32 { NEXT as *mut u32 }", {"ROOT", "NEXT"}, ["leak"]),
        ("static ROOT: usize = 0x0400_12cc; static NEXT: usize = ROOT; fn leak() -> usize { NEXT }", {"ROOT", "NEXT"}, ["leak"]),
        ("type Hidden = PerTidTelemetryBank; type Again = Hidden; fn leak(_: Again) {}", {"Hidden", "Again"}, ["leak"]),
        ("use crate::dtcm::PerTidTelemetryBank as Hidden; const ROOT: usize = table_01; fn leak(_: Hidden) -> usize { ROOT }", {"Hidden", "ROOT"}, ["leak"]),
        ("use crate::dtcm::{PerTidTelemetryBank as Hidden, table_02 as Root}; const NEXT: usize = Root; fn leak(_: Hidden) -> usize { NEXT }", {"Hidden", "Root", "NEXT"}, ["leak"]),
        ("use crate::{dtcm::{PerTidTelemetryBank as Hidden}}; fn leak(_: Hidden) {}", {"Hidden"}, ["leak"]),
        ("fn leak() -> *mut u32 { (DTCM_STATE_BASE + 0x12d0) as *mut u32 }", set(), ["leak"]),
        ("const ENTRY: usize = DTCM_STATE_BASE + 0x12d0; const NEXT: usize = ENTRY; fn leak() -> *mut u32 { NEXT as *mut u32 }", {"ENTRY", "NEXT"}, ["leak"]),
        ("fn leak() -> *mut u32 { DtcmAddress::from_offset(0x12d0).cast_mut() }", set(), ["leak"]),
        ("const ENTRY: DtcmAddress = DtcmAddress::from_offset(0x12d0); const NEXT: DtcmAddress = ENTRY; fn leak() -> usize { NEXT.get() }", {"ENTRY", "NEXT"}, ["leak"]),
        ("fn leak() -> usize { DtcmAddress::from_offset_unchecked(0x12d0).get() }", set(), ["leak"]),
        ("use crate::{dtcm::{DtcmAddress as HiddenAddress}}; const ENTRY: HiddenAddress = HiddenAddress::from_offset(0x12d0); const NEXT: HiddenAddress = ENTRY; fn leak() -> usize { NEXT.get() }", {"ENTRY", "NEXT"}, ["leak"]),
        ("fn reset() { unsafe { core::ptr::write_volatile(0x0400_12cc as *mut u32, 0) } }", set(), ["reset"]),
    )
    for fixture, expected_aliases, expected_functions in fixtures:
        views, _ = family_aliases(fixture)
        if not expected_aliases <= views or functions_consuming_family(fixture, views) != expected_functions:
            raise SystemExit("checker self-test failed: alias or raw-pointer family API was accepted")


def check_source() -> None:
    source = (ROOT / "src/dtcm.rs").read_text()
    compact = normalized(source)
    failures = [f"src/dtcm.rs: missing exact inventory: {item}" for item in REQUIRED if normalized(item) not in compact]
    for field, offset in zip(FIELDS, range(0, 0x140, 0x20)):
        required = f"offset_of!(PerTidTelemetryBank, {field})"
        if compact.count(required) < 2 or f"0x{offset:03x}" not in compact:
            failures.append(f"src/dtcm.rs: missing pinned {field} offset 0x{offset:03x}")
    for root in (*ROOTS, RANGE[1]):
        spelling = f"0x{root >> 16:04x}_{root & 0xffff:04x}"
        if spelling not in source:
            failures.append(f"src/dtcm.rs: missing physical boundary {spelling}")
    declaration = re.search(r"(?:#\[[^\n]*\]\s*)*#\[repr\(C, align\(4\)\)\]\s*struct\s+PerTidTelemetryBank\s*\{[^}]*\}", source)
    if declaration is None or normalized(declaration.group()) != normalized(STRUCT) or "derive" in declaration.group():
        failures.append("src/dtcm.rs: PerTidTelemetryBank must be the exact private non-derived inventory")
    if source.count("struct PerTidTelemetryBank") != 1 or "OpaqueBytes<0x144>" in source:
        failures.append("src/dtcm.rs: old opaque extent or duplicate bank remains")
    telemetry_checker = (ROOT / "tools/check-ampdu-telemetry-layout.py").read_text()
    completion_checker = (ROOT / "tools/check-ampdu-completion-control-layout.py").read_text()
    if "(0x040012A0, 0x040012C8)" not in telemetry_checker:
        failures.append("adjacent A-MPDU telemetry checker no longer ends at 0x040012c8")
    if "(0x0400140C, 0x04001410)" not in completion_checker:
        failures.append("adjacent A-MPDU completion-control checker no longer begins at 0x0400140c")
    paths = source_paths()
    rust = {path.relative_to(ROOT).as_posix(): mask_test_module(code_only(path.read_text(errors="replace")))
            for path in paths if path.suffix == ".rs"}
    rust["src/dtcm.rs"] = rust["src/dtcm.rs"].replace(STRUCT, " " * len(STRUCT))
    production = "\n".join(rust.values())
    views, declarations = family_aliases(production)
    for kind, name, _ in declarations:
        if name in views:
            failures.append(f"additional per-TID telemetry {kind} alias is forbidden: {name}")
    view_pattern = re.compile(rf"\b(?:{'|'.join(sorted(map(re.escape, views)))})\b")
    for match in re.finditer(r"\bimpl(?:\s*<[^>]*>)?\s+[^\{]*PerTidTelemetryBank", production):
        failures.append("production impl for PerTidTelemetryBank is forbidden")
    for match in re.finditer(r"\b(?:const|static)\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)[^;]*;", production):
        if view_pattern.search(match.group()):
            failures.append(f"direct per-TID telemetry const/static alias is forbidden: {match.group(1)}")
    for match in re.finditer(r"\b(?:unsafe\s+)?(?:const\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*(?:per_tid|telemetry_bank)[A-Za-z0-9_]*)", production, re.I):
        failures.append(f"named per-TID telemetry production API is forbidden: {match.group(1)}")
    forbidden_operation = re.compile(r"\*(?:const|mut)|&(?:mut\s+)?|\b(?:read|write)(?:_volatile)?\s*\(|\b(?:value|init(?:ialize)?|reset|unchecked|generic_offset|slice|iter(?:ator)?)\b", re.I)
    for relative, code in rust.items():
        for function in functions_consuming_family(code, views):
            failures.append(f"{relative}: production function API over per-TID telemetry storage is forbidden: {function}")
        for line_number, line in enumerate(code.splitlines(), 1):
            if view_pattern.search(line) and forbidden_operation.search(line):
                failures.append(f"{relative}:{line_number}: pointer/reference/read/write/value/init/offset/slice API is forbidden")
            for value in owned_literals(line):
                if value in OFFSETS and relative != "src/dtcm.rs":
                    failures.append(f"{relative}:{line_number}: raw per-TID DTCM offset 0x{value:x} is forbidden")
    for path in paths:
        relative = path.relative_to(ROOT).as_posix()
        if relative in OWNER_FILES: continue
        code = rust[relative] if relative.endswith(".rs") else code_only(
            path.read_text(errors="replace"), path.suffix in {".py", ".sh", ".toml"}, path.suffix in {".py", ".sh", ".toml"})
        for match in LITERAL.finditer(code):
            value = int(match.group().replace("_", ""), 16)
            if in_range(value):
                line = code.count("\n", 0, match.start()) + 1
                failures.append(f"{relative}:{line}: per-TID telemetry physical literal {match.group()} is outside reviewed owners")
    if failures: raise SystemExit("\n".join(failures))
    print(f"INITIALIZED PER-TID TELEMETRY BANK SOURCE DRIFT-EVIDENCE GATE PASSED files={len(paths)}")


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
        raise SystemExit(f"INITIALIZED PER-TID TELEMETRY BANK LINKED DRIFT GATE FAILED\nliterals={dict(literals)!r}\nxrefs={dict(xrefs)!r}")
    print("INITIALIZED PER-TID TELEMETRY BANK LINKED DRIFT-EVIDENCE GATE PASSED literals=0 decoded_xrefs=0")


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
        raise SystemExit(f"initialized-per-TID-telemetry-bank drift gate failed: {error}")
