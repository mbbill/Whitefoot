#!/usr/bin/env python3
"""Focused hostile tests for the exact-v0.9 discrepancy-sidecar contract."""

from __future__ import annotations

import copy
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import discrepancies as discrepancies
import discrepancy_inputs as authority
import discrepancy_predicates as predicates
import semantics
import semantics_io


EXPECTED_OPEN_IDS = (
    "discrepancy:v0.9/affine-deref-storage-lifecycle",
    "discrepancy:v0.9/diag3-retained-proof-ref",
    "discrepancy:v0.9/eff1-row-canonicality",
    "discrepancy:v0.9/eff2-local-region-effects",
    "discrepancy:v0.9/fn3-contract-member-semantics",
    "discrepancy:v0.9/fn7-main-return-spelling",
    "discrepancy:v0.9/op1-dotless-reservation",
)


def _source_atoms_by_owner(source_index: dict) -> dict[str, list[str]]:
    atoms = []
    for record in source_index["rules"]:
        atoms.append(
            (
                record["source"]["byte_start"],
                record["source"]["byte_end"],
                record["id"],
                record["rule_id"],
            )
        )
    for collection in semantics.CHILD_COLLECTIONS:
        for record in source_index[collection]:
            atoms.append(
                (
                    record["source"]["byte_start"],
                    record["source"]["byte_end"],
                    record["id"],
                    record["owner_rule"],
                )
            )
    atoms.sort(key=lambda item: (item[0], item[1], item[2].encode("ascii")))
    grouped: dict[str, list[str]] = {}
    for _, _, atom_id, owner in atoms:
        grouped.setdefault(owner, []).append(atom_id)
    return grouped


def make_test_catalog(
    facet_renames: dict[str, str] | None = None,
) -> bytes:
    """Build the live catalog, optionally renaming facets for hostile tests."""
    specification = authority.SPEC_PATH.read_bytes()
    source_index_bytes = authority.SOURCE_INDEX_PATH.read_bytes()
    source_index = semantics.parse_strict_json(
        source_index_bytes, "test source index"
    )
    assert isinstance(source_index, dict)
    live = copy.deepcopy(load_live_fragments())
    owned = {rule for fragment in live for rule in fragment["rules"]}
    atoms_by_owner = _source_atoms_by_owner(source_index)
    missing_rules = [
        record for record in source_index["rules"] if record["rule_id"] not in owned
    ]
    clauses = []
    facets = []
    for record in missing_rules:
        owner = record["rule_id"]
        facet_id = f"facet:{owner}/test-contract"
        facets.append(
            {
                "id": facet_id,
                "owner_rule": owner,
                "owner_stage": "semantic-check",
                "required_evidence": ["static-audit"],
                "required_lanes": ["frontend"],
                "source_atoms": atoms_by_owner[owner],
            }
        )
        source = record["source"]
        clauses.append(
            {
                "byte_end": source["byte_end"],
                "byte_start": source["byte_start"],
                "disposition": "facet",
                "facet_ids": [facet_id],
                "owner_rule": owner,
                "sha256": source["sha256"],
            }
        )
    if missing_rules:
        live.append(
            {
                "schema": 1,
                "kind": "whitefoot-semantic-decomposition-fragment",
                "rules": sorted(
                    (record["rule_id"] for record in missing_rules), key=str.encode
                ),
                "clauses": sorted(clauses, key=lambda item: item["byte_start"]),
                "facets": sorted(
                    facets, key=lambda item: item["id"].encode("ascii")
                ),
            }
        )

    if facet_renames:
        for fragment in live:
            for facet in fragment["facets"]:
                facet["id"] = facet_renames.get(facet["id"], facet["id"])
            fragment["facets"].sort(key=lambda item: item["id"].encode("ascii"))
            for clause in fragment["clauses"]:
                clause["facet_ids"] = sorted(
                    (facet_renames.get(item, item) for item in clause["facet_ids"]),
                    key=str.encode,
                )

    catalog = semantics.build_static_catalog(
        [semantics.canonical_bytes(fragment) for fragment in live],
        specification,
        source_index_bytes,
    )
    return semantics.canonical_bytes(catalog)


