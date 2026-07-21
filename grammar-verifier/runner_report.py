"""Closed envelope parsing for raw grammar-verifier reports."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Mapping, Optional

from runner_common_report import validate_common
from runner_common_wire import KIND
from runner_inputs import Inputs, fail
from runner_oracle_report import validate_oracle
from runner_static_report import validate_static


@dataclass(frozen=True)
class RawReport:
    engine: str
    raw: bytes
    common: Optional[bytes]
    specific: Optional[bytes]
    failure: Optional[tuple[str, str]]
    domains: Mapping[tuple[bytes, bytes], tuple[bytes, ...]]
    observations: Mapping[str, object]


def parse_report(raw: bytes, expected_engine: str, inputs: Inputs) -> RawReport:
    if expected_engine not in ("static", "oracle"):
        fail("report_engine", "the runner requested an unknown engine report")
    if len(raw) > inputs.limits["max_engine_output_bytes"]:
        fail("report_size", f"{expected_engine} report exceeds its declared byte bound")
    if not raw.endswith(b"\n") or b"\r" in raw:
        fail("report_layout", f"{expected_engine} report is not canonical LF text")
    try:
        raw.decode("ascii")
    except UnicodeDecodeError:
        fail("report_encoding", f"{expected_engine} report is not strict ASCII")
    lines = raw[:-1].split(b"\n")
    if any(len(line) > inputs.limits["max_line_bytes"] for line in lines):
        fail("report_line", f"{expected_engine} report contains an oversized line")
    engine_line = b"ENGINE\t" + expected_engine.encode("ascii")
    if len(lines) < 4 or lines[0] != b"WFGRREPORT1" or lines[1] != engine_line or lines[-1] != b"END":
        fail("report_envelope", f"{expected_engine} report has an invalid envelope")
    if lines[2].startswith(b"FAIL\t"):
        if len(lines) != 4:
            fail("report_failure", f"{expected_engine} failure report has extra records")
        fields = lines[2].split(b"\t")
        if (
            len(fields) != 3
            or fields[1] not in (b"input", b"extraction", b"resource", b"internal")
            or not KIND.fullmatch(fields[2])
        ):
            fail("report_failure", f"{expected_engine} failure report is malformed")
        return RawReport(
            expected_engine,
            raw,
            None,
            None,
            (fields[1].decode("ascii"), fields[2].decode("ascii")),
            {},
            {},
        )
    section = b"STATIC" if expected_engine == "static" else b"ORACLE"
    markers = (b"COMMON-BEGIN", b"COMMON-END", section + b"-BEGIN", section + b"-END")
    if any(lines.count(marker) != 1 for marker in markers):
        fail("report_sections", f"{expected_engine} report has missing or duplicate section markers")
    common_begin, common_end, specific_begin, specific_end = (lines.index(marker) for marker in markers)
    if (
        common_begin != 2
        or specific_begin != common_end + 1
        or specific_end != len(lines) - 2
        or common_end <= common_begin
    ):
        fail("report_sections", f"{expected_engine} report sections are reordered")
    common_lines = lines[common_begin + 1 : common_end]
    specific_lines = lines[specific_begin + 1 : specific_end]
    schemas = validate_common(common_lines, inputs)
    domains, observations = (
        validate_static(specific_lines, inputs, schemas)
        if expected_engine == "static"
        else validate_oracle(specific_lines, inputs, schemas)
    )
    common = b"\n".join(common_lines) + b"\n"
    specific = b"\n".join(specific_lines) + (b"\n" if specific_lines else b"")
    return RawReport(expected_engine, raw, common, specific, None, domains, observations)
