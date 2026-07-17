#!/usr/bin/env python3
"""Verify the frozen targeted Rust 1.97 concrete-trait implementation set."""

from __future__ import annotations

import collections
import csv
import hashlib
import pathlib
import re

from build_trait_impl_crosswalk import FIELDS, OUTPUT, build_rows, render


EXPECTED_TOTAL = 334
EXPECTED_ALL_KEY_DIGEST = "56006d4e5d16d52f1e07f8db1fb3dd307313cf2c31414a011269f094ec318a27"
EXPECTED_FAMILY_COUNTS = {
    "BORROW_PROJECTION": 24,
    "CLONE": 16,
    "COMPARISON_HASH": 78,
    "CONVERSION": 40,
    "DEFAULT": 54,
    "DEREF": 10,
    "DROP": 7,
    "EXTEND": 22,
    "FROM_ITERATOR": 21,
    "INDEX": 14,
    "INTO_ITERATOR": 26,
    "RANGE_STEP": 22,
}
EXPECTED_FAMILY_DIGESTS = {
    "BORROW_PROJECTION": "e87c45d49e152bf4a33298c20fb99b67ab0b28c8c2867f830b66b86d1bda3020",
    "CLONE": "d5646128ae601ef2338b57894308abf6a6173a7741324d7b81179deb0e1b2c6c",
    "COMPARISON_HASH": "febf705cd7d9773b0c0c8e4f67297de1c63f171d2c2d0ba95769d564d683b344",
    "CONVERSION": "41dbe355abf891a1183c50a20daed32b8c8d708feba854eb963b4d1d5a33067a",
    "DEFAULT": "447134fffa9de5a2d4b5051d98a51e4a97af2d95f05f44a45387b5d716094a11",
    "DEREF": "b63727250d4f4e6e7b4c9b0881b6a47cda8950f7a59c2f7865ecd9be405f7d79",
    "DROP": "a35f17f6e8cf8106a2bcc507be9ece1fbee2b31395cce942454946e3f1fbf412",
    "EXTEND": "b7aa6bc1facb817d1731c80f5420e5f9f0807086ec595fd96b76b4c36b08f42b",
    "FROM_ITERATOR": "f0dd0d5477986ed727a07a8b953c4ab2b3eaac234268f26ae92a45d5a0b51c62",
    "INDEX": "1e99c3f7891e9672d56dfcb41c03721e0c28a6aa87ccd7dd347501cced7d971e",
    "INTO_ITERATOR": "a0bdb9ff8aed0d0e6fe8d3bb85ca2ff446c4a73dfdc0ae9b50675076476f95d0",
    "RANGE_STEP": "74e758c544d73322f909da6d0853ae3a74a68e8bc9832f2d55a22a4b175bf14e",
}
EXPECTED_TRAIT_COUNTS = {
    "AsMut": 6,
    "AsRef": 9,
    "Borrow": 5,
    "BorrowMut": 4,
    "Clone": 16,
    "Default": 54,
    "Deref": 6,
    "DerefMut": 4,
    "Drop": 7,
    "Eq": 12,
    "Extend": 22,
    "From": 31,
    "FromIterator": 21,
    "Hash": 10,
    "Index": 8,
    "IndexMut": 6,
    "IntoIterator": 26,
    "Ord": 10,
    "PartialEq": 36,
    "PartialOrd": 10,
    "Step": 22,
    "TryFrom": 9,
}
EXPECTED_STEP_IMPLEMENTERS = {
    "Char",
    "Ipv4Addr",
    "Ipv6Addr",
    "NonZero<u8>",
    "NonZero<u16>",
    "NonZero<u32>",
    "NonZero<u64>",
    "NonZero<u128>",
    "NonZero<usize>",
    "char",
    "i8",
    "i16",
    "i32",
    "i64",
    "i128",
    "isize",
    "u8",
    "u16",
    "u32",
    "u64",
    "u128",
    "usize",
}
EXPECTED_CONTRACTS = {
    "TRAIT-INTOITER-01",
    "TRAIT-EXTEND-01",
    "TRAIT-COLLECT-01",
    "TRAIT-INDEX-01",
    "TRAIT-DEREF-01",
    "TRAIT-BORROW-01",
    "TRAIT-CONVERT-01",
    "TRAIT-CLONE-01",
    "TRAIT-DEFAULT-01",
    "TRAIT-CMP-01",
    "TRAIT-DROP-01",
    "RANGE-ITER-HALFOPEN-01",
    "RANGE-ITER-FROM-01",
    "RANGE-ITER-INCLUSIVE-01",
}


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"trait implementation crosswalk verification failed: {message}")


def digest(keys: list[str]) -> str:
    return hashlib.sha256("\n".join(keys).encode("utf-8")).hexdigest()


