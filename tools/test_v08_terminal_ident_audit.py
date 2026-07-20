#!/usr/bin/env python3
"""Hostile tests for the exact-v0.8 terminal-versus-IDENT gap."""

from __future__ import annotations

import copy
import unittest

import facet_discrepancies as discrepancies
import facet_discrepancy_inputs as authority
import facet_discrepancy_predicates as predicates
import semantic_catalog


CATALOG = semantic_catalog.canonical_bytes(semantic_catalog.build_from_files())


class TerminalIdentGapTests(unittest.TestCase):
    """Pin the gap without choosing terminal priority or keyword semantics."""

    @classmethod
    def setUpClass(cls) -> None:
        document = discrepancies.build_sidecar(CATALOG)
        cls.by_id = {record["id"]: record for record in document["records"]}

    def test_terminal_ident_partition_gap_has_exact_census_and_impact(self) -> None:
        record = self.by_id[
            "discrepancy:v0.8/gram-terminal-ident-partition"
        ]
        self.assertEqual(record["class"], "internal-specification-gap")
        self.assertEqual(
            record["resolution_authorities"],
            ["successor-numbered-specification"],
        )
        self.assertEqual(
            record["affected_facet_ids"],
            [
                "facet:FORM-3/ident-lexical-class",
                "facet:FORM-6/unit-grammar-positions-disjoint",
                "facet:FORM-6/unit-lowercase-keyword",
                "facet:FORM-6/unit-production-local-resolution",
                "facet:FORM-6/unit-token-type-position",
                "facet:FORM-6/unit-token-value-position",
                "facet:GRAM-1/deterministic-single-parse",
                "facet:GRAM-1/two-token-overlap-resolution",
                "facet:GRAM-3/constant-value-tree-shape",
                "facet:GRAM-3/type-argument-shapes",
                "facet:GRAM-5/atom-form-closed-set",
                "facet:GRAM-5/call-and-callee-shapes",
                "facet:GRAM-5/expression-form-closed-set",
                "facet:GRAM-5/place-chain-shapes",
                "facet:GRAM-6/index-place-only",
                "facet:META-2/context-independent-spellings",
            ],
        )
        evidence = record["evidence"]
        self.assertEqual(evidence["syntax_production_count"], 59)
        self.assertEqual(evidence["fixed_terminal_occurrence_count"], 51)
        self.assertEqual(evidence["fixed_terminal_count"], 47)
        self.assertEqual(evidence["ident_matching_terminal_count"], 47)
        self.assertEqual(
            evidence["fixed_terminal_inventory_sha256"],
            "5e437fbb7371fbaf00be9f341264c6414de6a23ca13877fe28f43a1644d9e376",
        )
        self.assertEqual(evidence["production_terminal_map_row_count"], 27)
        self.assertEqual(
            evidence["production_terminal_map_sha256"],
            "35150331a150a188564f4073898aa35c53bd51df77f9b55222b8a68b38526145",
        )
        self.assertEqual(
            evidence["ambiguous_spellings"],
            [
                "deref",
                "f32",
                "f64",
                "i16",
                "i32",
                "i64",
                "i8",
                "index",
                "u16",
                "u32",
                "u64",
                "u8",
                "unit",
            ],
        )
        self.assertEqual(
            evidence["type_argument_ambiguous_spellings"],
            [
                "f32",
                "f64",
                "i16",
                "i32",
                "i64",
                "i8",
                "u16",
                "u32",
                "u64",
                "u8",
                "unit",
            ],
        )
        self.assertEqual(
            evidence["place_call_ambiguous_spellings"],
            ["deref", "index"],
        )
        self.assertEqual(
            [item["context"] for item in evidence["competing_derivations"]],
            [
                "type-argument-primitive-or-unit",
                "constant-value-unit",
                "expression-unit",
                "expression-deref",
                "expression-index",
            ],
        )
        self.assertEqual(
            evidence["competing_derivations"][0]["spellings"],
            evidence["type_argument_ambiguous_spellings"],
        )
        self.assertEqual(
            evidence["derivation_count_witnesses"],
            [
                {
                    "call_derivations": 1,
                    "place_derivations": 1,
                    "spelling": "deref(p)",
                    "total_derivations": 2,
                },
                {
                    "call_derivations": 1,
                    "place_derivations": 1,
                    "spelling": "index<T>(p, q)",
                    "total_derivations": 2,
                },
                {
                    "call_derivations": 2,
                    "place_derivations": 1,
                    "spelling": "deref(unit)",
                    "total_derivations": 3,
                },
                {
                    "call_derivations": 2,
                    "place_derivations": 1,
                    "spelling": "index<i32>(p, q)",
                    "total_derivations": 3,
                },
                {
                    "call_derivations": 4,
                    "place_derivations": 2,
                    "spelling": "index<unit>(p, unit)",
                    "total_derivations": 6,
                },
            ],
        )
        self.assertEqual(
            evidence["single_derivation_boundaries"],
            [
                {
                    "call_derivations": 0,
                    "place_derivations": 1,
                    "spelling": "deref(p).field",
                    "total_derivations": 1,
                },
                {
                    "call_derivations": 1,
                    "place_derivations": 0,
                    "spelling": "deref<T>(p)",
                    "total_derivations": 1,
                },
                {
                    "call_derivations": 0,
                    "place_derivations": 1,
                    "spelling": "index<T>(p, q).field",
                    "total_derivations": 1,
                },
            ],
        )
        self.assertEqual(
            set(evidence["production_sources"]),
            {
                "production:GRAM-3:type",
                "production:GRAM-3:targs",
                "production:GRAM-3:targ",
                "production:GRAM-3:const",
                "production:GRAM-3:cvalue",
                "production:GRAM-5:expr",
                "production:GRAM-5:atom",
                "production:GRAM-5:call",
                "production:GRAM-5:callee",
                "production:GRAM-5:construct",
                "production:GRAM-5:fieldinit_list",
                "production:GRAM-5:fieldinit",
                "production:GRAM-5:borrow_expr",
                "production:GRAM-5:atom_list",
                "production:GRAM-5:place",
                "production:GRAM-5:pbase",
                "production:GRAM-5:psuffix",
            },
        )
        self.assertEqual(
            evidence["production_sources"]["production:GRAM-3:type"]["sha256"],
            "eb8e907bbb7270682ca8dfda756933d356fd7f17d333c49dc9099725fb60ad58",
        )
        self.assertEqual(
            evidence["production_sources"]["production:GRAM-3:const"]["sha256"],
            "7f10580fec0ea9c8c10ef9ec932daf33c49c99b04078c368c2b5cb40463e5e8f",
        )
        self.assertEqual(
            evidence["production_sources"]["production:GRAM-5:atom_list"][
                "sha256"
            ],
            "f905b6c93d16eade935b53a25d28c7b345766f7b89a4c9ef92fc6c906445f01c",
        )
        self.assertEqual(
            evidence["production_sources"]["production:GRAM-5:psuffix"][
                "sha256"
            ],
            "f49077b55c1dceac8a465bec2c1e926ff5eaf92d18c6fd0e874605f3c175ce32",
        )
        self.assertEqual(evidence["form5_unit_literal_source"]["byte_start"], 9389)
        self.assertEqual(evidence["form5_unit_literal_source"]["byte_end"], 9397)
        self.assertEqual(evidence["form3_typeid_source"]["byte_start"], 8323)
        self.assertEqual(evidence["form3_typeid_source"]["byte_end"], 8349)
        self.assertEqual(evidence["form3_regionid_source"]["byte_start"], 8351)
        self.assertEqual(evidence["form3_regionid_source"]["byte_end"], 8378)
        self.assertEqual(evidence["form3_opname_source"]["byte_start"], 8454)
        self.assertEqual(evidence["form3_opname_source"]["byte_end"], 8510)
        self.assertEqual(evidence["form3_rule_source"]["byte_start"], 8272)
        self.assertEqual(evidence["form3_rule_source"]["byte_end"], 8689)
        self.assertEqual(evidence["form5_rule_source"]["byte_start"], 8821)
        self.assertEqual(evidence["form5_rule_source"]["byte_end"], 10195)
        self.assertEqual(
            evidence["missing_contract"],
            "fixed-terminal-versus-ident-priority-or-exclusion",
        )

    def test_terminal_ident_impact_covers_exact_conflict_clauses_only(self) -> None:
        catalog = semantic_catalog.parse_strict_json(CATALOG, "test catalog")
        conflict_spans = {
            (8272, 8323),
            (10196, 10259),
            (10259, 10302),
            (10302, 10350),
            (10350, 10401),
            (10401, 10520),
            (12131, 12207),
            (12207, 12268),
            (13966, 14001),
            (14001, 14035),
            (14132, 14233),
            (15132, 15174),
            (15174, 15237),
            (15237, 15309),
            (15309, 15342),
            (15573, 15606),
            (15606, 15715),
            (16000, 16036),
            (62677, 62816),
        }
        clauses = {
            (item["byte_start"], item["byte_end"]): item
            for item in catalog["clauses"]
        }
        self.assertTrue(conflict_spans <= set(clauses))
        mapped = {
            facet
            for span in conflict_spans
            for facet in clauses[span]["facet_ids"]
        }
        reviewed_exclusions = {
            "facet:GRAM-3/constant-values-declaration-only"
        }
        self.assertEqual(
            mapped - reviewed_exclusions,
            set(
                self.by_id[
                    "discrepancy:v0.8/gram-terminal-ident-partition"
                ]["affected_facet_ids"]
            ),
        )
        self.assertEqual(
            mapped
            - set(
                self.by_id[
                    "discrepancy:v0.8/gram-terminal-ident-partition"
                ]["affected_facet_ids"]
            ),
            reviewed_exclusions,
        )

    def test_terminal_ident_predicate_cannot_self_authorize_a_priority(self) -> None:
        authorities = authority.load_authorities()
        appended = authorities.specification + (
            b"\nNon-normative test mutation: fixed terminals precede IDENT.\n"
        )
        observation = predicates.observe_gram_terminal_ident_partition(
            appended,
            authorities.source_index,
            enforce_pins=False,
        )
        self.assertTrue(observation.is_open)
        mutated = authorities.specification.replace(
            b"IDENT `[a-z][a-z0-9_]*`",
            b"IDENT `[a-z][a-z0-9_]+`",
            1,
        )
        with self.assertRaisesRegex(
            predicates.DiscrepancyError,
            "source is stale|exact fragment",
        ):
            predicates.observe_gram_terminal_ident_partition(
                mutated,
                authorities.source_index,
                enforce_pins=False,
            )

    def test_terminal_ident_predicate_binds_every_derivation_premise(self) -> None:
        authorities = authority.load_authorities()

        def mutate_indexed_source(
            collection: str,
            identifier: str,
            old: bytes,
            new: bytes,
        ) -> tuple[bytes, dict]:
            self.assertEqual(len(old), len(new))
            specification = authorities.specification.replace(old, new, 1)
            self.assertNotEqual(specification, authorities.specification)
            source_index = copy.deepcopy(authorities.source_index)
            records = {item["id"]: item for item in source_index[collection]}
            record = records[identifier]
            span = record["source"]
            exact = predicates.ExactSource.from_bytes(
                "spec/kernel-spec-v0.8.md",
                specification,
            ).evidence(span["byte_start"], span["byte_end"])
            record["source"] = {
                field: exact[field]
                for field in (
                    "byte_end",
                    "byte_start",
                    "line_end",
                    "line_start",
                    "sha256",
                )
            }
            return specification, source_index

        mutations = (
            (
                "type-ident-alternative",
                "syntax_productions",
                "production:GRAM-3:type",
                b"| TYPEID targs?",
                b"| IDENT |TYPEID",
            ),
            (
                "ident-headed-construct",
                "syntax_productions",
                "production:GRAM-5:construct",
                b"construct      := TYPEID",
                b"construct      := IDENT ",
            ),
            (
                "ident-borrow-expression",
                "syntax_productions",
                "production:GRAM-5:borrow_expr",
                b'borrow_expr    := "&" REGIONID place',
                b"borrow_expr    := IDENT             ",
            ),
            (
                "lowercase-region-id",
                "rules",
                "rule:FORM-3",
                b"REGIONID `'[a-z][a-z0-9_]*`",
                b"REGIONID `[a-z][a-z0-9_]*` ",
            ),
            (
                "identifier-shaped-literal",
                "rules",
                "rule:FORM-5",
                b"`unit`; STRING",
                b"`unit`; IDENT ",
            ),
        )
        for label, collection, identifier, old, new in mutations:
            with self.subTest(label=label):
                specification, source_index = mutate_indexed_source(
                    collection,
                    identifier,
                    old,
                    new,
                )
                with self.assertRaises(predicates.DiscrepancyError):
                    predicates.observe_gram_terminal_ident_partition(
                        specification,
                        source_index,
                    )


if __name__ == "__main__":
    unittest.main()
