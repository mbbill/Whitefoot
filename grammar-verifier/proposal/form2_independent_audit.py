from __future__ import annotations

import argparse
import os
from pathlib import Path
import sys
import tempfile


MODULE_ROOT = Path(__file__).resolve().parent
if str(MODULE_ROOT) not in sys.path:
    sys.path.insert(0, str(MODULE_ROOT))

from form2_independent_inputs import IndependentInputError  # noqa: E402
from form2_independent_lex import IndependentLexError  # noqa: E402
from form2_independent_render import IndependentRenderError  # noqa: E402
from form2_independent_repairs import IndependentRepairError  # noqa: E402
from form2_independent_report import (  # noqa: E402
    IndependentReportError,
    build_independent_artifacts,
)
from form2_independent_syntax import (  # noqa: E402
    IndependentParseError,
    IndependentTreeError,
)
from form2_independent_topology import IndependentTopologyError  # noqa: E402


EVIDENCE_ROOT = MODULE_ROOT.parent / "evidence"
INDEPENDENT_ARTIFACT_NAMES = frozenset(
    {
        "form2-independent-evidence.sha256",
        "form2-independent-report.json",
    }
)


class IndependentAuditError(RuntimeError):
    pass


def _artifact_inventory(output: Path) -> frozenset[str]:
    if not output.exists():
        return frozenset()
    if not output.is_dir() or output.is_symlink():
        raise IndependentAuditError("independent artifact output is not a regular directory")
    try:
        return frozenset(
            path.name
            for path in output.iterdir()
            if path.name.startswith("form2-independent-")
        )
    except OSError as error:
        raise IndependentAuditError(f"cannot inspect independent artifacts: {error}") from error


def _reject_unexpected_artifacts(output: Path) -> None:
    unexpected = _artifact_inventory(output) - INDEPENDENT_ARTIFACT_NAMES
    if unexpected:
        raise IndependentAuditError(
            f"unexpected independent artifacts exist: {sorted(unexpected)!r}"
        )


def expected_artifacts() -> dict[str, bytes]:
    return build_independent_artifacts(compare_primary=True)


def check_artifacts(output: Path) -> None:
    inventory = _artifact_inventory(output)
    if inventory != INDEPENDENT_ARTIFACT_NAMES:
        raise IndependentAuditError(
            "independent artifact inventory is incomplete or contains superseded files"
        )
    for name, expected in expected_artifacts().items():
        path = output / name
        try:
            actual = path.read_bytes()
        except OSError as error:
            raise IndependentAuditError(f"cannot read independent artifact {path}: {error}") from error
        if actual != expected:
            raise IndependentAuditError(f"independent artifact is stale: {path}")


def _atomic_write(path: Path, raw: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary_name = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    temporary = Path(temporary_name)
    try:
        with os.fdopen(descriptor, "wb") as stream:
            stream.write(raw)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, path)
    finally:
        try:
            temporary.unlink()
        except FileNotFoundError:
            pass


def write_artifacts(output: Path) -> None:
    _reject_unexpected_artifacts(output)
    for name, raw in expected_artifacts().items():
        _atomic_write(output / name, raw)


def main() -> int:
    parser = argparse.ArgumentParser(description="independently verify structural FORM-2 evidence")
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true")
    mode.add_argument("--write", action="store_true")
    parser.add_argument("--output", type=Path, default=EVIDENCE_ROOT)
    arguments = parser.parse_args()
    try:
        if arguments.write:
            write_artifacts(arguments.output)
        else:
            check_artifacts(arguments.output)
    except (
        IndependentAuditError,
        IndependentInputError,
        IndependentLexError,
        IndependentParseError,
        IndependentRenderError,
        IndependentRepairError,
        IndependentReportError,
        IndependentTopologyError,
        IndependentTreeError,
        OSError,
        ValueError,
    ) as error:
        parser.exit(1, f"independent FORM-2 audit failed: {error}\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
