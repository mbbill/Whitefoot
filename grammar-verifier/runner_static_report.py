"""Closed validation for static-auditor report records."""

from __future__ import annotations

from typing import Optional

from runner_common_schema import DocumentSchema
from runner_common_wire import IDENTIFIER, PATH, SHA256, canonical_hex, canonical_uint
from runner_inputs import Inputs, fail
from runner_report_wire import (
    case_inputs as _case_inputs,
    decoded_hex as _decoded_hex,
    domain_inputs as _domain_inputs,
    expectations as _expectations,
    predicate_hex as _predicate_hex,
    predicate_order as _predicate_order,
    record_fields as _record_fields,
    witness_stream_hex as _witness_stream_hex,
    word_hex as _word_hex,
)


STATIC_FIELDS = {
    "STATIC-NULLABLE": 4,
    "STATIC-FIRST": 4,
    "STATIC-FOLLOW": 4,
    "STATIC-INTERSECTION": 5,
    "STATIC-DECISION": 7,
    "STATIC-CONFLICT": 10,
    "STATIC-DELTA": 4,
    "STATIC-CASE": 6,
    "STATIC-DOMAIN": 7,
    "STATIC-TRANSITION": 6,
}
DECISION_KINDS = (b"choice", b"optional", b"repeat0", b"repeat1")
TRANSITION_CASES = {
    b"fixed-ident-partition": (b"deref-x", b"expr", b"deref(x)"),
}


