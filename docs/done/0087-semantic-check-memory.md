# Batch 0087 — four gigabytes to check a 1400-line program

Branch: `batch/0087-memory`, from `integration/2026-08-27` at `79b29665`.
Deliverable: one commit on that branch, plus this record.

## Charter

Batch 0083 cut the semantic check of `tests/programs/wfgrep.wf` from about 25 s
to about 7 s and observed, in passing, that its peak footprint fell from 5.9 GB
to 4.5 GB. Spending gigabytes to check 1400 lines is a defect in its own right.
The charter asked for a 4× cut in peak resident memory with no behavior change
and no time regression, and named three leads from the 0083 record: the
whole-inventory clones each [CLM-2] rerun makes, the dense closure matrices,
and the interning index.

Two of the three leads are wrong, and the measurement below says so.

## Measured attribution

Temporary counters in `Checker::claim_residuality_outcome`,
`counterfactual_witness` and `DerivationLedger`, on `wfgrep.wf` before the
change. Bytes are the arena's own vectors plus the heap their nodes own; the
index column is the interning table.

| live structure at the [CLM-2] peak | arena bytes | index bytes |
| --- | --- | --- |
| `functions` — the baseline analysis, still unpruned | 755 M | 693 M |
| one rerun's `scratch` inventory | 755 M | 693 M |
| `CounterfactualReuse`'s stored analyses | 514 M | 0 |
| `baseline_functions` — a clone kept alive by scope | 514 M | 0 |
| `masked_functions` — `counterfactual_witness`'s clone | 514 M | 0 |
| **total** | **3052 M** | **1386 M** |

That same instrumented run measured a peak footprint of 4878 M and a maximum
resident set of 2859 MiB; the un-instrumented medians are under **Result**.
Resident memory tracks the arena column, not the index column: a hash table's
unused slots are allocated but never touched.

Inside one inventory:

| quantity | value |
| --- | --- |
| `size_of::<DerivationNode>()` | **208 bytes** |
| retained nodes, all functions | 2 313 251 |
| nodes surviving `finish`'s prune | **15 655 (0.68 %)** |
| arena vector capacity vs. length | about 1.5× |
| `size_of::<Option<ClosedCell>>()`, matrix dimension | 48 bytes, ≤ 91 |

and by node kind, over those 2 313 251:

| kind | count |
| --- | --- |
| `JoinBound` | 1 134 036 |
| `TransitiveBound` | 931 897 |
| `JoinDistinct` | 131 371 |
| `DisequalityFromStrictBound` | 72 216 |
| `SubsumedBound` | 32 620 |
| everything else | 10 762 |
| all nine `Postcondition*` kinds together | **349** |

Three readings decide the batch.

- **`phase_a.to_vec()` copies nothing.** `claim_counterfactual_inventory` is
  cloned from the phase-A inventory *before* the baseline entailment pass, so
  every ledger in it is empty: 0 nodes, 0 bytes. The 0083 record's first lead
  is a copy of an empty structure. What the rerun actually costs is the
  analysis it then performs into that copy.
- **The dense closure matrix is not a memory cost.** 91² cells of 48 bytes is
  400 KB, transient, one at a time. The 0083 record's second lead is four parts
  in ten thousand of the peak.
- **The arena is 99.3 % garbage held by one flat array whose width is set by a
  variant that occurs 349 times.** `PostconditionCall` carries a `NodePath`, a
  `Relation`, a summary reference and four vectors — 200 bytes — and every
  transitivity step, which needs 36, pays for it. Nothing can prune the garbage
  before [CLM-3] registers its roots, so the fix is to make each entry cost
  less and to stop keeping five inventories of them.

## Mechanism

Five changes. None changes which candidates the closure accepts, in what order,
or with which proof; four are pure representation and one removes a copy. That
claim is about proof search, so **Oracle** below measures it at the derivation
and not only at the emitted program — an adversarial review broke the first
version of change 4 while the emitted program stayed identical everywhere.

