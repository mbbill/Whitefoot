#!/usr/bin/env python3
"""Verify the generated G0 trait-implementation topology routing registry."""

from __future__ import annotations

import collections
import csv
import hashlib
import io
import re

import build_g0_trait_impl_topology_routing as build


EXPECTED_OUTPUT_SHA256 = "162e4af0fb8da0e3be306250e84727f949a28e5956435d61df57633825ab45d0"
EXPECTED_ORDERED_TARGET_SHA256 = "c390eed6aef3b8091f7a87e637f0c60b1734bd66e6af8718ae8258a12379f3f9"
EXPECTED_CLASS_PINS = {
    "CLASS-DENSE": (119, "F-DENSE", "NONE", "F-DENSE", "NONE", "da744a49e860ff7865e60ec3835bc40b07464a89df75a75a67c9514eaef6a608"),
    "CLASS-DEQUE": (22, "F-DEQUE", "NONE", "F-DEQUE", "NONE", "625e1b947a6762793614602f05917501b4e81dbe58fd3167e1b7aa502a74dd4e"),
    "CLASS-DYNAMIC-BORROW": (5, "F-DYNAMIC-BORROW", "NONE", "F-DYNAMIC-BORROW", "NONE", "894076f0fc4e9b0b8d76b4bfdbd5cb1763bad7c47d1709aff198883276681ce1"),
    "CLASS-HEAP": (7, "F-HEAP", "F-DENSE", "F-HEAP", "NONE", "e170eed3e6333a5a90365f15d7ab03fee1ceff77bc8e3e508fef86e9e9633a03"),
    "CLASS-ITERATION": (22, "F-ITERATION", "NONE", "F-ITERATION", "NONE", "bcb4be0bb25f329e02370c773dbe186eccec7fedb5202895cffff845602e9180"),
    "CLASS-LINKED": (14, "GATE-LINKED-COMPOSITION", "F-DENSE,F-IDENTITY,F-RECURSIVE", "NONE", "GATE-LINKED-COMPOSITION", "568c3bd6f7ae28ba7012e943ab52521be1289b0352a9fbd2a0f5c7d58105af04"),
    "CLASS-ORDERED": (27, "F-ORDERED", "F-DENSE,F-IDENTITY,F-RECURSIVE", "F-ORDERED", "NONE", "ffab0aaf622bb3cdb43007b0d90c4d0a34656a9c51350da61777824ec9b1e529"),
    "CLASS-RECURSIVE-BOX": (10, "F-RECURSIVE", "NONE", "F-RECURSIVE", "NONE", "7fbf5a258e12cb575f37308928829a5eac369c3589758521229d538d189c75ce"),
    "CLASS-SHARED": (16, "F-SHARED", "NONE", "F-SHARED", "NONE", "395158b90dc0785e53f627275c82702ef1f78fcf31a643507752e726e64f22ce"),
    "CLASS-SHARED-DENSE": (8, "F-SHARED", "F-DENSE", "F-SHARED", "NONE", "1e12b033252803dd4753727b38f21cc9a954194d37e711357104024783894b59"),
    "CLASS-SHARED-TEXT": (4, "F-SHARED", "F-TEXT", "F-SHARED", "NONE", "95066203e83eea3d80d6b11ef8aae10263eaadbf67b3c95188a670951989b95b"),
    "CLASS-SPARSE": (20, "F-SPARSE", "F-DENSE", "F-SPARSE", "NONE", "d4cff809854f4544ae5289c6d57946de2654fe541a68da81fcdb83eb6d5605a9"),
    "CLASS-TEXT": (60, "F-TEXT", "F-DENSE", "F-TEXT", "NONE", "bc1d08c10e2ceeae083beed253e9f5a8cdf9d04442c50d6ccdbff96e75a6c7a8"),
}

def fail(message: str) -> None:
    raise SystemExit(f"G0 trait-impl topology-routing verification failed: {message}")


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def parse_ids(text: str) -> set[str]:
    return set() if text == "NONE" else set(text.split(","))


