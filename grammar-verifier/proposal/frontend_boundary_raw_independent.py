"""Independent raw lexical byte scanner for frontend-boundary evidence."""

from __future__ import annotations

import re
from typing import Sequence


class IndependentRawError(RuntimeError):
    pass


LOWER_WORD = re.compile(rb"[a-z][a-z0-9_]*")
UPPER_WORD = re.compile(rb"[A-Z][A-Za-z0-9]*")
REGION_BODY = re.compile(rb"[a-z][a-z0-9_]*")
NUMBER_START = re.compile(rb"-?[0-9]")
SINGLE_PUNCTUATION = frozenset(b"(){}[]<>,:;.= &") - {0x20}
SUFFIXES = (b"wrap", b"trap", b"checked", b"sat", b"strict")


def _scalar(source: bytes, cursor: int) -> tuple[str, int] | None:
    remaining = len(source) - cursor
    for width in range(1, min(4, remaining) + 1):
        candidate = source[cursor : cursor + width]
        try:
            decoded = candidate.decode("utf-8", "strict")
        except UnicodeDecodeError:
            continue
        if len(decoded) == 1 and decoded.encode("utf-8") == candidate:
            return decoded, width
    return None


def _where(source_number: int, begin: int, finish: int) -> list[int]:
    return [source_number, begin, finish]


def _rejected(rule: str, source_number: int, begin: int, finish: int) -> dict[str, object]:
    return {
        "family": "source-language-rejection",
        "location": {
            "coordinate": _where(source_number, begin, finish),
            "kind": "SourceBytes",
        },
        "rule": rule,
    }


def _formed(kind: str, source_number: int, source: bytes, begin: int, finish: int) -> dict[str, object]:
    return {
        "kind": kind,
        "source_coordinate": _where(source_number, begin, finish),
        "spelling_hex": source[begin:finish].hex(),
    }


def _space(source_number: int, source: bytes, begin: int, finish: int) -> dict[str, object]:
    return {
        "source_coordinate": _where(source_number, begin, finish),
        "spelling_hex": source[begin:finish].hex(),
    }


def _quoted(source_number: int, source: bytes, opening: int) -> tuple[int, dict[str, object]]:
    position = opening + 1
    while position < len(source):
        decoded = _scalar(source, position)
        if decoded is None:
            return len(source), _rejected("FORM-2", source_number, position, position + 1)
        character, width = decoded
        codepoint = ord(character)
        if character == '"':
            end = position + 1
            return end, _formed("string", source_number, source, opening, end)
        if character == "\\":
            follower_position = position + 1
            if follower_position == len(source):
                return len(source), _rejected(
                    "FORM-5", source_number, position, position + 1
                )
            follower = _scalar(source, follower_position)
            if follower is None:
                return len(source), _rejected(
                    "FORM-2", source_number, follower_position, follower_position + 1
                )
            follower_character, follower_width = follower
            if ord(follower_character) > 0x7F:
                return len(source), _rejected(
                    "FORM-5",
                    source_number,
                    position,
                    follower_position + follower_width,
                )
            if follower_character not in ('"', "\\", "n"):
                return len(source), _rejected(
                    "FORM-5", source_number, position, follower_position + 1
                )
            position += 2
            continue
        if codepoint > 0x7F:
            return len(source), _rejected(
                "FORM-5", source_number, position, position + width
            )
        if codepoint < 0x20 or codepoint == 0x7F:
            return len(source), _rejected(
                "FORM-5", source_number, position, position + 1
            )
        position += 1
    return len(source), _rejected("FORM-5", source_number, opening, len(source))


def _number_end(source: bytes, begin: int) -> int:
    position = begin + 1
    while position < len(source):
        byte = source[position]
        if (
            48 <= byte <= 57
            or 65 <= byte <= 90
            or 97 <= byte <= 122
            or byte in (46, 95)
        ):
            position += 1
            continue
        if byte in (43, 45) and source[position - 1] in (69, 101):
            position += 1
            continue
        return position
    return position


