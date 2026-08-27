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
or with which proof; four are pure representation and one removes a copy.

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
   `intern_for` no longer clones the node it interns.
   693 M → 55 M per inventory.
2. **`DerivationLedger::settle()`** (`entailment/state.rs`), called by
   `flow::analyze_candidate` and `analyze_candidate_masked` — the two entry
   points that deliberately defer `finish`. It shrinks the arena's vectors to
   their length and drops the interning index the way a clone drops it, marking
   it stale so a later query replays it. Neither is observable: capacity is not
   read anywhere, `finish_with_event_roots` rebuilds every vector at its
   retained length, and the [DIAG-2] byte metric is taken after that rebuild.
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

## Result

Gate profile, `whitefootc --par -o /dev/null`, five runs interleaved between
the two binaries on an otherwise idle machine; medians, with maximum resident
set size from `/usr/bin/time -l`.

| | before | after | ratio |
| --- | --- | --- | --- |
| `wfgrep.wf` peak RSS | 3125.0 MiB | **419.0 MiB** | **7.46×** |
| `wfgrep.wf` wall | 6.86 s | 4.23 s | 1.62× faster |
| `dir_walk.wf` peak RSS | 436.6 MiB | **93.8 MiB** | **4.65×** |
| `dir_walk.wf` wall | 0.72 s | 0.62 s | 1.16× faster |

The suite run that checks `wfgrep.wf` — the `programs` gate binary filtered to
`wfgrep`, twelve tests over the compiler's own thread pool, medians of three
interleaved runs:

| | before | after | ratio |
| --- | --- | --- | --- |
| peak RSS | 4807.7 MiB | **919.4 MiB** | **5.23×** |
| wall | 10.67 s | 4.94 s | 2.16× faster |

Every step also made the check faster, which is the shape a memory fix takes
when the memory was copies: an index entry that stores twelve bytes instead of
228 and never copies a node, arena entries copied at 64 bytes instead of 208,
and a `FunctionEntailment` moved rather than deep-copied are all strictly less
work than what they replace.

## Oracle

`whitefootc --par --emit-llvm --par-ledger --stack-ledger` over **623 sources**
— every `tests/programs/*.wf` (25), `tests/codegen/**/*.wf` (95) and
`tests/conformance/cases/*.wf` (503) — comparing stdout, stderr, exit status
and the emitted LLVM IR against a compiler built from `79b29665` exported into
a scratch tree: **0 differences** (`diff -r` over 2132 captured files —
stdout, stderr and exit status for all 623, plus the IR text of the 262 that
compile).

`cargo test --profile gate --all-targets --locked --offline`: all library
tests, 54 maintained programs, conformance — green. Canonical `make check`
green end to end (`== WHITEFOOT ALL TESTS GREEN ==`).

## Regression guards

Three new, each asserting a mechanism rather than a number, plus one existing
guard adapted to the lending API.

- `interning_into_a_settled_ledger_keeps_the_original_identity`
  (`entailment/state.rs`) — interning a node already present into a *settled*
  ledger returns the original identity and grows nothing, and still separates
  proof views. `settle` drops the index the way a clone drops it rather than
  the way `finish` resets it; getting that backwards would give one proof step
  two identities, and every [CLM-2] rerun analyzes a settled ledger again.
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
- Temporary instrumentation — arena and index footprints per structure, the
  node-kind histogram, the post-`finish` node count — was removed before
  committing. The numbers it produced are the attribution tables above.
- `mcts_mem/` was not consulted or written: no recorded design choice was
  re-decided.

## Approval classes for the merge

- No specification bytes change.
- No conformance content changes.
- No new root entries.
