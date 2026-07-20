#!/usr/bin/env python3
"""Bounded v0.8 audit of fixed grammar terminals versus IDENT."""

from __future__ import annotations

import hashlib
import re
from dataclasses import dataclass
from typing import Any, Sequence


IDENT = re.compile(rb"[a-z][a-z0-9_]*")
TYPE_ID = re.compile(rb"[A-Z][A-Za-z0-9]*")
REGION_ID = re.compile(rb"'[a-z][a-z0-9_]*")
OP_NAME = re.compile(rb"[a-z][a-z0-9_]*\.(wrap|trap|checked|sat|strict)")
QUOTED_LOWER_TERMINAL = re.compile(rb'"([a-z][a-z0-9_]*)"')

REQUIRED_PRODUCTION_SHAPES = {
    "production:GRAM-3:type": (
        b'type := "i8"|"i16"|"i32"|"i64"|"u8"|"u16"|"u32"|"u64"|'
        b'"f32"|"f64"|"unit" | TYPEID targs? | "array" "<" type "," const '
        b'">" | "slice" "<" REGIONID "," type ">" | "box" "<" type ">" | '
        b'"arena" "<" REGIONID "," type ">" | "buffer" "<" type ">"'
    ),
    "production:GRAM-3:targs": b'targs := "<" targ ("," targ)* ">"',
    "production:GRAM-3:targ": b"targ := type | REGIONID | const",
    "production:GRAM-3:const": b'const := "[0-9]+" | IDENT',
    "production:GRAM-3:cvalue": (
        b'cvalue := literal | IDENT | "[" cvalue ("," cvalue)* "]"'
    ),
    "production:GRAM-5:expr": b"expr := atom | call | construct",
    "production:GRAM-5:atom": (
        b'atom := literal | "move" place | place | borrow_expr'
    ),
    "production:GRAM-5:call": (
        b'call := callee targs? "(" ( atom_list | fieldinit_list )? ")"'
    ),
    "production:GRAM-5:callee": b"callee := IDENT | OPNAME",
    "production:GRAM-5:construct": (
        b'construct := TYPEID targs? "(" fieldinit_list? ")"'
    ),
    "production:GRAM-5:fieldinit_list": (
        b'fieldinit_list := fieldinit ("," fieldinit)*'
    ),
    "production:GRAM-5:fieldinit": b'fieldinit := IDENT ":" atom',
    "production:GRAM-5:borrow_expr": (
        b'borrow_expr := "&" REGIONID place | "&uniq" REGIONID place'
    ),
    "production:GRAM-5:atom_list": b'atom_list := atom ("," atom)*',
    "production:GRAM-5:place": b"place := pbase psuffix*",
    "production:GRAM-5:pbase": (
        b'pbase := IDENT | "deref" "(" place ")" '
        b'| "index" "<" type ">" "(" place "," atom ")"'
    ),
    "production:GRAM-5:psuffix": b'psuffix := "." IDENT',
}

