#!/usr/bin/env python3
"""Executable candidate-neutral mathematical oracle for dense Lock A.

The oracle computes expected abstract states only.  Candidate execution remains
unauthorized.  Each candidate binding receives the same semantic transition;
the candidate ID records coverage of its lifecycle binding, not an implementation.
"""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import tempfile
from pathlib import Path
from typing import Iterable

import dense_contract_registry as registry


HERE = Path(__file__).resolve().parent
TRACE_OUTPUT = HERE / "DENSE-MATHEMATICAL-SOUNDNESS-TRACES.jsonl"
MANIFEST_OUTPUT = HERE / "DENSE-SOUNDNESS-PROTOCOL-MANIFEST.json"
STATE_SCHEMA_VERSION = "dense-mathematical-state-v1"
TRACE_SCHEMA_VERSION = "dense-soundness-trace-v1"
MAX_USIZE_64 = 18446744073709551615


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_json(value: object) -> str:
    return sha256_bytes(json.dumps(value, sort_keys=True, separators=(",", ":")).encode("utf-8"))


def file_sha256(path: Path) -> str:
    return sha256_bytes(path.read_bytes())


def _fact_instances(root: str, version: int, length: int, capacity: int, zst: bool) -> list[dict[str, object]]:
    facts: list[dict[str, object]] = [
        {
            "instance_id": f"fact:root:{root}:v{version}",
            "fact_id": "FACT-DENSE-ROOT-VERSION",
            "owner_root": root,
            "version": version,
            "values": {"root": root, "version": version},
        },
        {
            "instance_id": f"fact:prefix:{root}:v{version}:l{length}:c{capacity}",
            "fact_id": "FACT-DENSE-LIVE-PREFIX",
            "owner_root": root,
            "version": version,
            "values": {"length": length, "capacity": capacity},
        },
        {
            "instance_id": f"fact:capacity:{root}:v{version}:c{capacity}",
            "fact_id": "FACT-DENSE-CAPACITY",
            "owner_root": root,
            "version": version,
            "values": {"capacity": capacity, "zst": zst},
        },
    ]
    for index in range(length):
        facts.append(
            {
                "instance_id": f"fact:slot:{root}:v{version}:i{index}",
                "fact_id": "FACT-DENSE-SLOT-LIVE",
                "owner_root": root,
                "version": version,
                "values": {"index": index},
            }
        )
    if zst:
        facts.append(
            {
                "instance_id": f"fact:zst:{root}:v{version}:l{length}",
                "fact_id": "FACT-DENSE-ZST-LOGICAL",
                "owner_root": root,
                "version": version,
                "values": {"length": length, "physical_bytes": 0},
            }
        )
    return facts


def make_state(
    values: Iterable[str] = ("value:v0", "value:v1", "value:v2"),
    *,
    capacity: int = 4,
    root: str = "root:A0",
    address: str = "address:0x1000",
    version: int = 0,
    zst: bool = False,
    nested_input_owners: Iterable[str] = (),
    retained_external_owners: Iterable[str] = (),
    protocol_owned_owners: Iterable[str] = ("owner:BASE",),
    typed_values: dict[str, dict[str, object]] | None = None,
) -> dict[str, object]:
    sequence = list(values)
    nested = list(nested_input_owners)
    retained = list(retained_external_owners)
    protocol = list(protocol_owned_owners)
    universe = sorted(sequence + nested + retained + protocol)
    if len(universe) != len(set(universe)):
        raise ValueError("initial owner universe contains a duplicate token")
    logical_capacity = MAX_USIZE_64 if zst else capacity
    has_block = not zst and capacity > 0
    state = {
        "state_schema_version": STATE_SCHEMA_VERSION,
        "admitted": True,
        "terminal": "NORMAL",
        "base_owner": "owner:BASE",
        "protocol_owner": None,
        "sequence": sequence,
        "payload_owner_by_slot": {str(index): owner for index, owner in enumerate(sequence)},
        "nested_input_owners": nested,
        "external_owners": retained,
        "returned_owners": [],
        "destroyed_owners": [],
        "protocol_owned_owners": protocol,
        "owner_universe": universe,
        "owner_universe_sha256": sha256_json(universe),
        "length": len(sequence),
        "capacity": logical_capacity,
        "version": version,
        "allocation": {
            "kind": "ZST_VIRTUAL_ROOT" if zst else ("ALLOCATOR_BLOCK" if has_block else "EMPTY_ROOT"),
            "root_id": root,
            "block_owner": f"allocation-owner:{root}" if has_block else None,
            "address_token": "address:zst-shared" if zst else (address if has_block else None),
            "acquired_bytes": capacity * 8 if has_block else 0,
            "allocation_calls": 1 if has_block else 0,
            "released_roots": [],
        },
        "borrows": {"active": [], "invalidated": []},
        "facts": {
            "active": _fact_instances(root, version, len(sequence), logical_capacity, zst),
            "invalidated": [],
        },
        "behavior_calls": [],
        "diagnostics": [],
        "invalid_payload_access_count": 0,
        "zst": zst,
        "typed_values": typed_values or {
            owner: {"sort_key": index, "predicate_keep": index % 2 == 0, "original_index": index}
            for index, owner in enumerate(sequence)
        },
    }
    validate_state(state)
    return state


def static_state() -> dict[str, object]:
    return {
        "state_schema_version": STATE_SCHEMA_VERSION,
        "admitted": False,
        "terminal": "STATIC",
        "base_owner": None,
        "protocol_owner": None,
        "sequence": [],
        "payload_owner_by_slot": {},
        "external_owners": [],
        "nested_input_owners": [],
        "returned_owners": [],
        "destroyed_owners": [],
        "protocol_owned_owners": [],
        "owner_universe": [],
        "owner_universe_sha256": sha256_json([]),
        "length": 0,
        "capacity": 0,
        "version": 0,
        "allocation": {
            "kind": "NONE",
            "root_id": None,
            "block_owner": None,
            "address_token": None,
            "acquired_bytes": 0,
            "allocation_calls": 0,
            "released_roots": [],
        },
        "borrows": {"active": [], "invalidated": []},
        "facts": {"active": [], "invalidated": []},
        "behavior_calls": [],
        "diagnostics": [],
        "invalid_payload_access_count": 0,
        "zst": False,
        "typed_values": {},
    }


def validate_state(state: dict[str, object]) -> None:
    if state["state_schema_version"] != STATE_SCHEMA_VERSION:
        raise ValueError("unknown state schema")
    if not state["admitted"]:
        if state["allocation"]["root_id"] is not None or state["sequence"]:
            raise ValueError("static state carries runtime authority")
        return
    sequence = state["sequence"]
    owner_map = state["payload_owner_by_slot"]
    if state["length"] != len(sequence) or state["length"] != len(owner_map):
        raise ValueError("length/owner map mismatch")
    if list(owner_map) != [str(index) for index in range(len(sequence))]:
        raise ValueError("slot key mismatch")
    if list(owner_map.values()) != sequence:
        raise ValueError("sequence/owner mismatch")
    if len(sequence) != len(set(sequence)):
        raise ValueError("duplicate live owner")
    role_lists = (
        sequence,
        state["nested_input_owners"],
        state["returned_owners"],
        state["external_owners"],
        state["destroyed_owners"],
        state["protocol_owned_owners"],
    )
    role_tokens = [token for role in role_lists for token in role]
    if len(role_tokens) != len(set(role_tokens)):
        raise ValueError("affine token occupies more than one owner role")
    if sorted(role_tokens) != state["owner_universe"]:
        missing = sorted(set(state["owner_universe"]) - set(role_tokens))
        extra = sorted(set(role_tokens) - set(state["owner_universe"]))
        raise ValueError(f"owner conservation mismatch missing={missing} extra={extra}")
    if state["owner_universe_sha256"] != sha256_json(state["owner_universe"]):
        raise ValueError("owner universe was mutated")
    if state["length"] > state["capacity"]:
        raise ValueError("length exceeds capacity")
    live = set(sequence)
    if live & set(state["returned_owners"]):
        raise ValueError("returned owner remains live")
    if live & set(state["destroyed_owners"]):
        raise ValueError("destroyed owner remains live")
    root = state["allocation"]["root_id"]
    if state["base_owner"] is not None or state["protocol_owner"] is not None:
        if root is None:
            raise ValueError("live state has no root")
        if (
            not state["zst"]
            and state["capacity"] > 0
            and state["allocation"]["block_owner"] is None
        ):
            raise ValueError("positive-size live state has no allocation owner")
    if state["zst"]:
        if state["capacity"] != MAX_USIZE_64 or state["allocation"]["acquired_bytes"] != 0:
            raise ValueError("ZST policy mismatch")
        if state["allocation"]["block_owner"] is not None or state["allocation"]["allocation_calls"] != 0:
            raise ValueError("ZST allocated payload storage")
    if len(state["allocation"]["released_roots"]) != len(
        set(state["allocation"]["released_roots"])
    ):
        raise ValueError("allocation root released more than once")
    for fact in state["facts"]["active"]:
        if fact["owner_root"] != root or fact["version"] != state["version"]:
            raise ValueError("stale active fact")
    for borrow in state["borrows"]["active"]:
        if borrow["owner_root"] != root or borrow["version"] != state["version"]:
            raise ValueError("stale active borrow")
    if state["invalid_payload_access_count"] != 0:
        raise ValueError("oracle expected invalid payload access")


def _refresh_state_facts(state: dict[str, object], reason: str) -> None:
    old = state["facts"]["active"]
    state["facts"]["invalidated"].extend(
        {"instance_id": fact["instance_id"], "reason": reason} for fact in old
    )
    state["facts"]["active"] = _fact_instances(
        state["allocation"]["root_id"],
        state["version"],
        state["length"],
        state["capacity"],
        state["zst"],
    )


def _set_sequence(state: dict[str, object], sequence: list[str], reason: str) -> None:
    state["sequence"] = sequence
    state["payload_owner_by_slot"] = {str(index): owner for index, owner in enumerate(sequence)}
    state["length"] = len(sequence)
    state["version"] += 1
    _refresh_state_facts(state, reason)


def _grow_root(state: dict[str, object], same_address: bool = False) -> None:
    if state["zst"]:
        return
    old_root = state["allocation"]["root_id"]
    new_capacity = max(state["capacity"] * 2, state["length"] + 1)
    state["allocation"]["released_roots"].append(old_root)
    state["allocation"]["root_id"] = "root:A1"
    state["allocation"]["block_owner"] = "allocation-owner:root:A1"
    if not same_address:
        state["allocation"]["address_token"] = "address:0x2000"
    state["allocation"]["acquired_bytes"] = new_capacity * 8
    state["allocation"]["allocation_calls"] += 1
    state["capacity"] = new_capacity
    state["version"] += 1
    state["borrows"]["invalidated"].extend(
        {"borrow_id": borrow["borrow_id"], "reason": "root transfer"}
        for borrow in state["borrows"]["active"]
    )
    state["borrows"]["active"] = []
    _refresh_state_facts(state, "root transfer")


def _acquire_constructed_root(state: dict[str, object], capacity: int) -> None:
    old_facts = list(state["facts"]["active"])
    state["allocation"] = {
        "kind": "ALLOCATOR_BLOCK" if capacity else "EMPTY_ROOT",
        "root_id": "root:A1",
        "block_owner": "allocation-owner:root:A1" if capacity else None,
        "address_token": "address:0x2000" if capacity else None,
        "acquired_bytes": capacity * 8,
        "allocation_calls": 1 if capacity else 0,
        "released_roots": [],
    }
    state["capacity"] = capacity
    state["version"] = 1
    state["facts"]["invalidated"].extend(
        {"instance_id": fact["instance_id"], "reason": "constructor root acquisition"}
        for fact in old_facts
    )
    state["facts"]["active"] = _fact_instances(
        "root:A1", 1, state["length"], capacity, False
    )


def vacant_input_state() -> dict[str, object]:
    state = static_state()
    state["admitted"] = True
    state["terminal"] = "NORMAL"
    state["owner_universe_sha256"] = sha256_json(state["owner_universe"])
    validate_state(state)
    return state


