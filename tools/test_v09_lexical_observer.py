#!/usr/bin/env python3
"""Hostile codec and Rust/model differential tests for the v0.9 observer."""

from __future__ import annotations

import contextlib
import hashlib
import json
import os
import random
import subprocess
import sys
import tempfile
import unittest
from dataclasses import dataclass
from pathlib import Path
from unittest import mock

import v09_lexical_model as model
import v09_lexical_observer as observer
import v09_lexical_observer_runner as observer_runner


ROOT = Path(__file__).resolve().parents[1]
COMPILER_TOOLS = ROOT / "compiler" / "tools"
sys.path.insert(0, str(COMPILER_TOOLS))
import cargo_policy  # noqa: E402


SOURCE_LIMITS = observer.SourceLimits(
    max_sources=4_096,
    max_logical_path_bytes=4_096,
    max_source_bytes=1_048_576,
    max_total_source_bytes=1_048_576,
    max_binding_bytes=2_097_152,
)
LEX_LIMITS = model.LexLimits(
    max_sources=4_096,
    max_source_bytes=1_048_576,
    max_total_source_bytes=1_048_576,
    max_token_bytes=1_048_576,
    max_tokens=1_048_576,
    max_lexemes=1_048_576,
)
FIXTURES = ROOT / "frontend-corpus" / "v0.9" / "lexical-fixtures.json"


@dataclass(frozen=True)
class DifferentialCase:
    """One immutable model-independent differential input."""

    identifier: str
    sources: tuple[bytes, ...]
    limits: model.LexLimits


