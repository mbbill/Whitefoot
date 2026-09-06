# Contract relation vs affine relation — scoping notes (read-only investigation)

Tree: main-v044 @ 82f6d6a, spec v0.44. Compiler: scratchpad/target/release/whitefootc.
All verdicts below were produced by `sh wf.sh <file>` in this directory.

## 1. Corrected measurement table

The brief's table is wrong about contracts. Measured:

| written form | verdict | rule |
|---|---|---|
| `requires a + b;` (w1) | rejected | OP-5 InvalidPredicateCondition — **it parses**; it is just not Bool |
| `requires a >= b + 1_u64;` (w2) | rejected | GRAM-5 parse, `expected [";", "(", "[", ".", "::"]` at `+` |
| `ensures ok + a;` (w4) | rejected | OP-5 — **parses** |
| `ensures ok >= a + 1_u64;` (w5) | rejected | GRAM-5 parse at `+` |
| `requires len(deref(out)) >= len(src);` (s2) | **accepted** | — |
| `requires len(deref(out)) >= len(src) + 2_u64;` (q2) | rejected | GRAM-2 parse at `+` |
| `requires len(deref(out)) >= 2_u64 * len(src);` (q3) | rejected | GRAM-5 parse at `*` |
| `define needed = count + 2_u64;` (t5) | rejected | FN-8 InvalidRequires |
| `define needed = count +wrap 2_u64;` (t1) | **forms**; fails later at the caller | FN-8 UndischargedCallRequirement, goal `len(out) >= len(src) +wrap 2_u64`, Unproved |
| `define needed = count *sat count;` (t3) | **forms** (nonlinear!); fails at the caller | same |
| `define twice = 2_u64;` (v5) | rejected | FN-8 — a bare-atom define is inadmissible |
| `ensures n <= len(src) +wrap 1_u64;` (v4) | rejected | GRAM-2 parse at `+wrap` (arity) |

Conclusions:
* `+` is **not** rejected by the grammar in clause position. `clause_expr` admits
  `infix_op`. What the grammar refuses is a **second operator**.
* Inside a `define` the exact rows `+ - * / %` are refused by a semantic check;
  the total rows `+wrap +sat *wrap *sat` are **admitted, including nonlinear
  `count *sat count`**. So "contract clauses admit no binary arithmetic" is false.
* Addition and multiplication are *not* treated alike beyond the exact/total split.

## 2. Where `+` is actually rejected

`compiler/src/semantic/check/requires.rs:813-816` in `validate_clause_infix`:

    let operator = self.infix_operator_node(tail)?;
    if self.infix_operation(operator)?.is_exact() {
        return self.invalid_clause(clause, entry);
    }

`is_exact()` is `compiler/src/semantic/model.rs:748-761` = `{+ - * / % iabs ineg ishl ishr}`.
Spec authority: FN-8, `spec/kernel-spec.md:1310` — "every proof-required exact or
otherwise partial operation are inadmissible even when another clause states their domain".

The arity wall is `spec/kernel-spec.md:318-319`:

    clause_expr := (atom | call | construct)
                   ((infix_op | compare_op) (atom | call | construct))?

Root cause of the exact ban: a clause is type-checked as an ordinary **runtime**
expression — `requires.rs:236` calls `self.check_expression(...)`. An `invariant`
never does; it goes through `check_affine_expression` (proofs.rs:293) and
`affine_expr` "denotes a mathematical integer expression and never a runtime
evaluation" (spec:3250). Contracts are runtime-typed, invariants are proof-typed.

Note: the OP-2 domain obligation is raised only while the entailment pass walks a
`CheckedExpression` that is in the function **body** (`flow.rs:5286`). A clause's
`CheckedExpression` is dropped in `check_requires`, so admitting `+` in a clause
would raise no obligation. The ban is formation-time policy, not a downstream need.

## 3. What the contract path builds

