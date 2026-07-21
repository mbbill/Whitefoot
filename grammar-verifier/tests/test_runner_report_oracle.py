from __future__ import annotations

import unittest

from support_common import inputs
from support_oracle import oracle_report
from runner_inputs import BoundBytes, Inputs, RunnerError
from runner_report import parse_report
from runner_trace import _maximum_trace_depth


class OracleReportTests(unittest.TestCase):
    def test_trace_depth_bound_allows_long_production_alias_chains(self) -> None:
        value = inputs()
        old_unsound_bound = (
            value.limits["oracle_max_source_tokens"]
            + value.limits["max_ebnf_depth"]
            + 2
        )
        self.assertGreater(_maximum_trace_depth(10_000, value), old_unsound_bound)

    def test_orphan_and_incomplete_oracle_traces_are_rejected(self) -> None:
        value = inputs()
        baseline = oracle_report(value)
        orphan = baseline.replace(
            b"CASE-TRACE\tcurrent\t64657265662d70\t0",
            b"CASE-TRACE\tcurrent\t756e6b6e6f776e\t0",
            1,
        )
        with self.assertRaisesRegex(RunnerError, "orphan"):
            parse_report(orphan, "oracle", value)
        missing = baseline.replace(
            next(line + b"\n" for line in baseline.splitlines() if line.startswith(b"STREAM-TRACE\tcurrent")),
            b"",
            1,
        )
        with self.assertRaisesRegex(RunnerError, "incomplete"):
            parse_report(missing, "oracle", value)

    def test_oracle_rejects_duplicate_stream_derivations(self) -> None:
        value = inputs()
        lines = oracle_report(value).splitlines()
        trace_index = next(
            index for index, line in enumerate(lines) if line.startswith(b"STREAM-TRACE\tcurrent")
        )
        stream_index = next(
            index for index, line in enumerate(lines) if line.startswith(b"STREAM\tcurrent")
        )
        stream_fields = lines[stream_index].split(b"\t")
        stream_fields[-2:] = (b"many", b"2")
        lines[stream_index] = b"\t".join(stream_fields)
        duplicate = lines[trace_index].split(b"\t")
        duplicate[4] = b"1"
        lines.insert(trace_index + 1, b"\t".join(duplicate))
        with self.assertRaisesRegex(RunnerError, "duplicate stream derivation"):
            parse_report(b"\n".join(lines) + b"\n", "oracle", value)

    def test_oracle_domain_and_metrics_are_resource_bound(self) -> None:
        value = inputs()
        baseline = oracle_report(value)
        huge = b"9" * 1000
        domain_prefix = (
            b"DOMAIN\tcurrent\t66697865642d6c6f776572776f72642d63616c6c73"
            b"\t65787072\t78\t1\t"
        )
        changed = baseline.replace(domain_prefix, domain_prefix[:-2] + huge + b"\t", 1)
        with self.assertRaisesRegex(RunnerError, "u64"):
            parse_report(changed, "oracle", value)
        wrong_stream_count = baseline.replace(b"METRIC\tcurrent\t3\t", b"METRIC\tcurrent\t2\t", 1)
        with self.assertRaisesRegex(RunnerError, "authored workload"):
            parse_report(wrong_stream_count, "oracle", value)
        excessive_tokens = baseline.replace(b"METRIC\tcurrent\t3\t8\t", b"METRIC\tcurrent\t3\t769\t", 1)
        with self.assertRaisesRegex(RunnerError, "per-stream resource bound"):
            parse_report(excessive_tokens, "oracle", value)

    def test_oracle_case_delta_must_classify_the_trace_union(self) -> None:
        value = inputs()
        baseline = oracle_report(value)
        missing = baseline.replace(
            next(line + b"\n" for line in baseline.splitlines() if line.startswith(b"CASE-DELTA")),
            b"",
            1,
        )
        with self.assertRaisesRegex(RunnerError, "complete trace union"):
            parse_report(missing, "oracle", value)

    def test_oracle_trace_decoder_rejects_malformed_trees_and_spans(self) -> None:
        value = inputs()
        baseline = oracle_report(value)
        line = next(line for line in baseline.splitlines() if line.startswith(b"CASE-TRACE"))
        fields = line.split(b"\t")
        trace = bytes.fromhex(fields[-1].decode("ascii"))
        malformed = []
        malformed.append(trace[:-1])
        bad_length = bytearray(trace)
        bad_length[1:5] = (len(trace) + 1).to_bytes(4, "big")
        malformed.append(bytes(bad_length))
        malformed.append(trace.replace(b"production:", b"xxxxxxxxxxx", 1))
        impossible_path = trace.replace(b":0:choice:", b":9:choice:", 1)
        self.assertNotEqual(impossible_path, trace)
        malformed.append(impossible_path)
        changed_span = trace.replace(b":0:5", b":0:9", 1)
        self.assertNotEqual(changed_span, trace)
        malformed.append(changed_span)
        for mutant in malformed:
            with self.subTest(mutant=mutant[:20]):
                changed_fields = list(fields)
                changed_fields[-1] = mutant.hex().encode("ascii")
                changed = baseline.replace(line, b"\t".join(changed_fields), 1)
                with self.assertRaisesRegex(RunnerError, "trace|token"):
                    parse_report(changed, "oracle", value)

    def test_oracle_trace_token_count_obeys_a_reduced_limit(self) -> None:
        value = inputs()
        limits = dict(value.limits)
        limits["oracle_max_source_tokens"] = 3
        reduced = Inputs(value.sections, value.expectations, limits)
        with self.assertRaisesRegex(RunnerError, "source-token bound"):
            parse_report(oracle_report(value), "oracle", reduced)

    def test_oracle_expectation_mismatch_is_rejected(self) -> None:
        value = inputs()
        changed = BoundBytes(
            "expectations",
            value.expectations.data.replace(
                b"case\tderef-p\tcurrent\tmany",
                b"case\tderef-p\tcurrent\tone",
                1,
            ),
        )
        altered = Inputs(value.sections, changed, value.limits)
        with self.assertRaisesRegex(RunnerError, "expectations"):
            parse_report(oracle_report(value), "oracle", altered)

    def test_oracle_trace_delta_policy_is_enforced(self) -> None:
        value = inputs()
        changed = BoundBytes(
            "expectations",
            value.expectations.data.replace(
                b"case-delta\tderef-p\ttrace-subset",
                b"case-delta\tderef-p\ttrace-replacement",
                1,
            ),
        )
        altered = Inputs(value.sections, changed, value.limits)
        with self.assertRaisesRegex(RunnerError, "trace-replacement"):
            parse_report(oracle_report(value), "oracle", altered)


if __name__ == "__main__":
    unittest.main()