def _one(source_number: int, source: bytes) -> dict[str, object]:
    tokens: list[dict[str, object]] = []
    trivia: list[dict[str, object]] = []
    position = 0
    while position != len(source):
        decoded = _scalar(source, position)
        if decoded is None:
            return _rejected("FORM-2", source_number, position, position + 1)
        character, width = decoded
        value = ord(character)
        if value > 0x7F:
            return _rejected("FORM-1", source_number, position, position + width)
        if value == 0x20:
            end = position + 1
            while end < len(source) and source[end] == 0x20:
                end += 1
            trivia.append(_space(source_number, source, position, end))
            position = end
            continue
        if value == 0x0A:
            trivia.append(_space(source_number, source, position, position + 1))
            position += 1
            continue
        if value < 0x20 or value == 0x7F:
            return _rejected("FORM-2", source_number, position, position + 1)
        if source[position : position + 2] in (b"//", b"/*"):
            return _rejected("FORM-4", source_number, position, position + 2)
        if character == '"':
            position, result = _quoted(source_number, source, position)
            if result.get("family") == "source-language-rejection":
                return result
            tokens.append(result)
            continue
        if character in ("'", "@"):
            body = REGION_BODY.match(source, position + 1)
            if body is None:
                return _rejected("FORM-3", source_number, position, position + 1)
            end = body.end()
            tokens.append(
                _formed(
                    "region" if character == "'" else "label",
                    source_number,
                    source,
                    position,
                    end,
                )
            )
            position = end
            continue
        lower = LOWER_WORD.match(source, position)
        if lower is not None:
            base_end = lower.end()
            chosen = base_end
            kind = "lower-word"
            if base_end < len(source) and source[base_end] == 0x2E:
                suffix_start = base_end + 1
                matches = [
                    suffix_start + len(suffix)
                    for suffix in SUFFIXES
                    if source.startswith(suffix, suffix_start)
                    and (
                        suffix_start + len(suffix) == len(source)
                        or not (
                            source[suffix_start + len(suffix)] == 95
                            or 48 <= source[suffix_start + len(suffix)] <= 57
                            or 65 <= source[suffix_start + len(suffix)] <= 90
                            or 97 <= source[suffix_start + len(suffix)] <= 122
                        )
                    )
                ]
                if len(matches) > 1:
                    raise IndependentRawError("operation suffix table is ambiguous")
                if matches:
                    chosen = matches[0]
                    kind = "opname"
            tokens.append(_formed(kind, source_number, source, position, chosen))
            position = chosen
            continue
        upper = UPPER_WORD.match(source, position)
        if upper is not None:
            tokens.append(
                _formed("upper-word", source_number, source, position, upper.end())
            )
            position = upper.end()
            continue
        if source[position : position + 2] in (b"->", b"=>"):
            tokens.append(
                _formed("punctuation", source_number, source, position, position + 2)
            )
            position += 2
            continue
        if NUMBER_START.match(source, position) is not None:
            end = _number_end(source, position)
            tokens.append(_formed("numeric", source_number, source, position, end))
            position = end
            continue
        if value in SINGLE_PUNCTUATION:
            tokens.append(
                _formed("punctuation", source_number, source, position, position + 1)
            )
            position += 1
            continue
        return _rejected("FORM-1", source_number, position, position + 1)
    return {
        "family": "raw-source-complete",
        "source_ordinal": source_number,
        "tokens": tokens,
        "trivia": trivia,
    }


def scan_sources(sources: Sequence[bytes]) -> dict[str, object]:
    results: list[dict[str, object]] = []
    for source_number, source in enumerate(sources):
        result = _one(source_number, source)
        if result.get("family") == "source-language-rejection":
            return result
        results.append(result)
    return {"family": "raw-scan-complete", "sources": results}


