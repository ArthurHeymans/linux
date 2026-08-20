#!/usr/bin/env python3
"""Bound exact-parent code generation drift in timing- and ordering-sensitive paths.

This is a focused qualification gate, not proof of whole-firmware equivalence.
It compares symbol size, decoded instruction count, LLVM stack metadata, memory
mnemonic order where stable, and the global IRQ/barrier instruction sequence.
"""

from __future__ import annotations

import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class Limit:
    selector: str
    size: int
    instructions: int
    stack: int
    exact_memory_order: bool = False


LIMITS = [
    Limit("HostTxDriver13service_index", 0x20, 16, 0),
    Limit("vendor_host_tx21program_pipe_eligible", 0x20, 16, 0),
    Limit("tx37service_single_probe_runtime_inactive", 0x30, 24, 32),
    Limit("tx21prepare_probe_context", 0x10, 8, 16),
    Limit("scan7service", 0x30, 24, 16),
    Limit("join12activate_sta", 0, 0, 0, True),
    Limit("join5reset", 0x20, 20, 16),
    Limit("vif12activate_sta", 0x30, 24, 32),
    Limit("vif8teardown", 0x10, 12, 8),
    Limit("platform23program_station_address", 0x08, 6, 0),
    Limit("mac23reinitialize_after_wake", 0, 0, 0),
]

INSTRUCTION = re.compile(r"^\s*([0-9a-f]+):\s+([a-z][a-z0-9.]*)\s*(.*)$")
HEADER = re.compile(r"^([0-9a-f]+) <(.+)>:$")
MEMORY = re.compile(r"^(?:ldr|str|ldm|stm|push|pop)")
IRQ_BARRIER = {"mrs", "msr", "cpsid", "cpsie", "dmb", "dsb", "isb"}


def run(*args: str) -> str:
    return subprocess.run(args, check=True, text=True, stdout=subprocess.PIPE).stdout


def raw_symbols(path: Path) -> dict[str, tuple[int, int]]:
    symbols: dict[str, tuple[int, int]] = {}
    for line in run("llvm-nm", "-S", "--size-sort", str(path)).splitlines():
        match = re.match(r"^([0-9a-f]+)\s+([0-9a-f]+)\s+\w\s+(.+)$", line)
        if match:
            symbols[match.group(3)] = (int(match.group(1), 16), int(match.group(2), 16))
    return symbols


def stacks(path: Path) -> dict[str, int]:
    result: dict[str, int] = {}
    pending: list[str] = []
    for line in run("llvm-readobj", "--stack-sizes", str(path)).splitlines():
        if match := re.search(r"Functions: \[(.+)\]", line):
            pending = match.group(1).split(", ")
        elif pending and (match := re.search(r"Size: 0x([0-9a-fA-F]+)", line)):
            size = int(match.group(1), 16)
            for name in pending:
                result[name] = size
            pending = []
    return result


def disassembly(path: Path) -> tuple[dict[str, list[tuple[str, str]]], list[tuple[str, str]]]:
    functions: dict[str, list[tuple[str, str]]] = {}
    irq_barrier: list[tuple[str, str]] = []
    current: str | None = None
    for line in run("llvm-objdump", "-d", "--no-show-raw-insn", str(path)).splitlines():
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
        operands = re.sub(r"0x[0-9a-f]+", "#", match.group(3).split("@")[0]).strip()
        functions[current].append((mnemonic, operands))
        if mnemonic in IRQ_BARRIER:
            irq_barrier.append((mnemonic, operands))
    return functions, irq_barrier


def select(symbols: dict[str, tuple[int, int]], selector: str) -> str:
    matches = [name for name in symbols if selector in name]
    if len(matches) != 1:
        raise SystemExit(f"selector {selector!r} matched {matches!r}")
    return matches[0]


def main() -> None:
    if len(sys.argv) != 3:
        raise SystemExit(f"usage: {sys.argv[0]} PARENT_ELF CANDIDATE_ELF")
    parent, candidate = map(Path, sys.argv[1:])
    parent_symbols, candidate_symbols = raw_symbols(parent), raw_symbols(candidate)
    parent_stacks, candidate_stacks = stacks(parent), stacks(candidate)
    parent_disasm, parent_irq = disassembly(parent)
    candidate_disasm, candidate_irq = disassembly(candidate)

    failures: list[str] = []
    rows: list[str] = []
    for limit in LIMITS:
        parent_name = select(parent_symbols, limit.selector)
        candidate_name = select(candidate_symbols, limit.selector)
        parent_size = parent_symbols[parent_name][1]
        candidate_size = candidate_symbols[candidate_name][1]
        parent_body = parent_disasm.get(parent_name, [])
        candidate_body = candidate_disasm.get(candidate_name, [])
        parent_count, candidate_count = len(parent_body), len(candidate_body)
        parent_stack = parent_stacks.get(parent_name, 0)
        candidate_stack = candidate_stacks.get(candidate_name, 0)
        deltas = (
            candidate_size - parent_size,
            candidate_count - parent_count,
            candidate_stack - parent_stack,
        )
        if abs(deltas[0]) > limit.size or abs(deltas[1]) > limit.instructions or abs(deltas[2]) > limit.stack:
            failures.append(f"{limit.selector}: deltas size/instructions/stack={deltas} exceed {limit}")
        if limit.exact_memory_order:
            parent_memory = [mnemonic for mnemonic, _ in parent_body if MEMORY.match(mnemonic)]
            candidate_memory = [mnemonic for mnemonic, _ in candidate_body if MEMORY.match(mnemonic)]
            if parent_memory != candidate_memory:
                failures.append(f"{limit.selector}: decoded memory mnemonic order changed")
        rows.append(
            f"{limit.selector}: size={parent_size:#x}->{candidate_size:#x} "
            f"instructions={parent_count}->{candidate_count} stack={parent_stack}->{candidate_stack}"
        )

    if parent_irq != candidate_irq:
        failures.append(
            f"global IRQ/barrier sequence changed: parent={parent_irq!r} candidate={candidate_irq!r}"
        )
    print("\n".join(rows))
    print(f"irq_barrier_sequence={len(candidate_irq)} exact")
    if failures:
        raise SystemExit("HOT CODEGEN GATE FAILED\n" + "\n".join(failures))
    print("HOT CODEGEN GATE PASSED (bounded focused comparison; not whole-parent equivalence)")


if __name__ == "__main__":
    try:
        main()
    except (OSError, subprocess.CalledProcessError) as error:
        raise SystemExit(f"hot codegen gate failed: {error}")
