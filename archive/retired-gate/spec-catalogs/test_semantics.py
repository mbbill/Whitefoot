#!/usr/bin/env python3
"""Hostile tests for the authored semantic-decomposition contract."""

from __future__ import annotations

import copy
import contextlib
import io
import json
import os
import subprocess
import sys
import tempfile
import unittest
from collections import defaultdict
from pathlib import Path
from unittest import mock

import semantics


SPECIFICATION = semantics.SPEC_PATH.read_bytes()
SOURCE_INDEX_BYTES = semantics.SOURCE_INDEX_PATH.read_bytes()
SOURCE_INDEX = semantics.parse_strict_json(
    SOURCE_INDEX_BYTES, "test source index"
)

# This is an independent test oracle, not a view of the production table.  A
# disposition-table edit therefore requires an intentional test-contract edit.
EXPECTED_REVIEWED_DISPOSITIONS = (
    (
        "explicitly-deferred",
        "FORM-5",
        15_749,
        15_788,
        "0cbec3066cc7e216554e03b29f95f8664198f04cdbd6df0e0c7a97e791a49ed4",
    ),
    (
        "explicitly-deferred",
        "LEX-1",
        17_941,
        18_098,
        "23d98d39550622dd24d013800c1262209082accd33b73fa02ac3d840d7c4bacb",
    ),
    (
        "explicitly-nonnormative",
        "OWN-9",
        41_341,
        41_809,
        "1853bc29443f66b3e54c800026931ba0e380278a6a42980768580457b2d912a2",
    ),
)
UNAUTHORIZED_META_5_MARKER = (
    "explicitly-deferred",
    "META-5",
    97_705,
    97_752,
    "f5e0ed0496f9455a2ec0dc9c97a3f01223d18e1cec605f7a785c37670577fa07",
)


def exact_hash(start: int, end: int) -> str:
    return semantics.sha256(SPECIFICATION[start:end])


def source_atoms() -> tuple[dict[str, list[str]], list[str], list[str]]:
    positioned: list[tuple[int, int, str, str]] = []
    positioned_all: list[tuple[int, int, str, str]] = []
    for rule in SOURCE_INDEX["rules"]:
        source = rule["source"]
        positioned_all.append(
            (
                source["byte_start"],
                source["byte_end"],
                rule["id"],
                rule["rule_id"],
            )
        )
    for collection in semantics.CHILD_COLLECTIONS:
        for atom in SOURCE_INDEX[collection]:
            source = atom["source"]
            positioned.append(
                (
                    source["byte_start"],
                    source["byte_end"],
                    atom["id"],
                    atom["owner_rule"],
                )
            )
    positioned.sort(key=lambda item: (item[0], item[1], item[2].encode("ascii")))
    positioned_all.extend(positioned)
    positioned_all.sort(
        key=lambda item: (item[0], item[1], item[2].encode("ascii"))
    )
    by_owner: dict[str, list[str]] = defaultdict(list)
    for _, _, atom_id, owner in positioned:
        by_owner[owner].append(atom_id)
    return (
        by_owner,
        [item[2] for item in positioned],
        [item[2] for item in positioned_all],
    )


CHILDREN_BY_OWNER, CHILD_ORDER, ATOM_ORDER = source_atoms()
RULE_SOURCE = {rule["rule_id"]: rule["source"] for rule in SOURCE_INDEX["rules"]}
RULES = sorted(RULE_SOURCE, key=str.encode)

# The synthetic one-facet-per-rule data below is structural-only.  It exercises
# the catalog contract and makes no claim about the rule's semantic facets.

def facet_for(rule: str) -> dict[str, object]:
    return {
        "id": f"facet:{rule}/semantic-obligation",
        "owner_rule": rule,
        "source_atoms": [f"rule:{rule}", *CHILDREN_BY_OWNER[rule]],
        "owner_stage": "semantic-check",
        "required_lanes": ["frontend", "checker", "verifier"],
        "required_evidence": ["conformance-accept", "diagnostic", "static-audit"],
    }


def clause_for(rule: str) -> dict[str, object]:
    source = RULE_SOURCE[rule]
    return {
        "owner_rule": rule,
        "byte_start": source["byte_start"],
        "byte_end": source["byte_end"],
        "sha256": source["sha256"],
        "disposition": "facet",
        "facet_ids": [f"facet:{rule}/semantic-obligation"],
    }


def fragment_for(rules: list[str]) -> dict[str, object]:
    return {
        "schema": 1,
        "kind": "whitefoot-semantic-decomposition-fragment",
        "rules": sorted(rules, key=str.encode),
        "clauses": sorted(
            (clause_for(rule) for rule in rules),
            key=lambda clause: clause["byte_start"],
        ),
        "facets": sorted(
            (facet_for(rule) for rule in rules),
            key=lambda facet: facet["id"].encode("ascii"),
        ),
    }


