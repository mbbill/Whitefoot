#!/usr/bin/env python3
"""Generate and verify the exact-v0.9 normative source index.

This tool performs structural extraction only. It does not infer semantic
facets from prose and therefore cannot claim implementation completeness.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from bisect import bisect_right
from collections import Counter
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, Iterable, List, Optional, Sequence, Tuple


ROOT = Path(__file__).resolve().parents[2]
SPEC_PATH = ROOT / "spec" / "kernel-spec-v0.9.md"
OUTPUT_PATH = ROOT / "tests" / "spec-catalogs" / "v0.9" / "source.json"
SPEC_RELATIVE_PATH = "spec/kernel-spec-v0.9.md"
OUTPUT_RELATIVE_PATH = "tests/spec-catalogs/v0.9/source.json"
SPEC_SHA256 = "bdfb461d1901f610633c5cbcd2477d24df3c77ca90599b9580c8289e50b82b68"

EXPECTED_COUNTS = {
    "byte_exact_fences": 2,
    "core_grammar_productions": 58,
    "inline_grammar_productions": 4,
    "operation_name_occurrences": 84,
    "operation_names": 83,
    "operation_rows": 44,
    "report_rows": 4,
    "rules": 92,
    "sections": 17,
    "syntax_productions": 62,
}
EXPECTED_CORE_GRAMMAR_OWNERS = {
    "GRAM-2": 24,
    "GRAM-3": 5,
    "GRAM-4": 17,
    "GRAM-5": 12,
}
EXPECTED_INLINE_GRAMMAR_OWNERS = {
    "CONST-1": ("const",),
    "CONST-2": ("cvalue",),
    "EFF-1": ("effects", "effect"),
}
EXPECTED_FENCE_DIGESTS = {
    "PRE-1": "547eedebc7d9f262580c824045acf6b4643b10e42e388ce399479f901240c469",
    "EX-1": "490b202c156669e29030a4e6c2b2a86434da0aa7d33005f3db5079d830cbec71",
}
EXPECTED_DOTLESS_NAME_COUNTS = {
    "listed": 20,
    "listed_only": 0,
    "table": 51,
    "table_only": 31,
}

RULE_START = re.compile(rb"^\[([A-Z]+-[0-9]+[a-z]?)\]")
SECTION_START = re.compile(rb"^## ([0-9]+)\. (.+)$")
PRODUCTION_START = re.compile(rb"^([a-z_]+)\s*:=")
INLINE_PRODUCTION = re.compile(rb"`([a-z_]+)\s*:=([^`\r\n]*)`")
OPERATION_NAME = re.compile(r"[a-z_]+(?:\.[a-z]+)?")
DECLARED_DOTLESS_LIST = re.compile(
    rb"or a dotless IDENT \(`([^`\r\n]+)`\); both"
)


class CatalogError(ValueError):
    """The pinned specification does not have the required closed structure."""


def sha256(data: bytes) -> str:
    """Return a lowercase SHA-256 identity."""
    return hashlib.sha256(data).hexdigest()


def canonical_bytes(value: Dict[str, Any]) -> bytes:
    """Serialize one deterministic, reviewable JSON object."""
    return (
        json.dumps(value, ensure_ascii=True, indent=2, sort_keys=True) + "\n"
    ).encode("ascii")


def fail(message: str) -> None:
    """Raise one structural catalog failure."""
    raise CatalogError(message)


@dataclass(frozen=True)
class SourceText:
    """Exact LF-terminated specification bytes with stable source coordinates."""

    raw: bytes
    lines: Tuple[bytes, ...]
    line_offsets: Tuple[int, ...]

    @classmethod
    def parse(cls, specification: bytes) -> "SourceText":
        """Validate the byte envelope without normalizing it."""
        try:
            specification.decode("utf-8")
        except UnicodeDecodeError as error:
            fail(f"specification is not UTF-8: {error}")
        if b"\r" in specification:
            fail("specification contains CR bytes")
        if not specification.endswith(b"\n"):
            fail("specification must end in LF")
        lines = tuple(specification.splitlines(keepends=True))
        if any(not line.endswith(b"\n") for line in lines):
            fail("every specification line must end in LF")
        offsets = [0]
        for line in lines:
            offsets.append(offsets[-1] + len(line))
        return cls(specification, lines, tuple(offsets))

    def display(self, line_number: int) -> str:
        """Decode one line without its already-validated terminating LF."""
        return self.lines[line_number - 1][:-1].decode("utf-8")

    def line_span(self, line_start: int, line_end: int) -> Dict[str, Any]:
        """Describe an inclusive one-based line interval exactly."""
        if not (1 <= line_start <= line_end <= len(self.lines)):
            fail(f"invalid source line span {line_start}-{line_end}")
        byte_start = self.line_offsets[line_start - 1]
        byte_end = self.line_offsets[line_end]
        return self.byte_span(byte_start, byte_end)

    def byte_span(self, byte_start: int, byte_end: int) -> Dict[str, Any]:
        """Describe a zero-based half-open byte interval without normalization."""
        if not (0 <= byte_start < byte_end <= len(self.raw)):
            fail(f"invalid source byte span {byte_start}-{byte_end}")
        line_start = bisect_right(self.line_offsets, byte_start)
        line_end = bisect_right(self.line_offsets, byte_end - 1)
        return {
            "byte_end": byte_end,
            "byte_start": byte_start,
            "line_end": line_end,
            "line_start": line_start,
            "sha256": sha256(self.raw[byte_start:byte_end]),
        }

    def rule_bytes(self, interval: Tuple[int, int]) -> Tuple[int, bytes]:
        """Return the absolute byte start and exact bytes for a rule interval."""
        line_start, line_end = interval
        byte_start = self.line_offsets[line_start - 1]
        byte_end = self.line_offsets[line_end]
        return byte_start, self.raw[byte_start:byte_end]


def trim_blank_end(source: SourceText, start: int, end: int) -> int:
    """Remove blank separator lines from an inclusive interval's end."""
    while end >= start and source.lines[end - 1] == b"\n":
        end -= 1
    return end