1. **The interning index keys a hash, not a node** (`entailment/state.rs`).
   `InternIndex` was `HashMap<(ProofView, DerivationNode), DerivationId>`, so
   the table held a second full copy of every node — a 224-byte key beside a
   4-byte value. It is now `HashMap<u64, DerivationId>` keyed by the node's own
   hash, and `probe_intern` walks from that key, stepping by one while the key
   it reaches is held by a different node. Every probe still compares the
   candidate against `nodes` and `node_views` before reporting a hit, so a
   shared hash costs one step and never a wrong identity; nothing is ever
   removed from the index, only rebuilt or cleared whole, so a free key always
   ends the walk. `rebuild_intern_index` replays the same walk in identity
   order and still lets a repeated identity keep the later entry.
   `intern_for` no longer clones the node it interns. Absence is the index's
   only state and is read off the ledger — empty entries beside a non-empty
   arena means a replay is owed — rather than tracked by a flag each dropper
   has to set correctly.
   693 M → 55 M per inventory.
2. **`DerivationLedger::settle()`** (`entailment/state.rs`), called by
   `flow::analyze_candidate` and `analyze_candidate_masked` — the two entry
   points that deliberately defer `finish`. It shrinks the arena's vectors to
   their length and drops the interning index. Neither is observable: the index
   is derived from the nodes, so the next interning replays it, and every
   vector it releases is one `finish_with_event_roots` rebuilds at its retained
   length before the [DIAG-2] byte metric reads its capacity. `roots` is the
   exception — that pass rewrites each root's identity in place instead of
   rebuilding the vector, so its capacity does reach the metric — and `settle`
   leaves it alone.
   755 M → 514 M per inventory, before the width change below.
3. **The arena entry narrows from 208 bytes to 64.** `PostconditionCall` and
   `PostconditionDeliveryJoin` now hold one `Box<…Detail>` each, and the
   `relation: Relation` of the six other postcondition steps becomes
   `Box<Relation>`. Those nine kinds are 349 of 2.3 M nodes, so nine pointer
   loads buy 144 bytes on each of the other 2 312 902. What remains at 64 is
   `SourceBound`, `SubsumedBound` and `JoinBound`, whose `i128` bounds are the
   floor.
4. **`CounterfactualReuse` lends its analysis instead of copying it**
   (`semantic/check.rs`). The entry now holds `Option<FunctionEntailment>`:
   `take` moves the value into the rerun's inventory, `lend` records the
   published context of a value the inventory already holds, and `reclaim`
   takes every lent value back after `counterfactual_witness` and before the
   inventory is dropped. An entry is lent exactly when its value is absent, so
   a function the rerun analyzed under a mask that names it keeps the
   untargeted value an earlier rerun recorded — the same rule the copying
   version had. One copy exists at any moment instead of two.
   `reclaim` clears the published FN-9 summaries as the value comes back: the
   SCC scheduler stamps them onto the inventory's copy, each rerun takes that
   publication decision again under its own mask, and what the entry holds is
   the analysis, which emits `summary: None`.
   514 M removed.
5. **Provenance takes borrowed functions.** `analyze_program_provenance_with_frozen`,
   `freeze_program_provenance`, `refresh_entailment_views` and their five
   helpers take `&[&CheckedFunction]`; `CarrierReconstructor` holds the same.
   `counterfactual_witness`'s `masked_functions`, `check_program`'s
   `baseline_functions` and `phase_a_functions` become vectors of references.
   PRV reads no derivation arena — `grep -c derivations src/semantic/provenance.rs`
   is 0 — but the borrowed slice needs no such argument.
   514 M + 514 M removed.

`baseline_functions` is explicitly dropped once the last baseline claim
judgment is made, because its borrow otherwise outlives the move of the
inventory it names.

## Alternatives rejected

