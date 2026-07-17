#!/usr/bin/env python3
"""Build the fail-closed G0 cluster-to-family/gate routing registry."""

from __future__ import annotations

import argparse
import collections
import csv
import hashlib
import io
import re
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
CENSUS = ROOT / "RUST-DATA-CONTRACT-CENSUS.tsv"
MATRIX = ROOT / "DERIVATION-MATRIX.tsv"
CLUSTER_REGISTRY = ROOT / "G0-COVERAGE-CLUSTER-REGISTRY.tsv"
PAYLOAD_SCOPE = ROOT / "PAYLOAD-SCOPE-CLASSIFICATION.tsv"
CAPABILITY_REGISTRY = ROOT / "CAPABILITY-OBLIGATION-REGISTRY.tsv"
FAMILY_REQUIREMENTS = ROOT / "G0-FAMILY-REQUIREMENT-REGISTRY.tsv"
VOCABULARY = ROOT / "G0-FAMILY-GATE-VOCABULARY.md"
EVIDENCE_UNIVERSE = ROOT / "G0-COVERAGE-EVIDENCE-UNIVERSE.tsv"
TRAIT_IMPL_CROSSWALK = ROOT / "RUST-1.97.0-TRAIT-IMPL-CROSSWALK.tsv"
TRAIT_IMPL_TOPOLOGY = ROOT / "G0-TRAIT-IMPL-TOPOLOGY-ROUTING.tsv"
OUTPUT = ROOT / "G0-CLUSTER-FAMILY-ROUTING.tsv"

CENSUS_FIELDS = [
    "contract_id",
    "family",
    "rust_surfaces",
    "pre_state",
    "input_ownership",
    "post_state_result",
    "invalidation",
    "failure_drop_abandonment",
    "complexity",
    "layout_identity_order",
    "behavior_parameter",
    "implementation_privilege_evidence",
    "xlang_current_status",
    "required_obligations",
    "source_refs",
]
MATRIX_FIELDS = [
    "contract_id",
    "capability_ids",
    "current_route",
    "status_code",
    "ordinary_library_derivation_sketch",
    "ownership_exit_drop_argument",
    "asymptotic_argument",
    "structural_costs_and_pathology",
    "fact_channels_and_invalidators",
    "negative_canaries",
    "family_lock_or_deferral",
    "evidence_refs",
]
CLUSTER_REGISTRY_FIELDS = [
    "cluster_ordinal",
    "cluster_id",
    "family",
    "semantic_class",
    "importability",
    "refinement_policy",
    "evidence_universe_policy",
    "allowed_evidence_dispositions",
    "prohibited_direct_uses",
    "census_row_sha256",
    "derivation_row_sha256",
    "policy_version",
]
PAYLOAD_SCOPE_FIELDS = [
    "contract_id",
    "stored_borrow_scope",
    "scope_owner_contract_ids",
    "rationale",
]

FIELDS = [
    "cluster_ordinal",
    "route_identity",
    "cluster_id",
    "census_family",
    "primary_refinement_owner_or_gate_stage",
    "required_predecessor_family_ids",
    "required_predecessor_gate_stage_ids",
    "implicated_or_reopening_family_ids",
    "implicated_or_reopening_gate_stage_ids",
    "delegated_owner_family_ids",
    "delegated_owner_contract_ids",
    "required_crosscut_dimension_ids",
    "route_category_id",
    "route_category_rationale",
    "assignment_rationale_id",
    "route_state",
    "routing_granularity",
    "evidence_child_rebind_policy",
    "prohibited_cluster_uses",
    "evidence_child_count",
    "evidence_child_identity_sha256",
    "trait_impl_relation_count",
    "trait_impl_key_sha256",
    "trait_impl_required_predecessor_family_ids",
    "trait_impl_required_predecessor_gate_stage_ids",
    "trait_impl_topology_family_ids",
    "trait_impl_topology_gate_stage_ids",
    "trait_impl_primary_resolution_policy",
    "matrix_status_code",
    "payload_scope_class",
    "routing_basis",
    "applicability_policy",
    "source_artifacts",
    "census_row_sha256",
    "derivation_row_sha256",
    "cluster_registry_row_sha256",
    "payload_scope_row_sha256",
    "route_spec_sha256",
    "vocabulary_sha256",
    "family_requirement_registry_sha256",
    "evidence_universe_sha256",
    "trait_impl_crosswalk_sha256",
    "trait_impl_topology_routing_sha256",
    "primary_route_count",
    "primary_route_sha256",
    "state_route_count",
    "state_route_sha256",
    "policy_version",
]

