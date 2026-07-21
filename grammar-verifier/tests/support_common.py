"""Canonical common-ledger fixtures for runner-boundary tests."""

from __future__ import annotations

import hashlib
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[1]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

from runner_inputs import BoundBytes, Inputs, parse_limits  # noqa: E402


IDENT_MODIFIER = (
    b" excluding every lowercase token spelling produced by exact fixed grammar atoms "
    b"in the complete grammar"
)


def _document(exclude_fixed_lowerwords: bool) -> bytes:
    ident = b"IDENT `[a-z][a-z0-9_]*`"
    if exclude_fixed_lowerwords:
        ident += IDENT_MODIFIER
    return b"".join(
        (
            b"[FORM-3] Lexical forms.\n",
            b"Lexical classes: ",
            ident,
            b"; TYPEID `[A-Z][A-Za-z0-9]*`.\n",
            b"[GRAM-1] Grammar.\n",
            b'call := IDENT "(" IDENT ")"\n',
            b'fixed_call := "deref" "(" IDENT ")"\n',
            b"expr := fixed_call | call\n",
        )
    )


def inputs() -> Inputs:
    limits_raw = (ROOT / "limits.txt").read_bytes()
    sections = (
        BoundBytes("limits", limits_raw),
        BoundBytes("current", _document(False)),
        BoundBytes("proposal", _document(True)),
        BoundBytes(
            "cases",
            b"whitefoot.grammar-cases.v1\n"
            b"case\tderef-p\texpr\t6465726566287029\n"
            b"case\tderef-x\texpr\t6465726566287829\n",
        ),
        BoundBytes(
            "domains",
            b"whitefoot.grammar-domains.v1\n"
            b"domain\tfixed-lowerword-calls\tfixed-lowerword-call\texpr\t78\n",
        ),
    )
    expectations = BoundBytes(
        "expectations",
        b"whitefoot.grammar-expectations.v2\n"
        b"case\tderef-p\tcurrent\tmany\n"
        b"case\tderef-p\tproposal\tone\n"
        b"case\tderef-x\tcurrent\tmany\n"
        b"case\tderef-x\tproposal\tone\n"
        b"case-delta\tderef-p\ttrace-subset\n"
        b"case-delta\tderef-x\ttrace-subset\n"
        b"transition\tfixed-ident-partition\tremoves-call-through-ident\n",
    )
    return Inputs(sections, expectations, parse_limits(limits_raw))