TRANSITION_MEMBERS: dict[str, tuple[str, ...]] = {
    "STATIC_REJECT": tuple(sorted(registry.EXCLUDED_MEMBERS)),
    "PRESERVE": (
        "DENSE-COMPARE", "DENSE-FIXED-EACH", "DENSE-FIXED-VIEW", "DENSE-HASH-TRAVERSAL",
        "DENSE-INDEX-SHARED", "DENSE-INDEX-UNIQ", "DENSE-META", "DENSE-OWNER-VIEW",
        "DENSE-VIEW-ARRAY-CHUNKS", "DENSE-VIEW-AS-FIXED", "DENSE-VIEW-CONSUME-SPLIT",
        "DENSE-VIEW-DISJOINT-UNIQ", "DENSE-VIEW-END", "DENSE-VIEW-END-CHUNK",
        "DENSE-VIEW-END-SPLIT", "DENSE-VIEW-GET-SHARED", "DENSE-VIEW-GET-UNIQ",
        "DENSE-VIEW-META", "DENSE-VIEW-SPLIT-CHECKED", "DENSE-VIEW-SPLIT-TRAP",
    ),
    "CONSTRUCT_EMPTY": ("DENSE-DEFAULT", "DENSE-NEW", "DENSE-WITH-CAPACITY"),
    "CONSTRUCT_VALUES": (
        "DENSE-COLLECT", "DENSE-CONCAT", "DENSE-CONVERT", "DENSE-FIXED-MAP",
        "DENSE-FRESH-CLONE", "DENSE-JOIN", "DENSE-REPEAT",
    ),
    "TRANSFORM": (
        "DENSE-CLONE-FROM", "DENSE-COPY-FROM", "DENSE-COPY-WITHIN", "DENSE-DEDUP",
        "DENSE-DEDUP-BY", "DENSE-DEDUP-BY-KEY", "DENSE-FILL-CLONE", "DENSE-FILL-WITH",
        "DENSE-RETAIN", "DENSE-RETAIN-MUT", "DENSE-SELECT-UNSTABLE",
        "DENSE-SORT-STABLE", "DENSE-SORT-STABLE-CACHED-KEY", "DENSE-SORT-UNSTABLE",
        "DENSE-SWAP-WITH-VIEW", "DENSE-TAKE-WITH-DEFAULT",
    ),
    "GROW_PRESERVE": (
        "DENSE-INTO-BOXED", "DENSE-RESERVE", "DENSE-RESERVE-EXACT", "DENSE-SHRINK-TO",
        "DENSE-SHRINK-TO-FIT", "DENSE-TRY-RESERVE", "DENSE-TRY-RESERVE-EXACT",
    ),
    "PUSH": ("DENSE-PUSH", "DENSE-PUSH-UNIQ"),
    "INSERT": ("DENSE-INSERT", "DENSE-INSERT-UNIQ"),
    "APPEND": ("DENSE-APPEND-MOVE",),
    "EXTEND": ("DENSE-EXTEND-CLONE", "DENSE-EXTEND-ITER", "DENSE-EXTEND-WITHIN"),
    "RESIZE": ("DENSE-RESIZE-CLONE", "DENSE-RESIZE-WITH"),
    "POP": ("DENSE-POP",),
    "POP_IF": ("DENSE-POP-IF",),
    "REMOVE": ("DENSE-REMOVE",),
    "SWAP_REMOVE": ("DENSE-SWAP-REMOVE",),
    "CLEAR": ("DENSE-CLEAR",),
    "TRUNCATE": ("DENSE-TRUNCATE",),
    "REPLACE": ("DENSE-REPLACE",),
    "REVERSE": ("DENSE-REVERSE",),
    "ROTATE": ("DENSE-ROTATE",),
    "SWAP": ("DENSE-SWAP",),
    "SPLIT_OFF": ("DENSE-SPLIT-OFF",),
    "INTO_OWNER": ("DENSE-INTO-FLATTENED", "DENSE-INTO-OWNER"),
    "OWN_ITER": ("DENSE-ITER-OWN",),
    "BORROW_ITER": ("DENSE-ITER-SHARED", "DENSE-ITER-UNIQ"),
    "DROP": ("DENSE-DROP",),
    "INIT": ("DENSE-INIT-CLONE", "DENSE-INIT-COPY"),
    "EAGER_EXTRACT": ("DENSE-EAGER-EXTRACT",),
    "EAGER_SPLICE": ("DENSE-EAGER-SPLICE",),
}


def transition_class_by_member() -> dict[str, str]:
    result = registry.semantic_transition_by_member()
    local_canary: dict[str, str] = {}
    for transition_class, members in TRANSITION_MEMBERS.items():
        for member in members:
            local_canary[member] = transition_class
    if local_canary != result:
        raise ValueError("oracle transition canary differs from owner-role registry authority")
    return result


def _outcome_code(contract: dict[str, str]) -> str:
    marker = ".OUT."
    if marker not in contract["outcome_id"]:
        raise ValueError(f"malformed outcome ID: {contract['outcome_id']}")
    return contract["outcome_id"].split(marker, 1)[1]


ABORT_CODES = {
    "BEHAVIOR_ABORT", "BOUNDS_TRAP", "CAPACITY_OVERFLOW_TRAP", "OOM_ABORT",
    "PRECONDITION_TRAP",
}
RECOVERABLE_CODES = {
    "CHECKED_ERROR", "UNDERFILL_CLOSE_REJECTED", "OVERFILL_REJECTED",
    "CAPACITY_ERROR_RETURN", "ALLOCATION_ERROR_RETURN",
}


def _typed_value_rows(tokens: list[str], keys: list[int] | None = None) -> dict[str, dict[str, object]]:
    actual_keys = keys if keys is not None else list(range(len(tokens)))
    return {
        token: {
            "sort_key": actual_keys[index],
            "predicate_keep": actual_keys[index] % 2 == 0,
            "original_index": index,
            "label": token,
        }
        for index, token in enumerate(tokens)
    }


def canonical_initial_state(
    contract: dict[str, str],
    owner_role: dict[str, str] | None = None,
) -> dict[str, object]:
    transition_class = transition_class_by_member()[contract["member_contract_id"]]
    member = contract["member_contract_id"]
    code = _outcome_code(contract)
    if transition_class == "STATIC_REJECT":
        state = static_state()
    else:
        values = ["value:v0", "value:v1", "value:v2"]
        keys = [2, 1, 2]
        if member.startswith("DENSE-DEDUP"):
            values = ["value:v0", "value:v1", "value:v2", "value:v3"]
            keys = [1, 1, 2, 2]
        if code in {"ABSENT", "EMPTY", "EMPTY_NO_CHANGE", "TERMINAL_NONE"}:
            values = []
            keys = []
        nested: list[str] = []
        protocol: list[str] = []
        retained: list[str] = []
        if transition_class not in {"CONSTRUCT_EMPTY", "CONSTRUCT_VALUES", "INIT"}:
            protocol.append("owner:BASE")
        if member in registry.OFFERED_VALUE_MEMBERS:
            nested.append("value:OFFERED")
        if member == "DENSE-APPEND-MOVE":
            nested.extend(("value:s0", "value:s1"))
            protocol.append("owner:SOURCE")
        if member in registry.PRODUCER_MEMBERS:
            if member == "DENSE-EAGER-SPLICE":
                produced = ["value:replacement0", "value:replacement1"]
            elif member == "DENSE-FILL-WITH":
                produced = ["value:e0", "value:e1", "value:e2"]
            else:
                produced = ["value:e0", "value:e1"]
            nested.extend(produced)
            protocol.append("owner:PRODUCER")
        if member in registry.CLONE_RESULT_MEMBERS:
            clone_count = 3 if member == "DENSE-CLONE-FROM" else 2
            nested.extend(f"value:c{index}" for index in range(clone_count))
        if member == "DENSE-FIXED-MAP":
            nested.extend(("value:p0", "value:p1", "value:input0", "value:input1"))
            protocol.append("owner:ARRAY")
        if transition_class == "CONSTRUCT_VALUES" and not nested:
            nested.extend(("value:p0", "value:p1"))
            protocol.append("owner:INPUT")
        if transition_class == "INIT":
            if not nested:
                nested.extend(("value:p0", "value:p1"))
            protocol.extend(("owner:DEST", "owner:RESULT"))
        elif transition_class in {"CONSTRUCT_EMPTY", "CONSTRUCT_VALUES"}:
            protocol.append("owner:RESULT")
        if member in {"DENSE-RESIZE-CLONE", "DENSE-FILL-CLONE"}:
            nested.append("value:SEED")
        if transition_class == "RESIZE" and not any(
            token.startswith(("value:r", "value:c", "value:e", "value:SEED"))
            for token in nested
        ):
            nested.append("value:r0")
        if member == "DENSE-TAKE-WITH-DEFAULT":
            nested.append("value:default0")
        if transition_class == "SPLIT_OFF":
            protocol.append("owner:SUFFIX")
        if transition_class == "EAGER_EXTRACT":
            protocol.append("owner:REMOVED")
        if transition_class == "EAGER_SPLICE":
            protocol.append("owner:REMOVED")
        if transition_class in {"OWN_ITER", "BORROW_ITER"}:
            protocol.append("owner:CURSOR")
        if member in registry.BEHAVIOR_ABORT_MEMBERS:
            retained.append("owner:BEHAVIOR")
        if transition_class in {"CONSTRUCT_EMPTY", "CONSTRUCT_VALUES", "INIT"}:
            values = []
            keys = []
        typed = _typed_value_rows(values, keys)
        typed.update(_typed_value_rows(nested))
        growth_payload_count = sum(
            token.startswith((
                "value:OFFERED", "value:s", "value:e", "value:c",
                "value:r", "value:SEED", "value:replacement",
            ))
            for token in nested
        )
        no_grow_capacity = max(len(values) + growth_payload_count, 4)
        state = make_state(
            values,
            capacity=(
                len(values)
                if code in {"SUCCESS_GROW", "SUCCESS_RELOCATE"}
                else (
                    no_grow_capacity
                    if values or transition_class not in {"CONSTRUCT_EMPTY", "CONSTRUCT_VALUES", "INIT"}
                    else 0
                )
            ),
            nested_input_owners=nested,
            retained_external_owners=retained,
            protocol_owned_owners=protocol,
            typed_values=typed,
        )
        if transition_class in {"CONSTRUCT_EMPTY", "CONSTRUCT_VALUES", "INIT"}:
            state["base_owner"] = None
    if owner_role is not None:
        expected_semantics = owner_role["transition_semantics_id"].split(".", 1)[0]
        if expected_semantics != transition_class:
            raise ValueError("owner-role transition foreign key mismatch")
        state["owner_role_id"] = owner_role["owner_role_id"]
        state["owner_role_authority_sha256"] = sha256_json(owner_role)
    validate_state(state)
    return state


def _move_between_roles(
    state: dict[str, object],
    token: str,
    source_field: str,
    target_field: str,
) -> None:
    source = state[source_field]
    target = state[target_field]
    if token not in source:
        raise ValueError(f"owner token {token} is not in {source_field}")
    source.remove(token)
    target.append(token)


def _remove_nested_for_live(state: dict[str, object], tokens: Iterable[str]) -> None:
    for token in tokens:
        if token not in state["nested_input_owners"]:
            raise ValueError(f"live result token was not a nested input owner: {token}")
        state["nested_input_owners"].remove(token)


def _add_result_borrow(state: dict[str, object], unique: bool) -> None:
    state["borrows"]["active"].append(
        {
            "borrow_id": "borrow:RESULT0",
            "mode": "unique" if unique else "shared",
            "owner_root": state["allocation"]["root_id"],
            "version": state["version"],
            "footprint": [0, 1],
        }
    )


def _record_behavior(state: dict[str, object], count: int, result: str) -> None:
    for ordinal in range(count):
        state["behavior_calls"].append(
            {
                "call_ordinal": ordinal,
                "environment_owner": "owner:BEHAVIOR",
                "receiver_slot": ordinal % max(state["length"], 1),
                "result": result,
                "effect": "declared-effectful-direct-call",
            }
        )


def _record_exact_behavior_call(
    state: dict[str, object],
    operation: str,
    inputs: list[str],
    result: object,
) -> None:
    state["behavior_calls"].append(
        {
            "call_ordinal": len(state["behavior_calls"]),
            "operation": operation,
            "environment_owner": "owner:BEHAVIOR",
            "input_owners": inputs,
            "result": result,
            "effect": "declared-effectful-direct-call",
        }
    )


def _stable_insertion_sort(state: dict[str, object], tokens: list[str]) -> list[str]:
    result: list[str] = []
    for token in tokens:
        insert_at = len(result)
        while insert_at > 0:
            left = token
            right = result[insert_at - 1]
            left_key = state["typed_values"][left]["sort_key"]
            right_key = state["typed_values"][right]["sort_key"]
            comparison = -1 if left_key < right_key else (1 if left_key > right_key else 0)
            _record_exact_behavior_call(state, "COMPARE", [left, right], comparison)
            if comparison >= 0:
                break
            insert_at -= 1
        result.insert(insert_at, token)
    return result


