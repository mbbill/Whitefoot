import json
from hashlib import sha256
from pathlib import Path
import sys
import unittest


sys.path.insert(0, str(Path(__file__).parent))

from manifest import (  # noqa: E402
    FAMILY_CODEC,
    FAMILY_COMPILER,
    GENERATOR_REVISION,
    ManifestError,
    PARAMETER_NAMES,
    SourceIdentity,
    WorkloadManifest,
    decode,
    encode,
    verify_source_bytes,
)


def fixture(family: str = FAMILY_COMPILER, units: int = 1) -> tuple[WorkloadManifest, bytes]:
    source_length = (116 + 477 * units) if family == FAMILY_COMPILER else (46 + 696 * units)
    source = bytes((index * 17 + 3) & 0xFF for index in range(source_length))
    manifest = WorkloadManifest(
        family=family,
        units=units,
        generator_revision=GENERATOR_REVISION,
        parameters=(
            ("name_decimal_width", 6),
            ("source_records", 1),
            ("unit_count", units),
        ),
        sources=(
            SourceIdentity(
                f"demand/{family}-{units:06d}.wf",
                len(source),
                sha256(source).hexdigest(),
            ),
        ),
    )
    return manifest, source


class ManifestTests(unittest.TestCase):
    def test_round_trip_pins_exact_canonical_bytes(self) -> None:
        manifest, source = fixture()
        encoded = encode(manifest)
        self.assertTrue(encoded.endswith(b"\n"))
        self.assertEqual(decode(encoded), manifest)
        self.assertEqual(verify_source_bytes(manifest, (source,)), (sha256(source).digest(),))
        parsed = json.loads(encoded)
        self.assertEqual(tuple(parsed), (
            "family", "generator_revision", "parameters", "schema", "sources", "units"
        ))
        self.assertEqual(tuple(record["name"] for record in parsed["parameters"]), PARAMETER_NAMES)

    def test_noncanonical_json_and_unknown_fields_fail_closed(self) -> None:
        manifest, _ = fixture()
        encoded = encode(manifest)
        self.assertRaises(ManifestError, decode, encoded[:-1])
        self.assertRaises(ManifestError, decode, encoded.replace(b'"family":', b'"extra":0,"family":', 1))
        self.assertRaises(ManifestError, decode, encoded.replace(b'"units":1', b'"units":true'))
        self.assertRaises(ManifestError, decode, encoded.replace(b'"name_decimal_width"', b'"other"'))
        duplicate = encoded.replace(b'"units":1}', b'"units":1,"units":1}')
        self.assertRaises(ManifestError, decode, duplicate)

    def test_generator_revision_unit_and_source_relations_are_closed(self) -> None:
        manifest, _ = fixture()
        self.assertRaises(
            ManifestError,
            encode,
            WorkloadManifest(
                manifest.family,
                manifest.units,
                "0" * 64,
                manifest.parameters,
                manifest.sources,
            ),
        )
        bad_parameters = tuple(
            (name, 2 if name == "source_records" else value)
            for name, value in manifest.parameters
        )
        self.assertRaises(
            ManifestError,
            encode,
            WorkloadManifest(
                manifest.family,
                manifest.units,
                manifest.generator_revision,
                bad_parameters,
                manifest.sources,
            ),
        )
        mismatched_units = tuple(
            (name, 2 if name == "unit_count" else value)
            for name, value in manifest.parameters
        )
        self.assertRaises(
            ManifestError,
            encode,
            WorkloadManifest(
                manifest.family,
                manifest.units,
                manifest.generator_revision,
                mismatched_units,
                manifest.sources,
            ),
        )

    def test_path_alphabet_and_normalization_are_exact(self) -> None:
        manifest, _ = fixture()
        for path in (
            "/absolute.wf", "a//b.wf", "a/./b.wf", "a/../b.wf", "a\\b.wf",
            "a b.wf", "a@b.wf", "é.wf", "a/",
        ):
            source = manifest.sources[0]
            mutated = WorkloadManifest(
                manifest.family,
                manifest.units,
                manifest.generator_revision,
                manifest.parameters,
                (SourceIdentity(path, source.byte_length, source.sha256),),
            )
            with self.assertRaises(ManifestError, msg=path):
                encode(mutated)

    def test_source_bytes_are_checked_only_by_length_and_digest(self) -> None:
        manifest, source = fixture(FAMILY_CODEC)
        verify_source_bytes(manifest, (source,))
        altered = bytearray(source)
        altered[-1] ^= 1
        self.assertRaises(ManifestError, verify_source_bytes, manifest, (bytes(altered),))
        self.assertRaises(ManifestError, verify_source_bytes, manifest, (source[:-1],))
        self.assertRaises(ManifestError, verify_source_bytes, manifest, ())

    def test_unit_boundaries(self) -> None:
        manifest, _ = fixture(units=32_768)
        self.assertEqual(decode(encode(manifest)).units, 32_768)
        for units in (0, 32_769):
            invalid, _ = fixture(units=units)
            self.assertRaises(ManifestError, encode, invalid)


if __name__ == "__main__":
    unittest.main()
