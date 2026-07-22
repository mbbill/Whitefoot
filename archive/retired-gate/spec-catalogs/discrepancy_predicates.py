#!/usr/bin/env python3
"""Recompute the closed exact-v0.9 discrepancy predicate registry."""

from __future__ import annotations

import json
import re
from bisect import bisect_right
from dataclasses import dataclass
from typing import Any, Mapping, Sequence

try:
    import discrepancy_inputs as inputs
except ModuleNotFoundError:  # Support import as ``tools.*``.
    from tools import discrepancy_inputs as inputs  # type: ignore

DiscrepancyError = inputs.DiscrepancyError

MAX_MANIFEST_ENTRIES = 20_000

IDENT = re.compile(rb"[a-z][a-z0-9_]*")
OP_NAME = re.compile(rb"`([a-z_]+(?:\.[a-z]+)?)`")
DOTLESS_LIST = re.compile(rb"or a dotless IDENT \(`([^`\r\n]+)`\); both")


@dataclass(frozen=True)
class ExactSource:
    """Exact bytes plus byte-to-line coordinates for evidence anchors."""

    path: str
    raw: bytes
    line_offsets: tuple[int, ...]

    @classmethod
    def from_bytes(cls, path: str, raw: bytes) -> "ExactSource":
        offsets = [0]
        for line in raw.splitlines(keepends=True):
            offsets.append(offsets[-1] + len(line))
        return cls(path, raw, tuple(offsets))

    def evidence(self, byte_start: int, byte_end: int) -> dict[str, Any]:
        if not 0 <= byte_start < byte_end <= len(self.raw):
            raise DiscrepancyError(
                f"invalid evidence span {self.path}:{byte_start}-{byte_end}"
            )
        return {
            "byte_end": byte_end,
            "byte_start": byte_start,
            "line_end": bisect_right(self.line_offsets, byte_end - 1),
            "line_start": bisect_right(self.line_offsets, byte_start),
            "path": self.path,
            "sha256": inputs.sha256(self.raw[byte_start:byte_end]),
        }

    def unique(self, fragment: bytes) -> tuple[int, int]:
        count = self.raw.count(fragment)
        if count != 1:
            raise DiscrepancyError(
                f"expected one exact fragment in {self.path}, found {count}: "
                f"{fragment!r}"
            )
        start = self.raw.index(fragment)
        return start, start + len(fragment)


@dataclass(frozen=True)
class Observation:
    """The recomputed truth value and exact evidence of one predicate."""

    is_open: bool
    evidence: dict[str, Any]


@dataclass(frozen=True)
class Registration:
    identifier: str
    discrepancy_class: str
    predicate_identifier: str
    affected_facet_ids: tuple[str, ...]
    resolution_authorities: tuple[str, ...]


