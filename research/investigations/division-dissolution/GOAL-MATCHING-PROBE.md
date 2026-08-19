# Goal matching over a `bor` root — what the mechanism actually does

Status: MECHANISM INVESTIGATION ONLY (batch 0072, W3, executor G2). This
file answers the mechanical questions `OPEN-QUESTION.md` raises about option
(b) — a goal-matched division obligation. It proposes no specification
change, contains no delta text, and makes no recommendation about whether to
adopt (b). The owner has deferred every arithmetic-trap-elimination decision
until a systematic audit of all trap sites; this is an input to that audit.

Baseline: ACTIVE v0.32 at `spec/kernel-spec.md`, SHA-256
`5ea3927aef20d08e1c9c80a50242628f2c469974261b68c696ee2db3934e6bf5`, compiler
at `762fb016`. Probe sources and full verbatim output:
`goal-matching-probes/`.

## How the questions were made observable

v0.32 attaches no goal-shaped obligation to anything, so "would a claim
discharge a goal-matched division obligation" cannot be asked of the division
site directly. It can be asked of the one goal-matching judgment v0.32 *does*
have: [FN-8]'s ordinary-call requirement. A callee whose `requires` block
expands to the GoalTemplate

```
bor(ine(n, -2147483648_i32), ine(d, -1_i32))
```

is exactly the retained safe condition of a signed two-variable division,
carried as a goal whose root is a `bor` call. Whatever discharges that goal in
a caller is exactly what would discharge a goal-matched division obligation of
the same shape, because [ENT-4] has one derivability rule for `+G` and [FN-8]
has one goal-equality rule, and neither is parameterised by which rule
attached the goal.

Each probe is that callee plus one caller that varies exactly one thing.

## The answers

**Q1 — Does a claim over a `bor` expression establish that goal as a whole?
Yes.**

`g1.wf` writes the disjunction in canonical ANF, claims the resulting `Bool`
binding, and calls. The program is **accepted**. [ENT-3.S3] establishes every
goal in the claim predicate's goal-origin set with positive sign, and the
goal-origin set of a bare `Bool` binding is the binding datum *plus* its
complete origin expansion — here the whole `bor` tree, because every leaf is
a pure, total, non-trapping table operation over place datums and typed
literals. The `+bor(…)` fact is established opaquely, with no L0 projection
(a non-comparison root has none) and an empty signed decomposition set
(only `-bor` decomposes). Nothing disjunctive enters the conjunctive fact
state; the goal is carried and used whole, exactly as `OPEN-QUESTION.md`
predicted.

**Q2 — Does exact goal identity match a goal whose root is a `bor` call
rather than a comparison? Yes, and only on exact tree equality.**

`g1.wf`'s acceptance is that match. Three negative controls fix how exact it
is:

- `g3.wf` claims only the left child (`ine(n, -2147483648_i32)`). **Rejected**
  — [FN-8]: "a complete `band`, `bor`, `bxor`, or `bnot` tree is one goal that
  no evidence for its children ever composes."
- `g4.wf` claims `bor(divisor_ok, dividend_ok)` — the same disjunction with
  the operands swapped. **Rejected.** No equality step commutes operands.
- `g5.wf` claims `ine(n, minimum_value)` where `minimum_value` is a named
  const of value −2147483648, against a callee spelling the literal.
  **Rejected.** A named-const datum retains its declaration identity and is
  never folded to a literal.

So the match works, and the writer must reproduce the compiler's canonical
tree byte for byte in operand order, operation choice, and constant spelling.

**Q3 — Does the checked program's claim accountability handle it? Yes; and
under v0.32 the claim does not replace the division's own test.**

`g8.wf` is `g1.wf` with its LLVM inspected. The emitted module holds two trap
records:

```
{"rule_id":"CLM-1","message":"division_safe","function":"caller","node_path":[1,0,7,0]}
{"rule_id":"OP-2","message":"","function":"divide_safely","node_path":[0,0,5,0,0,0]}
```

The claim is a named, justified, retained runtime check with a named trap
record — [CLM-2] gives a `bor` predicate no comparison origin, so it is
neither redundant nor refutable, is accepted, and traps whenever it evaluates
false. Its emitted body is the disjunction itself:

```llvm
define internal i32 @wf_caller(i32 %v0, i32 %v1) {
entry:
  %v2 = select i1 true, i32 -2147483648, i32 -2147483648
  %v3 = icmp ne i32 %v0, %v2
  %v4 = select i1 true, i32 -1, i32 -1
  %v5 = icmp ne i32 %v1, %v4
  %v6 = or i1 %v3, %v5
  br i1 %v6, label %check.cont.b0.i5, label %check.trap.b0.i5
```

The callee's division site is unaffected by any of it. Under v0.32 a signed
`/` with two non-constant operands is outside the divisor class, so it carries
no obligation and keeps its complete anonymous trap:

