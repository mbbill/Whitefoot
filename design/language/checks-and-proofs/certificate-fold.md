Decision: A certificate's nonlinear monomials fold to admitted multiplications matched by declaration, with degree at most two and any surviving nonlinear monomial rejecting, because matching by the value an operand is known to equal failed on every derived stride and accepted only a stride copied straight from a parameter, instead of folding by the operand's known value.

Decision: A binding used in a fold is an opaque handle only between the fold and the residual and stands for its known value everywhere else, because the multiplication's domain site needs the interval while the fold site needs the identity and they are different sites, instead of one global representation for the binding.

Decision: Multiplicities are unsigned, because a signed multiplicity would need its own nonnegativity obligation at the scaling step and no program checked so far has needed one, instead of admitting signed multiplicities.

Decision: A product operand may itself be a product and each fold removes one finite monomial, because expanding operands can exceed degree two or replace a provable affine residual with an unprovable one, instead of expanding product operands.

Rejected:
- Fold by the operand's known value: rejected because `let stride = width + padding;` gives the product and the certificate different arithmetic at the fold, so nothing matched except a bare parameter stride.
- Opaque handle as the binding's representation for every reader: rejected because it closed the fold and broke ordinary premises about the binding, relocating the failures without changing the accepted set.
