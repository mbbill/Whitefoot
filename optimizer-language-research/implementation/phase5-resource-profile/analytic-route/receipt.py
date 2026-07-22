"""Canonical binary receipts for the independent analytic route."""

from dataclasses import dataclass
from hashlib import sha256
from pathlib import Path
import stat
import struct

from manifest import (
    ManifestError,
    WorkloadManifest,
    decode as decode_manifest,
    verify_source_bytes,
)
from measure import (
    FIELD_NAMES,
    Measurement,
    MeasurementError,
    measure,
    validate_shape,
)


DOMAIN = b"WHITEFOOT-ANALYTIC-RECEIPT-V1\0"
VERSION = 1
PROPOSAL_SHA256 = bytes.fromhex(
    "7fc48cc30f94d25be5be1106e3265d92c1b0cdf2bfea5a7a17759a12f3cf092d"
)
SPECIFICATION_SHA256 = bytes.fromhex(
    "71073e25219455896250e15e13d1ffdbfc443c87a9b28cb9906d73a020dc33e9"
)
PROFILE_SEMANTICS_SHA256 = bytes.fromhex(
    "981878811e38716acfd5dc0bbacccf278c68b2db29aa987af98937e65649d754"
)
WORK_SHA256 = bytes.fromhex(
    "2d085436e8d9288a982ef83a13554c2310cead38892e8223d7f2661b60b3c7e7"
)
STORAGE_SHA256 = bytes.fromhex(
    "6d624da13ddd48d6dd46f3a2feaac38b83b51e4154e0e70e08a73524e9e7505a"
)
STATUS = "trace-incomplete"
CODE_DOMAIN = b"WHITEFOOT-ANALYTIC-CODE-V1\0"
CODE_FILES = (
    "dependency_audit.py",
    "manifest.py",
    "measure.py",
    "receipt.py",
    "relation.py",
    "run.py",
    "selection.py",
)
MAX_CODE_FILE_BYTES = 1 << 20
BUNDLE_DOMAIN = b"WHITEFOOT-ANALYTIC-SOURCE-BUNDLE-V1\0"
U16 = struct.Struct(">H")
U32 = struct.Struct(">I")
U64 = struct.Struct(">Q")
FIELD = struct.Struct(">HBQ")
DIGEST_BYTES = 32


class ReceiptError(ValueError):
    """Receipt bytes are malformed, noncanonical, or identity-inconsistent."""


@dataclass(frozen=True)
class AnalyticReceipt:
    status: str
    code_digest: bytes
    manifest_bytes: bytes
    manifest_digest: bytes
    bundle_digest: bytes
    source_digests: tuple[bytes, ...]
    measurement: Measurement


def code_identity() -> bytes:
    """Hash the exact ordered closed runtime code set with file identities."""
    root = Path(__file__).parent
    output = bytearray(CODE_DOMAIN)
    output.extend(U16.pack(len(CODE_FILES)))
    for name in CODE_FILES:
        path = root / name
        metadata = path.lstat()
        if stat.S_ISLNK(metadata.st_mode) or not stat.S_ISREG(metadata.st_mode):
            raise ReceiptError(f"analytic code identity file is not regular: {name}")
        if metadata.st_size < 0 or metadata.st_size > MAX_CODE_FILE_BYTES:
            raise ReceiptError(f"analytic code identity file is outside its cap: {name}")
        value = path.read_bytes()
        after = path.stat()
        if (
            len(value) != metadata.st_size
            or (after.st_dev, after.st_ino, after.st_size)
            != (metadata.st_dev, metadata.st_ino, metadata.st_size)
        ):
            raise ReceiptError(f"analytic code identity file changed while read: {name}")
        encoded_name = name.encode("ascii")
        output.extend(U16.pack(len(encoded_name)))
        output.extend(encoded_name)
        output.extend(U64.pack(len(value)))
        output.extend(sha256(value).digest())
    return sha256(output).digest()


