from __future__ import annotations

from typing import Any

from form2_inputs import ProtectedSource, sha256
from form2_tree import CandidateParser, ParseAttempt, parse_one
from form2_zero_cases import (
    ExactEdit,
    PROTECTED_REPAIR_KEYS,
    ZERO_CASES,
    ZeroDerivationAuditError,
)
def source_key(source: ProtectedSource) -> str:
    return source.identifier or "pending-const2-item"


def _positions(raw: bytes, needle: bytes) -> list[int]:
    if not needle:
        raise ZeroDerivationAuditError("an exact-edit before fragment is empty")
    result: list[int] = []
    cursor = 0
    while True:
        offset = raw.find(needle, cursor)
        if offset < 0:
            return result
        result.append(offset)
        cursor = offset + len(needle)


def apply_exact_edits(
    raw: bytes, edits: tuple[ExactEdit, ...]
) -> tuple[bytes, list[dict[str, Any]]]:
    spans: list[tuple[int, int, bytes, ExactEdit]] = []
    for edit in edits:
        positions = _positions(raw, edit.before)
        if len(positions) != edit.expected_count:
            raise ZeroDerivationAuditError(
                "exact-edit occurrence count changed: "
                f"expected {edit.expected_count}, found {len(positions)}"
            )
        spans.extend(
            (offset, offset + len(edit.before), edit.after, edit)
            for offset in positions
        )
    spans.sort(key=lambda span: span[0])
    for left, right in zip(spans, spans[1:]):
        if left[1] > right[0]:
            raise ZeroDerivationAuditError("exact-edit spans overlap")
    output = bytearray()
    records: list[dict[str, Any]] = []
    cursor = 0
    for start, end, replacement, edit in spans:
        output.extend(raw[cursor:start])
        output.extend(replacement)
        records.append(
            {
                "after_hex": replacement.hex(),
                "before_hex": raw[start:end].hex(),
                "byte_end_before": end,
                "byte_start_before": start,
                "purpose": edit.purpose,
            }
        )
        cursor = end
    output.extend(raw[cursor:])
    return bytes(output), records


def audit_zero_source(
    parser: CandidateParser,
    source: ProtectedSource,
    exact_attempt: ParseAttempt,
) -> tuple[dict[str, Any], bytes | None]:
    key = source_key(source)
    spec = ZERO_CASES.get(key)
    if spec is None:
        raise ZeroDerivationAuditError(f"unclassified zero-derivation source: {source.path}")
    if source.path != spec.path:
        raise ZeroDerivationAuditError(f"zero-derivation source path changed: {key}")
    if exact_attempt.classification != "zero":
        raise ZeroDerivationAuditError(f"zero-derivation source now derives: {key}")
    expected = None if source.manifest is None else source.manifest.get("expect")
    required = (
        None
        if spec.expected_rule is None
        else {"kind": "reject", "rule": spec.expected_rule}
    )
    if expected != required:
        raise ZeroDerivationAuditError(f"expected verdict changed for zero case: {key}")

    control, edit_records = apply_exact_edits(source.raw, spec.control_edits)
    control_attempt = parse_one(parser, control)
    if control_attempt.classification != "one" or control_attempt.derivation is None:
        raise ZeroDerivationAuditError(
            f"intended-violation control does not have one derivation: {key}"
        )
    repair = control if key in PROTECTED_REPAIR_KEYS else None
    record = {
        "control": {
            "edits": edit_records,
            "purpose": spec.control_purpose,
            "sha256": sha256(control),
            "unique_complete_derivation": True,
        },
        "disposition": spec.disposition,
        "expected_rule": spec.expected_rule,
        "id": source.identifier,
        "path": source.path,
        "protected_source_sha256": source.sha256,
        "repair_proposal": None
        if repair is None
        else {
            "after_byte_count": len(repair),
            "after_sha256": sha256(repair),
            "before_byte_count": len(source.raw),
            "before_sha256": source.sha256,
            "edits": edit_records,
            "intended_verdict_change_proposed": False,
            "owner_approval_required": True,
            "rationale": spec.repair_rationale,
            "runtime_behavior_change_proposed": False,
        },
    }
    return record, repair


def validate_zero_inventory(observed: set[str]) -> None:
    expected = set(ZERO_CASES)
    if observed != expected:
        raise ZeroDerivationAuditError(
            "exact zero-derivation inventory changed: "
            f"missing={sorted(expected - observed)!r}, "
            f"extra={sorted(observed - expected)!r}"
        )
