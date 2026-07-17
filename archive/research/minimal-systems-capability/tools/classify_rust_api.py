#!/usr/bin/env python3
"""Map every canonical stable Rust census row to independent evidence and need routes."""

from __future__ import annotations

import csv
import pathlib
import re
from collections import Counter
from dataclasses import dataclass


ROOT = pathlib.Path(__file__).resolve().parent.parent
INVENTORY = ROOT / "RUST-1.97.0-API-INVENTORY.tsv"
MODULES = ROOT / "RUST-1.97.0-MODULE-ACCOUNTING.tsv"
SURFACE_MAP = ROOT / "RUST-DATA-SURFACE-MAP.tsv"
ITERATION_SURFACE_MAP = ROOT / "RUST-D10-SURFACE-MAP.tsv"
CONTRACT_CENSUS = ROOT / "RUST-DATA-CONTRACT-CENSUS.tsv"
CAPABILITY_REGISTRY = ROOT / "CAPABILITY-OBLIGATION-REGISTRY.tsv"
RULES = ROOT / "DOMAIN-CLASSIFICATION-RULES.tsv"
OUTPUT = ROOT / "RUST-1.97.0-DOMAIN-CLASSIFICATION.tsv"
SUMMARY = ROOT / "RUST-1.97.0-DOMAIN-SUMMARY.tsv"
MODULE_OUTPUT = ROOT / "RUST-1.97.0-MODULE-DOMAIN-MAP.tsv"

DOMAIN_IDS = {
    "FFI, ABI, and OS resources": "D18",
    "async, tasks, and pinning": "D23",
    "atomics and synchronization": "D22",
    "build-time metadata and code generation": "D14",
    "bytes and text": "D11",
    "compiler and runtime support": "D26",
    "conversion and behavior": "D03",
    "error presentation and chaining": "D05",
    "filesystems and paths": "D16",
    "formatting and presentation": "D12",
    "heap allocation and runtime": "D07",
    "interior mutability and dynamic aliasing": "D08",
    "language organization and abstraction": "D02",
    "macros and convenience surface": "D14",
    "networking": "D19",
    "numeric values and algorithms": "D01",
    "ownership and borrowing": "D03",
    "ownership and capability predicates": "D03",
    "ownership, layout, and value transitions": "D04",
    "pinning and address-sensitive values": "D23",
    "prelude and reexports": "D25",
    "process environment": "D17",
    "ranges and traversal": "D10",
    "raw pointers and provenance": "D04",
    "recoverable results and variants": "D05",
    "runtime type identity and reflection": "D13",
    "scalar and aggregate values": "D01",
    "sequential and associative data structures": "D09",
    "sequential data structures": "D09",
    "shared ownership and conversions": "D08",
    "shared ownership and weak identity": "D08",
    "static behavior and views": "D03",
    "static behavior contracts": "D03",
    "synchronous I/O": "D15",
    "synchronous iteration": "D10",
    "target intrinsics and dispatch": "D24",
    "target/compiler hints": "D24",
    "threads and thread-local state": "D21",
    "time and clocks": "D20",
    "trap, panic, and diagnostics": "D06",
    "unique ownership and data structures": "D07",
}

