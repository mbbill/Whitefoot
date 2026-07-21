from __future__ import annotations

from form2_independent_lex import IndependentToken
from form2_independent_parse_core import IndependentParserCore
from form2_independent_parse_expressions import IndependentExpressionParserMixin
from form2_independent_parse_items import IndependentItemParserMixin
from form2_independent_parse_signatures import IndependentSignatureParserMixin
from form2_independent_parse_statements import IndependentStatementParserMixin
from form2_independent_parse_types import IndependentTypeParserMixin
from form2_independent_syntax import IndependentForest


class IndependentParser(
    IndependentItemParserMixin,
    IndependentSignatureParserMixin,
    IndependentTypeParserMixin,
    IndependentStatementParserMixin,
    IndependentExpressionParserMixin,
    IndependentParserCore,
):
    pass


def parse_independently(
    tokens: tuple[IndependentToken, ...], source_length: int
) -> IndependentForest:
    return IndependentParser(tokens, source_length).parse_item_forest()