REGISTRATIONS = (
    Registration(
        "discrepancy:v0.9/affine-deref-storage-lifecycle",
        "internal-specification-gap",
        "predicate:affine-deref-storage-lifecycle-completeness",
        (
            "facet:STOR-3/deallocation-compiler-derived",
            "facet:STOR-3/drop-and-arena-release-artifact-operations",
        ),
        ("successor-numbered-specification",),
    ),
    Registration(
        "discrepancy:v0.9/diag3-retained-proof-ref",
        "internal-specification-gap",
        "predicate:diag3-retained-check-proof-ref-completeness",
        ("facet:DIAG-3/check-report-schema",),
        ("successor-numbered-specification",),
    ),
    Registration(
        "discrepancy:v0.9/eff1-row-canonicality",
        "specification-protected-surface-conflict",
        "predicate:eff1-row-canonicality-completeness",
        (
            "facet:EFF-1/canonical-effect-order",
            "facet:EFF-1/effect-row-grammar",
        ),
        (
            "owner-approved-protected-surface-change",
            "successor-numbered-specification",
        ),
    ),
    Registration(
        "discrepancy:v0.9/eff2-local-region-effects",
        "internal-specification-inconsistency",
        "predicate:eff2-local-region-effect-row-consistency",
        (
            "facet:EFF-2/effect-row-bidirectional-exactness",
            "facet:EFF-2/syntactic-effect-exhibit-closure",
            "facet:EX-1/byte-exact-canonical-program",
            "facet:FN-7/main-effect-ceiling",
        ),
        ("successor-numbered-specification",),
    ),
    Registration(
        "discrepancy:v0.9/fn3-contract-member-semantics",
        "internal-specification-gap",
        "predicate:fn3-contract-member-semantics-completeness",
        (
            "facet:FN-3/contract-member-checking-boundary",
            "facet:FN-5/behavior-parameterization",
            "facet:FN-5/env-struct-direct-calls",
        ),
        ("successor-numbered-specification",),
    ),
    Registration(
        "discrepancy:v0.9/op1-dotless-reservation",
        "internal-specification-ambiguity",
        "predicate:op1-dotless-reservation-set-equality",
        ("facet:OP-1/dotless-operation-reservation",),
        ("successor-numbered-specification",),
    ),
    Registration(
        "discrepancy:v0.9/fn7-main-return-spelling",
        "internal-specification-inconsistency",
        "predicate:fn7-main-return-spelling-consistency",
        (
            "facet:EX-1/byte-exact-canonical-program",
            "facet:FN-7/main-return-spelling",
            "facet:GRAM-2/function-and-contract-declaration-shapes",
            "facet:GRAM-3/return-mode-type-shape",
        ),
        ("successor-numbered-specification",),
    ),
)


def _pin(actual: Any, expected: Any, label: str) -> None:
    if inputs.canonical_bytes(actual) != inputs.canonical_bytes(expected):
        raise DiscrepancyError(f"{label} no longer matches the pinned audit")


def _exact_fragment_sources(
    source: ExactSource,
    fragments: Mapping[str, bytes],
    expected: Mapping[str, Mapping[str, Any]],
    label: str,
    *,
    enforce_pins: bool,
) -> dict[str, Any]:
    """Locate unique exact fragments and optionally pin their spans and hashes."""
    evidence: dict[str, Any] = {}
    audit: dict[str, Any] = {}
    for name, fragment in fragments.items():
        span = source.unique(fragment)
        record = source.evidence(*span)
        evidence[f"{name}_source"] = record
        audit[name] = {
            "byte_end": span[1],
            "byte_start": span[0],
            "sha256": record["sha256"],
        }
    if enforce_pins:
        _pin(audit, expected, label)
    return evidence


def _protected_case_evidence(
    manifest: bytes,
    case_sources: Mapping[str, bytes],
    protected_surface: Mapping[str, Any],
    identifier: str,
) -> dict[str, Any]:
    """Validate the protected surface and return one exact case projection."""
    entries, cases = _manifest_entries(manifest)
    protected_ids = _protected_surface(
        entries,
        cases,
        case_sources,
        protected_surface,
    )
    if identifier not in protected_ids or identifier not in cases:
        raise DiscrepancyError(f"protected case is missing: {identifier}")
    path = f"tests/conformance/cases/{identifier}.wf"
    if path not in case_sources:
        raise DiscrepancyError(f"protected case source is missing: {path}")
    entry = cases[identifier]
    return {
        "manifest": {
            field: entry.get(field)
            for field in ("id", "rules", "expect", "status")
        },
        "path": path,
        "sha256": inputs.sha256(case_sources[path]),
    }


