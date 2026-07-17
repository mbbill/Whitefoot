#!/usr/bin/env python3
"""Generate and verify the exact historical E0.1 disposition authority."""

from __future__ import annotations

import argparse
import csv
import hashlib
import io
from pathlib import Path

import dense_owner_decisions


HERE = Path(__file__).resolve().parent
G0 = HERE.parent
OUTPUT = HERE / "DENSE-E01-DISPOSITION-AUTHORITY.tsv"
MEMBER_CLASSIFICATION_OUTPUT = HERE / "DENSE-E01-MEMBER-CLASSIFICATION.tsv"
E01 = G0 / "E01-TRACEABILITY.md"
TEMPLATE = G0 / "FAMILY-LOCK-A-TEMPLATE.md"

E01_SHA256 = "2973109ddfee2b6caf8f0b4eedbbbfc55ec933e12c842735dd76c490f106e613"
TEMPLATE_SHA256 = "4da87c54342544e252492f552c5c52b7f41431f3e61bcfd4f7919207d7febc77"

SCHEMA = [
    "e01_input_id",
    "historical_input",
    "template_row_ordinal",
    "disposition",
    "current_authorities",
    "current_authority_sha256s",
    "required_unit_ids",
    "contract_relation",
    "measurement_disposition",
    "blocked_or_separate_claims",
    "new_mandatory_operations_absent_from_e01",
    "e01_traceability_sha256",
    "family_lock_template_sha256",
    "status",
]
MEMBER_CLASSIFICATION_SCHEMA = [
    "member_contract_id",
    "contract_statuses",
    "e01_relation_class",
    "e01_input_ids",
    "exact_contract_present_in_e01",
    "new_mandatory_operation_absent_from_e01",
    "exact_reason",
    "status",
]

EXPECTED_IDS = [f"E01-DISP-{index:02d}" for index in range(1, 14)]
EXPECTED_CANDIDATES = {
    "C-ATOMIC-TRANSITIONS",
    "C-LINEAR-REBUILD",
    "C-DERIVED-REPAIR",
    "C-PROOF-CARRYING-STATE",
    "C-RUNTIME-TOPOLOGY",
}
COPY_MEMBER_IDS = {
    "DENSE-COPY-FROM",
    "DENSE-COPY-WITHIN",
    "DENSE-INIT-COPY",
}
CLONE_MEMBER_IDS = {
    "DENSE-CLONE-FROM",
    "DENSE-CONCAT",
    "DENSE-EXTEND-CLONE",
    "DENSE-EXTEND-WITHIN",
    "DENSE-FILL-CLONE",
    "DENSE-FRESH-CLONE",
    "DENSE-INIT-CLONE",
    "DENSE-JOIN",
    "DENSE-REPEAT",
    "DENSE-RESIZE-CLONE",
}
RAW_EVIDENCE_MEMBER_IDS = {
    "DENSE-ALIGN-EVIDENCE",
    "DENSE-BOX-INIT-EVIDENCE",
    "DENSE-INIT-AUTHORITY-EVIDENCE",
    "DENSE-LEN-AUTHORITY-EVIDENCE",
    "DENSE-RAW-SPARE-REJECTED",
    "DENSE-UNCHECKED-ACCESS-EVIDENCE",
}
LAZY_LIFECYCLE_EVIDENCE_MEMBER_IDS = {
    "DENSE-LAZY-DRAIN-EVIDENCE",
    "DENSE-LAZY-EXTRACT-EVIDENCE",
    "DENSE-LAZY-SPLICE-EVIDENCE",
}


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def exact_member_rows() -> list[dict[str, str]]:
    path = HERE / "DENSE-EXACT-MEMBER-OUTCOME-REGISTRY.tsv"
    with path.open(encoding="utf-8", newline="") as handle:
        return list(csv.DictReader(handle, delimiter="\t"))


