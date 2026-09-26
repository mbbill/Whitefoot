//! Every diagnostic sentence this work added, pinned by a probe that compiles
//! a source program and compares the exact rendered text.
//!
//! A diagnostic sentence is the product: it is what a writer reads and acts
//! on. A sentence no test compares is free to drift, and the gate verification
//! of 2026-08-28 found fifty-four of them in exactly that state — landed,
//! rendered, and not pinned by a test. This module is the one home for the corpus that
//! closes that: one minimal source per form, the rule it must cite, and the
//! exact fragments its rendered rejection must contain. Adding a sentence to
//! the compiler means adding a row here.
//!
//! A repair [DIAG-1] is a sentence with a promise in it, so its row in
//! `REPAIRS` pins more: besides the rejected source and the repair it
//! carries, one program for each alternative the row carries out, which must
//! be accepted with the repaired construct live.
//!
//! The rows are deliberately redundant with the per-item tests in
//! `driver::tests` and in `semantic::tests`: those pin one sentence beside the
//! program that motivated it and explain why the sentence says what it says,
//! and this table proves no sentence is missing from that set. A sentence
//! removed from the compiler fails here; a sentence reworded fails here.
//!
//! The sentences listed below are not rows, because no source program reaches
//! them. Each is a defensive arm behind an earlier rejection or behind a
//! spelling v0.60 removed, and the reason is checkable one by one. The list is
//! kept complete rather than counted: a count is a second record of the same
//! thing and drifts, and this one already had.
//!
//! - `parameter #{ordinal}` in `entailment::flow::render_goal_datum`, and the
//!   `[parameter #{ordinal}]` subscript step beside it, name a formal, and
//!   only *concrete* goals are rendered.
//! - `no operand in position {index} for this row` in
//!   `check::expressions::calls` needs more operands than the selected row
//!   takes; [OP-1] rejects the arity first.
//! - [EFF-1]'s non-parameter-root reason and its repair need an effect root
//!   that resolves to a value and is not a parameter; the resolver rejects
//!   every such root as an unresolved `EffectRoot` use first.
//! - The two subscript-operand sentences of `check::expressions::flat_storage`
//!   -- "a written move, which consumes rather than indexes" and "an atom that
//!   is not a place", both under the expectation "a place, which a subscript
//!   indexes" -- were reached in v0.59 through the measure-former call
//!   `len_of(...)`, whose operand was an ordinary atom. v0.60 reads a measure
//!   as the `psuffix` `P.len` [OP-15, MSR-1], and [GRAM-5] writes a subscript
//!   only below a `place`, so no source presents either operand shape.
//! - "an array, buffer, or slice place", the same module's scalar-base
//!   sentence, retires with the same former for the same reason: a measure
//!   member read on an unmeasured place is [MSR-1]'s own [TYPE-5] rejection
//!   carrying the measured types, not this one.
//!
//! Two bullets this list used to carry are gone with their sentences rather
//! than with their reachability: `check::expressions::region_spelling` and
//! `slice_of`'s "a borrow of a runtime value binding or a named const" pair no
//! longer exist anywhere in the compiler, regions and view formers having left
//! the language [REF-1, REF-4].
//!
//! C2 deletes the PAR-3 staging report and its three exclusive sentences.
//! Ordinary call diagnostics remain pinned below.

use super::{CompilationFailureKind, CompilerLimits, compile};
use crate::SourceInput;

/// One probe: a minimal source, the rule its rejection must cite, and the
/// exact fragments the rendered rejection must contain.
struct Probe {
    /// The compiled unit's name, which also names the form under test.
    name: &'static str,
    /// The complete source. Minimal on purpose: everything in it is either the
    /// form under test or an ordinary caller that inhabits its signature.
    source: &'static [u8],
    /// The numbered rule [DIAG-1] must select.
    rule: &'static str,
    /// Exact substrings of the rendered rejection.
    sentences: &'static [&'static str],
}

const PROBES: &[Probe] = &[
    // -------------------------------------------------------------------
    // [FORM-3] name slots: the lexical class a grammar position writes.
    // -------------------------------------------------------------------
    Probe {
        name: "const-name-is-not-an-ident.wf",
        source: br#"const Limit: u64 = 8_u64;

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "FORM-3",
        // The whole payload, so the field labels the record prints are pinned
        // with the sentence they carry.
        sentences: &[
            "\n  expected: [IDENT]\n  found: \"Limit\"\n  mechanical_fix: an IDENT slot admits only [FORM-3]'s IDENT `[a-z][a-z0-9_]*`, so a `const`, `fn`, parameter, `let`, field, or binder name is lowercase and is never a TYPEID `[A-Z][A-Za-z0-9]*`, a LABEL `@[a-z][a-z0-9_]*`, or an OPNAME; rename the name written here to the IDENT shape",
        ],
    },
    Probe {
        name: "struct-name-is-not-a-typeid.wf",
        source: br#"struct shape {
  seq: u64;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "FORM-3",
        sentences: &[
            "a TYPEID slot admits only [FORM-3]'s TYPEID `[A-Z][A-Za-z0-9]*`, so a struct, enum, contract, variant, or constructor name is capitalized and is never an IDENT `[a-z][a-z0-9_]*`, a LABEL `@[a-z][a-z0-9_]*`, or an OPNAME; rename the name written here to the TYPEID shape",
        ],
    },
    // Retired with the region spelling: the REGIONID name-slot probe
    // `region-parameter-is-not-a-regionid.wf` pinned the sentence a `[r]`
    // region parameter published, and v0.60 has no REGIONID lexical class,
    // no region parameter and no such sentence to pin. Its successors are
    // the three name-slot probes that remain above — IDENT, TYPEID and
    // LABEL — whose own sentences no longer list REGIONID either.
    Probe {
        name: "break-target-is-not-a-label.wf",
        source: br#"fn main() -> status: ExitStatus pure {
  loop @spin {
    break spin;
  }
  return exit_status(code: 0_u8);
}
"#,
        rule: "FORM-3",
        sentences: &[
            "a LABEL slot admits only [FORM-3]'s LABEL `@[a-z][a-z0-9_]*`, so write the leading `@`; an IDENT `[a-z][a-z0-9_]*`, a TYPEID `[A-Z][A-Za-z0-9]*`, and an OPNAME are other lexical classes and none is admitted here",
        ],
    },
    // -------------------------------------------------------------------
    // [GRAM-2] and [GRAM-9]: the two repairs a grammar position fixes.
    // -------------------------------------------------------------------
    // The section-order repair is pinned by a requirement written after a
    // postcondition rather than by a definition written after a requirement.
    // Both are the same [GRAM-2] mistake and the sentence names both, but the
    // frontier a `define` leaves after the [MSR-5] clause production is one
    // whose expectation list carries IDENT, and [FORM-3]'s reserved-word row
    // owns it before [GRAM-2]'s production repair is reached. That selection
    // is recorded for the owner in the batch report; the sentence itself
    // stays pinned here.
    Probe {
        name: "requires-written-after-ensures.wf",
        source: br#"fn count(end: u64) -> lines: u64 pure contract {
  ensures lines <= 8_u64;
  requires end <= 8_u64;
} {
  return 0_u64;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "GRAM-2",
        sentences: &[
            "a `contract_block` is written in one fixed order: all `define` definitions first, then all `requires` requirements, then all `ensures` postconditions. A clause of an earlier section written after a later one is not admitted, so move it above the first clause of the later section",
        ],
    },
    Probe {
        name: "forbidden-atom-in-a-body.wf",
        source: br#"fn double(value: u64) -> out: u64 pure {
  return value +wrap value;
}

fn helper(value: u64) -> out: u64 pure {
  let a = double(value: double(value: value));
  return a;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "GRAM-9",
        sentences: &[
            "a `call` or `construct` in an atom position does not derive [GRAM-9]: bind the inner call with its own preceding `let` in this body and write that binder in the atom position — `let inner = f(x: 0_u64); let outer = g(y: inner);`",
        ],
    },
    Probe {
        name: "forbidden-atom-in-a-contract-block.wf",
        source: br#"fn count(data: &[u8], start: u64, end: u64) -> lines: u64 reads(data) contract {
  requires imax(start, imin(start, end)) <= end;
} {
  return 0_u64;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "GRAM-9",
        sentences: &[
            "a `call` or `construct` in an atom position does not derive [GRAM-9]: a `contract_block` has no `let`, so bind the inner call with a preceding `define` in this same block and write that binder in the atom position — `define inner = f(x: 0_u64); requires g(y: inner);`",
        ],
    },
    // -------------------------------------------------------------------
    // [MSR-3] and [CALL-6]: the two judgments the fact machinery adds.
    // -------------------------------------------------------------------
    Probe {
        name: "entry-of-a-shared-parameter.wf",
        source: br#"fn record(destination: &[u8]) -> written: u64 reads(destination) contract {
  ensures written == deref(entry(destination)).len;
} {
  return deref(destination).len;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "MSR-3",
        sentences: &[
            "InvalidEntryFormer",
            "entry",
        ],
    },
    Probe {
        name: "contradictory-published-relations.wf",
        source: br#"fn measure(taken: Array<u8, 4>) -> measured: u64 pure contract {
  ensures measured <= taken.len;
  ensures taken.len < measured;
} {
  return 0_u64;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "CALL-6",
        sentences: &[
            "ContradictoryPublishedRelations",
            "state one consistent relation set: a contract whose clauses cannot hold together publishes every fact at every caller",
        ],
    },
    // -------------------------------------------------------------------
    // [TYPE-6]: the four colliding situations.
    // -------------------------------------------------------------------
    Probe {
        name: "collides-with-a-prelude-declaration.wf",
        source: br#"struct Option {
  seq: u64;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-6",
        sentences: &[
            "a source declaration never displaces, overrides, or shadows a PRE-1 prelude declaration of the same spelling and domain, and neither declaration resolves after the collision; rename this declaration",
        ],
    },
    Probe {
        name: "collides-with-a-prelude-opaque-declaration.wf",
        source: br#"struct DirectoryRead {
  seq: u64;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-6",
        sentences: &[
            "a source declaration never displaces, overrides, or shadows a PRE-1 prelude declaration of the same spelling and domain, and neither declaration resolves after the collision; rename this declaration",
        ],
    },
    Probe {
        name: "redeclared-in-one-scope.wf",
        source: br#"fn main() -> status: ExitStatus pure {
  let count = 1_u64;
  let count = 2_u64;
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-6",
        sentences: &[
            "one scope declares each spelling once in a domain, so this is a redeclaration and not a shadow; rename this declaration, or delete the earlier one when nothing reads it",
        ],
    },
    Probe {
        name: "shadows-a-consumed-binding.wf",
        source: br#"struct Ticket {
  seq: u64;
}

fn consume(ticket: Ticket) -> seq: u64 pure {
  return ticket.seq;
}

fn main() -> status: ExitStatus pure {
  let permit = Ticket(seq: 1_u64);
  let used = consume(ticket: move permit);
  if used == 1_u64 {
    let permit = Ticket(seq: 2_u64);
    let again = consume(ticket: move permit);
  }
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-6",
        sentences: &[
            "]: DeclarationCollision\n",
            "\n  spelling: permit\n",
            "a declaration's scope ends with the block that declares it, and not where its value is consumed: a binding whose value was moved is dead as a value while its declaration stays live, so an inner declaration of the same spelling still collides with it. Rename the inner declaration, or close the block that declares the outer one before this point",
        ],
    },
    // -------------------------------------------------------------------
    // [FN-8, CALL-1] and [OP-4]: the residual names the caller's place.
    // -------------------------------------------------------------------
    Probe {
        name: "prelude-range-residual.wf",
        source: br#"fn main(out: OutputStream, factory: HandleFactory) -> status: ExitStatus pure {
  let header = array_filled::<u8, 4>(value: 65_u8);
  let payload = array_filled::<u8, 9>(value: 66_u8);
  let wide = payload.len;
  let view = &header[0_u64..4_u64];
  let sent = write_once(factory: &factory, output: &out, source: view, start: 0_u64, end: wide);
  return exit_status(code: 0_u8);
}
"#,
        rule: "FN-8",
        // Regression for the range-root substitution: the goal must retain
        // the range holder's own `len` rather than becoming `header.len`
        // (ref4-neg-a-requirement-over-a-range-reference-is-the-ranges-length).
        sentences: &["\n  instantiated_goal: wide <= deref(view).len\n"],
    },
    Probe {
        name: "bounds-residual.wf",
        source: br#"fn main() -> status: ExitStatus pure {
  let table = array_filled::<u8, 4>(value: 0_u8);
  let other = array_filled::<u8, 9>(value: 0_u8);
  let pick = other.len;
  let one = table[pick];
  return exit_status(code: 0_u8);
}
"#,
        rule: "OP-4",
        sentences: &["\n  residual: pick < table.len\n"],
    },
    // -------------------------------------------------------------------
    // [FN-2]: written type and region arguments.
    // -------------------------------------------------------------------
    // C2 removes SYS-2's separate region-argument diagnostic. Ordinary
    // FN-2 arity and FORM-8 call-region probes below retain their coverage.
    // The two region-arity rows this table carried until v0.42 are retired
    // with the sentences they pinned: [FORM-8] gives a call exactly one legal
    // region-argument list, so "no region argument list" and "too many region
    // arguments" are no longer faults a call can commit. Their replacements
    // are the two FORM-8 rows at the end of this table. The generic half of
    // FN-2's arity sentence survives and is pinned here.
    Probe {
        name: "call-without-its-type-arguments.wf",
        source: br#"fn identity<T: Int>(value: T) -> out: T pure {
  return value;
}

