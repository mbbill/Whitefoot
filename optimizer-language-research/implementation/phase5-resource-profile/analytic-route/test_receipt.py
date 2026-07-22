from dataclasses import replace
from hashlib import sha256
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


sys.path.insert(0, str(Path(__file__).parent))

from manifest import FAMILY_CODEC, encode as encode_manifest  # noqa: E402
from receipt import (  # noqa: E402
    DOMAIN,
    STATUS,
    ReceiptError,
    build,
    decode,
    encode,
    identity,
)
from test_manifest import fixture  # noqa: E402


class ReceiptTests(unittest.TestCase):
    def test_receipt_round_trip_binds_manifest_bundle_sources_and_all_counts(self) -> None:
        manifest, source = fixture(FAMILY_CODEC, 1)
        manifest_bytes = encode_manifest(manifest)
        encoded = build(manifest_bytes, (source,))
        receipt = decode(encoded)
        self.assertEqual(receipt.manifest_digest, sha256(manifest_bytes).digest())
        self.assertEqual(receipt.source_digests, (sha256(source).digest(),))
        self.assertEqual(len(receipt.bundle_digest), 32)
        self.assertEqual(len(receipt.measurement.actuals), 33)
        self.assertEqual(encode(receipt), encoded)
        self.assertEqual(identity(encoded), sha256(encoded).digest())

    def test_identity_binding_tag_and_trailing_mutations_fail_closed(self) -> None:
        manifest, source = fixture()
        encoded = build(encode_manifest(manifest), (source,))
        proposal = bytearray(encoded)
        identity_start = len(DOMAIN) + 2 + 2 + len(STATUS)
        proposal[identity_start] ^= 1
        self.assertRaises(ReceiptError, decode, bytes(proposal))
        work = bytearray(encoded)
        work[identity_start + 3 * 32] ^= 1
        self.assertRaises(ReceiptError, decode, bytes(work))
        storage = bytearray(encoded)
        storage[identity_start + 4 * 32] ^= 1
        self.assertRaises(ReceiptError, decode, bytes(storage))
        code = bytearray(encoded)
        code[identity_start + 5 * 32] ^= 1
        self.assertRaises(ReceiptError, decode, bytes(code))

        first_tag = (
            len(DOMAIN)
            + 2
            + 2
            + len(STATUS)
            + 8 * 32
            + 4
            + len(encode_manifest(manifest))
            + 2
            + 32
            + 2
        )
        tag = bytearray(encoded)
        tag[first_tag + 1] = 2
        self.assertRaises(ReceiptError, decode, bytes(tag))
        self.assertRaises(ReceiptError, decode, encoded + b"\x00")

        diagnostic = bytearray(encoded)
        diagnostic[-1] ^= 1
        self.assertRaises(ReceiptError, decode, bytes(diagnostic))

        embedded_manifest = bytearray(encoded)
        manifest_start = len(DOMAIN) + 2 + 2 + len(STATUS) + 8 * 32 + 4
        embedded_manifest[manifest_start] ^= 1
        self.assertRaises(ReceiptError, decode, bytes(embedded_manifest))

    def test_derived_and_work_shape_cannot_be_reencoded_after_mutation(self) -> None:
        manifest, source = fixture()
        receipt = decode(build(encode_manifest(manifest), (source,)))
        missing = replace(
            receipt,
            measurement=replace(
                receipt.measurement, derived=receipt.measurement.derived[:-1]
            ),
        )
        self.assertRaises(ReceiptError, encode, missing)
        changed = replace(
            receipt,
            measurement=replace(
                receipt.measurement,
                trace_gaps=receipt.measurement.trace_gaps[:-1],
            ),
        )
        self.assertRaises(ReceiptError, encode, changed)
        actuals = list(receipt.measurement.actuals)
        actuals[18] += 1
        inconsistent_count = replace(
            receipt,
            measurement=replace(receipt.measurement, actuals=tuple(actuals)),
        )
        self.assertRaises(ReceiptError, encode, inconsistent_count)
        wrong_code = replace(receipt, code_digest=b"\x00" * 32)
        self.assertRaises(ReceiptError, encode, wrong_code)

    def test_different_source_identity_changes_bundle_and_receipt_identity(self) -> None:
        manifest, source = fixture()
        first = decode(build(encode_manifest(manifest), (source,)))
        changed_source = bytes(reversed(source))
        changed_identity = replace(
            manifest.sources[0], sha256=sha256(changed_source).hexdigest()
        )
        changed_manifest = replace(manifest, sources=(changed_identity,))
        second = decode(build(encode_manifest(changed_manifest), (changed_source,)))
        self.assertNotEqual(first.bundle_digest, second.bundle_digest)
        self.assertNotEqual(
            identity(encode(first)), identity(encode(second))
        )

    def test_actual_producer_bytes_cross_the_independent_manifest_boundary(self) -> None:
        generator = Path(__file__).parent.parent / "workloads.py"
        with tempfile.TemporaryDirectory() as directory:
            source_path = Path(directory) / "source.wf"
            manifest_path = Path(directory) / "manifest.json"
            subprocess.run(
                (
                    sys.executable,
                    str(generator),
                    "--family",
                    "compiler",
                    "--units",
                    "2",
                    "--output",
                    str(source_path),
                    "--manifest-output",
                    str(manifest_path),
                ),
                check=True,
                capture_output=True,
            )
            encoded = build(
                manifest_path.read_bytes(),
                (source_path.read_bytes(),),
            )
            receipt = decode(encoded)
            self.assertEqual(receipt.measurement.by_name()["max_tokens"], 230)
            self.assertIsNone(
                receipt.measurement.by_name()["max_resolution_work"]
            )


if __name__ == "__main__":
    unittest.main()
