from __future__ import annotations

import unittest

from support_common import inputs
from support_static import static_report
from runner_inputs import RunnerError
from runner_report import parse_report
from runner_report_wire import witness_stream_hex


class StaticReportTests(unittest.TestCase):
    def test_every_static_record_field_is_closed(self) -> None:
        value = inputs()
        mutations = (
            (b"STATIC-NULLABLE\tcurrent\t65787072\t0", b"STATIC-NULLABLE\tcurrent\t65787072\t2"),
            (b"STATIC-FIRST\tcurrent\t65787072", b"STATIC-FIRST\tcurrent\tzz"),
            (b"STATIC-FOLLOW\tcurrent\t65787072", b"STATIC-FOLLOW\tcurrent\tzz"),
            (
                b"STATIC-INTERSECTION\tcurrent\t" + b"fixed:6465726566".hex().encode("ascii"),
                b"STATIC-INTERSECTION\tcurrent\tzz",
            ),
            (b"STATIC-DECISION\tcurrent\t65787072\t0", b"STATIC-DECISION\tcurrent\t65787072\tx"),
            (b"STATIC-CONFLICT\tcurrent\t65787072\t0", b"STATIC-CONFLICT\tcurrent\t65787072\tx"),
            (b"STATIC-DELTA\tconflict\tremoved", b"STATIC-DELTA\tunknown\tremoved"),
            (b"STATIC-CASE\tcurrent\tderef-p\t65787072", b"STATIC-CASE\tcurrent\tderef-p\tzz"),
            (b"STATIC-DOMAIN\tcurrent\t", b"STATIC-DOMAIN\tcurrent\tzz"),
            (b"STATIC-TRANSITION\tfixed-ident-partition\t1", b"STATIC-TRANSITION\tfixed-ident-partition\tx"),
        )
        baseline = static_report(value)
        for old, new in mutations:
            with self.subTest(old=old):
                self.assertIn(old, baseline)
                with self.assertRaises(RunnerError):
                    parse_report(baseline.replace(old, new, 1), "static", value)

    def test_static_delta_must_be_complete_and_introduction_free(self) -> None:
        value = inputs()
        baseline = static_report(value)
        missing = baseline.replace(
            next(line + b"\n" for line in baseline.splitlines() if line.startswith(b"STATIC-DELTA\tconflict")),
            b"",
            1,
        )
        with self.assertRaisesRegex(RunnerError, "complete evidence union"):
            parse_report(missing, "static", value)
        introduced = baseline.replace(b"STATIC-DELTA\tconflict\tremoved", b"STATIC-DELTA\tconflict\tintroduced", 1)
        with self.assertRaisesRegex(RunnerError, "complete evidence union|permits no introduced"):
            parse_report(introduced, "static", value)

    def test_proposal_must_have_zero_strong_ll2_conflicts(self) -> None:
        value = inputs()
        lines = static_report(value).splitlines()
        conflict_index = next(
            index
            for index, line in enumerate(lines)
            if line.startswith(b"STATIC-CONFLICT\tcurrent")
        )
        proposal_conflict = lines[conflict_index].replace(
            b"STATIC-CONFLICT\tcurrent",
            b"STATIC-CONFLICT\tproposal",
            1,
        )
        lines.insert(conflict_index + 1, proposal_conflict)
        delta_index = next(
            index
            for index, line in enumerate(lines)
            if line.startswith(b"STATIC-DELTA\tconflict\tremoved")
        )
        lines[delta_index] = lines[delta_index].replace(
            b"STATIC-DELTA\tconflict\tremoved",
            b"STATIC-DELTA\tconflict\tretained",
            1,
        )
        with self.assertRaisesRegex(RunnerError, "zero strong-LL\\(2\\)"):
            parse_report(b"\n".join(lines) + b"\n", "static", value)

    def test_static_core_analysis_coverage_cannot_be_omitted(self) -> None:
        value = inputs()
        removed_tags = (
            b"STATIC-NULLABLE\t",
            b"STATIC-FIRST\t",
            b"STATIC-FOLLOW\t",
            b"STATIC-DECISION\t",
            b"STATIC-CONFLICT\t",
        )
        lines = [
            line
            for line in static_report(value).splitlines()
            if not line.startswith(removed_tags)
            and not line.startswith(b"STATIC-DELTA\tconflict\t")
        ]
        with self.assertRaisesRegex(RunnerError, "coverage is incomplete"):
            parse_report(b"\n".join(lines) + b"\n", "static", value)

    def test_static_conflict_and_intersection_identities_are_bound(self) -> None:
        value = inputs()
        baseline = static_report(value)
        lines = baseline.splitlines()
        conflict_index = next(index for index, line in enumerate(lines) if line.startswith(b"STATIC-CONFLICT"))
        conflict = lines[conflict_index].split(b"\t")
        conflict[6] = b"2"
        lines[conflict_index] = b"\t".join(conflict)
        with self.assertRaisesRegex(RunnerError, "decision arms"):
            parse_report(b"\n".join(lines) + b"\n", "static", value)
        lines = baseline.splitlines()
        intersection_index = next(
            index
            for index, line in enumerate(lines)
            if line.startswith(b"STATIC-INTERSECTION\tcurrent")
            and b"66697865643a36343635373236353636" in line
            and b"6c65783a34393434343534653534" in line
        )
        intersection = lines[intersection_index].split(b"\t")
        intersection[2], intersection[3] = intersection[3], intersection[2]
        lines[intersection_index] = b"\t".join(intersection)
        with self.assertRaisesRegex(RunnerError, "canonically ordered"):
            parse_report(b"\n".join(lines) + b"\n", "static", value)

    def test_static_conflict_witness_bounds_and_eof_order(self) -> None:
        self.assertEqual(witness_stream_hex(b"2d"), (b"-",))
        self.assertEqual(witness_stream_hex(b"37382c2d"), (b"78", b"-"))
        self.assertEqual(witness_stream_hex(b"2d2c2d"), (b"-", b"-"))
        for malformed in (b"2d2c3738", b"37382c37392c3761"):
            with self.subTest(malformed=malformed):
                with self.assertRaises(RunnerError):
                    witness_stream_hex(malformed)

        value = inputs()
        lines = static_report(value).splitlines()
        index = next(index for index, line in enumerate(lines) if line.startswith(b"STATIC-CONFLICT"))
        fields = lines[index].split(b"\t")
        fields[-1] = b"2d"
        lines[index] = b"\t".join(fields)
        with self.assertRaisesRegex(RunnerError, "arity"):
            parse_report(b"\n".join(lines) + b"\n", "static", value)

    def test_static_transition_is_bound_to_the_exact_case(self) -> None:
        value = inputs()
        baseline = static_report(value)
        wrong_witness = baseline.replace(b"6465726566287829\nSTATIC-END", b"6465726566\nSTATIC-END", 1)
        with self.assertRaisesRegex(RunnerError, "exact authored case"):
            parse_report(wrong_witness, "static", value)
        wrong_count = baseline.replace(
            b"STATIC-CASE\tcurrent\tderef-x\t65787072\t6465726566287829\t1",
            b"STATIC-CASE\tcurrent\tderef-x\t65787072\t6465726566287829\t2",
            1,
        )
        with self.assertRaisesRegex(RunnerError, "exact authored case"):
            parse_report(wrong_count, "static", value)


if __name__ == "__main__":
    unittest.main()
