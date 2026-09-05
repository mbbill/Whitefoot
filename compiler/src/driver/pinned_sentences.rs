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
//! The rows are deliberately redundant with the per-item tests in
//! `driver::tests` and in `semantic::tests`: those pin one sentence beside the
//! program that motivated it and explain why the sentence says what it says,
//! and this table proves no sentence is missing from that set. A sentence
//! removed from the compiler fails here; a sentence reworded fails here.
//!
//! Seven sentences are not rows, because no source program reaches them. Each
//! is a defensive arm behind an earlier rejection, and the reason is checkable
//! one by one:
//!
//! - `'region#{}` in `check::expressions::region_spelling` renders a region
//!   whose declaration is unreachable; every region in a checked mode came
//!   from a resolved declaration.
//! - `parameter #{ordinal}` in `entailment::flow::render_goal_datum` names a
//!   formal, and only *concrete* goals are rendered.
//! - `no operand in position {index} for this row` in
//!   `check::expressions::calls` needs more operands than the selected row
//!   takes; [OP-1] rejects the arity first.
//! - [EFF-1]'s non-parameter-root reason and its repair need an effect root
//!   that resolves to a value and is not a parameter; the resolver rejects
//!   every such root as an unresolved `EffectRoot` use first.
//! - `slice_of`'s "a borrow of a runtime value binding or a named const" pair
//!   needs a place base that resolves to neither; the resolver admits only
//!   those two classes in a `PlaceBase` use.
//!
//! Three more sentences belong to the staged-permission report, which an
//! *accepted* program prints through the notice channel rather than through a
//! rejection. They are pinned where that report is built:
//! `semantic::tests::staged_permission` compares
//! `StagedDenial::writer_form()` verbatim for the two condition-2 remedies,
//! and `driver::tests` compares the one-position remedy.

use super::{CompilationFailureKind, CompilerLimits, compile};
use crate::SourceInput;

/// One probe: a minimal source, the rule its rejection must cite, and the
/// exact fragments the rendered rejection must contain.
struct Probe {
    /// The compiled unit's name, which also names the form under test.
    name: &'static str,
    /// The complete source. Minimal on purpose: everything in it is either the
    /// form under test or the entry point the language requires.
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

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "FORM-3",
        // The whole payload, so the field names of the hand-written `Debug`
        // are pinned with the sentence they carry.
        sentences: &[
            r#"SyntaxIssue { rule: Form3, coordinate: SyntaxCoordinate { source: SourceId(0), start: ByteOffset(6), end: ByteOffset(11) }, expected: ["IDENT"], mechanical_fix: "an IDENT slot admits only [FORM-3]'s IDENT `[a-z][a-z0-9_]*`, so a `const`, `fn`, parameter, `let`, field, or binder name is lowercase and is never a TYPEID `[A-Z][A-Za-z0-9]*`, a REGIONID `'[a-z][a-z0-9_]*`, a LABEL `@[a-z][a-z0-9_]*`, or an OPNAME; rename the name written here to the IDENT shape" }"#,
        ],
    },
    Probe {
        name: "struct-name-is-not-a-typeid.wf",
        source: br#"struct shape {
  seq: u64;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "FORM-3",
        sentences: &[
            "a TYPEID slot admits only [FORM-3]'s TYPEID `[A-Z][A-Za-z0-9]*`, so a struct, enum, contract, variant, or constructor name is capitalized and is never an IDENT `[a-z][a-z0-9_]*`, a REGIONID `'[a-z][a-z0-9_]*`, a LABEL `@[a-z][a-z0-9_]*`, or an OPNAME; rename the name written here to the TYPEID shape",
        ],
    },
    Probe {
        name: "region-parameter-is-not-a-regionid.wf",
        source: br#"fn f[r](x: own u64) -> out: own u64 pure {
  return x;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "FORM-3",
        sentences: &[
            "a REGIONID slot admits only [FORM-3]'s REGIONID `'[a-z][a-z0-9_]*`, the one region spelling, so write the leading apostrophe; an IDENT `[a-z][a-z0-9_]*`, a TYPEID `[A-Z][A-Za-z0-9]*`, a LABEL `@[a-z][a-z0-9_]*`, and an OPNAME are other lexical classes and none is admitted here",
        ],
    },
    Probe {
        name: "break-target-is-not-a-label.wf",
        source: br#"command fn main() -> status: own ExitStatus pure {
  loop @spin {
    break spin;
  }
  return exit_status(code: 0_u8);
}
"#,
        rule: "FORM-3",
        sentences: &[
            "a LABEL slot admits only [FORM-3]'s LABEL `@[a-z][a-z0-9_]*`, so write the leading `@`; an IDENT `[a-z][a-z0-9_]*`, a TYPEID `[A-Z][A-Za-z0-9]*`, a REGIONID `'[a-z][a-z0-9_]*`, and an OPNAME are other lexical classes and none is admitted here",
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
        source: br#"fn count(end: own u64) -> lines: own u64 pure contract {
  ensures lines <= 8_u64;
  requires end <= 8_u64;
} {
  return 0_u64;
}

command fn main() -> status: own ExitStatus pure {
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
        source: br#"fn double(value: own u64) -> out: own u64 pure {
  return value +wrap value;
}