def valid_fragments() -> list[dict[str, object]]:
    """Return complete structural-only fixtures, never semantic evidence."""
    return [fragment_for(RULES)]


def build(fragments: list[dict[str, object]]) -> dict[str, object]:
    """Build a structural-only synthetic fixture, never production evidence."""
    return semantics.build_static_catalog(
        [semantics.canonical_bytes(fragment) for fragment in fragments],
        SPECIFICATION,
        SOURCE_INDEX_BYTES,
    )


def locate(fragment: dict[str, object], collection: str, owner: str) -> dict[str, object]:
    return next(
        entry
        for entry in fragment[collection]
        if entry["owner_rule"] == owner
    )


def replace_with_reviewed_exclusion(
    fragment: dict[str, object],
    reviewed: tuple[str, str, int, int, str],
) -> None:
    """Apply one exact reviewed exclusion to a structural-only fixture."""
    disposition, rule, start, end, digest = reviewed
    original = locate(fragment, "clauses", rule)
    rule_start = original["byte_start"]
    rule_end = original["byte_end"]
    if not (rule_start <= start < end <= rule_end):
        raise AssertionError(f"reviewed exclusion escapes {rule}")
    if exact_hash(start, end) != digest:
        raise AssertionError(f"reviewed exclusion hash is stale for {rule}")
    replacement = []
    if rule_start < start:
        prefix = copy.deepcopy(original)
        prefix["byte_end"] = start
        prefix["sha256"] = exact_hash(rule_start, start)
        replacement.append(prefix)
    replacement.append(
        {
            "owner_rule": rule,
            "byte_start": start,
            "byte_end": end,
            "sha256": digest,
            "disposition": disposition,
            "facet_ids": [],
        }
    )
    if end < rule_end:
        suffix = copy.deepcopy(original)
        suffix["byte_start"] = end
        suffix["sha256"] = exact_hash(end, rule_end)
        replacement.append(suffix)
    index = fragment["clauses"].index(original)
    fragment["clauses"][index : index + 1] = replacement
    if start == rule_start and end == rule_end:
        fragment["facets"].remove(locate(fragment, "facets", rule))


class StaticCatalogTests(unittest.TestCase):
    def test_complete_in_memory_decomposition_builds_closed_catalog(self) -> None:
        catalog = build(valid_fragments())
        self.assertEqual(
            set(catalog),
            {
                "schema",
                "kind",
                "specification",
                "source_index",
                "decomposition_sha256",
                "clauses",
                "facets",
                "source_atom_coverage",
            },
        )
        self.assertEqual(catalog["schema"], 1)
        self.assertEqual(catalog["kind"], "whitefoot-static-semantic-catalog")
        self.assertEqual(
            catalog["specification"],
            {
                "path": "spec/kernel-spec-v0.9.md",
                "version": "0.9",
                "sha256": semantics.SPEC_SHA256,
            },
        )
        self.assertEqual(
            catalog["source_index"],
            {
                "path": "facets/v0.9/source.json",
                "sha256": semantics.sha256(SOURCE_INDEX_BYTES),
            },
        )
        self.assertEqual(len(catalog["clauses"]), 92)
        self.assertEqual(len(catalog["facets"]), 92)
        self.assertEqual(len(catalog["source_atom_coverage"]), 204)
        self.assertEqual(
            [entry["source_atom"] for entry in catalog["source_atom_coverage"]],
            ATOM_ORDER,
        )
        normalized = {"clauses": catalog["clauses"], "facets": catalog["facets"]}
        self.assertEqual(
            catalog["decomposition_sha256"],
            semantics.sha256(semantics.canonical_bytes(normalized)),
        )
        forbidden = {"handler", "status", "test", "witness", "prose"}
        self.assertFalse(forbidden & set(json.dumps(catalog).split('"')))

    def test_fragment_file_order_does_not_change_normalized_catalog(self) -> None:
        even = fragment_for(RULES[::2])
        odd = fragment_for(RULES[1::2])
        self.assertEqual(build([even, odd]), build([odd, even]))

    def test_canonical_output_is_ascii_and_deterministic(self) -> None:
        first = semantics.canonical_bytes(build(valid_fragments()))
        second = semantics.canonical_bytes(build(valid_fragments()))
        self.assertEqual(first, second)
        self.assertEqual(first[-1:], b"\n")
        self.assertNotEqual(first[-2:], b"\n\n")
        first.decode("ascii")