def observe_op1(specification: bytes, *, enforce_pins: bool = True) -> Observation:
    """Compare OP-1 table dotless names with its explicit listed set."""
    source = ExactSource.from_bytes("spec/kernel-spec-v0.9.md", specification)
    header = b"| op | domain | signature | effects |\n"
    table_start, _ = source.unique(header)
    table_end = specification.find(b"\n\n", table_start)
    if table_end < 0:
        raise DiscrepancyError("OP-1 table has no terminating blank line")
    table_end += 1
    lines = specification[table_start:table_end].splitlines(keepends=True)
    if len(lines) < 3 or lines[:2] != [
        header,
        b"|---|---|---|---|\n",
    ]:
        raise DiscrepancyError("OP-1 table header is not the audited shape")

    occurrences: list[str] = []
    for line in lines[2:]:
        cells = line.split(b"|")
        if len(cells) != 6 or cells[0] or cells[-1] != b"\n":
            raise DiscrepancyError("OP-1 table row is malformed")
        names = [match.decode("ascii") for match in OP_NAME.findall(cells[1])]
        if not names:
            raise DiscrepancyError("OP-1 table row has no operation name")
        occurrences.extend(names)

    table_dotless = list(dict.fromkeys(name for name in occurrences if "." not in name))
    listed_match = DOTLESS_LIST.search(specification, table_end)
    if listed_match is None:
        raise DiscrepancyError("OP-1 explicit dotless identifier list is missing")
    listed = listed_match.group(1).decode("ascii").split(" ")
    if len(listed) != len(set(listed)) or any(
        IDENT.fullmatch(name.encode("ascii")) is None for name in listed
    ):
        raise DiscrepancyError("OP-1 explicit dotless list is malformed or duplicated")
    table_set = set(table_dotless)
    listed_set = set(listed)
    table_only = [name for name in table_dotless if name not in listed_set]
    listed_only = [name for name in listed if name not in table_set]
    evidence = {
        "listed_distinct_dotless_count": len(listed),
        "listed_dotless_identifiers": listed,
        "listed_only_count": len(listed_only),
        "listed_only_identifiers": listed_only,
        "listed_source": source.evidence(listed_match.start(1), listed_match.end(1)),
        "operation_name_occurrence_count": len(occurrences),
        "operation_row_count": len(lines) - 2,
        "table_distinct_dotless_count": len(table_dotless),
        "table_dotless_identifiers": table_dotless,
        "table_only_count": len(table_only),
        "table_only_identifiers": table_only,
        "table_source": source.evidence(table_start, table_end),
    }
    if enforce_pins:
        _pin(
            {
                "listed_count": len(listed),
                "listed_sha256": evidence["listed_source"]["sha256"],
                "listed_only_count": len(listed_only),
                "occurrences": len(occurrences),
                "rows": len(lines) - 2,
                "table_count": len(table_dotless),
                "table_only_count": len(table_only),
                "table_sha256": evidence["table_source"]["sha256"],
                "unique_operations": len(set(occurrences)),
            },
            {
                "listed_count": 20,
                "listed_sha256": "bca1f3a8ad911092756f1f18a459de95cd91062991b837b792e9d9de78fd41fc",
                "listed_only_count": 0,
                "occurrences": 84,
                "rows": 44,
                "table_count": 51,
                "table_only_count": 31,
                "table_sha256": "415a65e25e5c070ccbb7a51ebfb0b3d4ff2a8c42f2f151d3a23720198c352297",
                "unique_operations": 83,
            },
            "OP-1 discrepancy evidence",
        )
    return Observation(table_set != listed_set, evidence)


def _manifest_entries(raw: bytes) -> tuple[list[dict[str, Any]], dict[str, dict[str, Any]]]:
    entries: list[dict[str, Any]] = []
    cases: dict[str, dict[str, Any]] = {}
    for line_number, line in enumerate(raw.splitlines(), 1):
        if not line.strip() or line.lstrip().startswith(b"#"):
            continue
        if len(entries) >= MAX_MANIFEST_ENTRIES:
            raise DiscrepancyError(
                f"manifest exceeds {MAX_MANIFEST_ENTRIES} semantic entries"
            )
        value = inputs.strict_json_loads(
            line,
            max_bytes=inputs.MAX_MANIFEST_BYTES,
            label=f"manifest line {line_number}",
        )
        if not isinstance(value, dict):
            raise DiscrepancyError(f"manifest line {line_number} is not an object")
        entries.append(value)
        if "id" in value:
            identifier = value["id"]
            if not isinstance(identifier, str) or inputs.CASE_ID.fullmatch(identifier) is None:
                raise DiscrepancyError(f"invalid manifest case id at line {line_number}")
            if identifier in cases:
                raise DiscrepancyError(f"duplicate manifest case id: {identifier}")
            cases[identifier] = value
    return entries, cases