def validate_static(
    lines: list[bytes],
    inputs: Inputs,
    schemas: dict[bytes, DocumentSchema],
) -> tuple[dict[tuple[bytes, bytes], tuple[bytes, ...]], dict[str, object]]:
    ranks = {tag: rank for rank, tag in enumerate(STATIC_FIELDS)}
    previous: Optional[tuple[int, bytes]] = None
    exact_seen: set[bytes] = set()
    semantic_seen: set[tuple[object, ...]] = set()
    case_inputs = _case_inputs(inputs)
    domain_inputs = _domain_inputs(inputs)
    observed_cases: set[tuple[bytes, bytes]] = set()
    domain_claims: dict[tuple[bytes, bytes], tuple[bytes, ...]] = {}
    case_counts: dict[tuple[bytes, bytes], int] = {}
    classified: dict[str, dict[bytes, set[bytes]]] = {
        "intersection": {b"current": set(), b"proposal": set()},
        "conflict": {b"current": set(), b"proposal": set()},
    }
    deltas: dict[tuple[bytes, bytes], bytes] = {}
    transitions = _expectations(inputs)[1]
    observed_transitions: dict[bytes, tuple[int, int, bytes, bytes]] = {}
    nullable_lhs = {document: set() for document in (b"current", b"proposal")}
    first_lhs = {document: set() for document in (b"current", b"proposal")}
    follow_lhs = {document: set() for document in (b"current", b"proposal")}
    decision_arms: dict[tuple[bytes, bytes, bytes, bytes], set[int]] = {}
    decision_words: dict[tuple[bytes, bytes, bytes, bytes, int], set[tuple[bytes, ...]]] = {}
    self_intersections = {document: set() for document in (b"current", b"proposal")}
    for line in lines:
        tag, fields = _record_fields(line, STATIC_FIELDS, "static")
        if line in exact_seen:
            fail("report_specific", "static emitted a duplicate record")
        if tag in (
            "STATIC-NULLABLE",
            "STATIC-FIRST",
            "STATIC-FOLLOW",
            "STATIC-INTERSECTION",
            "STATIC-DECISION",
            "STATIC-CONFLICT",
            "STATIC-CASE",
            "STATIC-DOMAIN",
        ) and fields[1] not in (b"current", b"proposal"):
            fail("report_specific", "static emitted an unknown document")
        if tag == "STATIC-NULLABLE":
            canonical_hex(fields[2])
            if fields[3] not in (b"0", b"1"):
                fail("report_specific", "static nullable value is not zero or one")
            semantic_key = (tag, fields[1], fields[2])
            nullable_lhs[fields[1]].add(fields[2])
        elif tag in ("STATIC-FIRST", "STATIC-FOLLOW"):
            canonical_hex(fields[2])
            _word_hex(fields[3], schemas[fields[1]].terminal_predicates())
            semantic_key = (tag, fields[1], fields[2], fields[3])
            (first_lhs if tag == "STATIC-FIRST" else follow_lhs)[fields[1]].add(fields[2])
        elif tag == "STATIC-INTERSECTION":
            left = _predicate_hex(fields[2])
            right = _predicate_hex(fields[3])
            allowed = schemas[fields[1]].terminal_predicates()
            if left not in allowed or right not in allowed:
                fail("static_predicate", "static intersection uses a predicate absent from the common ledger")
            if _predicate_order(left) > _predicate_order(right):
                fail("static_predicate", "static intersection predicates are not canonically ordered")
            witness = _decoded_hex(fields[4], optional=True)
            if left == b"end" or right == b"end":
                if left != b"end" or right != b"end" or witness is not None:
                    fail("static_predicate", "static intersection has an invalid end witness")
            else:
                if witness is None:
                    fail("static_predicate", "static non-end intersection omits its witness")
                for descriptor in (left, right):
                    if descriptor.startswith(b"fixed:"):
                        fixed = bytes.fromhex(descriptor.removeprefix(b"fixed:").decode("ascii"))
                        if witness != fixed:
                            fail("static_predicate", "static fixed intersection has the wrong witness")
                    elif descriptor == b"pattern:digits" and not witness.isdigit():
                        fail("static_predicate", "static pattern intersection has a nondigit witness")
            semantic_key = (tag, fields[1], fields[2], fields[3])
            if fields[2] == fields[3]:
                self_intersections[fields[1]].add(left)
            key_hex = b"\t".join(fields[2:4]).hex().encode("ascii")
            classified["intersection"][fields[1]].add(key_hex)
        elif tag == "STATIC-DECISION":
            canonical_hex(fields[2])
            if not PATH.fullmatch(fields[3]) or fields[4] not in DECISION_KINDS:
                fail("report_specific", "static decision path or kind is not canonical")
            canonical_uint(fields[5])
            arm = canonical_uint(fields[5])
            word = _word_hex(fields[6], schemas[fields[1]].terminal_predicates())
            node = schemas[fields[1]].nodes.get((fields[2], fields[3]))
            if node is None or node[0] != fields[4]:
                fail("static_decision", "static decision is absent from the common grammar ledger")
            decision_arms.setdefault((fields[1], fields[2], fields[3], fields[4]), set()).add(arm)
            decision_words.setdefault((fields[1], fields[2], fields[3], fields[4], arm), set()).add(word)
            semantic_key = (tag, fields[1], fields[2], fields[3], fields[4], fields[5], fields[6])
        elif tag == "STATIC-CONFLICT":
            canonical_hex(fields[2])
            if not PATH.fullmatch(fields[3]) or fields[4] not in DECISION_KINDS:
                fail("report_specific", "static conflict path or kind is not canonical")
            left_arm = canonical_uint(fields[5])
            right_arm = canonical_uint(fields[6])
            if left_arm >= right_arm:
                fail("static_conflict", "static conflict arms are not canonically ordered")
            if schemas[fields[1]].nodes.get((fields[2], fields[3]), (None, None))[0] != fields[4]:
                fail("static_conflict", "static conflict is absent from the common grammar ledger")
            left_word = _word_hex(fields[7], schemas[fields[1]].terminal_predicates())
            right_word = _word_hex(fields[8], schemas[fields[1]].terminal_predicates())
            witness = _witness_stream_hex(fields[9])
            decision_prefix = (fields[1], fields[2], fields[3], fields[4])
            if (
                left_word not in decision_words.get((*decision_prefix, left_arm), set())
                or right_word not in decision_words.get((*decision_prefix, right_arm), set())
            ):
                fail("static_conflict", "static conflict words do not belong to their decision arms")
            if len(left_word) != len(right_word) or len(witness) != len(left_word):
                fail("static_witness", "static conflict witness arity disagrees with its lookahead words")
            for left_predicate, right_predicate, token in zip(left_word, right_word, witness):
                if (left_predicate == b"end") != (right_predicate == b"end"):
                    fail("static_witness", "static conflict claims an impossible end intersection")
                expects_end = left_predicate == right_predicate == b"end"
                if (token == b"-") != expects_end:
                    fail("static_witness", "static conflict witness has a misplaced end marker")
            semantic_key = (tag, fields[1], *fields[2:9])
            key_hex = b"\t".join(fields[2:9]).hex().encode("ascii")
            classified["conflict"][fields[1]].add(key_hex)
        elif tag == "STATIC-DELTA":
            if fields[1] not in (b"intersection", b"conflict"):
                fail("report_specific", "static emitted an unknown delta kind")
            if fields[2] not in (b"removed", b"retained", b"introduced"):
                fail("report_specific", "static emitted an unknown conflict-delta status")
            canonical_hex(fields[3])
            deltas[(fields[1], fields[3])] = fields[2]
            semantic_key = (tag, fields[1], fields[3])
        elif tag == "STATIC-CASE":
            if not IDENTIFIER.fullmatch(fields[2]):
                fail("static_cases", "static emitted a noncanonical case id")
            canonical_hex(fields[3])
            canonical_hex(fields[4])
            count = canonical_uint(fields[5])
            if fields[2] not in case_inputs or (fields[3], fields[4]) != case_inputs[fields[2]]:
                fail("static_cases", "static case identity does not equal cases.txt")
            if fields[3] not in schemas[fields[1]].productions:
                fail("static_cases", "static case start is absent from the common grammar ledger")
            semantic_key = (tag, fields[1], fields[2])
            observed_cases.add((fields[2], fields[1]))
            case_counts[(fields[2], fields[1])] = count
        elif tag == "STATIC-DOMAIN":
            for index in (2, 3, 4):
                canonical_hex(fields[index])
            if not SHA256.fullmatch(fields[6]):
                fail("static_domains", "static emitted a malformed stream digest")
            identifier = bytes.fromhex(fields[2].decode("ascii"))
            if identifier not in domain_inputs or (fields[3], fields[4]) != domain_inputs[identifier]:
                fail("static_domains", "static domain identity does not equal domains.txt")
            if fields[3] not in schemas[fields[1]].productions:
                fail("static_domains", "static domain start is absent from the common grammar ledger")
            canonical_uint(fields[5])
            semantic_key = (tag, fields[1], fields[2])
            domain_claims[(fields[2], fields[1])] = tuple(fields[3:7])
        else:
            if not IDENTIFIER.fullmatch(fields[1]):
                fail("report_specific", "static emitted an unknown transition")
            current_count = canonical_uint(fields[2])
            proposal_count = canonical_uint(fields[3])
            if not IDENTIFIER.fullmatch(fields[4]):
                fail("report_specific", "static emitted a noncanonical transition status")
            _decoded_hex(fields[5], optional=True)
            observed_transitions[fields[1]] = (current_count, proposal_count, fields[4], fields[5])
            semantic_key = (tag, fields[1])
        if semantic_key in semantic_seen:
            fail("report_duplicate", "static records repeat a semantic key")
        semantic_seen.add(semantic_key)
        exact_seen.add(line)
        key = (ranks[tag], line)
        if previous is not None and key < previous:
            fail("report_order", "static records are not in canonical order")
        previous = key
    for document, schema in schemas.items():
        if (
            nullable_lhs[document] != set(schema.productions)
            or first_lhs[document] != set(schema.productions)
            or follow_lhs[document] != set(schema.productions)
        ):
            fail("static_coverage", "static nullable, FIRST, or FOLLOW coverage is incomplete")
        if self_intersections[document] != set(schema.terminal_predicates()):
            fail("static_coverage", "static terminal self-intersection coverage is incomplete")
        for (lhs, path), (kind, _value) in schema.nodes.items():
            if kind not in DECISION_KINDS:
                continue
            expected_arms = (
                set(range(len(schema.children[(lhs, path)])))
                if kind == b"choice"
                else {0, 1}
            )
            if decision_arms.get((document, lhs, path, kind), set()) != expected_arms:
                fail("static_coverage", "static decision-arm coverage is incomplete")
    expected_transition_ids = set(transitions)
    if set(observed_transitions) != expected_transition_ids:
        fail("static_expectations", "static transition results do not equal expectations.txt")
    if set(observed_transitions) != set(TRANSITION_CASES):
        fail("static_expectations", "format-v1 transition registry is incomplete or unknown")
    for identifier, (current_count, proposal_count, status, witness) in observed_transitions.items():
        if status != transitions[identifier] or current_count != 1 or proposal_count != 0:
            fail("static_expectations", "static transition does not establish the required removal")
        case_id, required_start, required_source = TRANSITION_CASES[identifier]
        case_start_hex, case_source_hex = case_inputs.get(case_id, (b"", b""))
        if (
            case_start_hex != required_start.hex().encode("ascii")
            or case_source_hex != required_source.hex().encode("ascii")
            or witness != case_source_hex
            or current_count != case_counts.get((case_id, b"current"))
            or proposal_count != case_counts.get((case_id, b"proposal"))
        ):
            fail("static_expectations", "static transition is not bound to its exact authored case")
    expected_deltas: dict[tuple[bytes, bytes], bytes] = {}
    for kind, documents in classified.items():
        for key_hex in documents[b"current"] | documents[b"proposal"]:
            if key_hex in documents[b"current"] and key_hex in documents[b"proposal"]:
                status = b"retained"
            elif key_hex in documents[b"current"]:
                status = b"removed"
            else:
                status = b"introduced"
            expected_deltas[(kind.encode("ascii"), key_hex)] = status
    if deltas != expected_deltas:
        fail("static_delta", "static delta records do not classify the complete evidence union")
    if b"introduced" in deltas.values():
        fail("static_delta", "the current policy permits no introduced intersection or conflict")
    if classified["conflict"][b"proposal"]:
        fail("static_conflict", "the proposal must have zero strong-LL(2) decision conflicts")
    required_cases = {(identifier, document) for identifier in case_inputs for document in (b"current", b"proposal")}
    if observed_cases != required_cases:
        fail("static_cases", "static omitted a required authored case")
    required_domains = {
        (identifier.hex().encode("ascii"), document)
        for identifier in domain_inputs
        for document in (b"current", b"proposal")
    }
    if set(domain_claims) != required_domains:
        fail("static_domains", "static omitted a required generated domain")
    delta_counts = {
        kind: {
            status: sum(
                1
                for (record_kind, _key), record_status in deltas.items()
                if record_kind == kind.encode("ascii") and record_status == status.encode("ascii")
            )
            for status in ("introduced", "removed", "retained")
        }
        for kind in ("conflict", "intersection")
    }
    transition_observations = {
        identifier.decode("ascii"): {
            "current_count": current_count,
            "proposal_count": proposal_count,
            "status": status.decode("ascii"),
            "witness_hex": witness.decode("ascii"),
        }
        for identifier, (current_count, proposal_count, status, witness) in observed_transitions.items()
    }
    return domain_claims, {
        "delta_status_counts": delta_counts,
        "transitions": transition_observations,
    }
