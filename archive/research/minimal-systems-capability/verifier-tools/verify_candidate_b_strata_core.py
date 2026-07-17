#!/usr/bin/env python3
"""Verify the normalized Candidate B-Strata Phase 1 core and four hard boundaries."""

from __future__ import annotations

import csv
import re
import sys
from collections import defaultdict
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
BASE = (
    ROOT
    / "optimizer-language-research"
    / "implementation"
    / "minimal-systems-capability"
)
CORE = BASE / "CANDIDATE-B-STRATA-CORE.md"
RULES = BASE / "CANDIDATE-B-STRATA-RULES.tsv"
NORMALIZATION = BASE / "CANDIDATE-B-STRATA-NORMALIZATION.tsv"
ORIGINS = BASE / "CANDIDATE-B-STRATA-AUTHORITY-ORIGINS.tsv"
LINEAGE = BASE / "CANDIDATE-B-STRATA-LINEAGE.tsv"
INTERACTIONS = BASE / "CANDIDATE-B-STRATA-INTERACTIONS.tsv"
BOUNDARIES = BASE / "CANDIDATE-B-STRATA-BOUNDARY-PROOFS.tsv"

RULE_FIELDS = (
    "rule_id",
    "family",
    "rule_kind",
    "multiplicity",
    "input_authorities",
    "output_authorities",
    "authority_origin",
    "normal_behavior",
    "recoverable_behavior",
    "abort_behavior",
    "abandonment_behavior",
    "binding_and_invalidators",
    "disposition",
    "checking",
    "erasure",
    "native_need",
    "deletion_witness",
    "hostile_negative",
    "falsifier",
)
NORMALIZATION_FIELDS = (
    "stratum_id",
    "exact_statement",
    "target_rule",
    "dependencies",
    "derivation",
    "primitive_count",
    "runtime_delta",
    "deletion_result",
    "falsifier",
    "status",
)
ORIGIN_FIELDS = (
    "authority_id",
    "kind",
    "multiplicity",
    "binding",
    "origin_class",
    "origin_rule",
    "origin_inputs",
    "consumers",
    "linear_transfers",
    "invalidators",
    "forbidden_producers",
    "origin_rank",
)
LINEAGE_FIELDS = (
    "rule_id",
    "output_lineage_map",
    "affine_equation",
    "true_origins",
    "failure_equation",
    "checking",
    "status",
)
INTERACTION_FIELDS = (
    "authority_id",
    "emitting_rules",
    "accepting_rules",
    "exchange_kind",
    "exchange_condition",
    "boundedness_argument",
    "forbidden_exchange",
    "status",
)
BOUNDARY_FIELDS = (
    "boundary_id",
    "boundary",
    "project",
    "revision",
    "operation_scope",
    "source_anchors",
    "precondition",
    "positive_derivation",
    "output_authority",
    "single_fault",
    "rejection",
    "native_event_manifest",
    "forbidden_delta",
    "scope_control",
    "status",
)

PRIMITIVE_RULES = (
    "K1-NAMES",
    "K1-PLACE",
    "K1-PARTITION",
    "K1-RELAYOUT",
    "K2-LEAF",
    "K2-COMPOSE",
    "K2-CLASSIFY",
    "K2-STRICT",
    "K2-LOAN",
    "K2-CONTROL",
    "K2-CALLABLE",
    "K2-OBSERVER",
    "K2-FACT",
    "K3-OWNER",
    "K3-BORROW",
    "K3-CONTROL",
    "K3-DISPOSE",
    "K3-CALLABLE",
    "K3-ATOMIC",
    "K3-OBSERVE",
    "K3-EXTERNAL",
    "K3-FACT",
    "K3-RELEASE",
)
DERIVED_RULES = tuple(f"D-BS{i}" for i in range(1, 9))
RULE_IDS = PRIMITIVE_RULES + DERIVED_RULES
STRATA = tuple(f"BS-{i}" for i in range(1, 9))
BOUNDARY_PROJECTS = {
    ("NONFORGEABLE-LIVENESS", "Hashbrown"),
    ("NONFORGEABLE-LIVENESS", "mimalloc"),
    ("NONFORGEABLE-LIVENESS", "SQLite"),
    ("NONFORGEABLE-LIVENESS", "Crossbeam Epoch"),
    ("OBSERVER-CLOSURE", "mimalloc"),
    ("OBSERVER-CLOSURE", "Crossbeam Epoch"),
    ("ERASED-AFFINE-ONCE", "Crossbeam Epoch"),
    ("EXACT-EXTERNAL-REPAIR", "SQLite"),
}
REVISIONS = {
    "Hashbrown": "c62a63a61b7caf2de8f9ecb7b06a66b0ab6bdf3d",
    "mimalloc": "30b2d9d89099bee08e9f67a1ffb3e12e7ba45227",
    "SQLite": "92a6c5c3636faa021ecc3be5403a00f50f65eda7",
    "Crossbeam Epoch": "9c3182abebb36bdc9446d75d4644190fef70fa01",
}
PROJECT_TOKENS = ("hashbrown", "mimalloc", "sqlite", "crossbeam")
CJK = re.compile(r"[\u3400-\u4dbf\u4e00-\u9fff\uf900-\ufaff]")