def _protected_surface(
    entries: Sequence[dict[str, Any]],
    manifest_cases: Mapping[str, dict[str, Any]],
    case_sources: Mapping[str, bytes],
    protected: Mapping[str, Any],
) -> tuple[str, ...]:
    """Recompute every baseline entry while ignoring additive entries."""
    live: dict[str, str] = {}
    for entry in entries:
        if "id" in entry and entry["id"] in protected:
            identifier = entry["id"]
            path = f"tests/conformance/cases/{identifier}.wf"
            if path not in case_sources:
                raise DiscrepancyError(f"protected manifest case has no source: {path}")
            projection = {
                field: entry.get(field)
                for field in ("id", "rules", "expect", "status")
            }
            encoded = json.dumps(
                projection, ensure_ascii=True, sort_keys=True, separators=(",", ":")
            ).encode("ascii")
            key = identifier
            digest = inputs.sha256(encoded + b"\0" + case_sources[path])
        elif "rule" in entry and f"rule:{entry['rule']}" in protected:
            key = f"rule:{entry['rule']}"
            digest = inputs.sha256(
                json.dumps(
                    entry, ensure_ascii=True, sort_keys=True, separators=(",", ":")
                ).encode("ascii")
            )
        else:
            continue
        if key in live:
            raise DiscrepancyError(f"duplicate protected-surface key in manifest: {key}")
        live[key] = digest

    case_ids = []
    for key, expected_digest in protected.items():
        if not isinstance(key, str):
            raise DiscrepancyError("guard baseline conformance key must be a string")
        if not isinstance(expected_digest, str) or inputs.HEX_SHA256.fullmatch(
            expected_digest
        ) is None:
            raise DiscrepancyError(
                f"guard baseline conformance digest is invalid for {key!r}"
            )
        if key not in live:
            raise DiscrepancyError(f"protected conformance entry is missing: {key}")
        if live[key] != expected_digest:
            raise DiscrepancyError(f"protected conformance entry changed: {key}")
        if inputs.CASE_ID.fullmatch(key):
            if key not in manifest_cases:
                raise DiscrepancyError(f"protected manifest case is missing: {key}")
            case_ids.append(key)
    case_ids.sort(key=str.encode)
    return tuple(case_ids)


def observe_fn7(specification: bytes, *, enforce_pins: bool = True) -> Observation:
    """Compare FN-7's main return spelling with grammar and EX-1."""
    source = ExactSource.from_bytes("spec/kernel-spec-v0.9.md", specification)
    fn_decl = (
        b'fn_decl      := "fn" IDENT generics? region_params? "(" param_list? ")"\n'
        b'                "->" rtype effects requires_block? "{" doc? stmt* "}"\n'
    )
    rtype = b"rtype  := mode type\n"
    fn7_signature = b"fn main() -> unit"
    example_line = b"fn main() -> own unit traps {\n"
    fn_decl_span = source.unique(fn_decl)
    rtype_span = source.unique(rtype)
    fn7_span = source.unique(fn7_signature)
    example_span = source.unique(example_line)
    evidence = {
        "example_main_return_spelling": "own unit",
        "example_main_source": source.evidence(*example_span),
        "fn7_main_return_spelling": "unit",
        "fn7_main_source": source.evidence(*fn7_span),
        "fn_decl_return_nonterminal": "rtype",
        "fn_decl_source": source.evidence(*fn_decl_span),
        "rtype_shape": "mode type",
        "rtype_source": source.evidence(*rtype_span),
    }
    if enforce_pins:
        _pin(
            {
                key: evidence[key]["sha256"]
                for key in (
                    "example_main_source",
                    "fn7_main_source",
                    "fn_decl_source",
                    "rtype_source",
                )
            },
            {
                "example_main_source": (
                    "3c21e58f403384c5f0b6f119e1c6e64"
                    "e916b48bea8a58355724ec5a9f4642165"
                ),
                "fn7_main_source": (
                    "2687d72b3742432ba69de3203f96d293"
                    "08347ac8825558f829fa766f6a3b8fc8"
                ),
                "fn_decl_source": (
                    "7937beaad997465d1e80dcc0eae3573d"
                    "3de754bd5b7762a561760bf98adc9508"
                ),
                "rtype_source": "c9e5b6dead005a9feb5a7adb849ce60d0bc0dcd6051f3b64892a0f7c383eeac4",
            },
            "FN-7 discrepancy evidence",
        )
    return Observation(True, evidence)