fn main() -> status: ExitStatus pure {
  let doubled = identity(value: 1_u64);
  return exit_status(code: 0_u8);
}
"#,
        rule: "FN-2",
        sentences: &[
            "]: TypeMismatch\n",
            "\n  expected: 1 written generic argument\n  found: no explicit argument list\n",
        ],
    },
    // -------------------------------------------------------------------
    // [TYPE-5] generic argument lists and [FN-3] bounds.
    // -------------------------------------------------------------------
    Probe {
        name: "construct-without-its-type-arguments.wf",
        source: br#"struct Pair<T: drop> {
  left: T;
  right: T;
}

fn main() -> status: ExitStatus pure {
  let p = Pair(left: 1_u64, right: 2_u64);
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[
            "]: TypeMismatch\n",
            "\n  expected: 1 written generic argument\n  found: no explicit argument list\n",
        ],
    },
    Probe {
        name: "construct-with-too-many-type-arguments.wf",
        source: br#"struct Pair<T: drop> {
  left: T;
  right: T;
}

fn main() -> status: ExitStatus pure {
  let p = Pair<u64, u64>(left: 1_u64, right: 2_u64);
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[
            "]: TypeMismatch\n",
            "\n  expected: 1 written expanded generic argument\n  found: 2 written expanded generic arguments\n",
        ],
    },
    Probe {
        name: "const-written-in-a-type-parameter-position.wf",
        source: br#"struct Pair<T: drop> {
  left: T;
  right: T;
}

fn main() -> status: ExitStatus pure {
  let p = Pair<4>(left: 1_u64, right: 2_u64);
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[
            "]: TypeMismatch\n",
            "\n  expected: a type argument occupies this parameter position\n  found: a nonmatching behavior argument\n",
        ],
    },
    Probe {
        name: "type-written-in-a-const-parameter-position.wf",
        source: br#"struct Row<const n: u64> {
  count: u64;
}

fn main() -> status: ExitStatus pure {
  let r = Row<u64>(count: 1_u64);
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[
            "]: TypeMismatch\n",
            "\n  expected: a const argument occupies this parameter position\n  found: a nonmatching behavior argument\n",
        ],
    },
    Probe {
        name: "type-arguments-on-a-form-that-declares-none.wf",
        source: br#"struct Plain {
  value: u64;
}

fn main() -> status: ExitStatus pure {
  let p = Plain<u64>(value: 1_u64);
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[
            "]: TypeMismatch\n",
            "\n  expected: 0 written expanded generic arguments\n  found: 1 written expanded generic argument\n",
        ],
    },
    Probe {
        name: "type-arguments-on-a-type-that-takes-none.wf",
        source: br#"fn take(value: Bool<u8>) -> out: u64 pure {
  return 0_u64;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[
            "]: TypeMismatch\n",
            "\n  expected: this type spelled with no type arguments\n  found: a written `<...>` type-argument list on a type that takes none\n",
        ],
    },
    Probe {
        name: "int-bound-is-not-satisfied.wf",
        source: br#"fn widen<T: Int>(value: T) -> out: T pure {
  return value;
}

fn main() -> status: ExitStatus pure {
  let a = widen::<f64>(value: 1.0_f64);
  return exit_status(code: 0_u8);
}
"#,
        rule: "FN-3",
        sentences: &[
            "]: TypeMismatch\n",
            "\n  expected: an integer type, which the parameter's `Int` bound requires\n  found: f64\n",
        ],
    },
    Probe {
        name: "float-bound-is-not-satisfied.wf",
        source: br#"fn scale<T: Float>(value: T) -> out: T pure {
  return value;
}

fn main() -> status: ExitStatus pure {
  let a = scale::<u64>(value: 1_u64);
  return exit_status(code: 0_u8);
}
"#,
        rule: "FN-3",
        sentences: &[
            "]: TypeMismatch\n",
            "\n  expected: a float type, which the parameter's `Float` bound requires\n  found: u64\n",
        ],
    },
    // -------------------------------------------------------------------
    // [TYPE-5] Result and Option: both spellings that carry the arguments.
    // -------------------------------------------------------------------
    Probe {
        name: "result-without-type-arguments.wf",
        source: br#"fn pick(value: u64) -> out: Result pure {
  return Ok(value: value);
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[
            "]: TypeMismatch\n",
            "\n  expected: Result with both type arguments written: as a type `Result<u64, IoError>`, and as a variant constructor `Ok<u64, IoError>(value: v)`\n  found: Result with no written type-argument list\n",
        ],
    },
    Probe {
        name: "result-with-one-type-argument.wf",
        source: br#"fn pick(value: u64) -> out: Result<u64> pure {
  return Ok<u64>(value: value);
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[
            "]: TypeMismatch\n",
            "\n  expected: Result<T, E> with exactly two type arguments\n  found: a Result type-argument list of a different length\n",
        ],
    },
    Probe {
        name: "result-with-a-const-type-argument.wf",
        source: br#"fn pick(value: u64) -> out: Result<4, IoError> pure {
  return Ok<4, IoError>(value: value);
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[
            "]: TypeMismatch\n",
            "\n  expected: a type in each Result type-argument position\n  found: a const argument in a Result type-argument position\n",
        ],
    },
    Probe {
        name: "option-without-type-arguments.wf",
        source: br#"fn pick(value: u64) -> out: Option pure {
  return Some(value: value);
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[
            "]: TypeMismatch\n",
            "\n  expected: Option with its type argument written: as a type `Option<u64>`, and as a variant constructor `Some<u64>(value: v)`\n  found: Option with no written type-argument list\n",
        ],
    },
    Probe {
        name: "option-with-two-type-arguments.wf",
        source: br#"fn pick(value: u64) -> out: Option<u64, u64> pure {
  return Some<u64, u64>(value: value);
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[
            "]: TypeMismatch\n",
            "\n  expected: Option<T> with exactly one type argument\n  found: an Option type-argument list of a different length\n",
        ],
    },
    Probe {
        name: "option-with-a-const-type-argument.wf",
        source: br#"fn pick(value: u64) -> out: Option<4> pure {
  return Some<4>(value: value);
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[
            "]: TypeMismatch\n",
            "\n  expected: a type in the Option type-argument position\n  found: a const argument in the Option type-argument position\n",
        ],
    },
    // Retired with the measure-former call spelling. v0.59 read a measure by
    // calling `len_of(P)`, and these three probes pinned that call's own
    // operand sentences (`indexed-operand-is-a-move`,
    // `indexed-operand-is-not-a-place`, `indexed-place-is-a-scalar`). v0.60
    // reads a measure as the `psuffix` `P.len` [OP-15, MSR-1], so no source
    // reaches those sentences; they join the unreachable list above. The
    // measure place's own defects are pinned by the [OP-4] and [INV-1] probes.
    // Retired with the view kinds it named. `Slice<T>`, arenas and providers
    // are not types in v0.60, and [GRAM-3]'s `type` has no reference
    // production, so no written source reaches [STOR-5] at all: the rule is
    // now the closing clause for a substituted generic instance alone. Its
    // sentence joins the unreachable list in this module's header.
    // -------------------------------------------------------------------
    // [EFF-1] and [EFF-2]: the declared row.
    // -------------------------------------------------------------------
    // Retired with the permission marker. v0.60 has no `&uniq`, so a
    // `writes` path rooted at a reference parameter is the ordinary spelling
    // [REF-1, EFF-1] and there is no shared-versus-unique defect left to
    // publish. The successors are the two [EFF-1] row defects pinned below.
    Probe {
        // Renamed from `repeated-effect-category.wf`: the source repeats one
        // *path* within one category, `reads(left), reads(left)`, which is
        // exactly what the pinned sentence says a row may not do. Nothing here
        // repeats a category.
        name: "repeated-effect-path.wf",
        source: br#"fn touch(left: &u64, right: &u64) -> out: u64 reads(left), reads(left), reads(right) {
  let a = deref(left);
  let b = deref(right);
  return a +wrap b;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "EFF-1",
        sentences: &[
            "]: InvalidEffectRow\n",
            "\n  reason: a row lists each path at most once per category, and this entry repeats one\n  mechanical_fix: delete the repeated entry; `writes(p)` already subsumes `reads(p)`, so the pair is never written for one path\n",
        ],
    },
    Probe {
        name: "effect-suffix-on-a-non-struct.wf",
        source: br#"fn touch(value: &u64) -> out: u64 reads(value.count) {
  return deref(value);
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "EFF-1",
        sentences: &[
            "]: InvalidEffectRow\n",
            "\n  reason: each effect-path suffix must select a field, payload, measure, window part, or indexed position admitted by its prefix type\n  mechanical_fix: select a member or position admitted by the prefix type, or name the reference parameter's complete state; use .inner for Box contents\n",
        ],
    },
    Probe {
        name: "effect-suffix-names-an-undeclared-field.wf",
        source: br#"struct Pair {
  left: u64;
  right: u64;
}

fn touch(pair: &Pair) -> out: u64 reads(pair.middle) {
  return deref(pair).left;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "EFF-1",
        sentences: &[
            "]: InvalidEffectRow\n",
            "\n  reason: an effect-path suffix names a member its selected type does not declare\n  mechanical_fix: name a declared member of that type, or the reference parameter itself\n",
        ],
    },
    Probe {
        name: "declared-row-is-narrower-than-the-body.wf",
        source: br#"fn touch(data: &[u8]) -> out: u64 pure {
  return deref(data).len;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "EFF-2",
        sentences: &[
            "]: EffectMismatch\n",
            "\n  expected_row: reads(data.len)\n  found_row: pure\n  missing: [reads(data.len)]\n  extra: []\n  mechanical_fix: declare the row as `reads(data.len)`, which covers every access the body makes and no other\n",
        ],
    },
    Probe {
        // The body reads and writes `stats.count` and never touches
        // `stats.total`: the suggestion drops the read the write subsumes
        // [EFF-1], nothing is missing, and the untouched write is extra.
        name: "declared-row-writes-an-unexhibited-field.wf",
        source: br#"struct Stats {
  count: u64;
  total: u64;
}

fn record(stats: &Stats) -> result: unit writes(stats.count), writes(stats.total) {
  let old = deref(stats).count;
  set deref(stats).count = old +wrap 1_u64;
  return unit;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "EFF-2",
        sentences: &[
            "\n  expected_row: writes(stats.count)\n  found_row: writes(stats.count), writes(stats.total)\n  missing: []\n  extra: [writes(stats.total)]\n",
        ],
    },
    Probe {
        name: "read-subsumed-by-a-write-of-the-same-path.wf",
        source: br#"fn bump(value: &u64) -> out: u64 reads(value), writes(value) {
  let old = deref(value);
  set deref(value) = old +wrap 1_u64;
  return old;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "EFF-1",
        sentences: &[
            "]: SubsumedEffectEntry\n",
            "\n  entry: reads(value)\n  covering: writes(value)\n",
        ],
    },
    Probe {
        name: "write-below-a-written-path.wf",
        source: br#"struct Pair {
  first: u8;
  second: u8;
}

fn reset(pair: &Pair) -> result: unit writes(pair), writes(pair.first) {
  set deref(pair) = Pair(first: 0_u8, second: 0_u8);
  return unit;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "EFF-1",
        sentences: &[
            "]: SubsumedEffectEntry\n",
            "\n  entry: writes(pair.first)\n  covering: writes(pair)\n",
        ],
    },
    Probe {
        name: "read-below-a-read-path.wf",
        source: br#"struct Pair {
  first: u8;
  second: u8;
}

