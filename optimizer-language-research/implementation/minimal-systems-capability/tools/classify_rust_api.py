#!/usr/bin/env python3
"""Map every canonical stable Rust census row to one domain route."""

from __future__ import annotations

import csv
import pathlib
import re
from collections import Counter


ROOT = pathlib.Path(__file__).resolve().parent.parent
INVENTORY = ROOT / "RUST-1.97.0-API-INVENTORY.tsv"
MODULES = ROOT / "RUST-1.97.0-MODULE-ACCOUNTING.tsv"
RULES = ROOT / "DOMAIN-CLASSIFICATION-RULES.tsv"
OUTPUT = ROOT / "RUST-1.97.0-DOMAIN-CLASSIFICATION.tsv"
SUMMARY = ROOT / "RUST-1.97.0-DOMAIN-SUMMARY.tsv"
MODULE_OUTPUT = ROOT / "RUST-1.97.0-MODULE-DOMAIN-MAP.tsv"

DOMAIN_IDS = {
    "FFI, ABI, and OS resources": "D18",
    "async, tasks, and pinning": "D23",
    "atomics and synchronization": "D22",
    "build-time metadata and code generation": "D14",
    "bytes and text": "D11",
    "conversion and behavior": "D03",
    "error presentation and chaining": "D05",
    "filesystems and paths": "D16",
    "formatting and presentation": "D12",
    "heap allocation and runtime": "D07",
    "interior mutability and dynamic aliasing": "D08",
    "language organization and abstraction": "D02",
    "macros and convenience surface": "D14",
    "networking": "D19",
    "numeric values and algorithms": "D01",
    "ownership and borrowing": "D03",
    "ownership and capability predicates": "D03",
    "ownership, layout, and value transitions": "D04",
    "pinning and address-sensitive values": "D23",
    "prelude and reexports": "D25",
    "process environment": "D17",
    "ranges and traversal": "D10",
    "raw pointers and provenance": "D04",
    "recoverable results and variants": "D05",
    "runtime type identity and reflection": "D13",
    "scalar and aggregate values": "D01",
    "sequential and associative data structures": "D09",
    "sequential data structures": "D09",
    "shared ownership and conversions": "D08",
    "shared ownership and weak identity": "D08",
    "static behavior and views": "D03",
    "static behavior contracts": "D03",
    "synchronous I/O": "D15",
    "synchronous iteration": "D10",
    "target intrinsics and dispatch": "D24",
    "target/compiler hints": "D24",
    "threads and thread-local state": "D21",
    "time and clocks": "D20",
    "trap, panic, and diagnostics": "D06",
    "unique ownership and data structures": "D07",
}


def load_rules() -> list[dict[str, str]]:
    with RULES.open(encoding="utf-8", newline="") as handle:
        rows = list(csv.DictReader(handle, delimiter="\t"))
    seen: set[str] = set()
    for row in rows:
        if row["rule_id"] in seen:
            raise SystemExit(f"duplicate classification rule: {row['rule_id']}")
        seen.add(row["rule_id"])
        if not row["match"].startswith(("path:", "regex:", "kind:")):
            raise SystemExit(f"unknown match form in {row['rule_id']}: {row['match']}")
        if row["domain"] not in DOMAIN_IDS:
            raise SystemExit(f"unknown domain in {row['rule_id']}: {row['domain']}")
    return rows


def matching_rules(row: dict[str, str], rules: list[dict[str, str]]) -> list[tuple[int, dict[str, str]]]:
    matches: list[tuple[int, dict[str, str]]] = []
    for rule in rules:
        selector = rule["match"]
        if selector.startswith("path:"):
            prefix = selector[5:]
            if row["item_path"] == prefix or row["item_path"].startswith(prefix + "::"):
                matches.append((3000 + len(prefix), rule))
        elif selector.startswith("regex:"):
            pattern = selector[6:]
            if re.fullmatch(pattern, row["item_path"]):
                matches.append((2000 + len(pattern), rule))
        elif selector.startswith("kind:") and row["item_kind"] == selector[5:]:
            matches.append((1000, rule))
    return matches


