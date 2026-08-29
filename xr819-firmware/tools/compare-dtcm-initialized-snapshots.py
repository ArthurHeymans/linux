#!/usr/bin/env python3
"""Compare target DTCM snapshots against the reviewed startup writer contract."""

from __future__ import annotations

import argparse
import json
import tempfile
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTRACT_PATH = ROOT / "tools" / "dtcm-initialized-snapshot-contract.json"


@dataclass(frozen=True)
class Range:
    start: int
    end: int
    owner: str


def number(value: str) -> int:
    return int(value, 0)


def load_contract() -> dict:
    try:
        contract = json.loads(CONTRACT_PATH.read_text())
    except (OSError, json.JSONDecodeError) as error:
        raise SystemExit(f"cannot read {CONTRACT_PATH}: {error}") from error
    if contract.get("schema") != 1:
        raise SystemExit(f"unsupported snapshot contract schema in {CONTRACT_PATH}")
    size = number(contract["size"])
    transitions = contract.get("transitions")
    if size != 0x2078 or not isinstance(transitions, dict):
        raise SystemExit("initialized-image snapshot contract has invalid bounds")
    return contract


def expanded_transition(contract: dict, name: str, stack: tuple[str, ...] = ()) -> tuple[list[Range], list[dict]]:
    if name in stack:
        raise SystemExit(f"snapshot contract include cycle: {' -> '.join((*stack, name))}")
    try:
        transition = contract["transitions"][name]
    except KeyError as error:
        raise SystemExit(f"unknown snapshot transition: {name}") from error

    ranges: list[Range] = []
    expected: list[dict] = []
    for included in transition.get("include", []):
        child_ranges, child_expected = expanded_transition(contract, included, (*stack, name))
        ranges.extend(child_ranges)
        expected.extend(child_expected)
    for item in transition.get("allowed_changes", []):
        ranges.append(Range(number(item["start"]), number(item["end"]), item["owner"]))
    expected.extend(transition.get("expected_after", []))
    return ranges, expected


def validate_contract(contract: dict) -> None:
    size = number(contract["size"])
    for name in contract["transitions"]:
        ranges, expected = expanded_transition(contract, name)
        for item in ranges:
            if not (0 <= item.start < item.end <= size):
                raise SystemExit(f"{name}: invalid allowed range {item}")
        ordered = sorted(ranges, key=lambda item: (item.start, item.end, item.owner))
        for left, right in zip(ordered, ordered[1:]):
            if left.end > right.start and left != right:
                raise SystemExit(f"{name}: overlapping allowed ranges {left} and {right}")
        for item in expected:
            offset = number(item["offset"])
            try:
                value = bytes.fromhex(item["bytes"])
            except ValueError as error:
                raise SystemExit(f"{name}: invalid expected bytes for {item.get('field')}") from error
            if not value or offset < 0 or offset + len(value) > size:
                raise SystemExit(f"{name}: expected value outside initialized image: {item}")
            if not all(any(region.start <= byte < region.end for region in ranges) for byte in range(offset, offset + len(value))):
                raise SystemExit(f"{name}: expected field is outside its allowed writer ranges: {item.get('field')}")


def read_snapshot(path: Path, size: int) -> bytes:
    try:
        data = path.read_bytes()
    except OSError as error:
        raise SystemExit(f"cannot read snapshot {path}: {error}") from error
    if len(data) != size:
        raise SystemExit(f"{path}: expected 0x{size:x} bytes, found 0x{len(data):x}")
    return data


def owner_for(offset: int, ranges: list[Range]) -> str | None:
    owners = sorted({item.owner for item in ranges if item.start <= offset < item.end})
    return "+".join(owners) if owners else None


def changed_runs(before: bytes, after: bytes) -> list[tuple[int, int]]:
    changed = [index for index, pair in enumerate(zip(before, after)) if pair[0] != pair[1]]
    if not changed:
        return []
    runs: list[tuple[int, int]] = []
    start = previous = changed[0]
    for offset in changed[1:]:
        if offset != previous + 1:
            runs.append((start, previous + 1))
            start = offset
        previous = offset
    runs.append((start, previous + 1))
    return runs


def compare(contract: dict, transition: str, before: bytes, after: bytes) -> list[str]:
    ranges, expected = expanded_transition(contract, transition)
    failures: list[str] = []
    for offset, (old, new) in enumerate(zip(before, after)):
        if old != new and owner_for(offset, ranges) is None:
            failures.append(
                f"unowned change at +0x{offset:04x}: 0x{old:02x} -> 0x{new:02x}"
            )
    for item in expected:
        offset = number(item["offset"])
        value = bytes.fromhex(item["bytes"])
        observed = after[offset : offset + len(value)]
        if observed != value:
            failures.append(
                f"{item['field']} at +0x{offset:04x}: expected {value.hex()}, found {observed.hex()}"
            )
    return failures


def self_test(contract: dict) -> None:
    size = number(contract["size"])
    zero = bytes(size)
    canonical = bytearray(zero)
    for transition in contract["transitions"]:
        canonical[:] = zero
        for item in expanded_transition(contract, transition)[1]:
            offset = number(item["offset"])
            value = bytes.fromhex(item["bytes"])
            canonical[offset : offset + len(value)] = value
        if compare(contract, transition, zero, bytes(canonical)):
            raise SystemExit(
                f"snapshot comparator rejected its canonical {transition} fixture"
            )
    corrupt = bytearray(zero)
    for item in expanded_transition(contract, "zero-to-platform")[1]:
        offset = number(item["offset"])
        value = bytes.fromhex(item["bytes"])
        corrupt[offset : offset + len(value)] = value
    corrupt[0x1000] ^= 1
    if not compare(contract, "zero-to-platform", zero, bytes(corrupt)):
        raise SystemExit("snapshot comparator accepted an unowned initialized-image change")
    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "short.bin"
        path.write_bytes(b"short")
        try:
            read_snapshot(path, size)
        except SystemExit:
            pass
        else:
            raise SystemExit("snapshot comparator accepted a truncated snapshot")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("before", type=Path, nargs="?")
    parser.add_argument("after", type=Path, nargs="?")
    parser.add_argument("--transition", choices=("zero-to-platform", "platform-to-startup", "warm-entry-to-startup"))
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()

    contract = load_contract()
    validate_contract(contract)
    self_test(contract)
    if args.self_test:
        if args.before is not None or args.after is not None or args.transition is not None:
            raise SystemExit("--self-test does not accept snapshots or --transition")
        print("DTCM INITIALIZED-IMAGE SNAPSHOT CONTRACT SELF-TEST PASSED")
        return
    if args.before is None or args.after is None or args.transition is None:
        raise SystemExit("provide BEFORE AFTER --transition NAME, or use --self-test")

    size = number(contract["size"])
    before = read_snapshot(args.before, size)
    after = read_snapshot(args.after, size)
    failures = compare(contract, args.transition, before, after)
    if failures:
        raise SystemExit("\n".join(failures))

    ranges, _ = expanded_transition(contract, args.transition)
    runs = changed_runs(before, after)
    for start, end in runs:
        owners = sorted(
            owner
            for owner in {owner_for(offset, ranges) for offset in range(start, end)}
            if owner is not None
        )
        print(f"+0x{start:04x}..+0x{end:04x} owners={','.join(owners)}")
    print(
        f"DTCM SNAPSHOT TRANSITION PASSED name={args.transition} changed_bytes="
        f"{sum(end - start for start, end in runs)} runs={len(runs)}"
    )


if __name__ == "__main__":
    main()
