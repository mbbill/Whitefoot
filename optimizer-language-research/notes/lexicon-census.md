# D3 lexicon census (2026-07-07) — per-keyword rulings under LEX-1

Method: for each borrowed surface name, compare the prior's semantics (chiefly Rust)
with ours; PASS = genuine semantic match (mnemonic admissible, self-contained
definition still mandatory); HOLD = partial match, decision deferred; FAIL = rename.

| name | prior | semantic comparison | ruling |
|---|---|---|---|
| Ok/Err, Some/None | Rust Result/Option constructors | identical semantics (sum-type constructors, same roles); our match is wildcard-free but that is the construct, not the names | PASS |
| Result/Option/Bool | Rust/universal | identical | PASS |
| box<T> | Rust Box<T> | heap-owned unique pointer, moves, drops with owner — identical; ours is a lowercase type-former per TYPE-1 convention | PASS |
| fn, let, return, match, enum, struct | cross-language | declaration/binding/dispatch semantics match priors; let differs (no rebinding, no inference) but the differences are restrictions, not divergences — a prior-driven writer errs toward REJECTED code, never wrong-but-accepted code | PASS (restriction-safe) |
| &uniq | (replaced mut) | ruled 2026-07-03 | PASS (LEX-1 ruling of record) |
| 'r region sigil | Rust lifetimes 'a | PARTIAL: names a borrow region (match) but Rust prior expects inference/elision (divergence); mitigations: regions always explicit, unknown-region hard error | HOLD (kept provisionally; revisit with W1 experiment data) |
| @l loop label | Rust 'label: | different sigil from prior; low collision risk | PASS |
| deref/index as places | C */[] and Rust auto-deref | prefix-call form is alien (R3-provisional GRAM-5 register item); NAMES are transparent | PASS (names), form stays provisional |
| set | (no direct prior; BASIC/Lisp setf adjacent) | assignment keyword; explicit-over-symbolic chosen with zero census risk | PASS |

Errs-toward-rejection principle (new, reusable): a borrowed name is safe even under prior
divergence if every prior-driven misuse produces a CHECKER REJECTION rather than accepted-
but-wrong code. let (no rebinding), match (no wildcard), 'r (no inference) all satisfy it —
the dangerous class is prior-driven misuse that type-checks, which none of these admit.
