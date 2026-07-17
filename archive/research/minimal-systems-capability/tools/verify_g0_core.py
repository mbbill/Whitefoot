#!/usr/bin/env python3
"""Verify the complete G0-Core research artifact set."""

from __future__ import annotations

import csv
import hashlib
import json
import re
import subprocess
import sys
from collections import Counter
from pathlib import Path

from build_g0_manifest import ARTIFACTS as MANIFEST_ARTIFACTS


ROOT = Path(__file__).resolve().parent.parent
REPO = ROOT.parents[2]
TOOLS = ROOT / "tools"
MANIFEST = ROOT / "G0-CORE-ARTIFACT-MANIFEST.json"

EXPECTED_ROWS = {
    "CAPABILITY-OBLIGATION-REGISTRY.tsv": 49,
    "RUST-1.97.0-API-INVENTORY.tsv": 17135,
    "RUST-1.97.0-MODULE-ACCOUNTING.tsv": 297,
    "RUST-1.97.0-DOMAIN-CLASSIFICATION.tsv": 5555,
    "RUST-1.97.0-MODULE-DOMAIN-MAP.tsv": 297,
    "RUST-DATA-CONTRACT-CENSUS.tsv": 276,
    "G0-COVERAGE-CLUSTER-REGISTRY.tsv": 276,
    "G0-CLUSTER-FAMILY-ROUTING.tsv": 276,
    "G0-COVERAGE-EVIDENCE-UNIVERSE.tsv": 1961,
    "G0-FAMILY-REQUIREMENT-REGISTRY.tsv": 49,
    "RUST-DATA-SURFACE-MAP.tsv": 545,
    "RUST-DATA-UNSAFE-EVIDENCE-MAP.tsv": 35,
    "RUST-D10-SURFACE-MAP.tsv": 175,
    "RUST-1.97.0-TRAIT-IMPL-CROSSWALK.tsv": 334,
    "G0-TRAIT-IMPL-TOPOLOGY-ROUTING.tsv": 334,
    "DERIVATION-MATRIX.tsv": 276,
    "PAYLOAD-SCOPE-CLASSIFICATION.tsv": 276,
    "PAYLOAD-SCOPE-OVERLAY.tsv": 294,
}

AUTHORED_TEXT = [
    "G0-CORE-CHARTER.md",
    "RUST-CENSUS-NOTES.md",
    "DOMAIN-CLASSIFICATION-RULES.tsv",
    "RUST-1.97.0-DOMAIN-CLASSIFICATION.tsv",
    "RUST-1.97.0-DOMAIN-SUMMARY.tsv",
    "RUST-1.97.0-MODULE-DOMAIN-MAP.tsv",
    "SYSTEMS-DOMAIN-LEDGER.md",
    "RUST-DATA-CONTRACT-CENSUS.md",
    "RUST-DATA-CONTRACT-CENSUS.tsv",
    "G0-COVERAGE-CLUSTER-REGISTRY.tsv",
    "G0-CLUSTER-FAMILY-ROUTING.tsv",
    "G0-COVERAGE-EVIDENCE-UNIVERSE.tsv",
    "G0-FAMILY-REQUIREMENT-REGISTRY.tsv",
    "G0-FAMILY-GATE-VOCABULARY.md",
    "RUST-DATA-SURFACE-MAP.tsv",
    "RUST-DATA-UNSAFE-EVIDENCE-MAP.tsv",
    "RUST-D10-SURFACE-MAP.tsv",
    "RUST-1.97.0-TRAIT-IMPL-CROSSWALK.tsv",
    "G0-TRAIT-IMPL-TOPOLOGY-ROUTING.tsv",
    "RUST-TRAIT-IMPL-CROSSWALK.md",
    "CAPABILITY-OBLIGATION-REGISTRY.tsv",
    "SEMANTIC-OBLIGATION-REGISTRY.md",
    "DERIVATION-MATRIX.tsv",
    "PAYLOAD-SCOPE-CLASSIFICATION.tsv",
    "PAYLOAD-SCOPE-OVERLAY.tsv",
    "WITNESS-REGISTRY.md",
    "E01-TRACEABILITY.md",
    "FAMILY-LOCK-A-TEMPLATE.md",
    "G0-CORE-REPORT.md",
    "tools/extract_rust_api.py",
    "tools/verify_rust_census.py",
    "tools/classify_rust_api.py",
    "tools/build_rust_data_unsafe_evidence_map.py",
    "tools/verify_rust_data_contract_census.py",
    "tools/verify_rust_data_unsafe_evidence_map.py",
    "tools/build_g0_coverage_cluster_registry.py",
    "tools/verify_g0_coverage_cluster_registry.py",
    "tools/build_g0_cluster_family_routing.py",
    "tools/verify_g0_cluster_family_routing.py",
    "tools/build_g0_coverage_evidence_universe.py",
    "tools/verify_g0_coverage_evidence_universe.py",
    "tools/build_g0_family_requirement_registry.py",
    "tools/verify_g0_family_requirement_registry.py",
    "tools/verify_g0_combined_dependency_dag.py",
    "tools/build_d10_surface_map.py",
    "tools/build_trait_impl_crosswalk.py",
    "tools/verify_trait_impl_crosswalk.py",
    "tools/build_g0_trait_impl_topology_routing.py",
    "tools/verify_g0_trait_impl_topology_routing.py",
    "tools/verify_trait_impl_canaries.py",
    "tools/verify_derivation_matrix.py",
    "tools/build_payload_scope_overlay.py",
    "tools/verify_payload_scope_overlay.py",
    "tools/verify_behavior_canaries.py",
    "tools/verify_g0_core.py",
    "tools/build_g0_manifest.py",
    "canaries/README.md",
    "canaries/xlang_behavior_receiver_effects.rs",
    "canaries/xlang_buildhasher_root_swap.rs",
    "canaries/xlang_buildhasher_transfer.rs",
    "canaries/xlang_clone_helper_source_effects.rs",
    "canaries/xlang_clone_source_effects.rs",
    "canaries/xlang_repeat_n_source_effects.rs",
    "canaries/xlang_range_step_stable_entrances.rs",
    "canaries/xlang_range_step_ascii_char_rejected.rs",
    "canaries/xlang_range_step_downstream_impl_rejected.rs",
]