fn inspect(pair: &Pair) -> result: u8 reads(pair), reads(pair.first) {
  let whole = deref(pair);
  return whole.second;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "EFF-1",
        sentences: &[
            "]: SubsumedEffectEntry\n",
            "\n  entry: reads(pair.first)\n  covering: reads(pair)\n",
        ],
    },
    // -------------------------------------------------------------------
    // [TYPE-5] places, subscripts, and flat storage.
    // -------------------------------------------------------------------
    Probe {
        name: "buffer-length-is-not-a-u64.wf",
        source: br#"fn main() -> status: ExitStatus pure {
  let flag = 1_u64 > 0_u64;
  let store = box_array_filled::<u8>(count: flag, value: 0_u8);
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &["]: TypeMismatch\n", "\n  expected: u64\n  found: Bool\n"],
    },
    Probe {
        // Field suffixes after indices are supported; this scalar element
        // still has no fields. Pin that type rule, not the retired path limit.
        name: "scalar-buffer-element-has-no-fields.wf",
        source: br#"fn main() -> status: ExitStatus pure {
  let store = array_filled::<u8, 4>(value: 0_u8);
  let one = store[0_u64].value;
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[
            "]: TypeMismatch\n",
            "\n  expected: a source struct, whose declared field this suffix selects\n  found: u8\n",
        ],
    },
    // -------------------------------------------------------------------
    // [OWN-4], [OWN-6], and [OWN-10]: regions and reborrows.
    // -------------------------------------------------------------------
    // The former two-statement source is now an acceptance witness in the
    // semantic borrow tests. Pin the retained local-region condition here.
    // Retired with `slice_of` and the permission marker: the four probes
    // `slice-of-a-non-borrow`, `slice-of-a-unique-borrow`,
    // `borrow-kind-does-not-match-the-destination` and
    // `shared-borrow-where-a-unique-one-is-required` each pinned a sentence
    // about a borrow kind or a view former, and v0.60 has neither [REF-1,
    // REF-4]. The general [TYPE-5] mismatch sentence they shared stays
    // pinned by `buffer-length-is-not-a-u64` and
    // `slice-value-where-a-scalar-is-required` below.
    Probe {
        name: "slice-value-where-a-scalar-is-required.wf",
        source: br#"const digits: Array<u8, 2> =[48_u8, 49_u8];

fn measure(view: u64) -> out: u64 pure {
  return view;
}

fn main() -> status: ExitStatus pure {
  let view = &digits[0_u64..2_u64];
  let n = measure(view: view);
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        // The actual here is the range reference `view`, whose kind is
        // `&[u8]`: [TYPE-8] states that "`&T` and `&[T]` are reference kinds
        // and not types", and [REF-4] gives the formation `&digits[0..2]`
        // exactly that kind. The element type `u8` is the type of
        // `deref(view)[i]` and of nothing at this argument position, so a
        // payload naming it describes a value the program does not contain.
        //
        // [TYPE-5] fixes that "argument types match declared parameter types
        // exactly" and therefore that this call is rejected; it does not fix
        // the rendering of a call-argument mismatch, and [DIAG-3] requires
        // byte identity "only where this specification explicitly fixes both
        // selection and encoding". Pinned here are the two facts the
        // specification does fix, inside the `expected`/`found` framing every
        // other TYPE-5 probe in this module shares: the declared parameter's
        // mode and type, written the way [TYPE-5] writes a `set` target's
        // ("carrying expected `own T` and the actual mode and type") because
        // [FN-1] makes a parameter a mode and a type together; and the actual,
        // named as the range reference it is. The actual carries no mode
        // because a reference kind has none.
        sentences: &[
            "\n  expected: own u64\n",
            "\n  found: &[u8]\n",
        ],
    },
    // -------------------------------------------------------------------
    // [TYPE-5] and [FORM-5]: projections, replacement, and operands.
    // -------------------------------------------------------------------
    Probe {
        name: "projection-of-a-non-struct.wf",
        source: br#"fn peek(value: u64) -> out: u64 pure {
  return value.count;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[
            "]: TypeMismatch\n",
            "\n  expected: a source struct, whose declared field this suffix selects\n  found: u64\n",
        ],
    },
    Probe {
        name: "projection-of-an-undeclared-field.wf",
        source: br#"struct Pair {
  left: u64;
  right: u64;
}

fn peek(pair: Pair) -> out: u64 pure {
  return pair.middle;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[
            "]: TypeMismatch\n",
            "\n  expected: a declared field of Pair\n  found: the field name `middle`, which that struct does not declare\n",
        ],
    },
    Probe {
        name: "replacement-value-has-another-type.wf",
        source: br#"struct Ticket {
  seq: u64;
}

fn main() -> status: ExitStatus pure {
  let ticket = Ticket(seq: 1_u64);
  set ticket = 2_u64;
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        // [TYPE-5] fixes both halves of this payload: "the right-hand side of
        // `set p = e;` must produce exactly `own T`", and "a different
        // right-hand-side mode or type is a hard error citing TYPE-5 at the
        // complete `expr` child of the `set_stmt`, carrying expected `own T`
        // and the actual mode and type". A bare type on either side drops the
        // mode the rule names.
        sentences: &["]: TypeMismatch\n", "\n  expected: own Ticket\n  found: own u64\n"],
    },
    Probe {
        name: "boolean-operand-is-an-integer.wf",
        source: br#"fn main() -> status: ExitStatus pure {
  let flag = band(1_u64, 2_u64);
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &["]: TypeMismatch\n", "\n  expected: own Bool\n  found: own u64\n"],
    },
    Probe {
        name: "match-scrutinee-is-not-an-enum.wf",
        source: br#"fn main() -> status: ExitStatus pure {
  let value = 1_u64;
  match value {
    Ok(value: inner) => {
    }
  }
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[
            "]: TypeMismatch\n",
            "\n  expected: an enum scrutinee, whose variants the arms match\n  found: u64\n",
        ],
    },
    Probe {
        name: "generic-numeric-identity-of-a-non-numeric-type.wf",
        source: br#"fn zeroed<T: drop>(sample: T) -> out: T pure {
  return 0_T;
}

fn main() -> status: ExitStatus pure {
  let flag = 1_u64 > 0_u64;
  let a = zeroed::<Bool>(sample: flag);
  return exit_status(code: 0_u8);
}
"#,
        rule: "FORM-5",
        sentences: &[
            "]: TypeMismatch\n",
            "\n  expected: an integer or float type, whose 0 and 1 this form names\n",
        ],
    },
    // -------------------------------------------------------------------
    // [FN-8]: the goal, rendered in the source terms of the caller.
    // -------------------------------------------------------------------
    Probe {
        name: "goal-with-an-infix-operation.wf",
        source: br#"fn need(x: u64) -> out: u64 pure contract {
  define bumped = x +wrap 1_u64;
  requires bumped < 10_u64;
} {
  return x;
}

fn main() -> status: ExitStatus pure {
  let s = 3_u64;
  let r = need(x: s);
  return exit_status(code: 0_u8);
}
"#,
        rule: "FN-8",
        sentences: &["\n  instantiated_goal: s +wrap 1_u64 < 10_u64\n"],
    },
    Probe {
        name: "goal-with-a-numeric-conversion.wf",
        source: br#"fn need(x: u32) -> out: u32 pure contract {
  define wide = cvt::<u32, u64>(x);
  requires wide < 10_u64;
} {
  return x;
}

fn main() -> status: ExitStatus pure {
  let bytes = array_filled::<u8, 1>(value: 3_u8);
  let raw = bytes[0_u64];
  let s = cvt::<u8, u32>(raw);
  let r = need(x: s);
  return exit_status(code: 0_u8);
}
"#,
        rule: "FN-8",
        // The actual is read from a buffer element rather than written as a
        // literal: a conversion of a known constant is now discharged by the
        // affine route, which reaches a call goal that projects to no L0
        // relation, so a literal actual would prove the requirement and print
        // no diagnostic. The pinned sentence is unchanged.
        sentences: &["\n  instantiated_goal: cvt::<u32, u64>(s) < 10_u64\n"],
    },
    Probe {
        name: "goal-with-a-reinterpretation.wf",
        source: br#"fn need(x: i64) -> out: i64 pure contract {
  define raw = reinterpret::<i64, u64>(x);
  requires raw < 10_u64;
} {
  return x;
}

fn main() -> status: ExitStatus pure {
  let s = 3_i64;
  let r = need(x: s);
  return exit_status(code: 0_u8);
}
"#,
        rule: "FN-8",
        sentences: &["\n  instantiated_goal: reinterpret::<i64, u64>(s) < 10_u64\n"],
    },
    Probe {
        name: "goal-with-a-float-literal.wf",
        source: br#"fn need(x: f64) -> out: f64 pure contract {
  requires flt(x, 1.0_f64);
} {
  return x;
}

fn main() -> status: ExitStatus pure {
  let v = 2.0_f64;
  let r = need(x: v);
  return exit_status(code: 0_u8);
}
"#,
        rule: "FN-8",
        sentences: &["\n  instantiated_goal: flt(v, 1.0_f64)\n"],
    },
    Probe {
        name: "goal-over-an-admitted-index-actual.wf",
        source: br#"fn need(x: u8) -> out: u8 pure contract {
  requires x < 10_u8;
} {
  return x;
}

fn main() -> status: ExitStatus pure {
  let data = array_filled::<u8, 4>(value: 0_u8);
  let r = need(x: data[0_u64]);
  return exit_status(code: 0_u8);
}
"#,
        rule: "FN-8",
        sentences: &["\n  instantiated_goal: data[0_u64] < 10_u8\n"],
    },
    Probe {
        name: "goal-over-a-dereferenced-holder.wf",
        source: br#"fn need(names: &[u8], pos: u64) -> out: u64 pure contract {
  define spare = deref(names).len;
  requires pos <= spare;
} {
  return pos;
}

fn outer(names: &[u8]) -> out: u64 pure {
  let r = need(names: names, pos: 9_u64);
  return r;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "FN-8",
        // [OP-15] spells a measure read through a reference `deref(names).len`;
        // the renderer currently drops the `deref`. The pinned sentence is the
        // specification spelling and stays failing until the renderer is fixed.
        sentences: &["\n  instantiated_goal: 9_u64 <= deref(names).len\n"],
    },
    Probe {
        // A generic callee is named as a call writes it [FN-2], never by the
        // symbol that keys its lowering.
        name: "goal-of-a-generic-instance.wf",
        source: br#"fn need<const n: u64>(x: u64) -> out: u64 pure contract {
  requires x < n;
} {
  return x;
}

fn main() -> status: ExitStatus pure {
  let r = need::<4>(x: 9_u64);
  return exit_status(code: 0_u8);
}
"#,
        rule: "FN-8",
        sentences: &["\n  concrete_callee: need::<4>\n"],
    },
    // -------------------------------------------------------------------
    // [FN-9]: the selected return, named by its instance.
    // -------------------------------------------------------------------
    Probe {
        name: "postcondition-of-a-generic-instance.wf",
        source: br#"fn bad<T: Int>(value: T) -> result: T pure contract {
  ensures result < value;
} {
  return value;
}

fn main() -> status: ExitStatus pure {
  let ignored = bad::<u8>(value: 0_u8);
  return exit_status(code: 0_u8);
}
"#,
        rule: "FN-9",
        sentences: &["\n  concrete_function: bad::<u8>\n"],
    },
    // -------------------------------------------------------------------
    // [PROV-6] and [GRAM-8]: a generic nominal instance is named as its
    // type is written [GRAM-3], never by the key the checker interned it
    // under.
    // -------------------------------------------------------------------
    Probe {
        name: "unconsumed-instance-of-a-generic-nodrop-struct.wf",
        source: br#"nodrop struct Token<T> {
  value: T;
}

fn main() -> status: ExitStatus pure {
  let token = Token<u64>(value: 1_u64);
  return exit_status(code: 0_u8);
}
"#,
        rule: "PROV-6",
        sentences: &["\n  binding: token\n  obligation: Token<u64>\n"],
    },
    Probe {
        name: "const-of-a-generic-struct-with-a-wrong-field.wf",
        source: br#"struct Wrap<T> {
  value: T;
}

const w: Wrap<u64> = Wrap<u64>(other: 1_u64);

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "GRAM-8",
        sentences: &[
            "]: InvalidConstructionFields\n",
            "\n  constructor: Wrap<u64>\n  declared_fields: [value]\n",
        ],
    },
    // [FORM-8] one canonical region spelling: each position a region can
    // occupy, written exactly where the surrounding text does not fix it.
    // [FORM-8, OWN-11] a loop body is itself a region block, so a block that
    // is the body's only statement spells that one region twice.
    // -------------------------------------------------------------------
    // [LIV-1] and [LIV-2]: join-checked liveness and the one commit.
    // -------------------------------------------------------------------
    Probe {
        name: "branches-disagree-about-a-binding.wf",
        source: br#"fn measure(cell: Box<Array<u8>>) -> size: u64 pure {
  let n = cell.inner.len;
  return n;
}

fn main() -> status: ExitStatus pure {
  let c = box_array_filled::<u8>(count: 4_u64, value: 0_u8);
  let flag = 1_u64;
  let taken = 0_u64;
  if flag == 1_u64 {
    set taken = measure(cell: move c);
  } else {
    set taken = 7_u64;
  }
  return exit_status(code: 0_u8);
}
"#,
        rule: "LIV-1",
        sentences: &[
            "\n  binding: c\n",
            "\n  live_predecessor: the `else` branch\n",
            "\n  dead_predecessor: the `if` branch\n",
            "every predecessor of a join agrees on a binding\'s live-or-dead status: consume it on every predecessor, on none, or commit a value back into it before the predecessor that consumed it reaches the join",
        ],
    },
    Probe {
        name: "one-iteration-leaves-an-outer-binding-dead.wf",
        source: br#"fn measure(cell: Box<Array<u8>>) -> size: u64 pure {
  let n = cell.inner.len;
  return n;
}

