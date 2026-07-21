from __future__ import annotations

from collections import Counter
from typing import Any

from form2_inputs import ProtectedInputs, sha256
from form2_lex import token_lexeme_stream, tokenize
from form2_model import audit_proposed
from form2_render import (
    inject_isolated_fixture_defect,
    render_derivation,
)
from form2_structural_assembly import (
    STRUCTURAL_ARTIFACT_NAMES,
    StructuralReportError,
    StructuralRun,
    assemble_structural_artifacts,
)
from form2_tree import (
    CandidateParser,
    ParseAttempt,
    load_candidate_parser,
    parse_one,
)
from form2_zero_audit import (
    PROTECTED_REPAIR_KEYS,
    ZeroDerivationAuditError,
    audit_zero_source,
    source_key,
    validate_zero_inventory,
)


FORM2_FIXTURES = frozenset(
    ("form2-neg-noncanonical-ws", "x-form-form2-tab-indent")
)


def _attempt_record(attempt: ParseAttempt) -> dict[str, Any]:
    return {
        "chart_item_count": attempt.chart_item_count,
        "classification": attempt.classification,
        "failure": attempt.failure,
        "packed_edge_count": attempt.packed_edge_count,
        "proof_node_count": attempt.proof_node_count,
        "source_token_count": attempt.source_token_count,
        "stage": attempt.stage,
    }


def _recover_tab_fixture(raw: bytes) -> tuple[bytes, dict[str, Any]]:
    if raw.count(b"\t") != 2:
        raise StructuralReportError("tab FORM-2 fixture no longer contains exactly two tabs")
    recovered = raw.replace(b"\t", b"  ")
    before = token_lexeme_stream(tokenize(raw).tokens)
    after = token_lexeme_stream(tokenize(recovered).tokens)
    if before != after:
        raise StructuralReportError("tab recovery changes the fixture token lexemes")
    return recovered, {
        "kind": "replace-two-tab-indents-with-canonical-depth-one-spaces-for-tree-recovery",
        "lexeme_projection_sha256_after": sha256(after),
        "lexeme_projection_sha256_before": sha256(before),
        "replacement_count": 2,
    }


def _merge_counts(target: Counter[str], values: dict[str, int]) -> None:
    target.update(values)


def _validate_round_trip(
    parser: CandidateParser,
    before: ParseAttempt,
    canonical: bytes,
) -> ParseAttempt:
    if before.derivation is None:
        raise StructuralReportError("round-trip input lacks a derivation")
    after = parse_one(parser, canonical)
    if after.classification != "one" or after.derivation is None:
        raise StructuralReportError("rendered bytes do not have one complete derivation")
    if (
        before.derivation.source_forest_projection_sha256
        != after.derivation.source_forest_projection_sha256
    ):
        raise StructuralReportError("rendering changes the source-local item forest")
    if (
        before.derivation.terminal_projection_sha256
        != after.derivation.terminal_projection_sha256
    ):
        raise StructuralReportError("rendering changes terminal lexemes")
    rerendered = render_derivation(after.derivation).raw
    if rerendered != canonical:
        raise StructuralReportError("render/parse/render is not byte-idempotent")
    return after


def _fixture_output(
    identifier: str,
    canonical: bytes,
) -> tuple[bytes, dict[str, Any]]:
    migrated, defect = inject_isolated_fixture_defect(identifier, canonical)
    start = defect["byte_start"]
    actual = bytes.fromhex(defect["actual_hex"])
    expected = bytes.fromhex(defect["canonical_hex"])
    if canonical[:start] != migrated[:start]:
        raise StructuralReportError("FORM-2 fixture differs before its isolated defect")
    if canonical[start + len(expected) :] != migrated[start + len(actual) :]:
        raise StructuralReportError("FORM-2 fixture differs after its isolated defect")
    if migrated == canonical:
        raise StructuralReportError("FORM-2 fixture output accidentally became canonical")
    audit = audit_proposed(migrated).report
    expected_counts = {"indentation-two-spaces-per-brace-level": 1}
    if (
        audit["definite_and_interpretation_bound_violation_count"] != 1
        or audit["violation_counts_by_clause"] != expected_counts
    ):
        raise StructuralReportError(
            "FORM-2 fixture output does not isolate one indentation defect"
        )
    return migrated, {
        **defect,
        "audit_violation_counts": expected_counts,
    }


