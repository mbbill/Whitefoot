#!/usr/bin/env python3
"""Build the exact stable-unsafe data-floor evidence crosswalk."""

from __future__ import annotations

import argparse
import collections
import csv
import io
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
INVENTORY = ROOT / "RUST-1.97.0-API-INVENTORY.tsv"
CONTRACTS = ROOT / "RUST-DATA-CONTRACT-CENSUS.tsv"
OUTPUT = ROOT / "RUST-DATA-UNSAFE-EVIDENCE-MAP.tsv"

FIELDS = [
    "canonical_key",
    "representative_surface_crate",
    "representative_module_path",
    "representative_item_path",
    "member_kind",
    "member_name",
    "source_path",
    "representative_docs_path",
    "seed_surface_paths",
    "selected_seed_rendering_count",
    "canonical_rendering_count",
    "stability",
    "caller_safety",
    "evidence_cluster_id",
    "evidence_disposition",
    "markers",
]

SEED_PATHS = {
    "std::array",
    "std::slice",
    "std::str",
    "alloc::boxed::Box",
    "alloc::vec::Vec",
    "alloc::collections::vec_deque::VecDeque",
    "alloc::collections::linked_list::LinkedList",
    "alloc::collections::binary_heap::BinaryHeap",
    "alloc::collections::btree_map::BTreeMap",
    "alloc::collections::btree_set::BTreeSet",
    "std::collections::hash_map::HashMap",
    "std::collections::hash_set::HashSet",
    "alloc::string::String",
    "alloc::rc::Rc",
    "alloc::rc::Weak",
    "core::cell::RefCell",
}

ROUTE_PAIRS = {
    "RAW-UNSAFE-ACCESS-01": {
        ("std::collections::HashMap", "get_disjoint_unchecked_mut"),
        ("core::slice", "as_chunks_unchecked"),
        ("core::slice", "as_chunks_unchecked_mut"),
        ("core::slice", "get_disjoint_unchecked_mut"),
        ("core::slice", "get_unchecked"),
        ("core::slice", "get_unchecked_mut"),
        ("core::slice", "split_at_mut_unchecked"),
        ("core::slice", "split_at_unchecked"),
    },
    "RAW-UNSAFE-ALIGN-01": {
        ("core::slice", "align_to"),
        ("core::slice", "align_to_mut"),
    },
    "RAW-UNSAFE-INIT-01": {
        ("alloc::boxed::Box", "assume_init"),
        ("alloc::rc::Rc", "assume_init"),
        ("core::slice", "assume_init_drop"),
        ("core::slice", "assume_init_mut"),
        ("core::slice", "assume_init_ref"),
    },
    "RAW-UNSAFE-RECONSTRUCT-01": {
        ("alloc::boxed::Box", "from_raw"),
        ("alloc::rc::Rc", "from_raw"),
        ("alloc::rc::Weak", "from_raw"),
        ("alloc::string::String", "from_raw_parts"),
        ("alloc::vec::Vec", "from_raw_parts"),
    },
    "RAW-UNSAFE-LEN-01": {
        ("alloc::vec::Vec", "set_len"),
    },
    "RAW-UNSAFE-TEXT-01": {
        ("alloc::string::String", "as_mut_vec"),
        ("alloc::string::String", "from_utf8_unchecked"),
        ("core::str", "as_bytes_mut"),
        ("core::str", "from_utf8_unchecked"),
        ("core::str", "from_utf8_unchecked_mut"),
        ("core::str", "get_unchecked"),
        ("core::str", "get_unchecked_mut"),
        ("core::str", "slice_mut_unchecked"),
        ("core::str", "slice_unchecked"),
    },
    "RAW-UNSAFE-RC-01": {
        ("alloc::rc::Rc", "decrement_strong_count"),
        ("alloc::rc::Rc", "increment_strong_count"),
    },
    "RAW-UNSAFE-BORROW-01": {
        ("core::cell::RefCell", "try_borrow_unguarded"),
    },
}

EXPECTED_CLUSTER_COUNTS = {
    "RAW-UNSAFE-ACCESS-01": 8,
    "RAW-UNSAFE-ALIGN-01": 2,
    "RAW-UNSAFE-INIT-01": 7,
    "RAW-UNSAFE-RECONSTRUCT-01": 5,
    "RAW-UNSAFE-LEN-01": 1,
    "RAW-UNSAFE-TEXT-01": 9,
    "RAW-UNSAFE-RC-01": 2,
    "RAW-UNSAFE-BORROW-01": 1,
}


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"unsafe evidence-map build failed: {message}")


