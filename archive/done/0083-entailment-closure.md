# Batch 0083 — the counterfactual rerun: entailment cost without a verdict change

Branch: `batch/0083-ent-closure`, from main at `0295399d`.
Deliverable: one commit on that branch, plus this record.

## Charter

Semantic checking of `tests/programs/wfgrep.wf` cost about 25 s in the gate
profile and the cost grew super-linearly in program size. The charter named a
working hypothesis — that every kill event and every proof view recomputes a
full closure over the whole term table, so the fix is an incremental closure —
and said the hypothesis was to be verified, not assumed. It was verified, and
it is wrong in its premise. The measured cause is one level above the closure.

## Measured attribution

Temporary counters in `close_with_excluded_term`, `intern_for` and
`analyze_candidate_with_mask`, on `wfgrep.wf` before the change:

| quantity | value |
| --- | --- |
| closure calls | 8628 (7176 full, 1416 already-closed fast path, 36 absorbing) |
| term-table size | 37 average, **91 maximum** |
| live bounds entering one closure | 1629 average of 1974 matrix cells — 83 % already closed |
| transitivity triples evaluated | 2 459 053 657 |
| accepted candidates (one interned proof step each) | 18 110 587 |
| `intern_for` calls / misses | 27 766 612 / 13 879 104 |
| fixed-point rounds per closure | 2.87 |
| whole-function analyses | 102 |

The term table is small, so the closure is not blowing up on program size and
there is nothing to make incremental in the shape the charter assumed. What is
super-linear is *how many times the same analysis runs*. Fingerprinting each
closure's complete input — view, bounds and their proofs, disequalities, opaque
goals, term and goal counts, excluded term — gives 1153 distinct inputs behind
7176 full closures. Separating the two hypotheses that fit that number:

- within one function analysis: **520** repeats, 131 705 268 triples (5 %);
- across analyses: **6023** repeats, 2 070 180 103 triples (**84 %**).

Rolling the fingerprints into a per-analysis digest closes it exactly:
**82 of the 102 whole-function analyses are bit-identical repeats of an earlier
one**, carrying 2 047 733 828 of the 2 459 053 657 triples (83 %).

The reason is `Checker::claim_residuality_outcome`. [CLM-2] asks, once per
claim component, whether the program still checks with that component blinded,
and answers by rerunning `analyze_function_inventory_masked` over the *whole*
concrete inventory. The entailment walk reads its claim mask at five points
(`flow.rs` 6779, 7093, 7185, 7256 and `flow/sources.rs` 260). Three consult
only whether a mask is present. The two that read the mask's identity both
require `mask.function == self.function.id`. So the analysis of a function no
mask names is one value shared by every masked rerun, recomputed once per
claim component.

Sampling agreed and added a second cost the counters do not see: about 15 % of
the run was `hashbrown::RawTable<(ProofView, DerivationNode), DerivationId>`
being cloned and dropped — the derivation ledger's interning index, copied by
`phase_a.to_vec()` for every rerun and by `counterfactual_witness`, and then
overwritten unread.

## Mechanism

Five changes, in descending order of measured value. None changes which
candidates the closure accepts, in what order, or with which proof. The
per-step figures below are wall clock on `wfgrep.wf`, taken immediately after
each step; the rigorous interleaved comparison is in **Result**.

1. **`CounterfactualReuse` (`semantic/check.rs`).** One entry per function
   holds its untargeted masked analysis beside the exact published-postcondition
   context it was computed under. `analyze_function_inventory_reusing` consults
   it only when the mask does not name that function *and* the context still
   compares equal by value; otherwise it analyzes and stores. The cache is
   scoped to one `claim_residuality_outcome` (and one generic-schema claim
   loop), across which every other analysis input is fixed. The entry is
   captured before the publication step, so a reused entailment enters the
   rerun in the same state a fresh analysis would.
   25.0 s → 15.4 s.
2. **`InternIndex` (`semantic/entailment/state.rs`).** The ledger's interning
   index is derived from its own nodes, so cloning a ledger now clones an empty
   index and marks it stale; the first `intern_for` on the copy rebuilds it by
   replaying the nodes in identity order. `finish_with_event_roots` still clears
   it deliberately after remapping identities, and leaves it fresh, so the two
   ways the entries can be absent stay distinguishable. Equality ignores the
   index, which is correct: it is a function of the nodes.
   15.4 s → 8.2 s, and peak footprint 5.9 GB → 4.5 GB.
