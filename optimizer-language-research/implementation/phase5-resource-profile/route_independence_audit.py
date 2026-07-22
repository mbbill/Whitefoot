"""Closed static dependency audit for both resource-evidence routes.

This audit is deliberately outside both routes.  It does not prove that either
route is correct; it proves the narrower construction fact that their non-test
Python files stay inside two reviewed dependency and filesystem boundaries.
"""

from __future__ import annotations

import ast
from pathlib import Path


ROOT = Path(__file__).resolve().parent
ROUTE_ROOTS = {
    "source": ROOT / "source-route",
    "analytic": ROOT / "analytic-route",
}

EXPECTED_IMPORTS = {
    "source": {
        "counts.py": {"__future__", "dataclasses", "model", "roles", "topology"},
        "fn8.py": {"__future__", "model", "topology"},
        "identities.py": {"__future__", "ast", "hashlib", "model", "pathlib"},
        "inventory_selection.py": {
            "__future__", "model", "resolution", "topology",
        },
        "lexical_selection.py": {
            "__future__", "model", "resolution", "roles", "topology",
        },
        "manifest.py": {
            "__future__", "hashlib", "identities", "json", "model",
            "parser_adapter",
        },
        "measurement.py": {
            "__future__", "counts", "fn8", "hashlib", "identities",
            "inventory_selection", "lexical_selection", "manifest", "model",
            "parser_adapter", "receipt_schema", "resolution", "roles", "struct",
            "topology",
        },
        "model.py": {"__future__", "dataclasses", "typing"},
        "parser_adapter.py": {
            "__future__", "form2_independent_lex", "form2_independent_parse",
            "identities", "model", "pathlib", "re", "sys",
        },
        "receipt.py": {
            "__future__", "json", "measurement", "model", "receipt_validation",
        },
        "receipt_diagnostic.py": {
            "__future__", "dataclasses", "model", "receipt_values",
        },
        "receipt_schema.py": {"counts"},
        "receipt_structure.py": {
            "__future__", "counts", "dataclasses", "hashlib", "identities",
            "model", "parser_adapter", "receipt_diagnostic", "receipt_schema",
            "receipt_values", "struct",
        },
        "receipt_validation.py": {"__future__", "model", "receipt_structure"},
        "receipt_values.py": {"__future__", "model", "receipt_schema"},
        "resolution.py": {
            "__future__", "dataclasses", "model", "roles", "topology",
        },
        "roles.py": {
            "__future__", "dataclasses", "identities", "model", "re", "topology",
        },
        "run.py": {
            "__future__", "argparse", "hashlib", "model", "os", "pathlib",
            "receipt", "stat", "tempfile",
        },
        "topology.py": {"__future__", "dataclasses", "model", "typing"},
    },
    "analytic": {
        "dependency_audit.py": {"ast", "pathlib", "receipt"},
        "manifest.py": {"dataclasses", "hashlib", "json", "pathlib", "typing"},
        "measure.py": {
            "__future__", "dataclasses", "manifest", "relation", "selection",
            "typing",
        },
        "receipt.py": {
            "dataclasses", "hashlib", "manifest", "measure", "pathlib", "stat",
            "struct",
        },
        "relation.py": {"__future__", "dataclasses", "manifest"},
        "run.py": {
            "__future__", "argparse", "hashlib", "json", "manifest", "measure",
            "os", "pathlib", "receipt", "stat", "tempfile",
        },
        "selection.py": {"__future__", "relation"},
    },
}

PARSER_FILES = {
    "form2_independent_lex.py",
    "form2_independent_parse.py",
    "form2_independent_parse_core.py",
    "form2_independent_parse_expressions.py",
    "form2_independent_parse_items.py",
    "form2_independent_parse_signatures.py",
    "form2_independent_parse_statements.py",
    "form2_independent_parse_types.py",
    "form2_independent_syntax.py",
}
PARSER_IMPORTS = {"form2_independent_lex", "form2_independent_parse"}
NEUTRAL_SOURCE_INPUTS = {
    "PROPOSAL.md",
    "kernel-spec-v0.10-candidate.md",
    "schema.py",
    "SCHEMA-SEMANTICS.md",
    "WORK-SCHEDULE.md",
    "STORAGE-MODEL.md",
}

