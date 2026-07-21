#!/usr/bin/env python3
"""Independent framing and validation for the v0.9 lexical observer.

The observer is a non-authorizing development adapter.  One observation is the
request/response byte pair: the request binds the exact canonical source
bundle, while the response binds exact v0.9 and reports only the lexer's
existing outcome categories.  This module deliberately contains no capability
metadata, facet identity, parser rule, or ambient compiler-build discovery.
"""

from __future__ import annotations

from collections.abc import Sequence
from dataclasses import dataclass
from typing import Literal, Union

import v09_lexical_model as model


REQUEST_MAGIC = b"WFLEXREQ"
RESPONSE_MAGIC = b"WFLEXRSP"
SOURCE_BINDING_MAGIC = b"WFSOURCE"
PROTOCOL_VERSION = 1
SOURCE_BINDING_VERSION = 1
SPEC_HASH = bytes.fromhex(model.SPEC_SHA256)

TOKEN_KINDS = (
    "lower_word_form",
    "upper_word_form",
    "region_form",
    "label_form",
    "operation_name_form",
    "number_form",
    "string_form",
    "left_paren",
    "right_paren",
    "left_brace",
    "right_brace",
    "left_bracket",
    "right_bracket",
    "left_angle",
    "right_angle",
    "comma",
    "colon",
    "semicolon",
    "dot",
    "equal",
    "thin_arrow",
    "fat_arrow",
    "ampersand",
)
TRIVIA_KINDS = ("spaces", "line_feed")
PIECE_KINDS = TOKEN_KINDS + TRIVIA_KINDS
SOURCE_ISSUE_KINDS = (
    "invalid_utf8",
    "unexpected_byte",
    "missing_region_name",
    "missing_label_name",
    "unterminated_string",
    "invalid_string_byte",
    "invalid_string_escape",
)
RESOURCE_LIMIT_KINDS = (
    "sources",
    "source_bytes",
    "total_source_bytes",
    "token_bytes",
    "tokens",
    "lexemes",
)
STORAGE_KINDS = ("lexemes", "source_boundaries")


class ProtocolError(ValueError):
    """Bytes do not form the one supported observer protocol value."""


@dataclass(frozen=True)
class SourceLimits:
    """Explicit input and source-binding ceilings carried by one request."""

    max_sources: int
    max_logical_path_bytes: int
    max_source_bytes: int
    max_total_source_bytes: int
    max_binding_bytes: int

    def __post_init__(self) -> None:
        for name, value in vars(self).items():
            maximum = model.U32_MAX if name == "max_sources" else model.U64_MAX
            if type(value) is not int or not 0 <= value <= maximum:
                raise ValueError(
                    f"{name} must be an integer from 0 through {maximum}"
                )


@dataclass(frozen=True)
class BoundSource:
    """One exact logical path and raw byte string in binding order."""

    logical_path: str
    exact: bytes

    def __post_init__(self) -> None:
        _logical_path_bytes(self.logical_path)
        if type(self.exact) is not bytes:
            raise TypeError("source bytes must be exact immutable bytes")


@dataclass(frozen=True)
class ObserverRequest:
    """A validated request together with the context needed to check its reply."""

    wire_bytes: bytes
    binding: bytes
    sources: tuple[BoundSource, ...]
    source_limits: SourceLimits
    lex_limits: model.LexLimits

    def __post_init__(self) -> None:
        if type(self.wire_bytes) is not bytes or type(self.binding) is not bytes:
            raise TypeError("request transports must be exact immutable bytes")
        if type(self.source_limits) is not SourceLimits:
            raise TypeError("source limits must be SourceLimits")
        if type(self.lex_limits) is not model.LexLimits:
            raise TypeError("lexical limits must be LexLimits")
        sources = _require_sources(self.sources)
        if sources != self.sources:
            raise ValueError("request sources must be one immutable tuple")
        if decode_source_binding(self.binding, self.source_limits) != sources:
            raise ValueError("request binding does not encode its declared sources")
        if (
            encode_request(self.binding, self.source_limits, self.lex_limits)
            != self.wire_bytes
        ):
            raise ValueError("request bytes are not the canonical request transport")