def common_lines(value: Inputs) -> list[bytes]:
    result = []
    for section in value.sections:
        result.append(
            b"\t".join(
                (
                    b"BIND",
                    section.name.encode("ascii"),
                    str(len(section.data)).encode("ascii"),
                    hashlib.sha256(section.data).hexdigest().encode("ascii"),
                )
            )
        )
    rank = {
        tag: index
        for index, tag in enumerate(
            (b"RULE", b"SURFACE", b"PROD", b"NODE", b"LEX", b"FIXED", b"REF", b"COVERAGE")
        )
    }
    for document in (b"current", b"proposal"):
        source = value.section(document.decode("ascii")).data
        records: list[bytes] = []
        grammar_rule = source.index(b"[GRAM-1]")
        records.extend(
            (
                b"\t".join(
                    (
                        b"RULE",
                        document,
                        b"FORM-3".hex().encode("ascii"),
                        b"0",
                        str(grammar_rule).encode("ascii"),
                    )
                ),
                b"\t".join(
                    (
                        b"RULE",
                        document,
                        b"GRAM-1".hex().encode("ascii"),
                        str(grammar_rule).encode("ascii"),
                        str(len(source)).encode("ascii"),
                    )
                ),
            )
        )
        production_spans: dict[bytes, tuple[int, int, int, int]] = {}
        for lhs in (b"call", b"fixed_call", b"expr"):
            definition_start = source.index(lhs + b" :=")
            definition_end = source.index(b"\n", definition_start)
            rhs_start = source.index(b":=", definition_start, definition_end) + 2
            while source[rhs_start] == 0x20:
                rhs_start += 1
            production_spans[lhs] = (
                definition_start,
                definition_end,
                rhs_start,
                definition_end,
            )
            records.append(
                b"\t".join(
                    (
                        b"PROD",
                        document,
                        b"GRAM-1".hex().encode("ascii"),
                        lhs.hex().encode("ascii"),
                        *(str(offset).encode("ascii") for offset in production_spans[lhs]),
                    )
                )
            )
        node_specs = {
            b"call": (
                (b"ref", b"IDENT"),
                (b"fixed", b"("),
                (b"ref", b"IDENT"),
                (b"fixed", b")"),
            ),
            b"fixed_call": (
                (b"fixed", b"deref"),
                (b"fixed", b"("),
                (b"ref", b"IDENT"),
                (b"fixed", b")"),
            ),
            b"expr": ((b"ref", b"fixed_call"), (b"ref", b"call")),
        }
        for lhs, leaves in node_specs.items():
            _definition_start, _definition_end, rhs_start, rhs_end = production_spans[lhs]
            root_kind = b"choice" if lhs == b"expr" else b"sequence"
            records.append(
                b"\t".join(
                    (
                        b"NODE",
                        document,
                        lhs.hex().encode("ascii"),
                        b"0",
                        root_kind,
                        str(rhs_start).encode("ascii"),
                        str(rhs_end).encode("ascii"),
                        b"-",
                    )
                )
            )
            cursor = rhs_start
            for index, (kind, spelling) in enumerate(leaves):
                source_spelling = b'"' + spelling + b'"' if kind == b"fixed" else spelling
                start = source.index(source_spelling, cursor, rhs_end)
                end = start + len(source_spelling)
                cursor = end
                path = b"0." + str(index).encode("ascii")
                node_value = spelling.hex().encode("ascii")
                records.append(
                    b"\t".join(
                        (
                            b"NODE",
                            document,
                            lhs.hex().encode("ascii"),
                            path,
                            kind,
                            str(start).encode("ascii"),
                            str(end).encode("ascii"),
                            node_value,
                        )
                    )
                )
                if kind == b"ref":
                    records.append(
                        b"\t".join(
                            (
                                b"REF",
                                document,
                                lhs.hex().encode("ascii"),
                                path,
                                str(start).encode("ascii"),
                                str(end).encode("ascii"),
                                node_value,
                            )
                        )
                    )
                else:
                    atom_kind = b"lowerword" if spelling == b"deref" else b"punctuation"
                    expansion = atom_kind + b":" + node_value
                    records.append(
                        b"\t".join(
                            (
                                b"FIXED",
                                document,
                                lhs.hex().encode("ascii"),
                                path,
                                str(start).encode("ascii"),
                                str(end).encode("ascii"),
                                node_value,
                                expansion.hex().encode("ascii"),
                            )
                        )
                    )
        class_cue_start = source.index(b"Lexical classes:")
        records.append(
            b"\t".join(
                (
                    b"SURFACE",
                    document,
                    b"lexical-cue",
                    str(class_cue_start).encode("ascii"),
                    str(class_cue_start + len(b"Lexical classes:")).encode("ascii"),
                    b"FORM-3".hex().encode("ascii"),
                )
            )
        )
        for lhs in (b"call", b"fixed_call", b"expr"):
            assignment = source.index(b":=", production_spans[lhs][0], production_spans[lhs][1])
            records.append(
                b"\t".join(
                    (
                        b"SURFACE",
                        document,
                        b"assignment",
                        str(assignment).encode("ascii"),
                        str(assignment + 2).encode("ascii"),
                        b"GRAM-1".hex().encode("ascii"),
                    )
                )
            )
        lexical_specs = (
            (
                b"IDENT",
                b"[a-z][a-z0-9_]*",
                b"fixed-lowerwords" if document == b"proposal" else b"none",
            ),
            (b"TYPEID", b"[A-Z][A-Za-z0-9]*", b"none"),
        )
        for name, pattern, exclusion in lexical_specs:
            source_form = name + b" `" + pattern + b"`"
            if exclusion == b"fixed-lowerwords":
                source_form += IDENT_MODIFIER
            start = source.index(source_form)
            end = start + len(source_form)
            predicate = b"pattern=" + pattern + b";exclude=" + exclusion
            records.append(
                b"\t".join(
                    (
                        b"LEX",
                        document,
                        b"FORM-3".hex().encode("ascii"),
                        name.hex().encode("ascii"),
                        b"regex",
                        str(start).encode("ascii"),
                        str(end).encode("ascii"),
                        predicate.hex().encode("ascii"),
                    )
                )
            )

        def order_key(line: bytes) -> tuple[int, int, bytes]:
            fields = line.split(b"\t")
            tag = fields[0]
            primary_index = {
                b"RULE": 3,
                b"SURFACE": 3,
                b"PROD": 4,
                b"NODE": 5,
                b"LEX": 5,
                b"FIXED": 4,
                b"REF": 4,
            }[tag]
            return int(fields[primary_index]), rank[tag], line

        records.sort(key=order_key)
        result.extend(records)
        result.append(b"COVERAGE\t" + document + b"\t3\t0\t0\t1\t0")
    return result