ROUTE_STATES = ("ACTIVE", "SCOPED_LATER", "BOUNDARY", "PROTECTED", "DELEGATED")
POLICY_VERSION = "xlang-g0-cluster-family-routing-v1"
ROUTING_BASIS = (
    "EXPLICIT_PRIMARY_REFINEMENT_MAP_PLUS_EXACT_CAPABILITY_DIMENSION_AND_"
    "PAYLOAD_SCOPE_DIMENSION_MAPS_PLUS_EXACT_TRAIT_IMPL_TYPED_TOPOLOGY_ROUTES"
)
APPLICABILITY_POLICY = (
    "LOCK_AUDIT_DOMAIN_INCLUDES_CLUSTER_IFF_PRIMARY_REFINEMENT_OWNER_OR_GATE_"
    "STAGE_EQUALS_LOCK_TARGET_OR_LOCK_TARGET_IS_IN_IMPLICATED_OR_REOPENING_"
    "FAMILY_OR_GATE_SET;CLUSTER_INCLUSION_NEVER_CONFERS_EVIDENCE_CHILD_"
    "APPLICABILITY;EVIDENCE_CHILD_APPLICABLE_IFF_LOCK_TARGET_IS_IN_"
    "INDEPENDENTLY_DERIVED_A_OF_E;EACH_APPLICABLE_EVIDENCE_TARGET_PAIR_"
    "REQUIRES_EXACT_MEMBER_OUTCOME_OR_CLAIM_BLOCKING_EXCLUSION;NON_APPLICABLE_"
    "TARGET_FORBIDS_REFINEMENT_PREDECESSOR_PROOF_OR_EXCLUSION;PRIMARY_"
    "REFINEMENT_IS_NOT_SINGLE_"
    "CLOSURE_CUSTODY;GATE_REQUIRES_ALL_PREDECESSORS_CLOSED;SCOPED_LATER_"
    "REQUIRES_OWNER_REAUTHORIZATION;BOUNDARY_FORBIDS_ORDINARY_SURFACE;DELEGATED_"
    "REQUIRES_ALL_DELEGATED_OWNER_FAMILY_ALLOCATION_ERROR_OUTCOMES_WITH_NO_"
    "INDEPENDENT_MEMBER_OR_BACKWARD_APPLICABILITY;EXACT_TRAIT_IMPL_TOPOLOGY_PRIMARY_"
    "OVERRIDES_COARSE_CLUSTER_FAMILY_FOR_THAT_CHILD;OPERATION_GATE_REMAINS_"
    "ADDITIONALLY_APPLICABLE;TOPOLOGY_GATE_IS_ITS_REQUIRED_GATE_PREDECESSOR;"
    "EACH_GATE_REQUIRES_AN_INDEPENDENT_TERMINAL_AND_NEITHER_GATE_EXCLUSION_"
    "ERASES_THE_OTHER;ACTIVE_IS_NOT_AUTHORIZATION"
)
ROUTING_GRANULARITY = "REFINE_PER_EVIDENCE_CHILD"
EVIDENCE_CHILD_REBIND_POLICY = (
    "INDEPENDENT_EXACT_CHILD_TO_FAMILY_OR_GATE_AUDIT_REQUIRED_BEFORE_MEMBER_"
    "OUTCOME_OR_EXCLUSION;CONCRETE_TRAIT_IMPL_JOINS_IMPL_KEY_TO_EXACT_TOPOLOGY_"
    "PRIMARY;COARSE_CLUSTER_PRIMARY_IS_NOT_INHERITED_BY_A_DIFFERENT_TOPOLOGY_"
    "CHILD;OPERATION_GATE_AND_TOPOLOGY_GATE_TERMINALS_ARE_DISTINCT;CLUSTER_"
    "UNION_NEVER_INHERITED"
)
TRAIT_IMPL_PRIMARY_RESOLUTION_POLICY = (
    "EXACT_IMPL_KEY_TOPOLOGY_PRIMARY_REPLACES_COARSE_CLUSTER_FAMILY;IF_COARSE_"
    "PRIMARY_IS_OPERATION_GATE_THEN_OPERATION_GATE_IS_ADDITIONAL_AND_ANY_"
    "DISTINCT_TOPOLOGY_GATE_IS_REQUIRED_PREDECESSOR;ALL_APPLICABLE_TERMINALS_"
    "ARE_KEYED_BY_EVIDENCE_IDENTITY_PLUS_OWNER_OR_GATE"
)
ROUTE_CATEGORY_RATIONALES = {
    "ROUTE-ACTIVE-FAMILY-DIRECT": "An active non-trait cluster starts refinement in its exact named family; predecessors do not gain evidence applicability.",
    "ROUTE-ACTIVE-FAMILY-COMPOUND": "An active non-trait cluster exposes exact children in more than one family; the cluster union is discovery only and every child is rebound independently.",
    "ROUTE-ACTIVE-TRAIT-FAMILY": "The coarse trait family is a starting stage only; each concrete impl child replaces it with its exact topology primary.",
    "ROUTE-ACTIVE-TRAIT-GATE": "The operation gate remains applicable in addition to each exact impl topology primary; distinct topology gates are required predecessors with independent terminals.",
    "ROUTE-ACTIVE-CROSS-FAMILY-GATE": "A non-trait cross-family gate starts only after its named predecessor families; predecessor evidence is not re-owned by the gate.",
    "ROUTE-SCOPED-LATER-FAMILY": "The exact later family is preserved without authorization to draft or implement it; all local dimensions must be rebound when authorized.",
    "ROUTE-BOUNDARY-FAMILY": "The checked underlying need is retained at a named boundary family without importing Rust raw spelling.",
    "ROUTE-BOUNDARY-REJECTION-GATE": "The raw or uninitialized spelling remains rejected while exact checked family needs and dimensions remain discoverable.",
    "ROUTE-DELEGATED-GATE": "The row closes only through every exact delegated owner outcome and has no independent operation-level closure.",
}
ROUTE_CATEGORY_AUTHORITY = {
    "ROUTE-ACTIVE-FAMILY-DIRECT": (
        "EXACT_FAMILY_BY_ASSIGNMENT",
        "EXACT_ROUTE_LOCAL_SET_OR_NONE",
        "NONE",
        "PRIMARY_FAMILY_ONLY",
        "EXACT_LOCAL_DIMENSION_SET_OR_NONE",
        "ACTIVE",
        "INDEPENDENT_EXACT_CHILD_REBIND;PREDECESSORS_NEVER_GAIN_APPLICABILITY",
        "Direct active family route; G0 cluster-routing discipline.",
    ),
    "ROUTE-ACTIVE-FAMILY-COMPOUND": (
        "EXACT_FAMILY_BY_ASSIGNMENT",
        "EXACT_ROUTE_LOCAL_SET_OR_NONE",
        "NONE",
        "PRIMARY_PLUS_EXPLICIT_REOPENING_SET",
        "EXACT_LOCAL_DIMENSION_SET_OR_NONE",
        "ACTIVE",
        "DISCOVERY_UNION_ONLY;EVERY_EXACT_CHILD_SPLIT_AND_REBOUND",
        "Compound active family route; boxed-init hostile review.",
    ),
    "ROUTE-ACTIVE-TRAIT-FAMILY": (
        "COARSE_START_FAMILY",
        "EXACT_ROUTE_LOCAL_SET_OR_NONE",
        "NONE",
        "EXACT_IMPL_TOPOLOGY_DISCOVERY_UNION",
        "EXACT_LOCAL_DIMENSION_SET_OR_NONE",
        "ACTIVE",
        "EXACT_IMPL_KEY_TOPOLOGY_PRIMARY_OVERRIDES_COARSE_FAMILY",
        "Trait-family route; exact 334-row topology routing.",
    ),
    "ROUTE-ACTIVE-TRAIT-GATE": (
        "EXACT_OPERATION_GATE",
        "EXACT_IMPL_TOPOLOGY_FAMILY_UNION",
        "EXACT_DISTINCT_TOPOLOGY_GATE_UNION_OR_NONE",
        "EXACT_IMPL_TOPOLOGY_DISCOVERY_UNION",
        "EXACT_LOCAL_DIMENSION_SET_OR_NONE",
        "ACTIVE",
        "OPERATION_GATE_ADDITIONAL;EVERY_GATE_TERMINAL_INDEPENDENT",
        "Trait operation-gate route; gate-composition hostile review.",
    ),
    "ROUTE-ACTIVE-CROSS-FAMILY-GATE": (
        "EXACT_GATE_BY_ASSIGNMENT",
        "EXACT_ROUTE_LOCAL_SET_OR_NONE",
        "NONE",
        "EXACT_REOPENING_SET_OR_NONE",
        "EXACT_LOCAL_DIMENSION_SET_OR_NONE",
        "ACTIVE",
        "GATE_AFTER_EXACT_PREDECESSORS;NO_PREDECESSOR_EVIDENCE_OWNERSHIP",
        "Cross-family gate route; typed gate vocabulary.",
    ),
    "ROUTE-SCOPED-LATER-FAMILY": (
        "EXACT_LATER_FAMILY",
        "EXACT_TRUE_TOPOLOGY_PREDECESSORS_OR_NONE",
        "NONE",
        "PRIMARY_FAMILY_ONLY",
        "EXACT_LOCAL_DIMENSION_SET_OR_NONE",
        "SCOPED_LATER",
        "NO_DRAFT_OR_IMPLEMENTATION_WITHOUT_OWNER_REAUTHORIZATION",
        "Later-family preservation; G0 owner scope ruling.",
    ),
    "ROUTE-BOUNDARY-FAMILY": (
        "EXACT_BOUNDARY_FAMILY",
        "EXACT_CHECKED_INPUT_FAMILIES_OR_NONE",
        "NONE",
        "PRIMARY_FAMILY_ONLY",
        "EXACT_LOCAL_DIMENSION_SET_OR_NONE",
        "BOUNDARY",
        "CHECKED_NEED_ONLY;NO_RUST_RAW_SPELLING_IMPORT",
        "Boundary-family route; G0 boundary evidence review.",
    ),
    "ROUTE-BOUNDARY-REJECTION-GATE": (
        "GATE-RAW-SPELLING-REJECTION",
        "EXACT_CHECKED_INPUT_FAMILIES_OR_NONE",
        "NONE",
        "EXACT_CHECKED_REOPENING_SET_OR_NONE",
        "EXACT_LOCAL_DIMENSION_SET_OR_NONE",
        "BOUNDARY",
        "RAW_OR_UNINITIALIZED_SPELLING_REJECTED;CHECKED_NEED_PRESERVED",
        "Boundary rejection route; CONSTITUTION and PATTERNS P8.",
    ),
    "ROUTE-DELEGATED-GATE": (
        "GATE-FAMILY-ALLOCATION-ERROR",
        "NONE",
        "NONE",
        "NONE",
        "DIM-FAILURE",
        "DELEGATED",
        "EXACT_DELEGATED_OWNER_OUTCOMES_ONLY;NO_INDEPENDENT_MEMBER",
        "Delegated allocation route; ALLOC-ERROR-01 hostile review.",
    ),
}
ASSIGNMENT_RATIONALE_AUTHORITY = {
    "ASSIGN-OUTER-TOPOLOGY-DIRECT": (
        "Use the cluster's externally observable storage or traversal topology as its primary family; do not infer another topology from capability labels.",
        "Distinct deque, ordered, sparse, iteration, and other outer contracts remain independently refinable, and predecessors never gain evidence applicability.",
        "G0-CORE-REPORT.md Sections 1 and 9; G0-COVERAGE-CLUSTER-REGISTRY.tsv",
    ),
    "ASSIGN-DENSE-FOUNDATION": (
        "Route contiguous views, initialized-prefix sequence operations, and generic replace or take starting points to F-DENSE with local dimensions rebound per exact child.",
        "These operations require contiguous live-range state or dense reusable ownership facts; the coarse cluster remains discovery rather than closure.",
        "general-purpose-data-structure-capability-RESEARCH.md Sections 4 through 6; RUST-DATA-CONTRACT-CENSUS.tsv",
    ),
    "ASSIGN-GENERIC-BOX": (
        "Route generic Box construction to F-RECURSIVE without importing allocator-service or dense-sequence topology.",
        "Generic Box is the uniquely owned recursive substrate; an allocator type parameter alone is not a user-visible allocator contract.",
        "G0-CORE-REPORT.md later-family boundary; RUST-1.97.0-TRAIT-IMPL-CROSSWALK.tsv",
    ),
    "ASSIGN-BOXED-INIT-SPLIT": (
        "Start BOX-INIT-01 in F-RECURSIVE and explicitly reopen F-DENSE for boxed slice or array children, requiring independent child splitting.",
        "Scalar generic Box and boxed dense payload constructors share a coarse census cluster but not one closure family.",
        "RUST-DATA-CONTRACT-CENSUS.tsv BOX-INIT-01; hostile topology review",
    ),
    "ASSIGN-SEQUENCE-BACKED-HEAP": (
        "Route BinaryHeap operations to F-HEAP after the exact dense backing-sequence predecessor.",
        "Heap repair is the outer contract while dense storage is reusable input, not a second owner of heap evidence.",
        "general-purpose-data-structure-capability-RESEARCH.md Sections 4 and 5; canonical predecessor union",
    ),
    "ASSIGN-TEXT-OVER-DENSE": (
        "Route byte and UTF-8 string semantics to F-TEXT after F-DENSE storage.",
        "Text validity and boundary-safe edits are the outer contract; dense bytes are a predecessor and cannot disposition text evidence.",
        "general-purpose-data-structure-capability-RESEARCH.md Section 4; G0-CORE-REPORT.md",
    ),
    "ASSIGN-LINKED-COMPOSITION": (
        "Route linked-list operations through GATE-LINKED-COMPOSITION after dense, identity, and recursive predecessors.",
        "No one predecessor alone supplies linked composition, stable node identity, recursive ownership, and bounded destruction.",
        "general-purpose-data-structure-capability-RESEARCH.md Sections 4 and 7.1; gate vocabulary",
    ),
    "ASSIGN-EXACT-TRAIT-TOPOLOGY": (
        "Use the coarse trait family only as a starting stage and replace it per concrete impl_key with the exact topology primary.",
        "A trait cluster spans unrelated implementer topologies; flat family custody would let one family erase another wrapper or container obligation.",
        "RUST-1.97.0-TRAIT-IMPL-CROSSWALK.tsv; G0-TRAIT-IMPL-TOPOLOGY-ROUTING.tsv",
    ),
    "ASSIGN-OPERATION-GATE-COMPOSITION": (
        "Keep the cross-family operation gate additionally applicable after every exact topology primary and distinct topology gate predecessor.",
        "Operation semantics and implementer topology are separate obligations keyed by the same evidence child; neither terminal or exclusion erases the other.",
        "gate vocabulary; exact trait-impl topology routing; hostile gate-composition review",
    ),
    "ASSIGN-SCOPED-LATER-WRAPPER": (
        "Preserve the exact later wrapper family and only its true topology predecessors, with all ownership, access, failure, and resource dimensions rebound locally.",
        "Pinning, shared ownership, dynamic borrow, Unicode, and type identity are distinct outer contracts and remain unauthorized for implementation.",
        "G0-CORE-REPORT.md later-family boundary; owner scope ruling",
    ),
    "ASSIGN-RAW-BOUNDARY": (
        "Retain checked ABI, allocation, or underlying family needs while rejecting writer-visible raw or uninitialized spelling.",
        "Boundary evidence preserves completeness without importing Rust unsafe mechanisms or weakening source-level checks.",
        "CONSTITUTION.md; PATTERNS.md P8; G0-CORE-REPORT.md boundary evidence",
    ),
    "ASSIGN-DELEGATED-ALLOCATION": (
        "Delegate recoverable allocation-error evidence to every exact delegated owner outcome with DIM-FAILURE rebound locally.",
        "The row has no independent operation and cannot close until all named owner outcomes discharge their branch.",
        "RUST-DATA-CONTRACT-CENSUS.tsv ALLOC-ERROR-01; gate vocabulary",
    ),
}
PROHIBITED_CLUSTER_USES = (
    "FAMILY_CLOSURE_UNIT;MEMBER_CONTRACT;OUTCOME_CONTRACT;FAMILY_E;FAMILY_P;"
    "CAPABILITY_INHERITANCE;COST_INHERITANCE;CANDIDATE_CONSTRUCTION;SCORED_EXPERIMENT"
)
SOURCE_ARTIFACTS = (
    "RUST-DATA-CONTRACT-CENSUS.tsv;DERIVATION-MATRIX.tsv;"
    "G0-COVERAGE-CLUSTER-REGISTRY.tsv;PAYLOAD-SCOPE-CLASSIFICATION.tsv;"
    "CAPABILITY-OBLIGATION-REGISTRY.tsv;G0-FAMILY-REQUIREMENT-REGISTRY.tsv;"
    "G0-FAMILY-GATE-VOCABULARY.md;G0-COVERAGE-EVIDENCE-UNIVERSE.tsv;"
    "RUST-1.97.0-TRAIT-IMPL-CROSSWALK.tsv;"
    "G0-TRAIT-IMPL-TOPOLOGY-ROUTING.tsv"
)


