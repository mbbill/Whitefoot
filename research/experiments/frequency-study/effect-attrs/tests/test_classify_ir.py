from __future__ import annotations

import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


HERE = Path(__file__).resolve().parent
ROOT = HERE.parent
FIXTURES = HERE / "fixtures"
sys.path.insert(0, str(ROOT))

import classify_ir  # noqa: E402


def host_clang_reads_the_fixture_dialect() -> str | None:
    """The reason the host clang cannot validate these fixtures, or `None`.

    Every fixture here is written in the `memory(...)` attribute dialect,
    because that attribute is what this experiment classifies. LLVM 16
    introduced it; a clang older than that rejects the fixtures at the parser
    with "expected top-level entity" and says nothing about whether they are
    well formed. Asking the compiler with a one-line probe rather than reading
    its version string keeps this a statement about what the toolchain
    understands.

    Found on the macOS gate runner in batch 0090, whose Xcode 15.4 ships Apple
    clang 15.
    """
    clang = shutil.which("clang")
    if clang is None:
        return "clang validates the LLVM fixture"
    with tempfile.TemporaryDirectory() as directory:
        probe = Path(directory) / "memory-attribute.ll"
        probe.write_text("declare void @probe() memory(none)\n")
        completed = subprocess.run(
            [
                clang,
                "-x",
                "ir",
                "-S",
                "-emit-llvm",
                str(probe),
                "-o",
                str(Path(directory) / "probe.out.ll"),
            ],
            check=False,
            capture_output=True,
        )
    if completed.returncode == 0:
        return None
    return f"{clang} predates the memory(...) attribute every fixture uses"


FIXTURE_DIALECT_LIMIT = host_clang_reads_the_fixture_dialect()


class FixtureMixin:
    def report(self) -> dict[str, object]:
        paths = [
            FIXTURES / "caller.ll",
            FIXTURES / "definitions-a.ll",
            FIXTURES / "definitions-b.ll",
        ]
        return classify_ir.run(paths, root=FIXTURES)

    def records(self) -> list[dict[str, object]]:
        return self.report()["records"]

    def record(self, symbol: str, occurrence: int = 0) -> dict[str, object]:
        matches = [record for record in self.records() if record["symbol"] == symbol]
        return matches[occurrence]


class ParseTests(FixtureMixin, unittest.TestCase):
    def test_empty_and_non_ir_inputs_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "not-ir.ll"
            for source in ("", "this is not LLVM IR\n"):
                path.write_text(source)
                with self.assertRaisesRegex(
                    classify_ir.ParseError, "no recognizable LLVM module"
                ):
                    classify_ir.parse_module(path)

    def test_parameter_readonly_is_not_a_function_attribute(self) -> None:
        module = classify_ir.parse_module(FIXTURES / "caller.ll", label="caller.ll")
        target = next(f for f in module.functions if f.symbol == "read_target")
        self.assertEqual(target.facts.memory.state, "absent")
        self.assertTrue(target.facts.nounwind)

    def test_memory_default_and_location_override_are_normalized(self) -> None:
        module = classify_ir.parse_module(
            FIXTURES / "definitions-a.ll", label="definitions-a.ll"
        )
        target = next(f for f in module.functions if f.symbol == "read_target")
        self.assertEqual(target.facts.memory.summary(), "read-only")
        self.assertEqual(
            dict(target.facts.memory.locations),
            {
                "argmem": "none",
                "inaccessiblemem": "read",
                "errnomem": "read",
                "other": "read",
            },
        )

    def test_quoted_attributes_do_not_contribute_facts_or_close_groups(self) -> None:
        declaration = self.record("fake_quoted_attrs")
        self.assertEqual(
            declaration["declaration_facts"]["memory"]["state"], "absent"
        )
        self.assertFalse(declaration["declaration_facts"]["nounwind"])
        self.assertEqual(declaration["disposition"], "strong-total-pure-gap")
        self.assertEqual(
            declaration["definition_facts"]["memory"]["summary"], "none"
        )

    def test_invoke_labels_bundles_and_metadata_do_not_contribute_call_facts(self) -> None:
        invoke, invoke_bundle = classify_ir._call_attribute_prefix(
            " nounwind memory(read, argmem: none) "
            "to label %willreturn unwind label %speculatable"
        )
        self.assertFalse(invoke_bundle)
        invoke_facts = classify_ir._effect_facts(invoke)
        self.assertTrue(invoke_facts.nounwind)
        self.assertFalse(invoke_facts.willreturn)
        self.assertFalse(invoke_facts.speculatable)
        self.assertEqual(invoke_facts.memory.summary(), "read-only")

        bundled, bundled_bundle = classify_ir._call_attribute_prefix(
            ' nounwind [ "willreturn"(i1 true) ], !speculatable !1'
        )
        self.assertTrue(bundled_bundle)
        bundled_facts = classify_ir._effect_facts(bundled)
        self.assertTrue(bundled_facts.nounwind)
        self.assertFalse(bundled_facts.willreturn)
        self.assertFalse(bundled_facts.speculatable)

    def test_missing_attribute_group_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "bad.ll"
            path.write_text(
                "declare i64 @f(i64)\n"
                "define i64 @caller() {\n"
                "  %x = call i64 @f(i64 1) #9\n"
                "  ret i64 %x\n"
                "}\n"
            )
            with self.assertRaises(classify_ir.ParseError):
                classify_ir.parse_module(path)


