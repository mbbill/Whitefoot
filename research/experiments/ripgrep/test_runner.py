#!/usr/bin/env python3

from fractions import Fraction
import copy
import json
from pathlib import Path
import unittest

import runner


HERE = Path(__file__).resolve().parent


class FingerprintTests(unittest.TestCase):
    def test_lf_records_preserve_delimiters_and_final_fragment(self):
        self.assertEqual(
            [b"one\n", b"two"],
            runner.split_delimited_records(b"one\ntwo", b"\n"),
        )

    def test_trailing_delimiter_does_not_create_empty_record(self):
        self.assertEqual(
            [b"one\n", b"two\n"],
            runner.split_delimited_records(b"one\ntwo\n", b"\n"),
        )

    def test_file_blocks_ignore_only_block_order(self):
        left = runner.output_fingerprint(
            b"./a:1:1:one\n./a:2:1:two\n./b:1:1:three\n",
            "file_blocks",
        )
        right = runner.output_fingerprint(
            b"./b:1:1:three\n./a:1:1:one\n./a:2:1:two\n",
            "file_blocks",
        )
        self.assertEqual(left, right)

    def test_file_blocks_preserve_intra_file_order(self):
        ordered = runner.output_fingerprint(
            b"./a:1:1:one\n./a:2:1:two\n", "file_blocks"
        )
        reversed_lines = runner.output_fingerprint(
            b"./a:2:1:two\n./a:1:1:one\n", "file_blocks"
        )
        self.assertNotEqual(ordered["sha256"], reversed_lines["sha256"])

    def test_file_blocks_preserve_duplicates(self):
        once = runner.output_fingerprint(
            b"./a:1:1:one\n", "file_blocks"
        )
        twice = runner.output_fingerprint(
            b"./a:1:1:one\n./a:1:1:one\n", "file_blocks"
        )
        self.assertNotEqual(once["sha256"], twice["sha256"])
        self.assertNotEqual(once["records"], twice["records"])

    def test_file_blocks_reject_discontiguous_file(self):
        with self.assertRaises(runner.ProtocolError):
            runner.output_fingerprint(
                b"./a:1:1:one\n./b:1:1:two\n./a:2:1:three\n",
                "file_blocks",
            )

    def test_exact_mode_preserves_order(self):
        left = runner.output_fingerprint(b"a\nb\n", "exact")
        right = runner.output_fingerprint(b"b\na\n", "exact")
        self.assertNotEqual(left["sha256"], right["sha256"])

    def test_nul_records_use_same_multiset_rule(self):
        left = runner.output_fingerprint(b"a\0b\0", "nul_records")
        right = runner.output_fingerprint(b"b\0a\0", "nul_records")
        self.assertEqual(left, right)


class ScheduleAndStatisticsTests(unittest.TestCase):
    def test_arm_order_alternates_per_case(self):
        self.assertEqual(("official", "native"), runner.arm_order(0, 0))
        self.assertEqual(("native", "official"), runner.arm_order(1, 0))
        self.assertEqual(("native", "official"), runner.arm_order(0, 1))

    def test_rotation_is_deterministic(self):
        items = [{"id": "a"}, {"id": "b"}, {"id": "c"}]
        self.assertEqual(["b", "c", "a"],
                         [item["id"] for item in runner.rotated(items, 1)])

    def test_paired_bootstrap_is_deterministic(self):
        first = runner.paired_bootstrap_ratio(
            [200, 220, 210], [100, 110, 105], 1000, 17
        )
        second = runner.paired_bootstrap_ratio(
            [200, 220, 210], [100, 110, 105], 1000, 17
        )
        self.assertEqual(first, second)
        self.assertEqual(2.0, first["official_over_native"])

    def test_bootstrap_median_is_deterministic(self):
        first = runner.bootstrap_median([9, 10, 11], 1000, 19)
        second = runner.bootstrap_median([9, 10, 11], 1000, 19)
        self.assertEqual(first, second)
        self.assertEqual(10.0, first["median_ns"])


