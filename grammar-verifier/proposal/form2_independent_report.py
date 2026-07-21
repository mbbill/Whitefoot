from __future__ import annotations

from collections import Counter
import difflib
import hashlib
from typing import Any, Optional

from form2_independent_compare import (
    COMPLETE_STAGE,
    REPAIR_COMPLETE_STAGE,
    RENDERED_STAGES,
    TAB_RECOVERY_STAGE,
    IndependentComparisonError,
    compare_primary as compare_with_primary,
)
from form2_independent_inputs import (
    MODULE_ROOT,
    IndependentInputs,
    IndependentSource,
    canonical_json,
    digest,
    load_independent_inputs,
)
from form2_independent_lex import (
    IndependentLexError,
    IndependentToken,
    lex_independently,
    terminal_projection,
)
from form2_independent_parse import parse_independently
from form2_independent_render import FORM2_TAB_NEGATIVE, render_for_migration
from form2_independent_repairs import (
    IndependentRepairError,
    audit_failure_controls,
)
from form2_independent_syntax import (
    IndependentForest,
    IndependentParseError,
    source_forest_projection,
)


SOURCE_MANIFEST = MODULE_ROOT / "FORM2_INDEPENDENT_SOURCES"

REPORT_NAME = "form2-independent-report.json"
CHECKSUM_NAME = "form2-independent-evidence.sha256"

class IndependentReportError(RuntimeError):
    pass


def _validate_candidate(raw: bytes) -> None:
    required = (
        b"# Kernel Specification v0.9\n",
        b"[FORM-2] Each source file is UTF-8.",
        b"The left-attachment set contains `(`, `[`, `<`, `&`, and `.`.",
        b"The right-attachment set contains `)`, `]`, `>`, `,`, `;`, `.`, `:`, `(`, and `<`.",
        b"`fn f()`, `fn f<T>()`, and `fn f ['r]()`",
        b'law          := "law" IDENT "(" (law_arg',
        b'requires_block:= "requires" "{" requires_entry* "}"',
        b"requires_entry:= doc | stmt",
        b'value_match := "match" expr "{" arm+ "}"',
    )
    missing = [fragment.decode("ascii") for fragment in required if fragment not in raw]
    if missing:
        raise IndependentReportError(f"candidate contract is missing fragments: {missing!r}")
    forbidden = (
        b"fn f ['r']()",
        b"and a `requires_let_stmt`",
        b'law          := "law" LAWNAME',
    )
    present = [fragment.decode("ascii") for fragment in forbidden if fragment in raw]
    if present:
        raise IndependentReportError(f"candidate retains superseded fragments: {present!r}")


def _source_manifest() -> dict[str, str]:
    try:
        raw = SOURCE_MANIFEST.read_bytes()
    except OSError as error:
        raise IndependentReportError(f"cannot read {SOURCE_MANIFEST}: {error}") from error
    rows = raw.splitlines()
    if not rows or rows != sorted(rows) or len(rows) != len(set(rows)):
        raise IndependentReportError("FORM2_INDEPENDENT_SOURCES is not sorted and unique")
    result: dict[str, str] = {}
    for row in rows:
        if not row or row != row.strip():
            raise IndependentReportError("FORM2_INDEPENDENT_SOURCES has a malformed row")
        try:
            relative = row.decode("ascii")
        except UnicodeDecodeError as error:
            raise IndependentReportError("source-manifest path is not ASCII") from error
        path = MODULE_ROOT.parent / relative
        if not path.is_file() or path.is_symlink():
            raise IndependentReportError(f"independent source is not regular: {relative}")
        result[f"grammar-verifier/{relative}"] = digest(path.read_bytes())
    result["grammar-verifier/proposal/FORM2_INDEPENDENT_SOURCES"] = digest(raw)
    return dict(sorted(result.items()))


def _expectation_projection(inputs: IndependentInputs) -> bytes:
    rows = [
        {
            "expect": source.manifest["expect"],
            "id": source.identifier,
            "status": source.manifest["status"],
        }
        for source in inputs.sources
        if source.manifest is not None
    ]
    return canonical_json(rows)