@dataclass(frozen=True)
class ObservedPiece:
    """One exact source-local member of a complete lexical partition."""

    source_ordinal: int
    start: int
    end: int
    kind: str
    exact: bytes


@dataclass(frozen=True)
class CompleteObservation:
    """A lossless partition reported for every ordered source."""

    pieces: tuple[ObservedPiece, ...]
    source_ranges: tuple[tuple[int, int], ...]
    token_count: int
    outcome: Literal["complete"] = "complete"


@dataclass(frozen=True)
class SourceIssueObservation:
    """One source-local lexical issue."""

    source_ordinal: int
    start: int
    end: int
    kind: str
    exact: bytes
    outcome: Literal["source_issue"] = "source_issue"


@dataclass(frozen=True)
class LimitExceededObservation:
    """A caller-selected lexical ceiling was exceeded."""

    limit: str
    maximum: int
    actual: int
    outcome: Literal["resource_failure"] = "resource_failure"
    resource_kind: Literal["limit_exceeded"] = "limit_exceeded"


@dataclass(frozen=True)
class AddressSpaceExceededObservation:
    """An output count could not fit the host address space."""

    storage: str
    requested: int
    outcome: Literal["resource_failure"] = "resource_failure"
    resource_kind: Literal["address_space_exceeded"] = "address_space_exceeded"


@dataclass(frozen=True)
class StorageUnavailableObservation:
    """The allocator could not reserve already-counted lexical output."""

    storage: str
    requested: int
    outcome: Literal["resource_failure"] = "resource_failure"
    resource_kind: Literal["storage_unavailable"] = "storage_unavailable"


@dataclass(frozen=True)
class InvalidProducedSpanObservation:
    """The implementation reported a scanner-produced invalid range."""

    source_ordinal: int
    start: int
    end: int
    outcome: Literal["compiler_failure"] = "compiler_failure"
    compiler_kind: Literal["invalid_produced_span"] = "invalid_produced_span"


@dataclass(frozen=True)
class PassDisagreementObservation:
    """The immutable lexical passes disagreed on one source."""

    source_ordinal: int
    outcome: Literal["compiler_failure"] = "compiler_failure"
    compiler_kind: Literal["pass_disagreement"] = "pass_disagreement"


@dataclass(frozen=True)
class PassCountDisagreementObservation:
    """The immutable lexical passes reported different output counts."""

    expected_lexemes: int
    actual_lexemes: int
    expected_tokens: int
    actual_tokens: int
    outcome: Literal["compiler_failure"] = "compiler_failure"
    compiler_kind: Literal["pass_count_disagreement"] = "pass_count_disagreement"


@dataclass(frozen=True)
class CounterOverflowObservation:
    """A checked internal lexical counter overflowed."""

    outcome: Literal["compiler_failure"] = "compiler_failure"
    compiler_kind: Literal["counter_overflow"] = "counter_overflow"


ObservationOutcome = Union[
    CompleteObservation,
    SourceIssueObservation,
    LimitExceededObservation,
    AddressSpaceExceededObservation,
    StorageUnavailableObservation,
    InvalidProducedSpanObservation,
    PassDisagreementObservation,
    PassCountDisagreementObservation,
    CounterOverflowObservation,
]


@dataclass(frozen=True)
class DecodedResponse:
    """One strict response interpreted with its exact originating request."""

    binding: bytes
    sources: tuple[BoundSource, ...]
    outcome: ObservationOutcome


def prepare_request(
    sources: Sequence[BoundSource],
    source_limits: SourceLimits,
    lex_limits: model.LexLimits,
) -> ObserverRequest:
    """Build one request and retain the exact context required for its reply."""

    frozen_sources = _require_sources(sources)
    binding = encode_source_binding(frozen_sources, source_limits)
    wire_bytes = encode_request(binding, source_limits, lex_limits)
    return ObserverRequest(
        wire_bytes,
        binding,
        frozen_sources,
        source_limits,
        lex_limits,
    )