- **Pruning the counterfactual ledgers early.** 99.3 % of every arena is
  unreachable from any root, and `finish` already deletes exactly that. It
  cannot run before [CLM-3] registers its strict roots, and running it on a
  rerun's scratch would remap the identities recorded in
  `FunctionPostconditionProof`, which feed the next schedule component's
  analysis, and would set an obligation's `derivation` to `None` wherever that
  proof is not itself a root — which is precisely what `masked_terminal_witness`
  reads. That is an acceptance change, not a memory change.
- **`Rc<FunctionEntailment>` shared between the baseline analysis and the
  reruns.** The tempting version seeds the reuse cache from the baseline pass,
  which would remove the rerun's whole first inventory. It is wrong: the
  entailment walk consults *whether* a mask is present at three of its five
  mask reads, so an unmasked baseline analysis is not the value a masked rerun
  computes for an untargeted function. Sharing without seeding buys nothing —
  the two inventories hold genuinely different values — so the ownership churn
  would pay for itself only at the copies change 4 and 5 remove outright.
- **One `DerivationNode::Postcondition(Box<PostconditionNode>)` for all nine
  kinds.** It reaches the same 64 bytes through about 130 edited match sites
  instead of 60, and it renames every postcondition pattern in the tests.
- **Growing the arena by a smaller factor than doubling.** The residue above
  the live inventories is the last `Vec` reallocation of the largest function,
  which holds the old and new buffers at once. A 1.25× growth policy would trim
  it at four times the copying, and it is a hand-rolled growth rule layered on
  `Vec`. Narrowing the entry shrinks the same residue by the same factor for
  free.
- **A wall-clock or RSS regression assertion.** Both are machine numbers. The
  guards below assert mechanism.

## What an adversarial review found, and what it cost

The first version of this record claimed that none of the five changes can
alter a derivation, and offered 623 identical compilations as the evidence. An
adversarial review refuted that claim with a 39-line program and found a second
defect the corpus cannot reach. Both are repaired here, both are guarded, and
the claim is now backed by a derivation-level oracle rather than by emitted
output alone. The IR evidence was never wrong; it was answering a weaker
question than the claim asked.

**A published summary leaked from one counterfactual rerun into the next.** The
copying version called `store` before the SCC scheduler's publish loop, so the
cached value always carried `summary: None` — what the analyzer emits. Lending
inverted that order: the value now lives in the rerun's inventory, the publish
loop stamps a `VerifiedPostconditionSummary` onto its postcondition proofs, and
`reclaim` took it back afterwards. `proof.summary` is not bookkeeping — it is
read to build `verified_postconditions` and `verified_postcondition_proofs`,
which are both the analyzer's `EntailmentContext` and the reuse key — so the
stamp is acceptance-bearing input to every later analysis.

The review's program exposes it. `alpha` and `beta` are a two-function SCC,
each with an `ensures` that only `beta`'s claim can discharge; `gamma` holds an
unrelated, earlier-sorting claim; `delta` calls `alpha` and feeds the result to
a `requires`.

```
const small: array<i32, 4> =[10_i32, 20_i32, 30_i32, 40_i32];

fn alpha(value: own u64) -> result: own u64 traps contract {
  ensures ile(result, 100_u64);
} {
  let ignored = beta(value: value);
  return 5_u64;
}

fn gamma(i: own u64) -> result: own i32 traps {
  let bounded = i % 4_u64;
  let inside = ilt(bounded, 4_u64);
  claim gamma_bound: inside because "premises: p\nderivation: d\nconclusion: c\nchecker gap: g\nconsumers: u";
  return small[bounded];
}

fn beta(value: own u64) -> result: own u64 traps contract {
  ensures ile(result, 100_u64);
} {
  let ignored = alpha(value: value);
  let bounded = value % 1024_u64;
  let inside = ile(bounded, 100_u64);
  claim beta_bound: inside because "premises: p\nderivation: d\nconclusion: c\nchecker gap: g\nconsumers: u";
  return bounded;
}

fn accept_bound(value: own u64, limit: own u64) -> result: own u64 pure contract {
  define admitted = ile(value, limit);
  requires admitted;
} {
  return value;
}

fn delta(value: own u64) -> result: own u64 traps {
  let a = alpha(value: value);
  let ok = accept_bound(value: a, limit: 100_u64);
  return ok;
}

command fn main() -> status: own ExitStatus traps {
  let g = gamma(i: 1_u64);
  let d = delta(value: 3_u64);
  return exit_status(code: 0_u8);
}
```

