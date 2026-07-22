"""Closed file and Python dependency audit for the analytic route."""

import ast
from pathlib import Path

from receipt import CODE_FILES


ALLOWED_FILES = frozenset(
    (
        "README.md",
        "dependency_audit.py",
        "manifest.py",
        "measure.py",
        "receipt.py",
        "relation.py",
        "run.py",
        "selection.py",
        "test_dependency_audit.py",
        "test_manifest.py",
        "test_measure.py",
        "test_receipt.py",
        "test_run.py",
        "test_selection.py",
    )
)
RUNTIME_FILES = (
    "dependency_audit.py",
    "manifest.py",
    "measure.py",
    "receipt.py",
    "relation.py",
    "run.py",
    "selection.py",
)
LOCAL_MODULES = frozenset(
    (
        "dependency_audit",
        "manifest",
        "measure",
        "receipt",
        "relation",
        "run",
        "selection",
        "test_manifest",
    )
)
STANDARD_MODULES = frozenset(
    (
        "argparse",
        "ast",
        "__future__",
        "dataclasses",
        "hashlib",
        "json",
        "os",
        "pathlib",
        "stat",
        "struct",
        "subprocess",
        "sys",
        "tempfile",
        "typing",
        "unittest",
    )
)
FORBIDDEN_RUNTIME_TEXT = (
    "evidence" + "_manifest",
    "source-route",
    "grammar-verifier",
    "diagnostic_evidence",
    "workloads.py",
    "archive/",
    "compiler/",
)


class DependencyAuditError(ValueError):
    """The route file set or dependency closure is not sealed."""


def audit(root: Path) -> None:
    if CODE_FILES != RUNTIME_FILES:
        raise DependencyAuditError("receipt code identity and audit runtime set differ")
    observed_files = set()
    for entry in root.iterdir():
        if entry.name == "__pycache__" and entry.is_dir() and not entry.is_symlink():
            continue
        if entry.is_symlink() or not entry.is_file():
            raise DependencyAuditError(f"unexpected non-regular route entry: {entry.name}")
        observed_files.add(entry.name)
    if observed_files != ALLOWED_FILES:
        missing = sorted(ALLOWED_FILES - observed_files)
        extra = sorted(observed_files - ALLOWED_FILES)
        raise DependencyAuditError(f"route file set differs: missing={missing} extra={extra}")

    allowed_modules = STANDARD_MODULES | LOCAL_MODULES
    for path in sorted(root.glob("*.py")):
        text = path.read_text(encoding="utf-8")
        try:
            tree = ast.parse(text, filename=str(path))
        except SyntaxError as error:
            raise DependencyAuditError(f"cannot parse dependency source: {path.name}") from error
        for node in ast.walk(tree):
            modules = []
            if isinstance(node, ast.Import):
                modules = [alias.name.split(".", 1)[0] for alias in node.names]
            elif isinstance(node, ast.ImportFrom):
                if node.level != 0 or node.module is None:
                    raise DependencyAuditError(f"relative import in {path.name}")
                modules = [node.module.split(".", 1)[0]]
            elif isinstance(node, ast.Call) and isinstance(node.func, ast.Name):
                if node.func.id == "__import__":
                    raise DependencyAuditError(f"dynamic import in {path.name}")
            for module in modules:
                if module not in allowed_modules:
                    raise DependencyAuditError(
                        f"dependency {module!r} is outside the closure in {path.name}"
                    )
        if not path.name.startswith("test_") and path.name != "dependency_audit.py":
            for forbidden in FORBIDDEN_RUNTIME_TEXT:
                if forbidden in text:
                    raise DependencyAuditError(
                        f"forbidden runtime dependency text {forbidden!r} in {path.name}"
                    )
