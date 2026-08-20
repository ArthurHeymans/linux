#!/usr/bin/env python3
"""Source and decoded-literal/xref drift gates for the typed VIF ABI.

The linked check intentionally follows decoded PC-relative literal loads. It
never scans aligned words as addresses, because Thumb instructions can be
misclassified as data. Source checks cover the physical range, record starts,
and the historical synthesized low-16-bit forms.
"""

from __future__ import annotations

import argparse
import collections
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VIF_RANGE = (0x04003E98, 0x040049A8)
SYNTHESIZED = {0x3E98, 0x4248, 0x45F8, 0x49A8}
LITERAL = re.compile(r"0x[0-9a-fA-F_]+")
SOURCE_EXTENSIONS = {".rs", ".py", ".sh", ".c", ".h", ".cc", ".cpp", ".s", ".S", ".ld", ".x", ".toml"}
OWNER_FILES = {"src/dtcm.rs", "src/vif.rs", "tools/check-vif-layout.py"}
FORBIDDEN_FORMS = ("VIF_BASE", "VIF_STRIDE", "VIF_RECORDS", "VIF_RECORD_SIZE")

# Exact decoded literal-load xrefs in the qualified feature-free image. This is
# regenerated only after reviewing disassembly and operation ordering.
ALLOWED_DECODED_XREFS: collections.Counter[tuple[str, int]] = collections.Counter(
    {
        ('_RNvMs_NtCsiHlLB2CErfM_14xr819_firmware14host_tx_driverNtB4_12HostTxDriver13service_index', 0x04003EB0): 2,
        ('_RNvMs_NtCsiHlLB2CErfM_14xr819_firmware14host_tx_driverNtB4_12HostTxDriver13service_index', 0x04003FF4): 1,
        ('_RNvMs_NtCsiHlLB2CErfM_14xr819_firmware14host_tx_driverNtB4_12HostTxDriver22cancelled_confirmation', 0x04003EC8): 1,
        ('_RNvMs_NtCsiHlLB2CErfM_14xr819_firmware14host_tx_driverNtB4_12HostTxDriver5admit', 0x04003E9E): 3,
        ('_RNvMs_NtCsiHlLB2CErfM_14xr819_firmware14host_tx_driverNtB4_12HostTxDriver5admit', 0x04003FC2): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware14vendor_host_tx17free_host_context', 0x04003E9E): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware14vendor_host_tx21program_pipe_eligible', 0x04003EB4): 3,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware14vendor_host_tx21program_pipe_eligible', 0x04003FF4): 2,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware14vendor_host_tx22reject_unscheduled_pas', 0x04003EC8): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware14vendor_host_tx22release_pending_to_pas', 0x04003EC8): 2,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware2tx21prepare_probe_context', 0x04003EBC): 2,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware2tx21prepare_probe_context', 0x04003FC4): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware2tx26service_host_management_tx', 0x04003EB1): 2,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware2tx37service_single_probe_runtime_inactive', 0x04003EB0): 4,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware2tx37service_single_probe_runtime_inactive', 0x04003FC0): 6,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware2tx37service_single_probe_runtime_inactive', 0x0400425C): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware3mac23reinitialize_after_wake', 0x04003EB0): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware3mac23reinitialize_after_wake', 0x0400425E): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware3phy24begin_channel_transition', 0x04003E98): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware3phy24begin_channel_transition', 0x04004611): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware3phy24begin_channel_transition', 0x04004614): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware3vif12activate_sta', 0x04003EB0): 2,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware3vif12activate_sta', 0x04003FB0): 2,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware3vif8snapshot', 0x04003EB0): 2,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware3vif8teardown', 0x04003EB0): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware3vif8teardown', 0x04004261): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware4join12activate_sta', 0x04003EB1): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware4join5reset', 0x04003EB1): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware4join5reset', 0x04004261): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware4scan7service', 0x04003E9A): 2,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware4scan7service', 0x04004261): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware8platform23program_station_address', 0x04003ECC): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware8platform23program_station_address', 0x0400427C): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware8platform23program_station_address', 0x0400462C): 1,
        ('rust_main', 0x04003EB1): 1,
        ('rust_main', 0x04004261): 1,
    }
)


def code_only(source: str) -> str:
    output: list[str] = []
    index = 0
    state = "code"
    depth = 0
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
            elif source[index] == '"':
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
        elif source[index] == '"':
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
        and path.suffix in SOURCE_EXTENSIONS
        and "target" not in path.parts
        and ".git" not in path.parts
    )


def production_code(relative: str, code: str) -> str:
    if relative.endswith(".rs"):
        marker = code.find("#[cfg(test)]")
        if marker >= 0:
            return code[:marker]
    return code


def check_source() -> None:
    failures: list[str] = []
    for path in source_paths():
        relative = path.relative_to(ROOT).as_posix()
        if relative in OWNER_FILES:
            continue
        code = production_code(relative, code_only(path.read_text(errors="replace")))
        for match in LITERAL.finditer(code):
            value = int(match.group().replace("_", ""), 16)
            if VIF_RANGE[0] <= value <= VIF_RANGE[1] or value in SYNTHESIZED:
                line = code.count("\n", 0, match.start()) + 1
                failures.append(f"{relative}:{line}: VIF literal {match.group()} is outside typed owner")
        for form in FORBIDDEN_FORMS:
            for match in re.finditer(rf"\b{form}\b", code):
                line = code.count("\n", 0, match.start()) + 1
                failures.append(f"{relative}:{line}: synthesized VIF form {form} is outside typed owner")
    if failures:
        raise SystemExit("\n".join(failures))
    print(f"VIF SOURCE DRIFT GATE PASSED files={len(source_paths())}")


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
        header = re.match(r"^[0-9a-fA-F]+ <(.+)>:$", line)
        if header:
            current_symbol = header.group(1)
            continue
        if ".word" in line:
            continue
        reference = re.search(r"@\s+0x([0-9a-fA-F]+)(?:\s|<|$)", line)
        if reference is None:
            continue
        value = words.get(int(reference.group(1), 16))
        if value is not None and VIF_RANGE[0] <= value <= VIF_RANGE[1]:
            xrefs[(current_symbol, value)] += 1
    return xrefs


def check_elf(path: Path) -> None:
    actual = decoded_literal_xrefs(path)
    if actual != ALLOWED_DECODED_XREFS:
        missing = ALLOWED_DECODED_XREFS - actual
        extra = actual - ALLOWED_DECODED_XREFS
        raise SystemExit(
            "VIF decoded literal/xref manifest changed: "
            f"missing={dict(missing)!r} extra={dict(extra)!r} actual={dict(actual)!r}"
        )
    print(f"VIF LINKED DRIFT GATE PASSED decoded_xrefs={sum(actual.values())}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("elf", nargs="?", type=Path)
    args = parser.parse_args()
    check_source()
    if args.elf is not None:
        check_elf(args.elf)


if __name__ == "__main__":
    try:
        main()
    except (OSError, subprocess.CalledProcessError) as error:
        raise SystemExit(f"VIF drift gate failed: {error}")