def encode_source_binding(
    sources: Sequence[BoundSource], limits: SourceLimits
) -> bytes:
    """Independently encode the canonical WFSOURCE v1 transport."""

    if type(limits) is not SourceLimits:
        raise TypeError("source limits must be SourceLimits")
    frozen_sources = _require_sources(sources)
    if len(frozen_sources) > limits.max_sources:
        raise ValueError("source count exceeds the configured source limit")

    encoded = bytearray(SOURCE_BINDING_MAGIC)
    encoded.extend(SOURCE_BINDING_VERSION.to_bytes(2, "big"))
    encoded.extend(SPEC_HASH)
    encoded.extend(len(frozen_sources).to_bytes(8, "big"))
    total_source_bytes = 0
    for source in frozen_sources:
        path = _logical_path_bytes(source.logical_path)
        if len(path) > limits.max_logical_path_bytes:
            raise ValueError("logical path exceeds the configured source limit")
        if len(source.exact) > limits.max_source_bytes:
            raise ValueError("source bytes exceed the configured source limit")
        total_source_bytes += len(source.exact)
        if total_source_bytes > limits.max_total_source_bytes:
            raise ValueError("total source bytes exceed the configured source limit")
        encoded.extend(len(path).to_bytes(8, "big"))
        encoded.extend(path)
        encoded.extend(len(source.exact).to_bytes(8, "big"))
        encoded.extend(source.exact)

    if len(encoded) > limits.max_binding_bytes:
        raise ValueError("source binding exceeds the configured source limit")
    return bytes(encoded)


def decode_source_binding(
    encoded: bytes, limits: SourceLimits
) -> tuple[BoundSource, ...]:
    """Decode one exact, canonical, v0.9-bound WFSOURCE value."""

    if type(limits) is not SourceLimits:
        raise TypeError("source limits must be SourceLimits")
    reader = _Reader(encoded)
    if len(encoded) > limits.max_binding_bytes:
        raise ProtocolError("source binding exceeds the configured source limit")
    if reader.take(len(SOURCE_BINDING_MAGIC)) != SOURCE_BINDING_MAGIC:
        raise ProtocolError("invalid source-binding magic")
    if reader.read_u16() != SOURCE_BINDING_VERSION:
        raise ProtocolError("unsupported source-binding version")
    if reader.take(len(SPEC_HASH)) != SPEC_HASH:
        raise ProtocolError("source binding does not name exact Whitefoot v0.9")

    source_count = reader.read_u64()
    if source_count > limits.max_sources or source_count > model.U32_MAX:
        raise ProtocolError("source count exceeds the configured source limit")
    if source_count > reader.remaining // 17:
        raise ProtocolError("source count cannot fit in the source binding")

    sources: list[BoundSource] = []
    logical_paths: set[str] = set()
    total_source_bytes = 0
    for _ in range(source_count):
        path_length = reader.read_u64()
        if path_length > limits.max_logical_path_bytes:
            raise ProtocolError("logical path exceeds the configured source limit")
        path_bytes = reader.take_u64(path_length)
        try:
            logical_path = path_bytes.decode("ascii", errors="strict")
            source = BoundSource(logical_path, b"")
        except (UnicodeDecodeError, TypeError, ValueError) as error:
            raise ProtocolError("source binding contains an invalid logical path") from error
        if logical_path in logical_paths:
            raise ProtocolError("source binding contains a duplicate logical path")
        logical_paths.add(logical_path)

        source_length = reader.read_u64()
        if source_length > limits.max_source_bytes:
            raise ProtocolError("source bytes exceed the configured source limit")
        total_source_bytes += source_length
        if total_source_bytes > limits.max_total_source_bytes:
            raise ProtocolError("total source bytes exceed the configured source limit")
        exact = reader.take_u64(source_length)
        sources.append(BoundSource(source.logical_path, exact))

    reader.finish()
    frozen_sources = tuple(sources)
    if encode_source_binding(frozen_sources, limits) != encoded:
        raise ProtocolError("source binding is not canonical")
    return frozen_sources


