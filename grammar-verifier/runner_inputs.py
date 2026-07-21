"""Exact input framing and stable source-revision observations."""

from __future__ import annotations

from dataclasses import dataclass
import hashlib
import os
from pathlib import Path
import re
import stat
from typing import Mapping


CURRENT_SHA256 = "d04336f7fa8d1a6a0f03fe58a17f972b658217a73a3dff91a906b4ba295328a8"
SUCCESSOR_SHA256 = "bdfb461d1901f610633c5cbcd2477d24df3c77ca90599b9580c8289e50b82b68"
MAGIC = b"WFGRAMV1"
FRAME_NAMES = ("limits", "current", "proposal", "cases", "domains")
TRANSITION_IDS = (b"fixed-ident-partition",)
HARD_PROFILE_BYTES = 8_192
HARD_DOCUMENT_BYTES = 1_048_576
HARD_AUXILIARY_BYTES = 262_144
HARD_SOURCE_FILE_BYTES = 2_097_152
HARD_SOURCE_TREE_BYTES = 16_777_216

LIMIT_MAXIMA = {
    "cpu_timeout_seconds": 60,
    "max_case_bytes": 131_072,
    "max_cases": 1_024,
    "max_definitions": 1_024,
    "max_document_bytes": 524_288,
    "max_domain_bytes": 131_072,
    "max_domains": 64,
    "max_ebnf_depth": 128,
    "max_engine_output_bytes": 8_388_608,
    "max_final_report_bytes": 16_777_216,
    "max_generated_streams": 100_000,
    "max_grammar_nodes": 65_536,
    "max_lexical_definitions": 128,
    "max_line_bytes": 16_384,
    "max_lines": 4_096,
    "max_rules": 1_024,
    "max_symbol_bytes": 256,
    "max_terminal_occurrences": 8_192,
    "oracle_max_chart_items": 1_000_000,
    "oracle_max_packed_edges": 1_000_000,
    "oracle_max_proof_nodes": 1_000_000,
    "oracle_max_source_tokens": 256,
    "static_max_lookahead_words": 262_144,
    "static_max_product_states": 1_000_000,
    "static_max_work": 10_000_000,
    "wall_timeout_seconds": 60,
}
LIMIT_KEYS = frozenset(LIMIT_MAXIMA)

_HEX = re.compile(rb"(?:[0-9a-f]{2})+\Z")
_IDENTIFIER = re.compile(rb"[a-z][a-z0-9-]*\Z")
_SYMBOL = re.compile(rb"[a-z][a-z0-9_]*\Z")


class RunnerError(RuntimeError):
    """A closed runner-boundary failure with no grammar conclusion."""

    def __init__(self, code: str, message: str):
        super().__init__(message)
        self.code = code


def fail(code: str, message: str) -> None:
    raise RunnerError(code, message)


@dataclass(frozen=True)
class BoundBytes:
    name: str
    data: bytes

    @property
    def binding(self) -> dict[str, object]:
        return {
            "byte_length": len(self.data),
            "sha256": hashlib.sha256(self.data).hexdigest(),
        }


@dataclass(frozen=True)
class Inputs:
    sections: tuple[BoundBytes, ...]
    expectations: BoundBytes
    limits: Mapping[str, int]
    installation: Mapping[str, object] | None = None

    def section(self, name: str) -> BoundBytes:
        return next(item for item in self.sections if item.name == name)


@dataclass(frozen=True)
class SourceRevision:
    file_count: int
    byte_length: int
    sha256: str

    def value(self) -> dict[str, object]:
        return {
            "byte_length": self.byte_length,
            "file_count": self.file_count,
            "sha256": self.sha256,
        }


