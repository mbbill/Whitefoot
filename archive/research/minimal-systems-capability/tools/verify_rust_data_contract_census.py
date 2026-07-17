#!/usr/bin/env python3
"""Verify exact Rust seed-to-cluster coverage and the D11 scope boundary."""

from __future__ import annotations

import collections
import csv
import hashlib
import pathlib
import re
import subprocess

from build_d10_surface_map import FIELDS as D10_MAP_FIELDS
from build_d10_surface_map import OPS_RANGE_PATHS
from build_d10_surface_map import build_rows as build_d10_rows
from build_rust_data_unsafe_evidence_map import FIELDS as UNSAFE_MAP_FIELDS
from build_rust_data_unsafe_evidence_map import build_rows as build_unsafe_map_rows


ROOT = pathlib.Path(__file__).resolve().parent.parent
INVENTORY = ROOT / "RUST-1.97.0-API-INVENTORY.tsv"
CONTRACTS = ROOT / "RUST-DATA-CONTRACT-CENSUS.tsv"
CENSUS_DOC = ROOT / "RUST-DATA-CONTRACT-CENSUS.md"
SURFACE_MAP = ROOT / "RUST-DATA-SURFACE-MAP.tsv"
UNSAFE_EVIDENCE_MAP = ROOT / "RUST-DATA-UNSAFE-EVIDENCE-MAP.tsv"
D10_SURFACE_MAP = ROOT / "RUST-D10-SURFACE-MAP.tsv"
TRAIT_IMPL_CROSSWALK = ROOT / "RUST-1.97.0-TRAIT-IMPL-CROSSWALK.tsv"

RUST_197_COMMIT = "2d8144b7880597b6e6d3dfd63a9a9efae3f533d3"
RANGE_SOURCE_HASHES = {
    "iter/range.rs": "ae3f9307f4b4972f418561ae2a0311586eb3dde782359b8aaef3244915256464",
    "range.rs": "454a3cd2eee96dd54f7803615a253d34ef43ebde5feecb97676779bc4709707e",
    "range/iter.rs": "5a66a8cdec1770047bb8a783bfe336dbfe9aa20dad613c230438cc6a9c0a2f59",
}
SEALED_STABLE_STEP_TYPE_LIST = (
    "u8,u16,u32,u64,u128,usize,i8,i16,i32,i64,i128,isize,char,"
    "NonZero<u8>,NonZero<u16>,NonZero<u32>,NonZero<u64>,NonZero<u128>,"
    "NonZero<usize>,Ipv4Addr,Ipv6Addr"
)

