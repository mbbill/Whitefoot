#!/usr/bin/env python3
"""Build the exact Rust 1.97 iteration/range contract crosswalk.

The selection predicate is intentionally independent of the domain classifier:
the crosswalk must detect a bad D10 classification rather than inherit it.
"""

from __future__ import annotations

import argparse
import csv
import io
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
INVENTORY = ROOT / "RUST-1.97.0-API-INVENTORY.tsv"
OUTPUT = ROOT / "RUST-D10-SURFACE-MAP.tsv"

OPS_RANGE_PATHS = {
    "core::ops::Bound",
    "core::ops::Range",
    "core::ops::RangeBounds",
    "core::ops::RangeFrom",
    "core::ops::RangeFull",
    "core::ops::RangeInclusive",
    "core::ops::RangeTo",
    "core::ops::RangeToInclusive",
}

FIELDS = [
    "canonical_key",
    "representative_path",
    "member_name",
    "route_kind",
    "route_id",
    "route_reason",
]

ITERATOR_METHOD_ROUTES = {
    "next": "TRAIT-ITER-01",
    "size_hint": "TRAIT-ITER-01",
    "by_ref": "ITER-ADAPT-REBORROW-01",
    "chain": "ITER-ADAPT-CHAIN-01",
    "cloned": "ITER-ADAPT-DUPLICATE-01",
    "collect": "TRAIT-COLLECT-01",
    "copied": "ITER-ADAPT-DUPLICATE-01",
    "cycle": "ITER-ADAPT-CYCLE-01",
    "enumerate": "ITER-ADAPT-TRANSFORM-01",
    "filter": "ITER-ADAPT-SELECT-01",
    "filter_map": "ITER-ADAPT-SELECT-01",
    "flat_map": "ITER-ADAPT-NEST-01",
    "flatten": "ITER-ADAPT-NEST-01",
    "fuse": "ITER-ADAPT-FUSE-01",
    "inspect": "ITER-ADAPT-TRANSFORM-01",
    "map": "ITER-ADAPT-TRANSFORM-01",
    "map_while": "ITER-ADAPT-POSITION-01",
    "peekable": "ITER-ADAPT-PEEK-01",
    "rev": "ITER-ADAPT-DIRECTION-01",
    "scan": "ITER-ADAPT-STATE-01",
    "skip": "ITER-ADAPT-POSITION-01",
    "skip_while": "ITER-ADAPT-POSITION-01",
    "step_by": "ITER-ADAPT-POSITION-01",
    "take": "ITER-ADAPT-POSITION-01",
    "take_while": "ITER-ADAPT-POSITION-01",
    "zip": "ITER-ADAPT-ZIP-01",
    "all": "ITER-CONSUME-SHORT-01",
    "any": "ITER-CONSUME-SHORT-01",
    "find": "ITER-CONSUME-SHORT-01",
    "find_map": "ITER-CONSUME-SHORT-01",
    "nth": "ITER-CONSUME-SHORT-01",
    "position": "ITER-CONSUME-SHORT-01",
    "rposition": "ITER-CONSUME-SHORT-01",
    "try_fold": "ITER-CONSUME-SHORT-01",
    "try_for_each": "ITER-CONSUME-SHORT-01",
    "count": "ITER-CONSUME-FOLD-01",
    "fold": "ITER-CONSUME-FOLD-01",
    "for_each": "ITER-CONSUME-FOLD-01",
    "last": "ITER-CONSUME-FOLD-01",
    "product": "ITER-CONSUME-FOLD-01",
    "reduce": "ITER-CONSUME-FOLD-01",
    "sum": "ITER-CONSUME-FOLD-01",
    "cmp": "ITER-CONSUME-RELATION-01",
    "eq": "ITER-CONSUME-RELATION-01",
    "ge": "ITER-CONSUME-RELATION-01",
    "gt": "ITER-CONSUME-RELATION-01",
    "is_sorted": "ITER-CONSUME-RELATION-01",
    "is_sorted_by": "ITER-CONSUME-RELATION-01",
    "is_sorted_by_key": "ITER-CONSUME-RELATION-01",
    "le": "ITER-CONSUME-RELATION-01",
    "lt": "ITER-CONSUME-RELATION-01",
    "max": "ITER-CONSUME-RELATION-01",
    "max_by": "ITER-CONSUME-RELATION-01",
    "max_by_key": "ITER-CONSUME-RELATION-01",
    "min": "ITER-CONSUME-RELATION-01",
    "min_by": "ITER-CONSUME-RELATION-01",
    "min_by_key": "ITER-CONSUME-RELATION-01",
    "ne": "ITER-CONSUME-RELATION-01",
    "partial_cmp": "ITER-CONSUME-RELATION-01",
    "partition": "ITER-CONSUME-FANOUT-01",
    "unzip": "ITER-CONSUME-FANOUT-01",
}

