#!/usr/bin/env python3
"""Verify exact cluster coverage and obligation coherence in DERIVATION-MATRIX.tsv."""

from __future__ import annotations

import argparse
import csv
import re
from collections import Counter
from pathlib import Path


EXPECTED_HEADER = [
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

VALID_STATUSES = {"E", "P", "U", "X", "FRAME", "DEFERRED", "BOUNDARY", "NG"}
FORBIDDEN_PLACEHOLDERS = {"todo", "tbd", "placeholder", "n/a", "unknown"}

STATUS_LEGEND = {
    "E": "coarse cluster envelope has a direct route; not an exact member or closure unit",
    "P": "coarse cluster envelope has pattern evidence; not an exact member or closure unit",
    "U": "unproved workaround or current subset lacking a complete proof or cost result",
    "X": "current semantic, soundness, asymptotic, or structural gap",
    "FRAME": "named trusted-boundary contract requiring a reviewed dossier; not authorization",
    "DEFERRED": "outside the current G0 domain; no G0 derivation is claimed",
    "BOUNDARY": "Rust boundary evidence whose unchecked or raw spelling is inadmissible while the underlying checked need remains routed",
    "NG": "owner-ratified writer-visible non-goal with any safe displacement tracked separately",
}

E_INCOMPATIBLE_REGISTRY_STATUSES = {
    "deferred_domain",
    "gap",
    "gap_or_scoped",
    "selected_unimplemented",
}

STATIC_ONLY_BEHAVIOR_CONTRACTS = {
    "ARR-VIEW-01",
    "VIEW-GET-01",
    "VIEW-GET-02",
    "TEXT-GET-01",
    "VIEW-DISJOINT-01",
    "HEAP-META-01",
    "OMAP-META-01",
    "OSET-META-01",
    "HMAP-META-01",
    "HSET-META-01",
    "OMAP-END-01",
    "OMAP-ITER-01",
    "MAP-OCCUPIED-01",
    "MAP-VACANT-01",
    "BOX-DOWNCAST-01",
    "RC-DOWNCAST-01",
    "TRAIT-FUSED-01",
}

# These rows were the closed shared-receiver repair. Eighteen previously lacked
# all three runtime-behavior obligations; RangeBounds lacked FL-CALLBACK and
# EX-ABORT; Index already had AB-BEHAVIOR and EX-ABORT but lacked FL-CALLBACK.
RUNTIME_BEHAVIOR_REPAIR_CONTRACTS = {
    "VIEW-COPY-01",
    "SEQ-DRAIN-01",
    "DEQUE-RANGE-01",
    "DEQUE-DRAIN-01",
    "STRING-PUSH-01",
    "STRING-DRAIN-01",
    "STRING-REPLACE-01",
    "TRAIT-INTOITER-01",
    "TRAIT-ITER-01",
    "TRAIT-DOUBLE-01",
    "TRAIT-DEREF-01",
    "TRAIT-BORROW-01",
    "TRAIT-EXACT-01",
    "ITER-ADAPT-CHAIN-01",
    "ITER-ADAPT-ZIP-01",
    "ITER-ADAPT-REBORROW-01",
    "ITER-ADAPT-DIRECTION-01",
    "ITER-ADAPT-FUSE-01",
    "RANGE-BOUNDS-PROTOCOL-01",
    "TRAIT-INDEX-01",
}

CLONE_SOURCE_EFFECT_CONTRACTS = {
    "VIEW-CLONE-01",
    "VIEW-FILL-01",
    "VIEW-ALLOC-01",
    "VIEW-CONCAT-01",
    "INIT-WRITE-01",
    "SEQ-EXTEND-COPY-01",
    "SEQ-RESIZE-01",
    "DEQUE-RESIZE-01",
    "RC-UNIQUE-01",
    "RC-UNWRAP-01",
    "ITER-SOURCE-REPEAT-01",
    "ITER-ADAPT-DUPLICATE-01",
    "ITER-ADAPT-CYCLE-01",
    "RANGE-BOUND-CLONE-01",
    "TRAIT-CONVERT-01",
    "TRAIT-CLONE-01",
}

EXPECTED_AB_BEHAVIOR_COUNT = 127

SEALED_STABLE_STEP_RANGE_CAPABILITIES = {
    "RANGE-ITER-HALFOPEN-01": {
        "EX-NORMAL", "EX-ABANDON", "BR-CURSOR", "IT-OWN", "IT-COMPOSE"
    },
    "RANGE-ITER-FROM-01": {
        "EX-NORMAL", "EX-ABANDON", "EX-ABORT", "BR-CURSOR", "IT-OWN", "IT-COMPOSE"
    },
    "RANGE-ITER-INCLUSIVE-01": {
        "EX-NORMAL", "EX-ABANDON", "BR-CURSOR", "IT-OWN", "IT-COMPOSE"
    },
}

SEALED_STABLE_STEP_TYPE_LIST = (
    "u8,u16,u32,u64,u128,usize,i8,i16,i32,i64,i128,isize,char,"
    "NonZero<u8>,NonZero<u16>,NonZero<u32>,NonZero<u64>,NonZero<u128>,"
    "NonZero<usize>,Ipv4Addr,Ipv6Addr"
)
SEALED_STEP_SOURCE_HASH = "ae3f9307f4b4972f418561ae2a0311586eb3dde782359b8aaef3244915256464"

RANGE_BOUNDS_DESCRIPTOR_CONTRACTS = {
    "VIEW-COPY-01",
    "SEQ-EXTEND-COPY-01",
    "SEQ-DRAIN-01",
    "DEQUE-RANGE-01",
    "DEQUE-DRAIN-01",
    "OMAP-RANGE-01",
    "OSET-RANGE-01",
    "STRING-PUSH-01",
    "STRING-DRAIN-01",
    "STRING-REPLACE-01",
}
ACTIVE_RANGE_BOUNDS_STATE_CONTRACTS = {
    "SEQ-EXTRACT-01",
    "SEQ-SPLICE-01",
    "OMAP-FILTER-01",
    "OSET-FILTER-01",
}
BASE_RANGE_BOUNDS_BEHAVIOR_CONTRACTS = {
    "RANGE-BOUNDS-PROTOCOL-01",
    "RANGE-BOUNDS-CONTAINS-01",
}

RANGE_QUERY_EFFECT_CONTRACTS = {
    "RANGE-CONTAINS-HALFOPEN-01",
    "RANGE-EMPTY-HALFOPEN-01",
    "RANGE-CONTAINS-FROM-01",
    "RANGE-CONTAINS-INCLUSIVE-01",
    "RANGE-EMPTY-INCLUSIVE-01",
    "RANGE-CONTAINS-TO-INCLUSIVE-01",
    "RANGE-BOUNDS-CONTAINS-01",
    "RANGE-LEGACY-INCLUSIVE-CONTAINS-01",
    "RANGE-LEGACY-INCLUSIVE-EMPTY-01",
    "RANGE-CONTAINS-TO-EXCLUSIVE-01",
}

NON_ATOMIC_CONTRACTS = {"TRAIT-EXTEND-01", "TRAIT-COLLECT-01"}
INTERNAL_ONLY_DISJOINTNESS_CONTRACTS = {
    "ARR-VIEW-01",
    "VIEW-GET-01",
    "VIEW-GET-02",
    "VIEW-SORT-01",
    "VIEW-SORT-02",
    "VIEW-REORDER-01",
}

FACT_PHRASES = {
    "FT-STATE": "live/occupancy facts",
    "FT-REFINE": "refinement facts",
    "FT-IDENTITY": "identity facts",
    "FT-BORROW": "dynamic-borrow facts",
    "FT-SHARED": "shared-lifecycle facts",
}

CANARY_PHRASES = {
    "OW-DROP": "verify exact live-value drops",
    "OW-RELOCATE": "detect moved-from read",
    "BR-DISJOINT": "duplicate/overlapping mutable outputs",
    "FL-ALLOC": "inject allocation/capacity failure",
    "FL-CALLBACK": "trap/fail the invoked behavior",
    "FT-STATE": "recorded live-state invalidator",
    "FT-REFINE": "reject invalid refinement",
    "FT-BORROW": "borrow-count overflow",
    "FT-SHARED": "last-strong",
}

EFF4_CLAUSE = "Under xlang [EFF-4], an invoked-behavior trap aborts immediately"
GENERIC_EFF4_CLAUSE = "Under xlang [EFF-4], a trap aborts immediately"
EFF4_REFERENCE = "spec/kernel-spec-v0.6.md [EFF-4]"
OP9_OOM_CLAUSE = "Under xlang [OP-9], OOM is a TCB-level divergent allocation-policy edge"
OP9_REFERENCE = "spec/kernel-spec-v0.6.md [OP-9]"

COMPOSITION_FAMILIES = {
    "iteration_protocol",
    "iteration_producer",
    "iteration_adapter",
    "iteration_consumer",
    "range_iteration",
    "bulk_construction_protocol",
}

RESULT_BORROW_BOUNDARY_EXCEPTIONS = {
    "RC-CYCLIC-01",
    "RAW-SAFE-PTR-01",
    "RAW-SAFE-LEAK-01",
    "RAW-UNSAFE-ACCESS-01",
    "RAW-UNSAFE-BORROW-01",
}

OWNERSHIP_NORMAL_EXIT_BOUNDARY_EXCEPTIONS = {"RAW-SAFE-LEAK-01"}

RETURNED_CURSOR_CONTRACTS = {
    "SEQ-DRAIN-01",
    "SEQ-EXTRACT-01",
    "SEQ-SPLICE-01",
    "DEQUE-DRAIN-01",
    "HEAP-DRAIN-01",
    "LIST-EXTRACT-01",
    "OMAP-FILTER-01",
    "OSET-FILTER-01",
    "HMAP-DRAIN-01",
    "HMAP-FILTER-01",
    "HSET-DRAIN-01",
    "HSET-FILTER-01",
    "STRING-DRAIN-01",
}

UNIQUE_RESULT_REBORROW_CONTRACTS = {
    "ARR-VIEW-01",
    "ARR-EACH-01",
    "VIEW-ARRAY-01",
    "VIEW-SPLIT-01",
    "VIEW-SPLIT-02",
    "VIEW-ARRAY-CHUNKS-01",
    "VIEW-CHUNKS-01",
    "VIEW-CHUNKBY-01",
    "TEXT-GET-01",
    "TEXT-SPLIT-AT-01",
    "TEXT-SPLIT-AT-02",
    "SEQ-VIEW-01",
    "DEQUE-VIEW-01",
    "DEQUE-CONTIG-01",
    "DEQUE-RANGE-01",
    "HEAP-PEEK-01",
    "OMAP-RANGE-01",
    "HMAP-LOOKUP-01",
    "HMAP-DISJOINT-01",
    "STRING-VIEW-01",
    "TRAIT-INDEX-01",
    "TRAIT-DEREF-01",
    "TRAIT-BORROW-01",
    "HELPER-CURSOR-VIEW-01",
    "VIEW-END-CHUNK-01",
    "VIEW-END-SPLIT-01",
    "VIEW-DISJOINT-01",
    "INIT-WRITE-01",
    "TEXT-VALIDATE-01",
    "SEQ-PUSH-01",
    "SEQ-INSERT-01",
    "DEQUE-PUSH-01",
    "DEQUE-INSERT-01",
    "LIST-PUSH-01",
    "RC-UNIQUE-01",
    "REFCELL-OWNER-01",
    "HELPER-ARRAY-INTOITER-01",
}

UNIQUE_CURSOR_REBORROW_CONTRACTS = {
    "VIEW-ITER-01",
    "VIEW-SPLIT-PRED-01",
    "DEQUE-ITER-01",
    "LIST-ITER-01",
    "OMAP-ITER-01",
    "HMAP-ITER-01",
}

DIRECT_REBORROW_CENSUS_CONTRACTS = {
    "VIEW-END-CHUNK-01",
    "VIEW-END-SPLIT-01",
    "VIEW-DISJOINT-01",
    "INIT-WRITE-01",
    "TEXT-VALIDATE-01",
    "SEQ-PUSH-01",
    "SEQ-INSERT-01",
    "DEQUE-PUSH-01",
    "DEQUE-INSERT-01",
    "LIST-PUSH-01",
    "RC-UNIQUE-01",
    "REFCELL-OWNER-01",
    "HELPER-ARRAY-INTOITER-01",
}

ITERATOR_ADAPTER_RESULT_CONTRACTS = {
    "ITER-ADAPT-TRANSFORM-01",
    "ITER-ADAPT-DUPLICATE-01",
    "ITER-ADAPT-SELECT-01",
    "ITER-ADAPT-POSITION-01",
    "ITER-ADAPT-CHAIN-01",
    "ITER-ADAPT-ZIP-01",
    "ITER-ADAPT-NEST-01",
    "ITER-ADAPT-STATE-01",
    "ITER-ADAPT-PEEK-01",
    "ITER-ADAPT-REBORROW-01",
    "ITER-ADAPT-DIRECTION-01",
    "ITER-ADAPT-FUSE-01",
    "ITER-ADAPT-CYCLE-01",
}

RETAINED_BORROW_STATE_CONTRACTS = {
    "VIEW-CHUNKBY-01",
    "VIEW-SPLIT-PRED-01",
    "TEXT-MATCH-ITER-01",
    "TEXT-SPLIT-PATTERN-01",
    "SEQ-EXTRACT-01",
    "LIST-EXTRACT-01",
    "OMAP-FILTER-01",
    "OSET-FILTER-01",
    "HMAP-FILTER-01",
    "HSET-FILTER-01",
    "ITER-SOURCE-VALUE-01",
    "ITER-SOURCE-REPEAT-01",
    "ITER-SOURCE-CALLBACK-01",
    "ITER-ADAPT-TRANSFORM-01",
    "ITER-ADAPT-SELECT-01",
    "ITER-ADAPT-POSITION-01",
    "ITER-ADAPT-NEST-01",
    "ITER-ADAPT-STATE-01",
    "ITER-ADAPT-PEEK-01",
    "ITER-CONSUME-FOLD-01",
    "ITER-CONSUME-SHORT-01",
    "ITER-CONSUME-RELATION-01",
    "ITER-CONSUME-FANOUT-01",
}

RETAINED_CALLABLE_CURSOR_CONTRACTS = {
    "VIEW-CHUNKBY-01",
    "VIEW-SPLIT-PRED-01",
    "TEXT-MATCH-ITER-01",
    "TEXT-SPLIT-PATTERN-01",
    "SEQ-EXTRACT-01",
    "LIST-EXTRACT-01",
    "OMAP-FILTER-01",
    "OSET-FILTER-01",
    "HMAP-FILTER-01",
    "HSET-FILTER-01",
}

CURSOR_ONLY_BORROW_STATE_CONTRACTS = {
    "ITER-ADAPT-DUPLICATE-01",
    "ITER-ADAPT-CHAIN-01",
    "ITER-ADAPT-ZIP-01",
    "ITER-ADAPT-REBORROW-01",
    "ITER-ADAPT-DIRECTION-01",
    "ITER-ADAPT-FUSE-01",
    "ITER-ADAPT-CYCLE-01",
}

NONLENDING_UNIQUE_DISJOINT_CONTRACTS = {
    "VIEW-ITER-01",
    "VIEW-CHUNKS-01",
    "VIEW-CHUNKBY-01",
    "VIEW-SPLIT-PRED-01",
    "DEQUE-RANGE-01",
    "DEQUE-ITER-01",
    "LIST-ITER-01",
    "OMAP-RANGE-01",
    "OMAP-ITER-01",
    "HMAP-ITER-01",
    "TRAIT-INTOITER-01",
    "TRAIT-ITER-01",
    "TRAIT-DOUBLE-01",
    "ITER-ADAPT-TRANSFORM-01",
    "ITER-ADAPT-SELECT-01",
    "ITER-ADAPT-POSITION-01",
    "ITER-ADAPT-CHAIN-01",
    "ITER-ADAPT-ZIP-01",
    "ITER-ADAPT-NEST-01",
    "ITER-ADAPT-STATE-01",
    "ITER-ADAPT-PEEK-01",
    "ITER-ADAPT-REBORROW-01",
    "ITER-ADAPT-DIRECTION-01",
    "ITER-ADAPT-FUSE-01",
    "ITER-ADAPT-CYCLE-01",
}

UNIQUE_CONSUMER_RETAINED_DISJOINT_CONTRACTS = {
    "ITER-CONSUME-FOLD-01",
    "ITER-CONSUME-SHORT-01",
    "ITER-CONSUME-RELATION-01",
    "ITER-CONSUME-FANOUT-01",
    "TRAIT-EXTEND-01",
    "TRAIT-COLLECT-01",
}

STORED_ITEM_CONSUMER_CONTRACTS = {"TRAIT-EXTEND-01", "TRAIT-COLLECT-01"}

OWNED_YIELD_RETURNED_CURSOR_CONTRACTS = RETURNED_CURSOR_CONTRACTS

SAME_INDEX_SWAP_CONTRACTS = {"VIEW-SWAP-01", "DEQUE-SWAP-01"}

ITERATOR_ADAPTER_PROVENANCE_FRAGMENTS = {
    "ITER-ADAPT-TRANSFORM-01": ("map output provenance", "returned capture from owner A"),
    "ITER-ADAPT-DUPLICATE-01": ("copied structurally preserves", "Copy &&A to &A"),
    "ITER-ADAPT-SELECT-01": ("filter returns the accepted upstream Item", "source B and capture A"),
    "ITER-ADAPT-POSITION-01": ("map_while output follows its callable contract", "map_while with source B and capture A"),
    "ITER-ADAPT-CHAIN-01": ("branch-tagged first- or second-cursor provenance", "Swap first/second source tags"),
    "ITER-ADAPT-ZIP-01": ("tuple field retains the exact provenance", "Swap tuple-field provenance"),
    "ITER-ADAPT-NEST-01": ("actual active front/back inner cursor yield", "outer source B with mapper capture A"),
    "ITER-ADAPT-STATE-01": ("fixed B cannot freshly borrow adapter-owned state", "pre-existing external reference stored in State"),
    "ITER-ADAPT-PEEK-01": ("peek and peek_mut return receiver-bounded outer borrows", "Hold peek or peek_mut"),
    "ITER-ADAPT-REBORROW-01": ("by_ref returns a receiver-bounded &mut Self", "Reject wrapper escape"),
    "ITER-ADAPT-DIRECTION-01": ("rev preserves the exact provenance", "Reject attribution to direction-adapter storage"),
    "ITER-ADAPT-FUSE-01": ("pre-terminal Item preserves exact upstream provenance", "Reject provenance minted from terminal/fused metadata"),
    "ITER-ADAPT-CYCLE-01": ("current original or cloned-cursor epoch", "Reject assumed original-owner provenance"),
}

OFFERED_OWNER_FAILURE_CONTRACTS = {
    "SEQ-PUSH-01",
    "SEQ-INSERT-01",
    "DEQUE-PUSH-01",
    "DEQUE-INSERT-01",
    "LIST-PUSH-01",
    "OMAP-INSERT-01",
    "MAP-ENTRY-01",
    "MAP-VACANT-01",
    "OSET-INSERT-01",
    "HMAP-INSERT-01",
    "HSET-INSERT-01",
}

RELOCATION_REPAIR_CONTRACTS = {
    "HMAP-INSERT-01",
    "SEQ-DRAIN-01",
    "SEQ-EXTRACT-01",
    "DEQUE-RESIZE-01",
    "DEQUE-DRAIN-01",
    "OMAP-FILTER-01",
    "OSET-FILTER-01",
    "STRING-POP-01",
    "STRING-DRAIN-01",
}

PLAIN_NORMAL_EXIT_REPAIRS = {
    "VIEW-REORDER-01",
    "VIEW-SWAP-01",
    "VIEW-CLONE-01",
    "VIEW-FILL-01",
    "DEQUE-SWAP-01",
    "RANGE-BOUND-CLONE-01",
    "RANGE-BOUND-MAP-01",
    "HELPER-REMAINDER-01",
    "HELPER-CURSOR-VIEW-01",
}

REPAIRED_RESULT_BORROW_CONTRACTS = {
    "VIEW-SELECT-01",
    "INIT-WRITE-01",
    "SEQ-PUSH-01",
    "SEQ-INSERT-01",
    "DEQUE-CONTIG-01",
    "DEQUE-PUSH-01",
    "DEQUE-INSERT-01",
    "LIST-PUSH-01",
    "MAP-ENTRY-01",
    "MAP-OCCUPIED-01",
    "MAP-VACANT-01",
    "HMAP-META-01",
    "HSET-META-01",
    "STRING-DECODE-LOSSY-01",
    "STRING-DECODE-ERROR-01",
    "TRAIT-CONVERT-01",
    "TRAIT-DOUBLE-01",
    "ITER-CONSUME-FOLD-01",
    "ITER-CONSUME-SHORT-01",
    "ITER-CONSUME-RELATION-01",
    "TRAIT-COLLECT-01",
}

EXACT_REPAIR_CAPABILITIES = {
    "VIEW-SWAP-01": "ST-FULL,OW-SWAP,EX-NORMAL,EX-ABORT,BR-DISJOINT",
    "VIEW-CHUNKBY-01": "ST-FULL,OW-MOVEOUT,OW-DROP,EX-NORMAL,EX-ABANDON,EX-ABORT,BR-PROV,BR-REBORROW,BR-RESULT,BR-STORED,BR-DISJOINT,BR-CURSOR,FL-CALLBACK,AB-BEHAVIOR,AB-STATEFUL,IT-SHARED,IT-UNIQ",
    "VIEW-SPLIT-PRED-01": "ST-FULL,OW-MOVEOUT,OW-DROP,EX-NORMAL,EX-ABANDON,EX-ABORT,BR-PROV,BR-REBORROW,BR-RESULT,BR-STORED,BR-DISJOINT,BR-CURSOR,FL-CALLBACK,AB-BEHAVIOR,AB-STATEFUL,IT-SHARED,IT-UNIQ",
    "TEXT-MATCH-ITER-01": "ST-REFINE,OW-MOVEOUT,OW-DROP,EX-NORMAL,EX-ABANDON,EX-ABORT,BR-PROV,BR-RESULT,BR-STORED,BR-CURSOR,FL-CALLBACK,AB-BEHAVIOR,AB-STATEFUL,IT-SHARED,FT-REFINE",
    "TEXT-SPLIT-PATTERN-01": "ST-REFINE,OW-MOVEOUT,OW-DROP,EX-NORMAL,EX-ABANDON,EX-ABORT,BR-PROV,BR-RESULT,BR-STORED,BR-CURSOR,FL-CALLBACK,AB-BEHAVIOR,AB-STATEFUL,IT-SHARED,FT-REFINE",
    "SEQ-EXTRACT-01": "ST-DENSE,ST-HOLE,OW-MOVEOUT,OW-RELOCATE,OW-DROP,EX-NORMAL,EX-ABANDON,EX-ABORT,BR-PROV,BR-REBORROW,BR-RESULT,BR-STORED,BR-INVALIDATE,BR-CURSOR,FL-CALLBACK,AB-BEHAVIOR,AB-STATEFUL,IT-OWN,FT-STATE",
    "SEQ-SPLICE-01": "ST-DENSE,ST-HOLE,OW-INIT,OW-MOVEOUT,OW-RELOCATE,OW-DROP,EX-NORMAL,EX-ABANDON,EX-ABORT,BR-PROV,BR-REBORROW,BR-RESULT,BR-STORED,BR-DISJOINT,BR-INVALIDATE,BR-CURSOR,FL-CAPACITY,FL-ALLOC,FL-CALLBACK,AB-BEHAVIOR,AB-STATEFUL,AB-GENERIC,IT-SHARED,IT-UNIQ,IT-OWN,IT-COMPOSE,FT-STATE",
    "DEQUE-SWAP-01": "ST-RING,OW-SWAP,EX-NORMAL,EX-ABORT,BR-DISJOINT,FT-STATE",
    "LIST-EXTRACT-01": "ST-DEPENDENT,ST-HOLE,OW-MOVEOUT,OW-DROP,EX-NORMAL,EX-ABANDON,EX-ABORT,BR-PROV,BR-REBORROW,BR-RESULT,BR-STORED,BR-INVALIDATE,BR-CURSOR,FL-CALLBACK,AB-BEHAVIOR,AB-STATEFUL,IT-OWN",
    "OMAP-FILTER-01": "ST-DEPENDENT,ST-HOLE,OW-MOVEOUT,OW-RELOCATE,OW-DROP,EX-NORMAL,EX-ABANDON,EX-ABORT,BR-PROV,BR-REBORROW,BR-RESULT,BR-STORED,BR-INVALIDATE,BR-CURSOR,FL-CALLBACK,AB-BEHAVIOR,AB-STATEFUL,IT-OWN",
    "OSET-FILTER-01": "ST-DEPENDENT,ST-HOLE,OW-MOVEOUT,OW-RELOCATE,OW-DROP,EX-NORMAL,EX-ABANDON,EX-ABORT,BR-PROV,BR-REBORROW,BR-RESULT,BR-STORED,BR-INVALIDATE,BR-CURSOR,FL-CALLBACK,AB-BEHAVIOR,AB-STATEFUL,IT-OWN",
    "HMAP-FILTER-01": "ST-SPARSE,ST-HOLE,OW-MOVEOUT,OW-DROP,EX-NORMAL,EX-ABANDON,EX-ABORT,BR-PROV,BR-REBORROW,BR-RESULT,BR-STORED,BR-INVALIDATE,BR-CURSOR,FL-CALLBACK,AB-BEHAVIOR,AB-STATEFUL,IT-OWN,FT-STATE",
    "HSET-FILTER-01": "ST-SPARSE,ST-HOLE,OW-MOVEOUT,OW-DROP,EX-NORMAL,EX-ABANDON,EX-ABORT,BR-PROV,BR-REBORROW,BR-RESULT,BR-STORED,BR-INVALIDATE,BR-CURSOR,FL-CALLBACK,AB-BEHAVIOR,AB-STATEFUL,IT-OWN,FT-STATE",
    "TRAIT-EXTEND-01": "ST-DENSE,ST-RING,ST-SPARSE,ST-DEPENDENT,OW-INIT,OW-MOVEOUT,OW-RELOCATE,OW-DROP,EX-NORMAL,EX-ABORT,BR-PROV,BR-RESULT,BR-STORED,BR-DISJOINT,BR-CURSOR,FL-CAPACITY,FL-ALLOC,FL-CALLBACK,AB-SEAL,AB-BEHAVIOR,AB-STATEFUL,AB-GENERIC,IT-SHARED,IT-UNIQ,IT-OWN,IT-COMPOSE,FT-STATE,NT-FIXED,NT-P2",
    "TRAIT-COLLECT-01": "ST-DENSE,ST-RING,ST-SPARSE,ST-DEPENDENT,OW-INIT,OW-MOVEOUT,OW-RELOCATE,OW-DROP,EX-NORMAL,EX-ABANDON,EX-ABORT,BR-PROV,BR-RESULT,BR-STORED,BR-DISJOINT,BR-CURSOR,FL-CAPACITY,FL-ALLOC,FL-CALLBACK,AB-SEAL,AB-BEHAVIOR,AB-STATEFUL,AB-GENERIC,IT-SHARED,IT-UNIQ,IT-OWN,IT-COMPOSE,FT-STATE,NT-FIXED,NT-P2",
    "ITER-CONSUME-FOLD-01": "OW-MOVEOUT,OW-DROP,EX-NORMAL,EX-ABANDON,EX-ABORT,BR-PROV,BR-RESULT,BR-STORED,BR-DISJOINT,BR-CURSOR,FL-CALLBACK,AB-BEHAVIOR,AB-STATEFUL,AB-GENERIC,IT-SHARED,IT-UNIQ,IT-OWN,IT-COMPOSE",
    "ITER-CONSUME-SHORT-01": "OW-MOVEOUT,OW-DROP,EX-NORMAL,EX-ABANDON,EX-ABORT,BR-PROV,BR-RESULT,BR-STORED,BR-DISJOINT,BR-CURSOR,FL-CALLBACK,AB-BEHAVIOR,AB-STATEFUL,AB-GENERIC,IT-SHARED,IT-UNIQ,IT-OWN,IT-COMPOSE",
    "ITER-CONSUME-RELATION-01": "OW-MOVEOUT,OW-DROP,EX-NORMAL,EX-ABANDON,EX-ABORT,BR-PROV,BR-RESULT,BR-STORED,BR-DISJOINT,BR-CURSOR,FL-CALLBACK,AB-BEHAVIOR,AB-STATEFUL,AB-GENERIC,IT-SHARED,IT-UNIQ,IT-OWN,IT-COMPOSE",
    "ITER-CONSUME-FANOUT-01": "ST-DENSE,ST-RING,ST-SPARSE,ST-DEPENDENT,OW-INIT,OW-MOVEOUT,OW-RELOCATE,OW-DROP,EX-NORMAL,EX-ABANDON,EX-ABORT,BR-PROV,BR-RESULT,BR-STORED,BR-DISJOINT,BR-CURSOR,FL-CAPACITY,FL-ALLOC,FL-CALLBACK,AB-SEAL,AB-BEHAVIOR,AB-STATEFUL,AB-GENERIC,IT-SHARED,IT-UNIQ,IT-OWN,IT-COMPOSE,FT-STATE,NT-FIXED,NT-P2",
    "RANGE-BOUNDS-PROTOCOL-01": "EX-ABORT,BR-PROV,BR-RESULT,FL-CALLBACK,AB-BEHAVIOR,AB-GENERIC",
    "RANGE-VALUE-INCLUSIVE-01": "OW-DROP,EX-NORMAL,AB-GENERIC",
    "TRAIT-CLONE-01": "OW-REPLACE,OW-CLONE,OW-DROP,EX-NORMAL,EX-ABANDON,EX-ABORT,BR-PROV,BR-RESULT,BR-CURSOR,FL-ALLOC,FL-CALLBACK,ID-SHARED,AB-SEAL,AB-BEHAVIOR,AB-GENERIC,FT-BORROW,FT-SHARED",
    "TRAIT-CONVERT-01": "ST-DENSE,ST-REFINE,OW-INIT,OW-MOVEOUT,OW-RELOCATE,OW-CLONE,OW-DROP,EX-NORMAL,EX-ABORT,BR-PROV,BR-RESULT,FL-ALLOC,FL-ATOMIC,FL-CALLBACK,AB-BEHAVIOR,AB-GENERIC,FT-REFINE",
    "VIEW-CONCAT-01": "ST-DENSE,OW-INIT,OW-CLONE,OW-DROP,EX-NORMAL,EX-ABORT,FL-CAPACITY,FL-ALLOC,FL-ATOMIC,FL-CALLBACK,AB-BEHAVIOR,AB-GENERIC,NT-FIXED",
    "TRAIT-DEFAULT-01": "OW-INIT,EX-NORMAL,EX-ABORT,BR-PROV,BR-RESULT,FL-ALLOC,FL-CALLBACK,AB-BEHAVIOR,AB-GENERIC",
    "TEXT-PARSE-01": "ST-REFINE,EX-NORMAL,EX-ABORT,BR-PROV,BR-RESULT,FL-CALLBACK,AB-BEHAVIOR,AB-GENERIC,FT-REFINE",
    "HEAP-PEEK-01": "ST-DENSE,ST-HOLE,OW-MOVEOUT,OW-SWAP,OW-RELOCATE,EX-NORMAL,EX-ABANDON,EX-ABORT,BR-PROV,BR-REBORROW,BR-RESULT,BR-INVALIDATE,BR-CURSOR,FL-CALLBACK,AB-BEHAVIOR,FT-STATE",
    "VIEW-CLONE-01": "ST-FULL,OW-CLONE,EX-NORMAL,EX-ABORT,FL-CALLBACK,AB-BEHAVIOR",
    "VIEW-FILL-01": "ST-FULL,OW-REPLACE,OW-CLONE,OW-DROP,EX-NORMAL,EX-ABORT,FL-CALLBACK,AB-BEHAVIOR,AB-STATEFUL",
    "HEAP-DRAIN-01": "ST-DENSE,OW-MOVEOUT,OW-DROP,EX-NORMAL,EX-ABANDON,BR-PROV,BR-REBORROW,BR-RESULT,BR-INVALIDATE,BR-CURSOR,IT-OWN,FT-STATE",
    "RAW-SAFE-LEAK-01": "OW-MOVEOUT,ID-ADDRESS",
    "BOX-DOWNCAST-01": "OW-MOVEOUT,EX-NORMAL,FL-ATOMIC,AB-SEAL",
    "RC-DOWNCAST-01": "OW-MOVEOUT,EX-NORMAL,FL-ATOMIC,ID-SHARED,AB-SEAL,FT-SHARED",
    "RC-UNWRAP-01": "OW-MOVEOUT,OW-CLONE,OW-DROP,EX-NORMAL,EX-ABORT,FL-ATOMIC,FL-CALLBACK,ID-SHARED,AB-BEHAVIOR,FT-SHARED",
    "VIEW-CONSUME-01": "EX-NORMAL,BR-PROV,BR-REBORROW,BR-RESULT,BR-DISJOINT,BR-INVALIDATE,BR-CURSOR,FL-ATOMIC",
    "SEQ-DEDUP-01": "ST-DENSE,ST-HOLE,OW-RELOCATE,OW-DROP,EX-NORMAL,EX-ABORT,BR-DISJOINT,FL-CALLBACK,AB-BEHAVIOR,AB-STATEFUL,IT-UNIQ,FT-STATE",
    "RC-UNIQUE-01": "OW-REPLACE,OW-RELOCATE,OW-CLONE,OW-DROP,EX-NORMAL,EX-ABORT,BR-PROV,BR-REBORROW,BR-RESULT,FL-ALLOC,FL-CALLBACK,ID-SHARED,AB-BEHAVIOR,FT-SHARED",
    "REFCELL-OWNER-01": "OW-INIT,OW-MOVEOUT,OW-DROP,EX-NORMAL,BR-PROV,BR-REBORROW,BR-RESULT,AB-SEAL",
    "REF-GUARD-01": "EX-NORMAL,EX-ABANDON,EX-ABORT,BR-PROV,BR-RESULT,BR-DISJOINT,BR-INVALIDATE,BR-CURSOR,FL-CALLBACK,AB-SEAL,AB-BEHAVIOR,AB-STATEFUL,FT-BORROW",
    "HELPER-REMAINDER-01": "EX-NORMAL,BR-PROV,BR-RESULT,BR-DISJOINT,BR-INVALIDATE,BR-CURSOR",
    "HELPER-ARRAY-INTOITER-01": "ST-FULL,ST-HOLE,OW-MOVEOUT,OW-DROP,EX-NORMAL,EX-ABANDON,BR-PROV,BR-REBORROW,BR-RESULT,BR-INVALIDATE,BR-CURSOR,AB-SEAL,IT-OWN,FT-STATE",
    "VIEW-SORT-02": "ST-FULL,ST-HOLE,OW-SWAP,OW-RELOCATE,EX-NORMAL,EX-ABORT,FL-CALLBACK,AB-BEHAVIOR,AB-STATEFUL",
    "VIEW-SELECT-01": "ST-FULL,ST-HOLE,OW-SWAP,OW-RELOCATE,EX-NORMAL,EX-ABORT,BR-PROV,BR-RESULT,BR-DISJOINT,FL-CALLBACK,AB-BEHAVIOR,AB-STATEFUL",
    "DEQUE-RANGE-01": "ST-RING,EX-NORMAL,EX-ABANDON,EX-ABORT,BR-PROV,BR-REBORROW,BR-RESULT,BR-DISJOINT,BR-INVALIDATE,BR-CURSOR,FL-CALLBACK,AB-BEHAVIOR,IT-SHARED,IT-UNIQ,FT-STATE",
    "OMAP-END-01": "ST-DEPENDENT,EX-NORMAL,EX-ABANDON,BR-PROV,BR-REBORROW,BR-RESULT,BR-INVALIDATE,BR-CURSOR",
    "MAP-ENTRY-01": "ST-SPARSE,ST-DEPENDENT,ST-HOLE,OW-INIT,OW-REPLACE,OW-RELOCATE,OW-DROP,EX-NORMAL,EX-ABANDON,EX-ABORT,BR-PROV,BR-REBORROW,BR-RESULT,BR-INVALIDATE,BR-CURSOR,FL-CAPACITY,FL-ALLOC,FL-ATOMIC,FL-CALLBACK,AB-SEAL,AB-BEHAVIOR,AB-STATEFUL,FT-STATE",
    "MAP-OCCUPIED-01": "ST-SPARSE,ST-DEPENDENT,ST-HOLE,OW-MOVEOUT,OW-REPLACE,OW-RELOCATE,OW-DROP,EX-NORMAL,EX-ABANDON,BR-PROV,BR-REBORROW,BR-RESULT,BR-INVALIDATE,BR-CURSOR,AB-SEAL,FT-STATE",
    "MAP-VACANT-01": "ST-SPARSE,ST-DEPENDENT,ST-HOLE,OW-INIT,OW-MOVEOUT,OW-RELOCATE,OW-DROP,EX-NORMAL,EX-ABANDON,EX-ABORT,BR-PROV,BR-RESULT,BR-INVALIDATE,BR-CURSOR,FL-CAPACITY,FL-ALLOC,FL-ATOMIC,AB-SEAL,FT-STATE",
    "STRING-DECODE-STRICT-01": "ST-DENSE,ST-REFINE,OW-INIT,OW-DROP,EX-NORMAL,AB-SEAL,FT-STATE,FT-REFINE",
    "STRING-DECODE-LOSSY-01": "ST-DENSE,ST-REFINE,OW-INIT,OW-DROP,EX-NORMAL,EX-ABORT,BR-PROV,BR-RESULT,FL-CAPACITY,FL-ALLOC,FL-ATOMIC,AB-SEAL,FT-STATE,FT-REFINE",
    "STRING-DECODE-ERROR-01": "ST-DENSE,ST-REFINE,OW-MOVEOUT,OW-DROP,EX-NORMAL,BR-PROV,BR-RESULT,AB-SEAL,FT-STATE,FT-REFINE",
    "HMAP-META-01": "ST-SPARSE,OW-DROP,EX-NORMAL,EX-ABORT,BR-PROV,BR-RESULT,FL-CAPACITY,FL-ALLOC,AB-SEAL,AB-GENERIC,NT-FIXED,NT-P2",
    "HSET-META-01": "ST-SPARSE,OW-DROP,EX-NORMAL,EX-ABORT,BR-PROV,BR-RESULT,FL-CAPACITY,FL-ALLOC,AB-SEAL,AB-GENERIC,NT-FIXED,NT-P2",
}

EXACT_REPAIR_FRAGMENTS = {
    "HEAP-PEEK-01": {
        "ownership_exit_drop_argument": (
            "PeekMut::pop consumes either guard state",
            "moves the maximum out as the sole returned owner",
            "performs ordinary heap pop without an extra guard pre-sift",
        ),
        "negative_canaries": (
            "actual mutation, and pop after mutable exposure",
            "open hole on normal exit",
            "finalizer-dependent repair",
        ),
    },
    "VIEW-CLONE-01": {
        "ownership_exit_drop_argument": (
            "Clone::clone_from may reuse destination resources",
            "neither returns the old T nor guarantees whole-value destruction",
            "every surviving destination borrow leaf keeps its root",
            "every overwritten leaf ends exactly once",
            "every newly installed leaf follows the declared clone_from behavior-effect relation",
            "Reused destination allocation or storage grants no provenance",
        ),
        "negative_canaries": (
            "trap/fail the invoked behavior at every clone_from call index",
            "reject any derivation that requires the old whole T to be returned or dropped",
            "forbids destination resource reuse",
            "shared-reference destinations whose clone_from retargets one leaf",
            "reject preservation of the overwritten root or provenance minted from destination storage",
        ),
        "evidence_refs": (
            "pinned Rust 1.97.0 library/core/src/slice/mod.rs:5602-5613",
        ),
    },
    "VIEW-FILL-01": {
        "ownership_exit_drop_argument": (
            "For fill with n=0, Clone is never called and the seed is destroyed once",
            "clone_from may reuse destination resources and does not imply whole-value destruction",
            "For fill_with with n=0, the producer is called zero times and destroyed once",
            "each producer call completes before its result replaces one slot",
            "every newly installed leaf follows the declared clone_from behavior-effect relation",
            "Every T returned by fill_with follows the callable result-provenance relation",
            "destination storage reuse grants no provenance",
        ),
        "negative_canaries": (
            "Exercise n=0 and n=1 for fill and fill_with",
            "destination-reusing clone_from",
            "retargets a shared-reference leaf",
            "fill_with producer that selects two independent roots",
            "reject mandatory whole-value drop",
        ),
        "evidence_refs": (
            "library/core/src/slice/specialize.rs:8-17",
            "library/core/src/slice/mod.rs:4167-4197",
        ),
    },
    "HEAP-DRAIN-01": {
        "fact_channels_and_invalidators": (
            "Each yielded T becomes a sole owner and mints no payload borrow",
            "Every pre-drain element, slot-live, provenance, and heap-order fact expires",
        ),
        "negative_canaries": (
            "construct drain, and reject every old owner/version payload read or fact reuse",
            "repeat for clear",
            "same-address or same-slot reuse",
        ),
    },
    "RAW-SAFE-LEAK-01": {
        "ordinary_library_derivation_sketch": (
            "inadmissible as an ordinary exact-disposition route",
            "destroying neither payload nor allocation",
        ),
        "ownership_exit_drop_argument": (
            "exact reviewed exception to the ordinary OW-* to EX-NORMAL implication",
            "deferred resource-abandonment contract",
        ),
        "negative_canaries": (
            "reject any claim that the payload or allocation was dropped",
        ),
    },
    "RC-UNWRAP-01": {
        "ownership_exit_drop_argument": (
            "FL-ATOMIC applies specifically to try_unwrap",
            "into_inner(None) and unwrap_or_clone do not inherit rollback",
            "nonunique Err returns the exact original Rc/allocation with strong count unchanged",
            "Every fallback Clone result leaf follows the declared Clone result-provenance relation",
            "may select, swap, or coalesce independently valid roots",
            "never derives from the temporary Rc receiver, old allocation storage, or call frame",
        ),
        "negative_canaries": (
            "For try_unwrap only",
            "into_inner(None) decrements exactly one strong count",
            "unwrap_or_clone fallback consumes one Rc without rollback",
            "custom Clone that swaps or coalesces two independent shared roots",
            "On the unique path require zero Clone calls",
        ),
    },
    "RC-UNIQUE-01": {
        "ownership_exit_drop_argument": (
            "With strong=1 and weak=0 it performs no allocation, Clone, relocation, or payload drop and retains allocation identity",
            "With strong>1 it allocates and CloneToUninit creates the new payload before the handle is replaced",
            "With strong=1 and weak>0 it allocates, relocates the payload exactly once without Clone or payload drop",
            "all old Weak upgrades become None",
            OP9_OOM_CLAUSE,
            "Under xlang [EFF-4], an invoked-behavior trap aborts immediately",
            "Neither edge is recoverable, so FL-ATOMIC is not implied",
        ),
        "fact_channels_and_invalidators": (
            "strong-shared clone branch and weak-only relocation branch mint a new allocation relation",
            "Every mutable result is a bounded reborrow of the exact unique parent and current post-branch allocation",
        ),
        "negative_canaries": (
            "For strong=1,weak=0 require zero allocation, zero Clone, zero relocation, and retained identity",
            "For strong>1 require one allocation and one CloneToUninit",
            "For strong=1,weak>0 require one allocation, zero Clone, one ownership-preserving relocation",
            "Reject FL-ATOMIC",
        ),
        "evidence_refs": (
            "library/alloc/src/rc.rs:1935-1945,2090-2132",
            "spec/kernel-spec-v0.6.md [OP-9]",
        ),
    },
    "REFCELL-OWNER-01": {
        "ownership_exit_drop_argument": (
            "new moves T into one live cell payload whose ordinary owner destruction drops T exactly once",
            "into_inner consumes the cell and moves T out",
            "get_mut returns a bounded mutable child under static outer uniqueness",
            "does not consult or reset the dynamic borrow counter",
        ),
        "negative_canaries": (
            "Destroy a cell made by new and drop its T exactly once",
            "consume through into_inner and reject any drop of the moved-out slot",
            "call get_mut under static uniqueness without a dynamic borrow-state transition",
        ),
        "evidence_refs": (
            "library/core/src/cell.rs:971-1000,1330-1333",
        ),
    },
    "REF-GUARD-01": {
        "ownership_exit_drop_argument": (
            "RefMut::filter_map may mutate the referent before returning None",
            "no referent rollback or general failure atomicity is promised",
            "Only RefMut::map_split requires disjoint unique callback results",
            "Ref::map_split allows overlapping shared results",
            "The split input guard is consumed and never revives",
            "may derive from captured external storage",
        ),
        "fact_channels_and_invalidators": (
            "each referent leaf retains the actual callback-returned storage provenance",
            "RefMut::map_split outputs have pairwise-disjoint unique footprints",
            "Ref::map_split outputs may overlap",
            "rather than pointer inequality",
        ),
        "negative_canaries": (
            "mutate the referent and then return None",
            "Reject referent rollback and any FL-ATOMIC inference",
            "Accept overlapping Ref::map_split shared outputs",
            "Accept disjoint empty or ZST RefMut outputs even when addresses compare equal",
            "Reject revival of the consumed parent guard",
            "Preserve captured external referent provenance separately from original-cell guard bookkeeping",
        ),
    },
    "HELPER-ARRAY-INTOITER-01": {
        "ownership_exit_drop_argument": (
            "exactly data[front..back] is live",
            "data[..front] and data[back..] are dead or moved",
            "Front/back progress removes one endpoint from the live interval before moving it out",
            "Normal abandonment destroys exactly the live interval and no exterior slot",
        ),
        "fact_channels_and_invalidators": (
            "sealed owner/version/front/back interval is the sole authority for T reads, views, moves, and drops",
            "next, next_back, abandonment, and destruction invalidate the previous interval version",
        ),
        "negative_canaries": (
            "exhaust every mixed next/next_back crossing and yield each index at most once",
            "drop exactly the live interval once and the dead exterior zero times",
            "Reject forged, noncanonical, out-of-bounds, or stale interval state",
            "hidden Clone/allocation/per-slot tags",
        ),
        "evidence_refs": (
            "library/core/src/array/iter.rs:54-82,211-225,234-329",
            "library/core/src/array/iter/iter_inner.rs:34-82,126-143,159-176,224-256",
        ),
    },
    "VIEW-SORT-02": {
        "negative_canaries": ("close every transient hole before every normal return",),
    },
    "VIEW-SELECT-01": {
        "negative_canaries": ("close every transient hole before every normal return",),
    },
    "DEQUE-RANGE-01": {
        "ownership_exit_drop_argument": (
            "Invalid bounds trap before cursor creation and mint no result borrow",
            "First or repeated terminal None retains the deque borrow",
            "Cursor destruction, consuming close, or proven last use releases the borrow",
            "exact nonoverlapping place",
        ),
        "negative_canaries": (
            "at each physical wrap boundary",
            "skipped, repeated, or overlapping unique yields",
            "any result borrow from invalid bounds",
            "terminal None alone grants no generic cleanup",
        ),
    },
    "OMAP-END-01": {
        "ownership_exit_drop_argument": (
            "Empty returns None and mints no borrow or guard",
            "no finalizer or writer-called finish operation is required",
        ),
        "negative_canaries": (
            "after every nonconsuming guard reborrow",
            "finalizer-dependent repair",
        ),
    },
    "MAP-ENTRY-01": {
        "fact_channels_and_invalidators": (
            "occupied Entry::key borrows the stored map key",
            "vacant Entry::key borrows the candidate key owned by that guard",
            "insertion-result value borrows and occupied guards are tied to the map owner/state version",
        ),
        "negative_canaries": (
            "occupied key borrow not derived from the stored map key",
            "vacant key borrow not derived from the guard-owned candidate",
            "candidate-borrow or and_modify-reborrow survival",
        ),
        "asymptotic_argument": (
            "B-tree entry lookup O(log n)",
            "hash entry lookup expected O(1)",
        ),
    },
    "MAP-OCCUPIED-01": {
        "fact_channels_and_invalidators": (
            "key/get/get_mut is tied to map storage through each bounded guard-reborrow region",
            "into_mut consumes the guard and preserves a map-owner/state-version result borrow",
            "Replacement and removal branches mint no result borrow",
        ),
        "asymptotic_argument": (
            "key/get/get_mut/into_mut/replace O(1)",
            "B-tree removal adds O(log n) repair",
        ),
    },
    "MAP-VACANT-01": {
        "fact_channels_and_invalidators": (
            "key derives from the candidate key owned by the vacant guard",
            "returned V borrow or occupied guard derives from the map owner/state version",
            "Failure mints no result borrow",
        ),
        "asymptotic_argument": (
            "hash insert expected O(1) and no-grow after entry pre-reserve",
            "B-tree insertion may allocate/split and adds O(log n) repair",
        ),
    },
    "STRING-DECODE-STRICT-01": {
        "ownership_exit_drop_argument": (
            "same allocation into the sole String owner",
            "same allocation into the sole FromUtf8Error owner",
        ),
        "asymptotic_argument": ("zero allocation",),
        "negative_canaries": ("strict result-borrow", "duplicate/lost vector ownership"),
    },
    "STRING-DECODE-LOSSY-01": {
        "fact_channels_and_invalidators": (
            "only by the entirely valid borrowed branch",
            "replacement/owned and failure branches mint no result borrow",
        ),
        "negative_canaries": (
            "borrowed Cow result for invalid or replaced bytes",
            "wrong-input result-borrow provenance",
        ),
        "asymptotic_argument": ("valid branch performs zero allocation",),
    },
    "STRING-DECODE-ERROR-01": {
        "fact_channels_and_invalidators": (
            "tied to the FromUtf8Error owner and exact held-vector range",
            "utf8_error and into_bytes mint no result borrow",
        ),
        "negative_canaries": (
            "original consumed caller binding",
            "byte rescan/allocation/copy",
        ),
        "asymptotic_argument": ("O(1), zero allocation, and no byte rescan",),
    },
    "HMAP-META-01": {
        "ownership_exit_drop_argument": (
            "`with_hasher` consumes the caller's builder binding",
            "`with_capacity` has no offered builder",
            "Eventual destruction of a nonempty owner",
            "not recoverable failure",
        ),
        "negative_canaries": (
            "Verify zero allocator calls for `with_hasher`",
            "Reject any `Failure(...)` constructor branch",
        ),
        "evidence_refs": ("spec/kernel-spec-v0.6.md [OP-9]",),
    },
    "HSET-META-01": {
        "ownership_exit_drop_argument": (
            "`with_hasher` consumes the caller's builder binding",
            "`with_capacity` has no offered builder",
            "Eventual destruction of a nonempty owner",
            "not recoverable failure",
        ),
        "negative_canaries": (
            "Verify zero allocator calls for `with_hasher`",
            "Reject any `Failure(...)` constructor branch",
        ),
        "evidence_refs": ("spec/kernel-spec-v0.6.md [OP-9]",),
    },
}

EXACT_CENSUS_FRAGMENTS = {
    "VIEW-FILL-01": {
        "invalidation": (
            "clone_from to reuse resources",
            "empty fill drops the seed once",
            "empty fill_with invokes the producer zero times",
        ),
        "failure_drop_abandonment": ("promises no rollback", "every destination remains live and valid"),
        "source_refs": ("library/core/src/slice/specialize.rs:8-17",),
    },
    "REF-GUARD-01": {
        "failure_drop_abandonment": (
            "RefMut mutations made before None persist",
            "no referent rollback is promised",
        ),
        "required_obligations": (
            "separate bookkeeping-versus-referent provenance",
            "member-scoped RefMut split disjointness",
            "overlapping Ref split",
            "persistent RefMut side effects on filter failure",
        ),
        "source_refs": (
            "library/core/src/cell.rs:1703-1711,1754-1785,1899-1912,1967-2007,2045-2088",
        ),
    },
    "RC-UNIQUE-01": {
        "post_state_result": (
            "identity-preserving access",
            "shared-strong cloning",
            "weak-only ownership-preserving relocation",
        ),
        "failure_drop_abandonment": (
            "OP-9 TCB-level divergent OOM policy",
            "CloneToUninit trap aborts under EFF-4",
            "Neither edge is recoverable",
        ),
        "required_obligations": (
            "branch-exact allocation/clone/relocation",
            "weak disassociation",
            "no recoverable-failure atomicity is claimed",
        ),
        "source_refs": ("library/alloc/src/rc.rs:1935-1945,2090-2132",),
    },
    "REFCELL-OWNER-01": {
        "failure_drop_abandonment": (
            "new cell owner destruction drops live T exactly once",
            "into_inner moves T out and leaves no payload drop",
        ),
        "required_obligations": (
            "exact payload drop",
            "branch-specific live-versus-moved payload destruction",
        ),
        "source_refs": ("library/core/src/cell.rs:971-1000,1330-1333",),
    },
    "HELPER-ARRAY-INTOITER-01": {
        "post_state_result": ("live interval [front,back)",),
        "invalidation": (
            "each endpoint move changes the sealed interval version",
            "returned view is tied to the iterator and blocks progress",
        ),
        "failure_drop_abandonment": (
            "drops exactly data[front..back] once and no dead exterior slot",
        ),
        "required_obligations": (
            "Sealed interval-state integrity",
            "mixed-end crossing",
            "no per-slot occupancy representation",
        ),
        "source_refs": (
            "library/core/src/array/iter.rs:54-82,211-225,234-329",
            "library/core/src/array/iter/iter_inner.rs:34-82,126-143,159-176,224-256",
        ),
    },
    "HMAP-META-01": {
        "failure_drop_abandonment": (
            "expose no recoverable failure result",
            "OP-9 TCB-level divergent policy edge",
        ),
        "required_obligations": ("Do not invent a recoverable allocator result",),
    },
    "HSET-META-01": {
        "failure_drop_abandonment": (
            "expose no recoverable failure result",
            "OP-9 TCB-level divergent policy edge",
        ),
        "required_obligations": ("Do not invent a recoverable allocator result",),
    },
    "RC-UNWRAP-01": {
        "failure_drop_abandonment": (
            "try_unwrap Err returns the exact Rc with strong count unchanged",
            "into_inner None consumes that Rc",
            "unwrap_or_clone nonunique consumes its Rc",
        ),
        "source_refs": ("library/alloc/src/rc.rs:1054-1070,1105-1107,2161-2163",),
    },
    "HEAP-DRAIN-01": {
        "invalidation": (
            "valid empty heap allocation",
            "First or repeated terminal None leaves drain cursor/source authority live",
        ),
        "required_obligations": (
            "invalidate every prior element, slot-live, provenance, and heap-order fact",
        ),
    },
}

GLOBAL_CURSOR_LIFECYCLE_ROWS = {
    "VIEW-ITER-01",
    "VIEW-WINDOW-01",
    "VIEW-CHUNKS-01",
    "VIEW-CHUNKBY-01",
    "VIEW-SPLIT-PRED-01",
    "BYTE-ASCII-05",
    "BYTE-UTF8-CHUNKS-01",
    "TEXT-ITER-01",
    "TEXT-UTF16-01",
    "TEXT-MATCH-ITER-01",
    "TEXT-SPLIT-PATTERN-01",
    "TEXT-LINES-01",
    "TEXT-ESCAPE-01",
    "DEQUE-RANGE-01",
    "OMAP-RANGE-01",
    "SET-ALG-01",
    "SET-ALG-02",
    "DEQUE-ITER-01",
    "LIST-ITER-01",
    "HEAP-VIEW-01",
    "OMAP-ITER-01",
    "OSET-RANGE-01",
    "HMAP-ITER-01",
    "HSET-ITER-01",
    "TRAIT-ITER-01",
    "TRAIT-DOUBLE-01",
    "TRAIT-INTOITER-01",
    "ITER-SOURCE-VALUE-01",
    "RANGE-LEGACY-HALFOPEN-STATE-01",
    "RANGE-LEGACY-INCLUSIVE-STATE-01",
    "SEQ-DRAIN-01",
    "SEQ-EXTRACT-01",
    "SEQ-SPLICE-01",
    "DEQUE-DRAIN-01",
    "STRING-DRAIN-01",
    "HEAP-DRAIN-01",
    "HMAP-DRAIN-01",
    "HSET-DRAIN-01",
    "LIST-EXTRACT-01",
    "OMAP-FILTER-01",
    "OSET-FILTER-01",
    "HMAP-FILTER-01",
    "HSET-FILTER-01",
}

LAST_USE_SPLIT_ROWS = {
    "VIEW-ITER-01",
    "VIEW-WINDOW-01",
    "VIEW-CHUNKS-01",
    "VIEW-CHUNKBY-01",
    "VIEW-SPLIT-PRED-01",
    "BYTE-ASCII-05",
    "BYTE-UTF8-CHUNKS-01",
    "TEXT-ITER-01",
    "TEXT-UTF16-01",
    "TEXT-MATCH-ITER-01",
    "TEXT-SPLIT-PATTERN-01",
    "TEXT-LINES-01",
    "TEXT-ESCAPE-01",
    "DEQUE-RANGE-01",
    "OMAP-RANGE-01",
    "SET-ALG-01",
    "SET-ALG-02",
}

CENTRAL_ALLOCATION_ITER_ROWS = {
    "DEQUE-ITER-01",
    "HEAP-VIEW-01",
    "HMAP-ITER-01",
    "HSET-ITER-01",
}

TOPOLOGY_ITER_ROWS = {"LIST-ITER-01", "OMAP-ITER-01", "OSET-RANGE-01"}

OWNED_FILTER_CLOSE_ROWS = {
    "LIST-EXTRACT-01",
    "OMAP-FILTER-01",
    "OSET-FILTER-01",
    "HMAP-FILTER-01",
    "HSET-FILTER-01",
}

MIXED_ITER_CLOSE_ROWS = CENTRAL_ALLOCATION_ITER_ROWS | TOPOLOGY_ITER_ROWS | {
    "TRAIT-INTOITER-01"
}

LIFECYCLE_REQUIRED_FRAGMENTS = {
    "RANGE-VALUE-INCLUSIVE-01": {
        "ordinary_library_derivation_sketch": (
            "It is not a cursor",
            "public two-field descriptor",
        ),
        "ownership_exit_drop_argument": (
            "Calling iter clones the descriptor into an independent iterator",
            "does not mutate or release the original descriptor",
        ),
        "negative_canaries": (
            "Reject any hidden exhausted bit",
            "BR-CURSOR capability",
        ),
    },
    "ITER-SOURCE-VALUE-01": {
        "ownership_exit_drop_argument": (
            "After the sole yield, the cursor retains no T",
            "those leaves move with T and remain independently live",
        ),
        "negative_canaries": (
            "require one T destruction only in the before-yield case",
        ),
    },
    "ITER-SOURCE-CALLBACK-01": {
        "ownership_exit_drop_argument": (
            "OnceWith consumes F on the first next",
            "RepeatWith has no normal None",
            "FromFn invokes F on every next",
            "Successors performs no F call for an absent initial seed",
        ),
        "negative_canaries": (
            "FromFn None-then-Some resurrection",
            "first or repeated None must not destroy any retained F",
        ),
    },
    "ITER-SOURCE-REPEAT-01": {
        "ownership_exit_drop_argument": (
            "repeat_n(seed, 0) destroys seed during construction",
            "first n - 1 yields clone the retained seed",
            "final yield moves it and leaves none retained",
            "Early destruction drops only a still-retained seed once",
        ),
        "negative_canaries": (
            "Repeat unused and partially consumed destruction",
            "RepeatN at n = 0 construction",
            "exactly n - 1 clones",
            "exactly one drop only when a seed actually remains",
        ),
        "evidence_refs": (
            "repeat.rs:60-84",
            "repeat_n.rs:59-73,82-90,114-130",
        ),
    },
    "ITER-ADAPT-CHAIN-01": {
        "ownership_exit_drop_argument": (
            "permanently retires and destroys A",
            "non-fused B may later return Some",
            "Reverse traversal applies the symmetric rule",
        ),
    },
    "ITER-ADAPT-ZIP-01": {
        "ownership_exit_drop_argument": (
            "polls A before B",
            "x is destroyed in that call and is not cached",
            "Either non-fused side may later resume",
        ),
    },
    "ITER-ADAPT-NEST-01": {
        "ownership_exit_drop_argument": (
            "wrap the outer cursor in Fuse",
            "active inner cursor is permanently retired and destroyed at its first None",
            "general Fuse branch may destroy a non-fused outer",
            "FusedIterator-specialized outer may remain stored",
        ),
    },
    "ITER-ADAPT-STATE-01": {
        "ownership_exit_drop_argument": (
            "Scan has no done bit",
            "callback-produced None consumes that input but retains State, F, and upstream",
            "transient upstream None likewise retains all three owners",
        ),
    },
    "ITER-ADAPT-PEEK-01": {
        "ownership_exit_drop_argument": (
            "Direct next forwards an upstream None without caching it",
            "Peek may cache None",
            "later next or peek may repoll a non-fused upstream and return Some",
        ),
    },
    "ITER-ADAPT-FUSE-01": {
        "ownership_exit_drop_argument": (
            "general implementation retires and destroys upstream state",
            "FusedIterator specialization retains upstream state",
            "does not destroy the Fuse adapter",
        ),
    },
    "ITER-ADAPT-CYCLE-01": {
        "ownership_exit_drop_argument": (
            "Construction consumes current I, calls I::clone exactly once",
            "stores orig = clone(I) and iter = I",
            "Each next first polls current iter exactly once",
            "clones orig exactly once, replaces and destroys the old current epoch exactly once",
            "polls the new clone exactly once",
            "empty template repeats this clone/replace/poll sequence on every next",
            "Final cursor destruction disposes the distinct retained orig template and current epoch exactly once each",
        ),
        "structural_costs_and_pathology": (
            "first epoch follows the consumed iter",
            "each restart follows a freshly cloned orig epoch",
            "Clone may change state, roots, contents, or order",
            "Periodicity and unbounded repetition require a separately frozen clone-equivalence",
        ),
        "negative_canaries": (
            "one construction Clone and its EFF-4 trap",
            "repeated None on an empty template",
            "non-fused current epoch returning None followed by Some from a clone",
            "state- and order-changing Clone",
            "one old-current destruction per restart",
            "final exact destruction of the distinct orig and current fields",
            "Reject universal periodicity or source-order equivalence",
        ),
        "evidence_refs": ("cycle.rs:15-23,34-40,60-80",),
    },
    "SEQ-DRAIN-01": {
        "ownership_exit_drop_argument": (
            "Construction shortens len to the range start",
            "First and repeated terminal None destroy nothing",
            "overlap-safely moves the untouched tail",
        ),
    },
    "SEQ-EXTRACT-01": {
        "ownership_exit_drop_argument": (
            "Construction sets len to zero",
            "First and repeated terminal None do not copy the untouched tail",
            "sets final len, and then destroys retained F exactly once",
        ),
    },
    "SEQ-SPLICE-01": {
        "ownership_exit_drop_argument": (
            "zero replacement-iterator calls",
            "drains every remaining removed T",
            "restores len through nested Drain cleanup",
        ),
    },
    "DEQUE-DRAIN-01": {
        "ownership_exit_drop_argument": (
            "First and repeated terminal None destroy no remaining T",
            "relocates the shorter retained side as required",
            "fixes head and len",
        ),
    },
    "STRING-DRAIN-01": {
        "ownership_exit_drop_argument": (
            "every next and first or repeated terminal None likewise leaves all bytes unchanged",
            "removes the entire original byte range regardless of yield progress",
        ),
    },
    "HEAP-DRAIN-01": {
        "ownership_exit_drop_argument": (
            "valid empty allocation",
            "no structural tail repair is pending",
            "First and repeated terminal None nevertheless retain the cursor and source borrow",
        ),
    },
    "HMAP-DRAIN-01": {
        "ownership_exit_drop_argument": (
            "moves the RawTable allocation into the cursor",
            "First and repeated terminal None retain that allocation",
            "returns the empty allocation to the base map",
        ),
    },
    "HSET-DRAIN-01": {
        "ownership_exit_drop_argument": (
            "moves the RawTable allocation into the cursor",
            "First and repeated terminal None retain that allocation",
            "returns the empty allocation to the base set",
        ),
    },
    "LIST-EXTRACT-01": {
        "ownership_exit_drop_argument": (
            "fully unlinked, link- and len-repaired, deallocated",
            "ExtractIf has no structural Drop repair",
            "First and repeated terminal None retain the source borrow and F",
        ),
    },
    "OMAP-FILTER-01": {
        "ownership_exit_drop_argument": (
            "There is no structural Drop repair",
            "First and repeated terminal None retain source authority, F, and RangeBounds R",
            "destroys F and then R exactly once",
        ),
    },
    "OSET-FILTER-01": {
        "ownership_exit_drop_argument": (
            "There is no structural Drop repair",
            "First and repeated terminal None retain source authority, F, and RangeBounds R",
            "destroys F and then R exactly once",
        ),
    },
    "HMAP-FILTER-01": {
        "ownership_exit_drop_argument": (
            "RawTable::remove updates control bytes, item count, and growth metadata before yielding",
            "There is no structural Drop repair",
            "Stored BuildHasher S remains in the source owner, is never used, moved, or destroyed",
        ),
    },
    "HSET-FILTER-01": {
        "ownership_exit_drop_argument": (
            "RawTable::remove updates control bytes, item count, and growth metadata before yielding",
            "There is no structural Drop repair",
            "Stored BuildHasher S remains in the source owner, is never used, moved, or destroyed",
        ),
    },
}

LIFECYCLE_CENSUS_FRAGMENTS = {
    "RANGE-VALUE-INCLUSIVE-01": {
        "required_obligations": ("strict separation from cursor authority or terminal state",),
        "source_refs": ("library/core/src/range.rs:235-259", "335-356 iter cloning"),
    },
    "ITER-SOURCE-CALLBACK-01": {
        "required_obligations": ("FromFn None-to-Some resurrection", "RepeatWith nontermination"),
        "source_refs": ("from_fn.rs:60-72", "successors.rs:41-67"),
    },
    "ITER-SOURCE-REPEAT-01": {
        "failure_drop_abandonment": (
            "repeat_n(seed, 0) drops seed during construction",
            "final yield moves it",
            "post-final None or destruction drops none",
        ),
        "required_obligations": (
            "RepeatN zero-count construction drop",
            "n - 1 clone count",
            "final seed move",
        ),
        "source_refs": ("repeat.rs:60-84", "repeat_n.rs:59-73,82-90,114-130"),
    },
    "ITER-ADAPT-CYCLE-01": {
        "failure_drop_abandonment": (
            "Construction calls Clone exactly once",
            "Every current-epoch None calls Clone once",
            "replaces and destroys old current once",
            "polls the new clone once",
            "distinct orig template and current epoch exactly once each",
        ),
        "layout_identity_order": (
            "First epoch follows consumed iter",
            "each restart follows its freshly cloned epoch",
            "Clone may change state/order",
            "periodicity requires a separate clone-equivalence premise",
        ),
        "source_refs": ("cycle.rs:15-23,34-40,60-80",),
    },
    "ITER-ADAPT-STATE-01": {
        "invalidation": ("Scan has no done bit",),
        "source_refs": ("scan.rs:17-25,37-48",),
    },
    "ITER-ADAPT-NEST-01": {
        "invalidation": ("Outer traversal is wrapped in Fuse",),
        "source_refs": ("flatten.rs:357-370,513-530",),
    },
    "HMAP-FILTER-01": {
        "source_refs": ("src/map.rs:182-185,896-908,955-965,2588-2611",),
    },
    "HSET-FILTER-01": {
        "source_refs": ("src/map.rs:182-185,896-908,955-965,2588-2611",),
    },
}


def read_tsv(path: Path) -> tuple[list[str], list[dict[str, str]]]:
    with path.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        if reader.fieldnames is None:
            raise ValueError(f"{path}: missing header")
        rows = list(reader)
        return reader.fieldnames, rows


def duplicate_values(values: list[str]) -> list[str]:
    counts = Counter(values)
    return sorted(value for value, count in counts.items() if count != 1)


def census_implicates_allocation_failure(row: dict[str, str]) -> bool:
    text = row.get("failure_drop_abandonment", "").lower()
    terms = (
        "allocation",
        "allocate",
        "allocator",
        "oom",
        "capacity",
        "growth",
        "reallocate",
    )
    return row.get("contract_id", "").startswith("ALLOC-") or any(
        term in text for term in terms
    ) or "reserve" in row.get("rust_surfaces", "").lower()


def census_implicates_disjointness(row: dict[str, str]) -> bool:
    text = " ".join(
        row.get(field, "")
        for field in (
            "post_state_result",
            "invalidation",
            "required_obligations",
            "implementation_privilege_evidence",
        )
    ).lower()
    contract_id = row.get("contract_id", "")
    return (
        "disjoint" in text
        or "partition" in text
        or "SWAP" in contract_id
        or contract_id == "RAW-UNSAFE-ACCESS-01"
    )


def census_implicates_abort(row: dict[str, str]) -> bool:
    text = row.get("failure_drop_abandonment", "").lower().replace("not panic", "")
    return any(marker in text for marker in ("panic", "trap", "unwind", "aborting"))


def census_implicates_abandonable_cursor(row: dict[str, str]) -> bool:
    text = " ".join(
        row.get(field, "")
        for field in (
            "failure_drop_abandonment",
            "required_obligations",
            "implementation_privilege_evidence",
            "post_state_result",
        )
    ).lower()
    return "abandon" in text and any(
        marker in text
        for marker in ("cursor", "guard", "iterator", "iteration", "yield", "traversal")
    )


def verify(root: Path) -> list[str]:
    errors: list[str] = []
    census_path = root / "RUST-DATA-CONTRACT-CENSUS.tsv"
    registry_path = root / "CAPABILITY-OBLIGATION-REGISTRY.tsv"
    matrix_path = root / "DERIVATION-MATRIX.tsv"

    census_header, census_rows = read_tsv(census_path)
    registry_header, registry_rows = read_tsv(registry_path)
    matrix_header, matrix_rows = read_tsv(matrix_path)

    if "contract_id" not in census_header:
        errors.append(f"{census_path}: missing contract_id column")
    if "capability_id" not in registry_header:
        errors.append(f"{registry_path}: missing capability_id column")
    if matrix_header != EXPECTED_HEADER:
        errors.append(
            f"{matrix_path}: header mismatch: expected {EXPECTED_HEADER!r}, "
            f"found {matrix_header!r}"
        )

    census_ids = [row.get("contract_id", "").strip() for row in census_rows]
    registry_ids = [row.get("capability_id", "").strip() for row in registry_rows]
    matrix_ids = [row.get("contract_id", "").strip() for row in matrix_rows]

    for label, values in (
        ("census contract", census_ids),
        ("registry capability", registry_ids),
        ("matrix contract", matrix_ids),
    ):
        duplicates = duplicate_values(values)
        if duplicates:
            errors.append(f"duplicate or empty {label} IDs: {duplicates}")

    census_set = set(census_ids)
    matrix_set = set(matrix_ids)
    if census_set != matrix_set:
        missing = sorted(census_set - matrix_set)
        extra = sorted(matrix_set - census_set)
        if missing:
            errors.append(f"matrix is missing contracts: {missing}")
        if extra:
            errors.append(f"matrix has unknown contracts: {extra}")

    if matrix_ids != census_ids:
        errors.append("matrix contract order differs from the canonical census order")

    registry_set = set(registry_ids)
    registry_rank = {capability_id: index for index, capability_id in enumerate(registry_ids)}
    registry_status = {
        row.get("capability_id", "").strip(): row.get("current_xlang_status", "").strip()
        for row in registry_rows
    }
    registry_by_id = {
        row.get("capability_id", "").strip(): row for row in registry_rows
    }
    registry_fragments = {
        "OW-RELOCATE": ("drain repair", "compaction", "rehash", "without source drops"),
        "OW-CLONE": (
            "in-place clone-from",
            "may reuse destination resources",
            "without imposing a mandatory drop/reallocation tax",
        ),
        "OW-DROP": ("rejected duplicate", "displaced value", "moved-from or dead slots are never dropped"),
        "BR-PROV": (
            "Every borrowed leaf retains exactly one access-provenance root",
            "finite product or tagged sum of independent singleton leaf relations",
        ),
        "BR-REBORROW": ("unique drains and returned cursors", "cannot be used to retarget or escape"),
        "BR-RESULT": (
            "exact provenance expression over declared input and callable-capture relations",
            "Each returned borrow leaf selects exactly one allowed source",
            "product fields and sum branches retain their own tags",
        ),
        "BR-STORED": (
            "arbitrary retained value field or payload",
            "callable environments, scan state, cached Items, and collection payloads",
            "arbitrary storage or projection requires BR-STORED",
        ),
        "BR-DISJOINT": (
            "non-lending unique iteration",
            "pointer inequality alone is never a proof",
        ),
        "BR-CURSOR": (
            "finite typed field/branch/epoch map of external source authorities",
            "grants no arbitrary borrow-bearing T, State, callable-environment, cached-Item, or collection field",
            "already yielded external borrow retains its source lifetime unless receiver-bounded",
        ),
        "AB-STATEFUL": (
            "AB-STATEFUL grants no retained-borrow storage",
            "borrow-bearing callable environment or separate state additionally requires BR-PROV and BR-STORED",
        ),
    }
    for capability_id, fragments in registry_fragments.items():
        registry_text = " ".join(registry_by_id.get(capability_id, {}).values())
        for fragment in fragments:
            if fragment not in registry_text:
                errors.append(
                    f"{capability_id}: registry lost adjudicated fragment {fragment!r}"
                )
    census_by_id = {
        row.get("contract_id", "").strip(): row for row in census_rows
    }

    matrix_by_id = {
        row.get("contract_id", "").strip(): row for row in matrix_rows
    }
    behavior_ids = {
        contract_id
        for contract_id, row in matrix_by_id.items()
        if "AB-BEHAVIOR" in row.get("capability_ids", "").split(",")
    }
    if len(behavior_ids) != EXPECTED_AB_BEHAVIOR_COUNT:
        errors.append(
            "AB-BEHAVIOR carrier count differs: "
            f"{len(behavior_ids)} != {EXPECTED_AB_BEHAVIOR_COUNT}"
        )
    for contract_id in sorted(behavior_ids):
        capabilities = set(
            matrix_by_id[contract_id].get("capability_ids", "").split(",")
        )
        missing = {"FL-CALLBACK", "EX-ABORT"} - capabilities
        if missing:
            errors.append(
                f"{contract_id}: AB-BEHAVIOR lacks mandatory {sorted(missing)}"
            )
    for contract_id in sorted(RUNTIME_BEHAVIOR_REPAIR_CONTRACTS):
        row = matrix_by_id.get(contract_id, {})
        census_row = census_by_id.get(contract_id, {})
        capabilities = set(row.get("capability_ids", "").split(","))
        required = {"AB-BEHAVIOR", "FL-CALLBACK", "EX-ABORT"}
        if not required <= capabilities:
            errors.append(
                f"{contract_id}: closed behavior repair lacks {sorted(required - capabilities)}"
            )
        matrix_text = " ".join(row.values()).lower()
        census_text = " ".join(census_row.values()).lower()
        for fragment in (
            "effectful by default",
            "including a call through a shared receiver",
            "repeated calls consume the preceding post-state",
            "infer no purity, idempotence, repeatability, leaf-map preservation",
            "trap/fail the invoked behavior",
            "xlang_behavior_receiver_effects",
        ):
            if fragment not in matrix_text:
                errors.append(
                    f"{contract_id}: behavior repair lost matrix fragment {fragment!r}"
                )
        for fragment in (
            "effectful by default",
            "including through a shared receiver",
            "later calls consume the preceding post-state",
            "do not infer purity, idempotence, leaf-map preservation",
            "immediate abort, no unwind cleanup or recoverable post-state promise",
        ):
            if fragment not in census_text:
                errors.append(
                    f"{contract_id}: behavior repair lost census fragment {fragment!r}"
                )

    static_overlap = behavior_ids & STATIC_ONLY_BEHAVIOR_CONTRACTS
    if static_overlap:
        errors.append(
            "static/type-only rows acquired runtime behavior authority: "
            f"{sorted(static_overlap)}"
        )

    clone_from_only = {"VIEW-CLONE-01", "VIEW-FILL-01"}
    for contract_id in sorted(CLONE_SOURCE_EFFECT_CONTRACTS):
        row = matrix_by_id.get(contract_id, {})
        census_row = census_by_id.get(contract_id, {})
        capabilities = set(row.get("capability_ids", "").split(","))
        required = {"AB-BEHAVIOR", "FL-CALLBACK", "EX-ABORT"}
        if not required <= capabilities:
            errors.append(
                f"{contract_id}: Clone source effect lacks {sorted(required - capabilities)}"
            )
        clone_text = " ".join((*row.values(), *census_row.values())).lower()
        for fragment in (
            "clone source-effect contract",
            "a broken clone law cannot weaken ownership",
            "compiled clone source-effect, helper, and repeat_n canaries",
            "repeated-call post-source chaining",
        ):
            if fragment not in clone_text:
                errors.append(
                    f"{contract_id}: Clone source-effect repair lost {fragment!r}"
                )
        if contract_id in clone_from_only or contract_id == "TRAIT-CLONE-01":
            for fragment in (
                "valid pre-source and pre-destination owners",
                "declared source and destination behavior-effect relations",
                "overwritten or otherwise ended leaves end once",
                "destination storage reuse grants no root",
            ):
                if fragment not in clone_text:
                    errors.append(
                        f"{contract_id}: clone_from source-effect repair lost {fragment!r}"
                    )
        if contract_id not in clone_from_only:
            for fragment in (
                "one valid pre-source owner to a valid post-source owner plus one valid result",
                "joint declared source-effect and result-provenance relations",
                "repeated calls use the preceding post-source state",
            ):
                if fragment not in clone_text:
                    errors.append(
                        f"{contract_id}: fresh Clone source-effect repair lost {fragment!r}"
                    )

    range_effect_contracts = (
        RANGE_BOUNDS_DESCRIPTOR_CONTRACTS
        | ACTIVE_RANGE_BOUNDS_STATE_CONTRACTS
        | BASE_RANGE_BOUNDS_BEHAVIOR_CONTRACTS
    )
    for contract_id in sorted(range_effect_contracts):
        row = matrix_by_id.get(contract_id, {})
        census_row = census_by_id.get(contract_id, {})
        combined = " ".join((*row.values(), *census_row.values())).lower()
        for fragment in (
            "rangebounds call is a runtime shared-receiver effect",
            "same nonconsumed outer descriptor owner remains valid",
            "unique transfer ends in the descriptor before destination liveness",
            "later call consumes the preceding post-descriptor state",
            "endpoint result region is receiver-bounded",
            "physical root independently remains",
            "broken rangebounds laws mint no bounds",
        ):
            if fragment not in combined:
                errors.append(
                    f"{contract_id}: RangeBounds effect repair lost {fragment!r}"
                )
    for contract_id in sorted(RANGE_BOUNDS_DESCRIPTOR_CONTRACTS):
        capabilities = set(
            matrix_by_id.get(contract_id, {}).get("capability_ids", "").split(",")
        )
        if not {"AB-BEHAVIOR", "FL-CALLBACK", "EX-ABORT"} <= capabilities:
            errors.append(f"{contract_id}: by-value RangeBounds behavior route is incomplete")
        if "BR-STORED" in capabilities:
            errors.append(f"{contract_id}: call-scoped RangeBounds descriptor became active storage")
    for contract_id in sorted(ACTIVE_RANGE_BOUNDS_STATE_CONTRACTS):
        capabilities = set(
            matrix_by_id.get(contract_id, {}).get("capability_ids", "").split(",")
        )
        if "BR-STORED" not in capabilities:
            errors.append(f"{contract_id}: retained RangeBounds state lost BR-STORED")
    for contract_id in sorted(BASE_RANGE_BOUNDS_BEHAVIOR_CONTRACTS):
        capabilities = set(
            matrix_by_id.get(contract_id, {}).get("capability_ids", "").split(",")
        )
        if "BR-STORED" in capabilities:
            errors.append(f"{contract_id}: base-only RangeBounds behavior acquired BR-STORED")

    for contract_id in sorted(RANGE_QUERY_EFFECT_CONTRACTS):
        invalidation = census_by_id.get(contract_id, {}).get("invalidation", "").lower()
        if "no ownership change" in invalidation:
            errors.append(f"{contract_id}: stale whole-owner RangeBounds claim remains")
        if not all(
            fragment in invalidation
            for fragment in (
                "no operand or descriptor owner shell is consumed",
                "internal leaf maps only under declared relations",
            )
        ):
            errors.append(f"{contract_id}: range-query effect qualifier is incomplete")

    for contract_id in ("TRAIT-DEREF-01", "TRAIT-BORROW-01"):
        row = matrix_by_id.get(contract_id, {})
        census_row = census_by_id.get(contract_id, {})
        combined = " ".join((*row.values(), *census_row.values())).lower()
        for fragment in (
            "result region is receiver-bounded",
            "physical root independently follows the declared relation",
            "actual receiver-field storage or a pre-existing external/static root",
            "bounded child reborrow or declared unique transfer",
            "receiver reuse remains excluded until the child ends",
            "receiver value, its address, nor the call frame is a physical root",
        ):
            if fragment not in combined:
                errors.append(
                    f"{contract_id}: receiver-region/physical-root split lost {fragment!r}"
                )
        capabilities = set(row.get("capability_ids", "").split(","))
        if "BR-STORED" in capabilities:
            errors.append(f"{contract_id}: projection result was misclassified as stored state")

    stale_claims = {
        "RANGE-BOUNDS-PROTOCOL-01": "receiver remains unchanged",
        "SET-REL-01": "no mutation",
        "SEQ-POP-01": "empty/rejected predicate leaves sequence unchanged",
        "DEQUE-POP-01": "empty/rejected predicate leaves deque unchanged",
        "HMAP-RESERVE-01": "capacity/table changes while entries preserved",
        "HSET-RESERVE-01": "capacity changes; values preserved",
        "TEXT-REPLACE-01": "inputs unchanged",
        "VIEW-ALLOC-01": "borrow source unchanged",
        "SEQ-EXTEND-COPY-01": "source values remain",
        "TRAIT-CONVERT-01": "leaves the borrowed source unchanged",
    }
    for contract_id, stale in stale_claims.items():
        combined = " ".join(
            (*matrix_by_id.get(contract_id, {}).values(), *census_by_id.get(contract_id, {}).values())
        ).lower()
        if stale in combined:
            errors.append(f"{contract_id}: stale behavior-effect claim remains: {stale!r}")
    positive_effect_qualifiers = {
        "SET-REL-01": "no set topology or payload-membership mutation is required",
        "SEQ-POP-01": "predicate and examined endpoint may retain declared behavior-authorized internal effects",
        "DEQUE-POP-01": "predicate and examined endpoint may retain declared behavior-authorized internal effects",
        "HMAP-RESERVE-01": "logical entry ownership and membership are preserved; stored behavior-state leaf maps may evolve",
        "HSET-RESERVE-01": "logical value ownership and membership are preserved; stored behavior-state leaf maps may evolve",
        "TEXT-REPLACE-01": "shared behavior receiver need not preserve its internal leaf map",
        "TRAIT-CMP-01": "shared access implies neither purity nor an unchanged operand leaf map",
        "VIEW-ALLOC-01": "borrow source outer owner remains valid for clone while its internal leaf map may evolve",
        "SEQ-EXTEND-COPY-01": "source outer owners remain valid while clone may evolve their internal leaf maps",
        "TRAIT-CONVERT-01": "borrowed source outer owner valid while clone may evolve its internal leaf map",
    }
    for contract_id, fragment in positive_effect_qualifiers.items():
        combined = " ".join(
            (*matrix_by_id.get(contract_id, {}).values(), *census_by_id.get(contract_id, {}).values())
        ).lower()
        if fragment not in combined:
            errors.append(
                f"{contract_id}: behavior-effect qualifier lost {fragment!r}"
            )
    for line_number, row in enumerate(matrix_rows, start=2):
        contract_id = row.get("contract_id", "").strip() or f"line {line_number}"

        for field in EXPECTED_HEADER:
            value = row.get(field)
            if value is None or not value.strip():
                errors.append(f"{contract_id}: empty field {field}")
                continue
            if "\n" in value or "\r" in value:
                errors.append(f"{contract_id}: embedded newline in {field}")
            if value.strip().lower() in FORBIDDEN_PLACEHOLDERS:
                errors.append(f"{contract_id}: placeholder value in {field}: {value!r}")

        status = row.get("status_code", "").strip()
        if status not in VALID_STATUSES:
            errors.append(f"{contract_id}: invalid status_code {status!r}")
        elif status in {"E", "P"}:
            errors.append(
                f"{contract_id}: G0 coverage clusters are non-importable and cannot receive {status}"
            )

        raw_capabilities = row.get("capability_ids", "")
        capability_ids = [part.strip() for part in raw_capabilities.split(",")]
        if any(not capability_id for capability_id in capability_ids):
            errors.append(f"{contract_id}: malformed comma-separated capability_ids")
        if any("*" in capability_id for capability_id in capability_ids):
            errors.append(f"{contract_id}: capability wildcard was not expanded")
        capability_duplicates = duplicate_values(capability_ids)
        if capability_duplicates:
            errors.append(
                f"{contract_id}: duplicate capability IDs: {capability_duplicates}"
            )
        unknown_capabilities = sorted(set(capability_ids) - registry_set)
        if unknown_capabilities:
            errors.append(
                f"{contract_id}: unknown capability IDs: {unknown_capabilities}"
            )

        known_capabilities = [
            capability_id
            for capability_id in capability_ids
            if capability_id in registry_rank
        ]
        if known_capabilities != sorted(known_capabilities, key=registry_rank.get):
            errors.append(f"{contract_id}: capability IDs are not in registry order")

        capabilities = set(capability_ids)
        census_row = census_by_id.get(contract_id, {})
        canaries = row.get("negative_canaries", "")
        facts = row.get("fact_channels_and_invalidators", "")
        ownership = row.get("ownership_exit_drop_argument", "")
        evidence = row.get("evidence_refs", "")
        sketch = row.get("ordinary_library_derivation_sketch", "")
        structural = row.get("structural_costs_and_pathology", "")
        family_lock = row.get("family_lock_or_deferral", "")

        exact_capabilities = EXACT_REPAIR_CAPABILITIES.get(contract_id)
        if exact_capabilities is not None and raw_capabilities != exact_capabilities:
            errors.append(
                f"{contract_id}: repaired capability set differs: {raw_capabilities!r}"
            )
        for field, fragments in EXACT_REPAIR_FRAGMENTS.get(contract_id, {}).items():
            value = row.get(field, "")
            for fragment in fragments:
                if fragment.lower() not in value.lower():
                    errors.append(
                        f"{contract_id}: {field} lost exact repair fragment {fragment!r}"
                    )
        for field, fragments in EXACT_CENSUS_FRAGMENTS.get(contract_id, {}).items():
            value = census_row.get(field, "")
            for fragment in fragments:
                if fragment.lower() not in value.lower():
                    errors.append(
                        f"{contract_id}: census {field} lost source-adjudicated fragment {fragment!r}"
                    )

        for field, fragments in LIFECYCLE_REQUIRED_FRAGMENTS.get(
            contract_id, {}
        ).items():
            value = row.get(field, "")
            for fragment in fragments:
                if fragment.lower() not in value.lower():
                    errors.append(
                        f"{contract_id}: {field} lost lifecycle fragment {fragment!r}"
                    )
        for field, fragments in LIFECYCLE_CENSUS_FRAGMENTS.get(
            contract_id, {}
        ).items():
            value = census_row.get(field, "")
            for fragment in fragments:
                if fragment.lower() not in value.lower():
                    errors.append(
                        f"{contract_id}: census {field} lost lifecycle fragment {fragment!r}"
                    )

        if contract_id in GLOBAL_CURSOR_LIFECYCLE_ROWS:
            lifecycle_text = " ".join(
                (
                    ownership,
                    canaries,
                    census_row.get("invalidation", ""),
                    census_row.get("failure_drop_abandonment", ""),
                )
            ).lower()
            if not all(token in lifecycle_text for token in ("first", "repeated", "none")):
                errors.append(
                    f"{contract_id}: lifecycle omits first/repeated None states"
                )
            if not any(
                token in lifecycle_text
                for token in ("cursor destruction", "consuming close", "proven last use")
            ):
                errors.append(
                    f"{contract_id}: lifecycle omits the actual authority-ending transition"
                )
            for stale_claim in (
                "normal exhaustion and permitted abandonment release",
                "normal exhaustion and abandonment release",
                "normal exhaustion frees the storage",
                "base owner becomes usable again only after cursor exhaustion",
                "exhaustion or abandonment destroys retained r",
                "during exhaustion or early abandonment",
            ):
                if stale_claim in ownership.lower():
                    errors.append(
                        f"{contract_id}: stale terminal-None cleanup claim remains: {stale_claim!r}"
                    )

        if contract_id in LAST_USE_SPLIT_ROWS:
            lifecycle_text = " ".join(
                (
                    ownership,
                    canaries,
                    census_row.get("failure_drop_abandonment", ""),
                    census_row.get("required_obligations", ""),
                )
            ).lower()
            for fragment in (
                "proven last use may end only pure cursor/source-borrow authority",
                "only when no repair, owned-state destruction, or allocation disposition remains",
                "cursor destruction or consuming close performs any pending repair, owned-state destruction, or allocation disposition",
                "terminal none may retire or replace retained substate only under the exact concrete helper transition",
                "exact concrete subcursor or epoch retirement",
                "separate pure last-use authority release from destruction/close duties",
            ):
                if fragment not in lifecycle_text:
                    errors.append(
                        f"{contract_id}: last-use/close partition lacks {fragment!r}"
                    )
            for stale_claim in (
                "proven last use performs the exact contract-specific cleanup",
                "destruction, consuming close, or proven last use performs cleanup",
                "destruction, consuming close, or proven last use performs the exact contract-specific cleanup",
            ):
                if stale_claim in lifecycle_text:
                    errors.append(
                        f"{contract_id}: proven last use still performs cleanup: {stale_claim!r}"
                    )

        if contract_id in CENTRAL_ALLOCATION_ITER_ROWS:
            lifecycle_text = f"{ownership} {canaries} {census_row.get('failure_drop_abandonment', '')} {census_row.get('required_obligations', '')}".lower()
            for fragment in (
                "retains its allocation",
                "exact central allocation to remain cursor-owned after terminal none",
                "rejecting that rule for linkedlist and btree",
            ):
                if fragment not in lifecycle_text:
                    errors.append(
                        f"{contract_id}: central-allocation iterator disposition lacks {fragment!r}"
                    )

        if contract_id in TOPOLOGY_ITER_ROWS:
            lifecycle_text = f"{ownership} {canaries} {census_row.get('failure_drop_abandonment', '')} {census_row.get('required_obligations', '')}".lower()
            family_fragment = (
                "deallocates each yielded node incrementally"
                if contract_id == "LIST-ITER-01"
                else "family-specific node/tree traversal state"
            )
            for fragment in (
                family_fragment,
                "reject any union of these resource shapes",
                "reject union with contiguous/hash central-allocation retention",
            ):
                if fragment not in lifecycle_text:
                    errors.append(
                        f"{contract_id}: topology-specific iterator disposition lacks {fragment!r}"
                    )
            if "a consuming cursor retains its allocation" in lifecycle_text:
                errors.append(
                    f"{contract_id}: generic central-allocation retention leaked into topology iterator"
                )

        if contract_id in {"TRAIT-ITER-01", "TRAIT-DOUBLE-01"}:
            lifecycle_text = f"{ownership} {canaries} {census_row.get('failure_drop_abandonment', '')} {census_row.get('required_obligations', '')}".lower()
            for fragment in (
                "trait itself authorizes neither transition",
                "proven last use may end only pure borrow authority",
                "pending repair, owned state, or allocation disposition persists",
                "delegate pure last-use release and destruction/close disposition as separate concrete transitions",
                "exact subcursor/epoch retirement is separately delegated",
            ):
                if fragment not in lifecycle_text:
                    errors.append(
                        f"{contract_id}: trait lifecycle delegation lacks {fragment!r}"
                    )

        if contract_id in OWNED_FILTER_CLOSE_ROWS | MIXED_ITER_CLOSE_ROWS:
            lifecycle_text = f"{ownership} {census_row.get('failure_drop_abandonment', '')} {census_row.get('required_obligations', '')}".lower()
            required = (
                "proven last use cannot close this cursor"
                if contract_id in OWNED_FILTER_CLOSE_ROWS
                else "proven last use may end only the borrowed entrance's pure source authority"
            )
            for fragment in (required, "last-use release cannot discharge pending"):
                if fragment not in lifecycle_text:
                    errors.append(
                        f"{contract_id}: pending close/last-use partition lacks {fragment!r}"
                    )
            if (
                contract_id in OWNED_FILTER_CLOSE_ROWS
                and "base owner becomes usable only after cursor destruction, consuming close, or proven last use" in lifecycle_text
            ):
                errors.append(
                    f"{contract_id}: retained owned state is still discharged by proven last use"
                )

        ownership_capabilities = {
            capability_id
            for capability_id in capabilities
            if capability_id.startswith("OW-")
        }
        if (
            ownership_capabilities
            and contract_id not in OWNERSHIP_NORMAL_EXIT_BOUNDARY_EXCEPTIONS
            and "EX-NORMAL" not in capabilities
        ):
            errors.append(f"{contract_id}: ownership transition lacks EX-NORMAL")
        if (
            "EX-NORMAL" in capabilities
            and contract_id not in SEALED_STABLE_STEP_RANGE_CAPABILITIES
            and "validity on every normal exit" not in sketch.lower()
        ):
            errors.append(f"{contract_id}: EX-NORMAL lacks canonical derivation prose")

        if census_row.get("family", "") == "borrowed_iteration" and "BR-CURSOR" in capabilities:
            required = {"EX-NORMAL", "EX-ABANDON"}
            if not required <= capabilities:
                errors.append(
                    f"{contract_id}: borrowed cursor iteration lacks normal/abandonment exits"
                )

        if contract_id in RETURNED_CURSOR_CONTRACTS:
            required = {"BR-PROV", "BR-REBORROW", "BR-RESULT", "BR-CURSOR"}
            if not required <= capabilities:
                errors.append(
                    f"{contract_id}: returned source cursor lacks provenance/reborrow/result duties"
                )
            if "result-borrow provenance for the returned cursor" not in facts.lower():
                errors.append(f"{contract_id}: returned cursor provenance fact is missing")
            if "wrong-source provenance" not in canaries.lower():
                errors.append(f"{contract_id}: returned cursor wrong-source canary is missing")
            if "bounded unique reborrow" not in ownership:
                errors.append(f"{contract_id}: returned cursor parent reborrow is missing")
            if contract_id in OWNED_YIELD_RETURNED_CURSOR_CONTRACTS:
                if (
                    "each yielded item transfers ownership and mints no fresh borrow from the cursor or source"
                    not in facts.lower()
                ):
                    errors.append(
                        f"{contract_id}: owning returned cursor confuses cursor provenance with Item provenance"
                    )
                if (
                    "reject any claim that an owned yielded item freshly borrows from the cursor or source"
                    not in canaries.lower()
                ):
                    errors.append(
                        f"{contract_id}: owning returned cursor lacks the owned-Item provenance canary"
                    )
                for false_claim in (
                    "the cursor and every borrowed yield",
                    "cursor or borrowed yield",
                    "each borrowed yield derives from the returned cursor",
                ):
                    if false_claim in facts.lower():
                        errors.append(
                            f"{contract_id}: stale returned-cursor yield claim remains: {false_claim!r}"
                        )

        if contract_id in UNIQUE_RESULT_REBORROW_CONTRACTS:
            if "BR-REBORROW" not in capabilities:
                errors.append(f"{contract_id}: unique borrowed result lacks BR-REBORROW")
            if "every mutable result is a bounded reborrow of the exact unique parent" not in facts.lower():
                errors.append(f"{contract_id}: bounded result-reborrow fact is missing")
            if "reject child-borrow retargeting" not in canaries.lower():
                errors.append(f"{contract_id}: bounded result-reborrow canary is missing")
            if (
                contract_id in DIRECT_REBORROW_CENSUS_CONTRACTS
                and "bounded unique result reborrow"
                not in census_row.get("required_obligations", "").lower()
            ):
                errors.append(f"{contract_id}: census unique-result reborrow obligation is missing")

        if contract_id in UNIQUE_CURSOR_REBORROW_CONTRACTS:
            if "BR-REBORROW" not in capabilities:
                errors.append(f"{contract_id}: unique cursor lacks BR-REBORROW")
            if "unique cursor derives from a bounded reborrow of the exact parent" not in facts.lower():
                errors.append(f"{contract_id}: unique cursor parent-reborrow fact is missing")
            if "reject unique-cursor retargeting" not in canaries.lower():
                errors.append(f"{contract_id}: unique cursor reborrow canary is missing")
            if "shared and consuming members do not" not in ownership.lower():
                errors.append(f"{contract_id}: cursor reborrow is not member-scoped")
            if "bounded unique cursor reborrow" not in census_row.get("required_obligations", "").lower():
                errors.append(f"{contract_id}: census unique-cursor reborrow obligation is missing")

        if contract_id in NONLENDING_UNIQUE_DISJOINT_CONTRACTS:
            if "BR-DISJOINT" not in capabilities:
                errors.append(
                    f"{contract_id}: non-lending unique Items lack BR-DISJOINT"
                )
            disjoint_fact_markers = (
                "each yielded unique borrow is pairwise disjoint from every still-live sibling",
                "protocol preserves pairwise disjoint source places",
                "non-lending unique items preserve their declared source disjointness",
            )
            if not any(marker in facts.lower() for marker in disjoint_fact_markers):
                errors.append(
                    f"{contract_id}: pairwise-live unique-Item disjointness fact is missing"
                )
            if (
                "keep one unique yield live, request later forward and reverse yields"
                not in canaries.lower()
            ):
                errors.append(
                    f"{contract_id}: pairwise-live unique-Item canary is missing"
                )
            if (
                "non-lending pairwise-live unique-item disjointness"
                not in census_row.get("required_obligations", "").lower()
            ):
                errors.append(
                    f"{contract_id}: census unique-Item disjointness obligation is missing"
                )

        if contract_id in UNIQUE_CONSUMER_RETAINED_DISJOINT_CONTRACTS:
            if "BR-DISJOINT" not in capabilities:
                errors.append(
                    f"{contract_id}: retained unique consumer state lacks BR-DISJOINT"
                )
            if (
                "each yielded unique borrow is pairwise disjoint from every still-live sibling retained by the accumulator, candidate, or destination"
                not in facts.lower()
            ):
                errors.append(
                    f"{contract_id}: consumer-retained unique sibling fact is missing"
                )
            if (
                "retain one unique yield in an accumulator, candidate, residual, or destination"
                not in canaries.lower()
                or "advance the upstream cursor" not in canaries.lower()
            ):
                errors.append(
                    f"{contract_id}: consumer-retained unique sibling canary is missing"
                )
            if (
                "non-lending pairwise-live unique-item disjointness"
                not in census_row.get("required_obligations", "").lower()
            ):
                errors.append(
                    f"{contract_id}: census retained-consumer disjointness obligation is missing"
                )

        if contract_id in STORED_ITEM_CONSUMER_CONTRACTS:
            required = {
                "BR-PROV",
                "BR-STORED",
                "BR-DISJOINT",
                "BR-CURSOR",
                "IT-SHARED",
                "IT-UNIQ",
                "IT-OWN",
            }
            required.add("BR-RESULT")
            if not required <= capabilities:
                errors.append(
                    f"{contract_id}: borrow-bearing Item storage lacks {sorted(required - capabilities)}"
                )
            if (
                "every borrow leaf stored in a destination preserves the exact source yield root"
                not in facts.lower()
            ):
                errors.append(
                    f"{contract_id}: stored Item source preservation is missing"
                )
            if (
                "store a reference-valued item in the destination"
                not in canaries.lower()
                or "fresh borrow into destination-owned storage" not in canaries.lower()
            ):
                errors.append(
                    f"{contract_id}: stored Item provenance canary is missing"
                )
            if (
                "stored-borrow preservation for arbitrary retained item fields"
                not in census_row.get("required_obligations", "").lower()
            ):
                errors.append(
                    f"{contract_id}: census stored Item obligation is missing"
                )
            if contract_id == "TRAIT-COLLECT-01" and (
                "returned collection never becomes a fresh storage source"
                not in facts.lower()
            ):
                errors.append(
                    "TRAIT-COLLECT-01: returned-owner result provenance is missing"
                )

        if contract_id in SAME_INDEX_SWAP_CONTRACTS:
            if not {"OW-SWAP", "BR-DISJOINT"} <= capabilities:
                errors.append(f"{contract_id}: indexed swap branch accounting is incomplete")
            if (
                "the equal-index member branch is a checked no-op" not in sketch.lower()
                or "apply only to the unequal-index exchange branch" not in sketch.lower()
            ):
                errors.append(f"{contract_id}: equal-versus-unequal branch split is missing")
            if (
                "mints no second unique output or borrow" not in facts.lower()
                or "only the unequal-index exchange branch requires" not in facts.lower()
            ):
                errors.append(f"{contract_id}: same-index one-place fact is missing")
            if (
                "accept an in-bounds equal-index" not in canaries.lower()
                or "first-index failure" not in canaries.lower()
                or "second-index failure" not in canaries.lower()
                or "reject duplicate/overlapping mutable outputs before any borrow escapes"
                not in canaries.lower()
            ):
                errors.append(f"{contract_id}: indexed swap branch canaries are incomplete")
            census_text = " ".join(census_row.values()).lower()
            if (
                "equal in-bounds indices" not in census_text
                or "checked no-op" not in census_text
                or "first then second" not in census_text
            ):
                errors.append(f"{contract_id}: census same-index contract is incomplete")
            for stale in ("runtime distinctness", "unconditional dynamic distinctness"):
                if stale in census_text:
                    errors.append(f"{contract_id}: stale unconditional swap claim remains: {stale}")

        if contract_id in ITERATOR_ADAPTER_RESULT_CONTRACTS:
            if not {"BR-PROV", "BR-RESULT"} <= capabilities:
                errors.append(f"{contract_id}: iterator adapter lacks result provenance")
            fact_fragment, canary_fragment = ITERATOR_ADAPTER_PROVENANCE_FRAGMENTS[contract_id]
            if fact_fragment.lower() not in facts.lower():
                errors.append(f"{contract_id}: adapter-specific provenance fact is missing")
            if canary_fragment.lower() not in canaries.lower():
                errors.append(f"{contract_id}: adapter-specific provenance canary is missing")
            if "does not expire merely because" not in facts.lower():
                errors.append(f"{contract_id}: yielded external-borrow lifetime is missing")
            if "adapter-owned storage" not in canaries.lower():
                errors.append(f"{contract_id}: adapter-storage provenance canary is missing")
            for stale_claim in (
                "result expires on adapter/cursor invalidation",
                "owned, copied, cloned, scalar, and rejected branches mint no payload borrow",
            ):
                if stale_claim in facts.lower():
                    errors.append(
                        f"{contract_id}: stale lending/provenance claim remains: {stale_claim!r}"
                    )
            if "non-lending iterator::item provenance" not in census_row.get(
                "required_obligations", ""
            ).lower():
                errors.append(f"{contract_id}: census non-lending provenance obligation is missing")
            if "iterator.rs:42-78 non-lending item signature" not in census_row.get(
                "source_refs", ""
            ).lower():
                errors.append(f"{contract_id}: census non-lending source evidence is missing")
            if "iterator.rs:42-78 non-lending Item signature" not in evidence:
                errors.append(f"{contract_id}: matrix non-lending source evidence is missing")
            adapter_invalidation = census_row.get("invalidation", "")
            if (
                "Any borrowed output" in adapter_invalidation
                and ". Any borrowed output" not in adapter_invalidation
            ):
                errors.append(
                    f"{contract_id}: appended borrowed-output census prose lacks sentence punctuation"
                )

        if contract_id in RETAINED_BORROW_STATE_CONTRACTS:
            required = {"BR-PROV", "BR-RESULT", "BR-STORED", "BR-CURSOR"}
            if not required <= capabilities:
                errors.append(
                    f"{contract_id}: retained borrow-bearing protocol state lacks {sorted(required - capabilities)}"
                )
            if (
                "every borrow leaf that remains live across a protocol move"
                not in facts.lower()
            ):
                errors.append(
                    f"{contract_id}: retained-borrow source-preservation fact is missing"
                )
            for fragment in (
                "may end old leaves and create new leaves only under its declared behavior or result-provenance relation",
                "no retained field or temporary reborrow becomes an unauthorized output source",
            ):
                if fragment not in facts.lower():
                    errors.append(
                        f"{contract_id}: retained-borrow transition rule lacks {fragment!r}"
                    )
            if "store a reference-valued retained field" not in canaries.lower():
                errors.append(
                    f"{contract_id}: retained-borrow movement/invalidation canary is missing"
                )
            for fragment in (
                "retargeting of a still-live leaf",
                "exercise a lawful transition that ends one old leaf and writes a new leaf",
                "not the transition itself",
            ):
                if fragment not in canaries.lower():
                    errors.append(
                        f"{contract_id}: retained-borrow transition canary lacks {fragment!r}"
                    )
            if "across protocol moves, calls, projection, yield, and abandonment" in facts.lower():
                errors.append(
                    f"{contract_id}: calls are still modeled as same-leaf provenance preservation"
                )
            if (
                "stored-borrow preservation for arbitrary retained t, f, state, or cached item fields"
                not in census_row.get("required_obligations", "").lower()
            ):
                errors.append(
                    f"{contract_id}: census retained-borrow obligation is missing"
                )

        if contract_id in RETAINED_CALLABLE_CURSOR_CONTRACTS:
            required = {"OW-MOVEOUT", "OW-DROP", "BR-STORED", "AB-STATEFUL"}
            if not required <= capabilities:
                errors.append(
                    f"{contract_id}: retained callable ownership lacks {sorted(required - capabilities)}"
                )
            if (
                "owned f or pattern/searcher state moves into the returned cursor"
                not in ownership.lower()
                or "destroyed exactly once on normal cursor destruction"
                not in ownership.lower()
            ):
                errors.append(
                    f"{contract_id}: retained callable move/drop algebra is incomplete"
                )
            if (
                "use a drop-counted affine callable or pattern value"
                not in canaries.lower()
                or "reject hidden clone, leak, or double drop"
                not in canaries.lower()
            ):
                errors.append(
                    f"{contract_id}: retained callable exact-destruction canary is missing"
                )
            if (
                "reject hidden copy or clone of callable, pattern, or searcher state"
                not in structural.lower()
            ):
                errors.append(
                    f"{contract_id}: retained callable structural-cost rule is missing"
                )
            if (
                "exact once destruction of owned callable, pattern, or searcher state"
                not in census_row.get("required_obligations", "").lower()
            ):
                errors.append(
                    f"{contract_id}: census retained callable ownership duty is missing"
                )

        if contract_id in CURSOR_ONLY_BORROW_STATE_CONTRACTS:
            if "BR-STORED" in capabilities:
                errors.append(
                    f"{contract_id}: opaque cursor-state authority was widened to BR-STORED"
                )
            if "stored-borrow preservation for arbitrary retained" in census_row.get(
                "required_obligations", ""
            ).lower():
                errors.append(
                    f"{contract_id}: cursor-only census row falsely imports arbitrary retained storage"
                )

        if contract_id in {"BYTE-ASCII-05", "TEXT-UTF16-01"}:
            if not {"BR-PROV", "BR-RESULT", "BR-CURSOR"} <= capabilities:
                errors.append(f"{contract_id}: source-borrowing cursor result is incomplete")
            if "result-borrow provenance for the returned cursor" not in facts.lower():
                errors.append(f"{contract_id}: source-cursor provenance fact is missing")

        if contract_id in OFFERED_OWNER_FAILURE_CONTRACTS:
            if "FL-ATOMIC" not in capabilities:
                errors.append(f"{contract_id}: offered-owner failure lacks FL-ATOMIC")
            owner_text = ownership.lower()
            for fragment in ("binding", "failure(", "sole owner", "unchanged", "no normal result"):
                if fragment not in owner_text:
                    errors.append(
                        f"{contract_id}: offered-owner failure algebra lacks {fragment!r}"
                    )
            if "dead once" not in owner_text and "dead when" not in owner_text:
                errors.append(
                    f"{contract_id}: offered-owner failure algebra does not kill the caller binding"
                )

        if contract_id in RELOCATION_REPAIR_CONTRACTS and "OW-RELOCATE" not in capabilities:
            errors.append(f"{contract_id}: relocation/compaction repair lacks OW-RELOCATE")
        if contract_id in PLAIN_NORMAL_EXIT_REPAIRS and "EX-NORMAL" not in capabilities:
            errors.append(f"{contract_id}: repaired normal-exit duty regressed")

        if contract_id in {"VIEW-WINDOW-01", "VIEW-CHUNKBY-01"}:
            if not {"EX-NORMAL", "EX-ABANDON"} <= capabilities:
                errors.append(f"{contract_id}: abandonable borrowed traversal exit is incomplete")
        if contract_id in {
            "LIST-META-01",
            "OMAP-META-01",
            "OSET-META-01",
            "HMAP-META-01",
            "HSET-META-01",
        }:
            if not {"OW-DROP", "EX-NORMAL"} <= capabilities:
                errors.append(f"{contract_id}: owning metadata row lacks eventual destruction")
            if "eventual destruction of a nonempty owner" not in ownership.lower():
                errors.append(f"{contract_id}: eventual owner-destruction argument is missing")

        if contract_id == "SEQ-SPLICE-01":
            required = {"FL-CALLBACK", "AB-BEHAVIOR", "AB-STATEFUL", "AB-GENERIC"}
            if not required <= capabilities:
                errors.append("SEQ-SPLICE-01: replacement iterator behavior duties are incomplete")
        if contract_id in {"HMAP-RESERVE-01", "HSET-RESERVE-01"}:
            required = {"FL-CALLBACK", "AB-BEHAVIOR", "AB-STATEFUL"}
            if not required <= capabilities:
                errors.append(f"{contract_id}: rehash behavior duties are incomplete")
            if "rehash invokes stored hashing behavior" not in ownership:
                errors.append(f"{contract_id}: rehash callback argument is missing")

        if contract_id in {"OSET-INSERT-01", "HSET-INSERT-01"}:
            if not {"OW-REPLACE", "OW-DROP"} <= capabilities:
                errors.append(f"{contract_id}: replace contract lacks replacement/drop duties")
        if contract_id == "OMAP-BULK-01" and not {"OW-REPLACE", "OW-DROP"} <= capabilities:
            errors.append("OMAP-BULK-01: duplicate-key replacement duties are incomplete")
        if contract_id == "OSET-BULK-01" and "OW-DROP" not in capabilities:
            errors.append("OSET-BULK-01: duplicate ownership disposition lacks OW-DROP")
        if contract_id == "VIEW-FILL-01" and "OW-DROP" not in capabilities:
            errors.append("VIEW-FILL-01: displaced destination values lack OW-DROP")
        if contract_id == "VIEW-FILL-01" and (
            "Each successful assignment destroys the displaced destination owner exactly once"
            in ownership
        ):
            errors.append("VIEW-FILL-01: unqualified whole-value destruction claim returned")

        if contract_id in {"HMAP-META-01", "HSET-META-01"}:
            if "FL-ATOMIC" in capabilities:
                errors.append(f"{contract_id}: current OP-9 constructor row must not assume FL-ATOMIC")
            if "Failure(" in ownership:
                errors.append(f"{contract_id}: invented recoverable constructor result returned")
            if OP9_OOM_CLAUSE not in ownership or OP9_REFERENCE not in evidence:
                errors.append(f"{contract_id}: current OOM path lacks exact OP-9 translation")
            if "EFF-4/OP-9" in ownership:
                errors.append(f"{contract_id}: EFF-4 capacity trap and OP-9 OOM are conflated")
            if "owner-tied borrow provenance" not in sketch.lower():
                errors.append(f"{contract_id}: hasher result lacks owner-tied provenance")
            if "result-borrow" not in canaries.lower():
                errors.append(f"{contract_id}: hasher result lacks negative provenance canary")

        if contract_id == "ITER-ADAPT-PEEK-01":
            if "BR-REBORROW" not in capabilities:
                errors.append("ITER-ADAPT-PEEK-01: peek_mut lacks BR-REBORROW")
            if "receiver-bounded outer borrows" not in facts:
                errors.append("ITER-ADAPT-PEEK-01: receiver-bounded cache layer is missing")
            if "nonborrowed generic R branch" not in canaries:
                errors.append("ITER-ADAPT-PEEK-01: generic mapped-result canary is missing")
            if (
                "next_if and next_if_eq return the accepted upstream Item with exact upstream provenance"
                not in facts
            ):
                errors.append(
                    "ITER-ADAPT-PEEK-01: next_if/next_if_eq upstream provenance is missing"
                )
            if "reject receiver/cache attribution for those Items" not in canaries:
                errors.append(
                    "ITER-ADAPT-PEEK-01: next_if/next_if_eq receiver-attribution canary is missing"
                )
            if "next_if and next_if_eq return receiver-bounded" in facts:
                errors.append(
                    "ITER-ADAPT-PEEK-01: next_if/next_if_eq still have false receiver provenance"
                )

        if contract_id == "RC-UNIQUE-01":
            if "FL-ATOMIC" in capabilities:
                errors.append("RC-UNIQUE-01: divergent OOM/abort edges must not imply FL-ATOMIC")
            if OP9_OOM_CLAUSE not in ownership or OP9_REFERENCE not in evidence:
                errors.append("RC-UNIQUE-01: make_mut OOM edge lacks exact OP-9 translation")
            if EFF4_REFERENCE not in evidence:
                errors.append("RC-UNIQUE-01: CloneToUninit abort lacks direct EFF-4 evidence")

        if contract_id == "TRAIT-CLONE-01":
            clone_text = f"{sketch} {ownership} {facts} {canaries}".lower()
            for fragment in (
                "ref::clone",
                "same exact referent",
                "shared-borrow unit",
                "borrow-count overflow",
                "referent payload is not cloned",
            ):
                if fragment not in clone_text:
                    errors.append(
                        f"TRAIT-CLONE-01: Ref::clone guard branch lacks {fragment!r}"
                    )
            for fragment in (
                "each normal guard end decrements one",
                "deliberate abandonment does not mint a release fact",
                "without inventing a decrement",
            ):
                if fragment not in clone_text:
                    errors.append(
                        f"TRAIT-CLONE-01: guard-leak policy lacks {fragment!r}"
                    )
            for fragment in (
                "declared clone result-provenance relation",
                "may select, swap, or coalesce independently valid roots",
                "overwritten old destination leaf ends exactly once",
                "reused destination allocation or storage grants no provenance",
                "for ref::clone only reject",
                "shared-reference clone_from",
                "destination referent to change from its old root to the source root",
            ):
                if fragment not in clone_text:
                    errors.append(
                        f"TRAIT-CLONE-01: generic Clone provenance lacks {fragment!r}"
                    )
            if "destination validity and every leaf relation survive replacement or reuse" in clone_text:
                errors.append(
                    "TRAIT-CLONE-01: clone_from still preserves overwritten destination provenance"
                )

        if contract_id == "TRAIT-DEFAULT-01":
            default_text = f"{sketch} {ownership} {facts} {canaries}".lower()
            for fragment in (
                "direct borrowed-empty",
                "well-aligned zero-length view",
                "exact zero length",
                "empty access footprint",
                "equal pointer values",
                "no br-disjoint",
                "associated default result-provenance relation",
                "independently valid static, global, promoted, or otherwise declared root",
                "no leaf is fabricated from the new owner storage, temporary receiver, or call frame",
                "owned default result containing two independently rooted",
            ):
                if fragment not in default_text:
                    errors.append(
                        f"TRAIT-DEFAULT-01: borrowed-empty branch lacks {fragment!r}"
                    )
            if "borrowed-empty default" not in evidence.lower():
                errors.append(
                    "TRAIT-DEFAULT-01: borrowed-empty branch lacks pinned source evidence"
                )

        if contract_id == "RANGE-BOUNDS-PROTOCOL-01":
            range_text = f"{sketch} {ownership} {facts} {canaries}".lower()
            for fragment in (
                "physical root remains the pre-existing external referent",
                "result-region facts expire with the receiver borrow",
                "receiver-field root substituted for an external root",
            ):
                if fragment not in range_text:
                    errors.append(
                        f"RANGE-BOUNDS-PROTOCOL-01: provenance split lacks {fragment!r}"
                    )
            if "all provenance facts remain tied to the receiver" in range_text:
                errors.append(
                    "RANGE-BOUNDS-PROTOCOL-01: external endpoint provenance was collapsed into receiver storage"
                )

        if contract_id == "TEXT-PARSE-01":
            parse_text = f"{sketch} {ownership} {facts} {canaries}".lower()
            for fragment in (
                "no input lifetime",
                "independently valid static root",
                "promoted zero-sized root",
                "empty footprint grants no storage or disjointness authority",
                "never derives from the input text",
                "non-static external source",
            ):
                if fragment not in parse_text:
                    errors.append(
                        f"TEXT-PARSE-01: result provenance lacks {fragment!r}"
                    )
            for forbidden in ("static/external root", "external/static root"):
                if forbidden in parse_text:
                    errors.append(
                        f"TEXT-PARSE-01: result provenance is overbroad: {forbidden!r}"
                    )

        if contract_id == "TRAIT-CONVERT-01":
            convert_text = f"{sketch} {ownership} {facts} {canaries}".lower()
            for fragment in (
                "borrowed-view conversion branch",
                "moving or representation-reusing an owned payload mints no fresh borrow",
                "pre-existing payload borrow leaf preserves its exact external or promoted-empty root",
                "clone-based conversion instead follows the separately frozen clone result-provenance relation",
                "invokes t::clone exactly n times on normal success",
                "no post-state or cleanup is promised",
                "memory safety does not depend on trap-edge cleanup",
                "clone-based results instead follow the declared clone relation",
                "use a t::clone that swaps or coalesces two shared roots",
                "exercise n=0, n=1, and a trap at every clone index",
                "inject allocation/capacity failure only at the contract's allocating points",
            ):
                if fragment not in convert_text:
                    errors.append(
                        f"TRAIT-CONVERT-01: conversion provenance split lacks {fragment!r}"
                    )
            for forbidden in (
                "any result borrow from consuming or owned-result branches",
                "movement, explicit clone, or representation reuse",
                "every pre-existing payload borrow leaf preserves its exact external or promoted-empty root through movement, explicit clone",
                "after a clone trap, every initialized output prefix owner is destroyed exactly once",
                "cleans exactly the initialized prefix",
            ):
                if forbidden in convert_text:
                    errors.append(
                        f"TRAIT-CONVERT-01: conversion branches remain falsely unioned: {forbidden!r}"
                    )

        if contract_id == "VIEW-CONCAT-01":
            concat_text = (
                f"{sketch} {ownership} {row.get('asymptotic_argument', '')} "
                f"{facts} {canaries} {evidence}"
            ).lower()
            for fragment in (
                "concat, join, and connect branches invoke t::clone",
                "repeat is a distinct t: copy branch and invokes clone zero times",
                "declared clone result-provenance relation",
                "may select, swap, or coalesce independently valid roots",
                "memory safety does not depend on trap-edge cleanup",
                "exact clone-call count",
                "use a t::clone that swaps or coalesces two shared roots",
                "library/alloc/src/slice.rs:509-512,725-780",
            ):
                if fragment not in concat_text:
                    errors.append(
                        f"VIEW-CONCAT-01: Clone/Copy branch split lacks {fragment!r}"
                    )

        if contract_id == "VIEW-ALLOC-01":
            alloc_text = (
                f"{sketch} {ownership} {row.get('asymptotic_argument', '')} "
                f"{facts} {canaries} {evidence}"
            ).lower()
            for fragment in (
                "to_vec borrows a slice, allocates once, and invokes t::clone exactly once per source element",
                "declared clone result-provenance relation",
                "may select, swap, or coalesce independently valid roots",
                "into_vec consumes the boxed-slice owner",
                "moves or representation-reuses every existing payload owner without clone",
                "exactly n clone calls",
                "require zero clone calls",
                "library/alloc/src/slice.rs:394-433,476-482",
            ):
                if fragment not in alloc_text:
                    errors.append(
                        f"VIEW-ALLOC-01: to_vec/into_vec split lacks {fragment!r}"
                    )

        if contract_id in {"SEQ-EXTRACT-01", "SEQ-SPLICE-01"}:
            range_text = f"{ownership} {facts} {canaries}".lower()
            for fragment in (
                "rangebounds r is evaluated only while live",
                "destroyed exactly once before",
                "cursor retains no r owner",
                "r endpoint capture provenance remains tied to its independent sources",
                "no cursor, yielded item, payload, or repair authority derives from r",
                "drop-tracked borrow-bearing r",
            ):
                if fragment not in range_text:
                    errors.append(
                        f"{contract_id}: eager RangeBounds disposition lacks {fragment!r}"
                    )

        if contract_id in {"OMAP-FILTER-01", "OSET-FILTER-01"}:
            range_text = f"{ownership} {facts} {canaries}".lower()
            for fragment in (
                "moves rangebounds r into the returned extractif cursor",
                "first and repeated terminal none retain r unchanged",
                "cursor destruction from unused, partial, or terminal state destroys retained r exactly once after f",
                "each borrow leaf retained inside r keeps its external source root",
                "no yielded",
                "authority derives from r",
                "drop-tracked r with external borrow leaves",
            ):
                if fragment not in range_text:
                    errors.append(
                        f"{contract_id}: retained RangeBounds disposition lacks {fragment!r}"
                    )

        hash_state_fragments = {
            "HMAP-FILTER-01": (
                "leave the exact stored buildhasher s in the source owner",
                "s is neither moved nor destroyed",
                "unused, partial, and exhausted cursor states",
                "no slot-live fact, payload provenance, cursor authority, mutation authority, or yielded item derives from s",
                "require zero s movement or destruction while the source owner lives",
            ),
            "HSET-FILTER-01": (
                "leave the exact stored buildhasher s in the source owner",
                "s is neither moved nor destroyed",
                "unused, partial, and exhausted cursor states",
                "no slot-live fact, payload provenance, cursor authority, mutation authority, or yielded item derives from s",
                "require zero s movement or destruction while the source owner lives",
            ),
            "SET-REL-01": (
                "call-scoped-reborrows only the stored buildhasher s",
                "neither moved nor destroyed",
                "boolean result, payload provenance, and storage facts never derive from s",
            ),
            "SET-ALG-02": (
                "lazy iterator retains source-set borrows",
                "call-scoped-reborrows the relevant stored buildhasher s",
                "every s remains in its source set",
                "every yielded value derives only from its selected source-set payload",
            ),
            "TRAIT-INDEX-01": (
                "only the hashmap branch call-scoped-reborrows stored buildhasher s",
                "s remains map-owned",
                "returned value borrow derives from exact map payload storage",
                "no result borrow, payload authority, mutation authority, or occupancy fact is retargeted through s",
            ),
            "TRAIT-INTOITER-01": (
                "owning hashmap or hashset entrance consumes stored buildhasher s",
                "destroys it exactly once before returning the payload cursor",
                "shared and unique entrances leave s in the source owner",
                "no cursor contains s",
                "no yielded item, cursor source map, result borrow, or traversal authority derives from s",
            ),
            "TRAIT-EXTEND-01": (
                "keeps the same stored buildhasher s throughout extension",
                "call-scoped-reborrows s for every required hash operation",
                "extend neither moves nor destroys s",
                "each required hash operation invokes s::build_hasher exactly once",
                "one call-local h owner",
                "buildhasher result-provenance relation",
                "never derives from the call-scoped &s receiver, s-owned field storage, destination storage, or the call frame",
                "destroyed exactly once before that insertion step completes",
                "hash output may influence the logical probe, destination, or duplicate decision",
                "alone mints no occupancy, liveness, uniqueness, or check-elision fact",
                "no accepted item, payload location, public result borrow, or destination fact derives from s or h",
            ),
            "TRAIT-COLLECT-01": (
                "default-constructs exactly one buildhasher s into partial-output state",
                "every s borrow leaf follows the default result-provenance relation",
                "never derives from partial-output storage, the returned collection, or the call frame",
                "transfers that same s into the completed owner",
                "every normal cleanup path destroys a constructed s exactly once",
                "empty input performs zero hash calls but still returns one s owner",
                "each required hash operation invokes s::build_hasher exactly once",
                "one call-local h owner",
                "buildhasher result-provenance relation",
                "never derives from the call-scoped &s receiver, s-owned field storage, partial-output or returned-collection storage, or the call frame",
                "destroyed exactly once before that insertion step completes",
                "hash output may influence the logical probe, destination, or duplicate decision",
                "alone mints no occupancy, liveness, uniqueness, or check-elision fact",
                "never become sources for s or h leaves or derive authority from s or h",
            ),
            "TRAIT-CMP-01": (
                "receives caller-owned mutable hasher h",
                "uses only call-scoped reborrows for writes",
                "neither moves nor destroys h",
                "no relation/hash result, traversed payload, or storage fact derives from h",
            ),
        }
        if contract_id in hash_state_fragments:
            hash_text = f"{ownership} {facts} {canaries}".lower()
            if "AB-STATEFUL" not in capabilities:
                errors.append(f"{contract_id}: hash behavior state lacks AB-STATEFUL")
            for fragment in hash_state_fragments[contract_id]:
                if fragment not in hash_text:
                    errors.append(
                        f"{contract_id}: hash behavior-state route lacks {fragment!r}"
                    )
            if contract_id in {"TRAIT-EXTEND-01", "TRAIT-COLLECT-01"} and (
                "library/core/src/hash/mod.rs:637-656,694-701"
                not in evidence.lower()
            ):
                errors.append(
                    f"{contract_id}: generated-hasher source evidence is missing"
                )

        hash_behavior_effect_contracts = {
            "HMAP-RESERVE-01",
            "HMAP-LOOKUP-01",
            "HMAP-DISJOINT-01",
            "HMAP-INSERT-01",
            "HMAP-REMOVE-01",
            "HSET-RESERVE-01",
            "HSET-LOOKUP-01",
            "HSET-INSERT-01",
            "HSET-REMOVE-01",
            "MAP-ENTRY-01",
            "SET-REL-01",
            "SET-ALG-02",
            "TRAIT-INDEX-01",
            "TRAIT-EXTEND-01",
            "TRAIT-COLLECT-01",
        }
        if contract_id in hash_behavior_effect_contracts:
            hash_text = f"{ownership} {facts} {canaries}".lower()
            for fragment in (
                "same stored buildhasher s owner remains valid",
                "declared buildhasher behavior-effect relation",
                "s's post-call leaves jointly follow",
                "unique leaf transferred from s into h ends in s before becoming live in h",
                "never simultaneously live in both",
                "same h owner remains valid",
                "declared hasher behavior-effect relation",
                "destroyed exactly once with its remaining state",
                "address or storage of an s field",
                "compiled root-swap and unique-transfer buildhasher canaries",
                "s and h mint no payload, public-result, occupancy",
                "equivalence, or check-elision authority",
            ):
                if fragment not in hash_text:
                    errors.append(
                        f"{contract_id}: BuildHasher/Hasher behavior-effect law lacks {fragment!r}"
                    )
            if "library/core/src/hash/mod.rs:258-357,637-656,694-701" not in evidence.lower():
                errors.append(
                    f"{contract_id}: BuildHasher/Hasher behavior source evidence is missing"
                )
            for stale in (
                "preserve s and that root across the operation",
                "preserve the same s owner and root",
                "each s capture leaf retains its independent external root",
                "every h capture leaf retains its independent external root across writes",
            ):
                if stale in hash_text:
                    errors.append(
                        f"{contract_id}: stale same-leaf-set claim remains: {stale!r}"
                    )

        if contract_id == "TRAIT-CMP-01":
            hash_text = f"{ownership} {facts} {canaries}".lower()
            for fragment in (
                "only hashmap/hashset equality iterates the left operand",
                "only right-hand s is invoked",
                "left-hand s remains retained and unreborrowed",
                "length mismatch and empty equality perform zero build_hasher calls",
                "each performed right-hand probe creates exactly one generated h",
                "only the hash implementation branch uses caller-owned h and never invokes buildhasher",
                "other comparison branches use neither role",
                "must never be unioned into every comparison",
                "buildhasher behavior-effect",
                "hasher behavior-effect",
                "transferred unique leaf ends in s before becoming live in h",
            ):
                if fragment not in hash_text:
                    errors.append(
                        f"{contract_id}: disjoint equality/Hash hasher partition lacks {fragment!r}"
                    )
            for fragment in (
                "library/std/src/collections/hash/map.rs:1319-1328",
                "set.rs:1027-1036",
                "library/core/src/hash/mod.rs:258-357,637-656,694-701",
            ):
                if fragment not in evidence.lower():
                    errors.append(
                        f"{contract_id}: hasher partition source evidence lacks {fragment!r}"
                    )
            if "every h capture leaf retains its independent external root across writes" in hash_text:
                errors.append(
                    f"{contract_id}: stale caller-H leaf-preservation claim remains"
                )

        exact_call_fragments = {
            "RANGE-BOUND-MAP-01": (
                "included and excluded consume it in exactly one invocation",
                "unbounded invokes it zero times and destroys it unused",
                "t, u, and environment disposition are never unioned",
            ),
            "RC-CYCLIC-01": (
                "invoked exactly once and consumed by that invocation",
                "there is no normal uninvoked-environment branch",
                "reject zero or multiple normal calls",
                "returned t may preserve only callable-capture or independently valid provenance authorized by the result relation",
                "reject any leaf rooted in the temporary provisional &weak<t> argument, weak identity, callable-environment storage, rc allocation, or call frame",
            ),
            "REF-GUARD-01": (
                "invokes and consumes its owned fnonce environment exactly once",
                "ref::clone has no callable environment",
                "reject zero/multiple projection calls",
            ),
            "REFCELL-REPLACE-01": (
                "replace_with invokes and consumes its owned fnonce environment exactly once",
                "replace, take, and swap have no callable environment",
                "reject zero/multiple replace_with calls",
            ),
        }
        if contract_id in exact_call_fragments:
            call_text = f"{ownership} {facts} {canaries}".lower()
            for fragment in exact_call_fragments[contract_id]:
                if fragment not in call_text:
                    errors.append(
                        f"{contract_id}: exact callable partition lacks {fragment!r}"
                    )

        noncached_key_contracts = {
            "VIEW-SEARCH-02",
            "VIEW-ORDER-CHECK-01",
            "VIEW-SORT-02",
            "VIEW-SELECT-01",
            "SEQ-DEDUP-01",
            "DEQUE-SEARCH-01",
        }
        key_text = f"{ownership} {facts} {canaries}".lower()
        if contract_id == "VIEW-SORT-01":
            for fragment in (
                "sort_by_cached_key retains and later destroys its key array exactly once",
                "for cached keys, abandon at every construction/ordering position",
            ):
                if fragment not in key_text:
                    errors.append(
                        f"VIEW-SORT-01: cached-key-array route lacks {fragment!r}"
                    )
        elif "retains and later destroys its key array" in key_text:
            errors.append(
                f"{contract_id}: only VIEW-SORT-01 may retain a cached-key array"
            )
        if contract_id in noncached_key_contracts:
            for fragment in (
                "every operation-local key/projection result is consumed or destroyed exactly once",
                "this contract owns no cached key array",
                "invented retained-key-array authority",
            ):
                if fragment not in key_text:
                    errors.append(
                        f"{contract_id}: operation-local key route lacks {fragment!r}"
                    )

        if contract_id == "REF-GUARD-01" and "BR-REBORROW" in capabilities:
            errors.append(
                "REF-GUARD-01: consumed map_split input guard must not be modeled as a parent reborrow"
            )

        facts_off_sentence = "Facts-off retains checks and identical semantics."
        if facts.count(facts_off_sentence) > 1:
            errors.append(f"{contract_id}: duplicate facts-off sentence")
        sketch_abort_variants = (
            "Memory-safe abort edges are independently required." in sketch
            and "Memory-safe abort edges remain an independent obligation." in sketch
        )
        if sketch_abort_variants:
            errors.append(f"{contract_id}: duplicate abort-obligation templates")
        for lower_start in (
            ". reject",
            ". abandon",
            ". detect",
            ". verify",
            ". assert",
            ". inject",
            ". trap",
            ". close",
            ". exercise",
        ):
            if lower_start in canaries:
                errors.append(f"{contract_id}: lowercase canary sentence start {lower_start!r}")

        if "BR-RESULT" in capabilities and "BR-PROV" not in capabilities:
            errors.append(f"{contract_id}: BR-RESULT lacks BR-PROV")
        census_result_marker = (
            "result-borrow provenance"
            in census_row.get("required_obligations", "").lower()
        )
        if census_result_marker and not {"BR-PROV", "BR-RESULT"} <= capabilities:
            errors.append(
                f"{contract_id}: census result-borrow marker lacks BR-PROV/BR-RESULT"
            )

        result_text = census_row.get("post_state_result", "").lower()
        returns_borrow = any(
            marker in result_text for marker in ("borrow", "reference", "guard")
        )
        if returns_borrow and contract_id not in RESULT_BORROW_BOUNDARY_EXCEPTIONS:
            if not {"BR-PROV", "BR-RESULT"} <= capabilities:
                errors.append(
                    f"{contract_id}: returned-borrow contract lacks BR-PROV/BR-RESULT"
                )
        if contract_id in REPAIRED_RESULT_BORROW_CONTRACTS:
            if not {"BR-PROV", "BR-RESULT"} <= capabilities:
                errors.append(
                    f"{contract_id}: exact result-borrow repair capabilities regressed"
                )
            if "result-borrow provenance" not in facts.lower():
                errors.append(f"{contract_id}: result-borrow fact scope is missing")
            if "result-borrow" not in canaries.lower():
                errors.append(f"{contract_id}: result-borrow negative canary is missing")
            if (
                "owner-tied borrow provenance" not in sketch.lower()
                or "result-borrow provenance" not in sketch.lower()
            ):
                errors.append(f"{contract_id}: result-borrow derivation markers are missing")
            if "access/reborrow" not in family_lock:
                errors.append(f"{contract_id}: result-borrow family lock is missing")

        if "ST-HOLE" in capabilities and "EX-NORMAL" not in capabilities:
            errors.append(f"{contract_id}: ST-HOLE lacks EX-NORMAL")
        if "EX-ABANDON" in capabilities and "EX-NORMAL" not in capabilities:
            errors.append(f"{contract_id}: EX-ABANDON lacks EX-NORMAL")
        if "FL-CALLBACK" in capabilities and "EX-ABORT" not in capabilities:
            errors.append(f"{contract_id}: FL-CALLBACK lacks EX-ABORT")
        if census_implicates_abort(census_row) and "EX-ABORT" not in capabilities:
            errors.append(f"{contract_id}: positive panic/trap edge lacks EX-ABORT")
        if (
            census_implicates_abandonable_cursor(census_row)
            and contract_id != "TRAIT-DROP-01"
            and not {"EX-NORMAL", "EX-ABANDON", "BR-CURSOR"} <= capabilities
        ):
            errors.append(
                f"{contract_id}: abandonable cursor/guard lacks EX-NORMAL, EX-ABANDON, or BR-CURSOR"
            )
        if "EX-ABORT" in capabilities:
            if contract_id == "ALLOC-OOM-01":
                if OP9_OOM_CLAUSE not in ownership or OP9_REFERENCE not in evidence:
                    errors.append("ALLOC-OOM-01: OOM edge lacks the pinned OP-9 translation")
            elif not (
                EFF4_CLAUSE in ownership or GENERIC_EFF4_CLAUSE in ownership
            ):
                errors.append(f"{contract_id}: EX-ABORT lacks an explicit EFF-4 clause")
            elif EFF4_REFERENCE not in evidence:
                errors.append(f"{contract_id}: EX-ABORT lacks direct EFF-4 evidence")
        if contract_id == "HEAP-MUTATE-01":
            if "FL-ATOMIC" not in capabilities:
                errors.append("HEAP-MUTATE-01: recoverable growth lacks FL-ATOMIC")
            if (
                "Failure(error, own T)" not in ownership
                or "sole owner of the offered value" not in ownership
                or "heap contents" not in ownership
                or "unchanged" not in ownership
            ):
                errors.append(
                    "HEAP-MUTATE-01: recoverable failure result or unchanged state is incomplete"
                )
            if (
                "before the first destructive commit" not in canaries
                or "no transient hole or post-commit fact escapes" not in canaries
            ):
                errors.append("HEAP-MUTATE-01: recoverable failure canary is incomplete")
            if (
                "mints no post-state fact" not in facts
                or "preserves the pre-call owner/version" not in facts
            ):
                errors.append("HEAP-MUTATE-01: recoverable failure fact discipline is incomplete")

        if census_row.get("family", "") in COMPOSITION_FAMILIES:
            if "IT-COMPOSE" not in capabilities:
                errors.append(f"{contract_id}: composable iterator contract lacks IT-COMPOSE")
            if "without intermediate materialization" not in f"{sketch} {structural}":
                errors.append(
                    f"{contract_id}: composition route does not forbid intermediate materialization"
                )

        if contract_id in SEALED_STABLE_STEP_RANGE_CAPABILITIES:
            expected_capabilities = SEALED_STABLE_STEP_RANGE_CAPABILITIES[contract_id]
            if capabilities != expected_capabilities:
                errors.append(
                    f"{contract_id}: sealed Step capabilities {sorted(capabilities)} differ "
                    f"from {sorted(expected_capabilities)}"
                )
            combined_range_text = " ".join((*row.values(), *census_row.values()))
            for fragment in (
                SEALED_STABLE_STEP_TYPE_LIST,
                "21 standard borrow-free Copy types",
                "Step is unstable and downstream-stable code cannot implement it",
                "not because the shared receiver is pure",
                "public or unsafe Step authority",
                SEALED_STEP_SOURCE_HASH,
            ):
                if fragment not in combined_range_text:
                    errors.append(
                        f"{contract_id}: sealed Step contract lost {fragment!r}"
                    )
            if census_row.get("behavior_parameter", "").split(";", 1)[0] != "None":
                errors.append(f"{contract_id}: sealed Step row acquired user behavior")
            if contract_id == "RANGE-ITER-FROM-01":
                for fragment in (
                    "maximum representable value is yielded once",
                    "next demanded item traps before cursor mutation",
                    "never wraps",
                ):
                    if fragment not in combined_range_text:
                        errors.append(
                            f"{contract_id}: checked overflow contract lost {fragment!r}"
                        )
            elif contract_id == "RANGE-ITER-INCLUSIVE-01":
                if "without computing last + 1" not in combined_range_text:
                    errors.append(
                        f"{contract_id}: inclusive endpoint-once rule lost"
                    )
        if contract_id in {"TRAIT-EXACT-01", "TRAIT-FUSED-01"}:
            if "FT-STATE" in capabilities:
                errors.append(
                    f"{contract_id}: stable metadata/marker contract is not a live-state fact channel"
                )

        if status == "E":
            incompatible = sorted(
                capability_id
                for capability_id in capabilities
                if registry_status.get(capability_id)
                in E_INCOMPATIBLE_REGISTRY_STATUSES
            )
            if incompatible:
                errors.append(
                    f"{contract_id}: E route depends on non-established capabilities: "
                    f"{incompatible}"
                )
            if not row.get("current_route", "").startswith(
                ("Direct current route:", "Derived current route:")
            ):
                errors.append(
                    f"{contract_id}: E current_route must identify a direct or derived route"
                )
        if status == "P" and "proved" not in row.get("current_route", "").lower():
            errors.append(f"{contract_id}: P current_route must identify the proved pattern")
        if status == "U" and "unproved" not in row.get("current_route", "").lower():
            errors.append(f"{contract_id}: U current_route must identify what is unproved")
        if status == "FRAME" and "boundary" not in row.get("current_route", "").lower():
            errors.append(f"{contract_id}: FRAME current_route must name a boundary")
        if status == "DEFERRED" and "later-domain" not in row.get("current_route", ""):
            errors.append(f"{contract_id}: DEFERRED current_route must identify a later-domain route")
        if status == "BOUNDARY":
            if "Boundary evidence" not in row.get("current_route", ""):
                errors.append(
                    f"{contract_id}: BOUNDARY current_route must identify boundary evidence"
                )
            if "boundary evidence" not in row.get("family_lock_or_deferral", ""):
                errors.append(
                    f"{contract_id}: BOUNDARY row must preserve the underlying checked need"
                )
        if status == "NG" and "non-goal" not in row.get("current_route", ""):
            errors.append(f"{contract_id}: NG current_route must identify the non-goal")

        behavior = census_row.get("behavior_parameter", "").lower()
        if "FL-CALLBACK" in capabilities:
            if (
                (
                    not behavior
                    or behavior == "none"
                    or "statically selected" in behavior
                    or contract_id in STATIC_ONLY_BEHAVIOR_CONTRACTS
                )
                and contract_id not in RUNTIME_BEHAVIOR_REPAIR_CONTRACTS
            ):
                errors.append(
                    f"{contract_id}: FL-CALLBACK has no runtime-invoked behavior"
                )
            if EFF4_CLAUSE not in ownership:
                errors.append(f"{contract_id}: callback route omits the EFF-4 trap translation")
            if EFF4_REFERENCE not in evidence:
                errors.append(f"{contract_id}: callback route omits the EFF-4 evidence reference")
            if "Rust failure evidence:" not in ownership:
                errors.append(
                    f"{contract_id}: callback route does not separate Rust failure evidence"
                )
        elif CANARY_PHRASES["FL-CALLBACK"] in canaries or "invoked-behavior containment" in sketch:
            errors.append(f"{contract_id}: callback prose appears without FL-CALLBACK")

        if (
            "AB-BEHAVIOR" in capabilities
            and contract_id in STATIC_ONLY_BEHAVIOR_CONTRACTS
        ):
            errors.append(
                f"{contract_id}: static/type-only context is misclassified as callable behavior"
            )

        if "FL-ALLOC" in capabilities and not census_implicates_allocation_failure(census_row):
            errors.append(
                f"{contract_id}: FL-ALLOC is present without a census allocation-failure edge"
            )
        if "FL-ALLOC" not in capabilities and (
            CANARY_PHRASES["FL-ALLOC"] in canaries
            or "explicit allocation-failure policy" in sketch
        ):
            errors.append(f"{contract_id}: allocation-failure prose appears without FL-ALLOC")

        if "FL-ATOMIC" in capabilities and contract_id in NON_ATOMIC_CONTRACTS:
            errors.append(
                f"{contract_id}: FL-ATOMIC contradicts the census partial-progress contract"
            )
        if "FL-ATOMIC" not in capabilities and "recoverable failure atomicity" in sketch:
            errors.append(f"{contract_id}: atomicity prose appears without FL-ATOMIC")
        if "FL-ATOMIC" in capabilities and "recoverable failure atomicity" not in sketch:
            errors.append(f"{contract_id}: FL-ATOMIC lacks canonical atomicity prose")
        preserves_offered_owner = any(
            phrase in f"{sketch} {ownership}".lower()
            for phrase in (
                "preserve offered",
                "preserve the offered",
                "returns offered",
                "returns the offered",
                "offered owner unchanged",
                "offered affine",
                "leaves every offered owner",
                "sole owner of the offered",
            )
        )
        if preserves_offered_owner and "FL-ATOMIC" not in capabilities:
            errors.append(
                f"{contract_id}: offered-owner recoverable failure lacks FL-ATOMIC"
            )

        if "BR-DISJOINT" in capabilities:
            if contract_id in INTERNAL_ONLY_DISJOINTNESS_CONTRACTS:
                errors.append(
                    f"{contract_id}: BR-DISJOINT is not an exposed/runtime contract"
                )
            if not census_implicates_disjointness(census_row):
                errors.append(
                    f"{contract_id}: BR-DISJOINT lacks a census disjointness obligation"
                )
            if (
                "reject duplicate/overlapping mutable outputs before any borrow escapes"
                not in canaries.lower()
            ):
                errors.append(
                    f"{contract_id}: BR-DISJOINT lacks the exact overlap-before-escape canary"
                )
        elif (
            CANARY_PHRASES["BR-DISJOINT"] in canaries
            or "disjointness facts" in facts
            or "checked multi-place disjointness" in sketch
        ):
            errors.append(f"{contract_id}: disjointness prose appears without BR-DISJOINT")

        if "FT-STATE" in capabilities and not (
            capabilities & {"ST-DENSE", "ST-RING", "ST-SPARSE", "ST-DEPENDENT", "ST-HOLE"}
            or contract_id == "TRAIT-DROP-01"
        ):
            errors.append(f"{contract_id}: FT-STATE has no partial/live-set topology")
        if "FT-REFINE" in capabilities and "ST-REFINE" not in capabilities:
            errors.append(f"{contract_id}: FT-REFINE has no refinement-sealed source")
        if "FT-IDENTITY" in capabilities and not (
            capabilities & {"ID-LOGICAL", "ID-POOL"}
        ):
            errors.append(f"{contract_id}: FT-IDENTITY has no pool/logical identity")
        if "FT-BORROW" in capabilities and not (
            "borrow" in census_row.get("family", "")
            or contract_id in {
                "TRAIT-CLONE-01",
                "TRAIT-DROP-01",
                "RAW-UNSAFE-BORROW-01",
            }
        ):
            errors.append(f"{contract_id}: FT-BORROW has no dynamic-borrow contract")
        if "FT-SHARED" in capabilities and not (
            "ID-SHARED" in capabilities or contract_id == "TRAIT-DROP-01"
        ):
            errors.append(f"{contract_id}: FT-SHARED has no shared-lifecycle contract")

        for capability_id, fact_phrase in FACT_PHRASES.items():
            if (capability_id in capabilities) != (fact_phrase in facts):
                errors.append(
                    f"{contract_id}: {capability_id} and its fact-channel prose disagree"
                )
        for capability_id, canary_phrase in CANARY_PHRASES.items():
            canary_matches = canary_phrase.lower() in canaries.lower()
            if capability_id in capabilities and not canary_matches:
                errors.append(
                    f"{contract_id}: {capability_id} lacks its negative canary"
                )
            if (
                capability_id not in capabilities
                and canary_matches
                and capability_id not in {"FT-SHARED", "OW-RELOCATE"}
            ):
                errors.append(
                    f"{contract_id}: {canary_phrase!r} canary lacks {capability_id}"
                )

        registry_marker = "CAPABILITY-OBLIGATION-REGISTRY.tsv:"
        matches = re.findall(
            r"CAPABILITY-OBLIGATION-REGISTRY\.tsv:([A-Z0-9,-]+)", evidence
        )
        if evidence.count(registry_marker) != 1 or len(matches) != 1:
            errors.append(f"{contract_id}: evidence must contain one registry capability list")
        else:
            evidence_capabilities = matches[0].split(",")
            if evidence_capabilities != capability_ids:
                errors.append(
                    f"{contract_id}: evidence capability list differs from capability_ids"
                )
        census_reference = f"RUST-DATA-CONTRACT-CENSUS.tsv:{contract_id}"
        if census_reference not in evidence:
            errors.append(f"{contract_id}: missing exact census evidence reference")

    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root",
        type=Path,
        default=Path(__file__).resolve().parent.parent,
        help="minimal-systems-capability artifact directory",
    )
    args = parser.parse_args()

    errors = verify(args.root.resolve())
    if errors:
        for error in errors:
            print(f"derivation matrix: FAIL: {error}")
        return 1

    _, census_rows = read_tsv(args.root / "RUST-DATA-CONTRACT-CENSUS.tsv")
    _, registry_rows = read_tsv(args.root / "CAPABILITY-OBLIGATION-REGISTRY.tsv")
    print(
        "derivation matrix: PASS — "
        f"{len(census_rows)} non-importable coverage clusters mapped exactly once against "
        f"{len(registry_rows)} registered capabilities"
    )
    print(
        "status legend: "
        + "; ".join(f"{code}={meaning}" for code, meaning in STATUS_LEGEND.items())
    )
    print(
        "status note: G0 rows cannot receive E or P; current_route is a coarse "
        "obligation screen, not an exact member route. Redundant Rust aliases were "
        "removed during coverage clustering and therefore have no matrix status row."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
