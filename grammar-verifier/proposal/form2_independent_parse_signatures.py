from __future__ import annotations

from form2_independent_syntax import IndependentNode


class IndependentSignatureParserMixin:
    def _parse_generics(self) -> IndependentNode:
        start = self.cursor
        self._take_raw(b"<")
        children = [self._parse_gparam()]
        while self._raw() == b",":
            self._take_raw(b",")
            children.append(self._parse_gparam())
        self._take_raw(b">")
        return self._node("generics", start, children)

    def _parse_gparam(self) -> IndependentNode:
        start = self.cursor
        children: list[IndependentNode] = []
        if self._kind() == "TYPEID":
            self._take_kind("TYPEID")
            if self._raw() == b":":
                self._take_raw(b":")
                self._take_kind("TYPEID")
        else:
            self._take_raw(b"const")
            self._take_kind("IDENT")
            self._take_raw(b":")
            children.append(self._parse_type())
        return self._node("gparam", start, children)

    def _parse_region_params(self) -> IndependentNode:
        start = self.cursor
        self._take_raw(b"[")
        self._take_kind("REGIONID")
        while self._raw() == b",":
            self._take_raw(b",")
            self._take_kind("REGIONID")
        self._take_raw(b"]")
        return self._node("region_params", start, [])

    def _parse_param_list(self) -> IndependentNode:
        start = self.cursor
        children = [self._parse_param()]
        while self._raw() == b",":
            self._take_raw(b",")
            children.append(self._parse_param())
        return self._node("param_list", start, children)

    def _parse_param(self) -> IndependentNode:
        start = self.cursor
        self._take_kind("IDENT")
        self._take_raw(b":")
        children = [self._parse_mode(), self._parse_type()]
        return self._node("param", start, children)