def _exact_transform(
    state: dict[str, object], member: str
) -> tuple[list[str], list[str], list[dict[str, object]]]:
    before = list(state["sequence"])
    destroyed: list[str] = []
    events: list[dict[str, object]] = []
    if member in {
        "DENSE-SORT-STABLE", "DENSE-SORT-STABLE-CACHED-KEY",
        "DENSE-SORT-UNSTABLE", "DENSE-SELECT-UNSTABLE",
    }:
        if member == "DENSE-SORT-STABLE-CACHED-KEY":
            for token in before:
                _record_exact_behavior_call(
                    state, "KEY", [token], state["typed_values"][token]["sort_key"]
                )
        result = _stable_insertion_sort(state, before)
        events.append(
            {
                "event": "EXACT_ORDER_TRANSFORM",
                "algorithm": "stable-insertion-reference" if "STABLE" in member else "frozen-selection-reference",
                "before": before,
                "after": result,
            }
        )
        return result, destroyed, events
    if member.startswith("DENSE-RETAIN"):
        result = []
        for token in before:
            keep = bool(state["typed_values"][token]["predicate_keep"])
            _record_exact_behavior_call(state, "PREDICATE", [token], keep)
            (result if keep else destroyed).append(token)
        events.append({"event": "EXACT_RETAIN", "kept": result, "removed": destroyed})
        return result, destroyed, events
    if member.startswith("DENSE-DEDUP"):
        result = []
        for token in before:
            if not result:
                result.append(token)
                continue
            equal = (
                state["typed_values"][result[-1]]["sort_key"]
                == state["typed_values"][token]["sort_key"]
            )
            _record_exact_behavior_call(state, "ADJACENT_EQUAL", [result[-1], token], equal)
            (destroyed if equal else result).append(token)
        events.append({"event": "EXACT_DEDUP", "kept": result, "removed": destroyed})
        return result, destroyed, events
    if member in {"DENSE-CLONE-FROM", "DENSE-FILL-CLONE", "DENSE-FILL-WITH"}:
        replacements = [
            token for token in state["nested_input_owners"]
            if token.startswith("value:c") or token.startswith("value:e")
        ][: len(before)]
        if member == "DENSE-FILL-CLONE":
            replacements.append("value:SEED")
        if len(replacements) != len(before):
            raise ValueError(f"exact replacement witness count mismatch: {member}")
        for old, new in zip(before, replacements):
            _record_exact_behavior_call(state, "REPLACE_BEHAVIOR", [old], new)
        _remove_nested_for_live(state, replacements)
        destroyed.extend(before)
        events.append({"event": "EXACT_REPLACE_ALL", "old": before, "new": replacements})
        return replacements, destroyed, events
    if member == "DENSE-TAKE-WITH-DEFAULT":
        replacement = "value:default0"
        _record_exact_behavior_call(state, "DEFAULT", [], replacement)
        _remove_nested_for_live(state, [replacement])
        destroyed.extend(before)
        events.append({"event": "EXACT_TAKE_DEFAULT", "old": before, "new": [replacement]})
        return [replacement], destroyed, events
    events.append({"event": "EXACT_DECLARED_IN_PLACE_TRANSFORM", "before": before, "after": before})
    return before, destroyed, events


def _move_protocol_token(state: dict[str, object], token: str, target_field: str) -> None:
    if token in state["protocol_owned_owners"]:
        _move_between_roles(state, token, "protocol_owned_owners", target_field)


def _finish_owner_partition(
    state: dict[str, object],
    member: str,
    transition_class: str,
    code: str,
) -> None:
    if transition_class == "STATIC_REJECT" or code in ABORT_CODES:
        return
    if code in RECOVERABLE_CODES:
        for field in ("protocol_owned_owners", "nested_input_owners", "external_owners"):
            for token in list(state[field]):
                _move_between_roles(state, token, field, "returned_owners")
        return

    if transition_class == "DROP":
        _move_protocol_token(state, "owner:BASE", "destroyed_owners")
    elif transition_class == "OWN_ITER":
        if code == "CURSOR_CREATED":
            _move_protocol_token(state, "owner:BASE", "destroyed_owners")
            _move_protocol_token(state, "owner:CURSOR", "returned_owners")
        elif code in {"YIELD_FRONT", "YIELD_BACK", "TERMINAL_NONE"}:
            _move_protocol_token(state, "owner:BASE", "destroyed_owners")
            _move_protocol_token(state, "owner:CURSOR", "returned_owners")
        elif code == "CLOSE_OR_DROP":
            _move_protocol_token(state, "owner:BASE", "destroyed_owners")
            _move_protocol_token(state, "owner:CURSOR", "destroyed_owners")
    elif transition_class == "BORROW_ITER":
        if code in {"CURSOR_CREATED", "NEXT_SOME", "TERMINAL_NONE"}:
            _move_protocol_token(state, "owner:CURSOR", "returned_owners")
        else:
            _move_protocol_token(state, "owner:CURSOR", "destroyed_owners")
    elif transition_class in {"CONSTRUCT_EMPTY", "CONSTRUCT_VALUES", "INIT"}:
        _move_protocol_token(state, "owner:RESULT", "returned_owners")
        for token in list(state["protocol_owned_owners"]):
            _move_between_roles(state, token, "protocol_owned_owners", "destroyed_owners")
    else:
        _move_protocol_token(state, "owner:BASE", "returned_owners")
        if transition_class == "APPEND":
            _move_protocol_token(state, "owner:SOURCE", "returned_owners")
        if member in registry.PRODUCER_MEMBERS:
            _move_protocol_token(state, "owner:PRODUCER", "destroyed_owners")
        if transition_class in {"EAGER_EXTRACT", "EAGER_SPLICE"}:
            _move_protocol_token(state, "owner:REMOVED", "returned_owners")
        if transition_class == "SPLIT_OFF":
            _move_protocol_token(state, "owner:SUFFIX", "returned_owners")
    if state["nested_input_owners"]:
        raise ValueError(
            f"normal result left nested offered owners undisposed: {state['nested_input_owners']}"
        )


def expected_transition(contract: dict[str, str], initial: dict[str, object]) -> tuple[dict[str, object], list[dict[str, object]], str]:
    state = copy.deepcopy(initial)
    member = contract["member_contract_id"]
    transition_class = transition_class_by_member()[member]
    code = _outcome_code(contract)
    events: list[dict[str, object]] = []

    if transition_class == "STATIC_REJECT":
        diagnostic = f"DENSE-STATIC-REJECT-{member}"
        state["diagnostics"].append(diagnostic)
        events.append({"event": "STATIC_REJECTION", "diagnostic": diagnostic})
        validate_state(state)
        return state, events, diagnostic

    if code in ABORT_CODES:
        if code == "BEHAVIOR_ABORT":
            _record_behavior(state, 1, "ABORT")
        state["terminal"] = "ABORT"
        diagnostic = f"DENSE-{code}"
        state["diagnostics"].append(diagnostic)
        events.append(
            {
                "event": "PRE_ABORT_TERMINAL",
                "failure_point": contract["commitment_point"],
                "diagnostic": diagnostic,
                "invalid_payload_access_count": 0,
            }
        )
        validate_state(state)
        return state, events, diagnostic

    if code in RECOVERABLE_CODES:
        state["terminal"] = "RECOVERABLE_ERROR"
        _finish_owner_partition(state, member, transition_class, code)
        events.append(
            {
                "event": "PRECOMMIT_OWNER_RETURN",
                "failure_point": contract["commitment_point"],
                "returned_owners": list(state["returned_owners"]),
            }
        )
        validate_state(state)
        return state, events, "NONE_NORMAL_ERROR_RESULT"

    if code == "ABSENT":
        events.append({"event": "RETURN_NONE", "owners_changed": []})
        _finish_owner_partition(state, member, transition_class, code)
        validate_state(state)
        return state, events, "NONE"
    if code == "PRESENT":
        _add_result_borrow(state, member.endswith("UNIQ"))
        events.append({"event": "RETURN_BORROW", "borrow_id": "borrow:RESULT0"})
        _finish_owner_partition(state, member, transition_class, code)
        validate_state(state)
        return state, events, "NONE"

    if transition_class == "PRESERVE":
        if code == "SUCCESS" and contract["behavior_calls"] != "ZERO":
            _record_behavior(state, min(state["length"], 2), "NORMAL")
        if "UNIQ" in member or "SPLIT" in member or "FIXED-EACH" in member:
            _add_result_borrow(state, unique=True)
        events.append({"event": "PRESERVE_OR_BORROW", "sequence": list(state["sequence"])})
    elif transition_class == "CONSTRUCT_EMPTY":
        if member == "DENSE-WITH-CAPACITY":
            _acquire_constructed_root(state, 4)
        events.append({"event": "CONSTRUCT_EMPTY", "root": state["allocation"]["root_id"]})
    elif transition_class in {"CONSTRUCT_VALUES", "INIT"}:
        if member == "DENSE-FIXED-MAP":
            produced = ["value:p0", "value:p1"]
            for token in ("value:input0", "value:input1"):
                _move_between_roles(state, token, "nested_input_owners", "destroyed_owners")
        elif any(token.startswith("value:c") for token in state["nested_input_owners"]):
            produced = [token for token in state["nested_input_owners"] if token.startswith("value:c")]
        elif any(token.startswith("value:e") for token in state["nested_input_owners"]):
            produced = [token for token in state["nested_input_owners"] if token.startswith("value:e")]
        else:
            produced = [token for token in state["nested_input_owners"] if token.startswith("value:p")]
        _remove_nested_for_live(state, produced)
        _set_sequence(state, produced, "constructor writes exact output owners")
        _acquire_constructed_root(state, len(produced))
        if contract["behavior_calls"] != "ZERO":
            for token in produced:
                _record_exact_behavior_call(state, "CONSTRUCT_ITEM", [token], "NORMAL")
        events.append({"event": "CONSTRUCT_VALUES", "owners": list(state["sequence"])})
    elif transition_class == "TRANSFORM":
        sequence, removed, transform_events = _exact_transform(state, member)
        for token in removed:
            if token in state["sequence"]:
                state["destroyed_owners"].append(token)
        _set_sequence(state, sequence, f"{member} transform")
        events.extend(transform_events)
    elif transition_class == "GROW_PRESERVE":
        if code in {"SUCCESS_GROW", "SUCCESS_RELOCATE"}:
            _grow_root(state, same_address=False)
            events.append({"event": "ROOT_TRANSFER", "new_root": "root:A1"})
        else:
            events.append({"event": "NO_ROOT_CHANGE", "root": state["allocation"]["root_id"]})
    elif transition_class == "PUSH":
        if code == "SUCCESS_GROW":
            state["capacity"] = state["length"]
            _grow_root(state, same_address=True)
        _set_sequence(state, list(state["sequence"]) + ["value:OFFERED"], "push commit")
        _remove_nested_for_live(state, ["value:OFFERED"])
        if member.endswith("UNIQ"):
            _add_result_borrow(state, unique=True)
        events.append({"event": "PUSH", "destination_index": state["length"] - 1})
    elif transition_class == "INSERT":
        if code == "SUCCESS_GROW":
            state["capacity"] = state["length"]
            _grow_root(state)
        sequence = list(state["sequence"])
        sequence.insert(1, "value:OFFERED")
        _remove_nested_for_live(state, ["value:OFFERED"])
        _set_sequence(state, sequence, "insert relocation")
        if member.endswith("UNIQ"):
            _add_result_borrow(state, unique=True)
        events.append({"event": "INSERT", "index": 1, "relocated_sources": [1, 2]})
    elif transition_class == "APPEND":
        if code == "SUCCESS_GROW":
            _grow_root(state)
        appended = ["value:s0", "value:s1"]
        _remove_nested_for_live(state, appended)
        _set_sequence(state, list(state["sequence"]) + appended, "append move")
        events.append({"event": "APPEND", "source_owners": ["value:s0", "value:s1"]})
    elif transition_class == "EXTEND":
        if code == "SUCCESS_GROW":
            _grow_root(state)
        added = [
            token for token in state["nested_input_owners"]
            if token.startswith("value:e") or token.startswith("value:c")
        ]
        _remove_nested_for_live(state, added)
        _set_sequence(state, list(state["sequence"]) + added, "extend")
        if "ITER" in member:
            for token in added:
                _record_exact_behavior_call(state, "NEXT_SOME", ["owner:PRODUCER"], token)
            _record_exact_behavior_call(state, "NEXT_NONE", ["owner:PRODUCER"], None)
        elif contract["behavior_calls"] != "ZERO":
            for token in added:
                _record_exact_behavior_call(state, "CLONE", [], token)
        events.append({"event": "EXTEND", "added": added})
    elif transition_class == "RESIZE":
        if code == "SUCCESS_GROW":
            _grow_root(state)
        added = [
            token for token in state["nested_input_owners"]
            if token.startswith(("value:r", "value:c", "value:e", "value:SEED"))
        ]
        _remove_nested_for_live(state, added)
        _set_sequence(state, list(state["sequence"]) + added, "resize growth")
        if contract["behavior_calls"] != "ZERO":
            for token in added:
                _record_exact_behavior_call(state, "RESIZE_PRODUCE", [], token)
        events.append({"event": "RESIZE", "new_length": state["length"]})
    elif transition_class == "POP":
        if code == "EMPTY":
            events.append({"event": "POP_EMPTY"})
        else:
            value = state["sequence"][-1]
            _set_sequence(state, list(state["sequence"][:-1]), "pop move-out")
            state["returned_owners"].append(value)
            events.append({"event": "POP_VALUE", "returned_owner": value})
    elif transition_class == "POP_IF":
        if code == "EMPTY":
            if "owner:BEHAVIOR" in state["external_owners"]:
                _move_between_roles(state, "owner:BEHAVIOR", "external_owners", "destroyed_owners")
            events.append({"event": "POP_IF_EMPTY"})
        elif code == "PREDICATE_FALSE":
            _record_exact_behavior_call(state, "PREDICATE", [state["sequence"][-1]], False)
            events.append({"event": "PREDICATE_FALSE"})
        else:
            _record_exact_behavior_call(state, "PREDICATE", [state["sequence"][-1]], True)
            value = state["sequence"][-1]
            _set_sequence(state, list(state["sequence"][:-1]), "pop_if move-out")
            state["returned_owners"].append(value)
            events.append({"event": "PREDICATE_TRUE", "returned_owner": value})
    elif transition_class == "REMOVE":
        value = state["sequence"][1]
        sequence = list(state["sequence"])
        del sequence[1]
        _set_sequence(state, sequence, "ordered remove")
        state["returned_owners"].append(value)
        events.append({"event": "ORDERED_REMOVE", "index": 1, "returned_owner": value})
    elif transition_class == "SWAP_REMOVE":
        sequence = list(state["sequence"])
        value = sequence[1]
        sequence[1] = sequence[-1]
        sequence.pop()
        _set_sequence(state, sequence, "swap remove")
        state["returned_owners"].append(value)
        events.append({"event": "SWAP_REMOVE", "index": 1, "returned_owner": value})
    elif transition_class == "CLEAR":
        state["destroyed_owners"].extend(state["sequence"])
        _set_sequence(state, [], "clear")
        events.append({"event": "CLEAR", "destroyed": list(state["destroyed_owners"])})
    elif transition_class == "TRUNCATE":
        if code == "SUFFIX_DESTROYED":
            state["destroyed_owners"].extend(state["sequence"][1:])
            _set_sequence(state, list(state["sequence"][:1]), "truncate")
        events.append({"event": "TRUNCATE", "new_length": state["length"]})
    elif transition_class == "REPLACE":
        old = state["sequence"][1]
        sequence = list(state["sequence"])
        sequence[1] = "value:OFFERED"
        _remove_nested_for_live(state, ["value:OFFERED"])
        _set_sequence(state, sequence, "replace")
        state["returned_owners"].append(old)
        events.append({"event": "REPLACE", "returned_owner": old})
    elif transition_class == "REVERSE":
        _set_sequence(state, list(reversed(state["sequence"])), "reverse")
        events.append({"event": "REVERSE"})
    elif transition_class == "ROTATE":
        sequence = list(state["sequence"])
        _set_sequence(state, sequence[1:] + sequence[:1], "rotate")
        events.append({"event": "ROTATE", "mid": 1})
    elif transition_class == "SWAP":
        sequence = list(state["sequence"])
        sequence[0], sequence[1] = sequence[1], sequence[0]
        _set_sequence(state, sequence, "swap")
        events.append({"event": "SWAP", "left": 0, "right": 1})
    elif transition_class == "SPLIT_OFF":
        suffix = state["sequence"][1:]
        state["returned_owners"].extend(suffix)
        _set_sequence(state, list(state["sequence"][:1]), "split_off")
        events.append({"event": "SPLIT_OFF", "suffix_owner": "owner:SUFFIX", "suffix_values": suffix})
    elif transition_class == "INTO_OWNER":
        state["version"] += 1
        _refresh_state_facts(state, "owner form conversion")
        events.append({"event": "OWNER_FORM_CONVERSION", "root": state["allocation"]["root_id"]})
    elif transition_class == "BORROW_ITER":
        if code == "CURSOR_CREATED":
            state["protocol_owner"] = "cursor:C0"
            _add_result_borrow(state, unique=member.endswith("UNIQ"))
        elif code == "NEXT_SOME":
            _add_result_borrow(state, unique=member.endswith("UNIQ"))
        elif code == "CURSOR_DESTROYED":
            state["protocol_owner"] = None
            state["borrows"]["invalidated"].extend(
                {"borrow_id": borrow["borrow_id"], "reason": "cursor destroyed"}
                for borrow in state["borrows"]["active"]
            )
            state["borrows"]["active"] = []
        events.append({"event": code})
    elif transition_class == "OWN_ITER":
        if code == "CURSOR_CREATED":
            state["base_owner"] = None
            state["protocol_owner"] = "cursor:C0"
        elif code in {"YIELD_FRONT", "YIELD_BACK"}:
            state["base_owner"] = None
            state["protocol_owner"] = "cursor:C0"
            index = 0 if code == "YIELD_FRONT" else len(state["sequence"]) - 1
            value = state["sequence"][index]
            sequence = list(state["sequence"])
            del sequence[index]
            _set_sequence(state, sequence, code.lower())
            state["returned_owners"].append(value)
        elif code == "TERMINAL_NONE":
            state["base_owner"] = None
            state["protocol_owner"] = "cursor:C0"
        elif code == "CLOSE_OR_DROP":
            state["base_owner"] = None
            state["protocol_owner"] = None
            state["destroyed_owners"].extend(state["sequence"])
            _set_sequence(state, [], "cursor close")
            root = state["allocation"]["root_id"]
            if not state["zst"]:
                state["allocation"]["released_roots"].append(root)
            state["allocation"]["block_owner"] = None
            state["facts"]["invalidated"].extend(
                {"instance_id": fact["instance_id"], "reason": "cursor close"}
                for fact in state["facts"]["active"]
            )
            state["facts"]["active"] = []
        events.append({"event": code})
    elif transition_class == "DROP":
        state["destroyed_owners"].extend(state["sequence"])
        _set_sequence(state, [], "owner drop")
        root = state["allocation"]["root_id"]
        if not state["zst"]:
            state["allocation"]["released_roots"].append(root)
        state["allocation"]["block_owner"] = None
        state["base_owner"] = None
        state["facts"]["active"] = []
        events.append(
            {
                "event": "OWNER_DROP",
                "released_root": root if not state["zst"] else "NONE_ZST_VIRTUAL_ROOT",
            }
        )
    elif transition_class == "EAGER_EXTRACT":
        removed = state["sequence"][1::2]
        retained = state["sequence"][::2]
        for index, token in enumerate(state["sequence"]):
            _record_exact_behavior_call(state, "PREDICATE", [token], index % 2 == 1)
        state["returned_owners"].extend(removed)
        _set_sequence(state, list(retained), "eager extract")
        events.append({"event": "EAGER_EXTRACT", "removed_owner": "owner:REMOVED", "removed_values": removed})
    elif transition_class == "EAGER_SPLICE":
        if code == "SUCCESS_GROW":
            _grow_root(state)
        removed = state["sequence"][1:2]
        replacements = [
            token for token in state["nested_input_owners"]
            if token.startswith("value:replacement")
        ]
        _remove_nested_for_live(state, replacements)
        sequence = [state["sequence"][0]] + replacements + state["sequence"][2:]
        state["returned_owners"].extend(removed)
        _set_sequence(state, sequence, "eager splice")
        for token in replacements:
            _record_exact_behavior_call(state, "NEXT_SOME", ["owner:PRODUCER"], token)
        _record_exact_behavior_call(state, "NEXT_NONE", ["owner:PRODUCER"], None)
        events.append({"event": "EAGER_SPLICE", "removed_values": removed})
    else:
        raise ValueError(f"unimplemented transition class: {transition_class}")

    _finish_owner_partition(state, member, transition_class, code)
    validate_state(state)
    return state, events, "NONE"


