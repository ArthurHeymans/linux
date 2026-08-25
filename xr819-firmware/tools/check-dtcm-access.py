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
    r"(?:\s*\([^()\n]*\))?\s*\.\s*get\s*\(\s*\)"
    r"(?:\s+as\s+(?:u32|usize))?\s*"
    r"(?:<<|>>|[+*/%|&^-]|\.\s*(?:wrapping|checked|saturating)_(?:add|sub|mul)\s*\()"
)
DIRECT_INTEGER_ALIAS = re.compile(
    r"\bconst\s+[A-Za-z_][A-Za-z0-9_]*\s*:\s*(?:u32|usize)\s*="
    r"[^;]*?(?:crate\s*::\s*)?dtcm\s*::\s*[^;]*?\.\s*get\s*\(\s*\)"
)
LOCAL_INTEGER_ALIAS = re.compile(
    r"\blet\s+(?:mut\s+)?(?P<name>[A-Za-z_][A-Za-z0-9_]*)"
    r"(?:\s*:\s*(?:u32|usize))?\s*=\s*(?:\(\s*)*"
    r"(?:crate\s*::\s*)?dtcm\s*::\s*[^;]*?\.\s*get\s*\(\s*\)"
    r"(?:\s+as\s+(?:u32|usize))?\s*\)*\s*;"
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


def local_integer_alias_arithmetic(code: str) -> list[tuple[int, str]]:
    failures: list[tuple[int, str]] = []
    for match in LOCAL_INTEGER_ALIAS.finditer(code):
        name = match.group("name")
        depth = code.count("{", 0, match.end()) - code.count("}", 0, match.end())
        end = len(code)
        cursor = match.end()
        current_depth = depth
        while cursor < len(code):
            if code[cursor] == "{":
                current_depth += 1
            elif code[cursor] == "}":
                current_depth -= 1
                if current_depth < depth:
                    end = cursor
                    break
            cursor += 1
        scope_start = match.end()
        scope = list(code[scope_start:end])
        for redeclaration in re.finditer(
            rf"\blet\s+(?:mut\s+)?{re.escape(name)}\b", "".join(scope)
        ):
            absolute = scope_start + redeclaration.start()
            redeclaration_depth = code.count("{", 0, absolute) - code.count("}", 0, absolute)
            if redeclaration_depth == depth:
                scope[redeclaration.start():] = " " * (len(scope) - redeclaration.start())
                break
            shadow_end = redeclaration.end()
            shadow_depth = redeclaration_depth
            while shadow_end < len(scope):
                absolute_shadow = scope_start + shadow_end
                current_shadow_depth = (
                    code.count("{", 0, absolute_shadow)
                    - code.count("}", 0, absolute_shadow)
                )
                if current_shadow_depth < shadow_depth:
                    break
                shadow_end += 1
            scope[redeclaration.start():shadow_end] = " " * (shadow_end - redeclaration.start())
        scope_text = "".join(scope)
        operator = re.compile(
            r"(?:\+=|-=|<<=|>>=|<<|>>|[+*/%|&^-]|\.\s*(?:wrapping|checked|saturating)_(?:add|sub|mul)\s*\()"
        )
        used_for_arithmetic = False
        for use in re.finditer(rf"\b{re.escape(name)}\b", scope_text):
            suffix = scope_text[use.end():]
            direct = re.match(r"(?:\s+as\s+(?:u32|usize))?\s*", suffix)
            if direct and operator.match(suffix, direct.end()):
                used_for_arithmetic = True
                break
            prefix = scope_text[:use.start()]
            opening = len(prefix.rstrip()) - 1
            if opening >= 0 and prefix.rstrip().endswith("("):
                before_opening = prefix.rstrip()[:-1].rstrip()
                if not before_opening or not re.search(r"[A-Za-z0-9_\)\]]$", before_opening):
                    grouped = re.match(
                        r"(?:\s+as\s+(?:u32|usize))?\s*\)\s*", suffix
                    )
                    if grouped and operator.match(suffix, grouped.end()):
                        used_for_arithmetic = True
                        break
        if used_for_arithmetic:
            failures.append((match.start(), name))
    return failures


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
        for match in DIRECT_INTEGER_ALIAS.finditer(code):
            line = code.count("\n", 0, match.start()) + 1
            failures.append(
                f"{relative}:{line}: integer alias for a DTCM field root is forbidden"
            )
        for position, name in local_integer_alias_arithmetic(code):
            line = code.count("\n", 0, position) + 1
            failures.append(
                f"{relative}:{line}: arithmetic through local DTCM integer alias {name!r} is owned by {OWNER}"
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
        "crate::dtcm::TABLE.get() as u32 + index * 4",
        "dtcm::field(index).get() as usize - 1",
        "dtcm::field().get() & !3",
        "crate::dtcm::TABLE.get().checked_add(4)",
    )
    accepted = (
        "crate::dtcm::table_entry_unchecked(index).get()",
        "address.get() + index * 4",
    )
    alias_rejected = (
        "const ROOT: usize = crate::dtcm::TABLE.get();",
        "const ROOT: u32 = dtcm::field().get() as u32;",
        "const ROOT: usize = (crate::dtcm::TABLE.get());",
        "const ROOT: usize = crate::dtcm::TABLE\n    .get();",
    )
    local_alias_rejected = (
        "fn f() { let root = crate::dtcm::TABLE.get(); use_word(root + 4); }",
        "fn f() { let mut root: usize = dtcm::field().get(); root += 4; }",
        "fn f() { let root = dtcm::field().get() as u32; use_word(root.wrapping_add(4)); }",
        "fn f() { let root = (crate::dtcm::TABLE.get()); use_word((root as u32) | 3); }",
        "fn f() { let root = dtcm::field().get(); { let root = 0; use_word(root + 1); } use_word(root + 4); }",
    )
    local_alias_accepted = (
        "fn f() { let address = crate::dtcm::field().get(); use_word(address); }",
        "fn f() { let state = crate::dtcm::field().get() as *mut u8; state.write_volatile(0); }",
        "fn f() { let root = dtcm::field().get(); { let root = 0; use_word(root + 1); } use_word(root); }",
        "fn f() { let root = dtcm::field().get(); let root = 0; use_word(root + 1); }",
        "fn f() { let value = read_u32(dtcm::field().get()); use_word(value + 1); }",
    )
    if not all(DIRECT_FIELD_ARITHMETIC.search(source) for source in rejected):
        raise SystemExit("DTCM direct-field-arithmetic regression fixture was not rejected")
    if any(DIRECT_FIELD_ARITHMETIC.search(source) for source in accepted):
        raise SystemExit("DTCM direct-field-arithmetic regression fixture was falsely rejected")
    if not all(DIRECT_INTEGER_ALIAS.search(source) for source in alias_rejected):
        raise SystemExit("DTCM integer-alias regression fixture was not rejected")
    if any(DIRECT_INTEGER_ALIAS.search(source) for source in accepted):
        raise SystemExit("DTCM integer-alias regression fixture was falsely rejected")
    if not all(local_integer_alias_arithmetic(source) for source in local_alias_rejected):
        raise SystemExit("DTCM local integer-alias arithmetic regression fixture was not rejected")
    if any(local_integer_alias_arithmetic(source) for source in local_alias_accepted):
        raise SystemExit("DTCM local integer-alias arithmetic regression fixture was falsely rejected")


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