def section_for_line(sections: Sequence[Tuple[int, str]], line_number: int) -> str:
    """Return the nearest preceding numbered section title."""
    preceding = [title for start, title in sections if start < line_number]
    if not preceding:
        fail(f"rule at line {line_number} has no numbered section")
    return preceding[-1]


def extract_rules(
    source: SourceText,
) -> Tuple[List[Dict[str, Any]], Dict[str, Tuple[int, int]]]:
    """Extract every line-start rule and its exact source interval."""
    sections: List[Tuple[int, str]] = []
    starts: List[Tuple[str, int]] = []
    heading_lines = set()
    for line_number, line in enumerate(source.lines, 1):
        section = SECTION_START.match(line[:-1])
        if section is not None:
            heading_lines.add(line_number)
            sections.append(
                (
                    line_number,
                    f"{section.group(1).decode('ascii')}. "
                    f"{section.group(2).decode('utf-8')}",
                )
            )
        rule = RULE_START.match(line)
        if rule is not None:
            starts.append((rule.group(1).decode("ascii"), line_number))

    section_numbers = [
        int(title.split(".", 1)[0]) for _, title in sections
    ]
    if section_numbers != list(range(1, EXPECTED_COUNTS["sections"] + 1)):
        fail(f"expected contiguous sections 1-17, found {section_numbers}")

    identifiers = [identifier for identifier, _ in starts]
    duplicates = sorted(
        identifier for identifier, count in Counter(identifiers).items() if count != 1
    )
    if duplicates:
        fail(f"duplicate rule definitions: {duplicates}")
    if len(starts) != EXPECTED_COUNTS["rules"]:
        fail(f"expected 92 rules, found {len(starts)}")

    records: List[Dict[str, Any]] = []
    intervals: Dict[str, Tuple[int, int]] = {}
    boundary_lines = sorted(heading_lines | {line_number for _, line_number in starts})
    for identifier, start in starts:
        following = [line_number for line_number in boundary_lines if line_number > start]
        end = (following[0] - 1) if following else len(source.lines)
        end = trim_blank_end(source, start, end)
        if end < start:
            fail(f"empty rule body for {identifier}")
        intervals[identifier] = (start, end)
        records.append(
            {
                "id": f"rule:{identifier}",
                "rule_id": identifier,
                "section": section_for_line(sections, start),
                "source": source.line_span(start, end),
            }
        )
    return records, intervals