def observe_affine_deref_lifecycle(
    specification: bytes,
    *,
    enforce_pins: bool = True,
) -> Observation:
    """Record affine referent move permission beside unspecified cleanup."""
    source = ExactSource.from_bytes("spec/kernel-spec-v0.9.md", specification)
    evidence = _exact_fragment_sources(
        source,
        {
            "own1_partial_move": (
                b"After a move, the whole binding rooting `p` is dead "
                b"(partial moves kill the whole binding)"
            ),
            "stor2_deref_access": b"Content access is through `deref`.",
            "stor3_derived_drop": (
                b"every drop and arena release appears as an explicit operation "
                b"in the elaborated artifact"
            ),
            "type7_affine_move": (
                b"a use of that place copies it when T is copy and requires `move` "
                b"when T is affine [OWN-1]"
            ),
        },
        {
            "own1_partial_move": {
                "byte_end": 36990,
                "byte_start": 36900,
                "sha256": (
                    "385e180cf9a88c4fc0a2673b3a2a710f"
                    "a8f27b7006dcef1c74d7c3acae2404fc"
                ),
            },
            "stor2_deref_access": {
                "byte_end": 45822,
                "byte_start": 45788,
                "sha256": (
                    "4e6f6d92f02bc31211cf29b2c676cd18"
                    "91dffcb246752d9ed97cf2c70b57ca25"
                ),
            },
            "stor3_derived_drop": {
                "byte_end": 45977,
                "byte_start": 45889,
                "sha256": (
                    "450ffec2a8008c821a4181e34333a4c55"
                    "bd938408183f45d4554779ea803dfad"
                ),
            },
            "type7_affine_move": {
                "byte_end": 34088,
                "byte_start": 33999,
                "sha256": (
                    "9e03775a3ee20b6d5779bb6fbe51ab8c"
                    "a921cf58907d431e07a04714e86c0283"
                ),
            },
        },
        "affine deref lifecycle evidence",
        enforce_pins=enforce_pins,
    )
    evidence["missing_contract"] = (
        "backing-allocation-and-remaining-payload-disposition-after-affine-referent-move"
    )
    return Observation(True, evidence)


def observe_eff1_row_canonicality(
    specification: bytes,
    manifest: bytes,
    case_sources: Mapping[str, bytes],
    protected_surface: Mapping[str, Any],
    *,
    enforce_pins: bool = True,
) -> Observation:
    """Record underdefined row canonicality and the protected rejection."""
    source = ExactSource.from_bytes("spec/kernel-spec-v0.9.md", specification)
    evidence = _exact_fragment_sources(
        source,
        {
            "effect_kind_order": (
                b"in exactly this canonical order (reads, writes, allocates, traps)"
            ),
            "row_grammar": b"Row grammar: `effects := \"pure\" | effect (\",\" effect)*`",
        },
        {
            "effect_kind_order": {
                "byte_end": 74566,
                "byte_start": 74501,
                "sha256": (
                    "8294f4112b0cca9c4c12fd43e3a1b702"
                    "9187dd480248961b653f58dbddbb0763"
                ),
            },
            "row_grammar": {
                "byte_end": 74366,
                "byte_start": 74311,
                "sha256": (
                    "bca021af57224f0154c77083fe0baf3e"
                    "d3b0a0b88aea8e2b9189b934cd59ea15"
                ),
            },
        },
        "EFF-1 row-canonicality evidence",
        enforce_pins=enforce_pins,
    )
    evidence["protected_case"] = _protected_case_evidence(
        manifest,
        case_sources,
        protected_surface,
        "x-eff-dup-reads-effect",
    )
    evidence["missing_contract"] = (
        "effect-kind-and-region-or-allocation-entry-uniqueness-and-order"
    )
    return Observation(True, evidence)


