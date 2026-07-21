from __future__ import annotations

import re

from form2_independent_syntax import IndependentNode


PRIMITIVES = frozenset(
    {b"i8", b"i16", b"i32", b"i64", b"u8", b"u16", b"u32", b"u64", b"f32", b"f64", b"unit"}
)
TYPE_STARTERS = PRIMITIVES | frozenset({b"array", b"slice", b"box", b"arena", b"buffer"})
EFFECT_STARTERS = frozenset({b"reads", b"writes", b"allocates", b"traps"})


class IndependentTypeParserMixin:
    def _parse_type(self) -> IndependentNode:
        self._enter()
        try:
            start = self.cursor
            raw = self._raw()
            children: list[IndependentNode] = []
            if raw in PRIMITIVES:
                self.cursor += 1
            elif self._kind() == "TYPEID":
                self._take_kind("TYPEID")
                if self._raw() == b"<":
                    children.append(self._parse_targs())
            elif raw == b"array":
                self._take_raw(b"array")
                self._take_raw(b"<")
                children.append(self._parse_type())
                self._take_raw(b",")
                children.append(self._parse_const())
                self._take_raw(b">")
            elif raw == b"slice":
                self._take_raw(b"slice")
                self._take_raw(b"<")
                self._take_kind("REGIONID")
                self._take_raw(b",")
                children.append(self._parse_type())
                self._take_raw(b">")
            elif raw in (b"box", b"buffer"):
                self.cursor += 1
                self._take_raw(b"<")
                children.append(self._parse_type())
                self._take_raw(b">")
            elif raw == b"arena":
                self._take_raw(b"arena")
                self._take_raw(b"<")
                self._take_kind("REGIONID")
                self._take_raw(b",")
                children.append(self._parse_type())
                self._take_raw(b">")
            else:
                raise self._error("expected type")
            return self._node("type", start, children)
        finally:
            self._leave()

    def _parse_rtype(self) -> IndependentNode:
        start = self.cursor
        children = [self._parse_mode(), self._parse_type()]
        return self._node("rtype", start, children)

    def _parse_mode(self) -> IndependentNode:
        start = self.cursor
        if self._raw() == b"own":
            self._take_raw(b"own")
        elif self._raw() == b"&":
            self._take_raw(b"&")
            if self._raw() == b"uniq":
                self._take_raw(b"uniq")
            self._take_kind("REGIONID")
        else:
            raise self._error("expected ownership mode")
        return self._node("mode", start, [])

    def _parse_targs(self) -> IndependentNode:
        start = self.cursor
        self._take_raw(b"<")
        children = [self._parse_targ()]
        while self._raw() == b",":
            self._take_raw(b",")
            children.append(self._parse_targ())
        self._take_raw(b">")
        return self._node("targs", start, children)

    def _parse_targ(self) -> IndependentNode:
        start = self.cursor
        if self._kind() == "REGIONID":
            self._take_kind("REGIONID")
            return self._node("targ", start, [])
        if self._kind() == "IDENT" or (
            self._kind() == "NUMBER" and re.fullmatch(rb"[0-9]+", self._raw() or b"")
        ):
            return self._node("targ", start, [self._parse_const()])
        if self._kind() == "TYPEID" or self._raw() in TYPE_STARTERS:
            return self._node("targ", start, [self._parse_type()])
        raise self._error("expected type, region, or constant argument")

    def _parse_const(self) -> IndependentNode:
        start = self.cursor
        if self._kind() == "IDENT":
            self._take_kind("IDENT")
        elif self._kind() == "NUMBER" and re.fullmatch(rb"[0-9]+", self._raw() or b""):
            self.cursor += 1
        else:
            raise self._error("expected constant expression")
        return self._node("const", start, [])

    def _parse_cvalue(self) -> IndependentNode:
        start = self.cursor
        children: list[IndependentNode] = []
        if self._raw() == b"[":
            self._take_raw(b"[")
            children.append(self._parse_cvalue())
            while self._raw() == b",":
                self._take_raw(b",")
                children.append(self._parse_cvalue())
            self._take_raw(b"]")
        elif self._kind() == "IDENT":
            self._take_kind("IDENT")
        else:
            self._take_literal()
        return self._node("cvalue", start, children)

    def _parse_effects(self) -> IndependentNode:
        start = self.cursor
        children: list[IndependentNode] = []
        if self._raw() == b"pure":
            self._take_raw(b"pure")
        else:
            children.append(self._parse_effect())
            while self._raw() == b",":
                self._take_raw(b",")
                children.append(self._parse_effect())
        return self._node("effects", start, children)

    def _parse_effect(self) -> IndependentNode:
        start = self.cursor
        raw = self._raw()
        if raw not in EFFECT_STARTERS:
            raise self._error("expected effect row")
        self.cursor += 1
        if raw in (b"reads", b"writes"):
            self._take_raw(b"(")
            self._take_kind("REGIONID")
            while self._kind() == "REGIONID":
                self._take_kind("REGIONID")
            self._take_raw(b")")
        elif raw == b"allocates":
            self._take_raw(b"(")
            count = 0
            while self._raw() in (b"heap", b"arena"):
                if self._raw() == b"heap":
                    self._take_raw(b"heap")
                else:
                    self._take_raw(b"arena")
                    self._take_kind("REGIONID")
                count += 1
            if count == 0:
                raise self._error("allocates effect requires a target")
            self._take_raw(b")")
        return self._node("effect", start, [])