GLOBAL_FORBIDDEN_TEXT = (
    "evidence_manifest",
    "cross_route_agreement",
    "workloads",
    "diagnostic_evidence",
    "archive/",
    "compiler/",
    "whitefoot_",
)
ROUTE_FORBIDDEN_TEXT = {
    "source": ("analytic-route", "analytic_route", "route-b"),
    "analytic": (
        "source-route", "source_route", "route-a", "grammar-verifier",
        "form2_independent",
    ),
}
TEXT_EXCEPTIONS = {
    ("source", "identities.py"): {
        "analytic-route", "cross_route_agreement", "diagnostic_evidence",
        "evidence_manifest", "workloads", "archive/", "whitefoot_", "route-b",
    },
    ("analytic", "dependency_audit.py"): {
        "evidence_manifest", "workloads", "diagnostic_evidence", "archive/",
        "compiler/", "source-route", "grammar-verifier",
    },
}

SENSITIVE_CALLS = {
    "Path", "open", "read_bytes", "read_text", "write_bytes", "write_text",
    "mkdir", "unlink", "replace", "rename", "glob", "rglob", "iterdir",
    "stat", "lstat", "is_file", "is_dir", "is_symlink", "resolve",
    "absolute", "touch", "symlink_to", "chmod", "fstat", "read", "close",
    "NamedTemporaryFile", "fsync", "abspath",
}
ALLOWED_SENSITIVE_CALLS = {
    ("source", "identities.py"): {
        "Path", "resolve", "is_file", "is_symlink", "read_bytes", "glob",
    },
    ("source", "parser_adapter.py"): {"Path", "resolve"},
    ("source", "roles.py"): {"read_bytes"},
    ("source", "run.py"): {
        "Path", "lstat", "open", "fstat", "read", "close", "is_dir",
        "NamedTemporaryFile", "fsync", "replace", "unlink", "abspath",
    },
    ("analytic", "dependency_audit.py"): {
        "iterdir", "is_dir", "is_symlink", "is_file", "glob", "read_text",
    },
    ("analytic", "receipt.py"): {"Path", "lstat", "read_bytes", "stat"},
    ("analytic", "run.py"): {
        "Path", "lstat", "open", "fstat", "read", "close", "is_dir",
        "NamedTemporaryFile", "fsync", "replace", "unlink", "abspath",
    },
}
DYNAMIC_NAME_CALLS = {"eval", "exec", "compile", "__import__"}
DYNAMIC_ATTRIBUTE_CALLS = {"eval", "exec", "import_module", "run_module", "run_path"}
DYNAMIC_LITERAL_EXCEPTIONS = {("analytic", "dependency_audit.py"): {"__import__"}}


class RouteIndependenceError(ValueError):
    """A route escaped the reviewed static independence boundary."""


def _imports(tree: ast.AST, filename: str) -> set[str]:
    result: set[str] = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            result.update(alias.name for alias in node.names)
        elif isinstance(node, ast.ImportFrom):
            if node.level != 0 or node.module is None:
                raise RouteIndependenceError(f"relative import in {filename}")
            result.add(node.module)
    return result


def _call_name(node: ast.Call) -> str | None:
    if isinstance(node.func, ast.Name):
        return node.func.id
    if isinstance(node.func, ast.Attribute):
        return node.func.attr
    return None


def _joined_literal(node: ast.AST) -> str | None:
    if isinstance(node, ast.Constant) and isinstance(node.value, (str, bytes)):
        if isinstance(node.value, bytes):
            return node.value.decode("ascii", errors="ignore")
        return node.value
    if isinstance(node, ast.BinOp) and isinstance(node.op, ast.Add):
        left = _joined_literal(node.left)
        right = _joined_literal(node.right)
        if left is not None and right is not None:
            return left + right
    return None


