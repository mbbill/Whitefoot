import unittest

from route_independence_audit import (
    EXPECTED_IMPORTS,
    RouteIndependenceError,
    audit,
    audit_python_source,
)


class RouteIndependenceAuditTests(unittest.TestCase):
    def test_live_routes_have_closed_independent_boundaries(self) -> None:
        audit()

    def _minimal_source(self, route: str, filename: str) -> str:
        lines = []
        for module in sorted(EXPECTED_IMPORTS[route][filename]):
            lines.append(f"import {module}")
        return "\n".join(lines) + "\n"

    def test_rejects_cross_route_reference(self) -> None:
        text = self._minimal_source("analytic", "selection.py")
        text += 'value = "source-route"\n'
        with self.assertRaises(RouteIndependenceError):
            audit_python_source("analytic", "selection.py", text)

    def test_rejects_producer_and_proposal_model_references(self) -> None:
        base = self._minimal_source("source", "fn8.py")
        for fragment in ("workloads.py", "evidence_manifest", "diagnostic_evidence"):
            with self.subTest(fragment=fragment):
                with self.assertRaises(RouteIndependenceError):
                    audit_python_source("source", "fn8.py", base + repr(fragment))

    def test_rejects_dynamic_code_and_imports(self) -> None:
        base = self._minimal_source("source", "fn8.py")
        for expression in ("eval('1')", "exec('pass')", "__import__('json')"):
            with self.subTest(expression=expression):
                with self.assertRaises(RouteIndependenceError):
                    audit_python_source("source", "fn8.py", base + expression)

    def test_rejects_unapproved_filesystem_authority(self) -> None:
        text = self._minimal_source("source", "fn8.py") + "open('outside')\n"
        with self.assertRaises(RouteIndependenceError):
            audit_python_source("source", "fn8.py", text)

    def test_rejects_import_outside_exact_file_closure(self) -> None:
        text = self._minimal_source("analytic", "selection.py") + "import subprocess\n"
        with self.assertRaises(RouteIndependenceError):
            audit_python_source("analytic", "selection.py", text)


if __name__ == "__main__":
    unittest.main()
