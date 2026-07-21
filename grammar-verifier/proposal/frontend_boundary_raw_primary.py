"""Primary byte-state model for the proposal's raw lexical boundary."""

from __future__ import annotations

from typing import Sequence


class PrimaryRawError(RuntimeError):
    pass


_PUNCTUATION = b"(){}[]<>,:;.= &"
_OPERATION_SUFFIXES = (b"wrap", b"trap", b"checked", b"sat", b"strict")


def _ascii_letter(byte: int) -> bool:
    return 65 <= byte <= 90 or 97 <= byte <= 122


def _lower(byte: int) -> bool:
    return 97 <= byte <= 122


def _digit(byte: int) -> bool:
    return 48 <= byte <= 57


def _name_tail(byte: int) -> bool:
    return _ascii_letter(byte) or _digit(byte) or byte == 95


def _upper_tail(byte: int) -> bool:
    return _ascii_letter(byte) or _digit(byte)


def _lower_tail(byte: int) -> bool:
    return _lower(byte) or _digit(byte) or byte == 95


def _utf8_scalar(source: bytes, start: int) -> tuple[int, int] | None:
    first = source[start]
    if first < 0x80:
        return first, 1
    if 0xC2 <= first <= 0xDF:
        width, value, minimum = 2, first & 0x1F, 0x80
    elif 0xE0 <= first <= 0xEF:
        width, value, minimum = 3, first & 0x0F, 0x800
    elif 0xF0 <= first <= 0xF4:
        width, value, minimum = 4, first & 0x07, 0x10000
    else:
        return None
    if start + width > len(source):
        return None
    for follower in source[start + 1 : start + width]:
        if follower & 0xC0 != 0x80:
            return None
        value = (value << 6) | (follower & 0x3F)
    if value < minimum or 0xD800 <= value <= 0xDFFF or value > 0x10FFFF:
        return None
    return value, width


def _coordinate(source_ordinal: int, start: int, end: int) -> list[int]:
    return [source_ordinal, start, end]


def _defect(rule: str, source_ordinal: int, start: int, end: int) -> dict[str, object]:
    return {
        "family": "source-language-rejection",
        "location": {
            "coordinate": _coordinate(source_ordinal, start, end),
            "kind": "SourceBytes",
        },
        "rule": rule,
    }


def _token(kind: str, source_ordinal: int, source: bytes, start: int, end: int) -> dict[str, object]:
    return {
        "kind": kind,
        "source_coordinate": _coordinate(source_ordinal, start, end),
        "spelling_hex": source[start:end].hex(),
    }


def _trivia(source_ordinal: int, source: bytes, start: int, end: int) -> dict[str, object]:
    return {
        "source_coordinate": _coordinate(source_ordinal, start, end),
        "spelling_hex": source[start:end].hex(),
    }


def _string(
    source_ordinal: int, source: bytes, start: int
) -> tuple[int, dict[str, object]]:
    cursor = start + 1
    while cursor < len(source):
        scalar = _utf8_scalar(source, cursor)
        if scalar is None:
            return len(source), _defect("FORM-2", source_ordinal, cursor, cursor + 1)
        value, width = scalar
        if value == 34:
            end = cursor + 1
            return end, _token("string", source_ordinal, source, start, end)
        if value == 92:
            if cursor + 1 == len(source):
                return len(source), _defect("FORM-5", source_ordinal, cursor, cursor + 1)
            follower = _utf8_scalar(source, cursor + 1)
            if follower is None:
                return len(source), _defect(
                    "FORM-2", source_ordinal, cursor + 1, cursor + 2
                )
            next_value, next_width = follower
            if next_value >= 0x80:
                return len(source), _defect(
                    "FORM-5", source_ordinal, cursor, cursor + 1 + next_width
                )
            if next_value not in (34, 92, 110):
                return len(source), _defect(
                    "FORM-5", source_ordinal, cursor, cursor + 2
                )
            cursor += 2
            continue
        if value >= 0x80:
            return len(source), _defect(
                "FORM-5", source_ordinal, cursor, cursor + width
            )
        if value < 0x20 or value == 0x7F:
            return len(source), _defect(
                "FORM-5", source_ordinal, cursor, cursor + 1
            )
        cursor += 1
    return len(source), _defect("FORM-5", source_ordinal, start, len(source))