@dataclass(frozen=True)
class RouteGroup:
    owner: str
    predecessors: tuple[str, ...]
    extra_implicated: tuple[str, ...]
    state: str
    cluster_ids: tuple[str, ...]
    required_dimensions: tuple[str, ...] = ()
    delegated_owner_families: tuple[str, ...] = ()


def ids(text: str) -> tuple[str, ...]:
    return tuple(text.split())


# Every cluster identity is listed literally in exactly one group. This is the
# reviewed primary-refinement map; family names and census prose are never
# fuzzy-matched. A primary route never makes the coarse cluster a closure unit.
ROUTE_GROUPS = (
    RouteGroup(
        "F-DENSE",
        (),
        (),
        "ACTIVE",
        ids(
            """ARR-VIEW-01 ARR-EACH-01 VIEW-META-01 VIEW-GET-01 VIEW-GET-02
            VIEW-END-01 VIEW-ARRAY-01 VIEW-END-CHUNK-01 VIEW-END-SPLIT-01
            VIEW-SPLIT-01 VIEW-SPLIT-02 VIEW-CONSUME-01 VIEW-DISJOINT-01
            VIEW-ARRAY-CHUNKS-01"""
        ),
    ),
    RouteGroup(
        "F-ITERATION",
        (),
        (),
        "ACTIVE",
        ids("HELPER-REMAINDER-01 HELPER-CURSOR-VIEW-01"),
    ),
    RouteGroup(
        "F-DENSE",
        (),
        (),
        "ACTIVE",
        ids("TRAIT-DEREF-01 TRAIT-BORROW-01"),
    ),
    RouteGroup(
        "F-DENSE",
        (),
        (),
        "ACTIVE",
        ids("ARR-MAP-01 INIT-WRITE-01 MEM-REPLACE-01 MEM-TAKE-01"),
        ("DIM-OWNERSHIP",),
    ),
    RouteGroup(
        "F-RECURSIVE",
        (),
        (),
        "ACTIVE",
        ids("BOX-NEW-01"),
    ),
    RouteGroup(
        "F-RECURSIVE",
        (),
        ("F-DENSE",),
        "ACTIVE",
        ids("BOX-INIT-01"),
        ("DIM-OWNERSHIP", "DIM-FAILURE"),
    ),
    RouteGroup(
        "F-DENSE",
        (),
        (),
        "ACTIVE",
        ids("TRAIT-CLONE-01"),
    ),
    RouteGroup(
        "F-DENSE",
        (),
        (),
        "ACTIVE",
        ids("TRAIT-DEFAULT-01"),
    ),
    RouteGroup(
        "F-DENSE",
        (),
        (),
        "ACTIVE",
        ids("TRAIT-DROP-01"),
    ),
    RouteGroup(
        "F-DENSE",
        (),
        (),
        "ACTIVE",
        ids(
            """VIEW-SORT-01 VIEW-SORT-02 VIEW-SELECT-01 VIEW-REORDER-01
            VIEW-SWAP-01 VIEW-COPY-01 VIEW-CLONE-01 VIEW-FILL-01 VIEW-ALLOC-01
            VIEW-CONCAT-01 SEQ-META-01 SEQ-RESERVE-01 SEQ-TRY-RESERVE-01
            SEQ-SHRINK-01 SEQ-VIEW-01 SEQ-PUSH-01 SEQ-INSERT-01 SEQ-POP-01
            SEQ-REMOVE-01 SEQ-APPEND-01 SEQ-EXTEND-COPY-01 SEQ-RESIZE-01
            SEQ-TRUNCATE-01 SEQ-RETAIN-01 SEQ-DEDUP-01 SEQ-DRAIN-01
            SEQ-EXTRACT-01 SEQ-SPLICE-01 SEQ-SPLIT-01 SEQ-CONVERT-01"""
        ),
    ),
    RouteGroup(
        "F-DEQUE",
        (),
        (),
        "ACTIVE",
        ids(
            """DEQUE-META-01 DEQUE-RESERVE-01 DEQUE-VIEW-01 DEQUE-CONTIG-01
            DEQUE-ACCESS-01 DEQUE-RANGE-01 DEQUE-ITER-01 DEQUE-PUSH-01
            DEQUE-POP-01 DEQUE-INSERT-01 DEQUE-REMOVE-01 DEQUE-SWAP-01
            DEQUE-BULK-01 DEQUE-RESIZE-01 DEQUE-RETAIN-01 DEQUE-DRAIN-01
            DEQUE-ROTATE-01 DEQUE-SEARCH-01"""
        ),
    ),
    RouteGroup(
        "GATE-LINKED-COMPOSITION",
        ("F-DENSE", "F-IDENTITY", "F-RECURSIVE"),
        (),
        "ACTIVE",
        ids(
            """LIST-META-01 LIST-END-01 LIST-PUSH-01 LIST-POP-01 LIST-BULK-01
            LIST-DROP-01 LIST-ITER-01 LIST-SEARCH-01 LIST-EXTRACT-01"""
        ),
    ),
    RouteGroup(
        "F-HEAP",
        ("F-DENSE",),
        (),
        "ACTIVE",
        ids(
            """HEAP-META-01 HEAP-RESERVE-01 HEAP-PEEK-01 HEAP-MUTATE-01
            HEAP-APPEND-01 HEAP-RETAIN-01 HEAP-DRAIN-01 HEAP-VIEW-01
            HEAP-CONVERT-01"""
        ),
    ),
    RouteGroup(
        "F-ORDERED",
        ("F-DENSE", "F-IDENTITY", "F-RECURSIVE"),
        (),
        "ACTIVE",
        ids(
            """OMAP-META-01 OMAP-LOOKUP-01 OMAP-END-01 OMAP-INSERT-01
            OMAP-REMOVE-01 OMAP-RANGE-01 OMAP-ITER-01 OMAP-FILTER-01
            OMAP-BULK-01 OMAP-CLEAR-01 OSET-META-01 OSET-LOOKUP-01
            OSET-INSERT-01 OSET-REMOVE-01 OSET-RANGE-01 OSET-FILTER-01
            OSET-BULK-01"""
        ),
    ),
    RouteGroup(
        "GATE-KEYED-ENTRY-CROSS-FAMILY",
        ("F-SPARSE", "F-ORDERED"),
        (),
        "ACTIVE",
        ids("MAP-ENTRY-01 MAP-OCCUPIED-01 MAP-VACANT-01"),
    ),
    RouteGroup(
        "GATE-SET-CROSS-FAMILY",
        ("F-SPARSE", "F-ORDERED"),
        (),
        "ACTIVE",
        ids("SET-REL-01 SET-ALG-01 SET-ALG-02"),
    ),
    RouteGroup(
        "F-SPARSE",
        ("F-DENSE",),
        (),
        "ACTIVE",
        ids(
            """HMAP-META-01 HMAP-RESERVE-01 HMAP-LOOKUP-01
            HMAP-DISJOINT-01 HMAP-INSERT-01 HMAP-REMOVE-01 HMAP-ITER-01
            HMAP-DRAIN-01 HMAP-FILTER-01 HSET-META-01 HSET-RESERVE-01
            HSET-LOOKUP-01 HSET-INSERT-01 HSET-REMOVE-01 HSET-ITER-01
            HSET-DRAIN-01 HSET-FILTER-01"""
        ),
    ),
    RouteGroup(
        "F-TEXT",
        ("F-DENSE",),
        (),
        "ACTIVE",
        ids(
            """BYTE-ASCII-01 BYTE-ASCII-02 BYTE-ASCII-03 BYTE-ASCII-04
            BYTE-ASCII-05 BYTE-UTF8-CHUNKS-01 TEXT-META-01 TEXT-BYTES-01
            TEXT-VALIDATE-01 TEXT-BOUNDARY-01 TEXT-GET-01 TEXT-SPLIT-AT-01
            TEXT-SPLIT-AT-02 TEXT-ITER-01 TEXT-UTF16-01 TEXT-SEARCH-01
            TEXT-MATCH-ITER-01 TEXT-SPLIT-PATTERN-01 TEXT-LINES-01
            TEXT-TRIM-01 TEXT-ESCAPE-01 TEXT-PARSE-01 TEXT-ASCII-01
            TEXT-REPLACE-01 TEXT-REPEAT-01 TEXT-BOX-CONVERT-01 STRING-META-01
            STRING-VIEW-01 STRING-RESERVE-01 STRING-PUSH-01 STRING-POP-01
            STRING-INSERT-01 STRING-TRUNCATE-01 STRING-SPLIT-01
            STRING-RETAIN-01 STRING-DRAIN-01 STRING-REPLACE-01
            STRING-DECODE-STRICT-01 STRING-DECODE-LOSSY-01
            STRING-DECODE-ERROR-01 STRING-UTF16-01 STRING-CONVERT-01"""
        ),
    ),
    RouteGroup(
        "F-ITERATION",
        (),
        (),
        "ACTIVE",
        ids(
            """VIEW-ITER-01 VIEW-WINDOW-01 VIEW-CHUNKS-01 VIEW-CHUNKBY-01
            VIEW-SPLIT-PRED-01 VIEW-SEARCH-01 VIEW-SEARCH-02
            VIEW-ORDER-CHECK-01 HELPER-ARRAY-INTOITER-01 TRAIT-ITER-01
            TRAIT-DOUBLE-01 TRAIT-EXACT-01 TRAIT-FUSED-01
            ITER-SOURCE-VALUE-01 ITER-SOURCE-REPEAT-01
            ITER-SOURCE-CALLBACK-01 ITER-ADAPT-TRANSFORM-01
            ITER-ADAPT-DUPLICATE-01 ITER-ADAPT-SELECT-01
            ITER-ADAPT-POSITION-01 ITER-ADAPT-CHAIN-01 ITER-ADAPT-ZIP-01
            ITER-ADAPT-NEST-01 ITER-ADAPT-STATE-01 ITER-ADAPT-PEEK-01
            ITER-ADAPT-REBORROW-01 ITER-ADAPT-DIRECTION-01
            ITER-ADAPT-FUSE-01 ITER-ADAPT-CYCLE-01 ITER-CONSUME-FOLD-01
            ITER-CONSUME-SHORT-01 ITER-CONSUME-RELATION-01
            ITER-CONSUME-FANOUT-01 RANGE-VALUE-HALFOPEN-01
            RANGE-CONTAINS-HALFOPEN-01 RANGE-EMPTY-HALFOPEN-01
            RANGE-ITER-HALFOPEN-01 RANGE-VALUE-FROM-01
            RANGE-CONTAINS-FROM-01 RANGE-ITER-FROM-01
            RANGE-VALUE-INCLUSIVE-01 RANGE-CONTAINS-INCLUSIVE-01
            RANGE-EMPTY-INCLUSIVE-01 RANGE-ITER-INCLUSIVE-01
            RANGE-VALUE-TO-INCLUSIVE-01 RANGE-CONTAINS-TO-INCLUSIVE-01
            RANGE-BOUND-VALUE-01 RANGE-BOUND-BORROW-01 RANGE-BOUND-CLONE-01
            RANGE-BOUND-MAP-01 RANGE-LEGACY-HALFOPEN-STATE-01
            RANGE-BOUNDS-PROTOCOL-01 RANGE-BOUNDS-CONTAINS-01
            RANGE-LEGACY-FROM-STATE-01 RANGE-VALUE-FULL-01
            RANGE-LEGACY-INCLUSIVE-STATE-01
            RANGE-LEGACY-INCLUSIVE-CONTAINS-01
            RANGE-LEGACY-INCLUSIVE-EMPTY-01
            RANGE-LEGACY-INCLUSIVE-ACCESS-01 RANGE-LEGACY-INCLUSIVE-INTO-01
            RANGE-VALUE-TO-EXCLUSIVE-01 RANGE-CONTAINS-TO-EXCLUSIVE-01"""
        ),
    ),
    RouteGroup(
        "F-ITERATION",
        (),
        (),
        "ACTIVE",
        ids("TRAIT-INTOITER-01"),
    ),
    RouteGroup(
        "GATE-BULK-CONSTRUCTION-CROSS-FAMILY",
        (
            "F-DENSE",
            "F-DEQUE",
            "F-SPARSE",
            "F-ORDERED",
            "F-HEAP",
            "F-RECURSIVE",
            "F-TEXT",
            "F-ITERATION",
        ),
        (),
        "ACTIVE",
        ids("TRAIT-EXTEND-01 TRAIT-COLLECT-01"),
    ),
    RouteGroup(
        "GATE-INDEX-CROSS-FAMILY",
        ("F-DENSE", "F-DEQUE", "F-SPARSE", "F-ORDERED", "F-TEXT"),
        (),
        "ACTIVE",
        ids("TRAIT-INDEX-01"),
    ),
    RouteGroup(
        "GATE-CONVERSION-CROSS-FAMILY",
        ("F-DENSE", "F-TEXT"),
        (),
        "ACTIVE",
        ids("TRAIT-CONVERT-01"),
    ),
    RouteGroup(
        "F-DENSE",
        (),
        (),
        "ACTIVE",
        ids("TRAIT-CMP-01"),
    ),
    RouteGroup(
        "F-PIN-ADDRESS",
        ("F-DENSE",),
        (),
        "SCOPED_LATER",
        ids("VIEW-ELEMENT-OFFSET-01"),
    ),
    RouteGroup(
        "F-UNICODE",
        ("F-TEXT",),
        (),
        "SCOPED_LATER",
        ids("TEXT-UNICASE-01"),
    ),
    RouteGroup(
        "F-PIN-ADDRESS",
        ("F-RECURSIVE",),
        (),
        "SCOPED_LATER",
        ids("BOX-PIN-01"),
    ),
    RouteGroup(
        "F-TYPE-IDENTITY",
        ("F-RECURSIVE",),
        (),
        "SCOPED_LATER",
        ids("BOX-DOWNCAST-01"),
    ),
    RouteGroup(
        "F-SHARED",
        (),
        (),
        "SCOPED_LATER",
        ids(
            """RC-NEW-01 RC-WEAK-01 RC-COUNT-01 RC-IDENTITY-01 RC-UNIQUE-01
            RC-UNWRAP-01 RC-CYCLIC-01"""
        ),
        ("DIM-OWNERSHIP", "DIM-FAILURE", "DIM-RESOURCE-LIFETIME"),
    ),
    RouteGroup(
        "F-SHARED",
        ("F-DENSE",),
        (),
        "SCOPED_LATER",
        ids("RC-INIT-01"),
        ("DIM-OWNERSHIP", "DIM-FAILURE", "DIM-RESOURCE-LIFETIME"),
    ),
    RouteGroup(
        "F-PIN-ADDRESS",
        ("F-SHARED",),
        (),
        "SCOPED_LATER",
        ids("RC-PIN-01"),
        ("DIM-OWNERSHIP", "DIM-RESOURCE-LIFETIME"),
    ),
    RouteGroup(
        "F-TYPE-IDENTITY",
        ("F-SHARED",),
        (),
        "SCOPED_LATER",
        ids("RC-DOWNCAST-01"),
    ),
    RouteGroup(
        "F-DYNAMIC-BORROW",
        (),
        (),
        "SCOPED_LATER",
        ids(
            """REFCELL-OWNER-01 REFCELL-BORROW-01 REFCELL-TRY-01 REF-GUARD-01
            REFCELL-REPLACE-01"""
        ),
        (
            "DIM-ACCESS",
            "DIM-OWNERSHIP",
            "DIM-FAILURE",
            "DIM-RESOURCE-LIFETIME",
        ),
    ),
    RouteGroup(
        "GATE-FAMILY-ALLOCATION-ERROR",
        (),
        (),
        "DELEGATED",
        ids("ALLOC-ERROR-01"),
        ("DIM-FAILURE",),
        ("F-DENSE", "F-DEQUE", "F-SPARSE", "F-HEAP", "F-TEXT"),
    ),
    RouteGroup("F-ALLOC", (), (), "BOUNDARY", ids("ALLOC-OOM-01")),
    RouteGroup(
        "F-ABI",
        ("F-DENSE", "F-TEXT", "F-SHARED", "F-DYNAMIC-BORROW"),
        (),
        "BOUNDARY",
        ids("RAW-SAFE-PTR-01"),
        ("DIM-ACCESS", "DIM-OWNERSHIP", "DIM-RESOURCE-LIFETIME"),
    ),
    RouteGroup(
        "F-ABI",
        ("F-DENSE", "F-RECURSIVE", "F-TEXT", "F-SHARED"),
        (),
        "BOUNDARY",
        ids("RAW-SAFE-OWNERSHIP-01 RAW-UNSAFE-RECONSTRUCT-01"),
        ("DIM-OWNERSHIP", "DIM-RESOURCE-LIFETIME"),
    ),
    RouteGroup(
        "GATE-RAW-SPELLING-REJECTION",
        ("F-DENSE", "F-RECURSIVE", "F-TEXT"),
        (),
        "BOUNDARY",
        ids("RAW-SAFE-LEAK-01"),
        ("DIM-RESOURCE-LIFETIME",),
    ),
    RouteGroup(
        "GATE-RAW-SPELLING-REJECTION",
        (),
        ("F-DENSE",),
        "BOUNDARY",
        ids("RAW-SAFE-SPARE-01 RAW-UNSAFE-LEN-01"),
        ("DIM-OWNERSHIP",),
    ),
    RouteGroup(
        "GATE-RAW-SPELLING-REJECTION",
        (),
        ("F-DENSE", "F-RECURSIVE", "F-SHARED"),
        "BOUNDARY",
        ids("RAW-UNSAFE-INIT-01"),
        ("DIM-OWNERSHIP",),
    ),
    RouteGroup(
        "GATE-RAW-SPELLING-REJECTION",
        (),
        ("F-DENSE", "F-SPARSE"),
        "BOUNDARY",
        ids("RAW-UNSAFE-ACCESS-01"),
        ("DIM-ACCESS",),
    ),
    RouteGroup(
        "GATE-RAW-SPELLING-REJECTION",
        (),
        ("F-DENSE",),
        "BOUNDARY",
        ids("RAW-UNSAFE-ALIGN-01"),
        ("DIM-ACCESS",),
    ),
    RouteGroup(
        "GATE-RAW-SPELLING-REJECTION",
        (),
        ("F-TEXT",),
        "BOUNDARY",
        ids("RAW-UNSAFE-TEXT-01"),
    ),
    RouteGroup(
        "GATE-RAW-SPELLING-REJECTION",
        (),
        ("F-SHARED",),
        "BOUNDARY",
        ids("RAW-UNSAFE-RC-01"),
    ),
    RouteGroup(
        "GATE-RAW-SPELLING-REJECTION",
        (),
        ("F-DYNAMIC-BORROW",),
        "BOUNDARY",
        ids("RAW-UNSAFE-BORROW-01"),
    ),
)