def fenced_intervals(
    source: SourceText, start: int, end: int
) -> List[Tuple[int, int]]:
    """Return interior intervals for complete plain Markdown fences in one rule."""
    markers = [
        line_number
        for line_number in range(start, end + 1)
        if source.lines[line_number - 1] == b"```\n"
    ]
    if len(markers) % 2 != 0:
        fail(f"unclosed Markdown fence in lines {start}-{end}")
    return [
        (markers[index] + 1, markers[index + 1] - 1)
        for index in range(0, len(markers), 2)
    ]


def extract_core_grammar(
    source: SourceText, intervals: Dict[str, Tuple[int, int]]
) -> List[Dict[str, Any]]:
    """Extract the four normative fenced grammar blocks."""
    records: List[Dict[str, Any]] = []
    for owner, expected_count in EXPECTED_CORE_GRAMMAR_OWNERS.items():
        if owner not in intervals:
            fail(f"missing grammar owner {owner}")
        fences = fenced_intervals(source, *intervals[owner])
        if len(fences) != 1:
            fail(f"{owner} must contain exactly one grammar fence")
        fence_start, fence_end = fences[0]
        starts: List[Tuple[str, int]] = []
        for line_number in range(fence_start, fence_end + 1):
            line = source.lines[line_number - 1]
            match = PRODUCTION_START.match(line)
            if match is not None:
                starts.append((match.group(1).decode("ascii"), line_number))
            elif not line[:1].isspace() or not line.strip():
                fail(f"orphan grammar continuation at line {line_number}")
        if len(starts) != expected_count:
            fail(f"{owner} expected {expected_count} productions, found {len(starts)}")
        if not starts or starts[0][1] != fence_start:
            fail(f"{owner} grammar fence does not begin with a production")
        for owner_ordinal, (lhs, line_start) in enumerate(starts, 1):
            index = owner_ordinal - 1
            line_end = (
                starts[index + 1][1] - 1 if index + 1 < len(starts) else fence_end
            )
            line_end = trim_blank_end(source, line_start, line_end)
            records.append(
                {
                    "id": f"production:{owner}:{lhs}",
                    "lhs": lhs,
                    "owner_ordinal": owner_ordinal,
                    "owner_rule": owner,
                    "source": source.line_span(line_start, line_end),
                    "source_form": "fenced-core",
                }
            )
    return records


def extract_inline_grammar(
    source: SourceText, intervals: Dict[str, Tuple[int, int]]
) -> List[Dict[str, Any]]:
    """Extract grammar definitions embedded normatively in reviewed rule prose."""
    records: List[Dict[str, Any]] = []
    for owner, expected_names in EXPECTED_INLINE_GRAMMAR_OWNERS.items():
        if owner not in intervals:
            fail(f"missing inline grammar owner {owner}")
        rule_start, rule_bytes = source.rule_bytes(intervals[owner])
        matches = list(INLINE_PRODUCTION.finditer(rule_bytes))
        names = tuple(match.group(1).decode("ascii") for match in matches)
        if names != expected_names:
            fail(f"{owner} inline productions are {names}, expected {expected_names}")
        for owner_ordinal, match in enumerate(matches, 1):
            lhs = match.group(1).decode("ascii")
            byte_start = rule_start + match.start(1)
            byte_end = rule_start + match.end(0) - 1
            records.append(
                {
                    "id": f"production:{owner}:{lhs}",
                    "lhs": lhs,
                    "owner_ordinal": owner_ordinal,
                    "owner_rule": owner,
                    "source": source.byte_span(byte_start, byte_end),
                    "source_form": "inline-rule",
                }
            )
    return records


def extract_syntax_productions(
    source: SourceText, intervals: Dict[str, Tuple[int, int]]
) -> List[Dict[str, Any]]:
    """Extract every closed syntax production without interpreting semantics."""
    core = extract_core_grammar(source, intervals)
    inline = extract_inline_grammar(source, intervals)
    records = core + inline
    identifiers = [record["id"] for record in records]
    if len(set(identifiers)) != len(identifiers):
        fail("syntax production identifiers are not globally unique")
    if len(core) != EXPECTED_COUNTS["core_grammar_productions"]:
        fail(f"expected 58 core grammar productions, found {len(core)}")
    if len(inline) != EXPECTED_COUNTS["inline_grammar_productions"]:
        fail(f"expected 4 inline grammar productions, found {len(inline)}")
    if len(records) != EXPECTED_COUNTS["syntax_productions"]:
        fail(f"expected 62 syntax productions, found {len(records)}")
    if source.raw.count(b":=") != len(records):
        fail("the specification contains an unaccounted ':=' definition")
    return records


