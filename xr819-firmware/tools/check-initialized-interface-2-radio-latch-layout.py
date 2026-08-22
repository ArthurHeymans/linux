#!/usr/bin/env python3
"""Source/linked drift evidence for exactly [0x0400124F, 0x04001250).

Empty aligned linked-literal and decoded PC-relative-xref multisets are drift
 evidence only, explicitly not writer closure.
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
RANGE = (0x0400124F, 0x04001250)
OFFSETS = {0x124f}
LITERAL = re.compile(r"0x[0-9a-fA-F_]+")
SOURCE_EXTENSIONS = {".rs", ".py", ".sh", ".c", ".h", ".S", ".s", ".asm", ".inc", ".ld", ".x", ".toml", ".mk"}
OWNER_FILES = {
    "src/dtcm.rs",
    "tools/check-initialized-interface-2-radio-latch-layout.py",
    "tools/check-initialized-multi-vif-beacon-timer-layout.py",
    "tools/check-initialized-measurement-dwell-timer-layout.py",
    "tools/check-initialized-measurement-control-reset-words-layout.py",
}
STRUCT = "#[repr(C, align(4))] struct InitializedMultiVifBeaconTimerTail { opaque_00: OpaqueBytes<0x0f>, interface_2_radio_latch: SharedU8, timer: TimerEntry, opaque_24: OpaqueBytes<0x18>, measurement_dwell_timer: TimerEntry, dtim_capture_latch: SharedU8, opaque_51: OpaqueBytes<0x03>, measurement_control_word_0: SharedU32, measurement_control_word_1: SharedU32, measurement_control_word_2: SharedU32 }"
IMAGE = "phy_watchdog_counter: PhyWatchdogCounter, multi_vif_beacon_timer_tail: InitializedMultiVifBeaconTimerTail,"
ASSERTIONS = (
    "assert_type_layout!(SharedU8, 0x01, 1);",
    "assert_type_layout!(InitializedMultiVifBeaconTimerTail, 0x60, 4);",
    "assert!(core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, opaque_00) == 0);",
    "assert!(core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, interface_2_radio_latch) == 0x0f);",
    "assert!(core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, timer) == 0x10);",
    "assert!(core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, opaque_24) == 0x24);",
    "assert!(core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_dwell_timer) == 0x3c);",
    "assert!(core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, dtim_capture_latch) == 0x50);",
    "assert!(core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, opaque_51) == 0x51);",
    "assert!(core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_0) == 0x54);",
    "assert!(core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_1) == 0x58);",
    "assert!(core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, measurement_control_word_2) == 0x5c);",
    "offset_of!(InitializedVendorImage, multi_vif_beacon_timer_tail) == 0x1240",
    "offset_of!(InitializedVendorImage, ampdu_counters) == 0x12a0",
    "assert_type_layout!(InitializedVendorImage, 0x2078, 4);",
    "assert_type_layout!(DtcmLayout, DTCM_STATE_SIZE, 4);",
    "assert_type_layout!(SharedDtcmState, DTCM_STATE_SIZE, 4);",
)
TEST_ITEMS = (
    "fn initialized_interface_2_radio_latch_address_is_exact()",
    "let tail = DTCM_STATE_BASE + core::mem::offset_of!(InitializedVendorImage, multi_vif_beacon_timer_tail)",
    "let latch = tail + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, interface_2_radio_latch)",
    "let timer = tail + core::mem::offset_of!(InitializedMultiVifBeaconTimerTail, timer)",
    "tail, 0x0400_1240",
    "size_of::<OpaqueBytes<0x0f>>(), 0x0400_124f",
    "latch, 0x0400_124f",
    "latch + core::mem::size_of::<SharedU8>(), 0x0400_1250",
    "timer, 0x0400_1250",
    "timer + core::mem::size_of::<TimerEntry>(), 0x0400_1264",
    "[0x00, 0x0f, 0x10, 0x24, 0x3c, 0x50, 0x51, 0x54, 0x58, 0x5c]",
    "size_of::<SharedU8>(), 0x01",
    "align_of::<SharedU8>(), 1",
    "size_of::<InitializedMultiVifBeaconTimerTail>(), 0x60",
    "align_of::<InitializedMultiVifBeaconTimerTail>(), 4",
    "offset_of!(InitializedVendorImage, multi_vif_beacon_timer_tail), 0x1240",
    "offset_of!(InitializedVendorImage, ampdu_counters), 0x12a0",
    "size_of::<InitializedVendorImage>(), 0x2078",
    "align_of::<InitializedVendorImage>(), 4",
    "size_of::<DtcmLayout>(), DTCM_STATE_SIZE",
    "align_of::<DtcmLayout>(), 4",
    "size_of::<SharedDtcmState>(), DTCM_STATE_SIZE",
    "align_of::<SharedDtcmState>(), 4",
)
ALLOWED_LINKED_LITERALS: collections.Counter[int] = collections.Counter()
ALLOWED_DECODED_XREFS: collections.Counter[tuple[str, int]] = collections.Counter()


def normalized(text: str) -> str:
    return " ".join(text.split())


def code_only(source: str, hash_comments: bool = False, rust: bool = False) -> str:
    out: list[str] = []
    i, state, depth, quote = 0, "code", 0, ""
    while i < len(source):
        if state == "code":
            lifetime = rust and source[i] == "'" and re.match(r"'[A-Za-z_]\w*(?!')", source[i:])
            if source.startswith("//", i): out.extend("  "); i += 2; state = "line"
            elif source.startswith("/*", i): out.extend("  "); i += 2; state = "block"; depth = 1
            elif hash_comments and source[i] == "#" and not source.startswith("#[", i): out.append(" "); i += 1; state = "line"
            elif lifetime: out.append(source[i]); i += 1
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
    declarations: list[tuple[str, str]] = []
    for pattern in (r"\b(?:const|static)\s+(?:mut\s+)?([A-Za-z_]\w*)\s*:[^=;]+\s*=\s*([^;]*);", r"\btype\s+([A-Za-z_]\w*)\s*=\s*([^;]*);"):
        declarations.extend(re.findall(pattern, code))
    imports = re.findall(r"\b(?:interface_2_radio_latch|InitializedMultiVifBeaconTimerTail)\s+as\s+([A-Za-z_]\w*)", code)
    aliases = {"InitializedMultiVifBeaconTimerTail", "interface_2_radio_latch", *imports}
    aliases |= {name for name, init in declarations if any(int(x.replace("_", ""), 16) in OFFSETS for x in LITERAL.findall(init))}
    while True:
        expanded = aliases | {name for name, init in declarations if set(re.findall(r"\b[A-Za-z_]\w*\b", init)) & aliases}
        if expanded == aliases: break
        aliases = expanded
    pattern = re.compile(rf"\b(?:{'|'.join(map(re.escape, sorted(aliases)))})\b")
    consumers: list[str] = []
    for match in re.finditer(r"\b(?:unsafe\s+)?(?:const\s+)?fn\s+([A-Za-z_]\w*)[^\{;]*\{", code):
        depth, i = 1, match.end()
        while i < len(code) and depth:
            depth += (code[i] == "{") - (code[i] == "}"); i += 1
        body = code[match.start():i]
        if pattern.search(body) or any(int(x.replace("_", ""), 16) in OFFSETS or RANGE[0] <= int(x.replace("_", ""), 16) < RANGE[1] for x in LITERAL.findall(body)):
            consumers.append(match.group(1))
    return aliases, consumers


def self_tests() -> None:
    fixtures = (
        "const ROOT: usize = 0x124f; const NEXT: usize = ROOT; fn leak() -> *mut u8 { NEXT as *mut u8 }",
        "static ROOT: usize = 0x124f; fn read() -> u8 { ROOT as u8 }",
        "type Hidden = InitializedMultiVifBeaconTimerTail; fn borrow(_: &Hidden) {}",
        "use crate::dtcm::interface_2_radio_latch as HIDDEN; fn write() { core::ptr::write_volatile(HIDDEN as *mut u8, 1) }",
        "fn reset() { unsafe { core::ptr::write_volatile(0x0400_124f as *mut u8, 0) } }",
        "fn init(index: usize) -> usize { 0x124fusize.wrapping_add(index) }",
        "fn slice(_: InitializedMultiVifBeaconTimerTail) -> core::iter::Empty<u8> { core::iter::empty() }",
    )
    for fixture in fixtures:
        if not aliases_and_consumers(code_only(fixture, rust=True))[1]:
            raise SystemExit("checker self-test failed: alias/API rejection regressed")
    if not {"ROOT", "NEXT"} <= aliases_and_consumers(fixtures[0])[0]:
        raise SystemExit("checker self-test failed: transitive const rejection regressed")


def source_paths() -> list[Path]:
    return sorted(path for path in ROOT.rglob("*") if path.is_file() and path.suffix in SOURCE_EXTENSIONS and "target" not in path.parts and ".git" not in path.parts)


def check_source() -> None:
    source = (ROOT / "src/dtcm.rs").read_text(); compact = normalized(source); failures: list[str] = []
    for item in (STRUCT, IMAGE, *ASSERTIONS, *TEST_ITEMS):
        if normalized(item) not in compact: failures.append(f"src/dtcm.rs: missing exact inventory: {item}")
    declaration = re.search(r"(?:#\[[^\n]*\]\s*)*#\[repr\(C, align\(4\)\)\]\s*struct\s+InitializedMultiVifBeaconTimerTail\s*\{[^}]*\}", source)
    if declaration is None or normalized(declaration.group()) != normalized(STRUCT) or "derive" in declaration.group():
        failures.append("InitializedMultiVifBeaconTimerTail must be the exact private non-derived inventory")
    if source.count("struct InitializedMultiVifBeaconTimerTail") != 1 or source.count("interface_2_radio_latch: SharedU8") != 1 or "INTERFACE_2_RADIO_LATCH" in source:
        failures.append("old, duplicate, or address-constant latch inventory is forbidden")
    test = named_function(source, "initialized_interface_2_radio_latch_address_is_exact")
    if test is None or any(normalized(item) not in normalized(test) for item in TEST_ITEMS): failures.append("focused exact-address test inventory changed")
    production = mask_tests(code_only(source, rust=True)).replace(STRUCT, " " * len(STRUCT))
    aliases, consumers = aliases_and_consumers(production)
    allowed_aliases = {"InitializedMultiVifBeaconTimerTail", "interface_2_radio_latch", "MEASUREMENT_DWELL_TIMER", "MULTI_VIF_BEACON_TIMER"}
    if aliases != allowed_aliases: failures.append(f"direct/transitive latch aliases are forbidden: {sorted(aliases - allowed_aliases)}")
    if consumers: failures.append(f"production latch operational consumers are forbidden: {consumers}")
    forbidden = re.compile(r"\b(?:interface_2_radio_latch|INTERFACE_2_RADIO_LATCH)[A-Za-z0-9_]*(?:ptr|pointer|ref|mut|value|read|write|reset|init|offset|unchecked|slice|iter|get|set)\b", re.I)
    for path in source_paths():
        relative = path.relative_to(ROOT).as_posix()
        code = mask_tests(code_only(path.read_text(errors="replace"), rust=True)) if path.suffix == ".rs" else code_only(path.read_text(errors="replace"), path.suffix in {".py", ".sh", ".toml"})
        if relative.endswith(".rs") and forbidden.search(code): failures.append(f"{relative}: forbidden latch API")
        if relative in OWNER_FILES: continue
        for match in LITERAL.finditer(code):
            value = int(match.group().replace("_", ""), 16)
            if RANGE[0] <= value < RANGE[1] or value in OFFSETS:
                failures.append(f"{relative}:{code.count(chr(10), 0, match.start()) + 1}: owned latch literal/offset outside reviewed owners")
        if relative.endswith(".rs") and re.search(r"\b(?:interface_2_radio_latch|INTERFACE_2_RADIO_LATCH)\b", code): failures.append(f"{relative}: latch alias or API outside owner")
    if failures: raise SystemExit("\n".join(failures))
    print(f"INITIALIZED INTERFACE-2 RADIO LATCH SOURCE DRIFT-EVIDENCE GATE PASSED files={len(source_paths())}")


def linked_literals(path: Path) -> collections.Counter[int]:
    with tempfile.NamedTemporaryFile() as output:
        subprocess.run(["llvm-objcopy", "--dump-section", f".text={output.name}", str(path), "/dev/null"], check=True)
        data = Path(output.name).read_bytes()
    return collections.Counter(value for offset in range(0, len(data) - 3, 4) if RANGE[0] <= (value := struct.unpack_from("<I", data, offset)[0]) < RANGE[1])


def decoded_xrefs(path: Path) -> collections.Counter[tuple[str, int]]:
    text = subprocess.run(["llvm-objdump", "-d", "--print-imm-hex", str(path)], check=True, text=True, stdout=subprocess.PIPE).stdout
    words = {int(a, 16): int(v, 16) for a, v in re.findall(r"^\s*([0-9a-fA-F]+):.*?\.word\s+0x([0-9a-fA-F]+)\s*$", text, re.M)}
    symbol = "<outside-symbol>"; result: collections.Counter[tuple[str, int]] = collections.Counter()
    for line in text.splitlines():
        if header := re.match(r"^[0-9a-fA-F]+ <(.+)>:$", line): symbol = header.group(1); continue
        if ".word" in line: continue
        if ref := re.search(r"@\s+0x([0-9a-fA-F]+)(?:\s|<|$)", line):
            value = words.get(int(ref.group(1), 16))
            if value is not None and RANGE[0] <= value < RANGE[1]: result[(symbol, value)] += 1
    return result


def main() -> None:
    self_tests(); parser = argparse.ArgumentParser(); parser.add_argument("elf", nargs="?", type=Path); parser.add_argument("--dump", action="store_true"); args = parser.parse_args(); check_source()
    if args.elf:
        literals, xrefs = linked_literals(args.elf), decoded_xrefs(args.elf)
        if args.dump: print(f"ALLOWED_LINKED_LITERALS={dict(literals)!r}\nALLOWED_DECODED_XREFS={dict(xrefs)!r}"); return
        if literals != ALLOWED_LINKED_LITERALS or xrefs != ALLOWED_DECODED_XREFS: raise SystemExit(f"INITIALIZED INTERFACE-2 RADIO LATCH LINKED DRIFT GATE FAILED\nliterals={dict(literals)!r}\nxrefs={dict(xrefs)!r}")
        print("INITIALIZED INTERFACE-2 RADIO LATCH LINKED DRIFT-EVIDENCE GATE PASSED literals=0 decoded_xrefs=0 not_writer_closure=1")


if __name__ == "__main__":
    try: main()
    except (OSError, subprocess.CalledProcessError) as error: raise SystemExit(f"initialized-interface-2-radio-latch drift gate failed: {error}")