ITER_TYPE_ROUTES = {
    "Chain": "ITER-ADAPT-CHAIN-01",
    "Cloned": "ITER-ADAPT-DUPLICATE-01",
    "Copied": "ITER-ADAPT-DUPLICATE-01",
    "Cycle": "ITER-ADAPT-CYCLE-01",
    "Empty": "ITER-SOURCE-VALUE-01",
    "Enumerate": "ITER-ADAPT-TRANSFORM-01",
    "Filter": "ITER-ADAPT-SELECT-01",
    "FilterMap": "ITER-ADAPT-SELECT-01",
    "FlatMap": "ITER-ADAPT-NEST-01",
    "Flatten": "ITER-ADAPT-NEST-01",
    "FromFn": "ITER-SOURCE-CALLBACK-01",
    "Fuse": "ITER-ADAPT-FUSE-01",
    "Inspect": "ITER-ADAPT-TRANSFORM-01",
    "Map": "ITER-ADAPT-TRANSFORM-01",
    "MapWhile": "ITER-ADAPT-POSITION-01",
    "Once": "ITER-SOURCE-VALUE-01",
    "OnceWith": "ITER-SOURCE-CALLBACK-01",
    "Peekable": "ITER-ADAPT-PEEK-01",
    "Repeat": "ITER-SOURCE-REPEAT-01",
    "RepeatN": "ITER-SOURCE-REPEAT-01",
    "RepeatWith": "ITER-SOURCE-CALLBACK-01",
    "Rev": "ITER-ADAPT-DIRECTION-01",
    "Scan": "ITER-ADAPT-STATE-01",
    "Skip": "ITER-ADAPT-POSITION-01",
    "SkipWhile": "ITER-ADAPT-POSITION-01",
    "StepBy": "ITER-ADAPT-POSITION-01",
    "Successors": "ITER-SOURCE-CALLBACK-01",
    "Take": "ITER-ADAPT-POSITION-01",
    "TakeWhile": "ITER-ADAPT-POSITION-01",
    "Zip": "ITER-ADAPT-ZIP-01",
}

ITER_FUNCTION_ROUTES = {
    "chain": "ITER-ADAPT-CHAIN-01",
    "empty": "ITER-SOURCE-VALUE-01",
    "from_fn": "ITER-SOURCE-CALLBACK-01",
    "once": "ITER-SOURCE-VALUE-01",
    "once_with": "ITER-SOURCE-CALLBACK-01",
    "repeat": "ITER-SOURCE-REPEAT-01",
    "repeat_n": "ITER-SOURCE-REPEAT-01",
    "repeat_with": "ITER-SOURCE-CALLBACK-01",
    "successors": "ITER-SOURCE-CALLBACK-01",
    "zip": "ITER-ADAPT-ZIP-01",
}

PEEKABLE_METHODS = {
    "next_if",
    "next_if_eq",
    "next_if_map",
    "next_if_map_mut",
    "peek",
    "peek_mut",
}