CONTRACT_FIELDS = [
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

MAP_FIELDS = [
    "canonical_key",
    "item_path",
    "member_name",
    "source_path",
    "primary_contract_id",
    "markers",
]

SEEDS = {
    "std::array": "array",
    "std::slice": "slice",
    "std::str": "str",
    "alloc::boxed::Box": "Box",
    "alloc::vec::Vec": "Vec",
    "alloc::collections::vec_deque::VecDeque": "VecDeque",
    "alloc::collections::linked_list::LinkedList": "LinkedList",
    "alloc::collections::binary_heap::BinaryHeap": "BinaryHeap",
    "alloc::collections::btree_map::BTreeMap": "BTreeMap",
    "alloc::collections::btree_set::BTreeSet": "BTreeSet",
    "std::collections::hash_map::HashMap": "HashMap",
    "std::collections::hash_set::HashSet": "HashSet",
    "alloc::string::String": "String",
    "alloc::rc::Rc": "Rc",
    "alloc::rc::Weak": "Weak",
    "core::cell::RefCell": "RefCell",
}

ALLOWED_MARKERS = {
    "stable_safe_seed",
    "raw_obligation",
    "one_hop_protocol",
    "one_hop_helper_return",
    "allocation_cross_cut",
    "initialization_cross_cut",
}

ONE_HOP_RETURN_NAMES = set(
    """
    GetDisjointMutError ArrayWindows ChunkBy ChunkByMut Chunks ChunksExact
    ChunksExactMut ChunksMut EscapeAscii Iter IterMut RChunks RChunksExact
    RChunksExactMut RChunksMut RSplit RSplitMut RSplitN RSplitNMut Split
    SplitInclusive SplitInclusiveMut SplitMut SplitN SplitNMut Windows
    Utf8Chunks Bytes CharIndices Chars EncodeUtf16 EscapeDebug EscapeDefault
    EscapeUnicode Lines LinesAny MatchIndices Matches RMatchIndices RMatches
    RSplitTerminator SplitAsciiWhitespace SplitTerminator SplitWhitespace
    Utf8Error IntoIter Drain ExtractIf Splice PeekMut Keys Values ValuesMut
    IntoKeys IntoValues Range RangeMut Entry OccupiedEntry VacantEntry
    Difference SymmetricDifference Intersection Union FromUtf8Error
    FromUtf16Error Ref RefMut BorrowError BorrowMutError TryReserveError
    """.split()
)

ONE_HOP_HELPER_PARTITION = {
    "slice": (26, 25),
    "str": (24, 23),
    "array": (1, 1),
    "sequence_and_heap": (16, 15),
    "ordered": (22, 19),
    "unordered": (21, 18),
    "string": (3, 1),
    "refcell": (4, 0),
    "allocation": (1, 0),
}

ONE_HOP_NON_ITERATOR_HELPERS = {
    "slice::GetDisjointMutError",
    "str::Utf8Error",
    "binary_heap::PeekMut",
    "btree_map::Entry",
    "btree_map::OccupiedEntry",
    "btree_map::VacantEntry",
    "hash_map::Entry",
    "hash_map::OccupiedEntry",
    "hash_map::VacantEntry",
    "string::FromUtf8Error",
    "string::FromUtf16Error",
    "cell::Ref",
    "cell::RefMut",
    "cell::BorrowError",
    "cell::BorrowMutError",
    "alloc::TryReserveError",
}

LIFECYCLE_CENSUS_REQUIRED = {
    "RANGE-VALUE-INCLUSIVE-01": (
        "Calling iter clones an independent iterator",
        "strict separation from cursor authority or terminal state",
        "library/core/src/range.rs:235-259",
        "335-356 iter cloning",
    ),
    "ITER-SOURCE-VALUE-01": (
        "After the yield the cursor retains no T",
        "borrow-bearing yielded T remains independently live",
        "terminal None after the sole yield destroys nothing",
    ),
    "ITER-SOURCE-CALLBACK-01": (
        "OnceWith consumes F on its first yield",
        "FromFn may return None and later Some",
        "Successors performs no F call for an absent initial seed",
    ),
    "ITER-ADAPT-CHAIN-01": (
        "permanently retires A",
        "B remains directly pollable and may return Some after a transient None",
    ),
    "ITER-ADAPT-ZIP-01": (
        "first polls A and then B",
        "unpaired A item",
    ),
    "ITER-ADAPT-NEST-01": (
        "Outer traversal is wrapped in Fuse",
        "active inner cursor is permanently retired and destroyed at its first None",
    ),
    "ITER-ADAPT-STATE-01": (
        "Scan has no done bit",
        "Callback None is not termination",
        "consumes that input",
    ),
    "ITER-ADAPT-PEEK-01": (
        "Direct next forwards a transient upstream None without caching it",
        "Peek caches None",
    ),
    "ITER-ADAPT-FUSE-01": (
        "general implementation retires and destroys upstream state",
        "FusedIterator specialization retains upstream state",
    ),
    "SEQ-DRAIN-01": (
        "neither first nor repeated terminal None restores the tail",
        "moves the untouched tail, restores len",
    ),
    "SEQ-EXTRACT-01": (
        "Construction sets len to zero",
        "First or repeated terminal None does not copy the untouched tail",
    ),
    "SEQ-SPLICE-01": (
        "zero replacement calls performed by those terminal calls",
        "restores len through nested Drain cleanup",
    ),
    "DEQUE-DRAIN-01": (
        "First or repeated terminal None does not drop the remainder",
        "fixes head and len",
    ),
    "STRING-DRAIN-01": (
        "construction and every next call leave the source bytes unchanged",
        "removes the entire original byte range regardless of yield progress",
    ),
    "HEAP-DRAIN-01": (
        "valid empty heap allocation",
        "no tail repair",
    ),
    "HMAP-DRAIN-01": (
        "moves the RawTable allocation into the cursor",
        "returns the empty allocation to the base map",
    ),
    "HSET-DRAIN-01": (
        "moves the RawTable allocation into the cursor",
        "returns the empty allocation to the base set",
    ),
    "LIST-EXTRACT-01": (
        "fully unlinked and len-adjusted before yield",
        "ExtractIf has no structural Drop repair",
    ),
    "OMAP-FILTER-01": (
        "First or repeated terminal None retains F and R",
        "no structural Drop repair",
    ),
    "OSET-FILTER-01": (
        "First or repeated terminal None retains F and R",
        "no structural Drop repair",
    ),
    "HMAP-FILTER-01": (
        "updates control bytes, item count, and growth metadata before yield",
        "src/map.rs:182-185,896-908,955-965,2588-2611",
    ),
    "HSET-FILTER-01": (
        "updates control bytes, item count, and growth metadata before yield",
        "src/map.rs:182-185,896-908,955-965,2588-2611",
    ),
    "ITER-SOURCE-REPEAT-01": (
        "repeat_n(seed, 0) drops seed during construction",
        "final yield moves it",
        "post-final None or destruction drops none",
        "repeat_n.rs:59-73,82-90,114-130",
    ),
    "ITER-ADAPT-CYCLE-01": (
        "Construction calls Clone exactly once",
        "Every current-epoch None calls Clone once",
        "replaces and destroys old current once",
        "polls the new clone once",
        "First epoch follows consumed iter",
        "Clone may change state/order",
        "cycle.rs:15-23,34-40,60-80",
    ),
}

HASH_EFFECT_CENSUS_CONTRACTS = {
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

LAST_USE_SPLIT_ROWS = {
    "VIEW-ITER-01", "VIEW-WINDOW-01", "VIEW-CHUNKS-01", "VIEW-CHUNKBY-01",
    "VIEW-SPLIT-PRED-01", "BYTE-ASCII-05", "BYTE-UTF8-CHUNKS-01",
    "TEXT-ITER-01", "TEXT-UTF16-01", "TEXT-MATCH-ITER-01",
    "TEXT-SPLIT-PATTERN-01", "TEXT-LINES-01", "TEXT-ESCAPE-01",
    "DEQUE-RANGE-01", "OMAP-RANGE-01", "SET-ALG-01", "SET-ALG-02",
}
CENTRAL_ALLOCATION_ITER_ROWS = {
    "DEQUE-ITER-01", "HEAP-VIEW-01", "HMAP-ITER-01", "HSET-ITER-01"
}
TOPOLOGY_ITER_ROWS = {"LIST-ITER-01", "OMAP-ITER-01", "OSET-RANGE-01"}


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"rust data-contract census verification failed: {message}")


def read_tsv(path: pathlib.Path) -> tuple[list[str], list[dict[str, str]]]:
    with path.open(encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        fields = list(reader.fieldnames or [])
        rows = list(reader)
    require(all(None not in row for row in rows), f"extra columns in {path.name}")
    return fields, rows


def verify_pinned_range_sources() -> None:
    result = subprocess.run(
        ["rustc", "+1.97.0", "--print", "sysroot"],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )
    require(result.returncode == 0, "rustc +1.97.0 sysroot is unavailable")
    core_source = (
        pathlib.Path(result.stdout.strip())
        / "lib/rustlib/src/rust/library/core/src"
    )
    for relative, expected_hash in RANGE_SOURCE_HASHES.items():
        source = core_source / relative
        require(source.is_file(), f"missing pinned Rust source {relative}")
        actual_hash = hashlib.sha256(source.read_bytes()).hexdigest()
        require(
            actual_hash == expected_hash,
            f"Rust {relative} source identity changed: {actual_hash}",
        )
    step_source = (core_source / "iter/range.rs").read_text(encoding="utf-8")
    for fragment in (
        '#[unstable(feature = "step_trait", issue = "42168")]',
        "unsafe_impl_trusted_step![AsciiChar char i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize Ipv4Addr Ipv6Addr];",
        "unsafe_impl_trusted_step![NonZero<u8> NonZero<u16> NonZero<u32> NonZero<u64> NonZero<u128> NonZero<usize>];",
    ):
        require(fragment in step_source, f"pinned Step source lost {fragment!r}")


def token_present(text: str, token: str) -> bool:
    return re.search(
        rf"(?<![A-Za-z0-9_]){re.escape(token)}(?![A-Za-z0-9_])", text
    ) is not None


def returns_one_hop_helper(signature: str) -> bool:
    if "->" not in signature:
        return False
    result = signature.split("->", 1)[1]
    return any(token_present(result, name) for name in ONE_HOP_RETURN_NAMES)


def selected_inventory(
    rows: list[dict[str, str]], safety: str
) -> dict[str, dict[str, str]]:
    selected: dict[str, dict[str, str]] = {}
    for row in rows:
        if (
            row["item_path"] not in SEEDS
            or row["member_kind"] != "provided_or_inherent_method"
            or row["stability"] != "stable"
            or row["caller_safety"] != safety
        ):
            continue
        selected.setdefault(row["canonical_key"], row)
    return selected


def main() -> None:
    verify_pinned_range_sources()
    inventory_fields, inventory = read_tsv(INVENTORY)
    require("canonical_key" in inventory_fields, "raw inventory schema changed")

    contract_fields, contracts = read_tsv(CONTRACTS)
    require(contract_fields == CONTRACT_FIELDS, "contract TSV schema changed")
    require(len(contracts) == 276, "expected exactly 276 coverage/evidence rows")
    require(
        all(all(row[field] for field in CONTRACT_FIELDS) for row in contracts),
        "empty required contract field",
    )
    contract_ids = [row["contract_id"] for row in contracts]
    require(len(contract_ids) == len(set(contract_ids)), "duplicate contract_id")
    contract_by_id = {row["contract_id"]: row for row in contracts}
    _, trait_impls = read_tsv(TRAIT_IMPL_CROSSWALK)
    require(len(trait_impls) == 334, "expected 334 selected concrete trait implementations")
    trait_family_counts = collections.Counter(row["selection_family"] for row in trait_impls)
    require(
        trait_family_counts
        == {
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
        },
        f"selected concrete trait family counts changed: {dict(trait_family_counts)}",
    )
    trait_evidence = {
        "TRAIT-INTOITER-01": ("INTO_ITERATOR", "exact selected direct impl count=26"),
        "TRAIT-EXTEND-01": ("EXTEND", "exact selected direct impl count=22"),
        "TRAIT-COLLECT-01": ("FROM_ITERATOR", "exact selected direct impl count=21"),
        "TRAIT-INDEX-01": ("INDEX", "Index=8,IndexMut=6"),
        "TRAIT-DEREF-01": ("DEREF", "total selected counts Deref=6,DerefMut=4"),
        "TRAIT-BORROW-01": ("BORROW_PROJECTION", "AsRef=9,AsMut=6,Borrow=5,BorrowMut=4"),
        "TRAIT-CONVERT-01": ("CONVERSION", "From=31,TryFrom=9"),
        "TRAIT-CLONE-01": ("CLONE", "exact selected direct Clone count=16"),
        "TRAIT-DEFAULT-01": ("DEFAULT", "exact selected direct Default count=54"),
        "TRAIT-CMP-01": ("COMPARISON_HASH", "PartialEq=36,Eq=12,PartialOrd=10,Ord=10,Hash=10"),
        "TRAIT-DROP-01": ("DROP", "exact selected direct Drop impl count=7"),
    }
    for contract_id, (family, evidence_fragment) in trait_evidence.items():
        row = contract_by_id[contract_id]
        require(
            f"selection_family={family}" in row["source_refs"],
            f"{contract_id} does not cite its concrete trait implementation family",
        )
        require(
            evidence_fragment in row["implementation_privilege_evidence"],
            f"{contract_id} selected implementation count is stale",
        )
    step_impls = [row for row in trait_impls if row["selection_family"] == "RANGE_STEP"]
    require(
        len(step_impls) == 22
        and sum(row["stable_surface_reachable"] == "YES" for row in step_impls) == 21
        and [row["implementer"] for row in step_impls if row["stable_surface_reachable"] == "NO"] == ["Char"]
        and all(row["ownership_shape"] == "COPY_BORROW_FREE" for row in step_impls),
        "Range Step implementation/reachability split changed",
    )
    for contract_id in (
        "RANGE-ITER-HALFOPEN-01",
        "RANGE-ITER-FROM-01",
        "RANGE-ITER-INCLUSIVE-01",
    ):
        require(
            "selection_family=RANGE_STEP" in contract_by_id[contract_id]["source_refs"],
            f"{contract_id} lacks exact Step implementation evidence",
        )
    census_doc = CENSUS_DOC.read_text(encoding="utf-8")
    normalized_census_doc = re.sub(r"\s+", " ", census_doc)
    for fragment in (
        "Every row has semantic class `G0_COVERAGE_CLUSTER`",
        "it is not an exact operation contract, a Family Lock import, an experiment unit, or evidence for family-level `E` or `P`",
        "Such a union proves nothing about an individual member",
        "assign a stable `member_contract_id` to each distinct member contract and an `outcome_id`",
        "Two declarations may share a member ID only after the lock proves equality across all of those dimensions",
    ):
        require(
            fragment in normalized_census_doc,
            f"census lost the D11 non-importability rule {fragment!r}",
        )
    for stale in (
        "276 normalized contracts",
        "276 normalized rows",
        "Merging is allowed only for aliases or convenience spellings",
    ):
        require(
            stale not in normalized_census_doc,
            f"census retains false exact-normalization claim {stale!r}",
        )
    step_items = [
        row for row in inventory
        if row["item_path"] == "core::iter::Step" and row["member_kind"] == "item"
    ]
    require(
        len(step_items) == 1
        and step_items[0]["stability"] == "unstable"
        and step_items[0]["caller_safety"] == "unsafe",
        "Step is no longer the exact unstable unsafe source boundary",
    )
    ascii_items = [
        row for row in inventory
        if row["item_path"] == "core::ascii::Char" and row["member_kind"] == "item"
    ]
    require(
        len(ascii_items) == 1 and ascii_items[0]["stability"] == "unstable",
        "AsciiChar exclusion is no longer pinned",
    )
    for contract_id in (
        "RANGE-ITER-HALFOPEN-01",
        "RANGE-ITER-FROM-01",
        "RANGE-ITER-INCLUSIVE-01",
    ):
        row_text = " ".join(contract_by_id[contract_id].values())
        for fragment in (
            SEALED_STABLE_STEP_TYPE_LIST,
            "exact stable-callable set is the 21 listed borrow-free Copy types",
            "not because a shared receiver is pure",
            RUST_197_COMMIT,
            RANGE_SOURCE_HASHES["iter/range.rs"],
        ):
            require(fragment in row_text, f"{contract_id} lost sealed Step fragment {fragment!r}")
        require(
            contract_by_id[contract_id]["behavior_parameter"].startswith("None;"),
            f"{contract_id} acquired runtime user behavior",
        )
    for contract_id, fragments in LIFECYCLE_CENSUS_REQUIRED.items():
        row_text = " ".join(contract_by_id[contract_id].values()).lower()
        for fragment in fragments:
            require(
                fragment.lower() in row_text,
                f"{contract_id} lost exact lifecycle fragment {fragment!r}",
            )
    for contract_id in LAST_USE_SPLIT_ROWS:
        row_text = " ".join(contract_by_id[contract_id].values()).lower()
        for fragment in (
            "proven last use may release only pure cursor/source-borrow authority",
            "only when no repair, owned retained-state destruction, or allocation disposition remains",
            "destruction or consuming close performs every pending repair, owned-state destruction, and allocation disposition",
            "terminal none may retire or replace retained substate only under the exact concrete helper transition",
            "separate pure last-use authority release from destruction/close duties",
        ):
            require(fragment in row_text, f"{contract_id} lost last-use split {fragment!r}")
        require(
            "proven last use performs the exact contract-specific cleanup" not in row_text,
            f"{contract_id} still treats proven last use as cleanup",
        )
    for contract_id in CENTRAL_ALLOCATION_ITER_ROWS:
        row_text = " ".join(contract_by_id[contract_id].values()).lower()
        for fragment in (
            "allocation remains cursor-owned until destruction",
            "preserve this family's exact central-allocation retention after terminal none",
            "reject applying it to linkedlist or btree",
        ):
            require(fragment in row_text, f"{contract_id} lost central-allocation split {fragment!r}")
    for contract_id in TOPOLOGY_ITER_ROWS:
        row_text = " ".join(contract_by_id[contract_id].values()).lower()
        family_fragment = (
            "deallocates nodes incrementally"
            if contract_id == "LIST-ITER-01"
            else "family-specific btree node/traversal state"
        )
        require(family_fragment in row_text, f"{contract_id} lost topology disposition")
        require(
            "reject union with contiguous/hash central-allocation retention" in row_text,
            f"{contract_id} lost allocation-union rejection",
        )
    for contract_id in ("TRAIT-ITER-01", "TRAIT-DOUBLE-01"):
        row_text = " ".join(contract_by_id[contract_id].values()).lower()
        for fragment in (
            "proven last use may end only pure borrow authority",
            "pending repair, owned state, or allocation disposition persists",
            "exact subcursor/epoch retirement is separately delegated",
            "trait authorizes neither",
        ):
            require(fragment in row_text, f"{contract_id} lost trait lifecycle split {fragment!r}")
    for contract_id in HASH_EFFECT_CENSUS_CONTRACTS:
        row_text = " ".join(contract_by_id[contract_id].values()).lower()
        for fragment in (
            "same stored buildhasher s owner remains valid",
            "declared buildhasher behavior-effect relation",
            "s's post-call leaves jointly follow",
            "unique leaf transferred from s into h ends in s before becoming live in h",
            "never simultaneously live in both",
            "declared hasher behavior-effect relation",
            "destroyed exactly once with its remaining state",
            "address or storage of an s field",
            "joint post-s/initial-h result provenance",
            "strict s/h fact firewall",
            "library/core/src/hash/mod.rs:258-357,637-656,694-701",
        ):
            require(
                fragment in row_text,
                f"{contract_id} lost BuildHasher/Hasher effect fragment {fragment!r}",
            )
        for stale in (
            "preserve s and that root across the operation",
            "preserve the same s owner and root",
            "each s capture leaf retains its independent external root",
            "every h capture leaf retains its independent external root across writes",
        ):
            require(stale not in row_text, f"{contract_id} retains stale hasher claim {stale!r}")
    cmp_text = " ".join(contract_by_id["TRAIT-CMP-01"].values()).lower()
    for fragment in (
        "only hashmap/hashset equality iterates the left operand",
        "only right-hand s is invoked",
        "left-hand s remains retained and unreborrowed",
        "length mismatch and empty equality perform zero build_hasher calls",
        "each performed right-hand probe creates exactly one generated h",
        "only the hash implementation branch uses caller-owned h and never invokes buildhasher",
        "other comparison branches use neither role",
        "must never be unioned into every comparison",
        "library/std/src/collections/hash/map.rs:1319-1328",
        "set.rs:1027-1036",
        "library/core/src/hash/mod.rs:258-357,637-656,694-701",
    ):
        require(fragment in cmp_text, f"TRAIT-CMP-01 lost hasher partition fragment {fragment!r}")
    require(
        "every h capture leaf retains its independent external root across writes" not in cmp_text,
        "TRAIT-CMP-01 retains stale caller-H leaf-preservation claim",
    )
    helper_total = sum(total for total, _ in ONE_HOP_HELPER_PARTITION.values())
    iterator_total = sum(
        iterator_count for _, iterator_count in ONE_HOP_HELPER_PARTITION.values()
    )
    require(helper_total == 118, "one-hop helper partition must total 118")
    require(iterator_total == 102, "one-hop iterator subset must total 102")
    require(
        helper_total - iterator_total == len(ONE_HOP_NON_ITERATOR_HELPERS) == 16,
        "one-hop non-iterator exclusion set must contain exactly 16 helpers",
    )
    trait_iter = contract_by_id["TRAIT-ITER-01"]
    require(
        trait_iter["rust_surfaces"]
        == "Iterator::{next,size_hint} on the 102 applicable one-hop iterator helpers; the exact 16 non-iterator helpers are excluded in RUST-DATA-CONTRACT-CENSUS.md",
        "TRAIT-ITER-01 helper subset is not the exact 102/16 partition",
    )
    require(
        "mechanically pinned 102 iterator/16 non-iterator helper partition"
        in trait_iter["source_refs"],
        "TRAIT-ITER-01 lacks its exact helper-partition evidence marker",
    )
    census_doc = CENSUS_DOC.read_text(encoding="utf-8")
    require(
        "Exactly **102 of the 118 helper instances implement the applicable iterator\nprotocol**"
        in census_doc,
        "census prose lost the exact iterator-helper partition",
    )
    doc_exclusion_tokens = {
        "`slice::GetDisjointMutError`",
        "`str::Utf8Error`",
        "`binary_heap::PeekMut`",
        "`btree_map::{Entry, OccupiedEntry,\nVacantEntry}`",
        "`hash_map::{Entry, OccupiedEntry, VacantEntry}`",
        "`string::{FromUtf8Error, FromUtf16Error}`",
        "`cell::{Ref, RefMut, BorrowError,\nBorrowMutError}`",
        "`alloc::TryReserveError`",
    }
    require(
        all(token in census_doc for token in doc_exclusion_tokens),
        "census prose lost an exact non-iterator helper exclusion",
    )
    require("STRING-DECODE-01" not in contract_by_id, "heterogeneous UTF-8 decode row survived")
    require(
        {
            "STRING-DECODE-STRICT-01",
            "STRING-DECODE-LOSSY-01",
            "STRING-DECODE-ERROR-01",
        }
        <= set(contract_by_id),
        "UTF-8 strict/lossy/error contracts are not split exactly",
    )

    map_fields, surface_map = read_tsv(SURFACE_MAP)
    require(map_fields == MAP_FIELDS, "surface-map TSV schema changed")
    require(len(surface_map) == 545, "surface map must contain exactly 545 rows")
    require(
        all(all(row[field] for field in MAP_FIELDS) for row in surface_map),
        "empty required surface-map field",
    )
    map_keys = [row["canonical_key"] for row in surface_map]
    require(len(map_keys) == len(set(map_keys)), "duplicate canonical declaration mapping")

    stable_safe = selected_inventory(inventory, "safe")
    stable_unsafe = selected_inventory(inventory, "unsafe")
    require(len(stable_safe) == 545, "raw stable-safe seed set is not 545")
    require(len(stable_unsafe) == 35, "raw stable-unsafe seed set is not 35")
    require(set(map_keys) == set(stable_safe), "surface map has an omission or extra key")
    string_routes = {
        row["member_name"]: row["primary_contract_id"]
        for row in surface_map
        if row["item_path"] == "alloc::string::String"
        and row["member_name"] in {"from_utf8", "from_utf8_lossy"}
    }
    require(
        string_routes
        == {
            "from_utf8": "STRING-DECODE-STRICT-01",
            "from_utf8_lossy": "STRING-DECODE-LOSSY-01",
        },
        "String UTF-8 constructors do not route to the split contracts",
    )
    helper_signatures = {
        row["member_name"]: row["signature"]
        for row in inventory
        if row["item_path"] == "alloc::string::FromUtf8Error"
        and row["member_name"] in {"as_bytes", "into_bytes", "utf8_error"}
        and row["stability"] == "stable"
        and row["caller_safety"] == "safe"
    }
    require(
        helper_signatures
        == {
            "as_bytes": "pub fn as_bytes(&self) -> &[u8]",
            "into_bytes": "pub fn into_bytes(self) -> Vec<u8>",
            "utf8_error": "pub fn utf8_error(&self) -> Utf8Error",
        },
        "FromUtf8Error helper ownership/borrow signatures changed",
    )
    require(
        contract_by_id["STRING-DECODE-STRICT-01"]["complexity"]
        == "O(n) validation and O(1) owner wrapping; zero allocation",
        "strict UTF-8 complexity/allocation contract changed",
    )
    require(
        contract_by_id["STRING-DECODE-ERROR-01"]["complexity"]
        == "O(1), zero allocation, and no rescan",
        "FromUtf8Error accessor complexity contract changed",
    )
    exact_contract_fragments = {
        "STRING-DECODE-STRICT-01": {
            "post_state_result": (
                "String owner reusing the vector allocation",
                "FromUtf8Error owner containing that same vector",
            ),
            "invalidation": ("no result borrow is minted",),
        },
        "STRING-DECODE-LOSSY-01": {
            "post_state_result": (
                "valid input returns borrowed valid text",
                "invalid input returns an owned String",
            ),
            "invalidation": (
                "Borrowed result is tied to the input byte owner/range",
                "owned result has independent ownership",
            ),
        },
        "STRING-DECODE-ERROR-01": {
            "post_state_result": (
                "as_bytes returns a borrow tied to the error owner",
                "into_bytes consumes the error and returns the sole Vec<u8> owner",
            ),
            "invalidation": ("no borrow is tied to the original consumed caller binding",),
        },
        "MAP-ENTRY-01": {
            "post_state_result": (
                "stored map key when occupied",
                "guard-owned candidate when vacant",
                "map-stored value borrow or occupied guard",
            ),
            "failure_drop_abandonment": (
                "Hash entry reserves before returning a vacant guard",
                "B-tree insertion may allocate/split later",
            ),
            "complexity": (
                "B-tree entry lookup O(log n)",
                "hash entry lookup expected O(1)",
            ),
        },
        "MAP-OCCUPIED-01": {
            "post_state_result": (
                "borrow the map-stored K/V",
                "into_mut consumes the guard and returns a map-stored V borrow",
            ),
            "complexity": ("B-tree removal adds O(log n) repair",),
        },
        "MAP-VACANT-01": {
            "post_state_result": (
                "guard/candidate-derived K borrow",
                "map-stored V borrow",
                "map-derived occupied guard",
            ),
            "failure_drop_abandonment": (
                "Hash vacant insertion is no-grow because entry pre-reserved",
                "B-tree insertion may allocate/split",
            ),
        },
    }
    for contract_id, field_fragments in exact_contract_fragments.items():
        for field, fragments in field_fragments.items():
            value = contract_by_id[contract_id][field]
            for fragment in fragments:
                require(
                    fragment in value,
                    f"{contract_id} {field} lost exact ownership/provenance: {fragment}",
                )

    d10_fields, d10_surface_map = read_tsv(D10_SURFACE_MAP)
    require(d10_fields == D10_MAP_FIELDS, "D10 surface-map TSV schema changed")
    require(len(d10_surface_map) == 175, "D10 surface map must contain exactly 175 rows")
    require(
        all(all(row[field] for field in D10_MAP_FIELDS) for row in d10_surface_map),
        "empty required D10 surface-map field",
    )
    d10_keys = [row["canonical_key"] for row in d10_surface_map]
    require(len(d10_keys) == len(set(d10_keys)), "duplicate D10 canonical mapping")
    require(not set(d10_keys) & set(map_keys), "D10 map overlaps the 16-seed map")
    require(
        d10_surface_map == build_d10_rows(),
        "D10 surface map is not the exact mechanically derived crosswalk",
    )
    def is_range_path(path: str) -> bool:
        return (
            path in {"core::range", "std::range"}
            or path.startswith("core::range::")
            or path in OPS_RANGE_PATHS
        )

    require(
        sum(not is_range_path(row["representative_path"]) for row in d10_surface_map) == 132,
        "D10 map must retain exactly 132 iteration declarations",
    )
    require(
        sum(is_range_path(row["representative_path"]) for row in d10_surface_map) == 43,
        "D10 map must retain exactly 43 range declarations",
    )
    require(
        {row["route_kind"] for row in d10_surface_map}
        == {"contract", "redundant_surface"},
        "D10 route kinds changed",
    )
    require(
        sum(row["route_kind"] == "contract" for row in d10_surface_map) == 138,
        "D10 cluster-route count changed",
    )
    require(
        sum(row["route_kind"] == "redundant_surface" for row in d10_surface_map) == 37,
        "D10 redundant-surface count changed",
    )
    peekable_routes = {
        row["member_name"]: row["route_kind"]
        for row in d10_surface_map
        if row["representative_path"] == "core::iter::Peekable"
    }
    require(
        peekable_routes
        == {
            "Peekable": "redundant_surface",
            "next_if": "contract",
            "next_if_eq": "contract",
            "next_if_map": "contract",
            "next_if_map_mut": "contract",
            "peek": "contract",
            "peek_mut": "contract",
        },
        "Peekable type and operation routes are not distinguished exactly",
    )
    for mapped in d10_surface_map:
        require(
            mapped["route_id"] in contract_by_id,
            f"unknown D10 contract_id {mapped['route_id']}",
        )

    require("TRAIT-EXACT-01" in contract_by_id, "missing ExactSizeIterator contract")
    require("TRAIT-FUSED-01" in contract_by_id, "missing FusedIterator contract")
    require(
        "FusedIterator" not in contract_by_id["TRAIT-EXACT-01"]["rust_surfaces"],
        "ExactSizeIterator and FusedIterator must remain independent contracts",
    )
    require(
        "ExactSizeIterator" not in contract_by_id["TRAIT-FUSED-01"]["rust_surfaces"],
        "FusedIterator must not imply exact size",
    )
    range_primary = {
        row["route_id"]
        for row in d10_surface_map
        if is_range_path(row["representative_path"])
        and row["route_kind"] == "contract"
    }
    require(
        len(range_primary) == 29,
        "range value/query/iteration evidence must remain split into 29 coarse clusters",
    )
    required_range_contracts = {
        "RANGE-BOUND-VALUE-01",
        "RANGE-BOUND-BORROW-01",
        "RANGE-BOUND-CLONE-01",
        "RANGE-BOUND-MAP-01",
        "RANGE-LEGACY-HALFOPEN-STATE-01",
        "RANGE-BOUNDS-PROTOCOL-01",
        "RANGE-BOUNDS-CONTAINS-01",
        "RANGE-LEGACY-FROM-STATE-01",
        "RANGE-VALUE-FULL-01",
        "RANGE-LEGACY-INCLUSIVE-STATE-01",
        "RANGE-LEGACY-INCLUSIVE-CONTAINS-01",
        "RANGE-LEGACY-INCLUSIVE-EMPTY-01",
        "RANGE-LEGACY-INCLUSIVE-ACCESS-01",
        "RANGE-LEGACY-INCLUSIVE-INTO-01",
        "RANGE-VALUE-TO-EXCLUSIVE-01",
        "RANGE-CONTAINS-TO-EXCLUSIVE-01",
    }
    require(
        required_range_contracts <= set(contract_by_id),
        "legacy core::ops range obligations are incomplete",
    )

    forbidden_primary_prefixes = ("TRAIT-", "ALLOC-", "RAW-UNSAFE-", "HELPER-")
    for mapped in surface_map:
        raw = stable_safe[mapped["canonical_key"]]
        for field in ("item_path", "member_name", "source_path"):
            require(
                mapped[field] == raw[field],
                f"{mapped['canonical_key']} disagrees with inventory field {field}",
            )
        contract_id = mapped["primary_contract_id"]
        require(contract_id in contract_by_id, f"unknown contract_id {contract_id}")
        require(
            not contract_id.startswith(forbidden_primary_prefixes),
            f"cross-cutting/evidence row used as primary: {contract_id}",
        )
        contract_surface = contract_by_id[contract_id]["rust_surfaces"]
        require(
            token_present(contract_surface, SEEDS[raw["item_path"]]),
            f"{contract_id} does not name seed type {raw['item_path']}",
        )
        require(
            token_present(contract_surface, raw["member_name"]),
            f"{contract_id} does not name member {raw['member_name']}",
        )

        markers = set(mapped["markers"].split(";"))
        require(markers <= ALLOWED_MARKERS, f"unknown markers on {mapped['canonical_key']}")
        require("stable_safe_seed" in markers, "missing stable_safe_seed marker")
        is_raw = contract_id.startswith("RAW-SAFE-")
        require(
            ("raw_obligation" in markers) == is_raw,
            f"raw-obligation marker mismatch on {mapped['canonical_key']}",
        )
        require(
            ("one_hop_helper_return" in markers)
            == returns_one_hop_helper(raw["signature"]),
            f"one-hop helper marker mismatch on {mapped['canonical_key']}",
        )

    unsafe_contracts = [
        row for row in contracts if row["contract_id"].startswith("RAW-UNSAFE-")
    ]
    require(len(unsafe_contracts) == 8, "expected eight unsafe evidence clusters")
    unsafe_fields, unsafe_map = read_tsv(UNSAFE_EVIDENCE_MAP)
    require(unsafe_fields == UNSAFE_MAP_FIELDS, "unsafe evidence-map TSV schema changed")
    require(len(unsafe_map) == 35, "unsafe evidence map must contain exactly 35 rows")
    require(
        unsafe_map == build_unsafe_map_rows(),
        "unsafe evidence map is not the exact inventory/census derivation",
    )
    unsafe_keys = [row["canonical_key"] for row in unsafe_map]
    require(
        len(unsafe_keys) == len(set(unsafe_keys)),
        "duplicate canonical declaration in unsafe evidence map",
    )
    require(
        set(unsafe_keys) == set(stable_unsafe),
        "unsafe evidence map has an omission or extra declaration key",
    )
    unsafe_contract_ids = {row["contract_id"] for row in unsafe_contracts}
    for mapped in unsafe_map:
        require(
            mapped["evidence_cluster_id"] in unsafe_contract_ids,
            f"unsafe declaration has unknown evidence cluster: {mapped['canonical_key']}",
        )
        require(
            mapped["evidence_disposition"] == "RAW_EVIDENCE_ONLY_NO_XLANG_SURFACE",
            f"unsafe declaration is not evidence-only: {mapped['canonical_key']}",
        )

    print(
        "rust data-contract census: PASS — 276 non-importable coverage/evidence clusters, "
        "545 canonical stable-safe declarations mapped exactly once, "
        "175 D10 declarations routed exactly once (132 iteration, 43 range; "
        "138 cluster, 37 redundant), "
        "35 canonical stable-unsafe declarations mapped exactly once to evidence clusters"
    )


if __name__ == "__main__":
    main()
