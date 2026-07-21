"""Independent source-EBNF diagnostic interpreter for proposal evidence."""

from __future__ import annotations

import re
from typing import Any, Sequence


class IndependentDiagnosticError(RuntimeError):
    pass


FIXED_WORDS = frozenset(
    "allocates arena break check conform const contract deref doc effects else enum fn give heap "
    "index let loop match move pure reads region requires return struct trap traps try unit uniq writes".split()
)
NAME_CLASSES = ("IDENT", "TYPEID", "REGIONID", "LABEL", "OPNAME")
ATOM_OCCURRENCES = ("atom_list", "fieldinit", "index-offset")
CONSTRUCT_ENTRIES = (
    "item",
    "program-item-star",
    "requires-entry",
    "requires-entry-star",
    "stmt",
    "stmt-star",
)


def _matches(name: str, token: dict[str, Any]) -> bool:
    word = token["spelling"]
    if name[:6] == "fixed:":
        return word == name[6:]
    if name == "SOURCE_END":
        return token["shape"] == "source-end"
    if name == "IDENT":
        return re.fullmatch(r"[a-z][a-z0-9_]*", word) is not None and word not in FIXED_WORDS
    if name == "TYPEID":
        return re.fullmatch(r"[A-Z][A-Za-z0-9]*", word) is not None
    if name == "REGIONID":
        return re.fullmatch(r"'[a-z][a-z0-9_]*", word) is not None
    if name == "LABEL":
        return re.fullmatch(r"@[a-z][a-z0-9_]*", word) is not None
    if name == "OPNAME":
        return re.fullmatch(r"[a-z][a-z0-9_]*\.(wrap|trap|checked|sat|strict)", word) is not None
    if name == "LITERAL":
        integer = re.fullmatch(r"-?[0-9]+_(i8|i16|i32|i64|u8|u16|u32|u64)", word)
        floating = re.fullmatch(r"-?(0|[1-9][0-9]*)\.[0-9]+(e-?(0|[1-9][0-9]*))?_(f32|f64)", word)
        return word in ("unit", "0_T", "1_T") or token["shape"] == "string" or integer is not None or floating is not None
    raise IndependentDiagnosticError(f"unknown terminal predicate: {name}")


def _shape_before_restriction(name: str, token: dict[str, Any]) -> bool:
    shape = token["shape"]
    if name == "IDENT":
        return shape == "lower-word"
    if name == "TYPEID":
        return shape == "upper-word"
    if name == "REGIONID":
        return shape == "region"
    if name == "LABEL":
        return shape == "label"
    if name == "OPNAME":
        return shape in ("lower-word", "opname") and "." in token["spelling"]
    if name == "LITERAL":
        return shape in ("numeric", "string")
    return False


def _terminal_owner(name: str) -> str | None:
    if name in NAME_CLASSES:
        return "FORM-3"
    if name == "LITERAL":
        return "FORM-5"
    return None


def _display(name: str) -> str:
    return name.split(":", 1)[1] if name.startswith("fixed:") else name


def _checked_token(raw: object, source: bytes) -> dict[str, Any]:
    if not isinstance(raw, dict) or sorted(raw) != ["shape", "span", "spelling"]:
        raise IndependentDiagnosticError("lookahead token vocabulary changed")
    shape, spelling, span = raw["shape"], raw["spelling"], raw["span"]
    permitted = {"label", "lower-word", "numeric", "opname", "punctuation", "region", "source-end", "string", "upper-word"}
    if shape not in permitted or not isinstance(spelling, str) or not spelling.isascii():
        raise IndependentDiagnosticError("lookahead token class or spelling is invalid")
    if not isinstance(span, list) or len(span) != 2 or any(type(item) is not int for item in span):
        raise IndependentDiagnosticError("lookahead token interval is malformed")
    begin, finish = span
    if begin < 0 or begin > finish or finish > len(source):
        raise IndependentDiagnosticError("lookahead token interval escapes its source")
    if shape == "source-end":
        if spelling != "" or begin != len(source) or finish != len(source):
            raise IndependentDiagnosticError("lookahead SOURCE_END is not the source end")
    elif source[begin:finish] != spelling.encode("ascii"):
        raise IndependentDiagnosticError("lookahead token bytes disagree with source")
    return raw


