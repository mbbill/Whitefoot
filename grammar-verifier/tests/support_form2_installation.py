"""Reconstruct the approved pre-migration FORM-2 corpus from installed v0.9."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile


VERIFIER_ROOT = Path(__file__).resolve().parents[1]
REPOSITORY = VERIFIER_ROOT.parent
PROPOSAL = VERIFIER_ROOT / "proposal"

import sys

if str(PROPOSAL) not in sys.path:
    sys.path.insert(0, str(PROPOSAL))

from form2_independent_inputs import (  # noqa: E402
    IndependentInputs,
    IndependentSource,
)
from form2_inputs import (  # noqa: E402
    ProtectedInputs,
    ProtectedSource,
    _extract_form2_rule,
)


BASELINE_SHA256 = "9d4ff925668a3341543d555c5243ef0b74ca5e7e275617ff4808d90c290dc48a"
MANIFEST_SHA256 = "20bb50032c112150c3d9a7387a17bde708922e426550b47b64f2214cd7341d69"
PATCHES = (
    (
        VERIFIER_ROOT / "evidence" / "form2-structural-migration.patch",
        "4b626ff44a9bc3cec96e41d9f3fa93b937a36397b7970b9310d39039cf8eb1f2",
    ),
    (
        VERIFIER_ROOT / "evidence" / "v0.9-post-form2-case-intent.patch",
        "62916bfc1bcc9e4eaa0461c33015cb30a2abe113f3aebcc807a3b8c492c0d54a",
    ),
    (
        VERIFIER_ROOT / "evidence" / "v0.9-manifest-metadata.patch",
        "ae48711659c881ab2e3ca4794641ffae948ed52a2e1bdf62f61da764c7be48a6",
    ),
)


def _digest(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def _manifest(raw: bytes) -> dict[str, dict[str, object]]:
    rows: dict[str, dict[str, object]] = {}
    for line in raw.decode("utf-8").splitlines():
        if not line or line.startswith("#"):
            continue
        row = json.loads(line)
        identifier = row.get("id")
        if identifier is not None:
            rows[identifier] = row
    return rows


def _reverse_installed_corpus(root: Path) -> None:
    shutil.copytree(REPOSITORY / "conformance", root / "conformance")
    for patch, expected in PATCHES:
        if _digest(patch.read_bytes()) != expected:
            raise AssertionError(f"approved patch changed: {patch.name}")
    for patch, _ in reversed(PATCHES):
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


def reconstruct_form2_inputs() -> tuple[ProtectedInputs, IndependentInputs]:
    """Return both historical input models after an exact C/B/A reversal."""

    inventory_report = json.loads(
        (VERIFIER_ROOT / "evidence" / "form2-structural-layout-evidence.json").read_text(
            encoding="ascii"
        )
    )
    inventory = inventory_report["protected_inventory"]["inventory"]
    with tempfile.TemporaryDirectory(prefix="whitefoot-form2-preimage-") as directory:
        root = Path(directory)
        _reverse_installed_corpus(root)
        manifest_raw = (root / "conformance" / "manifest.jsonl").read_bytes()
        if _digest(manifest_raw) != MANIFEST_SHA256:
            raise AssertionError("reconstructed manifest is not the approved preimage")
        manifest = _manifest(manifest_raw)

        primary_sources: list[ProtectedSource] = []
        independent_sources: list[IndependentSource] = []
        manifest_ids: set[str] = set()
        for row in inventory:
            relative = row["path"]
            raw = (root / relative).read_bytes()
            if _digest(raw) != row["sha256"]:
                raise AssertionError(f"reconstructed source is not the approved preimage: {relative}")
            identifier = row["id"]
            metadata = manifest.get(identifier) if identifier is not None else None
            if identifier is not None:
                if metadata is None:
                    raise AssertionError(f"manifest record missing for {identifier}")
                manifest_ids.add(identifier)
            primary_sources.append(
                ProtectedSource(identifier, relative, raw, row["sha256"], metadata)
            )
            independent_sources.append(
                IndependentSource(identifier, relative, raw, metadata)
            )
        if manifest_ids != set(manifest):
            raise AssertionError("reconstructed manifest and source inventory disagree")

    specification = (REPOSITORY / "spec" / "kernel-spec-v0.8.md").read_bytes()
    form2_rule, form2_rule_sha256 = _extract_form2_rule(specification)
    candidate = (PROPOSAL / "kernel-spec-successor-candidate.md").read_bytes()
    primary = ProtectedInputs(
        baseline_raw=b"",
        baseline_sha256=BASELINE_SHA256,
        manifest_raw=manifest_raw,
        manifest_sha256=MANIFEST_SHA256,
        specification_raw=specification,
        specification_sha256=_digest(specification),
        form2_rule=form2_rule,
        form2_rule_sha256=form2_rule_sha256,
        sources=tuple(primary_sources),
    )
    independent = IndependentInputs(
        baseline_sha256=BASELINE_SHA256,
        candidate=candidate,
        candidate_sha256=_digest(candidate),
        manifest_sha256=MANIFEST_SHA256,
        sources=tuple(independent_sources),
    )
    return primary, independent
