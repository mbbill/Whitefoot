# SYS count postconditions: what v0.20 states, implies, and omits

Status: survey of `spec/kernel-spec-v0.20.md` §16, executed 2026-08-06 in
support of PROBE-TAINT.md's load-bearing claim that "the SYS boundary
operations' count bounds must be normative postconditions". This file records
what the current normative text actually says. It proposes no amendment and
changes no rule; the sentences below are drafting material for a later spec
proposal under `docs/WORKFLOW.md`.

## Scope

[SYS-2] declares eleven operations. Seven return a `u64` count, length, or
size and are surveyed here. Four return no count and are out of scope:
`arg_get` (`Result<HostString, ArgError>`), `relative_path`
(`Result<RelativePath, PathError>`), `open_read` (`Result<ReadFile, IoError>`),
and `exit_status` (`ExitStatus`). `arg_get` still appears below because the
missing fact about `args_count` is a fact about `arg_get`'s success condition.

Classification is about the **upper bound on the returned count** — the
property cursor arithmetic needs. (a) means the rule text states a checkable
relation between the returned count and a named program value. (b) means the
relation is derivable from the prose but is never written as a relation. (c)
means no text bears on it.

## Headline

**No operation is (a).** The four one-attempt transfer operations are (b): the
bound is real, is clearly intended, and is reachable in two or three prose
steps, but is never written as a relation between the result and the parameter
that bounds it. The three pure length/count operations are (c): nothing in the
spec relates their results to anything a program holds.

The single sharpest piece of evidence: the identifier `capacity` occurs in
exactly five places in the whole specification — the four [SYS-2] signature
lines that declare it and one [SYS-8] sentence about *range validation*. It
never once occurs in a sentence about a returned value.

## Table

| operation | returns | class | exact text carrying the property (rule ID) | minimal sentence to add |
|---|---|---|---|---|
| `read_once` | `ReadOutcome` / `ReadBytes(count: u64)` | **b** | "A target returning a count outside the validated range violates its compiler-owned contract; source code does not defend against it." [SYS-8] | "On `ReadBytes(count)`, `count` is at most the requested `capacity`, and the checked program retains that bound [DIAG-2]." |
| `write_once` | `Result<u64, IoError>` | **b** | "its accepted count means exactly that the host operation accepted that prefix" [SYS-12]; the same [SYS-8] target-contract sentence | "On `Ok(accepted)`, `accepted` is at most the requested `count`, and the checked program retains that bound [DIAG-2]." |
| `host_copy_bytes` | `Result<u64, CopyError>` | **b** | "A successful copy changes exactly the requested destination prefix and leaves the rest of the buffer unchanged." [SYS-8] | "On `Ok(copied)`, `copied` is at most the requested `capacity`, and the checked program retains that bound [DIAG-2]." |
| `host_copy_utf8` | `Result<u64, Utf8CopyError>` | **b** | same [SYS-8] sentence; "only then copies the complete encoding" [SYS-8] | "On `Ok(copied)`, `copied` is at most the requested `capacity`, and the checked program retains that bound [DIAG-2]." |
| `args_count` | `own u64` | **c** | "`args_count` is total." [SYS-9] — the complete normative text about its result | "`arg_get` returns `Ok` exactly when `position` is less than the count `args_count` returns for the same `Args`, and the checked program retains that relation [DIAG-2]." |
| `host_bytes_len` | `own u64` | **c** | "`host_bytes_len` returns the exact count of the host string's native bytes" [SYS-9] | "That count is exactly the `required` length a `host_copy_bytes` on the same host string reports, so a `host_copy_bytes` whose `capacity` is at least that count returns `Ok` with exactly that count, and the checked program retains that relation [DIAG-2]." |
| `host_utf8_len` | `Result<u64, Utf8Error>` | **c** | "on a successful length, copy, or write the `Ok` payload is the exact `u64` byte, encoded, or accepted length" [SYS-6] | "On `Ok(length)`, a `host_copy_utf8` on the same host string neither returns `Utf8CopyInvalid()` nor, for a `capacity` of at least `length`, returns `Utf8CopyTooSmall(required)`, and the checked program retains that relation [DIAG-2]." |

## Per-operation notes

### `read_once` — (b), the strongest (b)

Three sentences bear on the bound, and together they make it unmistakable
without stating it:

> On `ReadBytes(count)` exactly the first `count` bytes of the requested range
> may have changed, every other byte of the buffer is unchanged, and the file
> cursor advances by exactly `count`. [SYS-8]

> A target returning a count outside the validated range violates its
> compiler-owned contract; source code does not defend against it. [SYS-8]

> Range validation precedes every other action. For `read_once` the range is
> `offset` and `capacity`[.] [SYS-8]

This is one sentence away from (a), but it is not (a), for four reasons worth
keeping distinct when the amendment is drafted:

1. The consequent is target conformance ("violates its compiler-owned
   contract"), not a property of the returned value. It obliges an
   implementation; it grants the checker nothing.
2. "outside the validated range" is range *membership* applied to a *count*.
   The validated range is a set of buffer positions `offset .. offset +
   capacity`; a count is a length, not a position in it. Read literally, a
   count equal to `capacity` is outside the range and a count equal to
   `offset` is inside it — the opposite of the intent. A reader has to
   silently reinterpret the phrase to recover `count <= capacity`.
3. It names no operation and no parameter, so no checker rule can cite it as
   the origin of a fact about `capacity`.
4. "source code does not defend against it" instructs the *writer* not to
   check. It does not say the *checked program* may assume.

The first sentence has the same gap in a different form: it bounds which
*bytes may have changed*, not the magnitude of the returned number. Deriving
`count <= capacity` from it requires the extra step that a prefix of the
requested range cannot be longer than the range.

Lower bound, by contrast, **is** stated (a)-grade — and is conditional, which
a drafter must not flatten:

> `read_once` returns `ReadBytes(count)` only for a count greater than zero
> [SYS-8, in the paragraph opening "For a nonempty range"]

> For a zero-length range, `read_once` and `write_once` report a count of
> zero and issue no host transfer. A zero-length read is never reported as
> `ReadEnd`. [SYS-8]

So `ReadBytes(count) ⇒ count > 0` is **false** in general: a `capacity` of
zero yields `ReadBytes(0)`. The true stated fact is `capacity > 0 ∧
ReadBytes(count) ⇒ count > 0`. Any postcondition written for the checker must
carry that guard.

### `write_once` — (b)

Same generic [SYS-8] target-contract sentence, plus its own:

> `write_once` performs at most one host output attempt [SYS-8], and its
> accepted count means exactly that the host operation accepted that prefix:
> it promises neither line atomicity nor storage durability. [SYS-12]

"that prefix" is a prefix of the requested range, so `accepted <= count`
follows — again by reading a byte-disposition statement as a magnitude
statement. Note the bounding parameter here is spelled `count`, not
`capacity` ([SYS-8]: "for `write_once` it is `offset` and `count`"), so the
result and the bound share no name with the read and copy cases; a single
blanket sentence has to say so.

The [SYS-6] outcome table adds only "the `Ok` payload is the exact `u64`
byte, encoded, or accepted length" — a naming of the quantity, not a bound.
The stated `Ok(0)` exclusion is again nonempty-range-only.

### `host_copy_bytes` and `host_copy_utf8` — (b)

> A successful copy changes exactly the requested destination prefix and
> leaves the rest of the buffer unchanged. [SYS-8]

> `host_copy_bytes` performs the lossless transfer defined by [SYS-9] and has
> no failure mode beyond `CopyTooSmall(required)`. `host_copy_utf8` first
> validates and measures the encoding and returns `Utf8CopyInvalid()` or
> `Utf8CopyTooSmall(required)` without writing any byte, and only then copies
> the complete encoding. [SYS-8]

The chain to `copied <= capacity` is longer here than for `read_once` and
crosses two rules: the [SYS-6] sentence says the `Ok` payload is the exact
byte or encoded length *of the source*; [SYS-8] says the successful copy
writes the *complete* source into a *prefix of the requested range*; the
range has size `capacity`. Only the conjunction bounds the result. Nothing
states that the returned number equals the number of bytes written, which is
the joint on which that whole derivation turns.

### `args_count` — (c)

The complete normative text about this operation's result is "`args_count` is
total." [SYS-9] and the [SYS-6] table row "`own u64`; total, no failure
outcome". Nothing bounds it — nor can anything, since an argument count is
environment magnitude with no program-side referent.

The load-bearing missing fact is not a bound but the **index relation** to
`arg_get`. [SYS-6] says only that "`InvalidIndex` states that the requested
argument index is not present and returns no value", which never says which
indices are present. Consequently a source loop of the obvious shape —
`position` from zero while `position` is less than `args_count(args)` — cannot
prove its `arg_get` succeeds, and every argument read carries a dead `Err`
branch the checker cannot discharge.

Stability, which the relation also needs, is already stated and needs no
addition: "`args_count` and `arg_get` borrow it and leave it live, and no
operation changes its source-visible state" [SYS-9].

### `host_bytes_len` and `host_utf8_len` — (c)

> On such a family `host_bytes_len` returns the exact count of the host
> string's native bytes[.] [SYS-9]

This defines the value against the *host string*, which is not a program
quantity, so there is nothing checkable in it. `host_utf8_len` gets even less:
only the shared [SYS-6] clause naming the `Ok` payload the exact encoded
length.

These two are the lowest-priority rows. Their missing relation removes a
*failure branch* (a copy into a buffer sized by the length cannot report
`CopyTooSmall`) rather than unblocking cursor arithmetic — the copy's own
`copied <= capacity` bound is what the arithmetic needs, and that is a (b) row
above. They are listed because the length-then-size-then-copy sequence is the
only way v0 source can read an argument at all, so the dead-branch cost is
paid at every such site.

### Failure-payload counts

`CopyTooSmall(required: u64)` and `Utf8CopyTooSmall(required: u64)` also
return counts. Their meaning is stated better than any success count:

> `CopyTooSmall(required)` and `Utf8CopyTooSmall(required)` state the exact
> length the destination range must have for the same call to succeed.
> [SYS-6]

That is a stated sufficient-and-necessary length for a retry, so a
grow-and-retry loop is provably terminating in one step. What is absent is the
companion bound `required > capacity` on the failing call — needed only if a
program computes a new size by comparing the two. No current probe needs it;
recorded, not proposed.

## Two cross-cutting findings

**The spec has no postcondition vocabulary at all.** The only contract
construct in v0.20 is [FN-8] `requires`, and it is explicitly the wrong shape:
"The block is a checked callee-entry prologue, not an assumption and not a
caller proof obligation". There is no `ensures` on `fn_sig`, and system
operations have no body to carry one. So these bounds cannot be expressed as
source contracts on the operations; they have to be normative prose attached
to the operation's semantic ID, which [QUAL-1] already declares binds "the
operation's signature, complete outcome set, ownership transitions, memory and
external effects".

**The style precedent for that prose already exists in the same section.**
[SYS-12] states a checker-visible fact in exactly the needed form: "The
checked program additionally retains the conservative fact that redirection
may make the two owners the same sink [DIAG-2]". Every proposed sentence above
is written to that template — a fact, plus its retention in the checked
program, plus the [DIAG-2] citation — so the amendment introduces no new
mechanism, only new facts in an existing form.

## Consolidated alternative

The four (b) rows are one paragraph rather than four sentences, appended to
[SYS-8]'s "Buffer and cursor disposition is exact." paragraph and **replacing
in place** its current target-facing sentence:

> Every successful count is bounded by the caller's validated range, and the
> checked program retains that bound as a fact about the returned value
> [DIAG-2]. On `ReadBytes(count)` the count is at most the requested
> `capacity`; on a successful `write_once` the accepted length is at most the
> requested `count`; on a successful `host_copy_bytes` or `host_copy_utf8` the
> copied length is at most the requested `capacity`. These are postconditions
> of the operations, not defensive obligations on source: a target returning a
> larger count violates its compiler-owned contract [QUAL-1], and source code
> neither checks nor branches on that possibility.

The three (c) rows do not consolidate — they belong to different rules
([SYS-9] for all three, but to different paragraphs and different operand
relationships) and are listed individually above.

## Summary

Seven count-returning operations surveyed, out of eleven system operations.

| classification | count | operations |
|---|---|---|
| (a) stated as a checkable bound | **0** | — |
| (b) implied in prose, not stated as a bound | **4** | `read_once`, `write_once`, `host_copy_bytes`, `host_copy_utf8` |
| (c) absent | **3** | `args_count`, `host_bytes_len`, `host_utf8_len` |

Four of the seven need one sentence each (or one shared paragraph); three need
a relation stated for the first time. PROBE-TAINT.md's two load-bearing
assumptions — `read_once` returns `count <= capacity` and `host_copy_bytes`
returns `copied <= capacity` — are both (b): true under the current text,
usable by a reader, and not yet usable by a checker.