class ClassificationTests(FixtureMixin, unittest.TestCase):
    def test_callsite_and_declaration_facts_are_joined(self) -> None:
        with_callsite = self.record("pure_total", 0)
        bare_call = self.record("pure_total", 1)
        self.assertEqual(with_callsite["disposition"], "already-visible")
        self.assertEqual(with_callsite["callsite_facts"]["memory"]["summary"], "none")
        self.assertEqual(with_callsite["effective_caller_facts"]["tier"], "strong-total-pure")
        self.assertEqual(bare_call["disposition"], "strong-total-pure-gap")
        self.assertEqual(
            bare_call["missing_required_facts"],
            ["memory(none)", "nounwind", "willreturn"],
        )

        read_with_callsite = self.record("read_target", 0)
        read_bare = self.record("read_target", 1)
        self.assertEqual(read_with_callsite["disposition"], "already-visible")
        self.assertEqual(
            read_with_callsite["effective_caller_facts"]["memory"]["summary"],
            "read-only",
        )
        self.assertEqual(read_bare["disposition"], "read-only-total-gap")

    def test_willreturn_and_speculatable_are_never_equated(self) -> None:
        spec_only = self.record("spec_only")
        self.assertEqual(spec_only["disposition"], "definition-ineligible")
        self.assertFalse(spec_only["definition_facts"]["willreturn"])
        self.assertTrue(spec_only["definition_facts"]["speculatable"])
        self.assertIn("willreturn", spec_only["definition_eligibility_missing"])

        quoted = self.record('"quoted target"')
        self.assertEqual(quoted["disposition"], "definition-ineligible")
        self.assertTrue(quoted["definition_facts"]["speculatable"])
        self.assertFalse(quoted["definition_facts"]["willreturn"])

    def test_unsupported_absent_and_may_write_remain_distinct(self) -> None:
        unsupported_definition = self.record("unsupported_definition")
        self.assertEqual(
            unsupported_definition["disposition"], "definition-facts-unsupported"
        )
        self.assertEqual(
            unsupported_definition["definition_facts"]["memory"]["state"],
            "unsupported",
        )

        unsupported_caller = self.record("unsupported_caller")
        self.assertEqual(
            unsupported_caller["disposition"], "caller-facts-unsupported"
        )
        self.assertEqual(
            unsupported_caller["effective_caller_facts"]["memory"]["state"],
            "unsupported",
        )

        absent = self.record("legacy_pure")
        self.assertEqual(absent["effective_caller_facts"]["memory"]["state"], "absent")
        self.assertEqual(absent["disposition"], "strong-total-pure-gap")

        may_write = self.record("explicit_write_caller")
        self.assertEqual(
            may_write["effective_caller_facts"]["memory"]["summary"], "may-write"
        )
        self.assertEqual(may_write["disposition"], "strong-total-pure-gap")
        self.assertEqual(may_write["missing_required_facts"], ["memory(none)"])

    def test_linkage_filters_local_collisions_before_ambiguity(self) -> None:
        private_only = self.record("private_only")
        self.assertEqual(private_only["disposition"], "linkage-incompatible")
        self.assertEqual(
            private_only["incompatible_definitions"],
            [{"module": "definitions-a.ll", "linkage": "internal"}],
        )

        collision = self.record("external_with_internal_collision")
        self.assertEqual(collision["disposition"], "strong-total-pure-gap")
        self.assertEqual(collision["definition_module"], "definitions-a.ll")
        self.assertEqual(collision["definition_linkage"], "external")

        ambiguous = self.record("ambiguous")
        self.assertEqual(ambiguous["disposition"], "ambiguous-definition")

    def test_only_physical_calls_become_records(self) -> None:
        report = self.report()
        self.assertEqual(report["summary"]["ir_call_instructions"], 18)
        self.assertEqual(report["summary"]["unsupported_ir_call_instructions"], 1)
        self.assertEqual(report["summary"]["opaque_declaration_calls"], 17)
        self.assertEqual(len(report["records"]), 17)

    def test_report_is_byte_deterministic_across_input_order(self) -> None:
        paths = [
            FIXTURES / "definitions-b.ll",
            FIXTURES / "caller.ll",
            FIXTURES / "definitions-a.ll",
        ]
        first = classify_ir.run(paths, root=FIXTURES)
        second = classify_ir.run(list(reversed(paths)), root=FIXTURES)
        encoded_first = json.dumps(first, sort_keys=True, separators=(",", ":"))
        encoded_second = json.dumps(second, sort_keys=True, separators=(",", ":"))
        self.assertEqual(encoded_first, encoded_second)