def member_classification_rows() -> list[dict[str, str]]:
    source = exact_member_rows()
    by_member: dict[str, list[dict[str, str]]] = {}
    for item in source:
        by_member.setdefault(item["member_contract_id"], []).append(item)
    rows = []
    for member_id in sorted(by_member):
        contracts = by_member[member_id]
        statuses = sorted({item["status"] for item in contracts})
        active = any(status != "EXCLUDED_BLOCKS_NAMED_CLAIM" for status in statuses)
        if active:
            relation_class = "NEW_MANDATORY_EXACT_CONTRACT"
            new_mandatory = "YES"
            relation_ids = {"E01-DISP-09"}
            reason = (
                "At least one exact contract is admitted by the Family Lock. "
                "E0.1 contains no observably equal member/outcome contract, so "
                "historical lineage cannot remove it from the new mandatory set."
            )
        elif member_id in RAW_EVIDENCE_MEMBER_IDS:
            relation_class = "RAW_OR_INITIALIZATION_AUTHORITY_EVIDENCE"
            new_mandatory = "NO"
            relation_ids = {"E01-DISP-10"}
            reason = (
                "This all-excluded member records a raw, unchecked, alignment, "
                "initialization-authority, length-authority, or box evidence route; "
                "it remains rejection evidence rather than a mandatory operation."
            )
        elif member_id in LAZY_LIFECYCLE_EVIDENCE_MEMBER_IDS:
            relation_class = "LAZY_LIFECYCLE_EVIDENCE_NOT_PROMOTED"
            new_mandatory = "NO"
            relation_ids = {"E01-DISP-09"}
            reason = (
                "This all-excluded lazy lifecycle member is OD-4 evidence. It is "
                "not raw-state evidence and is not a promoted mandatory operation."
            )
        else:
            raise SystemExit(f"unclassified all-excluded E0.1 member: {member_id}")

        if member_id in COPY_MEMBER_IDS:
            relation_ids.add("E01-DISP-02")
        if member_id in CLONE_MEMBER_IDS:
            relation_ids.add("E01-DISP-08")
        if member_id in {"DENSE-FILL-WITH", "DENSE-REPEAT", "DENSE-RESIZE-WITH"}:
            relation_ids.add("E01-DISP-07")
        if member_id in {"DENSE-FIXED-EACH", "DENSE-FIXED-MAP", "DENSE-FIXED-VIEW"}:
            relation_ids.add("E01-DISP-06")
        if member_id in {
            "DENSE-INIT-AUTHORITY-EVIDENCE",
            "DENSE-PUSH",
            "DENSE-WITH-CAPACITY",
        }:
            relation_ids.add("E01-DISP-03")

        rows.append({
            "member_contract_id": member_id,
            "contract_statuses": ",".join(statuses),
            "e01_relation_class": relation_class,
            "e01_input_ids": ",".join(sorted(relation_ids)),
            "exact_contract_present_in_e01": "NO",
            "new_mandatory_operation_absent_from_e01": new_mandatory,
            "exact_reason": reason,
            "status": "FROZEN_RESEARCH_CLASSIFICATION_NO_E01_RESTART",
        })
    return rows


def validate_member_classification(rows: list[dict[str, str]]) -> None:
    source = exact_member_rows()
    source_ids = {item["member_contract_id"] for item in source}
    if len(source_ids) != 93 or len(rows) != 93:
        raise SystemExit("E0.1 member classification is not the exact 93-member domain")
    if [row["member_contract_id"] for row in rows] != sorted(source_ids):
        raise SystemExit("E0.1 member classification is missing, duplicate, or unsorted")
    excluded_ids = {
        member_id for member_id in source_ids
        if all(
            item["status"] == "EXCLUDED_BLOCKS_NAMED_CLAIM"
            for item in source if item["member_contract_id"] == member_id
        )
    }
    if excluded_ids != RAW_EVIDENCE_MEMBER_IDS | LAZY_LIFECYCLE_EVIDENCE_MEMBER_IDS:
        raise SystemExit("E0.1 raw/lazy excluded-member partition changed")
    new_ids = {
        row["member_contract_id"] for row in rows
        if row["new_mandatory_operation_absent_from_e01"] == "YES"
    }
    if new_ids != source_ids - excluded_ids or len(new_ids) != 84:
        raise SystemExit("E0.1 new mandatory exact-member set is incomplete")
    clone_ids = {
        item["member_contract_id"] for item in source
        if "clone" in item["behavior_call_count_order_effects"].lower()
    }
    if clone_ids != CLONE_MEMBER_IDS:
        raise SystemExit("E0.1 semantic Clone member set changed")
    copy_ids = {
        item["member_contract_id"] for item in source
        if (
            "COPY" in item["member_contract_id"]
            and any(
                "Copy" in item[field]
                for field in (
                    "pre_state",
                    "post_state",
                    "result_owners",
                    "resource_ceiling",
                )
            )
        )
    }
    if copy_ids != COPY_MEMBER_IDS:
        raise SystemExit("E0.1 semantic Copy member set changed")
    if any(row["exact_contract_present_in_e01"] != "NO" for row in rows):
        raise SystemExit("E0.1 exact-contract equality was asserted without proof")