# Every capability ID has an explicit topology-family and local-dimension
# interpretation. Empty tuples are reviewed no-extra-target results, not an
# unknown fallback. Dimensions never become owners or predecessors.
CAPABILITY_FAMILY_IMPLICATIONS = {
    "ST-FULL": (),
    "ST-AOS": (),
    "ST-DENSE": (),
    "ST-RING": (),
    "ST-SPARSE": (),
    "ST-DEPENDENT": (),
    "ST-HOLE": (),
    "ST-REFINE": (),
    "OW-INIT": (),
    "OW-MOVEOUT": (),
    "OW-REPLACE": (),
    "OW-SWAP": (),
    "OW-RELOCATE": (),
    "OW-CLONE": (),
    "OW-DROP": (),
    "EX-NORMAL": (),
    "EX-ABANDON": (),
    "EX-ABORT": (),
    "BR-PROV": (),
    "BR-REBORROW": (),
    "BR-RESULT": (),
    "BR-STORED": (),
    "BR-DISJOINT": (),
    "BR-INVALIDATE": (),
    "BR-CURSOR": (),
    "FL-CAPACITY": (),
    "FL-ALLOC": (),
    "FL-ATOMIC": (),
    "FL-CALLBACK": (),
    "ID-LOGICAL": (),
    "ID-FRESH": (),
    "ID-POOL": (),
    "ID-ADDRESS": (),
    "ID-SHARED": (),
    "AB-SEAL": (),
    "AB-BEHAVIOR": (),
    "AB-STATEFUL": (),
    "AB-GENERIC": (),
    "IT-SHARED": (),
    "IT-UNIQ": (),
    "IT-OWN": (),
    "IT-COMPOSE": (),
    "FT-STATE": (),
    "FT-REFINE": (),
    "FT-IDENTITY": (),
    "FT-BORROW": (),
    "FT-SHARED": (),
    "NT-FIXED": (),
    "NT-P2": (),
}