def build_differential_cases() -> tuple[DifferentialCase, ...]:
    """Freeze authored and generated inputs without consulting either lexer."""

    cases: list[DifferentialCase] = []
    fixture = json.loads(FIXTURES.read_text(encoding="utf-8"))
    defaults = fixture["default_limits"]
    for case in fixture["cases"]:
        cases.append(
            DifferentialCase(
                f"authored/{case['id']}",
                tuple(bytes.fromhex(value) for value in case["sources_hex"]),
                model.LexLimits(**(defaults | case.get("limits", {}))),
            )
        )

    for byte in range(256):
        cases.extend(
            (
                DifferentialCase(
                    f"byte/top/{byte:02x}", (bytes((byte,)),), LEX_LIMITS
                ),
                DifferentialCase(
                    f"byte/string/{byte:02x}",
                    (bytes((0x22, byte, 0x22)),),
                    LEX_LIMITS,
                ),
                DifferentialCase(
                    f"byte/escape/{byte:02x}",
                    (bytes((0x22, 0x5C, byte, 0x22)),),
                    LEX_LIMITS,
                ),
            )
        )

    utf8_seams = (
        b"\xc2",
        b"\xc2\x80",
        b"\xdf\xbf",
        b"\xe0\x9f\xbf",
        b"\xe0\xa0\x80",
        b"\xed\x9f\xbf",
        b"\xed\xa0\x80",
        b"\xef\xbf\xbf",
        b"\xf0\x8f\xbf\xbf",
        b"\xf0\x90\x80\x80",
        b"\xf4\x8f\xbf\xbf",
        b"\xf4\x90\x80\x80",
        b"\xf0\x9f\x92",
        b"\xf0\x9f\x92\xa9",
        b"\x80",
        b"\xc0\xaf",
        b"\xe2(\xa1",
    )
    for ordinal, exact in enumerate(utf8_seams):
        cases.append(DifferentialCase(f"utf8/{ordinal:02d}", (exact,), LEX_LIMITS))

    for suffix in (b"checked", b"strict", b"wrap", b"trap", b"sat"):
        for continuation in (b"", b"x", b"0", b"_", b".", b" ", b"\n"):
            identifier = f"operation/{suffix.decode('ascii')}/{continuation.hex()}"
            cases.append(
                DifferentialCase(identifier, (b"x." + suffix + continuation,), LEX_LIMITS)
            )

    numeric_and_arrow_seams = (
        b"-",
        b"->",
        b"=>",
        b"==>",
        b"--1",
        b"-x",
        b"-0_i32",
        b"-2147483648_i32",
        b"01_i32",
        b"1._f64",
        b"1.0e_f64",
        b"1.0e+_f64",
        b"1.0e+2_f64",
        b"1.0E-2_f64",
        b"1..2",
        b"1e+2x",
    )
    for ordinal, exact in enumerate(numeric_and_arrow_seams):
        cases.append(DifferentialCase(f"number-arrow/{ordinal:02d}", (exact,), LEX_LIMITS))

    source_boundary_cases = (
        (),
        (b"",),
        (b"", b"", b""),
        (b"foo.", b"wrap"),
        (b"foo", b".", b"wrap"),
        (b"-", b"1_i32"),
        (b"=", b">", b"@9"),
        (b"\"", b"\""),
        (b"@9", b"valid", b"\xff"),
        (b"\xff", b"valid", b"@9"),
        (b"same", b"same"),
    )
    for ordinal, sources in enumerate(source_boundary_cases):
        cases.append(
            DifferentialCase(f"source-boundary/{ordinal:02d}", sources, LEX_LIMITS)
        )

    limit_cases = (
        ((b"", b""), "max_sources", 1, 2),
        ((b"ab",), "max_source_bytes", 1, 2),
        ((b"a", b"b"), "max_total_source_bytes", 1, 2),
        ((b"ab",), "max_token_bytes", 1, 2),
        ((b"a b",), "max_tokens", 1, 2),
        ((b"a b",), "max_lexemes", 2, 3),
    )
    for sources, field, below, exact in limit_cases:
        for boundary, value in (("below", below), ("exact", exact)):
            limits = model.LexLimits(**(vars(LEX_LIMITS) | {field: value}))
            cases.append(
                DifferentialCase(f"limit/{field}/{boundary}", sources, limits)
            )

    precedence_cases = (
        (
            "sources",
            (b"\xff", b""),
            model.LexLimits(1, 0, 0, 0, 0, 0),
        ),
        (
            "total-source-bytes",
            (b"\xff\xff",),
            model.LexLimits(1, 0, 0, 0, 0, 0),
        ),
        (
            "source-bytes",
            (b"\xff\xff",),
            model.LexLimits(1, 1, 2, 0, 0, 0),
        ),
        (
            "source-issue",
            (b"\xff",),
            model.LexLimits(1, 1, 1, 0, 0, 0),
        ),
        (
            "lexemes",
            (b"ab",),
            model.LexLimits(1, 2, 2, 0, 0, 0),
        ),
        (
            "token-bytes",
            (b"ab",),
            model.LexLimits(1, 2, 2, 1, 0, 1),
        ),
        (
            "tokens",
            (b"ab",),
            model.LexLimits(1, 2, 2, 2, 0, 1),
        ),
    )
    cases.extend(
        DifferentialCase(f"precedence/{name}", sources, limits)
        for name, sources, limits in precedence_cases
    )

    generator = random.Random(0x57464C45584F4253)
    for ordinal in range(64):
        sources = tuple(
            bytes(generator.randrange(256) for _ in range(generator.randrange(33)))
            for _ in range(1 + generator.randrange(3))
        )
        cases.append(DifferentialCase(f"seeded/{ordinal:02d}", sources, LEX_LIMITS))

    identifiers = [case.identifier for case in cases]
    if len(identifiers) != len(set(identifiers)):
        raise AssertionError("differential case identifiers are not unique")
    return tuple(cases)