VALID_SURFACE_EVIDENCE_STATUSES = {
    "safe_contract_anchor",
    "safe_boundary_evidence",
    "unsafe_boundary_evidence",
    "rust_surface_only",
}
VALID_NEED_ROUTE_KINDS = {
    "G0_CONTRACT",
    "LIB_CONTRACT",
    "LATER_FAMILY",
    "FRAME",
    "REDUNDANT",
    "NG",
    "NO_INDEPENDENT_NEED",
}
VALID_FRAME_IDS = {
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

DOMAIN_OVERRIDES = {
    "D04": "raw pointers and provenance",
    "D08": "shared ownership and weak identity",
    "D13": "runtime type identity and reflection",
    "D23": "pinning and address-sensitive values",
    "D24": "target intrinsics and dispatch",
}


@dataclass(frozen=True)
class SurfaceRoute:
    contract_id: str
    route_kind: str
    source: str


@dataclass(frozen=True)
class NeedRoute:
    kind: str
    route_id: str
    reason: str
    frames: tuple[str, ...] = ()
    domain_id: str | None = None


def load_ids(path: pathlib.Path, field: str) -> set[str]:
    with path.open(encoding="utf-8", newline="") as handle:
        return {row[field] for row in csv.DictReader(handle, delimiter="\t")}


def load_surface_routes(contract_ids: set[str]) -> dict[str, SurfaceRoute]:
    routes: dict[str, SurfaceRoute] = {}
    with SURFACE_MAP.open(encoding="utf-8", newline="") as handle:
        for row in csv.DictReader(handle, delimiter="\t"):
            contract_id = row["primary_contract_id"]
            if contract_id not in contract_ids:
                raise SystemExit(f"unknown detailed contract route: {contract_id}")
            routes[row["canonical_key"]] = SurfaceRoute(
                contract_id, "contract", SURFACE_MAP.name
            )

    if ITERATION_SURFACE_MAP.is_file():
        with ITERATION_SURFACE_MAP.open(encoding="utf-8", newline="") as handle:
            rows = list(csv.DictReader(handle, delimiter="\t"))
        expected = {
            "canonical_key",
            "representative_path",
            "member_name",
            "route_kind",
            "route_id",
            "route_reason",
        }
        if not rows or set(rows[0]) != expected:
            raise SystemExit(
                f"{ITERATION_SURFACE_MAP.name} does not have the required exact schema"
            )
        iteration_keys: set[str] = set()
        for row in rows:
            if row["canonical_key"] in iteration_keys:
                raise SystemExit(
                    f"duplicate iteration route for {row['canonical_key']}"
                )
            iteration_keys.add(row["canonical_key"])
            if row["route_kind"] not in {"contract", "redundant_surface"}:
                raise SystemExit(
                    f"invalid iteration route kind for {row['canonical_key']}: "
                    f"{row['route_kind']}"
                )
            if row["route_id"] not in contract_ids:
                raise SystemExit(
                    f"unknown iteration contract route for {row['canonical_key']}: "
                    f"{row['route_id']}"
                )
            incoming = SurfaceRoute(
                row["route_id"], row["route_kind"], ITERATION_SURFACE_MAP.name
            )
            existing = routes.get(row["canonical_key"])
            if existing is not None and (
                existing.contract_id != incoming.contract_id
                or existing.route_kind != incoming.route_kind
            ):
                raise SystemExit(
                    f"conflicting detailed/iteration route for {row['canonical_key']}: "
                    f"{existing} versus {incoming}"
                )
            routes[row["canonical_key"]] = incoming
    return routes


def load_rules() -> list[dict[str, str]]:
    with RULES.open(encoding="utf-8", newline="") as handle:
        rows = list(csv.DictReader(handle, delimiter="\t"))
    seen: set[str] = set()
    for row in rows:
        if row["rule_id"] in seen:
            raise SystemExit(f"duplicate classification rule: {row['rule_id']}")
        seen.add(row["rule_id"])
        if not row["match"].startswith(("path:", "regex:", "kind:")):
            raise SystemExit(f"unknown match form in {row['rule_id']}: {row['match']}")
        if row["domain"] not in DOMAIN_IDS:
            raise SystemExit(f"unknown domain in {row['rule_id']}: {row['domain']}")
    return rows


def matching_rules(
    row: dict[str, str], rules: list[dict[str, str]]
) -> list[tuple[int, dict[str, str]]]:
    matches: list[tuple[int, dict[str, str]]] = []
    for rule in rules:
        selector = rule["match"]
        if selector.startswith("path:"):
            prefix = selector[5:]
            if row["item_path"] == prefix or row["item_path"].startswith(prefix + "::"):
                matches.append((3000 + len(prefix), rule))
        elif selector.startswith("regex:"):
            pattern = selector[6:]
            if re.fullmatch(pattern, row["item_path"]):
                matches.append((5000 + len(pattern), rule))
        elif selector.startswith("kind:") and row["item_kind"] == selector[5:]:
            matches.append((1000, rule))
    return matches


def selected_rule(
    row: dict[str, str], rules: list[dict[str, str]]
) -> dict[str, str] | None:
    matches = matching_rules(row, rules)
    if not matches:
        return None
    top_score = max(score for score, _ in matches)
    winners = [rule for score, rule in matches if score == top_score]
    if len(winners) != 1:
        raise SystemExit(
            f"ambiguous route for {row['item_path']}::{row.get('member_name', '')}: "
            + ",".join(rule["rule_id"] for rule in winners)
        )
    return winners[0]


def domain_override(row: dict[str, str], route: SurfaceRoute | None) -> str | None:
    name = row["member_name"]
    path = row["item_path"]
    contract_id = route.contract_id if route is not None else ""
    if route is not None and route.source == ITERATION_SURFACE_MAP.name:
        return "D10"
    if name.startswith("downcast") and (
        "::Box" in path or "::Rc" in path or "::Arc" in path
    ):
        return "D13"
    if contract_id in {"BOX-DOWNCAST-01", "RC-DOWNCAST-01"}:
        return "D13"
    if name in {"pin", "pin_in", "into_pin"} and (
        "::Box" in path or "::Rc" in path or "::Arc" in path
    ):
        return "D23"
    if contract_id in {"BOX-PIN-01", "RC-PIN-01"}:
        return "D23"
    if "volatile" in name:
        return "D24"
    if contract_id in {
        "RAW-SAFE-PTR-01",
        "RAW-SAFE-OWNERSHIP-01",
        "RAW-SAFE-LEAK-01",
    }:
        return "D04"
    return None


def contract_need_route(
    row: dict[str, str], surface_route: SurfaceRoute
) -> NeedRoute:
    contract_id = surface_route.contract_id
    if surface_route.route_kind == "redundant_surface":
        return NeedRoute(
            "REDUNDANT",
            contract_id,
            "the iteration crosswalk identifies this Rust rendering as a duplicate of the named canonical contract",
        )
    if contract_id in {"BOX-DOWNCAST-01", "RC-DOWNCAST-01"}:
        return NeedRoute(
            "LATER_FAMILY",
            contract_id,
            "runtime type refinement and downcast belong to D13 rather than the data floor",
            domain_id="D13",
        )
    if contract_id in {"BOX-PIN-01", "RC-PIN-01"}:
        return NeedRoute(
            "LATER_FAMILY",
            contract_id,
            "physical address stability belongs to the D23 pinning family",
            domain_id="D23",
        )
    if contract_id.startswith(("RC-", "REFCELL-", "REF-GUARD-")):
        return NeedRoute(
            "LATER_FAMILY",
            contract_id,
            "shared ownership or dynamic borrowing requires the D08 family",
            frames=("F-SYNC",) if "::Arc" in row["item_path"] else (),
            domain_id="D08",
        )
    if contract_id == "RAW-SAFE-SPARE-01":
        return NeedRoute(
            "G0_CONTRACT",
            contract_id,
            "the spare-capacity demand is a G0 partial-initialization contract; Rust's raw view is evidence only",
        )
    if contract_id in {
        "RAW-SAFE-PTR-01",
        "RAW-SAFE-OWNERSHIP-01",
        "RAW-SAFE-LEAK-01",
    }:
        return NeedRoute(
            "LATER_FAMILY",
            contract_id,
            "raw address export, ownership transfer, or lifetime suppression requires a checked D04 family",
            frames=("F-MEM",),
            domain_id="D04",
        )
    frames: tuple[str, ...] = ()
    if contract_id in {
        "BOX-NEW-01",
        "BOX-INIT-01",
        "SEQ-RESERVE-01",
        "SEQ-TRY-RESERVE-01",
        "SEQ-SHRINK-01",
        "DEQUE-RESERVE-01",
        "HEAP-RESERVE-01",
        "HMAP-RESERVE-01",
        "HSET-RESERVE-01",
        "STRING-RESERVE-01",
        "ALLOC-ERROR-01",
        "ALLOC-OOM-01",
    }:
        frames = ("F-ALLOC",)
    return NeedRoute(
        "G0_CONTRACT",
        contract_id,
        "a coarse non-importable coverage cluster is in the bounded sequential data-floor accounting",
        frames=frames,
    )


def is_partial_initialization(row: dict[str, str]) -> bool:
    text = " ".join((row["item_path"], row["member_name"], row["signature"]))
    return any(
        marker in text
        for marker in (
            "MaybeUninit",
            "new_uninit",
            "new_zeroed",
            "assume_init",
            "set_len",
            "spare_capacity",
        )
    )


def is_raw_ownership(row: dict[str, str]) -> bool:
    name = row["member_name"]
    path = row["item_path"]
    signature = row["signature"]
    if name in {
        "from_ptr",
        "as_ptr",
        "as_mut_ptr",
        "expose_provenance",
        "with_exposed_provenance",
        "with_exposed_provenance_mut",
    }:
        return True
    if name.startswith(("from_raw_parts", "into_raw_parts")):
        return True
    if name.startswith(("from_raw_fd", "into_raw_fd", "from_raw_handle", "into_raw_handle", "from_raw_socket", "into_raw_socket")):
        return True
    if name in {"from_raw", "into_raw"}:
        raw_owner = any(
            marker in path
            for marker in ("::Box", "::Rc", "::Arc", "::Weak", "::CString", "::Waker")
        )
        return raw_owner or "*const" in signature or "*mut" in signature
    return False


def default_route(row: dict[str, str], rule: dict[str, str]) -> NeedRoute:
    rule_id = rule["rule_id"]
    name = row["member_name"]
    path = row["item_path"]

    if row["item_kind"] == "mod":
        return NeedRoute(
            "NO_INDEPENDENT_NEED",
            f"NAMESPACE:{rule_id}",
            "a module namespace is accounted by its declarations and adds no caller contract",
        )

    if "volatile" in name:
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D24:CHECKED-VOLATILE-MMIO",
            "volatile and MMIO access require a checked D24 contract and a reviewed device boundary",
            frames=("F-MMIO",),
            domain_id="D24",
        )

    if name.startswith("downcast") and (
        "::Box" in path or "::Rc" in path or "::Arc" in path
    ):
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D13:RUNTIME-TYPE-REFINEMENT",
            "Box, Rc, and Arc downcast require runtime type identity in D13",
            domain_id="D13",
        )
    if name in {"pin", "pin_in", "into_pin"} and (
        "::Box" in path or "::Rc" in path or "::Arc" in path
    ):
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D23:PIN-ADDRESS-STABILITY",
            "pinning is an address-stability contract in D23",
            domain_id="D23",
        )

    if is_partial_initialization(row):
        if "::Rc" in path or "::Arc" in path:
            return NeedRoute(
                "LATER_FAMILY",
                "FAMILY:D08:SHARED-PARTIAL-INITIALIZATION",
                "partial initialization is required, but shared ownership keeps this declaration in D08",
                frames=("F-MEM",),
                domain_id="D08",
            )
        return NeedRoute(
            "G0_CONTRACT",
            "CAP:OW-INIT",
            "checked partial initialization is a named G0 ownership obligation",
            frames=("F-MEM",),
        )

    if is_raw_ownership(row):
        if rule_id.startswith("DOM-SYNC-"):
            return NeedRoute(
                "LATER_FAMILY",
                "FAMILY:D22:CHECKED-ATOMIC-ADDRESSING",
                "raw atomic storage access requires the D22 memory-model family",
                frames=("F-SYNC",),
            )
        if rule_id.startswith("DOM-OS-") or rule_id == "DOM-OS":
            return NeedRoute(
                "LATER_FAMILY",
                "FAMILY:D18:CHECKED-OS-RESOURCE-TRANSFER",
                "raw OS handle transfer requires an owned-resource contract and reviewed platform frame",
                frames=("F-ABI",),
            )
        if rule_id.startswith("DOM-FFI-"):
            return NeedRoute(
                "LATER_FAMILY",
                "FAMILY:D18:CHECKED-FOREIGN-OWNERSHIP-TRANSFER",
                "foreign raw ownership transfer requires a checked ABI-boundary contract",
                frames=("F-ABI",),
            )
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D04:CHECKED-ADDRESS-OWNERSHIP",
            "raw address or ownership transfer requires a checked provenance family",
            frames=("F-MEM",),
            domain_id="D04",
        )

    if name in {"forget", "forget_unsized", "leak"}:
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D04:EXPLICIT-RESOURCE-ABANDONMENT",
            "lifetime suppression requires a checked abandonment and resource-accounting contract",
            frames=("F-MEM",),
            domain_id="D04",
        )

    if rule_id.startswith("DOM-TARGET-HINT-"):
        if name in {"unreachable_unchecked", "assert_unchecked"}:
            return NeedRoute(
                "NO_INDEPENDENT_NEED",
                "CAP:EX-ABORT",
                "the unchecked hint has no independent need beyond a checked aborting assertion",
                frames=("F-TARGET",),
            )
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D24:PROOF-DERIVED-TARGET-HINTS",
            "optimizer hints must be compiler-derived from checked facts",
            frames=("F-TARGET",),
        )
    if rule_id.startswith("DOM-TARGET-INTRINSICS-"):
        return NeedRoute(
            "NG",
            "NG:D24:WRITER-INTRINSIC-AUTHORITY",
            "the generic intrinsic catalog has no safe writer contract as a whole",
            frames=("F-TARGET",),
        )
    if rule_id == "DOM-TARGET-X86-DETECT":
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D24:TARGET-FEATURE-DETECTION",
            "machine-feature truth requires D24 and a target frame",
            frames=("F-TARGET",),
        )
    if rule_id.startswith(("DOM-TARGET-", "DOM-UNSTABLE-TARGET-SIMD")):
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D24:TARGET-OPERATIONS",
            "target operations require a separate D24 family",
            frames=("F-TARGET",),
        )

    if rule_id.startswith(("DOM-NUMERIC-", "DOM-UNSTABLE-AUTODIFF-")):
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D01:COMPLETE-NUMERICS",
            "complete numeric, floating, and target-sensitive behavior requires D01 closure",
        )
    if rule_id.startswith("DOM-TEXT-ASCII-"):
        return NeedRoute(
            "LIB_CONTRACT",
            "LIB:D11:ASCII-ALGORITHMS",
            "ASCII algorithms are ordinary checked libraries over byte/text prerequisites",
        )
    if rule_id.startswith(("DOM-TEXT-CHAR-", "DOM-TEXT-BSTR-")):
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D11:COMPLETE-TEXT",
            "complete character and byte-string behavior requires the D11 text family",
        )
    if rule_id.startswith(("DOM-TEXT-STR-", "DOM-TEXT-STRING-")):
        return NeedRoute(
            "G0_CONTRACT",
            "CAP:ST-REFINE",
            "checked UTF-8 storage and refinement are in the G0 data floor",
        )

    g0_rules = (
        "DOM-DATA-",
        "DOM-ITER-",
        "DOM-RANGE-",
        "DOM-ERROR-OPTION-",
        "DOM-ERROR-RESULT-",
        "DOM-REFERENCE-",
    )
    if rule_id.startswith(g0_rules):
        capability = "ST-DENSE"
        if "ARRAY" in rule_id:
            capability = "ST-FULL"
        elif "SLICE" in rule_id or "REFERENCE" in rule_id:
            capability = "BR-PROV"
        elif "ITER" in rule_id or "RANGE" in rule_id:
            capability = "IT-COMPOSE"
        elif "OPTION" in rule_id or "RESULT" in rule_id:
            capability = "EX-NORMAL"
        elif "COLLECTIONS" in rule_id:
            capability = "AB-GENERIC"
        elif "BOX" in rule_id:
            capability = "OW-MOVEOUT"
        return NeedRoute(
            "G0_CONTRACT",
            f"CAP:{capability}",
            "the declaration belongs to a named G0 data-floor obligation pending exact family closure",
        )

    if rule_id.startswith(("DOM-BEHAVIOR-CMP-", "DOM-BEHAVIOR-HASH-")):
        return NeedRoute(
            "G0_CONTRACT",
            "CAP:AB-BEHAVIOR",
            "statically callable equality, ordering, or hashing is a G0 behavior obligation",
        )
    if rule_id.startswith("DOM-BEHAVIOR-CLONE-"):
        return NeedRoute(
            "G0_CONTRACT",
            "CAP:OW-CLONE",
            "explicit clone is a G0 ownership obligation distinct from relocation",
        )
    if rule_id.startswith(("DOM-BEHAVIOR-BORROW-CORE", "DOM-BEHAVIOR-BORROW-STD")):
        return NeedRoute(
            "G0_CONTRACT",
            "CAP:BR-PROV",
            "borrowed views are a G0 provenance obligation",
        )
    if rule_id == "DOM-BEHAVIOR-BORROW-ALLOC":
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D08:SHARED-OWNED-BORROWING",
            "copy-on-write and shared owned borrowing require D08",
        )
    if rule_id.startswith(("DOM-BEHAVIOR-CONVERT-", "DOM-BEHAVIOR-DEFAULT-")):
        return NeedRoute(
            "LIB_CONTRACT",
            "LIB:D03:CHECKED-CONVERSION",
            "safe conversion and default construction are ordinary checked-library contracts",
        )

    if rule_id.startswith("DOM-OWN-MEM-"):
        if name == "swap":
            return NeedRoute(
                "G0_CONTRACT", "CAP:OW-SWAP", "whole-place swap is a G0 ownership atom"
            )
        if name == "replace":
            return NeedRoute(
                "G0_CONTRACT",
                "MEM-REPLACE-01",
                "whole-place replacement is routed to a coarse non-importable G0 coverage cluster",
            )
        if name == "take":
            return NeedRoute(
                "G0_CONTRACT",
                "MEM-TAKE-01",
                "whole-place take is routed to a coarse non-importable G0 coverage cluster",
            )
        if name in {"drop", "drop_in_place"}:
            return NeedRoute(
                "G0_CONTRACT", "CAP:OW-DROP", "exact destruction is a G0 ownership obligation"
            )
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D04:LAYOUT-AND-VALUE-TRANSITIONS",
            "general layout and value transitions require D04 closure",
            frames=("F-MEM",),
        )
    if rule_id.startswith("DOM-OWN-MARKER-"):
        if name in {"Copy", "Sized"}:
            return NeedRoute(
                "G0_CONTRACT",
                "CAP:NT-FIXED",
                "the protected fixed/Copy baseline is a G0 prerequisite",
            )
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D03:OWNERSHIP-CAPABILITY-PREDICATES",
            "thread, pin, and layout predicates require their owning later families",
        )
    if rule_id.startswith(("DOM-RAW-PTR-", "DOM-RAW-POINTER-")):
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D04:CHECKED-ADDRESS-PROVENANCE",
            "raw pointer operations require a checked D04 address/provenance family",
            frames=("F-MEM",),
        )

    if rule_id.startswith("DOM-ALLOC-"):
        if name in {"alloc", "alloc_zeroed", "dealloc", "handle_alloc_error"}:
            return NeedRoute(
                "FRAME",
                "FRAME:F-ALLOC:DEFAULT-HEAP-SERVICE",
                "the declaration is the allocation boundary service itself",
                frames=("F-ALLOC",),
            )
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D07:CUSTOM-ALLOCATION",
            "allocator selection and layout policy require D07 closure",
            frames=("F-ALLOC",),
        )
    if rule_id.startswith(("DOM-SHARED-RC-", "DOM-SHARED-ARC-")):
        frames = ("F-SYNC",) if "ARC" in rule_id else ()
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D08:SHARED-OWNERSHIP",
            "shared and weak ownership require a separate D08 family",
            frames=frames,
        )
    if rule_id.startswith("DOM-SHARED-CELL-"):
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D08:DYNAMIC-BORROWING",
            "interior mutation and runtime borrowing require D08 closure",
        )

    if rule_id.startswith("DOM-ERROR-"):
        if name.startswith("downcast"):
            return NeedRoute(
                "LATER_FAMILY",
                "FAMILY:D13:RUNTIME-TYPE-REFINEMENT",
                "dynamic error downcast depends on runtime type identity",
                domain_id="D13",
            )
        return NeedRoute(
            "LIB_CONTRACT",
            "LIB:D05:ERROR-VALUES",
            "general error values and chaining are checked-library contracts",
        )

    if rule_id.startswith("DOM-PANIC-MACROS"):
        return NeedRoute(
            "G0_CONTRACT",
            "CAP:EX-ABORT",
            "the underlying checked assertion or trap is a G0 abort contract",
            frames=("F-TRAP",),
        )
    if rule_id == "DOM-PANIC-BACKTRACE":
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D06:BACKTRACE-DIAGNOSTICS",
            "stack capture and symbolization require a diagnostic family",
            frames=("F-TRAP",),
        )
    if rule_id.startswith("DOM-PANIC-"):
        if name.startswith("catch_") or "unwind" in name:
            return NeedRoute(
                "NG",
                "NG:D06:PANIC-UNWIND-RECOVERY",
                "panic unwinding and recovery conflict with current EFF-4 abort semantics",
                frames=("F-TRAP",),
            )
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D06:TRAP-DIAGNOSTICS",
            "panic payload, hook, and diagnostic services require D06 closure",
            frames=("F-TRAP",),
        )
    if rule_id.startswith("DOM-FMT-MACROS"):
        return NeedRoute(
            "LIB_CONTRACT",
            "LIB:D12:FORMATTING",
            "the macro spelling exposes a formatting contract that belongs in checked libraries",
        )
    if rule_id.startswith("DOM-FMT-"):
        return NeedRoute(
            "LIB_CONTRACT",
            "LIB:D12:FORMATTING",
            "formatting is an ordinary library after text and behavior prerequisites",
        )
    if rule_id.startswith("DOM-REFLECT-"):
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D13:RUNTIME-TYPE-IDENTITY",
            "runtime type identity and reflection require D13 closure",
        )
    if rule_id.startswith("DOM-ASYNC-OPS-"):
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D23:ASYNC-TASK-CANCELLATION",
            "async callable protocols require suspension, task, and cancellation semantics",
            frames=("F-ASYNC",),
        )
    if rule_id.startswith("DOM-OWN-DROP-OPS-"):
        return NeedRoute(
            "G0_CONTRACT",
            "CAP:OW-DROP",
            "exact destruction is a G0 ownership and lifecycle obligation",
        )
    if rule_id.startswith("DOM-BEHAVIOR-OPS-"):
        if row["item_path"].rsplit("::", 1)[-1] in {"Deref", "DerefMut"}:
            return NeedRoute(
                "G0_CONTRACT",
                "TRAIT-DEREF-01",
                "borrowed owner projection is routed to a coarse non-importable G0 coverage cluster",
            )
        if row["item_path"].rsplit("::", 1)[-1] in {"Index", "IndexMut"}:
            return NeedRoute(
                "G0_CONTRACT",
                "TRAIT-INDEX-01",
                "checked borrowed indexing is routed to a coarse non-importable G0 coverage cluster",
            )
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D03:STATIC-OPERATOR-BEHAVIOR",
            "the complete generic operator-behavior family requires D03 closure",
        )
    if rule_id.startswith("DOM-LANG-CONTROL-OPS-"):
        return NeedRoute(
            "LIB_CONTRACT",
            "LIB:D02:CONTROL-FLOW-VALUES",
            "control-flow result values and combinators are ordinary checked-library contracts",
        )
    if rule_id.startswith("DOM-LANG-OPS-NAMESPACE-"):
        return NeedRoute(
            "REDUNDANT",
            "RED:D02:OPS-NAMESPACE",
            "the module namespace adds no independent caller capability",
        )
    if rule_id.startswith(("DOM-LANG-CALL-OPS-", "DOM-LANG-OPS-")):
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D02:CALLABLE-ABSTRACTION",
            "the complete operation/callable surface requires D02 closure",
        )
    if rule_id == "DOM-LANG-MATCHES":
        return NeedRoute(
            "G0_CONTRACT",
            "CAP:AB-BEHAVIOR",
            "the underlying match operation is a checked G0 language prerequisite",
        )
    if rule_id.startswith(("DOM-LANG-FUTURE-", "DOM-LANG-ASYNC-ITER-", "DOM-LANG-TASK-")):
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D23:ASYNC-TASK-CANCELLATION",
            "async tasks, wakeups, and cancellation require D23 closure",
            frames=("F-ASYNC",),
        )
    if rule_id.startswith("DOM-LANG-PIN-"):
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D23:PIN-ADDRESS-STABILITY",
            "pinning and address stability require D23 closure",
        )
    if rule_id.startswith("DOM-SYNC-"):
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D22:ATOMICS-AND-SYNCHRONIZATION",
            "memory ordering, synchronization, and reclamation require D22 closure",
            frames=("F-SYNC",),
        )
    if rule_id == "DOM-THREAD" or rule_id.startswith("DOM-THREAD-LOCAL"):
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D21:THREADS-AND-TLS",
            "threads, parking, joins, and TLS require D21 closure",
            frames=("F-THREAD", "F-SYNC"),
        )

    if rule_id == "DOM-IO-CORE":
        return NeedRoute(
            "LIB_CONTRACT",
            "LIB:D15:IO-ADAPTORS",
            "pure I/O interfaces and adaptors are ordinary checked libraries",
        )
    if rule_id == "DOM-IO-STD":
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D15:SYNCHRONOUS-IO",
            "OS-backed synchronous I/O requires D15 closure",
            frames=("F-IO",),
        )
    if rule_id in {"DOM-FS", "DOM-OS-FS"}:
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D16:FILESYSTEMS",
            "filesystem operations and resources require D16 closure",
            frames=("F-FS",),
        )
    if rule_id == "DOM-PATH":
        return NeedRoute(
            "LIB_CONTRACT",
            "LIB:D16:LEXICAL-PATHS",
            "lexical path manipulation is a checked library contract",
        )
    if rule_id in {"DOM-ENV", "DOM-PROCESS", "DOM-OS-PROCESS", "DOM-PROCESS-CORE"}:
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D17:PROCESS-ENVIRONMENT",
            "environment and process control require D17 closure",
            frames=("F-PROC",),
        )
    if rule_id.startswith("DOM-FFI-") or rule_id in {"DOM-OS", "DOM-OS-CORE"}:
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D18:NARROW-ABI-AND-OS-RESOURCES",
            "FFI and OS resources require a narrow D18 family",
            frames=("F-ABI",),
        )
    if rule_id in {"DOM-NET-CORE"}:
        return NeedRoute(
            "LIB_CONTRACT",
            "LIB:D19:NETWORK-ADDRESS-VALUES",
            "network address values and parsing are checked libraries",
        )
    if rule_id in {"DOM-NET-STD", "DOM-OS-NET"}:
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D19:NETWORKING",
            "sockets, DNS, and network resources require D19 closure",
            frames=("F-NET",),
        )
    if rule_id == "DOM-TIME-CORE":
        return NeedRoute(
            "LIB_CONTRACT",
            "LIB:D20:DURATION-VALUES",
            "duration arithmetic is a checked value library",
        )
    if rule_id == "DOM-TIME-STD":
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D20:CLOCKS-AND-TIMERS",
            "clock reads, sleep, and timers require D20 closure",
            frames=("F-CLOCK",),
        )
    if rule_id == "DOM-OS-THREAD":
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D21:THREADS-AND-TLS",
            "platform thread extensions require D21 closure",
            frames=("F-THREAD",),
        )

    if rule_id.startswith("DOM-BOOL-"):
        return NeedRoute(
            "G0_CONTRACT",
            "CAP:NT-FIXED",
            "the protected fixed/Copy scalar baseline is a G0 prerequisite",
        )
    if rule_id.startswith(("DOM-TUPLE-", "DOM-UNIT-")):
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D01:AGGREGATE-VALUES",
            "product and unit values require an explicit aggregate-value family disposition",
        )
    if rule_id.startswith(("DOM-PRIMITIVE-", "DOM-PRELUDE-")):
        return NeedRoute(
            "REDUNDANT",
            f"CANON:{rule_id}",
            "the namespace or prelude rendering refers to canonical declarations elsewhere",
        )
    if rule_id.startswith("DOM-REFERENCE-"):
        return NeedRoute(
            "G0_CONTRACT", "CAP:BR-PROV", "checked references are a G0 provenance obligation"
        )
    if rule_id == "DOM-FN-CORE":
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D02:CALLABLE-ABSTRACTION",
            "general function values and callable abstraction require D02 closure",
        )

    if rule_id == "DOM-BUILD-GLOBAL-ALLOCATOR":
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D14:BUILD-REGISTRATION",
            "allocator registration is a build-system contract",
            frames=("F-BUILD", "F-ALLOC"),
        )
    if rule_id == "DOM-BUILD-TEST-ATTRIBUTE":
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D14:BUILD-REGISTRATION",
            "test discovery is a build-system contract",
            frames=("F-BUILD",),
        )
    if rule_id == "DOM-SURFACE-KEYWORD":
        return NeedRoute(
            "NO_INDEPENDENT_NEED",
            "NAMESPACE:RUST-KEYWORD",
            "a Rust keyword page is not an independent library capability",
        )
    if rule_id == "DOM-SURFACE-MACRO":
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D14:MACRO-AND-BUILD-SURFACE",
            "an unmatched Rust macro is evidence for later macro/build accounting",
            frames=("F-BUILD",),
        )
    if rule_id in {"DOM-SURFACE-ATTR", "DOM-SURFACE-DERIVE"}:
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D14:GENERATED-AND-BUILD-SURFACE",
            "attribute and derive behavior requires D14 build/generation accounting",
            frames=("F-BUILD",),
        )

    if rule_id.startswith("DOM-UNSTABLE-LANG-"):
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D02:LANGUAGE-ABSTRACTION",
            "the declaration belongs to a later language-abstraction family",
        )
    if rule_id.startswith("DOM-UNSTABLE-TARGET-UB-CHECKS"):
        return NeedRoute(
            "NO_INDEPENDENT_NEED",
            "CAP:EX-ABORT",
            "compiler UB-check plumbing has no independent writer need",
            frames=("F-TARGET",),
        )
    if rule_id.startswith("DOM-UNSTABLE-TARGET-BINDER"):
        return NeedRoute(
            "NG",
            "NG:D24:UNSAFE-BINDER-AUTHORITY",
            "unsafe binder authority is intrinsically a writer trust escape",
        )
    if rule_id.startswith("DOM-UNSTABLE-TARGET-"):
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D24:TARGET-TOOLING",
            "target tooling requires D24 closure",
            frames=("F-TARGET",),
        )
    if rule_id.startswith("DOM-UNSTABLE-RANDOM-"):
        return NeedRoute(
            "LATER_FAMILY",
            "FAMILY:D26:RANDOMNESS-AND-ENTROPY",
            "random algorithms and entropy require separate runtime accounting",
            frames=("F-ABI",),
        )

    return NeedRoute(
        "LATER_FAMILY",
        f"FAMILY:{DOMAIN_IDS[rule['domain']]}:UNNORMALIZED-STABLE-SURFACE",
        "the stable declaration remains an explicit later-domain normalization obligation",
    )


