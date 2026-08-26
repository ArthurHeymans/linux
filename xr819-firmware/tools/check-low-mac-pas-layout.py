#!/usr/bin/env python3
"""Drift gates for known low-MAC/PAS source forms and linked literal accesses.

This is a reviewed-access manifest, not proof that every computed DTCM reference
has been recovered. The ELF checks combine exact in-range literal words with
literal-pool loads decoded to their containing symbols.
"""

from __future__ import annotations

import argparse
import collections
import importlib.util
import re
import struct
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FAMILY = (0x04003678, 0x04003E78)
LITERAL = re.compile(r"0x[0-9a-fA-F_]+")
SOURCE_EXTENSIONS = {".rs", ".py", ".sh", ".c", ".h", ".cc", ".cpp", ".s", ".S", ".ld", ".x", ".toml"}
FORBIDDEN_FORMS = (
    "LOW_MAC_PAS_ROOT",
    "PAS_BASE",
    "PAS_STATE_BASE",
    "PAS_VIF_STATE",
    "PIPE_RETRY_MASK_TABLE",
)

# Exact aligned words in the linked feature-free image. This catches added or
# removed known-family values, including words not reached by a decoded load.
ALLOWED_LINKED_LITERALS: collections.Counter[int] = collections.Counter(
    {
        0x04003680: 1,
        0x04003684: 3,
        0x04003688: 1,
        0x04003768: 1,
        0x04003A50: 1,
        0x04003A68: 6,
        0x04003A6A: 6,
        0x04003A6C: 4,
        0x04003A6D: 5,
        0x04003A70: 2,
        0x04003ACC: 1,
        0x04003AD0: 1,
        0x04003AD1: 2,
        0x04003AD2: 1,
        0x04003AD8: 1,
        0x04003AE8: 3,
        0x04003AE9: 1,
        0x04003AEC: 1,
        0x04003B34: 2,
        0x04003B74: 2,
        0x04003C18: 2,
        0x04003CA0: 1,
        0x04003CD8: 1,
    }
)

# Filled from decoded PC-relative literal loads in the reviewed linked image.
# Keys are (containing ELF symbol, loaded value); counts preserve repeated xrefs
# even when several instructions reuse one literal-pool word.
ALLOWED_DECODED_XREFS: collections.Counter[tuple[str, int]] = collections.Counter(
    {
        ('_RINvNtCsiHlLB2CErfM_14xr819_firmware2tx21complete_tx_pipe_slotNtB2_21SingleProbeMacBackendEB4_', 0x04003A6C): 1,
        ('_RINvNtCsiHlLB2CErfM_14xr819_firmware2tx21complete_tx_pipe_slotNtB2_21SingleProbeMacBackendEB4_', 0x04003CD8): 1,
        ('_RINvNtCsiHlLB2CErfM_14xr819_firmware2tx27build_single_frame_durationNtB2_19VolatileMacPipeMmioEB4_', 0x04003B34): 1,
        ('_RINvNtCsiHlLB2CErfM_14xr819_firmware2tx32execute_single_probe_publicationNtB2_19VolatileMacPipeMmioEB4_', 0x04003A6C): 1,
        ('_RINvNtCsiHlLB2CErfM_14xr819_firmware2tx32execute_single_probe_publicationNtB2_19VolatileMacPipeMmioEB4_', 0x04003B74): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware2tx32finalize_staged_host_class0_pipe', 0x04003A6C): 1,
        ('_RNvMs1_NtCsiHlLB2CErfM_14xr819_firmware3phyNtB5_26ChannelTransitionScheduler5start', 0x04003A68): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware14vendor_host_tx34reserve_non_aggregate_scheduler_at', 0x04003A6D): 1,
        ('_RNvMs_NtCsiHlLB2CErfM_14xr819_firmware14host_tx_driverNtB4_12HostTxDriver5admit', 0x04003A6A): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware14vendor_host_tx21program_pipe_eligible', 0x04003AEC): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware14vendor_host_tx22release_pending_to_pas', 0x04003A6D): 4,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware2tx21prepare_probe_context', 0x04003AE9): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware2tx23probe_runtime_quiescent', 0x04003A6C): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware2tx24initialize_internal_pool', 0x04003688): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware2tx31prepare_single_frame_pas_timing', 0x04003AD8): 2,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware2tx37service_single_probe_runtime_inactive', 0x04003684): 2,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware2tx37service_single_probe_runtime_inactive', 0x04003A6A): 7,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware3mac19build_control_frame', 0x04003A70): 2,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware3mac23reinitialize_after_wake', 0x04003A68): 2,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware3mac23reinitialize_after_wake', 0x04003AD0): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware3mac23reinitialize_after_wake', 0x04003AD2): 2,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware3mac23reprogram_after_channel', 0x04003AE8): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware3mac27program_joined_station_mode', 0x04003AD1): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware3mac28install_response_descriptors', 0x04003A70): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware3mac31finish_unjoined_scan_radio_stop', 0x04003A68): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware3mac31initialize_vendor_startup_state', 0x04003768): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware3mac31initialize_vendor_startup_state', 0x04003A6D): 3,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware3phy24begin_channel_transition', 0x04003A50): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware3phy24begin_channel_transition', 0x04003C18): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware3phy24begin_channel_transition', 0x04003CA0): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware3phy25finish_channel_transition', 0x04003A6D): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware3vif10apply_edca', 0x04003A68): 2,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware3vif10apply_edca', 0x04003B74): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware3vif12activate_sta', 0x04003680): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware3vif12activate_sta', 0x04003AD1): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware3vif12activate_sta', 0x04003C18): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware3vif17reset_pas_backoff', 0x04003B34): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware3vif8teardown', 0x04003684): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware3vif8teardown', 0x04003AE8): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware4join12activate_sta', 0x04003AE8): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware4scan7service', 0x04003A6D): 1,
        ('_RNvNtCsiHlLB2CErfM_14xr819_firmware8platform23program_station_address', 0x04003ACC): 1,
        ('rust_main', 0x04003A68): 2,
    }
)