fn main() -> status: ExitStatus pure {
  let c = box_array_filled::<u8>(count: 4_u64, value: 0_u8);
  for (i in 0_u64..2_u64) {
    let taken = measure(cell: move c);
  }
  return exit_status(code: 0_u8);
}
"#,
        rule: "OWN-11",
        sentences: &[
            "\n  binding: c\n",
            "one iteration must leave every outer binding in the status the next one starts from: commit a value back into it before the backedge, or declare and consume it inside the body",
        ],
    },
    // [VIEW-4]: the same commit at a *copy* view, whose target [LIV-2]'s own
    // condition would admit with nothing consumed.
    // -------------------------------------------------------------------
    // [BLK-1] and [PROV-1]: the container nominals a construct may not name
    // and the store region an extent always writes.
    // -------------------------------------------------------------------
    // -------------------------------------------------------------------
    // [INV-1]: the one `call` an affine factor admits is a measure former.
    // -------------------------------------------------------------------
    Probe {
        name: "an-affine-factor-that-is-not-a-measure.wf",
        source: br#"fn main() -> status: ExitStatus pure {
  let limit = 4_u64;
  let seen = 0_u64;
  for (
    at in 0_u64..4_u64,
    invariant bounded: seen <= imin(limit, limit)
  ) {
  }
  return exit_status(code: 0_u8);
}
"#,
        rule: "INV-1",
        sentences: &[
            "an affine factor calls something other than a measure former",
            "write P.len, P.cap or P.head over a measured place",
        ],
    },
    // Retired with the rules whose sentences they pinned. Each probe below
    // cited a rule v0.60 does not have, so the sentence it compared no longer
    // exists anywhere in the compiler and no source can reach it:
    //   - slice-of-arena-content-under-a-foreign-region.wf [OWN-10]
    //   - borrow-of-local-storage-under-a-parameter-region.wf [OWN-10]
    //   - returned-child-reborrow-names-a-foreign-region.wf [OWN-10]
    //   - returned-borrow-of-a-local-region.wf [OWN-4]
    //   - caller-region-for-an-argument-child.wf [OWN-6]
    //   - region-written-at-an-unrelated-parameter.wf [FORM-8]
    //   - region-elided-at-a-result.wf [FORM-8]
    //   - region-parameter-list-out-of-order.wf [FORM-8]
    //   - region-written-at-the-innermost-borrow.wf [FORM-8]
    //   - region-block-is-the-whole-loop-body.wf [FORM-8]
    //   - region-block-name-nothing-references.wf [FORM-8]
    //   - region-argument-the-call-determines.wf [FORM-8]
    //   - prelude-region-argument-the-call-determines.wf [FORM-8]
    //   - two-targets-of-one-commit-overlap.wf [LIV-2]
    //   - a-commit-target-carrying-a-region.wf [LIV-2]
    //   - a-commit-displacing-a-live-loan.wf [VIEW-4]
    //   - a-construct-naming-a-run.wf [BLK-1]
    //   - a-construct-naming-a-provider.wf [BLK-1]
    //   - an-extent-eliding-its-store-region.wf [FORM-8]
    // The region and loan rules [OWN-3, OWN-4, OWN-6, OWN-10, FORM-8, VIEW-4,
    // BLK-1, LIV-2] left the language with regions, permission markers, views
    // and the fallible store take. Their successors are the reference rules
    // [REF-1, REF-3, REF-4] and the window rules [WIN-1, WIN-3, OP-10],
    // whose own sentences are pinned by the probes that remain above.
];

/// Every sentence in the corpus is rendered by a program that reaches it.
#[test]
fn every_diagnostic_sentence_is_pinned_by_a_probe() {
    for probe in PROBES {
        let failure = compile(
            &[SourceInput::new(probe.name, probe.source)],
            CompilerLimits::default(),
        )
        .expect_err(probe.name);
        assert_eq!(
            failure.kind(),
            CompilationFailureKind::Source,
            "{}: a probe rejects source, and never stops on compiler capability: {failure}",
            probe.name
        );
        assert_eq!(
            failure.rule_id(),
            Some(probe.rule),
            "{}: {failure}",
            probe.name
        );
        // The complete record `whitefootc` prints, kind name included. Every
        // field is one `\n  label: value` line; the closing newline lets a
        // probe pin a last field as a complete line too.
        let rendered = format!("{failure}\n");
        for sentence in probe.sentences {
            assert!(
                rendered.contains(sentence),
                "{}: the rendered rejection no longer carries this sentence.\nwanted: {sentence}\ngot:    {rendered}",
                probe.name,
            );
        }
    }
}

/// One repair [DIAG-1], pinned with the programs it produces: a rejected
/// source, the rule and the exact repair its rejection carries, and one
/// source for each alternative the pair carries out.
///
/// DIAG-1 asks two things of a repair's alternatives: carried out as the
/// repair directs, the rejected judgment succeeds where its construct runs,
/// in a state that is not contradictory; and nothing it writes is text a rule
/// rejects. A sentence alone can drift from both, as the printed repairs did
/// before v0.72. A pair makes the repair checkable: each repaired source must
/// be accepted, and no judgment in a function it declares may succeed only
/// because the state it is asked in is contradictory. A guard around a
/// refuted goal, for one, compiles and is caught by the second condition.
///
/// A green run shows that each pinned alternative, applied to its own probe,
/// works. It does not show that every repair works in every program.
struct RepairPair {
    /// The compiled unit's name, which also names the case under test.
    name: &'static str,
    /// The rejected source.
    rejected: &'static [u8],
    /// The numbered rule [DIAG-1] must select.
    rule: &'static str,
    /// Exact substrings of the rendered rejection: the disposition where the
    /// rule carries one, and the repair.
    sentences: &'static [&'static str],
    /// One program for each alternative the pair carries out.
    repaired: &'static [&'static [u8]],
}

const REPAIRS: &[RepairPair] = &[
    // -------------------------------------------------------------------
    // [FN-8] an ordinary call's requirement.
    // -------------------------------------------------------------------
    RepairPair {
        name: "call-requirement-refuted.wf",
        rejected: br#"fn small(x: u64) -> result: u64 pure contract {
  requires x < 10_u64;
} {
  return x;
}

fn main() -> status: ExitStatus pure {
  let r = small(x: 20_u64);
  return exit_status(code: 0_u8);
}
"#,
        rule: "FN-8",
        sentences: &[
            "\n  disposition: Refuted\n",
            "\n  mechanical_fix: `20_u64 < 10_u64` is false for the values that reach this call, so no fact can establish it here: pass arguments that satisfy it, or change the statements or requirements that fix those values\n",
        ],
        repaired: &[br#"fn small(x: u64) -> result: u64 pure contract {
  requires x < 10_u64;
} {
  return x;
}

fn main() -> status: ExitStatus pure {
  let r = small(x: 5_u64);
  return exit_status(code: 0_u8);
}
"#],
    },
    RepairPair {
        name: "call-requirement-over-parameters.wf",
        rejected: br#"fn small(x: u64) -> result: u64 pure contract {
  requires x < 10_u64;
} {
  return x;
}

fn caller(y: u64) -> result: u64 pure {
  let r = small(x: y);
  return r;
}

fn main() -> status: ExitStatus pure {
  let r = caller(y: 3_u64);
  return exit_status(code: 0_u8);
}
"#,
        rule: "FN-8",
        sentences: &[
            "\n  disposition: Unproved\n",
            "\n  mechanical_fix: add `requires y < 10_u64;` to the `contract` of `caller`, which each caller then establishes; or guard the call with `if y < 10_u64` where skipping it is the intended behavior\n",
        ],
        repaired: &[
            br#"fn small(x: u64) -> result: u64 pure contract {
  requires x < 10_u64;
} {
  return x;
}

fn caller(y: u64) -> result: u64 pure contract {
  requires y < 10_u64;
} {
  let r = small(x: y);
  return r;
}

fn main() -> status: ExitStatus pure {
  let r = caller(y: 3_u64);
  return exit_status(code: 0_u8);
}
"#,
            br#"fn small(x: u64) -> result: u64 pure contract {
  requires x < 10_u64;
} {
  return x;
}

fn caller(y: u64) -> result: u64 pure {
  if y < 10_u64 {
    let r = small(x: y);
    return r;
  }
  return 0_u64;
}

fn main() -> status: ExitStatus pure {
  let r = caller(y: 3_u64);
  return exit_status(code: 0_u8);
}
"#,
        ],
    },
    RepairPair {
        name: "call-requirement-over-a-computed-value.wf",
        rejected: br#"fn small(x: u64) -> result: u64 pure contract {
  requires x < 10_u64;
} {
  return x;
}

fn clamp(y: u64) -> result: u64 pure {
  if y < 10_u64 {
    return y;
  }
  return 9_u64;
}

fn caller(y: u64) -> result: u64 pure {
  let z = clamp(y: y);
  let r = small(x: z);
  return r;
}

fn main() -> status: ExitStatus pure {
  let r = caller(y: 3_u64);
  return exit_status(code: 0_u8);
}
"#,
        rule: "FN-8",
        sentences: &[
            "\n  disposition: Unproved\n",
            "\n  mechanical_fix: `z < 10_u64` is not proved before this call: when facts that reach the call imply it, prove it with an `invariant` whose `use` steps name them (a loop's header `invariant` for a value the loop computes); when the callee whose result it reads can prove the bound, state it in that callee's `ensures`; or guard the call with `if z < 10_u64` where skipping it is the intended behavior\n",
        ],
        repaired: &[
            br#"fn small(x: u64) -> result: u64 pure contract {
  requires x < 10_u64;
} {
  return x;
}

fn clamp(y: u64) -> result: u64 pure contract {
  ensures result < 10_u64;
} {
  if y < 10_u64 {
    return y;
  }
  return 9_u64;
}

fn caller(y: u64) -> result: u64 pure {
  let z = clamp(y: y);
  let r = small(x: z);
  return r;
}

fn main() -> status: ExitStatus pure {
  let r = caller(y: 3_u64);
  return exit_status(code: 0_u8);
}
"#,
            br#"fn small(x: u64) -> result: u64 pure contract {
  requires x < 10_u64;
} {
  return x;
}

fn clamp(y: u64) -> result: u64 pure {
  if y < 10_u64 {
    return y;
  }
  return 9_u64;
}

fn caller(y: u64) -> result: u64 pure {
  let z = clamp(y: y);
  if z < 10_u64 {
    let r = small(x: z);
    return r;
  }
  return 0_u64;
}

fn main() -> status: ExitStatus pure {
  let r = caller(y: 3_u64);
  return exit_status(code: 0_u8);
}
"#,
        ],
    },
    RepairPair {
        // A field of an element read through a range reference is no admitted
        // value [ENT-3], so the goal keeps the argument's occurrence-local
        // value, which DIAG-1 spells for the payload.
        name: "call-requirement-over-an-argument-evaluated-in-the-call.wf",
        rejected: br#"struct Stats {
  count: u64;
}

fn small(x: u64) -> result: u64 pure contract {
  requires x < 10_u64;
} {
  return x;
}

fn first(view: &[Stats]) -> result: u64 reads(view) contract {
  requires 1_u64 <= deref(view).len;
} {
  let r = small(x: deref(view)[0_u64].count);
  return r;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "FN-8",
        sentences: &[
            "\n  instantiated_goal: argument #0 pre-transfer value < 10_u64\n",
            "\n  mechanical_fix: argument #0 is evaluated inside the call, where no fact names its value: bind it with one preceding `let`, establish the requirement over that binding, and pass the binding, borrowing it when the parameter is a reference\n",
        ],
        repaired: &[br#"struct Stats {
  count: u64;
}

fn small(x: u64) -> result: u64 pure contract {
  requires x < 10_u64;
} {
  return x;
}

fn first(view: &[Stats]) -> result: u64 reads(view) contract {
  requires 1_u64 <= deref(view).len;
} {
  let c = deref(view)[0_u64].count;
  if c < 10_u64 {
    let r = small(x: c);
    return r;
  }
  return 0_u64;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#],
    },
    RepairPair {
        name: "call-requirement-over-a-range-formed-at-the-call.wf",
        rejected: br#"fn need(v: &[u64]) -> result: u64 pure contract {
  requires 2_u64 <= deref(v).len;
} {
  return 0_u64;
}

fn caller(k: u64) -> result: u64 pure {
  let values = array_filled::<u64, 4>(value: 0_u64);
  if k <= 4_u64 {
    let r = need(v: &values[0_u64..k]);
    return r;
  }
  return 0_u64;
}

fn main() -> status: ExitStatus pure {
  let r = caller(k: 3_u64);
  return exit_status(code: 0_u8);
}
"#,
        rule: "FN-8",
        sentences: &[
            "\n  disposition: Unproved\n",
            "` reads a value no fact can name until a `let` binds it: bind that value with one preceding `let`, use the binding in the call, and establish the relation over the binding\n",
        ],
        repaired: &[br#"fn need(v: &[u64]) -> result: u64 pure contract {
  requires 2_u64 <= deref(v).len;
} {
  return 0_u64;
}

fn caller(k: u64) -> result: u64 pure {
  let values = array_filled::<u64, 4>(value: 0_u64);
  if k <= 4_u64 {
    let part = &values[0_u64..k];
    if 2_u64 <= deref(part).len {
      let r = need(v: part);
      return r;
    }
  }
  return 0_u64;
}

fn main() -> status: ExitStatus pure {
  let r = caller(k: 3_u64);
  return exit_status(code: 0_u8);
}
"#],
    },
    // -------------------------------------------------------------------
    // [FN-9] a normal-result relation at one selected return.
    // -------------------------------------------------------------------
    RepairPair {
        name: "postcondition-refuted.wf",
        rejected: br#"fn f() -> result: u64 pure contract {
  ensures result < 10_u64;
} {
  return 20_u64;
}

fn main() -> status: ExitStatus pure {
  let r = f();
  return exit_status(code: 0_u8);
}
"#,
        rule: "FN-9",
        sentences: &[
            "\n  disposition: Refuted\n",
            "\n  mechanical_fix: the value this `return` delivers makes the postcondition false: return a value that satisfies it, state a postcondition this return satisfies, or change the requirements that fix the returned value\n",
        ],
        repaired: &[
            br#"fn f() -> result: u64 pure contract {
  ensures result < 10_u64;
} {
  return 5_u64;
}

fn main() -> status: ExitStatus pure {
  let r = f();
  return exit_status(code: 0_u8);
}
"#,
            br#"fn f() -> result: u64 pure contract {
  ensures result <= 20_u64;
} {
  return 20_u64;
}

fn main() -> status: ExitStatus pure {
  let r = f();
  return exit_status(code: 0_u8);
}
"#,
        ],
    },
    RepairPair {
        name: "postcondition-unproved.wf",
        rejected: br#"fn f(x: u64) -> result: u64 pure contract {
  ensures result < 10_u64;
} {
  return x;
}