def route_for(
    row: dict[str, str], rule: dict[str, str], surface_route: SurfaceRoute | None
) -> NeedRoute:
    if surface_route is not None:
        return contract_need_route(row, surface_route)
    return default_route(row, rule)


def boundary_reason(
    row: dict[str, str], rule: dict[str, str], surface_route: SurfaceRoute | None
) -> str | None:
    if row["item_kind"] == "mod":
        return None
    if row["caller_safety"] == "unsafe":
        return "Rust requires caller-provided safety obligations"
    contract_id = surface_route.contract_id if surface_route is not None else ""
    if contract_id.startswith("RAW-SAFE-"):
        return "safe Rust surface exports a raw address, ownership, lifetime, or spare-capacity obligation"
    rule_id = rule["rule_id"]
    text = " ".join((row["item_path"], row["member_name"], row["signature"]))
    if rule_id.startswith(("DOM-RAW-PTR-", "DOM-RAW-POINTER-")):
        return "safe Rust surface still exposes raw-pointer or provenance state"
    if any(marker in text for marker in ("*const", "*mut", "MaybeUninit", "ManuallyDrop", "UnsafeCell")):
        return "safe Rust surface crosses raw-pointer, partial-initialization, or manual-lifetime state"
    if row["member_name"] in {"forget", "forget_unsized", "leak"}:
        return "safe Rust surface suppresses ordinary lifetime completion"
    if is_raw_ownership(row):
        return "safe Rust surface crosses a raw address or ownership boundary"
    if "unsafe trait" in row["signature"]:
        return "Rust marks the implemented behavior contract unsafe"
    return None


