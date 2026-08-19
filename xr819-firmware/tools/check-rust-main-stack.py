#!/usr/bin/env python3
"""Reject regressions in the normal rust_main call-chain stack depth."""

from __future__ import annotations

import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

SYSTEM_STACK_BYTES = 0x0400_C000 - 0x0400_A500
QUALIFIED_CALL_CHAIN_BYTES = SYSTEM_STACK_BYTES
EXCEPTION_STACK_BYTES = 0x100
EXCEPTION_VENEER_BYTES = 14 * 4


@dataclass(frozen=True)
class Function:
    frame: int
    calls: frozenset[str]
    has_metadata: bool
    has_indirect_call: bool = False


def run_tool(*arguments: str) -> str:
    return subprocess.run(
        arguments,
        check=True,
        stdout=subprocess.PIPE,
        text=True,
    ).stdout


def stack_sizes(elf: Path) -> dict[str, int]:
    output = run_tool("llvm-readobj", "--stack-sizes", str(elf))
    sizes: dict[str, int] = {}
    for entry in re.findall(r"Entry \{(.*?)\n  \}", output, re.DOTALL):
        functions = re.search(r"Functions: \[([^\]]+)\]", entry)
        size = re.search(r"Size: 0x([0-9a-fA-F]+)", entry)
        if functions is None or size is None:
            continue
        for name in functions.group(1).split(", "):
            sizes[name] = int(size.group(1), 16)
    if not sizes:
        raise RuntimeError("ELF has no LLVM stack-size metadata; build with -Z emit-stack-sizes")
    return sizes


def normalized_target(name: str) -> str:
    return name.split("+0x", 1)[0]


def saved_register_bytes(register_list: str) -> int:
    count = 0
    for item in (part.strip() for part in register_list.split(",")):
        if "-" not in item:
            count += 1
            continue
        first, last = item.split("-", 1)
        if not first.startswith("r") or not last.startswith("r"):
            raise RuntimeError(f"unsupported saved-register range {item}")
        count += int(last[1:]) - int(first[1:]) + 1
    return count * 4


def disassembly_functions(elf: Path, metadata: dict[str, int]) -> dict[str, Function]:
    lines = run_tool("llvm-objdump", "-d", str(elf)).splitlines()
    header = re.compile(r"^[0-9a-f]+ <([^>]+)>:")
    direct_call = re.compile(r"\bblx?\s+0x[0-9a-f]+\s+<([^>]+)>")
    indirect_call = re.compile(r"\bblx?\s+r(?:1[0-5]|[0-9])\b")
    push = re.compile(r"\bpush\s+\{([^}]+)\}")
    stack_subtract = re.compile(r"\bsub\s+sp,\s*#0x([0-9a-f]+)")

    bodies: dict[str, list[str]] = {}
    calls: dict[str, set[str]] = {}
    indirect: set[str] = set()
    current: str | None = None
    for line in lines:
        if match := header.match(line):
            name = match.group(1)
            current = name
            bodies.setdefault(name, [])
            calls.setdefault(name, set())
            continue
        if current is None:
            continue
        bodies[current].append(line)
        if match := direct_call.search(line):
            target = normalized_target(match.group(1))
            if target != current:
                calls[current].add(target)
        elif indirect_call.search(line):
            indirect.add(current)

    functions: dict[str, Function] = {}
    all_names = set(bodies) | set(metadata)
    for name in all_names:
        if name in metadata:
            frame = metadata[name]
        else:
            # Handwritten assembly has no LLVM metadata. Its fixed prologue is
            # still inspectable; reject less regular stack manipulation below.
            prologue = bodies.get(name, [])[:16]
            frame = sum(
                saved_register_bytes(match.group(1))
                for line in prologue
                if (match := push.search(line))
            ) + sum(
                int(match.group(1), 16)
                for line in prologue
                if (match := stack_subtract.search(line))
            )
        functions[name] = Function(
            frame=frame,
            calls=frozenset(calls.get(name, set())),
            has_metadata=name in metadata,
            has_indirect_call=name in indirect,
        )
    return functions