def split_table_row(line: bytes, expected_cells: int, context: str) -> List[str]:
    """Split one simple Markdown table row with a closed cell count."""
    text = line[:-1].decode("utf-8")
    if not text.startswith("|") or not text.endswith("|"):
        fail(f"{context} is not a closed Markdown row")
    cells = [cell.strip() for cell in text[1:-1].split("|")]
    if len(cells) != expected_cells:
        fail(f"{context} expected {expected_cells} cells, found {len(cells)}")
    return cells


def table_rows(
    source: SourceText,
    interval: Tuple[int, int],
    header: str,
    separator: str,
    expected_cells: int,
) -> Tuple[int, List[Tuple[int, List[str]]]]:
    """Find one exact Markdown table and return its consecutive data rows."""
    start, end = interval
    header_lines = [
        line_number
        for line_number in range(start, end + 1)
        if source.display(line_number) == header
    ]
    if len(header_lines) != 1:
        fail(f"expected one table header {header!r}, found {len(header_lines)}")
    header_line = header_lines[0]
    if header_line + 1 > end or source.display(header_line + 1) != separator:
        fail(f"table separator drifted after line {header_line}")
    rows: List[Tuple[int, List[str]]] = []
    line_number = header_line + 2
    while line_number <= end and source.lines[line_number - 1].startswith(b"|"):
        rows.append(
            (
                line_number,
                split_table_row(
                    source.lines[line_number - 1],
                    expected_cells,
                    f"table line {line_number}",
                ),
            )
        )
        line_number += 1
    if not rows:
        fail(f"table at line {header_line} has no data rows")
    return header_line, rows


def operation_names(cell: str, line_number: int) -> List[str]:
    """Extract operation spellings from one exact OP-1 first cell."""
    names = re.findall(r"`([^`]+)`", cell)
    if not names or "".join(f"`{name}`" for name in names) != cell.replace(" ", ""):
        fail(f"OP-1 operation cell at line {line_number} has unexpected syntax")
    if any(OPERATION_NAME.fullmatch(name) is None for name in names):
        fail(f"OP-1 operation cell at line {line_number} has an invalid name")
    return names


def unique_in_order(values: Iterable[str]) -> List[str]:
    """Deduplicate a source-order sequence without hash-order dependence."""
    result: List[str] = []
    seen = set()
    for value in values:
        if value not in seen:
            seen.add(value)
            result.append(value)
    return result


def extract_operation_table(
    source: SourceText, intervals: Dict[str, Tuple[int, int]]
) -> Tuple[List[Dict[str, Any]], List[str], Dict[str, Any]]:
    """Extract the closed OP-1 operation table and its ordered spellings."""
    header_line, rows = table_rows(
        source,
        intervals["OP-1"],
        "| op | domain | signature | effects |",
        "|---|---|---|---|",
        4,
    )
    if len(rows) != EXPECTED_COUNTS["operation_rows"]:
        fail(f"expected 44 OP-1 rows, found {len(rows)}")
    records: List[Dict[str, Any]] = []
    occurrences: List[str] = []
    for ordinal, (line_number, cells) in enumerate(rows, 1):
        names = operation_names(cells[0], line_number)
        occurrences.extend(names)
        row_source = source.line_span(line_number, line_number)
        records.append(
            {
                "domain": cells[1],
                "effects": cells[3],
                "id": f"operation-row:OP-1:{ordinal:03d}",
                "names": names,
                "ordinal": ordinal,
                "owner_rule": "OP-1",
                "signature": cells[2],
                "source": row_source,
            }
        )
    if len(occurrences) != EXPECTED_COUNTS["operation_name_occurrences"]:
        fail(f"expected 84 operation-name occurrences, found {len(occurrences)}")
    if len(unique_in_order(occurrences)) != EXPECTED_COUNTS["operation_names"]:
        fail("expected 83 unique operation names")
    table_source = source.line_span(header_line, rows[-1][0])
    return records, occurrences, table_source