fn main() -> status: ExitStatus pure {
  let r = f(x: 3_u64);
  return exit_status(code: 0_u8);
}
"#,
        rule: "FN-9",
        sentences: &[
            "\n  disposition: Unproved\n",
            // The returned value is a parameter and no call returns anything
            // here, so no callee's `ensures` is offered.
            "\n  mechanical_fix: the postcondition is not proved where this `return` delivers its value: add a `requires` over the parameters the value is computed from, prove the bound before the return with an `invariant` whose `use` steps name the facts it follows from, or state a postcondition the body proves\n",
        ],
        repaired: &[br#"fn f(x: u64) -> result: u64 pure contract {
  requires x < 10_u64;
  ensures result < 10_u64;
} {
  return x;
}

fn main() -> status: ExitStatus pure {
  let r = f(x: 3_u64);
  return exit_status(code: 0_u8);
}
"#],
    },
    RepairPair {
        // The returned value is a call's result, so the callee's `ensures`
        // is the route that bounds it.
        name: "postcondition-over-a-call-result.wf",
        rejected: br#"fn clamp(y: u64) -> result: u64 pure {
  if y < 10_u64 {
    return y;
  }
  return 9_u64;
}

fn f(x: u64) -> result: u64 pure contract {
  ensures result < 10_u64;
} {
  let z = clamp(y: x);
  return z;
}

fn main() -> status: ExitStatus pure {
  let r = f(x: 3_u64);
  return exit_status(code: 0_u8);
}
"#,
        rule: "FN-9",
        sentences: &[
            "\n  disposition: Unproved\n",
            "\n  mechanical_fix: the postcondition is not proved where this `return` delivers its value: add a `requires` over the parameters the value is computed from, prove the bound before the return with an `invariant` whose `use` steps name the facts it follows from, state it in the `ensures` of the callee whose result the value reads when that callee can prove it, or state a postcondition the body proves\n",
        ],
        repaired: &[br#"fn clamp(y: u64) -> result: u64 pure contract {
  ensures result < 10_u64;
} {
  if y < 10_u64 {
    return y;
  }
  return 9_u64;
}

fn f(x: u64) -> result: u64 pure contract {
  ensures result < 10_u64;
} {
  let z = clamp(y: x);
  return z;
}

fn main() -> status: ExitStatus pure {
  let r = f(x: 3_u64);
  return exit_status(code: 0_u8);
}
"#],
    },
    RepairPair {
        name: "postcondition-with-no-selected-return.wf",
        rejected: br#"fn only_error(value: i32) -> out: Result<i32, i32> pure contract {
  ensures when Ok(value: payload): payload == value;
} {
  return Err<i32, i32>(error: value);
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "FN-9",
        sentences: &[
            "\n  residual: no selected normal exit\n  mechanical_fix: no `return` of this function delivers a value this clause's route selects: return such a value on some path, or delete the clause\n",
        ],
        repaired: &[
            br#"fn only_error(value: i32) -> out: Result<i32, i32> pure contract {
  ensures when Ok(value: payload): payload == value;
} {
  return Ok<i32, i32>(value: value);
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
            br#"fn only_error(value: i32) -> out: Result<i32, i32> pure {
  return Err<i32, i32>(error: value);
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        ],
    },
    // -------------------------------------------------------------------
    // [OP-2] an exact integer operation's `.defined` domain.
    // -------------------------------------------------------------------
    RepairPair {
        name: "integer-domain-refuted.wf",
        rejected: br#"fn main() -> status: ExitStatus pure {
  let y = 255_u8 + 1_u8;
  return exit_status(code: y);
}
"#,
        rule: "OP-2",
        sentences: &[
            "\n  disposition: Refuted\n",
            "\n  mechanical_fix: the operands that reach this operation make `255_u8 +defined 1_u8` false, so the exact operation cannot execute here: change the operands or their type, or write the `+wrap`, `+checked` or `+sat` form for the result the program intends\n",
        ],
        repaired: &[
            br#"fn main() -> status: ExitStatus pure {
  let y = 254_u8 + 1_u8;
  return exit_status(code: y);
}
"#,
            br#"fn main() -> status: ExitStatus pure {
  let y = 255_u8 +wrap 1_u8;
  return exit_status(code: y);
}
"#,
        ],
    },
    RepairPair {
        name: "integer-domain-over-parameters.wf",
        rejected: br#"fn bump(x: u8) -> result: u8 pure {
  let y = x + 1_u8;
  return y;
}

fn main() -> status: ExitStatus pure {
  let r = bump(x: 7_u8);
  return exit_status(code: r);
}
"#,
        rule: "OP-2",
        sentences: &[
            "\n  disposition: Unproved\n",
            "\n  mechanical_fix: add `requires x +defined 1_u8;` to the `contract` of `bump`, which each caller then establishes; or guard the operation with `if x +defined 1_u8` where skipping it is the intended behavior; or write the `+wrap`, `+checked` or `+sat` form\n",
        ],
        repaired: &[
            br#"fn bump(x: u8) -> result: u8 pure contract {
  requires x +defined 1_u8;
} {
  let y = x + 1_u8;
  return y;
}

fn main() -> status: ExitStatus pure {
  let r = bump(x: 7_u8);
  return exit_status(code: r);
}
"#,
            br#"fn bump(x: u8) -> result: u8 pure {
  if x +defined 1_u8 {
    let y = x + 1_u8;
    return y;
  }
  return 0_u8;
}

fn main() -> status: ExitStatus pure {
  let r = bump(x: 7_u8);
  return exit_status(code: r);
}
"#,
            br#"fn bump(x: u8) -> result: u8 pure {
  let y = x +wrap 1_u8;
  return y;
}

fn main() -> status: ExitStatus pure {
  let r = bump(x: 7_u8);
  return exit_status(code: r);
}
"#,
        ],
    },
    RepairPair {
        // [ENT-5] a write kills facts on its own path only: the arm that
        // writes `deref(p)` comes first, and the goal in its sibling still
        // reads the entry value, which a requirement describes.
        name: "integer-domain-in-the-arm-after-a-sibling-write.wf",
        rejected: br#"fn bump(p: &u64, flag: Bool) -> result: u64 writes(p) {
  if flag {
    set deref(p) = 0_u64;
    return 0_u64;
  } else {
    let v = deref(p) + 1_u64;
    return v;
  }
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "OP-2",
        sentences: &[
            "\n  disposition: Unproved\n",
            "\n  mechanical_fix: add `requires deref(p) +defined 1_u64;` to the `contract` of `bump`, which each caller then establishes; or guard the operation with `if deref(p) +defined 1_u64` where skipping it is the intended behavior, adding to the effect row any read that condition makes which the row does not yet declare; or write the `+wrap`, `+checked` or `+sat` form\n",
        ],
        repaired: &[br#"fn bump(p: &u64, flag: Bool) -> result: u64 writes(p) contract {
  requires deref(p) +defined 1_u64;
} {
  if flag {
    set deref(p) = 0_u64;
    return 0_u64;
  } else {
    let v = deref(p) + 1_u64;
    return v;
  }
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#],
    },
    RepairPair {
        // The same program with its arms in the other order selects the same
        // repair: which arm the walk visits first changes nothing.
        name: "integer-domain-in-the-arm-before-a-sibling-write.wf",
        rejected: br#"fn bump(p: &u64, flag: Bool) -> result: u64 writes(p) {
  if flag {
    let v = deref(p) + 1_u64;
    return v;
  } else {
    set deref(p) = 0_u64;
    return 0_u64;
  }
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "OP-2",
        sentences: &[
            "\n  disposition: Unproved\n",
            "\n  mechanical_fix: add `requires deref(p) +defined 1_u64;` to the `contract` of `bump`, which each caller then establishes; or guard the operation with `if deref(p) +defined 1_u64` where skipping it is the intended behavior, adding to the effect row any read that condition makes which the row does not yet declare; or write the `+wrap`, `+checked` or `+sat` form\n",
        ],
        repaired: &[br#"fn bump(p: &u64, flag: Bool) -> result: u64 writes(p) contract {
  requires deref(p) +defined 1_u64;
} {
  if flag {
    let v = deref(p) + 1_u64;
    return v;
  } else {
    set deref(p) = 0_u64;
    return 0_u64;
  }
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#],
    },
    RepairPair {
        name: "integer-domain-over-a-loop-value.wf",
        rejected: br#"fn total() -> result: u64 pure {
  let sum = 0_u64;
  for (i in 0_u64..4_u64) {
    set sum = sum + i;
  }
  return sum;
}

fn main() -> status: ExitStatus pure {
  let r = total();
  return exit_status(code: 0_u8);
}
"#,
        rule: "OP-2",
        sentences: &[
            "\n  disposition: Unproved\n",
            "\n  mechanical_fix: `sum +defined i` is not proved here: when facts that reach the operation imply it, prove it with an `invariant` whose `use` steps name them (a loop's header `invariant` for a value the loop computes); or guard the operation with `if sum +defined i` where skipping it is the intended behavior; or write the `+wrap`, `+checked` or `+sat` form\n",
        ],
        repaired: &[
            br#"fn total() -> result: u64 pure {
  let sum = 0_u64;
  for (
    i in 0_u64..4_u64,
    invariant bounded: sum <= 3_u64 * i
  ) {
    set sum = sum + i;
  }
  return sum;
}

fn main() -> status: ExitStatus pure {
  let r = total();
  return exit_status(code: 0_u8);
}
"#,
            br#"fn total() -> result: u64 pure {
  let sum = 0_u64;
  for (i in 0_u64..4_u64) {
    if sum +defined i {
      set sum = sum + i;
    }
  }
  return sum;
}

fn main() -> status: ExitStatus pure {
  let r = total();
  return exit_status(code: 0_u8);
}
"#,
            br#"fn total() -> result: u64 pure {
  let sum = 0_u64;
  for (i in 0_u64..4_u64) {
    set sum = sum +wrap i;
  }
  return sum;
}

fn main() -> status: ExitStatus pure {
  let r = total();
  return exit_status(code: 0_u8);
}
"#,
        ],
    },
    RepairPair {
        // An element read is no term [ENT-2], but it is part of the goal's
        // identity, which a condition naming the same expression establishes
        // [ENT-3].
        name: "integer-domain-over-an-element.wf",
        rejected: br#"fn bump(values: &Array<u8, 2>) -> result: u8 reads(values) {
  let y = deref(values)[0_u64] + 1_u8;
  return y;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "OP-2",
        sentences: &[
            "\n  disposition: Unproved\n",
            "\n  mechanical_fix: `deref(values)[0_u64] +defined 1_u8` is not proved here: when facts that reach the operation imply it, prove it with an `invariant` whose `use` steps name them (a loop's header `invariant` for a value the loop computes); or guard the operation with `if deref(values)[0_u64] +defined 1_u8` where skipping it is the intended behavior, adding to the effect row any read that condition makes which the row does not yet declare; or write the `+wrap`, `+checked` or `+sat` form\n",
        ],
        repaired: &[br#"fn bump(values: &Array<u8, 2>) -> result: u8 reads(values) {
  if deref(values)[0_u64] +defined 1_u8 {
    let y = deref(values)[0_u64] + 1_u8;
    return y;
  }
  return 0_u8;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#],
    },
    // -------------------------------------------------------------------
    // [OP-6] a bare conversion's domain.
    // -------------------------------------------------------------------
    RepairPair {
        name: "conversion-domain-refuted.wf",
        rejected: br#"fn main() -> status: ExitStatus pure {
  let b = cvt::<u32, u8>(256_u32);
  return exit_status(code: b);
}
"#,
        rule: "OP-6",
        sentences: &[
            "\n  disposition: Refuted\n",
            "\n  mechanical_fix: the value that reaches this conversion is outside `u8`: convert a value `u8` holds, choose a destination type that holds this one, or use `cvt.checked::<u32, u8>` and handle its `Err`\n",
        ],
        repaired: &[
            br#"fn main() -> status: ExitStatus pure {
  let b = cvt::<u32, u8>(255_u32);
  return exit_status(code: b);
}
"#,
            br#"fn main() -> status: ExitStatus pure {
  let b = cvt.checked::<u32, u8>(256_u32);
  match b {
    Ok(value: v) => {
      return exit_status(code: v);
    }
    Err(error: e) => {
      return exit_status(code: 1_u8);
    }
  }
}
"#,
        ],
    },
    RepairPair {
        name: "conversion-domain-over-parameters.wf",
        rejected: br#"fn narrow(x: u32) -> result: u8 pure {
  return cvt::<u32, u8>(x);
}

fn main() -> status: ExitStatus pure {
  let r = narrow(x: 7_u32);
  return exit_status(code: r);
}
"#,
        rule: "OP-6",
        sentences: &[
            "\n  disposition: Unproved\n",
            "\n  mechanical_fix: add `requires cvt.defined::<u32, u8>(x);` to the `contract` of `narrow`, which each caller then establishes; or guard the conversion with `if cvt.defined::<u32, u8>(x)` where skipping it is the intended behavior; or use `cvt.checked::<u32, u8>` and handle its `Err`\n",
        ],
        repaired: &[
            br#"fn narrow(x: u32) -> result: u8 pure contract {
  requires cvt.defined::<u32, u8>(x);
} {
  return cvt::<u32, u8>(x);
}