def main() -> None:
    expected = build.render(build.build_rows())
    if not build.OUTPUT.is_file():
        fail(f"missing {build.OUTPUT.name}")
    actual = build.OUTPUT.read_text(encoding="utf-8")
    actual_sha256 = sha256_bytes(actual.encode())
    if actual_sha256 != EXPECTED_OUTPUT_SHA256:
        fail(
            "reviewed output hash changed; update the independent pin only after "
            "reviewing the complete regenerated diff"
        )
    if actual != expected:
        fail(f"{build.OUTPUT.name} differs from deterministic regeneration")
    rows = list(csv.DictReader(io.StringIO(actual), delimiter="\t"))
    if list(rows[0]) != build.FIELDS:
        fail("output schema changed")
    if len(rows) != 334:
        fail(f"output has {len(rows)} rows, expected 334")
    if [row["impl_ordinal"] for row in rows] != [str(i) for i in range(1, 335)]:
        fail("impl ordinals are not the exact 1..334 sequence")
    impl_keys = [row["impl_key"] for row in rows]
    if len(set(impl_keys)) != 334 or any(
        not re.fullmatch(r"[0-9a-f]{64}", key) for key in impl_keys
    ):
        fail("impl_key set is not 334 unique SHA-256 identities")
    crosswalk_fields, crosswalk_rows = build.read_tsv(build.CROSSWALK)
    if impl_keys != [row["impl_key"] for row in crosswalk_rows]:
        fail("impl_key set or order differs from the pinned crosswalk")
    if [row["source_row_sha256"] for row in rows] != [
        build.row_sha256(crosswalk_fields, row) for row in crosswalk_rows
    ]:
        fail("one or more source-row identities are stale")

    families, gates, _, vocabulary_sha256 = build.parse_vocabulary()
    for row in rows:
        row_predecessors = parse_ids(row["required_predecessor_family_ids"])
        row_predecessor_gates = parse_ids(row["required_predecessor_gate_stage_ids"])
        row_families = parse_ids(row["implicated_or_reopening_family_ids"])
        row_gates = parse_ids(row["implicated_or_reopening_gate_stage_ids"])
        primary = row["primary_refinement_family_or_gate"]
        if (
            primary not in families | gates
            or not row_predecessors <= families
            or not row_predecessor_gates <= gates
            or not row_families <= families
            or not row_gates <= gates
        ):
            fail(f"{row['impl_key']} uses an ID outside the typed vocabulary")
        if not row_families and not row_gates:
            fail(f"{row['impl_key']} has no topology target")
        if any(
            target.startswith("DIM-")
            for target in row_predecessors
            | row_predecessor_gates
            | row_families
            | row_gates
            | {primary}
        ):
            fail(f"{row['impl_key']} routes to a crosscut dimension")
        if row["vocabulary_sha256"] != vocabulary_sha256:
            fail(f"{row['impl_key']} has a stale vocabulary hash")
        if row["classification_policy"] != build.CLASSIFICATION_POLICY:
            fail(f"{row['impl_key']} weakens the closed-classifier policy")
        if row["topology_scope_policy"] != build.TOPOLOGY_SCOPE_POLICY:
            fail(f"{row['impl_key']} changes the topology-scope policy")

    ordered_target_sha256 = sha256_bytes(
        "".join(
            f"{row['impl_key']}|{row['primary_refinement_family_or_gate']}|"
            f"{row['required_predecessor_family_ids']}|"
            f"{row['required_predecessor_gate_stage_ids']}|"
            f"{row['implicated_or_reopening_family_ids']}|"
            f"{row['implicated_or_reopening_gate_stage_ids']}\n"
            for row in rows
        ).encode()
    )
    if ordered_target_sha256 != EXPECTED_ORDERED_TARGET_SHA256:
        fail("reviewed ordered impl_key-to-target digest changed")

    classifier = build.closed_classifier()
    if set(classifier) != {row["implementer"] for row in rows}:
        fail("closed classifier differs from the routed implementer universe")
    by_classifier: dict[str, list[dict[str, str]]] = collections.defaultdict(list)
    for row in rows:
        expected_classifier_id, exact_class = classifier[row["implementer"]]
        if row["exact_classifier_id"] != expected_classifier_id:
            fail(f"{row['impl_key']} has the wrong exact classifier")
        expected_primary = exact_class.primary_owner_or_gate
        expected_predecessors = set(exact_class.predecessor_family_ids)
        expected_families = {expected_primary} if expected_primary.startswith("F-") else set()
        expected_gates = {expected_primary} if expected_primary.startswith("GATE-") else set()
        if row["primary_refinement_family_or_gate"] != expected_primary:
            fail(f"{row['impl_key']} has the wrong exact primary target")
        if parse_ids(row["required_predecessor_family_ids"]) != expected_predecessors:
            fail(f"{row['impl_key']} has the wrong exact predecessor set")
        if row["required_predecessor_gate_stage_ids"] != "NONE":
            fail(f"{row['impl_key']} has an unexpected predecessor gate")
        if parse_ids(row["implicated_or_reopening_family_ids"]) != expected_families:
            fail(f"{row['impl_key']} has the wrong exact implicated family set")
        if parse_ids(row["implicated_or_reopening_gate_stage_ids"]) != expected_gates:
            fail(f"{row['impl_key']} has the wrong exact implicated gate set")
        by_classifier[expected_classifier_id].append(row)
    if set(by_classifier) != set(build.EXACT_CLASSES):
        fail("one or more exact classifier classes are absent")
    if set(by_classifier) != set(EXPECTED_CLASS_PINS):
        fail("exact classifier set differs from the independent reviewed pins")
    for classifier_id, class_rows in by_classifier.items():
        class_digest = sha256_bytes(
            "".join(f"{row['impl_key']}\n" for row in class_rows).encode()
        )
        if any(
            row["classifier_member_count"] != str(len(class_rows))
            or row["classifier_member_sha256"] != class_digest
            for row in class_rows
        ):
            fail(f"{classifier_id} count/digest aggregate is stale")
        (
            expected_count,
            expected_primary,
            expected_predecessors,
            expected_families,
            expected_gates,
            expected_digest,
        ) = (
            EXPECTED_CLASS_PINS[classifier_id]
        )
        if (
            len(class_rows) != expected_count
            or {row["primary_refinement_family_or_gate"] for row in class_rows}
            != {expected_primary}
            or {row["required_predecessor_family_ids"] for row in class_rows}
            != {expected_predecessors}
            or {row["implicated_or_reopening_family_ids"] for row in class_rows}
            != {expected_families}
            or {row["implicated_or_reopening_gate_stage_ids"] for row in class_rows}
            != {expected_gates}
            or class_digest != expected_digest
        ):
            fail(f"{classifier_id} differs from its independent reviewed pin")

    print(
        "G0 trait-impl topology routing verifier: PASS — exact 334-row set/order, "
        "unique source identities, 142 closed implementers, typed targets, and "
        f"deterministic bytes and independent review pins (sha256={actual_sha256})"
    )


if __name__ == "__main__":
    main()