class GuardTests(unittest.TestCase):
    def test_placeholder_rejection_is_exact(self):
        with self.assertRaises(runner.ProtocolError):
            runner.reject_placeholders({"value": "TODO"})
        runner.reject_placeholders({"pattern": "TODO|FIXME|XXX"})

    def test_work_path_cannot_escape(self):
        with self.assertRaises(runner.ProtocolError):
            runner.resolve_work_path(Path("/work/root"), "../outside")

    def test_expected_mismatch_fails(self):
        with self.assertRaises(runner.ProtocolError):
            runner.expected_equal({"status": 0}, {"status": 1}, "test")


class ManifestShapeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.manifest = json.loads((HERE / "manifest.json").read_bytes())

    def test_manifest_has_nine_equal_weight_cases(self):
        cases = self.manifest["cases"]
        self.assertEqual(9, len(cases))
        self.assertEqual(9, len({case["id"] for case in cases}))
        weights = [
            Fraction(case["weight"]["numerator"], case["weight"]["denominator"])
            for case in cases
        ]
        self.assertEqual(Fraction(1, 1), sum(weights))
        self.assertEqual({Fraction(1, 9)}, set(weights))

    def test_only_official_and_native_are_contenders(self):
        contenders = {
            name
            for name, binary in self.manifest["binaries"].items()
            if binary["role"] == "contender"
        }
        self.assertEqual({"official", "native"}, contenders)

    def test_default_engine_and_warm_cache_are_explicit(self):
        self.assertIn("--engine=default", self.manifest["common_argv"])
        self.assertEqual(
            "warm-conditioned", self.manifest["measurement"]["cache_state"]
        )
        self.assertIn(
            "unavailable", self.manifest["measurement"]["cold_cache"]
        )
        self.assertEqual(0.95, self.manifest["measurement"]["confidence_level"])
        self.assertEqual(
            10000, self.manifest["measurement"]["bootstrap_samples"]
        )

    def test_every_timed_case_has_a_frozen_oracle(self):
        expected = {case["id"] for case in self.manifest["cases"]}
        self.assertEqual(expected, set(self.manifest["oracles"]["cases"]))
        for case in self.manifest["cases"]:
            oracle = self.manifest["oracles"]["cases"][case["id"]]
            self.assertEqual(0, oracle["status"])
            self.assertGreater(oracle["stdout"]["records"], 0)
            self.assertEqual(case["stdout_mode"], oracle["stdout"]["mode"])
            if case["stdout_mode"] == "file_blocks":
                self.assertGreater(oracle["stdout"]["blocks"], 0)

    def test_controls_cover_exit_one_and_two(self):
        statuses = {
            control["expected_status"] for control in self.manifest["controls"]
        }
        self.assertEqual({1, 2}, statuses)

    def test_phase_normalization_accepts_only_evidence_transitions(self):
        frozen = copy.deepcopy(self.manifest)
        runner.validate_phase_shape(frozen)

        selected = copy.deepcopy(frozen)
        selected["phase"] = "selected-before-baseline"
        selected["selection_evidence"] = {
            "path": "raw/selection.jsonl",
            "sha256": "selection-digest",
        }
        for case in selected["cases"]:
            case["selected_comparator"] = "official"
        runner.validate_phase_shape(selected)
        self.assertEqual(
            frozen,
            runner.normalized_frozen_manifest(selected),
        )

        complete = copy.deepcopy(selected)
        complete["phase"] = "complete"
        complete["baseline_evidence"] = {
            "path": "raw/baseline.jsonl",
            "sha256": "baseline-digest",
        }
        runner.validate_phase_shape(complete)
        self.assertEqual(
            selected,
            runner.normalized_selected_manifest(complete),
        )

    def test_unknown_phase_is_rejected(self):
        manifest = copy.deepcopy(self.manifest)
        manifest["phase"] = "complete "
        with self.assertRaises(runner.ProtocolError):
            runner.validate_phase_shape(manifest)


if __name__ == "__main__":
    unittest.main()