def extract_operation_name_sets(
    source: SourceText,
    intervals: Dict[str, Tuple[int, int]],
    occurrences: Sequence[str],
    table_source: Dict[str, Any],
) -> Dict[str, Any]:
    """Extract both OP-1 dotless-name sets and their mechanical differences."""
    table_names = unique_in_order(
        name for name in occurrences if "." not in name
    )
    rule_start, rule_bytes = source.rule_bytes(intervals["OP-1"])
    matches = list(DECLARED_DOTLESS_LIST.finditer(rule_bytes))
    if len(matches) != 1:
        fail(f"expected one OP-1 declared dotless list, found {len(matches)}")
    match = matches[0]
    listed_names = match.group(1).decode("ascii").split(" ")
    if any(OPERATION_NAME.fullmatch(name) is None or "." in name for name in listed_names):
        fail("OP-1 listed dotless names contain an invalid identifier")
    if len(set(listed_names)) != len(listed_names):
        fail("OP-1 listed dotless names contain a duplicate")
    table_only = sorted(name for name in table_names if name not in listed_names)
    listed_only = sorted(name for name in listed_names if name not in table_names)
    actual_counts = {
        "listed": len(listed_names),
        "listed_only": len(listed_only),
        "table": len(table_names),
        "table_only": len(table_only),
    }
    if actual_counts != EXPECTED_DOTLESS_NAME_COUNTS:
        fail(f"OP-1 dotless-name discrepancy drifted: {actual_counts}")
    byte_start = rule_start + match.start(1)
    byte_end = rule_start + match.end(1)
    return {
        "listed_dotless_identifiers": listed_names,
        "listed_only_identifiers": listed_only,
        "listed_source": source.byte_span(byte_start, byte_end),
        "table_dotless_identifiers": table_names,
        "table_only_identifiers": table_only,
        "table_source": table_source,
    }


def extract_report_rows(
    source: SourceText, intervals: Dict[str, Tuple[int, int]]
) -> List[Dict[str, Any]]:
    """Extract the closed DIAG-3 report-family table."""
    _, rows = table_rows(
        source,
        intervals["DIAG-3"],
        "| report | fields (all required) |",
        "|---|---|",
        2,
    )
    if len(rows) != EXPECTED_COUNTS["report_rows"]:
        fail(f"expected 4 DIAG-3 rows, found {len(rows)}")
    records: List[Dict[str, Any]] = []
    names: List[str] = []
    for line_number, cells in rows:
        report = cells[0]
        if re.fullmatch(r"[a-z][a-z-]*", report) is None:
            fail(f"invalid report identifier {report!r}")
        names.append(report)
        records.append(
            {
                "fields": cells[1],
                "id": f"report:DIAG-3:{report}",
                "owner_rule": "DIAG-3",
                "report": report,
                "source": source.line_span(line_number, line_number),
            }
        )
    if len(set(names)) != len(names):
        fail("DIAG-3 report names are not unique")
    return records


def extract_byte_exact_fences(
    source: SourceText, intervals: Dict[str, Tuple[int, int]]
) -> List[Dict[str, Any]]:
    """Extract exact prelude and worked-example code-fence identities."""
    if sum(line == b"```\n" for line in source.lines) != 12:
        fail("the specification must contain exactly six plain code fences")
    records: List[Dict[str, Any]] = []
    for owner in ("PRE-1", "EX-1"):
        fences = fenced_intervals(source, *intervals[owner])
        if len(fences) != 1:
            fail(f"{owner} must contain exactly one byte-exact code fence")
        line_start, line_end = fences[0]
        record_source = source.line_span(line_start, line_end)
        expected_digest = EXPECTED_FENCE_DIGESTS[owner]
        if record_source["sha256"] != expected_digest:
            fail(f"{owner} byte-exact payload digest drifted")
        records.append(
            {
                "byte_length": record_source["byte_end"] - record_source["byte_start"],
                "id": f"fence:{owner}",
                "owner_rule": owner,
                "source": record_source,
            }
        )
    return records


