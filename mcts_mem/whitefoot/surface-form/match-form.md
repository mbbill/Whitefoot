- The conditional form is selected by the scrutinee's type, not by the writer: a Bool condition takes `if`, an enum scrutinee takes `match`, and each is the sole form for its class.
- An else-free conditional is the one spelling of the empty alternative; an `else` whose block is empty is rejected, while an empty then-block is admitted.
- An `else` whose block holds exactly one conditional flattens to `else if`, universally.
- A conditional value is a `let` initializer in either form: on every control path each arm or branch terminates in exactly one `give`, or diverges, and the binding's type is the derived common delivery type.
- The continuation of a conditional is an enumerated merge point in the same control-flow idiom as the loop, with the empty join defined as the contradictory state for both.

## Facts

- 2026-07-08 statement: the adopted spelling ("contained let-initializer value-match with an explicit give terminator") won over two further weighed rivals recorded in the batch-1 delta cluster — the fully-general always-expression match, and a formatting-containment trick that was shown unsound; the give spelling itself is minimality-selected and R3-provisional, pending a writer-tier experiment. (sourced)
- 2026-07-07 statement: the GRAM-4/EX-1 contradiction was discovered by construction — the worked example could not be written under the narrower scrutinee rule — and the resolution had exactly two candidates on record: widen the scrutinee to expr, or canonicalize the example to bind-then-match. (sourced)
- 2026-08-09 (a01bc707) statement: the empty-then and empty-else cases are deliberately asymmetric, and the asymmetry falls out of one-spelling rather than taste — the else-free form already spells the empty alternative, so an admitted empty `else` would be its second spelling. (sourced)
- 2026-08-09 (a01bc707) pitfall: the grammar hangs BOTH statement sequences of a conditional off one node, unlike the loop, region and arm forms which each carry their own body node, so any pass that enumerates bodies by child production silently skips both branches. Key such passes on the brace pairs already recorded on the node instead. (code)
- 2026-08-09 (a01bc707) pitfall: that omission cost two separate defects before it was understood as one cause. A scope builder opened a nested body for loop, region and arm and none for either conditional, so a branch-local binding declared into the enclosing block and a positive case rejected on a declaration collision; and two checks read the expression node with an only-child accessor, which the infix form breaks because it is the one alternative with two children. (code)
- 2026-08-09 (a01bc707) measurement: the conditional appears in eleven expression positions, not the nine a position list drawn from the delta prose reported; the two omitted were the conditions of the two conditional forms themselves. (code)

## Moves

- 2026-07-07 (7c1d7641) replaced [[bind-then-match]]: widening the scrutinee to expr resolved the GRAM-4/EX-1 contradiction; bind-then-match was rejected under R3/W1 because it taxes the sole conditional idiom with a mechanical temporary at every use and adds weak-writer naming burden (sourced)
- 2026-07-08 (e687100a) replaced [[helper-fn-conditional-idiom]]: conditional initialization is the most common pattern an AI writer needs; the helper-function idiom's recorded provenance was the literal R3 disqualifier ("cheapest to specify") and value delivery via give removes a mechanical helper function per conditional value (sourced)
- 2026-08-09 (a01bc707) replaced [[match-only-conditional]]: a Bool scrutinee's two arm labels are always exactly `True()` and `False()` in fixed order, so the arm ceremony carried no information the condition did not already carry, and the R3-provisional register had marked no-if as minimality-selected pending the writer evidence this batch supplied (sourced)