fn main() -> status: ExitStatus pure {
  let r = narrow(x: 7_u32);
  return exit_status(code: r);
}
"#,
            br#"fn narrow(x: u32) -> result: u8 pure {
  if cvt.defined::<u32, u8>(x) {
    return cvt::<u32, u8>(x);
  }
  return 0_u8;
}

fn main() -> status: ExitStatus pure {
  let r = narrow(x: 7_u32);
  return exit_status(code: r);
}
"#,
            br#"fn narrow(x: u32) -> result: u8 pure {
  let b = cvt.checked::<u32, u8>(x);
  match b {
    Ok(value: v) => {
      return v;
    }
    Err(error: e) => {
      return 0_u8;
    }
  }
}

fn main() -> status: ExitStatus pure {
  let r = narrow(x: 7_u32);
  return exit_status(code: r);
}
"#,
        ],
    },
    RepairPair {
        name: "conversion-domain-over-a-computed-integer.wf",
        rejected: br#"fn widen(x: u32) -> result: u32 pure {
  return x;
}

fn narrow(x: u32) -> result: u8 pure {
  let v = widen(x: x);
  return cvt::<u32, u8>(v);
}

fn main() -> status: ExitStatus pure {
  let r = narrow(x: 7_u32);
  return exit_status(code: r);
}
"#,
        rule: "OP-6",
        sentences: &[
            "\n  disposition: Unproved\n",
            "\n  mechanical_fix: `cvt.defined::<u32, u8>(v)` is not proved here: when facts that reach the conversion imply it, prove it with an `invariant` whose `use` steps name them (a loop's header `invariant` for a value the loop computes); when the callee whose result it reads can prove the bound, state it in that callee's `ensures`; or guard the conversion with `if cvt.defined::<u32, u8>(v)` where skipping it is the intended behavior; or use `cvt.checked::<u32, u8>` and handle its `Err`\n",
        ],
        repaired: &[br#"fn widen(x: u32) -> result: u32 pure {
  return x;
}

fn narrow(x: u32) -> result: u8 pure {
  let v = widen(x: x);
  if cvt.defined::<u32, u8>(v) {
    return cvt::<u32, u8>(v);
  }
  return 0_u8;
}

fn main() -> status: ExitStatus pure {
  let r = narrow(x: 7_u32);
  return exit_status(code: r);
}
"#],
    },
    RepairPair {
        name: "conversion-domain-over-a-computed-float.wf",
        rejected: br#"fn pick(a: f64) -> result: f64 pure {
  return a;
}

fn round(a: f64) -> result: i32 pure {
  let v = pick(a: a);
  return cvt::<f64, i32>(v);
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "OP-6",
        sentences: &[
            "\n  disposition: Unproved\n",
            "\n  mechanical_fix: `cvt.defined::<f64, i32>(v)` is not proved here, and no fact bounds a float operand: guard the conversion with `if cvt.defined::<f64, i32>(v)` where skipping it is the intended behavior, or use `cvt.checked::<f64, i32>` and handle its `Err`\n",
        ],
        repaired: &[br#"fn pick(a: f64) -> result: f64 pure {
  return a;
}

fn round(a: f64) -> result: i32 pure {
  let v = pick(a: a);
  if cvt.defined::<f64, i32>(v) {
    return cvt::<f64, i32>(v);
  }
  return 0_i32;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#],
    },
    // -------------------------------------------------------------------
    // [OP-4] a subscript's bound.
    // -------------------------------------------------------------------
    RepairPair {
        name: "bounds-refuted.wf",
        rejected: br#"fn main() -> status: ExitStatus pure {
  let values = array_filled::<u8, 4>(value: 0_u8);
  let v = values[5_u64];
  return exit_status(code: v);
}
"#,
        rule: "OP-4",
        sentences: &[
            "\n  disposition: Refuted\n",
            "\n  mechanical_fix: `5_u64 < values.len` is false where this access executes: index within the storage, or give the storage a length that holds this index\n",
        ],
        repaired: &[
            br#"fn main() -> status: ExitStatus pure {
  let values = array_filled::<u8, 4>(value: 0_u8);
  let v = values[3_u64];
  return exit_status(code: v);
}
"#,
            br#"fn main() -> status: ExitStatus pure {
  let values = array_filled::<u8, 6>(value: 0_u8);
  let v = values[5_u64];
  return exit_status(code: v);
}
"#,
        ],
    },
    RepairPair {
        // An offset that grows with the storage, such as its own length, is
        // out of range at every length, so no longer storage is offered: the
        // statements that fix the offset are what the repair changes.
        name: "bounds-refuted-at-an-offset-that-is-no-constant.wf",
        rejected: br#"fn main() -> status: ExitStatus pure {
  let values = array_filled::<u8, 4>(value: 0_u8);
  let k = values.len;
  let v = values[k];
  return exit_status(code: v);
}
"#,
        rule: "OP-4",
        sentences: &[
            "\n  disposition: Refuted\n",
            "\n  mechanical_fix: `k < values.len` is false where this access executes: index within the storage, or change the statements or requirements that fix the index\n",
        ],
        repaired: &[br#"fn main() -> status: ExitStatus pure {
  let values = array_filled::<u8, 4>(value: 0_u8);
  let k = values.len - 1_u64;
  let v = values[k];
  return exit_status(code: v);
}
"#],
    },
    RepairPair {
        name: "bounds-over-parameters.wf",
        rejected: br#"fn get(b: &[u8], i: u64) -> result: u8 reads(b) {
  return deref(b)[i];
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "OP-4",
        sentences: &[
            "\n  disposition: Unproved\n",
            "\n  mechanical_fix: add `requires i < deref(b).len;` to the `contract` of `get`, which each caller then establishes; or guard the access with `if i < deref(b).len` where skipping it is the intended behavior, adding to the effect row any read that condition makes which the row does not yet declare\n",
        ],
        repaired: &[
            br#"fn get(b: &[u8], i: u64) -> result: u8 reads(b) contract {
  requires i < deref(b).len;
} {
  return deref(b)[i];
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
            br#"fn get(b: &[u8], i: u64) -> result: u8 reads(b) {
  if i < deref(b).len {
    return deref(b)[i];
  }
  return 0_u8;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        ],
    },
    RepairPair {
        name: "bounds-over-a-computed-offset.wf",
        rejected: br#"fn widen(x: u64) -> result: u64 pure {
  return x;
}

fn get(b: &[u8], i: u64) -> result: u8 reads(b) {
  let k = widen(x: i);
  return deref(b)[k];
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "OP-4",
        sentences: &[
            "\n  disposition: Unproved\n",
            "\n  mechanical_fix: `k < deref(b).len` is not proved here: when facts that reach the access imply it, prove it with an `invariant` whose `use` steps name them (a loop's header `invariant` for a value the loop computes); when the callee whose result it reads can prove the bound, state it in that callee's `ensures`; or guard the access with `if k < deref(b).len` where skipping it is the intended behavior, adding to the effect row any read that condition makes which the row does not yet declare\n",
        ],
        repaired: &[br#"fn widen(x: u64) -> result: u64 pure {
  return x;
}

fn get(b: &[u8], i: u64) -> result: u8 reads(b) {
  let k = widen(x: i);
  if k < deref(b).len {
    return deref(b)[k];
  }
  return 0_u8;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#],
    },
    RepairPair {
        // An offset that is itself an element is no term [ENT-2], and a
        // bound names terms alone, so no condition over it establishes one.
        name: "bounds-over-an-element-offset.wf",
        rejected: br#"fn pick(order: &[u64], lens: &[u8], j: u64) -> result: u8 reads(order), reads(lens) contract {
  requires j < deref(order).len;
} {
  return deref(lens)[deref(order)[j]];
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "OP-4",
        sentences: &[
            "\n  disposition: Unproved\n",
            "\n  mechanical_fix: `deref(order)[j] < deref(lens).len` reads a value no fact can name until a `let` binds it: bind that value with one preceding `let`, use the binding in the access, and establish the relation over the binding\n",
        ],
        repaired: &[br#"fn pick(order: &[u64], lens: &[u8], j: u64) -> result: u8 reads(order), reads(lens) contract {
  requires j < deref(order).len;
} {
  let k = deref(order)[j];
  if k < deref(lens).len {
    return deref(lens)[k];
  }
  return 0_u8;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#],
    },
    RepairPair {
        // [ENT-2, FN-8] a requirement forms its places at body entry, in the
        // state the requirements written before it build, and evaluates
        // nothing: the bound comes from an earlier requirement, and no guard
        // can skip a clause.
        name: "bounds-of-a-place-a-requirement-forms.wf",
        rejected: br#"fn pick(rows: &Slots<Slots<u8, 8>, 4>, i: u64, k: u64) -> value: u8 reads(rows) contract {
  requires k < deref(rows)[i].len;
} {
  return deref(rows)[i][k];
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "OP-4",
        sentences: &[
            "\n  residual: i < deref(rows).len\n  disposition: Unproved\n",
            "\n  mechanical_fix: add `requires i < deref(rows).len;` to the `contract` of `pick` ahead of the requirement that forms this place, which each caller then establishes\n",
        ],
        repaired: &[br#"fn pick(rows: &Slots<Slots<u8, 8>, 4>, i: u64, k: u64) -> value: u8 reads(rows) contract {
  requires i < deref(rows).len;
  requires k < deref(rows)[i].len;
} {
  return deref(rows)[i][k];
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#],
    },
    RepairPair {
        // The earlier requirement makes `i` the length, so the place the
        // second requirement forms is refuted there: the requirements before
        // it are what fix the index.
        name: "bounds-refuted-in-a-place-a-requirement-forms.wf",
        rejected: br#"fn pick(rows: &Slots<Slots<u8, 8>, 4>, i: u64, k: u64) -> value: u8 reads(rows) contract {
  requires i == deref(rows).len;
  requires k < deref(rows)[i].len;
} {
  return deref(rows)[i][k];
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "OP-4",
        sentences: &[
            "\n  residual: i < deref(rows).len\n  disposition: Refuted\n",
            "\n  mechanical_fix: `i < deref(rows).len` is false where this place is formed: index within the storage, or change the requirements before this one that fix the index\n",
        ],
        repaired: &[br#"fn pick(rows: &Slots<Slots<u8, 8>, 4>, i: u64, k: u64) -> value: u8 reads(rows) contract {
  requires i < deref(rows).len;
  requires k < deref(rows)[i].len;
} {
  return deref(rows)[i][k];
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#],
    },
    // -------------------------------------------------------------------
    // [OP-9] an allocation's size.
    // -------------------------------------------------------------------
    // Each OP-9 repair names the ceiling as the language's limit for the
    // element type and asks for the count the program needs, because the
    // selected target admits less [STOR-6]; every repaired program here
    // states such a count and builds [`every_allocation_repair_builds`].
    RepairPair {
        name: "allocation-fit-refuted.wf",
        rejected: br#"fn main() -> status: ExitStatus pure {
  let block = box_slots_new::<i64>(capacity: 18446744073709551615_u64);
  return exit_status(code: 0_u8);
}
"#,
        rule: "OP-9",
        sentences: &[
            "\n  disposition: Refuted\n",
            "\n  mechanical_fix: `18446744073709551615_u64 <= 2305843009213693951_u64` is false, so this allocation cannot be formed: request the count the program needs. `2305843009213693951_u64` is the language's limit for this element type, not a bound to write: the selected target admits a smaller count, so a bound at or near that limit stops at target layout [STOR-6]\n",
        ],
        repaired: &[br#"fn main() -> status: ExitStatus pure {
  let block = box_slots_new::<i64>(capacity: 4_u64);
  return exit_status(code: 0_u8);
}
"#],
    },
    RepairPair {
        name: "allocation-fit-over-parameters.wf",
        rejected: br#"fn make(length: u64) -> values: Box<Slots<i64>> pure {
  let block = box_slots_new::<i64>(capacity: length);
  return move block;
}

fn main() -> status: ExitStatus pure {
  let values = make(length: 4_u64);
  return exit_status(code: 0_u8);
}
"#,
        rule: "OP-9",
        sentences: &[
            "\n  disposition: Unproved\n",
            "\n  mechanical_fix: with N the largest count the program needs, add `requires length <= N;` to the `contract` of `make`, which each caller then establishes; or guard the allocation with `if length <= N` where refusing a larger count is the intended behavior. `2305843009213693951_u64` is the language's limit for this element type, not a bound to write: the selected target admits a smaller count, so a bound at or near that limit stops at target layout [STOR-6]\n",
        ],
        repaired: &[
            br#"fn make(length: u64) -> values: Box<Slots<i64>> pure contract {
  requires length <= 1000_u64;
} {
  let block = box_slots_new::<i64>(capacity: length);
  return move block;
}

fn main() -> status: ExitStatus pure {
  let values = make(length: 4_u64);
  return exit_status(code: 0_u8);
}
"#,
            br#"fn make(length: u64) -> values: Box<Slots<i64>> pure {
  if length <= 1000_u64 {
    let block = box_slots_new::<i64>(capacity: length);
    return move block;
  }
  let empty = box_slots_new::<i64>(capacity: 0_u64);
  return move empty;
}

fn main() -> status: ExitStatus pure {
  let values = make(length: 4_u64);
  return exit_status(code: 0_u8);
}
"#,
        ],
    },
    RepairPair {
        name: "allocation-fit-over-a-computed-count.wf",
        rejected: br#"fn widen(x: u64) -> result: u64 pure {
  return x;
}

fn make(length: u64) -> values: Box<Slots<i64>> pure {
  let n = widen(x: length);
  let block = box_slots_new::<i64>(capacity: n);
  return move block;
}

