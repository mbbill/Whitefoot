#!/usr/bin/env python3
"""Verify the pinned Rust 1.97.0 census artifacts and independent seed counts."""

from __future__ import annotations

import csv
import hashlib
import json
import pathlib
import sys
from collections import Counter


ROOT = pathlib.Path(__file__).resolve().parent.parent
INVENTORY = ROOT / "RUST-1.97.0-API-INVENTORY.tsv"
MODULES = ROOT / "RUST-1.97.0-MODULE-ACCOUNTING.tsv"
MANIFEST = ROOT / "RUST-1.97.0-CENSUS-MANIFEST.json"

EXPECTED_COUNTS = {
    "inventory_rows": 17135,
    "stable_safe_rows": 10267,
    "stable_unsafe_rows": 560,
    "unstable_rows": 6308,
    "canonical_stable_safe_declarations": 5278,
    "canonical_stable_unsafe_declarations": 277,
    "module_rows": 297,
    "collapsed_module_rows": 29,
    "missing_pages": 0,
    "external_module_links": 0,
    "item_table_entries_without_descriptions": 2124,
}

# Independent narrow extractor counts recorded by the data-structure census.
EXPECTED_SEEDS = {
    "std::array": (5, 0),
    "std::slice": (121, 13),
    "std::str": (74, 7),
    "alloc::boxed::Box": (13, 3),
    "alloc::vec::Vec": (44, 2),
    "alloc::collections::vec_deque::VecDeque": (54, 0),
    "alloc::collections::linked_list::LinkedList": (20, 0),
    "alloc::collections::binary_heap::BinaryHeap": (23, 0),
    "alloc::collections::btree_map::BTreeMap": (31, 0),
    "alloc::collections::btree_set::BTreeSet": (27, 0),
    "std::collections::hash_map::HashMap": (32, 1),
    "std::collections::hash_set::HashSet": (30, 0),
    "alloc::string::String": (35, 3),
    "alloc::rc::Rc": (19, 5),
    "alloc::rc::Weak": (7, 1),
    "core::cell::RefCell": (12, 1),
}


def digest(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"census verification failed: {message}")


