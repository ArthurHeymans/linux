#!/usr/bin/env python3
"""Gate and report the complete exact-parent text-symbol delta.

The checked JSON manifest records every parent-only symbol, candidate-only
symbol, and common symbol whose exact bytes differ. It also records section
hashes, symbol counts, decoded instruction/stack metrics, normalized instruction
hashes, and memory-operation signatures. This is drift evidence rather than
proof of behavioral equivalence or retained-vendor ownership closure.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import tempfile
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_MANIFEST = ROOT / "tools/host-context-codegen-manifest.json"

HEADER = re.compile(r"^([0-9a-f]+) <(.+)>:$")
INSTRUCTION = re.compile(r"^\s*([0-9a-f]+):\s+([a-z][a-z0-9.]*)\s*(.*)$")
MEMORY = re.compile(r"^(?:ldr|str|ldm|stm|push|pop)")
IRQ_BARRIER = {"mrs", "msr", "cpsid", "cpsie", "dmb", "dsb", "isb"}

# These selectors make omission of the independently reviewed paths impossible,
# even if a future build happens to leave one byte-identical to the parent.
REQUIRED_SELECTORS = (
    "rust_main",
    "HostTxDriver5admit",
    "HostTxDriver13service_index",
    "HostTxDriver18confirmation_state",
    "HostTxDriver19finish_confirmation",
    "HostTxDriver22cancelled_confirmation",
    "vendor_host_tx17free_host_context",
    "HostSchedulerReservation16publish_in_batch",
    "vendor_host_tx22release_pending_to_pas",
    "vendor_host_tx21program_pipe_eligible",
    "tx25prepare_host_frame_timing",
    "PreparedProbePublication6cancel",
    "tx27release_wsm_context_address",
    "tx27prepare_context_publication",
    "tx29emit_host_frame_descriptor_at",
    "tx30emit_prepared_probe_descriptor",
    "tx33release_unpublished_probe_context",
    "tx21prepare_probe_context",
    "tx37service_single_probe_runtime_inactive",
    "Transport14enqueue_output",
    "Transport15release_request",
    "Transport24publish_request_in_place",
    "Transport7publish",
)

# These routines preserve the parent's decoded load/store mnemonic order. The
# host recorder separately checks exact source-level address/width/value order;
# it does not execute ARM or HIF behavior.
EXACT_MEMORY_ORDER_SELECTORS = (
    "vendor_host_tx17free_host_context",
    "Transport24publish_request_in_place",
)


@dataclass(frozen=True)
class SymbolRecord:
    address: int
    size: int
    instructions: int
    stack: int
    sha256: str
    normalized_instructions_sha256: str
    memory_mnemonics_sha256: str
    memory_operations: int


def run_text(*args: str) -> str:
    return subprocess.run(
        args,
        check=True,
        text=True,
        stdout=subprocess.PIPE,
    ).stdout


def digest(data: bytes | str) -> str:
    if isinstance(data, str):
        data = data.encode()
    return hashlib.sha256(data).hexdigest()


def text_section(path: Path) -> bytes:
    with tempfile.NamedTemporaryFile() as output:
        subprocess.run(
            ["llvm-objcopy", "--dump-section", f".text={output.name}", str(path), "/dev/null"],
            check=True,
        )
        return Path(output.name).read_bytes()


def raw_symbols(path: Path) -> dict[str, tuple[int, int]]:
    symbols: dict[str, tuple[int, int]] = {}
    for line in run_text("llvm-nm", "-S", "--size-sort", str(path)).splitlines():
        match = re.match(r"^([0-9a-f]+)\s+([0-9a-f]+)\s+[tT]\s+(.+)$", line)
        if match and int(match.group(2), 16) != 0:
            symbols[match.group(3)] = (int(match.group(1), 16), int(match.group(2), 16))
    return symbols


def stacks(path: Path) -> dict[str, int]:
    result: dict[str, int] = {}
    pending: list[str] = []
    for line in run_text("llvm-readobj", "--stack-sizes", str(path)).splitlines():
        if match := re.search(r"Functions: \[(.+)\]", line):
            pending = match.group(1).split(", ")
        elif pending and (match := re.search(r"Size: 0x([0-9a-fA-F]+)", line)):
            size = int(match.group(1), 16)
            for name in pending:
                result[name] = size
            pending = []
    return result


def normalize_operands(operands: str) -> str:
    operands = operands.split("@")[0]
    operands = re.sub(r"<[^>]+>", "<symbol>", operands)
    operands = re.sub(r"(?<![A-Za-z0-9_])(?:0x)?[0-9a-f]+", "#", operands)
    return " ".join(operands.split())


def disassembly(path: Path) -> tuple[dict[str, list[tuple[str, str]]], list[tuple[str, str]]]:
    functions: dict[str, list[tuple[str, str]]] = {}
    irq_barrier: list[tuple[str, str]] = []
    current: str | None = None
    for line in run_text("llvm-objdump", "-d", "--no-show-raw-insn", str(path)).splitlines():
        if match := HEADER.match(line):
            name = match.group(2)
            current = name
            functions.setdefault(name, [])
            continue
        if current is None or ".word" in line:
            continue
        match = INSTRUCTION.match(line)
        if not match:
            continue
        mnemonic = match.group(2)
        operands = normalize_operands(match.group(3))
        functions[current].append((mnemonic, operands))
        if mnemonic in IRQ_BARRIER:
            irq_barrier.append((mnemonic, operands))
    return functions, irq_barrier


def records(path: Path) -> tuple[dict[str, SymbolRecord], str, list[tuple[str, str]]]:
    text = text_section(path)
    symbols = raw_symbols(path)
    stack_sizes = stacks(path)
    functions, irq_barrier = disassembly(path)
    result: dict[str, SymbolRecord] = {}
    for name, (address, size) in symbols.items():
        body = functions.get(name, [])
        normalized = "\n".join(f"{mnemonic} {operands}" for mnemonic, operands in body)
        memory = [mnemonic for mnemonic, _ in body if MEMORY.match(mnemonic)]
        result[name] = SymbolRecord(
            address=address,
            size=size,
            instructions=len(body),
            stack=stack_sizes.get(name, 0),
            sha256=digest(text[address : address + size]),
            normalized_instructions_sha256=digest(normalized),
            memory_mnemonics_sha256=digest("\n".join(memory)),
            memory_operations=len(memory),
        )
    return result, digest(text), irq_barrier


def select(symbols: dict[str, SymbolRecord], selector: str) -> str:
    matches = [name for name in symbols if selector in name]
    if len(matches) != 1:
        raise SystemExit(f"selector {selector!r} matched {matches!r}")
    return matches[0]


def build_report(parent: Path, candidate: Path) -> dict[str, Any]:
    parent_symbols, parent_text_hash, parent_irq = records(parent)
    candidate_symbols, candidate_text_hash, candidate_irq = records(candidate)
    parent_names = set(parent_symbols)
    candidate_names = set(candidate_symbols)
    common = parent_names & candidate_names

    required: dict[str, dict[str, object]] = {}
    for selector in REQUIRED_SELECTORS:
        parent_name = select(parent_symbols, selector)
        candidate_name = select(candidate_symbols, selector)
        required[selector] = {
            "parent": parent_name,
            "candidate": candidate_name,
            "parent_record": asdict(parent_symbols[parent_name]),
            "candidate_record": asdict(candidate_symbols[candidate_name]),
        }

    exact_memory_order: dict[str, dict[str, object]] = {}
    for selector in EXACT_MEMORY_ORDER_SELECTORS:
        parent_name = select(parent_symbols, selector)
        candidate_name = select(candidate_symbols, selector)
        parent_record = parent_symbols[parent_name]
        candidate_record = candidate_symbols[candidate_name]
        exact = (
            parent_record.memory_operations == candidate_record.memory_operations
            and parent_record.memory_mnemonics_sha256 == candidate_record.memory_mnemonics_sha256
        )
        exact_memory_order[selector] = {
            "exact": exact,
            "parent_operations": parent_record.memory_operations,
            "candidate_operations": candidate_record.memory_operations,
            "parent_sha256": parent_record.memory_mnemonics_sha256,
            "candidate_sha256": candidate_record.memory_mnemonics_sha256,
        }
        if not exact:
            raise SystemExit(f"{selector}: decoded memory mnemonic order changed")

    changed = {
        name: {
            "parent": asdict(parent_symbols[name]),
            "candidate": asdict(candidate_symbols[name]),
        }
        for name in sorted(common)
        if parent_symbols[name].sha256 != candidate_symbols[name].sha256
    }
    return {
        "schema": 1,
        "parent_text_sha256": parent_text_hash,
        "candidate_text_sha256": candidate_text_hash,
        "parent_symbol_count": len(parent_symbols),
        "candidate_symbol_count": len(candidate_symbols),
        "common_symbol_count": len(common),
        "unchanged_common_symbol_count": len(common) - len(changed),
        "parent_only": {
            name: asdict(parent_symbols[name]) for name in sorted(parent_names - candidate_names)
        },
        "candidate_only": {
            name: asdict(candidate_symbols[name]) for name in sorted(candidate_names - parent_names)
        },
        "changed_common": changed,
        "required": required,
        "exact_memory_order": exact_memory_order,
        "irq_barrier": {
            "exact": parent_irq == candidate_irq,
            "parent": parent_irq,
            "candidate": candidate_irq,
        },
    }


def print_report(report: dict[str, Any]) -> None:
    changed = report["changed_common"]
    parent_only = report["parent_only"]
    candidate_only = report["candidate_only"]
    print(
        "symbol_inventory "
        f"parent={report['parent_symbol_count']} candidate={report['candidate_symbol_count']} "
        f"common={report['common_symbol_count']} unchanged={report['unchanged_common_symbol_count']} "
        f"changed={len(changed)} parent_only={len(parent_only)} candidate_only={len(candidate_only)}"
    )
    print("required_symbol_rows:")
    for selector, entry in report["required"].items():
        parent = entry["parent_record"]
        candidate = entry["candidate_record"]
        print(
            f"  {selector}: size={parent['size']:#x}->{candidate['size']:#x} "
            f"instructions={parent['instructions']}->{candidate['instructions']} "
            f"stack={parent['stack']}->{candidate['stack']} "
            f"memory_ops={parent['memory_operations']}->{candidate['memory_operations']}"
        )
    print("parent_only_symbols:")
    for name in parent_only:
        print(f"  {name}")
    print("candidate_only_symbols:")
    for name in candidate_only:
        print(f"  {name}")
    print("complete_exact_byte_residual:")
    for name, entry in changed.items():
        parent = entry["parent"]
        candidate = entry["candidate"]
        print(
            f"  {name}: size={parent['size']:#x}->{candidate['size']:#x} "
            f"instructions={parent['instructions']}->{candidate['instructions']} "
            f"stack={parent['stack']}->{candidate['stack']} "
            f"sha256={parent['sha256']}->{candidate['sha256']}"
        )
    irq = report["irq_barrier"]
    print(f"irq_barrier_sequence={len(irq['candidate'])} exact={irq['exact']}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("parent", type=Path)
    parser.add_argument("candidate", type=Path)
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    parser.add_argument("--write-manifest", action="store_true")
    args = parser.parse_args()

    report = build_report(args.parent, args.candidate)
    print_report(report)
    if not report["irq_barrier"]["exact"]:
        raise SystemExit("global IRQ/barrier sequence changed")

    serialized = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if args.write_manifest:
        args.manifest.write_text(serialized)
        print(f"WROTE COMPLETE CODEGEN DRIFT MANIFEST {args.manifest}")
        return
    if not args.manifest.exists():
        raise SystemExit(f"missing reviewed codegen manifest: {args.manifest}")
    expected = json.loads(args.manifest.read_text())
    normalized_report = json.loads(serialized)
    if expected != normalized_report:
        raise SystemExit(
            "HOT CODEGEN GATE FAILED: complete symbol delta differs from reviewed manifest; "
            "run with --write-manifest only after reviewing every reported change"
        )
    print("HOT CODEGEN GATE PASSED (complete reviewed symbol delta; drift evidence, not closure)")


if __name__ == "__main__":
    try:
        main()
    except (OSError, subprocess.CalledProcessError) as error:
        raise SystemExit(f"hot codegen gate failed: {error}")