fn main() -> status: ExitStatus pure {
  let values = make(length: 4_u64);
  return exit_status(code: 0_u8);
}
"#,
        rule: "OP-9",
        sentences: &[
            "\n  disposition: Unproved\n",
            "\n  mechanical_fix: `n` is not bounded here: with N the largest count the program needs, when facts that reach the allocation imply `n <= N`, prove it with an `invariant` whose `use` steps name them (a loop's header `invariant` for a value the loop computes); when the callee whose result it reads can prove that bound, state it in the callee's `ensures`; or guard the allocation with `if n <= N` where refusing a larger count is the intended behavior. `2305843009213693951_u64` is the language's limit for this element type, not a bound to write: the selected target admits a smaller count, so a bound at or near that limit stops at target layout [STOR-6]\n",
        ],
        repaired: &[
            br#"fn widen(x: u64) -> result: u64 pure contract {
  ensures result <= 1000_u64;
} {
  if x <= 1000_u64 {
    return x;
  }
  return 1000_u64;
}

fn make(length: u64) -> values: Box<Slots<i64>> pure {
  let n = widen(x: length);
  let block = box_slots_new::<i64>(capacity: n);
  return move block;
}

fn main() -> status: ExitStatus pure {
  let values = make(length: 4_u64);
  return exit_status(code: 0_u8);
}
"#,
            br#"fn widen(x: u64) -> result: u64 pure {
  return x;
}

fn make(length: u64) -> values: Box<Slots<i64>> pure {
  let n = widen(x: length);
  if n <= 1000_u64 {
    let block = box_slots_new::<i64>(capacity: n);
    return move block;
  }
  let empty = box_slots_new::<i64>(capacity: 0_u64);
  return move empty;
}

fn main() -> status: ExitStatus pure {
  let values = make(length: 4_u64);
  return exit_status(code: 0_u8);
}
"#,
        ],
    },
    // -------------------------------------------------------------------
    // [REF-4] one range-formation conjunct.
    // -------------------------------------------------------------------
    RepairPair {
        name: "range-formation-refuted.wf",
        rejected: br#"fn main() -> status: ExitStatus pure {
  let values = array_filled::<u64, 4>(value: 0_u64);
  let view = &values[0_u64..5_u64];
  return exit_status(code: 0_u8);
}
"#,
        rule: "REF-4",
        sentences: &[
            "\n  disposition: Refuted\n",
            "` is false where this range is formed: choose endpoints that satisfy it\n",
        ],
        repaired: &[br#"fn main() -> status: ExitStatus pure {
  let values = array_filled::<u64, 4>(value: 0_u64);
  let view = &values[0_u64..4_u64];
  return exit_status(code: 0_u8);
}
"#],
    },
    RepairPair {
        name: "range-formation-over-parameters.wf",
        rejected: br#"fn part(values: &[u64], hi: u64) -> result: u64 pure {
  let view = &deref(values)[0_u64..hi];
  return 0_u64;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "REF-4",
        sentences: &[
            "\n  disposition: Unproved\n",
            "\n  mechanical_fix: add `requires hi <= deref(values).len;` to the `contract` of `part`, which each caller then establishes; or guard the range with `if hi <= deref(values).len` where skipping it is the intended behavior, adding to the effect row any read that condition makes which the row does not yet declare\n",
        ],
        repaired: &[
            br#"fn part(values: &[u64], hi: u64) -> result: u64 pure contract {
  requires hi <= deref(values).len;
} {
  let view = &deref(values)[0_u64..hi];
  return 0_u64;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
            br#"fn part(values: &[u64], hi: u64) -> result: u64 reads(values.len) {
  if hi <= deref(values).len {
    let view = &deref(values)[0_u64..hi];
  }
  return 0_u64;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        ],
    },
    RepairPair {
        name: "range-formation-over-a-computed-endpoint.wf",
        rejected: br#"fn widen(x: u64) -> result: u64 pure {
  return x;
}

fn part(values: &[u64], hi: u64) -> result: u64 pure {
  let h = widen(x: hi);
  let view = &deref(values)[0_u64..h];
  return 0_u64;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "REF-4",
        sentences: &[
            "\n  disposition: Unproved\n",
            "\n  mechanical_fix: `h <= deref(values).len` is not proved here: when facts that reach the range imply it, prove it with an `invariant` whose `use` steps name them (a loop's header `invariant` for a value the loop computes); when the callee whose result it reads can prove the bound, state it in that callee's `ensures`; or guard the range with `if h <= deref(values).len` where skipping it is the intended behavior, adding to the effect row any read that condition makes which the row does not yet declare\n",
        ],
        repaired: &[br#"fn widen(x: u64) -> result: u64 pure {
  return x;
}

fn part(values: &[u64], hi: u64) -> result: u64 reads(values.len) {
  let h = widen(x: hi);
  if h <= deref(values).len {
    let view = &deref(values)[0_u64..h];
  }
  return 0_u64;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#],
    },
    // -------------------------------------------------------------------
    // [OP-14] `free_empty`'s requirement that the window is empty.
    // -------------------------------------------------------------------
    RepairPair {
        name: "empty-run-release-refuted.wf",
        rejected: br#"nodrop struct Token {
  value: u64;
}

fn main() -> status: ExitStatus pure {
  let r = slots_new::<Token, 2>();
  let t = Token(value: 1_u64);
  place_back(window: &r, value: move t);
  free_empty(window: move r);
  return exit_status(code: 0_u8);
}
"#,
        rule: "OP-14",
        sentences: &[
            "\n  disposition: Refuted\n",
            "\n  mechanical_fix: the window still holds elements here: take every element out and consume it before `free_empty`\n",
        ],
        repaired: &[br#"nodrop struct Token {
  value: u64;
}

fn main() -> status: ExitStatus pure {
  let r = slots_new::<Token, 2>();
  let t = Token(value: 1_u64);
  place_back(window: &r, value: move t);
  let back = take_back(window: &r);
  let Token(value: v) = move back;
  free_empty(window: move r);
  return exit_status(code: 0_u8);
}
"#],
    },
    RepairPair {
        name: "empty-run-release-over-a-parameter.wf",
        rejected: br#"fn release(window: Box<Slots<u8>>) -> result: unit pure {
  free_empty(window: move window);
  return unit;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "OP-14",
        sentences: &[
            "\n  disposition: Unproved\n",
            "\n  mechanical_fix: add `requires window.inner.len == 0_u64;` to the `contract` of `release`, which each caller then establishes, or take every element out and consume it before this call, so that its zero length is established here\n",
        ],
        repaired: &[br#"fn release(window: Box<Slots<u8>>) -> result: unit pure contract {
  requires window.inner.len == 0_u64;
} {
  free_empty(window: move window);
  return unit;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#],
    },
    // -------------------------------------------------------------------
    // [INV-1] local and loop invariants: refuted when the state derives the
    // negation of one of the target's bounds [MSR-4].
    // -------------------------------------------------------------------
    RepairPair {
        name: "local-invariant-refuted.wf",
        rejected: br#"fn main() -> status: ExitStatus pure {
  let x = 255_u8;
  invariant fits: x <= 254_u8;
  let y = x +wrap 1_u8;
  return exit_status(code: y);
}
"#,
        rule: "INV-1",
        sentences: &[
            "\n  disposition: Refuted\n",
            "\n  mechanical_fix: `fits` is false where it is stated: correct the relation, or state one that the facts reaching it imply\n",
        ],
        repaired: &[br#"fn main() -> status: ExitStatus pure {
  let x = 255_u8;
  invariant fits: x <= 255_u8;
  let y = x +wrap 1_u8;
  return exit_status(code: y);
}
"#],
    },
    RepairPair {
        name: "local-invariant-unproved.wf",
        rejected: br#"fn check(x: u8) -> result: u8 pure {
  invariant fits: x <= 254_u8;
  return x;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "INV-1",
        sentences: &[
            "\n  disposition: Unproved\n",
            "\n  mechanical_fix: `fits` is not proved from the facts that reach it: establish them before it, add `use` steps naming the facts it follows from, or weaken it\n",
        ],
        repaired: &[
            br#"fn check(x: u8) -> result: u8 pure contract {
  requires x <= 254_u8;
} {
  invariant fits: x <= 254_u8;
  return x;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
            br#"fn check(x: u8) -> result: u8 pure {
  invariant fits: x <= 255_u8;
  return x;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        ],
    },
    RepairPair {
        name: "loop-invariant-base-refuted.wf",
        rejected: br#"fn main() -> status: ExitStatus pure {
  let sum = 10_u64;
  for (
    i in 0_u64..4_u64,
    invariant low: sum <= 5_u64
  ) {
  }
  return exit_status(code: 0_u8);
}
"#,
        rule: "INV-1",
        sentences: &[
            "\n  obligation: Base\n",
            "\n  disposition: Refuted\n",
            "\n  mechanical_fix: `low` is false on entry to the loop: correct it, or change the values the loop starts from\n",
        ],
        repaired: &[
            br#"fn main() -> status: ExitStatus pure {
  let sum = 0_u64;
  for (
    i in 0_u64..4_u64,
    invariant low: sum <= 5_u64
  ) {
  }
  return exit_status(code: 0_u8);
}
"#,
            br#"fn main() -> status: ExitStatus pure {
  let sum = 10_u64;
  for (
    i in 0_u64..4_u64,
    invariant low: sum <= 10_u64
  ) {
  }
  return exit_status(code: 0_u8);
}
"#,
        ],
    },
    RepairPair {
        name: "loop-invariant-base-unproved.wf",
        rejected: br#"fn run(start: u64) -> result: u64 pure {
  let sum = start;
  for (
    i in 0_u64..4_u64,
    invariant low: sum <= 5_u64
  ) {
  }
  return sum;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "INV-1",
        sentences: &[
            "\n  obligation: Base\n",
            "\n  disposition: Unproved\n",
            "\n  mechanical_fix: `low` is not proved on entry to the loop: establish before the loop the facts it follows from, or weaken or correct it\n",
        ],
        repaired: &[br#"fn run(start: u64) -> result: u64 pure contract {
  requires start <= 5_u64;
} {
  let sum = start;
  for (
    i in 0_u64..4_u64,
    invariant low: sum <= 5_u64
  ) {
  }
  return sum;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#],
    },
    RepairPair {
        name: "loop-invariant-backedge-refuted.wf",
        rejected: br#"fn main() -> status: ExitStatus pure {
  let sum = 0_u64;
  for (
    i in 0_u64..4_u64,
    invariant fixed: sum <= 0_u64
  ) {
    set sum = sum + 1_u64;
  }
  return exit_status(code: 0_u8);
}
"#,
        rule: "INV-1",
        sentences: &[
            "\n  obligation: Backedge\n",
            "\n  disposition: Refuted\n",
            "\n  mechanical_fix: an iteration makes `fixed` false at the next loop header: correct it, or change the body so that every iteration preserves it\n",
        ],
        repaired: &[br#"fn main() -> status: ExitStatus pure {
  let sum = 0_u64;
  for (
    i in 0_u64..4_u64,
    invariant fixed: sum <= i
  ) {
    set sum = sum + 1_u64;
  }
  return exit_status(code: 0_u8);
}
"#],
    },
    RepairPair {
        name: "loop-invariant-backedge-unproved.wf",
        rejected: br#"fn run(step: u64) -> result: u64 pure {
  let sum = 0_u64;
  for (
    i in 0_u64..4_u64,
    invariant bounded: sum <= i
  ) {
    set sum = sum +wrap step;
  }
  return sum;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "INV-1",
        sentences: &[
            "\n  obligation: Backedge\n",
            "\n  disposition: Unproved\n",
            "\n  mechanical_fix: `bounded` is not proved preserved at the next loop header: strengthen the invariant prefix, weaken or correct it, or establish in the body the facts from which every reachable fallthrough preserves it\n",
        ],
        repaired: &[br#"fn run(step: u64) -> result: u64 pure contract {
  requires step <= 1_u64;
} {
  let sum = 0_u64;
  for (
    i in 0_u64..4_u64,
    invariant bounded: sum <= i
  ) {
    set sum = sum + step;
  }
  return sum;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#],
    },
    // -------------------------------------------------------------------
    // Repairs that no goal selects.
    // -------------------------------------------------------------------
    RepairPair {
        name: "declared-row-is-narrower-than-the-body.wf",
        rejected: br#"fn touch(data: &[u8]) -> out: u64 pure {
  return deref(data).len;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "EFF-2",
        sentences: &[
            "\n  mechanical_fix: declare the row as `reads(data.len)`, which covers every access the body makes and no other\n",
        ],
        repaired: &[br#"fn touch(data: &[u8]) -> out: u64 reads(data.len) {
  return deref(data).len;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#],
    },
    RepairPair {
        name: "row-writes-before-reads.wf",
        rejected: br#"struct Stats {
  count: u64;
  total: u64;
}

fn record(stats: &Stats) -> result: unit writes(stats.count), reads(stats.count) {
  let old = deref(stats).count;
  set deref(stats).count = old +wrap 1_u64;
  return unit;
}

fn main() -> status: ExitStatus pure {
  let stats = Stats(count: 0_u64, total: 0_u64);
  record(stats: &stats);
  return exit_status(code: 0_u8);
}
"#,
        rule: "EFF-1",
        sentences: &[
            "\n  mechanical_fix: write every `reads` entry before the first `writes` entry, and delete each entry whose path is a `writes` entry's path or lies below it, which that `writes` already covers\n",
        ],
        repaired: &[br#"struct Stats {
  count: u64;
  total: u64;
}

fn record(stats: &Stats) -> result: unit writes(stats.count) {
  let old = deref(stats).count;
  set deref(stats).count = old +wrap 1_u64;
  return unit;
}

fn main() -> status: ExitStatus pure {
  let stats = Stats(count: 0_u64, total: 0_u64);
  record(stats: &stats);
  return exit_status(code: 0_u8);
}
"#],
    },
    RepairPair {
        name: "call-passes-one-place-to-two-arguments.wf",
        rejected: br#"struct Cell {
  value: u64;
}

fn copy_across(source: &Cell, destination: &Cell) -> result: unit reads(source.value), writes(destination.value) {
  let v = deref(source).value;
  set deref(destination).value = v;
  return unit;
}

fn main() -> status: ExitStatus pure {
  let a = Cell(value: 4_u64);
  copy_across(source: &a, destination: &a);
  return exit_status(code: 0_u8);
}
"#,
        rule: "EFF-5",
        sentences: &[
            "\n  mechanical_fix: pass places that do not overlap, or pass the shared place through one argument only\n",
        ],
        repaired: &[br#"struct Cell {
  value: u64;
}

fn copy_across(source: &Cell, destination: &Cell) -> result: unit reads(source.value), writes(destination.value) {
  let v = deref(source).value;
  set deref(destination).value = v;
  return unit;
}

fn main() -> status: ExitStatus pure {
  let a = Cell(value: 4_u64);
  let b = Cell(value: 0_u64);
  copy_across(source: &a, destination: &b);
  return exit_status(code: 0_u8);
}
"#],
    },
    RepairPair {
        // [EFF-2] admits a row whose two entries overlap through one
        // parameter, and [EFF-5] refuses it at every call, so the repair is at
        // the callee's row, not at the call.
        name: "row-entries-overlapping-through-one-argument.wf",
        rejected: br#"struct Stats {
  count: u64;
  total: u64;
}

