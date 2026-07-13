#!/usr/bin/env python3
"""Compile and correctness-check one generated utf8parse candidate."""

from __future__ import annotations

import contextlib
import io
import json
from pathlib import Path
import re
import subprocess
import sys
import tarfile
import tempfile
from typing import Any

from benchmark import (
    CARGO,
    CLANG,
    MACOS_SDK,
    NATIVE_TARGET,
    PYTHON,
    audit_cargo_config_absence,
    audit_clang_default_configs,
    cargo_cli_neutralization,
    locked_build_tool_manifest,
    sanitized_build_environment,
    verify_utf8parse_registry,
)


sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
HARNESS = HERE / "harness"
sys.path.insert(0, str(ROOT / "prototype/democ"))
import democ  # noqa: E402


class HarnessFailure(RuntimeError):
    """The locked evaluator failed independently of the candidate."""


MAX_DIAGNOSTIC_CHARS = 65_536
EXPECTED_CORPUS_CASES = 84_041


def diagnostic(code: str, message: str) -> dict[str, str]:
    return {"code": code, "message": message[:MAX_DIAGNOSTIC_CHARS]}


def result(
    compile_passed: bool,
    compile_diagnostics: list[dict[str, str]],
    correctness_passed: bool,
    correctness_diagnostics: list[dict[str, str]],
    proof: dict[str, Any] | None = None,
) -> dict[str, Any]:
    value: dict[str, Any] = {
        "compile": {"passed": compile_passed, "diagnostics": compile_diagnostics},
        "correctness": {
            "passed": correctness_passed,
            "diagnostics": correctness_diagnostics,
        },
    }
    if proof is not None:
        value["proof"] = proof
    return value


def run(
    command: list[str],
    *,
    cwd: Path | None = None,
    env: dict[str, str] | None = None,
    timeout: int = 120,
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
        cwd=cwd,
        env=env,
        timeout=timeout,
    )


LLVM_FUNCTION = re.compile(
    r"^define\b[^\n@]*@([A-Za-z_.$][A-Za-z0-9_.$-]*)\(", re.MULTILINE
)


def namespace_module(ir: str, namespace: str, entry: str) -> str:
    definitions = set(LLVM_FUNCTION.findall(ir))
    if "parse" not in definitions:
        raise RuntimeError("compiled module does not define the required parse entry")
    replacements = {
        name: entry if name == "parse" else f"xlang_{namespace}_{name}"
        for name in definitions
    }
    names = "|".join(re.escape(name) for name in sorted(definitions, key=len, reverse=True))
    return re.sub(
        rf"@({names})(?=\()",
        lambda match: f"@{replacements[match.group(1)]}",
        ir,
    )


EXPECTED_PARSE_EFFECTS = [
    "reads",
    "(",
    "'r",
    ")",
    ",",
    "writes",
    "(",
    "'r",
    ")",
    ",",
    "traps",
]
EXPECTED_PARSE_PARAMS = [
    {
        "name": "out",
        "mode": {"kind": "ref", "region": "r", "uniq": True},
        "ty": "buffer<u32>",
    },
    {"name": "src", "mode": {"kind": "own"}, "ty": "buffer<u8>"},
]


def public_api_mismatch(functions: list[dict[str, Any]]) -> str | None:
    if any(function.get("name") == "main" for function in functions):
        return "function main is forbidden for this candidate"

    parsers = [function for function in functions if function.get("name") == "parse"]
    if len(parsers) != 1:
        return "the source must define exactly one function named parse"

    parse = parsers[0]
    if parse.get("regions") != ["r"]:
        return "parse must declare exactly the region list ['r']"
    if parse.get("params") != EXPECTED_PARSE_PARAMS:
        return "parse parameters must be exactly out: &uniq 'r buffer<u32>, src: own buffer<u8>"
    if parse.get("rmode") != {"kind": "own"} or parse.get("rty") != "u64":
        return "parse must return exactly own u64"
    if parse.get("effects") != EXPECTED_PARSE_EFFECTS:
        return "parse effects must be exactly reads('r), writes('r), traps"
    if not parse.get("requires"):
        return "parse must contain a nonempty requires block"
    return None