class StrictJsonTests(unittest.TestCase):
    def test_canonical_fragment_round_trips(self) -> None:
        fragment = valid_fragments()[0]
        raw = semantics.canonical_bytes(fragment)
        self.assertEqual(semantics.parse_fragment_bytes(raw), fragment)

    def test_duplicate_keys_floats_non_ascii_and_trailing_bytes_fail(self) -> None:
        hostile = (
            b'{"schema":1,"schema":1}',
            b'{"schema":1.0}\n',
            b'{"name":"\xc3\xa9"}\n',
            semantics.canonical_bytes({"schema": 1}) + b" ",
        )
        for raw in hostile:
            with self.subTest(raw=raw):
                with self.assertRaises(semantics.SemanticCatalogError):
                    semantics.parse_fragment_bytes(raw)

    def test_noncanonical_key_order_or_whitespace_fails(self) -> None:
        for raw in (b'{"b": 1, "a": 2}\n', b'{"a":2}\n', b'{"a": 2}'):
            with self.subTest(raw=raw):
                with self.assertRaises(semantics.SemanticCatalogError):
                    semantics.parse_fragment_bytes(raw)

    def test_canonical_encoder_rejects_all_floats_and_round_trips_supported_values(self) -> None:
        for value in (0.0, -1.25, float("nan"), float("inf"), float("-inf")):
            with self.subTest(forbidden=value):
                with self.assertRaises(semantics.SemanticCatalogError):
                    semantics.canonical_bytes(value)

        supported = (
            None,
            True,
            False,
            0,
            -123,
            "ASCII text",
            [],
            [None, True, 7, "value"],
            {},
            {"array": [False, 9], "object": {"key": "value"}},
        )
        for value in supported:
            with self.subTest(supported=value):
                raw = semantics.canonical_bytes(value)
                self.assertEqual(
                    semantics.parse_strict_json(raw, max_bytes=len(raw)),
                    value,
                )

    def test_unknown_keys_fail_at_every_authored_layer(self) -> None:
        for collection, owner in ((None, None), ("clauses", "SCOPE-1"), ("facets", "SCOPE-1")):
            fragments = valid_fragments()
            if collection is None:
                fragments[0]["unknown"] = 1
            else:
                locate(fragments[0], collection, owner)["unknown"] = 1
            with self.subTest(collection=collection):
                with self.assertRaisesRegex(semantics.SemanticCatalogError, "keys differ"):
                    build(fragments)