def _failure_row(
    source: IndependentSource,
    stage: str,
    reason: str,
    byte_offset: int,
    token_index: int | None,
) -> dict[str, Any]:
    return {
        "byte_offset": byte_offset,
        "expect": None if source.manifest is None else source.manifest["expect"],
        "id": source.identifier,
        "path": source.path,
        "reason": reason,
        "source_sha256": source.sha256,
        "stage": stage,
        "status": None if source.manifest is None else source.manifest["status"],
        "token_index": token_index,
    }


def _recover_tab_indents(raw: bytes, expected_count: int) -> bytes:
    if raw.count(b"\t") != expected_count or b"\r" in raw:
        raise IndependentReportError("tab fixture has an unexpected forbidden-byte shape")
    for line in raw.splitlines():
        if b"\t" in line and not line.startswith(b"\t"):
            raise IndependentReportError("tab fixture has a non-indentation tab")
    return raw.replace(b"\t", b"  ")


def _complete_row(
    source: IndependentSource,
    parse_raw: bytes,
    syntax_repaired_raw: bytes,
    tokens: tuple[IndependentToken, ...],
    forest: IndependentForest,
    *,
    stage: str,
    recovery: dict[str, Any] | None,
) -> tuple[dict[str, Any], bytes]:
    rendering = render_for_migration(tokens, forest, source.identifier)

    canonical_tokens = lex_independently(rendering.canonical)
    canonical_forest = parse_independently(
        canonical_tokens, len(rendering.canonical)
    )
    if terminal_projection(tokens) != terminal_projection(canonical_tokens):
        raise IndependentReportError(f"canonical render changed terminals: {source.path}")
    if source_forest_projection(forest) != source_forest_projection(
        canonical_forest
    ):
        raise IndependentReportError(
            f"canonical render changed source item forest: {source.path}"
        )
    rerendered = render_for_migration(
        canonical_tokens, canonical_forest, None
    ).canonical
    if rerendered != rendering.canonical:
        raise IndependentReportError(f"canonical render is not idempotent: {source.path}")

    migration_parse_bytes = rendering.migration
    if source.identifier == FORM2_TAB_NEGATIVE:
        migration_parse_bytes = _recover_tab_indents(rendering.migration, 1)
        if migration_parse_bytes != rendering.canonical:
            raise IndependentReportError("tab migration contains more than its isolated defect")
    migration_tokens = lex_independently(migration_parse_bytes)
    migration_forest = parse_independently(
        migration_tokens, len(migration_parse_bytes)
    )
    if terminal_projection(tokens) != terminal_projection(migration_tokens):
        raise IndependentReportError(f"migration changed terminals: {source.path}")
    if source_forest_projection(forest) != source_forest_projection(
        migration_forest
    ):
        raise IndependentReportError(
            f"migration changed source item forest: {source.path}"
        )

    try:
        before_tokens = lex_independently(source.raw)
    except IndependentLexError:
        if source.identifier != FORM2_TAB_NEGATIVE:
            raise IndependentReportError(
                f"rendered source has no independent raw token projection: {source.path}"
            )
        before_tokens = tokens
    if parse_raw != syntax_repaired_raw and source.identifier != FORM2_TAB_NEGATIVE:
        raise IndependentReportError(
            f"only the tab recovery may differ from the syntax-repaired input: {source.path}"
        )

    row = {
        "canonical_matches_source": rendering.canonical == source.raw,
        "canonical_sha256": digest(rendering.canonical),
        "expect": None if source.manifest is None else source.manifest["expect"],
        "id": source.identifier,
        "intentional_defect": rendering.intentional_defect,
        "migration_matches_source": rendering.migration == source.raw,
        "migration_sha256": digest(rendering.migration),
        "path": source.path,
        "recovery": recovery,
        "source_sha256": source.sha256,
        "stage": stage,
        "status": None if source.manifest is None else source.manifest["status"],
        "syntax_repaired_sha256": digest(syntax_repaired_raw),
        "terminal_count": len(tokens),
        "terminal_projection_sha256": digest(terminal_projection(tokens)),
        "terminal_projection_sha256_after": digest(
            terminal_projection(migration_tokens)
        ),
        "terminal_projection_sha256_before": digest(
            terminal_projection(before_tokens)
        ),
        "terminal_projection_sha256_render_input": digest(
            terminal_projection(tokens)
        ),
        "source_forest_node_count": len(forest.descendants()),
        "source_forest_projection_sha256": digest(
            source_forest_projection(forest)
        ),
        "bundle_topology_projection_sha256": (
            rendering.bundle_topology_projection_sha256
        ),
        "separator_owner_projection_sha256": (
            rendering.separator_owner_projection_sha256
        ),
    }
    return row, rendering.migration