EXPECTED_PRODUCTION_SHA256 = {
    "production:GRAM-3:type": (
        "eb8e907bbb7270682ca8dfda756933d356fd7f17d333c49dc9099725fb60ad58"
    ),
    "production:GRAM-3:targs": (
        "ea042669f6278458e76b7bbdab1b0f5489dad9672f99ef0a87263d4048fa42cf"
    ),
    "production:GRAM-3:targ": (
        "ee2ff41e9976603932679d73422ded3412a83cbc273599d0a931af46e5a8328d"
    ),
    "production:GRAM-3:const": (
        "7f10580fec0ea9c8c10ef9ec932daf33c49c99b04078c368c2b5cb40463e5e8f"
    ),
    "production:GRAM-3:cvalue": (
        "ff6abe7175e59b7cb9b0623f727b02d1e0949c766b0293871e86fd1d0a2706f1"
    ),
    "production:GRAM-5:expr": (
        "30d222ef9603fd5d0aa81e0515ce1f1b9df1f1dea2780da697bddc60647695b3"
    ),
    "production:GRAM-5:atom": (
        "d5f4da123e63126382aa144dfe92b918b9310d84b714667d28e1848e433ed9e6"
    ),
    "production:GRAM-5:call": (
        "06bd6a04c3e1faeef87c41863fc3c1e58a668147397b46e8c25c4277c32eaf0b"
    ),
    "production:GRAM-5:callee": (
        "ff032dff50a091b9bf4215725efeb5febec7699dd2f9f6beb382e2e0fd52758c"
    ),
    "production:GRAM-5:construct": (
        "ebb68f287d883ffc453c2abe79af07017a65b5467878f7492e1e2bf7d05be39e"
    ),
    "production:GRAM-5:fieldinit_list": (
        "88e9d4ff25bb61a81629c88e480df627236256566cb2577c49fcaf683c555be6"
    ),
    "production:GRAM-5:fieldinit": (
        "0c5c9b99b2c6c988f53bfb21d2e326a47028d2f1f2d9a9ef0e0f54834ed5f0bf"
    ),
    "production:GRAM-5:borrow_expr": (
        "96863547c90b53bf254706c4ced2ee644a6894a9225e24806923536785935fd5"
    ),
    "production:GRAM-5:atom_list": (
        "f905b6c93d16eade935b53a25d28c7b345766f7b89a4c9ef92fc6c906445f01c"
    ),
    "production:GRAM-5:place": (
        "2848a8fe9ef2cba32ba4b5155c198b4da1aa3d21032c2dcd18f5437f2e166ae2"
    ),
    "production:GRAM-5:pbase": (
        "95fd98f31347793769d8ed1ff0950acf70199a9d6507e4463a59dc075e7a3f0b"
    ),
    "production:GRAM-5:psuffix": (
        "f49077b55c1dceac8a465bec2c1e926ff5eaf92d18c6fd0e874605f3c175ce32"
    ),
}


class GrammarAuditError(ValueError):
    """The supplied grammar cannot support the pinned audit."""


@dataclass(frozen=True)
class Production:
    """One already source-verified syntax production."""

    identifier: str
    fragment: bytes
    sha256: str


@dataclass(frozen=True)
class Audit:
    """Deterministic terminal census and exact competing derivations."""

    ambiguous_spellings: tuple[bytes, ...]
    competing_derivations: tuple[dict[str, Any], ...]
    derivation_count_witnesses: tuple[dict[str, Any], ...]
    fixed_terminals: tuple[bytes, ...]
    fixed_terminal_occurrence_count: int
    ident_matching_fixed_terminals: tuple[bytes, ...]
    place_call_ambiguous_spellings: tuple[bytes, ...]
    production_sha256: tuple[tuple[str, str], ...]
    production_terminal_map_row_count: int
    production_terminal_map_sha256: str
    single_derivation_boundaries: tuple[dict[str, Any], ...]
    syntax_production_count: int
    type_argument_ambiguous_spellings: tuple[bytes, ...]

    def pin_payload(self) -> dict[str, Any]:
        """Return the minimal exact-v0.8 values that must not drift."""
        terminal_inventory = b"".join(word + b"\n" for word in self.fixed_terminals)
        return {
            "ambiguous_spellings": _text(self.ambiguous_spellings),
            "fixed_terminal_count": len(self.fixed_terminals),
            "fixed_terminal_inventory_sha256": _sha256(terminal_inventory),
            "fixed_terminal_occurrence_count": self.fixed_terminal_occurrence_count,
            "ident_matching_terminal_count": len(
                self.ident_matching_fixed_terminals
            ),
            "place_call_ambiguous_spellings": _text(
                self.place_call_ambiguous_spellings
            ),
            "production_sha256": dict(self.production_sha256),
            "production_terminal_map_row_count": (
                self.production_terminal_map_row_count
            ),
            "production_terminal_map_sha256": self.production_terminal_map_sha256,
            "syntax_production_count": self.syntax_production_count,
            "type_argument_ambiguous_spellings": _text(
                self.type_argument_ambiguous_spellings
            ),
        }

    def evidence(self) -> dict[str, Any]:
        """Return the non-authorizing JSON evidence projection."""
        pin = self.pin_payload()
        pin.pop("production_sha256")
        return {
            **pin,
            "competing_derivations": list(self.competing_derivations),
            "derivation_count_witnesses": list(
                self.derivation_count_witnesses
            ),
            "fixed_terminals": _text(self.fixed_terminals),
            "ident_matching_fixed_terminals": _text(
                self.ident_matching_fixed_terminals
            ),
            "single_derivation_boundaries": list(
                self.single_derivation_boundaries
            ),
        }