Masking `gamma_bound` publishes component 0 and stamps `alpha`; masking
`beta_bound` leaves component 0 unpublished, and the entry handed `alpha` back
with the earlier rerun's stamp. Every function scheduled after it then saw one
visible postcondition where the base compiler saw none, and `delta`'s
counterfactual arena was 39 nodes against the base's 30. `reclaim` now clears
the stamp, which restores the exact value the copying version cached and with
it the invariant stated at the `claim_counterfactual_inventory` site — no
baseline entailment or materialized parent leaks into a masked rewalk — and the
documented meaning of `FunctionPostconditionProof::summary`.

**`settle` composed with `clone` minted a second identity for one proof step.**
`InternIndex` carried a `stale` flag, and `Clone` recomputed it as
`!entries.is_empty()`. A settled ledger has empty entries, so its clone was
born looking freshly finished and never replayed the index: re-interning a node
the arena already held returned a new `DerivationId`. The same hole existed at
the base revision for a clone of a clone; what this batch added was a settled
state on every candidate ledger, which is exactly the state the copy
mishandled. The flag is gone. The index has one absent state, read off the
ledger — empty entries beside a non-empty arena — so a clone, a `settle` and a
`finish` are all owed the same replay and no dropper can get it wrong. Nothing
in production interns into a copied ledger today, so no compilation changed;
the defect was a live hazard rather than a live fault.

**One rationale in this record was wrong.** It said `settle`'s `shrink_to_fit`
is unobservable because capacity is not read anywhere. `DerivationLedger::
measure` reads six vector capacities into `metrics.retained_bytes`, and `roots`
is the one vector `finish_with_event_roots` rewrites in place instead of
rebuilding, so shrinking `roots` did move that metric. It reaches no
diagnostic today, so nothing observable changed, but the reason was false.
`settle` no longer touches `roots` — a handful of entries per function beside
millions of nodes — and the oracle below now measures the byte metric instead
of asserting it cannot move.

## Result

Gate profile, `whitefootc --par -o /dev/null`, five rounds interleaved across
three binaries; medians, with maximum resident set size from `/usr/bin/time
-l`. The middle column is this branch *before* the repair above, measured in
the same rounds, so the repair's own cost is visible rather than inferred.

| | `79b29665` | before repair | after repair | ratio |
| --- | --- | --- | --- | --- |
| `wfgrep.wf` peak RSS | 2406.8 MiB | 483.5 MiB | **481.3 MiB** | **5.00×** |
| `wfgrep.wf` wall | 15.57 s | 7.91 s | 8.80 s | 1.77× faster |
| `dir_walk.wf` peak RSS | 436.1 MiB | 94.5 MiB | **94.0 MiB** | **4.64×** |
| `dir_walk.wf` wall | 1.63 s | 1.54 s | 1.29 s | 1.26× faster |

The suite run that checks `wfgrep.wf` — the `programs` gate binary filtered to
`wfgrep`, twelve tests over the compiler's own thread pool, five interleaved
rounds:

| | `79b29665` | before repair | after repair | ratio |
| --- | --- | --- | --- | --- |
| peak RSS | 4265.9 MiB | 849.9 MiB | **849.9 MiB** | **5.02×** |
| wall | 28.78 s | 14.91 s | 12.88 s | 2.23× faster |

These rounds ran on a machine carrying other builds, and the walls are worth
only their ratios. The peaks are worth more, but not equally: `dir_walk.wf`
spans 434.8–437.7 MiB before and 93.3–95.9 MiB after, and the suite spans
3644.7–4726.8 MiB before and 829.0–921.9 MiB after, while the single-file
`wfgrep.wf` peak on `79b29665` spans 1460.4–4745.8 MiB. That last spread is
the measurement telling the truth: the peak is set by how many function
inventories are live at once, so on the old representation it tracks how much
of the check the machine let run concurrently. After the change each live unit
is small enough that the same variation costs 321.3–494.3 MiB. An earlier,
quieter run of the same comparison read 3125.0 MiB against 419.0 MiB, at the
top of that same range.