3. **Semi-naive fixed point.** A transitivity triple whose two premise cells
   both still hold the values they held when the traversal last offered it
   builds an identical candidate node against a conclusion cell that has only
   improved, so `insert_closed_candidate` rejects it without touching the
   ledger. Each cell records the round it last changed; a triple is skipped
   when neither premise changed since the previous round, and a middle whose
   incident cells are all stale is skipped whole (skipping it changes nothing,
   so the test stays valid through the block). Round 1 reports every cell fresh,
   so the first pass is still the complete one. Measured 8.06 → 6.74 s CPU.
4. **Maps built once.** The dense matrix is the only index the fixed point
   reads; `bounds` and `bound_proofs` were written on every accepted candidate
   and never read. They are now rebuilt once from the settled matrix, which
   drops eighteen million hashed pair inserts. Every consumer of those maps
   either sorts the keys or is order-insensitive — verified at every iteration
   site — so their content and every derived order are unchanged.
   Bound and proof also merged into one `ClosedCell` array, which removed the
   "bound/proof index diverged" panic path.
5. **Dense middles and a private hasher.** `closure_middle_terms` returns dense
   membership instead of a `HashSet<TermId>` (`TermId` is the dense
   function-local identity), and the interning index gets a deterministic
   multiply-rotate hasher. The index is private, never iterated, and rebuilt
   from nodes when absent, so its hash function cannot reach any output.

## Alternatives rejected

- **Incremental closure across the flow**, the charter's first candidate. A
  state would have to carry its closure between `close` calls for a delta to
  exist. Ordinary `close` deliberately returns a `ClosedState` and writes
  nothing back; making it write back would change what `kill` deletes, what
  `join` intersects, and what `live_l0_relations` reports. That is a semantic
  change, and the measurement does not ask for it: within-run repeats are 5 %.
- **Memoizing whole analyses inside `flow.rs`**, keyed on the arguments. Sound,
  but each stored `FunctionEntailment` carries a ledger of up to 1.5 M nodes
  and the run already peaked at 5.9 GB. Reusing the inventory clone that
  `claim_residuality_outcome` already makes costs no extra live memory.
- **Skipping re-analysis by reasoning about the mask alone**, without comparing
  the published context. A mask can change the masked function's own
  postconditions, which feed every later schedule component. Comparing the
  context by value needs no such induction and fails closed.
- **Per-triple freshness without the per-middle pre-check, and the pre-check
  without the per-triple test.** Measured interleaved against each other:
  8.06 s (neither), 7.05 s (middle only), 6.74 s (both). Both kept.
- **A wall-clock regression test.** The machine was under load from another
  worktree for most of this batch, which is exactly why a timing assertion
  would be worthless. The guards below assert mechanism instead.

## Result

Gate profile, `whitefootc --par -o /dev/null`, five runs interleaved between
the two binaries, on a machine carrying load average 8.8 from another
worktree's test suite. Both medians move together, so the ratio is the reading
that survives the noise; minima are given because interference can only add
time.

| | before (median / min) | after (median / min) | ratio |
| --- | --- | --- | --- |
| `wfgrep.wf` wall | 34.21 s / 30.45 s | 9.37 s / 7.68 s | 3.65× / 3.96× |
| `wfgrep.wf` CPU | 29.07 s / 24.22 s | 7.43 s / 6.50 s | **3.91×** / 3.73× |
| `dir_walk.wf` wall | 1.25 s / 0.96 s | 0.97 s / 0.91 s | 1.29× / 1.05× |
| `dir_walk.wf` CPU | 1.07 s / 0.86 s | 0.86 s / 0.78 s | 1.24× / 1.10× |

On an idle machine at the start of the batch the same baseline measured 25.0 s
wall for `wfgrep.wf` and 0.88 s for `dir_walk.wf`; the charter's 12 s figure
for `dir_walk.wf` was not reproducible under `--par -o /dev/null`.