fn helper(value: own u64) -> out: own u64 pure {
  let a = double(value: double(value: value));
  return a;
}

command fn main() -> status: own ExitStatus pure {
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
        source: br#"fn count(data: &buffer<u8>, start: own u64, end: own u64) -> lines: own u64 reads(data) contract {
  requires buffer_fits::<u8>(len_of(deref(data)));
} {
  return 0_u64;
}

command fn main() -> status: own ExitStatus pure {
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
        name: "uniq-state-measure-in-an-ensures.wf",
        source: br#"fn record(destination: &uniq buffer<u8>, value: own u8) -> written: own u64 reads(destination), writes(destination) contract {
  ensures written <= len_of(deref(destination));
} {
  return 0_u64;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "MSR-3",
        sentences: &[
            "InadmissibleStateParameterMeasure",
            "take the value by value and relate the result, or state the fact as a requires",
        ],
    },
    Probe {
        name: "contradictory-published-relations.wf",
        source: br#"fn measure(taken: own buffer<u8>) -> measured: own u64 reads(taken) contract {
  ensures measured <= len_of(taken);
  ensures len_of(taken) < measured;
} {
  return 0_u64;
}

command fn main() -> status: own ExitStatus pure {
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

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-6",
        sentences: &[
            "a source declaration never displaces, overrides, or shadows a PRE-1 prelude declaration of the same spelling and domain, and neither declaration resolves after the collision; rename this declaration",
        ],
    },
    Probe {
        name: "collides-with-a-system-declaration.wf",
        source: br#"struct DirectoryRead {
  seq: u64;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-6",
        sentences: &[
            "a source declaration never displaces, overrides, or shadows an admitted system declaration of the same spelling and domain [SYS-1, SYS-3], and neither declaration resolves after the collision; rename this declaration",
        ],
    },
    Probe {
        name: "redeclared-in-one-scope.wf",
        source: br#"command fn main() -> status: own ExitStatus pure {
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

fn consume(ticket: own Ticket) -> seq: own u64 pure {
  return ticket.seq;
}

command fn main() -> status: own ExitStatus pure {
  let permit = Ticket(seq: 1_u64);
  let used = consume(ticket: move permit);
  region {
    let permit = Ticket(seq: 2_u64);
    let again = consume(ticket: move permit);
  }
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-6",
        sentences: &[
            r#"DeclarationCollision { spelling: "permit""#,
            "a declaration's scope ends with the block that declares it, and not where its value is consumed: a binding whose value was moved is dead as a value while its declaration stays live, so an inner declaration of the same spelling still collides with it. Rename the inner declaration, or close the block that declares the outer one before this point",
        ],
    },
    // -------------------------------------------------------------------
    // [SYS-8] and [OP-4]: the residual is a place of the caller's program.
    // -------------------------------------------------------------------
    Probe {
        name: "system-range-residual.wf",
        source: br#"command fn main(command.stdout as out: own Output) -> status: own ExitStatus reads(out), writes(out), allocates(heap) {
  let header = buffer_new(4_u64, 65_u8);
  let payload = buffer_new(9_u64, 66_u8);
  let wide = len_of(payload);
  region {
    let sent = write_once(output: &uniq out, source: &header, start: 0_u64, end: wide);
  }
  return exit_status(code: 0_u8);
}
"#,
        rule: "SYS-8",
        sentences: &[r#"residual: "wide <= len_of(header)""#],
    },
    Probe {
        name: "bounds-residual.wf",
        source: br#"command fn main() -> status: own ExitStatus allocates(heap) {
  let table = buffer_new(4_u64, 0_u8);
  let other = buffer_new(9_u64, 0_u8);
  let pick = len_of(other);
  let one = table[pick];
  return exit_status(code: 0_u8);
}
"#,
        rule: "OP-4",
        sentences: &[r#"residual: "pick < len_of(table)""#],
    },
    // -------------------------------------------------------------------
    // [SYS-2] and [FN-2]: written type and region arguments.
    // -------------------------------------------------------------------
    Probe {
        name: "system-argument-does-not-name-a-region.wf",
        source: br#"command fn main(command.stdout as out: own Output) -> status: own ExitStatus reads(out), writes(out), allocates(heap) {
  let payload = buffer_new(4_u64, 65_u8);
  region {
    let sent = write_once::<ExitStatus>(output: &uniq out, source: &payload, start: 0_u64, end: 4_u64);
  }
  return exit_status(code: 0_u8);
}
"#,
        rule: "SYS-2",
        sentences: &[
            r#"TypeMismatch { expected: "a region argument in this position", found: "an argument that does not name a region" }"#,
        ],
    },
    // The two region-arity rows this table carried until v0.42 are retired
    // with the sentences they pinned: [FORM-8] gives a call exactly one legal
    // region-argument list, so "no region argument list" and "too many region
    // arguments" are no longer faults a call can commit. Their replacements
    // are the two FORM-8 rows at the end of this table. The generic half of
    // FN-2's arity sentence survives and is pinned here.
    Probe {
        name: "call-without-its-type-arguments.wf",
        source: br#"fn identity<T: Int>(value: own T) -> out: own T pure {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  let doubled = identity(value: 1_u64);
  return exit_status(code: 0_u8);
}
"#,
        rule: "FN-2",
        sentences: &[
            r#"TypeMismatch { expected: "1 written type argument", found: "no type-argument list" }"#,
        ],
    },
    // -------------------------------------------------------------------
    // [TYPE-5] generic argument lists and [FN-3] bounds.
    // -------------------------------------------------------------------
    Probe {
        name: "construct-without-its-type-arguments.wf",
        source: br#"struct Pair<T: affine> {
  left: T;
  right: T;
}