Every step also made the check faster, which is the shape a memory fix takes
when the memory was copies: an index entry that stores twelve bytes instead of
228 and never copies a node, arena entries copied at 64 bytes instead of 208,
and a `FunctionEntailment` moved rather than deep-copied are all strictly less
work than what they replace. The repair costs neither: clearing a handful of
`Option`s per function and leaving one small vector unshrunk are at the noise
floor of both tables.

## Oracle

Two comparisons, both against a compiler built from `79b29665` exported into a
scratch tree by `git archive`, over **623 sources** — every
`tests/programs/*.wf` (25), `tests/codegen/**/*.wf` (95) and
`tests/conformance/cases/*.wf` (503) — plus the review's `scc_publish.wf`.

**Emitted output.** `whitefootc --par --emit-llvm --par-ledger --stack-ledger`,
comparing stdout, stderr, exit status and the emitted LLVM IR: **0
differences** (`diff -r` over 2135 captured files — stdout, stderr and exit
status for all 624, plus the IR text of the 263 that compile).

**Derivations.** The claim this batch makes is about proof search, and identical
IR does not establish it: the refuted version of change 4 emitted identical IR
on every one of these sources while searching a different arena. A temporary
probe — added to both scratch trees, never to the worktree, removed before
committing — prints, for every function of every inventory analysis and for
every finished ledger, the arena's node count, parent-edge count, root count,
event count, published-summary count, and a hash of the whole DAG: each node's
proof view, its depth, its parent and retained-reference identities in order,
and its flow event. Those are `DerivationId` and `FlowEventId` values, so the
fingerprint is comparable across the two node representations even though the
node's bytes are not. On `wfgrep.wf` it covers 2 313 251 arena nodes at the
first of the six inventory analyses that program performs — the attribution
table's figure — and within 220 of that at the other five. Over the same 624
sources: **0 differences**.

The probe has teeth. Run against the branch *before* this repair it reports the
two lines the review reported — `published=1` where the base compiler has 0,
and `nodes=39 edges=21 roots=4` where the base compiler has `nodes=30
edges=15 roots=0` — on `scc_publish.wf`, and nowhere else in the corpus.

**The [DIAG-2] byte metric.** `settle`'s only claim is that it is invisible, so
it is measured rather than argued: the same probe records
`metrics.retained_bytes` beside every finished ledger, and the branch compared
against itself with `settle` disabled is byte-identical over all 624 sources.
That number is *not* comparable to the base compiler — narrowing the arena
entry from 208 bytes to 64 is exactly what it measures — and it is the only
field of the fingerprint that differs there.

`cargo test --profile gate --all-targets --locked --offline`: all library
tests, 54 maintained programs, conformance — green. Canonical `make check`
green end to end (`== WHITEFOOT ALL TESTS GREEN ==`).

## Regression guards

Five new, each asserting a mechanism rather than a number, plus two existing
guards extended. The three that cover the two repairs were checked against the
code they guard: reverting both repairs in a scratch copy of the tree turns
exactly those three red — `DerivationId(1)` where `DerivationId(0)` is owed,
twice, and a summary that survived the reclaim.

- `interning_into_a_settled_ledger_keeps_the_original_identity`
  (`entailment/state.rs`) — interning a node already present into a *settled*
  ledger returns the original identity and grows nothing, and still separates
  proof views. Getting that wrong would give one proof step two identities, and
  every [CLM-2] rerun analyzes a settled ledger again.
- `interning_into_a_clone_of_a_settled_ledger_keeps_the_original_identity`
  (`entailment/state.rs`) — the composition the check actually runs: a
  candidate ledger is settled once and every rerun clones it. This is the
  review's second finding. Neither guard above it composes the two, and the
  defect lived exactly in the composition.