def _tab_recovery_row(
    source: IndependentSource, lexical_error: IndependentLexError
) -> tuple[dict[str, Any], bytes]:
    if source.identifier != FORM2_TAB_NEGATIVE:
        raise IndependentReportError("tab recovery was requested for an unnamed fixture")
    recovered = _recover_tab_indents(source.raw, 2)
    tokens = lex_independently(recovered)
    forest = parse_independently(tokens, len(recovered))
    recovery = {
        "kind": "replace-two-tab-indents-with-depth-one-spaces-for-tree-recovery",
        "raw_byte_offset": lexical_error.offset,
        "raw_failure_reason": lexical_error.reason,
        "replacement_count": 2,
    }
    return _complete_row(
        source,
        recovered,
        source.raw,
        tokens,
        forest,
        stage=TAB_RECOVERY_STAGE,
        recovery=recovery,
    )


def _raw_failure_paths(inputs: IndependentInputs) -> frozenset[str]:
    failures: set[str] = set()
    for source in inputs.sources:
        try:
            tokens = lex_independently(source.raw)
            parse_independently(tokens, len(source.raw))
        except (IndependentLexError, IndependentParseError):
            failures.add(source.path)
    return frozenset(failures)


def _unified_patch(changes: list[tuple[str, bytes, bytes]]) -> bytes:
    output = bytearray()
    for path, before, after in changes:
        if before == after:
            continue
        lines = difflib.diff_bytes(
            difflib.unified_diff,
            before.splitlines(keepends=True),
            after.splitlines(keepends=True),
            fromfile=f"a/{path}".encode("utf-8"),
            tofile=f"b/{path}".encode("utf-8"),
            lineterm=b"\n",
        )
        for line in lines:
            output.extend(line)
            if not line.endswith(b"\n"):
                output.extend(
                    b"\n\\ No newline at end of file\n"
                )
    return bytes(output)


