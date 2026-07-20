#!/usr/bin/env python3
"""Fail closed on the Rust workspace, toolchain, spec, and dependency policy."""

from __future__ import annotations

import hashlib
import json
import os
import re
import subprocess
import sys
from pathlib import Path
from typing import Optional, Union

import rust_source_policy


ROOT = Path(__file__).resolve().parents[1]
REPOSITORY = ROOT.parent

EXPECTED_MEMBERS = (
    "whitefoot-contract",
    "whitefoot-verifier",
)
EXPECTED_MANIFESTS = {
    "whitefoot-contract": Path("crates/whitefoot-contract/Cargo.toml"),
    "whitefoot-verifier": Path("crates/whitefoot-verifier/Cargo.toml"),
}
EXPECTED_TARGETS = {
    "whitefoot-contract": {
        "name": "whitefoot_contract",
        "kind": ["lib"],
        "crate_types": ["lib"],
        "source": Path("crates/whitefoot-contract/src/lib.rs"),
        "doctest": False,
    },
    "whitefoot-verifier": {
        "name": "whitefoot_verifier",
        "kind": ["lib"],
        "crate_types": ["lib"],
        "source": Path("crates/whitefoot-verifier/src/lib.rs"),
        "doctest": False,
    },
}
EXPECTED_EDGES = {
    "whitefoot-contract": (),
    "whitefoot-verifier": ("whitefoot-contract",),
}
DEPENDENCY_FIELDS = {
    "name",
    "version",
    "source",
    "checksum",
    "license",
    "purpose",
    "features",
    "build_script",
    "proc_macro",
    "unsafe_tcb",
}
FORBIDDEN_BUILD_ENV = {
    "CARGO_BUILD_TARGET",
    "CARGO_ENCODED_RUSTFLAGS",
    "CARGO_HOME",
    "CARGO_TARGET_DIR",
    "RUSTC",
    "RUSTC_WRAPPER",
    "RUSTC_WORKSPACE_WRAPPER",
    "RUSTDOC",
    "RUSTDOCFLAGS",
    "RUSTFLAGS",
    "RUSTUP_TOOLCHAIN",
}
FORBIDDEN_BUILD_ENV_PREFIXES = (
    "CARGO_BUILD_",
    "CARGO_PROFILE_",
    "CARGO_TARGET_",
)
INCLUDE_INVOCATION = re.compile(r"\binclude_(?:bytes|str)\s*!\s*\(")
INCLUDE_DATA_IDENTIFIER = re.compile(r"\b(?:r#)?include_(?:bytes|str)\b")
LITERAL_INCLUDE = re.compile(
    r'\b(include_(?:bytes|str))\s*!\s*\(\s*"([^"\\\n]+)"\s*\)'
)
PATH_ATTRIBUTE_INVOCATION = re.compile(r"#\s*\[[^\]]*\bpath\s*=")
FORBIDDEN_INCLUDE_MACRO = re.compile(r"\b(?:r#)?include\b")
FORBIDDEN_MACRO_RULES = re.compile(r"\bmacro_rules\s*!")
FORBIDDEN_ENV_IDENTIFIER = re.compile(r"\b(?:r#)?(?:env|option_env)\b")
CFG_IDENTIFIER = re.compile(r"\b(?:r#)?(?:cfg|cfg_attr)\b")
CANONICAL_TEST_CFG = re.compile(r"#\s*\[\s*(cfg)\s*\(\s*test\s*\)\s*\]")
DIRECT_FACET_ID = re.compile(
    r"facet:[A-Z]+-[0-9]+[a-z]?/[a-z][a-z0-9]*(?:-[a-z0-9]+)*"
    r"(?![a-z0-9-])"
)
EXPECTED_COMPILE_TIME_DATA_INPUTS = {
    Path("kernel-spec-v0.8.sha256"),
}


def fail(message: str) -> None:
    """Report one policy failure without a Python traceback."""
    print(f"workspace policy: {message}", file=sys.stderr)
    raise SystemExit(1)


