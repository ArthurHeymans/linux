#!/usr/bin/env python3
"""Freeze direct access to the XR819 DTCM quarantine outside its owner.

Production DTCM address construction is confined to ``src/dtcm.rs``. This gate
keeps the reviewed literal inventory empty and rejects new raw addresses or
owner-only constructors elsewhere. The manifest is deliberately line-number
independent so ordinary source movement does not create churn.
"""

from __future__ import annotations

import argparse
import json
import re
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "tools" / "dtcm-direct-access-manifest.json"
OWNER = "src/dtcm.rs"
DTCM_BASE = 4 << 24
DTCM_END = DTCM_BASE + (10 << 12)
LITERAL = re.compile(r"0x[0-9a-fA-F_]+")
OWNER_ONLY_PATTERNS = (
    re.compile(r"DtcmAddress\s*::\s*(?:new|from_offset(?:_unchecked)?)\s*\("),
    re.compile(r"link_section\s*=\s*\"\.dtcm(?:\.|\")"),
)
DIRECT_FIELD_ARITHMETIC = re.compile(
    r"(?:crate\s*::\s*)?dtcm\s*::\s*[A-Za-z_][A-Za-z0-9_]*"
    r"(?:\s*\([^()\n]*\))?\s*\.\s*get\s*\(\s*\)\s*"
    r"(?:[+-]|\.\s*wrapping_(?:add|sub)\s*\()"
)


def code_only(source: str) -> str:
    """Replace comments and ordinary string contents while preserving lines."""
    output: list[str] = []
    index = 0
    state = "code"
    block_depth = 0
    while index < len(source):
        if state == "code":
            if source.startswith("//", index):
                output.extend("  ")
                index += 2
                state = "line-comment"
            elif source.startswith("/*", index):
                output.extend("  ")
                index += 2
                block_depth = 1
                state = "block-comment"
            elif source[index] == '"':
                output.append(" ")
                index += 1
                state = "string"
            else:
                output.append(source[index])
                index += 1
        elif state == "line-comment":
            if source[index] == "\n":
                output.append("\n")
                state = "code"
            else:
                output.append(" ")
            index += 1
        elif state == "block-comment":
            if source.startswith("/*", index):
                output.extend("  ")
                index += 2
                block_depth += 1
            elif source.startswith("*/", index):
                output.extend("  ")
                index += 2
                block_depth -= 1
                if block_depth == 0:
                    state = "code"
            else:
                output.append("\n" if source[index] == "\n" else " ")
                index += 1
        else:
            if source[index] == "\\" and index + 1 < len(source):
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


def without_cfg_test_items(code: str) -> str:
    """Mask each cfg(test)-annotated item without hiding later production code."""
    marker = "#[cfg(test)]"
    output = list(code)
    search_from = 0
    while (start := code.find(marker, search_from)) >= 0:
        cursor = start + len(marker)
        while cursor < len(code) and code[cursor].isspace():
            cursor += 1

        paren_depth = 0
        bracket_depth = 0
        while cursor < len(code):
            character = code[cursor]
            if character == "(":
                paren_depth += 1
            elif character == ")":
                paren_depth -= 1
            elif character == "[":
                bracket_depth += 1
            elif character == "]":
                bracket_depth -= 1
            elif paren_depth == 0 and bracket_depth == 0 and character in "{;,":
                if character == "{":
                    brace_depth = 1
                    cursor += 1
                    while cursor < len(code) and brace_depth:
                        if code[cursor] == "{":
                            brace_depth += 1
                        elif code[cursor] == "}":
                            brace_depth -= 1
                        cursor += 1
                    if cursor < len(code) and code[cursor] == ";":
                        cursor += 1
                else:
                    cursor += 1
                break
            cursor += 1

        for index in range(start, cursor):
            if output[index] != "\n":
                output[index] = " "
        search_from = max(cursor, start + len(marker))
    return "".join(output)