def _checked_predicate(raw: object) -> dict[str, Any]:
    if not isinstance(raw, dict) or sorted(raw) != ["name", "origin", "rank"]:
        raise IndependentDiagnosticError("SELECT_2 predicate vocabulary changed")
    name = raw["name"]
    if not isinstance(name, str) or not name or not (name.startswith("fixed:") or name in set(NAME_CLASSES) | {"LITERAL", "SOURCE_END"}):
        raise IndependentDiagnosticError("SELECT_2 predicate name is outside the evidence schema")
    if raw["origin"] not in ("inside", "caller") or type(raw["rank"]) is not int or raw["rank"] < 0:
        raise IndependentDiagnosticError("SELECT_2 provenance or terminal rank is invalid")
    return raw


def _score(row: list[dict[str, Any]], tokens: list[dict[str, Any]]) -> int:
    if len(row) != 2:
        raise IndependentDiagnosticError("SELECT_2 row width changed")
    position = 0
    while position < 2 and _matches(row[position]["name"], tokens[position]):
        position += 1
    if position == 2:
        raise IndependentDiagnosticError("failed frontier has a successful SELECT_2 row")
    return position


def _outcome(rule: str, source_number: int, span: list[int], expected: list[str]) -> dict[str, object]:
    return {
        "expected_terminals": expected,
        "family": "source-language-rejection",
        "location": {"coordinate": [source_number, span[0], span[1]], "kind": "SourceBytes"},
        "rule": rule,
    }


def _context(raw: object) -> dict[str, Any]:
    names = [
        "atom_slot",
        "dotted_window",
        "entry",
        "lookup_state",
        "mandatory_name_roots",
        "program_leftover",
    ]
    if not isinstance(raw, dict) or sorted(raw) != names:
        raise IndependentDiagnosticError("diagnostic context vocabulary changed")
    if raw["atom_slot"] is not None and raw["atom_slot"] not in ATOM_OCCURRENCES:
        raise IndependentDiagnosticError("atom occurrence is outside the closed set")
    if raw["entry"] is not None and raw["entry"] not in CONSTRUCT_ENTRIES:
        raise IndependentDiagnosticError("construct entry is outside the closed set")
    if raw["lookup_state"] not in ("declared", "not-applicable", "undeclared"):
        raise IndependentDiagnosticError("lookup-independence probe has an invalid state")
    if type(raw["program_leftover"]) is not bool:
        raise IndependentDiagnosticError("diagnostic leftover flag is not boolean")
    return raw


def _dotted_span(
    raw: object, source: bytes, boundary: dict[str, Any]
) -> list[int] | None:
    if raw is None:
        return None
    if not isinstance(raw, list) or len(raw) != 4:
        raise IndependentDiagnosticError("dotted actual-token window does not have width four")
    window = [_checked_token(item, source) for item in raw]
    if (
        not _matches("IDENT", window[0])
        or window[1]["spelling"] != "."
        or not _matches("IDENT", window[2])
        or window[3]["spelling"] not in ("(", "<")
    ):
        raise IndependentDiagnosticError("dotted actual-token window has an invalid shape")
    if any(left["span"][1] > right["span"][0] for left, right in zip(window, window[1:])):
        raise IndependentDiagnosticError("dotted actual-token window is not token ordered")
    member = any(
        item["span"] == boundary["span"]
        and item["shape"] == boundary["shape"]
        and item["spelling"] == boundary["spelling"]
        for item in window
    )
    return [window[0]["span"][0], window[2]["span"][1]] if member else None


def _name_node(raw: object) -> dict[str, Any]:
    if not isinstance(raw, dict) or not isinstance(raw.get("kind"), str):
        raise IndependentDiagnosticError("mandatory-name source topology is malformed")
    kind = raw["kind"]
    vocabularies = {
        "choice": ["arms", "kind"],
        "group": ["child", "kind"],
        "matched": ["kind"],
        "nullable": ["directions", "kind", "source"],
        "production": ["body", "kind"],
        "sequence": ["children", "kind"],
        "terminal": ["kind", "predicate"],
    }
    if kind not in vocabularies or sorted(raw) != vocabularies[kind]:
        raise IndependentDiagnosticError("mandatory-name topology vocabulary changed")
    if kind == "terminal":
        _checked_predicate(raw["predicate"])
    elif kind == "group":
        _name_node(raw["child"])
    elif kind == "production":
        _name_node(raw["body"])
    elif kind in ("choice", "sequence"):
        children = raw["arms" if kind == "choice" else "children"]
        if not isinstance(children, list) or not children:
            raise IndependentDiagnosticError("mandatory-name topology list is empty")
        for child in children:
            _name_node(child)
    elif kind == "nullable":
        if (
            not isinstance(raw["source"], str)
            or not raw["source"]
            or raw["source"][-1] not in "?*+"
            or not isinstance(raw["directions"], list)
            or len(raw["directions"]) != 2
        ):
            raise IndependentDiagnosticError("nullable source topology is not two-directional")
        for direction in raw["directions"]:
            _name_node(direction)
    return raw