class ResourceBoundTests(unittest.TestCase):
    def test_raw_depth_integer_string_and_list_limits_accept_limit_reject_plus_one(self) -> None:
        raw = semantics.canonical_bytes({"ok": True})
        self.assertEqual(
            semantics.parse_strict_json(raw, max_bytes=len(raw)), {"ok": True}
        )
        with self.assertRaises(semantics.SemanticCatalogError):
            semantics.parse_strict_json(raw, max_bytes=len(raw) - 1)

        with mock.patch.object(semantics, "MAX_JSON_DEPTH", 3):
            semantics.parse_strict_json(
                semantics.canonical_bytes([[[0]]])
            )
            with self.assertRaisesRegex(semantics.SemanticCatalogError, "depth"):
                semantics.parse_strict_json(
                    semantics.canonical_bytes([[[[0]]]])
                )

        with mock.patch.object(semantics, "MAX_INTEGER_DIGITS", 2):
            semantics.parse_strict_json(semantics.canonical_bytes(99))
            with self.assertRaisesRegex(semantics.SemanticCatalogError, "digit"):
                semantics.parse_strict_json(semantics.canonical_bytes(100))

        with mock.patch.object(semantics, "MAX_JSON_STRING_BYTES", 3):
            semantics.parse_strict_json(semantics.canonical_bytes("abc"))
            with self.assertRaisesRegex(semantics.SemanticCatalogError, "string"):
                semantics.parse_strict_json(semantics.canonical_bytes("abcd"))

        with mock.patch.object(semantics, "MAX_JSON_LIST_ITEMS", 2):
            semantics.parse_strict_json(semantics.canonical_bytes([0, 1]))
            with self.assertRaisesRegex(semantics.SemanticCatalogError, "list"):
                semantics.parse_strict_json(
                    semantics.canonical_bytes([0, 1, 2])
                )
        with mock.patch.object(semantics, "MAX_JSON_OBJECT_FIELDS", 2):
            semantics.parse_strict_json(
                semantics.canonical_bytes({"a": 0, "b": 1})
            )
            with self.assertRaisesRegex(semantics.SemanticCatalogError, "object"):
                semantics.parse_strict_json(
                    semantics.canonical_bytes({"a": 0, "b": 1, "c": 2})
                )
        with mock.patch.object(semantics, "MAX_JSON_NODES", 3):
            semantics.parse_strict_json(semantics.canonical_bytes([0, 1]))
            with self.assertRaisesRegex(semantics.SemanticCatalogError, "node"):
                semantics.parse_strict_json(
                    semantics.canonical_bytes([0, 1, 2])
                )

    def test_fragment_byte_count_clause_and_facet_limits_have_boundaries(self) -> None:
        fragment = semantics.canonical_bytes(valid_fragments()[0])
        with mock.patch.object(semantics, "MAX_FRAGMENT_COUNT", 1):
            semantics.build_static_catalog(
                [fragment], SPECIFICATION, SOURCE_INDEX_BYTES
            )
            with self.assertRaisesRegex(semantics.SemanticCatalogError, "fragment count"):
                semantics.build_static_catalog(
                    [fragment, fragment], SPECIFICATION, SOURCE_INDEX_BYTES
                )
        with mock.patch.object(semantics, "MAX_FRAGMENT_BYTES", len(fragment)):
            semantics.build_static_catalog(
                [fragment], SPECIFICATION, SOURCE_INDEX_BYTES
            )
        with mock.patch.object(semantics, "MAX_FRAGMENT_BYTES", len(fragment) - 1):
            with self.assertRaisesRegex(semantics.SemanticCatalogError, "fragment 0"):
                semantics.build_static_catalog(
                    [fragment], SPECIFICATION, SOURCE_INDEX_BYTES
                )
        with mock.patch.object(
            semantics, "MAX_FRAGMENT_TOTAL_BYTES", len(fragment)
        ):
            build(valid_fragments())
        with mock.patch.object(
            semantics, "MAX_FRAGMENT_TOTAL_BYTES", len(fragment) - 1
        ):
            with self.assertRaisesRegex(semantics.SemanticCatalogError, "aggregate"):
                build(valid_fragments())
        with mock.patch.object(semantics, "MAX_CLAUSE_COUNT", 92):
            build(valid_fragments())
        with mock.patch.object(semantics, "MAX_CLAUSE_COUNT", 91):
            with self.assertRaisesRegex(semantics.SemanticCatalogError, "clause count"):
                build(valid_fragments())
        with mock.patch.object(semantics, "MAX_FACET_COUNT", 92):
            build(valid_fragments())
        with mock.patch.object(semantics, "MAX_FACET_COUNT", 91):
            with self.assertRaisesRegex(semantics.SemanticCatalogError, "facet count"):
                build(valid_fragments())

    def test_normalized_output_limit_and_parser_runtime_failures_are_controlled(self) -> None:
        catalog = build(valid_fragments())
        catalog_size = len(semantics.canonical_bytes(catalog))
        with mock.patch.object(semantics, "MAX_NORMALIZED_OUTPUT_BYTES", catalog_size):
            build(valid_fragments())
        with mock.patch.object(
            semantics, "MAX_NORMALIZED_OUTPUT_BYTES", catalog_size - 1
        ):
            with self.assertRaisesRegex(semantics.SemanticCatalogError, "normalized-output"):
                build(valid_fragments())
        raw = semantics.canonical_bytes({"ok": True})
        for error in (RecursionError("deep"), OverflowError("large"), ValueError("bad")):
            with self.subTest(error=type(error).__name__), mock.patch.object(
                semantics.semantics_io.json, "loads", side_effect=error
            ):
                with self.assertRaises(semantics.SemanticCatalogError):
                    semantics.parse_strict_json(raw)