fn record(stats: &Stats) -> result: unit reads(stats), writes(stats.count) {
  let whole = deref(stats);
  set deref(stats).count = whole.total;
  return unit;
}

fn main() -> status: ExitStatus pure {
  let stats = Stats(count: 0_u64, total: 0_u64);
  record(stats: &stats);
  return exit_status(code: 0_u8);
}
"#,
        rule: "EFF-5",
        sentences: &[
            "\n  mechanical_fix: these two entries of the callee's row reach overlapping places through one argument, so every call rejects them: declare one `writes` entry of their common path in its row instead\n",
        ],
        repaired: &[br#"struct Stats {
  count: u64;
  total: u64;
}

fn record(stats: &Stats) -> result: unit writes(stats) {
  let whole = deref(stats);
  set deref(stats).count = whole.total;
  return unit;
}

fn main() -> status: ExitStatus pure {
  let stats = Stats(count: 0_u64, total: 0_u64);
  record(stats: &stats);
  return exit_status(code: 0_u8);
}
"#],
    },
    RepairPair {
        name: "swap-of-a-possible-ancestor.wf",
        rejected: br#"struct Node {
  value: u64;
  next: Option<Box<Node>>;
}

fn exchange(rows: &Array<Node, 2>, i: u64, j: u64) -> result: unit writes(rows) {
  if i < 2_u64 {
    if j < 2_u64 {
      let parent = &deref(rows)[j];
      match deref(parent).next {
        Some(value: child) => {
          swap(first: &deref(rows)[i], second: &deref(child).inner);
        }
        None() => {
        }
      }
    }
  }
  return unit;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "OP-11",
        sentences: &[
            "\n  mechanical_fix: prove before the `swap` that the two positions are distinct, or exchange places that are equal or disjoint without an ancestor relation\n",
        ],
        repaired: &[br#"struct Node {
  value: u64;
  next: Option<Box<Node>>;
}

fn exchange(rows: &Array<Node, 2>, i: u64, j: u64) -> result: unit writes(rows) {
  if i < j {
    if j < 2_u64 {
      let parent = &deref(rows)[j];
      match deref(parent).next {
        Some(value: child) => {
          swap(first: &deref(rows)[i], second: &deref(child).inner);
        }
        None() => {
        }
      }
    }
  }
  return unit;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#],
    },
    RepairPair {
        name: "value-match-whose-arms-all-return.wf",
        rejected: br#"fn choose(flag: Option<i32>) -> result: i32 pure {
  let picked = match flag {
    Some(value: inner) => {
      return inner;
    }
    None() => {
      return 0_i32;
    }
  }
  return picked;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "GIVE-1",
        sentences: &[
            "]: EmptyDeliverySet\n",
            "\n  binding: picked\n  mechanical_fix: every arm leaves by `return` or `break`, so no value reaches `picked`: drop `let picked =`, write the `match` as a statement, and delete the statements after it in this block, which no path reaches\n",
        ],
        repaired: &[br#"fn choose(flag: Option<i32>) -> result: i32 pure {
  match flag {
    Some(value: inner) => {
      return inner;
    }
    None() => {
      return 0_i32;
    }
  }
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#],
    },
    RepairPair {
        name: "set-of-an-undeclared-name.wf",
        rejected: br#"fn main() -> status: ExitStatus pure {
  set total = 1_u64;
  return exit_status(code: 0_u8);
}
"#,
        rule: "SET-1",
        sentences: &[
            "]: UndeclaredSetTarget\n",
            "\n  spelling: total\n  mechanical_fix: no binding `total` is in scope, so this `set` declares nothing: write it as `let total = ...;`, keeping its right-hand side, to declare the binding here, or name a binding that is in scope\n",
        ],
        repaired: &[br#"fn main() -> status: ExitStatus pure {
  let total = 1_u64;
  return exit_status(code: 0_u8);
}
"#],
    },
    RepairPair {
        name: "front-operation-on-a-slots-window.wf",
        rejected: br#"fn main() -> status: ExitStatus pure {
  let values = slots_new::<u8, 2>();
  place_front(window: &values, value: 7_u8);
  return exit_status(code: 0_u8);
}
"#,
        rule: "OP-10",
        sentences: &[
            "]: UnadmittedOperandShape\n",
            "\n  expected: a `Ring` operand, which is what this row admits\n  mechanical_fix: pass an operand of the admitted shape, or use an operation whose row admits this operand's shape\n",
        ],
        repaired: &[br#"fn main() -> status: ExitStatus pure {
  let values = slots_new::<u8, 2>();
  place_back(window: &values, value: 7_u8);
  return exit_status(code: 0_u8);
}
"#],
    },
    RepairPair {
        name: "bound-function-exceeds-the-formal-row.wf",
        rejected: br#"interface Disposer {
  fn release(factory: &HandleFactory, file: ReadFile) -> function_result: unit pure;
}

binding First : Disposer {
  release = release_read_file;
}

fn release_read_file(factory: &HandleFactory, file: ReadFile) -> function_result: unit writes(factory) {
  let closed = close_read(factory: factory, file: move file);
  return unit;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "FN-4",
        sentences: &[
            "]: BehaviorArgumentMismatch\n",
            "\n  mechanical_fix: supply a function whose signature, row and contract meet the formal interface, or weaken the formal interface to what the supplied function declares\n",
        ],
        repaired: &[br#"interface Disposer {
  fn release(factory: &HandleFactory, file: ReadFile) -> function_result: unit writes(factory);
}

binding First : Disposer {
  release = release_read_file;
}

fn release_read_file(factory: &HandleFactory, file: ReadFile) -> function_result: unit writes(factory) {
  let closed = close_read(factory: factory, file: move file);
  return unit;
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#],
    },
    RepairPair {
        name: "route-over-two-result-ordinals.wf",
        rejected: br#"fn probe(taken: Slots<u8, 8>) -> (outcome: Result<u64, Overflow>, other: Result<u64, Overflow>) pure contract {
  ensures when Ok(value: reported): reported == taken.len;
} {
  let measured = taken.len;
  return Ok<u64, Overflow>(value: measured), Ok<u64, Overflow>(value: measured);
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "CALL-4",
        sentences: &[
            "\n  mechanical_fix: more than one result can carry this route: name the one it applies to, writing `when r is` before its variant with `r` one of `outcome`, `other`\n",
        ],
        repaired: &[br#"fn probe(taken: Slots<u8, 8>) -> (outcome: Result<u64, Overflow>, other: Result<u64, Overflow>) pure contract {
  ensures when outcome is Ok(value: reported): reported == taken.len;
} {
  let measured = taken.len;
  return Ok<u64, Overflow>(value: measured), Ok<u64, Overflow>(value: measured);
}

fn main() -> status: ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#],
    },
];

/// Every function a source declares, by the name its `fn` introduces.
fn declared_functions(source: &[u8]) -> Vec<String> {
    let text = std::str::from_utf8(source).expect("a pinned source is UTF-8");
    text.split("fn ")
        .skip(1)
        .filter_map(|rest| {
            let name: String = rest
                .chars()
                .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
                .collect();
            (!name.is_empty()).then_some(name)
        })
        .collect()
}

/// The judgments in `source` that succeed only in a contradictory state, or
/// the rejection that stops it.
fn contradictory_successes(name: &str, source: &[u8]) -> Result<Vec<String>, String> {
    let functions = declared_functions(source);
    super::with_checked_program(
        &[SourceInput::new(name, source)],
        None,
        CompilerLimits::default(),
        |program, _| Ok(program.contradictory_successes(&functions)),
    )
    .map_err(|failure| failure.to_string())
}

/// Every repair the compiler prints is pinned with a program that carries it
/// out [DIAG-1]: each alternative is accepted, and its construct runs in a
/// state that is not contradictory.
#[test]
fn every_repair_is_pinned_with_a_repaired_program() {
    for pair in REPAIRS {
        let failure = compile(
            &[SourceInput::new(pair.name, pair.rejected)],
            CompilerLimits::default(),
        )
        .expect_err(pair.name);
        assert_eq!(
            failure.kind(),
            CompilationFailureKind::Source,
            "{}: {failure}",
            pair.name
        );
        assert_eq!(
            failure.rule_id(),
            Some(pair.rule),
            "{}: {failure}",
            pair.name
        );
        let rendered = format!("{failure}\n");
        for sentence in pair.sentences {
            assert!(
                rendered.contains(sentence),
                "{}: the rejection no longer carries this repair.\nwanted: {sentence}\ngot:    {rendered}",
                pair.name,
            );
        }
        for (alternative, source) in pair.repaired.iter().enumerate() {
            match contradictory_successes(pair.name, source) {
                Ok(contradictions) => assert!(
                    contradictions.is_empty(),
                    "{}: alternative {alternative} succeeds only where its state is contradictory: {contradictions:?}",
                    pair.name
                ),
                Err(rejection) => panic!(
                    "{}: alternative {alternative} is rejected:\n{rejection}",
                    pair.name
                ),
            }
        }
    }
}

/// [OP-9, STOR-6] an allocation's repair is carried out only when the
/// repaired program also builds: after checking, the selected target
/// qualifies the retained bound of every allocation the entry runs, which
/// [`every_repair_is_pinned_with_a_repaired_program`] does not reach.
#[test]
fn every_allocation_repair_builds() {
    for pair in REPAIRS.iter().filter(|pair| pair.rule == "OP-9") {
        for (alternative, source) in pair.repaired.iter().enumerate() {
            if let Err(failure) = compile(
                &[SourceInput::new(pair.name, source)],
                CompilerLimits::default(),
            ) {
                panic!(
                    "{}: alternative {alternative} does not build:\n{failure}",
                    pair.name
                );
            }
        }
    }
}

/// Why no OP-9 repair offers its ceiling as the bound to write: a program
/// that states it passes OP-9 and stops at target layout [STOR-6].
#[test]
fn an_allocation_bound_at_the_language_ceiling_stops_at_target_layout() {
    let source = br#"fn make(length: u64) -> values: Box<Slots<i64>> pure contract {
  requires length <= 2305843009213693951_u64;
} {
  let block = box_slots_new::<i64>(capacity: length);
  return move block;
}

fn main() -> status: ExitStatus pure {
  let values = make(length: 4_u64);
  return exit_status(code: 0_u8);
}
"#;
    super::check(
        &[SourceInput::new("ceiling.wf", source)],
        CompilerLimits::default(),
    )
    .expect("the language's ceiling passes OP-9");
    let failure = compile(
        &[SourceInput::new("ceiling.wf", source)],
        CompilerLimits::default(),
    )
    .expect_err("the ceiling exceeds the selected target's allocation domain");
    assert_eq!(
        failure.kind(),
        CompilationFailureKind::TargetLayout,
        "{failure}"
    );
}

/// The pair test's second condition is live: a guard around a refuted goal
/// is accepted, and the goal inside it succeeds only because the branch
/// state is contradictory, which is what makes guarding a refuted goal no
/// repair [DIAG-1].
#[test]
fn a_guard_around_a_refuted_goal_succeeds_only_in_a_contradictory_state() {
    let guarded = br#"fn main() -> status: ExitStatus pure {
  let x = 255_u8;
  if x +defined 1_u8 {
    let y = x + 1_u8;
    return exit_status(code: y);
  }
  return exit_status(code: 0_u8);
}
"#;
    let contradictions = contradictory_successes("guarded-refuted-goal.wf", guarded)
        .expect("the guarded program is accepted");
    assert!(
        !contradictions.is_empty(),
        "the operation inside the dead branch must be discharged by contradiction"
    );
}