command fn main() -> status: own ExitStatus pure {
  let p = Pair(left: 1_u64, right: 2_u64);
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[
            r#"TypeMismatch { expected: "1 written type argument", found: "no type-argument list" }"#,
        ],
    },
    Probe {
        name: "construct-with-too-many-type-arguments.wf",
        source: br#"struct Pair<T: affine> {
  left: T;
  right: T;
}

command fn main() -> status: own ExitStatus pure {
  let p = Pair<u64, u64>(left: 1_u64, right: 2_u64);
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[
            r#"TypeMismatch { expected: "1 written type argument", found: "2 written type arguments" }"#,
        ],
    },
    Probe {
        name: "const-written-in-a-type-parameter-position.wf",
        source: br#"struct Pair<T: affine> {
  left: T;
  right: T;
}

command fn main() -> status: own ExitStatus pure {
  let p = Pair<4>(left: 1_u64, right: 2_u64);
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[
            r#"TypeMismatch { expected: "a type in this type-argument position", found: "a const argument in a type-parameter position" }"#,
        ],
    },
    Probe {
        name: "type-written-in-a-const-parameter-position.wf",
        source: br#"struct Row<const n: u64> {
  count: u64;
}

command fn main() -> status: own ExitStatus pure {
  let r = Row<u64>(count: 1_u64);
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[
            r#"TypeMismatch { expected: "a const argument in this type-argument position", found: "a type in a const-parameter position" }"#,
        ],
    },
    Probe {
        name: "type-arguments-on-a-form-that-declares-none.wf",
        source: br#"struct Plain {
  value: u64;
}

command fn main() -> status: own ExitStatus pure {
  let p = Plain<u64>(value: 1_u64);
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[
            r#"TypeMismatch { expected: "no type arguments, because this form declares no generic parameters", found: "a written `<...>` type-argument list" }"#,
        ],
    },
    Probe {
        name: "type-arguments-on-a-type-that-takes-none.wf",
        source: br#"fn take(value: own Bool<u8>) -> out: own u64 pure {
  return 0_u64;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[
            r#"TypeMismatch { expected: "this type spelled with no type arguments", found: "a written `<...>` type-argument list on a type that takes none" }"#,
        ],
    },
    Probe {
        name: "int-bound-is-not-satisfied.wf",
        source: br#"fn widen<T: Int>(value: own T) -> out: own T pure {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  let a = widen::<f64>(value: 1.0_f64);
  return exit_status(code: 0_u8);
}
"#,
        rule: "FN-3",
        sentences: &[
            r#"TypeMismatch { expected: "an integer type, which the parameter's `Int` bound requires", found: "f64" }"#,
        ],
    },
    Probe {
        name: "float-bound-is-not-satisfied.wf",
        source: br#"fn scale<T: Float>(value: own T) -> out: own T pure {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  let a = scale::<u64>(value: 1_u64);
  return exit_status(code: 0_u8);
}
"#,
        rule: "FN-3",
        sentences: &[
            r#"TypeMismatch { expected: "a float type, which the parameter's `Float` bound requires", found: "u64" }"#,
        ],
    },
    // -------------------------------------------------------------------
    // [TYPE-5] Result and Option: both spellings that carry the arguments.
    // -------------------------------------------------------------------
    Probe {
        name: "result-without-type-arguments.wf",
        source: br#"fn pick(value: own u64) -> out: own Result pure {
  return Ok(value: value);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[
            r#"TypeMismatch { expected: "Result with both type arguments written: as a type `Result<u64, IoError>`, and as a variant constructor `Ok<u64, IoError>(value: v)`", found: "Result with no written type-argument list" }"#,
        ],
    },
    Probe {
        name: "result-with-one-type-argument.wf",
        source: br#"fn pick(value: own u64) -> out: own Result<u64> pure {
  return Ok<u64>(value: value);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[
            r#"TypeMismatch { expected: "Result<T, E> with exactly two type arguments", found: "a Result type-argument list of a different length" }"#,
        ],
    },
    Probe {
        name: "result-with-a-const-type-argument.wf",
        source: br#"fn pick(value: own u64) -> out: own Result<4, IoError> pure {
  return Ok<4, IoError>(value: value);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[
            r#"TypeMismatch { expected: "a type in each Result type-argument position", found: "a const argument in a Result type-argument position" }"#,
        ],
    },
    Probe {
        name: "option-without-type-arguments.wf",
        source: br#"fn pick(value: own u64) -> out: own Option pure {
  return Some(value: value);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[
            r#"TypeMismatch { expected: "Option with its type argument written: as a type `Option<u64>`, and as a variant constructor `Some<u64>(value: v)`", found: "Option with no written type-argument list" }"#,
        ],
    },
    Probe {
        name: "option-with-two-type-arguments.wf",
        source: br#"fn pick(value: own u64) -> out: own Option<u64, u64> pure {
  return Some<u64, u64>(value: value);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[
            r#"TypeMismatch { expected: "Option<T> with exactly one type argument", found: "an Option type-argument list of a different length" }"#,
        ],
    },
    Probe {
        name: "option-with-a-const-type-argument.wf",
        source: br#"fn pick(value: own u64) -> out: own Option<4> pure {
  return Some<4>(value: value);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[
            r#"TypeMismatch { expected: "a type in the Option type-argument position", found: "a const argument in the Option type-argument position" }"#,
        ],
    },
    Probe {
        name: "array-element-is-not-flat.wf",
        source: br#"fn take(value: own array<Option<u64>, 4>) -> out: own u64 pure {
  return 0_u64;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-2",
        sentences: &[
            r#"TypeMismatch { expected: "a flat element type: an integer, a float, Bool, unit, or a struct or enum whose fields are themselves flat element types", found: "Option<u64>" }"#,
        ],
    },
    // -------------------------------------------------------------------
    // [EFF-1] and [EFF-2]: the declared row.
    // -------------------------------------------------------------------
    Probe {
        name: "writes-through-a-shared-borrow.wf",
        source: br#"fn touch(data: &buffer<u8>) -> out: own u64 writes(data) {
  return len_of(deref(data));
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "EFF-1",
        sentences: &[
            r#"InvalidEffectRow { reason: "a `writes` path is rooted at a shared borrow parameter, which grants no exclusive access to that state", mechanical_fix: "declare that parameter `&uniq` or `own`, or drop the path from `writes`; an effect path grants no permission of its own" }"#,
        ],
    },
    Probe {
        name: "repeated-effect-category.wf",
        source: br#"fn touch(left: own u64, right: own u64) -> out: own u64 reads(left), reads(right) {
  return left;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "EFF-1",
        sentences: &[
            r#"InvalidEffectRow { reason: "a category appears at most once in one row, and the row is written in the canonical order reads, writes, allocates", mechanical_fix: "merge the repeated category's paths into one occurrence — `writes(cwd), writes(out)` is `writes(cwd, out)` — and order the categories reads, writes, allocates" }"#,
        ],
    },
    Probe {
        name: "effect-suffix-on-a-non-struct.wf",
        source: br#"fn touch(value: own u64) -> out: own u64 reads(value.count) {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "EFF-1",
        sentences: &[
            r#"InvalidEffectRow { reason: "each effect-path suffix selects one statically known field of a source struct, and this prefix is not a source struct", mechanical_fix: "name the parameter itself, which names the complete state it supplies; an enum payload, a subscript, and a `deref` spelling are outside the effect-path grammar" }"#,
        ],
    },
    Probe {
        name: "effect-suffix-names-an-undeclared-field.wf",
        source: br#"struct Pair {
  left: u64;
  right: u64;
}