class PartitionAndClauseTests(unittest.TestCase):
    def test_rules_must_be_a_unique_bytewise_sorted_exact_partition(self) -> None:
        mutations = []
        missing = valid_fragments()
        missing[0]["rules"].pop()
        mutations.append(missing)
        duplicate = valid_fragments()
        duplicate[0]["rules"].append(duplicate[0]["rules"][-1])
        mutations.append(duplicate)
        unsorted = valid_fragments()
        unsorted[0]["rules"][0], unsorted[0]["rules"][1] = (
            unsorted[0]["rules"][1], unsorted[0]["rules"][0]
        )
        mutations.append(unsorted)
        for fragments in mutations:
            with self.subTest(rules=fragments[0]["rules"][:3]):
                with self.assertRaises(semantics.SemanticCatalogError):
                    build(fragments)

    def test_clause_hash_gap_overlap_escape_and_order_fail(self) -> None:
        for mode in ("hash", "gap", "overlap", "escape", "order"):
            fragments = valid_fragments()
            fragment = fragments[0]
            clause = locate(fragment, "clauses", "SCOPE-1")
            if mode == "hash":
                clause["sha256"] = "0" * 64
            elif mode in {"gap", "overlap"}:
                original_end = clause["byte_end"]
                middle = clause["byte_start"] + 10
                first = copy.deepcopy(clause)
                second = copy.deepcopy(clause)
                first["byte_end"] = middle + (1 if mode == "overlap" else -1)
                first["sha256"] = exact_hash(first["byte_start"], first["byte_end"])
                second["byte_start"] = middle
                second["sha256"] = exact_hash(second["byte_start"], original_end)
                index = fragment["clauses"].index(clause)
                fragment["clauses"][index:index + 1] = [first, second]
            elif mode == "escape":
                clause["byte_end"] += 1
                clause["sha256"] = exact_hash(clause["byte_start"], clause["byte_end"])
            else:
                fragment["clauses"][0], fragment["clauses"][1] = (
                    fragment["clauses"][1], fragment["clauses"][0]
                )
            with self.subTest(mode=mode):
                with self.assertRaises(semantics.SemanticCatalogError):
                    build(fragments)

    def test_dispositions_are_closed_and_exclusion_requires_reviewed_tuple(self) -> None:
        for disposition in ("unknown", "explicitly-deferred", "explicitly-nonnormative"):
            fragments = valid_fragments()
            clause = locate(fragments[0], "clauses", "SCOPE-1")
            clause["disposition"] = disposition
            clause["facet_ids"] = []
            with self.subTest(disposition=disposition):
                with self.assertRaises(semantics.SemanticCatalogError):
                    build(fragments)

    def test_reviewed_disposition_table_matches_exact_spec_bound_contract(self) -> None:
        self.assertEqual(
            semantics.REVIEWED_DISPOSITIONS,
            EXPECTED_REVIEWED_DISPOSITIONS,
        )

    def test_every_exact_reviewed_disposition_can_be_excluded(self) -> None:
        for reviewed in EXPECTED_REVIEWED_DISPOSITIONS:
            fragments = valid_fragments()
            replace_with_reviewed_exclusion(fragments[0], reviewed)
            with self.subTest(rule=reviewed[1]):
                build(fragments)

    def test_exact_meta_5_deferred_marker_span_has_no_exclusion_authority(self) -> None:
        fragments = valid_fragments()
        replace_with_reviewed_exclusion(
            fragments[0], UNAUTHORIZED_META_5_MARKER
        )
        with self.assertRaisesRegex(
            semantics.SemanticCatalogError, "exact reviewed"
        ):
            build(fragments)

    def test_markers_meta_5_and_expanded_spans_have_no_authority(self) -> None:
        for rule, disposition in (
            ("FORM-7", "explicitly-deferred"),
            ("FN-4", "explicitly-nonnormative"),
            ("META-5", "explicitly-deferred"),
        ):
            fragments = valid_fragments()
            fragment = fragments[0]
            clause = locate(fragment, "clauses", rule)
            clause["disposition"] = disposition
            clause["facet_ids"] = []
            facet = locate(fragment, "facets", rule)
            fragment["facets"].remove(facet)
            with self.subTest(rule=rule):
                with self.assertRaisesRegex(
                    semantics.SemanticCatalogError,
                    "exact reviewed",
                ):
                    build(fragments)

    def test_fn_4_law_table_marker_has_no_exclusion_authority(self) -> None:
        fragments = valid_fragments()
        marker = b"The v0.9 law table is closed:"
        start = SPECIFICATION.index(marker)
        replace_with_reviewed_exclusion(
            fragments[0],
            (
                "explicitly-nonnormative",
                "FN-4",
                start,
                start + len(marker),
                semantics.sha256(marker),
            ),
        )
        with self.assertRaisesRegex(semantics.SemanticCatalogError, "exact reviewed"):
            build(fragments)

    def test_facet_clause_requires_nonempty_sorted_same_owner_ids(self) -> None:
        for mode in ("empty", "duplicate", "unsorted", "cross-owner"):
            fragments = valid_fragments()
            clause = locate(fragments[0], "clauses", "SCOPE-1")
            if mode == "empty":
                clause["facet_ids"] = []
            elif mode == "duplicate":
                clause["facet_ids"] *= 2
            elif mode == "unsorted":
                second = copy.deepcopy(locate(fragments[0], "facets", "SCOPE-1"))
                second["id"] = "facet:SCOPE-1/another-obligation"
                fragments[0]["facets"].append(second)
                fragments[0]["facets"].sort(key=lambda entry: entry["id"].encode("ascii"))
                clause["facet_ids"] = [
                    "facet:SCOPE-1/semantic-obligation",
                    "facet:SCOPE-1/another-obligation",
                ]
            else:
                clause["facet_ids"] = ["facet:SCOPE-2/semantic-obligation"]
            with self.subTest(mode=mode):
                with self.assertRaises(semantics.SemanticCatalogError):
                    build(fragments)