def surface_evidence(
    row: dict[str, str], rule: dict[str, str], surface_route: SurfaceRoute | None
) -> tuple[str, str]:
    reason = boundary_reason(row, rule, surface_route)
    if reason is not None:
        status = (
            "unsafe_boundary_evidence"
            if row["caller_safety"] == "unsafe"
            else "safe_boundary_evidence"
        )
        return status, reason
    if row["item_kind"] in {"macro", "keyword", "derive", "attr", "mod"}:
        return (
            "rust_surface_only",
            "the Rust namespace or source spelling is evidence for an underlying need, not an xlang surface",
        )
    if surface_route is not None and surface_route.route_kind == "redundant_surface":
        return (
            "rust_surface_only",
            "the exact iteration crosswalk identifies a redundant Rust rendering",
        )
    return (
        "safe_contract_anchor",
        "the safe Rust declaration supplies caller-contract evidence without selecting xlang admission",
    )


def safe_displacement_id(row: dict[str, str], route: NeedRoute) -> str:
    if "volatile" in row["member_name"]:
        return "FAMILY:D24:CHECKED-VOLATILE-MMIO"
    if is_partial_initialization(row):
        if "::Rc" in row["item_path"] or "::Arc" in row["item_path"]:
            return "FAMILY:D08:SHARED-PARTIAL-INITIALIZATION"
        return "CAP:OW-INIT"
    if row["member_name"] in {"forget", "forget_unsized", "leak"}:
        return "FAMILY:D04:EXPLICIT-RESOURCE-ABANDONMENT"
    if route.route_id == "RAW-SAFE-PTR-01":
        return "FAMILY:D04:CHECKED-ADDRESS-PROVENANCE"
    if route.route_id == "RAW-SAFE-OWNERSHIP-01":
        return "FAMILY:D04:CHECKED-OWNERSHIP-TRANSFER"
    if route.route_id == "RAW-SAFE-LEAK-01":
        return "FAMILY:D04:EXPLICIT-RESOURCE-ABANDONMENT"
    if route.kind == "NG":
        if route.route_id == "NG:D06:PANIC-UNWIND-RECOVERY":
            return "CAP:EX-ABORT"
        if "INTRINSIC" in route.route_id or "BINDER" in route.route_id:
            return "FAMILY:D24:PROOF-DERIVED-TARGET-OPERATIONS"
        return "CAP:EX-ABORT"
    return route.route_id