def fail(message: str) -> None:
    raise ValueError(message)


def read_tsv(path: Path, fields: tuple[str, ...]) -> list[dict[str, str]]:
    with path.open(encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        if tuple(reader.fieldnames or ()) != fields:
            fail(f"{path.name} schema mismatch: {reader.fieldnames}")
        rows = list(reader)
    for index, row in enumerate(rows, start=2):
        missing = [field for field in fields if not (row.get(field) or "").strip()]
        if missing:
            fail(f"{path.name}:{index} has empty fields: {missing}")
    return rows


def items(value: str) -> tuple[str, ...]:
    if value in {"-", "BASE", "TERMINAL", "OPTIMIZER-ONLY"}:
        return ()
    return tuple(part.strip() for part in value.split(",") if part.strip())


def require_phrases(path: Path, phrases: tuple[str, ...]) -> None:
    text = path.read_text(encoding="utf-8")
    for phrase in phrases:
        if phrase not in text:
            fail(f"{path.relative_to(ROOT)} omits {phrase!r}")


def main() -> int:
    artifacts = (CORE, RULES, NORMALIZATION, ORIGINS, LINEAGE, INTERACTIONS, BOUNDARIES)
    for path in artifacts:
        text = path.read_text(encoding="utf-8")
        match = CJK.search(text)
        if match:
            fail(f"{path.name} contains non-English CJK prose near {match.start()}")

    rules = read_tsv(RULES, RULE_FIELDS)
    if tuple(row["rule_id"] for row in rules) != RULE_IDS:
        fail("rule inventory or order differs from the frozen 23 primitive plus 8 derived rules")
    rules_by_id = {row["rule_id"]: row for row in rules}
    authorities: set[str] = set()
    producers: defaultdict[str, list[str]] = defaultdict(list)
    consumers: defaultdict[str, list[str]] = defaultdict(list)
    semantic_fields = (
        "family",
        "multiplicity",
        "authority_origin",
        "normal_behavior",
        "recoverable_behavior",
        "abort_behavior",
        "abandonment_behavior",
        "binding_and_invalidators",
        "disposition",
        "checking",
        "erasure",
    )
    for row in rules:
        rule_id = row["rule_id"]
        expected_kind = "derived" if rule_id.startswith("D-") else "primitive"
        if row["rule_kind"] != expected_kind:
            fail(f"{rule_id} has rule_kind {row['rule_kind']!r}, expected {expected_kind!r}")
        if rule_id.startswith("K2-") and not any(
            marker in row["multiplicity"]
            for marker in ("affine", "shared", "scoped", "closed")
        ):
            fail(f"{rule_id} does not fix K2 multiplicity")
        semantic = " ".join(row[field].lower() for field in semantic_fields)
        for token in PROJECT_TOKENS:
            if token in semantic:
                fail(f"{rule_id} semantic fields contain project identity {token!r}")
        if rule_id.startswith("D-"):
            if row["input_authorities"] != "-" or row["output_authorities"] != "-":
                fail(f"{rule_id} analytical derivation must not independently exchange authority")
            continue
        for authority in items(row["input_authorities"]):
            authorities.add(authority)
            consumers[authority].append(rule_id)
        for authority in items(row["output_authorities"]):
            authorities.add(authority)
            producers[authority].append(rule_id)

    origins = read_tsv(ORIGINS, ORIGIN_FIELDS)
    if {row["authority_id"] for row in origins} != authorities:
        missing = sorted(authorities - {row["authority_id"] for row in origins})
        extra = sorted({row["authority_id"] for row in origins} - authorities)
        fail(f"authority-origin inventory mismatch: missing={missing}, extra={extra}")
    origin_by_id = {row["authority_id"]: row for row in origins}
    for row in origins:
        authority = row["authority_id"]
        try:
            rank = int(row["origin_rank"])
        except ValueError:
            fail(f"{authority} has non-integer origin rank")
        for source in items(row["origin_inputs"]):
            if source not in origin_by_id:
                fail(f"{authority} has unknown origin input {source}")
            if int(origin_by_id[source]["origin_rank"]) >= rank:
                fail(f"origin edge {source} -> {authority} is cyclic or not rank-decreasing")
            if source == "A-FACT" and authority != "A-FACT":
                fail(f"fact feeds safety authority {authority}")
        for origin_rule in items(row["origin_rule"]):
            if origin_rule not in PRIMITIVE_RULES:
                fail(f"{authority} names unknown origin rule {origin_rule}")
    if any("A-FACT" in items(row["input_authorities"]) for row in rules):
        fail("a semantic rule accepts A-FACT as authority input")

    lineage = read_tsv(LINEAGE, LINEAGE_FIELDS)
    if tuple(row["rule_id"] for row in lineage) != PRIMITIVE_RULES:
        fail("lineage ledger must cover every primitive exactly once in rule order")
    for row in lineage:
        rule_id = row["rule_id"]
        expected_outputs = set(items(rules_by_id[rule_id]["output_authorities"]))
        if row["output_lineage_map"] == "-":
            mapped_outputs: set[str] = set()
        else:
            mapped_outputs = set()
            for clause in row["output_lineage_map"].split(";"):
                if "<=" not in clause:
                    fail(f"{rule_id} has malformed lineage clause {clause!r}")
                output, sources = clause.split("<=", 1)
                output = output.strip()
                if output in mapped_outputs:
                    fail(f"{rule_id} maps output {output} more than once")
                mapped_outputs.add(output)
                for source in re.findall(r"(?:A|N)-[A-Z]+", sources):
                    if source not in authorities:
                        fail(f"{rule_id} lineage map references unknown authority {source}")
        if mapped_outputs != expected_outputs:
            fail(
                f"{rule_id} lineage outputs differ: expected={sorted(expected_outputs)}, "
                f"mapped={sorted(mapped_outputs)}"
            )
        if row["status"] != "COVERED":
            fail(f"{rule_id} lineage is not COVERED")

    interactions = read_tsv(INTERACTIONS, INTERACTION_FIELDS)
    if {row["authority_id"] for row in interactions} != authorities:
        fail("interaction rows must cover every authority kind exactly once")
    if len(interactions) != len(authorities):
        fail("interaction ledger contains duplicate authority rows")
    for row in interactions:
        authority = row["authority_id"]
        expected_emitters = tuple(producers.get(authority, ()))
        expected_acceptors = tuple(consumers.get(authority, ()))
        actual_emitters = items(row["emitting_rules"])
        actual_acceptors = items(row["accepting_rules"])
        if actual_emitters != expected_emitters:
            fail(
                f"{authority} emitting rules differ: expected={expected_emitters}, "
                f"actual={actual_emitters}"
            )
        if actual_acceptors != expected_acceptors:
            fail(
                f"{authority} accepting rules differ: expected={expected_acceptors}, "
                f"actual={actual_acceptors}"
            )
        if row["status"] != "COVERED":
            fail(f"{authority} interaction is not COVERED")

    normalization = read_tsv(NORMALIZATION, NORMALIZATION_FIELDS)
    if tuple(row["stratum_id"] for row in normalization) != STRATA:
        fail("normalization must map BS-1 through BS-8 exactly once and in order")
    for index, row in enumerate(normalization, start=1):
        if row["target_rule"] != f"D-BS{index}":
            fail(f"BS-{index} targets {row['target_rule']!r}")
        dependencies = items(row["dependencies"])
        if not dependencies or any(rule not in PRIMITIVE_RULES for rule in dependencies):
            fail(f"BS-{index} has unknown or empty primitive dependencies")
        if int(row["primitive_count"]) != len(dependencies):
            fail(f"BS-{index} primitive_count differs from its dependency list")
        if row["runtime_delta"] != "NONE" or row["status"] != "DERIVED":
            fail(f"BS-{index} is not an erased acyclic derivation")

    boundaries = read_tsv(BOUNDARIES, BOUNDARY_FIELDS)
    actual_boundary_projects = {(row["boundary"], row["project"]) for row in boundaries}
    if len(boundaries) != 8 or actual_boundary_projects != BOUNDARY_PROJECTS:
        fail(f"boundary coverage differs: {sorted(actual_boundary_projects)}")
    if len({row["boundary_id"] for row in boundaries}) != len(boundaries):
        fail("boundary proof IDs are not unique")
    for row in boundaries:
        if row["revision"] != REVISIONS[row["project"]]:
            fail(f"{row['boundary_id']} has wrong source revision")
        if row["status"] != "PAPER-PASS":
            fail(f"{row['boundary_id']} is not PAPER-PASS")
        manifest = row["native_event_manifest"].lower()
        if not any(
            marker in manifest
            for marker in ("no added", "added events none", "no per-load event beyond")
        ):
            fail(f"{row['boundary_id']} lacks an explicit zero-added-event manifest")
    for boundary_id in ("QUIESCE-CROSSBEAM", "ONCE-CROSSBEAM"):
        row = next(row for row in boundaries if row["boundary_id"] == boundary_id)
        if "Default bag capacity 64, sanitizer/Miri 4." not in row["native_event_manifest"]:
            fail(f"{boundary_id} does not preserve the native default/sanitizer capacity split")
    sqlite_boundary = next(row for row in boundaries if row["boundary_id"] == "EXTERNAL-SQLITE")
    if "Indeterminate" not in sqlite_boundary["positive_derivation"]:
        fail("SQLite boundary omits conservative indeterminate external effects")
    if "Stable(READER)" not in sqlite_boundary["positive_derivation"]:
        fail("SQLite boundary omits the post-close Stable+Error allocation outcome")
    mimalloc_boundary = next(row for row in boundaries if row["boundary_id"] == "QUIESCE-MIMALLOC")
    if "outside the frozen anchors" not in mimalloc_boundary["scope_control"]:
        fail("mimalloc boundary does not delimit arbitrary-address page-map queries")

    require_phrases(
        CORE,
        (
            "K2 is not a second store of these resources.",
            "unique typed\nview",
            "total lineage map",
            "PlainData ::=",
            "source code\ncannot supply or omit invalidators",
            "A callable gate is exactly",
            "There is no generic provider-control or FFI catch-all.",
            "IngressSet",
            "PublishedCustody",
            "ClosedIngress",
            "that every ingress is closed.",
            "nominal persistent-DAG node",
            "OBS-CLOSE",
            "Waiting for\none, two, or any fixed number of generations cannot repair an ingress",
            "default Crossbeam bag capacity is 64",
            "sanitizer and Miri capacity is 4",
            "Indeterminate",
            "Stable(READER) + Error",
            "solver search, heuristic, timeout, backtracking",
            "Open lineage-conservation blocker",
            "Phase 1 remains fail-closed until the checker rejects both mutations",
        ),
    )
    fail(
        "lineage conservation is not yet enforced: validate sources against "
        "rule inputs or true origins and check branch-local affine multisets"
    )
    print(
        "Candidate B-Strata core: 23 primitives, 8 derived strata, "
        f"{len(authorities)} authority kinds, 8 hard-boundary witnesses, STRATA-CORE-PASS"
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ValueError as error:
        print(f"Candidate B-Strata core verification failed: {error}", file=sys.stderr)
        raise SystemExit(1)