class FacetContractTests(unittest.TestCase):
    def test_bad_owner_mismatched_and_duplicate_facet_ids_fail(self) -> None:
        for mode in ("bad", "mismatch", "duplicate"):
            fragments = valid_fragments()
            facet = locate(fragments[0], "facets", "SCOPE-1")
            clause = locate(fragments[0], "clauses", "SCOPE-1")
            if mode == "bad":
                facet["id"] = "facet:SCOPE-1/not_valid"
                clause["facet_ids"] = [facet["id"]]
            elif mode == "mismatch":
                facet["id"] = "facet:SCOPE-2/semantic-obligation"
                clause["facet_ids"] = [facet["id"]]
            else:
                duplicate = copy.deepcopy(facet)
                fragments[0]["facets"].append(duplicate)
            fragments[0]["facets"].sort(key=lambda entry: entry["id"].encode("ascii"))
            with self.subTest(mode=mode):
                with self.assertRaises(semantics.SemanticCatalogError):
                    build(fragments)

    def test_every_facet_must_be_mapped_by_a_clause(self) -> None:
        fragments = valid_fragments()
        extra = copy.deepcopy(locate(fragments[0], "facets", "SCOPE-1"))
        extra["id"] = "facet:SCOPE-1/unmapped-obligation"
        fragments[0]["facets"].append(extra)
        fragments[0]["facets"].sort(key=lambda entry: entry["id"].encode("ascii"))
        with self.assertRaisesRegex(semantics.SemanticCatalogError, "not mapped"):
            build(fragments)

    def test_source_atoms_must_be_complete_unique_same_owner_and_index_ordered(self) -> None:
        owner = "GRAM-2"
        self.assertGreater(len(CHILDREN_BY_OWNER[owner]), 2)
        for mode in ("missing-rule", "missing-child", "duplicate", "cross-owner", "unsorted"):
            fragments = valid_fragments()
            facet = locate(fragments[0], "facets", owner)
            if mode == "missing-rule":
                facet["source_atoms"].pop(0)
            elif mode == "missing-child":
                facet["source_atoms"].pop()
            elif mode == "duplicate":
                facet["source_atoms"].append(facet["source_atoms"][-1])
            elif mode == "cross-owner":
                facet["source_atoms"] = ["rule:GRAM-3"]
            else:
                facet["source_atoms"][0], facet["source_atoms"][1] = (
                    facet["source_atoms"][1], facet["source_atoms"][0]
                )
            with self.subTest(mode=mode):
                with self.assertRaises(semantics.SemanticCatalogError):
                    build(fragments)

    def test_stages_lanes_and_evidence_are_closed_nonempty_and_ordered(self) -> None:
        cases = (
            ("owner_stage", "unknown"),
            ("required_lanes", []),
            ("required_lanes", ["checker", "frontend"]),
            ("required_lanes", ["frontend", "frontend"]),
            ("required_evidence", []),
            ("required_evidence", ["static-audit", "property"]),
            ("required_evidence", ["unknown"]),
        )
        for field, value in cases:
            fragments = valid_fragments()
            locate(fragments[0], "facets", "SCOPE-1")[field] = value
            with self.subTest(field=field, value=value):
                with self.assertRaises(semantics.SemanticCatalogError):
                    build(fragments)