def read_regular(path: Path, maximum: int, label: str) -> bytes:
    """Read one bounded non-symlink file through one descriptor."""

    try:
        before_path = path.lstat()
    except OSError as error:
        fail("input_open", f"{label} is unavailable: {type(error).__name__}")
    if stat.S_ISLNK(before_path.st_mode) or not stat.S_ISREG(before_path.st_mode):
        fail("input_type", f"{label} is not a real regular file")
    flags = os.O_RDONLY | getattr(os, "O_CLOEXEC", 0) | getattr(os, "O_NOFOLLOW", 0)
    descriptor = -1
    try:
        descriptor = os.open(path, flags)
        before = os.fstat(descriptor)
        if before.st_size < 0 or before.st_size > maximum:
            fail("input_size", f"{label} exceeds its byte bound")
        chunks: list[bytes] = []
        remaining = before.st_size
        while remaining:
            chunk = os.read(descriptor, min(65_536, remaining))
            if not chunk:
                fail("input_changed", f"{label} changed while read")
            chunks.append(chunk)
            remaining -= len(chunk)
        if os.read(descriptor, 1):
            fail("input_changed", f"{label} grew while read")
        after = os.fstat(descriptor)
    except OSError as error:
        fail("input_read", f"{label} could not be read: {type(error).__name__}")
    finally:
        if descriptor >= 0:
            os.close(descriptor)
    identity = lambda value: (
        value.st_dev,
        value.st_ino,
        value.st_mode,
        value.st_size,
        value.st_mtime_ns,
        value.st_ctime_ns,
    )
    if identity(before) != identity(after):
        fail("input_changed", f"{label} changed while read")
    return b"".join(chunks)


def parse_limits(raw: bytes) -> dict[str, int]:
    try:
        text = raw.decode("ascii")
    except UnicodeDecodeError:
        fail("limits_encoding", "limits.txt is not strict ASCII")
    if not text.endswith("\n") or "\r" in text:
        fail("limits_layout", "limits.txt must have LF lines and one terminal LF")
    pairs: list[tuple[str, int]] = []
    for line in text[:-1].split("\n"):
        if line.count("=") != 1:
            fail("limits_layout", "a limit line does not have one equals sign")
        name, spelling = line.split("=", 1)
        if not re.fullmatch(r"[a-z][a-z0-9_]*", name) or not re.fullmatch(r"[1-9][0-9]*", spelling):
            fail("limits_layout", "a limit line is not canonical")
        maximum = LIMIT_MAXIMA.get(name)
        if maximum is None:
            fail("limits_fields", "limits.txt fields are missing, extra, duplicate, or reordered")
        maximum_spelling = str(maximum)
        if len(spelling) > len(maximum_spelling) or (
            len(spelling) == len(maximum_spelling) and spelling > maximum_spelling
        ):
            fail("limits_value", "a limits.txt value exceeds the format-v1 hard maximum")
        pairs.append((name, int(spelling)))
    names = [name for name, _ in pairs]
    if names != sorted(names) or len(names) != len(set(names)) or set(names) != LIMIT_KEYS:
        fail("limits_fields", "limits.txt fields are missing, extra, duplicate, or reordered")
    result = dict(pairs)
    return result


def _validate_table(raw: bytes, kind: str, maximum_records: int, maximum_symbol_bytes: int = 256) -> None:
    headers = {
        "cases": (b"whitefoot.grammar-cases.v1", b"case", 4),
        "domains": (b"whitefoot.grammar-domains.v1", b"domain", 5),
        "expectations": (b"whitefoot.grammar-expectations.v2", None, None),
    }
    if not raw.endswith(b"\n") or b"\r" in raw:
        fail(f"{kind}_layout", f"{kind}.txt is not canonical LF text")
    try:
        raw.decode("ascii")
    except UnicodeDecodeError:
        fail(f"{kind}_encoding", f"{kind}.txt is not strict ASCII")
    lines = raw[:-1].split(b"\n")
    header, tag, field_count = headers[kind]
    if not lines or lines[0] != header or any(not line for line in lines):
        fail(f"{kind}_header", f"{kind}.txt has an invalid header or blank line")
    body = lines[1:]
    if len(body) > maximum_records or body != sorted(body) or len(body) != len(set(body)):
        fail(f"{kind}_order", f"{kind}.txt exceeds its record cap or is not sorted and unique")
    logical_keys: set[tuple[bytes, ...]] = set()
    for line in body:
        fields = line.split(b"\t")
        if kind in ("cases", "domains"):
            if len(fields) != field_count or fields[0] != tag:
                fail(f"{kind}_record", f"{kind}.txt contains a malformed record")
            start_index = 2 if kind == "cases" else 3
            if not _IDENTIFIER.fullmatch(fields[1]) or not _SYMBOL.fullmatch(fields[start_index]):
                fail(f"{kind}_record", f"{kind}.txt contains a noncanonical id or start symbol")
            if len(fields[1]) > maximum_symbol_bytes or len(fields[start_index]) > maximum_symbol_bytes:
                fail(f"{kind}_record", f"{kind}.txt contains an oversized id or start symbol")
            if kind == "domains" and fields[2] != b"fixed-lowerword-call":
                fail("domains_record", "domains.txt contains an unknown generator")
            if not _HEX.fullmatch(fields[-1]):
                fail(f"{kind}_record", f"{kind}.txt contains noncanonical source hex")
            logical_key = (fields[1],)
        elif fields[0] == b"case":
            if (
                len(fields) != 4
                or not _IDENTIFIER.fullmatch(fields[1])
                or fields[2] not in (b"current", b"proposal")
                or fields[3] not in (b"zero", b"one", b"many")
            ):
                fail("expectations_record", "expectations.txt contains a malformed case expectation")
            if len(fields[1]) > maximum_symbol_bytes:
                fail("expectations_record", "expectations.txt contains an oversized case id")
            logical_key = (fields[0], fields[1], fields[2])
        elif fields[0] == b"case-delta":
            if (
                len(fields) != 3
                or not _IDENTIFIER.fullmatch(fields[1])
                or fields[2] not in (b"trace-identical", b"trace-replacement", b"trace-subset")
            ):
                fail("expectations_record", "expectations.txt contains a malformed case-delta policy")
            if len(fields[1]) > maximum_symbol_bytes:
                fail("expectations_record", "expectations.txt contains an oversized case id")
            logical_key = (fields[0], fields[1])
        elif fields[0] == b"transition":
            if len(fields) != 3 or not _IDENTIFIER.fullmatch(fields[1]) or not _IDENTIFIER.fullmatch(fields[2]):
                fail("expectations_record", "expectations.txt contains a malformed transition expectation")
            if len(fields[1]) > maximum_symbol_bytes or len(fields[2]) > maximum_symbol_bytes:
                fail("expectations_record", "expectations.txt contains an oversized transition id")
            logical_key = (fields[0], fields[1])
        else:
            fail("expectations_record", "expectations.txt contains an unknown record")
        if logical_key in logical_keys:
            fail(f"{kind}_duplicate", f"{kind}.txt repeats a logical record id")
        logical_keys.add(logical_key)