def _name_result(
    node: dict[str, Any], tokens: list[dict[str, Any]]
) -> tuple[str, set[str]]:
    kind = node["kind"]
    if kind == "matched":
        return "complete", set()
    if kind == "terminal":
        predicate = _checked_predicate(node["predicate"])
        if _matches(predicate["name"], tokens[0]):
            return "accepted", set()
        name = predicate["name"]
        if name in NAME_CLASSES and any(
            alternative != name and _matches(alternative, tokens[0])
            for alternative in NAME_CLASSES
        ):
            return "paths", {name}
        return "blocked", set()
    if kind == "group":
        return _name_result(node["child"], tokens)
    if kind == "production":
        return _name_result(node["body"], tokens)
    if kind == "choice":
        return "blocked", set()
    if kind == "sequence":
        for child in node["children"]:
            status, terminals = _name_result(child, tokens)
            if status == "complete":
                continue
            return status, terminals
        return "complete", set()
    if kind == "nullable":
        outcomes = [_name_result(direction, tokens) for direction in node["directions"]]
        if any(status == "accepted" for status, _ in outcomes):
            return "accepted", set()
        terminals = set().union(*(found for status, found in outcomes if status == "paths"))
        return ("paths", terminals) if terminals else ("blocked", set())
    raise AssertionError(kind)


def _has_start(node: dict[str, Any], start: dict[str, Any]) -> bool:
    kind = node["kind"]
    if kind == "terminal":
        return node["predicate"] == start
    if kind == "group":
        return _has_start(node["child"], start)
    if kind == "production":
        return _has_start(node["body"], start)
    if kind in ("choice", "sequence"):
        children = node["arms" if kind == "choice" else "children"]
        return any(_has_start(child, start) for child in children)
    if kind == "nullable":
        return any(_has_start(direction, start) for direction in node["directions"])
    return False


def _mandatory_names(
    raw: object,
    tokens: list[dict[str, Any]],
    maximal_predicates: list[dict[str, Any]],
) -> set[str]:
    if not isinstance(raw, list):
        raise IndependentDiagnosticError("mandatory-name roots are not a list")
    if raw:
        if any(not isinstance(value, dict) or sorted(value) != ["start", "topology"] for value in raw):
            raise IndependentDiagnosticError("mandatory-name provenance binding changed")
        starts = [_checked_predicate(value["start"]) for value in raw]
        projection = lambda value: (value["name"], value["origin"], value["rank"])
        if sorted(map(projection, starts)) != sorted(map(projection, maximal_predicates)):
            raise IndependentDiagnosticError(
                "mandatory-name roots do not bind every maximal predicate occurrence"
            )
    result: set[str] = set()
    for value in raw:
        start = _checked_predicate(value["start"])
        node = _name_node(value["topology"])
        if not _has_start(node, start):
            raise IndependentDiagnosticError(
                "mandatory-name topology omits its maximal predicate start"
            )
        status, found = _name_result(node, tokens)
        if status == "complete":
            raise IndependentDiagnosticError("mandatory-name root has no unconsumed terminal")
        if status == "paths":
            result.update(found)
    return result


