#!/usr/bin/env python3
"""Source/linked drift evidence for [0x04001572, 0x04001574).

The exact-width Rust writer is routed through the field-derived address owned by
``src/dtcm.rs``. Its aligned linked-literal and decoded PC-relative-xref evidence
remains pinned separately from the empty residual multisets. Empty residuals are
drift evidence only, never complete writer closure: vendor/IRQ/FIQ/indirect
mutation remains open.
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
RANGE = (0x04001572, 0x04001574)
OFFSETS = range(0x1572, 0x1574)
OVERLAPPING_BYTE_ROOTS = {0x04001570, 0x1570}
LITERAL = re.compile(r"0x[0-9a-fA-F_]+")
SOURCE_EXTENSIONS = {".rs", ".py", ".sh", ".c", ".h", ".S", ".s", ".asm", ".inc", ".ld", ".x", ".toml", ".mk"}
OWNER_FILES = {"src/dtcm.rs", "tools/check-initialized-pre-host-pas-radio-stop-word-layout.py"}
STRUCT = "#[repr(C, align(4))] struct PreHostPasRingObserved { opaque_00: OpaqueBytes<0x06>, radio_stop_word_02: SharedU16, opaque_08: OpaqueBytes<0x04> }"
IMAGE = "pre_host_pas_ring: PreHostPasRingObserved,"
ADDRESS = "pub(crate) const RADIO_STOP_WORD_02: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, pre_host_pas_ring) + core::mem::offset_of!(PreHostPasRingObserved, radio_stop_word_02));"
ASSERTIONS = (
    "assert_type_layout!(SharedU16, 0x02, 2);",
    "assert_type_layout!(PreHostPasRingObserved, 0x0c, 4);",
    "assert!(core::mem::offset_of!(PreHostPasRingObserved, opaque_00) == 0x00);",
    "assert!(core::mem::offset_of!(PreHostPasRingObserved, radio_stop_word_02) == 0x06);",
    "assert!(core::mem::offset_of!(PreHostPasRingObserved, opaque_08) == 0x08);",
    "assert!(core::mem::offset_of!(InitializedVendorImage, pre_host_pas_ring) == 0x156c);",
    "assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, pre_host_pas_ring) + core::mem::offset_of!(PreHostPasRingObserved, radio_stop_word_02) == 0x0400_1572);",
    "assert!(DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, pre_host_pas_ring) + core::mem::offset_of!(PreHostPasRingObserved, radio_stop_word_02) + core::mem::size_of::<SharedU16>() == 0x0400_1574);",
    "assert!(core::mem::offset_of!(InitializedVendorImage, host_pas_ring) == 0x1578);",
    "assert_type_layout!(InitializedVendorImage, 0x2078, 4);",
    "assert_type_layout!(DtcmLayout, DTCM_STATE_SIZE, 4);",
    "assert_type_layout!(SharedDtcmState, DTCM_STATE_SIZE, 4);",
)
TEST_ITEMS = (
    "fn initialized_pre_host_pas_radio_stop_word_address_is_exact()",
    "size_of::<SharedU16>(), 0x02",
    "align_of::<SharedU16>(), 2",
    "offset_of!(PreHostPasRingObserved, opaque_00), core::mem::offset_of!(PreHostPasRingObserved, radio_stop_word_02), core::mem::offset_of!(PreHostPasRingObserved, opaque_08)], [0x00, 0x06, 0x08]",
    "size_of::<PreHostPasRingObserved>(), 0x0c",
    "align_of::<PreHostPasRingObserved>(), 4",
    "observed, 0x0400_156c",
    "prefix + core::mem::size_of::<OpaqueBytes<0x06>>(), 0x0400_1572",
    "RADIO_STOP_WORD_02.get(), 0x0400_1572",
    "word + core::mem::size_of::<SharedU16>(), 0x0400_1574",
    "suffix, 0x0400_1574",
    "suffix + core::mem::size_of::<OpaqueBytes<0x04>>(), 0x0400_1578",
    "HOST_PAS_RING.get(), 0x0400_1578",
    "size_of::<InitializedVendorImage>(), 0x2078",
    "align_of::<InitializedVendorImage>(), 4",
    "size_of::<DtcmLayout>(), DTCM_STATE_SIZE",
    "align_of::<DtcmLayout>(), 4",
    "size_of::<SharedDtcmState>(), DTCM_STATE_SIZE",
    "align_of::<SharedDtcmState>(), 4",
)
TYPED_WRITER_LINKED_LITERALS = collections.Counter({0x04001572: 1})
TYPED_WRITER_DECODED_XREFS = collections.Counter({
    ("_RNvNtCsiHlLB2CErfM_14xr819_firmware4scan7service", 0x04001572): 1,
})
ALLOWED_LINKED_LITERALS: collections.Counter[int] = collections.Counter()
ALLOWED_DECODED_XREFS: collections.Counter[tuple[str, int]] = collections.Counter()


def normalized(text: str) -> str:
    return " ".join(text.split())


def code_only(source: str, hash_comments: bool = False, rust: bool = False) -> str:
    def rust_lifetime_at(offset: int) -> bool:
        if not rust or source[offset] != "'" or offset + 1 >= len(source) or not re.match(r"[A-Za-z_]", source[offset + 1]):
            return False
        identifier = re.match(r"[A-Za-z_][A-Za-z0-9_]*", source[offset + 1:])
        assert identifier is not None
        end = offset + 1 + identifier.end()
        return end >= len(source) or source[end] != "'"

    out: list[str] = []
    i = 0
    state = "code"
    depth = 0
    quote = ""
    while i < len(source):
        if state == "code":
            if source.startswith("//", i): out.extend("  "); i += 2; state = "line"
            elif source.startswith("/*", i): out.extend("  "); i += 2; state = "block"; depth = 1
            elif hash_comments and source[i] == "#" and not source.startswith("#[", i): out.append(" "); i += 1; state = "line"
            elif source[i] == "'" and rust_lifetime_at(i): out.append(source[i]); i += 1
            elif source[i] in "\"'": quote = source[i]; out.append(" "); i += 1; state = "string"
            else: out.append(source[i]); i += 1
        elif state == "line":
            out.append("\n" if source[i] == "\n" else " "); state = "code" if source[i] == "\n" else state; i += 1
        elif state == "block":
            if source.startswith("/*", i): out.extend("  "); i += 2; depth += 1
            elif source.startswith("*/", i): out.extend("  "); i += 2; depth -= 1; state = "code" if depth == 0 else state
            else: out.append("\n" if source[i] == "\n" else " "); i += 1
        elif source[i] == "\\" and i + 1 < len(source): out.extend("  "); i += 2
        elif source[i] == quote: out.append(" "); i += 1; state = "code"
        else: out.append("\n" if source[i] == "\n" else " "); i += 1
    return "".join(out)


def mask_tests(code: str) -> str:
    marker = re.search(r"#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]\s*mod\s+tests\s*\{", code)
    return code if marker is None else code[:marker.start()] + re.sub(r"[^\n]", " ", code[marker.start():])


def named_function(source: str, name: str) -> str | None:
    match = re.search(rf"\bfn\s+{re.escape(name)}\s*\([^)]*\)\s*\{{", source)
    if match is None: return None
    depth, i = 1, match.end()
    while i < len(source) and depth:
        depth += (source[i] == "{") - (source[i] == "}"); i += 1
    return source[match.start():i] if depth == 0 else None


def aliases_and_consumers(code: str) -> tuple[set[str], list[str]]:
    declarations = []
    for pattern in (r"\b(?:const|static)\s+(?:mut\s+)?([A-Za-z_]\w*)\s*:[^=;]+\s*=\s*([^;]*);", r"\btype\s+([A-Za-z_]\w*)\s*=\s*([^;]*);"):
        declarations.extend(re.findall(pattern, code))
    aliases = {"PreHostPasRingObserved", "radio_stop_word_02", "RADIO_STOP_WORD_02"}
    aliases |= {name for name, init in declarations if any(int(x.replace("_", ""), 16) in OFFSETS for x in LITERAL.findall(init))}
    while True:
        expanded = aliases | {name for name, init in declarations if set(re.findall(r"\b[A-Za-z_]\w*\b", init)) & aliases}
        if expanded == aliases: break
        aliases = expanded
    consumers = []
    pattern = re.compile(rf"\b(?:{'|'.join(map(re.escape, sorted(aliases)))})\b")
    for match in re.finditer(r"\b(?:unsafe\s+)?(?:const\s+)?fn\s+([A-Za-z_]\w*)[^\{;]*\{", code):
        depth, i = 1, match.end()
        while i < len(code) and depth:
            depth += (code[i] == "{") - (code[i] == "}"); i += 1
        body = code[match.start():i]
        if pattern.search(body) or any(int(x.replace("_", ""), 16) in OFFSETS or int(x.replace("_", ""), 16) in OVERLAPPING_BYTE_ROOTS for x in LITERAL.findall(body)): consumers.append(match.group(1))
    return aliases, consumers


def self_tests() -> None:
    fixtures = (
        "const ROOT: usize = 0x1572; const NEXT: usize = ROOT; fn leak() -> *mut u16 { NEXT as *mut u16 }",
        "type Hidden = PreHostPasRingObserved; fn read(_: &Hidden) -> u16 { 0 }",
        "fn leak_static() -> &'static PreHostPasRingObserved { loop {} }",
        "fn leak_named<'a>() -> &'a PreHostPasRingObserved { loop {} }",
        "const ROOT: DtcmAddress = RADIO_STOP_WORD_02; fn write() { unsafe { core::ptr::write_volatile(ROOT.get() as *mut u16, 0) } }",
        "fn table(index: usize) -> usize { 0x1572usize.wrapping_add(index) }",
        "fn bound_overlapping_bytes(index: usize) -> Option<usize> { (index < 4).then_some(0x1570 + index) }", 
        "fn iter(_: PreHostPasRingObserved) -> core::iter::Empty<u8> { core::iter::empty() }",
    )
    for fixture in fixtures:
        _, consumers = aliases_and_consumers(code_only(fixture, rust=True))
        if not consumers:
            raise SystemExit("checker self-test failed: alias/API rejection regressed")
    aliases, _ = aliases_and_consumers(fixtures[0])
    if not {"ROOT", "NEXT"} <= aliases:
        raise SystemExit("checker self-test failed: transitive alias rejection regressed")
    if normalized("#[derive(Copy)] " + STRUCT) == normalized(STRUCT):
        raise SystemExit("checker self-test failed: derive rejection regressed")


def source_paths() -> list[Path]:
    return sorted(path for path in ROOT.rglob("*") if path.is_file() and path.suffix in SOURCE_EXTENSIONS and "target" not in path.parts and ".git" not in path.parts)


def check_source() -> None:
    source = (ROOT / "src/dtcm.rs").read_text()
    compact = normalized(source)
    failures: list[str] = []
    for item in (STRUCT, IMAGE, ADDRESS, *ASSERTIONS, *TEST_ITEMS):
        if normalized(item) not in compact: failures.append(f"src/dtcm.rs: missing exact inventory: {item}")
    declaration = re.search(r"(?:#\[[^\n]*\]\s*)*#\[repr\(C, align\(4\)\)\]\s*struct\s+PreHostPasRingObserved\s*\{[^}]*\}", source)
    if declaration is None or normalized(declaration.group()) != normalized(STRUCT) or "derive" in declaration.group():
        failures.append("PreHostPasRingObserved must be the exact private non-derived inventory")
    if source.count("struct PreHostPasRingObserved") != 1 or source.count(IMAGE) != 1 or source.count(ADDRESS) != 1:
        failures.append("old or duplicate pre-host-PAS inventory/address constant remains")
    if "pre_host_pas_ring: OpaqueBytes<0x0c>" in source or re.search(r"struct\s+PreHostPasRingObserved[^}]*\[", source):
        failures.append("old opaque inventory or an array/table replacement is forbidden")
    test = named_function(source, "initialized_pre_host_pas_radio_stop_word_address_is_exact")
    if test is None or any(normalized(item) not in normalized(test) for item in TEST_ITEMS):
        failures.append("focused exact-address test inventory changed")
    production = mask_tests(code_only(source, rust=True)).replace(STRUCT, " " * len(STRUCT)).replace(ADDRESS, " " * len(ADDRESS))
    aliases, consumers = aliases_and_consumers(production)
    if len(aliases) != 3: failures.append(f"direct/transitive radio-stop aliases are forbidden: {sorted(aliases)}")
    if consumers: failures.append(f"production radio-stop operational consumers are forbidden: {consumers}")
    typed_write = "write_u16(crate::dtcm::RADIO_STOP_WORD_02.get(), 0);"
    mac_source = (ROOT / "src/mac.rs").read_text()
    if mac_source.count(typed_write) != 1:
        failures.append("src/mac.rs: exact typed 16-bit radio-stop write inventory changed")
    for path in source_paths():
        relative = path.relative_to(ROOT).as_posix()
        if relative in OWNER_FILES: continue
        code = mask_tests(code_only(path.read_text(errors="replace"), rust=True)) if path.suffix == ".rs" else code_only(path.read_text(errors="replace"), path.suffix in {".py", ".sh", ".toml"})
        if relative == "src/mac.rs":
            code = code.replace(typed_write, " " * len(typed_write))
        for match in LITERAL.finditer(code):
            value = int(match.group().replace("_", ""), 16)
            if RANGE[0] <= value < RANGE[1] or value in OFFSETS or value in OVERLAPPING_BYTE_ROOTS:
                failures.append(f"{relative}:{code.count(chr(10), 0, match.start()) + 1}: owned radio-stop literal/offset outside checker owners")
        if relative.endswith(".rs") and re.search(r"\b(?:RADIO_STOP_WORD_02|PreHostPasRingObserved|radio_stop_word_02)\b", code):
            failures.append(f"{relative}: direct/transitive radio-stop alias or API is forbidden")
    if failures: raise SystemExit("\n".join(failures))
    print(f"INITIALIZED PRE-HOST-PAS RADIO-STOP WORD SOURCE DRIFT-EVIDENCE GATE PASSED files={len(source_paths())}")


def linked_literals(path: Path) -> collections.Counter[int]:
    with tempfile.NamedTemporaryFile() as output:
        subprocess.run(["llvm-objcopy", "--dump-section", f".text={output.name}", str(path), "/dev/null"], check=True)
        data = Path(output.name).read_bytes()
    return collections.Counter(value for offset in range(0, len(data) - 3, 4) if RANGE[0] <= (value := struct.unpack_from("<I", data, offset)[0]) < RANGE[1])


def decoded_xrefs(path: Path) -> collections.Counter[tuple[str, int]]:
    text = subprocess.run(["llvm-objdump", "-d", "--print-imm-hex", str(path)], check=True, text=True, stdout=subprocess.PIPE).stdout
    words = {int(a, 16): int(v, 16) for a, v in re.findall(r"^\s*([0-9a-fA-F]+):.*?\.word\s+0x([0-9a-fA-F]+)\s*$", text, re.M)}
    symbol = "<outside-symbol>"
    result: collections.Counter[tuple[str, int]] = collections.Counter()
    for line in text.splitlines():
        if header := re.match(r"^[0-9a-fA-F]+ <(.+)>:$", line): symbol = header.group(1); continue
        if ".word" in line: continue
        if ref := re.search(r"@\s+0x([0-9a-fA-F]+)(?:\s|<|$)", line):
            value = words.get(int(ref.group(1), 16))
            if value is not None and RANGE[0] <= value < RANGE[1]: result[(symbol, value)] += 1
    return result


def main() -> None:
    self_tests()
    parser = argparse.ArgumentParser()
    parser.add_argument("elf", nargs="?", type=Path)
    parser.add_argument("--dump", action="store_true")
    args = parser.parse_args()
    check_source()
    if args.elf:
        actual_literals, actual_xrefs = linked_literals(args.elf), decoded_xrefs(args.elf)
        literals = actual_literals - TYPED_WRITER_LINKED_LITERALS
        xrefs = actual_xrefs - TYPED_WRITER_DECODED_XREFS
        retained_exact = actual_literals == TYPED_WRITER_LINKED_LITERALS and actual_xrefs == TYPED_WRITER_DECODED_XREFS
        if args.dump: print(f"ALLOWED_LINKED_LITERALS={dict(literals)!r}\nALLOWED_DECODED_XREFS={dict(xrefs)!r}"); return
        if not retained_exact or literals != ALLOWED_LINKED_LITERALS or xrefs != ALLOWED_DECODED_XREFS:
            raise SystemExit(f"INITIALIZED PRE-HOST-PAS RADIO-STOP WORD LINKED DRIFT GATE FAILED\nactual_literals={dict(actual_literals)!r}\nactual_xrefs={dict(actual_xrefs)!r}\nresidual_literals={dict(literals)!r}\nresidual_xrefs={dict(xrefs)!r}")
        print("INITIALIZED PRE-HOST-PAS RADIO-STOP WORD LINKED DRIFT-EVIDENCE GATE PASSED retained_writer=1 residual_literals=0 residual_decoded_xrefs=0")


if __name__ == "__main__":
    try: main()
    except (OSError, subprocess.CalledProcessError) as error: raise SystemExit(f"initialized-pre-host-pas-radio-stop-word drift gate failed: {error}")
