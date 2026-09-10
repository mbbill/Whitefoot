# First-match search as a semantic-proof probe

## Question and criterion

Can ordinary Whitefoot code, annotated with a small proposed proof vocabulary,
connect a readable first-match specification to a complete implementation?
Can a caller compose two independently checked range searches using only the
range contract, without reopening the search loop?

This is a syntax and proof-obligation experiment, not a selected language
amendment or an implemented proof checker. The active specification and compiler
are unchanged. The constitutional requirements are machine-checked correctness,
erased proof, and practically bounded checking; see the
[constitution](../../../docs/constitution.md) and the current
[source-proof grounds](../proof-certificate-architecture/SOURCE-CHECKING.md).

The discriminating criterion is to write all base, preservation, return,
exhaustion, and composition obligations, with every non-arithmetic lemma given
a body. A missing bridge from loaded storage to the mathematical sequence, an
unjustified universal claim, a circular lemma, or an unproved return counts as
a gap, not as a successful design. Runtime tests establish behavior on their
enumerated inputs only; review of proposed proof text is not machine proof.

First-match byte search is deliberately smaller than JSON: it needs sequence
contents, bounded quantification, first-match/absence guarantees, a loop, and
modular composition, without Unicode, allocation, recursive parsing, or
numeric conversion. A scalar arithmetic example would not exercise sequence
semantics or the absence case. JSON remains a later discriminator; this probe
cannot establish its feasibility. No performance advantage is claimed for
splitting a scalar scan into two calls.

## Artifacts and lifetime

- `search.wf` is runnable source under the current language, including bounded
  executable checks of the mathematical postcondition.
- `search.proposed.wf` shows the same search implementation with explicitly
  unsupported model and proof syntax. It is evidence, not conformance input.
- This document owns the candidate rule meanings, actual results, and gaps.

These files serve the semantic-contract experiment. Update them together while
the probe is used; merge or remove superseded sketches when a successor takes
over, retaining only evidence that still answers a concrete design question.

## Top-level promise

For every finite byte sequence `s` and byte `needle`, return the least index
whose byte equals `needle`, or `length(s)` when no such index exists. Terminate
without changing the input or performing external I/O. As for current
Whitefoot, execution remains conditional on the declared host/TCB and resource
availability assumptions; this probe allocates nothing in the search itself.

The implementation chooses an internal split and searches the prefix before
the suffix. The split is not an input-domain restriction of the top-level API.
The internal range search requires `0 <= lo <= hi <= length(s)` and returns an
index in `[lo, hi]`, using `hi` as the absence sentinel.

The predicate `First(s, needle, lo, hi, r)` states four obligations:

```text
lo <= r <= hi
every k in [lo, r) has s[k] != needle
if r < hi, then s[r] == needle
```

At the top level, `lo = 0` and `hi = length(s)`. When `r = hi`, the prefix
condition covers the entire input, proving absence. When `r < hi`, it proves
that the matching position is the first one. Thus neither an always-absent
implementation nor one returning an arbitrary matching index satisfies the
promise. This is a relation specification, independent of loop structure,
split selection, and implementation data representation.

## Concrete proposed syntax and checking rules

All additions are inside `proof { ... }` blocks in `search.proposed.wf`.
Deleting those blocks recovers the first three functions of `search.wf`, up to
whitespace. The executable `satisfies` function and command entry are a test
harness, not part of the proposed proof. No proof is obtained by evaluating
that test harness during compilation.

The following is a finite proposed calculus for reading the sketch. It is not
implemented or specified as accepted Whitefoot. The reader must not interpret
an unexplained word such as `arith` as a universal theorem prover.

