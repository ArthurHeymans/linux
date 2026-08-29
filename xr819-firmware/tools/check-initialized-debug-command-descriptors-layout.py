#!/usr/bin/env python3
"""Source/linked drift evidence for initialized debug-command descriptors.

This checker covers exactly [0x04001164, 0x040011ac). It is evidence against
source, layout-inventory, aligned linked-literal, and decoded PC-relative-xref
drift; it is not writer closure or ownership proof. Register-computed,
indirect, generic HIF/debug, vendor, IRQ, and FIQ mutation remains outside its
proof.
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
RANGE = (0x04001164, 0x04001164 + 0x48)
LITERAL = re.compile(r"0x[0-9a-fA-F_]+")
SOURCE_EXTENSIONS = {
    ".rs", ".py", ".sh", ".c", ".h", ".hh", ".hpp", ".hxx", ".cc",
    ".cpp", ".cxx", ".s", ".S", ".asm", ".inc", ".ld", ".lds",
    ".ldh", ".x", ".toml", ".mk",
}
SOURCE_FILENAMES = {"Makefile", "Kconfig"}
OWNER_FILES = {"src/dtcm.rs", "tools/check-initialized-debug-command-descriptors-layout.py"}
ALLOWED_LINKED_LITERALS: collections.Counter[int] = collections.Counter()
ALLOWED_DECODED_XREFS: collections.Counter[tuple[str, int]] = collections.Counter()

REQUIRED_INVENTORY = (
    "#[repr(C, align(4))] struct DebugCommandDescriptor { command_name: SharedU32, help_text: SharedU32, handler: SharedU32 }",
    "#[repr(C, align(4))] struct InitializedDebugCommandDescriptors { records: [DebugCommandDescriptor; 6] }",
    "tx_aggregate_expiration_delta: TxAggregateExpirationDelta",
    "debug_command_descriptors: InitializedDebugCommandDescriptors",
    "hif_control: InitializedHifControl",
    "pub(crate) const DEBUG_COMMAND_DESCRIPTORS: DtcmAddress = DtcmAddress::from_offset(core::mem::offset_of!(InitializedVendorImage, debug_command_descriptors));",
    "pub(crate) const fn debug_command_descriptor(index: usize) -> Option<DtcmAddress>",
    "pub(crate) const fn debug_command_name(index: usize) -> Option<DtcmAddress>",
    "pub(crate) const fn debug_command_help(index: usize) -> Option<DtcmAddress>",
    "pub(crate) const fn debug_command_handler(index: usize) -> Option<DtcmAddress>",
    "if index < 6",
    "index * core::mem::size_of::<DebugCommandDescriptor>()",
    "offset_of!(InitializedDebugCommandDescriptors, records)",
    "offset_of!(DebugCommandDescriptor, command_name)",
    "offset_of!(DebugCommandDescriptor, help_text)",
    "offset_of!(DebugCommandDescriptor, handler)",
    "assert_type_layout!(DebugCommandDescriptor, 0x0c, 4)",
    "offset_of!(DebugCommandDescriptor, command_name) == 0x00",
    "offset_of!(DebugCommandDescriptor, help_text) == 0x04",
    "offset_of!(DebugCommandDescriptor, handler) == 0x08",
    "assert_type_layout!(InitializedDebugCommandDescriptors, 0x48, 4)",
    "offset_of!(InitializedDebugCommandDescriptors, records) == 0x00",
    "size_of::<[DebugCommandDescriptor; 6]>() == 0x48",
    "offset_of!(InitializedVendorImage, tx_aggregate_expiration_delta) == 0x1160",
    "offset_of!(InitializedVendorImage, debug_command_descriptors) == 0x1164",
    "offset_of!(InitializedVendorImage, hif_control) == 0x11ac",
    "assert_type_layout!(InitializedVendorImage, 0x2078, 4)",
    "assert_type_layout!(DtcmLayout, DTCM_STATE_SIZE, 4)",
    "assert_type_layout!(SharedDtcmState, DTCM_STATE_SIZE, 4)",
    "debug_command_descriptor(6).is_none()",
    "debug_command_name(6).is_none()",
    "debug_command_help(6).is_none()",
    "debug_command_handler(6).is_none()",
    "DEBUG_COMMAND_DESCRIPTORS.get() + core::mem::size_of::<InitializedDebugCommandDescriptors>()",
    "INITIALIZED_HIF_CONTROL.get()",
)
FOCUSED_TEST_NAME = "initialized_debug_command_descriptor_addresses_are_exact"
REQUIRED_FOCUSED_TEST_INVENTORY = (
    "assert_eq!(TX_AGGREGATE_EXPIRATION_DELTA.get() + core::mem::size_of::<TxAggregateExpirationDelta>(), 0x0400_1164)",
    "assert_eq!(DEBUG_COMMAND_DESCRIPTORS.get(), 0x0400_1164)",
    "assert_eq!(core::mem::size_of::<DebugCommandDescriptor>(), 0x0c)",
    "assert_eq!(core::mem::align_of::<DebugCommandDescriptor>(), 4)",
    "[core::mem::offset_of!(DebugCommandDescriptor, command_name), core::mem::offset_of!(DebugCommandDescriptor, help_text), core::mem::offset_of!(DebugCommandDescriptor, handler)], [0x00, 0x04, 0x08]",
    "assert_eq!(core::mem::size_of::<InitializedDebugCommandDescriptors>(), 0x48)",
    "assert_eq!(core::mem::align_of::<InitializedDebugCommandDescriptors>(), 4)",
    "assert_eq!(core::mem::offset_of!(InitializedDebugCommandDescriptors, records), 0)",
    "[debug_command_descriptor(0).unwrap().get(), debug_command_descriptor(1).unwrap().get(), debug_command_descriptor(2).unwrap().get(), debug_command_descriptor(3).unwrap().get(), debug_command_descriptor(4).unwrap().get(), debug_command_descriptor(5).unwrap().get()], [0x0400_1164, 0x0400_1170, 0x0400_117c, 0x0400_1188, 0x0400_1194, 0x0400_11a0]",
    "[debug_command_name(0).unwrap().get(), debug_command_help(0).unwrap().get(), debug_command_handler(0).unwrap().get(), debug_command_name(1).unwrap().get(), debug_command_help(1).unwrap().get(), debug_command_handler(1).unwrap().get(), debug_command_name(2).unwrap().get(), debug_command_help(2).unwrap().get(), debug_command_handler(2).unwrap().get()], [0x0400_1164, 0x0400_1168, 0x0400_116c, 0x0400_1170, 0x0400_1174, 0x0400_1178, 0x0400_117c, 0x0400_1180, 0x0400_1184]",
    "[debug_command_name(3).unwrap().get(), debug_command_help(3).unwrap().get(), debug_command_handler(3).unwrap().get(), debug_command_name(4).unwrap().get(), debug_command_help(4).unwrap().get(), debug_command_handler(4).unwrap().get(), debug_command_name(5).unwrap().get(), debug_command_help(5).unwrap().get(), debug_command_handler(5).unwrap().get()], [0x0400_1188, 0x0400_118c, 0x0400_1190, 0x0400_1194, 0x0400_1198, 0x0400_119c, 0x0400_11a0, 0x0400_11a4, 0x0400_11a8]",
    "assert!(debug_command_descriptor(6).is_none())",
    "assert!(debug_command_name(6).is_none())",
    "assert!(debug_command_help(6).is_none())",
    "assert!(debug_command_handler(6).is_none())",
    "assert_eq!(DEBUG_COMMAND_DESCRIPTORS.get() + core::mem::size_of::<InitializedDebugCommandDescriptors>(), 0x0400_11ac)",
    "assert_eq!(DEBUG_COMMAND_DESCRIPTORS.get() + core::mem::size_of::<InitializedDebugCommandDescriptors>(), INITIALIZED_HIF_CONTROL.get())",
    "assert_eq!(core::mem::size_of::<InitializedVendorImage>(), 0x2078)",
    "assert_eq!(core::mem::size_of::<DtcmLayout>(), DTCM_STATE_SIZE)",
    "assert_eq!(core::mem::align_of::<DtcmLayout>(), 4)",
    "assert_eq!(core::mem::size_of::<SharedDtcmState>(), DTCM_STATE_SIZE)",
    "assert_eq!(core::mem::align_of::<SharedDtcmState>(), 4)",
)
APPROVED_ADDRESS_FUNCTIONS = {
    "debug_command_descriptor", "debug_command_name", "debug_command_help",
    "debug_command_handler",
}
EXISTING_GENERIC_DTCM_PRIMITIVES = {
    "new", "from_offset", "from_raw", "from_raw_unchecked", "get", "offset",
    "cast_mut", "shared_ptr",
}
RELEVANT_API_IDENTIFIERS = {
    "DebugCommandDescriptor", "InitializedDebugCommandDescriptors",
    "DEBUG_COMMAND_DESCRIPTORS", "debug_command_descriptors",
    *APPROVED_ADDRESS_FUNCTIONS,
}
EXPECTED_IDENTIFIER_COUNTS = {
    "DEBUG_COMMAND_DESCRIPTORS": 8,
    "DebugCommandDescriptor": 19,
    "InitializedDebugCommandDescriptors": 14,
    "debug_command_descriptor": 8,
    "debug_command_descriptors": 3,
    "debug_command_handler": 8,
    "debug_command_help": 8,
    "debug_command_name": 8,
}
FORBIDDEN_API = (
    "debug_command_descriptor_unchecked", "debug_command_generic_offset",
    "debug_command_offset", "debug_command_ptr", "debug_command_pointer",
    "debug_command_ref", "debug_command_mut", "debug_command_value",
    "debug_command_read", "debug_command_write", "debug_command_bytes",
    "debug_command_slice", "debug_command_iter", "debug_command_callable",
    "debug_command_callback", "debug_command_call", "debug_command_invoke",
    "debug_command_function", "debug_command_target", "debug_command_copy",
    "debug_command_validate", "debug_command_field", "debug_command_accessor",
    "debug_command_volatile", "debug_command_deref", "debug_command_as_",
    "debug_command_get", "debug_command_set", "debug_command_records",
)


def code_only(source: str, hash_comments: bool, single_quote_strings: bool) -> str:
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
            elif source[index] == '"' or (
                source[index] == "'" and (
                    single_quote_strings
                    or re.match(r"'(?:\\\\.|[^\\\\'\n])'", source[index:]) is not None
                )
            ):
                quote = source[index]; output.append(" "); index += 1; state = "string"
            else:
                output.append(source[index]); index += 1
        elif state == "line":
            if source[index] == "\n": output.append("\n"); state = "code"
            else: output.append(" ")
            index += 1
        elif state == "block":
            if source.startswith("/*", index):
                output.extend("  "); index += 2; depth += 1
            elif source.startswith("*/", index):
                output.extend("  "); index += 2; depth -= 1
                if depth == 0: state = "code"
            else:
                output.append("\n" if source[index] == "\n" else " "); index += 1
        elif source[index] == "\\" and index + 1 < len(source):
            output.extend("  "); index += 2
        elif source[index] == quote:
            output.append(" "); index += 1; state = "code"
        else:
            output.append("\n" if source[index] == "\n" else " "); index += 1
    return "".join(output)


def rust_functions(source: str) -> list[tuple[str, bool, str]]:
    """Return Rust function names, visibility, and complete declarations/bodies."""
    code = code_only(source, hash_comments=False, single_quote_strings=False)
    functions: list[tuple[str, bool, str]] = []
    declaration = re.compile(
        r"(?P<visibility>pub(?:\(crate\))?\s+)?(?:const\s+)?(?:unsafe\s+)?fn\s+"
        r"(?P<name>[A-Za-z_][A-Za-z0-9_]*)\b[^;{]*\{"
    )
    for match in declaration.finditer(code):
        depth = 1
        index = match.end()
        while index < len(code) and depth:
            if code[index] == "{": depth += 1
            elif code[index] == "}": depth -= 1
            index += 1
        if depth:
            raise SystemExit(f"src/dtcm.rs: unterminated function {match.group('name')}")
        functions.append((match.group("name"), match.group("visibility") is not None, code[match.start():index]))
    return functions


def normalized(source: str) -> str:
    return " ".join(source.split())


def generic_dtcm_api_violation(signature: str, body: str) -> bool:
    """Reject renamed generic DTCM pointer/value APIs by semantics, not spelling."""
    numeric_parameter = re.search(
        r"\b[A-Za-z_][A-Za-z0-9_]*\s*:\s*(?:usize|u32)\b", signature
    )
    pointer_like_result = re.search(
        r"->\s*(?:\*const\b|\*mut\b|&|.*\b(?:Iterator|Fn(?:Once|Mut)?)\b)",
        signature,
    )
    direct_dtcm_root = re.search(
        r"\b(?:DTCM_STATE_BASE|DTCM_STATE_END|DTCM_STATE_SIZE|DTCM_STATE)\b",
        body,
    )
    raw_pointer_or_effect = re.search(
        r"(?:\bas\s+\*(?:const|mut)\b|\b(?:read|write)_volatile\b|"
        r"\.read_volatile\s*\(|\.write_volatile\s*\(|\bcast_mut\s*\(|"
        r"\baddr_of(?:_mut)?\s*!|->\s*(?:\*const|\*mut|&))",
        body,
    )
    generic_offset_construction = re.search(
        r"\b(?:from_offset(?:_unchecked)?|new)\s*\([^)]*[+*\-]",
        body,
    )
    return bool(
        numeric_parameter
        and raw_pointer_or_effect
        and (direct_dtcm_root or generic_offset_construction)
    ) or bool(pointer_like_result and direct_dtcm_root and raw_pointer_or_effect)


def check_generic_api_regressions() -> None:
    renamed_pointer_api = """
        pub(crate) fn descriptor_window(slot: usize) -> *mut u32 {
            (DTCM_STATE_BASE + slot * 4) as *mut u32
        }
    """
    signature = renamed_pointer_api.split("{", 1)[0]
    if not generic_dtcm_api_violation(signature, renamed_pointer_api):
        raise SystemExit(
            "checker self-test failed: renamed generic DTCM pointer API was accepted"
        )
    bounded_address_api = """
        pub(crate) const fn debug_command_descriptor(index: usize) -> Option<DtcmAddress> {
            if index < 6 { Some(DtcmAddress::from_offset(DEBUG_COMMAND_DESCRIPTORS.offset() + index * 0x0c)) } else { None }
        }
    """
    if generic_dtcm_api_violation(
        bounded_address_api.split("{", 1)[0], bounded_address_api
    ):
        raise SystemExit(
            "checker self-test failed: bounded address-only descriptor API was rejected"
        )


def source_paths() -> list[Path]:
    return sorted(
        path for path in ROOT.rglob("*")
        if path.is_file()
        and (path.suffix in SOURCE_EXTENSIONS or path.name in SOURCE_FILENAMES)
        and "target" not in path.parts and ".git" not in path.parts
    )


def in_range(value: int) -> bool:
    return RANGE[0] <= value < RANGE[1]


def check_source() -> None:
    check_generic_api_regressions()
    dtcm = (ROOT / "src/dtcm.rs").read_text()
    missing = [item for item in REQUIRED_INVENTORY if item not in dtcm]
    forbidden = [item for item in FORBIDDEN_API if item in dtcm]
    failures = [f"src/dtcm.rs: missing reviewed inventory: {item}" for item in missing]
    failures += [f"src/dtcm.rs: forbidden broad/reference API: {item}" for item in forbidden]

    dtcm_code = code_only(dtcm, hash_comments=False, single_quote_strings=False)
    functions = rust_functions(dtcm)
    focused_tests = [body for name, _, body in functions if name == FOCUSED_TEST_NAME]
    focused_test_attribute = re.findall(
        rf"#\s*\[\s*test\s*\]\s*fn\s+{re.escape(FOCUSED_TEST_NAME)}\b",
        dtcm_code,
    )
    if len(focused_tests) != 1 or len(focused_test_attribute) != 1:
        failures.append(
            f"src/dtcm.rs: expected exactly one focused test named {FOCUSED_TEST_NAME}"
        )
    else:
        focused_test = normalized(focused_tests[0])
        failures += [
            f"src/dtcm.rs: focused test missing exact address/layout assertion: {item}"
            for item in REQUIRED_FOCUSED_TEST_INVENTORY
            if normalized(item) not in focused_test
        ]

    # Constrain declarations and bodies semantically rather than relying on an
    # API's spelling. Any public wrapper which mentions the decoded types,
    # root, enclosing field, approved accessors, or a local/physical address in
    # this interval is part of this slice's API surface.
    relevant_word = re.compile(
        r"\b(?:" + "|".join(map(re.escape, sorted(RELEVANT_API_IDENTIFIERS))) + r")\b"
    )
    actual_identifier_counts = {
        identifier: len(re.findall(rf"\b{re.escape(identifier)}\b", dtcm_code))
        for identifier in EXPECTED_IDENTIFIER_COUNTS
    }
    if actual_identifier_counts != EXPECTED_IDENTIFIER_COUNTS:
        failures.append(
            "src/dtcm.rs: initialized debug-command descriptor identifier surface drifted: "
            f"expected={EXPECTED_IDENTIFIER_COUNTS!r} actual={actual_identifier_counts!r}"
        )
    for name, public, body in functions:
        if not public or name in APPROVED_ADDRESS_FUNCTIONS:
            continue
        signature = body.split("{", 1)[0]
        if (
            name not in EXISTING_GENERIC_DTCM_PRIMITIVES
            and generic_dtcm_api_violation(signature, body)
        ):
            failures.append(
                "src/dtcm.rs: unexpected generic-offset/pointer/reference/callable "
                f"public API can reach quarantined DTCM: {name}"
            )
        literals = [
            int(match.group().replace("_", ""), 16)
            for match in LITERAL.finditer(body)
        ]
        if relevant_word.search(body) or any(
            in_range(value) or 0x1164 <= value <= 0x11AC for value in literals
        ):
            failures.append(
                f"src/dtcm.rs: unexpected public API reaches initialized debug-command descriptors: {name}"
            )
    public_nonfunctions = re.findall(
        r"pub(?:\(crate\))?\s+(?!(?:const\s+)?fn\b)(?:const|static|type)\s+[^;]+;",
        dtcm_code,
    )
    for declaration in public_nonfunctions:
        name = re.search(r"(?:const|static|type)\s+([A-Za-z_][A-Za-z0-9_]*)", declaration)
        literals = [
            int(match.group().replace("_", ""), 16)
            for match in LITERAL.finditer(declaration)
        ]
        if (relevant_word.search(declaration) or any(
            in_range(value) or 0x1164 <= value <= 0x11AC for value in literals
        )) and not (name and name.group(1) == "DEBUG_COMMAND_DESCRIPTORS"):
            failures.append(
                "src/dtcm.rs: unexpected public constant/static/type reaches "
                f"initialized debug-command descriptors: {normalized(declaration)}"
            )

    address_declaration = REQUIRED_INVENTORY[5]
    if dtcm.count(address_declaration) != 1:
        failures.append("src/dtcm.rs: sole address declaration is not unique")
    exact_once = REQUIRED_INVENTORY[0:10]
    failures += [
        f"src/dtcm.rs: reviewed declaration is not unique: {item}"
        for item in exact_once if dtcm.count(item) != 1
    ]
    for name in ("descriptor", "name", "help", "handler"):
        bounded = re.search(
            rf"fn debug_command_{name}\(index: usize\).*?\{{ if index < 6 .*?\}} else \{{ None \}} \}}",
            dtcm,
        )
        if bounded is None:
            failures.append(f"src/dtcm.rs: debug_command_{name} is not bounded by index < 6")
    public_names = re.findall(
        r"pub(?:\(crate\))?\s+(?:const\s+fn|const|fn|static|type|struct)\s+([A-Za-z0-9_]+)",
        dtcm,
    )
    related_public_names = [
        name for name in public_names if "debug_command" in name.lower()
    ]
    if related_public_names != [
        "DEBUG_COMMAND_DESCRIPTORS", "debug_command_descriptor",
        "debug_command_name", "debug_command_help", "debug_command_handler",
    ]:
        failures.append(
            "src/dtcm.rs: unexpected initialized debug-command public API: "
            f"{related_public_names!r}"
        )
    for path in source_paths():
        relative = path.relative_to(ROOT).as_posix()
        if relative in OWNER_FILES:
            continue
        code = code_only(
            path.read_text(errors="replace"),
            path.suffix in {".py", ".sh", ".toml"},
            path.suffix in {".py", ".sh", ".toml"},
        )
        for match in LITERAL.finditer(code):
            token = match.group().replace("_", "")
            value = int(token, 16)
            # Only full physical-address literals are gated. Short values such
            # as an MMIO payload 0x1100 are intentionally not synthesized.
            if len(token) >= 10 and in_range(value):
                line = code.count("\n", 0, match.start()) + 1
                failures.append(
                    f"{relative}:{line}: initialized-debug-command-descriptor literal {match.group()} "
                    "is outside dtcm.rs"
                )
    if failures:
        raise SystemExit("\n".join(failures))
    print(
        "INITIALIZED DEBUG COMMAND DESCRIPTORS SOURCE/LAYOUT DRIFT-EVIDENCE GATE PASSED "
        f"files={len(source_paths())} inventory={len(REQUIRED_INVENTORY)}"
    )


def linked_literals(path: Path) -> collections.Counter[int]:
    with tempfile.NamedTemporaryFile() as output:
        subprocess.run(
            ["llvm-objcopy", "--dump-section", f".text={output.name}", str(path), "/dev/null"],
            check=True,
        )
        data = Path(output.name).read_bytes()
    result: collections.Counter[int] = collections.Counter()
    for offset in range(0, len(data) - 3, 4):
        value = struct.unpack_from("<I", data, offset)[0]
        if in_range(value): result[value] += 1
    return result


def decoded_literal_xrefs(path: Path) -> collections.Counter[tuple[str, int]]:
    disassembly = subprocess.run(
        ["llvm-objdump", "-d", "--print-imm-hex", str(path)],
        check=True, text=True, stdout=subprocess.PIPE,
    ).stdout
    words = {
        int(address, 16): int(value, 16)
        for address, value in re.findall(
            r"^\s*([0-9a-fA-F]+):.*?\.word\s+0x([0-9a-fA-F]+)\s*$",
            disassembly, flags=re.MULTILINE,
        )
    }
    current_symbol = "<outside-symbol>"
    result: collections.Counter[tuple[str, int]] = collections.Counter()
    for line in disassembly.splitlines():
        if header := re.match(r"^[0-9a-fA-F]+ <(.+)>:$", line):
            current_symbol = header.group(1); continue
        if ".word" in line: continue
        reference = re.search(r"@\s+0x([0-9a-fA-F]+)(?:\s|<|$)", line)
        if reference is None: continue
        value = words.get(int(reference.group(1), 16))
        if value is not None and in_range(value): result[(current_symbol, value)] += 1
    return result


def check_elf(path: Path, dump: bool) -> None:
    literals = linked_literals(path)
    xrefs = decoded_literal_xrefs(path)
    if dump:
        print(f"ALLOWED_LINKED_LITERALS={dict(sorted(literals.items()))!r}")
        print(f"ALLOWED_DECODED_XREFS={dict(sorted(xrefs.items()))!r}")
        return
    if literals != ALLOWED_LINKED_LITERALS or xrefs != ALLOWED_DECODED_XREFS:
        raise SystemExit(
            "INITIALIZED DEBUG COMMAND DESCRIPTORS LINKED DRIFT GATE FAILED\n"
            f"literals expected={dict(ALLOWED_LINKED_LITERALS)!r} actual={dict(literals)!r}\n"
            f"xrefs expected={dict(ALLOWED_DECODED_XREFS)!r} actual={dict(xrefs)!r}"
        )
    print(
        "INITIALIZED DEBUG COMMAND DESCRIPTORS LINKED-XREF DRIFT-EVIDENCE GATE PASSED "
        f"literals={sum(literals.values())} decoded_xrefs={sum(xrefs.values())}; "
        "this is drift evidence, not writer closure or ownership proof"
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("elf", nargs="?", type=Path)
    parser.add_argument("--dump", action="store_true")
    args = parser.parse_args()
    check_source()
    if args.elf is not None: check_elf(args.elf, args.dump)


if __name__ == "__main__":
    try:
        main()
    except (OSError, subprocess.CalledProcessError) as error:
        raise SystemExit(f"initialized-debug-command-descriptors drift gate failed: {error}")