def fail(message: str) -> None:
    raise SystemExit(f"G0-Core verification failed: {message}")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read_tsv(relative: str) -> tuple[list[str], list[dict[str, str]]]:
    path = ROOT / relative
    with path.open(encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        fields = list(reader.fieldnames or [])
        rows = list(reader)
    if any(None in row for row in rows):
        fail(f"extra TSV columns in {relative}")
    if any(any("\r" in value or "\n" in value for value in row.values()) for row in rows):
        fail(f"embedded newline in {relative}")
    return fields, rows


def run_verifier(script: str, *arguments: str) -> None:
    result = subprocess.run(
        [sys.executable, "-B", str(TOOLS / script), *arguments],
        cwd=REPO,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )
    if result.returncode != 0:
        print(result.stdout, end="")
        fail(f"{script} returned {result.returncode}")
    print(result.stdout, end="")


def verify_generated_classifier() -> None:
    generated = [
        ROOT / "RUST-1.97.0-DOMAIN-CLASSIFICATION.tsv",
        ROOT / "RUST-1.97.0-DOMAIN-SUMMARY.tsv",
        ROOT / "RUST-1.97.0-MODULE-DOMAIN-MAP.tsv",
    ]
    before = {path: sha256(path) for path in generated}
    run_verifier("classify_rust_api.py")
    after = {path: sha256(path) for path in generated}
    if before != after:
        fail("generated domain outputs were stale before verification")


def verify_generated_payload_scope() -> None:
    generated = [
        ROOT / "PAYLOAD-SCOPE-CLASSIFICATION.tsv",
        ROOT / "PAYLOAD-SCOPE-OVERLAY.tsv",
    ]
    before = {path: sha256(path) for path in generated}
    run_verifier("build_payload_scope_overlay.py")
    after = {path: sha256(path) for path in generated}
    if before != after:
        fail("generated payload-scope outputs were stale before verification")
    run_verifier("verify_payload_scope_overlay.py")


def verify_generated_coverage_cluster_registry() -> None:
    run_verifier("build_g0_coverage_cluster_registry.py", "--check")


def verify_row_counts() -> None:
    for relative, expected in EXPECTED_ROWS.items():
        _, rows = read_tsv(relative)
        if len(rows) != expected:
            fail(f"{relative} has {len(rows)} rows, expected {expected}")


def verify_derivation_status_counts() -> None:
    _, rows = read_tsv("DERIVATION-MATRIX.tsv")
    counts = Counter(row["status_code"] for row in rows)
    expected = {
        "E": 0,
        "P": 0,
        "U": 15,
        "X": 229,
        "FRAME": 4,
        "DEFERRED": 19,
        "BOUNDARY": 9,
        "NG": 0,
    }
    actual = {status: counts.get(status, 0) for status in expected}
    if actual != expected or set(counts) - set(expected):
        fail(f"unexpected derivation status counts: {dict(counts)}")


def verify_domain_routes() -> None:
    fields, declarations = read_tsv("RUST-1.97.0-DOMAIN-CLASSIFICATION.tsv")
    expected_fields = [
        "canonical_key",
        "representative_path",
        "item_kind",
        "member_kind",
        "member_name",
        "caller_safety",
        "surface_evidence_status",
        "surface_evidence_reason",
        "rule_id",
        "domain_id",
        "domain",
        "canonical_contract_id",
        "need_route_kind",
        "need_route_id",
        "need_route_reason",
        "required_frame_ids",
        "safe_displacement_id",
        "ng_authority_reason",
        "canonical_route_or_blocked_claim",
    ]
    if fields != expected_fields:
        fail(f"unexpected declaration-route schema: {fields}")
    keys = [row["canonical_key"] for row in declarations]
    if len(keys) != len(set(keys)):
        fail("duplicate canonical declaration domain route")
    safety = Counter(row["caller_safety"] for row in declarations)
    if safety != {"safe": 5278, "unsafe": 277}:
        fail(f"unexpected domain safety counts: {dict(safety)}")
    evidence_counts = Counter(row["surface_evidence_status"] for row in declarations)
    if evidence_counts != {
        "safe_contract_anchor": 4792,
        "safe_boundary_evidence": 171,
        "unsafe_boundary_evidence": 277,
        "rust_surface_only": 315,
    }:
        fail(f"unexpected surface evidence counts: {dict(evidence_counts)}")
    route_counts = Counter(row["need_route_kind"] for row in declarations)
    if route_counts != {
        "G0_CONTRACT": 1081,
        "LIB_CONTRACT": 329,
        "LATER_FAMILY": 3876,
        "FRAME": 7,
        "REDUNDANT": 37,
        "NO_INDEPENDENT_NEED": 220,
        "NG": 5,
    }:
        fail(f"unexpected need-route counts: {dict(route_counts)}")

    valid_evidence = {
        "safe_contract_anchor",
        "safe_boundary_evidence",
        "unsafe_boundary_evidence",
        "rust_surface_only",
    }
    valid_routes = {
        "G0_CONTRACT",
        "LIB_CONTRACT",
        "LATER_FAMILY",
        "FRAME",
        "REDUNDANT",
        "NG",
        "NO_INDEPENDENT_NEED",
    }
    valid_frames = {
        "F-MEM",
        "F-ALLOC",
        "F-TRAP",
        "F-BUILD",
        "F-IO",
        "F-FS",
        "F-PROC",
        "F-ABI",
        "F-NET",
        "F-CLOCK",
        "F-THREAD",
        "F-SYNC",
        "F-ASYNC",
        "F-TARGET",
        "F-MMIO",
    }
    _, contracts = read_tsv("RUST-DATA-CONTRACT-CENSUS.tsv")
    contract_ids = {row["contract_id"] for row in contracts}
    _, capabilities = read_tsv("CAPABILITY-OBLIGATION-REGISTRY.tsv")
    capability_ids = {row["capability_id"] for row in capabilities}

    for row in declarations:
        key = row["canonical_key"]
        evidence = row["surface_evidence_status"]
        route = row["need_route_kind"]
        if evidence not in valid_evidence:
            fail(f"invalid surface evidence status for {key}: {evidence}")
        if route not in valid_routes:
            fail(f"invalid need route kind for {key}: {route}")
        if not row["surface_evidence_reason"]:
            fail(f"missing surface evidence reason for {key}")
        if not row["need_route_id"] or not row["need_route_reason"]:
            fail(f"missing need route identity or reason for {key}")
        if row["caller_safety"] == "unsafe" and evidence != "unsafe_boundary_evidence":
            fail(f"Rust-unsafe declaration is not boundary evidence: {key}")
        if evidence == "unsafe_boundary_evidence" and row["caller_safety"] != "unsafe":
            fail(f"unsafe boundary evidence has a safe Rust caller: {key}")
        if evidence == "safe_boundary_evidence" and row["caller_safety"] != "safe":
            fail(f"safe boundary evidence has an unsafe Rust caller: {key}")
        boundary = evidence in {"safe_boundary_evidence", "unsafe_boundary_evidence"}
        if boundary and not row["safe_displacement_id"]:
            fail(f"boundary evidence lacks a safe displacement: {key}")
        if row["safe_displacement_id"].startswith(("RAW-", "NG:")):
            fail(f"safe displacement names raw or non-goal evidence: {key}")
        if not boundary and route != "NG" and row["safe_displacement_id"]:
            fail(f"non-boundary non-NG row has a spurious safe displacement: {key}")
        if route == "NG":
            if not row["ng_authority_reason"]:
                fail(f"NG route lacks owner authority and reason: {key}")
            if not re.search(
                r"(?:CONSTITUTION|owner directive D[0-9]+|EFF-[0-9]+)",
                row["ng_authority_reason"],
            ):
                fail(f"NG route lacks a recognizable authority citation: {key}")
            if not row["safe_displacement_id"]:
                fail(f"NG route lacks a safe displacement: {key}")
        elif row["ng_authority_reason"]:
            fail(f"non-NG route carries an NG authority reason: {key}")

        frames = row["required_frame_ids"].split(";") if row["required_frame_ids"] else []
        if len(frames) != len(set(frames)) or any(frame not in valid_frames for frame in frames):
            fail(f"invalid required frame set for {key}: {frames}")
        if route == "FRAME":
            if not frames or not row["need_route_id"].startswith("FRAME:"):
                fail(f"terminal frame route is not an exact boundary service: {key}")
        elif row["need_route_id"].startswith("FRAME:"):
            fail(f"non-frame route uses a frame route ID: {key}")
        if row["domain_id"] in {"D15", "D16", "D17", "D18", "D19", "D20", "D21", "D22", "D23", "D24"} and route == "FRAME":
            fail(f"OS/FFI/concurrency/target declaration incorrectly terminates at FRAME: {key}")

        if route == "G0_CONTRACT":
            route_id = row["need_route_id"]
            known = route_id in contract_ids
            known = known or (
                route_id.startswith("CAP:") and route_id[4:] in capability_ids
            )
            known = known or bool(re.fullmatch(r"SPEC:[A-Z]+-[0-9]+", route_id))
            if not known:
                fail(f"G0 route lacks a stable contract, obligation, or spec ID: {key}")
        if row["canonical_contract_id"] and row["canonical_contract_id"] not in contract_ids:
            fail(f"unknown canonical contract ID for {key}")
        if not re.fullmatch(r"D(?:0[1-9]|1[0-9]|2[0-5])", row["domain_id"]):
            fail(f"invalid public domain ID {row['domain_id']}")

    ng_keys = {
        row["canonical_key"] for row in declarations if row["need_route_kind"] == "NG"
    }
    if ng_keys != {
        "src/core/intrinsics/mod.rs.html#2980|item|copy_nonoverlapping",
        "src/core/intrinsics/mod.rs.html#2991|item|copy",
        "src/core/intrinsics/mod.rs.html#3002|item|write_bytes",
        "src/std/panic.rs.html#358|item|catch_unwind",
        "src/std/panic.rs.html#390|item|resume_unwind",
    }:
        fail(f"unexpected true non-goal declarations: {sorted(ng_keys)}")

    _, surface_map = read_tsv("RUST-DATA-SURFACE-MAP.tsv")
    declaration_by_key = {row["canonical_key"]: row for row in declarations}
    mapped_contracts = {
        row["canonical_key"]: row["primary_contract_id"] for row in surface_map
    }
    iteration_map_path = ROOT / "RUST-D10-SURFACE-MAP.tsv"
    if iteration_map_path.is_file():
        iteration_fields, iteration_rows = read_tsv("RUST-D10-SURFACE-MAP.tsv")
        if iteration_fields != [
            "canonical_key",
            "representative_path",
            "member_name",
            "route_kind",
            "route_id",
            "route_reason",
        ]:
            fail("unexpected iteration surface-map schema")
        iteration_keys = [row["canonical_key"] for row in iteration_rows]
        if len(iteration_keys) != len(set(iteration_keys)):
            fail("duplicate canonical declaration in the iteration crosswalk")
        ops_range_paths = {
            "core::ops::Bound",
            "core::ops::Range",
            "core::ops::RangeBounds",
            "core::ops::RangeFrom",
            "core::ops::RangeFull",
            "core::ops::RangeInclusive",
            "core::ops::RangeTo",
            "core::ops::RangeToInclusive",
        }
        if sum(row["representative_path"] in ops_range_paths for row in iteration_rows) != 25:
            fail("D10 crosswalk lost the 25 legacy core::ops range declarations")
        peekable_routes = {
            row["member_name"]: row["route_kind"]
            for row in iteration_rows
            if row["representative_path"] == "core::iter::Peekable"
        }
        if peekable_routes != {
            "Peekable": "redundant_surface",
            "next_if": "contract",
            "next_if_eq": "contract",
            "next_if_map": "contract",
            "next_if_map_mut": "contract",
            "peek": "contract",
            "peek_mut": "contract",
        }:
            fail("D10 crosswalk does not distinguish Peekable operations from its type spelling")
        for row in iteration_rows:
            if row["route_kind"] not in {"contract", "redundant_surface"}:
                fail(f"invalid iteration crosswalk route kind for {row['canonical_key']}")
            if row["route_id"] not in contract_ids:
                fail(f"unknown iteration crosswalk contract for {row['canonical_key']}")
            declaration = declaration_by_key.get(row["canonical_key"])
            if declaration is None:
                fail(f"D10 crosswalk declaration is unclassified: {row['canonical_key']}")
            expected_need = (
                "G0_CONTRACT" if row["route_kind"] == "contract" else "REDUNDANT"
            )
            if declaration["domain_id"] != "D10" or declaration["need_route_kind"] != expected_need:
                fail(
                    "independently selected D10 declaration has the wrong domain/route: "
                    f"{row['canonical_key']}"
                )
            if declaration["canonical_contract_id"] != row["route_id"]:
                fail(f"D10 classifier contract differs: {row['canonical_key']}")
            previous = mapped_contracts.get(row["canonical_key"])
            if previous is not None and previous != row["route_id"]:
                fail(f"conflicting data and iteration crosswalk for {row['canonical_key']}")
            mapped_contracts[row["canonical_key"]] = row["route_id"]
    classified_contracts = {
        row["canonical_key"]: row["canonical_contract_id"]
        for row in declarations
        if row["canonical_contract_id"]
    }
    if classified_contracts != mapped_contracts:
        fail("detailed declaration-to-contract routes disagree with the surface map")

    def require_canary(
        path: str, member: str, expected: dict[str, str], minimum: int = 1
    ) -> None:
        matches = [
            row
            for row in declarations
            if row["representative_path"] == path and row["member_name"] == member
        ]
        if len(matches) < minimum:
            fail(f"missing semantic route canary {(path, member)}")
        for row in matches:
            actual = {field: row[field] for field in expected}
            if actual != expected:
                fail(
                    f"semantic route canary {(path, member)} is {actual}, expected {expected}"
                )

    require_canary(
        "alloc::vec::Vec",
        "spare_capacity_mut",
        {
            "surface_evidence_status": "safe_boundary_evidence",
            "domain_id": "D09",
            "need_route_kind": "G0_CONTRACT",
            "need_route_id": "RAW-SAFE-SPARE-01",
            "safe_displacement_id": "CAP:OW-INIT",
        },
    )
    require_canary(
        "core::mem::MaybeUninit",
        "assume_init",
        {
            "surface_evidence_status": "unsafe_boundary_evidence",
            "domain_id": "D04",
            "need_route_kind": "G0_CONTRACT",
            "need_route_id": "CAP:OW-INIT",
            "required_frame_ids": "F-MEM",
            "safe_displacement_id": "CAP:OW-INIT",
        },
    )
    require_canary(
        "core::slice",
        "get_unchecked",
        {
            "surface_evidence_status": "unsafe_boundary_evidence",
            "domain_id": "D09",
            "need_route_kind": "G0_CONTRACT",
            "need_route_id": "CAP:BR-PROV",
            "safe_displacement_id": "CAP:BR-PROV",
        },
    )
    require_canary(
        "core::sync::atomic::Atomic",
        "from_ptr",
        {
            "surface_evidence_status": "unsafe_boundary_evidence",
            "domain_id": "D22",
            "need_route_kind": "LATER_FAMILY",
            "need_route_id": "FAMILY:D22:CHECKED-ATOMIC-ADDRESSING",
            "required_frame_ids": "F-SYNC",
            "safe_displacement_id": "FAMILY:D22:CHECKED-ATOMIC-ADDRESSING",
        },
    )
    require_canary(
        "core::sync::atomic::AtomicPtr",
        "AtomicPtr",
        {
            "surface_evidence_status": "safe_boundary_evidence",
            "domain_id": "D22",
            "need_route_kind": "LATER_FAMILY",
            "need_route_id": "FAMILY:D22:ATOMICS-AND-SYNCHRONIZATION",
            "required_frame_ids": "F-SYNC",
            "safe_displacement_id": "FAMILY:D22:ATOMICS-AND-SYNCHRONIZATION",
        },
    )
    require_canary(
        "alloc::vec::Vec",
        "into_raw_parts",
        {
            "surface_evidence_status": "safe_boundary_evidence",
            "domain_id": "D04",
            "need_route_kind": "LATER_FAMILY",
            "need_route_id": "RAW-SAFE-OWNERSHIP-01",
            "required_frame_ids": "F-MEM",
            "safe_displacement_id": "FAMILY:D04:CHECKED-OWNERSHIP-TRANSFER",
        },
    )
    require_canary(
        "core::mem::forget",
        "forget",
        {
            "surface_evidence_status": "safe_boundary_evidence",
            "domain_id": "D04",
            "need_route_kind": "LATER_FAMILY",
            "need_route_id": "FAMILY:D04:EXPLICIT-RESOURCE-ABANDONMENT",
            "safe_displacement_id": "FAMILY:D04:EXPLICIT-RESOURCE-ABANDONMENT",
        },
    )
    require_canary(
        "alloc::boxed::Box",
        "downcast",
        {
            "domain_id": "D13",
            "need_route_kind": "LATER_FAMILY",
            "need_route_id": "BOX-DOWNCAST-01",
        },
        minimum=3,
    )
    require_canary(
        "alloc::boxed::Box",
        "into_pin",
        {
            "domain_id": "D23",
            "need_route_kind": "LATER_FAMILY",
            "need_route_id": "BOX-PIN-01",
        },
    )
    require_canary(
        "core::pointer",
        "read_volatile",
        {
            "surface_evidence_status": "unsafe_boundary_evidence",
            "domain_id": "D24",
            "need_route_kind": "LATER_FAMILY",
            "need_route_id": "FAMILY:D24:CHECKED-VOLATILE-MMIO",
            "required_frame_ids": "F-MMIO",
            "safe_displacement_id": "FAMILY:D24:CHECKED-VOLATILE-MMIO",
        },
        minimum=2,
    )
    require_canary(
        "std::os::unix::fs::FileExt",
        "read_at",
        {
            "domain_id": "D16",
            "need_route_kind": "LATER_FAMILY",
            "need_route_id": "FAMILY:D16:FILESYSTEMS",
            "required_frame_ids": "F-FS",
        },
    )
    require_canary(
        "std::os::unix::process::CommandExt",
        "exec",
        {
            "domain_id": "D17",
            "need_route_kind": "LATER_FAMILY",
            "need_route_id": "FAMILY:D17:PROCESS-ENVIRONMENT",
            "required_frame_ids": "F-PROC",
        },
    )
    require_canary(
        "core::mem::swap",
        "swap",
        {
            "domain_id": "D04",
            "need_route_kind": "G0_CONTRACT",
            "need_route_id": "CAP:OW-SWAP",
        },
    )
    require_canary(
        "std::is_x86_feature_detected",
        "is_x86_feature_detected",
        {
            "surface_evidence_status": "rust_surface_only",
            "domain_id": "D24",
            "need_route_kind": "LATER_FAMILY",
            "need_route_id": "FAMILY:D24:TARGET-FEATURE-DETECTION",
            "required_frame_ids": "F-TARGET",
        },
    )

    module_fields, modules = read_tsv("RUST-1.97.0-MODULE-DOMAIN-MAP.tsv")
    if module_fields != [
        "crate",
        "module_path",
        "mode",
        "rule_id",
        "domain_id",
        "domain",
        "module_route_kind",
        "module_route_id",
        "entry_digest",
    ]:
        fail(f"unexpected module-route schema: {module_fields}")
    module_paths = [row["module_path"] for row in modules]
    if len(module_paths) != len(set(module_paths)):
        fail("duplicate reachable module domain route")
    if any(
        row["module_route_kind"] != "NO_INDEPENDENT_NEED"
        or not row["module_route_id"].startswith("NAMESPACE:")
        for row in modules
    ):
        fail("module namespace incorrectly claims an independent need")
    domain_ids = {row["domain_id"] for row in declarations + modules}
    expected_ids = {f"D{index:02d}" for index in range(1, 27)}
    if domain_ids != expected_ids:
        fail(f"domain ledger coverage differs: {sorted(domain_ids)}")
    holding = [
        row["module_path"]
        for row in modules
        if row["rule_id"] == "DOM-UNSTABLE-RUNTIME-HOLDING"
    ]
    if holding != ["core::panicking", "core::panicking::panic_const"]:
        fail(f"unexpected unresolved D26 module holding routes: {holding}")


def verify_witness_budgets() -> None:
    witness = (ROOT / "WITNESS-REGISTRY.md").read_text(encoding="utf-8")

    _, capabilities = read_tsv("CAPABILITY-OBLIGATION-REGISTRY.tsv")
    _, contracts = read_tsv("RUST-DATA-CONTRACT-CENSUS.tsv")
    known_ids = {row["capability_id"] for row in capabilities}
    capability_by_id = {row["capability_id"]: row for row in capabilities}
    br_prov_obligation = capability_by_id["BR-PROV"]["proof_obligation"]
    if "cannot outlive that storage" not in br_prov_obligation:
        fail("BR-PROV lost its storage-lifetime prohibition")
    if "and outlives its storage" in br_prov_obligation:
        fail("BR-PROV reverses its storage-lifetime prohibition")
    for subject, capability_id, field, fragments in (
        (
            "per-leaf provenance algebra",
            "BR-PROV",
            "proof_obligation",
            (
                "Every borrowed leaf retains exactly one access-provenance root",
                "finite product or tagged sum of independent singleton leaf relations",
            ),
        ),
        (
            "result product/sum provenance algebra",
            "BR-RESULT",
            "semantic_minimum",
            (
                "Each returned borrow leaf selects exactly one allowed source",
                "product fields and sum branches retain their own tags",
                "never merges, swaps, or widens provenance",
            ),
        ),
        (
            "arbitrary retained-borrow boundary",
            "BR-STORED",
            "semantic_minimum",
            (
                "callable environments, scan state, cached Items, and collection payloads",
                "arbitrary storage or projection requires BR-STORED",
            ),
        ),
        (
            "cursor multi-root authority map",
            "BR-CURSOR",
            "proof_obligation",
            (
                "finite typed field/branch/epoch map of external source authorities",
                "exact footprint per source",
            ),
        ),
        (
            "cursor-only retained authority boundary",
            "BR-CURSOR",
            "semantic_minimum",
            (
                "grants no arbitrary borrow-bearing T, State, callable-environment, cached-Item, or collection field",
                "already yielded external borrow retains its source lifetime unless receiver-bounded",
            ),
        ),
        (
            "stateful-behavior retained-borrow exclusion",
            "AB-STATEFUL",
            "semantic_minimum",
            (
                "AB-STATEFUL grants no retained-borrow storage",
                "borrow-bearing callable environment or separate state additionally requires BR-PROV and BR-STORED",
            ),
        ),
    ):
        value = capability_by_id[capability_id][field]
        missing = [fragment for fragment in fragments if fragment not in value]
        if missing:
            fail(f"{subject} is incomplete: {missing}")
    if "every live incompatible shared or unique access" not in capability_by_id[
        "BR-DISJOINT"
    ]["proof_obligation"]:
        fail("BR-DISJOINT lost live-borrow versus write-footprint coverage")
    if "no result widens authority to the common root" not in capability_by_id[
        "BR-DISJOINT"
    ]["semantic_minimum"]:
        fail("BR-DISJOINT lost its whole-root authority prohibition")
    if "exact footprint per source" not in capability_by_id[
        "BR-CURSOR"
    ]["proof_obligation"]:
        fail("BR-CURSOR lost its footprint limit")
    semantic_registry = (ROOT / "SEMANTIC-OBLIGATION-REGISTRY.md").read_text(
        encoding="utf-8"
    )
    if "that proof never widens authority to the common root" not in semantic_registry:
        fail("G-9 lost the disjoint-footprint authority boundary")
    normalized_semantic_registry = re.sub(r"\s+", " ", semantic_registry)
    for subject, fragment in {
        "allocation-rooted provenance law": "preserve the exact backing-allocation identity and every live borrow rooted in that allocation",
        "token-slot provenance prohibition": "without deriving provenance from the token's slot or address",
        "payload-before-owner-release law": "Every payload is destroyed while its backing owner remains live, before that owner is released",
        "actual allocation-byte law": "Actual acquired bytes, including alignment, allocator rounding, and unused tail, are charged rather than inferred from payload extent alone",
        "dense held-out assignment": "The dense family uses H-FLATSET to exercise the public `ST-AOS`, `ST-DENSE`, and dense ownership transitions under selection without importing `FAM-DENSE`",
        "sparse held-out staging": "After dense adoption, the sparse family uses H-STORE to exercise public `ST-SPARSE` while consuming only the adopted exact dense capabilities, not a completed sequence",
        "clone-from resource-reuse law": "Clone-from updates an already live destination, may reuse its resources, and neither returns nor necessarily destroys the previous whole value",
        "deliberate-leak boundary": "The deliberate-leak contract remains classified `BOUNDARY`",
        "non-lending iterator provenance": "Ordinary yielded references carry pre-existing external provenance and may outlive adapter destruction",
        "receiver-bounded iterator exception": "Receiver-bounded `peek`, `peek_mut`, and `by_ref` results are a separate reborrow layer",
        "per-leaf aggregate provenance": "`BR-PROV` is assigned independently to every borrowed leaf of a result",
        "result provenance composition": "`BR-RESULT` composes those exact per-leaf relations",
        "stored-borrow boundary": "Arbitrary retained borrow-bearing `Item`, seed, callable environment, `State`, cache, or collection payload requires `BR-STORED`",
        "narrow cursor authority": "`BR-CURSOR` is narrower. It grants only an opaque cursor protocol authority",
        "owned cursor-yield distinction": "a cursor may borrow a source while yielding owned, borrow-free values",
        "non-lending unique disjointness": "it requires `BR-DISJOINT` as well as `BR-REBORROW`",
        "pointer-inequality rejection": "pointer inequality alone is never sufficient, including for empty or zero-sized places",
        "array IntoIter interval": "A sealed `[front, back)` interval is the exact live set; both exteriors are dead",
        "RefMut filter-map persistence": "`RefMut::filter_map(None)` returns the original guard with callback mutation retained",
        "RefMut split member scope": "Mutable split consumes its input and creates two member-scoped disjoint unique guards without a parent reborrow",
        "current OOM policy boundary": "Current OOM is a TCB-level condition under OP-9; a recoverable allocator is a separate contract",
    }.items():
        if fragment not in normalized_semantic_registry:
            fail(f"semantic registry lost the {subject}")
    known_ids.update(row["contract_id"] for row in contracts)
    known_ids.update(
        {
            "K-SCALAR",
            "FAM-DENSE",
            "FAM-UMAP",
            "F-MEM",
            "F-ALLOC",
            "F-TRAP",
            "F-BUILD",
            "F-IO",
            "F-FS",
            "F-PROC",
            "F-ABI",
            "F-NET",
            "F-CLOCK",
            "F-THREAD",
            "F-SYNC",
            "F-ASYNC",
            "F-TARGET",
            "F-MMIO",
        }
    )
    known_ids.update(re.findall(r"^\| ((?:B|W)-[A-Z0-9-]+) \|", witness, re.MULTILINE))
    known_ids.update(re.findall(r"^### ((?:H|O)-[A-Z0-9-]+)\b", witness, re.MULTILINE))
    known_ids.update(re.findall(r"^- \*\*((?:O)-[A-Z0-9-]+):", witness, re.MULTILINE))

    identifier = re.compile(
        r"\b(?:ST|OW|EX|BR|FL|ID|AB|IT|FT|BOX|K|FAM|B|W|H|O|F)-[A-Z0-9-]+\b"
    )
    unresolved = sorted(set(identifier.findall(witness)) - known_ids)
    if unresolved:
        fail(f"witness registry contains unresolved dependency IDs: {unresolved}")

    visible_budget_ids: dict[str, list[str]] = {}
    budget_texts: list[str] = []
    visible_start = witness.find("| ID | Role | Frozen observable contract")
    visible_end = witness.find("### 3.1 Exact visible-witness ownership", visible_start)
    if visible_start < 0 or visible_end < 0:
        fail("visible witness dependency table is missing")
    for line in witness[visible_start:visible_end].splitlines():
        if not line.startswith("| W-"):
            continue
        columns = line.split("|")
        if len(columns) != 7:
            fail(f"malformed visible witness row: {line[:80]}")
        budget_texts.append(columns[-2])
        budget_ids = identifier.findall(columns[-2])
        if not budget_ids or len(budget_ids) != len(set(budget_ids)):
            fail(f"empty or duplicate visible witness dependency budget: {columns[1].strip()}")
        visible_budget_ids[columns[1].strip()] = budget_ids

    held_sections = re.findall(r"^### (H-[A-Z0-9-]+)\b", witness, re.MULTILINE)
    expected_held_sections = ["H-FLATSET", "H-STORE", "H-LRU", "H-IPQ"]
    if held_sections != expected_held_sections:
        fail(f"held-out witness sections differ: {held_sections}")

    held_budget_ids: dict[str, list[str]] = {}
    for held_id in expected_held_sections:
        section_match = re.search(
            rf"^### {held_id}\b(?P<body>.*?)(?=^### |\Z)",
            witness,
            re.MULTILINE | re.DOTALL,
        )
        if section_match is None:
            fail(f"missing held-out witness section: {held_id}")
        body = section_match.group("body")
        budget_match = re.search(
            r"(?:Allowed dependencies:|Dependency budget:)\s*(?P<budget>.*?\.)",
            body,
            re.DOTALL,
        )
        if budget_match is None:
            fail(f"missing held-out dependency budget: {held_id}")
        budget_texts.append(budget_match.group("budget"))
        budget_ids = identifier.findall(budget_match.group("budget"))
        if not budget_ids or len(budget_ids) != len(set(budget_ids)):
            fail(f"empty or duplicate held-out dependency budget: {held_id}")
        held_budget_ids[held_id] = budget_ids

    all_budget_ids = [
        dependency
        for budget in list(visible_budget_ids.values()) + list(held_budget_ids.values())
        for dependency in budget
    ]
    if any(re.search(r"\b(?:ST|OW|EX|BR|FL|ID|AB|IT|FT|FAM)-\*", text) for text in budget_texts):
        fail("witness dependency budget contains a wildcard")
    unknown_budget_ids = sorted(set(all_budget_ids) - known_ids)
    if unknown_budget_ids:
        fail(f"witness dependency budgets contain unknown IDs: {unknown_budget_ids}")

    expected_pipe_budget = [
        "K-SCALAR",
        "BR-PROV",
        "BR-REBORROW",
        "BR-RESULT",
        "BR-DISJOINT",
        "BR-INVALIDATE",
        "BR-CURSOR",
        "OW-MOVEOUT",
        "OW-DROP",
        "EX-NORMAL",
        "EX-ABANDON",
        "EX-ABORT",
        "FL-CALLBACK",
        "AB-BEHAVIOR",
        "AB-STATEFUL",
        "AB-GENERIC",
        "IT-SHARED",
        "IT-UNIQ",
        "IT-OWN",
        "IT-COMPOSE",
    ]
    if visible_budget_ids["W-PIPE"] != expected_pipe_budget:
        fail(f"W-PIPE dependency budget differs: {visible_budget_ids['W-PIPE']}")
    normalized_witness = re.sub(r"\s+", " ", witness)
    for subject, fragment in {
        "borrow-free retained F/State/cache boundary": "Every retained callable environment, separate State, and cached or queued Item in this witness is region-free and borrow-free",
        "borrow-bearing state rejection": "Any borrow-bearing callable environment, State, or cached Item is rejected as outside W-PIPE",
        "unique sibling disjointness": "Every non-lending unique Item is disjoint from all still-live sibling Items",
        "chain/zip provenance map": "chain branches and zip fields retain their corresponding sources",
        "overlapping unique-source rejection": "overlapping unique sources are rejected",
        "cursor-map movement": "Moving the cursor preserves the complete provenance map",
    }.items():
        if fragment not in normalized_witness:
            fail(f"W-PIPE lost the {subject}")
    if "BR-STORED" in visible_budget_ids["W-PIPE"]:
        fail("W-PIPE silently widened its borrow-free retained-state budget to BR-STORED")

    direct_allocation_witnesses = {
        "W-POOL",
        "W-ARENA",
        "W-SMALL",
        "W-RECUR",
        "W-GAP",
        "H-FLATSET",
        "H-STORE",
    }
    all_parsed_budgets = visible_budget_ids | held_budget_ids
    missing_allocation_frame = sorted(
        witness_id
        for witness_id in direct_allocation_witnesses
        if "F-ALLOC" not in all_parsed_budgets[witness_id]
    )
    if missing_allocation_frame:
        fail(
            "direct-allocation witnesses omit the public allocator frame: "
            f"{missing_allocation_frame}"
        )
    if "ST-SPARSE" not in held_budget_ids["H-STORE"]:
        fail("H-STORE omits the required public sparse-state dependency")
    for forbidden_dependency in ("FAM-DENSE", "W-POOL"):
        if forbidden_dependency in held_budget_ids["H-STORE"]:
            fail(f"H-STORE imports forbidden completed storage: {forbidden_dependency}")
    if "FAM-DENSE" not in visible_budget_ids["W-ARENA"]:
        fail("W-ARENA omits the public affine backing-owner registry")
    for forbidden_dependency in ("BOX-NEW-01", "W-RECUR"):
        if forbidden_dependency in visible_budget_ids["W-ARENA"]:
            fail(f"W-ARENA imports an unpriced alternate owner registry: {forbidden_dependency}")

    expected_flatset_budget = [
        "K-SCALAR",
        "ST-AOS",
        "ST-DENSE",
        "ST-HOLE",
        "OW-INIT",
        "OW-MOVEOUT",
        "OW-RELOCATE",
        "OW-DROP",
        "EX-NORMAL",
        "EX-ABANDON",
        "EX-ABORT",
        "BR-PROV",
        "BR-REBORROW",
        "BR-RESULT",
        "BR-INVALIDATE",
        "BR-CURSOR",
        "FL-CAPACITY",
        "FL-ALLOC",
        "FL-ATOMIC",
        "FL-CALLBACK",
        "AB-SEAL",
        "AB-BEHAVIOR",
        "AB-GENERIC",
        "IT-SHARED",
        "FT-STATE",
        "F-ALLOC",
    ]
    if held_budget_ids["H-FLATSET"] != expected_flatset_budget:
        fail(
            "H-FLATSET dependency budget differs: "
            f"{held_budget_ids['H-FLATSET']}"
        )
    flatset_section = re.search(
        r"^### H-FLATSET\b(?P<body>.*?)(?=^### H-STORE\b)",
        witness,
        re.MULTILINE | re.DOTALL,
    )
    if flatset_section is None:
        fail("H-FLATSET held-out section is missing")
    flatset_text = flatset_section.group("body")
    for forbidden_dependency in (
        "FAM-DENSE",
        "ST-SPARSE",
        "OW-CLONE",
        "AB-STATEFUL",
        "ST-REFINE",
        "FT-REFINE",
    ):
        if forbidden_dependency in held_budget_ids["H-FLATSET"]:
            fail(f"H-FLATSET imports forbidden dependency: {forbidden_dependency}")
    if "a finished\nsequence/set/map" not in flatset_text:
        fail("H-FLATSET lost the completed-container prohibition")
    normalized_flatset = re.sub(r"\s+", " ", flatset_text)
    flatset_contract_fragments = {
        "affine record payload scope": "arbitrary region-free, borrow-free affine `T`, including drop-bearing record values",
        "exact insert algebra": "`insert(own T)` returns exactly `Inserted`, `Duplicate(own T)`, or `Failure(error, own T)`",
        "sole-owner recovery": "each nonsuccess result is the sole returned owner and leaves the set unchanged",
        "precommit comparison": "search and every comparator call finish before capacity commitment, relocation, or opening `ST-HOLE`",
        "no callback with a hole": "no comparator or other fallible callback runs while a hole is live",
        "broken-law containment": "A broken law may cause a contained logical error, but comparator results never authorize payload liveness",
        "owner-tied get": "returns `None` or an owner-tied shared borrow to the stored equivalent value",
        "safe iterator abandonment": "Abandonment leaves the set valid, owns no payload, and needs no repair",
        "iterator mutation exclusion": "structural mutation is rejected while the iterator or a yielded borrow is live",
        "dense-family assignment": "H-FLATSET is assigned to dense-family closure",
    }
    for subject, fragment in flatset_contract_fragments.items():
        if fragment not in normalized_flatset:
            fail(f"H-FLATSET lost the {subject}")

    required_fragments = {
        "borrow-free payload boundary": "Every B, W, and H retained data value in this registry is region-free and\ncontains no borrow",
        "stored-borrow exclusion": "No current witness budget does so. Opaque source-cursor authority",
        "cursor arbitrary-storage boundary": "projecting or\nstoring that cursor as arbitrary data would require `BR-STORED`",
        "public-token privilege boundary": "Every\nexecutable dependency token grants only the frozen public checked caller contract",
        "closed frame privilege rule": "A frame token likewise grants only its reviewed public checked",
        "arena borrow-safe reset": "statically rejected until every phase borrow ends",
        "arena exact write footprint": "write footprint contains only allocation metadata and fresh dead storage",
        "arena prior-borrow preservation": "each later write footprint is proved disjoint from every prior payload borrow",
        "arena no hidden mutation": "Whole-arena unique access, hidden shared-write/interior mutation, or a runtime borrow table",
        "arena positive preservation trace": "places a second payload while the first borrow is\nlive, and reads the first borrow again",
        "arena negative footprint canaries": "every forged, stale, or overlapping fresh-slot proof",
        "arena no address-family dependency": "without importing the deferred address-stability family",
        "arena required owner registry": "`FAM-DENSE` is a required exercised predecessor used only as the unbounded registry",
        "arena post-dense staging": "W-ARENA is a post-dense composition/access witness and cannot close, select, or adopt `FAM-DENSE`",
        "arena owner-before-payload commit": "committed to that registry before\n`own T` is written",
        "arena failed-registry ownership": "offered `own T` is the sole returned owner, any\nuncommitted block owner is destroyed once",
        "arena no raw owner escape": "no raw or copyable deallocation ticket",
        "arena homogeneous traces": "two distinct homogeneous instantiations",
        "arena regular-route zero terms": "`D_req = D_acq = D_peak = J = 0`",
        "arena oversized-route zero terms": "`R = P = 0` with zero regular-block commits",
        "arena heterogeneous representation exclusion": "Type erasure, runtime type/drop\nmetadata, or a per-placement heterogeneous representation branch",
        "arena exact block destruction": "free every retained block owner exactly once",
        "pool exhaustion policy": "insertion returns `IdentityExhausted(own T)`",
        "pool insertion ceiling": "Insertion is amortized O(1), including identity-exhaustion detection",
        "pool constant non-growing operations": "insertion, shared get, replace, and remove each touch O(1) slots and metadata",
        "pool aggregate history variable": "M is the sum, for the witness being accounted, of the frozen\n"
        "maximum slot counts of every imported W-POOL",
        "pool retired-history charge": "permanently retired identity/history\nslot is charged to M, never to peak live population",
        "ECS identity-exhaustion outcome": "`Failure(IdentityExhausted, all offered owners)`",
        "LRU unique lookup": "lookup takes unique cache access",
        "IPQ held-out heap implementation": "the held-out itself\nimplements heap order",
        "store exhaustive allowlist": "The allowlist above is exhaustive",
        "store exact success algebra": "exactly `Inserted`, `Duplicate(own T)`, or `Failure(error, own T)`",
        "store caller consumption": "caller's input binding is dead once the call begins",
        "store key-universe failure": "`Failure(OutOfUniverse, own T)` unchanged",
        "store required sparse dependency": "`ST-SPARSE` is a required exercised dependency of H-STORE",
        "store sparse-position proposition": "`p = position[k]`, `p < len`,\n`dense_key[p] == k`, and `valid_T(dense_value[p])`",
        "store sparse transition closure": "insert, remove, swap-repair, clear, and destruction\natomically establish, preserve, invalidate, or discharge it",
        "store dense-sparse admissibility": "classic dense/sparse-array representation",
        "store dense-only rejection": "does not\nsubstitute for the public `ST-SPARSE` contract",
        "lifetime-peak allocation accounting": "A `_peak` suffix means the maximum over one owner lifetime, not\n"
        "the final snapshot",
        "failed-attempt accounting": "F includes every call that reports failure and every\n"
        "successful acquisition later rolled back or released",
        "committing-call partition": "A successful acquisition is committing\n"
        "if and only if a valid base or result owner retains it",
        "noncommitting-byte accounting": "Every row also reports Z separately from\n"
        "retained memory and committing acquisition bytes",
        "committing-attempt accounting": "total attempted calls equal those committing calls plus\nF",
        "arena exact fragmentation accounting": "at most one current chunk remainder is omitted from\nP and covered by the separate +C term",
        "arena fragmentation accounting": "`ceil((R+P)/C)+1`",
        "arena static borrow exclusion": "There is no runtime `BorrowLive` result",
        "arena complete-payload scope": "No partial aggregate enters arena storage",
        "recursive failure atomicity": "failure occurs before commitment and returns every offered owner unchanged",
        "destruction allocation prohibition": "Destruction performs no fallible allocation",
        "witness code-size ceiling": "reachable\nwitness-specific machine code for each W or H contract must be O(1)",
        "metadata allocation-call ceiling": "metadata backing acquisitions with a lock-frozen constant",
    }
    for subject, fragment in required_fragments.items():
        if fragment not in witness:
            fail(f"witness registry lost the {subject}")
    for forbidden_fragment in (
        "A mixed trace concurrently",
        "multiple regular\nchunks and multiple dedicated oversized blocks",
    ):
        if forbidden_fragment in witness:
            fail(f"W-ARENA retains an impossible mixed-route trace: {forbidden_fragment}")
    for subject, fragment in {
        "homogeneous owner type": "one arena instantiation and owner store one fixed `T`",
        "whole-instantiation route": "selects the route for that entire instantiation",
        "no mixed owner": "One arena owner never mixes the two classes",
    }.items():
        if fragment not in normalized_witness:
            fail(f"W-ARENA lost the {subject}")
    if normalized_witness.count("forces at least two owner-registry growth events") != 2:
        fail("W-ARENA must exercise two registry growth events on each homogeneous route")
    arena_hardening_fragments = {
        "allocation-rooted arena provenance": "Each payload borrow is rooted in the arena owner and exact retained backing allocation, never in a registry slot or token address",
        "token-relocation preservation": "relocation changes neither backing-allocation identity nor live payload-borrow provenance",
        "token relocation is not evidence": "token relocation alone is not evidence",
        "payload-before-block destruction": "Only after the last payload in a block is destroyed may that sealed block owner be released, exactly once",
        "registry provenance canaries": "registry-slot/token-address provenance, borrow retargeting or invalidation after token relocation",
        "dedicated slack canaries": "dedicated-block alignment/rounding/slack, arbitrary dedicated over-allocation",
        "dedicated actual-byte definition": "D_acq is their lifetime sum of actual acquired usable bytes, including start-alignment padding, allocator rounding, and unused dedicated tail",
        "dedicated peak definition": "D_peak is the maximum simultaneously retained portion of those actual bytes",
        "dedicated growth bound": "D_acq <= alpha*D_req + beta*J",
    }
    for subject, fragment in arena_hardening_fragments.items():
        if fragment not in normalized_witness:
            fail(f"witness registry lost the {subject}")
    arena_row = next(line for line in witness.splitlines() if line.startswith("| W-ARENA "))
    if "ID-ADDRESS" in arena_row:
        fail("W-ARENA imports the deferred physical-address capability")
    for required_dependency in ("BR-DISJOINT", "BR-CURSOR"):
        if required_dependency not in visible_budget_ids["W-ARENA"]:
            fail(f"W-ARENA omits {required_dependency} for footprint-limited placement")
    if "FAM-HEAP" in witness:
        fail("H-IPQ imports a finished heap instead of testing reverse-index repair")
    for ambiguous in (
        "returns or preserves",
        "preserves the original owner and offered input",
        "with the offered owner unchanged",
        "returns `BorrowLive`",
    ):
        if ambiguous in witness:
            fail(f"witness registry retains ambiguous affine failure wording: {ambiguous}")

    resource_start = witness.find("### 3.2 Coarse mandatory resource envelopes")
    resource_end = witness.find("### 3.3 Visible controls", resource_start)
    if resource_start < 0 or resource_end < 0:
        fail("witness resource-envelope section is missing")
    resource_section = witness[resource_start:resource_end]
    resource_rows: list[str] = [
        line
        for line in resource_section.splitlines()
        if re.match(r"^\| (?:W|H)-[A-Z0-9-]+ \|", line)
    ]
    resource_ids: list[str] = []
    resource_by_id: dict[str, list[str]] = {}
    for line in resource_rows:
        columns = line.split("|")
        if len(columns) != 7 or any(not column.strip() for column in columns[1:6]):
            fail(f"malformed or empty witness resource row: {line[:80]}")
        resource_id = columns[1].strip()
        resource_ids.append(resource_id)
        resource_by_id[resource_id] = [column.strip() for column in columns[1:6]]
        if resource_id != "W-PIPE" and re.search(r"\bF\b", columns[4]) is None:
            fail(f"allocating witness omits failed-attempt accounting: {resource_id}")
    expected_resource_ids = {
        "W-POOL",
        "W-ARENA",
        "W-SMALL",
        "W-RECUR",
        "W-GRAPH",
        "W-ECS",
        "W-GAP",
        "W-PIPE",
        "H-STORE",
        "H-FLATSET",
        "H-LRU",
        "H-IPQ",
    }
    if set(resource_ids) != expected_resource_ids or len(resource_ids) != len(set(resource_ids)):
        fail(f"witness resource envelopes differ: {sorted(resource_ids)}")
    flatset_resource = resource_by_id["H-FLATSET"]
    for field_name, value, fragments in (
        ("persistent", flatset_resource[1], ("O(c)", "contiguous payload backing", "c=O(1+n_peak)")),
        ("peak", flatset_resource[2], ("O(c_peak)", "old/new backing transient", "O(1) operation-local state")),
        ("allocations", flatset_resource[3], ("O(1 + log(1+n_peak))", "add F", "add Z", "no per-element allocation")),
        ("traffic", flatset_resource[4], ("O(log n)", "O(n-position)", "Growth relocation is charged separately", "one O(n) pass", "No second linear search or extra full rebuild")),
    ):
        missing = [fragment for fragment in fragments if fragment not in value]
        if missing:
            fail(f"H-FLATSET {field_name} accounting omits {missing}")
    if "plus the imported W-POOL's separately reported committing allocation bound" not in resource_section:
        fail("W-ECS omits imported pool allocation accounting")
    pool_importers = {
        witness_id
        for witness_id, budget in all_parsed_budgets.items()
        if "W-POOL" in budget
    }
    if pool_importers != {"W-GRAPH", "W-ECS", "H-LRU"}:
        fail(f"unexpected W-POOL importer set: {sorted(pool_importers)}")
    dense_importers = {
        witness_id
        for witness_id, budget in all_parsed_budgets.items()
        if "FAM-DENSE" in budget
    }
    if dense_importers != {"W-ARENA", "W-GRAPH", "W-ECS", "H-IPQ"}:
        fail(f"unexpected FAM-DENSE importer set: {sorted(dense_importers)}")
    for importer in sorted(pool_importers):
        _, persistent, _, allocations, _ = resource_by_id[importer]
        if re.search(r"\bM\b", persistent) is None or "history" not in persistent:
            fail(f"{importer} omits imported pool retained-history accounting")
        if "W-POOL" not in allocations:
            fail(f"{importer} omits imported pool allocation accounting")
    pool_traffic = resource_by_id["W-POOL"][4]
    for fragment in ("non-growing insert", "O(1)", "vacancy", "retired history", "may scan"):
        if fragment not in pool_traffic:
            fail(f"W-POOL insertion accounting omits {fragment!r}")
    ecs_persistent = resource_by_id["W-ECS"][1]
    if "+ M" not in ecs_persistent or "not a peak-live-only bound" not in ecs_persistent:
        fail("W-ECS does not charge imported identity history independently of peak live population")
    lru_persistent = resource_by_id["H-LRU"][1]
    if "O(C + M) = O(C)" not in lru_persistent or "M = O(C)" not in lru_persistent:
        fail("H-LRU does not prove its imported pool history remains O(C)")
    arena_resource = resource_by_id["W-ARENA"]
    _, arena_persistent, arena_peak, arena_allocations, arena_traffic = arena_resource
    for field_name, value, fragments in (
        (
            "persistent",
            arena_persistent,
            (
                "O(K_peak)",
                "FAM-DENSE",
                "sealed block-owner metadata",
                "actual acquired usable bytes",
                "D_req",
                "D_acq",
                "D_peak",
            ),
        ),
        (
            "peak",
            arena_peak,
            (
                "O(K_peak)",
                "old/new owner-token storage",
                "actual usable bytes",
                "alignment padding",
                "allocator rounding",
                "unused tail",
                "moves no payload bytes",
            ),
        ),
        (
            "allocations",
            arena_allocations,
            (
                "J is exactly",
                "one per successfully placed oversized request",
                "FAM-DENSE",
                "O(1 + log(1 + K_peak))",
                "add F",
                "Z includes",
            ),
        ),
        (
            "traffic",
            arena_traffic,
            (
                "move owner tokens but zero payload bytes or backing allocations",
                "before releasing its retained block owner",
                "D_req",
                "D_acq",
                "D_peak",
                "J",
                "K_peak",
                "FAM-DENSE",
            ),
        ),
    ):
        missing = [fragment for fragment in fragments if fragment not in value]
        if missing:
            fail(f"W-ARENA {field_name} accounting omits {missing}")

    family_template = (ROOT / "FAMILY-LOCK-A-TEMPLATE.md").read_text(encoding="utf-8")
    normalized_family_template = re.sub(r"\s+", " ", family_template)
    for fragment in (
        "required predecessor DAG",
        "cannot be an adoption or closure gate for that same imported family",
        "post-adoption witness stage",
        "Using a capability selected by another family creates a predecessor edge even when no `FAM-X` token is imported",
        "A held-out assigned to the current family may exercise capabilities being selected in that lock, but it may not import that family's completed `FAM-X` container",
    ):
        if fragment not in normalized_family_template:
            fail(f"family-lock template lost imported-family staging rule: {fragment}")
    if "exact subset of H-FLATSET, H-STORE, H-LRU, H-IPQ" not in normalized_family_template:
        fail("family-lock template lost the exact four-held-out custody set")

    charter = (ROOT / "G0-CORE-CHARTER.md").read_text(encoding="utf-8")
    report = (ROOT / "G0-CORE-REPORT.md").read_text(encoding="utf-8")
    e01 = (ROOT / "E01-TRACEABILITY.md").read_text(encoding="utf-8")
    normalized_charter = re.sub(r"\s+", " ", charter)
    normalized_report = re.sub(r"\s+", " ", report)
    normalized_e01 = re.sub(r"\s+", " ", e01)
    for artifact, normalized, fragment in (
        ("G0-Core charter", normalized_charter, "four training-excluded held-out witnesses"),
        ("G0-Core report", normalized_report, "plus four separately budgeted held-outs"),
        ("E0.1 traceability", normalized_e01, "ordinary-library derivation and H-FLATSET"),
        ("E0.1 traceability", normalized_e01, "H-STORE requires public `ST-SPARSE`; it is explicitly deferred to sparse-family closure after dense adoption"),
    ):
        if fragment not in normalized:
            fail(f"{artifact} lost the held-out staging claim: {fragment}")

    ledger = (ROOT / "SYSTEMS-DOMAIN-LEDGER.md").read_text(encoding="utf-8")
    alloc_frame_match = re.search(r"^\| `F-ALLOC` \|(?P<body>.*)\|$", ledger, re.MULTILINE)
    if alloc_frame_match is None:
        fail("F-ALLOC frame accounting row is missing")
    alloc_frame_text = re.sub(r"\s+", " ", alloc_frame_match.group("body"))
    missing_alloc_frame = sorted(
        fragment
        for fragment in {
            "allocation-owner identity",
            "owner transfer that does not relocate the owned allocation",
            "actual acquired usable bytes and allocator rounding/slack",
        }
        if fragment not in alloc_frame_text
    )
    if missing_alloc_frame:
        fail(f"F-ALLOC frame accounting is incomplete: {missing_alloc_frame}")
    frame_match = re.search(r"^\| `F-MMIO` \|(?P<body>.*)\|$", ledger, re.MULTILINE)
    d24_match = re.search(r"^### D24\..*?(?=^### D25\.)", ledger, re.MULTILINE | re.DOTALL)
    if frame_match is None or d24_match is None:
        fail("F-MMIO frame or D24 accounting section is missing")
    frame_text = re.sub(r"\s+", " ", frame_match.group("body"))
    d24_text = re.sub(r"\s+", " ", d24_match.group(0))
    frame_fragments = {
        "exact source-path access-event multiplicity",
        "speculative, duplicated, fused, split, widened, narrowed, or elided accesses",
        "ordering against both relevant ordinary-memory and device accesses",
        "external device mutation and attenuation of cached facts",
        "mapping lifetime, and invalidation",
        "event identity, and atomicity/tearing",
        "including DMA",
        "fault/trap behavior",
    }
    missing_frame = sorted(fragment for fragment in frame_fragments if fragment not in frame_text)
    if missing_frame:
        fail(f"F-MMIO frame accounting is incomplete: {missing_frame}")
    d24_fragments = {
        "mapping/resource lifetime and invalidation",
        "event identity, and atomicity/tearing",
        "exact number of access events on each executed source path",
        "speculation, duplication, fusion, splitting, widening, narrowing, elision, or reordering",
        "ordering against relevant ordinary-memory as well as device accesses",
        "fact attenuation for externally mutable device state",
        "ordinary-memory region reachable by an external agent, including DMA",
        "fault/trap behavior",
    }
    missing_d24 = sorted(fragment for fragment in d24_fragments if fragment not in d24_text)
    if missing_d24:
        fail(f"D24 MMIO accounting is incomplete: {missing_d24}")


def verify_template_markers() -> None:
    template = (ROOT / "FAMILY-LOCK-A-TEMPLATE.md").read_text(encoding="utf-8")
    markers = re.findall(r"<[^>]+>", template)
    required_prefix = "<required" + ": "
    if not markers or any(not marker.startswith(required_prefix) for marker in markers):
        fail("Family Lock template has an invalid field marker")
    for relative in AUTHORED_TEXT:
        if relative == "FAMILY-LOCK-A-TEMPLATE.md":
            continue
        if ("<required" + ":") in (ROOT / relative).read_text(encoding="utf-8"):
            fail(f"unresolved template marker outside Family Lock template: {relative}")


def verify_g0_non_importability() -> None:
    artifacts = {
        "charter": (ROOT / "G0-CORE-CHARTER.md").read_text(encoding="utf-8"),
        "census": (ROOT / "RUST-DATA-CONTRACT-CENSUS.md").read_text(encoding="utf-8"),
        "report": (ROOT / "G0-CORE-REPORT.md").read_text(encoding="utf-8"),
        "template": (ROOT / "FAMILY-LOCK-A-TEMPLATE.md").read_text(encoding="utf-8"),
        "plan": (REPO / "THE-PLAN.md").read_text(encoding="utf-8"),
        "design memory": (REPO / "mcts_mem/xlang.md").read_text(encoding="utf-8"),
    }
    normalized = {name: re.sub(r"\s+", " ", text) for name, text in artifacts.items()}
    required = {
        "charter": (
            "gives every one of the 276 rows the machine-enforced semantic class `G0_COVERAGE_CLUSTER`",
            "No row is an exact operation-semantic unit, a Family Lock import unit, an experiment unit, or evidence for family-level `E` or `P`",
            "assign a stable `member_contract_id` to every distinct member contract and an `outcome_id`",
            "derive the complete evidence-key audit domain for every assigned or implicated cluster",
            "independently derive the exact applicable target set `A(e)` for every evidence identity `e`",
            "exactly one legal terminal per applicable `(e, target)`, no terminal for a non-applicable target, no orphan target",
            "Cluster routing is many-to-many audit-domain and reopening evidence only",
            "take `A(e)` from the composition of their exact `impl_key` topology row and the closed owning-cluster operation-gate assignment",
            "A present operation gate has the topology primary as its one child-specific immediate predecessor target",
            "additive to the operation gate's complete route-level predecessor family/gate set",
        ),
        "census": (
            "semantic class `G0_COVERAGE_CLUSTER`",
            "it is not an exact operation contract",
            "`member_contract_id`",
            "`outcome_id`",
            "1,961 evidence relations",
        ),
        "report": (
            "276 coarse coverage and obligation clusters",
            "not exact operation contracts",
            "`member_contract_id`",
            "`outcome_id`",
            "1,961-relation evidence universe",
            "complete cluster audit domain",
            "independently derive each evidence identity's exact applicable target set",
            "a lock emits one terminal exactly when its family or gate is in `A(e)`",
            "Concrete implementation applicability is a composite exact authority",
            "exactly 97 carry an additional gate: Extend 22, Collect 21, Index 14, and Convert 40",
            "remaining 281 relations carry explicit `NONE`/`NONE`",
        ),
        "template": (
            "semantic class `G0_COVERAGE_CLUSTER`",
            "Their capability and cost fields may be conservative unions and therefore cannot be copied into an exact member row",
            "Every `REFINED_IN_LOCK` key for the current target maps to exactly one `member_contract_id`",
            "member/outcome totality covers every normal, recoverable-failure, checked-trap, abandonment, and destruction outcome",
            "No G0 coverage cluster, derivation-matrix row, or overlay key alone can support `E`, `P`, candidate construction, or a scored experiment",
            "Parent-only or partial expansion is invalid",
            "every B control runs against every candidate",
            "complete evidence universe in the current lock's audit domain",
            "current family or gate has exactly one terminal disposition if and only if it is in `A(e)`",
            "A non-applicable target has no terminal in this lock",
            "topology route composed with the closed owning-cluster operation-gate assignment",
            "An ungated concrete identity carries an explicit closed `NONE`/`NONE`",
        ),
        "plan": (
            "276 coarse coverage and obligation clusters",
            "cannot be imported as Family Lock, experiment, or closure units",
            "`member_contract_id` and `outcome_id` units",
            "evidence-key universe of every routed cluster as an audit domain",
            "no terminal for a non-applicable target",
        ),
        "design memory": (
            "The 276 detailed rows are non-importable coverage clusters",
            "freeze stable `member_contract_id` and `outcome_id` units",
            "is the Family Lock's audit domain, not a cluster-wide applicability union",
            "every applicable `(evidence_identity, target_id)` pair terminates exactly once",
            "Concrete implementation applicability composes a closed 13-class implementer partition with a closed four-row owning-cluster operation-gate assignment",
        ),
    }
    for artifact, fragments in required.items():
        missing = [fragment for fragment in fragments if fragment not in normalized[artifact]]
        if missing:
            fail(f"{artifact} lost the D11 G0 non-importability boundary: {missing}")

    current_state_artifacts = {
        "charter": normalized["charter"],
        "census": normalized["census"],
        "report": normalized["report"],
        "template": normalized["template"],
        "plan": normalized["plan"],
    }
    forbidden = (
        "276 normalized contracts",
        "276 normalized rows",
        "Coverage is quantified over the complete normalized contract",
        "Every selected evidence key",
        "maps every selected declaration",
    )
    for artifact, text in current_state_artifacts.items():
        present = [fragment for fragment in forbidden if fragment in text]
        if present:
            fail(f"{artifact} retains false exact-normalization claim: {present}")


def verify_report_accounting() -> None:
    report = (ROOT / "G0-CORE-REPORT.md").read_text(encoding="utf-8")
    required = {
        "297 reachable public modules, with 29 architecture/intrinsic catalogs",
        "17,135 rendered declaration rows",
        "10,267 stable-safe and 560 stable-unsafe renderings",
        "5,278 canonical stable-safe and 277 canonical stable-unsafe",
        "2,124 entries\nhave no description",
        "All 5,555 canonical stable declarations",
        "26-domain systems ledger",
        "4,792 safe contract anchors",
        "171 Rust-safe\nboundary declarations, 277 unsafe boundary declarations, and 315 Rust-only",
        "1,081\nG0 contracts, 329 library contracts, 3,876 later-family contracts",
        "seven\n`FRAME`-routed declarations representing one default-heap boundary service",
        "37\nredundant declarations, 220 declarations with no independent need",
        "five\n`NG`-routed declarations representing two conceptual non-goals",
        "545 canonical stable-safe inherent declarations, 35 stable-unsafe evidence\n"
        "declarations, 118 one-hop helper types",
        "all 175\ncanonical stable iteration/range declarations",
        "132 iteration and\n43 range declarations",
        "138 declaration-to-contract routes and 37\nredundant-surface routes",
        "split into 29 coarse range-obligation clusters",
        "Exactly 26 rows\ncarry active `BR-STORED` obligations, 138 rows have 294 exact deferred\ncomplement branches, and 100 rows need no additional stored-borrow complement",
        "172 stored transitions, 86 borrow-bearing owned\nresults, and 36 retained protocol states",
        "Of those branches, 139 require result\nprovenance; 100 publicly return a borrow-bearing\nowner",
        "six exact executable Rust 1.97 behavior\n  and lifecycle counterexamples plus one positive and two negative Range/Step",
        "The targeted set has 334 exact implementation\nrows: 312 stable selected data-floor implementations and 22 internal Step",
        "1,133 exact external relations",
        "828 explicit selector relations",
        "49-row role registry is the exact union of 31 D11\noperation rows and 18 protected",
        "Every deferred, delegated, boundary-evidence,\nor frame-deferred state forbids unrestricted `E` or `P`",
        "49 operational obligations, 12 orthogonal proof\ndimensions, and 16 global laws",
        "The 276-row derivation matrix",
        "no complete direct route\n(`E`), no evidence-backed complete pattern route (`P`)",
        "15 unproved or narrow\nworkarounds (`U`), 229",
        "four\nnamed frame dependencies, 19 scoped deferrals, and nine\n"
        "Rust boundary-evidence rows",
        "No true `NG` occurs in the detailed matrix",
        "authority and provenance; mapping lifetime and invalidation; width, alignment,",
        "event identity, and atomicity/tearing; exact source-path event multiplicity",
        "whether speculation, duplication, fusion, splitting, widening, narrowing,\n"
        "elision, or reordering is permitted",
        "ordering against relevant ordinary-memory\nand device accesses",
        "fact attenuation for externally mutable device state and\n"
        "ordinary memory reachable by external agents, including DMA",
        "platform side\neffects; fault/trap behavior; and target/OS mapping",
    }
    missing = sorted(fragment for fragment in required if fragment not in report)
    if missing:
        fail(f"G0 report accounting is incomplete or stale: {missing}")
    normalized_report = re.sub(r"\s+", " ", report)
    unsafe_map_fragments = {
        "`RUST-DATA-UNSAFE-EVIDENCE-MAP.tsv`",
        "all 35 canonical stable-unsafe data-floor declarations",
        "Every unsafe route terminates in one of eight `RAW-UNSAFE-*` evidence clusters and admits no xlang surface",
    }
    missing_unsafe_map = sorted(
        fragment for fragment in unsafe_map_fragments if fragment not in normalized_report
    )
    if missing_unsafe_map:
        fail(f"G0 report lost exact unsafe evidence-map accounting: {missing_unsafe_map}")


def verify_no_stale_payload_accounting() -> None:
    stale_fragments = {
        "262" + "-branch overlay",
        "all 262" + " branch keys",
        "262 exact" + " deferred",
        "262 deferred" + " branches",
        "The 262" + " branches",
        "142 stored" + " transitions",
        "85 borrow-bearing" + " owned",
        "35 retained" + " protocol states",
    }
    hits: list[str] = []
    for relative in AUTHORED_TEXT:
        contents = (ROOT / relative).read_text(encoding="utf-8")
        for fragment in stale_fragments:
            if fragment in contents:
                hits.append(f"{relative}: {fragment}")
    if hits:
        fail(f"stale payload accounting remains in authored artifacts: {hits}")
    report = (ROOT / "G0-CORE-REPORT.md").read_text(encoding="utf-8")
    normalized_report = re.sub(r"\s+", " ", report)
    exact_repair_claims = {
        "only lossy conversion's all-valid branch borrows its input",
        "error-byte access later borrows the error owner",
        "vacant entry keys derive from the guard-owned candidate",
        "successful insertion changes subsequent result provenance to map storage",
        "H-STORE must exercise the public checked `ST-SPARSE` arbitrary-occupancy contract",
        "the already adopted public `FAM-DENSE` contract retains the sole sealed owner",
        "every payload borrow remains rooted in the arena and exact retained allocation rather than a registry slot or token address",
        "Payload destruction precedes release of its backing block owner",
        "D_req/D_acq/D_peak, K_peak/J, and F/Z",
        "W-ARENA therefore cannot gate or help select the dense family it imports",
        "Rust's `slice::fill` is a separating witness",
        "Current OP-9 OOM is a TCB-level divergent policy edge",
        "Cursor invalidation ends its authority to produce future results",
        "an already yielded external borrow ends only with its declared source relation",
        "Rust array `IntoIter` is a compact separating case",
        "`[front, back)` interval is the exact live set",
        "destruction drops exactly the remaining interval",
        "Rust `Rc::make_mut` separates three ownership paths",
        "a sole strong owner with no external weak owner mutates in place",
        "outstanding weak owners dissociates those weak handles by relocating the payload",
        "multiple strong owners clone the payload into a new allocation",
        "neither creates a recoverable `FL-ATOMIC` contract",
        "`into_inner` returns it by consuming the owner",
        "`get_mut` borrows it without changing dynamic borrow state",
        "Provenance is assigned per borrowed leaf, not once to an aggregate result",
        "`BR-CURSOR` grants only an opaque cursor's protocol authority",
        "Such general retention requires `BR-STORED`",
        "The cursor's source borrow and its yield provenance are separate",
        "both a bounded `BR-REBORROW` relation and `BR-DISJOINT`",
        "never from pointer inequality",
        "Receiver-bounded `peek`, `peek_mut`, and `by_ref` results are the explicit exception",
        "`RefMut::filter_map` returning `None` returns the original guard with any callback mutation retained",
        "requires `BR-DISJOINT` but not `BR-REBORROW`",
        "`Ref::map_split` may overlap",
        "The bookkeeping root whose dynamic borrow count the guard releases therefore need not be the storage-provenance root of the returned referent",
        "A vacant map entry guard owns its candidate key by value",
        "an occupied entry guard owns only cursor/map authority",
        "`Ref::clone` creates a sibling guard",
        "`IntoKeys` yields `K` and destroys every omitted `V`",
        "`RefCell::swap` similarly exchanges two stored owners and returns none",
        "Generic retained state is not limited to collection elements",
        "Stored-borrow preservation is transition-sensitive",
        "That is not retargeting of the same live leaf",
        "A custom `Clone` may select, swap, or coalesce independently valid borrow roots",
        "Destination reuse preserves allocation resources, not overwritten leaf provenance",
        "Hash filtering is an active `BR-STORED` case",
        "no slot-live fact, payload provenance, cursor authority, mutation authority, or yielded item derives from it",
        "`build_hasher` may end, replace, or move its internal leaves only under the declared `BuildHasher` behavior-effect relation",
        "a unique leaf moved from `S` ends there before becoming live in `H`, never in both owners",
        "Compiled root-swap and unique-transfer canaries exercise both state replacement and unique-leaf transfer",
        "HashMap/HashSet equality iterates the left operand and performs `get` or `contains` probes only against the right operand",
        "Length mismatch and empty equality perform zero `build_hasher` calls",
        "only the `Hash` implementation branch of `Cmp` reborrows caller-owned mutable `H` without `BuildHasher`",
        "These branch roles must not be unioned",
        "Cursor lifecycle is separate from logical iteration state",
        "First or repeated terminal `None` does not, by itself, release source authority",
        "Retained substate may nevertheless retire or be replaced at `None` only when the exact concrete transition says so",
        "Proven last use may release only pure cursor/source-borrow authority",
        "Cursor destruction or a consuming close performs those pending effects",
        "Linked-list owning iteration instead deallocates nodes incrementally",
        "B-tree owning iteration retains family-specific tree/traversal state",
        "These resource shapes are not unioned",
        "`Vec::splice` performs zero replacement calls at terminal `None`",
        "`String::drain` leaves every byte unchanged throughout traversal and terminal `None`",
        "Hash drain moves the raw allocation into its cursor and returns that empty allocation only on close",
        "Each accepted `next` completes unlinking or metadata/tree repair before yielding",
        "Generic iterator `None` is not a cleanup or fusion theorem",
        "`repeat_n(seed, 0)` drops the seed during construction and retains none",
        "the first `n - 1` yields clone the seed, the final yield moves it",
        "`Cycle` clones once during construction",
        "current `None` clones `orig` once, replaces and destroys the old current epoch once",
        "Periodicity and unbounded repetition require a separate clone-equivalence and nonempty premise",
        "`FromFn`, `Scan`, ordinary transform/filter adapters, `MapWhile`, `Zip`, and `Peekable` can return `None` and later `Some`",
        "`core::range::RangeInclusive<T>` is a plain public `{start,last}` descriptor, not a cursor",
        "deliberate abandonment follows the separately frozen guard-leak policy",
        "empty access footprint",
        "pointer values compare equal",
        "mints no `BR-DISJOINT`",
        "`FromStr` exposes no input lifetime",
        "independently valid static root",
        "promoted zero-sized root",
        "empty footprint grants no storage or disjointness authority",
        "`Any: 'static` does not mean borrow-free",
        "A borrowed view conversion mints a result borrow from its borrowed input owner",
        "each pre-existing payload borrow leaf keeps its exact external or promoted-empty root",
        "consumed container would be unsound",
        "Logical exhaustion does not by itself release a source borrow",
        "Base-owner reuse requires all such authorities to have ended",
        "Region bounds and physical roots must not be collapsed",
        "all 138 clusters with deferred stored-borrow branches remain ineligible for unrestricted `E` or `P`",
    }
    missing_repairs = sorted(
        fragment for fragment in exact_repair_claims if fragment not in normalized_report
    )
    if missing_repairs:
        fail(f"G0 report lost hostile-review repair claims: {missing_repairs}")
    forbidden = {
        "16,996",
        "5,523",
        "292 reachable",
        "4,762 safe contract anchors",
        "170 Rust-safe",
        "314 Rust-only",
        "1,080 G0 contracts",
        "3,846 later-family contracts",
        "219 declarations with no independent need",
        "16,432",
        "5,369",
        "150-declaration",
        "258-row",
        "two true non-goals",
        "seven boundary-frame services",
    }
    present = sorted(fragment for fragment in forbidden if fragment in report)
    if present:
        fail(f"G0 report retains stale claims: {present}")


def verify_repository_language_rule() -> None:
    han = re.compile(r"[\u3400-\u4dbf\u4e00-\u9fff]")
    forbidden_marker = re.compile(r"\b(?:TODO|TBD|FIXME)\b", re.IGNORECASE)
    for relative in AUTHORED_TEXT:
        path = ROOT / relative
        text = path.read_text(encoding="utf-8")
        if han.search(text):
            fail(f"CJK prose marker in authored artifact {relative}")
        marker_text = text
        if relative == "DOMAIN-CLASSIFICATION-RULES.tsv":
            _, rows = read_tsv(relative)
            marker_text = "\n".join(
                row["canonical_route_or_blocked_claim"] + "\t" + row["rationale"]
                for row in rows
            )
        elif relative == "RUST-1.97.0-DOMAIN-CLASSIFICATION.tsv":
            _, rows = read_tsv(relative)
            prose_fields = [
                "surface_evidence_reason",
                "domain",
                "need_route_reason",
                "ng_authority_reason",
                "canonical_route_or_blocked_claim",
            ]
            marker_text = "\n".join(
                "\t".join(row[field] for field in prose_fields) for row in rows
            )
        marker_text = re.sub(r"`[^`\n]*`", "", marker_text)
        if not relative.startswith("tools/") and forbidden_marker.search(marker_text):
            fail(f"unfinished-work marker in authored artifact {relative}")
        if any(line.rstrip() != line for line in text.splitlines()):
            fail(f"trailing whitespace in authored artifact {relative}")
    if (REPO / "AGENTS.md").read_bytes() != (REPO / "CLAUDE.md").read_bytes():
        fail("AGENTS.md and CLAUDE.md are not byte-identical")


def verify_manifest() -> None:
    if not MANIFEST.is_file():
        fail("missing G0-CORE-ARTIFACT-MANIFEST.json")
    payload = json.loads(MANIFEST.read_text(encoding="utf-8"))
    if payload.get("schema") != "xlang-g0-core-artifact-manifest-v1":
        fail("unexpected exact-artifact manifest schema")
    if payload.get("rust_release") != "1.97.0":
        fail("unexpected exact-artifact Rust release")
    if payload.get("rust_peeled_commit") != "2d8144b7880597b6e6d3dfd63a9a9efae3f533d3":
        fail("unexpected exact-artifact Rust commit")
    artifacts = payload.get("artifacts")
    if not isinstance(artifacts, list) or payload.get("artifact_count") != len(artifacts):
        fail("exact-artifact manifest count mismatch")
    manifest_paths = [entry.get("path") for entry in artifacts]
    if manifest_paths != MANIFEST_ARTIFACTS or len(manifest_paths) != len(set(manifest_paths)):
        fail("exact-artifact manifest path set or order differs from the frozen input list")
    for entry in artifacts:
        relative = entry.get("path")
        if not isinstance(relative, str):
            fail("manifest artifact path is not a string")
        path = ROOT / relative
        if not path.is_file():
            fail(f"manifest artifact is missing: {relative}")
        if entry.get("sha256") != sha256(path):
            fail(f"manifest hash mismatch: {relative}")
        if entry.get("bytes") != path.stat().st_size:
            fail(f"manifest byte count mismatch: {relative}")
        if path.suffix == ".tsv":
            _, rows = read_tsv(relative)
            if entry.get("data_rows") != len(rows):
                fail(f"manifest row count mismatch: {relative}")


def main() -> None:
    run_verifier("verify_rust_census.py")
    verify_generated_classifier()
    run_verifier("build_rust_data_unsafe_evidence_map.py", "--check")
    run_verifier("verify_rust_data_contract_census.py")
    run_verifier("verify_rust_data_unsafe_evidence_map.py")
    verify_generated_coverage_cluster_registry()
    run_verifier("verify_g0_coverage_cluster_registry.py")
    run_verifier("build_g0_cluster_family_routing.py", "--check")
    run_verifier("verify_g0_cluster_family_routing.py")
    run_verifier("build_g0_coverage_evidence_universe.py", "--check")
    run_verifier("verify_g0_coverage_evidence_universe.py")
    run_verifier("build_g0_family_requirement_registry.py", "--check")
    run_verifier("verify_g0_family_requirement_registry.py")
    run_verifier("build_trait_impl_crosswalk.py", "--check")
    run_verifier("verify_trait_impl_crosswalk.py")
    run_verifier("build_g0_trait_impl_topology_routing.py", "--check")
    run_verifier("verify_g0_trait_impl_topology_routing.py")
    run_verifier("verify_g0_combined_dependency_dag.py")
    run_verifier("verify_derivation_matrix.py")
    verify_generated_payload_scope()
    run_verifier("verify_behavior_canaries.py")
    run_verifier("verify_trait_impl_canaries.py")
    verify_row_counts()
    verify_derivation_status_counts()
    verify_domain_routes()
    verify_witness_budgets()
    verify_template_markers()
    verify_g0_non_importability()
    verify_report_accounting()
    verify_no_stale_payload_accounting()
    verify_repository_language_rule()
    verify_manifest()
    print(
        "G0-Core verification: PASS — exact census, domains, contracts, "
        "derivations, template, language rule, and manifest are consistent"
    )


if __name__ == "__main__":
    main()