```llvm
define internal i32 @wf_divide_safely(i32 %v0, i32 %v1) {
entry:
  %t0 = icmp eq i32 %v1, 0
  %t1 = icmp eq i32 %v0, -2147483648
  %t2 = icmp eq i32 %v1, -1
  %t3 = and i1 %t1, %t2
  %t4 = or i1 %t0, %t3
  br i1 %t4, label %overflow.trap.v2, label %overflow.cont.v2
```

The accountability machinery therefore works, but today a program written this
way pays **both** tests: the named claim in the caller and the anonymous
operation-internal test in the callee. Nothing in v0.32 connects the proved
goal to the division site.

**Q4 — Does a dominating `if bor(...)` branch establish the same goal via
[ENT-3.S1]? Yes, including under `deny_claims`.**

`g2.wf` replaces the claim with `if safe { … }` over the identical expression
and marks the callee `deny_claims`. The program is **accepted**. S1
establishes each goal in the condition's goal-origin set as `+G` at the
then-block entry, through the same expansion S3 uses, so the branch and the
claim produce the same fact. A `bor`-rooted goal therefore has a non-assertion
fact source, which is what a strict repair requires.

## Two mechanical limits found

**A symbolic generic body cannot spell `min(T)`.** `g6.wf` writes
`-2147483648_T` inside a `requires` block of `fn divide_safely<T: Int>`. It
does not lex:

```
whitefootc: TerminalClassification/Source [FORM-5]: TerminalIssue { token: TokenId { source: SourceId(0), start: ByteOffset(98), end: ByteOffset(111) }, owner: Form5 }
```

[FORM-5] gives an integer literal a mandatory concrete type suffix from the
closed set, and a type parameter is not one. A generic body dividing two
values of a signed type parameter therefore has no source spelling of the
disjunction that would hold for all of its instances — one claim cannot serve
`i8`, `i16`, `i32`, and `i64`, because each instance's goal carries a
different literal. This is a property of the literal grammar, not of the goal
machinery, and it bounds any goal-matched design that needs `min(T)` in the
goal.

**A subscripted operand has no goal origin at all.** `g7.wf` passes
`values[index]` as the dividend and claims the disjunction over the same
subscripted read. **Rejected**: [ENT-3] gives a subscript no goal origin, so
the claim establishes nothing about it, and [FN-8] substitutes a
compiler-owned ephemeral actual-value datum for the argument, which [ENT-2]
says "cannot be established by naming the original subscript again". The
diagnostic renders that datum explicitly:

```
instantiated_goal: "Boolean(Or)…(argument #0 pre-transfer value (caller=FunctionId(1), call=NodePath { components: [1, 0, 11, 0, 0, 0, 0] }, captured=Integer(I32), projections=[], type=Integer(I32)), Literal(Integer { ty: I32, bits: 2147483648 }))…"
```

and names the repair: bind the subscripted read through one preceding ordinary
`let` and establish the requirement over that binding. This is the same
one-rebinding step [ENT-6] already states for a subscripted offset atom; it
closes the case rather than blocking it.

## Diagnostic rendering, as it stands

`render_goal` prints the internal typed tree, not source syntax. The residual
a writer would be shown for an unproved `bor`-rooted goal today is:

```
Boolean(Or)<types=[], consts=[]>(Integer { operation: NotEqual, operand_type: Integer(I32) }<types=[], consts=[]>(Place { root: BindingId(0), projections: [], ty: Integer(I32) }, Literal(Integer { ty: I32, bits: 2147483648 })):Bool, Integer { operation: NotEqual, operand_type: Integer(I32) }<types=[], consts=[]>(Place { root: BindingId(1), projections: [], ty: Integer(I32) }, Literal(Integer { ty: I32, bits: 4294967295 })):Bool):Bool
```

Since Q2's answer is that the writer must reproduce the tree exactly, that
rendering is the whole repair instruction, and in its current form it does not
show the bytes to type. It is a compiler-side observation, recorded here
because any design that leans on exact goal matching inherits it.

## Verbatim probe output

`goal-matching-probes/OBSERVED.txt` holds the complete unedited run of all
eight probes. Summary line per probe:

| probe | varies | result |
| --- | --- | --- |
| `g1.wf` | claim over the exact `bor` tree | ACCEPTED |
| `g2.wf` | dominating `if` over the same tree, callee `deny_claims` | ACCEPTED |
| `g3.wf` | claim over one child only | rejected, [FN-8] `Unproved` |
| `g4.wf` | claim over the same `bor` with operands swapped | rejected, [FN-8] `Unproved` |
| `g5.wf` | named const in place of the literal | rejected, [FN-8] `Unproved` |
| `g6.wf` | literal typed at a generic parameter, `-2147483648_T` | rejected at lexing, [FORM-5] |
| `g7.wf` | subscripted dividend | rejected, [FN-8] `Unproved`, ephemeral actual-value datum in the goal |
| `g8.wf` | `g1.wf`, LLVM inspected | ACCEPTED; CLM-1 named check in the caller, OP-2 anonymous trap still in the callee |