def encode_request(
    binding: bytes,
    source_limits: SourceLimits,
    lex_limits: model.LexLimits,
) -> bytes:
    """Encode one strict big-endian WFLEXREQ v1 request."""

    if type(binding) is not bytes:
        raise TypeError("source binding must be exact immutable bytes")
    if type(source_limits) is not SourceLimits:
        raise TypeError("source limits must be SourceLimits")
    if type(lex_limits) is not model.LexLimits:
        raise TypeError("lexical limits must be LexLimits")
    decode_source_binding(binding, source_limits)

    encoded = bytearray(REQUEST_MAGIC)
    encoded.extend(PROTOCOL_VERSION.to_bytes(2, "big"))
    encoded.extend(source_limits.max_sources.to_bytes(4, "big"))
    for value in (
        source_limits.max_logical_path_bytes,
        source_limits.max_source_bytes,
        source_limits.max_total_source_bytes,
        source_limits.max_binding_bytes,
    ):
        encoded.extend(value.to_bytes(8, "big"))
    encoded.extend(lex_limits.max_sources.to_bytes(4, "big"))
    for value in (
        lex_limits.max_source_bytes,
        lex_limits.max_total_source_bytes,
        lex_limits.max_token_bytes,
        lex_limits.max_tokens,
        lex_limits.max_lexemes,
    ):
        encoded.extend(value.to_bytes(8, "big"))
    encoded.extend(len(binding).to_bytes(8, "big"))
    encoded.extend(binding)
    return bytes(encoded)


def decode_response(encoded: bytes, request: ObserverRequest) -> DecodedResponse:
    """Validate one complete response against its exact originating request."""

    if type(request) is not ObserverRequest:
        raise TypeError("response context must be ObserverRequest")
    reader = _Reader(encoded)
    if reader.take(len(RESPONSE_MAGIC)) != RESPONSE_MAGIC:
        raise ProtocolError("invalid lexical-observer response magic")
    if reader.read_u16() != PROTOCOL_VERSION:
        raise ProtocolError("unsupported lexical-observer response version")
    if reader.take(len(SPEC_HASH)) != SPEC_HASH:
        raise ProtocolError("response does not name exact Whitefoot v0.9")

    outcome_tag = reader.read_u8()
    if outcome_tag == 0:
        outcome = _decode_complete(reader, request.sources)
    elif outcome_tag == 1:
        outcome = _decode_source_issue(reader, request.sources)
    elif outcome_tag == 2:
        outcome = _decode_resource_failure(reader, request.lex_limits)
    elif outcome_tag == 3:
        outcome = _decode_compiler_failure(reader, len(request.sources))
    else:
        raise ProtocolError("unknown lexical-observer outcome tag")
    reader.finish()
    return DecodedResponse(request.binding, request.sources, outcome)


def project_model_outcome(outcome: model.LexOutcome) -> ObservationOutcome:
    """Project the independent model into the observer's neutral result shape."""

    if isinstance(outcome, model.Complete):
        return CompleteObservation(
            tuple(
                ObservedPiece(
                    piece.source_ordinal,
                    piece.start,
                    piece.end,
                    piece.kind,
                    piece.exact,
                )
                for piece in outcome.pieces
            ),
            outcome.source_ranges,
            outcome.token_count,
        )
    if isinstance(outcome, model.SourceIssue):
        return SourceIssueObservation(
            outcome.source_ordinal,
            outcome.start,
            outcome.end,
            outcome.kind,
            outcome.exact,
        )
    if isinstance(outcome, model.ResourceFailure):
        return LimitExceededObservation(
            outcome.limit,
            outcome.maximum,
            outcome.actual,
        )
    raise TypeError(f"unknown lexical model outcome {type(outcome)!r}")


