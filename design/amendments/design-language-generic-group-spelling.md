Node: language/generics

Decision: Ordered parameter groups are declared with `interface` and concrete argument groups with `binding`, and a group introduction in a function or nominal header is explicitly prefixed `interface`, including a group with no header parameters, because an unbounded type parameter and a zero-parameter group otherwise both have the spelling `<F>` and grammar selection would depend on name resolution, instead of keeping the `formal` and `actual` keywords or deciding whether a bare header name imports a group by looking up that name.

Decision: Forwarded groups, concrete argument groups and qualified member calls keep their existing argument-position spelling without an `interface` prefix, because those positions do not declare type parameters and have no corresponding ambiguity, instead of adding a marker at every use of a group.

Rejected:
- Give a group import priority over an unbounded type parameter when their grammar lookahead overlaps: rejected because parser priority would change the meaning of a declaration according to an unrelated declaration and would leave `<T>` without one syntactic meaning.
