"""Primary model of the proposal's source-EBNF diagnostic machine."""

from __future__ import annotations

import re
from typing import Any, Sequence


class PrimaryDiagnosticError(RuntimeError):
    pass


_IDENT = re.compile(r"[a-z][a-z0-9_]*\Z")
_TYPEID = re.compile(r"[A-Z][A-Za-z0-9]*\Z")
_REGION = re.compile(r"'[a-z][a-z0-9_]*\Z")
_LABEL = re.compile(r"@[a-z][a-z0-9_]*\Z")
_OPNAME = re.compile(r"[a-z][a-z0-9_]*\.(?:wrap|trap|checked|sat|strict)\Z")
_INTEGER = re.compile(r"-?[0-9]+_(?:i8|i16|i32|i64|u8|u16|u32|u64)\Z")
_FLOAT = re.compile(r"-?(?:0|[1-9][0-9]*)\.[0-9]+(?:e-?(?:0|[1-9][0-9]*))?_(?:f32|f64)\Z")
_RESERVED = {
    "allocates",
    "arena",
    "break",
    "check",
    "conform",
    "const",
    "contract",
    "deref",
    "doc",
    "effects",
    "else",
    "enum",
    "fn",
    "give",
    "heap",
    "index",
    "let",
    "loop",
    "match",
    "move",
    "pure",
    "reads",
    "region",
    "requires",
    "return",
    "struct",
    "trap",
    "traps",
    "try",
    "unit",
    "uniq",
    "writes",
}
_NAME_PREDICATES = {"IDENT", "LABEL", "OPNAME", "REGIONID", "TYPEID"}
_ATOM_SLOTS = {"atom_list", "fieldinit", "index-offset"}
_ENTRY_POINTS = {
    "item",
    "program-item-star",
    "requires-entry",
    "requires-entry-star",
    "stmt",
    "stmt-star",
}


def _token(raw: object, source: bytes) -> dict[str, Any]:
    if not isinstance(raw, dict) or set(raw) != {"shape", "span", "spelling"}:
        raise PrimaryDiagnosticError("diagnostic token fields changed")
    shape = raw["shape"]
    spelling = raw["spelling"]
    span = raw["span"]
    if shape not in {
        "label",
        "lower-word",
        "numeric",
        "opname",
        "punctuation",
        "region",
        "source-end",
        "string",
        "upper-word",
    }:
        raise PrimaryDiagnosticError("diagnostic token shape changed")
    if not isinstance(spelling, str) or not spelling.isascii():
        raise PrimaryDiagnosticError("diagnostic token spelling is not ASCII")
    if (
        not isinstance(span, list)
        or len(span) != 2
        or any(type(offset) is not int for offset in span)
        or span[0] < 0
        or span[1] < span[0]
        or span[1] > len(source)
    ):
        raise PrimaryDiagnosticError("diagnostic token span is invalid")
    if shape == "source-end":
        if spelling or span != [len(source), len(source)]:
            raise PrimaryDiagnosticError("SOURCE_END token is not the exact source end")
    elif source[span[0] : span[1]] != spelling.encode("ascii"):
        raise PrimaryDiagnosticError("diagnostic token is not source-anchored")
    return raw


def _predicate(raw: object) -> dict[str, Any]:
    if not isinstance(raw, dict) or set(raw) != {"name", "origin", "rank"}:
        raise PrimaryDiagnosticError("SELECT_2 predicate fields changed")
    name = raw["name"]
    if not isinstance(name, str) or not name:
        raise PrimaryDiagnosticError("SELECT_2 predicate name is invalid")
    if not (name.startswith("fixed:") or name in _NAME_PREDICATES | {"LITERAL", "SOURCE_END"}):
        raise PrimaryDiagnosticError(f"unsupported SELECT_2 predicate {name!r}")
    if raw["origin"] not in {"caller", "inside"}:
        raise PrimaryDiagnosticError("SELECT_2 provenance changed")
    if type(raw["rank"]) is not int or raw["rank"] < 0:
        raise PrimaryDiagnosticError("SELECT_2 terminal rank is invalid")
    return raw