RANGE_OPERATION_ROUTES = {
    ("core::range::Range", "Range"): "RANGE-VALUE-HALFOPEN-01",
    ("core::range::Range", "contains"): "RANGE-CONTAINS-HALFOPEN-01",
    ("core::range::Range", "is_empty"): "RANGE-EMPTY-HALFOPEN-01",
    ("core::range::Range", "iter"): "RANGE-ITER-HALFOPEN-01",
    ("core::range::RangeFrom", "RangeFrom"): "RANGE-VALUE-FROM-01",
    ("core::range::RangeFrom", "contains"): "RANGE-CONTAINS-FROM-01",
    ("core::range::RangeFrom", "iter"): "RANGE-ITER-FROM-01",
    ("core::range::RangeInclusive", "RangeInclusive"): "RANGE-VALUE-INCLUSIVE-01",
    ("core::range::RangeInclusive", "contains"): "RANGE-CONTAINS-INCLUSIVE-01",
    ("core::range::RangeInclusive", "is_empty"): "RANGE-EMPTY-INCLUSIVE-01",
    ("core::range::RangeInclusive", "iter"): "RANGE-ITER-INCLUSIVE-01",
    ("core::range::RangeToInclusive", "RangeToInclusive"): "RANGE-VALUE-TO-INCLUSIVE-01",
    ("core::range::RangeToInclusive", "contains"): "RANGE-CONTAINS-TO-INCLUSIVE-01",
    ("core::range::RangeIter", "RangeIter"): "RANGE-ITER-HALFOPEN-01",
    ("core::range::RangeInclusiveIter", "RangeInclusiveIter"): "RANGE-ITER-INCLUSIVE-01",
    ("core::range::RangeFromIter", "RangeFromIter"): "RANGE-ITER-FROM-01",
    ("core::ops::Bound", "Bound"): "RANGE-BOUND-VALUE-01",
    ("core::ops::Bound", "as_ref"): "RANGE-BOUND-BORROW-01",
    ("core::ops::Bound", "cloned"): "RANGE-BOUND-CLONE-01",
    ("core::ops::Bound", "map"): "RANGE-BOUND-MAP-01",
    ("core::ops::Range", "Range"): "RANGE-LEGACY-HALFOPEN-STATE-01",
    ("core::ops::Range", "contains"): "RANGE-CONTAINS-HALFOPEN-01",
    ("core::ops::Range", "is_empty"): "RANGE-EMPTY-HALFOPEN-01",
    ("core::ops::RangeBounds", "RangeBounds"): "RANGE-BOUNDS-PROTOCOL-01",
    ("core::ops::RangeBounds", "start_bound"): "RANGE-BOUNDS-PROTOCOL-01",
    ("core::ops::RangeBounds", "end_bound"): "RANGE-BOUNDS-PROTOCOL-01",
    ("core::ops::RangeBounds", "contains"): "RANGE-BOUNDS-CONTAINS-01",
    ("core::ops::RangeFrom", "RangeFrom"): "RANGE-LEGACY-FROM-STATE-01",
    ("core::ops::RangeFrom", "contains"): "RANGE-CONTAINS-FROM-01",
    ("core::ops::RangeFull", "RangeFull"): "RANGE-VALUE-FULL-01",
    ("core::ops::RangeInclusive", "RangeInclusive"): "RANGE-LEGACY-INCLUSIVE-STATE-01",
    ("core::ops::RangeInclusive", "new"): "RANGE-LEGACY-INCLUSIVE-STATE-01",
    ("core::ops::RangeInclusive", "contains"): "RANGE-LEGACY-INCLUSIVE-CONTAINS-01",
    ("core::ops::RangeInclusive", "is_empty"): "RANGE-LEGACY-INCLUSIVE-EMPTY-01",
    ("core::ops::RangeInclusive", "start"): "RANGE-LEGACY-INCLUSIVE-ACCESS-01",
    ("core::ops::RangeInclusive", "end"): "RANGE-LEGACY-INCLUSIVE-ACCESS-01",
    ("core::ops::RangeInclusive", "into_inner"): "RANGE-LEGACY-INCLUSIVE-INTO-01",
    ("core::ops::RangeTo", "RangeTo"): "RANGE-VALUE-TO-EXCLUSIVE-01",
    ("core::ops::RangeTo", "contains"): "RANGE-CONTAINS-TO-EXCLUSIVE-01",
    ("core::ops::RangeToInclusive", "RangeToInclusive"): "RANGE-VALUE-TO-INCLUSIVE-01",
    ("core::ops::RangeToInclusive", "contains"): "RANGE-CONTAINS-TO-INCLUSIVE-01",
}


