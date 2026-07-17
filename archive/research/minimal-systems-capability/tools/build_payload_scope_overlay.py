#!/usr/bin/env python3
"""Build the exact stored-borrow scope classification and conditional overlay."""

from __future__ import annotations

import argparse
import csv
import io
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
CENSUS = ROOT / "RUST-DATA-CONTRACT-CENSUS.tsv"
MATRIX = ROOT / "DERIVATION-MATRIX.tsv"
REGISTRY = ROOT / "CAPABILITY-OBLIGATION-REGISTRY.tsv"
CLASSIFICATION = ROOT / "PAYLOAD-SCOPE-CLASSIFICATION.tsv"
OVERLAY = ROOT / "PAYLOAD-SCOPE-OVERLAY.tsv"

CLASSIFICATION_HEADER = [
    "contract_id",
    "stored_borrow_scope",
    "scope_owner_contract_ids",
    "rationale",
]
OVERLAY_HEADER = [
    "contract_id",
    "branch_id",
    "route_scope",
    "role",
    "returns_borrow_bearing_owner",
    "requires_result_provenance",
    "condition",
    "conditional_capability_ids",
    "disposition",
    "reopening_trigger",
]

ROUTE_SCOPE = "REGION_FREE_BORROW_FREE"
DISPOSITION = "DEFERRED:stored-borrow-family"
REOPENING_TRIGGER = (
    "Stored-borrow Family Lock proves this exact conditional branch and reopens every "
    "shared representation, fact path, or generated-code dependency it changes."
)

CLONE_FRESH_SOURCE_EFFECT = (
    "The same Clone source owner remains valid on normal return, but Clone may end, "
    "replace, or move its internal borrow leaves under the declared Clone source "
    "behavior-effect relation. Post-call source leaves and result leaves jointly follow "
    "that source-effect relation and the Clone result-provenance relation: surviving "
    "source leaves keep their roots, every ended leaf ends once, every new leaf is "
    "authorized, and a unique leaf transferred to the result ends in the source before "
    "result liveness and is never simultaneously live in both. "
)
CLONE_FROM_SOURCE_EFFECT = (
    "The same Clone source and destination owners remain valid on normal return, but "
    "Clone::clone_from may end, replace, or move their internal borrow leaves under the "
    "declared Clone source and destination behavior-effect relation. Post-call source and "
    "destination leaves jointly follow that relation: surviving leaves keep their roots, "
    "every overwritten or otherwise ended leaf ends once, every new leaf is authorized, "
    "and a unique leaf transferred from source to destination ends in source before "
    "destination liveness and is never simultaneously live in both. Reused destination "
    "allocation or storage grants no provenance. "
)
CLONE_REPEATED_SOURCE_EFFECT = (
    "For repeated helper calls, call i + 1 consumes call i's post-source state rather "
    "than a frozen original source leaf map. "
)


@dataclass(frozen=True)
class Branch:
    contract_id: str
    branch_id: str
    role: str
    condition: str
    returned_state: bool = False
    requires_result_relation: bool = False