def new_mandatory_member_bindings() -> tuple[str, ...]:
    rows = member_classification_rows()
    validate_member_classification(rows)
    return tuple(
        f"member:{row['member_contract_id']}" for row in rows
        if row["new_mandatory_operation_absent_from_e01"] == "YES"
    )


def lazy_lifecycle_member_bindings() -> tuple[str, ...]:
    return tuple(
        f"member:{member_id}"
        for member_id in sorted(LAZY_LIFECYCLE_EVIDENCE_MEMBER_IDS)
    )


def new_mandatory_operations_value() -> str:
    return ";".join(new_mandatory_member_bindings())


def authority_binding(*names: str) -> tuple[str, str]:
    paths = [HERE / name for name in names]
    missing = [path.name for path in paths if not path.is_file()]
    if missing:
        raise SystemExit("missing E0.1 disposition authority: " + ", ".join(missing))
    return (
        ";".join(names),
        ";".join(f"{name}={sha256(path)}" for name, path in zip(names, paths)),
    )


def row(
    ordinal: int,
    historical_input: str,
    disposition: str,
    authority_names: tuple[str, ...],
    required_unit_ids: tuple[str, ...],
    contract_relation: str,
    measurement_disposition: str,
    blocked_or_separate_claims: str,
    new_operations: str = "NONE",
) -> dict[str, str]:
    authorities, hashes = authority_binding(*authority_names)
    return {
        "e01_input_id": f"E01-DISP-{ordinal:02d}",
        "historical_input": historical_input,
        "template_row_ordinal": str(ordinal),
        "disposition": disposition,
        "current_authorities": authorities,
        "current_authority_sha256s": hashes,
        "required_unit_ids": ";".join(required_unit_ids) if required_unit_ids else "NONE",
        "contract_relation": contract_relation,
        "measurement_disposition": measurement_disposition,
        "blocked_or_separate_claims": blocked_or_separate_claims,
        "new_mandatory_operations_absent_from_e01": new_operations,
        "e01_traceability_sha256": E01_SHA256,
        "family_lock_template_sha256": TEMPLATE_SHA256,
        "status": "FROZEN_RESEARCH_DISPOSITION_NO_E01_RESTART",
    }