class BlockerRegressionTests(unittest.TestCase):
    def test_valid_multiline_declaration_call_and_invoke_are_accumulated(self) -> None:
        paths = [
            FIXTURES / "multiline-caller.ll",
            FIXTURES / "multiline-definitions.ll",
        ]
        report = classify_ir.run(paths, root=FIXTURES)
        records = {record["symbol"]: record for record in report["records"]}
        self.assertEqual(records["multi_call"]["disposition"], "already-visible")
        self.assertEqual(records["multi_invoke"]["disposition"], "already-visible")
        self.assertEqual(
            records["multi_call"]["declaration_facts"]["memory"]["summary"],
            "none",
        )
        self.assertEqual(
            records["multi_invoke"]["callsite_facts"]["memory"]["summary"],
            "none",
        )
        self.assertEqual(report["summary"]["ir_call_instructions"], 3)
        self.assertEqual(report["summary"]["unsupported_ir_call_instructions"], 1)

    @unittest.skipIf(FIXTURE_DIALECT_LIMIT, FIXTURE_DIALECT_LIMIT or "")
    def test_multiline_probe_is_valid_llvm(self) -> None:
        # Parse and re-emit (-S -emit-llvm) rather than compile (-c): validity
        # of a fixture is a parse/verify property, and full code generation
        # drags in the host backend's capabilities. The unsupported-form fixture
        # carries a deliberate "deopt" operand bundle, and the wasi-sdk clang
        # a PATH may select crashes in WebAssembly instruction selection when
        # lowering it as a statepoint (LLVM 22.1.0-wasi-sdk); that is a
        # backend defect, not fixture invalidity. A malformed fixture still
        # fails here at the IR parser.
        with tempfile.TemporaryDirectory() as directory:
            for name in (
                "multiline-caller.ll",
                "multiline-definitions.ll",
                "unsupported-forms-caller.ll",
                "unsupported-forms-definitions.ll",
            ):
                clang = shutil.which("clang")
                completed = subprocess.run(
                    [
                        clang,
                        "-Wno-override-module",
                        "-x",
                        "ir",
                        "-S",
                        "-emit-llvm",
                        str(FIXTURES / name),
                        "-o",
                        str(Path(directory) / f"{name}.reemitted.ll"),
                    ],
                    check=False,
                    capture_output=True,
                    text=True,
                )
                # The compiler's own message is the whole diagnostic value of
                # this case: "returned non-zero exit status 1" says a fixture
                # is invalid without saying which line of it, and a reader on
                # another host cannot rerun the command.
                self.assertEqual(
                    completed.returncode,
                    0,
                    f"{name} did not parse under {clang}:\n{completed.stderr}",
                )

    def test_metadata_operand_bundle_and_inline_asm_fail_closed(self) -> None:
        paths = [
            FIXTURES / "unsupported-forms-caller.ll",
            FIXTURES / "unsupported-forms-definitions.ll",
        ]
        report = classify_ir.run(paths, root=FIXTURES)
        records = {record["symbol"]: record for record in report["records"]}
        bundled = records["bundle_target"]
        self.assertEqual(bundled["callsite_unsupported_reason"], "operand bundle")
        self.assertEqual(bundled["disposition"], "caller-facts-unsupported")

        metadata = records["metadata_target"]
        self.assertEqual(metadata["disposition"], "strong-total-pure-gap")
        self.assertFalse(metadata["callsite_facts"]["nounwind"])
        self.assertFalse(metadata["callsite_facts"]["willreturn"])
        self.assertEqual(metadata["callsite_facts"]["memory"]["state"], "absent")

        module = classify_ir.parse_module(
            FIXTURES / "unsupported-forms-caller.ll",
            label="unsupported-forms-caller.ll",
        )
        inline_asm = next(
            call
            for call in module.calls
            if call.unsupported_reason == "inline-asm callee"
        )
        self.assertIsNone(inline_asm.symbol)
        self.assertEqual(report["summary"]["unsupported_ir_call_instructions"], 2)

    def test_sigiled_function_metadata_do_not_contribute_attributes(self) -> None:
        facts = classify_ir._effect_facts("!readnone !nounwind !willreturn")
        self.assertEqual(facts.memory.state, "absent")
        self.assertFalse(facts.nounwind)
        self.assertFalse(facts.willreturn)

        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "metadata.ll"
            path.write_text(
                "declare i64 @target(i64), !readnone !0, !nounwind !0, !willreturn !0\n"
                "!0 = !{i32 1}\n"
            )
            module = classify_ir.parse_module(path)
            declaration = next(
                function
                for function in module.functions
                if function.symbol == "target"
            )
            self.assertEqual(declaration.facts.memory.state, "absent")
            self.assertFalse(declaration.facts.nounwind)
            self.assertFalse(declaration.facts.willreturn)

    def test_duplicate_resolved_paths_are_rejected(self) -> None:
        path = FIXTURES / "caller.ll"
        with self.assertRaises(classify_ir.ParseError):
            classify_ir.run([path, path])
        with tempfile.TemporaryDirectory() as directory:
            alias = Path(directory) / "alias.ll"
            alias.symlink_to(path)
            with self.assertRaises(classify_ir.ParseError):
                classify_ir.run([path, alias])

    def test_abi_signature_convention_and_address_space_mismatches_fail_closed(self) -> None:
        paths = [FIXTURES / "abi-caller.ll", FIXTURES / "abi-definitions.ll"]
        report = classify_ir.run(paths, root=FIXTURES)
        records = {record["symbol"]: record for record in report["records"]}
        self.assertEqual(records["type_mismatch"]["disposition"], "abi-mismatch")
        self.assertEqual(records["type_mismatch"]["abi_mismatches"], ["return-type"])
        self.assertEqual(
            records["convention_mismatch"]["abi_mismatches"],
            ["calling-convention"],
        )
        self.assertEqual(
            records["address_mismatch"]["abi_mismatches"], ["address-space"]
        )
        self.assertEqual(
            records["unsupported_signature"]["disposition"], "abi-unsupported"
        )
        self.assertEqual(records["call_mismatch"]["disposition"], "abi-mismatch")
        self.assertEqual(
            records["call_mismatch"]["abi_mismatches"],
            ["call-parameter-types", "call-return-type"],
        )


if __name__ == "__main__":
    unittest.main()
