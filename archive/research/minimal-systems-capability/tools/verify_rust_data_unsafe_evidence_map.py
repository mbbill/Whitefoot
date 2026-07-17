#!/usr/bin/env python3
"""Verify the exact stable-unsafe data-floor evidence crosswalk."""

from __future__ import annotations

import collections
import csv
import hashlib
from pathlib import Path

from build_rust_data_unsafe_evidence_map import (
    EXPECTED_CLUSTER_COUNTS,
    FIELDS,
    INVENTORY,
    OUTPUT,
    build_rows,
    selected_renderings,
)


EXPECTED_MAP_SHA256 = "c7e43770df4bb534ad02f9b2829b75a4f4c75a771135f427cf4269e30ee7d058"
EXPECTED_KEYSET_SHA256 = "e93833338e17d3d7592b2efa0c4c2bbe23b4c0b36c9eacbd2973a1197e125b22"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"unsafe evidence-map verification failed: {message}")


def read_tsv(path: Path) -> tuple[list[str], list[dict[str, str]]]:
    with path.open(encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        fields = list(reader.fieldnames or [])
        rows = list(reader)
    require(all(None not in row for row in rows), f"extra columns in {path.name}")
    return fields, rows


def main() -> None:
    require(OUTPUT.is_file(), f"missing {OUTPUT.name}")
    require(
        hashlib.sha256(OUTPUT.read_bytes()).hexdigest() == EXPECTED_MAP_SHA256,
        "map bytes differ from the pinned Rust 1.97.0 evidence crosswalk",
    )
    fields, rows = read_tsv(OUTPUT)
    require(fields == FIELDS, "map schema changed")
    require(len(rows) == 35, f"map has {len(rows)} rows, expected 35")
    require(
        all(all(row[field] for field in FIELDS) for row in rows),
        "map has an empty required field",
    )
    require(rows == build_rows(), "map is not the deterministic inventory/census derivation")

    keys = [row["canonical_key"] for row in rows]
    require(len(keys) == len(set(keys)), "a canonical declaration is mapped more than once")
    keyset_bytes = ("\n".join(sorted(keys)) + "\n").encode()
    require(
        hashlib.sha256(keyset_bytes).hexdigest() == EXPECTED_KEYSET_SHA256,
        "canonical stable-unsafe declaration key set changed",
    )

    _, inventory = read_tsv(INVENTORY)
    selected_keys = {row["canonical_key"] for row in selected_renderings(inventory)}
    require(set(keys) == selected_keys, "map omits or adds a selected declaration key")
    require(
        collections.Counter(row["evidence_cluster_id"] for row in rows)
        == EXPECTED_CLUSTER_COUNTS,
        "RAW-UNSAFE evidence-cluster distribution changed",
    )
    require(
        collections.Counter(row["representative_surface_crate"] for row in rows)
        == {"alloc": 14, "core": 20, "std": 1},
        "canonical representative crate distribution changed",
    )
    require(
        collections.Counter(row["selected_seed_rendering_count"] for row in rows)
        == {"1": 34, "2": 1},
        "selected seed rendering multiplicity changed",
    )
    require(
        collections.Counter(row["canonical_rendering_count"] for row in rows)
        == {"2": 33, "3": 2},
        "canonical rendering multiplicity changed",
    )
    for row in rows:
        require(row["stability"] == "stable", f"non-stable row {row['canonical_key']}")
        require(row["caller_safety"] == "unsafe", f"safe row {row['canonical_key']}")
        require(
            row["member_kind"] == "provided_or_inherent_method",
            f"wrong member kind on {row['canonical_key']}",
        )
        require(
            row["evidence_disposition"] == "RAW_EVIDENCE_ONLY_NO_XLANG_SURFACE",
            f"non-evidence disposition on {row['canonical_key']}",
        )
        require(
            row["markers"] == "canonical_stable_unsafe;unsafe_evidence_only",
            f"marker drift on {row['canonical_key']}",
        )
        require(
            row["source_path"] == row["canonical_key"].split("|", 1)[0],
            f"source identity disagrees with canonical key {row['canonical_key']}",
        )

    print(
        "unsafe evidence-map verification: PASS — 35 canonical stable-unsafe "
        "declarations map exactly once to eight evidence-only RAW-UNSAFE clusters"
    )


if __name__ == "__main__":
    main()