- `interning_into_a_cloned_ledger_keeps_the_original_identity` (batch 0083)
  now also interns into a clone *of a clone*, which is the same hole reached
  without `settle` — and the shape that was already broken at `79b29665`.
- `a_published_summary_is_not_lent_to_the_next_rerun` (`semantic/check.rs`) —
  a value reclaimed from an inventory whose postcondition proofs the scheduler
  published comes back with no summary on any of them. This is the review's
  first finding: the entry holds the analysis, and the analysis emits
  `summary: None`.
- `the_derivation_arena_entry_stays_narrow` (`entailment/state.rs`) —
  `size_of::<DerivationNode>() <= 64`. The arena is one flat array of millions
  of entries and the counterfactual holds two at once, so the widest variant
  sets the cost of the whole check: on `wfgrep.wf` two live inventories of
  2.3 M entries cost 4.6 MB of resident memory for each byte of this width.
  A future variant that inlines a `NodePath` or a
  `Relation` fails here instead of silently costing a gigabyte.
- `a_lent_counterfactual_entry_is_not_a_second_copy` (`semantic/check.rs`) — a
  recorded context holds no value until the rerun gives one back; `reclaim_one`
  moves the arena out of the inventory; the next `take` yields it and leaves
  the entry empty. A second take before the reclaim would mean two live copies,
  which is the whole point of the entry.
- `a_changed_published_context_is_not_reused` (batch 0083) keeps every one of
  its refusals — a different published FN-9 context, a different context
  length, a different function index — and now additionally asserts that the
  matching key succeeds, because with lending a hit is observable as the value
  leaving the entry.

## Judgment calls

- Two of the charter's three leads were dropped on measurement rather than
  investigated further: `phase_a.to_vec()` copies empty ledgers, and the dense
  matrix is 400 KB. The record states the numbers so the leads are not
  re-followed.
- The width change stops at 64 bytes rather than going lower. Below that sits
  `i128`, which the bound arithmetic needs; taking it out would be a semantic
  change made for memory.
- `relation: Box<Relation>` narrows six variants with a one-field change
  instead of a boxed payload struct each. The two variants that needed more
  than their relation moved — `PostconditionCall` and
  `PostconditionDeliveryJoin` — got a named `…Detail` struct, so the pattern
  `{ .. }` sites in both compiler and tests kept working.
- `CounterfactualReuse::reclaim_one` exists beside `reclaim` so the guard can
  drive one entry without building a `CheckedFunctionInventory`.
- `settle` drops the whole index rather than shrinking it. A shrink would
  rehash 2.3 M entries to keep a table nothing will add to; the rebuild is one
  linear pass and only happens if something interns again.
- The leaked summary is cleared on the way back into the entry rather than on
  the way out of it. Both restore the same value; clearing at `reclaim` is the
  form under which the entry never holds a post-publish entailment at all, so
  the invariant is a property of the cache rather than of every reader of it.
- The `stale` flag was deleted rather than repaired. Setting it correctly is
  three sites' obligation and two of them had it wrong; reading absence off the
  ledger is one condition that cannot disagree with itself. The cost is a
  rebuild after `finish` if anything interns again, which is the case that used
  to mint a duplicate identity.
- The review's program did not join `tests/programs`. It compiles identically
  on both sides of the defect it exposes — that is the whole point of the
  finding — so a corpus program cannot fail on it and would only cost a check.
  What guards the finding is the mechanism test; the program is reproduced
  above so the oracle can be rerun without the review's scratch tree.
- Temporary instrumentation — arena and index footprints per structure, the
  node-kind histogram, the post-`finish` node count, and the derivation
  fingerprint the oracle above compares — was removed before committing. The
  numbers it produced are the attribution tables above and the oracle result.
- `mcts_mem/` was not consulted or written: no recorded design choice was
  re-decided.

## Approval classes for the merge

- No specification bytes change.
- No conformance content changes.
- No new root entries.
