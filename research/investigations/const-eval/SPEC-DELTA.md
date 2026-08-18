# Const-evaluation and #35 rule deltas for the v0.31 candidate

> Superseded 2026-08-18: applied at v0.31 activation (eb8e8634). Historical design evidence; the active spec owns the normative text.

Batch 0070, W5 (CONST-1/CONST-2 deferred notes, OWN-1/FN-8 repair conflict
#35). This file is delta text for the lead — the single writer of the v0.31
candidate — to integrate; it edits no specification byte itself. Touched
rule ids: CONST-1, CONST-2, OWN-1, EFF-2. Two of the edits change wf-ebnf
fences, so the grammar verifier MUST run on the candidate and
`src/syntax/grammar/generated.rs` MUST be regenerated from the candidate
bytes (`cargo run --bin whitefoot-grammar-tables -- <candidate> >
src/syntax/grammar/generated.rs`) in the same change. The compiler
implementation is already landed and inert; integration flips exactly one
switch, `V031_CANDIDATE_SEMANTICS` in `compiler/src/semantic/mod.rs`, to
`true` in the same change as the candidate bytes and the regenerated
tables. No other compiler edit is needed except the two self-adjusting
points named under "Derived material" below.

## Edit 1 — CONST-1: one-operation const arithmetic

Replace the CONST-1 fence:

```wf-ebnf CONST-1
const := ("[0-9]+" | IDENT) (infix_op ("[0-9]+" | IDENT))?
```

Keep this CONST-1 sentence unchanged:

> A decimal integer literal is bare and u64 by position; an IDENT names an
> in-scope integer-typed const-generic parameter [GRAM-2] or a top-level
> integer-typed named-const item [CONST-2].

Replace the sentence `The set is closed and total: no operators, no calls,
no in-language computation in v0.` with the following sentences
(sentence-per-line):

A const-expression is at most one operation over two terms, exactly the
shape [GRAM-6] fixes for expressions: composition is by a named const or a
forwarded const parameter, and no precedence, associativity, or
parenthesization surface exists.
The tail reuses `infix_op`, and its spelling must be one of the five bare
operators `+`, `-`, `*`, `/`, `%`; a mode-suffixed spelling is a hard error
citing CONST-1 at the `infix_op` node, because const evaluation has no
runtime overflow mode — the grammar admits and the checker restricts,
META-2-clean by the `break` precedent [GIVE-1].

Keep these CONST-1 sentences unchanged:

> Constant-expressions are evaluated at monomorphization [FN-2].
> An IDENT resolving to a non-integer or array-typed const is a
> compile-time rejection [DIAG-1].

Replace the sentence `This closes the const-generic forwarding path:
`const N` is usable as an `array<T, N>` size and forwardable as a `const`
targ.` and the final DEFERRED sentence (`Const arithmetic is DEFERRED with
recorded delta; when added it carries a distinct const-eval overflow-policy
name, does not overload the runtime `.trap` OPNAMEs, and is excluded from
EFF-2's exhibits-traps relation.`) with:

Const evaluation is exact in the unsigned 64-bit domain under the
const-eval overflow policy named `const-reject`: an operation whose
mathematical result lies outside that domain, or whose divisor is zero, is
a compile-time rejection citing CONST-1 at the complete `const` node.
`const-reject` is disjoint from the runtime arithmetic modes: it never
overloads a runtime `.trap` OPNAME or a bare infix trap row, an accepted
const-expression executes no runtime check and cannot trap, and a
const-expression never enters EFF-2's exhibits-traps relation.
Inside a generic template an unevaluated const-expression is symbolic; two
symbolic const-expressions are identical exactly when their operation and
ordered terms are identical, with no commutation, constant folding, or
reassociation, exactly as [FN-8] fixes goal identity.
This keeps the const-generic forwarding path closed under the one
operation: `const N` is usable as an `array<T, N>` size, and a derived
expression such as `N * 2` is usable there and forwardable as a `const`
targ, with each concrete instantiation evaluating it to one u64 value.

## Edit 2 — CONST-2: struct-typed consts

Replace the CONST-2 fence:

```wf-ebnf CONST-2
cvalue := literal | IDENT | "[" cvalue ("," cvalue)* "]" | TYPEID targs? "(" (IDENT ":" cvalue ("," IDENT ":" cvalue)*)? ")"
```

Replace the sentence `` `type` must be const-eligible: a primitive
[TYPE-1], or `array<T, N>` of const-eligible T; `box`, `buffer`, `arena`,
and `slice` are not const-eligible (a const is pure static rodata: no
allocation, no region, no drop). `` with:

`type` must be const-eligible: a primitive [TYPE-1], `array<T, N>` of
const-eligible T, or a source `struct` whose every field type is
const-eligible; enums, `box`, `buffer`, `arena`, and `slice` are not
const-eligible (a const is pure static rodata: no allocation, no region,
no drop).

Extend the total-definition sentence (`The `cvalue` totally defines the
value (T1): ...`) with one additional clause, appended before its final
period:

, and a struct-typed const takes the construction form `TYPEID(field:
cvalue, ...)` naming its exact struct and writing every declared field in
declared order [GRAM-8], each field value a cvalue of the declared field
type

Keep these CONST-2 sentences unchanged:

> The const-dependency graph is acyclic and declaration-before-use
> [TYPE-6]; evaluation is substitution and layout only.
> A const item is never `move`d, `set`, or `&uniq`-borrowed.
> It is read via subscript/`len` (copy-out for copy elements) or
> shared-borrowed `&'r p` in any region [OWN-10], so a const table may be
> `slice_of`-viewed and passed to a consumer.

Insert after that read sentence:

A struct-typed const is additionally read via its field suffixes exactly
as subscript reads: a copy-scalar selection copies out, and a composite
selection keeps the whole-composite read rules.
A struct-typed const is laid out as one read-only static aggregate in the
nominal's ordinary representation.

Replace the final DEFERRED sentence (`Struct/enum-typed consts are
DEFERRED with recorded delta.`) with:

Enum-typed consts and written generic construction arguments in const
position are DEFERRED with recorded delta: a payload-enum const has no
non-consuming read path (a `match` scrutinee is an own place [OWN-13]),
and a tag-only-enum const additionally needs a constant-value family no
current program demands.

## Edit 3 — OWN-1: the #35 clause-conditional repair

Replace the OWN-1 sentence `Every other bare `place` expression of affine
type is a hard error (write `move p`), and `move p` on a copy value is a
hard error (copy values are used bare — one spelling per meaning,
FORM-1).` with:

Every other bare `place` expression of affine type is a hard error, and
`move p` on a copy value is a hard error (copy values are used bare — one
spelling per meaning, FORM-1).
The bare-affine mechanical fix is position-conditional: outside a
`requires` block it is write `move p`, while inside a `requires` block,
where [FN-8] rejects `move` itself, it is restate the clause over copy
operands or non-consuming admitted reads, so the repair never instructs a
spelling FN-8 forbids.

This closes finding #35 (governance/APPROVALS.md, batch 0068 audit item
(4)): OWN-1's unconditional `write move p` instruction sent a writer inside
a requires clause from one hard error (OWN-1 `BareAffineUse`) into a second
(FN-8 `InvalidRequires`), with no third spelling.

