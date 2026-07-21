from __future__ import annotations

import ast
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[1]
REPOSITORY = ROOT.parent


class IndependenceTests(unittest.TestCase):
    def test_production_compiler_does_not_import_or_invoke_verifier(self) -> None:
        compiler = REPOSITORY / "compiler"
        for path in compiler.rglob("*"):
            if not path.is_file() or "target" in path.parts or path.suffix not in {".rs", ".py", ".toml", ".lock"}:
                continue
            with self.subTest(path=path.relative_to(REPOSITORY).as_posix()):
                self.assertNotIn("grammar-verifier", path.read_text(encoding="utf-8"))

    def test_oracle_cannot_import_runner_static_or_compiler(self) -> None:
        oracle = ROOT / "oracle"
        if not oracle.exists():
            self.skipTest("Oracle source is being implemented independently")
        forbidden = {
            "runner_common_report",
            "runner_common_schema",
            "runner_common_wire",
            "runner_inputs",
            "runner_oracle_report",
            "runner_package",
            "runner_process",
            "runner_report",
            "runner_report_wire",
            "runner_static_report",
            "runner_trace",
            "run",
            "compiler",
            "static_auditor",
        }
        for path in oracle.rglob("*.py"):
            tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
            for node in ast.walk(tree):
                if isinstance(node, ast.Import):
                    names = {alias.name.split(".", 1)[0] for alias in node.names}
                elif isinstance(node, ast.ImportFrom) and node.level == 0:
                    names = {(node.module or "").split(".", 1)[0]}
                else:
                    continue
                with self.subTest(path=path.name, line=node.lineno):
                    self.assertTrue(names.isdisjoint(forbidden))

    def test_runner_does_not_import_either_engine(self) -> None:
        for path in (
            ROOT / "run.py",
            ROOT / "runner_common_report.py",
            ROOT / "runner_common_schema.py",
            ROOT / "runner_common_wire.py",
            ROOT / "runner_inputs.py",
            ROOT / "runner_oracle_report.py",
            ROOT / "runner_package.py",
            ROOT / "runner_process.py",
            ROOT / "runner_report.py",
            ROOT / "runner_report_wire.py",
            ROOT / "runner_static_report.py",
            ROOT / "runner_trace.py",
        ):
            text = path.read_text(encoding="utf-8")
            self.assertNotIn("from oracle", text)
            self.assertNotIn("import oracle", text)
            self.assertNotIn("from static", text)
            self.assertNotIn("import static", text)


if __name__ == "__main__":
    unittest.main()
