- `match` is the sole conditional construct; its scrutinee is a full expression, and every match is exhaustive over the scrutinee's variants.
- A conditional value is a `let`-initializer match: on every control path each arm terminates in exactly one `give` delivering the declared mode and type, or diverges; give-completeness is a structural last-statement recursion.
- Statement-position match remains for effect-only arms; `give` is legal only inside a let-initializer match arm.

## Facts

- 2026-07-08 statement: the adopted spelling ("contained let-initializer value-match with an explicit give terminator") won over two further weighed rivals recorded in the batch-1 delta cluster — the fully-general always-expression match, and a formatting-containment trick that was shown unsound; the give spelling itself is minimality-selected and R3-provisional, pending a writer-tier experiment. (sourced)
- 2026-07-07 statement: the GRAM-4/EX-1 contradiction was discovered by construction — the worked example could not be written under the narrower scrutinee rule — and the resolution had exactly two candidates on record: widen the scrutinee to expr, or canonicalize the example to bind-then-match. (sourced)

## Moves

- 2026-07-07 (7c1d7641) replaced [[bind-then-match]]: widening the scrutinee to expr resolved the GRAM-4/EX-1 contradiction; bind-then-match was rejected under R3/W1 because it taxes the sole conditional idiom with a mechanical temporary at every use and adds weak-writer naming burden (sourced)
- 2026-07-08 (e687100a) replaced [[helper-fn-conditional-idiom]]: conditional initialization is the most common pattern an AI writer needs; the helper-function idiom's recorded provenance was the literal R3 disqualifier ("cheapest to specify") and value delivery via give removes a mechanical helper function per conditional value (sourced)
- 2026-08-09 (a01bc707) replaced by [[match-form]]: a Bool scrutinee's two arm labels are always exactly `True()` and `False()` in fixed order, so the arm ceremony carried no information the condition did not already carry, and the R3-provisional register had marked no-if as minimality-selected pending the writer evidence this batch supplied (sourced)
