#!/usr/bin/env python3
"""Generate the non-authoritative v0.10 review candidate from exact v0.9."""

from __future__ import annotations

import argparse
import hashlib
import os
from dataclasses import dataclass
from pathlib import Path
import sys
import tempfile


BASE_SHA256 = "bdfb461d1901f610633c5cbcd2477d24df3c77ca90599b9580c8289e50b82b68"
EXPECTED_CANDIDATE_BYTES = 118_314
EXPECTED_CANDIDATE_SHA256 = "71073e25219455896250e15e13d1ffdbfc443c87a9b28cb9906d73a020dc33e9"
HERE = Path(__file__).resolve().parent
REPOSITORY = HERE.parents[1]
DEFAULT_BASE = REPOSITORY / "spec" / "kernel-spec-v0.9.md"
DEFAULT_PROPOSAL = HERE / "PROPOSAL.md"
DEFAULT_OUTPUT = HERE / "kernel-spec-v0.10-candidate.md"


class CandidateError(RuntimeError):
    """The pinned base or proposal markers do not satisfy the generator contract."""


@dataclass(frozen=True)
class AppliedEdit:
    name: str
    old: str
    new: str


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def candidate_block(proposal: str, name: str) -> str:
    begin = f"<!-- CANDIDATE:{name}:BEGIN -->"
    end = f"<!-- CANDIDATE:{name}:END -->"
    if proposal.count(begin) != 1 or proposal.count(end) != 1:
        raise CandidateError(f"candidate block {name!r} must have one begin and one end")
    start = proposal.index(begin) + len(begin)
    stop = proposal.index(end, start)
    value = proposal[start:stop].strip("\n")
    if not value:
        raise CandidateError(f"candidate block {name!r} is empty")
    return value + "\n\n"


def replace_unique(text: str, old: str, new: str, name: str) -> tuple[str, AppliedEdit]:
    count = text.count(old)
    if count != 1:
        raise CandidateError(f"edit {name!r} expected one anchor, found {count}")
    return text.replace(old, new, 1), AppliedEdit(name, old, new)


def delimited_old(text: str, start: str, end: str, name: str) -> str:
    if text.count(start) != 1 or text.count(end) != 1:
        raise CandidateError(f"edit {name!r} requires unique delimiters")
    first = text.index(start)
    last = text.index(end, first)
    if last <= first:
        raise CandidateError(f"edit {name!r} has reversed delimiters")
    return text[first:last]


def build_candidate(base_bytes: bytes, proposal_bytes: bytes) -> tuple[bytes, tuple[AppliedEdit, ...]]:
    actual = sha256(base_bytes)
    if actual != BASE_SHA256:
        raise CandidateError(f"base SHA-256 mismatch: expected {BASE_SHA256}, found {actual}")
    try:
        text = base_bytes.decode("utf-8")
        proposal = proposal_bytes.decode("utf-8")
    except UnicodeDecodeError as error:
        raise CandidateError("base and proposal must be UTF-8") from error

    edits: list[AppliedEdit] = []

    header_old = delimited_old(
        text,
        "# Kernel Specification v0.9\n",
        "Prior: DRAFT v0.8",
        "HEADER",
    )
    text, edit = replace_unique(text, header_old, candidate_block(proposal, "HEADER"), "HEADER")
    edits.append(edit)

    type_old = delimited_old(
        text,
        "[TYPE-6] Name-binding visibility is this total table:",
        "[TYPE-7] Reading through a reference is explicit.",
        "TYPE_6",
    )
    text, edit = replace_unique(text, type_old, candidate_block(proposal, "TYPE_6"), "TYPE_6")
    edits.append(edit)

    op_old = delimited_old(
        text,
        "An operation name is an OPNAME",
        "[OP-2] There are no wrap modes",
        "OP_1_NAMES",
    )
    text, edit = replace_unique(
        text,
        op_old,
        candidate_block(proposal, "OP_1_NAMES"),
        "OP_1_NAMES",
    )
    edits.append(edit)

    diag_old = delimited_old(
        text,
        "An input-envelope failure, resource failure",
        "[DIAG-2] Accepted programs elaborate",
        "DIAG_1_TAIL",
    )
    text, edit = replace_unique(
        text,
        diag_old,
        candidate_block(proposal, "DIAG_1_TAIL"),
        "DIAG_1_TAIL",
    )
    edits.append(edit)

    version_edits = (
        ("Source-law discharge in v0.9 requires", "Source-law discharge in v0.10 requires"),
        ("The v0.9 law table is closed:", "The v0.10 law table is closed:"),
        ("unavailable for source discharge in v0.9;", "unavailable for source discharge in v0.10;"),
    )
    for index, (old, new) in enumerate(version_edits, start=1):
        text, edit = replace_unique(text, old, new, f"FN4_VERSION_{index}")
        edits.append(edit)

    candidate = text.encode("utf-8")
    actual_hash = sha256(candidate)
    if len(candidate) != EXPECTED_CANDIDATE_BYTES or actual_hash != EXPECTED_CANDIDATE_SHA256:
        raise CandidateError(
            "candidate identity mismatch: "
            f"expected {EXPECTED_CANDIDATE_BYTES} bytes and {EXPECTED_CANDIDATE_SHA256}, "
            f"found {len(candidate)} bytes and {actual_hash}"
        )
    return candidate, tuple(edits)


def reverse_allowed_edits(candidate: bytes, edits: tuple[AppliedEdit, ...]) -> bytes:
    """Test helper: remove exactly the generator's declared edit surface."""
    text = candidate.decode("utf-8")
    for edit in reversed(edits):
        if text.count(edit.new) != 1:
            raise CandidateError(f"generated edit {edit.name!r} is not uniquely reversible")
        text = text.replace(edit.new, edit.old, 1)
    return text.encode("utf-8")


def write_atomic(path: Path, data: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    try:
        with os.fdopen(descriptor, "wb") as stream:
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, path)
    except BaseException:
        try:
            os.unlink(temporary)
        except FileNotFoundError:
            pass
        raise


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", type=Path, default=DEFAULT_BASE)
    parser.add_argument("--proposal", type=Path, default=DEFAULT_PROPOSAL)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--check", action="store_true", help="verify output instead of writing it")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    try:
        candidate, _ = build_candidate(args.base.read_bytes(), args.proposal.read_bytes())
        if args.check:
            if not args.output.exists() or args.output.read_bytes() != candidate:
                raise CandidateError(f"generated candidate is stale or absent: {args.output}")
        else:
            write_atomic(args.output, candidate)
    except (CandidateError, OSError) as error:
        print(f"candidate generation failed: {error}", file=sys.stderr)
        return 1
    print(f"{sha256(candidate)}  {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
