"""Closed validation for generalized-Oracle report records."""

from __future__ import annotations

import hashlib
from typing import Optional

from runner_common_schema import DocumentSchema
from runner_common_wire import SHA256, canonical_hex, canonical_uint
from runner_inputs import Inputs, fail
from runner_report_wire import (
    case_inputs as _case_inputs,
    domain_inputs as _domain_inputs,
    expectations as _expectations,
    record_fields as _record_fields,
)
from runner_trace import validate_trace_hex


ORACLE_FIELDS = {
    "CASE": 7,
    "CASE-TRACE": 5,
    "CASE-DELTA": 4,
    "DOMAIN": 7,
    "STREAM": 7,
    "STREAM-TRACE": 6,
    "METRIC": 7,
}


def _oracle_key(fields: list[bytes]) -> tuple[object, ...]:
    tag = fields[0]
    document_rank = 0 if len(fields) > 1 and fields[1] == b"current" else 1
    if tag == b"CASE":
        return (0, fields[2], document_rank, 0)
    if tag == b"CASE-TRACE":
        return (0, fields[2], document_rank, 1, canonical_uint(fields[3]))
    if tag == b"CASE-DELTA":
        status = {b"removed": 0, b"retained": 1, b"introduced": 2}.get(fields[2], 3)
        return (1, fields[1], status, fields[3])
    if tag == b"DOMAIN":
        return (2, fields[2], document_rank, 0)
    if tag == b"STREAM":
        return (2, fields[2], document_rank, 1, canonical_uint(fields[3]), 0)
    if tag == b"STREAM-TRACE":
        return (
            2,
            fields[2],
            document_rank,
            1,
            canonical_uint(fields[3]),
            1,
            canonical_uint(fields[4]),
        )
    return (3, document_rank)


