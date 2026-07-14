#!/usr/bin/env python3
"""Verify exact Rust seed-to-contract normalization coverage."""

from __future__ import annotations

import csv
import pathlib
import re


ROOT = pathlib.Path(__file__).resolve().parent.parent
INVENTORY = ROOT / "RUST-1.97.0-API-INVENTORY.tsv"
CONTRACTS = ROOT / "RUST-DATA-CONTRACT-CENSUS.tsv"
SURFACE_MAP = ROOT / "RUST-DATA-SURFACE-MAP.tsv"

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
    inventory_fields, inventory = read_tsv(INVENTORY)
    require("canonical_key" in inventory_fields, "raw inventory schema changed")

    contract_fields, contracts = read_tsv(CONTRACTS)
    require(contract_fields == CONTRACT_FIELDS, "contract TSV schema changed")
    require(len(contracts) == 224, "expected exactly 224 normalized/evidence rows")
    require(
        all(all(row[field] for field in CONTRACT_FIELDS) for row in contracts),
        "empty required contract field",
    )
    contract_ids = [row["contract_id"] for row in contracts]
    require(len(contract_ids) == len(set(contract_ids)), "duplicate contract_id")
    contract_by_id = {row["contract_id"]: row for row in contracts}

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
    for raw in stable_unsafe.values():
        short_type = SEEDS[raw["item_path"]]
        require(
            any(
                token_present(row["rust_surfaces"], short_type)
                and token_present(row["rust_surfaces"], raw["member_name"])
                for row in unsafe_contracts
            ),
            f"unsafe declaration lacks evidence cluster: {raw['canonical_key']}",
        )

    print(
        "rust data-contract census: PASS — 224 contract/evidence rows, "
        "545 canonical stable-safe declarations mapped exactly once, "
        "35 canonical stable-unsafe declarations retained in evidence clusters"
    )


if __name__ == "__main__":
    main()
