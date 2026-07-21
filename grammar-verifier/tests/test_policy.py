from __future__ import annotations

import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
REPOSITORY = ROOT.parent
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

from installed_policy import (  # noqa: E402
    DERIVATION_AMENDMENT_SHA256,
    HISTORICAL_V08_BINDINGS,
    PRE_V09_DERIVATION_LEDGER_SHA256,
    REVIEW_PACKET_BINDINGS,
    validate_derivation_ledger,
    validate_review_packet,
)
from runner_inputs import RunnerError, SUCCESSOR_SHA256  # noqa: E402
EXPECTED_CHANGED_RULES = {
    "CONST-1",
    "CONST-2",
    "DIAG-1",
    "EX-1",
    "FN-1",
    "FN-4",
    "FN-8",
    "FORM-2",
    "FORM-3",
    "FORM-4",
    "FORM-5",
    "FORM-7",
    "GIVE-1",
    "GRAM-1",
    "GRAM-2",
    "GRAM-3",
    "GRAM-4",
    "GRAM-7",
    "PRE-1",
    "PROG-1",
    "TYPE-6",
}
EXPECTED_ADDED_RULES = {"PROG-2"}
RULE_HEADING = re.compile(rb"(?m)^\[([A-Z]+-[0-9]+)\]")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def numbered_rules(source: bytes) -> dict[str, bytes]:
    matches = list(RULE_HEADING.finditer(source))
    result: dict[str, bytes] = {}
    for index, match in enumerate(matches):
        owner = match.group(1).decode("ascii")
        end = matches[index + 1].start() if index + 1 < len(matches) else len(source)
        if owner in result:
            raise AssertionError(f"duplicate numbered rule {owner}")
        result[owner] = source[match.start() : end]
    return result


def census() -> dict[str, object]:
    return json.loads((ROOT / "proposal" / "protected-surface-census.json").read_text(encoding="utf-8"))


def manifest_rows(path: Path) -> list[dict[str, object]]:
    lines = path.read_text(encoding="utf-8").splitlines()
    return [json.loads(line) for line in lines if line.strip() and not line.lstrip().startswith("#")]


def expectation_projection(rows: list[dict[str, object]]) -> bytes:
    projected = [
        {"expect": row["expect"], "id": row["id"], "status": row["status"]}
        for row in sorted(
            (entry for entry in rows if "id" in entry),
            key=lambda entry: str(entry["id"]).encode("utf-8"),
        )
    ]
    return (json.dumps(projected, ensure_ascii=True, separators=(",", ":"), sort_keys=True) + "\n").encode("ascii")


