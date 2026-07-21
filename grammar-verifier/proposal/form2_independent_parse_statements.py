from __future__ import annotations

from form2_independent_syntax import IndependentNode


class IndependentStatementParserMixin:
    def _parse_stmt(self) -> IndependentNode:
        start = self.cursor
        raw = self._raw()
        if raw == b"let":
            child = self._parse_let_stmt()
        elif raw == b"set":
            child = self._parse_set_stmt()
        elif raw == b"return":
            child = self._parse_return_stmt()
        elif raw == b"loop":
            child = self._parse_loop_stmt()
        elif raw == b"break":
            child = self._parse_break_stmt()
        elif raw == b"region":
            child = self._parse_region_stmt()
        elif raw == b"check":
            child = self._parse_check_stmt()
        elif raw == b"match":
            child = self._parse_match_stmt()
        elif raw == b"give":
            child = self._parse_give_stmt()
        elif self._kind() in ("IDENT", "OPNAME"):
            child = self._parse_expr_stmt()
        else:
            raise self._error("expected statement")
        return self._node("stmt", start, [child])

    def _parse_typed_let_head(self) -> list[IndependentNode]:
        self._take_raw(b"let")
        self._take_kind("IDENT")
        self._take_raw(b":")
        children = [self._parse_mode(), self._parse_type()]
        self._take_raw(b"=")
        return children

    def _parse_let_stmt(self) -> IndependentNode:
        start = self.cursor
        children = self._parse_typed_let_head()
        if self._raw() == b"match":
            children.append(self._parse_value_match())
        elif self._raw() == b"try":
            children.append(self._parse_try_let_rhs())
        else:
            children.append(self._parse_ordinary_let_rhs())
        return self._node("let_stmt", start, children)

    def _parse_ordinary_let_rhs(self) -> IndependentNode:
        start = self.cursor
        child = self._parse_expr()
        self._take_raw(b";")
        return self._node("ordinary_let_rhs", start, [child])

    def _parse_try_let_rhs(self) -> IndependentNode:
        start = self.cursor
        self._take_raw(b"try")
        child = self._parse_expr()
        self._take_raw(b";")
        return self._node("try_let_rhs", start, [child])

    def _parse_set_stmt(self) -> IndependentNode:
        start = self.cursor
        self._take_raw(b"set")
        children = [self._parse_place()]
        self._take_raw(b"=")
        children.append(self._parse_expr())
        self._take_raw(b";")
        return self._node("set_stmt", start, children)

    def _parse_expr_stmt(self) -> IndependentNode:
        start = self.cursor
        child = self._parse_call()
        self._take_raw(b";")
        return self._node("expr_stmt", start, [child])

    def _parse_return_stmt(self) -> IndependentNode:
        start = self.cursor
        self._take_raw(b"return")
        child = self._parse_expr()
        self._take_raw(b";")
        return self._node("return_stmt", start, [child])

    def _parse_loop_stmt(self) -> IndependentNode:
        start = self.cursor
        self._take_raw(b"loop")
        self._take_kind("LABEL")
        self._take_raw(b"{")
        children: list[IndependentNode] = []
        while self._raw() != b"}":
            children.append(self._parse_stmt())
        self._take_raw(b"}")
        return self._node("loop_stmt", start, children)

    def _parse_break_stmt(self) -> IndependentNode:
        start = self.cursor
        self._take_raw(b"break")
        self._take_kind("LABEL")
        self._take_raw(b";")
        return self._node("break_stmt", start, [])

    def _parse_region_stmt(self) -> IndependentNode:
        start = self.cursor
        self._take_raw(b"region")
        self._take_kind("REGIONID")
        self._take_raw(b"{")
        children: list[IndependentNode] = []
        while self._raw() != b"}":
            children.append(self._parse_stmt())
        self._take_raw(b"}")
        return self._node("region_stmt", start, children)

    def _parse_check_stmt(self) -> IndependentNode:
        start = self.cursor
        self._take_raw(b"check")
        child = self._parse_expr()
        self._take_raw(b"else")
        self._take_raw(b"trap")
        self._take_kind("STRING")
        self._take_raw(b";")
        return self._node("check_stmt", start, [child])

    def _parse_give_stmt(self) -> IndependentNode:
        start = self.cursor
        self._take_raw(b"give")
        child = self._parse_expr()
        self._take_raw(b";")
        return self._node("give_stmt", start, [child])

    def _parse_match_stmt(self) -> IndependentNode:
        start = self.cursor
        self._take_raw(b"match")
        children = [self._parse_expr()]
        self._take_raw(b"{")
        children.append(self._parse_arm())
        while self._kind() == "TYPEID":
            children.append(self._parse_arm())
        self._take_raw(b"}")
        return self._node("match_stmt", start, children)

    def _parse_value_match(self) -> IndependentNode:
        start = self.cursor
        self._take_raw(b"match")
        children = [self._parse_expr()]
        self._take_raw(b"{")
        children.append(self._parse_arm())
        while self._kind() == "TYPEID":
            children.append(self._parse_arm())
        self._take_raw(b"}")
        return self._node("value_match", start, children)

    def _parse_arm(self) -> IndependentNode:
        start = self.cursor
        self._take_kind("TYPEID")
        self._take_raw(b"(")
        children: list[IndependentNode] = []
        if self._raw() != b")":
            children.append(self._parse_fieldbind_list())
        self._take_raw(b")")
        self._take_raw(b"=>")
        self._take_raw(b"{")
        while self._raw() != b"}":
            children.append(self._parse_stmt())
        self._take_raw(b"}")
        return self._node("arm", start, children)

    def _parse_fieldbind_list(self) -> IndependentNode:
        start = self.cursor
        children = [self._parse_fieldbind()]
        while self._raw() == b",":
            self._take_raw(b",")
            children.append(self._parse_fieldbind())
        return self._node("fieldbind_list", start, children)

    def _parse_fieldbind(self) -> IndependentNode:
        start = self.cursor
        self._take_kind("IDENT")
        self._take_raw(b":")
        self._take_kind("IDENT")
        return self._node("fieldbind", start, [])