def maximum_call_chain(
    functions: dict[str, Function], root: str
) -> tuple[int, list[str], set[str]]:
    memo: dict[str, tuple[int, list[str], set[str]]] = {}

    def visit(name: str, active: tuple[str, ...]) -> tuple[int, list[str], set[str]]:
        if name in active:
            cycle = " -> ".join((*active[active.index(name) :], name))
            raise RuntimeError(f"recursive call cycle reachable from {root}: {cycle}")
        if name in memo:
            return memo[name]
        try:
            function = functions[name]
        except KeyError as error:
            raise RuntimeError(f"no disassembly or stack size for reachable function {name}") from error
        if function.has_indirect_call:
            raise RuntimeError(f"indirect call reachable from {root} in {name}")

        best_size = function.frame
        best_path = [name]
        assembly_fallback = set() if function.has_metadata else {name}
        best_fallback = assembly_fallback
        for callee in function.calls:
            callee_size, callee_path, callee_fallback = visit(callee, (*active, name))
            candidate = function.frame + callee_size
            if candidate > best_size:
                best_size = candidate
                best_path = [name, *callee_path]
                best_fallback = assembly_fallback | callee_fallback
        result = (best_size, best_path, best_fallback)
        memo[name] = result
        return result

    return visit(root, ())


def main() -> int:
    if len(sys.argv) != 2:
        print(f"usage: {Path(sys.argv[0]).name} ELF", file=sys.stderr)
        return 2

    try:
        elf = Path(sys.argv[1])
        metadata = stack_sizes(elf)
        functions = disassembly_functions(elf, metadata)
        chain, path, fallback = maximum_call_chain(functions, "rust_main")
        exception_chain, exception_path, exception_fallback = maximum_call_chain(
            functions, "xr819_exception_terminal"
        )
    except (OSError, RuntimeError, subprocess.CalledProcessError) as error:
        print(f"stack check failed: {error}", file=sys.stderr)
        return 1

    print(f"rust_main_stack_frame={functions['rust_main'].frame}")
    print(f"rust_main_max_call_chain={chain}")
    print(f"system_stack_capacity={SYSTEM_STACK_BYTES}")
    print(f"qualified_call_chain_limit={QUALIFIED_CALL_CHAIN_BYTES}")
    print("deepest_call_chain:")
    demangled = dict(zip(path, run_tool("llvm-cxxfilt", *path).splitlines(), strict=True))
    for name in path:
        source = "assembly-prologue" if name in fallback else "llvm"
        print(f"  {functions[name].frame:4d} {source:17s} {demangled[name]}")

    exception_total = EXCEPTION_VENEER_BYTES + exception_chain
    print(f"exception_stack_usage={exception_total}")
    print(f"exception_stack_capacity={EXCEPTION_STACK_BYTES}")
    print("deepest_exception_call_chain:")
    exception_demangled = dict(
        zip(
            exception_path,
            run_tool("llvm-cxxfilt", *exception_path).splitlines(),
            strict=True,
        )
    )
    print(f"  {EXCEPTION_VENEER_BYTES:4d} assembly-veneer   register save")
    for name in exception_path:
        source = "assembly-prologue" if name in exception_fallback else "llvm"
        print(f"  {functions[name].frame:4d} {source:17s} {exception_demangled[name]}")

    if exception_total > EXCEPTION_STACK_BYTES:
        print(
            "exception call chain exceeds its mode stack by "
            f"{exception_total - EXCEPTION_STACK_BYTES} bytes",
            file=sys.stderr,
        )
        return 1
    if chain > QUALIFIED_CALL_CHAIN_BYTES:
        print(
            "rust_main call chain exceeds the qualified baseline by "
            f"{chain - QUALIFIED_CALL_CHAIN_BYTES} bytes",
            file=sys.stderr,
        )
        return 1
    if chain > SYSTEM_STACK_BYTES:
        print(
            "warning: qualified rust_main call chain exceeds the nominal system "
            f"stack by {chain - SYSTEM_STACK_BYTES} bytes",
            file=sys.stderr,
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