def _validate_receipt(receipt: AnalyticReceipt) -> None:
    if receipt.status != STATUS:
        raise ReceiptError("receipt status is not trace-incomplete")
    if receipt.code_digest != code_identity():
        raise ReceiptError("receipt analytic code identity is wrong")
    try:
        manifest = decode_manifest(receipt.manifest_bytes)
    except ManifestError as error:
        raise ReceiptError("embedded manifest is not canonical") from error
    if sha256(receipt.manifest_bytes).digest() != receipt.manifest_digest:
        raise ReceiptError("embedded manifest digest is inconsistent")
    expected_source_digests = tuple(
        bytes.fromhex(source.sha256) for source in manifest.sources
    )
    if receipt.source_digests != expected_source_digests:
        raise ReceiptError("source identities disagree with embedded manifest")
    if receipt.bundle_digest != _bundle_identity(manifest, expected_source_digests):
        raise ReceiptError("bundle identity disagrees with embedded manifest")
    expected_measurement = measure(manifest)
    if receipt.measurement != expected_measurement:
        raise ReceiptError("measurement disagrees with embedded manifest relation")
    try:
        validate_shape(receipt.measurement)
    except MeasurementError as error:
        raise ReceiptError(str(error)) from error
    actual = receipt.measurement.by_name()
    derived = dict(receipt.measurement.derived)

    def available(name: str) -> int:
        value = actual[name]
        if value is None:
            raise ReceiptError(f"receipt relation unexpectedly needs unavailable {name}")
        return value

    sources = available("max_sources")
    tokens = available("max_tokens")
    nodes = available("max_production_nodes")
    events = available("max_declaration_events")
    lexical = available("max_lexical_uses")
    deferred = available("max_deferred_uses")
    if len(receipt.source_digests) != sources or sources == 0:
        raise ReceiptError("source identities disagree with max_sources")
    equations = (
        (available("max_classified_tokens"), tokens, "classified tokens"),
        (derived["terminals"], tokens, "terminals"),
        (derived["private_derivation_elements"], nodes + tokens, "derivation elements"),
        (available("max_mixed_elements"), nodes - 1 + tokens, "mixed elements"),
        (derived["gaps"], tokens, "gaps"),
        (derived["source_extents"], sources, "source extents"),
        (available("max_declarations"), events + 24, "declarations"),
        (available("max_lookup_entries"), 18 + 83 + derived["source_lookup_entries"], "lookup entries"),
        (available("max_ancestry_steps"), available("max_scopes") - 1, "ancestry steps"),
        (available("max_coverage_records"), nodes + events + lexical + deferred, "coverage records"),
    )
    for observed, expected, label in equations:
        if observed != expected:
            raise ReceiptError(f"receipt {label} equation is inconsistent")
    tree_sum = sum(
        derived[name]
        for name in (
            "derivation_tree_bytes", "node_tree_bytes", "mixed_tree_bytes",
            "terminal_tree_bytes", "source_extent_tree_bytes",
        )
    )
    if available("max_tree_bytes") != tree_sum:
        raise ReceiptError("receipt tree-byte equation is inconsistent")
    if receipt.measurement.expected_diagnostic == ("Complete",) and (any(
        available(name) != 0
        for name in (
            "max_diagnostic_origins", "max_diagnostic_paths",
            "max_diagnostic_path_components",
        )
    ) or derived["diagnostic_issue_elements"] != 0):
        raise ReceiptError("Complete receipt contains diagnostic payload counts")


def _text(output: bytearray, value: str) -> None:
    encoded = value.encode("ascii")
    if not encoded or len(encoded) > 0xFFFF:
        raise ReceiptError("receipt text is outside the closed ASCII bound")
    output.extend(U16.pack(len(encoded)))
    output.extend(encoded)


def _bundle_identity(
    manifest: WorkloadManifest, source_digests: tuple[bytes, ...]
) -> bytes:
    output = bytearray(BUNDLE_DOMAIN)
    output.extend(U32.pack(len(manifest.sources)))
    for source, digest in zip(manifest.sources, source_digests):
        path = source.logical_path.encode("ascii")
        output.extend(U16.pack(len(path)))
        output.extend(path)
        output.extend(U64.pack(source.byte_length))
        output.extend(digest)
    return sha256(output).digest()


def build(manifest_bytes: bytes, source_bytes: tuple[bytes, ...]) -> bytes:
    manifest = decode_manifest(manifest_bytes)
    source_digests = verify_source_bytes(manifest, source_bytes)
    receipt = AnalyticReceipt(
        status=STATUS,
        code_digest=code_identity(),
        manifest_bytes=manifest_bytes,
        manifest_digest=sha256(manifest_bytes).digest(),
        bundle_digest=_bundle_identity(manifest, source_digests),
        source_digests=source_digests,
        measurement=measure(manifest),
    )
    return encode(receipt)


