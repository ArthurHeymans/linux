#!/usr/bin/env python3
"""Drift-evidence gates for initialized MAC runtime accounting.

The source check covers production code in the project's Rust, Python, shell,
C/C++, assembly, linker-script, build, and TOML files. Individual Rust items
that are exclusively `cfg(test)` are masked so later production items remain
covered. The ELF manifests pin aligned literal words and decoded PC-relative
literal loads to containing symbols. These manifests are drift evidence, not
complete closure: they do not recover register-only computed addresses,
arbitrary MIB memory access, or retained vendor/IRQ ownership.
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
MAC_RUNTIME_ACCOUNTING_RANGE = (0x04001F78, 0x04001FCC)
SYNTHESIZED: set[int] = set()
LITERAL = re.compile(r"0x[0-9a-fA-F_]+")
SOURCE_EXTENSIONS = {
    ".rs", ".py", ".sh", ".c", ".h", ".hh", ".hpp", ".hxx", ".cc", ".cpp", ".cxx",
    ".s", ".S", ".asm", ".inc", ".ld", ".lds", ".ldh", ".x", ".toml", ".mk",
}
SOURCE_FILENAMES = {"Makefile", "Kconfig"}
OWNER_FILES = {
    "src/dtcm.rs",
    "tools/check-mac-runtime-accounting-layout.py",
}
ALLOWED_SOURCE_LITERALS: dict[str, set[int]] = {}
FORBIDDEN_FORMS = (
    "MAC_RUNTIME_ACCOUNTING_BASE",
    "MAC_SOFTWARE_RECORDS_BASE",
    "MAC_ACCOUNTING_PARAMETER_BASE",
)

# Regenerated only after reviewing the candidate disassembly and operation order.
ALLOWED_LINKED_LITERALS: collections.Counter[int] = collections.Counter({
    0x04001F78: 6,
    0x04001F84: 1,
    0x04001F90: 1,
    0x04001FBC: 6,
})
ALLOWED_DECODED_XREFS: collections.Counter[tuple[str, int]] = collections.Counter({
    ('_RINvNtCsiHlLB2CErfM_14xr819_firmware2tx21complete_tx_pipe_slotNtB2_21SingleProbeMacBackendEB4_', 0x04001F84): 2,
    ('_RNvNtCsiHlLB2CErfM_14xr819_firmware2tx26enter_mac_fatal_quiescence', 0x04001F78): 1,
    ('_RNvNtCsiHlLB2CErfM_14xr819_firmware2tx37service_single_probe_runtime_inactive', 0x04001F78): 14,
    ('_RNvNtCsiHlLB2CErfM_14xr819_firmware2tx37service_single_probe_runtime_inactive', 0x04001FBC): 8,
    ('_RNvNtCsiHlLB2CErfM_14xr819_firmware3phy25initialize_mac_core_mode0', 0x04001FBC): 1,
    ('_RNvNtCsiHlLB2CErfM_14xr819_firmware8platform18prepare_packet_dma', 0x04001F90): 1,
})


def code_only(source: str, hash_comments: bool, single_quote_strings: bool) -> str:
    output: list[str] = []
    index = 0
    state = "code"
    depth = 0
    quote = ""
    while index < len(source):
        if state == "code":
            if source.startswith("//", index):
                output.extend("  ")
                index += 2
                state = "line"
            elif source.startswith("/*", index):
                output.extend("  ")
                index += 2
                depth = 1
                state = "block"
            elif hash_comments and source[index] == "#":
                output.append(" ")
                index += 1
                state = "line"
            elif source[index] == '"' or (
                source[index] == "'"
                and (
                    single_quote_strings
                    or re.match(r"'(?:\\\\.|[^\\\\'\n])'", source[index:]) is not None
                )
            ):
                quote = source[index]
                output.append(" ")
                index += 1
                state = "string"
            else:
                output.append(source[index])
                index += 1
        elif state == "line":
            if source[index] == "\n":
                output.append("\n")
                state = "code"
            else:
                output.append(" ")
            index += 1
        elif state == "block":
            if source.startswith("/*", index):
                output.extend("  ")
                index += 2
                depth += 1
            elif source.startswith("*/", index):
                output.extend("  ")
                index += 2
                depth -= 1
                if depth == 0:
                    state = "code"
            else:
                output.append("\n" if source[index] == "\n" else " ")
                index += 1
        elif source[index] == "\\" and index + 1 < len(source):
            output.extend("  ")
            index += 2
        elif source[index] == quote:
            output.append(" ")
            index += 1
            state = "code"
        else:
            output.append("\n" if source[index] == "\n" else " ")
            index += 1
    return "".join(output)


def source_paths() -> list[Path]:
    return sorted(
        path
        for path in ROOT.rglob("*")
        if path.is_file()
        and (path.suffix in SOURCE_EXTENSIONS or path.name in SOURCE_FILENAMES)
        and "target" not in path.parts
        and ".git" not in path.parts
    )


TEST_ONLY_CFG = re.compile(r"#\s*\[\s*cfg\s*\(\s*(?:test\b|all\s*\(\s*test\b)")


def mask_test_only_items(code: str) -> str:
    """Mask complete cfg(test)-only items while preserving line numbers."""
    masked = list(code)
    search_from = 0
    while match := TEST_ONLY_CFG.search(code, search_from):
        attribute_end = code.find("]", match.end())
        if attribute_end < 0:
            break
        item_start = attribute_end + 1
        while item_start < len(code) and code[item_start].isspace():
            item_start += 1
        brace = code.find("{", item_start)
        semicolon = code.find(";", item_start)
        if semicolon >= 0 and (brace < 0 or semicolon < brace):
            item_end = semicolon + 1
        elif brace >= 0:
            depth = 1
            item_end = brace + 1
            while item_end < len(code) and depth:
                depth += (code[item_end] == "{") - (code[item_end] == "}")
                item_end += 1
            if depth:
                break
        else:
            break
        for index in range(match.start(), item_end):
            if masked[index] != "\n":
                masked[index] = " "
        search_from = item_end
    return "".join(masked)


def production_code(relative: str, code: str) -> str:
    return mask_test_only_items(code) if relative.endswith(".rs") else code


def in_family(value: int) -> bool:
    return MAC_RUNTIME_ACCOUNTING_RANGE[0] <= value < MAC_RUNTIME_ACCOUNTING_RANGE[1]


def check_source() -> None:
    paths = source_paths()
    failures: list[str] = []
    for path in paths:
        relative = path.relative_to(ROOT).as_posix()
        if relative in OWNER_FILES:
            continue
        code = production_code(
            relative,
            code_only(
                path.read_text(errors="replace"),
                path.suffix in {".py", ".sh", ".toml"},
                path.suffix in {".py", ".sh", ".toml"},
            ),
        )
        for match in LITERAL.finditer(code):
            value = int(match.group().replace("_", ""), 16)
            if (in_family(value) or value in SYNTHESIZED) and value not in ALLOWED_SOURCE_LITERALS.get(relative, set()):
                line = code.count("\n", 0, match.start()) + 1
                failures.append(f"{relative}:{line}: MAC-runtime-accounting literal {match.group()} is outside dtcm.rs")
        for form in FORBIDDEN_FORMS:
            for match in re.finditer(rf"\b{re.escape(form)}\b", code):
                line = code.count("\n", 0, match.start()) + 1
                failures.append(f"{relative}:{line}: synthesized MAC-runtime-accounting form {form} is outside dtcm.rs")
    if failures:
        raise SystemExit("\n".join(failures))
    print(f"MAC RUNTIME ACCOUNTING SOURCE DRIFT-EVIDENCE GATE PASSED files={len(paths)}")


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
        if in_family(value):
            result[value] += 1
    return result


def decoded_literal_xrefs(path: Path) -> collections.Counter[tuple[str, int]]:
    disassembly = subprocess.run(
        ["llvm-objdump", "-d", "--print-imm-hex", str(path)],
        check=True,
        text=True,
        stdout=subprocess.PIPE,
    ).stdout
    words = {
        int(address, 16): int(value, 16)
        for address, value in re.findall(
            r"^\s*([0-9a-fA-F]+):.*?\.word\s+0x([0-9a-fA-F]+)\s*$",
            disassembly,
            flags=re.MULTILINE,
        )
    }
    current_symbol = "<outside-symbol>"
    xrefs: collections.Counter[tuple[str, int]] = collections.Counter()
    for line in disassembly.splitlines():
        if header := re.match(r"^[0-9a-fA-F]+ <(.+)>:$", line):
            current_symbol = header.group(1)
            continue
        if ".word" in line:
            continue
        reference = re.search(r"@\s+0x([0-9a-fA-F]+)(?:\s|<|$)", line)
        if reference is None:
            continue
        value = words.get(int(reference.group(1), 16))
        if value is not None and in_family(value):
            xrefs[(current_symbol, value)] += 1
    return xrefs


def check_elf(path: Path, dump: bool) -> None:
    literals = linked_literals(path)
    xrefs = decoded_literal_xrefs(path)
    if dump:
        print(f"ALLOWED_LINKED_LITERALS={dict(sorted(literals.items()))!r}")
        print(f"ALLOWED_DECODED_XREFS={dict(sorted(xrefs.items()))!r}")
        return
    failures: list[str] = []
    if literals != ALLOWED_LINKED_LITERALS:
        failures.append(
            "linked literals changed: "
            f"missing={dict(ALLOWED_LINKED_LITERALS - literals)!r} "
            f"extra={dict(literals - ALLOWED_LINKED_LITERALS)!r} actual={dict(literals)!r}"
        )
    if xrefs != ALLOWED_DECODED_XREFS:
        failures.append(
            "decoded literal/xrefs changed: "
            f"missing={dict(ALLOWED_DECODED_XREFS - xrefs)!r} "
            f"extra={dict(xrefs - ALLOWED_DECODED_XREFS)!r} actual={dict(xrefs)!r}"
        )
    if failures:
        raise SystemExit("MAC RUNTIME ACCOUNTING LINKED DRIFT GATE FAILED\n" + "\n".join(failures))
    print(
        "MAC RUNTIME ACCOUNTING LINKED DRIFT-EVIDENCE GATE PASSED "
        f"literals={sum(literals.values())} decoded_xrefs={sum(xrefs.values())}"
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("elf", nargs="?", type=Path)
    parser.add_argument("--dump", action="store_true")
    args = parser.parse_args()
    check_source()
    if args.elf is not None:
        check_elf(args.elf, args.dump)


if __name__ == "__main__":
    try:
        main()
    except (OSError, subprocess.CalledProcessError) as error:
        raise SystemExit(f"MAC-runtime-accounting drift gate failed: {error}")