def _accepts(predicate: dict[str, Any], token: dict[str, Any]) -> bool:
    name = predicate["name"]
    spelling = token["spelling"]
    if name.startswith("fixed:"):
        return spelling == name[6:]
    if name == "SOURCE_END":
        return token["shape"] == "source-end"
    if name == "IDENT":
        return bool(_IDENT.fullmatch(spelling)) and spelling not in _RESERVED
    if name == "TYPEID":
        return bool(_TYPEID.fullmatch(spelling))
    if name == "REGIONID":
        return bool(_REGION.fullmatch(spelling))
    if name == "LABEL":
        return bool(_LABEL.fullmatch(spelling))
    if name == "OPNAME":
        return bool(_OPNAME.fullmatch(spelling))
    if name == "LITERAL":
        return (
            spelling == "unit"
            or token["shape"] == "string"
            or bool(_INTEGER.fullmatch(spelling))
            or bool(_FLOAT.fullmatch(spelling))
            or spelling in {"0_T", "1_T"}
        )
    raise AssertionError(name)


def _base_shape(predicate: str, token: dict[str, Any]) -> bool:
    return {
        "IDENT": token["shape"] == "lower-word",
        "LABEL": token["shape"] == "label",
        "LITERAL": token["shape"] in {"numeric", "string"},
        "OPNAME": token["shape"] in {"lower-word", "opname"} and "." in token["spelling"],
        "REGIONID": token["shape"] == "region",
        "TYPEID": token["shape"] == "upper-word",
    }.get(predicate, False)


def _owner(predicate: str) -> str | None:
    if predicate in _NAME_PREDICATES:
        return "FORM-3"
    if predicate == "LITERAL":
        return "FORM-5"
    return None


def _display(predicate: str) -> str:
    return predicate[6:] if predicate.startswith("fixed:") else predicate


def _row_score(row: list[dict[str, Any]], tokens: list[dict[str, Any]]) -> int:
    if len(row) != 2:
        raise PrimaryDiagnosticError("SELECT_2 row is not exactly two predicates")
    if not _accepts(row[0], tokens[0]):
        return 0
    if not _accepts(row[1], tokens[1]):
        return 1
    raise PrimaryDiagnosticError("diagnostic frontier contains a fully matching SELECT_2 row")


def _expected(rows: list[list[dict[str, Any]]], position: int) -> list[str]:
    ranks: dict[str, int] = {}
    for row in rows:
        predicate = row[position]
        name = predicate["name"]
        prior = ranks.get(name)
        if prior is not None and prior != predicate["rank"]:
            raise PrimaryDiagnosticError("one terminal predicate has conflicting grammar ranks")
        ranks[name] = predicate["rank"]
    ordered = sorted(ranks, key=lambda name: (name == "SOURCE_END", ranks[name]))
    return [_display(name) for name in ordered]


def _source_rejection(rule: str, source_ordinal: int, span: list[int], expected: list[str]) -> dict[str, object]:
    return {
        "expected_terminals": expected,
        "family": "source-language-rejection",
        "location": {"coordinate": [source_ordinal, span[0], span[1]], "kind": "SourceBytes"},
        "rule": rule,
    }


