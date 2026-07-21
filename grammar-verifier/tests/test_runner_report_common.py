from __future__ import annotations

import unittest

from support_common import inputs
from support_oracle import oracle_report
from support_static import static_report
from runner_inputs import Inputs, RunnerError
from runner_report import parse_report


class ReportCommonTests(unittest.TestCase):
    def test_closed_static_and_oracle_reports_validate(self) -> None:
        value = inputs()
        static = parse_report(static_report(value), "static", value)
        oracle = parse_report(oracle_report(value), "oracle", value)
        self.assertEqual(static.common, oracle.common)
        self.assertIsNone(static.failure)
        self.assertIsNone(oracle.failure)

    def test_failure_report_is_closed_but_not_success(self) -> None:
        value = inputs()
        raw = b"WFGRREPORT1\nENGINE\tstatic\nFAIL\tresource\twork_limit\nEND\n"
        report = parse_report(raw, "static", value)
        self.assertEqual(report.failure, ("resource", "work_limit"))

    def test_oracle_underscore_failure_code_is_a_valid_inconclusive_report(self) -> None:
        value = inputs()
        raw = b"WFGRREPORT1\nENGINE\toracle\nFAIL\tinput\tframe_outer_limit\nEND\n"
        report = parse_report(raw, "oracle", value)
        self.assertEqual(report.failure, ("input", "frame_outer_limit"))

    def test_unknown_record_is_rejected(self) -> None:
        value = inputs()
        raw = static_report(value).replace(
            b"STATIC-TRANSITION",
            b"STATIC-UNKNOWN",
            1,
        )
        with self.assertRaisesRegex(RunnerError, "unknown"):
            parse_report(raw, "static", value)

    def test_duplicate_common_semantic_key_is_rejected(self) -> None:
        value = inputs()
        raw = static_report(value).replace(
            b"COVERAGE\tcurrent\t3\t0\t0\t1\t0",
            b"COVERAGE\tcurrent\t3\t0\t0\t1\t0\nCOVERAGE\tcurrent\t1\t0\t0\t0\t0",
            1,
        )
        with self.assertRaisesRegex(RunnerError, "duplicate|semantic key"):
            parse_report(raw, "static", value)

    def test_common_coverage_counts_equal_the_surface_ledger(self) -> None:
        value = inputs()
        changed = static_report(value).replace(
            b"COVERAGE\tcurrent\t3\t0\t0\t1\t0",
            b"COVERAGE\tcurrent\t2\t0\t0\t1\t0",
            1,
        )
        with self.assertRaisesRegex(RunnerError, "complete surface ledger"):
            parse_report(changed, "static", value)

    def test_common_fixed_expansion_is_closed_and_bounded(self) -> None:
        value = inputs()
        baseline = static_report(value)
        original = b"lowerword:6465726566".hex().encode("ascii")
        three_atoms = b"punctuation:64,punctuation:65,lowerword:726566".hex().encode("ascii")
        self.assertIn(original, baseline)
        changed = baseline.replace(original, three_atoms, 1)
        with self.assertRaisesRegex(RunnerError, "fixed expansion"):
            parse_report(changed, "static", value)
        wrong_kind = b"punctuation:6465726566".hex().encode("ascii")
        changed = baseline.replace(original, wrong_kind, 1)
        with self.assertRaisesRegex(RunnerError, "canonical atom partition and kinds"):
            parse_report(changed, "static", value)

    def test_malformed_common_hex_is_rejected(self) -> None:
        value = inputs()
        raw = static_report(value).replace(
            b"COVERAGE\tcurrent",
            b"RULE\tcurrent\tzz\t0\t1\nCOVERAGE\tcurrent",
            1,
        )
        with self.assertRaises(RunnerError):
            parse_report(raw, "static", value)

    def test_common_kind_fields_use_closed_vocabularies(self) -> None:
        value = inputs()
        mutants = (
            b"SURFACE\tcurrent\tunknown\t0\t1\t-",
            b"NODE\tcurrent\t65\t0\tunknown\t0\t1\t-",
            b"LEX\tcurrent\t-\t49\tunknown\t0\t1\t61",
        )
        for record in mutants:
            with self.subTest(record=record):
                raw = static_report(value).replace(
                    b"COVERAGE\tcurrent",
                    record + b"\nCOVERAGE\tcurrent",
                    1,
                )
                with self.assertRaisesRegex(RunnerError, "closed vocabulary"):
                    parse_report(raw, "static", value)

    def test_common_lexical_descriptors_are_closed_and_source_bound(self) -> None:
        value = inputs()
        lines = static_report(value).splitlines()
        index = next(
            index
            for index, line in enumerate(lines)
            if line.startswith(b"LEX\tcurrent\t") and b"\t4944454e54\tregex\t" in line
        )
        fields = lines[index].split(b"\t")
        fields[7] = b"bogus".hex().encode("ascii")
        lines[index] = b"\t".join(fields)
        with self.assertRaisesRegex(RunnerError, "lexical predicate|closed form"):
            parse_report(b"\n".join(lines) + b"\n", "static", value)

        lines = static_report(value).splitlines()
        index = next(
            index
            for index, line in enumerate(lines)
            if line.startswith(b"LEX\tproposal\t") and b"\t4944454e54\tregex\t" in line
        )
        fields = lines[index].split(b"\t")
        fields[6] = str(int(fields[5]) + len(b"IDENT `[a-z][a-z0-9_]*`")).encode("ascii")
        fields[7] = b"pattern=[a-z][a-z0-9_]*;exclude=none".hex().encode("ascii")
        lines[index] = b"\t".join(fields)
        with self.assertRaisesRegex(RunnerError, "omits.*source modifier"):
            parse_report(b"\n".join(lines) + b"\n", "static", value)

    def test_common_leaf_ledger_spans_equal_node_spans(self) -> None:
        value = inputs()
        baseline = static_report(value)
        selectors = (
            (b"NODE", b"66697865645f63616c6c", b"0.0"),
            (b"FIXED", b"66697865645f63616c6c", b"0.0"),
            (b"REF", b"66697865645f63616c6c", b"0.2"),
        )
        for tag, lhs, path in selectors:
            with self.subTest(tag=tag):
                lines = baseline.splitlines()
                index = next(
                    index
                    for index, line in enumerate(lines)
                    if line.startswith(tag + b"\tcurrent\t" + lhs + b"\t" + path + b"\t")
                )
                fields = lines[index].split(b"\t")
                end_index = 6 if tag == b"NODE" else 5
                fields[end_index] = str(int(fields[end_index]) + 1).encode("ascii")
                lines[index] = b"\t".join(fields)
                with self.assertRaisesRegex(RunnerError, "leaf.*disagrees|span"):
                    parse_report(b"\n".join(lines) + b"\n", "static", value)

    def test_common_rejects_overlapping_siblings_and_false_source_leaf(self) -> None:
        value = inputs()
        baseline = static_report(value)
        lines = baseline.splitlines()
        selected = []
        next_start = None
        for index, line in enumerate(lines):
            if line.startswith(b"NODE\tcurrent\t66697865645f63616c6c\t0.2\t"):
                next_start = int(line.split(b"\t")[5])
            if line.startswith(
                (b"NODE\tcurrent\t66697865645f63616c6c\t0.1\t", b"FIXED\tcurrent\t66697865645f63616c6c\t0.1\t")
            ):
                selected.append(index)
        self.assertIsNotNone(next_start)
        self.assertEqual(len(selected), 2)
        for index in selected:
            fields = lines[index].split(b"\t")
            fields[6 if fields[0] == b"NODE" else 5] = str(next_start + 1).encode("ascii")
            lines[index] = b"\t".join(fields)
        with self.assertRaisesRegex(RunnerError, "sibling spans overlap"):
            parse_report(b"\n".join(lines) + b"\n", "static", value)

        lines = baseline.splitlines()
        for index, line in enumerate(lines):
            fields = line.split(b"\t")
            if fields[:4] == [b"NODE", b"current", b"66697865645f63616c6c", b"0.0"]:
                fields[7] = b"defer".hex().encode("ascii")
                lines[index] = b"\t".join(fields)
            elif fields[:4] == [b"FIXED", b"current", b"66697865645f63616c6c", b"0.0"]:
                fields[6] = b"defer".hex().encode("ascii")
                fields[7] = b"lowerword:6465666572".hex().encode("ascii")
                lines[index] = b"\t".join(fields)
        with self.assertRaisesRegex(RunnerError, "quoted source spelling"):
            parse_report(b"\n".join(lines) + b"\n", "static", value)

    def test_common_rejects_production_lexical_namespace_overlap(self) -> None:
        value = inputs()
        lines = static_report(value).splitlines()
        for index, line in enumerate(lines):
            fields = line.split(b"\t")
            if fields[:2] == [b"LEX", b"current"] and fields[3] == b"IDENT".hex().encode("ascii"):
                fields[3] = b"call".hex().encode("ascii")
                lines[index] = b"\t".join(fields)
                break
        with self.assertRaisesRegex(RunnerError, "namespaces overlap"):
            parse_report(b"\n".join(lines) + b"\n", "static", value)

    def test_common_rejects_a_node_path_with_a_missing_prefix(self) -> None:
        value = inputs()
        baseline = static_report(value)
        changed = baseline.replace(
            b"NODE\tcurrent\t66697865645f63616c6c\t0.0\tfixed",
            b"NODE\tcurrent\t66697865645f63616c6c\t0.9.0\tfixed",
            1,
        )
        with self.assertRaisesRegex(RunnerError, "unrecorded prefix"):
            parse_report(changed, "static", value)

    def test_common_ledger_repeats_bound_semantic_limits(self) -> None:
        value = inputs()
        raw = static_report(value)
        for name in (
            "max_rules",
            "max_definitions",
            "max_grammar_nodes",
            "max_lexical_definitions",
            "max_terminal_occurrences",
            "max_symbol_bytes",
            "max_ebnf_depth",
        ):
            with self.subTest(name=name):
                limits = dict(value.limits)
                limits[name] = 1
                reduced = Inputs(value.sections, value.expectations, limits)
                with self.assertRaisesRegex(RunnerError, name):
                    parse_report(raw, "static", reduced)


if __name__ == "__main__":
    unittest.main()
