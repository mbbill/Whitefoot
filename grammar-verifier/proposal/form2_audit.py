from __future__ import annotations

import argparse
import os
from pathlib import Path
import sys
import tempfile

PROPOSAL_ROOT = Path(__file__).resolve().parent
if str(PROPOSAL_ROOT) not in sys.path:
    sys.path.insert(0, str(PROPOSAL_ROOT))

from form2_inputs import GRAMMAR_ROOT, InputError, load_protected_inputs
from form2_report import ARTIFACT_NAMES, ReportError, build_artifacts, checksum_index


EVIDENCE_ROOT = GRAMMAR_ROOT / "evidence"
CHECKSUM_NAME = "form2-evidence.sha256"


class AuditFailure(RuntimeError):
    pass


def expected_artifacts() -> dict[str, bytes]:
    inputs = load_protected_inputs()
    artifacts = build_artifacts(inputs)
    artifacts[CHECKSUM_NAME] = checksum_index(artifacts)
    return artifacts


def check_committed(output: Path = EVIDENCE_ROOT) -> None:
    expected = expected_artifacts()
    unexpected = []
    for name, raw in expected.items():
        path = output / name
        if path.is_symlink() or not path.is_file():
            raise AuditFailure(
                f"committed FORM-2 artifact is not a regular non-symlink file: {path}"
            )
        try:
            actual = path.read_bytes()
        except OSError as error:
            raise AuditFailure(f"cannot read committed FORM-2 artifact {path}: {error}") from error
        if actual != raw:
            raise AuditFailure(f"committed FORM-2 artifact is stale: {path}")
    for name in ARTIFACT_NAMES:
        if name not in expected:
            unexpected.append(name)
    if unexpected:
        raise AuditFailure(f"internal artifact inventory mismatch: {unexpected!r}")


def _atomic_write(path: Path, raw: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary_name = tempfile.mkstemp(
        prefix=f".{path.name}.", dir=path.parent
    )
    temporary = Path(temporary_name)
    try:
        with os.fdopen(descriptor, "wb") as stream:
            os.fchmod(stream.fileno(), 0o644)
            stream.write(raw)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, path)
    finally:
        try:
            temporary.unlink()
        except FileNotFoundError:
            pass


def write_artifacts(output: Path = EVIDENCE_ROOT) -> None:
    for name, raw in expected_artifacts().items():
        _atomic_write(output / name, raw)


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Audit exact protected sources against FORM-2 without applying changes"
    )
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true")
    mode.add_argument("--write", action="store_true")
    parser.add_argument("--output", type=Path, default=EVIDENCE_ROOT)
    arguments = parser.parse_args()
    try:
        if arguments.write:
            write_artifacts(arguments.output)
        else:
            check_committed(arguments.output)
    except (AuditFailure, InputError, ReportError, OSError, ValueError) as error:
        parser.exit(1, f"FORM-2 audit failed: {error}\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