| Construct | Proposed checking obligation |
|---|---|
| `Seq<u8>`, `Int`, `entry_contents(input)` | A finite mathematical byte sequence and immutable mathematical integer value images. The compiler must establish that the sequence length equals the entry slice length and that its elements denote the actual referent bytes. This is a new semantic bridge, not a writer-provided axiom. |
| Integer names in proof positions | Refer to current immutable value images, without executing arithmetic. Entry parameters used here are not reassigned. `size`, for example, is connected to `length(s)` by its checked `len_of(input)` operation; no name-based matching is allowed. |
| `predicate` | A nonrecursive transparent definition. A `requires` domain is established before forming its body or proof arguments. `First` is a conjunction with named fields: earlier range fields justify later bounded reads and `Clear` formation. An implication's premise is in scope while forming its conclusion. |
| `forall k in lo..hi`, `intro`, `instantiate` | Universal introduction uses a fresh arbitrary integer and the explicit bounds. Elimination substitutes the written integer after checking its bounds. The checker does not enumerate runtime elements or guess instantiations. |
| `lemma` | A named, parameterized, fully checked finite derivation. Calls supply every argument. The three lemma dependencies are acyclic; unchecked lemmas and circular references are not available. |
| `cases` | Explicit proof-only split on the written integer comparison, with both cases checked. It emits no runtime branch. |
| `arith(facts) proves goal` | Only the current fixed arithmetic fragment, applied to the written numeric premises plus type/domain facts, checks these small inequalities. `bounds` names the introduction bounds; `header` names the counted-loop bounds; `requires` names checked requirements; `types` names fixed type/domain facts. It has no sequence or quantified reasoning. Contradictory bounds use the existing contradiction rule. |
| `rewrite(..., at: ...)` | Check the stated equality and replace the indicated occurrence, in the direction required by the written target. No rewrite search or arbitrary normalization. |
| `load_image` | Check that the named actual load read the named input at the named index, that its bounds succeeded, and that the relevant content image is still current. This cannot be called on an unrelated value to manufacture an equation. |
| `edge` | The selected branch proposition, or its negation on the other edge. After an `if` whose true arm returns, only the false edge reaches the following proof. This is compiler-derived flow evidence, not an assumed proposition. |
| `invariant`, `base`, `next` | Establish the base before granting the arbitrary-header assumption, then prove the simultaneous next-header obligation at every reachable backedge. `next` substitutes the counted binder's mathematical successor. The writer cannot import a still-unproved next-header fact. |
| `exhausted(scanned)` | Export the proved invariant at `i = hi` only on exact normal counted exhaustion with `lo <= hi`; an empty range uses its checked base. It does not apply to `break` or an early return. This explicit projection is the only escape of the loop-local proof name. |
| `call_summary(value)` | Instantiate the checked callee relation for that actual call's result, arguments, and content image. The caller does not inspect its body. The image must remain valid through the call. |
| `close return` | Prove the function's declared semantic postcondition for the immediately following return, including every field. `introduce` checks an implication under its premise; `apply` eliminates it; `false_elim` requires an independently obtained contradiction. |
| `terminates structural` | Check finite counted loops, nonrecursive control, and terminating callees in acyclic call order. A `pure` but possibly diverging callee is insufficient. This probe needs no guessed ranking function or recursive summary. |

The sequence bridge stays valid here because the source has read-only slice
access, no input mutation, and no concurrent writable alias admitted by the
ownership rules. A write through an alias in a future example would require a
new content image, proof of preservation, or invalidation of facts about the
current contents. Old immutable images must not silently describe new storage.
Erased sequences have no runtime allocation, copy, or read pass.

The sketch's declarations before a function body attach to that function;
the block between a `for` header and its body attaches to that loop. Proof
names have their own scopes; `scanned` is body-local except for the explicit
exhaustion projection. `next` is an obligation annotation on the imminent
backedge, not an executable update. These placements are proposed syntax,
not a change to current invariant scope or current `use` arithmetic.

The step and join lemmas use only bounded universal introduction/elimination,
integer comparison splitting, and substitution. `clear_empty` additionally
uses contradiction. No user lemma is left as a trusted declaration.
There are three lemmas in total: empty, one-element extension, and concatenation.

## How the proof composes

The scan base is `Clear(s, needle, lo, lo)`. For an arbitrary current index i:

1. The counted guard and range requirement prove the actual load is in bounds.
2. The load bridge gives `byte == s[i]`.
3. On a matching edge, the old prefix invariant and the load equation prove
   all four fields of `First(..., i)` before returning i.
4. On the other edge, the load equation proves `s[i] != needle`. `clear_step`
   extends the prefix to `i + 1`, proving the backedge invariant.
5. Exact exhaustion substitutes `hi` for i, yielding the whole-range absence
   case. The hit implication has a false premise `hi < hi`.

`find_split` uses only the two instantiated `scan_range` contracts:

- A left hit already has no preceding hit from zero and lies within the
  complete input, so it satisfies the top-level promise.
- Otherwise, the left range bound plus the selected false edge proves
  `left == split`; rewriting the left prefix fact gives absence over the
  complete first segment.
- `clear_join` combines that fact with the right result's prefix fact.
  Its hit field and upper bound come directly from the right contract.

`find_first` chooses `length / 2` and transfers the checked `find_split`
contract. The ordinary compiler already proves that this internal split is
within the slice. All three bodies terminate by the proposed structural rule:
the scan has a finite counted loop with no user calls, and each wrapper makes
a finite number of calls to already terminating functions.

The public promise requires no knowledge of this decomposition. A different
implementation can retain the same promise, but must supply a different proof
when the existing one no longer matches its code.

## Checking-cost boundary

No proposed rule searches for a lemma, split point, quantifier witness, loop
invariant, or intermediate equation. Predicate unfolding is requested at a
specified definition and occurrence. A possible implementation would retain
explicitly shared terms and a proof DAG, compare actual term identities, and
check the indicated operations. It must account for term/substitution size
and the existing arithmetic query cost, not merely count proof lines.