def _interpret(source_number: int, source: bytes, frontier: object) -> dict[str, object]:
    if not isinstance(frontier, dict) or sorted(frontier) != ["arms", "context", "direct", "owner", "tokens"]:
        raise IndependentDiagnosticError("frontier vocabulary changed")
    if not isinstance(frontier["owner"], str) or not frontier["owner"] or type(frontier["direct"]) is not bool:
        raise IndependentDiagnosticError("frontier owner or direct flag is invalid")
    token_rows = frontier["tokens"]
    if not isinstance(token_rows, list) or len(token_rows) != 2:
        raise IndependentDiagnosticError("frontier does not carry two-token lookahead")
    tokens = [_checked_token(value, source) for value in token_rows]
    arm_rows = frontier["arms"]
    if not isinstance(arm_rows, list) or not arm_rows:
        raise IndependentDiagnosticError("frontier has no decision arms")
    arms: list[dict[str, Any]] = []
    flattened: list[tuple[int, dict[str, Any], int]] = []
    arm_scores: list[int] = []
    for arm_number, arm in enumerate(arm_rows):
        if not isinstance(arm, dict) or sorted(arm) != ["child", "rows"] or not isinstance(arm["rows"], list) or not arm["rows"]:
            raise IndependentDiagnosticError("decision arm is malformed")
        rows: list[list[dict[str, Any]]] = []
        local_scores: list[int] = []
        for row_raw in arm["rows"]:
            if not isinstance(row_raw, list):
                raise IndependentDiagnosticError("SELECT_2 row is not a list")
            row = [_checked_predicate(item) for item in row_raw]
            score = _score(row, tokens)
            rows.append(row)
            local_scores.append(score)
            flattened.append((score, {"row": row}, arm_number))
        arms.append({"child": arm["child"], "rows": rows})
        arm_scores.append(max(local_scores))
    if frontier["direct"] and (len(arms) != 1 or len(arms[0]["rows"]) != 1):
        raise IndependentDiagnosticError("direct mismatch is not a single-row frontier")
    furthest = max(arm_scores)
    maximal = [entry[1]["row"] for entry in flattened if entry[0] == furthest]
    ranks: dict[str, int] = {}
    for row in maximal:
        predicate = row[furthest]
        previous = ranks.setdefault(predicate["name"], predicate["rank"])
        if previous != predicate["rank"]:
            raise IndependentDiagnosticError("terminal occurrence rank is inconsistent")
    ordered_names = sorted(ranks, key=lambda name: (1 if name == "SOURCE_END" else 0, ranks[name]))
    expected = [_display(name) for name in ordered_names]
    boundary = tokens[furthest]
    context = _context(frontier["context"])

    dotted = _dotted_span(context["dotted_window"], source, boundary)
    if dotted is not None:
        return _outcome("FORM-3", source_number, dotted, expected)

    slot = context["atom_slot"]
    if slot is not None:
        first_kind = next((name for name in ("IDENT", "OPNAME", "TYPEID") if _matches(name, tokens[0])), None)
        if first_kind is not None and tokens[1]["spelling"] in ("(", "<"):
            return _outcome("GRAM-9", source_number, [tokens[0]["span"][0], tokens[1]["span"][1]], expected)

    candidates: list[tuple[int, str]] = []
    for row in maximal:
        predicate = row[furthest]
        name = predicate["name"]
        owner = _terminal_owner(name)
        if owner is None or _matches(name, boundary):
            continue
        if _shape_before_restriction(name, boundary):
            candidates.append((predicate["rank"], owner))
    maximal_predicates = [row[furthest] for row in maximal]
    mandatory_names = _mandatory_names(
        context["mandatory_name_roots"], tokens, maximal_predicates
    )
    direct_names = {row[furthest]["name"] for row in maximal}
    if frontier["direct"] and len(direct_names) == 1:
        direct_name = next(iter(direct_names))
        if direct_name in NAME_CLASSES:
            mandatory_names.add(direct_name)
    if len(mandatory_names) == 1:
        name = next(iter(mandatory_names))
        if name in NAME_CLASSES and any(
            alternative != name and _matches(alternative, boundary)
            for alternative in NAME_CLASSES
        ):
            matching_ranks = [
                row[furthest]["rank"]
                for row in maximal
                if row[furthest]["name"] == name
            ]
            if not matching_ranks:
                raise IndependentDiagnosticError(
                    "mandatory-name candidate is absent from maximal rows"
                )
            rank = min(matching_ranks)
            candidates.append((rank, "FORM-3"))
    if candidates:
        return _outcome(min(candidates)[1], source_number, boundary["span"], expected)

    if context["entry"] is not None and _matches("IDENT", tokens[0]):
        return _outcome("FORM-1", source_number, tokens[0]["span"], expected)

    if context["program_leftover"]:
        return _outcome("GRAM-2", source_number, tokens[0]["span"], ["SOURCE_END"])

    if frontier["direct"]:
        return _outcome(frontier["owner"], source_number, boundary["span"], expected)

    leaders = [number for number, score in enumerate(arm_scores) if score == furthest]
    if len(leaders) == 1:
        winner = leaders[0]
        rows = [row for row in arms[winner]["rows"] if _score(row, tokens) == furthest]
        if all(row[furthest]["origin"] == "inside" for row in rows):
            child = arms[winner]["child"]
            if child is None:
                raise IndependentDiagnosticError("diagnostic descent reaches a successful end")
            return _interpret(source_number, source, child)
    return _outcome(frontier["owner"], source_number, boundary["span"], expected)