def extract_source_index(
    specification: bytes, expected_spec_sha256: Optional[str] = SPEC_SHA256
) -> Dict[str, Any]:
    """Build the complete mechanical source index for exact v0.9."""
    actual_spec_sha256 = sha256(specification)
    if expected_spec_sha256 is not None and actual_spec_sha256 != expected_spec_sha256:
        fail(
            f"specification hash is {actual_spec_sha256}, "
            f"expected {expected_spec_sha256}"
        )
    source = SourceText.parse(specification)
    rules, intervals = extract_rules(source)
    syntax_productions = extract_syntax_productions(source, intervals)
    operation_rows, operation_occurrences, operation_table_source = (
        extract_operation_table(source, intervals)
    )
    operation_name_sets = extract_operation_name_sets(
        source, intervals, operation_occurrences, operation_table_source
    )
    report_rows = extract_report_rows(source, intervals)
    byte_fences = extract_byte_exact_fences(source, intervals)
    counts = {
        "byte_exact_fences": len(byte_fences),
        "core_grammar_productions": sum(
            item["source_form"] == "fenced-core" for item in syntax_productions
        ),
        "inline_grammar_productions": sum(
            item["source_form"] == "inline-rule" for item in syntax_productions
        ),
        "operation_name_occurrences": len(operation_occurrences),
        "operation_names": len(unique_in_order(operation_occurrences)),
        "operation_rows": len(operation_rows),
        "report_rows": len(report_rows),
        "rules": len(rules),
        "sections": len(
            [line for line in source.lines if SECTION_START.match(line[:-1]) is not None]
        ),
        "syntax_productions": len(syntax_productions),
    }
    if counts != EXPECTED_COUNTS:
        fail(f"source-index counts drifted: {counts}")
    return {
        "byte_exact_fences": byte_fences,
        "counts": counts,
        "generated_by": "tools/facet_catalog.py",
        "kind": "whitefoot-normative-source-index",
        "operation_name_sets": operation_name_sets,
        "operation_rows": operation_rows,
        "report_rows": report_rows,
        "rules": rules,
        "schema": 1,
        "scope": (
            "Structural source accounting only; semantic facet decomposition "
            "and implementation capability are separate artifacts."
        ),
        "specification": {
            "byte_length": len(specification),
            "path": SPEC_RELATIVE_PATH,
            "sha256": actual_spec_sha256,
            "version": "0.9",
        },
        "syntax_productions": syntax_productions,
    }


def generated_bytes() -> bytes:
    """Return the canonical checked-in source-index bytes."""
    return canonical_bytes(extract_source_index(SPEC_PATH.read_bytes()))


def check() -> None:
    """Fail when the checked-in source index is missing or stale."""
    expected = generated_bytes()
    try:
        actual = OUTPUT_PATH.read_bytes()
    except OSError as error:
        fail(f"cannot read {OUTPUT_RELATIVE_PATH}: {error}")
    if actual != expected:
        fail(
            f"{OUTPUT_RELATIVE_PATH} is stale; regenerate it only from the "
            "owner-approved exact specification"
        )
    print(
        "facet source index: exact v0.9 structure verified "
        "(92 rules, 62 syntax productions [58 fenced + 4 inline], "
        "44 OP-1 rows, 4 DIAG-3 rows, 2 byte-exact fences; "
        "OP-1 name-set difference exposed; semantic facets separate)"
    )


def write() -> None:
    """Regenerate the mechanical source index from exact approved v0.9 bytes."""
    OUTPUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT_PATH.write_bytes(generated_bytes())
    print(f"wrote {OUTPUT_RELATIVE_PATH}")


def parse_arguments(arguments: Optional[Sequence[str]] = None) -> argparse.Namespace:
    """Parse the intentionally small command surface."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("check", "write"))
    return parser.parse_args(arguments)


def main(arguments: Optional[Sequence[str]] = None) -> None:
    """Run one source-index command with a direct failure diagnostic."""
    options = parse_arguments(arguments)
    try:
        if options.command == "check":
            check()
        else:
            write()
    except (CatalogError, OSError, UnicodeError, json.JSONDecodeError) as error:
        print(f"facet source index: {error}", file=sys.stderr)
        raise SystemExit(1) from None


if __name__ == "__main__":
    main()