def _context(frontier: dict[str, Any]) -> dict[str, Any]:
    raw = frontier["context"]
    if not isinstance(raw, dict) or set(raw) != {
        "atom_slot",
        "dotted_window",
        "entry",
        "lookup_state",
        "mandatory_name_roots",
        "program_leftover",
    }:
        raise PrimaryDiagnosticError("diagnostic context fields changed")
    if raw["atom_slot"] is not None and raw["atom_slot"] not in _ATOM_SLOTS:
        raise PrimaryDiagnosticError("diagnostic atom occurrence changed")
    if raw["entry"] is not None and raw["entry"] not in _ENTRY_POINTS:
        raise PrimaryDiagnosticError("diagnostic construct entry changed")
    if raw["lookup_state"] not in {"declared", "not-applicable", "undeclared"}:
        raise PrimaryDiagnosticError("diagnostic lookup-state probe changed")
    if type(raw["program_leftover"]) is not bool:
        raise PrimaryDiagnosticError("diagnostic leftover flag changed")
    return raw


def _dotted_coordinate(
    raw: object, source: bytes, boundary: dict[str, Any]
) -> list[int] | None:
    if raw is None:
        return None
    if not isinstance(raw, list) or len(raw) != 4:
        raise PrimaryDiagnosticError("dotted diagnostic window is not four tokens")
    window = [_token(item, source) for item in raw]
    ident = {"name": "IDENT", "origin": "inside", "rank": 0}
    if (
        not _accepts(ident, window[0])
        or window[1]["spelling"] != "."
        or not _accepts(ident, window[2])
        or window[3]["spelling"] not in {"(", "<"}
    ):
        raise PrimaryDiagnosticError("dotted diagnostic window has the wrong token shapes")
    for left, right in zip(window, window[1:]):
        if left["span"][1] > right["span"][0]:
            raise PrimaryDiagnosticError("dotted diagnostic window tokens overlap or reorder")
    if not any(
        item["span"] == boundary["span"]
        and item["shape"] == boundary["shape"]
        and item["spelling"] == boundary["spelling"]
        for item in window
    ):
        return None
    return [window[0]["span"][0], window[2]["span"][1]]


def _transparent_shape(raw: object) -> dict[str, Any]:
    if not isinstance(raw, dict) or not isinstance(raw.get("kind"), str):
        raise PrimaryDiagnosticError("transparent-name topology node is malformed")
    kind = raw["kind"]
    expected_fields = {
        "choice": {"arms", "kind"},
        "group": {"child", "kind"},
        "matched": {"kind"},
        "nullable": {"directions", "kind", "source"},
        "production": {"body", "kind"},
        "sequence": {"children", "kind"},
        "terminal": {"kind", "predicate"},
    }
    if kind not in expected_fields or set(raw) != expected_fields[kind]:
        raise PrimaryDiagnosticError("transparent-name topology vocabulary changed")
    if kind == "terminal":
        _predicate(raw["predicate"])
    elif kind in {"group", "production"}:
        _transparent_shape(raw["child"] if kind == "group" else raw["body"])
    elif kind in {"choice", "sequence"}:
        field = "arms" if kind == "choice" else "children"
        if not isinstance(raw[field], list) or not raw[field]:
            raise PrimaryDiagnosticError("transparent-name topology list is empty")
        for child in raw[field]:
            _transparent_shape(child)
    elif kind == "nullable":
        if (
            not isinstance(raw["source"], str)
            or not raw["source"]
            or raw["source"][-1] not in "?*+"
            or not isinstance(raw["directions"], list)
            or len(raw["directions"]) != 2
        ):
            raise PrimaryDiagnosticError("nullable topology lacks two named source directions")
        for direction in raw["directions"]:
            _transparent_shape(direction)
    return raw


