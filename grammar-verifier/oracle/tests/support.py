"""Shared fixture construction for the isolated Oracle tests."""

from __future__ import annotations

from functools import lru_cache
from io import BytesIO
from pathlib import Path
import struct

from core import Inputs, Limits
from extract import GrammarDocument, extract_document
from ingress import MAGIC, read_frame
from report import Evidence, analyze


ORACLE_ROOT = Path(__file__).resolve().parents[1]
VERIFIER_ROOT = ORACLE_ROOT.parent
REPOSITORY_ROOT = VERIFIER_ROOT.parent


def fixture_sections() -> tuple[bytes, bytes, bytes, bytes, bytes]:
    return (
        (VERIFIER_ROOT / "limits.txt").read_bytes(),
        (REPOSITORY_ROOT / "spec" / "kernel-spec-v0.8.md").read_bytes(),
        (VERIFIER_ROOT / "proposal" / "kernel-spec-successor-candidate.md").read_bytes(),
        (VERIFIER_ROOT / "cases.txt").read_bytes(),
        (VERIFIER_ROOT / "domains.txt").read_bytes(),
    )


def frame_bytes(
    sections: tuple[bytes, bytes, bytes, bytes, bytes] | None = None,
) -> bytes:
    bound = fixture_sections() if sections is None else sections
    return MAGIC + struct.pack(">QQQQQ", *(len(item) for item in bound)) + b"".join(bound)


@lru_cache(maxsize=1)
def fixture_inputs() -> Inputs:
    return read_frame(BytesIO(frame_bytes()))


@lru_cache(maxsize=1)
def fixture_grammars() -> tuple[GrammarDocument, GrammarDocument]:
    inputs = fixture_inputs()
    return (
        extract_document("current", inputs.current.data, inputs.limits),
        extract_document("proposal", inputs.proposal.data, inputs.limits),
    )


@lru_cache(maxsize=1)
def fixture_evidence() -> Evidence:
    inputs = fixture_inputs()
    current, proposal = fixture_grammars()
    return analyze(inputs, current, proposal)


def limits_with(**changes: int) -> Limits:
    values = dict(fixture_inputs().limits.values)
    values.update(changes)
    return Limits(values)