fn touch(pair: own Pair) -> out: own u64 reads(pair.middle) {
  return pair.left;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "EFF-1",
        sentences: &[
            r#"InvalidEffectRow { reason: "an effect-path suffix names a field the struct does not declare", mechanical_fix: "name a declared field of that struct, or the parameter itself" }"#,
        ],
    },
    Probe {
        name: "declared-row-is-narrower-than-the-body.wf",
        source: br#"fn touch(data: &buffer<u8>) -> out: own u64 pure {
  return len_of(deref(data));
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "EFF-2",
        sentences: &[
            r#"EffectMismatch { expected_row: "reads(data)", found_row: "pure", missing: ["reads(data)"], extra: [], mechanical_fix: "declare exactly the row the body exhibits: add every missing category and path and remove every extra one; EFF-2 admits no wider and no narrower declaration than the union of the body-syntactic and release contributions" }"#,
        ],
    },
    // -------------------------------------------------------------------
    // [TYPE-5] places, subscripts, and flat storage.
    // -------------------------------------------------------------------
    Probe {
        name: "buffer-length-is-not-a-u64.wf",
        source: br#"command fn main() -> status: own ExitStatus allocates(heap) {
  let flag = 1_u64 > 0_u64;
  let store = buffer_new(flag, 0_u8);
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[r#"TypeMismatch { expected: "own u64", found: "own Bool" }"#],
    },
    Probe {
        name: "subscript-is-not-the-last-suffix.wf",
        source: br#"command fn main() -> status: own ExitStatus allocates(heap) {
  let store = buffer_new(4_u64, 0_u8);
  let one = store[0_u64].value;
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[
            r#"TypeMismatch { expected: "a subscript as the last suffix of the place", found: "a subscript followed by another suffix" }"#,
        ],
    },
    Probe {
        name: "indexed-operand-is-a-move.wf",
        source: br#"command fn main() -> status: own ExitStatus allocates(heap) {
  let store = buffer_new(4_u64, 0_u8);
  let n = len_of(move store);
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[
            r#"TypeMismatch { expected: "a place, which a subscript indexes", found: "a written `move`, which consumes rather than indexes" }"#,
        ],
    },
    Probe {
        name: "indexed-operand-is-not-a-place.wf",
        source: br#"command fn main() -> status: own ExitStatus pure {
  let n = len_of(1_u64);
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[
            r#"TypeMismatch { expected: "a place, which a subscript indexes", found: "an atom that is not a place" }"#,
        ],
    },
    Probe {
        name: "indexed-place-is-a-scalar.wf",
        source: br#"command fn main() -> status: own ExitStatus pure {
  let value = 1_u64;
  let n = len_of(value);
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[
            r#"TypeMismatch { expected: "an array, buffer, or slice place", found: "u64" }"#,
        ],
    },
    Probe {
        name: "slice-of-a-non-borrow.wf",
        source: br#"const digits: array<u8, 2> =[48_u8, 49_u8];

command fn main() -> status: own ExitStatus pure {
  let view = slice_of(digits);
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[
            r#"TypeMismatch { expected: "a written shared borrow of the viewed storage, `&'r place`", found: "an atom that is not a borrow expression" }"#,
        ],
    },
    Probe {
        name: "slice-of-a-unique-borrow.wf",
        source: br#"command fn main() -> status: own ExitStatus allocates(heap) {
  let store = buffer_new(4_u64, 0_u8);
  region {
    let view = slice_of(&uniq store);
  }
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[
            r#"TypeMismatch { expected: "a written shared borrow of the viewed storage, `&'r place`", found: "a `&uniq` borrow, which slice_of does not take" }"#,
        ],
    },
    // -------------------------------------------------------------------
    // [OWN-4], [OWN-6], and [OWN-10]: regions and reborrows.
    // -------------------------------------------------------------------
    Probe {
        name: "slice-of-arena-content-under-a-foreign-region.wf",
        source: br#"fn arena_view['v](storage: own arena<array<u8, 2>>, marker: &'v u64) -> result: &'v u64 pure {
  let view = slice_of(&'v deref(storage));
  return marker;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "OWN-10",
        sentences: &[
            r#"InvalidBorrowLifetime { region: "'v", binder: "storage", mechanical_fix: "arena content outlives its arena's own region, not the arena binding; that region is unwritten here, so write it on the arena and name it on this view, or take the view in a region it outlives" }"#,
        ],
    },
    Probe {
        name: "borrow-of-local-storage-under-a-parameter-region.wf",
        source: br#"fn sum(data: &buffer<u8>) -> out: own u64 reads(data) {
  return len_of(deref(data));
}

fn caller['r](anchor: &'r buffer<u8>) -> out: &'r buffer<u8> allocates(heap) {
  let local = buffer_new(4_u64, 0_u8);
  let counted = sum(data: &'r local);
  return anchor;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "OWN-10",
        sentences: &[
            r#"InvalidBorrowLifetime { region: "'r", binder: "local", mechanical_fix: "a borrow of local storage names a region introduced inside that binding's own scope: write `region 'r { ... }` after the binding and take the borrow inside it. A caller-supplied region parameter is never admitted here, because it outlives the storage." }"#,
        ],
    },
    Probe {
        name: "returned-child-reborrow-names-a-foreign-region.wf",
        source: br#"fn pass['b](holder: &buffer<u8>, marker: &'b u64) -> out: &'b buffer<u8> pure {
  return &'b deref(holder);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "OWN-10",
        sentences: &[
            r#"InvalidBorrowLifetime { region: "'b", binder: "holder", mechanical_fix: "a returned child reborrow names a region its holder's own region outlives; that region is unwritten here, so relate the holder's region to this result and name it on the returned reborrow" }"#,
        ],
    },
    Probe {
        name: "returned-borrow-of-a-local-region.wf",
        source: br#"fn leak['r0](x: &'r0 i32) -> return_value: &'r0 i32 pure {
  region 's {
    region {
      return &'s deref(x);
    }
  }
}

command fn main() -> return_value: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "OWN-4",
        sentences: &[
            r#"InvalidBorrowLifetime { region: "'r0", binder: "x", mechanical_fix: "the value's borrow is live for 's, and 'r0 is not inside it; store or pass it under a region 's outlives, or introduce 'r0 inside 's's block" }"#,
        ],
    },
    Probe {
        name: "two-statements-in-a-child-region.wf",
        source: br#"fn take(out: &uniq buffer<u8>) -> result: own unit pure {
  return unit;
}

fn invalid(out: &uniq buffer<u8>) -> result: own unit pure {
  region {
    take(out: &uniq deref(out));
    take(out: &uniq deref(out));
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "OWN-6",
        sentences: &[
            "a child reborrow's region admits exactly one statement, and a value that statement binds dies at the region's end, so `region 'r { let permit = reserve_file::<'r>(factory: &uniq 'r holder); match open_...(permit: move permit, ...) { ... } }` is two statements and cannot be repaired by shortening the region. The whole idiom is three parts: move the reserve and the open into one helper that takes the holder as `&uniq 'f` and returns the opened value (`fn open_source_from_factory['f, 'd](factory: &uniq 'f FileFactory, directory: &'d DirectoryRead) -> result: own Result<DirectorySource, IoError>`); make the single statement of the region the `match` on that helper's call; and write every statement that uses the opened value inside that `match` arm, because the opened value dies with the region (P4 linear threading, P15 recursive walker). The other route, `let stale = replace target = call(...);`, applies only where the call leaves the target's root alive: a call that consumes the target root — one taking `move permit` — rejects OWN-1 instead.",
        ],
    },
    Probe {
        name: "borrow-kind-does-not-match-the-destination.wf",
        source: br#"fn measure(data: &buffer<u8>) -> out: own u64 reads(data) {
  return len_of(deref(data));
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let store = buffer_new(4_u64, 0_u8);
  region {
    let n = measure(data: &uniq store);
  }
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[r#"TypeMismatch { expected: "a shared borrow", found: "&uniq buffer<u8>" }"#],
    },
    Probe {
        name: "shared-borrow-where-a-unique-one-is-required.wf",
        source: br#"command fn main(command.stdout as out: own Output) -> status: own ExitStatus reads(out), writes(out), allocates(heap) {
  let payload = buffer_new(4_u64, 65_u8);
  region {
    let sent = write_once(output: &out, source: &payload, start: 0_u64, end: 4_u64);
  }
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[r#"TypeMismatch { expected: "a `uniq` borrow", found: "&Output" }"#],
    },
    Probe {
        name: "slice-value-where-a-scalar-is-required.wf",
        source: br#"const digits: array<u8, 2> =[48_u8, 49_u8];

fn measure(view: own u64) -> out: own u64 pure {
  return view;
}

command fn main() -> status: own ExitStatus pure {
  region {
    let view = slice_of(&digits);
    let n = measure(view: move view);
  }
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[r#"TypeMismatch { expected: "u64", found: "slice<u8>" }"#],
    },
    // -------------------------------------------------------------------
    // [TYPE-5] and [FORM-5]: projections, replacement, and operands.
    // -------------------------------------------------------------------
    Probe {
        name: "projection-of-a-non-struct.wf",
        source: br#"fn peek(value: own u64) -> out: own u64 pure {
  return value.count;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[
            r#"TypeMismatch { expected: "a source struct, whose declared field this suffix selects", found: "u64" }"#,
        ],
    },
    Probe {
        name: "projection-of-an-undeclared-field.wf",
        source: br#"struct Pair {
  left: u64;
  right: u64;
}

fn peek(pair: own Pair) -> out: own u64 pure {
  return pair.middle;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[
            r#"TypeMismatch { expected: "a declared field of Pair", found: "the field name `middle`, which that struct does not declare" }"#,
        ],
    },
    Probe {
        name: "replacement-value-has-another-type.wf",
        source: br#"struct Ticket {
  seq: u64;
}

command fn main() -> status: own ExitStatus pure {
  let ticket = Ticket(seq: 1_u64);
  let stale = replace ticket = 2_u64;
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[r#"TypeMismatch { expected: "own Ticket", found: "own u64" }"#],
    },
    Probe {
        name: "boolean-operand-is-an-integer.wf",
        source: br#"command fn main() -> status: own ExitStatus pure {
  let flag = band(1_u64, 2_u64);
  return exit_status(code: 0_u8);
}
"#,
        rule: "TYPE-5",
        sentences: &[r#"TypeMismatch { expected: "own Bool", found: "own u64" }"#],
    },
    Probe {
        name: "match-scrutinee-is-not-an-enum.wf",
        source: br#"command fn main() -> status: own ExitStatus pure {
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
            r#"TypeMismatch { expected: "an enum scrutinee, whose variants the arms match", found: "u64" }"#,
        ],
    },
    Probe {
        name: "generic-numeric-identity-of-a-non-numeric-type.wf",
        source: br#"fn zeroed<T: affine>(sample: own T) -> out: own T pure {
  return 0_T;
}

command fn main() -> status: own ExitStatus pure {
  let flag = 1_u64 > 0_u64;
  let a = zeroed::<Bool>(sample: flag);
  return exit_status(code: 0_u8);
}
"#,
        rule: "FORM-5",
        sentences: &[
            r#"TypeMismatch { expected: "an integer or float type, whose 0 and 1 this form names""#,
        ],
    },
    // -------------------------------------------------------------------
    // [FN-8]: the goal, rendered in the source terms of the caller.
    // -------------------------------------------------------------------
    Probe {
        name: "goal-with-an-infix-operation.wf",
        source: br#"fn need(x: own u64) -> out: own u64 pure contract {
  define bumped = x +wrap 1_u64;
  requires bumped < 10_u64;
} {
  return x;
}

command fn main() -> status: own ExitStatus pure {
  let s = 3_u64;
  let r = need(x: s);
  return exit_status(code: 0_u8);
}
"#,
        rule: "FN-8",
        sentences: &[r#"instantiated_goal: "s +wrap 1_u64 < 10_u64""#],
    },
    Probe {
        name: "goal-with-a-numeric-conversion.wf",
        source: br#"fn need(x: own u32) -> out: own u32 pure contract {
  define wide = cvt::<u32, u64>(x);
  requires wide < 10_u64;
} {
  return x;
}

command fn main() -> status: own ExitStatus pure {
  let s = 3_u32;
  let r = need(x: s);
  return exit_status(code: 0_u8);
}
"#,
        rule: "FN-8",
        sentences: &[r#"instantiated_goal: "cvt::<u32, u64>(s) < 10_u64""#],
    },
    Probe {
        name: "goal-with-a-reinterpretation.wf",
        source: br#"fn need(x: own i64) -> out: own i64 pure contract {
  define raw = reinterpret::<i64, u64>(x);
  requires raw < 10_u64;
} {
  return x;
}

command fn main() -> status: own ExitStatus pure {
  let s = 3_i64;
  let r = need(x: s);
  return exit_status(code: 0_u8);
}
"#,
        rule: "FN-8",
        sentences: &[r#"instantiated_goal: "reinterpret::<i64, u64>(s) < 10_u64""#],
    },
    Probe {
        name: "goal-with-a-float-literal.wf",
        source: br#"fn need(x: own f64) -> out: own f64 pure contract {
  requires flt(x, 1.0_f64);
} {
  return x;
}

command fn main() -> status: own ExitStatus pure {
  let v = 2.0_f64;
  let r = need(x: v);
  return exit_status(code: 0_u8);
}
"#,
        rule: "FN-8",
        sentences: &[r#"instantiated_goal: "flt(v, Float { ty: F64, bits: 4607182418800017408 })""#],
    },
    Probe {
        name: "goal-over-an-admitted-index-actual.wf",
        source: br#"fn need(x: own u8) -> out: own u8 pure contract {
  requires x < 10_u8;
} {
  return x;
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let data = buffer_new(4_u64, 0_u8);
  let r = need(x: data[0_u64]);
  return exit_status(code: 0_u8);
}
"#,
        rule: "FN-8",
        sentences: &[r#"instantiated_goal: "data[0_u64] < 10_u8""#],
    },
    Probe {
        name: "goal-over-a-dereferenced-holder.wf",
        source: br#"fn need(names: &buffer<u8>, pos: own u64) -> out: own u64 pure contract {
  define spare = len_of(deref(names));
  requires pos <= spare;
} {
  return pos;
}

fn outer(names: &buffer<u8>) -> out: own u64 pure {
  let r = need(names: names, pos: 9_u64);
  return r;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "FN-8",
        sentences: &[r#"instantiated_goal: "9_u64 <= len_of(deref(names))""#],
    },
    // [FORM-8] one canonical region spelling: each position a region can
    // occupy, written exactly where the surrounding text does not fix it.
    Probe {
        name: "region-written-at-an-unrelated-parameter.wf",
        source: br#"fn peek['r](value: &'r i32) -> result: own i32 reads(value) {
  return deref(value);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "FORM-8",
        sentences: &[
            "drop the region name: no other position of this declaration names this region, so the position denotes one region of its own",
        ],
    },
    Probe {
        name: "region-elided-at-a-result.wf",
        source: br#"fn pass(value: &i32) -> result: &i32 pure {
  return value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "FORM-8",
        sentences: &[
            "write this result region: name the parameter region the result shares, or a region parameter of its own that the caller supplies",
        ],
    },
    Probe {
        name: "region-parameter-list-out-of-order.wf",
        source: br#"fn pass['s, 'r](first: &'r i32, second: &'s i32, third: &'s i32) -> result: &'r i32 pure {
  return first;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "FORM-8",
        sentences: &[
            "the region parameter list holds exactly the region names this declaration writes, once each, in the order of their first written occurrence, and is absent when it writes none",
        ],
    },
    Probe {
        name: "region-written-at-the-innermost-borrow.wf",
        source: br#"command fn main() -> status: own ExitStatus pure {
  let a = 40_i32;
  region 'r {
    let p = &'r a;
    let observed = deref(p);
    if observed == 40_i32 {
    } else {
      return exit_status(code: 1_u8);
    }
  }
  return exit_status(code: 0_u8);
}
"#,
        rule: "FORM-8",
        sentences: &[
            "drop the region name: this borrow takes the region of the region block that most closely encloses it, and a loop body is one",
        ],
    },
    // [FORM-8, OWN-11] a loop body is itself a region block, so a block that
    // is the body's only statement spells that one region twice.
    Probe {
        name: "region-block-is-the-whole-loop-body.wf",
        source: br#"command fn main() -> status: own ExitStatus pure {
  let a = 40_i32;
  for @scan (step in 0_u64..2_u64) {
    region {
      let p = &a;
      let observed = deref(p);
      if observed == 40_i32 {
      } else {
        return exit_status(code: 1_u8);
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#,
        rule: "FORM-8",
        sentences: &[
            "the loop body is its own region; remove the region block, keep its statements where they stand, and drop every region name it carried",
        ],
    },
    Probe {
        name: "region-block-name-nothing-references.wf",
        source: br#"command fn main() -> status: own ExitStatus pure {
  let a = 40_i32;
  region 'r {
    let p = &a;
    let observed = deref(p);
    if observed == 40_i32 {
    } else {
      return exit_status(code: 1_u8);
    }
  }
  return exit_status(code: 0_u8);
}
"#,
        rule: "FORM-8",
        sentences: &[
            "drop the region name: nothing inside this block names it, so the block is written `region { ... }`",
        ],
    },
    Probe {
        name: "region-argument-the-call-determines.wf",
        source: br#"fn peek(value: &i32) -> result: own i32 reads(value) {
  return deref(value);
}

command fn main() -> status: own ExitStatus pure {
  let a = 40_i32;
  region 'r {
    let p = &a;
    let v = peek::<'r>(value: p);
    if v == 40_i32 {
    } else {
      return exit_status(code: 1_u8);
    }
  }
  return exit_status(code: 0_u8);
}
"#,
        rule: "FORM-8",
        sentences: &[
            "write exactly the callee's region parameters that occur in no parameter type, in their declared order; every other region argument is determined by this call's own arguments and is not written",
        ],
    },
    Probe {
        name: "system-region-argument-the-call-determines.wf",
        source: br#"command fn main(command.args as args: own Args) -> status: own ExitStatus reads(args) {
  region 'a {
    let total = args_count::<'a>(args: &args);
  }
  return exit_status(code: 0_u8);
}
"#,
        rule: "FORM-8",
        sentences: &[
            "drop the region arguments: every system operation's region occurs at one parameter position, so this call's own arguments determine it",
        ],
    },
    // -------------------------------------------------------------------
    // [LIV-1] and [LIV-2]: join-checked liveness and the one commit.
    // -------------------------------------------------------------------
    Probe {
        name: "branches-disagree-about-a-binding.wf",
        source: br#"fn measure(cell: own buffer<u8>) -> size: own u64 reads(cell) {
  let n = len_of(cell);
  return n;
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let c = buffer_new(4_u64, 0_u8);
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
            r#"binding: "c""#,
            r#"live_predecessor: "the `else` branch""#,
            r#"dead_predecessor: "the `if` branch""#,
            "every predecessor of a join agrees on a binding\'s live-or-dead status: consume it on every predecessor, on none, or commit a value back into it before the predecessor that consumed it reaches the join",
        ],
    },
    Probe {
        name: "one-iteration-leaves-an-outer-binding-dead.wf",
        source: br#"fn measure(cell: own buffer<u8>) -> size: own u64 reads(cell) {
  let n = len_of(cell);
  return n;
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let c = buffer_new(4_u64, 0_u8);
  for (i in 0_u64..2_u64) {
    let taken = measure(cell: move c);
  }
  return exit_status(code: 0_u8);
}
"#,
        rule: "OWN-11",
        sentences: &[
            r#"binding: "c""#,
            "one iteration must leave every outer binding in the status the next one starts from: commit a value back into it before the backedge, or declare and consume it inside the body",
        ],
    },
    Probe {
        name: "two-targets-of-one-commit-overlap.wf",
        source: br#"fn two_bytes(bound: own u64) -> (low: own u8, high: own u8) pure {
  return 1_u8, 2_u8;
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let v = buffer_new(4_u64, 0_u8);
  let i = 0_u64;
  let j = 1_u64;
  set (v[i], v[j]) = two_bytes(bound: 4_u64);
  return exit_status(code: 0_u8);
}
"#,
        rule: "LIV-2",
        sentences: &[
            r#"first: "v[i]""#,
            r#"second: "v[j]""#,
            "one commit writes pairwise non-overlapping places; write the overlapping target in a statement of its own",
        ],
    },
    Probe {
        name: "a-commit-target-carrying-a-region.wf",
        source: br#"command fn main() -> status: own ExitStatus allocates(heap) {
  let data = buffer_new(4_u64, 0_u8);
  region {
    let view = slice_of(&data);
    let other = slice_of(&data);
    set view = other;
  }
  return exit_status(code: 0_u8);
}
"#,
        rule: "LIV-2",
        sentences: &[
            r#"target_type: "slice<u8>""#,
            "a slice\'s static origin set and an arena\'s confinement are fixed at initialization; bind a new slice or arena under a new let",
        ],
    },
    // -------------------------------------------------------------------
    // [BLK-1] and [PROV-1]: the container nominals a construct may not name
    // and the store region an extent always writes.
    // -------------------------------------------------------------------
    Probe {
        name: "a-construct-naming-a-run.wf",
        source: br#"command fn main() -> status: own ExitStatus pure {
  let made = FixedVector(len: 0_u64);
  return exit_status(code: 0_u8);
}
"#,
        rule: "BLK-1",
        sentences: &[
            r#"nominal: "FixedVector""#,
            "form the run with a formation operation",
        ],
    },
    Probe {
        name: "a-construct-naming-a-provider.wf",
        source: br#"command fn main() -> status: own ExitStatus pure {
  let made = Heap(cursor: 0_u64);
  return exit_status(code: 0_u8);
}
"#,
        rule: "BLK-1",
        sentences: &[
            r#"nominal: "Heap""#,
            "receive the provider as a parameter",
        ],
    },
    Probe {
        name: "an-extent-eliding-its-store-region.wf",
        source: br#"fn carve(extent: own Arena<4096, 16>) -> made: own u64 pure {
  return 0_u64;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        rule: "FORM-8",
        sentences: &[
            "write the store region this extent names: a bump extent's region is one the caller must choose, so it is written at every position",
        ],
    },
    // -------------------------------------------------------------------
    // [INV-1]: the one `call` an affine factor admits is a measure former.
    // -------------------------------------------------------------------
    Probe {
        name: "an-affine-factor-that-is-not-a-measure.wf",
        source: br#"command fn main() -> status: own ExitStatus pure {
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
            "write len_of(P), cap_of(P), room_of(P) or head_of(P) over a measured place",
        ],
    },
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
        for sentence in probe.sentences {
            assert!(
                failure.detail().contains(sentence),
                "{}: the rendered rejection no longer carries this sentence.\nwanted: {sentence}\ngot:    {}",
                probe.name,
                failure.detail()
            );
        }
    }
}