def ng_authority_reason(route: NeedRoute) -> str:
    if route.kind != "NG":
        return ""
    if route.route_id == "NG:D06:PANIC-UNWIND-RECOVERY":
        return "EFF-4 and owner directive D12 retain aborting traps and exclude panic unwinding from the current language"
    return "CONSTITUTION W3 forbids writer-emittable trust or unchecked authority; owner directive D12 treats unsafe APIs as evidence only"


def validate_route(
    row: dict[str, str], route: NeedRoute, capability_ids: set[str], contract_ids: set[str]
) -> None:
    if route.kind not in VALID_NEED_ROUTE_KINDS:
        raise SystemExit(f"invalid need route kind for {row['canonical_key']}: {route.kind}")
    if not route.route_id:
        raise SystemExit(f"missing need route ID for {row['canonical_key']}")
    if len(route.frames) != len(set(route.frames)) or any(
        frame not in VALID_FRAME_IDS for frame in route.frames
    ):
        raise SystemExit(f"invalid frame set for {row['canonical_key']}: {route.frames}")
    if route.kind == "G0_CONTRACT":
        if route.route_id in contract_ids:
            return
        if route.route_id.startswith("CAP:") and route.route_id[4:] in capability_ids:
            return
        if re.fullmatch(r"SPEC:[A-Z]+-[0-9]+", route.route_id):
            return
        raise SystemExit(
            f"G0 route lacks stable contract, obligation, or spec ID for {row['canonical_key']}: "
            f"{route.route_id}"
        )