`RelationTemplate` (`compiler/src/semantic/postcondition.rs:38-42`) =
`{ operation, operands: [RelationDatum; 2], normalized }`. It is **flat**: exactly
two leaf operands, no tree. `RelationDatum` (postcondition.rs:62-81) =
`Result{ty} | Parameter{ordinal, projections, ty} | NamedConst{decl, projections, ty}
 | Literal{value, origin} | Length(PostconditionPlace)`.

Formation: `ensures.rs:625-693 postcondition_relation` requires the root to be one
comparison and each side to *downcast to a single leaf* via
`ensures.rs:695-754 postcondition_relation_datum`; any nested arithmetic hits the
`ExpandedClauseExpression::Operation { .. } => None` arm at ensures.rs:751 and becomes
FN-9 InvalidPostconditionRelation.

Consumption:
* `flow.rs:2093 instantiate_call_postcondition_relation` — substitutes formals at a
  call, minting the MSR-3 call datum via `interned_call_datum` (flow.rs:2112, 2140);
  3 call sites (flow.rs:2303, 2473, 2613). Produces `Relation`.
* `flow.rs:1412 instantiate_postcondition_relation` / `1447 postcondition_relation_term`
  — same thing at a callee return, over body terms.
* `check/publication.rs` (whole file, 249 lines) — CALL-6 contradiction check.

`Relation` (`entailment/state.rs:23-35`) is a **two-term difference bound**:
`Bound{left: TermId, right: TermId, bound} | Equal | Distinct`.

## 4. The real divide: two fact domains

* **L0 / ENT-3 term domain** — `Relation` over `TermId`. Difference bounds only.
  Lengths (`len(P)`), places, call datums live here. This is the *fact base*.
* **Affine domain** — `AffineForm` (`entailment/affine.rs:55`) =
  `constant + sum(coeff * AffineTermId)`. General linear. Used for goals and
  invariants.

`ProofGoal::Ordering { relation, affine }` (flow.rs:1155) carries **both**, so the
prover already unifies them at goal level. Statements do not.

Crucially, even an invariant that *forms* with a coefficient can only *publish* at
L0 when it normalizes to a unit-coefficient difference bound —
`flow.rs:8978 checked_affine_relation_l0`, match at flow.rs:9035-9045 accepts only
`[]`, `[±1]`, `[(1,-1)]`, `[(-1,1)]` and returns `None` otherwise.

And `len(P)` has **no affine value at all**: `flow.rs:1337 affine_goal_value` has no
`ArrayLength`/`BufferLength`/`SliceLength` arm; those fall to the terminal
`GoalExpression::Operation { .. } => None` at flow.rs:1408. The spec states this
directly at `spec/kernel-spec.md:1429`: "a measure term is a clause operand and not
yet an affine atom".

## 5. The affine goal path already speaks `+ - *`

`flow.rs:1386-1401` (`affine_goal_value`) handles `AddExact`, `SubtractExact` and
literal-sided `MultiplyExact`. Verified end-to-end with symbolic operands:

* `u3_symbolic_add_goal.wf` — caller writes `let s = k + 1_u64;`, callee
  `requires n >= m` — **accepted (exit 0)**.
* `u4_symbolic_mul_goal.wf` — caller writes `let s = 2_u64 * k;` — **accepted**.

So the exact rows the clause language refuses are exactly the rows the goal→affine
converter is written to handle. They reach it only from the caller side today.

## 6. Where the affine route stops

* `x3_no_factor.wf` (`let needed = count;`, goal `len(out) >= needed`) — accepted.
* `x4_literal.wf` (`let needed = 8_u64;`) — accepted.
* `x2_factor_one.wf` (`let needed = 1_u64 * count;`) — **Unproved**.
* `x1_expansion_via_param.wf` (`let needed = 2_u64 * count;`) — **Unproved**.
* `y1_mul_no_move.wf` (product against `len(out)`, nothing moved) — **Unproved**;
  `y2_nomul_no_move.wf` (same without the product) — accepted.

A product makes the value a fresh affine atom with no L0 bound to the length term,
and the length has no affine image, so neither route closes. This is the same wall
from the other side: **the two domains meet only where a value is one atomic term.**
