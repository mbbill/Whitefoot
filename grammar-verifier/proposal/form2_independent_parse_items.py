from __future__ import annotations

from form2_independent_syntax import IndependentForest, IndependentNode


ITEM_STARTERS = frozenset({b"fn", b"struct", b"enum", b"contract", b"conform", b"const"})


class IndependentItemParserMixin:
    def parse_item_forest(self) -> IndependentForest:
        children: list[IndependentNode] = []
        while self.cursor < len(self.tokens):
            if self._raw() not in ITEM_STARTERS:
                raise self._error("expected a top-level item")
            children.append(self._parse_item())
        return IndependentForest(
            tuple(children), len(self.tokens), self.source_length
        )

    def _parse_item(self) -> IndependentNode:
        start = self.cursor
        raw = self._raw()
        if raw == b"fn":
            child = self._parse_fn_decl()
        elif raw == b"struct":
            child = self._parse_struct_decl()
        elif raw == b"enum":
            child = self._parse_enum_decl()
        elif raw == b"contract":
            child = self._parse_contract_decl()
        elif raw == b"conform":
            child = self._parse_conform_decl()
        elif raw == b"const":
            child = self._parse_const_decl()
        else:
            raise self._error("unrecognized top-level item")
        return self._node("item", start, [child])

    def _parse_struct_decl(self) -> IndependentNode:
        start = self.cursor
        self._take_raw(b"struct")
        self._take_kind("TYPEID")
        children: list[IndependentNode] = []
        if self._raw() == b"<":
            children.append(self._parse_generics())
        self._take_raw(b"{")
        if self._raw() == b"doc":
            children.append(self._parse_doc())
        while self._kind() == "IDENT":
            children.append(self._parse_field())
        self._take_raw(b"}")
        return self._node("struct_decl", start, children)

    def _parse_field(self) -> IndependentNode:
        start = self.cursor
        self._take_kind("IDENT")
        self._take_raw(b":")
        child = self._parse_type()
        self._take_raw(b";")
        return self._node("field", start, [child])

    def _parse_enum_decl(self) -> IndependentNode:
        start = self.cursor
        self._take_raw(b"enum")
        self._take_kind("TYPEID")
        children: list[IndependentNode] = []
        if self._raw() == b"<":
            children.append(self._parse_generics())
        self._take_raw(b"{")
        if self._raw() == b"doc":
            children.append(self._parse_doc())
        while self._kind() == "TYPEID":
            children.append(self._parse_variant())
        self._take_raw(b"}")
        return self._node("enum_decl", start, children)

    def _parse_variant(self) -> IndependentNode:
        start = self.cursor
        self._take_kind("TYPEID")
        self._take_raw(b"(")
        children: list[IndependentNode] = []
        if self._raw() != b")":
            children.append(self._parse_vfield_list())
        self._take_raw(b")")
        self._take_raw(b";")
        return self._node("variant", start, children)

    def _parse_vfield_list(self) -> IndependentNode:
        start = self.cursor
        children = [self._parse_vfield()]
        while self._raw() == b",":
            self._take_raw(b",")
            children.append(self._parse_vfield())
        return self._node("vfield_list", start, children)

    def _parse_vfield(self) -> IndependentNode:
        start = self.cursor
        self._take_kind("IDENT")
        self._take_raw(b":")
        child = self._parse_type()
        return self._node("vfield", start, [child])

    def _parse_fn_decl(self) -> IndependentNode:
        start = self.cursor
        self._take_raw(b"fn")
        self._take_kind("IDENT")
        children: list[IndependentNode] = []
        if self._raw() == b"<":
            children.append(self._parse_generics())
        if self._raw() == b"[":
            children.append(self._parse_region_params())
        self._take_raw(b"(")
        if self._raw() != b")":
            children.append(self._parse_param_list())
        self._take_raw(b")")
        self._take_raw(b"->")
        children.append(self._parse_rtype())
        children.append(self._parse_effects())
        if self._raw() == b"requires":
            children.append(self._parse_requires_block())
        self._take_raw(b"{")
        if self._raw() == b"doc":
            children.append(self._parse_doc())
        while self._raw() != b"}":
            if self._raw() is None:
                raise self._error("unterminated function body")
            children.append(self._parse_stmt())
        self._take_raw(b"}")
        return self._node("fn_decl", start, children)

    def _parse_requires_block(self) -> IndependentNode:
        start = self.cursor
        self._take_raw(b"requires")
        self._take_raw(b"{")
        children: list[IndependentNode] = []
        while self._raw() != b"}":
            if self._raw() is None:
                raise self._error("unterminated requires block")
            children.append(self._parse_requires_entry())
        self._take_raw(b"}")
        return self._node("requires_block", start, children)

    def _parse_requires_entry(self) -> IndependentNode:
        start = self.cursor
        if self._raw() == b"doc":
            child = self._parse_doc()
        else:
            child = self._parse_stmt()
        return self._node("requires_entry", start, [child])

    def _parse_contract_decl(self) -> IndependentNode:
        start = self.cursor
        self._take_raw(b"contract")
        self._take_kind("TYPEID")
        children: list[IndependentNode] = []
        if self._raw() == b"<":
            children.append(self._parse_generics())
        self._take_raw(b"{")
        if self._raw() == b"doc":
            children.append(self._parse_doc())
        while self._raw() == b"fn":
            children.append(self._parse_fn_sig())
        while self._raw() == b"law":
            children.append(self._parse_law())
        self._take_raw(b"}")
        return self._node("contract_decl", start, children)

    def _parse_fn_sig(self) -> IndependentNode:
        start = self.cursor
        self._take_raw(b"fn")
        self._take_kind("IDENT")
        children: list[IndependentNode] = []
        if self._raw() == b"[":
            children.append(self._parse_region_params())
        self._take_raw(b"(")
        if self._raw() != b")":
            children.append(self._parse_param_list())
        self._take_raw(b")")
        self._take_raw(b"->")
        children.append(self._parse_rtype())
        children.append(self._parse_effects())
        self._take_raw(b";")
        return self._node("fn_sig", start, children)

    def _parse_law(self) -> IndependentNode:
        start = self.cursor
        self._take_raw(b"law")
        self._take_kind("IDENT")
        self._take_raw(b"(")
        children: list[IndependentNode] = []
        if self._raw() != b")":
            children.append(self._parse_law_arg())
            while self._raw() == b",":
                self._take_raw(b",")
                children.append(self._parse_law_arg())
        self._take_raw(b")")
        self._take_raw(b";")
        return self._node("law", start, children)

    def _parse_law_arg(self) -> IndependentNode:
        start = self.cursor
        children: list[IndependentNode] = []
        if self._kind() == "IDENT":
            self._take_kind("IDENT")
        else:
            self._take_literal()
        return self._node("law_arg", start, children)

    def _parse_conform_decl(self) -> IndependentNode:
        start = self.cursor
        self._take_raw(b"conform")
        children = [self._parse_type()]
        self._take_raw(b":")
        self._take_kind("TYPEID")
        if self._raw() == b"<":
            children.append(self._parse_targs())
        self._take_raw(b"{")
        if self._raw() == b"doc":
            children.append(self._parse_doc())
        while self._kind() == "IDENT":
            children.append(self._parse_fn_bind())
        self._take_raw(b"}")
        return self._node("conform_decl", start, children)

    def _parse_const_decl(self) -> IndependentNode:
        start = self.cursor
        self._take_raw(b"const")
        self._take_kind("IDENT")
        self._take_raw(b":")
        children = [self._parse_type()]
        self._take_raw(b"=")
        children.append(self._parse_cvalue())
        self._take_raw(b";")
        return self._node("const_decl", start, children)

    def _parse_fn_bind(self) -> IndependentNode:
        start = self.cursor
        self._take_kind("IDENT")
        self._take_raw(b"=")
        self._take_kind("IDENT")
        self._take_raw(b";")
        return self._node("fn_bind", start, [])

    def _parse_doc(self) -> IndependentNode:
        start = self.cursor
        self._take_raw(b"doc")
        self._take_kind("STRING")
        self._take_raw(b";")
        return self._node("doc", start, [])