def case_manifest_sha256(cases: tuple[DifferentialCase, ...]) -> str:
    """Hash one closed, order-sensitive representation of differential inputs."""

    encoded = bytearray(b"WFLEXCASES\x00\x01")
    encoded.extend(observer.SPEC_HASH)
    encoded.extend(u32(SOURCE_LIMITS.max_sources))
    for value in (
        SOURCE_LIMITS.max_logical_path_bytes,
        SOURCE_LIMITS.max_source_bytes,
        SOURCE_LIMITS.max_total_source_bytes,
        SOURCE_LIMITS.max_binding_bytes,
    ):
        encoded.extend(u64(value))
    for case in cases:
        identifier = case.identifier.encode("ascii")
        encoded.extend(u32(len(identifier)) + identifier)
        encoded.extend(u32(len(case.sources)))
        for ordinal, source in enumerate(case.sources):
            path = generated_logical_path(ordinal).encode("ascii")
            encoded.extend(u32(len(path)) + path)
            encoded.extend(u64(len(source)) + source)
        encoded.extend(u32(case.limits.max_sources))
        for value in (
            case.limits.max_source_bytes,
            case.limits.max_total_source_bytes,
            case.limits.max_token_bytes,
            case.limits.max_tokens,
            case.limits.max_lexemes,
        ):
            encoded.extend(u64(value))
    return hashlib.sha256(encoded).hexdigest()


DIFFERENTIAL_CASES = build_differential_cases()
EXPECTED_CASE_COUNT = 942
EXPECTED_CASE_MANIFEST_SHA256 = (
    "3fdd322422999616b3d4bb09b01b23088c71c78d526d46305450220b571fa804"
)


def u8(value: int) -> bytes:
    return value.to_bytes(1, "big")


def u16(value: int) -> bytes:
    return value.to_bytes(2, "big")


def u32(value: int) -> bytes:
    return value.to_bytes(4, "big")


def u64(value: int) -> bytes:
    return value.to_bytes(8, "big")


def generated_logical_path(ordinal: int) -> str:
    """Return the exact portable path assigned to a generated source."""

    return f"generated/source-{ordinal}.wf"


def response(payload: bytes) -> bytes:
    return (
        observer.RESPONSE_MAGIC
        + u16(observer.PROTOCOL_VERSION)
        + observer.SPEC_HASH
        + payload
    )


def complete_payload(
    token_count: int, source_pieces: tuple[tuple[tuple[int, int, int], ...], ...]
) -> bytes:
    payload = bytearray(u8(0) + u64(token_count) + u32(len(source_pieces)))
    for pieces in source_pieces:
        payload.extend(u64(len(pieces)))
        for kind, start, end in pieces:
            payload.extend(u8(kind) + u64(start) + u64(end))
    return bytes(payload)


def request_for(
    sources: tuple[bytes, ...], limits: model.LexLimits = LEX_LIMITS
) -> observer.ObserverRequest:
    bound = tuple(
        observer.BoundSource(generated_logical_path(ordinal), exact)
        for ordinal, exact in enumerate(sources)
    )
    return observer.prepare_request(bound, SOURCE_LIMITS, limits)