def validate_oracle(
    lines: list[bytes],
    inputs: Inputs,
    schemas: dict[bytes, DocumentSchema],
) -> tuple[dict[tuple[bytes, bytes], tuple[bytes, ...]], dict[str, object]]:
    case_inputs = _case_inputs(inputs)
    domain_inputs = _domain_inputs(inputs)
    expected_cases, _transitions, expected_delta_policies = _expectations(inputs)
    observed_cases: dict[tuple[bytes, bytes], bytes] = {}
    case_sources: dict[tuple[bytes, bytes], tuple[bytes, bytes]] = {}
    trace_limits: dict[tuple[bytes, bytes], int] = {}
    trace_ordinals: dict[tuple[bytes, bytes], set[int]] = {}
    case_traces: dict[tuple[bytes, bytes], dict[int, bytes]] = {}
    case_deltas: dict[tuple[bytes, bytes], bytes] = {}
    domain_claims: dict[tuple[bytes, bytes], tuple[bytes, ...]] = {}
    streams: dict[tuple[bytes, bytes], list[tuple[int, bytes]]] = {}
    stream_trace_limits: dict[tuple[bytes, bytes, int], int] = {}
    stream_trace_ordinals: dict[tuple[bytes, bytes, int], set[int]] = {}
    stream_traces: dict[tuple[bytes, bytes, int], dict[int, bytes]] = {}
    metrics: dict[bytes, tuple[int, int, int, int, int]] = {}
    previous: Optional[tuple[object, ...]] = None
    exact_seen: set[bytes] = set()
    semantic_seen: set[tuple[object, ...]] = set()
    for line in lines:
        tag, fields = _record_fields(line, ORACLE_FIELDS, "oracle")
        if line in exact_seen:
            fail("report_specific", "oracle emitted a duplicate record")
        if tag in ("CASE", "CASE-TRACE", "DOMAIN", "STREAM", "STREAM-TRACE", "METRIC") and fields[1] not in (
            b"current",
            b"proposal",
        ):
            fail("report_specific", "oracle emitted an unknown document")
        if tag == "CASE":
            for index in (2, 3, 4):
                canonical_hex(fields[index])
            identifier = bytes.fromhex(fields[2].decode("ascii"))
            key = (identifier, fields[1])
            if identifier not in case_inputs or (fields[3], fields[4]) != case_inputs[identifier]:
                fail("oracle_cases", "oracle case identity does not equal cases.txt")
            if fields[3] not in schemas[fields[1]].productions:
                fail("oracle_cases", "oracle case start is absent from the common grammar ledger")
            if fields[5] not in (b"zero", b"one", b"many"):
                fail("oracle_cases", "oracle emitted an unknown derivation class")
            count = canonical_uint(fields[6])
            if count != {b"zero": 0, b"one": 1, b"many": 2}[fields[5]]:
                fail("oracle_cases", "oracle derivation class and retained trace count disagree")
            observed_cases[key] = fields[5]
            case_sources[key] = (fields[3], bytes.fromhex(fields[4].decode("ascii")))
            trace_limits[key] = count
            semantic_key = (tag, fields[1], fields[2])
        elif tag == "CASE-TRACE":
            canonical_hex(fields[2])
            canonical_hex(fields[4])
            key = (bytes.fromhex(fields[2].decode("ascii")), fields[1])
            ordinal = canonical_uint(fields[3])
            if key not in trace_limits or ordinal >= trace_limits[key]:
                fail("oracle_traces", "oracle emitted an orphan or out-of-range case trace")
            start_hex, source = case_sources[key]
            validate_trace_hex(fields[4], start_hex, source, inputs, schemas[fields[1]])
            trace_ordinals.setdefault(key, set()).add(ordinal)
            case_traces.setdefault(key, {})[ordinal] = fields[4]
            semantic_key = (tag, fields[1], fields[2], fields[3])
        elif tag == "CASE-DELTA":
            canonical_hex(fields[1])
            canonical_hex(fields[3])
            if fields[2] not in (b"removed", b"retained", b"introduced"):
                fail("oracle_delta", "oracle emitted an unknown derivation-delta status")
            case_deltas[(fields[1], fields[3])] = fields[2]
            semantic_key = (tag, fields[1], fields[3])
        elif tag == "DOMAIN":
            for index in (2, 3, 4):
                canonical_hex(fields[index])
            identifier = bytes.fromhex(fields[2].decode("ascii"))
            if identifier not in domain_inputs or (fields[3], fields[4]) != domain_inputs[identifier]:
                fail("oracle_domains", "oracle domain identity does not equal domains.txt")
            if fields[3] not in schemas[fields[1]].productions:
                fail("oracle_domains", "oracle domain start is absent from the common grammar ledger")
            if not SHA256.fullmatch(fields[6]):
                fail("oracle_domains", "oracle emitted a malformed stream digest")
            count = canonical_uint(fields[5])
            if count > inputs.limits["max_generated_streams"]:
                fail("oracle_domains", "oracle domain count exceeds the generated-stream limit")
            domain_claims[(fields[2], fields[1])] = tuple(fields[3:7])
            semantic_key = (tag, fields[1], fields[2])
        elif tag == "STREAM":
            canonical_hex(fields[2])
            canonical_hex(fields[4])
            count = canonical_uint(fields[6])
            if fields[5] not in (b"zero", b"one", b"many") or count != {
                b"zero": 0,
                b"one": 1,
                b"many": 2,
            }[fields[5]]:
                fail("oracle_streams", "oracle stream class and trace count disagree")
            domain_key = (fields[2], fields[1])
            if domain_key not in domain_claims:
                fail("oracle_streams", "oracle emitted a stream before its owning domain")
            ordinal = canonical_uint(fields[3])
            streams.setdefault(domain_key, []).append((ordinal, fields[4]))
            stream_trace_limits[(fields[2], fields[1], ordinal)] = count
            semantic_key = (tag, fields[1], fields[2], fields[3])
        elif tag == "STREAM-TRACE":
            canonical_hex(fields[2])
            stream_ordinal = canonical_uint(fields[3])
            trace_ordinal = canonical_uint(fields[4])
            canonical_hex(fields[5])
            trace_key = (fields[2], fields[1], stream_ordinal)
            if trace_key not in stream_trace_limits or trace_ordinal >= stream_trace_limits[trace_key]:
                fail("oracle_streams", "oracle emitted an orphan or out-of-range stream trace")
            domain_start = domain_claims[(fields[2], fields[1])][0]
            stream_source = next(
                bytes.fromhex(source_hex.decode("ascii"))
                for ordinal, source_hex in streams[(fields[2], fields[1])]
                if ordinal == stream_ordinal
            )
            validate_trace_hex(fields[5], domain_start, stream_source, inputs, schemas[fields[1]])
            stream_trace_ordinals.setdefault(trace_key, set()).add(trace_ordinal)
            stream_traces.setdefault(trace_key, {})[trace_ordinal] = fields[5]
            semantic_key = (tag, fields[1], fields[2], fields[3], fields[4])
        else:
            if fields[1] in metrics:
                fail("oracle_metrics", "oracle emitted duplicate document metrics")
            metrics[fields[1]] = tuple(canonical_uint(field) for field in fields[2:])
            semantic_key = (tag, fields[1])
        if semantic_key in semantic_seen:
            fail("report_duplicate", "oracle records repeat a semantic key")
        semantic_seen.add(semantic_key)
        exact_seen.add(line)
        key_order = _oracle_key(fields)
        if previous is not None and key_order < previous:
            fail("report_order", "oracle records are not in canonical order")
        previous = key_order
    if observed_cases != expected_cases:
        fail("oracle_expectations", "oracle case results do not equal expectations.txt")
    for key, count in trace_limits.items():
        if trace_ordinals.get(key, set()) != set(range(count)):
            fail("oracle_traces", "oracle retained case traces are incomplete or noncanonical")
        traces = case_traces.get(key, {})
        if len(set(traces.values())) != count:
            fail("oracle_traces", "oracle retained duplicate derivation traces")
    expected_case_deltas: dict[tuple[bytes, bytes], bytes] = {}
    expected_status_counts: dict[bytes, dict[bytes, int]] = {}
    class_count = {b"zero": 0, b"one": 1, b"many": 2}
    for identifier in case_inputs:
        current = set(case_traces.get((identifier, b"current"), {}).values())
        proposal = set(case_traces.get((identifier, b"proposal"), {}).values())
        for trace in current | proposal:
            if trace in current and trace in proposal:
                status = b"retained"
            elif trace in current:
                status = b"removed"
            else:
                status = b"introduced"
            expected_case_deltas[(identifier.hex().encode("ascii"), trace)] = status
        current_count = class_count[expected_cases[(identifier, b"current")]]
        proposal_count = class_count[expected_cases[(identifier, b"proposal")]]
        policy = expected_delta_policies.get(identifier)
        if policy == b"trace-subset":
            if not proposal.issubset(current):
                fail("oracle_delta", "case trace-subset policy is not satisfied")
            expected_counts = {
                b"removed": current_count - proposal_count,
                b"retained": proposal_count,
                b"introduced": 0,
            }
        elif policy == b"trace-identical":
            if proposal != current:
                fail("oracle_delta", "case trace-identical policy is not satisfied")
            expected_counts = {
                b"removed": 0,
                b"retained": current_count,
                b"introduced": 0,
            }
        elif policy == b"trace-replacement":
            if proposal & current:
                fail("oracle_delta", "case trace-replacement policy is not satisfied")
            expected_counts = {
                b"removed": current_count,
                b"retained": 0,
                b"introduced": proposal_count,
            }
        else:
            fail("oracle_expectations", "oracle case is missing a closed trace-delta policy")
        expected_status_counts[identifier.hex().encode("ascii")] = expected_counts
    if case_deltas != expected_case_deltas:
        fail("oracle_delta", "oracle case deltas do not classify the complete trace union")
    for identifier_hex, counts in expected_status_counts.items():
        observed = {
            status: sum(
                1
                for (record_identifier, _), record_status in case_deltas.items()
                if record_identifier == identifier_hex and record_status == status
            )
            for status in (b"removed", b"retained", b"introduced")
        }
        if observed != counts:
            fail("oracle_delta", "oracle case-delta status counts do not equal expectations.txt")
    required_domains = {
        (identifier.hex().encode("ascii"), document)
        for identifier in domain_inputs
        for document in (b"current", b"proposal")
    }
    if set(domain_claims) != required_domains:
        fail("oracle_domains", "oracle omitted a required generated domain")
    generated_by_document = {b"current": 0, b"proposal": 0}
    for key, claim in domain_claims.items():
        claimed_count = canonical_uint(claim[2])
        generated_by_document[key[1]] += claimed_count
        if generated_by_document[key[1]] > inputs.limits["max_generated_streams"]:
            fail("oracle_domains", "oracle aggregate domain count exceeds the generated-stream limit")
        ordered = sorted(streams.get(key, []))
        if len(ordered) != claimed_count or any(
            ordinal != expected for expected, (ordinal, _source) in enumerate(ordered)
        ):
            fail("oracle_streams", "oracle stream ordinals do not match the domain count")
        digest = hashlib.sha256()
        for _, source_hex in ordered:
            source = bytes.fromhex(source_hex.decode("ascii"))
            digest.update(len(source).to_bytes(8, "big"))
            digest.update(source)
        if digest.hexdigest().encode("ascii") != claim[3]:
            fail("oracle_streams", "oracle stream bytes do not match the domain digest")
    for key, count in stream_trace_limits.items():
        if stream_trace_ordinals.get(key, set()) != set(range(count)):
            fail("oracle_streams", "oracle retained stream traces are incomplete or noncanonical")
        traces = stream_traces.get(key, {})
        if len(set(traces.values())) != count:
            fail("oracle_streams", "oracle retained duplicate stream derivation traces")
    if set(metrics) != {b"current", b"proposal"}:
        fail("oracle_metrics", "oracle omitted document resource metrics")
    metric_limits = (
        None,
        "oracle_max_source_tokens",
        "oracle_max_chart_items",
        "oracle_max_packed_edges",
        "oracle_max_proof_nodes",
    )
    for document, values in metrics.items():
        parsed_streams = len(case_inputs) + generated_by_document[document]
        if values[0] != parsed_streams:
            fail("oracle_metrics", "oracle parsed-stream metric does not equal the authored workload")
        for value, limit_name in zip(values, metric_limits):
            if limit_name is not None and value > parsed_streams * inputs.limits[limit_name]:
                fail("oracle_metrics", "oracle cumulative metric exceeds its per-stream resource bound")
    case_observations: dict[str, dict[str, object]] = {}
    case_delta_counts: dict[str, dict[str, int]] = {}
    for identifier in case_inputs:
        name = identifier.decode("ascii")
        case_observations[name] = {
            document.decode("ascii"): {
                "class": observed_cases[(identifier, document)].decode("ascii"),
                "trace_count": trace_limits[(identifier, document)],
            }
            for document in (b"current", b"proposal")
        }
        identifier_hex = identifier.hex().encode("ascii")
        case_delta_counts[name] = {
            status: sum(
                1
                for (record_identifier, _trace), record_status in case_deltas.items()
                if record_identifier == identifier_hex and record_status == status.encode("ascii")
            )
            for status in ("introduced", "removed", "retained")
        }
    return domain_claims, {
        "case_delta_status_counts": case_delta_counts,
        "cases": case_observations,
    }