def _row_index(rows: Iterable[dict[str, str]], key: str) -> dict[str, dict[str, str]]:
    result: dict[str, dict[str, str]] = {}
    for row in rows:
        value = row[key]
        if value in result:
            raise ValueError(f"duplicate {key}: {value}")
        result[value] = row
    return result


def _case_id(kind: str, *parts: object) -> str:
    body = "::".join(str(part) for part in parts)
    return f"TRACE::{kind}::{body}"


def _case_base(
    kind: str,
    candidate_id: str,
    *identity: object,
    verdict: str = "EXPECTED_PASS",
    diagnostic: str = "NONE",
) -> dict[str, object]:
    return {
        "trace_schema_version": TRACE_SCHEMA_VERSION,
        "case_id": _case_id(kind, candidate_id, *identity),
        "case_kind": kind,
        "candidate_id": candidate_id,
        "verdict": verdict,
        "diagnostic": diagnostic,
        "candidate_execution_authorized": False,
    }


def _select_contract(
    contracts: list[dict[str, str]],
    member_id: str,
    *,
    outcome_code: str | None = None,
    policy_id: str | None = None,
) -> dict[str, str]:
    selected = [row for row in contracts if row["member_contract_id"] == member_id]
    if outcome_code is not None:
        selected = [row for row in selected if _outcome_code(row) == outcome_code]
    if policy_id is not None:
        selected = [row for row in selected if row["policy_variant_id"] == policy_id]
    if not selected:
        raise ValueError((member_id, outcome_code, policy_id))
    return sorted(selected, key=lambda row: row["contract_id"])[0]


def _primary_cases(
    contracts: list[dict[str, str]],
    owner_roles: list[dict[str, str]],
    bindings: list[dict[str, str]],
) -> list[dict[str, object]]:
    contract_by_id = _row_index(contracts, "contract_id")
    role_by_id = _row_index(owner_roles, "owner_role_id")
    result: list[dict[str, object]] = []
    for binding in sorted(
        bindings, key=lambda row: (row["candidate_id"], row["contract_id"])
    ):
        contract = contract_by_id[binding["contract_id"]]
        role = role_by_id[contract["owner_role_foreign_key"]]
        initial = canonical_initial_state(contract, role)
        expected, events, diagnostic = expected_transition(contract, initial)
        code = _outcome_code(contract)
        if transition_class_by_member()[contract["member_contract_id"]] == "STATIC_REJECT":
            kind = "EXACT_STATIC_REJECTION_BINDING"
        elif code in ABORT_CODES:
            kind = "EXACT_PRE_ABORT_BINDING"
        elif code in RECOVERABLE_CODES:
            kind = "EXACT_RECOVERABLE_FAILURE_BINDING"
        else:
            kind = "EXACT_NORMAL_BINDING"
        case = _case_base(kind, binding["candidate_id"], contract["contract_id"])
        case.update(
            {
                "contract_id": contract["contract_id"],
                "member_contract_id": contract["member_contract_id"],
                "outcome_id": contract["outcome_id"],
                "policy_variant_id": contract["policy_variant_id"],
                "profile_id": contract["profile_id"],
                "owner_role_id": role["owner_role_id"],
                "owner_role_authority_sha256": sha256_json(role),
                "transition_semantics_id": role["transition_semantics_id"],
                "operation_id": binding["operation_id"],
                "binding_kind": binding["binding_kind"],
                "lifecycle_class": binding["lifecycle_class"],
                "trigger": contract["trigger"],
                "commitment_point": contract["commitment_point"],
                "initial_state": initial,
                "expected_state": expected,
                "events": events,
                "expected_diagnostic": diagnostic,
                "initial_state_sha256": sha256_json(initial),
                "expected_state_sha256": sha256_json(expected),
                "owner_universe_sha256": initial["owner_universe_sha256"],
                "exact_owner_partition_before": {
                    field: list(initial[field])
                    for field in (
                        "sequence", "nested_input_owners", "external_owners",
                        "returned_owners", "destroyed_owners", "protocol_owned_owners",
                    )
                },
                "exact_owner_partition_after": {
                    field: list(expected[field])
                    for field in (
                        "sequence", "nested_input_owners", "external_owners",
                        "returned_owners", "destroyed_owners", "protocol_owned_owners",
                    )
                },
            }
        )
        result.append(case)
    return result


def _boundary_cases(contracts: list[dict[str, str]]) -> list[dict[str, object]]:
    contract = _select_contract(
        contracts, "DENSE-PUSH", outcome_code="SUCCESS_GROW", policy_id=registry.OD1_RESERVE_FIRST
    )
    result: list[dict[str, object]] = []
    for candidate_id in registry.CANDIDATE_IDS:
        for capacity in range(5):
            values = [f"value:b{index}" for index in range(capacity)]
            initial = make_state(
                values,
                capacity=capacity,
                nested_input_owners=("value:OFFERED",),
                protocol_owned_owners=("owner:BASE",),
                typed_values=_typed_value_rows(values + ["value:OFFERED"]),
            )
            expected, events, diagnostic = expected_transition(contract, initial)
            case = _case_base("CAPACITY_BOUNDARY_0_TO_4", candidate_id, capacity)
            case.update(
                {
                    "contract_id": contract["contract_id"],
                    "capacity": capacity,
                    "length": capacity,
                    "initial_state": initial,
                    "expected_state": expected,
                    "events": events,
                    "expected_diagnostic": diagnostic,
                    "expected_state_sha256": sha256_json(expected),
                }
            )
            result.append(case)
    return result