class CodecTests(unittest.TestCase):
    def test_source_binding_and_request_have_one_exact_field_order(self) -> None:
        sources = (
            observer.BoundSource("z.wf", b"\xff\x00"),
            observer.BoundSource("a/b.wf", b"fn bytes"),
        )
        limits = observer.SourceLimits(7, 99, 101, 202, 303)
        lex_limits = model.LexLimits(6, 88, 177, 55, 44, 99)
        binding = observer.encode_source_binding(sources, limits)
        expected_binding = bytearray(observer.SOURCE_BINDING_MAGIC)
        expected_binding.extend(u16(observer.SOURCE_BINDING_VERSION))
        expected_binding.extend(observer.SPEC_HASH)
        expected_binding.extend(u64(2))
        for source in sources:
            path = source.logical_path.encode("ascii")
            expected_binding.extend(u64(len(path)) + path)
            expected_binding.extend(u64(len(source.exact)) + source.exact)
        self.assertEqual(binding, bytes(expected_binding))
        self.assertEqual(observer.decode_source_binding(binding, limits), sources)

        encoded = observer.encode_request(binding, limits, lex_limits)
        expected = bytearray(observer.REQUEST_MAGIC)
        expected.extend(u16(observer.PROTOCOL_VERSION))
        expected.extend(u32(7))
        for value in (99, 101, 202, 303):
            expected.extend(u64(value))
        expected.extend(u32(6))
        for value in (88, 177, 55, 44, 99):
            expected.extend(u64(value))
        expected.extend(u64(len(binding)))
        expected.extend(binding)
        self.assertEqual(encoded, bytes(expected))
        self.assertEqual(len(encoded), 98 + len(binding))

    def test_source_limits_and_logical_paths_fail_closed(self) -> None:
        maximum = observer.SourceLimits(
            model.U32_MAX,
            model.U64_MAX,
            model.U64_MAX,
            model.U64_MAX,
            model.U64_MAX,
        )
        for field in vars(maximum):
            field_maximum = (
                model.U32_MAX if field == "max_sources" else model.U64_MAX
            )
            for invalid in (True, -1, field_maximum + 1):
                with self.subTest(field=field, invalid=invalid):
                    with self.assertRaises(ValueError):
                        observer.SourceLimits(**(vars(maximum) | {field: invalid}))

        for logical_path in ("", "/a", "a//b", ".", "a/..", "a b", "é"):
            with self.subTest(logical_path=logical_path):
                with self.assertRaises(ValueError):
                    observer.BoundSource(logical_path, b"")
        with self.assertRaises(TypeError):
            observer.BoundSource("a", bytearray())  # type: ignore[arg-type]
        duplicate = (
            observer.BoundSource("a", b"one"),
            observer.BoundSource("a", b"two"),
        )
        with self.assertRaises(ValueError):
            observer.encode_source_binding(duplicate, SOURCE_LIMITS)

    def test_source_binding_rejects_every_noncanonical_boundary(self) -> None:
        binding = observer.encode_source_binding(
            (observer.BoundSource("a.wf", b"abc"),), SOURCE_LIMITS
        )
        for end in range(len(binding)):
            with self.subTest(end=end):
                with self.assertRaises(observer.ProtocolError):
                    observer.decode_source_binding(binding[:end], SOURCE_LIMITS)

        mutations = []
        bad_magic = bytearray(binding)
        bad_magic[0] ^= 1
        mutations.append(bytes(bad_magic))
        bad_version = bytearray(binding)
        bad_version[9] = 2
        mutations.append(bytes(bad_version))
        bad_spec = bytearray(binding)
        bad_spec[10] ^= 1
        mutations.append(bytes(bad_spec))
        mutations.append(binding + b"\x00")
        for mutation in mutations:
            with self.subTest(mutation=mutation[:12]):
                with self.assertRaises(observer.ProtocolError):
                    observer.decode_source_binding(mutation, SOURCE_LIMITS)

        duplicate = bytearray(observer.SOURCE_BINDING_MAGIC)
        duplicate.extend(u16(observer.SOURCE_BINDING_VERSION))
        duplicate.extend(observer.SPEC_HASH)
        duplicate.extend(u64(2))
        for exact in (b"one", b"two"):
            duplicate.extend(u64(1) + b"a" + u64(len(exact)) + exact)
        with self.assertRaises(observer.ProtocolError):
            observer.decode_source_binding(bytes(duplicate), SOURCE_LIMITS)

    def test_complete_response_decodes_only_a_lossless_partition(self) -> None:
        request = request_for((b"a \n", b""))
        encoded = response(
            complete_payload(
                1,
                (
                    ((0, 0, 1), (23, 1, 2), (24, 2, 3)),
                    (),
                ),
            )
        )
        decoded = observer.decode_response(encoded, request)
        expected = observer.project_model_outcome(
            model.lex_v0_9((b"a \n", b""), LEX_LIMITS)
        )
        self.assertEqual(decoded.binding, request.binding)
        self.assertEqual(decoded.sources, request.sources)
        self.assertEqual(decoded.outcome, expected)
        self.assertEqual(
            encoded[:42],
            observer.RESPONSE_MAGIC + u16(1) + observer.SPEC_HASH,
        )

        bad_payloads = (
            complete_payload(0, (((0, 0, 1), (23, 1, 2), (24, 2, 3)), ())),
            complete_payload(1, (((0, 0, 1), (23, 2, 3)), ())),
            complete_payload(1, (((0, 0, 0), (23, 0, 2), (24, 2, 3)), ())),
            complete_payload(1, (((25, 0, 1), (23, 1, 2), (24, 2, 3)), ())),
            complete_payload(1, (((0, 0, 1),), ())),
            u8(0) + u64(1) + u32(1),
        )
        for payload in bad_payloads:
            with self.subTest(payload=payload.hex()):
                with self.assertRaises(observer.ProtocolError):
                    observer.decode_response(response(payload), request)

    def test_structurally_valid_comparator_mutations_cannot_match_the_model(self) -> None:
        one_token = request_for((b"a ",))
        expected = observer.project_model_outcome(
            model.lex_v0_9((b"a ",), LEX_LIMITS)
        )
        wrong_kind_and_count = observer.decode_response(
            response(
                complete_payload(
                    0,
                    (((23, 0, 1), (23, 1, 2)),),
                )
            ),
            one_token,
        ).outcome
        self.assertNotEqual(wrong_kind_and_count, expected)

        one_span = request_for((b"ab",))
        expected = observer.project_model_outcome(
            model.lex_v0_9((b"ab",), LEX_LIMITS)
        )
        wrong_segmentation = observer.decode_response(
            response(complete_payload(2, (((0, 0, 1), (0, 1, 2)),))),
            one_span,
        ).outcome
        self.assertNotEqual(wrong_segmentation, expected)

    def test_all_noncomplete_outcome_payloads_are_explicit_and_strict(self) -> None:
        request = request_for((b"@9",))
        issue = observer.decode_response(
            response(u8(1) + u32(0) + u64(0) + u64(1) + u8(3)), request
        ).outcome
        self.assertEqual(
            issue,
            observer.SourceIssueObservation(0, 0, 1, "missing_label_name", b"@"),
        )

        limited = request_for((b"a b",), model.LexLimits(1, 3, 3, 3, 1, 3))
        limit = observer.decode_response(
            response(u8(2) + u8(0) + u8(4) + u64(1) + u64(2)), limited
        ).outcome
        self.assertEqual(limit, observer.LimitExceededObservation("tokens", 1, 2))
        address = observer.decode_response(
            response(u8(2) + u8(1) + u8(0) + u64(9)), request
        ).outcome
        self.assertEqual(
            address, observer.AddressSpaceExceededObservation("lexemes", 9)
        )
        storage = observer.decode_response(
            response(u8(2) + u8(2) + u8(1) + u64(7)), request
        ).outcome
        self.assertEqual(
            storage,
            observer.StorageUnavailableObservation("source_boundaries", 7),
        )

        compiler_payloads = (
            (
                u8(3)
                + u8(0)
                + u32(0)
                + u64(model.U64_MAX)
                + u64(0),
                observer.InvalidProducedSpanObservation(0, model.U64_MAX, 0),
            ),
            (u8(3) + u8(1) + u32(0), observer.PassDisagreementObservation(0)),
            (
                u8(3) + u8(2) + u64(3) + u64(4) + u64(2) + u64(2),
                observer.PassCountDisagreementObservation(3, 4, 2, 2),
            ),
            (u8(3) + u8(3), observer.CounterOverflowObservation()),
        )
        for payload, expected in compiler_payloads:
            with self.subTest(expected=expected):
                actual = observer.decode_response(response(payload), request).outcome
                self.assertEqual(actual, expected)

        malformed = (
            u8(1) + u32(1) + u64(0) + u64(1) + u8(0),
            u8(1) + u32(0) + u64(1) + u64(1) + u8(0),
            u8(1) + u32(0) + u64(0) + u64(1) + u8(7),
            u8(2) + u8(0) + u8(4) + u64(2) + u64(3),
            u8(2) + u8(0) + u8(4) + u64(1) + u64(1),
            u8(2) + u8(1) + u8(2) + u64(1),
            u8(2) + u8(2) + u8(0) + u64(0),
            u8(3) + u8(2) + u64(2) + u64(2) + u64(3) + u64(1),
            u8(3) + u8(2) + u64(2) + u64(2) + u64(1) + u64(1),
            u8(3) + u8(0) + u32(1) + u64(0) + u64(1),
            u8(3) + u8(1) + u32(1),
            u8(3) + u8(4),
            u8(4),
        )
        for payload in malformed:
            with self.subTest(payload=payload.hex()):
                with self.assertRaises(observer.ProtocolError):
                    observer.decode_response(response(payload), limited)

    def test_response_identity_truncation_and_trailing_data_fail_closed(self) -> None:
        request = request_for(())
        valid = response(complete_payload(0, ()))
        bad_magic = bytearray(valid)
        bad_magic[0] ^= 1
        bad_version = bytearray(valid)
        bad_version[9] = 2
        bad_spec = bytearray(valid)
        bad_spec[10] ^= 1
        candidates = [bytes(bad_magic), bytes(bad_version), bytes(bad_spec), valid + b"\x00"]
        candidates.extend(valid[:end] for end in range(len(valid)))
        for candidate in candidates:
            with self.subTest(length=len(candidate)):
                with self.assertRaises(observer.ProtocolError):
                    observer.decode_response(candidate, request)

    def test_process_runner_enforces_live_output_caps_and_timeout(self) -> None:
        request = request_for(())
        with tempfile.TemporaryDirectory(
            prefix="whitefoot-observer-runner-"
        ) as temporary:
            root = Path(temporary)
            for descriptor, maximum in (
                (1, observer_runner.MAX_RESPONSE_BYTES),
                (2, observer_runner.MAX_STDERR_BYTES),
            ):
                emitter = root / f"emit-{descriptor}"
                chunks = (maximum // observer_runner.READ_CHUNK_BYTES) + 2
                emitter.write_text(
                    f"#!{sys.executable}\n"
                    "import os\n"
                    f"chunk = b'x' * {observer_runner.READ_CHUNK_BYTES}\n"
                    f"for _ in range({chunks}):\n"
                    f"    os.write({descriptor}, chunk)\n",
                    encoding="utf-8",
                )
                emitter.chmod(0o755)
                with self.subTest(descriptor=descriptor):
                    with self.assertRaises(
                        observer_runner.ObserverToolError
                    ) as raised:
                        observer_runner.invoke_observer_bytes(
                            emitter,
                            request,
                            timeout_seconds=5.0,
                            cwd=root,
                            environment={},
                        )
                    self.assertIn(b"exceeded the harness bound", raised.exception.stderr)

            sleeper = root / "sleep"
            sleeper.write_text("#!/bin/sh\nexec /bin/sleep 2\n", encoding="utf-8")
            sleeper.chmod(0o755)
            with self.assertRaises(subprocess.TimeoutExpired):
                observer_runner.invoke_observer_bytes(
                    sleeper,
                    request,
                    timeout_seconds=0.01,
                    cwd=root,
                    environment={},
                )


class RustDifferentialTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls._resources = contextlib.ExitStack()
        cls.addClassCleanup(cls._resources.close)
        cls.executable = cls._resources.enter_context(
            cargo_policy.built_lexical_observer()
        )
        cls.working_directory = Path(
            cls._resources.enter_context(
                tempfile.TemporaryDirectory(prefix="whitefoot-observer-test-")
            )
        )

    def test_case_manifest_is_frozen_before_execution_and_generator_is_independent(
        self,
    ) -> None:
        self.assertEqual(len(DIFFERENTIAL_CASES), EXPECTED_CASE_COUNT)
        self.assertEqual(
            case_manifest_sha256(DIFFERENTIAL_CASES),
            EXPECTED_CASE_MANIFEST_SHA256,
        )
        with mock.patch.object(
            model,
            "lex_v0_9",
            side_effect=AssertionError("generator called the model"),
        ), mock.patch.object(
            observer_runner,
            "invoke_observer",
            side_effect=AssertionError("generator called the Rust observer"),
        ):
            regenerated = build_differential_cases()
        self.assertEqual(regenerated, DIFFERENTIAL_CASES)

    def assert_differential(
        self,
        sources: tuple[bytes, ...],
        limits: model.LexLimits = LEX_LIMITS,
    ) -> None:
        request = request_for(sources, limits)
        actual = observer_runner.invoke_observer(
            self.executable,
            request,
            cwd=self.working_directory,
            environment={},
            timeout_seconds=5.0,
        ).outcome
        expected = observer.project_model_outcome(model.lex_v0_9(sources, limits))
        self.assertEqual(actual, expected)

    def test_all_frozen_authored_and_generated_cases_match(self) -> None:
        for case in DIFFERENTIAL_CASES:
            with self.subTest(case=case.identifier):
                self.assert_differential(case.sources, case.limits)

    def test_request_identity_binds_paths_and_order_without_affecting_lexing(self) -> None:
        first = observer.prepare_request(
            (
                observer.BoundSource("first.wf", b"a"),
                observer.BoundSource("second.wf", b"b"),
            ),
            SOURCE_LIMITS,
            LEX_LIMITS,
        )
        renamed = observer.prepare_request(
            (
                observer.BoundSource("renamed.wf", b"a"),
                observer.BoundSource("other.wf", b"b"),
            ),
            SOURCE_LIMITS,
            LEX_LIMITS,
        )
        first_response = observer_runner.invoke_observer_bytes(
            self.executable,
            first,
            cwd=self.working_directory,
            environment={},
        )
        renamed_response = observer_runner.invoke_observer_bytes(
            self.executable,
            renamed,
            cwd=self.working_directory,
            environment={},
        )
        self.assertNotEqual(first.wire_bytes, renamed.wire_bytes)
        self.assertEqual(first_response, renamed_response)
        self.assertNotEqual(
            hashlib.sha256(first.wire_bytes + first_response).digest(),
            hashlib.sha256(renamed.wire_bytes + renamed_response).digest(),
        )

        ordered = request_for((b"valid", b"@9"))
        reversed_sources = request_for((b"@9", b"valid"))
        ordered_outcome = observer_runner.invoke_observer(
            self.executable,
            ordered,
            cwd=self.working_directory,
            environment={},
        ).outcome
        reversed_outcome = observer_runner.invoke_observer(
            self.executable,
            reversed_sources,
            cwd=self.working_directory,
            environment={},
        ).outcome
        self.assertNotEqual(ordered_outcome, reversed_outcome)

    def test_tool_failures_publish_no_response_and_ambient_metadata_is_irrelevant(self) -> None:
        request = request_for((b"a b\n",))
        case_manifest = case_manifest_sha256(DIFFERENTIAL_CASES)
        clean = observer_runner.invoke_observer_bytes(
            self.executable,
            request,
            cwd=self.working_directory,
            environment={},
            timeout_seconds=5.0,
        )
        with tempfile.TemporaryDirectory(
            prefix="whitefoot-observer-ambient-"
        ) as ambient:
            ambient_path = Path(ambient)
            capability = (
                ambient_path
                / "capabilities"
                / "whitefoot-rust"
                / "v0.9"
                / "foundation.json"
            )
            capability.parent.mkdir(parents=True)
            capability.write_text(
                '{"pretend":"complete"}\n', encoding="utf-8"
            )
            report = ambient_path / "derived-capability-report.json"
            report.write_text('{"release":true}\n', encoding="utf-8")
            fake_directory = ambient_path / "bin"
            fake_directory.mkdir()
            fake = fake_directory / "whitefoot-lexical-observer"
            marker = ambient_path / "fake-observer-ran"
            fake.write_text(
                "#!/bin/sh\n" f"/usr/bin/touch {marker}\n" "exit 99\n",
                encoding="utf-8",
            )
            fake.chmod(0o755)
            if hasattr(os, "symlink"):
                os.symlink(capability, ambient_path / "capability-current.json")
            if hasattr(os, "mkfifo"):
                os.mkfifo(ambient_path / "capability-report.fifo")
            noisy = observer_runner.invoke_observer_bytes(
                self.executable,
                request,
                cwd=ambient_path,
                environment={
                    "LANG": "C",
                    "PATH": str(fake_directory),
                    "WHITEFOOT_CAPABILITY_OVERLAY": str(capability),
                    "WHITEFOOT_DERIVED_REPORT": str(report),
                    "WHITEFOOT_STATIC_CATALOG": "pretend-complete",
                },
                timeout_seconds=5.0,
            )
            returncode, stdout, stderr = self._run_raw(
                ["ignored-capability-path", os.fspath(capability)],
                request.wire_bytes,
                cwd=ambient_path,
                environment={"PATH": str(fake_directory)},
            )
            self.assertEqual(returncode, 0)
            self.assertEqual(stdout, clean)
            self.assertEqual(stderr, b"")
            self.assertFalse(marker.exists())
        self.assertEqual(noisy, clean)
        self.assertEqual(case_manifest_sha256(DIFFERENTIAL_CASES), case_manifest)

        malformed = (
            (request.wire_bytes[:-1], b"request-read-failed"),
            (request.wire_bytes + b"\x00", b"request-has-trailing-bytes"),
        )
        for wire_bytes, diagnostic in malformed:
            with self.subTest(length=len(wire_bytes)):
                returncode, stdout, stderr = self._run_raw([], wire_bytes)
                self.assertNotEqual(returncode, 0)
                self.assertEqual(stdout, b"")
                self.assertIn(diagnostic, stderr)

        for offset, replacement, diagnostic in (
            (0, b"X", b"request-magic-invalid"),
            (9, b"\x02", b"request-version-unsupported"),
        ):
            mutation = bytearray(request.wire_bytes)
            mutation[offset : offset + len(replacement)] = replacement
            returncode, stdout, stderr = self._run_raw([], bytes(mutation))
            self.assertNotEqual(returncode, 0)
            self.assertEqual(stdout, b"")
            self.assertIn(diagnostic, stderr)

        excessive_profile = bytearray(request.wire_bytes)
        excessive_profile[10:14] = u32(4_097)
        returncode, stdout, stderr = self._run_raw([], bytes(excessive_profile))
        self.assertNotEqual(returncode, 0)
        self.assertEqual(stdout, b"")
        self.assertIn(b"source-limits-outside-observer-profile", stderr)

        wrong_spec = bytearray(request.wire_bytes)
        wrong_spec[98 + 10] ^= 1
        returncode, stdout, stderr = self._run_raw([], bytes(wrong_spec))
        self.assertNotEqual(returncode, 0)
        self.assertEqual(stdout, b"")
        self.assertIn(b"specification-mismatch", stderr)

    def _run_raw(
        self,
        arguments: list[str],
        wire_bytes: bytes,
        *,
        cwd: Path | None = None,
        environment: dict[str, str] | None = None,
    ) -> tuple[int, bytes, bytes]:
        with tempfile.TemporaryFile() as stdout, tempfile.TemporaryFile() as stderr:
            completed = subprocess.run(
                [os.fspath(self.executable), *arguments],
                input=wire_bytes,
                stdout=stdout,
                stderr=stderr,
                check=False,
                timeout=5.0,
                cwd=self.working_directory if cwd is None else cwd,
                env={} if environment is None else environment,
            )
            self.assertLessEqual(stdout.tell(), observer_runner.MAX_RESPONSE_BYTES)
            self.assertLessEqual(stderr.tell(), observer_runner.MAX_STDERR_BYTES)
            stdout.seek(0)
            stdout_bytes = stdout.read()
            stderr.seek(0)
            stderr_bytes = stderr.read()
        return completed.returncode, stdout_bytes, stderr_bytes


if __name__ == "__main__":
    unittest.main()