def evaluate(candidate: Path) -> dict[str, Any]:
    try:
        if Path(sys.executable).resolve() != PYTHON.resolve():
            raise RuntimeError(f"evaluator must run under locked Python {PYTHON}")
        tool_identity = locked_build_tool_manifest()
        clang_config_identity = audit_clang_default_configs()
        verify_utf8parse_registry()
    except (OSError, RuntimeError, tarfile.TarError) as error:
        raise HarnessFailure(
            f"locked evaluator preflight failed: {error}"
        ) from error
    try:
        # Keep candidate bytes faithful to the archived model response. In
        # particular, do not apply Python's universal-newline conversion.
        with candidate.open("r", encoding="utf-8", newline="") as stream:
            source = stream.read()
    except (OSError, UnicodeError) as error:
        return result(
            False,
            [diagnostic("SOURCE_READ", str(error))],
            False,
            [diagnostic("NOT_RUN", "compile failed")],
        )

    compiler_text = io.StringIO()
    try:
        with contextlib.redirect_stdout(compiler_text), contextlib.redirect_stderr(compiler_text):
            _structs, _enums, functions, _contracts, _conforms, _consts = democ.parse_program(source)
    except (MemoryError, KeyboardInterrupt, GeneratorExit) as error:
        raise HarnessFailure(
            f"unexpected source-parser failure: {type(error).__name__}"
        ) from error
    except (SystemExit, Exception) as error:
        message = compiler_text.getvalue() or str(error) or type(error).__name__
        return result(
            False,
            [diagnostic("XLANG_COMPILE", message)],
            False,
            [diagnostic("NOT_RUN", "compile failed")],
        )

    api_mismatch = public_api_mismatch(functions)
    if api_mismatch is not None:
        return result(
            False,
            [diagnostic("XLANG_PUBLIC_API", api_mismatch)],
            False,
            [diagnostic("NOT_RUN", "compile failed")],
        )

    compiler_text.seek(0)
    compiler_text.truncate(0)
    try:
        with contextlib.redirect_stdout(compiler_text), contextlib.redirect_stderr(compiler_text):
            # Proof reports are deliberately not requested before first-green
            # freeze. They are attribution evidence, not repair feedback.
            facts_ir = democ.compile_program(source, alias=True)
            nofacts_ir = democ.compile_program(source, alias=False)
        facts_ir = namespace_module(facts_ir, "facts", "xlang_parse_facts")
        nofacts_ir = namespace_module(nofacts_ir, "nofacts", "xlang_parse_nofacts")
    except (democ.CheckError, SystemExit) as error:
        message = compiler_text.getvalue() or str(error) or type(error).__name__
        return result(
            False,
            [diagnostic("XLANG_COMPILE", message)],
            False,
            [diagnostic("NOT_RUN", "compile failed")],
        )
    except BaseException as error:
        raise HarnessFailure(
            f"unexpected compiler or namespacing failure: {type(error).__name__}"
        ) from error

    with tempfile.TemporaryDirectory(prefix="xlang-utf8parse-verify-") as temporary:
        build = Path(temporary)
        build_environment, _removed = sanitized_build_environment()
        cargo_config_identity = audit_cargo_config_absence(build)
        facts_ll = build / "facts.ll"
        nofacts_ll = build / "nofacts.ll"
        facts_obj = build / "facts.o"
        nofacts_obj = build / "nofacts.o"
        facts_ll.write_text(facts_ir, encoding="utf-8")
        nofacts_ll.write_text(nofacts_ir, encoding="utf-8")

        for ll, obj, label in (
            (facts_ll, facts_obj, "facts-on"),
            (nofacts_ll, nofacts_obj, "facts-off"),
        ):
            try:
                compiled = run(
                    [
                        str(CLANG), "--no-default-config", "-isysroot", str(MACOS_SDK),
                        "-O3", "-c", str(ll), "-o", str(obj),
                    ],
                    env=build_environment,
                )
            except subprocess.TimeoutExpired as error:
                raise HarnessFailure(
                    f"{label} locked native build exceeded the frozen execution limit"
                ) from error
            if compiled.returncode != 0:
                raise HarnessFailure(
                    f"{label} generated module failed locked native compilation"
                )

        target_dir = build / "cargo-target"
        try:
            cargo = run(
                [
                    str(CARGO),
                    *cargo_cli_neutralization(),
                    "rustc",
                    "--manifest-path",
                    str(HARNESS / "Cargo.toml"),
                    "--target-dir",
                    str(target_dir),
                    "--bin",
                    "verify",
                    "--release",
                    "--locked",
                    "--offline",
                    "--target",
                    NATIVE_TARGET,
                    "--",
                    "-C",
                    f"link-arg={facts_obj}",
                    "-C",
                    f"link-arg={nofacts_obj}",
                ],
                cwd=build,
                env=build_environment,
                timeout=180,
            )
        except subprocess.TimeoutExpired as error:
            raise HarnessFailure(
                "locked verifier build exceeded the frozen execution limit"
            ) from error
        if cargo.returncode != 0:
            raise HarnessFailure(
                "locked verifier build or link failed: " + (cargo.stderr or cargo.stdout)
            )
        if audit_cargo_config_absence(build) != cargo_config_identity:
            raise HarnessFailure("Cargo config search-chain state changed during evaluator build")
        if audit_clang_default_configs() != clang_config_identity:
            raise HarnessFailure("Clang default-config state changed during evaluator build")
        if locked_build_tool_manifest() != tool_identity:
            raise HarnessFailure("locked tool or SDK identity changed during evaluator build")

        try:
            verifier = run(
                [str(target_dir / NATIVE_TARGET / "release" / "verify")],
                env=build_environment,
                timeout=180,
            )
        except subprocess.TimeoutExpired as error:
            raise HarnessFailure(
                "locked correctness process exceeded the frozen execution limit"
            ) from error
        if verifier.returncode != 0:
            if verifier.returncode > 1:
                raise HarnessFailure(
                    verifier.stderr
                    or verifier.stdout
                    or f"verifier exited with status {verifier.returncode}"
                )
            detail = verifier.stderr or verifier.stdout
            if not detail and verifier.returncode < 0:
                detail = f"candidate terminated by signal {-verifier.returncode}"
            return result(
                True,
                [],
                False,
                [diagnostic("DIFFERENTIAL", detail or "candidate verifier failed")],
            )
        expected_summary = f"correct cases={EXPECTED_CORPUS_CASES}"
        if verifier.stdout.strip() != expected_summary or verifier.stderr:
            raise HarnessFailure(
                "verifier success output did not match the locked corpus summary"
            )

        boundary_exe = build / "boundary"
        try:
            boundary_link = run(
                [
                    str(CLANG),
                    "--no-default-config",
                    "-isysroot",
                    str(MACOS_SDK),
                    "-std=c11",
                    "-Wall",
                    "-Wextra",
                    "-Wpedantic",
                    "-Werror",
                    "-O3",
                    str(HERE / "boundary.c"),
                    str(facts_obj),
                    str(nofacts_obj),
                    "-o",
                    str(boundary_exe),
                ],
                env=build_environment,
            )
        except subprocess.TimeoutExpired as error:
            raise HarnessFailure(
                "locked boundary build exceeded the frozen execution limit"
            ) from error
        if boundary_link.returncode != 0:
            raise HarnessFailure(
                "locked boundary build or link failed: "
                + (boundary_link.stderr or boundary_link.stdout)
            )
        try:
            boundary = run(
                [str(boundary_exe)], env=build_environment, timeout=15
            )
        except subprocess.TimeoutExpired as error:
            raise HarnessFailure(
                "locked capacity process exceeded the frozen execution limit"
            ) from error
        if boundary.returncode != 0:
            if boundary.returncode == 2 or boundary.returncode < 0:
                raise HarnessFailure(
                    boundary.stderr
                    or boundary.stdout
                    or f"boundary verifier exited with status {boundary.returncode}"
                )
            return result(
                True,
                [],
                False,
                [
                    diagnostic(
                        "CAPACITY_BOUNDARY",
                        boundary.stderr or f"boundary exit {boundary.returncode}",
                    )
                ],
            )
        if boundary.stdout or boundary.stderr:
            raise HarnessFailure(
                "successful boundary verifier emitted unexpected output"
            )

        if audit_cargo_config_absence(build) != cargo_config_identity:
            raise HarnessFailure("Cargo config search-chain state changed during evaluator round")
        if audit_clang_default_configs() != clang_config_identity:
            raise HarnessFailure("Clang default-config state changed during evaluator round")
        if locked_build_tool_manifest() != tool_identity:
            raise HarnessFailure("locked tool or SDK identity changed during evaluator round")

        return result(
            True,
            [],
            True,
            [diagnostic("CORPUS", verifier.stdout.strip())],
        )


def main() -> int:
    if len(sys.argv) != 2:
        print(
            json.dumps(
                result(
                    False,
                    [diagnostic("USAGE", "expected candidate path")],
                    False,
                    [diagnostic("NOT_RUN", "usage error")],
                ),
                sort_keys=True,
            )
        )
        return 0
    try:
        value = evaluate(Path(sys.argv[1]).resolve())
    except HarnessFailure as error:
        print(f"utf8parse evaluator harness failure: {error}", file=sys.stderr)
        return 70
    print(json.dumps(value, sort_keys=True, separators=(",", ":")))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