def load_live_fragments() -> list[dict]:
    """Read and parse the actual checked-in decomposition fragments."""
    records = semantics_io.read_fragment_directory(
        semantics.ROOT,
        ("tests", "spec-catalogs", "v0.9", "decomposition"),
        max_count=semantics.MAX_FRAGMENT_COUNT,
        max_file_bytes=semantics.MAX_FRAGMENT_BYTES,
        max_total_bytes=semantics.MAX_FRAGMENT_TOTAL_BYTES,
    )
    return [
        semantics.parse_fragment_bytes(raw, f"live fragment {name}")
        for name, raw in records
    ]


CATALOG = make_test_catalog()


class PinnedPredicateTests(unittest.TestCase):
    """Pin every exact audit without treating records as waivers."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.document = discrepancies.build_sidecar(CATALOG)
        cls.by_id = {record["id"]: record for record in cls.document["records"]}

    def test_document_binds_all_four_exact_artifacts(self) -> None:
        self.assertEqual(
            self.document["bindings"],
            {
                "catalog_sha256": discrepancies.sha256(CATALOG),
                "guard_baseline_sha256": discrepancies.GUARD_BASELINE_SHA256,
                "source_index_sha256": discrepancies.SOURCE_INDEX_SHA256,
                "specification_sha256": discrepancies.SPEC_SHA256,
            },
        )
        self.assertEqual(self.document["format"], discrepancies.FORMAT)

    def test_op1_set_predicate_pins_exact_evidence(self) -> None:
        evidence = self.by_id[
            "discrepancy:v0.9/op1-dotless-reservation"
        ]["evidence"]
        self.assertEqual(evidence["operation_row_count"], 44)
        self.assertEqual(evidence["operation_name_occurrence_count"], 84)
        self.assertEqual(evidence["table_distinct_dotless_count"], 51)
        self.assertEqual(evidence["listed_distinct_dotless_count"], 20)
        self.assertEqual(evidence["table_only_count"], 31)
        self.assertEqual(evidence["listed_only_count"], 0)

    def test_fn7_predicate_pins_grammar_rule_and_example_spans(self) -> None:
        record = self.by_id["discrepancy:v0.9/fn7-main-return-spelling"]
        evidence = record["evidence"]
        self.assertEqual(evidence["fn_decl_return_nonterminal"], "rtype")
        self.assertEqual(evidence["rtype_shape"], "mode type")
        self.assertEqual(evidence["fn7_main_return_spelling"], "unit")
        self.assertEqual(evidence["example_main_return_spelling"], "own unit")
        self.assertEqual(
            record["affected_facet_ids"],
            [
                "facet:EX-1/byte-exact-canonical-program",
                "facet:FN-7/main-return-spelling",
                "facet:GRAM-2/function-and-contract-declaration-shapes",
                "facet:GRAM-3/return-mode-type-shape",
            ],
        )

    def test_registered_gaps_block_only_the_reviewed_facets(self) -> None:
        expected = {
            "discrepancy:v0.9/affine-deref-storage-lifecycle": [
                "facet:STOR-3/deallocation-compiler-derived",
                "facet:STOR-3/drop-and-arena-release-artifact-operations",
            ],
            "discrepancy:v0.9/diag3-retained-proof-ref": [
                "facet:DIAG-3/check-report-schema",
            ],
            "discrepancy:v0.9/eff1-row-canonicality": [
                "facet:EFF-1/canonical-effect-order",
                "facet:EFF-1/effect-row-grammar",
            ],
            "discrepancy:v0.9/eff2-local-region-effects": [
                "facet:EFF-2/effect-row-bidirectional-exactness",
                "facet:EFF-2/syntactic-effect-exhibit-closure",
                "facet:EX-1/byte-exact-canonical-program",
                "facet:FN-7/main-effect-ceiling",
            ],
            "discrepancy:v0.9/fn3-contract-member-semantics": [
                "facet:FN-3/contract-member-checking-boundary",
                "facet:FN-5/behavior-parameterization",
                "facet:FN-5/env-struct-direct-calls",
            ],
            "discrepancy:v0.9/fn7-main-return-spelling": [
                "facet:EX-1/byte-exact-canonical-program",
                "facet:FN-7/main-return-spelling",
                "facet:GRAM-2/function-and-contract-declaration-shapes",
                "facet:GRAM-3/return-mode-type-shape",
            ],
            "discrepancy:v0.9/op1-dotless-reservation": [
                "facet:OP-1/dotless-operation-reservation",
            ],
        }
        self.assertEqual(set(self.by_id), set(expected))
        for identifier, facets in expected.items():
            with self.subTest(identifier=identifier):
                self.assertEqual(self.by_id[identifier]["affected_facet_ids"], facets)

    def test_protected_conflicts_pin_exact_case_expectations(self) -> None:
        expected = {
            "discrepancy:v0.9/eff1-row-canonicality": (
                "x-eff-dup-reads-effect",
                {"kind": "reject", "rule": "EFF-1"},
                "fc1260d3d3bf3ef5fa0e55207e02b15e8a588dee69bca39aea4b66e3ea6e5521",
            ),
            "discrepancy:v0.9/eff2-local-region-effects": (
                "stor4-pos-arena-confined",
                {"exit": 0, "kind": "run"},
                "e3a3a7c5930c82de534781bc2a988c467b34cbc421fdc8cf0ffbcabdf9fd1ba0",
            ),
        }
        for identifier, (case_id, expectation, digest) in expected.items():
            with self.subTest(identifier=identifier):
                case = self.by_id[identifier]["evidence"]["protected_case"]
                self.assertEqual(case["manifest"]["id"], case_id)
                self.assertEqual(case["manifest"]["expect"], expectation)
                self.assertEqual(case["sha256"], digest)

    def test_gap_evidence_is_exactly_anchored(self) -> None:
        lifecycle = self.by_id[
            "discrepancy:v0.9/affine-deref-storage-lifecycle"
        ]["evidence"]
        self.assertEqual(lifecycle["type7_affine_move_source"]["byte_start"], 33999)
        self.assertEqual(lifecycle["stor3_derived_drop_source"]["byte_end"], 45977)
        report = self.by_id[
            "discrepancy:v0.9/diag3-retained-proof-ref"
        ]["evidence"]
        self.assertEqual(report["report_header_source"]["byte_start"], 93668)
        self.assertEqual(report["check_report_row_source"]["byte_end"], 93994)

    def test_records_have_closed_non_authorizing_fields(self) -> None:
        self.assertEqual(tuple(sorted(self.by_id)), EXPECTED_OPEN_IDS)
        for record in self.document["records"]:
            self.assertEqual(
                set(record),
                {
                    "affected_facet_ids",
                    "class",
                    "evidence",
                    "id",
                    "predicate_id",
                    "resolution_authorities",
                },
            )


class ProtectedSurfaceTests(unittest.TestCase):
    """Ignore additions while preserving baseline-bound active evidence."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.inputs = authority.load_authorities()
        cls.base = predicates.observe_eff1_row_canonicality(
            cls.inputs.specification,
            cls.inputs.manifest,
            cls.inputs.case_sources,
            cls.inputs.protected_conformance,
        )

    def test_additive_manifest_case_does_not_expand_or_change_audit(self) -> None:
        manifest = self.inputs.manifest + (
            b'{"expect":{"exit":0,"kind":"run"},"id":"additive-eff1",'
            b'"rules":["EFF-1"],"status":"runnable"}\n'
        )
        observation = predicates.observe_eff1_row_canonicality(
            self.inputs.specification,
            manifest,
            self.inputs.case_sources,
            self.inputs.protected_conformance,
        )
        self.assertEqual(observation, self.base)

    def test_protected_case_mutation_fails_against_baseline(self) -> None:
        cases = dict(self.inputs.case_sources)
        path = "tests/conformance/cases/x-eff-dup-reads-effect.wf"
        cases[path] += b"\n"
        with self.assertRaisesRegex(
            predicates.DiscrepancyError, "protected conformance entry changed"
        ):
            predicates.observe_eff1_row_canonicality(
                self.inputs.specification,
                self.inputs.manifest,
                cases,
                self.inputs.protected_conformance,
            )