def expected_rows() -> list[dict[str, str]]:
    return [
        row(
            1,
            "Current fixed buffer control",
            "RETAINED_AS_B_FIX",
            ("DENSE-PERFORMANCE-CONTROLS.tsv", "DENSE-PERFORMANCE-STRUCTURAL-GATES.tsv"),
            ("control:B-FIX", "structural:SG-B-FIX"),
            "The unchanged fixed fully initialized buffer remains a protected identity control, not a dense candidate.",
            "Exact source, layout, diagnostics, IR, optimized body, traps, calls, checks, branches, and allocation shape must remain unchanged under every candidate configuration.",
            "B-FIX cannot establish growable or partially live storage and grants no E0.1 restart.",
        ),
        row(
            2,
            "Declarative Copy route",
            "SUPERSEDED_AS_DENSE_MECHANISM_PRODUCTION_COPY_CHOICE_OPEN",
            (
                "DENSE-EXACT-MEMBER-OUTCOME-REGISTRY.tsv",
                "DENSE-E01-MEMBER-CLASSIFICATION.tsv",
                "DENSE-CANDIDATE-LIFECYCLE-REGISTRY.tsv",
                "DENSE-PERFORMANCE-STRUCTURAL-GATES.tsv",
            ),
            tuple(
                f"member:{member_id}" for member_id in sorted(COPY_MEMBER_IDS)
            ) + ("structural:SG-OWNER-EVENTS",),
            "Semantic duplication remains separate from fixed AoS eligibility and partial-state ownership. No declarative-Copy arm is a dense mechanism candidate.",
            "Copy and clone traffic remain explicitly attributed where an exact caller contract requests duplication; they cannot simulate affine relocation.",
            "The production spelling and admissible domain of declared Copy remain unresolved outside this lock.",
        ),
        row(
            3,
            "Affine fixed builder",
            "RETAINED_AS_HISTORICAL_EXACT_FILL_CONTROL_GENERAL_ROUTE_SUPERSEDED",
            ("DENSE-EXACT-MEMBER-OUTCOME-REGISTRY.tsv", "DENSE-CANDIDATE-LIFECYCLE-REGISTRY.tsv"),
            ("member:DENSE-FIXED-MAP", "member:DENSE-INIT-AUTHORITY-EVIDENCE", "member:DENSE-WITH-CAPACITY"),
            "The confined exact-fill builder remains historical positive evidence, but the five general candidates own every normal abandonment edge and arbitrary affine live prefix.",
            "Historical exact-fill observations are not imported as candidate scores. Applicable fixed construction and initialization contracts are remeasured under the current matrix.",
            "The old builder cannot claim unknown-length append, move-out, deletion, growth, arbitrary drop, or general dense closure.",
        ),
        row(
            4,
            "Automatic structural Copy",
            "NOT_SELECTED_ALTERNATIVE_PRODUCTION_CHOICE_OPEN",
            (
                "DENSE-EXACT-MEMBER-OUTCOME-REGISTRY.tsv",
                "DENSE-E01-MEMBER-CLASSIFICATION.tsv",
                "DENSE-CANDIDATE-LIFECYCLE-REGISTRY.tsv",
            ),
            tuple(
                f"member:{member_id}" for member_id in sorted(COPY_MEMBER_IDS)
            ),
            "Structural Copy is neither required nor selected by any dense candidate. Exact duplication contracts remain explicit and distinct from storage eligibility.",
            "No score or performance credit is assigned to inferred Copy; only exact requested duplication sites are counted.",
            "Protocol-token, field-evolution, and copy-provenance questions remain open for a later Copy-classification decision.",
        ),
        row(
            5,
            "Copy by default plus negative affine marker",
            "REJECTION_PRESERVED",
            ("DENSE-CANDIDATE-LIFECYCLE-REGISTRY.tsv", "DENSE-EXACT-MEMBER-OUTCOME-REGISTRY.tsv"),
            (),
            "Omission never grants duplicability. The dense payload envelope admits affine values without changing their semantic copy class.",
            "No default-Copy arm or measurement exists in the frozen candidate set.",
            "Reopening this fail-closed rejection requires separate owner evidence and is outside Family Lock A.",
        ),
        row(
            6,
            "Affine fixed-storage predicate",
            "REVISED_AS_STORAGE_ELIGIBILITY_WITHOUT_COPY",
            (
                "DENSE-EXACT-MEMBER-OUTCOME-REGISTRY.tsv",
                "DENSE-PERFORMANCE-LAYOUTS.tsv",
                "DENSE-PERFORMANCE-PAYLOADS.tsv",
            ),
            ("member:DENSE-FIXED-MAP", "member:DENSE-FIXED-VIEW", "member:DENSE-INIT-AUTHORITY-EVIDENCE"),
            "Target-derived fixed AoS layout eligibility is retained independently of Copy, while arbitrary affine liveness is governed by exact owner-transition contracts.",
            "ROW24, ROW56, and affine payload separators expose layout and ownership costs without inferring semantic duplication.",
            "No source Copy classification follows from layout, no-drop representation, or storage eligibility.",
        ),
        row(
            7,
            "Recursive or single-level recipe",
            "SUPERSEDED_BY_EXPLICIT_CALLABLE_CONSTRUCTION_CONTRACTS",
            ("DENSE-EXACT-MEMBER-OUTCOME-REGISTRY.tsv", "DENSE-COMMON-SUBSTRATE-REGISTRY.tsv"),
            ("member:DENSE-FILL-WITH", "member:DENSE-REPEAT", "member:DENSE-RESIZE-WITH"),
            "Fresh repeated construction uses explicit direct callable contracts and ordinary ANF bindings; no recipe grammar or nested-construction exception is assumed.",
            "Call counts, order, effects, retained state, partial failure, cleanup, and direct-call cost are explicit per contract.",
            "Any future recipe syntax remains a separate META-5 and grammar decision.",
        ),
        row(
            8,
            "Explicit Repeat or Clone",
            "RETAINED_AS_SEPARATE_DUPLICATION_CONTRACT",
            (
                "DENSE-EXACT-MEMBER-OUTCOME-REGISTRY.tsv",
                "DENSE-E01-MEMBER-CLASSIFICATION.tsv",
                "DENSE-PERFORMANCE-STRUCTURAL-GATES.tsv",
            ),
            tuple(
                f"member:{member_id}" for member_id in sorted(CLONE_MEMBER_IDS)
            ) + ("structural:SG-OWNER-EVENTS",),
            "Clone produces new semantic owners under an explicit behavior contract; relocation transfers an existing owner and never invokes Clone.",
            "Every clone call, produced owner, failure edge, old-leaf disposition, and payload byte is counted independently of relocation.",
            "Clone cannot discharge an affine move-out, hole, or live-prefix obligation.",
        ),
        row(
            9,
            "Per-slot builder or initialized-prefix owner",
            "REVISED_CONSTRUCTION_PROTOCOL_SEPARATED_FROM_STEADY_STATE",
            (
                "DENSE-EXACT-MEMBER-OUTCOME-REGISTRY.tsv",
                "DENSE-E01-MEMBER-CLASSIFICATION.tsv",
                "DENSE-CANDIDATE-LIFECYCLE-REGISTRY.tsv",
            ),
            new_mandatory_member_bindings()
            + lazy_lifecycle_member_bindings()
            + (
                "candidate:C-ATOMIC-TRANSITIONS",
                "candidate:C-LINEAR-REBUILD",
                "candidate:C-DERIVED-REPAIR",
                "candidate:C-PROOF-CARRYING-STATE",
                "candidate:C-RUNTIME-TOPOLOGY",
            ),
            "Dense[0,len) is the steady owner state. Construction and mutation may enter only the exact operation-local partial state of the selected candidate and must close every normal exit.",
            "Exact-fill history is replaced by the complete operation matrix, owner events, failure edges, abandonment paths, and same-shape Rust controls.",
            "No affine finish convention, underfill ban, or writer discipline satisfies the general contract.",
            new_mandatory_operations_value(),
        ),
        row(
            10,
            "Public raw or split uninitialized privilege",
            "REJECTION_PRESERVED",
            (
                "DENSE-EXACT-MEMBER-OUTCOME-REGISTRY.tsv",
                "DENSE-E01-MEMBER-CLASSIFICATION.tsv",
                "DENSE-CANDIDATE-OPERATION-REGISTRY.tsv",
            ),
            tuple(
                f"member:{member_id}"
                for member_id in sorted(RAW_EVIDENCE_MEMBER_IDS)
            ),
            "Writer-visible uninitialized bytes, raw spare capacity, unchecked access, forged length, split occupancy/payload mutation, and manual set_len remain statically rejected.",
            "Rejected surfaces have no executable or timed arm; candidate mechanisms expose only checked transitions or unforgeable proof state.",
            "This lock cannot reopen W3, create unsafe library privilege, or weaken a check for speed.",
        ),
        row(
            11,
            "Historical layout and capacity controls",
            "RETAINED_AND_EXPANDED_AS_EXACT_FIXTURES",
            (
                "DENSE-PERFORMANCE-LAYOUTS.tsv",
                "DENSE-PERFORMANCE-GROWTH-POLICIES.tsv",
                "DENSE-PERFORMANCE-MATRIX.tsv",
                "DENSE-PERFORMANCE-STRUCTURAL-GATES.tsv",
            ),
            (
                "layout:LAYOUT-AARCH64-DARWIN",
                "layout:LAYOUT-X86_64-LINUX",
                "layout:LAYOUT-I686-LINUX",
                "growth:GROW-RUST-1.97",
                "growth:GROW-RUST-EXACT-1.97",
                "growth:GROW-ZST-USIZE-MAX",
                "structural:SG-ARITHMETIC-32-64",
            ),
            "AoS versus SoA, full versus prefix, reserve versus growth, and target arithmetic remain separately attributed; no composite subtraction is called a one-variable effect.",
            "The current matrix freezes exact target layout, sizes, capacities, growth trajectories, allocation traffic, byte boundaries, and protected controls.",
            "The result can support only the selected exact target branch and cannot infer universal layout behavior.",
        ),
        row(
            12,
            "Historical numeric and statistical inputs",
            "BENEFIT_MARGINS_RETAINED_INFERENCE_AND_POWER_REPLACED",
            (
                "DENSE-PERFORMANCE-STATISTICS.json",
                "DENSE-PERFORMANCE-PROTOCOL.md",
                "DENSE-PERFORMANCE-STRUCTURAL-GATES.tsv",
            ),
            ("structural:SG-OPERATION-RUST-FLOOR",),
            "The 1.02 per-cell Rust noninferiority margin and 0.90 timing or 0.85 positive-memory benefit margins remain exact; old confidence-bound machinery is superseded.",
            "Primary decisions now use fixed-n strict raw-integer scheduled-mixture successes, exact worst-case tails, global Holm benefit control, and reference-only clustered planning power.",
            "No historical sample, candidate observation, pooled confidence interval, or prior E0.1 score enters this lock.",
        ),
        row(
            13,
            "Capability adoption, xlc migration, and default teaching",
            "SEPARATION_PRESERVED",
            ("dense_owner_decisions.py", "DENSE-PERFORMANCE-OWNER-BRANCHES.tsv"),
            (),
            "The six owner decisions only choose a research branch. Candidate construction, capability adoption, xlc migration, and default teaching remain distinct later authorizations.",
            "No pilot, candidate, score, held-out observation, migration result, or default-writer result exists.",
            "This disposition authorizes none of the three decisions and does not restart E0.1.",
        ),
    ]


