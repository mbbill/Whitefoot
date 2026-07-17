#!/usr/bin/env python3
"""Build the deterministic exact-byte manifest for G0-Core review."""

from __future__ import annotations

import csv
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
OUTPUT = ROOT / "G0-CORE-ARTIFACT-MANIFEST.json"

ARTIFACTS = [
    "../../../.gitattributes",
    "../../../CONSTITUTION.md",
    "../../../AGENTS.md",
    "../../../CLAUDE.md",
    "../../../PATTERNS.md",
    "../../../THE-PLAN.md",
    "../../../spec/kernel-spec-v0.6.md",
    "../../notes/user-directives.md",
    "../../../mcts_mem/xlang.md",
    "../../../mcts_mem/xlang/data-model.md",
    "../../../mcts_mem/xlang/ownership.md",
    "../../../mcts_mem/xlang/ownership/copy-classification.md",
    "../../../mcts_mem/xlang/ownership/copy-classification.alt/uniform-affine-enums.md",
    "../../../mcts_mem/xlang/ownership/no-reborrow.md",
    "../../../mcts_mem/xlang/pattern-doctrine.md",
    "../../../mcts_mem/xlang/pattern-doctrine.alt/unconstrained-architecture.md",
    "../../../mcts_mem/xlang/fact-channels.md",
    "../../../mcts_mem/xlang/surface-form.md",
    "../../../mcts_mem/xlang/toolchain.md",
    "../general-purpose-data-structure-capability-RESEARCH.md",
    "../../../experiments/port-study/binary-trees/RESULTS.md",
    "../../../experiments/data-layout-owning-sequence/README.md",
    "../../../experiments/data-layout-owning-sequence/BASELINE.md",
    "../../../experiments/data-layout-owning-sequence/DETACHED_CANDIDATE.patch",
    "../../../experiments/data-layout-owning-sequence/FLAT_DESIGN_CANDIDATE.md",
    "../../../experiments/data-layout-owning-sequence/HARNESS.md",
    "../../../experiments/data-layout-owning-sequence/HOSTILE_REVIEW_0.md",
    "../../../experiments/data-layout-owning-sequence/HOSTILE_REVIEW_1.md",
    "../../../experiments/data-layout-owning-sequence/OWNERSHIP_ROUTE_HOSTILE_REVIEW.md",
    "../../../experiments/data-layout-owning-sequence/OWNERSHIP_ROUTE_PROTOCOL.md",
    "../../../experiments/data-layout-owning-sequence/PROTOCOL.md",
    "../../../experiments/data-layout-owning-sequence/PROTOCOL_AMENDMENTS.md",
    "../../../experiments/data-layout-owning-sequence/RESEARCH.md",
    "../../../experiments/data-layout-owning-sequence/RESEARCH_REPORT.md",
    "../../../experiments/data-layout-owning-sequence/RESULTS.md",
    "../../../experiments/data-layout-owning-sequence/REVIEW_RESPONSE.md",
    "../../../experiments/data-layout-owning-sequence/native/fsoa_sample.c",
    "../../../experiments/data-layout-owning-sequence/run_baseline.py",
    "../../../experiments/data-layout-owning-sequence/schemas/lock-v1.schema.json",
    "../../../experiments/data-layout-owning-sequence/schemas/manifest-v1.schema.json",
    "../../../experiments/data-layout-owning-sequence/schemas/sample-v1.schema.json",
    "G0-CORE-CHARTER.md",
    "RUST-1.97.0-CENSUS-MANIFEST.json",
    "RUST-1.97.0-API-INVENTORY.tsv",
    "RUST-1.97.0-MODULE-ACCOUNTING.tsv",
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
    "canaries/README.md",
    "canaries/xlang_buildhasher_root_swap.rs",
    "canaries/xlang_buildhasher_transfer.rs",
    "canaries/xlang_clone_source_effects.rs",
    "canaries/xlang_clone_helper_source_effects.rs",
    "canaries/xlang_behavior_receiver_effects.rs",
    "canaries/xlang_repeat_n_source_effects.rs",
    "canaries/xlang_range_step_stable_entrances.rs",
    "canaries/xlang_range_step_ascii_char_rejected.rs",
    "canaries/xlang_range_step_downstream_impl_rejected.rs",
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
]


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def row_count(path: Path) -> int | None:
    if path.suffix != ".tsv":
        return None
    with path.open(encoding="utf-8", newline="") as handle:
        return sum(1 for _ in csv.reader(handle, delimiter="\t")) - 1


def main() -> None:
    missing = [relative for relative in ARTIFACTS if not (ROOT / relative).is_file()]
    if missing:
        raise SystemExit("missing G0-Core artifacts: " + ", ".join(missing))

    artifacts = []
    for relative in ARTIFACTS:
        path = ROOT / relative
        entry: dict[str, object] = {
            "path": relative,
            "sha256": sha256(path),
            "bytes": path.stat().st_size,
        }
        rows = row_count(path)
        if rows is not None:
            entry["data_rows"] = rows
        artifacts.append(entry)

    payload = {
        "schema": "xlang-g0-core-artifact-manifest-v1",
        "rust_release": "1.97.0",
        "rust_peeled_commit": "2d8144b7880597b6e6d3dfd63a9a9efae3f533d3",
        "artifact_count": len(artifacts),
        "artifacts": artifacts,
    }
    OUTPUT.write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print(f"G0-Core manifest: wrote {len(artifacts)} exact artifact records")


if __name__ == "__main__":
    main()
