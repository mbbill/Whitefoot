from pathlib import Path
import sys
import tempfile
import unittest


sys.path.insert(0, str(Path(__file__).parent))

from dependency_audit import DependencyAuditError, audit  # noqa: E402


class DependencyAuditTests(unittest.TestCase):
    def test_live_route_has_closed_file_and_import_sets(self) -> None:
        audit(Path(__file__).parent)

    def test_extra_file_and_external_import_fail_closed(self) -> None:
        source = Path(__file__).parent
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for path in source.iterdir():
                if path.is_file():
                    (root / path.name).write_bytes(path.read_bytes())
            extra = root / "unexpected.py"
            extra.write_text("import requests\n", encoding="utf-8")
            with self.assertRaises(DependencyAuditError):
                audit(root)
            extra.unlink()
            manifest = root / "manifest.py"
            manifest.write_text(
                manifest.read_text(encoding="utf-8") + "\nimport requests\n",
                encoding="utf-8",
            )
            with self.assertRaises(DependencyAuditError):
                audit(root)


if __name__ == "__main__":
    unittest.main()