def canonical_rows() -> dict[str, dict[str, str]]:
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
    return by_key


def main() -> None:
    rules = load_rules()
    contract_ids = load_ids(CONTRACT_CENSUS, "contract_id")
    capability_ids = load_ids(CAPABILITY_REGISTRY, "capability_id")
    surface_routes = load_surface_routes(contract_ids)
    by_key = canonical_rows()

    output: list[dict[str, str]] = []
    unclassified: list[str] = []
    for key, row in sorted(
        by_key.items(),
        key=lambda pair: (pair[1]["item_path"], pair[1]["member_name"], pair[0]),
    ):
        rule = selected_rule(row, rules)
        if rule is None:
            unclassified.append(
                f"{row['item_kind']}\t{row['item_path']}\t{row['member_name']}"
            )
            continue
        surface_route = surface_routes.get(key)
        route = route_for(row, rule, surface_route)
        validate_route(row, route, capability_ids, contract_ids)
        status, evidence_reason = surface_evidence(row, rule, surface_route)
        if status not in VALID_SURFACE_EVIDENCE_STATUSES:
            raise SystemExit(f"invalid surface evidence status for {key}: {status}")
        boundary = status in {"safe_boundary_evidence", "unsafe_boundary_evidence"}
        displacement = (
            safe_displacement_id(row, route)
            if boundary or route.kind == "NG"
            else ""
        )
        authority = ng_authority_reason(route)
        if boundary and not displacement:
            raise SystemExit(f"boundary evidence lacks safe displacement: {key}")
        if route.kind == "NG" and not displacement:
            raise SystemExit(f"NG route lacks safe displacement: {key}")
        if (route.kind == "NG") != bool(authority):
            raise SystemExit(f"NG authority invariant failed for {key}")

        domain_id = route.domain_id or domain_override(row, surface_route) or DOMAIN_IDS[rule["domain"]]
        domain = DOMAIN_OVERRIDES.get(domain_id, rule["domain"])
        output.append(
            {
                "canonical_key": key,
                "representative_path": row["item_path"],
                "item_kind": row["item_kind"],
                "member_kind": row["member_kind"],
                "member_name": row["member_name"],
                "caller_safety": row["caller_safety"],
                "surface_evidence_status": status,
                "surface_evidence_reason": evidence_reason,
                "rule_id": rule["rule_id"],
                "domain_id": domain_id,
                "domain": domain,
                "canonical_contract_id": (
                    surface_route.contract_id if surface_route is not None else ""
                ),
                "need_route_kind": route.kind,
                "need_route_id": route.route_id,
                "need_route_reason": route.reason,
                "required_frame_ids": ";".join(route.frames),
                "safe_displacement_id": displacement,
                "ng_authority_reason": authority,
                "canonical_route_or_blocked_claim": rule[
                    "canonical_route_or_blocked_claim"
                ],
            }
        )

    if unclassified:
        print("unclassified canonical stable declarations:")
        print("\n".join(unclassified))
        raise SystemExit(1)
    safety_counts = Counter(row["caller_safety"] for row in output)
    if len(output) != 5555 or safety_counts != {"safe": 5278, "unsafe": 277}:
        raise SystemExit(
            "expected 5555 canonical stable rows (5278 safe, 277 unsafe), "
            f"got {len(output)} and {dict(safety_counts)}"
        )

    fields = list(output[0])
    with OUTPUT.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(
            handle, fieldnames=fields, delimiter="\t", lineterminator="\n"
        )
        writer.writeheader()
        writer.writerows(output)

    counts = Counter(
        (
            row["domain_id"],
            row["domain"],
            row["caller_safety"],
            row["surface_evidence_status"],
            row["need_route_kind"],
        )
        for row in output
    )
    with SUMMARY.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.writer(handle, delimiter="\t", lineterminator="\n")
        writer.writerow(
            [
                "domain_id",
                "domain",
                "caller_safety",
                "surface_evidence_status",
                "need_route_kind",
                "canonical_stable_declarations",
            ]
        )
        for key, count in sorted(counts.items()):
            writer.writerow([*key, count])

    with MODULES.open(encoding="utf-8", newline="") as handle:
        raw_modules = list(csv.DictReader(handle, delimiter="\t"))
    module_output: list[dict[str, str]] = []
    for row in raw_modules:
        synthetic = {
            "item_path": row["module_path"],
            "item_kind": "mod",
            "member_name": row["module_path"].rsplit("::", 1)[-1],
        }
        rule = selected_rule(synthetic, rules)
        if rule is None and row["module_path"] in {"core", "alloc", "std"}:
            rule_id = "DOM-MODULE-ROOT"
            domain_id = "D25"
            domain = "prelude and reexports"
        elif rule is None and row["direct_stable_items"] == "0":
            rule_id = "DOM-UNSTABLE-RUNTIME-HOLDING"
            domain_id = "D26"
            domain = "compiler and runtime support"
        elif rule is None:
            raise SystemExit(f"unclassified reachable module: {row['module_path']}")
        else:
            rule_id = rule["rule_id"]
            domain_id = DOMAIN_IDS[rule["domain"]]
            domain = rule["domain"]
        module_output.append(
            {
                "crate": row["crate"],
                "module_path": row["module_path"],
                "mode": row["mode"],
                "rule_id": rule_id,
                "domain_id": domain_id,
                "domain": domain,
                "module_route_kind": "NO_INDEPENDENT_NEED",
                "module_route_id": f"NAMESPACE:{rule_id}",
                "entry_digest": row["entry_digest"],
            }
        )
    if len(module_output) != 297:
        raise SystemExit(f"expected 297 reachable modules, got {len(module_output)}")
    with MODULE_OUTPUT.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(
            handle,
            fieldnames=list(module_output[0]),
            delimiter="\t",
            lineterminator="\n",
        )
        writer.writeheader()
        writer.writerows(module_output)

    evidence_counts = Counter(row["surface_evidence_status"] for row in output)
    route_counts = Counter(row["need_route_kind"] for row in output)
    print(
        "rust domain classification: PASS — 5555 canonical stable declarations "
        "accounted (5278 safe, 277 unsafe); 297 reachable modules routed; "
        f"evidence={dict(sorted(evidence_counts.items()))}; "
        f"needs={dict(sorted(route_counts.items()))}"
    )


if __name__ == "__main__":
    main()