CAPABILITY_DIMENSION_IMPLICATIONS = {
    "ST-FULL": (),
    "ST-AOS": (),
    "ST-DENSE": (),
    "ST-RING": (),
    "ST-SPARSE": (),
    "ST-DEPENDENT": (),
    "ST-HOLE": ("DIM-OWNERSHIP",),
    "ST-REFINE": (),
    "OW-INIT": ("DIM-OWNERSHIP",),
    "OW-MOVEOUT": ("DIM-OWNERSHIP",),
    "OW-REPLACE": ("DIM-OWNERSHIP",),
    "OW-SWAP": ("DIM-OWNERSHIP",),
    "OW-RELOCATE": ("DIM-OWNERSHIP",),
    "OW-CLONE": ("DIM-OWNERSHIP",),
    "OW-DROP": ("DIM-OWNERSHIP", "DIM-RESOURCE-LIFETIME"),
    "EX-NORMAL": ("DIM-FAILURE",),
    "EX-ABANDON": ("DIM-FAILURE",),
    "EX-ABORT": ("DIM-FAILURE",),
    "BR-PROV": ("DIM-ACCESS",),
    "BR-REBORROW": ("DIM-ACCESS",),
    "BR-RESULT": ("DIM-ACCESS",),
    "BR-STORED": ("DIM-ACCESS", "DIM-STORED-BORROW"),
    "BR-DISJOINT": ("DIM-ACCESS",),
    "BR-INVALIDATE": ("DIM-ACCESS",),
    "BR-CURSOR": ("DIM-ACCESS",),
    "FL-CAPACITY": ("DIM-FAILURE",),
    "FL-ALLOC": ("DIM-FAILURE",),
    "FL-ATOMIC": ("DIM-FAILURE",),
    "FL-CALLBACK": ("DIM-FAILURE", "DIM-BEHAVIOR"),
    "ID-LOGICAL": (),
    "ID-FRESH": (),
    "ID-POOL": (),
    "ID-ADDRESS": (),
    "ID-SHARED": (),
    "AB-SEAL": (),
    "AB-BEHAVIOR": ("DIM-BEHAVIOR",),
    "AB-STATEFUL": ("DIM-BEHAVIOR",),
    "AB-GENERIC": ("DIM-BEHAVIOR",),
    "IT-SHARED": ("DIM-ACCESS",),
    "IT-UNIQ": ("DIM-ACCESS",),
    "IT-OWN": ("DIM-OWNERSHIP",),
    "IT-COMPOSE": ("DIM-BEHAVIOR",),
    "FT-STATE": (),
    "FT-REFINE": (),
    "FT-IDENTITY": (),
    "FT-BORROW": ("DIM-ACCESS",),
    "FT-SHARED": ("DIM-OWNERSHIP",),
    "NT-FIXED": (),
    "NT-P2": (),
}

PAYLOAD_SCOPE_DIMENSION_IMPLICATIONS = {
    "ACTIVE_BR_STORED": ("DIM-STORED-BORROW",),
    "DEFERRED_BRANCHES": ("DIM-STORED-BORROW",),
    "NO_STORED_BORROW_COMPLEMENT": (),
    "BOUNDARY_EVIDENCE_ONLY": (),
    "FRAME_SCOPE_DEFERRED": ("DIM-STORED-BORROW",),
    "DELEGATED_TO_FAMILY_BRANCHES": (),
}

STATUS_STATE_POLICY = {
    "E": "ACTIVE",
    "P": "ACTIVE",
    "U": "ACTIVE",
    "X": "ACTIVE",
    "DEFERRED": "SCOPED_LATER",
    "FRAME": "BOUNDARY",
    "BOUNDARY": "BOUNDARY",
}


def fail(message: str) -> None:
    raise SystemExit(f"G0 cluster family-routing build failed: {message}")


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def row_sha256(fields: list[str], row: dict[str, str]) -> str:
    return sha256_bytes(("\t".join(row[field] for field in fields) + "\n").encode("utf-8"))