def _decode_complete(
    reader: _Reader, sources: tuple[BoundSource, ...]
) -> CompleteObservation:
    token_count = reader.read_u64()
    source_count = reader.read_u32()
    if source_count != len(sources):
        raise ProtocolError("complete response source count does not match binding")

    pieces: list[ObservedPiece] = []
    source_ranges: list[tuple[int, int]] = []
    observed_tokens = 0
    for source_ordinal, source in enumerate(sources):
        source_start = len(pieces)
        piece_count = reader.read_u64()
        if piece_count > len(source.exact) or piece_count > reader.remaining // 17:
            raise ProtocolError("complete response piece count is impossible")
        cursor = 0
        for _ in range(piece_count):
            kind_tag = reader.read_u8()
            if kind_tag >= len(PIECE_KINDS):
                raise ProtocolError("unknown lexical piece kind")
            start = reader.read_u64()
            end = reader.read_u64()
            if start != cursor or end <= start or end > len(source.exact):
                raise ProtocolError("lexical pieces are not one contiguous partition")
            kind = PIECE_KINDS[kind_tag]
            if kind_tag < len(TOKEN_KINDS):
                observed_tokens += 1
            pieces.append(
                ObservedPiece(
                    source_ordinal,
                    start,
                    end,
                    kind,
                    source.exact[start:end],
                )
            )
            cursor = end
        if cursor != len(source.exact):
            raise ProtocolError("lexical partition does not cover its complete source")
        source_ranges.append((source_start, len(pieces)))

    if token_count != observed_tokens:
        raise ProtocolError("complete response token count is inconsistent")
    return CompleteObservation(tuple(pieces), tuple(source_ranges), token_count)


def _decode_source_issue(
    reader: _Reader, sources: tuple[BoundSource, ...]
) -> SourceIssueObservation:
    source_ordinal = reader.read_u32()
    start = reader.read_u64()
    end = reader.read_u64()
    kind_tag = reader.read_u8()
    if source_ordinal >= len(sources):
        raise ProtocolError("source issue names a source outside the binding")
    source = sources[source_ordinal].exact
    if start >= end or end > len(source):
        raise ProtocolError("source issue span is not nonempty and source-local")
    if kind_tag >= len(SOURCE_ISSUE_KINDS):
        raise ProtocolError("unknown source issue kind")
    return SourceIssueObservation(
        source_ordinal,
        start,
        end,
        SOURCE_ISSUE_KINDS[kind_tag],
        source[start:end],
    )


def _decode_resource_failure(
    reader: _Reader, limits: model.LexLimits
) -> ObservationOutcome:
    subtype = reader.read_u8()
    if subtype == 0:
        limit_tag = reader.read_u8()
        if limit_tag >= len(RESOURCE_LIMIT_KINDS):
            raise ProtocolError("unknown lexical resource limit")
        maximum = reader.read_u64()
        actual = reader.read_u64()
        limit = RESOURCE_LIMIT_KINDS[limit_tag]
        expected_maximum = {
            "sources": limits.max_sources,
            "source_bytes": limits.max_source_bytes,
            "total_source_bytes": limits.max_total_source_bytes,
            "token_bytes": limits.max_token_bytes,
            "tokens": limits.max_tokens,
            "lexemes": limits.max_lexemes,
        }[limit]
        if maximum != expected_maximum:
            raise ProtocolError("resource failure does not echo its requested ceiling")
        if actual <= maximum:
            raise ProtocolError("resource failure does not identify an excess value")
        return LimitExceededObservation(limit, maximum, actual)
    if subtype in (1, 2):
        storage_tag = reader.read_u8()
        if storage_tag >= len(STORAGE_KINDS):
            raise ProtocolError("unknown lexical output storage")
        requested = reader.read_u64()
        if requested == 0:
            raise ProtocolError("lexical output storage request must be nonzero")
        storage = STORAGE_KINDS[storage_tag]
        if subtype == 1:
            return AddressSpaceExceededObservation(storage, requested)
        return StorageUnavailableObservation(storage, requested)
    raise ProtocolError("unknown lexical resource-failure subtype")