def load_inputs(
    root: Path,
    expected_current_sha256: str = CURRENT_SHA256,
    *,
    installed: bool = False,
) -> Inputs:
    limits_raw = read_regular(root / "limits.txt", HARD_PROFILE_BYTES, "limits.txt")
    limits = parse_limits(limits_raw)
    document_cap = min(HARD_DOCUMENT_BYTES, limits["max_document_bytes"])
    case_cap = min(HARD_AUXILIARY_BYTES, limits["max_case_bytes"])
    domain_cap = min(HARD_AUXILIARY_BYTES, limits["max_domain_bytes"])
    candidate = read_regular(
        root / "proposal" / "kernel-spec-successor-candidate.md",
        document_cap,
        "reviewed successor candidate",
    )
    candidate_binding = BoundBytes("proposal", candidate).binding
    if candidate_binding["sha256"] != SUCCESSOR_SHA256:
        fail("proposal_hash", "the successor candidate is not the reviewed v0.9 bytes")
    installation: Mapping[str, object] | None = None
    proposal = candidate
    if installed:
        installed_specification = read_regular(
            root.parent / "spec" / "kernel-spec-v0.9.md",
            document_cap,
            "installed v0.9 specification",
        )
        if installed_specification != candidate:
            fail(
                "installed_specification",
                "installed v0.9 is not byte-for-byte identical to the reviewed candidate",
            )
        proposal = installed_specification
        installed_binding = BoundBytes("proposal", installed_specification).binding
        installation = {
            "candidate": {
                **candidate_binding,
                "path": "grammar-verifier/proposal/kernel-spec-successor-candidate.md",
            },
            "installed_specification": {
                **installed_binding,
                "path": "spec/kernel-spec-v0.9.md",
            },
            "mode": "installed-v0.9",
            "relation": "byte-identical",
        }
    sections = (
        BoundBytes("limits", limits_raw),
        BoundBytes("current", read_regular(root.parent / "spec" / "kernel-spec-v0.8.md", document_cap, "current specification")),
        BoundBytes("proposal", proposal),
        BoundBytes("cases", read_regular(root / "cases.txt", case_cap, "cases.txt")),
        BoundBytes("domains", read_regular(root / "domains.txt", domain_cap, "domains.txt")),
    )
    expectations = BoundBytes(
        "expectations",
        read_regular(root / "expectations.txt", HARD_AUXILIARY_BYTES, "expectations.txt"),
    )
    if sections[1].binding["sha256"] != expected_current_sha256:
        fail("current_hash", "the current specification is not the pinned v0.8 bytes")
    symbol_cap = limits["max_symbol_bytes"]
    _validate_table(sections[3].data, "cases", limits["max_cases"], symbol_cap)
    _validate_table(sections[4].data, "domains", limits["max_domains"], symbol_cap)
    expectation_cap = 3 * limits["max_cases"] + len(TRANSITION_IDS)
    _validate_table(expectations.data, "expectations", expectation_cap, symbol_cap)
    case_ids = {line.split(b"\t")[1] for line in sections[3].data.splitlines()[1:]}
    expectation_fields = [line.split(b"\t") for line in expectations.data.splitlines()[1:]]
    expected_case_keys = {(identifier, document) for identifier in case_ids for document in (b"current", b"proposal")}
    observed_case_keys = {
        (fields[1], fields[2]) for fields in expectation_fields if fields[0] == b"case"
    }
    observed_case_delta_ids = {
        fields[1] for fields in expectation_fields if fields[0] == b"case-delta"
    }
    observed_transitions = {
        fields[1] for fields in expectation_fields if fields[0] == b"transition"
    }
    if (
        observed_case_keys != expected_case_keys
        or observed_case_delta_ids != case_ids
        or observed_transitions != set(TRANSITION_IDS)
    ):
        fail(
            "expectations_coverage",
            "expectations.txt does not cover every case, case-delta policy, and closed transition",
        )
    return Inputs(sections, expectations, limits, installation)