def encode(receipt: AnalyticReceipt) -> bytes:
    for label, digest in (
        ("manifest", receipt.manifest_digest),
        ("bundle", receipt.bundle_digest),
    ):
        if len(digest) != DIGEST_BYTES:
            raise ReceiptError(f"{label} digest is not SHA-256")
    _validate_receipt(receipt)
    output = bytearray(DOMAIN)
    output.extend(U16.pack(VERSION))
    _text(output, receipt.status)
    output.extend(PROPOSAL_SHA256)
    output.extend(SPECIFICATION_SHA256)
    output.extend(PROFILE_SEMANTICS_SHA256)
    output.extend(WORK_SHA256)
    output.extend(STORAGE_SHA256)
    output.extend(receipt.code_digest)
    output.extend(receipt.manifest_digest)
    output.extend(receipt.bundle_digest)
    if len(receipt.manifest_bytes) > (1 << 32) - 1:
        raise ReceiptError("embedded manifest exceeds u32 receipt length")
    output.extend(U32.pack(len(receipt.manifest_bytes)))
    output.extend(receipt.manifest_bytes)
    output.extend(U16.pack(len(receipt.source_digests)))
    for digest in receipt.source_digests:
        if len(digest) != DIGEST_BYTES:
            raise ReceiptError("source identity is not SHA-256")
        output.extend(digest)
    output.extend(U16.pack(len(FIELD_NAMES)))
    for tag, value in enumerate(receipt.measurement.actuals, 1):
        output.extend(FIELD.pack(tag, value is not None, 0 if value is None else value))
    output.extend(U16.pack(len(receipt.measurement.derived)))
    for name, value in receipt.measurement.derived:
        _text(output, name)
        output.extend(U64.pack(value))
    output.extend(U16.pack(len(receipt.measurement.trace_gaps)))
    for name, reason in receipt.measurement.trace_gaps:
        _text(output, name)
        _text(output, reason)
    output.extend(U16.pack(len(receipt.measurement.expected_diagnostic)))
    for component in receipt.measurement.expected_diagnostic:
        _text(output, component)
    return bytes(output)


class _Reader:
    def __init__(self, value: bytes) -> None:
        self.value = value
        self.cursor = 0

    def take(self, amount: int) -> bytes:
        end = self.cursor + amount
        if amount < 0 or end > len(self.value):
            raise ReceiptError("receipt is truncated")
        result = self.value[self.cursor:end]
        self.cursor = end
        return result

    def u16(self) -> int:
        return U16.unpack(self.take(U16.size))[0]

    def u64(self) -> int:
        return U64.unpack(self.take(U64.size))[0]

    def u32(self) -> int:
        return U32.unpack(self.take(U32.size))[0]

    def text(self) -> str:
        raw = self.take(self.u16())
        try:
            value = raw.decode("ascii")
        except UnicodeDecodeError as error:
            raise ReceiptError("receipt text is not ASCII") from error
        if not value:
            raise ReceiptError("receipt text is empty")
        return value


def decode(encoded: bytes) -> AnalyticReceipt:
    reader = _Reader(encoded)
    if reader.take(len(DOMAIN)) != DOMAIN or reader.u16() != VERSION:
        raise ReceiptError("receipt domain or version is wrong")
    status = reader.text()
    if status != STATUS:
        raise ReceiptError("receipt status is not trace-incomplete")
    for expected, label in (
        (PROPOSAL_SHA256, "proposal"),
        (SPECIFICATION_SHA256, "specification"),
        (PROFILE_SEMANTICS_SHA256, "profile semantics"),
        (WORK_SHA256, "work schedule"),
        (STORAGE_SHA256, "storage model"),
    ):
        if reader.take(DIGEST_BYTES) != expected:
            raise ReceiptError(f"receipt {label} binding is wrong")
    code_digest = reader.take(DIGEST_BYTES)
    manifest_digest = reader.take(DIGEST_BYTES)
    bundle_digest = reader.take(DIGEST_BYTES)
    manifest_bytes = reader.take(reader.u32())
    source_digests = tuple(reader.take(DIGEST_BYTES) for _ in range(reader.u16()))
    if reader.u16() != len(FIELD_NAMES):
        raise ReceiptError("receipt field count is wrong")
    actuals = []
    for expected_tag in range(1, len(FIELD_NAMES) + 1):
        tag, available, value = FIELD.unpack(reader.take(FIELD.size))
        if tag != expected_tag:
            raise ReceiptError("receipt field tag or order is wrong")
        if available not in (0, 1) or (available == 0 and value != 0):
            raise ReceiptError("receipt field availability encoding is wrong")
        actuals.append(value if available else None)
    derived = tuple((reader.text(), reader.u64()) for _ in range(reader.u16()))
    trace_gaps = tuple((reader.text(), reader.text()) for _ in range(reader.u16()))
    diagnostic = tuple(reader.text() for _ in range(reader.u16()))
    if reader.cursor != len(encoded):
        raise ReceiptError("receipt has trailing bytes")
    receipt = AnalyticReceipt(
        status,
        code_digest,
        manifest_bytes,
        manifest_digest,
        bundle_digest,
        source_digests,
        Measurement(tuple(actuals), derived, trace_gaps, diagnostic),
    )
    _validate_receipt(receipt)
    if encode(receipt) != encoded:
        raise ReceiptError("receipt representation is not canonical")
    return receipt


def identity(encoded: bytes) -> bytes:
    decode(encoded)
    return sha256(encoded).digest()