def read_tsv(path: Path, expected_fields: list[str] | None = None) -> tuple[list[str], list[dict[str, str]]]:
    with path.open(encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        fields = list(reader.fieldnames or [])
        rows = list(reader)
    if expected_fields is not None and fields != expected_fields:
        fail(f"{path.name} schema changed")
    if not fields or any(None in row for row in rows):
        fail(f"{path.name} is malformed")
    if any(any("\r" in value or "\n" in value for value in row.values()) for row in rows):
        fail(f"{path.name} contains an embedded newline")
    return fields, rows


def markdown_authority_rows(
    text: str, begin_marker: str, end_marker: str, column_count: int
) -> list[list[str]]:
    if text.count(begin_marker) != 1 or text.count(end_marker) != 1:
        fail(f"authority markers are missing or duplicated: {begin_marker}")
    body = text.split(begin_marker, 1)[1].split(end_marker, 1)[0]
    table_lines = [line for line in body.splitlines() if line.startswith("|")]
    if len(table_lines) < 3:
        fail(f"authority table is empty: {begin_marker}")
    rows: list[list[str]] = []
    for line in table_lines[2:]:
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        if len(cells) != column_count:
            fail(f"authority table column count changed: {begin_marker}")
        rows.append(
            [
                cell[1:-1] if cell.startswith("`") and cell.endswith("`") else cell
                for cell in cells
            ]
        )
    return rows


def ordered_ids(values: set[str], vocabulary_order: dict[str, int]) -> str:
    if not values:
        return "NONE"
    try:
        return ",".join(sorted(values, key=vocabulary_order.__getitem__))
    except KeyError as exc:
        fail(f"undocumented family ID {exc.args[0]}")


def digest_identities(rows: list[dict[str, str]]) -> str:
    return sha256_bytes("".join(f"{row['route_identity']}\n" for row in rows).encode("utf-8"))


def parse_vocabulary() -> tuple[
    set[str], set[str], set[str], set[str], dict[str, int], str
]:
    if not VOCABULARY.is_file():
        fail(f"missing {VOCABULARY.name}")
    text = VOCABULARY.read_text(encoding="utf-8")
    tokens = re.findall(
        r"`((?:F|GATE|DIM)-[A-Z0-9-]+|ACTIVE|SCOPED_LATER|BOUNDARY|PROTECTED|DELEGATED)`",
        text,
    )
    if not tokens:
        fail("family/gate vocabulary has no machine-readable backticked IDs")
    families = {token for token in tokens if token.startswith("F-")}
    gates = {token for token in tokens if token.startswith("GATE-")}
    dimensions = {token for token in tokens if token.startswith("DIM-")}
    states = {token for token in tokens if token in ROUTE_STATES}
    order: dict[str, int] = {}
    for token in tokens:
        if token not in order:
            order[token] = len(order)
    if states != set(ROUTE_STATES):
        fail(f"vocabulary route-state set changed: {states}")
    if not dimensions:
        fail("family/gate vocabulary has no typed local dimensions")
    category_rows = markdown_authority_rows(
        text,
        "<!-- G0_CLUSTER_ROUTE_GROUP_AUTHORITY_BEGIN -->",
        "<!-- G0_CLUSTER_ROUTE_GROUP_AUTHORITY_END -->",
        9,
    )
    category_authority: dict[str, tuple[str, ...]] = {}
    for cells in category_rows:
        category_id = cells[0]
        if category_id in category_authority:
            fail(f"duplicate route-category authority row: {category_id}")
        category_authority[category_id] = tuple(cells[1:])
    if category_authority != ROUTE_CATEGORY_AUTHORITY:
        fail("route-category authority differs from the closed code table")

    rationale_rows = markdown_authority_rows(
        text,
        "<!-- G0_CLUSTER_ASSIGNMENT_RATIONALE_AUTHORITY_BEGIN -->",
        "<!-- G0_CLUSTER_ASSIGNMENT_RATIONALE_AUTHORITY_END -->",
        4,
    )
    rationale_authority: dict[str, tuple[str, ...]] = {}
    for cells in rationale_rows:
        rationale_id = cells[0]
        if rationale_id in rationale_authority:
            fail(f"duplicate assignment-rationale authority row: {rationale_id}")
        rationale_authority[rationale_id] = tuple(cells[1:])
    if rationale_authority != ASSIGNMENT_RATIONALE_AUTHORITY:
        fail("assignment-rationale authority differs from the closed code table")
    return (
        families,
        gates,
        dimensions,
        states,
        order,
        sha256_bytes(text.encode("utf-8")),
    )


def explicit_route_map() -> dict[str, RouteGroup]:
    routes: dict[str, RouteGroup] = {}
    for group in ROUTE_GROUPS:
        if group.state not in ROUTE_STATES:
            fail(f"unknown route state {group.state}")
        for cluster_id in group.cluster_ids:
            if cluster_id in routes:
                fail(f"duplicate explicit route for {cluster_id}")
            routes[cluster_id] = group
    return routes


def route_category(
    group: RouteGroup, has_trait_impl_children: bool
) -> tuple[str, str]:
    if group.state == "DELEGATED" and group.owner.startswith("GATE-"):
        category_id = "ROUTE-DELEGATED-GATE"
    elif group.state == "BOUNDARY" and group.owner == "GATE-RAW-SPELLING-REJECTION":
        category_id = "ROUTE-BOUNDARY-REJECTION-GATE"
    elif group.state == "BOUNDARY" and group.owner.startswith("F-"):
        category_id = "ROUTE-BOUNDARY-FAMILY"
    elif group.state == "SCOPED_LATER" and group.owner.startswith("F-"):
        category_id = "ROUTE-SCOPED-LATER-FAMILY"
    elif (
        group.state == "ACTIVE"
        and has_trait_impl_children
        and group.owner.startswith("GATE-")
    ):
        category_id = "ROUTE-ACTIVE-TRAIT-GATE"
    elif (
        group.state == "ACTIVE"
        and has_trait_impl_children
        and group.owner.startswith("F-")
    ):
        category_id = "ROUTE-ACTIVE-TRAIT-FAMILY"
    elif group.state == "ACTIVE" and group.owner.startswith("GATE-"):
        category_id = "ROUTE-ACTIVE-CROSS-FAMILY-GATE"
    elif (
        group.state == "ACTIVE"
        and group.owner.startswith("F-")
        and group.extra_implicated
    ):
        category_id = "ROUTE-ACTIVE-FAMILY-COMPOUND"
    elif group.state == "ACTIVE" and group.owner.startswith("F-"):
        category_id = "ROUTE-ACTIVE-FAMILY-DIRECT"
    else:
        fail(
            "route category has no exact structural case for "
            f"owner={group.owner}, state={group.state}, trait={has_trait_impl_children}"
        )
    return category_id, ROUTE_CATEGORY_RATIONALES[category_id]


def assignment_rationale_id(
    cluster_id: str, group: RouteGroup, has_trait_impl_children: bool
) -> str:
    if cluster_id == "BOX-NEW-01":
        rationale_id = "ASSIGN-GENERIC-BOX"
    elif cluster_id == "BOX-INIT-01":
        rationale_id = "ASSIGN-BOXED-INIT-SPLIT"
    elif group.state == "DELEGATED":
        rationale_id = "ASSIGN-DELEGATED-ALLOCATION"
    elif group.state == "BOUNDARY":
        rationale_id = "ASSIGN-RAW-BOUNDARY"
    elif group.state == "SCOPED_LATER":
        rationale_id = "ASSIGN-SCOPED-LATER-WRAPPER"
    elif has_trait_impl_children and group.owner.startswith("GATE-"):
        rationale_id = "ASSIGN-OPERATION-GATE-COMPOSITION"
    elif has_trait_impl_children:
        rationale_id = "ASSIGN-EXACT-TRAIT-TOPOLOGY"
    elif group.owner == "GATE-LINKED-COMPOSITION":
        rationale_id = "ASSIGN-LINKED-COMPOSITION"
    elif group.owner.startswith("GATE-"):
        rationale_id = "ASSIGN-OPERATION-GATE-COMPOSITION"
    elif group.owner == "F-HEAP":
        rationale_id = "ASSIGN-SEQUENCE-BACKED-HEAP"
    elif group.owner == "F-TEXT":
        rationale_id = "ASSIGN-TEXT-OVER-DENSE"
    elif group.owner == "F-DENSE":
        rationale_id = "ASSIGN-DENSE-FOUNDATION"
    elif group.state == "ACTIVE" and group.owner.startswith("F-"):
        rationale_id = "ASSIGN-OUTER-TOPOLOGY-DIRECT"
    else:
        fail(
            "assignment rationale has no exact structural case for "
            f"{cluster_id}, owner={group.owner}, state={group.state}"
        )
    if rationale_id not in ASSIGNMENT_RATIONALE_AUTHORITY:
        fail(f"assignment rationale is undocumented: {rationale_id}")
    return rationale_id


def build_rows() -> list[dict[str, str]]:
    census_fields, census_rows = read_tsv(CENSUS, CENSUS_FIELDS)
    matrix_fields, matrix_rows = read_tsv(MATRIX, MATRIX_FIELDS)
    cluster_fields, cluster_rows = read_tsv(CLUSTER_REGISTRY, CLUSTER_REGISTRY_FIELDS)
    payload_fields, payload_rows = read_tsv(PAYLOAD_SCOPE, PAYLOAD_SCOPE_FIELDS)
    capability_fields, capability_rows = read_tsv(CAPABILITY_REGISTRY)
    _, _ = read_tsv(FAMILY_REQUIREMENTS)
    (
        families,
        gates,
        dimensions,
        _,
        vocabulary_order,
        vocabulary_sha256,
    ) = parse_vocabulary()

    cluster_ids = [row["contract_id"] for row in census_rows]
    if len(cluster_ids) != 276 or len(set(cluster_ids)) != 276:
        fail("census is not the exact 276-cluster set")
    if [row["contract_id"] for row in matrix_rows] != cluster_ids:
        fail("matrix order differs from census")
    if [row["cluster_id"] for row in cluster_rows] != cluster_ids:
        fail("cluster-registry order differs from census")
    if [row["contract_id"] for row in payload_rows] != cluster_ids:
        fail("payload-scope order differs from census")
    if [row["cluster_ordinal"] for row in cluster_rows] != [str(i) for i in range(1, 277)]:
        fail("cluster ordinals are not the exact 1..276 sequence")

    routes = explicit_route_map()
    if set(routes) != set(cluster_ids):
        missing = sorted(set(cluster_ids) - set(routes))
        extra = sorted(set(routes) - set(cluster_ids))
        fail(f"explicit route-map set mismatch; missing={missing}; extra={extra}")

    if "capability_id" not in capability_fields:
        fail("capability registry lacks capability_id")
    capability_ids = [row["capability_id"] for row in capability_rows]
    if len(capability_ids) != len(set(capability_ids)):
        fail("capability registry has duplicate IDs")
    if set(CAPABILITY_FAMILY_IMPLICATIONS) != set(capability_ids):
        fail("capability family implication policy is not exact over the registry")
    if any(CAPABILITY_FAMILY_IMPLICATIONS.values()):
        fail(
            "capability labels cannot infer topology-family applicability; exact "
            "route groups and exact trait-impl child routes are authoritative"
        )
    if set(CAPABILITY_DIMENSION_IMPLICATIONS) != set(capability_ids):
        fail("capability dimension implication policy is not exact over the registry")
    payload_classes = {row["stored_borrow_scope"] for row in payload_rows}
    if set(PAYLOAD_SCOPE_DIMENSION_IMPLICATIONS) != payload_classes:
        fail("payload-scope dimension policy is not exact over the payload classes")

    used_primary_routes = {group.owner for group in ROUTE_GROUPS}
    used_gates = {owner for owner in used_primary_routes if owner.startswith("GATE-")}
    used_primary_families = {
        owner for owner in used_primary_routes if owner.startswith("F-")
    }
    if not used_gates <= gates:
        fail(f"undocumented gate IDs: {sorted(used_gates - gates)}")
    if not used_primary_families <= families:
        fail(
            "undocumented primary-refinement family IDs: "
            f"{sorted(used_primary_families - families)}"
        )
    used_predecessors = {
        predecessor for group in ROUTE_GROUPS for predecessor in group.predecessors
    }
    used_extra_families = {
        family for group in ROUTE_GROUPS for family in group.extra_implicated
    }
    used_dimensions = {
        dimension for group in ROUTE_GROUPS for dimension in group.required_dimensions
    }
    used_delegated_owners = {
        family
        for group in ROUTE_GROUPS
        for family in group.delegated_owner_families
    }
    if not used_predecessors | used_extra_families <= families:
        fail("an explicit route uses an undocumented predecessor or implicated family")
    if not used_dimensions <= dimensions:
        fail(f"explicit routes use undocumented dimensions {sorted(used_dimensions - dimensions)}")
    if not used_delegated_owners <= families:
        fail("an explicit route uses an undocumented delegated owner family")
    for group in ROUTE_GROUPS:
        if bool(group.delegated_owner_families) != (group.state == "DELEGATED"):
            fail("delegated owner families occur outside exactly the delegated route")
    if any(
        token.startswith("DIM-")
        for group in ROUTE_GROUPS
        for token in (group.owner,) + group.predecessors + group.extra_implicated
    ):
        fail("a DIM token is used as owner, predecessor, or implicated family")

    family_requirement_sha256 = sha256_bytes(FAMILY_REQUIREMENTS.read_bytes())
    evidence_fields, evidence_rows = read_tsv(EVIDENCE_UNIVERSE)
    for required_field in (
        "cluster_id",
        "evidence_identity",
        "cluster_relation_count",
        "cluster_relation_sha256",
    ):
        if required_field not in evidence_fields:
            fail(f"evidence universe lacks {required_field}")
    if len(evidence_rows) != 1961:
        fail(f"evidence universe has {len(evidence_rows)} rows, expected 1961")
    evidence_by_cluster: dict[str, list[dict[str, str]]] = collections.defaultdict(list)
    for evidence_row in evidence_rows:
        if not re.fullmatch(r"[0-9a-f]{64}", evidence_row["evidence_identity"]):
            fail("evidence universe contains an invalid evidence identity")
        evidence_by_cluster[evidence_row["cluster_id"]].append(evidence_row)
    if set(evidence_by_cluster) != set(cluster_ids):
        fail("evidence universe cluster set differs from the routing cluster set")
    for cluster_id, children in evidence_by_cluster.items():
        child_digest = sha256_bytes(
            "".join(f"{child['evidence_identity']}\n" for child in children).encode("utf-8")
        )
        if any(
            child["cluster_relation_count"] != str(len(children))
            or child["cluster_relation_sha256"] != child_digest
            for child in children
        ):
            fail(f"evidence child count/digest is stale for {cluster_id}")
    evidence_universe_sha256 = sha256_bytes(EVIDENCE_UNIVERSE.read_bytes())

    crosswalk_fields, crosswalk_rows = read_tsv(TRAIT_IMPL_CROSSWALK)
    topology_fields, topology_rows = read_tsv(TRAIT_IMPL_TOPOLOGY)
    required_topology_fields = {
        "impl_ordinal",
        "topology_route_identity",
        "impl_key",
        "owning_contract_ids",
        "primary_refinement_family_or_gate",
        "required_predecessor_family_ids",
        "required_predecessor_gate_stage_ids",
        "implicated_or_reopening_family_ids",
        "implicated_or_reopening_gate_stage_ids",
        "source_row_sha256",
        "vocabulary_sha256",
    }
    if not required_topology_fields <= set(topology_fields):
        fail("trait-impl topology routing schema is incomplete")
    if len(crosswalk_rows) != 334 or len(topology_rows) != 334:
        fail("trait-impl crosswalk/topology routing is not the exact 334-row set")
    if [row["impl_key"] for row in topology_rows] != [
        row["impl_key"] for row in crosswalk_rows
    ]:
        fail("trait-impl topology routing set/order differs from the crosswalk")
    if [row["impl_ordinal"] for row in topology_rows] != [str(i) for i in range(1, 335)]:
        fail("trait-impl topology ordinals are not the exact 1..334 sequence")
    for source_row, topology_row in zip(crosswalk_rows, topology_rows):
        if topology_row["source_row_sha256"] != row_sha256(crosswalk_fields, source_row):
            fail(f"trait-impl topology source hash is stale for {source_row['impl_key']}")
        if topology_row["vocabulary_sha256"] != vocabulary_sha256:
            fail(f"trait-impl topology vocabulary hash is stale for {source_row['impl_key']}")
        primary = topology_row["primary_refinement_family_or_gate"]
        predecessors = (
            set()
            if topology_row["required_predecessor_family_ids"] == "NONE"
            else set(topology_row["required_predecessor_family_ids"].split(","))
        )
        predecessor_gates = (
            set()
            if topology_row["required_predecessor_gate_stage_ids"] == "NONE"
            else set(topology_row["required_predecessor_gate_stage_ids"].split(","))
        )
        implicated_families = (
            set()
            if topology_row["implicated_or_reopening_family_ids"] == "NONE"
            else set(topology_row["implicated_or_reopening_family_ids"].split(","))
        )
        implicated_gates = (
            set()
            if topology_row["implicated_or_reopening_gate_stage_ids"] == "NONE"
            else set(topology_row["implicated_or_reopening_gate_stage_ids"].split(","))
        )
        if (
            primary not in families | gates
            or not predecessors <= families
            or not predecessor_gates <= gates
            or not implicated_families <= families
            or not implicated_gates <= gates
        ):
            fail(f"trait-impl topology route uses an untyped target for {source_row['impl_key']}")
        if any(
            target.startswith("DIM-")
            for target in predecessors
            | predecessor_gates
            | implicated_families
            | implicated_gates
            | {primary}
        ):
            fail("trait-impl topology route uses a DIM as a topology target")
        expected_primary_implication = (
            {primary} if primary.startswith("F-") else set()
        )
        expected_primary_gate = (
            {primary} if primary.startswith("GATE-") else set()
        )
        if implicated_families != expected_primary_implication:
            fail(f"trait-impl family applicability is not primary-only for {source_row['impl_key']}")
        if implicated_gates != expected_primary_gate:
            fail(f"trait-impl gate applicability is not primary-only for {source_row['impl_key']}")

    topology_by_cluster: dict[str, list[dict[str, str]]] = collections.defaultdict(list)
    for source_row, topology_row in zip(crosswalk_rows, topology_rows):
        owners = source_row["owning_contract_ids"].split(",")
        if not owners or any(owner not in set(cluster_ids) for owner in owners):
            fail(f"trait-impl crosswalk has an unknown owning cluster for {source_row['impl_key']}")
        for owner in owners:
            topology_by_cluster[owner].append(topology_row)
    if sum(len(rows) for rows in topology_by_cluster.values()) != 378:
        fail("trait-impl cluster expansion is not the exact 378 relations")

    evidence_trait_by_cluster: dict[str, list[dict[str, str]]] = collections.defaultdict(list)
    for evidence_row in evidence_rows:
        if evidence_row["evidence_kind"] == "CONCRETE_TRAIT_IMPL":
            evidence_trait_by_cluster[evidence_row["cluster_id"]].append(evidence_row)
    if sum(len(rows) for rows in evidence_trait_by_cluster.values()) != 378:
        fail("evidence universe does not contain exactly 378 concrete trait relations")
    if set(evidence_trait_by_cluster) != set(topology_by_cluster):
        fail("trait-impl cluster set differs between evidence and topology routing")
    for cluster_id, topology_children in topology_by_cluster.items():
        topology_keys = sorted(row["impl_key"] for row in topology_children)
        evidence_keys = sorted(
            row["evidence_key"] for row in evidence_trait_by_cluster[cluster_id]
        )
        if topology_keys != evidence_keys:
            fail(f"trait-impl evidence-key set differs for {cluster_id}")

    crosswalk_sha256 = sha256_bytes(TRAIT_IMPL_CROSSWALK.read_bytes())
    topology_routing_sha256 = sha256_bytes(TRAIT_IMPL_TOPOLOGY.read_bytes())
    output_rows: list[dict[str, str]] = []
    for ordinal, (census_row, matrix_row, cluster_row, payload_row) in enumerate(
        zip(census_rows, matrix_rows, cluster_rows, payload_rows), start=1
    ):
        cluster_id = census_row["contract_id"]
        group = routes[cluster_id]
        expected_state = STATUS_STATE_POLICY[matrix_row["status_code"]]
        if cluster_id == "ALLOC-ERROR-01":
            expected_state = "DELEGATED"
        if group.state != expected_state:
            fail(
                f"{cluster_id} state {group.state} disagrees with exact status-derived "
                f"state {expected_state}"
            )
        if group.state == "PROTECTED":
            fail(
                "coarse clusters cannot be marked PROTECTED; protected baselines are "
                "owned by the family-requirement canary registry"
            )

        capabilities = matrix_row["capability_ids"].split(",")
        if any(capability not in CAPABILITY_FAMILY_IMPLICATIONS for capability in capabilities):
            fail(f"{cluster_id} contains an unknown capability ID")
        payload_class = payload_row["stored_borrow_scope"]
        if payload_class not in PAYLOAD_SCOPE_DIMENSION_IMPLICATIONS:
            fail(f"{cluster_id} contains an unknown payload-scope class")

        trait_children = topology_by_cluster.get(cluster_id, [])
        trait_predecessors: set[str] = set()
        trait_predecessor_gates: set[str] = set()
        trait_families: set[str] = set()
        trait_gates: set[str] = set()
        for child in trait_children:
            if child["required_predecessor_family_ids"] != "NONE":
                trait_predecessors.update(
                    child["required_predecessor_family_ids"].split(",")
                )
            if child["required_predecessor_gate_stage_ids"] != "NONE":
                trait_predecessor_gates.update(
                    child["required_predecessor_gate_stage_ids"].split(",")
                )
            if child["implicated_or_reopening_family_ids"] != "NONE":
                trait_families.update(
                    child["implicated_or_reopening_family_ids"].split(",")
                )
            if child["implicated_or_reopening_gate_stage_ids"] != "NONE":
                trait_gates.update(
                    child["implicated_or_reopening_gate_stage_ids"].split(",")
                )

        implicated = set(group.extra_implicated) | trait_families
        if group.owner.startswith("F-"):
            implicated.add(group.owner)
        implicated_gates = set(trait_gates)
        if group.owner.startswith("GATE-"):
            implicated_gates.add(group.owner)
        if not implicated and not implicated_gates:
            fail(f"{cluster_id} has no implicated or reopening family/gate")
        if not implicated <= families:
            fail(f"{cluster_id} uses undocumented family IDs {sorted(implicated - families)}")
        if not implicated_gates <= gates:
            fail(f"{cluster_id} uses undocumented gate IDs {sorted(implicated_gates - gates)}")
        if group.owner.startswith("F-") and group.owner not in implicated:
            fail(f"{cluster_id} omits its primary family from applicability")

        required_dimensions = set(group.required_dimensions)
        for capability in capabilities:
            required_dimensions.update(CAPABILITY_DIMENSION_IMPLICATIONS[capability])
        required_dimensions.update(PAYLOAD_SCOPE_DIMENSION_IMPLICATIONS[payload_class])
        if not required_dimensions <= dimensions:
            fail(f"{cluster_id} uses undocumented local dimensions")

        predecessors = set(group.predecessors)
        predecessor_gates: set[str] = set()
        if group.owner.startswith("GATE-") and group.state == "ACTIVE" and trait_children:
            predecessors.update(trait_families)
            predecessor_gates.update(trait_predecessor_gates)
            predecessor_gates.update(trait_gates - {group.owner})
        if group.owner in predecessors:
            fail(f"{cluster_id} owner is also its predecessor")
        if not predecessor_gates <= gates or group.owner in predecessor_gates:
            fail(f"{cluster_id} has an invalid required predecessor gate set")
        predecessor_text = ordered_ids(predecessors, vocabulary_order)
        predecessor_gate_text = ordered_ids(predecessor_gates, vocabulary_order)
        implicated_text = ordered_ids(implicated, vocabulary_order)
        implicated_gate_text = ordered_ids(implicated_gates, vocabulary_order)
        dimension_text = ordered_ids(required_dimensions, vocabulary_order)
        delegated_owner_text = ordered_ids(
            set(group.delegated_owner_families), vocabulary_order
        )
        delegated_owner_contract_ids = "NONE"
        if group.state == "DELEGATED":
            delegated_contracts = payload_row["scope_owner_contract_ids"].split(";")
            if len(delegated_contracts) != 6 or len(set(delegated_contracts)) != 6:
                fail("delegated allocation payload scope is not six exact owner contracts")
            if any(contract_id not in routes for contract_id in delegated_contracts):
                fail("delegated allocation payload scope names an unknown cluster route")
            derived_owner_families = {routes[contract_id].owner for contract_id in delegated_contracts}
            if not derived_owner_families <= families:
                fail("delegated allocation payload scope resolves to a non-family owner")
            if derived_owner_families != set(group.delegated_owner_families):
                fail(
                    "delegated allocation family set differs from the exact payload-scope "
                    "owner-contract routes"
                )
            delegated_owner_contract_ids = ",".join(delegated_contracts)
        trait_predecessor_text = ordered_ids(trait_predecessors, vocabulary_order)
        trait_predecessor_gate_text = ordered_ids(
            trait_predecessor_gates, vocabulary_order
        )
        trait_family_text = ordered_ids(trait_families, vocabulary_order)
        trait_gate_text = ordered_ids(trait_gates, vocabulary_order)
        evidence_children = evidence_by_cluster[cluster_id]
        evidence_child_digest = sha256_bytes(
            "".join(
                f"{child['evidence_identity']}\n" for child in evidence_children
            ).encode("utf-8")
        )
        trait_impl_key_digest = sha256_bytes(
            "".join(f"{key}\n" for key in sorted(child["impl_key"] for child in trait_children)).encode()
        )
        route_category_id, route_category_rationale = route_category(
            group, bool(trait_children)
        )
        rationale_id = assignment_rationale_id(
            cluster_id, group, bool(trait_children)
        )
        route_spec_text = "\n".join(
            (
                cluster_id,
                group.owner,
                predecessor_text,
                predecessor_gate_text,
                ",".join(group.extra_implicated) if group.extra_implicated else "NONE",
                implicated_text,
                implicated_gate_text,
                dimension_text,
                delegated_owner_text,
                delegated_owner_contract_ids,
                trait_predecessor_text,
                trait_predecessor_gate_text,
                trait_family_text,
                trait_gate_text,
                trait_impl_key_digest,
                route_category_id,
                rationale_id,
                group.state,
            )
        ) + "\n"
        route_spec_sha256 = sha256_bytes(route_spec_text.encode("utf-8"))
        base = {
            "cluster_ordinal": str(ordinal),
            "cluster_id": cluster_id,
            "census_family": census_row["family"],
            "primary_refinement_owner_or_gate_stage": group.owner,
            "required_predecessor_family_ids": predecessor_text,
            "required_predecessor_gate_stage_ids": predecessor_gate_text,
            "implicated_or_reopening_family_ids": implicated_text,
            "implicated_or_reopening_gate_stage_ids": implicated_gate_text,
            "required_crosscut_dimension_ids": dimension_text,
            "delegated_owner_family_ids": delegated_owner_text,
            "delegated_owner_contract_ids": delegated_owner_contract_ids,
            "route_category_id": route_category_id,
            "route_category_rationale": route_category_rationale,
            "assignment_rationale_id": rationale_id,
            "route_state": group.state,
            "routing_granularity": ROUTING_GRANULARITY,
            "evidence_child_rebind_policy": EVIDENCE_CHILD_REBIND_POLICY,
            "prohibited_cluster_uses": PROHIBITED_CLUSTER_USES,
            "evidence_child_count": str(len(evidence_children)),
            "evidence_child_identity_sha256": evidence_child_digest,
            "trait_impl_relation_count": str(len(trait_children)),
            "trait_impl_key_sha256": trait_impl_key_digest,
            "trait_impl_required_predecessor_family_ids": trait_predecessor_text,
            "trait_impl_required_predecessor_gate_stage_ids": trait_predecessor_gate_text,
            "trait_impl_topology_family_ids": trait_family_text,
            "trait_impl_topology_gate_stage_ids": trait_gate_text,
            "trait_impl_primary_resolution_policy": TRAIT_IMPL_PRIMARY_RESOLUTION_POLICY,
            "matrix_status_code": matrix_row["status_code"],
            "payload_scope_class": payload_class,
            "routing_basis": ROUTING_BASIS,
            "applicability_policy": APPLICABILITY_POLICY,
            "source_artifacts": SOURCE_ARTIFACTS,
            "census_row_sha256": row_sha256(census_fields, census_row),
            "derivation_row_sha256": row_sha256(matrix_fields, matrix_row),
            "cluster_registry_row_sha256": row_sha256(cluster_fields, cluster_row),
            "payload_scope_row_sha256": row_sha256(payload_fields, payload_row),
            "route_spec_sha256": route_spec_sha256,
            "vocabulary_sha256": vocabulary_sha256,
            "family_requirement_registry_sha256": family_requirement_sha256,
            "evidence_universe_sha256": evidence_universe_sha256,
            "trait_impl_crosswalk_sha256": crosswalk_sha256,
            "trait_impl_topology_routing_sha256": topology_routing_sha256,
            "policy_version": POLICY_VERSION,
        }
        identity_fields = [base[field] for field in FIELDS if field not in {
            "route_identity",
            "primary_route_count",
            "primary_route_sha256",
            "state_route_count",
            "state_route_sha256",
        }]
        base["route_identity"] = sha256_bytes(
            ("\n".join(identity_fields) + "\n").encode("utf-8")
        )
        output_rows.append(base)

    by_primary: dict[str, list[dict[str, str]]] = collections.defaultdict(list)
    by_state: dict[str, list[dict[str, str]]] = collections.defaultdict(list)
    for row in output_rows:
        by_primary[row["primary_refinement_owner_or_gate_stage"]].append(row)
        by_state[row["route_state"]].append(row)
    for row in output_rows:
        primary_rows = by_primary[row["primary_refinement_owner_or_gate_stage"]]
        state_rows = by_state[row["route_state"]]
        row["primary_route_count"] = str(len(primary_rows))
        row["primary_route_sha256"] = digest_identities(primary_rows)
        row["state_route_count"] = str(len(state_rows))
        row["state_route_sha256"] = digest_identities(state_rows)
    return output_rows


def render(rows: list[dict[str, str]]) -> str:
    output = io.StringIO(newline="")
    writer = csv.DictWriter(output, fieldnames=FIELDS, delimiter="\t", lineterminator="\n")
    writer.writeheader()
    writer.writerows(rows)
    return output.getvalue()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--check",
        action="store_true",
        help="verify that the checked-in routing registry exactly matches generated bytes",
    )
    arguments = parser.parse_args()
    expected = render(build_rows())
    if arguments.check:
        if not OUTPUT.is_file() or OUTPUT.read_text(encoding="utf-8") != expected:
            fail(f"{OUTPUT.name} is missing or stale")
    else:
        OUTPUT.write_text(expected, encoding="utf-8")
    print(
        "G0 cluster family routing: PASS — 276 explicit many-to-many refinement "
        "routes; no unknown, duplicate, fuzzy, protected, or coarse-closure fallback"
    )


if __name__ == "__main__":
    main()