def _zst_cases(contracts: list[dict[str, str]]) -> list[dict[str, object]]:
    specifications = (
        ("POP", _select_contract(contracts, "DENSE-POP", outcome_code="VALUE_RETURNED")),
        ("DROP", _select_contract(contracts, "DENSE-DROP", outcome_code="OWNER_DESTROYED")),
        ("BORROW", _select_contract(contracts, "DENSE-INDEX-UNIQ", outcome_code="SUCCESS")),
        ("SWAP", _select_contract(contracts, "DENSE-SWAP", outcome_code="SUCCESS")),
    )
    result: list[dict[str, object]] = []
    for candidate_id in registry.CANDIDATE_IDS:
        for label, contract in specifications:
            values = ["zst-owner:0", "zst-owner:1", "zst-owner:2"]
            initial = make_state(
                values,
                capacity=0,
                zst=True,
                protocol_owned_owners=("owner:BASE",),
                typed_values=_typed_value_rows(values),
            )
            expected, events, diagnostic = expected_transition(contract, initial)
            case = _case_base("ZST_LOGICAL_OWNER", candidate_id, label)
            case.update(
                {
                    "zst_operation": label,
                    "contract_id": contract["contract_id"],
                    "initial_state": initial,
                    "expected_state": expected,
                    "events": events,
                    "expected_diagnostic": diagnostic,
                    "logical_owner_count_before": 3,
                    "logical_owner_count_after": len(expected["sequence"]),
                    "payload_allocation_calls": expected["allocation"]["allocation_calls"],
                    "payload_bytes": expected["allocation"]["acquired_bytes"],
                    "address_identity_authority": "NONE",
                }
            )
            result.append(case)
        overflow = _case_base(
            "ZST_LOGICAL_OWNER",
            candidate_id,
            "CHECKED_LEN_OVERFLOW",
            verdict="EXPECTED_REJECTION",
            diagnostic="DENSE-ZST-LENGTH-OVERFLOW-BEFORE-OWNER-TRANSFER",
        )
        overflow.update(
            {
                "zst_operation": "CHECKED_LEN_OVERFLOW",
                "length_before": MAX_USIZE_64,
                "offered_owner_before": "value:OFFERED",
                "offered_owner_after": "value:OFFERED",
                "payload_allocation_calls": 0,
                "payload_bytes": 0,
                "address_identity_authority": "NONE",
            }
        )
        result.append(overflow)
    return result


def _growth_root_cases() -> list[dict[str, object]]:
    result: list[dict[str, object]] = []
    for candidate_id in registry.CANDIDATE_IDS:
        for same_address in (True, False):
            initial = make_state(
                ("value:g0", "value:g1"), capacity=2,
                protocol_owned_owners=("owner:BASE",),
            )
            expected = copy.deepcopy(initial)
            old_address = expected["allocation"]["address_token"]
            _grow_root(expected, same_address=same_address)
            _move_protocol_token(expected, "owner:BASE", "returned_owners")
            validate_state(expected)
            label = "SAME_NUMERICAL_ADDRESS" if same_address else "MOVED_ADDRESS"
            case = _case_base("GROWTH_ROOT_SUCCESS", candidate_id, label)
            case.update(
                {
                    "allocator_result": label,
                    "initial_state": initial,
                    "expected_state": expected,
                    "old_root": initial["allocation"]["root_id"],
                    "new_root": expected["allocation"]["root_id"],
                    "old_address": old_address,
                    "new_address": expected["allocation"]["address_token"],
                    "old_root_fact_valid_after": False,
                    "same_address_implies_same_root": False,
                    "expected_state_sha256": sha256_json(expected),
                }
            )
            result.append(case)
    return result


def _cursor_semantics(stage: str, zst: bool = False) -> dict[str, object]:
    values = ["cursor-owner:0", "cursor-owner:1", "cursor-owner:2"]
    before = {
        "carrier_id": "OD0-AFFINE-SINGLE-LIVE-INTERVAL-v1",
        "master_allocation_count": 0 if zst else 1,
        "root": "root:ZST" if zst else "root:A0",
        "front": 0,
        "back": 3,
        "live_interval": [0, 3],
        "ordered_owners": values,
        "returned_owners": [],
        "destroyed_owners": [],
        "release_count": 0,
        "zst": zst,
        "allocation_calls": 0 if zst else 1,
        "runtime_topology_state": "NONE",
        "proof_topology_state": "NONE",
    }
    after = copy.deepcopy(before)
    events: list[dict[str, object]] = []
    if stage == "CREATED":
        events.append({"event": "CARRIER_CREATED", "interval": [0, 3]})
    elif stage == "YIELD_FRONT":
        owner = after["ordered_owners"].pop(0)
        after["front"] = 1
        after["live_interval"] = [1, 3]
        after["returned_owners"].append(owner)
        events.append({"event": "END_FRONT_THEN_YIELD", "index": 0, "owner": owner})
    elif stage == "YIELD_BACK":
        owner = after["ordered_owners"].pop()
        after["back"] = 2
        after["live_interval"] = [0, 2]
        after["returned_owners"].append(owner)
        events.append({"event": "END_BACK_THEN_YIELD", "index": 2, "owner": owner})
    elif stage == "TERMINAL":
        after["front"] = 3
        after["back"] = 3
        after["live_interval"] = [3, 3]
        after["returned_owners"] = list(after["ordered_owners"])
        after["ordered_owners"] = []
        events.append({"event": "EXHAUST_INTERVAL", "next": "NONE"})
    elif stage == "ABANDON":
        after["destroyed_owners"] = list(after["ordered_owners"])
        after["ordered_owners"] = []
        after["front"] = 3
        after["back"] = 3
        after["live_interval"] = [3, 3]
        after["release_count"] = 0 if zst else 1
        events.append(
            {
                "event": "DROP_EXACT_REMAINDER_THEN_RELEASE_ONCE",
                "owners": list(after["destroyed_owners"]),
                "release_count": after["release_count"],
            }
        )
    else:
        raise ValueError(stage)
    return {"stage": stage, "before": before, "after": after, "events": events}


def _cursor_cases() -> list[dict[str, object]]:
    result: list[dict[str, object]] = []
    for candidate_id in registry.CANDIDATE_IDS:
        for zst in (False, True):
            for stage in ("CREATED", "YIELD_FRONT", "YIELD_BACK", "TERMINAL", "ABANDON"):
                semantics = _cursor_semantics(stage, zst)
                semantic_hash = sha256_json(semantics)
                case = _case_base(
                    "COMMON_OWNING_CURSOR",
                    candidate_id,
                    "ZST" if zst else "POSITIVE_SIZE",
                    stage,
                )
                case.update(
                    {
                        "common_substrate_policy_id": registry.OD0_COMMON_SUBSTRATE,
                        "common_carrier_semantics": semantics,
                        "common_carrier_semantics_sha256": semantic_hash,
                        "candidate_private_cursor_authority": "NONE",
                        "candidate_private_proof_or_runtime_state": "NONE",
                        "conditional_on_unresolved_od0": True,
                    }
                )
                result.append(case)
    return result


def _stored_borrow_cases(stored_rows: list[dict[str, str]]) -> list[dict[str, object]]:
    result: list[dict[str, object]] = []
    for candidate_id in registry.CANDIDATE_IDS:
        for route in stored_rows:
            leaves = [
                {
                    "leaf_id": "leaf:0",
                    "mode": "unique",
                    "external_source_owner": "owner:EXTERNAL0",
                    "external_root": "root:EXTERNAL0",
                    "region": [0, 1],
                    "version": 7,
                }
            ]
            case = _case_base("STORED_BORROW_ROUTE", candidate_id, route["route_id"])
            case.update(
                {
                    "route_id": route["route_id"],
                    "member_contract_id": route["member_contract_id"],
                    "outer_owner_before": "owner:STATE-SOURCE",
                    "outer_owner_after_move": "owner:STATE-DESTINATION",
                    "source_ended_before_destination_live": True,
                    "leaves_before": leaves,
                    "leaves_after_move": copy.deepcopy(leaves),
                    "leaf_roots_are_external": True,
                    "receiver_or_call_frame_roots": [],
                    "normal_leaf_end_count": 1,
                    "failure_leaf_end_count": 1,
                    "region_free_erasure": {
                        "fields": 0,
                        "bytes": 0,
                        "loads": 0,
                        "stores": 0,
                        "branches": 0,
                        "calls": 0,
                        "provenance_metadata": 0,
                        "code_size_delta": 0,
                    },
                    "route_authority_sha256": sha256_json(route),
                }
            )
            result.append(case)
            for attack_id in route["negative_trace_ids"].split(","):
                attack = _case_base(
                    "STORED_BORROW_ATTACK",
                    candidate_id,
                    route["route_id"],
                    attack_id,
                    verdict="EXPECTED_REJECTION",
                    diagnostic=f"DENSE-REJECT-{attack_id}",
                )
                attack.update(
                    {
                        "route_id": route["route_id"],
                        "attack_id": attack_id,
                        "attempted_authority": {
                            "call_frame_leaf": "CALL-FRAME" in attack_id,
                            "stale_root": "STALE-ROOT" in attack_id,
                            "double_drop": "DOUBLE-DROP" in attack_id,
                            "escape": "ESCAPE" in attack_id,
                            "leaf_rebase": "LEAF-REBASE" in attack_id,
                            "owner_loss": "LOSS" in attack_id,
                            "extra_call": "EXTRA-CALL" in attack_id,
                            "partial_drop": "PARTIAL-DROP" in attack_id,
                        },
                        "payload_transition_authorized": False,
                    }
                )
                result.append(attack)
    return result


SCOPED_ATTACKS = (
    "ATTACK-SCOPED-ESCAPE",
    "ATTACK-SCOPED-CAPTURE",
    "ATTACK-SCOPED-RETURN",
    "ATTACK-SCOPED-REENTRY",
    "ATTACK-SCOPED-NONCOLLECT-ALLOCATION",
    "ATTACK-SCOPED-CALL-REPLAY",
    "ATTACK-SCOPED-UNREPAIRED-NORMAL-EXIT",
)


def _scoped_consumer_cases() -> list[dict[str, object]]:
    result: list[dict[str, object]] = []
    for candidate_id in registry.CANDIDATE_IDS:
        for length in range(4):
            source = [f"scoped-owner:{index}" for index in range(length)]
            for stop_after in range(length + 1):
                visited = source[:stop_after]
                unvisited = source[stop_after:]
                for collecting in (False, True):
                    calls = [
                        {
                            "call_ordinal": index,
                            "source_index": index,
                            "owner": token,
                            "consumer_state_before": index,
                            "consumer_state_after": index + 1,
                        }
                        for index, token in enumerate(visited)
                    ]
                    disposition = {
                        "live_dense_owners": unvisited,
                        "returned_payload_owners": visited if collecting else [],
                        "destroyed_payload_owners": [] if collecting else visited,
                        "returned_base_owner": "owner:BASE",
                        "returned_consumer_state": stop_after,
                    }
                    label = "COLLECTING" if collecting else "NONCOLLECTING"
                    case = _case_base(
                        "OD4_SCOPED_CONSUMER",
                        candidate_id,
                        length,
                        stop_after,
                        label,
                    )
                    case.update(
                        {
                            "od4_policy_id": registry.OD4_EAGER_SCOPED,
                            "length": length,
                            "early_stop_after": stop_after,
                            "collecting": collecting,
                            "behavior_calls": calls,
                            "exact_source_call_order": list(range(stop_after)),
                            "later_call_count": 0,
                            "owner_disposition": disposition,
                            "one_valid_dense_owner_on_normal_exit": True,
                            "result_allocation_calls": 1 if collecting and visited else 0,
                            "auxiliary_container_state": "O(1)",
                            "receiver_escaped": False,
                            "consumer_state_escaped": False,
                            "reentry_count": 0,
                            "conditional_on_unresolved_od4": True,
                        }
                    )
                    result.append(case)
        for attack_id in SCOPED_ATTACKS:
            attack = _case_base(
                "OD4_SCOPED_ATTACK",
                candidate_id,
                attack_id,
                verdict="EXPECTED_REJECTION",
                diagnostic=f"DENSE-REJECT-{attack_id}",
            )
            attack.update(
                {
                    "od4_policy_id": registry.OD4_EAGER_SCOPED,
                    "attack_id": attack_id,
                    "normal_exit_authorized": False,
                    "candidate_execution_authorized": False,
                    "conditional_on_unresolved_od4": True,
                }
            )
            result.append(attack)
    return result