def main() -> None:
    rules = load_rules()
    with INVENTORY.open(encoding="utf-8", newline="") as handle:
        raw = list(csv.DictReader(handle, delimiter="\t"))

    by_key: dict[str, dict[str, str]] = {}
    for row in raw:
        if row["stability"] != "stable":
            continue
        key = row["canonical_key"]
        current = by_key.get(key)
        preference = ("core", "alloc", "std").index(row["surface_crate"])
        if current is None:
            by_key[key] = row
            continue
        current_preference = ("core", "alloc", "std").index(current["surface_crate"])
        if (preference, len(row["item_path"]), row["item_path"]) < (
            current_preference,
            len(current["item_path"]),
            current["item_path"],
        ):
            by_key[key] = row

    output: list[dict[str, str]] = []
    unclassified: list[str] = []
    ambiguous: list[str] = []
    for key, row in sorted(by_key.items(), key=lambda pair: (pair[1]["item_path"], pair[1]["member_name"], pair[0])):
        matches = matching_rules(row, rules)
        if not matches:
            unclassified.append(f"{row['item_kind']}\t{row['item_path']}\t{row['member_name']}")
            continue
        top_score = max(score for score, _ in matches)
        winners = [rule for score, rule in matches if score == top_score]
        if len(winners) != 1:
            ambiguous.append(f"{row['item_path']}\t{row['member_name']}\t{','.join(r['rule_id'] for r in winners)}")
            continue
        rule = winners[0]
        output.append(
            {
                "canonical_key": key,
                "representative_path": row["item_path"],
                "item_kind": row["item_kind"],
                "member_kind": row["member_kind"],
                "member_name": row["member_name"],
                "caller_safety": row["caller_safety"],
                "writer_surface_status": (
                    "checked_surface" if row["caller_safety"] == "safe" else "unsafe_evidence_NG"
                ),
                "rule_id": rule["rule_id"],
                "domain_id": DOMAIN_IDS[rule["domain"]],
                "domain": rule["domain"],
                "domain_partition": rule["domain_partition"],
                "canonical_route_or_blocked_claim": rule["canonical_route_or_blocked_claim"],
            }
        )

    if unclassified:
        print("unclassified canonical stable-safe declarations:")
        print("\n".join(unclassified))
        raise SystemExit(1)
    if ambiguous:
        print("ambiguous canonical stable-safe declarations:")
        print("\n".join(ambiguous))
        raise SystemExit(1)
    safety_counts = Counter(row["caller_safety"] for row in output)
    if len(output) != 5369 or safety_counts != {"safe": 5096, "unsafe": 273}:
        raise SystemExit(
            "expected 5369 canonical stable rows (5096 safe, 273 unsafe), "
            f"got {len(output)} and {dict(safety_counts)}"
        )

    fields = list(output[0])
    with OUTPUT.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=fields, delimiter="\t", lineterminator="\n")
        writer.writeheader()
        writer.writerows(output)

    counts = Counter(
        (row["domain_id"], row["domain"], row["caller_safety"], row["domain_partition"])
        for row in output
    )
    with SUMMARY.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.writer(handle, delimiter="\t", lineterminator="\n")
        writer.writerow(
            [
                "domain_id",
                "domain",
                "caller_safety",
                "domain_partition",
                "canonical_stable_declarations",
            ]
        )
        for (domain_id, domain, safety, partition), count in sorted(counts.items()):
            writer.writerow([domain_id, domain, safety, partition, count])

    with MODULES.open(encoding="utf-8", newline="") as handle:
        raw_modules = list(csv.DictReader(handle, delimiter="\t"))
    module_output: list[dict[str, str]] = []
    for row in raw_modules:
        synthetic = {"item_path": row["module_path"], "item_kind": "mod"}
        matches = matching_rules(synthetic, rules)
        if not matches and row["module_path"] in {"core", "alloc", "std"}:
            module_output.append(
                {
                    "crate": row["crate"],
                    "module_path": row["module_path"],
                    "mode": row["mode"],
                    "rule_id": "DOM-MODULE-ROOT",
                    "domain_id": "D25",
                    "domain": "prelude and reexports",
                    "domain_partition": "RED",
                    "entry_digest": row["entry_digest"],
                }
            )
            continue
        if not matches and row["direct_stable_items"] == "0":
            module_output.append(
                {
                    "crate": row["crate"],
                    "module_path": row["module_path"],
                    "mode": row["mode"],
                    "rule_id": "DOM-UNSTABLE-RUNTIME-HOLDING",
                    "domain_id": "D26",
                    "domain": "compiler and runtime support",
                    "domain_partition": "FRAME+LATER+NG",
                    "entry_digest": row["entry_digest"],
                }
            )
            continue
        if not matches:
            raise SystemExit(f"unclassified reachable module: {row['module_path']}")
        top_score = max(score for score, _ in matches)
        winners = [rule for score, rule in matches if score == top_score]
        if len(winners) != 1:
            raise SystemExit(
                f"ambiguous reachable module {row['module_path']}: "
                + ",".join(rule["rule_id"] for rule in winners)
            )
        rule = winners[0]
        module_output.append(
            {
                "crate": row["crate"],
                "module_path": row["module_path"],
                "mode": row["mode"],
                "rule_id": rule["rule_id"],
                "domain_id": DOMAIN_IDS[rule["domain"]],
                "domain": rule["domain"],
                "domain_partition": rule["domain_partition"],
                "entry_digest": row["entry_digest"],
            }
        )
    if len(module_output) != 290:
        raise SystemExit(f"expected 290 reachable modules, got {len(module_output)}")
    with MODULE_OUTPUT.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(
            handle,
            fieldnames=list(module_output[0]),
            delimiter="\t",
            lineterminator="\n",
        )
        writer.writeheader()
        writer.writerows(module_output)

    print(
        "rust domain classification: PASS — 5369 canonical stable declarations "
        "accounted (5096 safe, 273 unsafe); 290 reachable modules routed"
    )


if __name__ == "__main__":
    main()