class CatalogAndRegistryTests(unittest.TestCase):
    """Reject catalog substitution and incomplete predicate registration."""

    def test_test_catalog_incorporates_live_owned_fragments(self) -> None:
        catalog = semantics.parse_strict_json(CATALOG, "catalog")
        catalog_ids = {facet["id"] for facet in catalog["facets"]}
        for fragment in load_live_fragments():
            self.assertTrue(
                {facet["id"] for facet in fragment["facets"]}.issubset(catalog_ids)
            )

    def test_checked_in_sidecar_equals_fresh_exact_recomputation(self) -> None:
        raw = authority.read_regular(
            authority.ROOT,
            discrepancies.SIDECAR_PATH,
            "checked-in discrepancy sidecar",
            authority.MAX_SIDECAR_BYTES,
        )
        self.assertEqual(raw, discrepancies.generated_sidecar_bytes())
        self.assertEqual(
            discrepancies.check_repository_sidecar().open_discrepancy_ids,
            EXPECTED_OPEN_IDS,
        )

    def test_catalog_validation_uses_exact_byte_builder_contract(self) -> None:
        original_build = semantics.build_static_catalog
        original_parse = semantics.parse_strict_json
        with mock.patch.object(
            semantics,
            "build_static_catalog",
            wraps=original_build,
        ) as build, mock.patch.object(
            semantics,
            "parse_strict_json",
            wraps=original_parse,
        ) as parse:
            discrepancies.build_sidecar(CATALOG)
        args, kwargs = build.call_args
        self.assertEqual(kwargs, {})
        self.assertTrue(all(type(fragment) is bytes for fragment in args[0]))
        self.assertEqual(args[1], authority.SPEC_PATH.read_bytes())
        self.assertEqual(args[2], authority.SOURCE_INDEX_PATH.read_bytes())
        self.assertIn(
            mock.call(
                CATALOG,
                "static catalog",
                max_bytes=authority.MAX_CATALOG_BYTES,
            ),
            parse.call_args_list,
        )

    def test_catalog_facet_ids_are_derived_from_same_exact_bytes(self) -> None:
        dummy = discrepancies.canonical_bytes(
            {"kind": "whitefoot-static-semantic-catalog", "test_only": True}
        )
        with self.assertRaisesRegex(discrepancies.DiscrepancyError, "static catalog"):
            discrepancies.build_sidecar(dummy)

        catalog_without_diag3 = make_test_catalog(
            {
                "facet:DIAG-3/check-report-schema":
                    "facet:DIAG-3/test-contract"
            }
        )
        with self.assertRaisesRegex(discrepancies.DiscrepancyError, "unknown facets"):
            discrepancies.build_sidecar(catalog_without_diag3)

        catalog_without_fn7 = make_test_catalog(
            {
                "facet:FN-7/main-return-spelling": (
                    "facet:FN-7/test-main-return"
                )
            }
        )
        with self.assertRaisesRegex(discrepancies.DiscrepancyError, "unknown facets"):
            discrepancies.build_sidecar(catalog_without_fn7)

    def test_catalog_hash_or_normalized_content_substitution_fails(self) -> None:
        value = semantics.parse_strict_json(CATALOG, "catalog")
        value["decomposition_sha256"] = "0" * 64
        with self.assertRaises(discrepancies.DiscrepancyError):
            discrepancies.build_sidecar(semantics.canonical_bytes(value))

    def test_registry_and_observation_ids_are_bijective(self) -> None:
        observations = {
            registration.identifier: predicates.Observation(False, {})
            for registration in predicates.REGISTRATIONS
        }
        predicates.validate_registry(observations)
        missing = dict(observations)
        missing.pop(next(iter(missing)))
        with self.assertRaisesRegex(predicates.DiscrepancyError, "differ"):
            predicates.validate_registry(missing)
        extra = dict(observations)
        extra["discrepancy:v0.9/unregistered"] = predicates.Observation(False, {})
        with self.assertRaisesRegex(predicates.DiscrepancyError, "differ"):
            predicates.validate_registry(extra)

    def test_duplicate_registration_fails(self) -> None:
        observations = {
            registration.identifier: predicates.Observation(False, {})
            for registration in predicates.REGISTRATIONS
        }
        with mock.patch.object(
            predicates,
            "REGISTRATIONS",
            predicates.REGISTRATIONS + (predicates.REGISTRATIONS[0],),
        ):
            with self.assertRaisesRegex(predicates.DiscrepancyError, "duplicate"):
                predicates.validate_registry(observations)