ACTIVE_BR_STORED = {
    "VIEW-CHUNKBY-01",
    "VIEW-SPLIT-PRED-01",
    "TEXT-MATCH-ITER-01",
    "TEXT-SPLIT-PATTERN-01",
    "SEQ-EXTRACT-01",
    "SEQ-SPLICE-01",
    "LIST-EXTRACT-01",
    "OMAP-FILTER-01",
    "OSET-FILTER-01",
    "HMAP-FILTER-01",
    "HSET-FILTER-01",
    "TRAIT-EXTEND-01",
    "TRAIT-COLLECT-01",
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

SEALED_STABLE_STEP_RANGE_CONTRACTS = {
    "RANGE-ITER-HALFOPEN-01",
    "RANGE-ITER-FROM-01",
    "RANGE-ITER-INCLUSIVE-01",
}

SEALED_STABLE_STEP_RATIONALE = (
    "Rust 1.97.0 exposes these iterators to stable callers only for the exact 21-type "
    "standard Step set: u8,u16,u32,u64,u128,usize,i8,i16,i32,i64,i128,isize,char,"
    "NonZero<u8>,NonZero<u16>,NonZero<u32>,NonZero<u64>,NonZero<u128>,"
    "NonZero<usize>,Ipv4Addr,Ipv6Addr. Step and TrustedStep are unstable, stable "
    "downstream code cannot add implementations, and every listed type is borrow-free "
    "Copy. The cursor copies descriptor values and retains no borrow-bearing generic "
    "payload; descriptor preservation follows from that sealed Copy source set, not from "
    "shared-receiver purity. Rust 1.97.0 commit "
    "2d8144b7880597b6e6d3dfd63a9a9efae3f533d3 library/core/src/iter/range.rs "
    "SHA-256 ae3f9307f4b4972f418561ae2a0311586eb3dde782359b8aaef3244915256464."
)

WHOLE_STORED_TRANSITION = {
    "VIEW-SORT-01",
    "VIEW-SORT-02",
    "VIEW-SELECT-01",
    "VIEW-REORDER-01",
    "VIEW-SWAP-01",
    "VIEW-COPY-01",
    "VIEW-CLONE-01",
    "VIEW-FILL-01",
    "INIT-WRITE-01",
    "SEQ-META-01",
    "SEQ-RESERVE-01",
    "SEQ-TRY-RESERVE-01",
    "SEQ-SHRINK-01",
    "SEQ-PUSH-01",
    "SEQ-INSERT-01",
    "SEQ-APPEND-01",
    "SEQ-EXTEND-COPY-01",
    "SEQ-RESIZE-01",
    "SEQ-TRUNCATE-01",
    "SEQ-RETAIN-01",
    "SEQ-DEDUP-01",
    "DEQUE-META-01",
    "DEQUE-RESERVE-01",
    "DEQUE-CONTIG-01",
    "DEQUE-PUSH-01",
    "DEQUE-INSERT-01",
    "DEQUE-SWAP-01",
    "DEQUE-RESIZE-01",
    "DEQUE-RETAIN-01",
    "DEQUE-ROTATE-01",
    "LIST-META-01",
    "LIST-PUSH-01",
    "LIST-DROP-01",
    "HEAP-META-01",
    "HEAP-RESERVE-01",
    "HEAP-APPEND-01",
    "HEAP-RETAIN-01",
    "OMAP-META-01",
    "OMAP-CLEAR-01",
    "OSET-META-01",
    "HMAP-RESERVE-01",
    "HSET-RESERVE-01",
    "RC-UNIQUE-01",
    "TRAIT-DROP-01",
}

WHOLE_BORROW_BEARING_RESULT = {
    "VIEW-ALLOC-01",
    "VIEW-CONCAT-01",
    "BOX-NEW-01",
    "BOX-PIN-01",
    "SEQ-POP-01",
    "SEQ-REMOVE-01",
    "SEQ-DRAIN-01",
    "SEQ-SPLIT-01",
    "SEQ-CONVERT-01",
    "DEQUE-POP-01",
    "DEQUE-REMOVE-01",
    "DEQUE-DRAIN-01",
    "LIST-POP-01",
    "HEAP-CONVERT-01",
    "RC-NEW-01",
    "RC-UNWRAP-01",
    "RC-CYCLIC-01",
    "RC-PIN-01",
    "HELPER-ARRAY-INTOITER-01",
    "MEM-REPLACE-01",
    "MEM-TAKE-01",
}

WHOLE_RESULT_CONDITIONS = {
    "VIEW-ALLOC-01": (
        "slice::into_vec transfers existing payload owners and preserves each live borrow "
        "leaf root, while slice::to_vec obtains every produced T through the declared Clone "
        "result-provenance relation; a Clone result may select, swap, or coalesce independent "
        "roots but may not derive from temporary source-view, receiver, allocation, or "
        "call-frame storage. "
        + CLONE_FRESH_SOURCE_EFFECT
        + CLONE_REPEATED_SOURCE_EFFECT
    ),
    "VIEW-CONCAT-01": (
        "Every borrow-bearing element produced by concat, join, connect, or repeat follows "
        "the selected Concat, Join, Copy, or Clone result-provenance relation; no produced "
        "leaf derives from temporary source-view, receiver, output-allocation, or call-frame "
        "storage. The Clone branches additionally obey this source effect; Copy repeat is "
        "excluded. "
        + CLONE_FRESH_SOURCE_EFFECT
        + CLONE_REPEATED_SOURCE_EFFECT
    ),
    "RC-UNWRAP-01": (
        "The unique try_unwrap, into_inner, or unwrap_or_clone branch moves the existing T "
        "and preserves each leaf root; unwrap_or_clone's shared fallback instead obtains T "
        "through the declared Clone result-provenance relation and may not derive a leaf from "
        "temporary receiver, Rc allocation, or call-frame storage. In that fallback the "
        "original shared payload is the Clone source. "
        + CLONE_FRESH_SOURCE_EFFECT
    ),
    "RC-CYCLIC-01": (
        "Rc::new_cyclic stores the T returned by its FnOnce producer; every live borrow leaf "
        "in T follows the producer result-provenance relation and derives only from declared "
        "captures or independent static, global, or promoted roots, never from the temporary "
        "provisional &Weak<T> argument, Weak identity, callable storage, or call frame."
    ),
    "MEM-TAKE-01": (
        "mem::take returns the pre-existing old T owner and preserves every old borrow-leaf "
        "root exactly; the separately recorded Default-result branch governs the newly "
        "installed T and cannot retarget the returned old value."
    ),
}

CALLABLE_STATE_MEMBERS = {
    "ARR-MAP-01": "array::map's owned FnMut environment",
    "VIEW-SEARCH-02": "the *_by, *_by_key, or partition_point callable environment",
    "VIEW-ORDER-CHECK-01": "the *_by or *_by_key callable environment",
    "VIEW-SORT-01": "the behavior-taking stable-sort callable environment",
    "VIEW-SORT-02": "the behavior-taking unstable-sort callable environment",
    "VIEW-SELECT-01": "the behavior-taking selection callable environment",
    "VIEW-FILL-01": "fill_with's producer environment",
    "TEXT-SEARCH-01": "the owned Pattern or searcher environment",
    "TEXT-TRIM-01": "the owned Pattern environment",
    "TEXT-REPLACE-01": "the owned Pattern environment",
    "SEQ-POP-01": "pop_if's predicate environment",
    "SEQ-RESIZE-01": "resize_with's producer environment",
    "SEQ-RETAIN-01": "the retain predicate environment",
    "SEQ-DEDUP-01": "the dedup_by or dedup_by_key environment",
    "DEQUE-POP-01": "the *_if predicate environment",
    "DEQUE-RESIZE-01": "the producer-form resize environment",
    "DEQUE-RETAIN-01": "the retain predicate environment",
    "DEQUE-SEARCH-01": "the behavior-taking search environment",
    "HEAP-RETAIN-01": "the retain predicate environment",
    "STRING-RETAIN-01": "the retain predicate environment",
    "RC-CYCLIC-01": "new_cyclic's one-shot producer environment",
    "REF-GUARD-01": "the map, filter_map, or map_split callable environment",
    "REFCELL-REPLACE-01": "replace_with's one-shot callable environment",
    "RANGE-BOUND-MAP-01": "Bound::map's one-shot callable environment",
}

KEY_RESULT_STATE_MEMBERS = {
    "VIEW-SEARCH-02": "binary_search_by_key's produced key",
    "VIEW-ORDER-CHECK-01": "is_sorted_by_key's produced key",
    "VIEW-SORT-01": "sort_by_key's produced key",
    "VIEW-SORT-02": "sort_unstable_by_key's produced key",
    "VIEW-SELECT-01": "select_nth_unstable_by_key's produced key",
    "SEQ-DEDUP-01": "dedup_by_key's produced key",
    "DEQUE-SEARCH-01": "binary_search_by_key's produced key",
}

RANGE_BOUNDS_STATE_MEMBERS = {
    "VIEW-COPY-01": "copy_within",
    "SEQ-EXTEND-COPY-01": "extend_from_within",
    "SEQ-DRAIN-01": "drain",
    "DEQUE-RANGE-01": "range",
    "DEQUE-DRAIN-01": "drain",
    "OMAP-RANGE-01": "range",
    "OSET-RANGE-01": "range",
    "STRING-PUSH-01": "extend_from_within",
    "STRING-DRAIN-01": "drain",
    "STRING-REPLACE-01": "replace_range",
}

HASH_STATE_CONTRACTS = {
    "HMAP-RESERVE-01",
    "HMAP-LOOKUP-01",
    "HMAP-DISJOINT-01",
    "HMAP-INSERT-01",
    "HMAP-REMOVE-01",
    "HMAP-ITER-01",
    "HMAP-DRAIN-01",
    "HSET-RESERVE-01",
    "HSET-LOOKUP-01",
    "HSET-INSERT-01",
    "HSET-REMOVE-01",
    "HSET-ITER-01",
    "HSET-DRAIN-01",
    "SET-REL-01",
    "SET-ALG-02",
    "TRAIT-INDEX-01",
    "TRAIT-CMP-01",
}

HASH_BEHAVIOR_STATE_CONTRACTS = {
    "HMAP-RESERVE-01",
    "HMAP-LOOKUP-01",
    "HMAP-DISJOINT-01",
    "HMAP-INSERT-01",
    "HMAP-REMOVE-01",
    "HSET-RESERVE-01",
    "HSET-LOOKUP-01",
    "HSET-INSERT-01",
    "HSET-REMOVE-01",
    "SET-REL-01",
    "SET-ALG-02",
    "TRAIT-INDEX-01",
    "TRAIT-CMP-01",
}

GENERATED_HASHER_RESULT_CONTRACTS = {
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
    "TRAIT-CMP-01",
}

# Every source/member adjudication is closed. Keep this guard so a future audit can
# make generation fail closed while an additional branch is being classified.
UNRESOLVED: set[str] = set()


def branch(
    contract_id: str,
    branch_id: str,
    role: str,
    condition: str,
    *,
    returned_state: bool = False,
    requires_result_relation: bool = False,
) -> Branch:
    scope_sentence = (
        "The base matrix route applies only when every retained generic value in this "
        "branch is region-free and borrow-free."
    )
    if scope_sentence.lower() not in condition.lower():
        condition = f"{condition} {scope_sentence}"
    return Branch(
        contract_id,
        branch_id,
        role,
        condition,
        returned_state,
        requires_result_relation,
    )


def explicit_branches() -> list[Branch]:
    rows: list[Branch] = []
    generic_condition = (
        "The generic retained payload contains one or more shared or unique borrow leaves; "
        "the base matrix route applies only when that payload is region-free and borrow-free."
    )
    for contract_id in sorted(WHOLE_STORED_TRANSITION):
        rows.append(
            branch(
                contract_id,
                "BORROW_BEARING_STORED_TRANSITION",
                "STORED_TRANSITION",
                generic_condition,
            )
        )
    for contract_id in sorted(WHOLE_BORROW_BEARING_RESULT):
        condition = WHOLE_RESULT_CONDITIONS.get(contract_id, generic_condition)
        rows.append(
            branch(
                contract_id,
                "BORROW_BEARING_OWNED_RESULT",
                "BORROW_BEARING_RESULT",
                condition,
                returned_state=True,
            )
        )

    member_rows = [
        branch("DEQUE-BULK-01", "APPEND_BORROW_BEARING_PAYLOAD", "STORED_TRANSITION", "VecDeque::append moves retained payload values containing borrow leaves between owners; the base matrix branch is region-free and borrow-free."),
        branch("DEQUE-BULK-01", "SPLIT_OFF_BORROW_BEARING_RESULT", "BORROW_BEARING_RESULT", "VecDeque::split_off returns a new owner whose retained payload values contain borrow leaves; the base matrix branch is region-free and borrow-free.", returned_state=True),
        branch("LIST-BULK-01", "APPEND_BORROW_BEARING_PAYLOAD", "STORED_TRANSITION", "LinkedList::append rewires retained payload values containing borrow leaves into another owner; the base matrix branch is region-free and borrow-free."),
        branch("LIST-BULK-01", "SPLIT_OFF_BORROW_BEARING_RESULT", "BORROW_BEARING_RESULT", "LinkedList::split_off returns a new owner whose retained payload values contain borrow leaves; the base matrix branch is region-free and borrow-free.", returned_state=True),
        branch("HEAP-MUTATE-01", "PUSH_BORROW_BEARING_PAYLOAD", "STORED_TRANSITION", "BinaryHeap::push stores a payload containing borrow leaves; the base matrix branch is region-free and borrow-free."),
        branch("HEAP-MUTATE-01", "POP_BORROW_BEARING_RESULT", "BORROW_BEARING_RESULT", "BinaryHeap::pop returns an owned payload containing borrow leaves; the base matrix branch is region-free and borrow-free.", returned_state=True),
        branch("HEAP-DRAIN-01", "CLEAR_BORROW_BEARING_PAYLOAD", "STORED_TRANSITION", "BinaryHeap::clear destroys retained payload values containing borrow leaves; the base matrix branch is region-free and borrow-free."),
        branch("HEAP-DRAIN-01", "DRAIN_BORROW_BEARING_RESULT", "BORROW_BEARING_RESULT", "BinaryHeap::drain returns owning traversal state that yields payload values containing borrow leaves; the base matrix branch is region-free and borrow-free.", returned_state=True),
        branch("OMAP-BULK-01", "APPEND_BORROW_BEARING_ENTRIES", "STORED_TRANSITION", "BTreeMap::append moves retained key or value roles containing borrow leaves between owners; the base matrix branch is region-free and borrow-free."),
        branch("OMAP-BULK-01", "SPLIT_OFF_BORROW_BEARING_RESULT", "BORROW_BEARING_RESULT", "BTreeMap::split_off returns a new owner whose key or value roles contain borrow leaves; the base matrix branch is region-free and borrow-free.", returned_state=True),
        branch("OSET-BULK-01", "APPEND_CLEAR_BORROW_BEARING_PAYLOAD", "STORED_TRANSITION", "BTreeSet::append or clear moves or destroys retained payload values containing borrow leaves; the base matrix branch is region-free and borrow-free."),
        branch("OSET-BULK-01", "SPLIT_OFF_BORROW_BEARING_RESULT", "BORROW_BEARING_RESULT", "BTreeSet::split_off returns a new owner whose retained payload values contain borrow leaves; the base matrix branch is region-free and borrow-free.", returned_state=True),
        branch("HMAP-DRAIN-01", "CLEAR_BORROW_BEARING_ENTRIES", "STORED_TRANSITION", "HashMap::clear destroys retained key or value roles containing borrow leaves; the base matrix branch is region-free and borrow-free."),
        branch("HMAP-DRAIN-01", "DRAIN_BORROW_BEARING_RESULT", "BORROW_BEARING_RESULT", "HashMap::drain returns owning traversal state that yields key or value roles containing borrow leaves; the base matrix branch is region-free and borrow-free.", returned_state=True),
        branch("HSET-DRAIN-01", "CLEAR_BORROW_BEARING_PAYLOAD", "STORED_TRANSITION", "HashSet::clear destroys retained payload values containing borrow leaves; the base matrix branch is region-free and borrow-free."),
        branch("HSET-DRAIN-01", "DRAIN_BORROW_BEARING_RESULT", "BORROW_BEARING_RESULT", "HashSet::drain returns owning traversal state that yields payload values containing borrow leaves; the base matrix branch is region-free and borrow-free.", returned_state=True),
        branch("DEQUE-ITER-01", "OWNING_INTOITER_BORROW_BEARING_RESULT", "BORROW_BEARING_RESULT", "VecDeque owning IntoIter state yields payload values containing borrow leaves; borrowed iteration members are not widened by this branch.", returned_state=True),
        branch("LIST-ITER-01", "OWNING_INTOITER_BORROW_BEARING_RESULT", "BORROW_BEARING_RESULT", "LinkedList owning IntoIter state yields payload values containing borrow leaves; borrowed iteration members are not widened by this branch.", returned_state=True),
        branch("OSET-RANGE-01", "OWNING_INTOITER_BORROW_BEARING_RESULT", "BORROW_BEARING_RESULT", "BTreeSet owning IntoIter state yields payload values containing borrow leaves; borrowed iter and range members are not widened by this branch.", returned_state=True),
        branch("HSET-ITER-01", "OWNING_INTOITER_BORROW_BEARING_RESULT", "BORROW_BEARING_RESULT", "HashSet owning IntoIter state yields payload values containing borrow leaves; borrowed iteration members are not widened by this branch.", returned_state=True),
        branch("TRAIT-INTOITER-01", "OWNING_ENTRANCE_BORROW_BEARING_RESULT", "BORROW_BEARING_RESULT", "The owning IntoIterator entrance retains and yields Item values containing borrow leaves; shared and unique entrances are not widened by this branch.", returned_state=True),
        branch("HEAP-VIEW-01", "OWNING_INTOITER_BORROW_BEARING_RESULT", "BORROW_BEARING_RESULT", "BinaryHeap owning IntoIter state yields payload values containing borrow leaves; as_slice and borrowed Iter members are not widened by this branch.", returned_state=True),
        branch("HEAP-PEEK-01", "PEEK_MUT_POP_BORROW_BEARING_RESULT", "BORROW_BEARING_RESULT", "PeekMut::pop returns an owned payload containing borrow leaves; peek and peek_mut outer-borrow members are not widened by this branch.", returned_state=True),
        branch("HMAP-META-01", "HASH_BUILDER_BORROW_BEARING_RESULT", "RETAINED_STATE", "with_hasher or with_capacity_and_hasher returns a map retaining BuildHasher state that contains borrow leaves; scalar metadata and hasher-view members are not widened.", returned_state=True),
        branch("HMAP-META-01", "EXISTING_HASH_BUILDER_BORROW_STATE", "RETAINED_STATE", "len, is_empty, capacity, or hasher operates on an existing map whose stored BuildHasher state contains live borrow leaves and remains valid across the call."),
        branch("HMAP-META-01", "EVENTUAL_BORROW_BEARING_PAYLOAD_DROP", "STORED_TRANSITION", "Destroying a nonempty HashMap destroys retained key or value roles containing borrow leaves; scalar metadata members are not widened."),
        branch("HSET-META-01", "HASH_BUILDER_BORROW_BEARING_RESULT", "RETAINED_STATE", "with_hasher or with_capacity_and_hasher returns a set retaining BuildHasher state that contains borrow leaves; scalar metadata and hasher-view members are not widened.", returned_state=True),
        branch("HSET-META-01", "EXISTING_HASH_BUILDER_BORROW_STATE", "RETAINED_STATE", "len, is_empty, capacity, or hasher operates on an existing set whose stored BuildHasher state contains live borrow leaves and remains valid across the call."),
        branch("HSET-META-01", "EVENTUAL_BORROW_BEARING_PAYLOAD_DROP", "STORED_TRANSITION", "Destroying a nonempty HashSet destroys retained payload values containing borrow leaves; scalar metadata members are not widened."),
        branch("BOX-INIT-01", "WRITE_SEAL_BORROW_BEARING_RESULT", "BORROW_BEARING_RESULT", "Box::write seals a fully initialized Box whose payload contains borrow leaves; uninitialized allocation constructors contain no live T and are not widened.", returned_state=True),
        branch("REFCELL-OWNER-01", "NEW_INTO_INNER_BORROW_BEARING_RESULT", "BORROW_BEARING_RESULT", "RefCell::new stores and into_inner returns a payload containing borrow leaves; get_mut is only an outer result borrow and is not widened.", returned_state=True),
        branch("BOX-DOWNCAST-01", "SUCCESS_CONCRETE_BORROW_PAYLOAD", "BORROW_BEARING_RESULT", "A successful downcast returns the same Box allocation as Box<T>, and T has a static lifetime yet contains one or more live static borrow leaves.", returned_state=True),
        branch("BOX-DOWNCAST-01", "FAILURE_ERASED_BORROW_PAYLOAD", "BORROW_BEARING_RESULT", "A failed downcast returns the exact Box<dyn Any> owner whose erased payload has a static lifetime yet contains one or more live static borrow leaves.", returned_state=True),
        branch("RC-DOWNCAST-01", "SUCCESS_CONCRETE_BORROW_PAYLOAD", "BORROW_BEARING_RESULT", "A successful downcast returns the same shared allocation as Rc<T>, and T has a static lifetime yet contains one or more live static borrow leaves.", returned_state=True),
        branch("RC-DOWNCAST-01", "FAILURE_ERASED_BORROW_PAYLOAD", "BORROW_BEARING_RESULT", "A failed downcast returns the exact Rc<dyn Any> owner whose erased payload has a static lifetime yet contains one or more live static borrow leaves.", returned_state=True),

        # Range values store generic endpoints. The three stable-callable iterator
        # contracts are deliberately absent: their exact Step set is sealed to 21
        # standard borrow-free Copy types by the pinned Rust 1.97 source.
        branch("RANGE-VALUE-HALFOPEN-01", "RANGE_ENDPOINTS", "RETAINED_STATE", "Range<Idx> retains start and end, and at least one endpoint contains a borrow leaf.", returned_state=True),
        branch("RANGE-VALUE-FROM-01", "RANGE_FROM_START", "RETAINED_STATE", "RangeFrom<Idx> retains a borrow-bearing start.", returned_state=True),
        branch("RANGE-VALUE-INCLUSIVE-01", "RANGE_INCLUSIVE_ENDPOINTS", "RETAINED_STATE", "A plain inclusive range retains a borrow-bearing start or last endpoint.", returned_state=True),
        branch("RANGE-VALUE-TO-INCLUSIVE-01", "RANGE_TO_INCLUSIVE_END", "RETAINED_STATE", "The retained inclusive upper endpoint contains a borrow leaf.", returned_state=True),
        branch("RANGE-VALUE-TO-EXCLUSIVE-01", "RANGE_TO_END", "RETAINED_STATE", "The retained exclusive upper endpoint contains a borrow leaf.", returned_state=True),
        branch("RANGE-BOUND-VALUE-01", "INCLUDED_EXCLUDED_PAYLOAD", "RETAINED_STATE", "Included(T) or Excluded(T) retains a borrow-bearing T; Unbounded is excluded.", returned_state=True),
        branch("RANGE-BOUND-CLONE-01", "CLONED_INPUT_BOUND_REF", "STORED_TRANSITION", "The consumed bounded input is Bound<&T> and therefore itself stores an external borrow leaf."),
        branch("RANGE-BOUND-CLONE-01", "CLONED_OUTPUT_PAYLOAD", "BORROW_BEARING_RESULT", "Cloned T contains borrow leaves and is stored in returned Included(T) or Excluded(T). The borrowed endpoint value is the Clone source. Every result leaf follows the declared Clone result-provenance relation and may select, swap, or coalesce independently valid roots; no leaf derives from the temporary Bound receiver, borrowed referent view, or call frame. " + CLONE_FRESH_SOURCE_EFFECT, returned_state=True),
        branch("RANGE-BOUND-MAP-01", "MAP_INPUT_PAYLOAD", "STORED_TRANSITION", "A bounded input T contains borrow leaves and is moved out of Bound<T> into the selected behavior."),
        branch("RANGE-BOUND-MAP-01", "MAP_OUTPUT_PAYLOAD", "BORROW_BEARING_RESULT", "Mapped U contains borrow leaves and is retained in returned Bound<U>.", returned_state=True),
        branch("RANGE-LEGACY-HALFOPEN-STATE-01", "RANGE_ENDPOINTS", "RETAINED_STATE", "The public legacy range value retains a borrow-bearing start or end independently of whether it is iterable.", returned_state=True),
        branch("RANGE-LEGACY-FROM-STATE-01", "RANGE_FROM_START", "RETAINED_STATE", "The public legacy unbounded cursor/value retains a borrow-bearing start.", returned_state=True),
        branch("RANGE-LEGACY-INCLUSIVE-STATE-01", "RANGE_INCLUSIVE_ENDPOINTS", "RETAINED_STATE", "Construction retains a borrow-bearing start or end alongside exhaustion state.", returned_state=True),
        branch("RANGE-LEGACY-INCLUSIVE-INTO-01", "INTO_INNER_ENDPOINTS", "BORROW_BEARING_RESULT", "Consuming into_inner moves borrow-bearing stored endpoints into the returned tuple.", returned_state=True),

        # Broad behavior contracts retain their direct borrowed-result duties in the
        # base matrix. These branches add only arbitrary borrow-bearing owned payload.
        branch("TRAIT-CONVERT-01", "FROM_OWNED_BORROW_PAYLOAD", "BORROW_BEARING_RESULT", "A selected From conversion moves or representation-reuses live generic payload into an owned array, Box, Vec, Rc-like, or equivalent result. Every pre-existing payload borrow leaf preserves its exact external or promoted-empty root; no fresh borrow derives from consumed container or allocation storage.", returned_state=True),
        branch("TRAIT-CONVERT-01", "FROM_CLONED_BORROW_PAYLOAD", "BORROW_BEARING_RESULT", "A selected From conversion explicitly clones payload from a borrowed source into a new owned result. Each cloned leaf follows the separately frozen Clone result-provenance relation and may select, swap, or coalesce independently valid roots; no result leaf derives from temporary source-view, receiver, container, or call-frame storage. " + CLONE_FRESH_SOURCE_EFFECT + CLONE_REPEATED_SOURCE_EFFECT, returned_state=True),
        branch("TRAIT-CONVERT-01", "TRY_FROM_OWNED_BORROW_PAYLOAD_OK", "BORROW_BEARING_RESULT", "A successful owned TryFrom conversion stores or transfers borrow-bearing payload into Ok(Self). Every pre-existing payload borrow leaf preserves its exact external or promoted-empty root; no fresh borrow derives from consumed container or allocation storage.", returned_state=True),
        branch("TRAIT-CONVERT-01", "TRY_FROM_BORROW_PAYLOAD_ERROR", "BORROW_BEARING_RESULT", "Recoverable failure returns the original owner or another error owner containing the original borrow-bearing live payload. Every pre-existing payload borrow leaf preserves its exact external or promoted-empty root; no fresh borrow derives from transient conversion state.", returned_state=True),
        branch("TRAIT-CLONE-01", "CLONE_FRESH_OWNED_PAYLOAD", "BORROW_BEARING_RESULT", "A fresh deep clone returns an owner or aggregate with live cloned payload borrow leaves. Each result leaf follows the declared Clone result-provenance relation and may select, swap, or coalesce independently valid roots; no leaf derives from temporary receiver or container storage. " + CLONE_FRESH_SOURCE_EFFECT, returned_state=True),
        branch("TRAIT-CLONE-01", "CLONE_FROM_OWNED_PAYLOAD", "STORED_TRANSITION", "clone_from updates an existing destination whose live payload contains borrow leaves. " + CLONE_FROM_SOURCE_EFFECT),
        branch("TRAIT-CLONE-01", "CLONE_SHARED_HANDLE_PAYLOAD", "BORROW_BEARING_RESULT", "Rc or Weak clone returns a new shared handle whose reachable live payload contains borrow leaves; leaf provenance is unchanged even though ownership identity is shared.", returned_state=True),
        branch("TRAIT-CLONE-01", "CLONE_CACHED_BORROW_BEARING_STATE", "RETAINED_STATE", "A helper clone returns independent protocol state containing an arbitrary cached Item, callable environment, or non-cursor State with borrow leaves. Each cloned leaf follows the declared Clone result-provenance relation; a source-map-only cursor clone remains covered by BR-CURSOR and is excluded. " + CLONE_FRESH_SOURCE_EFFECT, returned_state=True),
        branch("TRAIT-DEFAULT-01", "DEFAULT_LIVE_BORROW_PAYLOAD", "BORROW_BEARING_RESULT", "Default constructs one or more live T values inside Box<T>, Rc<T>, RefCell<T>, a fixed aggregate, or another owner, and the produced T contains borrow leaves. Every leaf follows the associated Default result-provenance relation and must name an independently valid static, global, promoted, or otherwise declared root; no leaf is fabricated from the new owner storage, temporary receiver, or call frame.", returned_state=True),
        branch("TEXT-PARSE-01", "SELF_OWNED_BORROW_PAYLOAD", "BORROW_BEARING_RESULT", "FromStr returns Ok(Self) as an owner or aggregate retaining independently valid static borrow leaves, including a promoted zero-sized empty-reference root; no result leaf can derive from the call-scoped input text, and empty footprint grants no storage or disjointness authority.", returned_state=True),
        branch("TEXT-PARSE-01", "ERROR_OWNED_BORROW_PAYLOAD", "BORROW_BEARING_RESULT", "FromStr returns Err(Self::Err) as an owner or aggregate retaining independently valid static borrow leaves, including a promoted zero-sized empty-reference root; no result leaf can derive from the call-scoped input text, and empty footprint grants no storage or disjointness authority.", returned_state=True),
        branch("RC-WEAK-01", "DOWNGRADE_BORROW_PAYLOAD_HANDLE", "BORROW_BEARING_RESULT", "Rc::downgrade returns Weak<T> preserving T's live borrow relations without keeping the payload alive; Weak::new is excluded.", returned_state=True),
        branch("RC-WEAK-01", "UPGRADE_BORROW_PAYLOAD_HANDLE", "BORROW_BEARING_RESULT", "A successful Weak::upgrade returns Rc<T> whose live T contains borrow leaves; failed upgrade and Weak::new are excluded.", returned_state=True),

        branch("ARR-MAP-01", "INPUT_ARRAY_BORROW_PAYLOAD", "STORED_TRANSITION", "The consumed [T; N] contains live borrow leaves; every T is moved to the callable or destroyed exactly once, regardless of whether U contains a borrow."),
        branch("ARR-MAP-01", "OUTPUT_ARRAY_BORROW_PAYLOAD", "BORROW_BEARING_RESULT", "The returned [U; N] contains live borrow leaves whose provenance is authorized by the selected callable.", returned_state=True),
        branch("ARR-MAP-01", "CALLABLE_ENV_BORROW_STATE", "STORED_TRANSITION", "The owned FnMut environment contains live borrow leaves, remains valid across every call, and is destroyed exactly once."),
        branch("VIEW-FILL-01", "CLONE_FROM_BORROW_EFFECT", "STORED_TRANSITION", "slice::fill invokes Clone::clone_from on each nonfinal live destination. clone_from returns no T. " + CLONE_FROM_SOURCE_EFFECT + CLONE_REPEATED_SOURCE_EFFECT),
        branch("VIEW-CLONE-01", "CLONE_FROM_BORROW_EFFECT", "STORED_TRANSITION", "clone_from_slice invokes Clone::clone_from for each live destination. clone_from returns no T. " + CLONE_FROM_SOURCE_EFFECT + CLONE_REPEATED_SOURCE_EFFECT),
        branch("VIEW-FILL-01", "PRODUCER_RESULT_BORROW_STATE", "STORED_TRANSITION", "fill_with stores each T returned by its FnMut producer. Every live borrow leaf in a produced T follows the callable result-provenance relation and never derives from a temporary destination reborrow, callable receiver, or call frame.", requires_result_relation=True),
        branch("INIT-WRITE-01", "CLONE_RESULT_BORROW_STATE", "STORED_TRANSITION", "write_clone_of_slice stores each T returned by Clone::clone. Every live borrow leaf follows the declared Clone result-provenance relation and never derives from the temporary source view, destination storage, receiver, or call frame. " + CLONE_FRESH_SOURCE_EFFECT + CLONE_REPEATED_SOURCE_EFFECT, requires_result_relation=True),
        branch("SEQ-EXTEND-COPY-01", "CLONE_RESULT_BORROW_STATE", "STORED_TRANSITION", "extend_from_slice and extend_from_within store each T returned by Clone::clone. Every live borrow leaf follows the declared Clone result-provenance relation and never derives from a temporary source view, destination allocation, receiver, or call frame. " + CLONE_FRESH_SOURCE_EFFECT + CLONE_REPEATED_SOURCE_EFFECT, requires_result_relation=True),
        branch("SEQ-RESIZE-01", "CLONE_RESULT_BORROW_STATE", "STORED_TRANSITION", "Vec::resize stores each T returned by Clone::clone. Every live borrow leaf follows the declared Clone result-provenance relation and never derives from the temporary seed reborrow, destination allocation, receiver, or call frame. " + CLONE_FRESH_SOURCE_EFFECT + CLONE_REPEATED_SOURCE_EFFECT, requires_result_relation=True),
        branch("SEQ-RESIZE-01", "PRODUCER_RESULT_BORROW_STATE", "STORED_TRANSITION", "Vec::resize_with stores each T returned by its producer. Every live borrow leaf follows the callable result-provenance relation and never derives from a temporary destination reborrow, callable receiver, or call frame.", requires_result_relation=True),
        branch("DEQUE-RESIZE-01", "CLONE_RESULT_BORROW_STATE", "STORED_TRANSITION", "VecDeque::resize stores each T returned by Clone::clone. Every live borrow leaf follows the declared Clone result-provenance relation and never derives from the temporary seed reborrow, destination allocation, receiver, or call frame. " + CLONE_FRESH_SOURCE_EFFECT + CLONE_REPEATED_SOURCE_EFFECT, requires_result_relation=True),
        branch("DEQUE-RESIZE-01", "PRODUCER_RESULT_BORROW_STATE", "STORED_TRANSITION", "VecDeque::resize_with stores each T returned by its producer. Every live borrow leaf follows the callable result-provenance relation and never derives from a temporary destination reborrow, callable receiver, or call frame.", requires_result_relation=True),
        branch("RC-UNIQUE-01", "MAKE_MUT_CLONE_RESULT_BORROW_STATE", "STORED_TRANSITION", "Rc::make_mut's multiple-strong branch stores the T produced by CloneToUninit or Clone from the original shared payload source. Every live borrow leaf follows the declared clone-result relation and never derives from the temporary Rc receiver, old or new allocation storage, or call frame; the weak-only relocation branch performs no clone and is excluded. " + CLONE_FRESH_SOURCE_EFFECT, requires_result_relation=True),
        branch("REFCELL-REPLACE-01", "RETURNED_BORROW_PAYLOAD", "BORROW_BEARING_RESULT", "replace, replace_with, or take returns the displaced T containing live borrow leaves; swap is excluded.", returned_state=True),
        branch("REFCELL-REPLACE-01", "SWAP_BORROW_PAYLOAD_STATE", "STORED_TRANSITION", "RefCell::swap exchanges two live T payloads containing borrow leaves between cells and returns no T owner."),
        branch("REFCELL-REPLACE-01", "INSTALLED_CALLBACK_RESULT_BORROW_STATE", "STORED_TRANSITION", "replace_with installs the T returned by FnOnce. Every live borrow leaf in the installed T follows the callable result-provenance relation and never derives from the temporary &mut T argument, cell storage, callable receiver, or call frame.", requires_result_relation=True),
        branch("REFCELL-REPLACE-01", "INSTALLED_DEFAULT_RESULT_BORROW_STATE", "STORED_TRANSITION", "RefCell::take installs the T returned by Default. Every live borrow leaf follows the declared Default result-provenance relation and never derives from the displaced old T, cell storage, receiver, or call frame.", requires_result_relation=True),
        branch("MEM-TAKE-01", "INSTALLED_DEFAULT_RESULT_BORROW_STATE", "STORED_TRANSITION", "mem::take installs the T returned by Default. Every live borrow leaf follows the declared Default result-provenance relation and never derives from the displaced old T, destination place, receiver, or call frame.", requires_result_relation=True),
        branch("TEXT-SEARCH-01", "PATTERN_SEARCHER_BORROW_STATE", "STORED_TRANSITION", "Pattern::into_searcher produces operation-local searcher state containing live borrow leaves. Every leaf follows the Pattern result-provenance relation, remains live through all selected searcher calls, and never derives from the temporary Pattern receiver or call frame.", requires_result_relation=True),
        branch("TEXT-TRIM-01", "PATTERN_SEARCHER_BORROW_STATE", "STORED_TRANSITION", "The Pattern branch produces operation-local searcher state containing live borrow leaves. Every leaf follows the Pattern result-provenance relation, remains live through boundary matching, and never derives from the temporary Pattern receiver or call frame.", requires_result_relation=True),
        branch("TEXT-REPLACE-01", "PATTERN_SEARCHER_BORROW_STATE", "STORED_TRANSITION", "Pattern::into_searcher produces operation-local searcher state containing live borrow leaves. Every leaf follows the Pattern result-provenance relation, remains live through replacement matching, and never derives from the temporary Pattern receiver or call frame.", requires_result_relation=True),
        branch("TRAIT-INTOITER-01", "OWNING_HASH_BUILDER_DROP", "STORED_TRANSITION", "Only owning HashMap and HashSet IntoIterator entrances destroy stored BuildHasher state containing live borrow leaves exactly once before returning the payload cursor; no owning cursor retains S. Borrowed entrances preserve the source owner and do not destroy S."),
        branch("TRAIT-CMP-01", "CALLER_HASHER_BORROW_STATE", "RETAINED_STATE", "Only the Hash implementation branch call-scoped-reborrows caller-owned mutable Hasher H. The same H owner remains valid after every normal Hasher call, but a call may end, replace, or move H's internal borrow leaves only under the declared Hasher behavior-effect relation. Every surviving leaf keeps its exact root, every newly installed leaf follows that relation, and an ended unique leaf is never simultaneously live with its replacement. H grants no payload, relation-result, occupancy, uniqueness, or check-elision authority."),

        # Entry handles distinguish owned candidate payload from cursor-only state.
        branch("MAP-ENTRY-01", "VACANT_GUARD_CANDIDATE_KEY", "RETAINED_STATE", "BTreeMap::entry or HashMap::entry returns a vacant guard owning candidate K with live borrow leaves; the guard later transfers or destroys K exactly once.", returned_state=True),
        branch("MAP-ENTRY-01", "OCCUPIED_ENTRY_CANDIDATE_DROP", "STORED_TRANSITION", "entry(K) finds an occupied slot, preserves the canonical stored key, and destroys candidate K containing live borrow leaves."),
        branch("MAP-ENTRY-01", "AND_MODIFY_STORED_VALUE", "STORED_TRANSITION", "Occupied Entry::and_modify reborrows and may mutate stored V containing live borrow leaves, then returns the guard with valid storage relations."),
        branch("MAP-ENTRY-01", "AND_MODIFY_CALLABLE_ENV", "STORED_TRANSITION", "The owned modifier environment contains live borrow leaves; it is invoked only for occupied entry state and otherwise is destroyed unused exactly once."),
        branch("MAP-ENTRY-01", "OR_INSERT_VACANT_STORE_KEY_VALUE", "STORED_TRANSITION", "Vacant Entry::or_insert stores candidate K and offered V, and either stored role contains live borrow leaves."),
        branch("MAP-ENTRY-01", "OR_INSERT_OCCUPIED_DROP_OFFERED_VALUE", "STORED_TRANSITION", "Occupied Entry::or_insert returns the stored value borrow and destroys offered V containing live borrow leaves."),
        branch("MAP-ENTRY-01", "OR_INSERT_WITH_VACANT_STORE_KEY_VALUE", "STORED_TRANSITION", "Vacant Entry::or_insert_with stores candidate K and produced V, and either stored role contains live borrow leaves. Every produced V leaf follows the callable result-provenance relation and never derives from a temporary guard, map storage, callable receiver, or call frame.", requires_result_relation=True),
        branch("MAP-ENTRY-01", "OR_INSERT_WITH_CALLABLE_ENV", "STORED_TRANSITION", "The owned producer environment contains live borrow leaves; the vacant branch invokes it once and the occupied branch destroys it unused exactly once."),
        branch("MAP-ENTRY-01", "OR_INSERT_WITH_KEY_VACANT_STORE_KEY_VALUE", "STORED_TRANSITION", "Vacant Entry::or_insert_with_key stores candidate K and produced V, including a V that contains a borrow leaf already retained by K. Every produced V leaf follows the callable result-provenance relation and may select an independently valid candidate-K leaf root, but never derives from a temporary key reborrow, guard, map storage, callable receiver, or call frame.", requires_result_relation=True),
        branch("MAP-ENTRY-01", "OR_INSERT_WITH_KEY_CALLABLE_ENV", "STORED_TRANSITION", "The owned keyed-producer environment contains live borrow leaves; the vacant branch invokes it once and the occupied branch destroys it unused exactly once."),
        branch("MAP-ENTRY-01", "OR_DEFAULT_VACANT_STORE_KEY_VALUE", "STORED_TRANSITION", "Vacant Entry::or_default stores candidate K and default-produced V, and either stored role contains live borrow leaves. Every produced V leaf follows the Default result-provenance relation and never derives from the guard, candidate key storage, map storage, receiver, or call frame.", requires_result_relation=True),
        branch("MAP-ENTRY-01", "INSERT_ENTRY_STORE_OR_REPLACE", "STORED_TRANSITION", "Entry::insert_entry stores offered V; the vacant branch also stores candidate K, while the occupied branch destroys the displaced V exactly once."),
        branch("MAP-ENTRY-01", "HASH_ENTRY_STORED_HASHER_STATE", "RETAINED_STATE", "Only the HashMap entry path invokes stored BuildHasher S. The same S owner remains map-owned and valid, but build_hasher may end, replace, or move internal borrow leaves only under the declared BuildHasher behavior-effect relation. Every surviving S leaf keeps its exact root and every newly installed S leaf follows that relation. A unique leaf transferred from S into generated H ends in S before it becomes live in H and is never simultaneously live in both owners; neither old nor new S leaf state grants payload or result provenance."),
        branch("MAP-ENTRY-01", "EXIT_RECOVERABLE_FAILURE", "BORROW_BEARING_RESULT", "Normalized recoverable precommit failure returns candidate K from hash-entry pre-reserve, or candidate K plus any already-offered or produced V from vacant B-tree insertion, and at least one returned owner contains borrow leaves.", returned_state=True),
        branch("MAP-OCCUPIED-01", "INSERT_STORE_OFFERED_VALUE", "STORED_TRANSITION", "OccupiedEntry::insert stores offered V containing live borrow leaves."),
        branch("MAP-OCCUPIED-01", "INSERT_RETURN_DISPLACED_VALUE", "BORROW_BEARING_RESULT", "OccupiedEntry::insert returns the displaced V containing live borrow leaves while the offered V remains valid in storage.", returned_state=True),
        branch("MAP-OCCUPIED-01", "REMOVE_DROP_KEY", "STORED_TRANSITION", "OccupiedEntry::remove destroys removed K containing live borrow leaves while returning V; B-tree repair preserves every surviving entry."),
        branch("MAP-OCCUPIED-01", "REMOVE_VALUE_RESULT", "BORROW_BEARING_RESULT", "OccupiedEntry::remove returns V containing live borrow leaves.", returned_state=True),
        branch("MAP-OCCUPIED-01", "REMOVE_ENTRY_PAIR_RESULT", "BORROW_BEARING_RESULT", "OccupiedEntry::remove_entry returns (K, V), with either live role containing borrow leaves.", returned_state=True),
        branch("MAP-OCCUPIED-01", "HASH_GUARD_MAP_HASHER_STATE", "RETAINED_STATE", "Only the HashMap occupied-guard path preserves the map's stored BuildHasher state containing live borrow leaves across guard access, replacement, removal, or abandonment."),
        branch("MAP-VACANT-01", "MEMBER_INTO_KEY", "BORROW_BEARING_RESULT", "VacantEntry::into_key returns sole candidate K containing borrow leaves.", returned_state=True),
        branch("MAP-VACANT-01", "MEMBER_INSERT", "STORED_TRANSITION", "VacantEntry::insert transfers candidate K and offered V into map storage and either contains borrow leaves; B-tree split or relocation preserves them."),
        branch("MAP-VACANT-01", "MEMBER_INSERT_ENTRY", "STORED_TRANSITION", "VacantEntry::insert_entry transfers candidate K and offered V into map storage and either contains borrow leaves; the returned occupied guard remains cursor-only."),
        branch("MAP-VACANT-01", "EXIT_PAYLOAD_DROP", "STORED_TRANSITION", "Abandoning a vacant guard destroys its retained candidate K and K contains borrow leaves."),
        branch("MAP-VACANT-01", "EXIT_RECOVERABLE_FAILURE", "BORROW_BEARING_RESULT", "Recoverable vacant insertion failure consumes the guard and returns sole candidate K plus offered V, and either returned owner contains borrow leaves.", returned_state=True),
        branch("MAP-VACANT-01", "HASH_GUARD_MAP_HASHER_STATE", "RETAINED_STATE", "Only the HashMap vacant-guard path preserves the map's stored BuildHasher state containing live borrow leaves across insertion, candidate recovery, failure, or abandonment."),
    ]
    for contract_id in ("OSET-INSERT-01", "HSET-INSERT-01"):
        member_rows.extend(
            [
                branch(contract_id, "INSERT_NOVEL_STORE", "STORED_TRANSITION", "Novel insert stores offered T containing live borrow leaves; any split or rehash preserves every surviving payload."),
                branch(contract_id, "INSERT_DUPLICATE_DROP_OFFERED", "STORED_TRANSITION", "Duplicate insert returns false, preserves the canonical stored representative, and destroys offered T containing live borrow leaves."),
                branch(contract_id, "REPLACE_ABSENT_STORE", "STORED_TRANSITION", "replace finds no equivalent element, returns None, and stores offered T containing live borrow leaves."),
                branch(contract_id, "REPLACE_PRESENT_STORE_OFFERED", "STORED_TRANSITION", "replace finds an equivalent element and stores offered T containing live borrow leaves."),
                branch(contract_id, "REPLACE_PRESENT_RETURN_DISPLACED", "BORROW_BEARING_RESULT", "replace finds an equivalent element and returns displaced T containing live borrow leaves.", returned_state=True),
                branch(contract_id, "EXIT_RECOVERABLE_FAILURE", "BORROW_BEARING_RESULT", "Normalized recoverable precommit failure from insert or absent-element replace returns offered T containing borrow leaves with the set unchanged.", returned_state=True),
            ]
        )

    for contract_id in ("OMAP-INSERT-01", "HMAP-INSERT-01"):
        member_rows.extend(
            [
                branch(contract_id, "ABSENT_STORE_KEY_VALUE", "STORED_TRANSITION", "Absent insert stores offered K and V; either live stored role contains borrow leaves."),
                branch(contract_id, "OCCUPIED_DROP_OFFERED_KEY", "STORED_TRANSITION", "An equivalent key exists, the canonical stored K survives, and offered K containing live borrow leaves is destroyed."),
                branch(contract_id, "OCCUPIED_STORE_OFFERED_VALUE", "STORED_TRANSITION", "An equivalent key exists and offered V containing live borrow leaves replaces the old V."),
                branch(contract_id, "OCCUPIED_RETURN_DISPLACED_VALUE", "BORROW_BEARING_RESULT", "An equivalent key exists and displaced V containing live borrow leaves is returned while offered V remains valid in storage.", returned_state=True),
                branch(contract_id, "EXIT_RECOVERABLE_FAILURE", "BORROW_BEARING_RESULT", "Recoverable precommit capacity or allocation failure returns offered K and V as the sole owners, with the map unchanged; either returned role contains live borrow leaves.", returned_state=True),
            ]
        )

    for contract_id in ("OMAP-REMOVE-01", "HMAP-REMOVE-01"):
        member_rows.extend(
            [
                branch(contract_id, "REMOVE_DROP_KEY", "STORED_TRANSITION", "remove destroys removed K containing live borrow leaves while returning only V; every surviving entry remains valid through repair or relocation."),
                branch(contract_id, "REMOVE_VALUE_RESULT", "BORROW_BEARING_RESULT", "remove returns V containing live borrow leaves.", returned_state=True),
                branch(contract_id, "REMOVE_ENTRY_PAIR_RESULT", "BORROW_BEARING_RESULT", "remove_entry returns (K, V), and either returned live role contains borrow leaves.", returned_state=True),
            ]
        )
    member_rows.append(
        branch("OMAP-REMOVE-01", "POP_ENDPOINT_PAIR_RESULT", "BORROW_BEARING_RESULT", "pop_first or pop_last returns an endpoint (K, V) pair, and either returned live role contains borrow leaves.", returned_state=True)
    )

    for contract_id in ("OMAP-ITER-01", "HMAP-ITER-01"):
        member_rows.extend(
            [
                branch(contract_id, "OWNING_INTOITER_PAIR_RESULT", "BORROW_BEARING_RESULT", "Owning IntoIter yields (K, V), and either yielded role contains live borrow leaves.", returned_state=True),
                branch(contract_id, "OWNING_INTO_KEYS_RESULT", "BORROW_BEARING_RESULT", "IntoKeys yields K containing live borrow leaves.", returned_state=True),
                branch(contract_id, "OWNING_INTO_KEYS_OMITTED_VALUE_DROP", "STORED_TRANSITION", "IntoKeys destroys every omitted V containing live borrow leaves exactly once."),
                branch(contract_id, "OWNING_INTO_VALUES_RESULT", "BORROW_BEARING_RESULT", "IntoValues yields V containing live borrow leaves.", returned_state=True),
                branch(contract_id, "OWNING_INTO_VALUES_OMITTED_KEY_DROP", "STORED_TRANSITION", "IntoValues destroys every omitted K containing live borrow leaves exactly once."),
            ]
        )
    member_rows.append(
        branch("HMAP-ITER-01", "OWNING_HASH_BUILDER_DROP", "STORED_TRANSITION", "Consuming HashMap into any owning iterator destroys stored BuildHasher state containing live borrow leaves exactly once before returning the iterator cursor; no owning cursor retains S.")
    )
    member_rows.append(
        branch("HSET-ITER-01", "OWNING_HASH_BUILDER_DROP", "STORED_TRANSITION", "Consuming HashSet into its owning iterator destroys stored BuildHasher state containing live borrow leaves exactly once before returning the iterator cursor; no owning cursor retains S.")
    )

    member_rows.extend(
        [
            branch("OSET-REMOVE-01", "REMOVE_DROP_STORED", "STORED_TRANSITION", "remove returns bool and destroys the matched stored T containing live borrow leaves."),
            branch("OSET-REMOVE-01", "TAKE_RESULT", "BORROW_BEARING_RESULT", "take returns the matched T containing live borrow leaves.", returned_state=True),
            branch("OSET-REMOVE-01", "POP_ENDPOINT_RESULT", "BORROW_BEARING_RESULT", "pop_first or pop_last returns endpoint T containing live borrow leaves.", returned_state=True),
            branch("HSET-REMOVE-01", "REMOVE_DROP_STORED", "STORED_TRANSITION", "remove returns bool and destroys the matched stored T containing live borrow leaves."),
            branch("HSET-REMOVE-01", "TAKE_RESULT", "BORROW_BEARING_RESULT", "take returns the matched T containing live borrow leaves.", returned_state=True),
        ]
    )

    for contract_id, member_scope in sorted(CALLABLE_STATE_MEMBERS.items()):
        if contract_id == "ARR-MAP-01":
            continue
        if contract_id == "RANGE-BOUND-MAP-01":
            condition = (
                "Bound::map owns a FnOnce environment containing live borrow leaves; "
                "Included and Excluded invoke it exactly once, Unbounded invokes it zero "
                "times, and every normal route destroys the environment exactly once."
            )
        elif contract_id == "RC-CYCLIC-01":
            condition = (
                "Rc::new_cyclic owns a FnOnce environment containing live borrow leaves, "
                "invokes it exactly once on every normal route, and destroys it exactly once."
            )
        elif contract_id == "REF-GUARD-01":
            condition = (
                "Each behavior-taking Ref or RefMut projection member owns an environment "
                "containing live borrow leaves, invokes it exactly once on every normal route, "
                "and destroys it exactly once; Ref::clone has no environment."
            )
        elif contract_id == "REFCELL-REPLACE-01":
            condition = (
                "RefCell::replace_with owns an environment containing live borrow leaves, "
                "invokes it exactly once on every normal route, and destroys it exactly once; "
                "replace, take, and swap have no environment."
            )
        else:
            condition = (
                f"For {member_scope}, the owned behavior environment contains live borrow "
                "leaves, remains valid for every permitted zero-or-more invocation path, and "
                "is destroyed exactly once including a not-invoked branch."
            )
        member_rows.append(
            branch(
                contract_id,
                "CALLABLE_ENV_BORROW_STATE",
                "STORED_TRANSITION",
                condition,
            )
        )
    for contract_id, member_scope in sorted(KEY_RESULT_STATE_MEMBERS.items()):
        member_rows.append(
            branch(
                contract_id,
                "KEY_RESULT_BORROW_STATE",
                "STORED_TRANSITION",
                f"{member_scope} contains live borrow leaves; each produced key remains valid through every comparison and is destroyed exactly once.",
                requires_result_relation=True,
            )
        )
    member_rows.append(
        branch("VIEW-SORT-01", "CACHED_KEY_BORROW_STATE", "STORED_TRANSITION", "sort_by_cached_key retains a key array whose live key values contain borrow leaves; every cached key follows the callable result-provenance relation, remains valid through sorting, and is destroyed exactly once.", requires_result_relation=True)
    )

    for contract_id, member_name in sorted(RANGE_BOUNDS_STATE_MEMBERS.items()):
        member_rows.append(
            branch(
                contract_id,
                "BORROW_BEARING_RANGE_DESCRIPTOR",
                "STORED_TRANSITION",
                f"{member_name} consumes a user-defined RangeBounds descriptor R containing live borrow leaves and calls RangeBounds only while R is live. The same R outer owner remains valid across each normal call, but a shared-receiver call may end, replace, or move R's internal leaves only under the declared RangeBounds behavior-effect relation. Surviving leaves keep roots, ended leaves end once, new leaves are relation-authorized, and any unique transfer ends in R before destination liveness; temporary &R, field addresses, and call frames mint no root. Each later call consumes the preceding post-R state. R is destroyed exactly once; no returned cursor or payload provenance derives from R.",
            )
        )

    for contract_id in sorted(HASH_STATE_CONTRACTS):
        s_behavior_effect = (
            "The same stored BuildHasher S owner remains valid, but each build_hasher call "
            "may end, replace, or move S's internal borrow leaves only under the declared "
            "BuildHasher behavior-effect relation. Every surviving S leaf keeps its exact "
            "root; every newly installed S leaf follows that relation. A unique leaf "
            "transferred from S into generated H ends in S before it becomes live in H and "
            "is never simultaneously live in both owners. "
        )
        if contract_id == "SET-REL-01":
            condition = (
                s_behavior_effect
                + "Only the HashSet relation branch invokes one or both sets' stored "
                "BuildHasher states; neither owner or its evolving leaf state grants payload "
                "or Boolean-result authority."
            )
        elif contract_id == "SET-ALG-02":
            condition = (
                s_behavior_effect
                + "The lazy HashSet algebra cursor retains its source-set borrows and invokes "
                "stored BuildHasher state on demand; S remains source-owned and grants no "
                "yielded-payload provenance."
            )
        elif contract_id == "TRAIT-INDEX-01":
            condition = (
                s_behavior_effect
                + "Only the HashMap Index branch invokes stored BuildHasher S; the returned "
                "value borrow derives from map storage, never from S."
            )
        elif contract_id == "TRAIT-CMP-01":
            condition = (
                s_behavior_effect
                + "Only HashMap and HashSet equality branches iterate the left operand and invoke "
                "the right operand's stored BuildHasher S through get or contains probes; the "
                "left S remains retained and unreborrowed. Length-mismatch and empty-equal paths "
                "perform zero build_hasher calls. Each performed right-hand probe creates exactly "
                "one generated H. Other comparison and caller-Hasher Hash branches are excluded. "
                "Neither S nor its evolving leaves grant relation-result or payload authority."
            )
        elif contract_id in {"HMAP-ITER-01", "HSET-ITER-01"}:
            condition = (
                "Only borrowed hash iterators preserve source-owned BuildHasher S containing "
                "live borrow leaves; S is not invoked, moved, or destroyed and is never stored "
                "in the returned cursor. The owning-iterator branch destroys S before returning "
                "its cursor state."
            )
        elif contract_id in {"HMAP-DRAIN-01", "HSET-DRAIN-01"}:
            condition = (
                "Drain and clear preserve source-owned BuildHasher S containing live borrow "
                "leaves without invoking, moving, or destroying it; no returned drain cursor "
                "retains S or derives payload, allocation, or repair authority from S."
            )
        elif contract_id in HASH_BEHAVIOR_STATE_CONTRACTS:
            condition = (
                s_behavior_effect
                + "The operation invokes stored BuildHasher state containing live borrow "
                "leaves; the S owner remains valid through the operation and its exact eventual "
                "destruction, and neither old nor new S leaf state grants payload or result "
                "provenance."
            )
        else:
            raise ValueError(f"unclassified hash-state lifecycle: {contract_id}")
        member_rows.append(
            branch(
                contract_id,
                "STORED_HASH_BUILDER_BORROW_STATE",
                "RETAINED_STATE",
                condition,
                returned_state=contract_id == "SET-ALG-02",
            )
        )

    for contract_id in sorted(GENERATED_HASHER_RESULT_CONTRACTS):
        operation_scope = (
            "Only the HashMap and HashSet equality branches iterate the left operand and invoke "
            "right-hand stored S through get or contains; left-hand S remains retained and "
            "unreborrowed. Length-mismatch and empty-equal paths create zero H owners, and each "
            "performed right-hand probe creates exactly one H. The Hash implementation branch "
            "instead uses caller-owned H without BuildHasher, and all other comparison branches "
            "use neither. "
            if contract_id == "TRAIT-CMP-01"
            else ""
        )
        member_rows.append(
            branch(
                contract_id,
                "GENERATED_HASHER_BORROW_STATE",
                "STORED_TRANSITION",
                operation_scope
                + "Each BuildHasher::build_hasher call returns exactly one call-local H owner "
                "whose initial borrow leaves and the post-call leaves retained by the same valid "
                "S owner jointly follow the declared BuildHasher result and behavior-effect "
                "relations. Every surviving S leaf keeps its exact root; each newly installed "
                "S leaf and each initial H leaf follows those relations. A unique leaf moved "
                "from S into H ends in S before becoming live in H and is never simultaneously "
                "live in both. No H leaf derives from the call-scoped &S receiver, the address "
                "or storage of an S field, or the call frame, although an existing leaf value "
                "stored in S may transfer with its independent external root. The same H owner "
                "remains valid across the required Hash::hash and Hasher calls; those calls may "
                "evolve H's internal leaves only under the declared Hasher behavior-effect "
                "relation, with surviving roots preserved and ended unique leaves not duplicated. "
                "H is accessed only by call-scoped reborrows and is destroyed exactly once before "
                "the operation or lazy cursor step completes. H or S provenance never becomes payload or "
                "public result-borrow provenance. Hash output may influence the logical probe, "
                "destination, or Boolean result, but alone mints no occupancy, liveness, "
                "uniqueness, or check-elision fact; occupancy metadata gates payload access "
                "and Eq gates equivalence.",
                requires_result_relation=True,
            )
        )
    rows.extend(member_rows)
    return rows


def read_tsv(path: Path) -> tuple[list[str], list[dict[str, str]]]:
    with path.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        if reader.fieldnames is None:
            raise ValueError(f"{path}: missing header")
        return reader.fieldnames, list(reader)


def render_tsv(header: list[str], rows: list[dict[str, str]]) -> str:
    output = io.StringIO(newline="")
    writer = csv.DictWriter(output, fieldnames=header, delimiter="\t", lineterminator="\n")
    writer.writeheader()
    writer.writerows(rows)
    return output.getvalue()


def build() -> tuple[str, str]:
    _, census_rows = read_tsv(CENSUS)
    _, matrix_rows = read_tsv(MATRIX)
    _, registry_rows = read_tsv(REGISTRY)
    contract_ids = [row["contract_id"] for row in census_rows]
    if contract_ids != [row["contract_id"] for row in matrix_rows]:
        raise ValueError("census and matrix contract order differs")
    contract_set = set(contract_ids)
    if UNRESOLVED:
        missing = sorted(UNRESOLVED - contract_set)
        if missing:
            raise ValueError(f"unknown unresolved contracts: {missing}")
        raise ValueError(
            "payload-scope source adjudication remains unresolved: "
            + ",".join(sorted(UNRESOLVED))
        )

    matrix = {row["contract_id"]: row for row in matrix_rows}
    registry_rank = {
        row["capability_id"]: index for index, row in enumerate(registry_rows)
    }
    branches = explicit_branches()
    keys = [(item.contract_id, item.branch_id) for item in branches]
    if len(keys) != len(set(keys)):
        raise ValueError("duplicate payload-scope overlay branch key")
    unknown = sorted({item.contract_id for item in branches} - contract_set)
    if unknown:
        raise ValueError(f"unknown overlay contracts: {unknown}")
    overlap = sorted({item.contract_id for item in branches} & ACTIVE_BR_STORED)
    if overlap:
        raise ValueError(f"active BR-STORED rows duplicated in overlay: {overlap}")

    overlay_rows: list[dict[str, str]] = []
    deferred_contracts: set[str] = set()
    for item in sorted(branches, key=lambda value: (contract_ids.index(value.contract_id), value.branch_id)):
        if item.role not in {"STORED_TRANSITION", "BORROW_BEARING_RESULT", "RETAINED_STATE"}:
            raise ValueError(f"{item.contract_id}/{item.branch_id}: unknown role {item.role}")
        desired = {"BR-PROV", "BR-STORED"}
        if (
            item.role == "BORROW_BEARING_RESULT"
            or item.returned_state
            or item.requires_result_relation
        ):
            desired.add("BR-RESULT")
        base = set(matrix[item.contract_id]["capability_ids"].split(","))
        conditional = sorted(desired - base, key=registry_rank.__getitem__)
        if not conditional:
            raise ValueError(
                f"{item.contract_id}/{item.branch_id}: conditional capability difference is empty"
            )
        deferred_contracts.add(item.contract_id)
        overlay_rows.append(
            {
                "contract_id": item.contract_id,
                "branch_id": item.branch_id,
                "route_scope": ROUTE_SCOPE,
                "role": item.role,
                "returns_borrow_bearing_owner": "yes" if item.returned_state else "no",
                "requires_result_provenance": (
                    "yes"
                    if item.role == "BORROW_BEARING_RESULT"
                    or item.returned_state
                    or item.requires_result_relation
                    else "no"
                ),
                "condition": item.condition,
                "conditional_capability_ids": ",".join(conditional),
                "disposition": DISPOSITION,
                "reopening_trigger": REOPENING_TRIGGER,
            }
        )

    actual_br_stored = {
        row["contract_id"]
        for row in matrix_rows
        if "BR-STORED" in row["capability_ids"].split(",")
    }
    if actual_br_stored != ACTIVE_BR_STORED:
        raise ValueError(
            "ACTIVE_BR_STORED differs from matrix: "
            f"missing={sorted(actual_br_stored - ACTIVE_BR_STORED)}, "
            f"extra={sorted(ACTIVE_BR_STORED - actual_br_stored)}"
        )

    classification_rows: list[dict[str, str]] = []
    for contract_id in contract_ids:
        status = matrix[contract_id]["status_code"]
        scope_owner_contract_ids = "NONE"
        if contract_id in ACTIVE_BR_STORED:
            classification = "ACTIVE_BR_STORED"
            rationale = (
                "The base matrix row explicitly covers arbitrary borrow-bearing retained "
                "values under BR-STORED and its pinned provenance, result, destruction, and "
                "disjointness obligations as applicable."
            )
        elif contract_id in deferred_contracts:
            classification = "DEFERRED_BRANCHES"
            rationale = (
                "One or more exact region-free/borrow-free base branches and their "
                "stored-borrow complements are recorded in PAYLOAD-SCOPE-OVERLAY.tsv."
            )
        elif contract_id == "ALLOC-ERROR-01":
            classification = "DELEGATED_TO_FAMILY_BRANCHES"
            scope_owner_contract_ids = (
                "SEQ-TRY-RESERVE-01;DEQUE-RESERVE-01;HEAP-RESERVE-01;"
                "HMAP-RESERVE-01;HSET-RESERVE-01;STRING-RESERVE-01"
            )
            rationale = (
                "This cross-cutting error contract adds no independent payload operation; "
                "SEQ-TRY-RESERVE-01, DEQUE-RESERVE-01, HEAP-RESERVE-01, HMAP-RESERVE-01, "
                "HSET-RESERVE-01, and the region-free STRING-RESERVE-01 row own the exact "
                "success-relocation and failure-preservation payload branches."
            )
        elif status == "BOUNDARY":
            classification = "BOUNDARY_EVIDENCE_ONLY"
            scope_owner_contract_ids = contract_id
            rationale = (
                "This raw or unchecked spelling is inadmissible boundary evidence; its "
                "underlying checked need and payload scope are routed through separate safe "
                "contracts, so this evidence row receives no derivation complement."
            )
        elif contract_id in {"RAW-SAFE-OWNERSHIP-01", "RAW-UNSAFE-RECONSTRUCT-01"}:
            classification = "FRAME_SCOPE_DEFERRED"
            scope_owner_contract_ids = contract_id
            rationale = (
                "This trusted-frame operation transfers or reconstructs a live generic owner; "
                "its stored-borrow scope is owned by separate frame/ABI authority, receives no "
                "ordinary derivation complement, and cannot close a safe-library capability route."
            )
        else:
            classification = "NO_STORED_BORROW_COMPLEMENT"
            base_capabilities = set(matrix[contract_id]["capability_ids"].split(","))
            if contract_id == "RC-INIT-01":
                rationale = (
                    "The returned Rc owns MaybeUninit<T> storage but no live T; no borrow leaf "
                    "exists until a separately locked exact fill/seal transition establishes T."
                )
            elif contract_id in SEALED_STABLE_STEP_RANGE_CONTRACTS:
                rationale = SEALED_STABLE_STEP_RATIONALE
            elif "BR-RESULT" in base_capabilities:
                rationale = (
                    "The base matrix flags every direct borrowed or borrow-bearing result "
                    "through BR-PROV/BR-RESULT obligations, and this coverage cluster retains no "
                    "additional arbitrary borrow-bearing value as data across public calls."
                )
            else:
                rationale = (
                    "This coverage cluster has no live borrow-bearing generic payload branch "
                    "that requires a stored-borrow complement."
                )
        classification_rows.append(
            {
                "contract_id": contract_id,
                "stored_borrow_scope": classification,
                "scope_owner_contract_ids": scope_owner_contract_ids,
                "rationale": rationale,
            }
        )

    return (
        render_tsv(CLASSIFICATION_HEADER, classification_rows),
        render_tsv(OVERLAY_HEADER, overlay_rows),
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="fail if checked-in outputs differ")
    args = parser.parse_args()
    classification, overlay = build()
    outputs = ((CLASSIFICATION, classification), (OVERLAY, overlay))
    if args.check:
        mismatches = [str(path) for path, expected in outputs if not path.exists() or path.read_text(encoding="utf-8") != expected]
        if mismatches:
            raise SystemExit("payload-scope outputs are stale: " + ", ".join(mismatches))
    else:
        for path, content in outputs:
            path.write_text(content, encoding="utf-8")
    print("payload-scope overlay: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