def load_module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def code_only(source: str, hash_comments: bool, single_quote_strings: bool) -> str:
    """Blank comments and quoted strings while preserving line positions."""
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
                state = "block"
                depth = 1
            elif hash_comments and source[index] == "#":
                output.append(" ")
                index += 1
                state = "line"
            elif source[index] == '"' or (
                source[index] == "'"
                and (
                    single_quote_strings
                    or re.match(r"'(?:\\\\.|[^\\\\'\\n])'", source[index:]) is not None
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
    paths = [path for path in ROOT.rglob("*") if path.is_file() and path.suffix in SOURCE_EXTENSIONS]
    return sorted(path for path in paths if "target" not in path.parts and ".git" not in path.parts)


def check_source() -> None:
    failures: list[str] = []
    for path in source_paths():
        relative = path.relative_to(ROOT).as_posix()
        if relative in {"src/dtcm.rs", "tools/check-low-mac-pas-layout.py", "tools/check-pre-vif-header-layout.py"}:
            continue
        code = code_only(
            path.read_text(errors="replace"),
            path.suffix in {".py", ".sh", ".toml"},
            path.suffix in {".py", ".sh", ".toml"},
        )
        for match in LITERAL.finditer(code):
            value = int(match.group().replace("_", ""), 16)
            if FAMILY[0] <= value < FAMILY[1]:
                line = code.count("\n", 0, match.start()) + 1
                failures.append(
                    f"{relative}:{line}: known low-MAC/PAS literal {match.group()} is outside dtcm.rs"
                )
        for form in FORBIDDEN_FORMS:
            for match in re.finditer(rf"\b{re.escape(form)}\b", code):
                line = code.count("\n", 0, match.start()) + 1
                failures.append(
                    f"{relative}:{line}: synthesized low-MAC/PAS form {form} is outside dtcm.rs"
                )
    if failures:
        raise SystemExit("\n".join(failures))
    print(f"LOW-MAC/PAS SOURCE DRIFT GATE PASSED files={len(source_paths())}")


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
        if value is not None and FAMILY[0] <= value < FAMILY[1]:
            xrefs[(current_symbol, value)] += 1
    return xrefs


def check_elf(path: Path) -> None:
    if not ALLOWED_LINKED_LITERALS:
        raise SystemExit("low-MAC/PAS linked literal manifest is empty")
    if not ALLOWED_DECODED_XREFS:
        raise SystemExit("low-MAC/PAS decoded literal/xref manifest is empty")
    packer = load_module("pack_sectioned_elf", ROOT / "tools" / "pack-sectioned-elf.py")
    data = path.read_bytes()
    text = next(section for section in packer.parse_sections(data) if section.name == ".text")
    payload = data[text.file_offset : text.file_offset + text.size]
    actual_literals = collections.Counter(
        value
        for (value,) in struct.iter_unpack("<I", payload[: len(payload) & ~3])
        if FAMILY[0] <= value < FAMILY[1]
    )
    if actual_literals != ALLOWED_LINKED_LITERALS:
        missing = ALLOWED_LINKED_LITERALS - actual_literals
        extra = actual_literals - ALLOWED_LINKED_LITERALS
        raise SystemExit(
            "low-MAC/PAS linked literal-word manifest changed: "
            f"missing={dict(missing)!r} extra={dict(extra)!r} actual={dict(actual_literals)!r}"
        )

    actual_xrefs = decoded_literal_xrefs(path)
    if actual_xrefs != ALLOWED_DECODED_XREFS:
        missing = ALLOWED_DECODED_XREFS - actual_xrefs
        extra = actual_xrefs - ALLOWED_DECODED_XREFS
        raise SystemExit(
            "low-MAC/PAS decoded literal/xref manifest changed: "
            f"missing={dict(missing)!r} extra={dict(extra)!r}"
        )
    print(
        "LOW-MAC/PAS LINKED DRIFT GATE PASSED "
        f"literal_words={sum(actual_literals.values())} decoded_xrefs={sum(actual_xrefs.values())}"
    )


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
    except (OSError, RuntimeError, subprocess.CalledProcessError, StopIteration) as error:
        raise SystemExit(f"low-MAC/PAS drift gate failed: {error}")