**About 3.9×, short of the 5× target.** The remaining cost is one honest
whole-inventory analysis plus one single-function analysis per claim component,
which is what the counterfactual actually asks for. Inside that, sampling puts
36 % of the remaining time in the [ENT-4] transitivity fixed point itself
(`close_with_excluded_term`, an O(n³) pass over a matrix that is dense because
every term has type-range edges through Z), about 20 % in interning and hashing
candidate proof steps, and about 10 % in cloning the inventory twice per
rerun — `phase_a.to_vec()` and `counterfactual_witness`'s
`checked.function.clone()`, both of which still copy every ledger's node
vector. Those three are the next places to look.

## Oracle

`whitefootc --par --emit-llvm --par-ledger --stack-ledger` over **623 sources**
— every `tests/programs/*.wf`, `tests/codegen/**/*.wf` and
`tests/conformance/cases/*.wf` — comparing stdout, stderr, exit status and the
emitted LLVM IR against a compiler built from `main` at `0295399d`:
**0 differences.** The linked Mach-O executable is not usable as an oracle: it
differs between two runs of the *same* binary, so the comparison is against the
IR text, which is byte-stable across runs.

`cargo test --profile gate --all-targets`: 1324 library tests (1321 + 3 new),
54 maintained programs, all green. Canonical `make check` green end to end
(`== WHITEFOOT ALL TESTS GREEN ==`, conformance `Pass=502 Skip=1`).

One observation worth recording: the first `make check` of this batch failed
`backend::tests::parallel::an_absent_worker_setting_starts_the_pool_and_an_explicit_opt_out_does_not`
on `defaulted > 0`. That assertion reads the runtime's own lane-grant counter
from a freshly emitted binary, so it depends on the OS scheduling a pool worker
while the rest of the suite saturates the machine, and the machine was carrying
load average 13.6 from another worktree. It did not reproduce: three isolated
runs and a full re-run of the gate all passed, and the same suite had passed
before it. Overlap emission is unchanged — `tests/programs/par_layout.wf` is
one of the 623 sources whose `--par --emit-llvm` output is byte-identical — and
this batch touches only semantic checking. It was not reproduced against the
baseline compiler, so this is an observation, not a verified pre-existing flake.

## Regression guards

Three, each asserting a mechanism rather than a number.

- `interning_into_a_cloned_ledger_keeps_the_original_identity` — interning a
  node already present into a *cloned* ledger returns the original identity and
  grows nothing, and still separates proof views. This is the invariant that
  makes the cheap clone sound; without the rebuild a clone silently gives one
  proof step two identities.
- `a_strengthened_bound_still_propagates_in_a_later_closure_round` — a
  disequality strengthens `a - b <= 0` to `-1` only after the first
  transitivity pass, so the strengthened edge must still reach `c`. Verified to
  fail (`assertion failed: closed.derives_bound(a, c, -1)`) when the freshness
  rule is changed to look back zero rounds instead of one.
- `a_changed_published_context_is_not_reused` — the reuse entry is refused for
  a different published FN-9 context, a different context length, and a
  different function index.

## Judgment calls

- The charter's mechanism was replaced by the measured one. The closure is not
  where the growth is; the [CLM-2] rerun is. The in-engine closure work was
  still done (items 3–5) because it is real and provably identical, but it is
  worth about 20 %, not 5×.
- `CounterfactualReuse` lives in `check.rs`, at the engine's front door rather
  than inside the closure. That is where the redundancy is.
- `PartialEq for InternIndex` returns true unconditionally. The index is
  derived from the nodes, so two ledgers with equal nodes are equal whatever
  their indices hold; the alternative — comparing a cache — would make a
  ledger unequal to its own clone.
- `InternHashBuilder` is a hand-written hasher rather than a dependency. It is
  used by exactly one private map that is never iterated.
- The record was written straight into `docs/done/`; `docs/ongoing/` does not
  exist in this repository and creating a directory to hold one file that
  immediately moves would leave a new empty entry behind.
- Temporary instrumentation (closure counters, input fingerprints, per-analysis
  digests) was removed before committing. The numbers it produced are the
  attribution table above.
- `mcts_mem/` was not consulted or written: no recorded design choice was
  re-decided.

## Approval classes for the merge

- No specification bytes change.
- No conformance content changes.
- No new root entries.