That description does not establish an implemented polynomial bound or useful
latency. The current arithmetic engine's measured limits still apply. In
particular, the sketch must not be implemented by expanding every bounded
quantifier, executing arbitrary recursive model functions, or adding automatic
proof search behind `close return`. Acceptance would remain independent of
timeouts and cumulative work limits. Runtime loop length and compiler proof
length are different quantities: this invariant uses one arbitrary iteration.

## Validation

The ordinary program builds with the unchanged compiler and exits zero. Its
test harness evaluates the postcondition directly instead of computing a
second search result: it checks range membership, the returned byte when
present, and every earlier position. Those test branches are legitimate
failure reporting in the harness; the three search functions add no branches
merely to satisfy the safety checker.

The bounded input family is every length from 0 through 8, every pattern from
0 through 255 used to populate bytes with 0 or 255, and needles 0, 255, and 1.
Shorter lengths repeat patterns. The harness checks every valid subrange and
every split, including empty segments. This is not an exhaustive test over
all byte sequences or a machine proof of the candidate semantics.

Reproduce the baseline from the repository root:

```sh
cargo build --manifest-path compiler/Cargo.toml --release --locked --offline --bin whitefootc
compiler/target/release/whitefootc research/investigations/semantic-proof-probe/search.wf -o /tmp/whitefoot-semantic-search
/tmp/whitefoot-semantic-search
```

Recorded on Darwin arm64 with Rust `1.98.1 (48a229cea 2026-09-01)`, using the
compiler at base `30168426` and these investigation files. These are exploratory
results; the repository does not retain a separately timestamped criterion:

| Check | Result |
|---|---|
| Release compiler build, locked/offline | Passed |
| Native baseline | Exit 0; 6,912 top-level, 34,560 split, and 126,720 range postcondition checks (168,192 total) |
| Erase every proposed `proof` block | The three resulting functions match the runnable functions after removing whitespace |
| Compile the proposed file directly | Rejected at `proof` on line 1 (FORM-1), as expected for unsupported syntax |
| `make static` | Passed |

A disposable safe-Rust driver generated each mutation in scratch storage,
compiled it with the same compiler, and ran the unchanged executable
postcondition harness. No compiler path or proof obligation was disabled.
The mutations are reproducible by applying the following edits independently
to `search.wf`; none are shipped as valid implementations.

| Mutation | Exact change in the search functions | Current compiler / native harness | Candidate obligation that would fail |
|---|---|---|---|
| Always absent | At the matching return in `scan_range`, return `hi` instead of `i` | Accepted / exit 1 | Whole-range `Clear` cannot follow when the current byte matches |
| Wrong byte | Change the scan comparison from `byte == needle` to `byte != needle` | Accepted / exit 1 | Return's hit equation contradicts the selected edge |
| False found | At scan exhaustion, return `lo` instead of `hi` | Accepted / exit 1 | Exhaustion does not establish a hit at `lo` |
| Discard left | Delete `if left < split { return left; }` in `find_split` | Accepted / exit 2 | No false-edge evidence establishes `left == split`; prefix absence cannot be obtained |
| Skip first match | Add `let seen = False();` before the scan loop; replace its matching arm with `if seen { return i; } set seen = True();` | Accepted / exit 1 | Falling through after the first matching byte cannot establish the extended no-match prefix |

The last column is a prediction from the written candidate derivations, not
an observed proof-checker rejection. Only ordinary compilation, native
execution, and textual erasure were run. The earlier version of the
always-absent mutation removed all reads and was rejected for its now-inexact
effect row; retaining the scan while returning the sentinel isolates the
logic gap without weakening any current check.

## What this experiment establishes and leaves open

There is a runnable implementation under the current safety rules, a
representation-independent top-level relation, and an explicit proposed
derivation for each return, loop edge, and composition step. Every user lemma
in that derivation has a body. The experiment makes the missing mechanisms
concrete instead of hiding them in a `prove_correct` placeholder.

It does **not** establish that Whitefoot now verifies this semantic relation.
The new syntax has no parser or checker. The contents/load/call bridge, the
quantified proof rules, their implementation soundness and costs, and the
new termination judgment remain unverified implementation work. A human or
agent review of this sketch cannot substitute for that work.

The criterion is therefore met as a written derivation candidate and a running
behavior probe, subject to those explicit primitive-rule assumptions. It is
not met as an end-to-end machine-checked semantic proof. A useful next
discriminator would be implementing these general rules for this same source
and checking that the baseline succeeds and the five mutations cannot close
their contracts, without function-name or corpus special cases.

This probe selects no permanent language design. The affected set is this
investigation only; active specification, current implementation map,
conformance evidence, and rule-index conclusions are unchanged. JSON's
recursive summaries, parsing failures, Unicode, mutable output models, and
performance-oriented representation refinements remain outside the evidence.