def _scan_source(source_ordinal: int, source: bytes) -> dict[str, object]:
    tokens: list[dict[str, object]] = []
    trivia: list[dict[str, object]] = []
    cursor = 0
    while cursor < len(source):
        scalar = _utf8_scalar(source, cursor)
        if scalar is None:
            return _defect("FORM-2", source_ordinal, cursor, cursor + 1)
        value, width = scalar
        if value >= 0x80:
            return _defect("FORM-1", source_ordinal, cursor, cursor + width)
        byte = value
        if byte == 0x20:
            start = cursor
            cursor += 1
            while cursor < len(source) and source[cursor] == 0x20:
                cursor += 1
            trivia.append(_trivia(source_ordinal, source, start, cursor))
            continue
        if byte == 0x0A:
            trivia.append(_trivia(source_ordinal, source, cursor, cursor + 1))
            cursor += 1
            continue
        if byte < 0x20 or byte == 0x7F:
            return _defect("FORM-2", source_ordinal, cursor, cursor + 1)
        if source.startswith((b"//", b"/*"), cursor):
            return _defect("FORM-4", source_ordinal, cursor, cursor + 2)
        if byte == 34:
            cursor, result = _string(source_ordinal, source, cursor)
            if result.get("family") == "source-language-rejection":
                return result
            tokens.append(result)
            continue
        if byte in (39, 64):
            if cursor + 1 >= len(source) or not _lower(source[cursor + 1]):
                return _defect("FORM-3", source_ordinal, cursor, cursor + 1)
            end = cursor + 2
            while end < len(source) and _lower_tail(source[end]):
                end += 1
            tokens.append(
                _token("region" if byte == 39 else "label", source_ordinal, source, cursor, end)
            )
            cursor = end
            continue
        if _lower(byte):
            base_end = cursor + 1
            while base_end < len(source) and _lower_tail(source[base_end]):
                base_end += 1
            operation_end = None
            if base_end < len(source) and source[base_end] == 46:
                suffix_start = base_end + 1
                for suffix in _OPERATION_SUFFIXES:
                    suffix_end = suffix_start + len(suffix)
                    if source[suffix_start:suffix_end] != suffix:
                        continue
                    if suffix_end < len(source) and _name_tail(source[suffix_end]):
                        continue
                    operation_end = suffix_end
                    break
            if operation_end is not None:
                tokens.append(
                    _token("opname", source_ordinal, source, cursor, operation_end)
                )
                cursor = operation_end
            else:
                tokens.append(_token("lower-word", source_ordinal, source, cursor, base_end))
                cursor = base_end
            continue
        if 65 <= byte <= 90:
            end = cursor + 1
            while end < len(source) and _upper_tail(source[end]):
                end += 1
            tokens.append(_token("upper-word", source_ordinal, source, cursor, end))
            cursor = end
            continue
        if source.startswith(b"->", cursor) or source.startswith(b"=>", cursor):
            tokens.append(_token("punctuation", source_ordinal, source, cursor, cursor + 2))
            cursor += 2
            continue
        if _digit(byte) or (byte == 45 and cursor + 1 < len(source) and _digit(source[cursor + 1])):
            end = cursor + 1
            while end < len(source):
                current = source[end]
                if _name_tail(current) or current == 46:
                    end += 1
                    continue
                if current in (43, 45) and source[end - 1] in (69, 101):
                    end += 1
                    continue
                break
            tokens.append(_token("numeric", source_ordinal, source, cursor, end))
            cursor = end
            continue
        if byte in _PUNCTUATION and byte != 0x20:
            tokens.append(_token("punctuation", source_ordinal, source, cursor, cursor + 1))
            cursor += 1
            continue
        return _defect("FORM-1", source_ordinal, cursor, cursor + 1)
    return {
        "family": "raw-source-complete",
        "source_ordinal": source_ordinal,
        "tokens": tokens,
        "trivia": trivia,
    }


def scan_sources(sources: Sequence[bytes]) -> dict[str, object]:
    completed: list[dict[str, object]] = []
    for source_ordinal, source in enumerate(sources):
        result = _scan_source(source_ordinal, source)
        if result.get("family") == "source-language-rejection":
            return result
        completed.append(result)
    return {"family": "raw-scan-complete", "sources": completed}