def main() -> None:
    with OUTPUT.open(encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        fields = list(reader.fieldnames or [])
        rows = list(reader)
    require(fields == FIELDS, "TSV schema changed")
    require(len(rows) == EXPECTED_TOTAL, f"expected {EXPECTED_TOTAL} rows, got {len(rows)}")
    require(all(None not in row for row in rows), "TSV contains extra columns")
    require(all(all(row[field] for field in FIELDS) for row in rows), "empty required field")
    require(render(build_rows()) == OUTPUT.read_text(encoding="utf-8"), "checked-in TSV is stale")

    keys = [row["impl_key"] for row in rows]
    require(len(keys) == len(set(keys)), "duplicate impl_key")
    require(all(re.fullmatch(r"[0-9a-f]{64}", key) for key in keys), "invalid impl_key")
    require(digest(keys) == EXPECTED_ALL_KEY_DIGEST, "whole selected implementation set changed")
    require(
        rows == sorted(
            rows,
            key=lambda row: (
                row["selection_family"], row["trait_path"], row["impl_signature"],
                row["source_identity"],
            ),
        ),
        "rows are not in canonical order",
    )

    family_counts = collections.Counter(row["selection_family"] for row in rows)
    require(dict(family_counts) == EXPECTED_FAMILY_COUNTS, f"family counts changed: {family_counts}")
    for family, expected_digest in EXPECTED_FAMILY_DIGESTS.items():
        actual = digest([row["impl_key"] for row in rows if row["selection_family"] == family])
        require(actual == expected_digest, f"{family} selected implementation set changed")

    trait_counts = collections.Counter(row["trait_path"].rsplit("::", 1)[-1] for row in rows)
    require(dict(trait_counts) == EXPECTED_TRAIT_COUNTS, f"trait counts changed: {trait_counts}")
    contracts = {
        contract
        for row in rows
        for contract in row["owning_contract_ids"].split(",")
    }
    require(contracts == EXPECTED_CONTRACTS, f"owning contract coverage changed: {contracts}")

    require(
        all(not pathlib.PurePosixPath(row["rustdoc_identity"]).is_absolute() for row in rows),
        "absolute rustdoc path leaked into artifact",
    )
    require(
        all(row["source_identity"].startswith("library/") for row in rows),
        "source identity is not repository-relative",
    )
    require(
        all(re.fullmatch(r"[0-9a-f]{64}", row["source_snippet_sha256"]) for row in rows),
        "invalid source snippet digest",
    )
    require(
        all(
            row["stability"] == "stable" and row["stable_surface_reachable"] == "YES"
            for row in rows if row["selection_family"] != "RANGE_STEP"
        ),
        "selected direct data-floor implementation is not stable and reachable",
    )

    step_rows = [row for row in rows if row["selection_family"] == "RANGE_STEP"]
    require({row["implementer"] for row in step_rows} == EXPECTED_STEP_IMPLEMENTERS, "Step set changed")
    require(
        {row["implementer"] for row in step_rows if row["stable_surface_reachable"] == "YES"}
        == EXPECTED_STEP_IMPLEMENTERS - {"Char"},
        "exact stable-reachable Step endpoint set changed",
    )
    require(sum(row["stable_surface_reachable"] == "YES" for row in step_rows) == 21, "Step stable reachability changed")
    require(
        all(row["ownership_shape"] == "COPY_BORROW_FREE" for row in step_rows),
        "Step endpoint set is no longer pinned as Copy and borrow-free",
    )
    unreachable = [row for row in step_rows if row["stable_surface_reachable"] == "NO"]
    require(
        len(unreachable) == 1
        and unreachable[0]["implementer"] == "Char"
        and "unstable" in unreachable[0]["stable_surface_note"],
        "Step must distinguish 22 implementations from 21 stable-reachable endpoint types",
    )
    require(
        all(
            row["owning_contract_ids"]
            == "RANGE-ITER-HALFOPEN-01,RANGE-ITER-FROM-01,RANGE-ITER-INCLUSIVE-01"
            for row in step_rows
        ),
        "Step row lost one of the three range-iterator owners",
    )

    into_rows = [row for row in rows if row["selection_family"] == "INTO_ITERATOR"]
    require(
        all("type Item =" in row["associated_bindings"] and "type IntoIter =" in row["associated_bindings"] for row in into_rows),
        "IntoIterator row lacks exact Item or IntoIter binding",
    )
    for row in rows:
        trait = row["trait_path"].rsplit("::", 1)[-1]
        if trait == "Index":
            require("type Output =" in row["associated_bindings"], "Index row lacks Output")
        if trait == "Deref":
            require("type Target =" in row["associated_bindings"], "Deref row lacks Target")
        if trait in {"IndexMut", "DerefMut"}:
            require(row["associated_bindings"] == "NONE", f"{trait} fabricated a new associated type")
        if trait != "Eq":
            require(row["required_method_shapes"] != "NONE", f"{trait} lacks exact method/result shape")

    index_counts = collections.Counter(
        row["trait_path"].rsplit("::", 1)[-1]
        for row in rows if row["selection_family"] == "INDEX"
    )
    require(index_counts == {"Index": 8, "IndexMut": 6}, f"Index split changed: {index_counts}")
    deref_rows = [row for row in rows if row["selection_family"] == "DEREF"]
    guard_rows = [row for row in deref_rows if row["implementer"].startswith(("Ref<", "RefMut<"))]
    base_rows = [row for row in deref_rows if row not in guard_rows]
    require(
        collections.Counter(row["trait_path"].rsplit("::", 1)[-1] for row in base_rows)
        == {"Deref": 4, "DerefMut": 3},
        "base-owner Deref split changed",
    )
    require(
        collections.Counter(row["trait_path"].rsplit("::", 1)[-1] for row in guard_rows)
        == {"Deref": 2, "DerefMut": 1},
        "guard Deref supplement changed",
    )

    print(
        "trait implementation crosswalk verification: PASS "
        "(334 rows; Step 22 implementations / 21 stable-reachable)"
    )


if __name__ == "__main__":
    main()