def _sha256(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def _text(words: Sequence[bytes]) -> list[str]:
    return [word.decode("ascii") for word in words]


def _grammar_shape(fragment: bytes) -> bytes:
    uncommented = (line.split(b"#", 1)[0] for line in fragment.splitlines())
    return b" ".join(b" ".join(uncommented).split())


def _counted_witness(
    spelling: str,
    place_derivations: int,
    call_derivations: int,
) -> dict[str, Any]:
    return {
        "call_derivations": call_derivations,
        "place_derivations": place_derivations,
        "spelling": spelling,
        "total_derivations": place_derivations + call_derivations,
    }


def audit(productions: Sequence[Production]) -> Audit:
    """Derive the overlap and its complete v0.8 witness census."""
    if len(productions) != 59:
        raise GrammarAuditError("terminal audit requires exactly 59 productions")
    by_identifier: dict[str, Production] = {}
    terminal_occurrences: list[bytes] = []
    terminal_rows: list[bytes] = []
    for production in productions:
        if production.identifier in by_identifier:
            raise GrammarAuditError(
                f"duplicate syntax production: {production.identifier}"
            )
        by_identifier[production.identifier] = production
        terminals = QUOTED_LOWER_TERMINAL.findall(production.fragment)
        terminal_occurrences.extend(terminals)
        if terminals:
            terminal_rows.append(
                production.identifier.encode("ascii")
                + b"\t"
                + b",".join(terminals)
                + b"\n"
            )

    missing = sorted(set(REQUIRED_PRODUCTION_SHAPES) - set(by_identifier))
    if missing:
        raise GrammarAuditError(f"terminal audit is missing productions: {missing}")
    for identifier, expected in REQUIRED_PRODUCTION_SHAPES.items():
        if _grammar_shape(by_identifier[identifier].fragment) != expected:
            raise GrammarAuditError(
                f"terminal audit production shape drifted: {identifier}"
            )

    fixed_terminals = tuple(sorted(set(terminal_occurrences)))
    ident_matching = tuple(
        word for word in fixed_terminals if IDENT.fullmatch(word)
    )
    type_fragment = by_identifier["production:GRAM-3:type"].fragment
    type_terminals = set(QUOTED_LOWER_TERMINAL.findall(type_fragment))
    type_constructors = set(
        re.findall(rb'"([a-z][a-z0-9_]*)"\s+"<"', type_fragment)
    )
    type_ambiguous = tuple(
        sorted(
            word
            for word in type_terminals - type_constructors
            if IDENT.fullmatch(word)
        )
    )
    pbase_fragment = by_identifier["production:GRAM-5:pbase"].fragment
    place_call_ambiguous = tuple(
        sorted(
            word
            for word in set(QUOTED_LOWER_TERMINAL.findall(pbase_fragment))
            if IDENT.fullmatch(word)
        )
    )
    ambiguous = tuple(sorted(set(type_ambiguous) | set(place_call_ambiguous)))
    if not set(ambiguous) <= set(ident_matching):
        raise GrammarAuditError("ambiguous terminals left the IDENT overlap")
    if b"unit" not in type_ambiguous:
        raise GrammarAuditError("terminal audit lost the unit type ambiguity")
    if any(
        TYPE_ID.fullmatch(word) is not None or OP_NAME.fullmatch(word) is not None
        for word in ambiguous
    ):
        raise GrammarAuditError("terminal audit lexical classes overlap unexpectedly")

    plain_types = set(type_ambiguous)

    def type_count(word: bytes) -> int:
        return int(word in plain_types) + int(TYPE_ID.fullmatch(word) is not None)

    def targ_count(word: bytes) -> int:
        return (
            type_count(word)
            + int(REGION_ID.fullmatch(word) is not None)
            + int(IDENT.fullmatch(word) is not None)
        )

    def place_count(word: bytes) -> int:
        return int(IDENT.fullmatch(word) is not None)

    def atom_count(word: bytes) -> int:
        return place_count(word) + int(word == b"unit")

    def callee_count(word: bytes) -> int:
        return int(IDENT.fullmatch(word) is not None) + int(
            OP_NAME.fullmatch(word) is not None
        )

    def deref_counts(argument: bytes) -> tuple[int, int]:
        return place_count(argument), callee_count(b"deref") * atom_count(argument)

    def index_counts(
        type_argument: bytes,
        base: bytes,
        offset: bytes,
    ) -> tuple[int, int]:
        place_derivations = (
            type_count(type_argument) * place_count(base) * atom_count(offset)
        )
        call_derivations = (
            callee_count(b"index")
            * targ_count(type_argument)
            * atom_count(base)
            * atom_count(offset)
        )
        return place_derivations, call_derivations

    counted_inputs = (
        ("deref(p)", deref_counts(b"p")),
        ("index<T>(p, q)", index_counts(b"T", b"p", b"q")),
        ("deref(unit)", deref_counts(b"unit")),
        ("index<i32>(p, q)", index_counts(b"i32", b"p", b"q")),
        ("index<unit>(p, unit)", index_counts(b"unit", b"p", b"unit")),
    )
    counted_witnesses = tuple(
        _counted_witness(spelling, counts[0], counts[1])
        for spelling, counts in counted_inputs
    )
    boundaries = (
        _counted_witness("deref(p).field", place_count(b"p"), 0),
        _counted_witness(
            "deref<T>(p)",
            0,
            callee_count(b"deref") * targ_count(b"T") * atom_count(b"p"),
        ),
        _counted_witness(
            "index<T>(p, q).field",
            type_count(b"T") * place_count(b"p") * atom_count(b"q"),
            0,
        ),
    )
    competing: list[dict[str, Any]] = [
        {
            "context": "type-argument-primitive-or-unit",
            "derivations": ["targ/type", "targ/const/IDENT"],
            "spelling_count": len(type_ambiguous),
            "spellings": _text(type_ambiguous),
        },
        {
            "context": "constant-value-unit",
            "derivations": ["cvalue/literal/unit", "cvalue/IDENT"],
            "spelling": "unit",
        },
        {
            "context": "expression-unit",
            "derivations": ["expr/atom/literal/unit", "expr/atom/place/IDENT"],
            "spelling": "unit",
        },
    ]
    place_paths = {
        b"deref": ("expression-deref", "deref(p)", "expr/atom/place/deref"),
        b"index": (
            "expression-index",
            "index<T>(p, i)",
            "expr/atom/place/index",
        ),
    }
    for word in place_call_ambiguous:
        if word not in place_paths:
            raise GrammarAuditError("unmodeled place-versus-call ambiguity")
        context, spelling, place_path = place_paths[word]
        competing.append(
            {
                "context": context,
                "derivations": [place_path, "expr/call/IDENT"],
                "spelling": spelling,
            }
        )

    required_hashes = tuple(
        (identifier, by_identifier[identifier].sha256)
        for identifier in REQUIRED_PRODUCTION_SHAPES
    )
    terminal_map = b"".join(terminal_rows)
    return Audit(
        ambiguous,
        tuple(competing),
        counted_witnesses,
        fixed_terminals,
        len(terminal_occurrences),
        ident_matching,
        place_call_ambiguous,
        required_hashes,
        len(terminal_rows),
        _sha256(terminal_map),
        boundaries,
        len(productions),
        type_ambiguous,
    )


EXPECTED_PIN_PAYLOAD = {
    "ambiguous_spellings": [
        "deref",
        "f32",
        "f64",
        "i16",
        "i32",
        "i64",
        "i8",
        "index",
        "u16",
        "u32",
        "u64",
        "u8",
        "unit",
    ],
    "fixed_terminal_count": 47,
    "fixed_terminal_inventory_sha256": (
        "5e437fbb7371fbaf00be9f341264c6414de6a23ca13877fe28f43a1644d9e376"
    ),
    "fixed_terminal_occurrence_count": 51,
    "ident_matching_terminal_count": 47,
    "place_call_ambiguous_spellings": ["deref", "index"],
    "production_sha256": EXPECTED_PRODUCTION_SHA256,
    "production_terminal_map_row_count": 27,
    "production_terminal_map_sha256": (
        "35150331a150a188564f4073898aa35c53bd51df77f9b55222b8a68b38526145"
    ),
    "syntax_production_count": 59,
    "type_argument_ambiguous_spellings": [
        "f32",
        "f64",
        "i16",
        "i32",
        "i64",
        "i8",
        "u16",
        "u32",
        "u64",
        "u8",
        "unit",
    ],
}
