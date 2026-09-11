Decision: A certificate's nonlinear monomials fold to admitted multiplications matched by declaration, with degree at most two and any surviving nonlinear monomial rejecting, because matching by image failed on every derived stride and accepted only a stride copied straight from a parameter, instead of folding by operand image.

Decision: A binding used in a fold is an opaque handle only between the fold and the residual and is its transparent image everywhere else, because the multiplication's domain site needs the interval while the fold site needs the identity and they are different sites, instead of one global representation for the binding.

Decision: Multiplicities are unsigned, because a signed multiplicity would need its own nonnegativity obligation at the scaling step and no measured case has asked for one, instead of admitting signed multiplicities.

Decision: A product operand may itself be a product and each fold removes one finite monomial, because expanding operands can exceed degree two or replace a provable affine residual with an unprovable one, instead of expanding product operands.

Rejected:
- Fold by operand image: rejected because `let stride = width + padding;` gives the product and the certificate different arithmetic at the fold, so nothing matched except a bare parameter stride.
- Handle equality published as an ordinary fact: rejected because the residual is discharged by the direct route, which does not consult published affine facts, so the equality was present, true, and unreachable.
- Opaque handle as the binding's representation for every reader: rejected because it closed the fold and broke ordinary premises about the binding, relocating the failures without changing the accepted set.
- Signed multiplicities: rejected because they need a nonnegativity obligation at the scaling step.