def _authored_sources(values: object) -> tuple[bytes, ...]:
    if not isinstance(values, list) or not values:
        raise PrimaryRawError("raw case has no source records")
    result: list[bytes] = []
    for value in values:
        if not isinstance(value, str) or value != value.lower() or len(value) % 2:
            raise PrimaryRawError("raw case source is not canonical hexadecimal")
        try:
            source = bytes.fromhex(value)
        except ValueError as error:
            raise PrimaryRawError("raw case source is malformed hexadecimal") from error
        if source.hex() != value:
            raise PrimaryRawError("raw case source spelling is not canonical")
        result.append(source)
    return tuple(result)


def _authored_span(source: bytes, value: object) -> tuple[int, int]:
    if (
        not isinstance(value, list)
        or len(value) != 2
        or any(type(part) is not int for part in value)
    ):
        raise PrimaryRawError("raw expected span is not two integers")
    start, end = value
    if start < 0 or start > end or end > len(source):
        raise PrimaryRawError("raw expected span escapes its source")
    return start, end


def _expand_authored(expectation: object, sources: tuple[bytes, ...]) -> dict[str, object]:
    if not isinstance(expectation, dict) or "family" not in expectation:
        raise PrimaryRawError("raw expectation is malformed")
    if expectation["family"] == "rejection":
        if set(expectation) != {"coordinate", "family", "rule"}:
            raise PrimaryRawError("raw rejection expectation fields changed")
        coordinate = expectation["coordinate"]
        if (
            not isinstance(coordinate, list)
            or len(coordinate) != 3
            or type(coordinate[0]) is not int
            or coordinate[0] < 0
            or coordinate[0] >= len(sources)
        ):
            raise PrimaryRawError("raw rejection coordinate has an invalid source")
        start, end = _authored_span(sources[coordinate[0]], coordinate[1:])
        return _defect(expectation["rule"], coordinate[0], start, end)
    if expectation["family"] != "complete" or set(expectation) != {"family", "sources"}:
        raise PrimaryRawError("raw completion expectation fields changed")
    rows = expectation["sources"]
    if not isinstance(rows, list) or len(rows) != len(sources):
        raise PrimaryRawError("raw completion source inventory changed")
    completed: list[dict[str, object]] = []
    for ordinal, (source, row) in enumerate(zip(sources, rows)):
        if not isinstance(row, dict) or set(row) != {"tokens", "trivia"}:
            raise PrimaryRawError("raw completion source fields changed")
        tokens: list[dict[str, object]] = []
        trivia: list[dict[str, object]] = []
        for token in row["tokens"]:
            if not isinstance(token, list) or len(token) != 3 or not isinstance(token[0], str):
                raise PrimaryRawError("raw expected token row is malformed")
            start, end = _authored_span(source, token[1:])
            tokens.append(_token(token[0], ordinal, source, start, end))
        for gap in row["trivia"]:
            start, end = _authored_span(source, gap)
            trivia.append(_trivia(ordinal, source, start, end))
        completed.append(
            {
                "family": "raw-source-complete",
                "source_ordinal": ordinal,
                "tokens": tokens,
                "trivia": trivia,
            }
        )
    return {"family": "raw-scan-complete", "sources": completed}


def evaluate_authored_cases(rows: object) -> list[dict[str, object]]:
    if not isinstance(rows, list) or not rows:
        raise PrimaryRawError("raw case collection is empty or malformed")
    names: set[str] = set()
    results: list[dict[str, object]] = []
    for row in rows:
        if not isinstance(row, dict) or set(row) != {"expect", "id", "sources_hex"}:
            raise PrimaryRawError("raw case fields changed")
        identifier = row["id"]
        if not isinstance(identifier, str) or not identifier or identifier in names:
            raise PrimaryRawError("raw case identifier is absent or duplicated")
        names.add(identifier)
        sources = _authored_sources(row["sources_hex"])
        expected = _expand_authored(row["expect"], sources)
        actual = scan_sources(sources)
        if actual != expected:
            raise PrimaryRawError(f"raw case {identifier} disagrees with its expectation")
        results.append({"id": identifier, "outcome": actual})
    return results
