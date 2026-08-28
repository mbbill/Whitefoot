#!/usr/bin/env python3

from __future__ import annotations

import shutil
import subprocess
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from io import StringIO
from pathlib import Path

import collect


# The target the recorded fingerprint was taken on. The fingerprint counts
# LLVM's own runtime alias-check shape -- 26 conflict predicates, 52 pointer
# comparisons -- and that shape is a vectorizer decision made per target. It
# is stable across rustc versions on the calibrated target: measured in batch
# 0090 under rustc 1.96.0, 1.97.1 and 1.98.0, all three reproduce it exactly.
# It is not the same number on another target: an x86-64 Linux runner produces
# a different, equally correct count, which this fingerprint cannot judge and
# must not be edited to accept.
CALIBRATED_HOST = "aarch64-apple-darwin"


def rustc_host() -> str | None:
    """The target triple this host's rustc compiles for, or None if absent."""
    rustc = shutil.which("rustc")
    if rustc is None:
        return None
    completed = subprocess.run(
        [rustc, "--version", "--verbose"],
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
    )
    if completed.returncode != 0:
        return None
    for line in completed.stdout.splitlines():
        if line.startswith("host: "):
            return line[len("host: ") :].strip()
    return None


class AliasVersioningAnalyzerTests(unittest.TestCase):
    def test_token_without_vector_cfg_is_rejected(self) -> None:
        ir = """
define void @decoy() {
entry:
  br label %vector.memcheck
vector.memcheck:
  br label %exit
exit:
  ret void
}
"""
        result = collect.analyze_ir(ir)
        self.assertEqual(result["raw_vector_memcheck_block_count"], 1)
        self.assertEqual(result["validated_alias_versioned_loop_count"], 0)
        self.assertEqual(
            result["rejected_memchecks"][0]["reason"], "no-reachable-vector-body"
        )

    def test_valid_cfg_counts_predicates(self) -> None:
        ir = """
define void @probe(ptr %a, ptr %b) {
entry:
  br label %vector.memcheck
vector.memcheck:
  %bound0 = icmp ult ptr %a, %b
  %bound1 = icmp ult ptr %b, %a
  %found.conflict = and i1 %bound0, %bound1
  br i1 %found.conflict, label %scalar, label %vector.ph
vector.ph:
  br label %vector.body
vector.body:
  br i1 false, label %exit, label %vector.body
scalar:
  br label %exit
exit:
  ret void
}
"""
        result = collect.analyze_ir(ir)
        self.assertEqual(result["validated_alias_versioned_loop_count"], 1)
        self.assertEqual(result["conflict_predicate_count"], 1)
        self.assertEqual(result["pointer_comparison_count"], 2)
        self.assertEqual(result["loops"][0]["vector_body_blocks"], ["vector.body"])

    def test_comment_decoys_do_not_supply_cfg_debug_or_predicates(self) -> None:
        ir = """
source_filename = "decoys.ll"
define void @decoy(ptr %a, ptr %b) {
entry:
  br label %vector.memcheck
vector.memcheck:
  ; %comment.cmp = icmp ult ptr %a, %b
  ; %found.conflict.comment = and i1 true, true
  br label %exit ; label %vector.body, !dbg !10
; vector.body:
;   ret void
exit:
  ret void
}
!10 = !DILocation(line: 99, column: 7, scope: !11)
!11 = distinct !DISubprogram(name: "decoy", file: !12)
!12 = !DIFile(filename: "decoy.rs", directory: "/tmp")
"""
        result = collect.analyze_ir(ir)
        self.assertEqual(result["raw_vector_memcheck_block_count"], 1)
        self.assertEqual(result["validated_alias_versioned_loop_count"], 0)
        self.assertEqual(result["conflict_predicate_count"], 0)
        self.assertEqual(result["pointer_comparison_count"], 0)

    def test_comment_debug_attachment_is_not_selected(self) -> None:
        ir = """
source_filename = "debug-decoy.ll"
define void @probe() {
entry:
  br label %vector.memcheck
vector.memcheck:
  ; %comment.cmp = icmp ult ptr null, null
  ; %found.conflict.comment = and i1 true, true
  br label %vector.body ; !dbg !10
vector.body:
  ret void
}
!10 = !DILocation(line: 99, column: 7, scope: !11)
!11 = distinct !DISubprogram(name: "probe", file: !12)
!12 = !DIFile(filename: "probe.rs", directory: "/tmp")
"""
        result = collect.analyze_ir(ir)
        self.assertEqual(result["validated_alias_versioned_loop_count"], 1)
        self.assertEqual(result["conflict_predicate_count"], 0)
        self.assertEqual(result["pointer_comparison_count"], 0)
        self.assertIsNone(result["loops"][0]["debug_location"])

    def test_semicolon_in_quoted_label_does_not_start_comment(self) -> None:
        ir = """
source_filename = "quoted-label.ll"
define void @probe() {
entry:
  br label %vector.memcheck
vector.memcheck:
  br label %"through;semi" ; label %comment.decoy
"through;semi":
  br label %vector.body
vector.body:
  ret void
}
"""
        result = collect.analyze_ir(ir)
        self.assertEqual(result["validated_alias_versioned_loop_count"], 1)
        self.assertEqual(result["loops"][0]["successors"], ["through;semi"])

    def test_cfg_deeper_than_128_blocks_is_traversed(self) -> None:
        blocks = [
            "define void @deep() {",
            "entry:",
            "  br label %vector.memcheck",
            "vector.memcheck:",
            "  br label %hop0",
        ]
        for index in range(129):
            successor = f"hop{index + 1}" if index < 128 else "vector.body"
            blocks.extend([f"hop{index}:", f"  br label %{successor}"])
        blocks.extend(["vector.body:", "  ret void", "}"])
        result = collect.analyze_ir("\n".join(blocks))
        self.assertEqual(result["validated_alias_versioned_loop_count"], 1)
        self.assertEqual(result["loops"][0]["vector_body_blocks"], ["vector.body"])

    def test_invalid_text_is_not_clean_zero_evidence(self) -> None:
        for ir in (
            "",
            "; comments only\n",
            "this is not LLVM IR\n",
            'source_filename = "decoy.ll"\nstill plain prose\n',
            "source_filename = not_a_quoted_filename\n",
        ):
            with self.subTest(ir=ir), self.assertRaises(ValueError):
                collect.analyze_ir(ir)

    def test_zero_function_llvm_module_is_accepted(self) -> None:
        result = collect.analyze_ir(
            'source_filename = "declarations.ll"\n'
            'target triple = "x86_64-unknown-linux-gnu"\n'
            "declare void @external()\n"
        )
        self.assertEqual(result["llvm_function_count"], 0)
        self.assertEqual(result["validated_alias_versioned_loop_count"], 0)

    def test_recognized_top_level_ir_between_functions_is_accepted(self) -> None:
        ir = """
source_filename = "two-functions.ll"
define void @first() #0 {
entry:
  ret void
}
attributes #0 = { nounwind }
declare void @external()
define void @second() {
entry:
  ret void
}
!0 = !{i32 1, !"flag", i32 1}
"""
        result = collect.analyze_ir(ir)
        self.assertEqual(result["llvm_function_count"], 2)

    def test_malformed_function_structure_is_rejected(self) -> None:
        cases = {
            "truncated": """
define void @truncated() {
entry:
  ret void
""",
            "nested": """
define void @outer() {
entry:
  ret void
define void @inner() {
entry:
  ret void
}
}
""",
            "duplicate-label": """
define void @duplicate() {
entry:
  br label %same
same:
  br label %same
same:
  ret void
}
""",
        }
        expected_messages = {
            "truncated": "missing closing",
            "nested": "nested function definition",
            "duplicate-label": "duplicate block label",
        }
        for name, ir in cases.items():
            with self.subTest(name=name), self.assertRaises(ValueError) as raised:
                collect.analyze_ir(ir)
            self.assertIn(expected_messages[name], str(raised.exception))

    def test_unconsumed_top_level_prose_is_rejected(self) -> None:
        function = """define void @first() {
entry:
  ret void
}
"""
        second = """define void @second() {
entry:
  ret void
}
"""
        for ir in (
            function + "trailing prose\n",
            function + "between prose\n" + second,
        ):
            with self.subTest(ir=ir), self.assertRaises(ValueError) as raised:
                collect.analyze_ir(ir)
            self.assertIn("unrecognized top-level LLVM IR", str(raised.exception))

    def test_calibration_match_requires_entire_fingerprint(self) -> None:
        analysis = {
            "raw_vector_memcheck_block_count": 2,
            "validated_alias_versioned_loop_count": 2,
            "first_party_alias_versioned_loop_count": 2,
            "rejected_memchecks": [],
            "conflict_predicate_count": 26,
            "pointer_comparison_count": 52,
        }
        self.assertTrue(collect._calibration_result(analysis)["matches_expected"])

        analysis_keys = {
            "raw_vector_memcheck_block_count": "raw_vector_memcheck_block_count",
            "validated_alias_versioned_loop_count": "validated_alias_versioned_loop_count",
            "first_party_alias_versioned_loop_count": (
                "first_party_alias_versioned_loop_count"
            ),
            "conflict_predicate_count": "conflict_predicate_count",
            "pointer_comparison_count": "pointer_comparison_count",
        }
        for fingerprint_key, analysis_key in analysis_keys.items():
            drifted = dict(analysis)
            drifted[analysis_key] = int(drifted[analysis_key]) + 1
            with self.subTest(field=fingerprint_key):
                self.assertFalse(
                    collect._calibration_result(drifted)["matches_expected"]
                )

        rejected = dict(analysis)
        rejected["rejected_memchecks"] = [{"reason": "calibration-decoy"}]
        self.assertFalse(collect._calibration_result(rejected)["matches_expected"])

    def test_cli_invalid_ir_exits_two_without_json(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "not-ir.txt"
            path.write_text("plain prose\n")
            stdout = StringIO()
            stderr = StringIO()
            with redirect_stdout(stdout), redirect_stderr(stderr):
                status = collect.main(["--ir", str(path)])
        self.assertEqual(status, 2)
        self.assertEqual(stdout.getvalue(), "")
        self.assertIn("unrecognized top-level LLVM IR", stderr.getvalue())

    def test_cli_malformed_function_ir_exits_two(self) -> None:
        malformed_inputs = (
            "define void @truncated() {\nentry:\n  ret void\n",
            """define void @outer() {
entry:
  ret void
define void @inner() {
entry:
  ret void
}
}
""",
            """define void @duplicate() {
entry:
  br label %same
same:
  br label %same
same:
  ret void
}
""",
        )
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "malformed.ll"
            for ir in malformed_inputs:
                path.write_text(ir)
                stdout = StringIO()
                stderr = StringIO()
                with redirect_stdout(stdout), redirect_stderr(stderr):
                    status = collect.main(["--ir", str(path)])
                with self.subTest(ir=ir):
                    self.assertEqual(status, 2)
                    self.assertEqual(stdout.getvalue(), "")
                    self.assertIn("error:", stderr.getvalue())

    @unittest.skipUnless(shutil.which("rustc"), "rustc is required for calibration")
    @unittest.skipUnless(
        rustc_host() == CALIBRATED_HOST,
        f"the recorded fingerprint is the memcheck shape LLVM emits for "
        f"{CALIBRATED_HOST}; another target vectorizes these kernels "
        f"differently and this fingerprint cannot judge that count",
    )
    def test_existing_scoped_alias_fixture_calibrates(self) -> None:
        report = collect.calibration_report()
        calibration = report["calibration"]
        self.assertTrue(calibration["matches_expected"])
        self.assertEqual(
            calibration["expected_fingerprint"],
            collect.EXPECTED_CALIBRATION_FINGERPRINT,
        )
        self.assertEqual(
            calibration["observed_fingerprint"],
            collect.EXPECTED_CALIBRATION_FINGERPRINT,
        )
        self.assertEqual(
            calibration["observed_first_party_alias_versioned_loop_count"], 2
        )
        analysis = report["analysis"]
        self.assertEqual(analysis["raw_vector_memcheck_block_count"], 2)
        self.assertEqual(analysis["conflict_predicate_count"], 26)
        self.assertEqual(analysis["pointer_comparison_count"], 52)
        self.assertEqual(len(analysis["rejected_memchecks"]), 0)
        for loop in analysis["loops"]:
            self.assertTrue(loop["first_party"])
            location = loop["debug_location"]
            self.assertEqual(
                location["path"],
                "experiments/scoped-alias-channel/rust_kernels.rs",
            )


if __name__ == "__main__":
    unittest.main()