def observe_eff2_local_region_effects(
    specification: bytes,
    manifest: bytes,
    case_sources: Mapping[str, bytes],
    protected_surface: Mapping[str, Any],
    *,
    enforce_pins: bool = True,
) -> Observation:
    """Record exact row checking where body-local regions cannot be named."""
    source = ExactSource.from_bytes("spec/kernel-spec-v0.9.md", specification)
    evidence = _exact_fragment_sources(
        source,
        {
            "eff2_bidirectional_check": (
                b"Rows are checked both ways against the syntactic definition"
            ),
            "eff2_local_exhibits": (
                b"they exhibit reads/writes/allocates per the operation table "
                b"and borrow modes they use"
            ),
            "example_local_region": (
                b"region 'r {\n    let p: &'r i32 = &'r a;"
            ),
            "fn7_main_effect_ceiling": (
                b"effect row at most `allocates(heap), traps`"
            ),
        },
        {
            "eff2_bidirectional_check": {
                "byte_end": 75131,
                "byte_start": 75072,
                "sha256": (
                    "eb726a4ce9068a4da2877415481fa8539"
                    "96b5f2679610f97460a373b44fd8096"
                ),
            },
            "eff2_local_exhibits": {
                "byte_end": 75070,
                "byte_start": 74985,
                "sha256": (
                    "85180ce9cdf2eddb8a9cbc45651832517"
                    "dd2258bede1efd8aa974e411ef2a07a"
                ),
            },
            "example_local_region": {
                "byte_end": 96566,
                "byte_start": 96527,
                "sha256": (
                    "ea63a7f4873f5f2b803e5f10e71f6730"
                    "026ecbe63389a5450331c3d82b0ce886"
                ),
            },
            "fn7_main_effect_ceiling": {
                "byte_end": 70657,
                "byte_start": 70614,
                "sha256": (
                    "a206d85444b96a73e8c06f8bd745ea46"
                    "fae4bfde3a281b26900bce4cbe4c982d"
                ),
            },
        },
        "EFF-2 local-region evidence",
        enforce_pins=enforce_pins,
    )
    evidence["protected_case"] = _protected_case_evidence(
        manifest,
        case_sources,
        protected_surface,
        "stor4-pos-arena-confined",
    )
    evidence["missing_contract"] = "body-local-region-effect-discharge-or-row-projection"
    return Observation(True, evidence)


def observe_contract_member_semantics(
    specification: bytes,
    *,
    enforce_pins: bool = True,
) -> Observation:
    """Record declared contracts without a complete member checking relation."""
    source = ExactSource.from_bytes("spec/kernel-spec-v0.9.md", specification)
    evidence = _exact_fragment_sources(
        source,
        {
            "fn3_conformance": (
                b"a `contract` declares fn signatures and laws; "
                b"`conform T : C { member = fn; }` declares conformance, "
                b"checked per member"
            ),
            "fn5_behavior": (
                b"Behavior parameterization is generics over contract-conforming "
                b"types (env-struct pattern)"
            ),
        },
        {
            "fn3_conformance": {
                "byte_end": 61801,
                "byte_start": 61682,
                "sha256": (
                    "e047d27e9a2bc0feef8cf97911c9eba2"
                    "723057e8bcdfb30c195a13ac237e8b89"
                ),
            },
            "fn5_behavior": {
                "byte_end": 69796,
                "byte_start": 69707,
                "sha256": (
                    "19bf72318a692d1ef8ddd3f9b83334e3"
                    "e2c3456e1a9ed644f0b360be29420c13"
                ),
            },
        },
        "contract-member semantics evidence",
        enforce_pins=enforce_pins,
    )
    evidence["missing_contract"] = (
        "member-set-signature-effect-substitution-law-and-call-resolution"
    )
    return Observation(True, evidence)