def _contains_reference(tree: ast.AST, text: str, fragment: str) -> bool:
    if fragment in text:
        return True
    return any(
        (literal := _joined_literal(node)) is not None and fragment in literal
        for node in ast.walk(tree)
    )


def _literal_assignment(tree: ast.AST, name: str, filename: str) -> object:
    matches = [
        node.value
        for node in ast.walk(tree)
        if isinstance(node, ast.Assign)
        and any(isinstance(target, ast.Name) and target.id == name for target in node.targets)
    ]
    if len(matches) != 1:
        raise RouteIndependenceError(f"{filename} does not define one {name}")
    try:
        return ast.literal_eval(matches[0])
    except (TypeError, ValueError) as error:
        raise RouteIndependenceError(f"{filename} {name} is not literal data") from error


def audit_python_source(route: str, filename: str, text: str) -> ast.AST:
    """Audit one file against its exact route-local static closure."""

    if route not in EXPECTED_IMPORTS or filename not in EXPECTED_IMPORTS[route]:
        raise RouteIndependenceError(f"unapproved route file: {route}/{filename}")
    try:
        tree = ast.parse(text, filename=f"{route}/{filename}")
    except SyntaxError as error:
        raise RouteIndependenceError(f"cannot parse {route}/{filename}") from error
    observed_imports = _imports(tree, filename)
    expected_imports = EXPECTED_IMPORTS[route][filename]
    if observed_imports != expected_imports:
        raise RouteIndependenceError(
            f"import closure differs in {route}/{filename}: "
            f"expected={sorted(expected_imports)} observed={sorted(observed_imports)}"
        )

    exceptions = TEXT_EXCEPTIONS.get((route, filename), set())
    for fragment in GLOBAL_FORBIDDEN_TEXT + ROUTE_FORBIDDEN_TEXT[route]:
        if _contains_reference(tree, text, fragment) and fragment not in exceptions:
            raise RouteIndependenceError(
                f"forbidden reference {fragment!r} in {route}/{filename}"
            )

    allowed_calls = ALLOWED_SENSITIVE_CALLS.get((route, filename), set())
    for node in ast.walk(tree):
        if not isinstance(node, ast.Call):
            continue
        name = _call_name(node)
        dynamic = (
            isinstance(node.func, ast.Name) and name in DYNAMIC_NAME_CALLS
        ) or (
            isinstance(node.func, ast.Attribute) and name in DYNAMIC_ATTRIBUTE_CALLS
        )
        if dynamic:
            raise RouteIndependenceError(
                f"dynamic code or import call {name} in {route}/{filename}"
            )
        if name in SENSITIVE_CALLS and name not in allowed_calls:
            raise RouteIndependenceError(
                f"unapproved filesystem call {name} in {route}/{filename}"
            )
    dynamic_literals = DYNAMIC_NAME_CALLS | DYNAMIC_ATTRIBUTE_CALLS
    literal_exceptions = DYNAMIC_LITERAL_EXCEPTIONS.get((route, filename), set())
    for node in ast.walk(tree):
        literal = _joined_literal(node)
        if literal in dynamic_literals and literal not in literal_exceptions:
            raise RouteIndependenceError(
                f"dynamic code or import name {literal} in {route}/{filename}"
            )

    parser_modules = {name for name in observed_imports if name.startswith("form2_")}
    if parser_modules and (route, filename) != ("source", "parser_adapter.py"):
        raise RouteIndependenceError(f"parser import escaped its adapter in {filename}")
    if (route, filename) == ("source", "parser_adapter.py"):
        if parser_modules != PARSER_IMPORTS or "sys.path.insert" not in text:
            raise RouteIndependenceError("source parser adapter boundary is not exact")
    elif "sys.path" in text:
        raise RouteIndependenceError(f"sys.path authority escaped the parser adapter in {filename}")
    return tree