class SourceBindingAndDiscoveryTests(unittest.TestCase):
    @staticmethod
    def write_repository(
        root: Path,
        *,
        source_index: bytes = SOURCE_INDEX_BYTES,
        fragments: list[dict[str, object]] | None = None,
    ) -> None:
        (root / "spec").mkdir()
        decomposition = root / "tests" / "spec-catalogs" / "v0.9" / "decomposition"
        decomposition.mkdir(parents=True)
        (root / "spec" / "kernel-spec-v0.9.md").write_bytes(SPECIFICATION)
        (root / "tests" / "spec-catalogs" / "v0.9" / "source.json").write_bytes(source_index)
        for index, fragment in enumerate(fragments or valid_fragments()):
            (decomposition / f"{index:02d}.json").write_bytes(
                semantics.canonical_bytes(fragment)
            )

    def test_source_index_requires_exact_bytes_not_an_object_substitution(self) -> None:
        with self.assertRaisesRegex(semantics.SemanticCatalogError, "exact bytes"):
            semantics.build_static_catalog(
                [semantics.canonical_bytes(valid_fragments()[0])],
                SPECIFICATION,
                SOURCE_INDEX,
            )

    def test_authority_lengths_are_checked_before_parsing(self) -> None:
        fragment = semantics.canonical_bytes(valid_fragments()[0])
        with mock.patch.object(
            semantics.semantics_io, "parse_canonical_json"
        ) as parser:
            with self.assertRaisesRegex(semantics.SemanticCatalogError, "byte length"):
                semantics.build_static_catalog(
                    [fragment], SPECIFICATION + b" ", SOURCE_INDEX_BYTES
                )
            parser.assert_not_called()
        with mock.patch.object(
            semantics.semantics_io, "parse_canonical_json"
        ) as parser:
            with self.assertRaisesRegex(semantics.SemanticCatalogError, "byte length"):
                semantics.build_static_catalog(
                    [fragment], SPECIFICATION, SOURCE_INDEX_BYTES + b" "
                )
            parser.assert_not_called()

    def test_structurally_plausible_source_index_substitution_fails_identity(self) -> None:
        substitute = copy.deepcopy(SOURCE_INDEX)
        substitute["scope"] += " Substituted."
        substitute_bytes = semantics.canonical_bytes(substitute)
        self.assertNotEqual(
            semantics.sha256(substitute_bytes),
            semantics.SOURCE_INDEX_SHA256,
        )
        with self.assertRaisesRegex(semantics.SemanticCatalogError, "source index"):
            semantics.build_static_catalog(
                [semantics.canonical_bytes(valid_fragments()[0])],
                SPECIFICATION,
                substitute_bytes,
            )

    def test_file_construction_rejects_source_index_substitution(self) -> None:
        substitute = copy.deepcopy(SOURCE_INDEX)
        substitute["scope"] += " Substituted."
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.write_repository(
                root, source_index=semantics.canonical_bytes(substitute)
            )
            with self.assertRaisesRegex(semantics.SemanticCatalogError, "source index"):
                semantics.build_from_files(root)

    def test_exact_specification_hash_is_required(self) -> None:
        changed = bytearray(SPECIFICATION)
        changed[0] ^= 1
        with self.assertRaisesRegex(semantics.SemanticCatalogError, "specification hash"):
            semantics.build_static_catalog(
                [semantics.canonical_bytes(valid_fragments()[0])],
                bytes(changed),
                SOURCE_INDEX_BYTES,
            )

    def test_fixed_inputs_and_fragment_paths_reject_symlinks(self) -> None:
        cases = ("root", "specification", "source-index", "directory", "fragment")
        for case in cases:
            with self.subTest(case=case), tempfile.TemporaryDirectory() as directory:
                base = Path(directory)
                real = base / "real"
                real.mkdir()
                self.write_repository(real)
                target = real
                if case == "root":
                    target = base / "root-link"
                    target.symlink_to(real, target_is_directory=True)
                elif case == "specification":
                    path = real / "spec" / "kernel-spec-v0.9.md"
                    path.unlink()
                    path.symlink_to(base / "outside-spec")
                    (base / "outside-spec").write_bytes(SPECIFICATION)
                elif case == "source-index":
                    path = real / "tests" / "spec-catalogs" / "v0.9" / "source.json"
                    path.unlink()
                    path.symlink_to(base / "outside-source")
                    (base / "outside-source").write_bytes(SOURCE_INDEX_BYTES)
                elif case == "directory":
                    path = real / "tests" / "spec-catalogs" / "v0.9" / "decomposition"
                    path.rename(base / "outside-decomposition")
                    path.symlink_to(base / "outside-decomposition", target_is_directory=True)
                else:
                    path = real / "tests" / "spec-catalogs" / "v0.9" / "decomposition" / "00.json"
                    path.unlink()
                    path.symlink_to(base / "outside-fragment")
                    (base / "outside-fragment").write_bytes(
                        semantics.canonical_bytes(valid_fragments()[0])
                    )
                with self.assertRaises(semantics.SemanticCatalogError):
                    semantics.build_from_files(target)

    def test_named_fifo_inputs_fail_in_subprocess_without_hanging(self) -> None:
        child_program = "\n".join(
            (
                "import pathlib, sys",
                "sys.path.insert(0, sys.argv[1])",
                "import semantics",
                "try:",
                "    semantics.build_from_files(pathlib.Path(sys.argv[2]))",
                "except semantics.SemanticCatalogError:",
                "    raise SystemExit(0)",
                "except BaseException as error:",
                "    print(type(error).__name__, error, file=sys.stderr)",
                "    raise SystemExit(3)",
                "raise SystemExit(2)",
            )
        )
        tools_directory = str(Path(__file__).resolve().parent)
        fifo_paths = (
            Path("spec/kernel-spec-v0.9.md"),
            Path("tests/spec-catalogs/v0.9/decomposition/00.json"),
        )
        for fifo_path in fifo_paths:
            with self.subTest(path=str(fifo_path)), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self.write_repository(root)
                path = root / fifo_path
                path.unlink()
                os.mkfifo(path)
                environment = dict(os.environ)
                environment["PYTHONDONTWRITEBYTECODE"] = "1"
                completed = subprocess.run(
                    [sys.executable, "-c", child_program, tools_directory, str(root)],
                    capture_output=True,
                    check=False,
                    env=environment,
                    text=True,
                    timeout=3,
                )
                self.assertEqual(
                    completed.returncode,
                    0,
                    msg=f"stdout={completed.stdout!r} stderr={completed.stderr!r}",
                )

    def test_directory_entry_and_json_fragment_caps_have_exact_boundaries(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.write_repository(root)
            decomposition = root / "tests" / "spec-catalogs" / "v0.9" / "decomposition"
            (decomposition / "README.md").write_text("ignored\n", encoding="ascii")
            (decomposition / "nested").mkdir()
            with mock.patch.object(
                semantics, "MAX_FRAGMENT_DIRECTORY_ENTRIES", 3
            ):
                self.assertEqual(
                    semantics.build_from_files(root), build(valid_fragments())
                )
                (decomposition / "ignored.bin").write_bytes(b"ignored\n")
                with self.assertRaisesRegex(
                    semantics.SemanticCatalogError, "entr"
                ):
                    semantics.build_from_files(root)

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fragments = [fragment_for(RULES[::2]), fragment_for(RULES[1::2])]
            self.write_repository(root, fragments=fragments)
            decomposition = root / "tests" / "spec-catalogs" / "v0.9" / "decomposition"
            with mock.patch.object(semantics, "MAX_FRAGMENT_COUNT", 2):
                self.assertEqual(
                    semantics.build_from_files(root), build(fragments)
                )
                (decomposition / "02.json").write_bytes(
                    semantics.canonical_bytes({})
                )
                with self.assertRaisesRegex(
                    semantics.SemanticCatalogError, "fragment count"
                ):
                    semantics.build_from_files(root)

    def test_public_file_build_and_partial_check_normalize_descriptor_oserrors(self) -> None:
        public_apis = (
            semantics.build_from_files,
            semantics.check_partial_from_files,
        )
        os_functions = ("open", "dup", "fstat", "read", "scandir")
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.write_repository(root)
            for public_api in public_apis:
                for function_name in os_functions:
                    with self.subTest(
                        api=public_api.__name__, os_function=function_name
                    ), mock.patch.object(
                        semantics.semantics_io.os,
                        function_name,
                        side_effect=OSError("injected descriptor failure"),
                    ):
                        with self.assertRaises(
                            semantics.SemanticCatalogError
                        ):
                            public_api(root)

    def test_discovery_is_direct_and_ignores_non_json_or_nested_files(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.write_repository(root)
            decomposition = root / "tests" / "spec-catalogs" / "v0.9" / "decomposition"
            (decomposition / "README.md").write_text("ignored\n", encoding="ascii")
            nested = decomposition / "nested"
            nested.mkdir()
            (nested / "hidden.json").write_bytes(semantics.canonical_bytes({}))
            self.assertEqual(semantics.build_from_files(root), build(valid_fragments()))


class LiveCatalogTests(unittest.TestCase):
    def test_live_partial_audit_confirms_the_complete_authored_partition(self) -> None:
        audit = semantics.check_partial_from_files()
        self.assertEqual(audit["rule_count"], len(RULES))
        self.assertEqual(audit["missing_rules"], [])
        self.assertEqual(audit["source_atom_count"], 204)
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            self.assertEqual(semantics.main(["check-partial"]), 0)
        self.assertEqual(
            output.getvalue(),
            f"{len(RULES)}/{len(RULES)} rules; 204/204 source atoms; missing: \n",
        )

    def test_live_full_build_is_complete_and_deterministic(self) -> None:
        catalog = semantics.build_from_files()
        self.assertEqual(len(catalog["source_atom_coverage"]), 204)
        self.assertEqual(
            catalog["specification"],
            {
                "path": semantics.SPEC_RELATIVE_PATH,
                "sha256": semantics.SPEC_SHA256,
                "version": semantics.SPEC_VERSION,
            },
        )
        self.assertEqual(catalog, semantics.build_from_files())
        self.assertEqual(
            semantics.canonical_bytes(catalog),
            semantics.canonical_bytes(semantics.build_from_files()),
        )


if __name__ == "__main__":
    unittest.main()
