from __future__ import annotations

from form2_independent_lex import satisfies_literal
from form2_independent_syntax import IndependentNode


class IndependentExpressionParserMixin:
    def _parse_expr(self) -> IndependentNode:
        self._enter()
        try:
            start = self.cursor
            if self._kind() == "TYPEID":
                child = self._parse_construct()
            elif self._kind() == "OPNAME":
                child = self._parse_call()
            elif self._kind() == "IDENT" and self._raw(1) in (b"(", b"<"):
                child = self._parse_call()
            else:
                child = self._parse_atom()
            return self._node("expr", start, [child])
        finally:
            self._leave()

    def _parse_atom(self) -> IndependentNode:
        start = self.cursor
        children: list[IndependentNode] = []
        token = self._peek()
        if token is not None and satisfies_literal(token):
            self._take_literal()
        elif self._raw() == b"move":
            self._take_raw(b"move")
            children.append(self._parse_place())
        elif self._raw() == b"&":
            children.append(self._parse_borrow_expr())
        else:
            children.append(self._parse_place())
        return self._node("atom", start, children)

    def _take_literal(self) -> None:
        token = self._peek()
        if token is None or not satisfies_literal(token):
            raise self._error("expected literal")
        self.cursor += 1

    def _parse_call(self) -> IndependentNode:
        start = self.cursor
        children = [self._parse_callee()]
        if self._raw() == b"<":
            children.append(self._parse_targs())
        self._take_raw(b"(")
        if self._raw() != b")":
            if self._kind() == "IDENT" and self._raw(1) == b":":
                children.append(self._parse_fieldinit_list())
            else:
                children.append(self._parse_atom_list())
        self._take_raw(b")")
        return self._node("call", start, children)

    def _parse_callee(self) -> IndependentNode:
        start = self.cursor
        if self._kind() not in ("IDENT", "OPNAME"):
            raise self._error("expected call callee")
        self.cursor += 1
        return self._node("callee", start, [])

    def _parse_atom_list(self) -> IndependentNode:
        start = self.cursor
        children = [self._parse_atom()]
        while self._raw() == b",":
            self._take_raw(b",")
            children.append(self._parse_atom())
        return self._node("atom_list", start, children)

    def _parse_construct(self) -> IndependentNode:
        start = self.cursor
        self._take_kind("TYPEID")
        children: list[IndependentNode] = []
        if self._raw() == b"<":
            children.append(self._parse_targs())
        self._take_raw(b"(")
        if self._raw() != b")":
            children.append(self._parse_fieldinit_list())
        self._take_raw(b")")
        return self._node("construct", start, children)

    def _parse_fieldinit_list(self) -> IndependentNode:
        start = self.cursor
        children = [self._parse_fieldinit()]
        while self._raw() == b",":
            self._take_raw(b",")
            children.append(self._parse_fieldinit())
        return self._node("fieldinit_list", start, children)

    def _parse_fieldinit(self) -> IndependentNode:
        start = self.cursor
        self._take_kind("IDENT")
        self._take_raw(b":")
        child = self._parse_atom()
        return self._node("fieldinit", start, [child])

    def _parse_borrow_expr(self) -> IndependentNode:
        start = self.cursor
        self._take_raw(b"&")
        if self._raw() == b"uniq":
            self._take_raw(b"uniq")
        self._take_kind("REGIONID")
        child = self._parse_place()
        return self._node("borrow_expr", start, [child])

    def _parse_place(self) -> IndependentNode:
        self._enter()
        try:
            start = self.cursor
            children = [self._parse_pbase()]
            while self._raw() == b".":
                children.append(self._parse_psuffix())
            return self._node("place", start, children)
        finally:
            self._leave()

    def _parse_pbase(self) -> IndependentNode:
        start = self.cursor
        children: list[IndependentNode] = []
        if self._kind() == "IDENT":
            self._take_kind("IDENT")
        elif self._raw() == b"deref":
            self._take_raw(b"deref")
            self._take_raw(b"(")
            children.append(self._parse_place())
            self._take_raw(b")")
        elif self._raw() == b"index":
            self._take_raw(b"index")
            self._take_raw(b"<")
            children.append(self._parse_type())
            self._take_raw(b">")
            self._take_raw(b"(")
            children.append(self._parse_place())
            self._take_raw(b",")
            children.append(self._parse_atom())
            self._take_raw(b")")
        else:
            raise self._error("expected place base")
        return self._node("pbase", start, children)

    def _parse_psuffix(self) -> IndependentNode:
        start = self.cursor
        self._take_raw(b".")
        self._take_kind("IDENT")
        return self._node("psuffix", start, [])