class StrictValidationTests(unittest.TestCase):
    """Reject forged schema, evidence, bindings, records, and release state."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.document = discrepancies.build_sidecar(CATALOG)

    def validate(self, document: object) -> discrepancies.AuditResult:
        return discrepancies.validate_sidecar(document, CATALOG)

    def test_exact_document_round_trips(self) -> None:
        raw = discrepancies.canonical_bytes(self.document)
        result = discrepancies.parse_and_validate_sidecar(raw, CATALOG)
        self.assertEqual(result.open_discrepancy_ids, EXPECTED_OPEN_IDS)

    def test_duplicate_json_keys_and_noncanonical_bytes_fail(self) -> None:
        with self.assertRaisesRegex(discrepancies.DiscrepancyError, "duplicate"):
            discrepancies.parse_canonical_sidecar(
                b'{"format":"first","format":"second"}\n'
            )
        with self.assertRaisesRegex(discrepancies.DiscrepancyError, "canonical"):
            discrepancies.parse_canonical_sidecar(
                b'{"bindings":{},"format":"x","records":[]}\n'
            )

    def test_unknown_fields_fail_at_every_schema_level(self) -> None:
        mutations = []
        top = copy.deepcopy(self.document)
        top["status"] = "open"
        mutations.append(top)
        binding = copy.deepcopy(self.document)
        binding["bindings"]["release_sha256"] = "0" * 64
        mutations.append(binding)
        record = copy.deepcopy(self.document)
        record["records"][0]["waiver"] = True
        mutations.append(record)
        evidence = copy.deepcopy(self.document)
        evidence["records"][0]["evidence"]["complete"] = True
        mutations.append(evidence)
        for mutation in mutations:
            with self.subTest(mutation=mutation):
                with self.assertRaises(discrepancies.DiscrepancyError):
                    self.validate(mutation)

    def test_missing_duplicate_reordered_and_unregistered_records_fail(self) -> None:
        missing = copy.deepcopy(self.document)
        missing["records"].pop()
        with self.assertRaises(discrepancies.DiscrepancyError):
            self.validate(missing)
        duplicate = copy.deepcopy(self.document)
        duplicate["records"].append(copy.deepcopy(duplicate["records"][-1]))
        with self.assertRaisesRegex(discrepancies.DiscrepancyError, "unique"):
            self.validate(duplicate)
        reordered = copy.deepcopy(self.document)
        reordered["records"].reverse()
        with self.assertRaisesRegex(discrepancies.DiscrepancyError, "sorted"):
            self.validate(reordered)
        unregistered = copy.deepcopy(self.document)
        unregistered["records"][0]["id"] = "discrepancy:v0.9/invented"
        with self.assertRaisesRegex(discrepancies.DiscrepancyError, "unregistered"):
            self.validate(unregistered)

    def test_release_recomputes_and_rejects_fabricated_state(self) -> None:
        raw = discrepancies.canonical_bytes(self.document)
        with self.assertRaisesRegex(discrepancies.DiscrepancyError, "release is blocked"):
            discrepancies.require_no_open_discrepancies(raw, CATALOG)
        fake = copy.deepcopy(self.document)
        fake["records"] = []
        with self.assertRaises(discrepancies.DiscrepancyError):
            discrepancies.require_no_open_discrepancies(
                discrepancies.canonical_bytes(fake), CATALOG
            )
        with self.assertRaisesRegex(discrepancies.DiscrepancyError, "immutable bytes"):
            discrepancies.require_no_open_discrepancies(
                discrepancies.AuditResult(()), CATALOG  # type: ignore[arg-type]
            )


class PathAndResourceTests(unittest.TestCase):
    """Fail closed on path indirection and bounded-input violations."""

    def test_protected_file_symlink_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cases = root / "tests" / "conformance" / "cases"
            cases.mkdir(parents=True)
            link = cases / "escape.wf"
            link.symlink_to(authority.SPEC_PATH)
            with self.assertRaisesRegex(authority.DiscrepancyError, "symlink"):
                authority.read_regular(root, link, "case", 1_000_000)

    def test_case_root_symlink_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "tests" / "conformance").mkdir(parents=True)
            case_root = root / "tests" / "conformance" / "cases"
            case_root.symlink_to(authority.CASE_ROOT, target_is_directory=True)
            with self.assertRaisesRegex(authority.DiscrepancyError, "symlink"):
                authority.require_directory(root, case_root, "case root")

    def test_fifo_file_is_rejected_without_blocking(self) -> None:
        child = "\n".join(
            (
                "import pathlib, sys",
                "sys.path.insert(0, sys.argv[1])",
                "import discrepancy_inputs as authority",
                "try:",
                "    authority.read_regular(pathlib.Path(sys.argv[2]), "
                "pathlib.Path(sys.argv[3]), 'fifo probe', 100)",
                "except authority.DiscrepancyError:",
                "    raise SystemExit(0)",
                "except BaseException as error:",
                "    print(type(error).__name__, error, file=sys.stderr)",
                "    raise SystemExit(3)",
                "raise SystemExit(2)",
            )
        )
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fifo = root / "authority.json"
            os.mkfifo(fifo)
            completed = subprocess.run(
                [
                    sys.executable,
                    "-c",
                    child,
                    str(Path(__file__).resolve().parent),
                    str(root),
                    str(fifo),
                ],
                capture_output=True,
                check=False,
                text=True,
                timeout=3,
            )
            self.assertEqual(
                completed.returncode,
                0,
                msg=f"stdout={completed.stdout!r} stderr={completed.stderr!r}",
            )

    def test_descriptor_oserror_is_normalized(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            nested = root / "nested"
            nested.mkdir()
            path = nested / "case.wf"
            path.write_bytes(b"source\n")
            original_open = authority.os.open
            original_fstat = authority.os.fstat
            for failure_ordinal in (1, 2, 3, 4):
                opened: list[int] = []
                fstat_calls = 0

                def recording_open(*args, **kwargs):
                    descriptor = original_open(*args, **kwargs)
                    opened.append(descriptor)
                    return descriptor

                def failing_fstat(descriptor):
                    nonlocal fstat_calls
                    fstat_calls += 1
                    if fstat_calls == failure_ordinal:
                        raise OSError("injected failure")
                    return original_fstat(descriptor)

                with self.subTest(failure_ordinal=failure_ordinal), mock.patch.object(
                    authority.os, "open", side_effect=recording_open
                ), mock.patch.object(
                    authority.os, "fstat", side_effect=failing_fstat
                ):
                    with self.assertRaises(authority.DiscrepancyError):
                        authority.read_regular(root, path, "case", 100)

                for descriptor in set(opened):
                    with self.assertRaises(OSError):
                        original_fstat(descriptor)

    def test_ancestor_swap_cannot_redirect_an_openat_read(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            safe = root / "safe"
            moved = root / "opened-safe"
            outside = root / "substitute"
            safe.mkdir()
            outside.mkdir()
            (safe / "case.wf").write_bytes(b"opened-directory\n")
            (outside / "case.wf").write_bytes(b"substituted-directory\n")
            original = authority._open_directory_at

            def swap_after_open(parent: int, component: str, label: str) -> int:
                descriptor = original(parent, component, label)
                if component == "safe":
                    safe.rename(moved)
                    safe.symlink_to(outside, target_is_directory=True)
                return descriptor

            with mock.patch.object(
                authority,
                "_open_directory_at",
                side_effect=swap_after_open,
            ):
                raw = authority.read_regular(
                    root,
                    root / "safe" / "case.wf",
                    "case",
                    1_000,
                )
            self.assertEqual(raw, b"opened-directory\n")
            self.assertEqual(
                (root / "safe" / "case.wf").read_bytes(),
                b"substituted-directory\n",
            )

    def test_json_depth_and_size_fail_with_controlled_errors(self) -> None:
        deep = b"[" * (authority.MAX_JSON_DEPTH + 1) + b"0" + b"]" * (
            authority.MAX_JSON_DEPTH + 1
        )
        with self.assertRaisesRegex(authority.DiscrepancyError, "nesting"):
            discrepancies.parse_canonical_sidecar(deep)
        oversized = b" " * (authority.MAX_SIDECAR_BYTES + 1)
        with self.assertRaisesRegex(authority.DiscrepancyError, "exceeding"):
            discrepancies.parse_canonical_sidecar(oversized)

    def test_json_integer_digit_limit_accepts_exact_and_rejects_plus_one(self) -> None:
        exact = b"-" + (b"9" * authority.MAX_JSON_INTEGER_DIGITS)
        self.assertEqual(
            authority.strict_json_loads(
                exact,
                max_bytes=100,
                label="integer probe",
            ),
            -int(b"9" * authority.MAX_JSON_INTEGER_DIGITS),
        )
        with self.assertRaisesRegex(
            authority.DiscrepancyError,
            rf"integer.*{authority.MAX_JSON_INTEGER_DIGITS}-digit",
        ):
            authority.strict_json_loads(
                b"9" * (authority.MAX_JSON_INTEGER_DIGITS + 1),
                max_bytes=100,
                label="integer probe",
            )

    def test_json_string_byte_limit_accepts_exact_and_rejects_plus_one(self) -> None:
        self.assertEqual(
            authority.strict_json_loads(
                b'"\\u00e9"',
                max_bytes=100,
                label="string probe",
                max_string_bytes=2,
            ),
            "\N{LATIN SMALL LETTER E WITH ACUTE}",
        )
        with self.assertRaisesRegex(authority.DiscrepancyError, "string over 1 bytes"):
            authority.strict_json_loads(
                b'"\\u00e9"',
                max_bytes=100,
                label="string probe",
                max_string_bytes=1,
            )

    def test_json_list_item_limit_accepts_exact_and_rejects_plus_one(self) -> None:
        self.assertEqual(
            authority.strict_json_loads(
                b"[0,1]",
                max_bytes=100,
                label="list probe",
                max_list_items=2,
            ),
            [0, 1],
        )
        with self.assertRaisesRegex(authority.DiscrepancyError, "list over 2 items"):
            authority.strict_json_loads(
                b"[0,1,2]",
                max_bytes=100,
                label="list probe",
                max_list_items=2,
            )

    def test_json_object_field_limit_accepts_exact_and_rejects_plus_one(self) -> None:
        self.assertEqual(
            authority.strict_json_loads(
                b'{"a":0,"b":1}',
                max_bytes=100,
                label="object probe",
                max_object_fields=2,
            ),
            {"a": 0, "b": 1},
        )
        with self.assertRaisesRegex(authority.DiscrepancyError, "object.*2-field"):
            authority.strict_json_loads(
                b'{"a":0,"b":1,"c":2}',
                max_bytes=100,
                label="object probe",
                max_object_fields=2,
            )

    def test_json_node_limit_accepts_exact_and_rejects_plus_one(self) -> None:
        self.assertEqual(
            authority.strict_json_loads(
                b"[0,1]",
                max_bytes=100,
                label="node probe",
                max_nodes=3,
            ),
            [0, 1],
        )
        with self.assertRaisesRegex(authority.DiscrepancyError, "2-node"):
            authority.strict_json_loads(
                b"[0,1]",
                max_bytes=100,
                label="node probe",
                max_nodes=2,
            )

    def test_file_and_protected_case_count_limits(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "one.wf"
            path.write_bytes(b"xx")
            with self.assertRaisesRegex(authority.DiscrepancyError, "exceeding"):
                authority.read_regular(root, path, "case", 1)
        with mock.patch.object(authority, "MAX_PROTECTED_CASE_FILES", 1):
            with self.assertRaisesRegex(authority.DiscrepancyError, "count"):
                authority._protected_case_ids({"one": "0" * 64, "two": "1" * 64})


if __name__ == "__main__":
    unittest.main()