def _fact_attack_cases(fact_rows: list[dict[str, str]]) -> list[dict[str, object]]:
    result: list[dict[str, object]] = []
    for candidate_id in registry.CANDIDATE_IDS:
        for fact in fact_rows:
            for attack_id in fact["negative_trace_ids"].split(","):
                case = _case_base(
                    "FACT_CHANNEL_ATTACK",
                    candidate_id,
                    fact["fact_id"],
                    attack_id,
                    verdict="EXPECTED_REJECTION",
                    diagnostic=f"DENSE-REJECT-{attack_id}",
                )
                case.update(
                    {
                        "fact_id": fact["fact_id"],
                        "attack_id": attack_id,
                        "attempted_consumer_dominates_access": False,
                        "attempted_fact_root_or_version_is_current": False,
                        "facts_off_keeps_dynamic_check": True,
                        "payload_access_authorized": False,
                        "fact_authority_sha256": sha256_json(fact),
                    }
                )
                result.append(case)
    return result


LIFECYCLE_ATTACKS = {
    "C-ATOMIC-TRANSITIONS": (
        "ATTACK-ATOMIC-OPEN-NORMAL-EXIT", "ATTACK-ATOMIC-MASTER-DUPLICATION",
        "ATTACK-ATOMIC-CALLBACK-CAPTURE", "ATTACK-PRIVATE-CURSOR-SUBSTITUTION",
    ),
    "C-LINEAR-REBUILD": (
        "ATTACK-LINEAR-OPEN-NORMAL-EXIT", "ATTACK-LINEAR-MASTER-DUPLICATION",
        "ATTACK-LINEAR-THIRD-RANGE", "ATTACK-PRIVATE-CURSOR-SUBSTITUTION",
    ),
    "C-DERIVED-REPAIR": (
        "ATTACK-DERIVED-REPAIR-REJECTS", "ATTACK-DERIVED-REPAIR-ALLOCATES",
        "ATTACK-DERIVED-REPAIR-TRAPS", "ATTACK-DERIVED-REPAIR-CALLS-BEHAVIOR",
        "ATTACK-PRIVATE-CURSOR-SUBSTITUTION",
    ),
    "C-PROOF-CARRYING-STATE": (
        "ATTACK-PROOF-MASTER-COPY", "ATTACK-PROOF-SECOND-MASTER",
        "ATTACK-PROOF-THIRD-LIVE-RANGE", "ATTACK-PRIVATE-CURSOR-SUBSTITUTION",
    ),
    "C-RUNTIME-TOPOLOGY": (
        "ATTACK-RUNTIME-BITMAP", "ATTACK-RUNTIME-PER-SLOT-TAG",
        "ATTACK-RUNTIME-THIRD-RANGE", "ATTACK-RUNTIME-PERSISTENT-TAG",
        "ATTACK-PRIVATE-CURSOR-SUBSTITUTION",
    ),
}


def _lifecycle_attack_cases(lifecycle_rows: list[dict[str, str]]) -> list[dict[str, object]]:
    lifecycle_by_candidate = _row_index(lifecycle_rows, "candidate_id")
    result: list[dict[str, object]] = []
    for candidate_id, attack_ids in LIFECYCLE_ATTACKS.items():
        lifecycle = lifecycle_by_candidate[candidate_id]
        for attack_id in attack_ids:
            case = _case_base(
                "CANDIDATE_LIFECYCLE_ATTACK",
                candidate_id,
                attack_id,
                verdict="EXPECTED_REJECTION",
                diagnostic=f"DENSE-REJECT-{attack_id}",
            )
            case.update(
                {
                    "attack_id": attack_id,
                    "lifecycle_class": lifecycle["lifecycle_class"],
                    "lifecycle_authority_sha256": sha256_json(lifecycle),
                    "normal_exit_authorized": False,
                    "construction_authorized": False,
                }
            )
            result.append(case)
    return result


def build_cases() -> list[dict[str, object]]:
    contracts = registry.build_contract_rows()
    owner_roles = registry.build_owner_role_rows()
    bindings = registry.build_binding_rows()
    lifecycle_rows = registry.build_lifecycle_rows()
    stored_rows = registry.build_stored_borrow_rows()
    fact_rows = registry.build_fact_rows()
    cases: list[dict[str, object]] = []
    cases.extend(_primary_cases(contracts, owner_roles, bindings))
    cases.extend(_boundary_cases(contracts))
    cases.extend(_zst_cases(contracts))
    cases.extend(_growth_root_cases())
    cases.extend(_cursor_cases())
    cases.extend(_stored_borrow_cases(stored_rows))
    cases.extend(_scoped_consumer_cases())
    cases.extend(_fact_attack_cases(fact_rows))
    cases.extend(_lifecycle_attack_cases(lifecycle_rows))
    ids = [case["case_id"] for case in cases]
    if len(ids) != len(set(ids)):
        raise ValueError("duplicate mathematical trace case ID")
    return cases


def _assert_exact_rows(
    name: str,
    actual: list[dict[str, str]],
    expected: list[dict[str, str]],
) -> None:
    if actual != expected:
        raise ValueError(f"{name} differs from the frozen executable registry")


def _validate_lifecycle_meaning(rows: list[dict[str, str]]) -> None:
    by_candidate = _row_index(rows, "candidate_id")
    if set(by_candidate) != set(registry.CANDIDATE_IDS):
        raise ValueError("lifecycle candidate universe mismatch")
    for row in rows:
        joined = " ".join(row.values()).lower()
        if row["construction_authorized"] != "NO":
            raise ValueError("lifecycle authorizes candidate construction")
        if "od-0 candidate-neutral affine single-live-interval carrier" not in row["owning_cursor_shape"].lower():
            raise ValueError("candidate lifecycle substituted a private owning cursor")
        if "common-experimental-substrate" not in row["owning_cursor_closure"].lower():
            raise ValueError("candidate lifecycle lost the unresolved common-substrate gate")
        if "bitmap" in row["partial_state_schema"].lower():
            raise ValueError("candidate lifecycle admits bitmap topology")
        if "manual deallocation" in joined or "unchecked pointer" in joined:
            raise ValueError("candidate lifecycle gained forbidden raw authority")

    proof = by_candidate["C-PROOF-CARRYING-STATE"]
    if proof["maximum_live_ranges"] != "2":
        raise ValueError("proof candidate exceeds two live ranges")
    if "one affine" not in proof["master_allocation_authority"].lower():
        raise ValueError("proof candidate lacks one affine master")
    if "noncopyable" not in proof["partial_state_schema"].lower():
        raise ValueError("proof candidate master is copyable")

    runtime = by_candidate["C-RUNTIME-TOPOLOGY"]
    if runtime["maximum_live_ranges"] != "2":
        raise ValueError("runtime candidate exceeds two live ranges")
    runtime_state = runtime["runtime_partial_state"].lower()
    if not all(fragment in runtime_state for fragment in ("dense/hole", "no bitmap", "no per-slot")):
        raise ValueError("runtime candidate topology is not exact Dense/Hole")

    repair = by_candidate["C-DERIVED-REPAIR"]
    action = repair["automatic_normal_exit_action"].lower()
    if not all(fragment in action for fragment in ("exactly one", "total", "nonallocating", "nontrapping", "behavior-free")):
        raise ValueError("derived repair is not one total zero-effect repair")
    if repair["incomplete_normal_exit"].startswith("REJECT"):
        raise ValueError("derived repair rejects a registered open normal exit")