## Edit 4 — EFF-2: const position excluded from exhibits-traps

Insert after the EFF-2 body-syntactic contribution sentence (`The
body-syntactic contribution is syntactic over the complete function body:
...`):

A bare operator inside a `const` [CONST-1] is const evaluation under
`const-reject`, not a trapping-mode operation, and contributes nothing to
any effect row.

## Derived material and integration notes

- Grammar: both fences change, so the native grammar verifier must run
  (baseline `spec/kernel-spec-v30.md` archive bytes, candidate = the
  v0.31 candidate) and the committed tables must be regenerated from the
  candidate; `whitefoot-grammar-tables --check` and the in-gate test
  re-verify the agreement.
- Compiler switch: flip `V031_CANDIDATE_SEMANTICS`
  (`compiler/src/semantic/mod.rs`) to `true` in the same change. The
  const-arithmetic paths are table-gated and need no switch; the switch
  gates struct-const eligibility and the clause-conditional repair.
- Self-adjusting tests: `requires_clause_bare_affine_use_carries_the_
  clause_conditional_repair` follows the switch and pins the exact repair
  wording on either side; `const_position_arithmetic_is_a_syntax_
  rejection_under_v030` (semantic/tests/const_eval.rs) asserts the v0.30
  parse rejection and must be updated to assert acceptance (or retired
  into a positive case) when the tables regenerate.
- Diagnostic names: the const-eval overflow policy surfaces as
  `SemanticIssueKind::ConstEvalOverflow { operation }` and the
  mode-suffix rejection as `ConstRuntimeArithmeticMode`, both citing
  CONST-1; neither overlaps a runtime trap kind.
- Compiler residues under the candidate, all explicit Unsupported (valid
  source, not rejections): written generic construction arguments in
  const position (`CompositeValues`); a runtime-index subscript into an
  array field of a struct const (`CompositeValues`; scalar field-path
  reads fold at compile time); nominal-typed consts are conservatively
  unavailable to the FN-9 selector preflight (ordinary checking is
  unaffected).
