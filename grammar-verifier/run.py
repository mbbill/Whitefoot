#!/usr/bin/env python3
"""Run and bind the two independent grammar-evidence engines."""

from __future__ import annotations

import os
from pathlib import Path
import sys
import tempfile


ROOT = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT))

from runner_inputs import (
    RunnerError,
    SourceRevision,
    load_inputs,
    make_frame,
    oracle_sources,
    runner_sources,
    source_revision,
    static_sources,
)
from runner_process import ProcessLimits, build_static, run_child
from runner_package import write_evidence
from runner_report import parse_report
from installed_policy import validate_review_packet


INSTALLED_OUTPUT = ROOT / "installed-v0.9-evidence"


def source_revisions(root: Path) -> dict[str, SourceRevision]:
    """Rediscover and bind every runner and engine source file."""

    static_root = root / "static-auditor"
    oracle_root = root / "oracle"
    observed_sources = {
        "runner": (root / "RUNNER_SOURCES", runner_sources(root)),
        "static": (static_root / "SOURCES", static_sources(static_root)),
        "oracle": (oracle_root / "SOURCES", oracle_sources(oracle_root)),
    }
    return {
        name: source_revision(manifest, expected)
        for name, (manifest, expected) in observed_sources.items()
    }


def run_repository(
    output: Path = INSTALLED_OUTPUT,
    *,
    installed: bool = True,
) -> bytes:
    validate_review_packet(ROOT)
    inputs = load_inputs(ROOT, installed=installed)
    frame = make_frame(inputs)
    static_root = ROOT / "static-auditor"
    oracle_root = ROOT / "oracle"
    before = source_revisions(ROOT)
    limits = ProcessLimits(
        output_bytes=inputs.limits["max_engine_output_bytes"],
        wall_seconds=float(inputs.limits["wall_timeout_seconds"]),
        cpu_seconds=inputs.limits["cpu_timeout_seconds"],
    )
    with tempfile.TemporaryDirectory(prefix="whitefoot-static-target-") as target:
        static_artifact = build_static(static_root, Path(target))
        static_raw = run_child(
            "static",
            (str(static_artifact),),
            static_root,
            frame,
            limits,
        )
        oracle_raw = run_child(
            "oracle",
            (sys.executable, "-I", "-S", "-B", str(oracle_root / "main.py")),
            oracle_root,
            frame,
            limits,
        )
    after = source_revisions(ROOT)
    if after != before:
        raise RunnerError(
            "source_changed",
            "a bound runner or engine source tree changed during execution",
        )
    reports = (
        parse_report(static_raw, "static", inputs),
        parse_report(oracle_raw, "oracle", inputs),
    )
    return write_evidence(output, ROOT, inputs, before, reports)


def main(arguments: tuple[str, ...] | None = None) -> int:
    arguments = tuple(sys.argv[1:]) if arguments is None else arguments
    if sys.platform == "win32" or os.name != "posix":
        print(
            "grammar verifier: unsupported_host: POSIX process limits are required",
            file=sys.stderr,
        )
        return 1
    if arguments not in ((), ("--review-packet-only",)):
        print("usage: run.py [--review-packet-only]", file=sys.stderr)
        return 2
    try:
        if arguments == ("--review-packet-only",):
            validate_review_packet(ROOT)
            print("historical grammar review packet validated")
            return 0
        run_repository()
    except RunnerError as error:
        print(f"grammar verifier: {error.code}: {error}", file=sys.stderr)
        return 1
    print(
        "installed v0.9 grammar evidence reproduced; "
        "historical review packet preserved; no compiler-authority claim"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
