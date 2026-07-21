from __future__ import annotations

import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
REPOSITORY = ROOT.parent
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
        inventory = census()
        current_sha = hashlib.sha256(current.read_bytes()).hexdigest()
        proposal_sha = hashlib.sha256(proposal.read_bytes()).hexdigest()
        self.assertEqual(inventory["active_current_specification"]["sha256"], current_sha)
        self.assertEqual(inventory["candidate_installation"]["proposal_sha256"], proposal_sha)
        delta = (ROOT / "proposal" / "DELTA.md").read_text(encoding="utf-8")
        declared = re.findall(r"SHA-256: `([0-9a-f]{64})`", delta)
        self.assertEqual(declared, [current_sha, proposal_sha])
        self.assertEqual(inventory["candidate_installation"]["candidate_byte_length"], len(proposal.read_bytes()))

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

    def test_census_protected_hashes_and_counts_match_repository_bytes(self) -> None:
        inventory = census()
        declared = inventory["protected_surface_baseline"]
        baseline_path = REPOSITORY / declared["path"]
        self.assertEqual(sha256(baseline_path), declared["sha256"])
        baseline = json.loads(baseline_path.read_text(encoding="utf-8"))
        self.assertEqual(len(baseline["kernel_specs"]), declared["kernel_specification_records"])
        self.assertEqual(len(baseline["conformance"]), declared["conformance_records"])
        self.assertEqual(
            {path: len(records) for path, records in baseline["oracles"].items()},
            declared["guarded_oracle_digest_counts"],
        )
        self.assertEqual(
            {path: len(records) for path, records in baseline["tests"].items()},
            declared["guarded_reference_test_counts"],
        )
        legacy = declared["legacy_unmanifested_case"]
        self.assertEqual(sha256(REPOSITORY / legacy["path"]), legacy["sha256"])

        reviewed = inventory["protected_surfaces_reviewed_without_expected_edits"]
        cases = reviewed["conformance_cases"]
        self.assertEqual(sha256(REPOSITORY / "conformance" / "manifest.jsonl"), cases["manifest_sha256"])
        discovered = sorted(
            f"conformance/cases/{identifier}.wf"
            for identifier in baseline["conformance"]
            if ":" not in identifier
            and b"deref(" in (REPOSITORY / "conformance" / "cases" / f"{identifier}.wf").read_bytes()
        )
        self.assertEqual(discovered, cases["deref_source_paths"])
        self.assertEqual(len(discovered), cases["deref_source_case_count"])
        encoded = b"".join(path.encode("utf-8") + b"\n" for path in sorted(discovered, key=lambda item: item.encode("utf-8")))
        self.assertEqual(
            cases["deref_source_path_inventory_encoding"],
            "UTF-8 paths sorted bytewise, each followed by LF",
        )
        self.assertEqual(hashlib.sha256(encoded).hexdigest(), cases["deref_source_path_inventory_sha256"])

        frozen = reviewed["frozen_oracles"]
        self.assertEqual(sorted(frozen["paths"]), sorted(frozen["sha256"]))
        for relative, digest in frozen["sha256"].items():
            self.assertEqual(sha256(REPOSITORY / relative), digest)
        reference = reviewed["reference_semantics_tests"]
        self.assertEqual(sha256(REPOSITORY / reference["path"]), reference["sha256"])

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
        original_rows = manifest_rows(REPOSITORY / "conformance" / "manifest.jsonl")
        original_projection = expectation_projection(original_rows)
        self.assertEqual(
            hashlib.sha256(original_projection).hexdigest(),
            "5fb0e54ec006c3fea82d5fc0d8c454e5e9f022ba472cdcc6a90c44a31ade2132",
        )

        with tempfile.TemporaryDirectory(prefix="whitefoot-protected-compose-") as temporary:
            root = Path(temporary)
            shutil.copytree(REPOSITORY / "conformance", root / "conformance")
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
            self.assertEqual(expectation_projection(final_rows), original_projection)
            self.assertEqual(len(final_rows), len(original_rows))
            for before, after in zip(original_rows, final_rows):
                for field in ("id", "rule", "rules", "expect", "status", "covered_by"):
                    self.assertEqual(before.get(field), after.get(field), field)

    def test_direct_v0_8_reference_inventory_is_complete_and_classified(self) -> None:
        inventory = census()
        classification = inventory["v0_8_reference_classification"]
        tracked = subprocess.run(
            ("git", "ls-files", "-z"),
            cwd=REPOSITORY,
            check=True,
            stdout=subprocess.PIPE,
        ).stdout.split(b"\0")
        current_digest = inventory["active_current_specification"]["sha256"].encode("ascii")
        needles = (b"v0.8", b"v08", current_digest)
        excluded = (b"archive/", b"mcts_mem/", b"grammar-verifier/")
        direct: list[bytes] = []
        for relative in tracked:
            if not relative or relative.startswith(excluded):
                continue
            data = (REPOSITORY / relative.decode("utf-8")).read_bytes()
            if any(needle in data for needle in needles):
                direct.append(relative)
        direct.sort()
        encoded = b"".join(path + b"\n" for path in direct)
        self.assertEqual(classification["canonical_path_encoding"], "UTF-8 paths sorted bytewise, each followed by LF")
        self.assertEqual(len(direct), classification["direct_text_reference_file_count"])
        self.assertEqual(hashlib.sha256(encoded).hexdigest(), classification["direct_text_reference_inventory_sha256"])

        active = set(inventory["active_target_references_to_update_after_approval"])
        historical = inventory["historical_v0_8_records_to_preserve"]

        def classified(relative: str) -> bool:
            return relative in active or any(
                relative.startswith(entry) if entry.endswith("/") else relative == entry
                for entry in historical
            )

        unclassified = [path.decode("utf-8") for path in direct if not classified(path.decode("utf-8"))]
        self.assertEqual(len(unclassified), classification["unclassified_file_count"])
        self.assertEqual(unclassified, [])


if __name__ == "__main__":
    unittest.main()