def inventory() -> tuple[dict[str, dict[str, int]], list[str]]:
    result: dict[str, dict[str, int]] = {}
    failures: list[str] = []
    for source_path in sorted((ROOT / "src").rglob("*.rs")):
        relative = source_path.relative_to(ROOT).as_posix()
        if relative == OWNER:
            continue
        code = without_cfg_test_items(code_only(source_path.read_text()))
        counts: Counter[int] = Counter()
        for match in LITERAL.finditer(code):
            value = int(match.group().replace("_", ""), 16)
            if DTCM_BASE <= value < DTCM_END:
                counts[value] += 1
        if counts:
            result[relative] = {
                f"0x{value:08x}": count for value, count in sorted(counts.items())
            }
        for pattern in OWNER_ONLY_PATTERNS:
            for match in pattern.finditer(code):
                line = code.count("\n", 0, match.start()) + 1
                failures.append(
                    f"{relative}:{line}: DTCM address/section construction is owned by {OWNER}"
                )
        for match in DIRECT_FIELD_ARITHMETIC.finditer(code):
            line = code.count("\n", 0, match.start()) + 1
            failures.append(
                f"{relative}:{line}: manual arithmetic on a DTCM field root is owned by {OWNER}"
            )
    return result, failures


def load_manifest() -> dict[str, dict[str, int]]:
    try:
        data = json.loads(MANIFEST.read_text())
    except (OSError, json.JSONDecodeError) as error:
        raise SystemExit(f"cannot read {MANIFEST}: {error}") from error
    if data.get("schema") != 1 or not isinstance(data.get("inventory"), dict):
        raise SystemExit(f"unsupported manifest schema in {MANIFEST}")
    return data["inventory"]


def write_manifest(current: dict[str, dict[str, int]]) -> None:
    data = {
        "schema": 1,
        "purpose": "Direct DTCM literals outside src/dtcm.rs; the reviewed production baseline is empty.",
        "inventory": current,
    }
    MANIFEST.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n")


def compare(
    expected: dict[str, dict[str, int]], current: dict[str, dict[str, int]]
) -> list[str]:
    failures: list[str] = []
    for path in sorted(set(expected) | set(current)):
        expected_values = expected.get(path, {})
        current_values = current.get(path, {})
        for literal in sorted(set(expected_values) | set(current_values)):
            old = expected_values.get(literal, 0)
            new = current_values.get(literal, 0)
            if new > old:
                failures.append(
                    f"{path}: direct DTCM literal {literal} grew from {old} to {new} occurrence(s)"
                )
            elif new < old:
                failures.append(
                    f"{path}: direct DTCM literal {literal} shrank from {old} to {new} occurrence(s); review and update the manifest"
                )
    return failures


def check_regressions() -> None:
    rejected = (
        "crate::dtcm::TABLE.get() + index * 4",
        "dtcm::field(index).get() - 1",
        "crate::dtcm::TABLE.get().wrapping_add(index)",
    )
    accepted = (
        "crate::dtcm::table_entry_unchecked(index).get()",
        "address.get() + index * 4",
    )
    if not all(DIRECT_FIELD_ARITHMETIC.search(source) for source in rejected):
        raise SystemExit("DTCM direct-field-arithmetic regression fixture was not rejected")
    if any(DIRECT_FIELD_ARITHMETIC.search(source) for source in accepted):
        raise SystemExit("DTCM direct-field-arithmetic regression fixture was falsely rejected")


def main() -> None:
    check_regressions()
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--update",
        action="store_true",
        help="rewrite the reviewed baseline after removing or explicitly auditing accesses",
    )
    args = parser.parse_args()

    current, owner_failures = inventory()
    if owner_failures:
        raise SystemExit("\n".join(owner_failures))
    if args.update:
        write_manifest(current)
        print(f"WROTE {MANIFEST.relative_to(ROOT)}")
        return

    failures = compare(load_manifest(), current)
    if failures:
        failures.append(
            "Replace new accesses with field-derived src/dtcm.rs APIs; use --update only for a reviewed inventory change."
        )
        raise SystemExit("\n".join(failures))

    occurrence_count = sum(sum(values.values()) for values in current.values())
    print(
        f"DTCM DIRECT-ACCESS INVENTORY PASSED ({occurrence_count} known literal occurrences; baseline matched)"
    )


if __name__ == "__main__":
    main()