def _transparent_result(
    node: dict[str, Any], tokens: list[dict[str, Any]]
) -> tuple[str, set[str]]:
    kind = node["kind"]
    if kind == "matched":
        return "complete", set()
    if kind == "terminal":
        predicate = _predicate(node["predicate"])
        if _accepts(predicate, tokens[0]):
            return "accepted", set()
        name = predicate["name"]
        if name in _NAME_PREDICATES and any(
            _accepts({"name": other, "origin": "inside", "rank": 0}, tokens[0])
            for other in _NAME_PREDICATES - {name}
        ):
            return "paths", {name}
        return "blocked", set()
    if kind == "group":
        return _transparent_result(node["child"], tokens)
    if kind == "production":
        return _transparent_result(node["body"], tokens)
    if kind == "choice":
        return "blocked", set()
    if kind == "sequence":
        for child in node["children"]:
            status, names = _transparent_result(child, tokens)
            if status == "complete":
                continue
            return status, names
        return "complete", set()
    if kind == "nullable":
        results = [_transparent_result(direction, tokens) for direction in node["directions"]]
        if any(status == "accepted" for status, _ in results):
            return "accepted", set()
        names = set().union(*(found for status, found in results if status == "paths"))
        return ("paths", names) if names else ("blocked", set())
    raise AssertionError(kind)


def _contains_predicate(node: dict[str, Any], predicate: dict[str, Any]) -> bool:
    kind = node["kind"]
    if kind == "terminal":
        return node["predicate"] == predicate
    if kind == "group":
        return _contains_predicate(node["child"], predicate)
    if kind == "production":
        return _contains_predicate(node["body"], predicate)
    if kind in {"choice", "sequence"}:
        field = "arms" if kind == "choice" else "children"
        return any(_contains_predicate(child, predicate) for child in node[field])
    if kind == "nullable":
        return any(_contains_predicate(child, predicate) for child in node["directions"])
    return False


def _transparent_names(
    raw: object,
    tokens: list[dict[str, Any]],
    maximal_predicates: list[dict[str, Any]],
) -> set[str]:
    if not isinstance(raw, list):
        raise PrimaryDiagnosticError("transparent-name root collection is malformed")
    if raw:
        if any(not isinstance(item, dict) or set(item) != {"start", "topology"} for item in raw):
            raise PrimaryDiagnosticError("transparent-name root binding fields changed")
        starts = [_predicate(item["start"]) for item in raw]
        key = lambda item: (item["name"], item["origin"], item["rank"])
        if sorted(map(key, starts)) != sorted(map(key, maximal_predicates)):
            raise PrimaryDiagnosticError(
                "transparent-name roots do not cover every maximal predicate occurrence"
            )
    names: set[str] = set()
    for item in raw:
        start = _predicate(item["start"])
        node = _transparent_shape(item["topology"])
        if not _contains_predicate(node, start):
            raise PrimaryDiagnosticError(
                "transparent-name topology does not contain its provenance start"
            )
        status, found = _transparent_result(node, tokens)
        if status == "complete":
            raise PrimaryDiagnosticError("transparent-name root is already complete")
        if status == "paths":
            names.update(found)
    return names