class ProposalBindingTests(unittest.TestCase):
    def test_declared_current_and_proposal_hashes_are_exact(self) -> None:
        current = REPOSITORY / "spec" / "kernel-spec-v0.8.md"
        proposal = ROOT / "proposal" / "kernel-spec-successor-candidate.md"
        installed = REPOSITORY / "spec" / "kernel-spec-v0.9.md"
        inventory = census()
        current_sha = hashlib.sha256(current.read_bytes()).hexdigest()
        proposal_sha = hashlib.sha256(proposal.read_bytes()).hexdigest()
        self.assertEqual(inventory["active_current_specification"]["sha256"], current_sha)
        self.assertEqual(inventory["candidate_installation"]["proposal_sha256"], proposal_sha)
        delta = (ROOT / "proposal" / "DELTA.md").read_text(encoding="utf-8")
        declared = re.findall(r"SHA-256: `([0-9a-f]{64})`", delta)
        self.assertEqual(declared, [current_sha, proposal_sha])
        self.assertEqual(inventory["candidate_installation"]["candidate_byte_length"], len(proposal.read_bytes()))
        self.assertEqual(proposal_sha, SUCCESSOR_SHA256)
        self.assertEqual(installed.read_bytes(), proposal.read_bytes())

    def test_phase_two_declares_no_guarded_edit(self) -> None:
        self.assertEqual(census()["phase_2_guarded_edits"], [])

    def test_hostile_review_bindings_are_exact_and_owner_visible(self) -> None:
        inventory = census()
        reviews = inventory["hostile_review_bindings"]
        expected_paths = {
            "grammar-verifier/proposal/CASE-INTENT-HOSTILE-REVIEW.md",
            "grammar-verifier/proposal/FN4-HOSTILE-REVIEW.md",
            "grammar-verifier/proposal/FORM2-FROZEN-HOSTILE-REVIEW.md",
            "grammar-verifier/proposal/REMAINING-SEMANTIC-HOSTILE-REVIEW.md",
            "grammar-verifier/proposal/SUCCESSOR-HOSTILE-REVIEW.md",
        }
        self.assertEqual({entry["path"] for entry in reviews}, expected_paths)
        delta = (ROOT / "proposal" / "DELTA.md").read_text(encoding="utf-8")
        for entry in reviews:
            path = REPOSITORY / entry["path"]
            self.assertEqual(sha256(path), entry["sha256"], entry["path"])
            self.assertIn(entry["path"].removeprefix("grammar-verifier/"), delta)
            self.assertIn(entry["sha256"], delta)

    def test_candidate_changes_only_the_declared_rule_set(self) -> None:
        current = (REPOSITORY / "spec" / "kernel-spec-v0.8.md").read_bytes()
        proposal = (ROOT / "proposal" / "kernel-spec-successor-candidate.md").read_bytes()
        self.assertTrue(current.startswith(b"# Kernel Specification v0.8\n\nStatus: DRAFT v0.8 "))
        self.assertTrue(
            proposal.startswith(
                b"# Kernel Specification v0.9\n\n"
                b"Status: DRAFT v0.9 (2026-07-21; canonical-frontend entrance closure)."
            )
        )
        self.assertIn(
            b"These exact bytes are authoritative only after their complete evidence",
            proposal.split(b"\n", 4)[2],
        )

        current_rules = numbered_rules(current)
        proposal_rules = numbered_rules(proposal)
        self.assertEqual(set(proposal_rules) - set(current_rules), EXPECTED_ADDED_RULES)
        self.assertEqual(set(current_rules) - set(proposal_rules), set())
        changed = {
            owner
            for owner in set(current_rules) & set(proposal_rules)
            if current_rules[owner] != proposal_rules[owner]
        }
        self.assertEqual(changed, EXPECTED_CHANGED_RULES)
        for owner in set(current_rules) - EXPECTED_CHANGED_RULES:
            self.assertEqual(current_rules[owner], proposal_rules[owner], owner)

    def test_historical_review_packet_and_census_remain_exact(self) -> None:
        validate_review_packet(ROOT)
        evidence_files = {
            path.name
            for path in (ROOT / "evidence").iterdir()
            if path.is_file()
        }
        self.assertEqual(evidence_files, set(REVIEW_PACKET_BINDINGS))
        for name, expected in REVIEW_PACKET_BINDINGS.items():
            self.assertEqual(sha256(ROOT / "evidence" / name), expected, name)
        for relative, expected in HISTORICAL_V08_BINDINGS.items():
            self.assertEqual(sha256(REPOSITORY / relative), expected, relative)

        inventory = census()
        declared = inventory["protected_surface_baseline"]
        self.assertEqual(
            declared["sha256"],
            "9d4ff925668a3341543d555c5243ef0b74ca5e7e275617ff4808d90c290dc48a",
        )
        self.assertEqual(declared["kernel_specification_records"], 9)
        self.assertEqual(declared["conformance_records"], 307)
        self.assertEqual(
            declared["legacy_unmanifested_case"]["sha256"],
            "ae99d9b9b99e02e9c6c5f2af54f0924b7b1a0f5ee0422d29958b01b597adf759",
        )
        self.assertEqual(
            (ROOT / "proposal" / "DELTA.md").read_bytes(),
            (ROOT / "evidence" / "proposal-delta.md").read_bytes(),
        )
        self.assertEqual(
            (ROOT / "proposal" / "protected-surface-census.json").read_bytes(),
            (ROOT / "evidence" / "protected-surface-census.json").read_bytes(),
        )

    def test_derivation_ledger_is_one_exact_append_to_the_pinned_prefix(self) -> None:
        amendment = (
            ROOT / "proposal" / "DERIVATION-LEDGER-v0.9-AMENDMENT.md"
        ).read_bytes()
        ledger = (REPOSITORY / "spec" / "derivation-ledger.md").read_bytes()
        self.assertEqual(hashlib.sha256(amendment).hexdigest(), DERIVATION_AMENDMENT_SHA256)
        validate_derivation_ledger(amendment, ledger)
        body_marker = b"<!-- BEGIN EXACT V0.9 DERIVATION-LEDGER APPEND -->\n\n"
        end_marker = b"\n<!-- END EXACT V0.9 DERIVATION-LEDGER APPEND -->\n"
        body = amendment.split(body_marker, 1)[1].split(end_marker, 1)[0]
        prefix = ledger[: -len(b"\n" + body)]
        self.assertEqual(hashlib.sha256(prefix).hexdigest(), PRE_V09_DERIVATION_LEDGER_SHA256)
        with self.assertRaises(RunnerError):
            validate_derivation_ledger(amendment, b"changed\n" + ledger)
        with self.assertRaises(RunnerError):
            validate_derivation_ledger(amendment, ledger[:-1] + b"x")

    def test_protected_install_patches_compose_exactly_and_preserve_outcomes(self) -> None:
        patches = (
            (
                ROOT / "evidence" / "form2-structural-migration.patch",
                "4b626ff44a9bc3cec96e41d9f3fa93b937a36397b7970b9310d39039cf8eb1f2",
            ),
            (
                ROOT / "evidence" / "v0.9-post-form2-case-intent.patch",
                "62916bfc1bcc9e4eaa0461c33015cb30a2abe113f3aebcc807a3b8c492c0d54a",
            ),
            (
                ROOT / "evidence" / "v0.9-manifest-metadata.patch",
                "ae48711659c881ab2e3ca4794641ffae948ed52a2e1bdf62f61da764c7be48a6",
            ),
        )
        for path, expected in patches:
            self.assertEqual(sha256(path), expected, path.name)

        expected_paths = (
            None,
            {
                "conformance/cases/fn4-pos-law-discharged.wf",
                "conformance/cases/fn4-pos-law-in-contract.wf",
                "conformance/cases/fn8-neg-requires-control.wf",
                "conformance/cases/gram1-pos-lookahead.wf",
                "conformance/cases/gram7-pos-two-productions.wf",
                "conformance/manifest.jsonl",
            },
            {"conformance/manifest.jsonl"},
        )
        installed_manifest = REPOSITORY / "conformance" / "manifest.jsonl"
        installed_rows = manifest_rows(installed_manifest)
        installed_projection = expectation_projection(installed_rows)
        self.assertEqual(
            hashlib.sha256(installed_projection).hexdigest(),
            "5fb0e54ec006c3fea82d5fc0d8c454e5e9f022ba472cdcc6a90c44a31ade2132",
        )
        self.assertEqual(installed_manifest.stat().st_size, 99869)
        self.assertEqual(
            sha256(installed_manifest),
            "0eff27bfb87ca14086f31f4b171d72c9eb1a49072aa4563a3f7c937d0b8bb90c",
        )

        with tempfile.TemporaryDirectory(prefix="whitefoot-protected-compose-") as temporary:
            root = Path(temporary)
            shutil.copytree(REPOSITORY / "conformance", root / "conformance")
            installed_snapshot = {
                path.relative_to(root).as_posix(): path.read_bytes()
                for path in sorted((root / "conformance").rglob("*"))
                if path.is_file()
            }
            for index, (patch, _) in enumerate(patches):
                numstat = subprocess.run(
                    ("git", "apply", "--numstat", str(patch)),
                    cwd=root,
                    check=True,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                    text=True,
                ).stdout.splitlines()
                observed_paths = {line.split("\t", 2)[2] for line in numstat}
                if expected_paths[index] is None:
                    self.assertEqual(len(observed_paths), 274)
                else:
                    self.assertEqual(observed_paths, expected_paths[index])

            for patch, _ in reversed(patches):
                subprocess.run(
                    ("git", "apply", "--reverse", "--check", str(patch)),
                    cwd=root,
                    check=True,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                )
                subprocess.run(
                    ("git", "apply", "--reverse", str(patch)),
                    cwd=root,
                    check=True,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                )

            base_manifest = root / "conformance" / "manifest.jsonl"
            self.assertEqual(base_manifest.stat().st_size, 99776)
            self.assertEqual(
                sha256(base_manifest),
                "20bb50032c112150c3d9a7387a17bde708922e426550b47b64f2214cd7341d69",
            )
            base_rows = manifest_rows(base_manifest)
            self.assertEqual(expectation_projection(base_rows), installed_projection)
            self.assertEqual(
                sha256(root / "conformance" / "cases" / "pending-const2-item.wf"),
                "ae99d9b9b99e02e9c6c5f2af54f0924b7b1a0f5ee0422d29958b01b597adf759",
            )

            for patch, _ in patches:
                subprocess.run(
                    ("git", "apply", "--check", str(patch)),
                    cwd=root,
                    check=True,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                )
                subprocess.run(
                    ("git", "apply", str(patch)),
                    cwd=root,
                    check=True,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                )

            final_sources = {
                "fn4-pos-law-discharged.wf": "9cd070cd331b163f0f230c8c57ee7c38f0d7aa23a6807987981bc29ee13c0418",
                "fn4-pos-law-in-contract.wf": "66f30c62380f95a332a00bd468ae9505307c87ca77db3c62dcb13f1e767b7d0d",
                "fn8-neg-requires-control.wf": "00a2b65bbfd272897a2b0596123c32e0069306c680cb77c4f7f229337c25202f",
                "gram1-pos-lookahead.wf": "3b146c7ac6185b12e5e703a4643cf0afd3c8b4f05ccc56fdd6ef5d6a07b71b18",
                "gram7-pos-two-productions.wf": "a1c1986fedbbc00c0756986582dccebe07b7aad013258ccb4936a40dc5d6e43e",
            }
            for name, expected in final_sources.items():
                self.assertEqual(sha256(root / "conformance" / "cases" / name), expected, name)

            final_manifest = root / "conformance" / "manifest.jsonl"
            self.assertEqual(final_manifest.stat().st_size, 99869)
            self.assertEqual(
                sha256(final_manifest),
                "0eff27bfb87ca14086f31f4b171d72c9eb1a49072aa4563a3f7c937d0b8bb90c",
            )
            final_rows = manifest_rows(final_manifest)
            self.assertEqual(expectation_projection(final_rows), installed_projection)
            self.assertEqual(len(final_rows), len(base_rows))
            for before, after in zip(base_rows, final_rows):
                for field in ("id", "rule", "rules", "expect", "status", "covered_by"):
                    self.assertEqual(before.get(field), after.get(field), field)
            final_snapshot = {
                path.relative_to(root).as_posix(): path.read_bytes()
                for path in sorted((root / "conformance").rglob("*"))
                if path.is_file()
            }
            self.assertEqual(final_snapshot, installed_snapshot)

    def test_historical_v0_8_reference_inventory_record_is_frozen(self) -> None:
        inventory = census()
        classification = inventory["v0_8_reference_classification"]
        self.assertEqual(
            classification["canonical_path_encoding"],
            "UTF-8 paths sorted bytewise, each followed by LF",
        )
        self.assertEqual(classification["direct_text_reference_file_count"], 58)
        self.assertEqual(
            classification["direct_text_reference_inventory_sha256"],
            "b12dc10231cd5daf42795997d66e3ba6468d8e4e22a4d61ec418ab5d177e92ae",
        )
        self.assertEqual(classification["unclassified_file_count"], 0)
        self.assertEqual(len(inventory["active_target_references_to_update_after_approval"]), 40)
        self.assertIn(
            "spec/kernel-spec-v0.8.md",
            inventory["historical_v0_8_records_to_preserve"],
        )


if __name__ == "__main__":
    unittest.main()