def route(row: dict[str, str]) -> tuple[str, str, str]:
    path = row["representative_path"]
    name = row["member_name"]
    item_kind = row["item_kind"]
    member_kind = row["member_kind"]

    if path in {"core::iter", "std::iter"}:
        return (
            "redundant_surface",
            "TRAIT-ITER-01",
            "Rust namespace or reexport spelling; iteration semantics route to the protocol clusters.",
        )
    if path in {"core::range", "std::range"}:
        return (
            "redundant_surface",
            "RANGE-VALUE-HALFOPEN-01",
            "Rust namespace or reexport spelling; range semantics route to the value clusters.",
        )

    protocol_routes = {
        "core::iter::IntoIterator": "TRAIT-INTOITER-01",
        "core::iter::Iterator": None,
        "core::iter::DoubleEndedIterator": None,
        "core::iter::ExactSizeIterator": "TRAIT-EXACT-01",
        "core::iter::FusedIterator": "TRAIT-FUSED-01",
        "core::iter::Extend": "TRAIT-EXTEND-01",
        "core::iter::FromIterator": "TRAIT-COLLECT-01",
        "core::iter::Sum": "ITER-CONSUME-FOLD-01",
        "core::iter::Product": "ITER-CONSUME-FOLD-01",
    }
    if path in protocol_routes and protocol_routes[path] is not None:
        return (
            "contract",
            protocol_routes[path] or "",
            "Stable protocol declaration routes to its normalized caller contract.",
        )
    if path == "core::iter::Iterator":
        if name in {"Iterator", "Item"}:
            route_id = "TRAIT-ITER-01"
        else:
            route_id = ITERATOR_METHOD_ROUTES.get(name, "")
        if not route_id:
            raise ValueError(f"unrouted stable Iterator declaration: {name}")
        return (
            "contract",
            route_id,
            "Stable iterator operation routes to its normalized caller contract.",
        )
    if path == "core::iter::DoubleEndedIterator":
        if name in {"DoubleEndedIterator", "next_back"}:
            route_id = "TRAIT-DOUBLE-01"
        elif name in {"nth_back", "rfind", "try_rfold"}:
            route_id = "ITER-CONSUME-SHORT-01"
        elif name == "rfold":
            route_id = "ITER-CONSUME-FOLD-01"
        else:
            raise ValueError(f"unrouted stable DoubleEndedIterator declaration: {name}")
        return (
            "contract",
            route_id,
            "Stable reverse-iteration operation routes to its normalized caller contract.",
        )
    if path == "core::iter::Peekable":
        if name == "Peekable" and member_kind == "item":
            return (
                "redundant_surface",
                "ITER-ADAPT-PEEK-01",
                "The opaque Peekable type spelling is represented by its cached-cursor operation contract.",
            )
        if name in PEEKABLE_METHODS and member_kind == "provided_or_inherent_method":
            return (
                "contract",
                "ITER-ADAPT-PEEK-01",
                "A stable Peekable operation contributes to the normalized cached-cursor contract.",
            )
        raise ValueError(
            f"unrouted stable Peekable declaration: {name} ({item_kind}/{member_kind})"
        )
    short_name = path.rsplit("::", 1)[-1]
    if short_name in ITER_TYPE_ROUTES and item_kind == "struct":
        return (
            "redundant_surface",
            ITER_TYPE_ROUTES[short_name],
            "Opaque Rust adapter/source type spelling is represented by the operation contract.",
        )
    if short_name in ITER_FUNCTION_ROUTES and item_kind == "fn":
        return (
            "contract",
            ITER_FUNCTION_ROUTES[short_name],
            "Stable producer/adapter function routes to its normalized caller contract.",
        )

    range_route = RANGE_OPERATION_ROUTES.get((path, name))
    if range_route is not None:
        return (
            "redundant_surface" if path.endswith("Iter") else "contract",
            range_route,
            "Stable range value, query, or iterator state routes to its exact bound-form contract.",
        )
    raise ValueError(f"unrouted D10 declaration: {path}::{name}")