def _frontier(source_ordinal: int, source: bytes, raw: object) -> dict[str, object]:
    if not isinstance(raw, dict) or set(raw) != {"arms", "context", "direct", "owner", "tokens"}:
        raise PrimaryDiagnosticError("diagnostic frontier fields changed")
    if not isinstance(raw["owner"], str) or not raw["owner"]:
        raise PrimaryDiagnosticError("diagnostic frontier owner is invalid")
    if type(raw["direct"]) is not bool:
        raise PrimaryDiagnosticError("diagnostic direct flag is invalid")
    tokens_raw = raw["tokens"]
    if not isinstance(tokens_raw, list) or len(tokens_raw) != 2:
        raise PrimaryDiagnosticError("diagnostic lookahead is not exactly two tokens")
    tokens = [_token(token, source) for token in tokens_raw]
    arms_raw = raw["arms"]
    if not isinstance(arms_raw, list) or not arms_raw:
        raise PrimaryDiagnosticError("diagnostic frontier has no arms")
    arms: list[tuple[list[list[dict[str, Any]]], object]] = []
    for arm in arms_raw:
        if not isinstance(arm, dict) or set(arm) != {"child", "rows"}:
            raise PrimaryDiagnosticError("diagnostic arm fields changed")
        if not isinstance(arm["rows"], list) or not arm["rows"]:
            raise PrimaryDiagnosticError("diagnostic arm has no SELECT_2 rows")
        rows = []
        for row_raw in arm["rows"]:
            if not isinstance(row_raw, list):
                raise PrimaryDiagnosticError("SELECT_2 row is not a list")
            rows.append([_predicate(item) for item in row_raw])
        arms.append((rows, arm["child"]))
    if raw["direct"] and (len(arms) != 1 or len(arms[0][0]) != 1):
        raise PrimaryDiagnosticError("direct mismatch does not contain exactly one row")

    scores: list[int] = []
    scored_rows: list[tuple[int, list[dict[str, Any]]]] = []
    for rows, _ in arms:
        row_scores = [(_row_score(row, tokens), row) for row in rows]
        scores.append(max(score for score, _ in row_scores))
        scored_rows.extend(row_scores)
    maximum = max(scores)
    maximal_rows = [row for score, row in scored_rows if score == maximum]
    expected = _expected(maximal_rows, maximum)
    boundary = tokens[maximum]
    context = _context(raw)

    dotted = _dotted_coordinate(context["dotted_window"], source, boundary)
    if dotted is not None:
        return _source_rejection("FORM-3", source_ordinal, dotted, expected)

    if context["atom_slot"] is not None:
        start_kind = next(
            (
                name
                for name in ("IDENT", "OPNAME", "TYPEID")
                if _accepts({"name": name, "origin": "inside", "rank": 0}, tokens[0])
            ),
            None,
        )
        if start_kind is not None and tokens[1]["spelling"] in {"(", "<"}:
            return _source_rejection(
                "GRAM-9",
                source_ordinal,
                [tokens[0]["span"][0], tokens[1]["span"][1]],
                expected,
            )

    expected_predicates = [row[maximum] for row in maximal_rows]
    qualifying: list[tuple[int, str]] = []
    for predicate in expected_predicates:
        name = predicate["name"]
        owner = _owner(name)
        if owner is None or _accepts(predicate, boundary):
            continue
        if _base_shape(name, boundary):
            qualifying.append((predicate["rank"], owner))
            continue
    mandatory_names = _transparent_names(
        context["mandatory_name_roots"], tokens, expected_predicates
    )
    direct_names = {predicate["name"] for predicate in expected_predicates}
    if raw["direct"] and len(direct_names) == 1:
        direct_name = next(iter(direct_names))
        if direct_name in _NAME_PREDICATES:
            mandatory_names.add(direct_name)
    if len(mandatory_names) == 1:
        name = next(iter(mandatory_names))
        if name in _NAME_PREDICATES and any(
            _accepts({"name": other, "origin": "inside", "rank": 0}, boundary)
            for other in _NAME_PREDICATES - {name}
        ):
            matching_ranks = [
                predicate["rank"]
                for predicate in expected_predicates
                if predicate["name"] == name
            ]
            if not matching_ranks:
                raise PrimaryDiagnosticError(
                    "transparent name candidate is absent from the frontier rows"
                )
            rank = min(matching_ranks)
            qualifying.append((rank, "FORM-3"))
    if qualifying:
        _, owner = min(qualifying)
        return _source_rejection(owner, source_ordinal, boundary["span"], expected)

    if context["entry"] is not None and _accepts(
        {"name": "IDENT", "origin": "inside", "rank": 0}, tokens[0]
    ):
        return _source_rejection("FORM-1", source_ordinal, tokens[0]["span"], expected)

    if context["program_leftover"]:
        return _source_rejection("GRAM-2", source_ordinal, tokens[0]["span"], ["SOURCE_END"])

    if raw["direct"]:
        return _source_rejection(raw["owner"], source_ordinal, boundary["span"], expected)

    winners = [index for index, score in enumerate(scores) if score == maximum]
    if len(winners) == 1:
        winner = winners[0]
        rows = [row for row in arms[winner][0] if _row_score(row, tokens) == maximum]
        inside = all(row[maximum]["origin"] == "inside" for row in rows)
        if inside:
            child = arms[winner][1]
            if child is None:
                raise PrimaryDiagnosticError("diagnostic traversal would reach a successful end")
            return _frontier(source_ordinal, source, child)
    return _source_rejection(raw["owner"], source_ordinal, boundary["span"], expected)