def validate_protocol_inputs(
    cases: list[dict[str, object]],
    *,
    contracts: list[dict[str, str]] | None = None,
    owner_roles: list[dict[str, str]] | None = None,
    bindings: list[dict[str, str]] | None = None,
    lifecycles: list[dict[str, str]] | None = None,
    operations: list[dict[str, str]] | None = None,
    zst_rows: list[dict[str, str]] | None = None,
    fact_rows: list[dict[str, str]] | None = None,
    stored_rows: list[dict[str, str]] | None = None,
    common_rows: list[dict[str, str]] | None = None,
    od4_rows: list[dict[str, str]] | None = None,
    canonical_cases: list[dict[str, object]] | None = None,
) -> dict[str, int]:
    canonical_contracts = registry.build_contract_rows()
    canonical_roles = registry.build_owner_role_rows()
    canonical_bindings = registry.build_binding_rows()
    canonical_lifecycles = registry.build_lifecycle_rows()
    canonical_operations = registry.build_operation_rows()
    canonical_zst = registry.build_zst_rows()
    canonical_facts = registry.build_fact_rows()
    canonical_stored = registry.build_stored_borrow_rows()
    canonical_common = registry.build_common_substrate_rows()
    canonical_od4 = registry.build_od4_rows()
    actual_contracts = contracts if contracts is not None else canonical_contracts
    actual_roles = owner_roles if owner_roles is not None else canonical_roles
    actual_bindings = bindings if bindings is not None else canonical_bindings
    actual_lifecycles = lifecycles if lifecycles is not None else canonical_lifecycles
    actual_operations = operations if operations is not None else canonical_operations
    actual_zst = zst_rows if zst_rows is not None else canonical_zst
    actual_facts = fact_rows if fact_rows is not None else canonical_facts
    actual_stored = stored_rows if stored_rows is not None else canonical_stored
    actual_common = common_rows if common_rows is not None else canonical_common
    actual_od4 = od4_rows if od4_rows is not None else canonical_od4

    _assert_exact_rows("contracts", actual_contracts, canonical_contracts)
    _assert_exact_rows("owner roles", actual_roles, canonical_roles)
    _assert_exact_rows("candidate bindings", actual_bindings, canonical_bindings)
    _assert_exact_rows("lifecycles", actual_lifecycles, canonical_lifecycles)
    _assert_exact_rows("operations", actual_operations, canonical_operations)
    _assert_exact_rows("ZST policies", actual_zst, canonical_zst)
    _assert_exact_rows("fact channels", actual_facts, canonical_facts)
    _assert_exact_rows("stored-borrow routes", actual_stored, canonical_stored)
    _assert_exact_rows("common substrate", actual_common, canonical_common)
    _assert_exact_rows("OD-4 policies", actual_od4, canonical_od4)
    _validate_lifecycle_meaning(actual_lifecycles)

    contract_by_id = _row_index(actual_contracts, "contract_id")
    role_by_id = _row_index(actual_roles, "owner_role_id")
    binding_keys = {
        (row["candidate_id"], row["contract_id"]) for row in actual_bindings
    }
    expected_binding_keys = {
        (candidate_id, contract_id)
        for candidate_id in registry.CANDIDATE_IDS
        for contract_id in contract_by_id
    }
    if binding_keys != expected_binding_keys or len(actual_bindings) != len(expected_binding_keys):
        raise ValueError("candidate/contract binding product is not exact")
    for contract in actual_contracts:
        role = role_by_id.get(contract["owner_role_foreign_key"])
        if role is None or role["contract_id"] != contract["contract_id"]:
            raise ValueError("contract owner-role foreign key is unresolved")
        if contract["candidate_execution_authorized"] != "NO":
            raise ValueError("contract authorizes candidate execution")
    if any(row["candidate_execution_authorized"] != "NO" for row in actual_roles):
        raise ValueError("owner-role registry authorizes candidate execution")
    if any(row["construction_authorized"] != "NO" for row in actual_bindings):
        raise ValueError("binding registry authorizes candidate construction")
    allowed_operation_statuses = {
        "DESCRIPTION_ONLY_CONSTRUCTION_NOT_AUTHORIZED",
        "REFERENCE_ADAPTER_ONLY_CONSTRUCTION_NOT_AUTHORIZED",
    }
    if any(row["authorization_status"] not in allowed_operation_statuses for row in actual_operations):
        raise ValueError("operation registry authorizes candidate construction")

    ids = [case.get("case_id") for case in cases]
    if len(ids) != len(set(ids)) or any(not value for value in ids):
        raise ValueError("trace case IDs are missing or duplicated")
    if any(case.get("candidate_execution_authorized") is not False for case in cases):
        raise ValueError("trace authorizes candidate execution")
    if canonical_cases is not None and cases != canonical_cases:
        raise ValueError("trace artifact differs from executable mathematical oracle")

    primary = [case for case in cases if str(case["case_kind"]).startswith("EXACT_")]
    primary_keys = {(case["candidate_id"], case["contract_id"]) for case in primary}
    if primary_keys != expected_binding_keys or len(primary) != len(expected_binding_keys):
        raise ValueError("trace does not cover every exact contract/candidate binding once")
    for case in primary:
        contract = contract_by_id[str(case["contract_id"])]
        role = role_by_id[contract["owner_role_foreign_key"]]
        initial = canonical_initial_state(contract, role)
        expected, events, diagnostic = expected_transition(contract, initial)
        if case["initial_state"] != initial:
            raise ValueError(f"primary initial state mismatch: {case['case_id']}")
        validate_state(case["initial_state"])  # type: ignore[arg-type]
        validate_state(case["expected_state"])  # type: ignore[arg-type]
        if case["expected_state"] != expected or case["events"] != events:
            raise ValueError(f"primary transition mismatch: {case['case_id']}")
        if case["expected_diagnostic"] != diagnostic:
            raise ValueError(f"primary diagnostic mismatch: {case['case_id']}")
        if case["initial_state_sha256"] != sha256_json(initial):
            raise ValueError("primary initial-state digest mismatch")
        if case["expected_state_sha256"] != sha256_json(expected):
            raise ValueError("primary expected-state digest mismatch")

    for case in cases:
        if "initial_state" in case:
            validate_state(case["initial_state"])  # type: ignore[arg-type]
        if "expected_state" in case:
            validate_state(case["expected_state"])  # type: ignore[arg-type]

    boundary = [case for case in cases if case["case_kind"] == "CAPACITY_BOUNDARY_0_TO_4"]
    if {(case["candidate_id"], case["capacity"]) for case in boundary} != {
        (candidate_id, capacity)
        for candidate_id in registry.CANDIDATE_IDS for capacity in range(5)
    }:
        raise ValueError("capacity 0..4 boundary matrix is incomplete")

    zst = [case for case in cases if case["case_kind"] == "ZST_LOGICAL_OWNER"]
    if {(case["candidate_id"], case["zst_operation"]) for case in zst} != {
        (candidate_id, operation)
        for candidate_id in registry.CANDIDATE_IDS
        for operation in ("POP", "DROP", "BORROW", "SWAP", "CHECKED_LEN_OVERFLOW")
    }:
        raise ValueError("ZST trace matrix is incomplete")
    if any(case["payload_allocation_calls"] != 0 or case["payload_bytes"] != 0 for case in zst):
        raise ValueError("ZST trace allocates payload storage")

    growth = [case for case in cases if case["case_kind"] == "GROWTH_ROOT_SUCCESS"]
    if len(growth) != len(registry.CANDIDATE_IDS) * 2:
        raise ValueError("same-address/moved-address growth matrix is incomplete")
    for case in growth:
        if case["old_root"] == case["new_root"] or case["old_root_fact_valid_after"]:
            raise ValueError("growth retained old-root authority")
        if case["allocator_result"] == "SAME_NUMERICAL_ADDRESS" and case["old_address"] != case["new_address"]:
            raise ValueError("same-address growth witness is not same-address")

    cursor = [case for case in cases if case["case_kind"] == "COMMON_OWNING_CURSOR"]
    expected_cursor_count = len(registry.CANDIDATE_IDS) * 2 * 5
    if len(cursor) != expected_cursor_count:
        raise ValueError("common cursor matrix is incomplete")
    grouped_cursor_hashes: dict[tuple[bool, str], set[str]] = {}
    for case in cursor:
        semantics = case["common_carrier_semantics"]
        key = (bool(semantics["before"]["zst"]), str(semantics["stage"]))
        grouped_cursor_hashes.setdefault(key, set()).add(str(case["common_carrier_semantics_sha256"]))
        if sha256_json(semantics) != case["common_carrier_semantics_sha256"]:
            raise ValueError("common cursor semantic digest mismatch")
        if case["candidate_private_cursor_authority"] != "NONE" or case["candidate_private_proof_or_runtime_state"] != "NONE":
            raise ValueError("candidate-private cursor mechanism entered a measured arm")
    if any(len(values) != 1 for values in grouped_cursor_hashes.values()):
        raise ValueError("common cursor differs by candidate")

    stored_positive = [case for case in cases if case["case_kind"] == "STORED_BORROW_ROUTE"]
    if {(case["candidate_id"], case["route_id"]) for case in stored_positive} != {
        (candidate_id, row["route_id"])
        for candidate_id in registry.CANDIDATE_IDS for row in actual_stored
    }:
        raise ValueError("stored-borrow route matrix is incomplete")
    for case in stored_positive:
        if case["leaves_before"] != case["leaves_after_move"] or not case["source_ended_before_destination_live"]:
            raise ValueError("stored-borrow move rebased or duplicated a leaf")
        if any(case["region_free_erasure"].values()):
            raise ValueError("stored-borrow region-free erasure has tax")
    expected_stored_attacks = {
        (candidate_id, row["route_id"], attack_id)
        for candidate_id in registry.CANDIDATE_IDS
        for row in actual_stored
        for attack_id in row["negative_trace_ids"].split(",")
    }
    actual_stored_attacks = {
        (case["candidate_id"], case["route_id"], case["attack_id"])
        for case in cases if case["case_kind"] == "STORED_BORROW_ATTACK"
    }
    if actual_stored_attacks != expected_stored_attacks:
        raise ValueError("stored-borrow hostile trace matrix is incomplete")

    scoped = [case for case in cases if case["case_kind"] == "OD4_SCOPED_CONSUMER"]
    for case in scoped:
        calls = case["behavior_calls"]
        if [call["call_ordinal"] for call in calls] != list(range(len(calls))):
            raise ValueError("scoped consumer call ordinals are not exact")
        if [call["source_index"] for call in calls] != case["exact_source_call_order"]:
            raise ValueError("scoped consumer source call order is wrong")
        if case["later_call_count"] != 0 or case["reentry_count"] != 0:
            raise ValueError("scoped consumer replayed or reentered behavior")
        if not case["collecting"] and case["result_allocation_calls"] != 0:
            raise ValueError("noncollecting scoped consumer allocated a result")
        disposition = case["owner_disposition"]
        owners = (
            disposition["live_dense_owners"]
            + disposition["returned_payload_owners"]
            + disposition["destroyed_payload_owners"]
        )
        if len(owners) != case["length"] or len(owners) != len(set(owners)):
            raise ValueError("scoped consumer owner partition is not exact")
    scoped_attack_ids = {
        (case["candidate_id"], case["attack_id"])
        for case in cases if case["case_kind"] == "OD4_SCOPED_ATTACK"
    }
    if scoped_attack_ids != {
        (candidate_id, attack_id)
        for candidate_id in registry.CANDIDATE_IDS for attack_id in SCOPED_ATTACKS
    }:
        raise ValueError("scoped-consumer hostile matrix is incomplete")

    fact_attack_ids = {
        (case["candidate_id"], case["fact_id"], case["attack_id"])
        for case in cases if case["case_kind"] == "FACT_CHANNEL_ATTACK"
    }
    expected_fact_attacks = {
        (candidate_id, row["fact_id"], attack_id)
        for candidate_id in registry.CANDIDATE_IDS
        for row in actual_facts
        for attack_id in row["negative_trace_ids"].split(",")
    }
    if fact_attack_ids != expected_fact_attacks:
        raise ValueError("fact-channel hostile matrix is incomplete")

    lifecycle_attacks = {
        (case["candidate_id"], case["attack_id"])
        for case in cases if case["case_kind"] == "CANDIDATE_LIFECYCLE_ATTACK"
    }
    if lifecycle_attacks != {
        (candidate_id, attack_id)
        for candidate_id, attack_ids in LIFECYCLE_ATTACKS.items()
        for attack_id in attack_ids
    }:
        raise ValueError("candidate-lifecycle hostile matrix is incomplete")

    fresh_cases = canonical_cases if canonical_cases is not None else build_cases()
    if cases != fresh_cases:
        raise ValueError("trace artifact differs from executable mathematical oracle")
    return {
        "contracts": len(actual_contracts),
        "candidate_bindings": len(actual_bindings),
        "cases": len(cases),
        "primary_cases": len(primary),
        "hostile_cases": sum(
            case["verdict"] == "EXPECTED_REJECTION" for case in cases
        ),
    }


def _expect_rejection(name: str, action) -> dict[str, str]:
    try:
        action()
    except (ValueError, KeyError, TypeError, json.JSONDecodeError) as error:
        return {"mutation_id": name, "result": "EXPECTED_REJECTION", "diagnostic": str(error)}
    raise ValueError(f"mutation survived hostile validation: {name}")


def _mutated_cases(
    canonical: list[dict[str, object]],
    mutate,
) -> list[dict[str, object]]:
    result = copy.deepcopy(canonical)
    mutate(result)
    return result