def canonical_stable_rows() -> list[dict[str, str]]:
    with INVENTORY.open(encoding="utf-8", newline="") as handle:
        raw = list(csv.DictReader(handle, delimiter="\t"))
    by_key: dict[str, dict[str, str]] = {}
    for row in raw:
        if row["stability"] != "stable":
            continue
        key = row["canonical_key"]
        current = by_key.get(key)
        preference = ("core", "alloc", "std").index(row["surface_crate"])
        if current is None:
            by_key[key] = row
            continue
        current_preference = ("core", "alloc", "std").index(current["surface_crate"])
        if (preference, len(row["item_path"]), row["item_path"]) < (
            current_preference,
            len(current["item_path"]),
            current["item_path"],
        ):
            by_key[key] = row
    return list(by_key.values())


def is_iteration_or_range(row: dict[str, str]) -> bool:
    path = row["item_path"]
    return (
        path in {"std::iter", "std::range"}
        or path == "core::iter"
        or path.startswith("core::iter::")
        or path == "core::range"
        or path.startswith("core::range::")
        or path in OPS_RANGE_PATHS
    )


def build_rows() -> list[dict[str, str]]:
    selected = [
        row
        for row in canonical_stable_rows()
        if row["caller_safety"] == "safe" and is_iteration_or_range(row)
    ]
    selected.sort(key=lambda row: (row["item_path"], row["member_name"], row["canonical_key"]))
    if len(selected) != 175:
        raise ValueError(
            "expected 175 canonical stable declarations from the independent "
            f"iteration/range predicate, found {len(selected)}"
        )
    rows: list[dict[str, str]] = []
    for inventory_row in selected:
        declaration = dict(inventory_row)
        declaration["representative_path"] = inventory_row["item_path"]
        route_kind, route_id, reason = route(declaration)
        rows.append(
            {
                "canonical_key": declaration["canonical_key"],
                "representative_path": declaration["representative_path"],
                "member_name": declaration["member_name"],
                "route_kind": route_kind,
                "route_id": route_id,
                "route_reason": reason,
            }
        )
    return rows


def render(rows: list[dict[str, str]]) -> str:
    buffer = io.StringIO(newline="")
    writer = csv.DictWriter(buffer, fieldnames=FIELDS, delimiter="\t", lineterminator="\n")
    writer.writeheader()
    writer.writerows(rows)
    return buffer.getvalue()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="fail unless the checked-in crosswalk is current")
    args = parser.parse_args()
    expected = render(build_rows())
    if args.check:
        if not OUTPUT.exists() or OUTPUT.read_text(encoding="utf-8") != expected:
            raise SystemExit("D10 surface map is missing or stale")
        print("D10 surface map: PASS — 175 canonical stable declarations routed exactly once")
        return 0
    OUTPUT.write_text(expected, encoding="utf-8")
    print("D10 surface map: wrote 175 canonical stable declarations")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