def evaluate_grammar_case(sources: Sequence[bytes], case: dict[str, Any]) -> dict[str, object]:
    if set(case) != {"bundle", "expect", "frontier", "id", "kind", "source"}:
        raise PrimaryDiagnosticError("grammar case fields changed")
    source_ordinal = case["source"]
    if type(source_ordinal) is not int or source_ordinal < 0 or source_ordinal >= len(sources):
        raise PrimaryDiagnosticError("grammar case source ordinal is invalid")
    return _frontier(source_ordinal, sources[source_ordinal], case["frontier"])


def evaluate_dotted_position_case(
    sources: Sequence[bytes], case: dict[str, Any]
) -> dict[str, object]:
    if set(case) != {"boundary_index", "bundle", "expect", "id", "kind", "source"}:
        raise PrimaryDiagnosticError("dotted-position case fields changed")
    source_ordinal = case["source"]
    if type(source_ordinal) is not int or source_ordinal not in range(len(sources)):
        raise PrimaryDiagnosticError("dotted-position source ordinal is invalid")
    if type(case["boundary_index"]) is not int or case["boundary_index"] not in range(4):
        raise PrimaryDiagnosticError("dotted-position boundary index is invalid")
    source = sources[source_ordinal]
    match = re.fullmatch(rb"([a-z][a-z0-9_]*)\.([a-z][a-z0-9_]*)([<(])", source)
    if match is None:
        raise PrimaryDiagnosticError("dotted-position source is not one exact four-token window")
    first = match.group(1).decode("ascii")
    second = match.group(2).decode("ascii")
    if first in _RESERVED or second in _RESERVED:
        raise PrimaryDiagnosticError("dotted-position source does not contain two IDENTs")
    first_end = len(match.group(1))
    second_end = first_end + 1 + len(match.group(2))
    spans = [[0, first_end], [first_end, first_end + 1], [first_end + 1, second_end], [second_end, len(source)]]
    index = case["boundary_index"]
    return {
        "attribution": _source_rejection("FORM-3", source_ordinal, [0, second_end], [";"]),
        "boundary_index": index,
        "boundary_span": spans[index],
        "family": "dotted-position-evidence",
    }


def evaluate_atom_shape_case(
    sources: Sequence[bytes], case: dict[str, Any]
) -> dict[str, object]:
    if set(case) != {"bundle", "expect", "id", "kind", "slot", "source", "tokens"}:
        raise PrimaryDiagnosticError("atom-shape case fields changed")
    source_ordinal = case["source"]
    if type(source_ordinal) is not int or source_ordinal < 0 or source_ordinal >= len(sources):
        raise PrimaryDiagnosticError("atom-shape source ordinal is invalid")
    if case["slot"] not in _ATOM_SLOTS:
        raise PrimaryDiagnosticError("atom-shape occurrence changed")
    source = sources[source_ordinal]
    raw_tokens = case["tokens"]
    if not isinstance(raw_tokens, list) or len(raw_tokens) != 2:
        raise PrimaryDiagnosticError("atom-shape lookahead is not two tokens")
    tokens = [_token(raw, source) for raw in raw_tokens]
    starts = any(
        _accepts({"name": name, "origin": "inside", "rank": 0}, tokens[0])
        for name in ("IDENT", "OPNAME", "TYPEID")
    ) and tokens[1]["spelling"] in {"(", "<"}
    return {
        "family": "diagnostic-shape-probe",
        "forbidden_call_or_construct_start": starts,
        "slot": case["slot"],
    }