def run_mutation_tests(canonical_cases: list[dict[str, object]]) -> list[dict[str, str]]:
    results: list[dict[str, str]] = []

    def validate_cases(mutated: list[dict[str, object]]) -> None:
        validate_protocol_inputs(mutated, canonical_cases=canonical_cases)

    contracts = registry.build_contract_rows()
    roles = registry.build_owner_role_rows()
    bindings = registry.build_binding_rows()
    lifecycles = registry.build_lifecycle_rows()
    operations = registry.build_operation_rows()
    zst_rows = registry.build_zst_rows()
    facts = registry.build_fact_rows()

    results.append(
        _expect_rejection(
            "MUTATE-DROP-CONTRACT",
            lambda: validate_protocol_inputs(
                canonical_cases, contracts=copy.deepcopy(contracts[:-1]),
                canonical_cases=canonical_cases,
            ),
        )
    )

    def corrupt_outcome() -> None:
        changed = copy.deepcopy(contracts)
        changed[0]["outcome_id"] += "::FORGED"
        validate_protocol_inputs(canonical_cases, contracts=changed, canonical_cases=canonical_cases)

    results.append(_expect_rejection("MUTATE-CORRUPT-OUTCOME", corrupt_outcome))
    results.append(
        _expect_rejection(
            "MUTATE-DROP-BINDING",
            lambda: validate_protocol_inputs(
                canonical_cases, bindings=copy.deepcopy(bindings[:-1]),
                canonical_cases=canonical_cases,
            ),
        )
    )
    results.append(
        _expect_rejection(
            "MUTATE-DROP-OWNER-ROLE",
            lambda: validate_protocol_inputs(
                canonical_cases, owner_roles=copy.deepcopy(roles[:-1]),
                canonical_cases=canonical_cases,
            ),
        )
    )

    def mutate_expected_state(field: str, mutation) -> list[dict[str, object]]:
        def apply(rows: list[dict[str, object]]) -> None:
            case = next(
                row for row in rows
                if str(row["case_kind"]).startswith("EXACT_NORMAL")
                and row["expected_state"]["admitted"]
                and row["expected_state"]["sequence"]
            )
            mutation(case["expected_state"])
            case["expected_state_sha256"] = sha256_json(case["expected_state"])
        return _mutated_cases(canonical_cases, apply)

    results.append(
        _expect_rejection(
            "MUTATE-OWNER-MINT",
            lambda: validate_cases(
                mutate_expected_state(
                    "returned_owners",
                    lambda state: state["returned_owners"].append("owner:FORGED"),
                )
            ),
        )
    )
    results.append(
        _expect_rejection(
            "MUTATE-OWNER-LOSS",
            lambda: validate_cases(
                mutate_expected_state(
                    "owner_universe",
                    lambda state: state["owner_universe"].pop(),
                )
            ),
        )
    )

    def add_extra_return(state: dict[str, object]) -> None:
        token = state["sequence"][0] if state["sequence"] else state["destroyed_owners"][0]
        state["returned_owners"].append(token)

    results.append(
        _expect_rejection(
            "MUTATE-EXTRA-RETURNED-OWNER",
            lambda: validate_cases(mutate_expected_state("returned_owners", add_extra_return)),
        )
    )

    def corrupt_allocation(rows: list[dict[str, object]]) -> None:
        case = next(row for row in rows if row["case_kind"] == "GROWTH_ROOT_SUCCESS")
        case["expected_state"]["allocation"]["root_id"] = case["old_root"]
        case["expected_state_sha256"] = sha256_json(case["expected_state"])

    results.append(
        _expect_rejection(
            "MUTATE-ALLOCATION-ROOT",
            lambda: validate_cases(_mutated_cases(canonical_cases, corrupt_allocation)),
        )
    )

    def corrupt_post_state(rows: list[dict[str, object]]) -> None:
        case = next(
            row for row in rows
            if str(row["case_kind"]).startswith("EXACT_NORMAL")
            and len(row["expected_state"]["sequence"]) >= 2
        )
        case["expected_state"]["sequence"].reverse()
        case["expected_state"]["payload_owner_by_slot"] = {
            str(index): owner for index, owner in enumerate(case["expected_state"]["sequence"])
        }
        case["expected_state_sha256"] = sha256_json(case["expected_state"])

    results.append(
        _expect_rejection(
            "MUTATE-POST-STATE",
            lambda: validate_cases(_mutated_cases(canonical_cases, corrupt_post_state)),
        )
    )

    def corrupt_lifecycle(candidate_id: str, field: str, value: str) -> None:
        changed = copy.deepcopy(lifecycles)
        next(row for row in changed if row["candidate_id"] == candidate_id)[field] = value
        validate_protocol_inputs(canonical_cases, lifecycles=changed, canonical_cases=canonical_cases)

    lifecycle_mutations = (
        ("MUTATE-PROOF-COPYABLE-MASTER", "C-PROOF-CARRYING-STATE", "partial_state_schema", "partition<T>{copyable master_root}"),
        ("MUTATE-PROOF-THIRD-RANGE", "C-PROOF-CARRYING-STATE", "maximum_live_ranges", "3"),
        ("MUTATE-DERIVED-REPAIR-REJECT", "C-DERIVED-REPAIR", "incomplete_normal_exit", "REJECT"),
        ("MUTATE-DERIVED-REPAIR-ALLOCATE", "C-DERIVED-REPAIR", "automatic_normal_exit_action", "Allocate and repair."),
        ("MUTATE-DERIVED-REPAIR-TRAP", "C-DERIVED-REPAIR", "automatic_normal_exit_action", "Trap and repair."),
        ("MUTATE-RUNTIME-BITMAP", "C-RUNTIME-TOPOLOGY", "runtime_partial_state", "One bitmap."),
        ("MUTATE-RUNTIME-THIRD-RANGE", "C-RUNTIME-TOPOLOGY", "maximum_live_ranges", "3"),
        ("MUTATE-PRIVATE-CURSOR", "C-ATOMIC-TRANSITIONS", "owning_cursor_shape", "Candidate-private transition cursor."),
    )
    for name, candidate_id, field, value in lifecycle_mutations:
        results.append(
            _expect_rejection(
                name,
                lambda candidate_id=candidate_id, field=field, value=value: corrupt_lifecycle(
                    candidate_id, field, value
                ),
            )
        )

    def corrupt_fact(fact_id: str, field: str, value: str) -> None:
        changed = copy.deepcopy(facts)
        next(row for row in changed if row["fact_id"] == fact_id)[field] = value
        validate_protocol_inputs(canonical_cases, fact_rows=changed, canonical_cases=canonical_cases)

    results.append(
        _expect_rejection(
            "MUTATE-STALE-FACT-CONSUMER",
            lambda: corrupt_fact("FACT-DENSE-ROOT-VERSION", "consumers", "Old-root payload access."),
        )
    )
    results.append(
        _expect_rejection(
            "MUTATE-FACT-INVALIDATOR-REMOVED",
            lambda: corrupt_fact("FACT-DENSE-LIVE-PREFIX", "invalidators", "NONE"),
        )
    )
    results.append(
        _expect_rejection(
            "MUTATE-LIVE-PREFIX-HOLE",
            lambda: corrupt_fact("FACT-DENSE-LIVE-PREFIX", "producer", "Any Hole state."),
        )
    )

    def corrupt_zst(field: str, value: str) -> None:
        changed = copy.deepcopy(zst_rows)
        next(row for row in changed if row["policy_variant_id"] == registry.OD3_INCLUDE_ZST)[field] = value
        validate_protocol_inputs(canonical_cases, zst_rows=changed, canonical_cases=canonical_cases)

    results.append(
        _expect_rejection(
            "MUTATE-ZST-ALLOCATION",
            lambda: corrupt_zst("payload_allocation", "Allocate one byte."),
        )
    )
    results.append(
        _expect_rejection(
            "MUTATE-ZST-DROP-COUNT",
            lambda: corrupt_zst("drop_rule", "Destroy one owner."),
        )
    )

    def authorize_contract() -> None:
        changed = copy.deepcopy(contracts)
        changed[0]["candidate_execution_authorized"] = "YES"
        validate_protocol_inputs(canonical_cases, contracts=changed, canonical_cases=canonical_cases)

    results.append(_expect_rejection("MUTATE-EXECUTION-AUTHORIZATION", authorize_contract))

    def authorize_case(rows: list[dict[str, object]]) -> None:
        rows[0]["candidate_execution_authorized"] = True

    results.append(
        _expect_rejection(
            "MUTATE-TRACE-EXECUTION-AUTHORIZATION",
            lambda: validate_cases(_mutated_cases(canonical_cases, authorize_case)),
        )
    )

    def wrong_sort(rows: list[dict[str, object]]) -> None:
        case = next(
            row for row in rows
            if row.get("member_contract_id") == "DENSE-SORT-STABLE"
            and row["case_kind"] == "EXACT_NORMAL_BINDING"
        )
        case["expected_state"]["sequence"] = list(reversed(case["expected_state"]["sequence"]))
        case["expected_state"]["payload_owner_by_slot"] = {
            str(index): owner for index, owner in enumerate(case["expected_state"]["sequence"])
        }
        case["expected_state_sha256"] = sha256_json(case["expected_state"])

    results.append(
        _expect_rejection(
            "MUTATE-WRONG-SORT-ORDER",
            lambda: validate_cases(_mutated_cases(canonical_cases, wrong_sort)),
        )
    )

    def wrong_call_order(rows: list[dict[str, object]]) -> None:
        case = next(
            row for row in rows
            if row.get("member_contract_id") == "DENSE-SORT-STABLE"
            and len(row["expected_state"]["behavior_calls"]) >= 2
        )
        calls = case["expected_state"]["behavior_calls"]
        calls[0], calls[1] = calls[1], calls[0]
        case["expected_state_sha256"] = sha256_json(case["expected_state"])

    results.append(
        _expect_rejection(
            "MUTATE-WRONG-BEHAVIOR-CALL-ORDER",
            lambda: validate_cases(_mutated_cases(canonical_cases, wrong_call_order)),
        )
    )

    def mutate_operation_authority() -> None:
        changed = copy.deepcopy(operations)
        changed[0]["authorization_status"] = "AUTHORIZED"
        validate_protocol_inputs(canonical_cases, operations=changed, canonical_cases=canonical_cases)

    results.append(
        _expect_rejection("MUTATE-OPERATION-AUTHORIZATION", mutate_operation_authority)
    )

    with tempfile.TemporaryDirectory(prefix="dense-oracle-") as directory:
        temp_root = Path(directory)
        executable = temp_root / "coherent_executable_registry.py"
        data = registry.CLOSED_COVERAGE_REGISTRY_PATH.read_bytes() + b"\nprint('injected')\n"
        executable.write_bytes(data)
        coherent_sha = sha256_bytes(data)
        results.append(
            _expect_rejection(
                "MUTATE-COHERENT-SHA-EXECUTABLE-REGISTRY",
                lambda: registry._load_closed_coverage_registry(executable, coherent_sha),
            )
        )
        for name, expected_sha in registry.COVERAGE_OUTPUT_SHA256.items():
            mutated_path = temp_root / name
            mutated_path.write_bytes((registry.HERE / name).read_bytes() + b"\n")
            results.append(
                _expect_rejection(
                    f"MUTATE-PINNED-COVERAGE-{name}",
                    lambda mutated_path=mutated_path, expected_sha=expected_sha: registry._read_pinned_coverage_tsv(
                        mutated_path, expected_sha
                    ),
                )
            )

    expected_ids = {row["mutation_id"] for row in results}
    if len(results) != len(expected_ids):
        raise ValueError("duplicate mutation-test ID")
    if any(row["result"] != "EXPECTED_REJECTION" for row in results):
        raise ValueError("a hostile mutation did not fail closed")
    return results


SEMANTIC_DEPENDENCIES = (
    "dense_contract_registry.py",
    "dense_soundness_oracle.py",
    "dense_meta5.py",
    "dense_owner_decisions.py",
    "dense_literal_registry.py",
    "dense_coverage_closed_registry.py",
    "DENSE-EVIDENCE-TARGET-AUTHORITY.tsv",
    "DENSE-EVIDENCE-MEMBER-AUTHORITY.tsv",
    "DENSE-OVERLAY-BRANCH-AUTHORITY.tsv",
    "DENSE-CAPABILITY-UNIT-AUTHORITY.tsv",
    "DENSE-SELECTOR-EXPANSION-AUTHORITY.tsv",
    "DENSE-ROLE-UNIT-AUTHORITY.tsv",
    "DENSE-EXACT-MEMBER-OUTCOME-REGISTRY.tsv",
    "DENSE-OWNER-ROLE-REGISTRY.tsv",
    "DENSE-COMMON-SUBSTRATE-REGISTRY.tsv",
    "DENSE-STORED-BORROW-ROUTE-REGISTRY.tsv",
    "DENSE-OD4-POLICY-REGISTRY.tsv",
    "DENSE-OD1-POLICY-REGISTRY.tsv",
    "DENSE-CANDIDATE-LIFECYCLE-REGISTRY.tsv",
    "DENSE-CANDIDATE-OPERATION-REGISTRY.tsv",
    "DENSE-CANDIDATE-CONTRACT-BINDINGS.tsv",
    "DENSE-CANDIDATE-DISTINCTION-REGISTRY.tsv",
    "DENSE-ZST-POLICY-REGISTRY.tsv",
    "DENSE-FACT-CHANNEL-REGISTRY.tsv",
    "DENSE-SYNTHETIC-UNIT-REGISTRY.tsv",
)


def _trace_bytes(cases: list[dict[str, object]]) -> bytes:
    return "".join(
        json.dumps(case, sort_keys=True, separators=(",", ":")) + "\n"
        for case in cases
    ).encode("utf-8")


def build_manifest(
    cases: list[dict[str, object]],
    trace_data: bytes,
    validation_counts: dict[str, int],
    mutations: list[dict[str, str]],
) -> dict[str, object]:
    dependencies = []
    for name in SEMANTIC_DEPENDENCIES:
        path = HERE / name
        data = path.read_bytes()
        dependencies.append(
            {"path": name, "bytes": len(data), "sha256": sha256_bytes(data)}
        )
    case_kind_counts: dict[str, int] = {}
    for case in cases:
        kind = str(case["case_kind"])
        case_kind_counts[kind] = case_kind_counts.get(kind, 0) + 1
    failure_primary = sum(
        case["case_kind"] in {
            "EXACT_PRE_ABORT_BINDING", "EXACT_RECOVERABLE_FAILURE_BINDING",
        }
        for case in cases
    )
    return {
        "schema_version": "dense-soundness-protocol-manifest-v1",
        "status": "OWNER_REVIEW_PROTOCOL_ONLY",
        "candidate_execution_authorized": False,
        "candidate_construction_authorized": False,
        "selection_or_scoring_authorized": False,
        "compiler_or_specification_change_authorized": False,
        "trace_artifact": {
            "path": TRACE_OUTPUT.name,
            "bytes": len(trace_data),
            "sha256": sha256_bytes(trace_data),
            "case_count": len(cases),
            "case_kind_counts": dict(sorted(case_kind_counts.items())),
            "failure_primary_case_count": failure_primary,
        },
        "validation_counts": validation_counts,
        "mutation_tests": mutations,
        "mutation_test_count": len(mutations),
        "semantic_dependencies": dependencies,
        "coverage_pins": {
            "closed_registry_sha256": registry.CLOSED_COVERAGE_REGISTRY_SHA256,
            "literal_loader_sha256": registry.LITERAL_REGISTRY_LOADER_SHA256,
            "generated_output_sha256": dict(sorted(registry.COVERAGE_OUTPUT_SHA256.items())),
        },
        "unresolved_owner_decisions": ["OD-0", "OD-1", "OD-3", "OD-4"],
        "common_cursor_rule": "All five arms use the identical OD-0 affine single-live-interval carrier; candidate-private substitutions are rejected.",
        "od4_rule": "Scoped consume/fold is conditional on OD-4 owner selection and remains nonescaping, direct, ordered, nonreentrant, and nonallocating unless collecting.",
        "authority_boundary": "This manifest and its traces are mathematical protocol evidence only; they authorize no candidate construction, execution, scoring, selection, language decision, specification change, compiler work, or production fact channel.",
    }


def _manifest_bytes(manifest: dict[str, object]) -> bytes:
    return (json.dumps(manifest, sort_keys=True, indent=2) + "\n").encode("utf-8")


def _read_trace(path: Path) -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    for ordinal, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if not line:
            raise ValueError(f"blank trace line: {ordinal}")
        value = json.loads(line)
        if not isinstance(value, dict):
            raise ValueError(f"non-object trace line: {ordinal}")
        rows.append(value)
    return rows


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--check", action="store_true",
        help="verify committed artifacts are byte-identical to the executable oracle",
    )
    args = parser.parse_args()

    registry_counts = registry.validate_registries()
    cases = build_cases()
    validation_counts = validate_protocol_inputs(cases, canonical_cases=cases)
    mutations = run_mutation_tests(cases)
    trace_data = _trace_bytes(cases)
    manifest = build_manifest(cases, trace_data, validation_counts, mutations)
    manifest_data = _manifest_bytes(manifest)

    if args.check:
        if not TRACE_OUTPUT.exists() or TRACE_OUTPUT.read_bytes() != trace_data:
            raise SystemExit("dense soundness trace artifact is stale")
        parsed = _read_trace(TRACE_OUTPUT)
        validate_protocol_inputs(parsed, canonical_cases=cases)
        if not MANIFEST_OUTPUT.exists() or MANIFEST_OUTPUT.read_bytes() != manifest_data:
            raise SystemExit("dense soundness protocol manifest is stale")
    else:
        TRACE_OUTPUT.write_bytes(trace_data)
        MANIFEST_OUTPUT.write_bytes(manifest_data)

    summary = {
        **registry_counts,
        **validation_counts,
        "mutation_tests": len(mutations),
        "trace_sha256": sha256_bytes(trace_data),
        "manifest_sha256": sha256_bytes(manifest_data),
    }
    print(f"dense mathematical soundness oracle: PASS {json.dumps(summary, sort_keys=True)}")


if __name__ == "__main__":
    main()