def evaluate_grammar_case(sources: Sequence[bytes], case: dict[str, Any]) -> dict[str, object]:
    if sorted(case) != ["bundle", "expect", "frontier", "id", "kind", "source"]:
        raise IndependentDiagnosticError("grammar probe vocabulary changed")
    source_number = case["source"]
    if type(source_number) is not int or source_number < 0 or source_number >= len(sources):
        raise IndependentDiagnosticError("grammar probe source ordinal is invalid")
    return _interpret(source_number, sources[source_number], case["frontier"])


def evaluate_dotted_position_case(
    sources: Sequence[bytes], case: dict[str, Any]
) -> dict[str, object]:
    if sorted(case) != ["boundary_index", "bundle", "expect", "id", "kind", "source"]:
        raise IndependentDiagnosticError("dotted-position probe vocabulary changed")
    source_number = case["source"]
    if type(source_number) is not int or source_number not in range(len(sources)):
        raise IndependentDiagnosticError("dotted-position source number is invalid")
    index = case["boundary_index"]
    if type(index) is not int or index not in range(4):
        raise IndependentDiagnosticError("dotted-position boundary member is invalid")
    source = sources[source_number]
    if not source or source[-1:] not in (b"(", b"<") or source.count(b".") != 1:
        raise IndependentDiagnosticError("dotted-position source lacks one bounded call window")
    left, right_with_open = source[:-1].split(b".")
    try:
        left_word = left.decode("ascii")
        right_word = right_with_open.decode("ascii")
    except UnicodeDecodeError as error:
        raise IndependentDiagnosticError("dotted-position words are not ASCII") from error
    ident_pattern = r"[a-z][a-z0-9_]*"
    if (
        re.fullmatch(ident_pattern, left_word) is None
        or re.fullmatch(ident_pattern, right_word) is None
        or left_word in FIXED_WORDS
        or right_word in FIXED_WORDS
    ):
        raise IndependentDiagnosticError("dotted-position window words are not IDENTs")
    left_end = len(left)
    right_end = len(source) - 1
    spans = [[0, left_end], [left_end, left_end + 1], [left_end + 1, right_end], [right_end, len(source)]]
    return {
        "attribution": _outcome("FORM-3", source_number, [0, right_end], [";"]),
        "boundary_index": index,
        "boundary_span": spans[index],
        "family": "dotted-position-evidence",
    }


def evaluate_atom_shape_case(
    sources: Sequence[bytes], case: dict[str, Any]
) -> dict[str, object]:
    if sorted(case) != ["bundle", "expect", "id", "kind", "slot", "source", "tokens"]:
        raise IndependentDiagnosticError("atom-shape probe vocabulary changed")
    source_number = case["source"]
    if type(source_number) is not int or source_number < 0 or source_number >= len(sources):
        raise IndependentDiagnosticError("atom-shape probe source ordinal is invalid")
    if case["slot"] not in ATOM_OCCURRENCES:
        raise IndependentDiagnosticError("atom-shape probe occurrence changed")
    raw_tokens = case["tokens"]
    if not isinstance(raw_tokens, list) or len(raw_tokens) != 2:
        raise IndependentDiagnosticError("atom-shape probe lookahead is not two tokens")
    tokens = [_checked_token(raw, sources[source_number]) for raw in raw_tokens]
    forbidden = any(_matches(name, tokens[0]) for name in ("IDENT", "OPNAME", "TYPEID")) and tokens[1]["spelling"] in ("(", "<")
    return {
        "family": "diagnostic-shape-probe",
        "forbidden_call_or_construct_start": forbidden,
        "slot": case["slot"],
    }