def make_frame(inputs: Inputs) -> bytes:
    if tuple(section.name for section in inputs.sections) != FRAME_NAMES:
        fail("frame_sections", "the engine sections are missing or reordered")
    lengths = b"".join(len(section.data).to_bytes(8, "big") for section in inputs.sections)
    return MAGIC + lengths + b"".join(section.data for section in inputs.sections)


def _manifest_entries(manifest: Path, expected: set[str]) -> list[tuple[str, bytes]]:
    raw = read_regular(manifest, HARD_SOURCE_FILE_BYTES, str(manifest.name))
    try:
        lines = raw.decode("ascii").splitlines()
    except UnicodeDecodeError:
        fail("source_manifest", f"{manifest} is not ASCII")
    if (
        not raw.endswith(b"\n")
        or not lines
        or lines != sorted(lines)
        or len(lines) != len(set(lines))
        or set(lines) != expected
    ):
        fail("source_manifest", f"{manifest} is not sorted, unique, and exhaustive")
    result: list[tuple[str, bytes]] = []
    total = 0
    for relative in lines:
        candidate = Path(relative)
        if candidate.is_absolute() or ".." in candidate.parts or candidate.as_posix() != relative:
            fail("source_manifest", f"{manifest} contains an unsafe path")
        data = read_regular(manifest.parent / candidate, HARD_SOURCE_FILE_BYTES, relative)
        total += len(data)
        if total > HARD_SOURCE_TREE_BYTES:
            fail("source_manifest", f"{manifest} exceeds the source-tree byte cap")
        result.append((relative, data))
    return result


def source_revision(manifest: Path, expected: set[str]) -> SourceRevision:
    entries = _manifest_entries(manifest, expected)
    digest = hashlib.sha256()
    total = 0
    for relative, data in entries:
        name = relative.encode("ascii")
        digest.update(len(name).to_bytes(8, "big"))
        digest.update(name)
        digest.update(len(data).to_bytes(8, "big"))
        digest.update(data)
        total += len(data)
    return SourceRevision(len(entries), total, digest.hexdigest())


def runner_sources(root: Path) -> set[str]:
    result = {
        "FORMAT.md",
        "Makefile",
        "RUNNER_SOURCES",
        "evidence/v0.9-manifest-metadata.patch",
        "evidence/v0.9-post-form2-case-intent.patch",
    }
    excluded_roots = {"oracle", "static-auditor"}
    result.update(
        path.relative_to(root).as_posix()
        for path in root.rglob("*.py")
        if path.is_file()
        and path.relative_to(root).parts[0] not in excluded_roots
        and "__pycache__" not in path.parts
    )
    return result


def static_sources(root: Path) -> set[str]:
    result = {"Cargo.lock", "Cargo.toml", "SOURCES", "rust-toolchain.toml"}
    result.update(path.relative_to(root).as_posix() for path in root.rglob("*.rs") if "target" not in path.parts)
    return result


def oracle_sources(root: Path) -> set[str]:
    result = {"SOURCES"}
    result.update(path.relative_to(root).as_posix() for path in root.rglob("*.py") if "__pycache__" not in path.parts)
    return result
