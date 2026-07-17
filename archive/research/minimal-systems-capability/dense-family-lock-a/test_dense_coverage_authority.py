#!/usr/bin/env python3
"""Hostile mutation tests for the dense coverage authorities."""

from __future__ import annotations

import copy
import hashlib
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock


HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import dense_coverage_authority as authority


class DenseCoverageAuthorityTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.context, cls.outputs = authority.build_authorities()

    def copied_outputs(self) -> dict[str, list[dict[str, object]]]:
        return copy.deepcopy(self.outputs)

    def assert_public_rejects(
        self, outputs: dict[str, list[dict[str, object]]]
    ) -> None:
        # Reuse the already loaded immutable snapshot so the mutation matrix
        # measures authority validation rather than repeated git process cost.
        with mock.patch.object(
            authority, "load_context", return_value=self.context
        ):
            with self.assertRaises(ValueError):
                authority.validate_authorities(self.context, outputs)

    def test_unmodified_authorities_validate(self) -> None:
        authority.validate_authorities(self.context, self.outputs)

    def test_closed_registry_is_exact_literal_data(self) -> None:
        loaded = authority.load_closed_registry()
        self.assertEqual(
            set(loaded),
            {
                "SCHEMA_VERSION",
                "CLUSTER_MEMBERS",
                "EXCLUDED_MEMBERS",
                "PROTOCOL_SYNTHETIC_MEMBERS",
                "DIRECT_ROUTE_CLASSES",
                "DIRECT_EVIDENCE_ASSIGNMENTS",
                "SELECTOR_CHILD_ASSIGNMENTS",
            },
        )

    def test_closed_registry_byte_mutation_is_rejected(self) -> None:
        data = bytearray(authority.CLOSED_REGISTRY_PATH.read_bytes())
        data[-2] = ord(" ") if data[-2] != ord(" ") else ord("#")
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "mutated_registry.py"
            path.write_bytes(data)
            with self.assertRaisesRegex(ValueError, "digest mismatch"):
                authority.load_closed_registry(path)

    def test_shared_literal_loader_byte_mutation_is_rejected(self) -> None:
        data = bytearray(authority.CLOSED_LITERAL_LOADER_PATH.read_bytes())
        data[-2] = ord(" ") if data[-2] != ord(" ") else ord("#")
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "mutated_literal_loader.py"
            path.write_bytes(data)
            with self.assertRaisesRegex(ValueError, "loader digest mismatch"):
                authority.load_shared_literal_loader(path)

    def test_hash_approved_executable_registry_mutation_is_rejected(self) -> None:
        original = authority.CLOSED_REGISTRY_PATH.read_bytes()
        mutated = original.replace(
            b'SCHEMA_VERSION = "dense-coverage-closed-registry-v2"',
            b'SCHEMA_VERSION = str("dense-coverage-closed-registry-v2")',
            1,
        )
        self.assertNotEqual(mutated, original)
        digest = hashlib.sha256(mutated).hexdigest()
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "executable_registry.py"
            path.write_bytes(mutated)
            with mock.patch.object(authority, "CLOSED_REGISTRY_SHA256", digest):
                with self.assertRaisesRegex(ValueError, "not literal data"):
                    authority.load_closed_registry(path)

    def test_direct_identity_authority_is_exact_and_closed(self) -> None:
        direct_rows = [
            row
            for row in self.outputs["DENSE-EVIDENCE-TARGET-AUTHORITY.tsv"]
            if row["subject_kind"] == "DIRECT_EVIDENCE"
        ]
        identities = {str(row["subject_identity"]) for row in direct_rows}
        self.assertEqual(identities, set(authority.DIRECT_EVIDENCE_ASSIGNMENTS))
        self.assertEqual(len(identities), 456)
        self.assertTrue(
            all(
                str(row["target_authority"]).startswith(
                    "EXACT_CLOSED_DIRECT_TARGET_ASSIGNMENT:"
                )
                for row in direct_rows
            )
        )

    def test_direct_identity_omission_from_closed_registry_is_rejected(self) -> None:
        assignments = dict(authority.DIRECT_EVIDENCE_ASSIGNMENTS)
        assignments.pop(next(iter(assignments)))
        with mock.patch.object(
            authority, "DIRECT_EVIDENCE_ASSIGNMENTS", assignments
        ):
            with self.assertRaisesRegex(ValueError, "456-identity universe"):
                authority.load_context(self.context.snapshot)

    def test_coherent_selector_target_reroute_is_rejected(self) -> None:
        outputs = self.copied_outputs()
        expansion = outputs["DENSE-SELECTOR-EXPANSION-AUTHORITY.tsv"]
        row = next(
            candidate
            for candidate in expansion
            if candidate["applicable_target_ids"] == authority.FAMILY_ID
            and candidate["f_dense_member_contract_ids"] != "NONE"
        )
        subject = str(row["child_identity"])
        row["applicable_target_ids"] = "F-SPARSE"
        row["f_dense_member_contract_ids"] = "NONE"
        target_rows = outputs["DENSE-EVIDENCE-TARGET-AUTHORITY.tsv"]
        target = next(
            candidate
            for candidate in target_rows
            if candidate["subject_identity"] == subject
            and candidate["target_id"] == authority.FAMILY_ID
        )
        target["target_id"] = "F-SPARSE"
        target["terminal_disposition"] = "EXCLUDED_BLOCKS_CLAIM"
        target["member_contract_ids"] = "NONE"
        target["blocked_claims"] = "F-SPARSE;COMPLETE-SYSTEMS-FLOOR"
        outputs["DENSE-EVIDENCE-MEMBER-AUTHORITY.tsv"] = [
            candidate
            for candidate in outputs["DENSE-EVIDENCE-MEMBER-AUTHORITY.tsv"]
            if candidate["subject_identity"] != subject
        ]
        with self.assertRaisesRegex(ValueError, "closed child registry"):
            authority.validate_selector_authority(self.context, expansion)
        self.assert_public_rejects(outputs)

    def test_selector_child_member_narrowing_is_rejected(self) -> None:
        outputs = self.copied_outputs()
        expansion = outputs["DENSE-SELECTOR-EXPANSION-AUTHORITY.tsv"]
        row = next(
            candidate
            for candidate in expansion
            if len(authority.csv_ids(str(candidate["f_dense_member_contract_ids"]))) > 1
            and candidate["applicable_target_ids"] == authority.FAMILY_ID
        )
        subject = str(row["child_identity"])
        members = authority.csv_ids(str(row["f_dense_member_contract_ids"]))
        removed = members[-1]
        retained = members[:-1]
        row["f_dense_member_contract_ids"] = ",".join(retained)
        target = next(
            candidate
            for candidate in outputs["DENSE-EVIDENCE-TARGET-AUTHORITY.tsv"]
            if candidate["subject_identity"] == subject
            and candidate["target_id"] == authority.FAMILY_ID
        )
        target["member_contract_ids"] = ",".join(retained)
        outputs["DENSE-EVIDENCE-MEMBER-AUTHORITY.tsv"] = [
            candidate
            for candidate in outputs["DENSE-EVIDENCE-MEMBER-AUTHORITY.tsv"]
            if not (
                candidate["subject_identity"] == subject
                and candidate["member_contract_id"] == removed
            )
        ]
        with self.assertRaisesRegex(ValueError, "closed child registry"):
            authority.validate_selector_authority(self.context, expansion)
        self.assert_public_rejects(outputs)

    def test_selector_child_member_substitution_is_rejected(self) -> None:
        outputs = self.copied_outputs()
        expansion = outputs["DENSE-SELECTOR-EXPANSION-AUTHORITY.tsv"]
        selected: tuple[dict[str, object], str] | None = None
        for row in expansion:
            members = authority.csv_ids(str(row["f_dense_member_contract_ids"]))
            if len(members) != 1 or row["applicable_target_ids"] != authority.FAMILY_ID:
                continue
            alternatives = [
                member
                for member in authority.CLUSTER_MEMBERS[str(row["cluster_id"])]
                if member not in members
                and (member in authority.EXCLUDED_MEMBERS)
                == (members[0] in authority.EXCLUDED_MEMBERS)
            ]
            if alternatives:
                selected = (row, alternatives[0])
                break
        self.assertIsNotNone(selected)
        row, replacement = selected  # type: ignore[misc]
        subject = str(row["child_identity"])
        original = str(row["f_dense_member_contract_ids"])
        row["f_dense_member_contract_ids"] = replacement
        target = next(
            candidate
            for candidate in outputs["DENSE-EVIDENCE-TARGET-AUTHORITY.tsv"]
            if candidate["subject_identity"] == subject
            and candidate["target_id"] == authority.FAMILY_ID
        )
        target["member_contract_ids"] = replacement
        member = next(
            candidate
            for candidate in outputs["DENSE-EVIDENCE-MEMBER-AUTHORITY.tsv"]
            if candidate["subject_identity"] == subject
            and candidate["member_contract_id"] == original
        )
        member["member_contract_id"] = replacement
        with self.assertRaisesRegex(ValueError, "closed child registry"):
            authority.validate_selector_authority(self.context, expansion)
        self.assert_public_rejects(outputs)

    def test_coherent_direct_target_reroute_is_independently_rejected(self) -> None:
        outputs = self.copied_outputs()
        target_rows = outputs["DENSE-EVIDENCE-TARGET-AUTHORITY.tsv"]
        target = next(
            row
            for row in target_rows
            if row["subject_kind"] == "DIRECT_EVIDENCE"
            and row["target_id"] == authority.FAMILY_ID
            and row["member_contract_ids"] != "NONE"
            and sum(
                candidate["subject_identity"] == row["subject_identity"]
                for candidate in target_rows
            )
            == 1
        )
        subject = str(target["subject_identity"])
        target["target_id"] = "F-SPARSE"
        target["terminal_disposition"] = "EXCLUDED_BLOCKS_CLAIM"
        target["member_contract_ids"] = "NONE"
        target["blocked_claims"] = "F-SPARSE;COMPLETE-SYSTEMS-FLOOR"
        member_rows = [
            row
            for row in outputs["DENSE-EVIDENCE-MEMBER-AUTHORITY.tsv"]
            if row["subject_identity"] != subject
        ]
        outputs["DENSE-EVIDENCE-MEMBER-AUTHORITY.tsv"] = member_rows
        with self.assertRaisesRegex(ValueError, "closed identity registry"):
            authority.validate_target_member_authority(
                self.context,
                outputs["DENSE-SELECTOR-EXPANSION-AUTHORITY.tsv"],
                target_rows,
                member_rows,
            )
        self.assert_public_rejects(outputs)

    def test_coherent_direct_member_substitution_is_independently_rejected(self) -> None:
        outputs = self.copied_outputs()
        target_rows = outputs["DENSE-EVIDENCE-TARGET-AUTHORITY.tsv"]
        selected: tuple[dict[str, object], str, str] | None = None
        for row in target_rows:
            if (
                row["subject_kind"] != "DIRECT_EVIDENCE"
                or row["target_id"] != authority.FAMILY_ID
            ):
                continue
            members = authority.csv_ids(str(row["member_contract_ids"]))
            if len(members) != 1:
                continue
            alternatives = [
                member
                for member in authority.CLUSTER_MEMBERS[str(row["cluster_id"])]
                if member not in members
                and (member in authority.EXCLUDED_MEMBERS)
                == (members[0] in authority.EXCLUDED_MEMBERS)
            ]
            if alternatives:
                selected = (row, members[0], alternatives[0])
                break
        self.assertIsNotNone(selected)
        target, original, replacement = selected  # type: ignore[misc]
        subject = str(target["subject_identity"])
        target["member_contract_ids"] = replacement
        member_rows = outputs["DENSE-EVIDENCE-MEMBER-AUTHORITY.tsv"]
        member = next(
            row
            for row in member_rows
            if row["subject_identity"] == subject
            and row["member_contract_id"] == original
        )
        member["member_contract_id"] = replacement
        with self.assertRaisesRegex(ValueError, "closed identity registry"):
            authority.validate_target_member_authority(
                self.context,
                outputs["DENSE-SELECTOR-EXPANSION-AUTHORITY.tsv"],
                target_rows,
                member_rows,
            )
        self.assert_public_rejects(outputs)

    def test_role_removal_and_substitution_are_rejected(self) -> None:
        for mutation in ("remove", "substitute"):
            with self.subTest(mutation=mutation):
                outputs = self.copied_outputs()
                roles = outputs["DENSE-ROLE-UNIT-AUTHORITY.tsv"]
                index = next(
                    index
                    for index, row in enumerate(roles)
                    if row["workload_or_operation"] == "H-FLATSET"
                    and row["binding_kind"] == "MEMBER"
                )
                if mutation == "remove":
                    roles.pop(index)
                else:
                    roles[index]["member_contract_id"] = "DENSE-CLEAR"
                with self.assertRaisesRegex(ValueError, "role authority differs"):
                    authority.validate_role_capability_authority(
                        self.context,
                        roles,
                        outputs["DENSE-CAPABILITY-UNIT-AUTHORITY.tsv"],
                    )
                self.assert_public_rejects(outputs)

    def test_six_closure_sensitive_capabilities_reject_removal_and_substitution(self) -> None:
        for capability in authority.audited_capability_groups():
            for mutation in ("remove", "substitute"):
                with self.subTest(capability=capability, mutation=mutation):
                    outputs = self.copied_outputs()
                    rows = outputs["DENSE-CAPABILITY-UNIT-AUTHORITY.tsv"]
                    index = next(
                        index
                        for index, row in enumerate(rows)
                        if row["capability_id"] == capability
                        and row["binding_kind"] == "MEMBER"
                    )
                    if mutation == "remove":
                        rows.pop(index)
                    else:
                        replacement = next(iter(authority.EXCLUDED_MEMBERS))
                        rows[index]["member_contract_id"] = replacement
                        rows[index]["applicability"] = "EXCLUDED-BLOCKS-CLAIM"
                        rows[index]["binding_kind"] = "EXCLUDED_MEMBER"
                    with self.assertRaisesRegex(ValueError, "capability authority differs"):
                        authority.validate_role_capability_authority(
                            self.context,
                            outputs["DENSE-ROLE-UNIT-AUTHORITY.tsv"],
                            rows,
                        )
                    self.assert_public_rejects(outputs)

    def test_ab_seal_covers_every_h_flatset_witness_member(self) -> None:
        roles = self.outputs["DENSE-ROLE-UNIT-AUTHORITY.tsv"]
        capabilities = self.outputs["DENSE-CAPABILITY-UNIT-AUTHORITY.tsv"]
        flatset = {
            str(row["member_contract_id"])
            for row in roles
            if row["workload_or_operation"] == "H-FLATSET"
            and row["binding_kind"] == "MEMBER"
        }
        seal = {
            str(row["member_contract_id"])
            for row in capabilities
            if row["capability_id"] == "AB-SEAL"
            and row["applicability"] == "REQUIRED"
            and row["binding_kind"] == "MEMBER"
        }
        self.assertEqual(flatset, set(authority.ROLE_MEMBER_BINDINGS["H-FLATSET"]))
        self.assertTrue(flatset <= seal)

    def test_active_br_stored_member_removal_is_rejected(self) -> None:
        outputs = self.copied_outputs()
        rows = outputs["DENSE-CAPABILITY-UNIT-AUTHORITY.tsv"]
        index = next(
            index
            for index, row in enumerate(rows)
            if row["capability_id"] == "BR-STORED"
            and row["applicability"] == "REQUIRED"
            and row["binding_kind"] == "MEMBER"
        )
        rows.pop(index)
        with self.assertRaisesRegex(ValueError, "capability authority differs"):
            authority.validate_role_capability_authority(
                self.context,
                outputs["DENSE-ROLE-UNIT-AUTHORITY.tsv"],
                rows,
            )
        self.assert_public_rejects(outputs)

    def test_active_br_stored_excluded_terminal_removal_is_rejected(self) -> None:
        outputs = self.copied_outputs()
        rows = outputs["DENSE-CAPABILITY-UNIT-AUTHORITY.tsv"]
        index = next(
            index
            for index, row in enumerate(rows)
            if row["capability_id"] == "BR-STORED"
            and row["applicability"] == "EXCLUDED-BLOCKS-CLAIM"
            and row["binding_kind"] == "EXCLUDED_MEMBER"
        )
        rows.pop(index)
        with self.assertRaisesRegex(ValueError, "capability authority differs"):
            authority.validate_role_capability_authority(
                self.context,
                outputs["DENSE-ROLE-UNIT-AUTHORITY.tsv"],
                rows,
            )
        self.assert_public_rejects(outputs)

    def test_o_rope_unique_terminal_omission_is_rejected(self) -> None:
        outputs = self.copied_outputs()
        roles = outputs["DENSE-ROLE-UNIT-AUTHORITY.tsv"]
        outputs["DENSE-ROLE-UNIT-AUTHORITY.tsv"] = [
            row for row in roles if row["control_or_witness_id"] != "O-ROPE-UNIQUE"
        ]
        with self.assertRaisesRegex(ValueError, "role authority differs"):
            authority.validate_role_capability_authority(
                self.context,
                outputs["DENSE-ROLE-UNIT-AUTHORITY.tsv"],
                outputs["DENSE-CAPABILITY-UNIT-AUTHORITY.tsv"],
            )
        self.assert_public_rejects(outputs)

    def test_excluded_evidence_member_terminal_removal_is_rejected(self) -> None:
        outputs = self.copied_outputs()
        members = outputs["DENSE-EVIDENCE-MEMBER-AUTHORITY.tsv"]
        removed = next(
            row
            for row in members
            if row["member_contract_id"] in authority.EXCLUDED_MEMBERS
            and row["subject_kind"] == "DIRECT_EVIDENCE"
        )
        subject = str(removed["subject_identity"])
        member = str(removed["member_contract_id"])
        outputs["DENSE-EVIDENCE-MEMBER-AUTHORITY.tsv"] = [
            row
            for row in members
            if not (
                row["subject_identity"] == subject
                and row["member_contract_id"] == member
            )
        ]
        target = next(
            row
            for row in outputs["DENSE-EVIDENCE-TARGET-AUTHORITY.tsv"]
            if row["subject_identity"] == subject
            and member in authority.csv_ids(str(row["member_contract_ids"]))
        )
        retained = tuple(
            item
            for item in authority.csv_ids(str(target["member_contract_ids"]))
            if item != member
        )
        target["member_contract_ids"] = ",".join(retained) or "NONE"
        with self.assertRaisesRegex(ValueError, "closed identity registry"):
            authority.validate_target_member_authority(
                self.context,
                outputs["DENSE-SELECTOR-EXPANSION-AUTHORITY.tsv"],
                outputs["DENSE-EVIDENCE-TARGET-AUTHORITY.tsv"],
                outputs["DENSE-EVIDENCE-MEMBER-AUTHORITY.tsv"],
            )
        self.assert_public_rejects(outputs)

    def test_outcome_resolver_preserves_multi_target_evidence_units(self) -> None:
        rows = self.outputs["DENSE-EVIDENCE-MEMBER-AUTHORITY.tsv"]
        grouped: dict[tuple[str, str, str], list[dict[str, object]]] = {}
        for row in rows:
            key = (
                str(row["subject_identity"]),
                str(row["cluster_id"]),
                str(row["member_contract_id"]),
            )
            grouped.setdefault(key, []).append(row)
        key, authority_rows = next(
            (key, values)
            for key, values in grouped.items()
            if len({str(row["target_id"]) for row in values}) > 1
        )
        subject, cluster_id, member = key
        contract = {
            "contract_id": "TEST-MULTI-TARGET-CONTRACT",
            "member_contract_id": member,
            "outcome_id": "TEST-OUTCOME",
            "cluster_id": cluster_id,
            "policy_variant_id": "TEST-POLICY",
            "status": "TEST-ONLY",
            "evidence_identity_ids": subject,
        }
        resolved = authority.resolve_evidence_outcomes(authority_rows, [contract])
        self.assertEqual(len(resolved), len(authority_rows))
        self.assertEqual(
            {str(row["target_id"]) for row in resolved},
            {str(row["target_id"]) for row in authority_rows},
        )
        with self.assertRaisesRegex(ValueError, "duplicate evidence/member authority"):
            authority.resolve_evidence_outcomes(
                [*authority_rows, dict(authority_rows[0])], [contract]
            )

    def test_every_local_input_authority_field_mutation_is_rejected(self) -> None:
        name = "DENSE-LOCAL-DECLARATIVE-INPUT-AUTHORITY.tsv"
        for row_index in range(len(self.outputs[name])):
            for field in authority.OUTPUT_FIELDS[name]:
                with self.subTest(row=row_index, field=field):
                    outputs = self.copied_outputs()
                    row = outputs[name][row_index]
                    row[field] = f"{row[field]}-MUTATED"
                    self.assert_public_rejects(outputs)

    def test_every_frozen_input_authority_field_mutation_is_rejected(self) -> None:
        name = "DENSE-FROZEN-G0-INPUT-AUTHORITY.tsv"
        for row_index in range(len(self.outputs[name])):
            for field in authority.OUTPUT_FIELDS[name]:
                with self.subTest(row=row_index, field=field):
                    outputs = self.copied_outputs()
                    row = outputs[name][row_index]
                    row[field] = f"{row[field]}-MUTATED"
                    self.assert_public_rejects(outputs)


if __name__ == "__main__":
    unittest.main()
