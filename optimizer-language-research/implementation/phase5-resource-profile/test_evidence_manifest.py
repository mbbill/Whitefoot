from hashlib import sha256
from pathlib import Path
import unittest

from evidence_manifest import ManifestError, build_manifest, encode_manifest


class EvidenceManifestTests(unittest.TestCase):
    def build(self, **changes: object) -> dict[str, object]:
        arguments: dict[str, object] = {
            "family": "codec",
            "units": 2,
            "revision": "ab" * 32,
            "parameters": {
                "name_decimal_width": 6,
                "source_records": 1,
                "unit_count": 2,
            },
            "sources": (("demand/codec.wf", b"fn main() {}\n"),),
        }
        arguments.update(changes)
        return build_manifest(**arguments)  # type: ignore[arg-type]

    def test_encoding_is_exact_ascii_json_with_terminal_lf(self) -> None:
        manifest = self.build()
        first = encode_manifest(manifest)
        second = encode_manifest(dict(reversed(tuple(manifest.items()))))
        self.assertEqual(first, second)
        self.assertEqual(first[-1:], b"\n")
        first.decode("ascii")

    def test_source_bytes_path_and_revision_are_bound(self) -> None:
        first = self.build()
        second = self.build(sources=(("demand/a.wf", b"a"),))
        third = self.build(sources=(("demand/b.wf", b"a"),))
        self.assertNotEqual(encode_manifest(first), encode_manifest(second))
        self.assertNotEqual(encode_manifest(second), encode_manifest(third))
        self.assertEqual(second["sources"][0]["sha256"], sha256(b"a").hexdigest())

    def test_invalid_scalar_path_digest_and_duplicate_path_fail(self) -> None:
        invalid = (
            {"units": 0},
            {"units": True},
            {"revision": "AB" * 32},
            {"revision": "ab" * 31},
            {"sources": (("../escape.wf", b"x"),)},
            {"sources": (("demand//noncanonical.wf", b"x"),)},
            {"sources": (("demand\\host.wf", b"x"),)},
            {"sources": (("demand/not@portable.wf", b"x"),)},
            {"parameters": {"unit_count": 2}},
            {
                "parameters": {
                    "name_decimal_width": 7,
                    "source_records": 1,
                    "unit_count": 2,
                }
            },
            {
                "sources": (
                    ("demand/same.wf", b"x"),
                    ("demand/same.wf", b"y"),
                )
            },
        )
        for changes in invalid:
            with self.subTest(changes=changes), self.assertRaises(ManifestError):
                self.build(**changes)

    def test_workload_routes_must_not_import_the_producer_codec(self) -> None:
        root = Path(__file__).parent
        for directory_name in ("source-route", "analytic-route"):
            directory = root / directory_name
            if not directory.exists():
                continue
            for path in directory.rglob("*.py"):
                self.assertNotIn("evidence_manifest", path.read_text())


if __name__ == "__main__":
    unittest.main()