def observe_diag3_retained_proof_ref(
    specification: bytes,
    *,
    enforce_pins: bool = True,
) -> Observation:
    """Record an all-required field whose retained-check value is undefined."""
    source = ExactSource.from_bytes("spec/kernel-spec-v0.9.md", specification)
    evidence = _exact_fragment_sources(
        source,
        {
            "check_report_row": (
                b"| check | function; per check: node_path, fact_class "
                b"(bounds/overflow/alias/user), status (retained/eliminated), "
                b"proof_ref (for eliminated: checker-derivation id) |"
            ),
            "report_header": b"| report | fields (all required) |",
        },
        {
            "check_report_row": {
                "byte_end": 93994,
                "byte_start": 93830,
                "sha256": (
                    "54c2c2e96227c74da8c6cefd61029eca"
                    "7146a57a58363622d6d46a821f90e17c"
                ),
            },
            "report_header": {
                "byte_end": 93702,
                "byte_start": 93668,
                "sha256": (
                    "f403fac660ee6442a1a568edd22055cc8"
                    "c476fe9e829de352d1eebf9471e8518"
                ),
            },
        },
        "DIAG-3 retained-proof-ref evidence",
        enforce_pins=enforce_pins,
    )
    evidence["missing_contract"] = "proof_ref-value-for-retained-check"
    return Observation(True, evidence)


def validate_registry(
    observations: Mapping[str, Observation],
) -> dict[str, Registration]:
    """Require a bijection between registrations and recomputed predicates."""
    registrations: dict[str, Registration] = {}
    predicate_ids: set[str] = set()
    for registration in REGISTRATIONS:
        if registration.identifier in registrations:
            raise DiscrepancyError(
                f"duplicate discrepancy registration: {registration.identifier}"
            )
        if registration.predicate_identifier in predicate_ids:
            raise DiscrepancyError(
                f"duplicate discrepancy predicate: {registration.predicate_identifier}"
            )
        if not registration.affected_facet_ids or list(
            registration.affected_facet_ids
        ) != sorted(set(registration.affected_facet_ids)):
            raise DiscrepancyError(
                f"affected facets are empty, duplicated, or unsorted: {registration.identifier}"
            )
        if not registration.resolution_authorities or list(
            registration.resolution_authorities
        ) != sorted(set(registration.resolution_authorities)):
            raise DiscrepancyError(
                "resolution authorities are empty, duplicated, or unsorted: "
                f"{registration.identifier}"
            )
        registrations[registration.identifier] = registration
        predicate_ids.add(registration.predicate_identifier)
    registered_ids = set(registrations)
    observed_ids = set(observations)
    if registered_ids != observed_ids:
        raise DiscrepancyError(
            "discrepancy registrations and predicate observations differ; "
            f"unobserved={sorted(registered_ids - observed_ids)}, "
            f"unregistered={sorted(observed_ids - registered_ids)}"
        )
    return registrations


def recompute(authorities: inputs.AuthorityInputs) -> dict[str, Observation]:
    """Recompute every registered predicate from one authority snapshot."""
    observations = {
        "discrepancy:v0.9/affine-deref-storage-lifecycle": (
            observe_affine_deref_lifecycle(
                authorities.specification,
                enforce_pins=True,
            )
        ),
        "discrepancy:v0.9/diag3-retained-proof-ref": (
            observe_diag3_retained_proof_ref(
                authorities.specification,
                enforce_pins=True,
            )
        ),
        "discrepancy:v0.9/eff1-row-canonicality": observe_eff1_row_canonicality(
            authorities.specification,
            authorities.manifest,
            authorities.case_sources,
            authorities.protected_conformance,
            enforce_pins=True,
        ),
        "discrepancy:v0.9/eff2-local-region-effects": (
            observe_eff2_local_region_effects(
                authorities.specification,
                authorities.manifest,
                authorities.case_sources,
                authorities.protected_conformance,
                enforce_pins=True,
            )
        ),
        "discrepancy:v0.9/fn3-contract-member-semantics": (
            observe_contract_member_semantics(
                authorities.specification,
                enforce_pins=True,
            )
        ),
        "discrepancy:v0.9/op1-dotless-reservation": observe_op1(
            authorities.specification,
            enforce_pins=True,
        ),
        "discrepancy:v0.9/fn7-main-return-spelling": observe_fn7(
            authorities.specification,
            enforce_pins=True,
        ),
    }
    validate_registry(observations)
    return observations
