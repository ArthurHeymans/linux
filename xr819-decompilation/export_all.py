#!/usr/bin/env python3
"""Export every defined Ghidra function through the ghidra-cli bridge."""

import json
import subprocess
import sys
from pathlib import Path

GHIDRA = "/home/arthur/.cargo/bin/ghidra"


def run(*args: str) -> str:
    result = subprocess.run(
        [GHIDRA, *args], check=True, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE
    )
    return result.stdout


def main() -> None:
    if len(sys.argv) != 4:
        raise SystemExit("usage: export_all.py PROJECT PROGRAM OUTPUT")
    project, program, output_name = sys.argv[1:]
    functions = json.loads(
        run(
            "function",
            "list",
            "--limit",
            "10000",
            "--sort",
            "address",
            "--project",
            project,
            "--program",
            program,
        )
    )
    output_path = Path(output_name)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open("w", encoding="utf-8") as output:
        output.write(
            f"/*\n * Ghidra decompiler export for {project} / {program}\n"
            f" * Functions: {len(functions)}\n * Reference pseudocode; not buildable source.\n */\n\n"
        )
        for index, function in enumerate(functions, 1):
            address = function["address"]
            name = function["name"]
            print(f"[{index}/{len(functions)}] {address} {name}", flush=True)
            output.write(
                "\n/* ======================================================================\n"
                f" * {address}  {name}\n"
                " * ====================================================================== */\n"
            )
            try:
                decompiled = json.loads(
                    run(
                        "decompile",
                        address,
                        "--project",
                        project,
                        "--program",
                        program,
                    )
                )
                if decompiled:
                    output.write(decompiled[0]["code"])
                    output.write("\n")
                else:
                    output.write("/* DECOMPILATION RETURNED NO RESULT */\n")
            except (subprocess.CalledProcessError, json.JSONDecodeError, KeyError) as error:
                output.write(f"/* DECOMPILATION FAILED: {error} */\n")
            output.flush()


if __name__ == "__main__":
    main()