def read_tsv(path: Path) -> tuple[list[str], list[dict[str, str]]]:
    with path.open(encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        fields = list(reader.fieldnames or [])
        rows = list(reader)
    require(all(None not in row for row in rows), f"extra columns in {path.name}")
    return fields, rows


def selected_renderings(rows: list[dict[str, str]]) -> list[dict[str, str]]:
    return [
        row
        for row in rows
        if row["item_path"] in SEED_PATHS
        and row["member_kind"] == "provided_or_inherent_method"
        and row["stability"] == "stable"
        and row["caller_safety"] == "unsafe"
    ]


def route_for(row: dict[str, str]) -> str:
    pair = (row["item_path"], row["member_name"])
    matches = [cluster_id for cluster_id, pairs in ROUTE_PAIRS.items() if pair in pairs]
    require(
        len(matches) == 1,
        f"{row['canonical_key']} has {len(matches)} RAW-UNSAFE route matches",
    )
    return matches[0]


def build_rows() -> list[dict[str, str]]:
    inventory_fields, inventory = read_tsv(INVENTORY)
    for field in (
        "canonical_key",
        "surface_crate",
        "module_path",
        "item_path",
        "member_kind",
        "member_name",
        "source_path",
        "docs_path",
        "duplicate_of",
        "stability",
        "caller_safety",
    ):
        require(field in inventory_fields, f"inventory lost field {field}")

    contract_fields, contracts = read_tsv(CONTRACTS)
    for field in ("contract_id", "family", "xlang_current_status"):
        require(field in contract_fields, f"contract census lost field {field}")
    contracts_by_id: dict[str, list[dict[str, str]]] = collections.defaultdict(list)
    for contract in contracts:
        contracts_by_id[contract["contract_id"]].append(contract)
    require(
        set(contracts_by_id) >= set(ROUTE_PAIRS),
        "one or more RAW-UNSAFE evidence clusters are absent",
    )
    for cluster_id in ROUTE_PAIRS:
        matches = contracts_by_id[cluster_id]
        require(len(matches) == 1, f"{cluster_id} has {len(matches)} census rows")
        require(
            matches[0]["family"] == "unsafe_implementation_evidence",
            f"{cluster_id} is no longer an unsafe evidence cluster",
        )
        require(
            matches[0]["xlang_current_status"].startswith("Forbidden xlang surface"),
            f"{cluster_id} no longer rejects the Rust-unsafe surface",
        )

    grouped: dict[str, list[dict[str, str]]] = collections.defaultdict(list)
    for rendering in selected_renderings(inventory):
        grouped[rendering["canonical_key"]].append(rendering)
    require(len(grouped) == 35, f"selected {len(grouped)} canonical declarations, expected 35")
    all_renderings: dict[str, list[dict[str, str]]] = collections.defaultdict(list)
    for rendering in inventory:
        if rendering["canonical_key"] in grouped:
            all_renderings[rendering["canonical_key"]].append(rendering)

    output: list[dict[str, str]] = []
    cluster_counts: collections.Counter[str] = collections.Counter()
    for canonical_key in sorted(grouped):
        seed_renderings = grouped[canonical_key]
        canonical_renderings = all_renderings[canonical_key]
        representatives = [row for row in canonical_renderings if not row["duplicate_of"]]
        require(
            len(representatives) == 1,
            f"{canonical_key} has {len(representatives)} canonical representatives",
        )
        representative = representatives[0]
        require(
            all(row["source_path"] == representative["source_path"] for row in canonical_renderings),
            f"{canonical_key} renderings disagree on source identity",
        )
        cluster_id = route_for(representative)
        cluster_counts[cluster_id] += 1
        output.append(
            {
                "canonical_key": canonical_key,
                "representative_surface_crate": representative["surface_crate"],
                "representative_module_path": representative["module_path"],
                "representative_item_path": representative["item_path"],
                "member_kind": representative["member_kind"],
                "member_name": representative["member_name"],
                "source_path": representative["source_path"],
                "representative_docs_path": representative["docs_path"],
                "seed_surface_paths": ";".join(
                    sorted({row["item_path"] for row in seed_renderings})
                ),
                "selected_seed_rendering_count": str(len(seed_renderings)),
                "canonical_rendering_count": str(len(canonical_renderings)),
                "stability": representative["stability"],
                "caller_safety": representative["caller_safety"],
                "evidence_cluster_id": cluster_id,
                "evidence_disposition": "RAW_EVIDENCE_ONLY_NO_XLANG_SURFACE",
                "markers": "canonical_stable_unsafe;unsafe_evidence_only",
            }
        )
    require(
        dict(cluster_counts) == EXPECTED_CLUSTER_COUNTS,
        f"RAW-UNSAFE cluster counts changed: {dict(cluster_counts)}",
    )
    return output


def render(rows: list[dict[str, str]]) -> str:
    output = io.StringIO(newline="")
    writer = csv.DictWriter(output, fieldnames=FIELDS, delimiter="\t", lineterminator="\n")
    writer.writeheader()
    writer.writerows(rows)
    return output.getvalue()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    expected = render(build_rows())
    if args.check:
        require(OUTPUT.is_file(), f"missing generated output {OUTPUT.name}")
        require(
            OUTPUT.read_text(encoding="utf-8") == expected,
            f"{OUTPUT.name} is stale; regenerate it",
        )
        print("unsafe evidence map: PASS — generated output is exact and current")
        return
    OUTPUT.write_text(expected, encoding="utf-8")
    print("unsafe evidence map: wrote 35 exact evidence-only routes")


if __name__ == "__main__":
    main()