- Scope decision for the lead: this delta admits struct-typed consts and
  defers BOTH enum const families with the recorded reason above. The
  batch brief said "struct/enum"; if the lead wants tag-only-enum consts
  in v0.31, the compiler needs a tag-only constant-value family
  (CheckedValue + IrConstant + emitter) that this batch deliberately did
  not build — payload-enum consts should stay deferred either way because
  no read path exists.

## Validation evidence (scratch candidate run)

Recorded by the executor from a throwaway copy of the repository
(`/Users/bytedance/do_not_scan/wf-v031-scratch`, deleted after use) with
exactly the above edits applied to the spec text (scratch candidate bytes
SHA-256 `25fe8cf625554ebd0e613409a0120df931c9b0e5e712d6f266c25ea9b456412c`
— informative only; the lead's integration produces the authoritative
candidate), tables regenerated from it, and the switch flipped. The
committed worktree stays byte-identical to v0.30 semantics.

Table regeneration and the native grammar verifier (baseline = the active
v0.30 bytes, candidate = the edited scratch spec):

    $ cargo run --bin whitefoot-grammar-tables -- ../spec/kernel-spec.md \
        > src/syntax/grammar/generated.rs        # exit 0
    $ cargo run --bin whitefoot-grammar -- ../baseline-v030.md ../spec/kernel-spec.md
    structural grammar candidate verified by the active compiler:
    73 productions, 96 decisions, 98 terminal predicates
    # exit 0

Positive evidence — `evidence-const.wf`, exercising an `n * 2` array size
through generic forwarding (rtype and `array_new` targ), `len` and
subscript over the doubled instance, and a struct const read by field:

    struct Window {
      width: u64;
      height: u64;
    }

    const frame: Window = Window(width: 3_u64, height: 2_u64);

    fn doubled<const n: u64>(fill: own u64) -> own array<u64, n * 2> pure {
      return array_new<u64, n * 2>(fill);
    }

    fn main() -> own unit traps {
      let cells = doubled<3>(fill: 7_u64);
      let count = len(cells);
      check ieq(count, 6_u64) else trap "doubled array length";
      let last = cells[5_u64];
      check ieq(last, 7_u64) else trap "doubled array fill";
      let width = frame.width;
      check ieq(width, 3_u64) else trap "struct const width";
      let height = frame.height;
      check ieq(height, 2_u64) else trap "struct const height";
      return unit;
    }

    $ ./compiler/target/debug/whitefootc evidence-const.wf -o evidence-const
    # exit 0
    $ ./evidence-const
    # exit 0
    $ ./compiler/target/debug/whitefootc --emit-llvm evidence-const.wf | grep -A1 "const frame"
    ; const frame
    @.wf_const.0 = private unnamed_addr constant %wf.t0 { i64 3, i64 2 }

Negative evidence — the const-eval overflow policy and the bare-spelling
restriction (`doubled<9223372036854775808>` makes `n * 2` leave u64; a
`*wrap` in const position names a runtime mode):

    $ ./compiler/target/debug/whitefootc t-overflow.wf
    whitefootc: Semantics/Source [CONST-1]: ... kind: ConstEvalOverflow { operation: "*" }
    # exit 1
    $ ./compiler/target/debug/whitefootc t-mode.wf
    whitefootc: Semantics/Source [CONST-1]: ... kind: ConstRuntimeArithmeticMode { mechanical_fix:
    "write the bare operator: const evaluation rejects overflow at compile time and has no runtime arithmetic modes" }
    # exit 1

Negative evidence — the #35 clause-conditional repair (a bare affine
`eeq` operand inside a requires block, the exact APPROVALS batch-0068
shape):

    $ ./compiler/target/debug/whitefootc t-requires.wf
    whitefootc: Semantics/Source [OWN-1]: ... kind: BareAffineUse { mechanical_fix:
    "restate the clause over copy operands or non-consuming admitted reads; a requires block admits no `move`" }
    # exit 1

Whole-suite inventory under the candidate scratch (`cargo test --profile
gate --lib`): 829 passed, 7 failed, and every failure is a pinned-identity
test the integration updates by construction, with no behavioral failure:
`spec::tests::{recorded,computed}_identity...` (the recorded active-spec
hash), four `syntax::grammar::tests` inventory/shape pins over the
regenerated tables, and
`const_eval::const_position_arithmetic_is_a_syntax_rejection_under_v030`
(this batch's own v0.30 inertness pin, to be flipped to a positive case).
The committed worktree's own gate stays green: 836 passed, 0 failed.
