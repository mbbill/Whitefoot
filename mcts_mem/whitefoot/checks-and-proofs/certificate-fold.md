- A written certificate may scale a premise by an unsigned value or a decimal. Its accumulated polynomial has degree at most two; every nonlinear monomial folds to the value image of an admitted exact multiplication, and a surviving nonlinear monomial rejects.
- The fold matches operand and multiplicity declarations rather than their expanded images.
- Each demanded binding contributes one opaque handle at its type range. The handle is killed with its binding image and is replaced by that image before the residual is proved.
- Only certificate folding uses opaque binding handles. Other proof consumers use ordinary value images.
- Multiplicities are restricted to unsigned values.
- Nonlinear monomials fold into existing multiplication results. Product operands are never expanded.
- Multiplication-domain judgments read transparent value images.

## Facts

- 2026-09-05 measurement: held to one shape with only the derivation of `stride` varying, folding by operand image accepted `let stride = stride_src;` and rejected `stride_src + padding`, `stride_src + 4_u64`, and `2_u64 * stride_src`. Folding by declaration accepts all four. A stride is definitionally derived — padding, DIB rounding, tile area, channel count, alignment — so the image rule reached matrix multiply, where the stride happens to be a parameter, and nothing else in the domain a nineteen-program sweep wrote for it. (sourced)
- 2026-09-05 rationale: the unsigned restriction was left in place rather than widened to signed. A signed multiplicity would need its own nonnegativity obligation at the scaling step, and no measured case has asked for one. (sourced)
- 2026-09-05 pitfall: the domain site and the fold site want opposite things from the same binding. Reading handles at the domain site loses the interval and fails the multiplication's own OP-2; reading images at the fold site loses the identity. Both were built. The resolution is that they are different sites, not that one representation wins. (sourced)
- 2026-09-05 measurement: a proof block costs about eight times per doubling of its entries — 389 ms at 64, 3.0 s at 128, 26.8 s at 256 — which puts PRF-1's admitted 4096-entry ceiling many hours out of reach. The ceiling is a structural limit on written source, not a work budget, and nothing in the corpus approaches it. Recorded in `compiler/README.md`. (sourced)

- 2026-09-06 (2620c413) constraint: product operands may themselves be products. Expanding them can exceed degree two or replace a provable affine residual with an unprovable one. Each fold instead removes one of the finite monomials. (sourced)

- 2026-09-09 limitation: the earlier 64/128/256-use timing record supplies neither stage attribution nor a pinned reproduction bundle; its 4096-use implication is extrapolation, not measurement. The documented checker separately queries target redundancy and relation premises, checks named-premise availability, accumulates the sum and proves a residual, so explicit step count alone cannot establish linear total cost. See `research/investigations/proof-certificate-architecture/SOURCE-CHECKING.md`. (sourced)

## Moves

- 2026-09-05 replaced [[fold-by-operand-image]]: a local's image is transparent, so `let stride = width + padding;` gives `stride` the image `width + padding`, and a product over `stride` and a certificate scaling by `stride` arrived at the fold as different arithmetic; matching by declaration is the rule the neighbouring premise already used (sourced)
- 2026-09-06 (2620c413) replaced [[fold-by-operand-image]]: an operand and a multiplicity name the same value when they name the same declaration, not when their images coincide (sourced)
- 2026-09-06 (2620c413) replaced [[handle-equality-as-fact]]: the handle needs to exist between the fold and the residual and be substituted away before proving, not to be carried into the proof as a premise (sourced)
- 2026-09-06 (2620c413) replaced [[opaque-binding-image]]: the handle is scoped to the interval between the fold and the residual rather than installed as the binding's representation (sourced)