def build_independent_artifacts(
    *,
    compare_primary: bool = True,
    inputs: Optional[IndependentInputs] = None,
) -> dict[str, bytes]:
    inputs = load_independent_inputs() if inputs is None else inputs
    _validate_candidate(inputs.candidate)
    raw_failure_paths = _raw_failure_paths(inputs)
    control_records, repairs = audit_failure_controls(inputs, raw_failure_paths)
    control_by_path = {row["path"]: row for row in control_records}
    files: list[dict[str, Any]] = []
    changes: list[tuple[str, bytes, bytes]] = []
    repair_changes: list[tuple[str, bytes, bytes]] = []
    stage_counts: Counter[str] = Counter()

    for source in inputs.sources:
        syntax_repaired = repairs.get(source.path, source.raw)
        parse_raw = syntax_repaired
        try:
            tokens = lex_independently(parse_raw)
        except IndependentLexError as error:
            if source.identifier == FORM2_TAB_NEGATIVE:
                row, migrated = _tab_recovery_row(source, error)
                changes.append((source.path, source.raw, migrated))
            else:
                row = _failure_row(
                    source, "no-lexical-formation", error.reason, error.offset, None
                )
        else:
            try:
                forest = parse_independently(tokens, len(parse_raw))
            except IndependentParseError as error:
                if source.path in repairs:
                    raise IndependentReportError(
                        f"proposed syntax repair does not derive: {source.path}"
                    ) from error
                row = _failure_row(
                    source,
                    "no-complete-derivation",
                    error.reason,
                    error.byte_offset,
                    error.token_index,
                )
                row["terminal_count"] = len(tokens)
                row["terminal_projection_sha256"] = digest(terminal_projection(tokens))
            else:
                stage = (
                    REPAIR_COMPLETE_STAGE
                    if source.path in repairs
                    else COMPLETE_STAGE
                )
                row, migrated = _complete_row(
                    source,
                    parse_raw,
                    syntax_repaired,
                    tokens,
                    forest,
                    stage=stage,
                    recovery=None,
                )
                changes.append((source.path, source.raw, migrated))
        if source.path in repairs:
            repair_changes.append((source.path, source.raw, syntax_repaired))
            row["protected_syntax_repair"] = control_by_path[source.path]
        else:
            row["protected_syntax_repair"] = None
        files.append(row)
        stage_counts[row["stage"]] += 1

    patch = _unified_patch(changes)
    repair_patch = _unified_patch(repair_changes)
    expectation_projection = _expectation_projection(inputs)
    try:
        comparison = (
            compare_with_primary(
                files,
                inputs.candidate_sha256,
                patch,
                repair_patch,
                control_records,
                expectation_projection,
            )
            if compare_primary
            else None
        )
    except IndependentComparisonError as error:
        raise IndependentReportError(str(error)) from error
    rendered_count = sum(stage_counts[stage] for stage in RENDERED_STAGES)
    remaining = [row for row in files if row["stage"] not in RENDERED_STAGES]
    if len(remaining) != 19 or any(
        not isinstance(row.get("expect"), dict)
        or row["expect"].get("kind") != "reject"
        for row in remaining
    ):
        raise IndependentReportError(
            "remaining no-derivation inventory is not exactly 19 intentional negatives"
        )
    repair_paths = sorted(repairs, key=str.encode)
    repair_rows = [
        {
            "after_sha256": digest(repairs[path]),
            "before_sha256": next(
                source.sha256 for source in inputs.sources if source.path == path
            ),
            "edits": control_by_path[path]["edits"],
            "id": control_by_path[path]["id"],
            "path": path,
        }
        for path in repair_paths
    ]
    report = {
        "authority": (
            "independent read-only FORM-2 proposal evidence; never language "
            "or compiler authority"
        ),
        "candidate_sha256": inputs.candidate_sha256,
        "failure_controls": control_records,
        "files": files,
        "manifest_expectation_projection_preserved": True,
        "manifest_expectation_projection_sha256": digest(expectation_projection),
        "manifest_sha256": inputs.manifest_sha256,
        "primary_comparison": comparison,
        "protected_baseline_sha256": inputs.baseline_sha256,
        "protected_syntax_repair_layer": {
            "expected_verdict_projection_changed": False,
            "patch_changed_paths": repair_paths,
            "patch_sha256": digest(repair_patch),
            "proposals": repair_rows,
            "source_count": len(repair_rows),
        },
        "schema": "whitefoot.form2-independent-structural.v4",
        "topology_policy": {
            "bundle_root": "exactly one global program node, including one-source bundles",
            "empty_source": (
                "no tree node; source identity and byte extent remain in BundleRootExtent"
            ),
            "separator_owner": (
                "derived from adjacent terminal leaves; leading, final, inter-item, "
                "and zero-item boundaries are SourceBytes"
            ),
            "source_projection": "ordered item forest with no program node",
        },
        "summary": {
            "canonical_source_count": sum(
                row.get("canonical_matches_source") is True for row in files
            ),
            "compiler_validated": False,
            "compiler_verdict_preservation_proven": False,
            "complete_protected_migration_proven": rendered_count == len(files),
            "evidence_ready_for_owner_review": comparison is not None,
            "migration_authorized": False,
            "migration_changed_source_count": sum(
                row.get("migration_matches_source") is False for row in files
            ),
            "protected_syntax_repair_source_count": len(repairs),
            "raw_failure_source_count": len(raw_failure_paths),
            "remaining_intentional_negative_count": len(remaining),
            "rendered_source_count": rendered_count,
            "source_count": len(files),
            "stage_counts": dict(sorted(stage_counts.items())),
        },
        "tool_sources": _source_manifest(),
    }
    artifacts = {REPORT_NAME: canonical_json(report)}
    checksum = "".join(
        f"{hashlib.sha256(artifacts[name]).hexdigest()}  {name}\n"
        for name in sorted(artifacts)
    ).encode("ascii")
    artifacts[CHECKSUM_NAME] = checksum
    return artifacts