def _decode_compiler_failure(reader: _Reader, source_count: int) -> ObservationOutcome:
    subtype = reader.read_u8()
    if subtype == 0:
        source_ordinal = reader.read_u32()
        if source_ordinal >= source_count:
            raise ProtocolError("compiler failure names a source outside the binding")
        return InvalidProducedSpanObservation(
            source_ordinal,
            reader.read_u64(),
            reader.read_u64(),
        )
    if subtype == 1:
        source_ordinal = reader.read_u32()
        if source_ordinal >= source_count:
            raise ProtocolError("compiler failure names a source outside the binding")
        return PassDisagreementObservation(source_ordinal)
    if subtype == 2:
        expected_lexemes = reader.read_u64()
        actual_lexemes = reader.read_u64()
        expected_tokens = reader.read_u64()
        actual_tokens = reader.read_u64()
        if expected_tokens > expected_lexemes or actual_tokens > actual_lexemes:
            raise ProtocolError("lexical pass counts violate token/lexeme ordering")
        if (
            expected_lexemes == actual_lexemes
            and expected_tokens == actual_tokens
        ):
            raise ProtocolError("lexical pass-count failure reports equal counts")
        return PassCountDisagreementObservation(
            expected_lexemes,
            actual_lexemes,
            expected_tokens,
            actual_tokens,
        )
    if subtype == 3:
        return CounterOverflowObservation()
    raise ProtocolError("unknown lexical compiler-failure subtype")


def _require_sources(sources: Sequence[BoundSource]) -> tuple[BoundSource, ...]:
    if isinstance(sources, (str, bytes, bytearray)):
        raise TypeError("sources must be an ordered sequence of BoundSource values")
    frozen_sources = tuple(sources)
    if any(type(source) is not BoundSource for source in frozen_sources):
        raise TypeError("each source must be exactly BoundSource")
    logical_paths = [source.logical_path for source in frozen_sources]
    if len(logical_paths) != len(set(logical_paths)):
        raise ValueError("logical source paths must be unique")
    return frozen_sources


def _logical_path_bytes(logical_path: str) -> bytes:
    if type(logical_path) is not str:
        raise TypeError("logical path must be exact text")
    try:
        encoded = logical_path.encode("ascii", errors="strict")
    except UnicodeEncodeError as error:
        raise ValueError("logical path must use the portable ASCII set") from error
    if not encoded:
        raise ValueError("logical path must not be empty")
    if encoded.startswith(b"/"):
        raise ValueError("logical path must be relative")
    if any(
        not (
            ord("0") <= byte <= ord("9")
            or ord("A") <= byte <= ord("Z")
            or ord("a") <= byte <= ord("z")
            or byte in b"._-/"
        )
        for byte in encoded
    ):
        raise ValueError("logical path contains a non-portable byte")
    components = encoded.split(b"/")
    if any(not component for component in components):
        raise ValueError("logical path contains an empty component")
    if any(component in (b".", b"..") for component in components):
        raise ValueError("logical path contains a dot component")
    return encoded


class _Reader:
    def __init__(self, encoded: bytes) -> None:
        if type(encoded) is not bytes:
            raise TypeError("protocol input must be exact immutable bytes")
        self._encoded = encoded
        self._offset = 0

    @property
    def remaining(self) -> int:
        return len(self._encoded) - self._offset

    def take(self, length: int) -> bytes:
        end = self._offset + length
        if end > len(self._encoded):
            raise ProtocolError(f"protocol value is truncated at byte {len(self._encoded)}")
        value = self._encoded[self._offset : end]
        self._offset = end
        return value

    def take_u64(self, length: int) -> bytes:
        if length > self.remaining:
            raise ProtocolError(f"protocol value is truncated at byte {len(self._encoded)}")
        return self.take(length)

    def read_u8(self) -> int:
        return int.from_bytes(self.take(1), "big")

    def read_u16(self) -> int:
        return int.from_bytes(self.take(2), "big")

    def read_u32(self) -> int:
        return int.from_bytes(self.take(4), "big")

    def read_u64(self) -> int:
        return int.from_bytes(self.take(8), "big")

    def finish(self) -> None:
        if self.remaining != 0:
            raise ProtocolError("protocol value has trailing bytes")