def read_tsv(path: Path) -> list[dict[str, str]]:
    with path.open(encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        rows = list(reader)
        if reader.fieldnames != SCHEMA:
            raise SystemExit("E0.1 disposition schema changed")
    return rows


def unit_domains() -> dict[str, set[str]]:
    def ids(filename: str, field: str) -> set[str]:
        with (HERE / filename).open(encoding="utf-8", newline="") as handle:
            return {row[field] for row in csv.DictReader(handle, delimiter="\t")}

    return {
        "control": ids("DENSE-PERFORMANCE-CONTROLS.tsv", "control_id"),
        "structural": ids("DENSE-PERFORMANCE-STRUCTURAL-GATES.tsv", "structural_gate_id"),
        "member": ids("DENSE-EXACT-MEMBER-OUTCOME-REGISTRY.tsv", "member_contract_id"),
        "candidate": ids("DENSE-CANDIDATE-LIFECYCLE-REGISTRY.tsv", "candidate_id"),
        "layout": ids("DENSE-PERFORMANCE-LAYOUTS.tsv", "layout_id"),
        "growth": ids("DENSE-PERFORMANCE-GROWTH-POLICIES.tsv", "growth_policy_id"),
    }


def validate(rows: list[dict[str, str]]) -> None:
    if sha256(E01) != E01_SHA256 or sha256(TEMPLATE) != TEMPLATE_SHA256:
        raise SystemExit("frozen E0.1 traceability or Family Lock template changed")
    if [row["e01_input_id"] for row in rows] != EXPECTED_IDS:
        raise SystemExit("E0.1 disposition does not contain the exact 13-row domain")
    if [row["template_row_ordinal"] for row in rows] != [str(i) for i in range(1, 14)]:
        raise SystemExit("E0.1 template row order changed")
    if rows != expected_rows():
        raise SystemExit("E0.1 disposition differs from the closed declarative authority")
    domains = unit_domains()
    if domains["candidate"] != EXPECTED_CANDIDATES:
        raise SystemExit("E0.1 disposition candidate domain changed")
    for item in rows:
        if item["required_unit_ids"] == "NONE":
            continue
        for binding in item["required_unit_ids"].split(";"):
            namespace, unit_id = binding.split(":", 1)
            if namespace not in domains or unit_id not in domains[namespace]:
                raise SystemExit(f"unknown E0.1 disposition unit: {binding}")
    if rows[4]["disposition"] != "REJECTION_PRESERVED":
        raise SystemExit("copy-by-default rejection was weakened")
    if rows[9]["disposition"] != "REJECTION_PRESERVED":
        raise SystemExit("public raw-state rejection was weakened")
    expected_copy = {
        f"member:{member_id}" for member_id in COPY_MEMBER_IDS
    }
    if (
        set(rows[1]["required_unit_ids"].split(";"))
        != expected_copy | {"structural:SG-OWNER-EVENTS"}
        or set(rows[3]["required_unit_ids"].split(";")) != expected_copy
    ):
        raise SystemExit("Copy lineage bindings are incomplete")
    expected_clone = {
        f"member:{member_id}" for member_id in CLONE_MEMBER_IDS
    }
    if set(rows[7]["required_unit_ids"].split(";")) != (
        expected_clone | {"structural:SG-OWNER-EVENTS"}
    ):
        raise SystemExit("Clone lineage bindings are incomplete")
    expected_new = set(new_mandatory_member_bindings())
    expected_lazy = set(lazy_lifecycle_member_bindings())
    expected_candidates = {
        f"candidate:{candidate_id}" for candidate_id in EXPECTED_CANDIDATES
    }
    if set(rows[8]["required_unit_ids"].split(";")) != (
        expected_new | expected_lazy | expected_candidates
    ):
        raise SystemExit("new dense member and lifecycle bindings are incomplete")
    if set(rows[8]["new_mandatory_operations_absent_from_e01"].split(";")) != expected_new:
        raise SystemExit("new dense operations absent from E0.1 are incomplete")
    expected_raw = {
        f"member:{member_id}" for member_id in RAW_EVIDENCE_MEMBER_IDS
    }
    if set(rows[9]["required_unit_ids"].split(";")) != expected_raw:
        raise SystemExit("raw and initialization-authority evidence is incomplete")
    if expected_lazy & expected_raw:
        raise SystemExit("lazy lifecycle evidence was misclassified as raw state")
    if any(option["selected"] != "UNRESOLVED_OWNER_DECISION" for option in dense_owner_decisions.flattened_decision_rows()):
        raise SystemExit("an owner branch was silently selected")


def encode(rows: list[dict[str, str]]) -> bytes:
    return encode_with_schema(rows, SCHEMA)


def encode_with_schema(rows: list[dict[str, str]], schema: list[str]) -> bytes:
    buffer = io.StringIO(newline="")
    writer = csv.DictWriter(buffer, fieldnames=schema, delimiter="\t", lineterminator="\n")
    writer.writeheader()
    writer.writerows(rows)
    return buffer.getvalue().encode("utf-8")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    arguments = parser.parse_args()
    member_rows = member_classification_rows()
    validate_member_classification(member_rows)
    expected_members = encode_with_schema(member_rows, MEMBER_CLASSIFICATION_SCHEMA)
    if arguments.check:
        if (
            not MEMBER_CLASSIFICATION_OUTPUT.is_file()
            or MEMBER_CLASSIFICATION_OUTPUT.read_bytes() != expected_members
        ):
            raise SystemExit("E0.1 member classification output is absent or stale")
    else:
        MEMBER_CLASSIFICATION_OUTPUT.write_bytes(expected_members)
    rows = expected_rows()
    validate(rows)
    expected = encode(rows)
    if arguments.check:
        if not OUTPUT.is_file() or OUTPUT.read_bytes() != expected:
            raise SystemExit("E0.1 disposition output is absent or stale")
    else:
        OUTPUT.write_bytes(expected)
    print(
        "Dense E0.1 disposition: PASS - 13 historical inputs and 93 exact "
        "members closed, no restart"
    )


if __name__ == "__main__":
    main()