def _audit_source_boundary(trees: dict[str, ast.AST], texts: dict[str, str]) -> None:
    identities = trees["identities.py"]
    parser_files = _literal_assignment(identities, "PARSER_FILES", "identities.py")
    if not isinstance(parser_files, dict) or set(parser_files) != PARSER_FILES:
        raise RouteIndependenceError("pinned grammar-verifier parser file set differs")
    if any(
        not isinstance(value, str)
        or len(value) != 64
        or any(character not in "0123456789abcdef" for character in value)
        for value in parser_files.values()
    ):
        raise RouteIndependenceError("pinned parser digest is not lowercase SHA-256")
    route_files = _literal_assignment(identities, "ROUTE_FILES", "identities.py")
    if set(route_files) != set(EXPECTED_IMPORTS["source"]):
        raise RouteIndependenceError("source route's own closed file list differs")
    identity_text = texts["identities.py"]
    if 'PARSER_ROOT = REPOSITORY_ROOT / "grammar-verifier/proposal"' not in identity_text:
        raise RouteIndependenceError("source parser root is not the approved pinned boundary")
    missing_inputs = sorted(name for name in NEUTRAL_SOURCE_INPUTS if name not in identity_text)
    if missing_inputs:
        raise RouteIndependenceError(f"source neutral input set is incomplete: {missing_inputs}")
    allowed_inputs_by_file = {
        "identities.py": NEUTRAL_SOURCE_INPUTS,
        "measurement.py": {"schema.py"},
        "receipt_structure.py": {
            "schema.py", "SCHEMA-SEMANTICS.md", "STORAGE-MODEL.md", "WORK-SCHEDULE.md",
        },
        "roles.py": {"kernel-spec-v0.10-candidate.md"},
    }
    for filename, tree in trees.items():
        allowed = NEUTRAL_SOURCE_INPUTS if filename == "identities.py" else set()
        allowed = allowed_inputs_by_file.get(filename, allowed)
        mentioned = {
            literal
            for node in ast.walk(tree)
            if (literal := _joined_literal(node)) in NEUTRAL_SOURCE_INPUTS
        }
        escaped = sorted(mentioned - allowed)
        if escaped:
            raise RouteIndependenceError(
                f"neutral profile/spec input escaped its approved reader in {filename}: {escaped}"
            )


def audit(root: Path = ROOT) -> None:
    """Audit both exact non-test route file sets and their source text."""

    trees_by_route: dict[str, dict[str, ast.AST]] = {}
    texts_by_route: dict[str, dict[str, str]] = {}
    for route, directory_name in (("source", "source-route"), ("analytic", "analytic-route")):
        directory = root / directory_name
        observed = {
            path.name
            for path in directory.glob("*.py")
            if not path.name.startswith("test_")
        }
        expected = set(EXPECTED_IMPORTS[route])
        if observed != expected:
            raise RouteIndependenceError(
                f"{route} non-test Python file set differs: "
                f"missing={sorted(expected - observed)} extra={sorted(observed - expected)}"
            )
        trees: dict[str, ast.AST] = {}
        texts: dict[str, str] = {}
        for filename in sorted(expected):
            path = directory / filename
            if path.is_symlink() or not path.is_file():
                raise RouteIndependenceError(f"route file is not regular: {route}/{filename}")
            try:
                text = path.read_text(encoding="utf-8")
            except (OSError, UnicodeDecodeError) as error:
                raise RouteIndependenceError(f"cannot read {route}/{filename}") from error
            texts[filename] = text
            trees[filename] = audit_python_source(route, filename, text)
        trees_by_route[route] = trees
        texts_by_route[route] = texts
    _audit_source_boundary(trees_by_route["source"], texts_by_route["source"])


def main() -> int:
    audit()
    print("route-independence-audit: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
