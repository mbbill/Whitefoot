#!/usr/bin/env python3
"""Fixed stdin/stdout entry point for the independent grammar Oracle."""

from __future__ import annotations

import os
from pathlib import Path
import sys


ORACLE_ROOT = Path(__file__).resolve().parent
sys.path.insert(0, str(ORACLE_ROOT))

from core import Failure  # noqa: E402
from extract import extract_document  # noqa: E402
from ingress import read_frame  # noqa: E402
from report import analyze, render_failure, render_success  # noqa: E402


def _run() -> bytes:
    inputs = read_frame(sys.stdin.buffer)
    current = extract_document("current", inputs.current.data, inputs.limits)
    proposal = extract_document("proposal", inputs.proposal.data, inputs.limits)
    evidence = analyze(inputs, current, proposal)
    return render_success(inputs, current, proposal, evidence)


def _write_all(raw: bytes) -> int:
    cursor = 0
    try:
        while cursor < len(raw):
            written = os.write(1, raw[cursor:])
            if written <= 0:
                return 1
            cursor += written
    except OSError:
        return 1
    return 0


def main() -> int:
    try:
        raw = _run()
    except Failure as error:
        raw = render_failure(error.family, error.code)
    except (MemoryError, RecursionError):
        raw = render_failure("resource", "host_capacity")
    except Exception:
        raw = render_failure("internal", "unexpected")
    return _write_all(raw)


if __name__ == "__main__":
    raise SystemExit(main())