def output(*arguments: str) -> str:
    """Run one read-only tool query in the compiler workspace."""
    result = subprocess.run(
        arguments,
        cwd=ROOT,
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if result.returncode != 0:
        detail = result.stderr.strip() or result.stdout.strip() or "no diagnostic"
        fail(f"{' '.join(arguments)} failed: {detail}")
    return result.stdout


def read_json(path: Path) -> dict:
    """Read one JSON object or fail with a policy diagnostic."""
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        fail(f"cannot read {path.relative_to(ROOT)}: {error}")
    if not isinstance(value, dict):
        fail(f"{path.relative_to(ROOT)} must contain one JSON object")
    return value


def sha256(path: Path) -> str:
    """Return the exact byte identity of a regular file."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def relative(path: Union[str, Path], root: Path = ROOT) -> Path:
    """Resolve a path and require it to remain inside compiler/."""
    resolved = Path(path).resolve()
    root = root.resolve()
    try:
        return resolved.relative_to(root)
    except ValueError:
        fail(f"path escapes compiler/: {resolved}")


def check_no_source_symlinks() -> None:
    """Reject source indirection; generated target contents are not source."""
    for directory, directory_names, file_names in os.walk(ROOT, followlinks=False):
        base = Path(directory)
        for name in [*directory_names, *file_names]:
            candidate = base / name
            if candidate.is_symlink():
                fail(f"symlink is forbidden: {candidate.relative_to(ROOT)}")
        if base == ROOT:
            directory_names[:] = [name for name in directory_names if name != "target"]


def check_no_workspace_build_tool_configuration(
    root: Path = ROOT,
    repository: Path = REPOSITORY,
) -> None:
    """Reject active Cargo, rustfmt, and Clippy configuration inputs."""
    present: list[Path] = []
    for directory, directory_names, file_names in os.walk(root, followlinks=False):
        base = Path(directory)
        if base == root:
            directory_names[:] = [name for name in directory_names if name != "target"]
        if base.name == ".cargo":
            present.extend(
                base / name
                for name in ("config", "config.toml")
                if name in file_names
            )
        present.extend(
            base / name
            for name in (".rustfmt.toml", "rustfmt.toml", ".clippy.toml", "clippy.toml")
            if name in file_names
        )
    present.extend(
        path
        for path in (
        repository / ".cargo" / "config",
        repository / ".cargo" / "config.toml",
        repository / ".rustfmt.toml",
        repository / "rustfmt.toml",
        repository / ".clippy.toml",
        repository / "clippy.toml",
        )
        if path.exists()
    )
    if present:
        rendered = sorted({str(path.resolve()) for path in present})
        fail(f"workspace build-tool configuration is forbidden: {rendered}")


def facet_metadata_reference(text: str) -> Optional[str]:
    """Name a direct static-catalog facet ID occurrence in production Rust."""
    facet = DIRECT_FACET_ID.search(text)
    if facet is not None:
        return f"direct facet ID {facet.group(0)!r}"
    return None


def check_no_production_facet_ids(root: Path = ROOT) -> None:
    """Reject direct facet-ID source occurrences in production Rust."""
    root = root.resolve()
    for path, text in production_rust_sources(root):
        relative_path = path.relative_to(root)
        reference = facet_metadata_reference(text)
        if reference is not None:
            fail(
                "production Rust must not contain direct static-catalog facet IDs: "
                f"{relative_path}: {reference}"
            )


def check_environment() -> None:
    """Reject host overrides that can silently change the checked build."""
    forbidden = sorted(
        name
        for name in os.environ
        if name in FORBIDDEN_BUILD_ENV
        or any(name.startswith(prefix) for prefix in FORBIDDEN_BUILD_ENV_PREFIXES)
    )
    if forbidden:
        fail(f"build override environment is forbidden: {forbidden}")


def production_rust_sources(root: Path = ROOT) -> tuple[tuple[Path, str], ...]:
    """Validate every Rust source under crates/ and its approved data inputs."""
    root = root.resolve()
    try:
        paths = sorted(root.glob("crates/**/*.rs"))
    except OSError as error:
        fail(f"cannot enumerate production Rust sources: {error}")
    sources: list[tuple[Path, str]] = []
    for discovered in paths:
        path = discovered.resolve()
        relative_path = relative(path, root)
        try:
            if not path.is_file():
                fail(f"production Rust source is not a regular file: {relative_path}")
            text = path.read_text(encoding="utf-8")
        except (OSError, UnicodeError) as error:
            fail(f"cannot inspect production Rust source {relative_path}: {error}")
        sources.append((path, text))

        try:
            views = rust_source_policy.lexical_views(text)
        except rust_source_policy.RustSourcePolicyError as error:
            fail(f"cannot lex production Rust source {relative_path}: {error}")
        if FORBIDDEN_INCLUDE_MACRO.search(views.code_only):
            fail(f"include! is forbidden in production Rust: {relative_path}")
        if FORBIDDEN_MACRO_RULES.search(views.code_only):
            fail(f"macro_rules! is forbidden in production Rust: {relative_path}")
        if FORBIDDEN_ENV_IDENTIFIER.search(views.code_only):
            fail(
                "compile-time environment channels are forbidden in production Rust: "
                f"{relative_path}"
            )
        cfg_identifiers = {
            match.start() for match in CFG_IDENTIFIER.finditer(views.code_only)
        }
        allowed_cfg_identifiers = {
            match.start(1) for match in CANONICAL_TEST_CFG.finditer(views.code_only)
        }
        if cfg_identifiers != allowed_cfg_identifiers:
            fail(
                "conditional compilation is limited to canonical #[cfg(test)]: "
                f"{relative_path}"
            )

        directive_text = views.comments_removed
        invocations = list(INCLUDE_INVOCATION.finditer(views.code_only))
        invocation_starts = {match.start() for match in invocations}
        data_identifiers = {
            match.start() for match in INCLUDE_DATA_IDENTIFIER.finditer(views.code_only)
        }
        if data_identifiers != invocation_starts:
            fail(
                "compile-time data macros must be direct invocations: "
                f"{path.relative_to(root)}"
            )
        literal_includes = [
            match
            for match in LITERAL_INCLUDE.finditer(directive_text)
            if match.start() in invocation_starts
        ]
        if len(invocations) != len(literal_includes):
            fail(
                "nonliteral or malformed include macro is forbidden: "
                f"{path.relative_to(root)}"
            )
        if PATH_ATTRIBUTE_INVOCATION.search(views.code_only):
            fail(f"#[path] is forbidden in production Rust: {relative_path}")

        for match in literal_includes:
            spelling = match.group(2)
            included = (path.parent / spelling).resolve()
            included_relative = relative(included, root)
            if not included.is_file():
                fail(
                    f"compile-time input does not name a regular file: "
                    f"{path.relative_to(root)} -> {spelling}"
                )
            if included_relative not in EXPECTED_COMPILE_TIME_DATA_INPUTS:
                fail(
                    "unapproved compile-time data input: "
                    f"{path.relative_to(root)} -> {included_relative}"
                )
    return tuple(sources)


def check_compile_time_inputs(root: Path = ROOT) -> None:
    """Require crate sources and compile-time data to remain closed."""
    production_rust_sources(root)


def check_toolchain() -> None:
    """Match the installed Rust tools to the exact reviewed lock."""
    lock = read_json(ROOT / "toolchain-lock.json")
    expected_fields = {
        "schema",
        "channel",
        "profile",
        "components",
        "rust_release",
        "rust_commit",
        "cargo_release",
        "cargo_commit",
        "rustfmt_release",
        "clippy_release",
        "llvm_release",
    }
    if set(lock) != expected_fields or lock.get("schema") != 1:
        fail("toolchain-lock.json has an unknown or incomplete schema")
    string_fields = expected_fields - {"schema", "components"}
    if not all(isinstance(lock[field], str) and lock[field] for field in string_fields):
        fail("toolchain-lock.json contains an empty or non-string identity")
    if lock["components"] != ["clippy", "rustfmt"]:
        fail("toolchain-lock.json components must be exactly clippy and rustfmt")

    expected_toolchain = (
        "[toolchain]\n"
        f"channel = \"{lock['channel']}\"\n"
        f"profile = \"{lock['profile']}\"\n"
        f"components = {json.dumps(lock['components'])}\n"
    )
    actual_toolchain = (ROOT / "rust-toolchain.toml").read_text(encoding="utf-8")
    if actual_toolchain != expected_toolchain:
        fail("rust-toolchain.toml differs from the exact toolchain lock")

    fields: dict[str, str] = {}
    first_line = ""
    for line in output("rustc", "--version", "--verbose").splitlines():
        if not first_line:
            first_line = line
        if ": " in line:
            key, value = line.split(": ", 1)
            fields[key] = value

    expected_rust = lock["rust_release"]
    expected_banner = f"rustc {expected_rust} ("
    if not first_line.startswith(expected_banner) or not first_line.endswith(")"):
        fail(f"rustc banner does not match the lock: {first_line}")
    if fields.get("release") != expected_rust:
        fail(f"rustc release is {fields.get('release')}, expected {expected_rust}")
    if fields.get("commit-hash") != lock["rust_commit"]:
        fail("rustc commit does not match toolchain-lock.json")
    if fields.get("LLVM version") != lock["llvm_release"]:
        fail("rustc LLVM version does not match toolchain-lock.json")

    cargo_lines = output("cargo", "--version", "--verbose").splitlines()
    cargo_banner = cargo_lines[0].split() if cargo_lines else []
    cargo_fields = {}
    for line in cargo_lines[1:]:
        if ": " in line:
            key, value = line.split(": ", 1)
            cargo_fields[key] = value
    if len(cargo_banner) < 2 or cargo_banner[1] != lock["cargo_release"]:
        fail("Cargo release does not match toolchain-lock.json")
    if cargo_fields.get("commit-hash") != lock["cargo_commit"]:
        fail("Cargo commit does not match toolchain-lock.json")

    rustfmt = output("rustfmt", "--version").strip()
    rustfmt_prefix = (
        f"rustfmt {lock['rustfmt_release']} ({lock['rust_commit'][:10]} "
    )
    if not rustfmt.startswith(rustfmt_prefix) or not rustfmt.endswith(")"):
        fail("rustfmt component does not match the locked Rust commit and release")

    clippy = output("cargo", "clippy", "--version").strip()
    clippy_prefix = f"clippy {lock['clippy_release']} ({lock['rust_commit'][:10]} "
    if not clippy.startswith(clippy_prefix) or not clippy.endswith(")"):
        fail("Clippy component does not match the locked Rust commit and release")


def check_specification() -> None:
    """Bind the compiler workspace to the immutable v0.8 bytes."""
    locked_text = (ROOT / "kernel-spec-v0.8.sha256").read_text(encoding="ascii")
    if len(locked_text) != 65 or not locked_text.endswith("\n"):
        fail("kernel specification lock must be 64 lowercase hex digits plus LF")
    locked = locked_text[:-1]
    if any(character not in "0123456789abcdef" for character in locked):
        fail("kernel specification lock is not lowercase hexadecimal")

    spec = REPOSITORY / "spec" / "kernel-spec-v0.8.md"
    actual = hashlib.sha256(spec.read_bytes()).hexdigest()
    if actual != locked:
        fail(f"kernel specification hash is {actual}, expected {locked}")


def manifest_table(path: Path, header: str) -> tuple[str, ...]:
    """Read one exact simple Cargo table without requiring host tomllib."""
    lines = path.read_text(encoding="utf-8").splitlines()
    positions = [index for index, line in enumerate(lines) if line.strip() == header]
    if len(positions) != 1:
        fail(f"{path.relative_to(ROOT)} must contain exactly one {header} table")
    body: list[str] = []
    for line in lines[positions[0] + 1 :]:
        stripped = line.strip()
        if stripped.startswith("["):
            break
        if stripped and not stripped.startswith("#"):
            body.append(stripped)
    return tuple(body)


def check_manifest_policy() -> None:
    """Require the reviewed inherited lint policy exactly."""
    root_manifest = ROOT / "Cargo.toml"
    expected_lints = {
        "[workspace.lints.rust]": (
            'unsafe_code = "forbid"',
            'rust_2018_idioms = { level = "deny", priority = -1 }',
            'unused_lifetimes = "deny"',
        ),
        "[workspace.lints.clippy]": (
            'all = { level = "deny", priority = -1 }',
            'unwrap_used = "deny"',
            'expect_used = "deny"',
            'panic = "deny"',
        ),
    }
    for header, expected in expected_lints.items():
        if manifest_table(root_manifest, header) != expected:
            fail(f"workspace lint policy drifted in {header}")
    lint_headers = {
        line.strip()
        for line in root_manifest.read_text(encoding="utf-8").splitlines()
        if line.strip().startswith("[workspace.lints")
    }
    if lint_headers != set(expected_lints):
        fail("workspace declared an unreviewed lint table")

    for name in EXPECTED_MEMBERS:
        member_path = ROOT / EXPECTED_MANIFESTS[name]
        if manifest_table(member_path, "[lints]") != ("workspace = true",):
            fail(f"{name} must inherit the complete workspace lint policy")
        member_lint_headers = {
            line.strip()
            for line in member_path.read_text(encoding="utf-8").splitlines()
            if line.strip().startswith("[lints")
        }
        if member_lint_headers != {"[lints]"}:
            fail(f"{name} declared a package-local lint override")


def normalized_dependency(dependency: dict) -> tuple:
    """Normalize one Cargo declaration for exact edge comparison."""
    dependency_path = dependency.get("path")
    normalized_path = None if dependency_path is None else relative(dependency_path)
    return (
        dependency.get("name"),
        dependency.get("req"),
        dependency.get("kind"),
        dependency.get("rename"),
        dependency.get("optional"),
        dependency.get("uses_default_features"),
        tuple(dependency.get("features", [])),
        dependency.get("target"),
        dependency.get("source"),
        dependency.get("registry"),
        normalized_path,
    )


def check_workspace_topology(metadata: dict) -> dict[str, dict]:
    """Close package membership, dependency edges, target kinds, and paths."""
    if Path(metadata.get("workspace_root", "")).resolve() != ROOT:
        fail("Cargo workspace root is not compiler/")

    package_by_id = {package["id"]: package for package in metadata.get("packages", [])}
    member_ids = metadata.get("workspace_members", [])
    default_member_ids = metadata.get("workspace_default_members", [])
    if member_ids != default_member_ids:
        fail("default workspace members differ from the complete workspace")
    try:
        members = [package_by_id[member_id] for member_id in member_ids]
    except KeyError as error:
        fail(f"workspace member is absent from Cargo metadata: {error}")
    if [package["name"] for package in members] != list(EXPECTED_MEMBERS):
        fail(f"workspace packages must be exactly {list(EXPECTED_MEMBERS)}")

    workspace_by_name = {package["name"]: package for package in members}
    if len(workspace_by_name) != len(members):
        fail("workspace package names are not unique")

    expected_dependency_rows = {
        "whitefoot-contract": (),
        "whitefoot-verifier": (
            (
                "whitefoot-contract",
                "*",
                None,
                None,
                False,
                True,
                (),
                None,
                None,
                None,
                Path("crates/whitefoot-contract"),
            ),
        ),
    }

    for name in EXPECTED_MEMBERS:
        package = workspace_by_name[name]
        expected_manifest = EXPECTED_MANIFESTS[name]
        if relative(package["manifest_path"]) != expected_manifest:
            fail(f"{name} manifest is not {expected_manifest}")
        if package.get("version") != "0.1.0":
            fail(f"{name} version is not 0.1.0")
        if package.get("license") != "MIT" or package.get("license_file") is not None:
            fail(f"{name} license identity drifted")
        if package.get("edition") != "2024" or package.get("rust_version") != "1.91.1":
            fail(f"{name} Rust language version drifted")
        if package.get("publish") != []:
            fail(f"{name} must remain non-publishable")
        if package.get("features") != {}:
            fail(f"{name} declared an unreviewed Cargo feature")
        if package.get("links") is not None:
            fail(f"{name} declared a native links contract")

        target_rows = package.get("targets", [])
        if len(target_rows) != 1:
            fail(f"{name} must have exactly one library target")
        target = target_rows[0]
        expected_target = EXPECTED_TARGETS[name]
        if (
            target.get("name") != expected_target["name"]
            or target.get("kind") != expected_target["kind"]
            or target.get("crate_types") != expected_target["crate_types"]
            or relative(target["src_path"]) != expected_target["source"]
            or target.get("edition") != "2024"
            or target.get("doc") is not True
            or target.get("doctest") is not expected_target["doctest"]
            or target.get("test") is not True
        ):
            fail(f"{name} target topology drifted")

        observed_dependencies = tuple(
            normalized_dependency(dependency) for dependency in package.get("dependencies", [])
        )
        if observed_dependencies != expected_dependency_rows[name]:
            fail(f"{name} dependency declarations drifted")

    resolve = metadata.get("resolve")
    if not isinstance(resolve, dict):
        fail("Cargo metadata omitted the resolved dependency graph")
    nodes = {node["id"]: node for node in resolve.get("nodes", [])}
    for name in EXPECTED_MEMBERS:
        package = workspace_by_name[name]
        node = nodes.get(package["id"])
        if node is None:
            fail(f"resolved graph omitted {name}")
        if node.get("features") != []:
            fail(f"{name} resolved unreviewed features")
        observed_edges = tuple(package_by_id[edge]["name"] for edge in node.get("dependencies", []))
        if observed_edges != EXPECTED_EDGES[name]:
            fail(f"{name} resolved dependency edges drifted")

    for package in metadata.get("packages", []):
        for target in package.get("targets", []):
            kinds = set(target.get("kind", []))
            crate_types = set(target.get("crate_types", []))
            if "custom-build" in kinds:
                fail(f"build scripts are forbidden: {package['name']} {package['version']}")
            if "proc-macro" in kinds or "proc-macro" in crate_types:
                fail(f"procedural macros are forbidden: {package['name']} {package['version']}")

    return package_by_id


def dependency_key(entry: dict) -> tuple[str, str, str]:
    """Return one complete external package identity."""
    return (entry["name"], entry["version"], entry["source"])


def cargo_lock_packages(
    path: Path,
) -> tuple[int, dict[tuple[str, str, Optional[str]], dict]]:
    """Read Cargo-generated lock identities without a host TOML dependency."""
    text = path.read_text(encoding="utf-8")
    version_match = re.search(r"(?m)^version = ([0-9]+)$", text)
    if version_match is None:
        fail("Cargo.lock has no format version")
    version = int(version_match.group(1))
    packages: dict[tuple[str, str, Optional[str]], dict] = {}
    for block in text.split("[[package]]")[1:]:
        fields: dict[str, str] = {}
        for name in ("name", "version", "source", "checksum"):
            matches = re.findall(rf'(?m)^{name} = "([^"\\]*)"$', block)
            if len(matches) > 1:
                fail(f"Cargo.lock package contains duplicate {name} fields")
            if matches:
                fields[name] = matches[0]
        if "name" not in fields or "version" not in fields:
            fail("Cargo.lock contains an incomplete package identity")
        key = (fields["name"], fields["version"], fields.get("source"))
        if key in packages:
            fail(f"Cargo.lock contains duplicate package identity {key}")
        packages[key] = fields
    return version, packages


def check_dependencies(metadata: dict, package_by_id: dict[str, dict]) -> None:
    """Bind every external package to the policy and exact Cargo.lock bytes."""
    policy = read_json(ROOT / "dependencies.json")
    if set(policy) != {"schema", "cargo_lock", "dependencies"} or policy.get("schema") != 1:
        fail("dependencies.json has an unknown or incomplete top-level schema")
    lock_identity = policy.get("cargo_lock")
    if not isinstance(lock_identity, dict) or set(lock_identity) != {"version", "sha256"}:
        fail("dependencies.json has an incomplete Cargo.lock identity")
    if lock_identity.get("version") != 4:
        fail("dependencies.json does not approve Cargo.lock format version 4")
    if lock_identity.get("sha256") != sha256(ROOT / "Cargo.lock"):
        fail("Cargo.lock bytes do not match dependencies.json")

    lock_version, lock_packages = cargo_lock_packages(ROOT / "Cargo.lock")
    if lock_version != lock_identity["version"]:
        fail("Cargo.lock format version does not match dependencies.json")

    metadata_keys = {
        (package["name"], package["version"], package.get("source"))
        for package in metadata.get("packages", [])
    }
    if set(lock_packages) != metadata_keys:
        fail("Cargo.lock package set differs from the resolved dependency graph")

    approved_entries = policy.get("dependencies")
    if not isinstance(approved_entries, list):
        fail("dependencies.json dependencies must be an array")
    approved: dict[tuple[str, str, str], dict] = {}
    for entry in approved_entries:
        if not isinstance(entry, dict) or set(entry) != DEPENDENCY_FIELDS:
            fail("external dependency approval has an incomplete schema")
        if not all(isinstance(entry[field], str) and entry[field] for field in (
            "name",
            "version",
            "source",
            "checksum",
            "license",
            "purpose",
        )):
            fail(f"external dependency approval contains an empty identity: {entry!r}")
        if (
            len(entry["checksum"]) != 64
            or any(character not in "0123456789abcdef" for character in entry["checksum"])
        ):
            fail(f"invalid dependency checksum for {entry['name']} {entry['version']}")
        features = entry["features"]
        if (
            not isinstance(features, list)
            or any(not isinstance(feature, str) or not feature for feature in features)
            or features != sorted(set(features))
        ):
            fail(f"dependency features are not a sorted unique string list: {entry['name']}")
        if any(not isinstance(entry[field], bool) for field in (
            "build_script",
            "proc_macro",
            "unsafe_tcb",
        )):
            fail(f"dependency review flags are not booleans: {entry['name']}")
        if entry["build_script"] or entry["proc_macro"]:
            fail(f"build-script and proc-macro dependencies are forbidden: {entry['name']}")
        key = dependency_key(entry)
        if key in approved:
            fail(f"duplicate dependency approval for {key}")
        approved[key] = entry

    member_ids = set(metadata.get("workspace_members", []))
    resolve_nodes = {
        node["id"]: node for node in metadata.get("resolve", {}).get("nodes", [])
    }
    observed: set[tuple[str, str, str]] = set()
    for package_id, package in package_by_id.items():
        if package_id in member_ids:
            continue
        source = package.get("source")
        if source is None:
            fail(f"external path dependency is forbidden: {package['manifest_path']}")
        key = (package["name"], package["version"], source)
        entry = approved.get(key)
        if entry is None:
            fail(f"unapproved dependency: {key}")
        lock_package = lock_packages.get(key)
        if lock_package is None or lock_package.get("checksum") != entry["checksum"]:
            fail(f"Cargo.lock checksum differs for {key}")
        if package.get("license") != entry["license"]:
            fail(f"license drift for {key}")
        resolved = resolve_nodes.get(package_id)
        if resolved is None or sorted(resolved.get("features", [])) != entry["features"]:
            fail(f"resolved feature drift for {key}")
        observed.add(key)

    if set(approved) != observed:
        fail(f"unused dependency approvals: {sorted(set(approved) - observed)}")


def metadata() -> dict:
    """Return the locked, offline Cargo graph."""
    return json.loads(
        output(
            sys.executable,
            "-B",
            str(ROOT / "tools" / "cargo_policy.py"),
            "metadata",
            "--format-version",
            "1",
            "--locked",
            "--offline",
        )
    )


def main() -> None:
    """Run every closed-world workspace policy check."""
    check_environment()
    check_no_source_symlinks()
    check_no_workspace_build_tool_configuration()
    check_no_production_facet_ids()
    check_compile_time_inputs()
    check_toolchain()
    check_specification()
    check_manifest_policy()
    cargo_metadata = metadata()
    package_by_id = check_workspace_topology(cargo_metadata)
    check_dependencies(cargo_metadata, package_by_id)
    print(
        "workspace policy: exact topology, toolchain, specification, lints, "
        "targets, dependency lock, source paths, and direct facet-ID source fence verified"
    )


if __name__ == "__main__":
    main()