def build_structural_artifacts(
    inputs: ProtectedInputs,
    tool_sources: dict[str, str],
) -> dict[str, bytes]:
    manifested_form2 = {
        source.identifier
        for source in inputs.sources
        if source.manifest is not None
        and source.manifest["expect"] == {"kind": "reject", "rule": "FORM-2"}
    }
    if manifested_form2 != FORM2_FIXTURES:
        raise StructuralReportError("protected FORM-2 fixture inventory changed")
    parser = load_candidate_parser()
    files: list[dict[str, Any]] = []
    patch_inputs: list[tuple[str, bytes, bytes]] = []
    parse_counts: Counter[str] = Counter()
    block_counts: Counter[str] = Counter()
    line_counts: Counter[str] = Counter()
    production_counts: Counter[str] = Counter()
    earlier_stage_counts: Counter[str] = Counter()
    total_empty_blocks = 0
    total_lines = 0
    total_items = 0
    total_requires_transitions = 0
    changed_count = 0
    canonical_migrated_count = 0
    rendered_count = 0
    blockers: list[dict[str, Any]] = []
    fixture_defects: list[dict[str, Any]] = []
    documentation_followups: list[dict[str, Any]] = []
    zero_derivation_audit: list[dict[str, Any]] = []
    observed_zero_keys: set[str] = set()
    protected_repairs: list[dict[str, Any]] = []
    repair_patch_inputs: list[tuple[str, bytes, bytes]] = []
    formatting_patch_inputs: list[tuple[str, bytes, bytes]] = []

    for source in inputs.sources:
        exact_attempt = parse_one(parser, source.raw)
        parse_counts[exact_attempt.classification] += 1
        parse_input = source.raw
        recovery: dict[str, Any] | None = None
        protected_repair: dict[str, Any] | None = None
        attempt = exact_attempt
        if exact_attempt.classification == "zero":
            try:
                zero_record, repaired = audit_zero_source(
                    parser, source, exact_attempt
                )
            except ZeroDerivationAuditError as error:
                raise StructuralReportError(str(error)) from error
            zero_derivation_audit.append(zero_record)
            observed_zero_keys.add(source_key(source))
            if repaired is not None:
                protected_repair = zero_record["repair_proposal"]
                if protected_repair is None:
                    raise StructuralReportError("repair proposal record is absent")
                protected_repairs.append(
                    {
                        **protected_repair,
                        "expected_rule": zero_record["expected_rule"],
                        "id": source.identifier,
                        "path": source.path,
                    }
                )
                parse_input = repaired
                attempt = parse_one(parser, parse_input)
                if attempt.classification != "one":
                    raise StructuralReportError(
                        f"proposed protected-source repair does not derive: {source.path}"
                    )
        if source.identifier == "x-form-form2-tab-indent":
            if exact_attempt.classification != "zero":
                raise StructuralReportError("tab fixture unexpectedly has a derivation")
            parse_input, recovery = _recover_tab_fixture(source.raw)
            if sha256(parse_input) != zero_derivation_audit[-1]["control"]["sha256"]:
                raise StructuralReportError("tab recovery and zero audit control differ")
            attempt = parse_one(parser, parse_input)
            if attempt.classification != "one":
                raise StructuralReportError("tab fixture is not trivia-recoverable")

        expected = None if source.manifest is None else source.manifest["expect"]
        status = None if source.manifest is None else source.manifest["status"]
        intended_form2 = source.identifier in FORM2_FIXTURES
        canonical: bytes | None = None
        migrated = source.raw
        round_trip: ParseAttempt | None = None
        structural_counts: dict[str, Any] | None = None
        bundle_topology_sha256: str | None = None
        separator_owners_sha256: str | None = None
        defect: dict[str, Any] | None = None

        if attempt.classification == "one" and attempt.derivation is not None:
            rendered = render_derivation(attempt.derivation)
            canonical = rendered.raw
            structural_counts = rendered.structural_counts
            bundle_topology_sha256 = rendered.bundle_topology_projection_sha256
            separator_owners_sha256 = rendered.separator_owner_projection_sha256
            canonical_audit = audit_proposed(canonical).report
            if (
                canonical_audit[
                    "definite_and_interpretation_bound_violation_count"
                ]
                or canonical_audit["lexical_observation_issues"]
            ):
                raise StructuralReportError(
                    f"renderer emits an observed FORM-2 violation: {source.path}"
                )
            round_trip = _validate_round_trip(parser, attempt, canonical)
            rendered_count += 1
            _merge_counts(block_counts, structural_counts["block_production_counts"])
            _merge_counts(
                line_counts, structural_counts["line_bearing_production_counts"]
            )
            _merge_counts(production_counts, structural_counts["production_counts"])
            total_empty_blocks += structural_counts["empty_block_count"]
            total_lines += structural_counts["physical_line_count"]
            total_items += structural_counts["top_level_item_count"]
            total_requires_transitions += structural_counts[
                "requires_body_transition_count"
            ]
            if intended_form2:
                migrated, defect = _fixture_output(source.identifier, canonical)
                documentation_followup = None
                if source.identifier == "form2-neg-noncanonical-ws":
                    documentation_followup = {
                        "current_manifest_doc": source.manifest.get("doc"),
                        "id": source.identifier,
                        "proposed_manifest_doc": (
                            "One body statement is indented by four spaces instead "
                            "of the canonical two; this isolated FORM-2 defect must "
                            "be rejected."
                        ),
                        "reason": (
                            "the complete migration removes the fixture's former "
                            "second indentation defect and colon-spacing defect"
                        ),
                    }
                    documentation_followups.append(documentation_followup)
                fixture_defects.append(
                    {
                        "canonical_sha256": sha256(canonical),
                        "defect": defect,
                        "id": source.identifier,
                        "manifest_documentation_followup": documentation_followup,
                        "migrated_sha256": sha256(migrated),
                        "path": source.path,
                    }
                )
            else:
                migrated = canonical
                canonical_migrated_count += 1
        else:
            if intended_form2:
                raise StructuralReportError(
                    f"FORM-2 fixture is not renderable after approved recovery: {source.path}"
                )
            expected_rule = None
            if isinstance(expected, dict) and expected.get("kind") == "reject":
                expected_rule = expected.get("rule")
            classification = (
                "candidate-grammar-contradicts-expected-accept"
                if isinstance(expected, dict) and expected.get("kind") == "accept"
                else "unmanifested-no-derivation"
                if expected is None
                else exact_attempt.stage
            )
            earlier_stage_counts[classification] += 1
            blocker = {
                "classification": classification,
                "expect": expected,
                "expected_owning_rule": expected_rule,
                "id": source.identifier,
                "parse": _attempt_record(exact_attempt),
                "path": source.path,
                "reason": (
                    "no complete derivation exists; a structural renderer "
                    "cannot fabricate one"
                ),
                "status": status,
            }
            blockers.append(blocker)

        changed_count += int(migrated != source.raw)
        patch_inputs.append((source.path, source.raw, migrated))
        syntax_repaired = source.raw if protected_repair is None else parse_input
        repair_patch_inputs.append((source.path, source.raw, syntax_repaired))
        formatting_patch_inputs.append((source.path, syntax_repaired, migrated))
        before_lexemes = token_lexeme_stream(tokenize(source.raw).tokens)
        parse_lexemes = token_lexeme_stream(tokenize(parse_input).tokens)
        migrated_lexemes = token_lexeme_stream(tokenize(migrated).tokens)
        if (
            attempt.derivation is not None
            and sha256(parse_lexemes)
            != attempt.derivation.terminal_projection_sha256
        ):
            raise StructuralReportError(
                f"audit lexer and derivation terminals disagree: {source.path}"
            )
        if canonical is not None and parse_lexemes != migrated_lexemes:
            raise StructuralReportError(f"migration changes token lexemes: {source.path}")
        file_record = {
            "after_byte_count": len(migrated),
            "after_sha256": sha256(migrated),
            "before_byte_count": len(source.raw),
            "before_sha256": source.sha256,
            "canonical_sha256": None if canonical is None else sha256(canonical),
            "exact_parse": _attempt_record(exact_attempt),
            "expect": expected,
            "id": source.identifier,
            "intended_form2_fixture": intended_form2,
            "isolated_defect": defect,
            "manifested": source.manifest is not None,
            "migration_disposition": (
                "one-isolated-form2-defect"
                if intended_form2
                else "owner-gated-syntax-repair-then-canonical-structural-render"
                if protected_repair is not None
                else "canonical-structural-render"
                if canonical is not None
                else "unchanged-no-complete-derivation"
            ),
            "path": source.path,
            "protected_syntax_repair": protected_repair,
            "recovery": recovery,
            "render_parse": None if attempt is exact_attempt else _attempt_record(attempt),
            "round_trip_parse": None
            if round_trip is None
            else _attempt_record(round_trip),
            "status": status,
            "structural_counts": structural_counts,
            "terminal_projection_sha256_after": sha256(migrated_lexemes),
            "terminal_projection_sha256_before": sha256(before_lexemes),
            "terminal_projection_sha256_render_input": sha256(parse_lexemes),
            "syntax_repaired_byte_count": len(syntax_repaired),
            "syntax_repaired_sha256": sha256(syntax_repaired),
            "source_forest_projection_sha256": None
            if attempt.derivation is None
            else attempt.derivation.source_forest_projection_sha256,
            "oracle_source_trace_forest_projection_sha256": None
            if attempt.derivation is None
            else attempt.derivation.source_trace_forest_projection_sha256,
            "bundle_topology_projection_sha256": bundle_topology_sha256,
            "separator_owner_projection_sha256": separator_owners_sha256,
        }
        files.append(file_record)

    try:
        validate_zero_inventory(observed_zero_keys)
    except ZeroDerivationAuditError as error:
        raise StructuralReportError(str(error)) from error
    if len(files) != 293 or len(fixture_defects) != 2:
        raise StructuralReportError("structural migration inventory is incomplete")
    if len(protected_repairs) != len(PROTECTED_REPAIR_KEYS):
        raise StructuralReportError("protected syntax-repair inventory is incomplete")
    return assemble_structural_artifacts(
        StructuralRun(
            inputs=inputs,
            parser=parser,
            tool_sources=tool_sources,
            files=files,
            blockers=blockers,
            fixture_defects=fixture_defects,
            documentation_followups=documentation_followups,
            zero_derivation_audit=zero_derivation_audit,
            protected_repairs=protected_repairs,
            patch_inputs=patch_inputs,
            repair_patch_inputs=repair_patch_inputs,
            formatting_patch_inputs=formatting_patch_inputs,
            parse_counts=parse_counts,
            block_counts=block_counts,
            line_counts=line_counts,
            production_counts=production_counts,
            earlier_stage_counts=earlier_stage_counts,
            total_empty_blocks=total_empty_blocks,
            total_lines=total_lines,
            total_items=total_items,
            total_requires_transitions=total_requires_transitions,
            changed_count=changed_count,
            canonical_migrated_count=canonical_migrated_count,
            rendered_count=rendered_count,
        )
    )