def _decode_case_sources(values: object) -> list[bytes]:
    if not isinstance(values, list) or len(values) == 0:
        raise IndependentRawError("authored raw probe has no sources")
    decoded: list[bytes] = []
    for value in values:
        if not isinstance(value, str) or len(value) % 2 or value.lower() != value:
            raise IndependentRawError("authored raw source is not canonical hexadecimal")
        try:
            source = bytes.fromhex(value)
        except ValueError as error:
            raise IndependentRawError("authored raw source has invalid hexadecimal") from error
        if source.hex() != value:
            raise IndependentRawError("authored raw source has a noncanonical spelling")
        decoded.append(source)
    return decoded


def _interval(source: bytes, value: object) -> tuple[int, int]:
    if (
        not isinstance(value, list)
        or len(value) != 2
        or any(type(part) is not int for part in value)
    ):
        raise IndependentRawError("authored raw interval is malformed")
    begin, finish = value
    if begin < 0 or finish < begin or finish > len(source):
        raise IndependentRawError("authored raw interval is outside its source")
    return begin, finish


def _expected(expectation: object, sources: list[bytes]) -> dict[str, object]:
    if not isinstance(expectation, dict):
        raise IndependentRawError("authored raw expectation is not an object")
    family = expectation.get("family")
    if family == "rejection":
        if sorted(expectation) != ["coordinate", "family", "rule"]:
            raise IndependentRawError("authored raw rejection vocabulary changed")
        where = expectation["coordinate"]
        if (
            not isinstance(where, list)
            or len(where) != 3
            or type(where[0]) is not int
            or where[0] not in range(len(sources))
        ):
            raise IndependentRawError("authored raw rejection source is invalid")
        begin, finish = _interval(sources[where[0]], where[1:])
        return _rejected(expectation["rule"], where[0], begin, finish)
    if family != "complete" or sorted(expectation) != ["family", "sources"]:
        raise IndependentRawError("authored raw completion vocabulary changed")
    rows = expectation["sources"]
    if not isinstance(rows, list) or len(rows) != len(sources):
        raise IndependentRawError("authored raw completion source count changed")
    output: list[dict[str, object]] = []
    for source_number, source in enumerate(sources):
        row = rows[source_number]
        if not isinstance(row, dict) or sorted(row) != ["tokens", "trivia"]:
            raise IndependentRawError("authored raw completion row vocabulary changed")
        formed: list[dict[str, object]] = []
        spaces: list[dict[str, object]] = []
        for token in row["tokens"]:
            if not isinstance(token, list) or len(token) != 3 or not isinstance(token[0], str):
                raise IndependentRawError("authored raw token expectation is malformed")
            begin, finish = _interval(source, token[1:])
            formed.append(_formed(token[0], source_number, source, begin, finish))
        for trivia in row["trivia"]:
            begin, finish = _interval(source, trivia)
            spaces.append(_space(source_number, source, begin, finish))
        output.append(
            {
                "family": "raw-source-complete",
                "source_ordinal": source_number,
                "tokens": formed,
                "trivia": spaces,
            }
        )
    return {"family": "raw-scan-complete", "sources": output}


def evaluate_authored_cases(rows: object) -> list[dict[str, object]]:
    if not isinstance(rows, list) or not rows:
        raise IndependentRawError("authored raw probe collection is malformed")
    identifiers: set[str] = set()
    output: list[dict[str, object]] = []
    for row in rows:
        if not isinstance(row, dict) or sorted(row) != ["expect", "id", "sources_hex"]:
            raise IndependentRawError("authored raw probe vocabulary changed")
        identifier = row["id"]
        if not isinstance(identifier, str) or not identifier or identifier in identifiers:
            raise IndependentRawError("authored raw probe identifier is invalid")
        identifiers.add(identifier)
        sources = _decode_case_sources(row["sources_hex"])
        actual = scan_sources(sources)
        if actual != _expected(row["expect"], sources):
            raise IndependentRawError(f"authored raw probe {identifier} disagrees")
        output.append({"id": identifier, "outcome": actual})
    return output
