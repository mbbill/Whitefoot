"""Bindings that preserve the review packet after v0.9 installation."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

from runner_inputs import fail, read_regular
from runner_package import PACKAGE_FILE_LIMIT, validate_published_package


REVIEW_PACKET_BINDINGS = {
    "float-canonicality.json": "4ee9b329a4fd72d0cd9ed33af94b019b7b7fe68116181f280113f9b9a744062e",
    "form2-evidence.sha256": "b907fc38adbcf9174a832d66e36917f4df8c1c434d26651d65e39d9a0ec72a68",
    "form2-independent-evidence.sha256": "d4dd3bd42759e9138a93514020ec2ca39adc2139651e43ee7a15ce21cf6f1fdc",
    "form2-independent-report.json": "142a34c3b9e9fd1f3c20da9848bda3984092a88b7c995c63a5c2dcf22333b404",
    "form2-protected-syntax-repairs.patch": "724dbb970c8ce7ede7a52daf3ad2c9286b7872137e83f495fbf845df75252479",
    "form2-structural-layout-evidence.json": "7bab5d114dc1b4d0818232c88c580b1247e139e911eee6501c116bd6422fdf80",
    "form2-structural-migration.json": "775d54381999b670619e240426de285b28bb6483647d697d67db483e68c5f099",
    "form2-structural-migration.patch": "4b626ff44a9bc3cec96e41d9f3fa93b937a36397b7970b9310d39039cf8eb1f2",
    "frontend-boundary-evidence.json": "03eddf37794a2397815998768d0cd07558e3c519974bf7f5f8d628d0a9ced208",
    "frontend-boundary-evidence.sha256": "49b73463efd482a470522b5f7c9e4630cfbf9530ba7def7dc8719567713d9ecf",
    "oracle.raw": "d86b73aff8779b1213ec4b6359a4d6c670158accb56046fae0b82be650cabddb",
    "package.json": "39436bbabaf194be43251a6afa028ff2b95c53c309ae906020672e2be959d03d",
    "package.sha256": "c1926829467f7efb323bb643616af8443760fc6c02e69770d29558e7297ca880",
    "proposal-delta.md": "4b4e583f8469b4f58fffac3263437300a58ca126e7f8dca5f6d055599938eab0",
    "protected-surface-census.json": "bb224651916f9ad49186b51c7963b9792ee7b698fff7c875dc4806b106241bda",
    "report.json": "1e26d171b58504e10a9a2d4510f1e5f6fd6c7190a70ad4cccacd558db0905789",
    "report.sha256": "e2616501268d79fc78e99898883720e6e9b08962f2bd13714b0aafbf74fa8dd7",
    "static.raw": "230b273c0df040e5a1b38690c082817843d5ce6febadeb9ec11f253523214d61",
    "v0.9-manifest-metadata.patch": "ae48711659c881ab2e3ca4794641ffae948ed52a2e1bdf62f61da764c7be48a6",
    "v0.9-post-form2-case-intent.patch": "62916bfc1bcc9e4eaa0461c33015cb30a2abe113f3aebcc807a3b8c492c0d54a",
}

REVIEW_CANDIDATE_SHA256 = (
    "bdfb461d1901f610633c5cbcd2477d24df3c77ca90599b9580c8289e50b82b68"
)

HISTORICAL_V08_BINDINGS = {
    "frontend-corpus/v0.8/README.md": "f6b56cf9c80d9d565d596383f8c01b85c91d544953b1d21c19cf66ba96144b03",
    "frontend-corpus/v0.8/lexical-fixtures.json": "3fdb149bd1948a929f1933a5f62c9bab759435fc224da8295dc7c02d9e093de3",
    "tools/test_v08_lexical_model.py": "bf1dbad474d1613dfc6fcaedeffe45390fc2a657af5da83e5020c536f4a724d6",
    "tools/test_v08_lexical_observer.py": "74ed3d209973b178f5da5912e466460a56b0801d0030f790355bfe00f829563e",
    "tools/test_v08_terminal_ident_audit.py": "9b126c9799caf9fc1afe23bd788a10e11716fe62151f779b6db77e8fd857fe16",
    "tools/v08_lexical_model.py": "dec2d90dd08d80046d8689e5b485b46b5773e5097b7e013f8ef722213d6e9283",
    "tools/v08_lexical_observer.py": "c0c974ebd33bec9123211be1a5dcfd3f901148f44ecc42ab398c21062267f59e",
    "tools/v08_lexical_observer_runner.py": "e113489dc2cc156e97bb04d43ad722d4588c57ef91781180765c69a95ee33fa6",
    "tools/v08_terminal_ident_audit.py": "5b50aa2606a588a334305b257e64b3b1661ccd733987f44fc10af90e308cee49",
}

DERIVATION_AMENDMENT_SHA256 = (
    "f29b326f446aa9e5f512d079f1dbd14e641e6d840f18b69faab0ea39950e52a0"
)
PRE_V09_DERIVATION_LEDGER_SHA256 = (
    "2e1e5805c4fcea569c53458c9bc901f5466a7d653a6b9986e6091004d4d3bada"
)
_LEDGER_BEGIN = b"<!-- BEGIN EXACT V0.9 DERIVATION-LEDGER APPEND -->\n\n"
_LEDGER_END = b"\n<!-- END EXACT V0.9 DERIVATION-LEDGER APPEND -->\n"


def _digest(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def validate_derivation_ledger(amendment: bytes, installed_ledger: bytes) -> None:
    """Require the installed ledger to be one exact append to a pinned prefix."""

    if _digest(amendment) != DERIVATION_AMENDMENT_SHA256:
        fail("review_packet", "the approved derivation-ledger amendment changed")
    if amendment.count(_LEDGER_BEGIN) != 1 or amendment.count(_LEDGER_END) != 1:
        fail("review_packet", "the derivation-ledger amendment delimiters changed")
    body = amendment.split(_LEDGER_BEGIN, 1)[1].split(_LEDGER_END, 1)
    if len(body) != 2 or body[1] != b"":
        fail("review_packet", "the derivation-ledger amendment has bytes after its end delimiter")
    installed_suffix = b"\n" + body[0]
    if not installed_ledger.endswith(installed_suffix):
        fail("review_packet", "the installed derivation ledger lacks the exact approved append")
    pre_v09 = installed_ledger[: -len(installed_suffix)]
    if _digest(pre_v09) != PRE_V09_DERIVATION_LEDGER_SHA256:
        fail("review_packet", "the pre-v0.9 derivation ledger bytes changed")


def validate_review_packet(root: Path) -> None:
    """Require every approved review artifact to retain its exact bytes."""

    evidence = root / "evidence"
    validate_published_package(evidence)
    for name, expected in REVIEW_PACKET_BINDINGS.items():
        raw = read_regular(evidence / name, PACKAGE_FILE_LIMIT, f"review packet {name}")
        if _digest(raw) != expected:
            fail("review_packet", f"historical review artifact changed: {name}")

    delta = read_regular(root / "proposal" / "DELTA.md", 1_048_576, "review DELTA.md")
    census_raw = read_regular(
        root / "proposal" / "protected-surface-census.json",
        1_048_576,
        "review protected-surface-census.json",
    )
    if delta != read_regular(evidence / "proposal-delta.md", 1_048_576, "packet DELTA.md"):
        fail("review_packet", "proposal DELTA.md differs from its approved packet copy")
    if census_raw != read_regular(
        evidence / "protected-surface-census.json",
        1_048_576,
        "packet protected-surface-census.json",
    ):
        fail("review_packet", "proposal census differs from its approved packet copy")

    candidate = read_regular(
        root / "proposal" / "kernel-spec-successor-candidate.md",
        1_048_576,
        "reviewed successor candidate",
    )
    if _digest(candidate) != REVIEW_CANDIDATE_SHA256:
        fail("review_packet", "the reviewed successor candidate bytes changed")

    try:
        census = json.loads(census_raw.decode("ascii"))
    except (UnicodeDecodeError, json.JSONDecodeError):
        fail("review_packet", "the approved protected-surface census is malformed")
    reviews = census.get("hostile_review_bindings") if isinstance(census, dict) else None
    if not isinstance(reviews, list):
        fail("review_packet", "the approved census lacks hostile-review bindings")
    repository = root.parent
    for review in reviews:
        if not isinstance(review, dict):
            fail("review_packet", "a hostile-review binding is malformed")
        relative = review.get("path")
        expected = review.get("sha256")
        if not isinstance(relative, str) or not isinstance(expected, str):
            fail("review_packet", "a hostile-review binding is incomplete")
        raw = read_regular(repository / relative, 1_048_576, f"hostile review {relative}")
        if _digest(raw) != expected:
            fail("review_packet", f"approved hostile review changed: {relative}")

    for relative, expected in HISTORICAL_V08_BINDINGS.items():
        raw = read_regular(
            repository / relative,
            PACKAGE_FILE_LIMIT,
            f"historical v0.8 snapshot {relative}",
        )
        if _digest(raw) != expected:
            fail("review_packet", f"historical v0.8 snapshot changed: {relative}")

    amendment = read_regular(
        root / "proposal" / "DERIVATION-LEDGER-v0.9-AMENDMENT.md",
        1_048_576,
        "approved derivation-ledger amendment",
    )
    installed_ledger = read_regular(
        repository / "spec" / "derivation-ledger.md",
        PACKAGE_FILE_LIMIT,
        "installed derivation ledger",
    )
    validate_derivation_ledger(amendment, installed_ledger)