def main() -> None:
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    require(manifest["schema"] == "rustdoc-public-api-v3", "unexpected schema")
    require(manifest["rust"]["version"] == "1.97.0", "unexpected Rust version")
    require(
        manifest["rust"]["commit"] == "2d8144b7880597b6e6d3dfd63a9a9efae3f533d3",
        "unexpected Rust commit",
    )
    require(
        manifest["rust"]["source_commit"]
        == "2d8144b7880597b6e6d3dfd63a9a9efae3f533d3",
        "unexpected Rust source commit",
    )
    require(manifest["counts"] == EXPECTED_COUNTS, "manifest counts changed")
    require(manifest["outputs"][INVENTORY.name] == digest(INVENTORY), "inventory digest mismatch")
    require(manifest["outputs"][MODULES.name] == digest(MODULES), "module digest mismatch")

    with INVENTORY.open(encoding="utf-8", newline="") as handle:
        rows = list(csv.DictReader(handle, delimiter="\t"))
    with MODULES.open(encoding="utf-8", newline="") as handle:
        modules = list(csv.DictReader(handle, delimiter="\t"))

    require(len(rows) == EXPECTED_COUNTS["inventory_rows"], "inventory row count changed")
    require(len(modules) == EXPECTED_COUNTS["module_rows"], "module row count changed")
    require(all(row["canonical_key"] for row in rows), "empty canonical key")
    require(all(row["docs_path"] for row in rows), "empty documentation path")
    expected_scope = {
        "collapse_prefixes": ["core::arch", "core::intrinsics"],
        "crates": ["core", "alloc", "std"],
        "member_section_policy": "count defining methods, deprecated methods, associated types, and associated constants; reject rustdoc trait-impl repeats",
        "stability_policy": "stable caller contracts are the anchor; unstable and unsafe rows remain evidence",
        "trait_impl_policy": "count defining trait declarations; do not expand repeated concrete impl methods",
        "item_class_policy": "fail closed on every unrecognized or ambiguous rustdoc item-table entry",
        "item_table_policy": "consume every dt and require zero or one adjacent dd; reject all orphan markup",
        "recognized_item_classes": [
            "attr",
            "constant",
            "derive",
            "enum",
            "fn",
            "keyword",
            "macro",
            "mod",
            "primitive",
            "static",
            "struct",
            "trait",
            "traitalias",
            "type",
            "union",
        ],
    }
    require(manifest["scope"] == expected_scope, "public-census scope changed")
    require(
        Counter(row["member_kind"] for row in rows)
        == {
            "item": 3691,
            "required_method": 764,
            "provided_or_inherent_method": 11992,
            "associated_type": 171,
            "associated_constant": 517,
        },
        "defining-member class counts changed",
    )

    def has_member(
        path: str,
        kind: str,
        name: str,
        stability: str,
        deprecated: str,
    ) -> bool:
        return any(
            row["item_path"] == path
            and row["member_kind"] == kind
            and row["member_name"] == name
            and row["stability"] == stability
            and row["deprecated"] == deprecated
            for row in rows
        )

    require(
        has_member("core::time::Duration", "associated_constant", "ZERO", "stable", "no")
        and has_member("core::time::Duration", "associated_constant", "MAX", "stable", "no"),
        "inherent associated constants are missing",
    )
    trait_aliases = [row for row in rows if row["item_kind"] == "traitalias"]
    expected_trait_aliases = [
        {
            "surface_crate": "core",
            "module_path": "core::ptr",
            "item_path": "core::ptr::Thin",
            "item_kind": "traitalias",
            "member_kind": "item",
            "member_name": "Thin",
            "stability": "unstable",
            "since": "",
            "deprecated": "no",
            "caller_safety": "safe",
            "signature": "trait Thin = Pointee<Metadata = ()> + PointeeSized;",
            "summary": "Pointers to types implementing this trait alias are “thin”.",
            "source_path": "src/core/ptr/metadata.rs.html#85",
            "docs_path": "core/ptr/traitalias.Thin.html",
            "canonical_key": "src/core/ptr/metadata.rs.html#85|item|Thin",
            "duplicate_of": "",
        },
        {
            "surface_crate": "std",
            "module_path": "std::ptr",
            "item_path": "std::ptr::Thin",
            "item_kind": "traitalias",
            "member_kind": "item",
            "member_name": "Thin",
            "stability": "unstable",
            "since": "",
            "deprecated": "no",
            "caller_safety": "safe",
            "signature": "trait Thin = Pointee<Metadata = ()>;",
            "summary": "Pointers to types implementing this trait alias are “thin”.",
            "source_path": "src/core/ptr/metadata.rs.html#85",
            "docs_path": "std/ptr/traitalias.Thin.html",
            "canonical_key": "src/core/ptr/metadata.rs.html#85|item|Thin",
            "duplicate_of": "core::ptr::Thin",
        },
    ]
    require(
        trait_aliases == expected_trait_aliases,
        "unstable Thin trait-alias renderings are missing or not canonicalized",
    )
    require(
        Counter(row["item_kind"] for row in rows)["traitalias"] == 2,
        "trait-alias item-kind count changed",
    )
    ptr_modules = {
        row["module_path"]: row for row in modules if row["module_path"] in {"core::ptr", "std::ptr"}
    }
    require(
        ptr_modules == {
            "core::ptr": {
                "crate": "core",
                "module_path": "core::ptr",
                "mode": "detailed",
                "direct_modules": "0",
                "direct_items": "39",
                "direct_stable_items": "32",
                "direct_unstable_items": "7",
                "entry_digest": "f2b821d730efa216d68b4b847c5e8ed265da82ebb82f1a60c65f0c11ff3d133f",
                "docs_path": "core/ptr/index.html",
            },
            "std::ptr": {
                "crate": "std",
                "module_path": "std::ptr",
                "mode": "detailed",
                "direct_modules": "0",
                "direct_items": "39",
                "direct_stable_items": "32",
                "direct_unstable_items": "7",
                "entry_digest": "9a71098225e30c2e55b39d9d3db63bcdfe2848a015eb0b741bcf66e323babfb0",
                "docs_path": "std/ptr/index.html",
            },
        },
        "ptr module accounting no longer pins the trait-alias entries",
    )
    module_by_path = {row["module_path"]: row for row in modules}
    restored_modules = {
        path: (
            module_by_path[path]["mode"],
            module_by_path[path]["direct_items"],
            module_by_path[path]["direct_stable_items"],
            module_by_path[path]["direct_unstable_items"],
            module_by_path[path]["entry_digest"],
        )
        for path in {
            "core::intrinsics::gpu",
            "core::panicking::panic_const",
            "std::intrinsics::gpu",
            "std::os::macos::raw",
            "std::os::windows::net",
        }
    }
    require(
        restored_modules
        == {
            "core::intrinsics::gpu": (
                "collapsed",
                "2",
                "0",
                "2",
                "05746b7b2b7f3bc2a931bec9a2c6e2cb3ace0b5aa5821e5319f711a94f251dcc",
            ),
            "core::panicking::panic_const": (
                "detailed",
                "22",
                "0",
                "22",
                "504c9c547edd7618826f6850259367595a4a7f11923a6f13bd9387a45da7daf9",
            ),
            "std::intrinsics::gpu": (
                "detailed",
                "0",
                "0",
                "0",
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            ),
            "std::os::macos::raw": (
                "detailed",
                "10",
                "10",
                "0",
                "1bcbf78c37830d85555f6ab8fef9887c6cdd5738356ea74bd02a0257f2979a5c",
            ),
            "std::os::windows::net": (
                "detailed",
                "4",
                "0",
                "4",
                "2ca9d3f619a079a25320ba276c7716dbf5c2435a923203eb6b70b74221b50e4d",
            ),
        },
        "description-less module entries no longer preserve the five restored modules",
    )
    require(
        (
            module_by_path["core::arch::loongarch64"]["mode"],
            module_by_path["core::arch::loongarch64"]["direct_items"],
            module_by_path["core::arch::loongarch64"]["direct_stable_items"],
            module_by_path["core::arch::loongarch64"]["direct_unstable_items"],
        )
        == ("collapsed", "1542", "0", "1542")
        and (
            module_by_path["std::os::unix::raw"]["direct_items"],
            module_by_path["std::os::unix::raw"]["direct_stable_items"],
            module_by_path["std::os::unix::raw"]["direct_unstable_items"],
        )
        == ("12", "12", "0"),
        "description-less entry accounting canaries changed",
    )
    require(
        has_member("core::error::Error", "provided_or_inherent_method", "cause", "stable", "yes")
        and has_member("std::net::TcpListener", "provided_or_inherent_method", "only_v6", "stable", "yes")
        and has_member("std::net::TcpListener", "provided_or_inherent_method", "set_only_v6", "stable", "yes"),
        "recovered deprecated defining methods are missing",
    )
    require(
        has_member("core::hash::SipHasher", "item", "SipHasher", "stable", "yes")
        and has_member("std::ascii::AsciiExt", "item", "AsciiExt", "stable", "yes")
        and has_member("std::intrinsics::copy", "item", "copy", "stable", "yes")
        and has_member(
            "std::intrinsics::copy_nonoverlapping",
            "item",
            "copy_nonoverlapping",
            "stable",
            "yes",
        )
        and has_member("std::intrinsics::write_bytes", "item", "write_bytes", "stable", "yes"),
        "deprecated top-level declarations are missing",
    )
    require(
        {"std::os::linux::raw", "std::os::macos::raw", "std::os::unix::raw"}
        <= {row["module_path"] for row in modules},
        "deprecated top-level modules are missing",
    )
    require(sum(row["deprecated"] == "yes" for row in rows) == 356, "deprecated row count changed")
    require(
        len(
            {
                row["canonical_key"]
                for row in rows
                if row["stability"] == "stable" and row["deprecated"] == "yes"
            }
        )
        == 198,
        "canonical stable deprecated-declaration count changed",
    )
    require(
        has_member(
            "std::os::unix::process::CommandExt",
            "required_method",
            "setsid",
            "unstable",
            "no",
        ),
        "local stability/deprecation envelope changed",
    )
    require(
        sum(row["mode"] == "collapsed" for row in modules) == 29,
        "collapsed-module set changed",
    )
    require(
        sum(int(row["direct_stable_items"]) for row in modules if row["mode"] == "collapsed")
        == 17424,
        "collapsed stable item accounting changed",
    )
    require(
        sum(int(row["direct_unstable_items"]) for row in modules if row["mode"] == "collapsed")
        == 14174,
        "collapsed unstable item accounting changed",
    )

    actual_seeds: dict[str, list[int]] = {path: [0, 0] for path in EXPECTED_SEEDS}
    canonical_safe: set[str] = set()
    canonical_unsafe: set[str] = set()
    for row in rows:
        if row["item_path"] not in EXPECTED_SEEDS:
            continue
        if row["member_kind"] != "provided_or_inherent_method" or row["stability"] != "stable":
            continue
        index = 0 if row["caller_safety"] == "safe" else 1
        actual_seeds[row["item_path"]][index] += 1
        (canonical_safe if index == 0 else canonical_unsafe).add(row["canonical_key"])

    for path, expected in EXPECTED_SEEDS.items():
        require(tuple(actual_seeds[path]) == expected, f"seed count changed for {path}")
    require(len(canonical_safe) == 545, "canonical stable-safe seed count is not 545")
    require(len(canonical_unsafe) == 35, "canonical stable-unsafe seed count is not 35")

    print(
        "rust census: PASS — 297 modules, 17,135 detailed rows, "
        "5,278 canonical stable-safe and 277 canonical stable-unsafe declarations; "
        "545/35 selected data-structure seeds"
    )


if __name__ == "__main__":
    main()
