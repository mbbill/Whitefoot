from __future__ import annotations

import hashlib
import json
from pathlib import Path
import re
import subprocess
import unittest


ROOT = Path(__file__).resolve().parents[1]
REPOSITORY = ROOT.parent
V09_TITLE = b"# Kernel Specification v0.9"
V09_STATUS = (
    b"Status: DRAFT v0.9 (2026-07-21; fixed-lowerword/IDENT partition and top-level signature visibility). "
    b"Excludes every lowercase token spelling produced by exact fixed grammar atoms from IDENT while preserving "
    b"fixed-token spellings, and makes all top-level function signatures visible throughout the closed compilation "
    b"unit while retaining lexical declaration-before-use for locals, regions, labels, and explicitly earlier "
    b"constants. Selection grounds: the fixed-lowerword/IDENT partition is evidence-selected by the independent "
    b"static and generalized-parser reports; signature visibility is owner-selected as A-01."
)
FORM3_BASE = b"[FORM-3] Lexical classes: IDENT `[a-z][a-z0-9_]*`"
FORM3_MODIFIER = b" excluding every lowercase token spelling produced by exact fixed grammar atoms in the complete grammar"
FN1_BASE = (
    b"[FN-1] Signatures state everything callers need: parameter modes/types, return mode/type, effect row, region "
    b"parameters. Bodies are checked against signatures; callers rely only on signatures."
)
FN1_ADDITION = (
    b" All top-level function signatures are visible throughout the closed compilation unit. Locals, regions, "
    b"labels, and explicitly earlier constants still follow lexical declaration-before-use."
)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def replace_once(source: bytes, old: bytes, new: bytes) -> bytes:
    if source.count(old) != 1:
        raise AssertionError(f"expected one exact replacement site for {old[:40]!r}")
    return source.replace(old, new, 1)


def census() -> dict[str, object]:
    return json.loads((ROOT / "proposal" / "protected-surface-census.json").read_text(encoding="utf-8"))


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

    def test_phase_two_declares_no_guarded_edit(self) -> None:
        self.assertEqual(census()["phase_2_guarded_edits"], [])

    def test_candidate_is_exact_v0_8_plus_only_the_three_declared_edits(self) -> None:
        current = (REPOSITORY / "spec" / "kernel-spec-v0.8.md").read_bytes()
        proposal = (ROOT / "proposal" / "kernel-spec-successor-candidate.md").read_bytes()
        title, status, remainder = current.split(b"\n\n", 2)
        self.assertEqual(title, b"# Kernel Specification v0.8")
        self.assertTrue(status.startswith(b"Status: DRAFT v0.8 "))
        reconstructed = V09_TITLE + b"\n\n" + V09_STATUS + b"\n\nPrior: " + status[len(b"Status: ") :] + b"\n\n" + remainder
        reconstructed = replace_once(reconstructed, FORM3_BASE, FORM3_BASE + FORM3_MODIFIER)
        reconstructed = replace_once(reconstructed, FN1_BASE, FN1_BASE + FN1_ADDITION)
        self.assertEqual(reconstructed, proposal)

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
