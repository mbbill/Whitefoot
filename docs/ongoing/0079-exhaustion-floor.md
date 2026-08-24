# Batch 0079 — the resource-exhaustion floor

Branch: `exhaust/floor` off `main` at `c704b9e6` (v0.35 active).
Authority: owner chartering directions, 2026-08-23, verbatim:

> 只要程序有可能跑到segfault,那就是wf的设计失败了。wf设计目标就是绝对的可
> 靠和和性能。现在连trap都是在正确程序里面不可能的事情,但却会因为资源耗尽
> 而segfault,感觉有点不可接受。我们得像个合适的办法来控制资源。但这个问题
> 可能比较复杂,需要好好考虑

and, after the v0.35 merge:

> 合并以后我在想还有那些后续比较大的事情,一个是合理处理资源的问题需要研究。
> …… 差不多的话你就开始资源方面的研究吧,我找另一个agent做concurrency研究

The concurrency lane (plan W2) is explicitly assigned elsewhere by the
owner and is out of this batch's scope.

## Inputs

Six research dossiers (2026-08-23, lead's scratch) and the lead's synthesis;
the synthesis is promoted beside this record's work as the design authority:
`research/investigations/exhaustion/DESIGN.md` (landed with the first
executor commit). Measured ground truth the design rests on: stack
exhaustion is a bare guard-page signal with zero diagnostic bytes in every
build; heap exhaustion is a bare abort() with zero bytes and the OOM edge is
optimizer-erasable; the compiler-generated drop glue for recursive nominals
is itself unboundedly recursive; worker overflow presents as SIGBUS where
main-thread overflow presents as SIGSEGV; an unprobed large frame can skip
a guard page on ELF targets (`probe-stack` is byte-free on Darwin).

## The v1 floor (scope)

F1 `probe-stack` on every generated function; F2 a runtime-owned main
stack of one stated size; F3 sigaltstack + dual-signal handlers on every
thread with a fixed-constant resource record, wild faults passing through
untouched; F4 the four allocation-refusal sites routed through one latched
resource abort; F5 iterative drop glue for recursive nominals; F6 the
stack ledger behind a developer flag with a predicted-vs-measured
regression case; F7 deterministic pool depth (the steal-race liveness
coin-flip dies); F8 the [SCOPE-3] exhaustion clause family prepared as a
merge-time recipe (fail-stop, containment, one latched record, distinct
from [DIAG-3], stated coverage limit); F9 the disclosure batch (stderr
bytes, parallel heap multiplier, the -O2 OOM-edge testing note).

Acceptance falsifier, adopted from the research: when the floor ships,
wfgrep's hand-written 16-level directory-recursion cap must be deletable —
a deep tree then produces a clean resource record, never corruption and
never a bare signal. If the cap cannot be deleted, the batch is not done.

## Approval classes

Spec bytes: none applied on the branch; the [SCOPE-3] clause family is
prepared as a recipe for owner application at merge. No protected
conformance changes planned (a resource death is not a [DIAG-3] trap and
the corpus delta is expected to be zero); any annotation need is flagged
the moment it appears. No new repository root entries
(`research/investigations/exhaustion/` sits in the existing investigations
home).

## Executor log

- F1 — every generated definition carries the target's `probe-stack`
  attribute, applied at the module chokepoint so completeness holds for
  definitions nobody has written yet. Verified on this Darwin host: all 42
  corpus executables (21 units x default and `--par`) byte-identical to the
  pre-change compiler's, and all 176 observable rows (4 worker settings)
  unchanged; on a large-frame program the emitted prologue gains the
  `___chkstk_darwin` page walk that the pre-change binary did not have, so
  the attribute is doing work rather than sitting inert. Test needles that
  pinned a `define ... {` line followed the added group; no assertion
  weakened.
- F2+F3 — `compiler/src/backend/wf_floor.c`, linked into every program.
  `@main` becomes a trampoline handing the program to `wf__floor_run`, which
  installs the handlers and runs the entry — now `@wf__main_body` — on a
  1 GiB thread; each pool lane arms itself at attach. Measured: with the
  environment's limit cut to 1 MiB a 2,000,000-frame recursion completes,
  where the pre-floor binary dies at exit 139 with no bytes. An exhausted
  entry and an exhausted lane both write exactly `{"resource":"stack"}` and
  abort; a wild fault still exits 139 with zero bytes and its core dump. All
  176 corpus observable rows byte-identical to the pre-change compiler's.
  Cost, measured rather than carried over from the research: +0.078 ms per
  process and +65,560 bytes peak footprint, of which the 1 GiB reservation
  contributes nothing (an 8 MiB and a 4 GiB thread measure the same) and the
  alternate signal stacks contribute nothing (16 KiB and 64 KiB measure the
  same) — it is the entry thread itself. The dossier's "0 ms, 0 RSS" holds
  for the reservation only.
  Two findings for the lead, neither in this scope: pool lanes are still
  sized from `RLIMIT_STACK`, so F2 widens the sequential-versus-lane ceiling
  gap that F7 owns; and whether a deep recursion reaches a lane at all is a
  steal-race coin flip, measured at 24/30 at the default and 13/20 to 17/20
  across worker counts 2 through 16 — F7's liveness item, reproduced here
  independently of `bt_skew`.
- F4 — the four allocation-refusal edges (`boxes.rs`, `arena.rs`, and both
  sites in `buffer.rs`, re-derived at HEAD) reach one private
  `@wf_resource_abort`, which writes `{"resource":"heap"}` through the
  existing trap writer under the existing first-trap-wins latch. The writer
  and latch conditions widen from "this module has claims" to "this module
  writes a record", so the two classes share one writer — which is what
  makes "no execution produces both records" a mechanism rather than an
  argument. Measured with the optimizer-defeating shape from e3 (a read at
  an index reachable only through a `u8` range, so `-O2` cannot delete the
  edge): a refused allocation went from exit 134 with zero bytes to exit 134
  with exactly the 20-byte record; a forced-false claim in a module that
  also allocates still writes exactly its `[DIAG-3]` record and no resource
  bytes. All 176 corpus rows still byte-identical.
  Deliberately out: the enum-discriminant abort is a compiler-invariant
  failure, not a resource one, and is untouched.
  Named gap for the lead, deliberately not closed here: each `buffer.rs`
  site carries a *second* abort edge — the target-domain ceiling guard —
  which `spec/kernel-spec.md:719` already calls a "non-continuing
  TCB/resource-failure path" and which still dies with zero bytes. It is a
  different condition from an allocator refusal (a request past the target's
  representable maximum, not memory running out), so naming its class is an
  F8 decision rather than an executor's; routing it to `"heap"` would be
  wrong and inventing a third class unasked would prejudge the clause.
- F1 follow-up — the research's open sub-task is closed: the stack-clash
  reproduction is no longer modelled in C. `research/investigations/`'s e3
  demonstrated it on IR whose only difference from clang's was the stripped
  attribute and left the `.wf`-level repro unbuilt. Built here: a `--par`
  program with a 7168-element local array per activation, at a depth past
  its lane's stack, compiled by the shipped compiler and then rebuilt with
  the attribute ablated from that one recursion. Both binaries carry the
  identical frame arithmetic — `sub sp, sp, #0x4a, lsl #12` then
  `sub sp, sp, #0xbd0`, 305,616 bytes per activation — and differ only by
  the `___chkstk_darwin` page walk before the drop. Probed: exit 134 with
  the record, 10/10. Ablated: **exit 0, 10/10** — the program runs with
  frames past the end of its own lane stack, finds what it touches mapped,
  and returns a normal answer for a computation that never fit.
  Instrumented across eleven frame sizes, the probed build's first fault is
  8 bytes below the stack every time; the ablated build's lands 1,552 to
  16,384 bytes below, or nowhere at all. `a_frame_larger_than_the_guard_
  region_is_still_reported` is the regression.
  Also measured, for whoever takes the batch falsifier: with `walk`'s
  16-level cap raised, wfgrep searches a 450-level tree cleanly and finds
  the file the capped build silently misses (exit 1, zero matches, output
  indistinguishable from a real absence). The cap is deletable on this
  evidence, but the stack was never its binding constraint — the host's path
  limit bounds such a tree to roughly 500 levels of single-character names
  long before depth threatens the ceiling, so deleting it does not by itself
  exercise the floor.
- F5 — the compiler-generated drop glue no longer descends the machine stack.
  A drop is recursive exactly when its type's cleanup can reach that type
  again, which is a cycle in the cleanup graph and is decided by strongly
  connected components over that graph — not by a name, a shape, or a corpus.
  Only the edges *inside* such a component move to a heap worklist; every
  other drop keeps the straight-line expansion whose depth its type bounds,
  which is why 19 of the 22 corpus units emit byte-identical modules and the
  three that change are exactly the programs with recursive `box` types
  (`par_layout`, `prefix_expression`, `recursive_tree`), in both worlds. All
  96 observable rows over eight programs, two worlds, and six worker settings
  are byte-identical to the pre-change compiler's.
  Measured falsifier, on `drop_deep.wf` — a program with no recursion in its
  source at all: at 35,000,000 levels the pre-change build writes
  `{"resource":"stack"}` and aborts (exit 134); the new build completes, exit
  0, 1.5 s, 2.25 GB peak. At 100,000,000 levels it still completes (6.4 GB
  peak). What bounds the traversal now is the memory the structure itself
  occupies, not a stack.
  Reclamation order, checked against [STOR-3] rather than asserted: the
  specification fixes reclamation order in exactly two places — reverse
  declaration order among the bindings a scope exit releases, and ascending
  index order among buffer elements. Both are untouched (the call sites and
  the buffer helper are unchanged). Within one value the release action is
  "compiler-owned semantic data", and [STOR-3] gives every
  memory-reclamation action the empty effect row, so nothing that moves is
  observable. Two things move. A box's own heap free now runs before its
  content rather than after — that is what keeps the pending list the size of
  the traversal's frontier instead of the depth it has reached, and it is an
  empty-row action. And a node's own non-deferred cleanup now runs before its
  deferred children rather than interleaved with them; every action that can
  move that way is an empty-row memory reclamation *except* one reachable
  shape, flagged here rather than papered over: an enum variant that declares
  a system-resource field *after* a recursive `box` field, whose [SYS-5]
  release is the one action with a non-empty row. In that shape the chain's
  `close` calls change from deepest-first to shallowest-first. No corpus type
  has it, and the specification fixes no order there, but it is a real
  difference and the owner packet should carry it.
  Deliberately out: a `buffer` in a cleanup cycle. It would need a second
  traversal shape (the element walk must resume where it left off) and no
  program can reach one — a nominal recursive through a buffer has no
  selected target layout and is refused before emission, measured identically
  on the pre-change compiler. `DropPlan::of` refuses that case as a compiler
  invariant instead, so lifting the layout limitation cannot silently restore
  recursive glue.
  Regressions: `no_compiler_derived_drop_reaches_itself` walks the emitted
  module's own drop-glue call graph and fails on any cycle, so a new nominal
  shape is covered without anyone extending a table (it fails against the
  pre-change compiler, whose `wf.drop.t0` calls `wf.drop.t0`);
  `an_ownership_chain_keeps_its_straight_line_drop` fails an emitter that put
  every program on a worklist; `a_deep_boxed_spine_is_reclaimed_without_a_
  record` runs the traversal end to end.
  One existing compiler test required rewriting rather than deleting:
  `programs::heap::recursively_boxed_tree_executes_with_derived_cleanup`
  pinned the old straight-line shape (first `@wf.drop` definition, two frees
  inside it). It now pins the same behaviour at the new mechanism — the entry
  drives a worklist, the per-node step hands both children to it and calls no
  drop helper, and the traversal releases each block it takes. Not protected
  conformance evidence; recorded here because the change forced it.
- F7 — a lane's stack is the entry's stack, byte for byte, and the steal-race
  liveness coin flip is gone. Both routed findings close on one lever. Lanes
  asked `RLIMIT_STACK` with an 8 MiB floor, which reintroduced exactly the
  environment dependence F2 removed from the entry and left a lane two orders
  of magnitude short of it; `wf_floor.c` now exports its constant and
  `par_runtime.c` asks for that, so "the same number" is a fact about the
  program rather than a comment in two files. `<sys/resource.h>` and
  `wf__par_stack_bytes` are gone.
  The argument, which is what makes this a fix rather than a bigger knob: a
  stolen call is an ordinary Whitefoot call that starts at the *bottom* of the
  stealing lane's own stack rather than continuing the offerer's. With lanes
  sized like the entry, no thread has less room than the entry has, so the
  deepest any schedule reaches is at least what the no-steal schedule reaches
  — stealing became strictly headroom-positive, where before it lost 128x.
  Measured on a 2,000,000-deep recursion, 30 runs at each of `WF_WORKERS`
  0/1/2/4/8/16 and the shipped default. Before: 30/30, 30/30, 11/30, 13/30,
  3/30, 2/30, 4/30 — every failure a `{"resource":"stack"}` record, never a
  bare signal. After: **30/30 at every one of the seven**.
  Residual, looked for rather than assumed away: a schedule that splits a deep
  chain across two lanes has more total headroom than one that does not, so a
  program in the band above the single-thread ceiling could in principle
  complete on some schedules only. Bisected at 1,000,000-level resolution on
  the same shape: 22M completes 10/10 and 23M fails 0/10 at four workers, and
  identically at sixteen; 23M fails 0/30 at both two and sixteen workers. The
  boundary is (1 GiB − ~6 KB)/48 B, the overlapped clone's own frame — a
  division, not a distribution — so no band was found. The mechanism is the
  bounded number of outstanding offers per lane (`WF_PAR_LANE_SLOTS` = 64):
  the top of a recursion is stealable and the rest of its depth is one
  thread's by construction. Stated as measured on this shape rather than as a
  theorem; a shape whose deep half is genuinely handed out repeatedly could
  still reach further on a luckier schedule, and that direction can only
  *raise* liveness, never lower it.
  Cost: none measurable. Sixteen lanes now reserve 16 GiB of address space
  between them; `par_layout` at `WF_WORKERS=16` measures 1.95 MB peak against
  1.80 MB before at identical wall time, and at four workers the two are
  within 50 bytes — the reservation is untouched pages, as F2 measured for the
  entry. All 96 observable rows still byte-identical to the pre-batch
  compiler's.
  Regression: `a_deep_recursion_completes_at_every_worker_count` runs the
  probe three times at each of the seven settings; against the pre-change
  runtime the eight-worker cell alone fails it with probability ~0.999.
  Two test consequences, both flagged rather than absorbed.
  `an_exhausted_lane_writes_the_same_resource_record` lost its discriminator:
  it identified a lane death *by depth*, which only worked while lanes and the
  entry had different ceilings. It now runs past the common ceiling and holds
  every run to the same standard (exactly the record, never a bare signal),
  and its doc comment says plainly what it can no longer prove. And
  `the_shipped_default_keeps_a_deep_recursion` in `backend/tests/parallel.rs`
  became vacuous at F2, not here: it survives a 60,000-deep recursion under an
  8 MB `ulimit -s` that no longer bounds any thread, so it would pass through
  a fourfold frame regression. Its comment now says so and names the
  instrument that replaces it — the F6 predicted-versus-measured ceiling.
- F6 — `--stack-ledger`, a developer channel that says what a level costs and
  how many of them the program's stack holds. Post-codegen by necessity, not
  by preference: the compiler's own pre-lowering frame accumulator reports
  **zero bytes** for `wf_spine`, the function that ends the program, because
  its real 16-to-48 bytes are the ABI frame record and the register
  allocator's spills — neither exists before LLVM runs. So the numbers come
  from `-fstack-usage` and the call graph from the assembly of the same
  compilation, Tarjan over that graph, and one line per frame, per cycle, and
  per acyclic chain.
  Sample, unedited (`--stack-ledger`, cold and outlined rows trimmed for the
  record only):

  ```
  STACK stack     1073741824 B  the entry thread and every worker lane
  STACK frame     wf_walk                                    1744 B  static
  STACK cycle     wf_walk                                    1744 B/level  615677 levels
  STACK chain     main            4240 B  main -> wf__floor_run -> wf__main_body -> wf_search_file -> ...
  ```

  That first cycle row is the batch falsifier's other half, in one line:
  wfgrep's hand-written `depth >= 16` cap, whose own doc string admits the
  truncation is indistinguishable from a complete search, is **38,479x**
  conservative. The bound was not careful, it was blind.
  Both `--par` worlds, on the `min_stack` shape, which is the report the 0076
  default-depth regression would have shipped against:

  ```
  STACK cycle     wf_spine              48 B/level  22369621 levels  overlapped clone
  STACK cycle     wf__par_seq_spine     16 B/level  67108864 levels  sequential clone
  ```

  and on `par_layout --par`, where the same side-by-side prices the overlap
  tax per recursion (`wf_build` 80 B/level against its clone's 48, `wf_layout`
  and `wf_layout_banded` 80 and 96 against 80) — structural, not the 3x the
  0076 record carried.
  Drop glue is in the ledger like anything else, and that is now a *negative*
  result worth reading: `par_layout --par` shows `STACK frame wf.drop.step.0
  64 B` and **no** `wf.drop` cycle row at all. Before F5 that row was a cycle.
  The regression `the_compilers_own_drop_glue_has_rows_and_no_cycle` fails the
  day one comes back.
  The model, re-derived here rather than carried over: the research measured
  `(stack − 6144)/frame`, where the 6144 was argv, environ, and auxv at the
  top of the process stack. The entry does not run on the process stack any
  more, so that term is gone and the ceiling is exactly `stack/frame`.
  Measured first-failing depth against the report at four frame widths — 16 B,
  10,272 B, 34,848 B, 291,760 B — deviations of at most 1,136, 2, 0, and 0
  levels, the largest being 0.0017% of its own ceiling. The regression asserts
  the program completes at 0.999x the reported ceiling and dies at 1.001x, at
  two frame widths three orders of magnitude apart; it fails the day the
  report and the machine stop agreeing, in either direction, and it names both
  numbers when it does.
  Two deviations from the brief, both deliberate. The ledger runs its own
  clang rather than adding `-fstack-usage` to the link that already happens:
  `-fstack-usage` writes its report beside the file it compiled, so the flag
  on the ordinary link would drop a `.su` into whatever directory the writer
  asked for output in, and the call graph needs assembly, which the link does
  not produce. One compilation into a directory the driver owns and removes
  gives both artifacts, guaranteed consistent, and none of it runs unless the
  flag is passed. And the three row classes from the research are two here:
  `bounded` would have to come from the compiler annotating the recursions it
  created (the loop splitter's ten-frame theorem), which is a channel that
  does not exist yet; inventing one for a report would have cost more than the
  report is worth, so a splitter's row reads as an ordinary cycle. Flagged as
  the one place this ledger is wrong in spirit.
  Stated limits, in the module's own doc comment: it is a build fact, not a
  source fact (per target, per optimization level, per host compiler); it
  covers the emitted module's machine functions, so the floor and parallel
  runtime translation units linked beside it are outside it; an inlined
  callee's bytes are inside its caller's row; and the one indirect edge in a
  Whitefoot program — the pool's thunk pointer — is not in the assembly, which
  is why a `wf__par_thunk_` row heads a chain of its own.
  Cost: none on the ordinary path. No corpus module changed, and all 96
  observable rows remain byte-identical to the pre-batch compiler's. The
  regression itself is 2.3 s; the first draft was 28 s because the host
  compiler spends nine seconds vectorizing a 7,168-element array fill, so the
  wide-frame probe is 256 elements for the same arithmetic under test.
  `FLOOR_STACK_BYTES` restates the runtime's constant on the Rust side and
  `the_ledger_and_the_runtime_name_the_same_stack` pins the two spellings
  together, because a ledger deriving depths from a stack no thread has would
  be worse than none: every number would be wrong by the same factor and
  nothing in the output would say so.
- Batch falsifier — wfgrep's 16-level directory-recursion cap is gone, with its
  `depth` parameter, its `too_deep` guard, and the doc clauses that recorded the
  truncation as a defect. On a 450-level tree the uncapped build finds the file
  at the bottom and the capped build reports exit 1 with empty output: a real
  absence and a truncated search are the same answer, which is what the doc
  string admitted. At 300 levels the capped build is worse still — it finds the
  one shallow match, exits **0**, writes nothing to standard error, and looks
  like a complete successful search that happened to miss half its input.
  `a_tree_far_deeper_than_the_deleted_cap_is_searched_completely` is the
  regression; against the capped program it sees one hit where it requires two.
  All twelve existing wfgrep cases pass unchanged.
  **What the falsifier does not exercise, measured rather than assumed.** The
  stack was never the cap's binding constraint and it is not the new one. `walk`
  refuses a display path past a thousand bytes (`ile(child_length, 1000_u64)`)
  and a chain of single-character directories spends two bytes a level, so this
  shape stops at **493 levels** — bisected here: 493 completes and finds the
  file, 494 returns exit 2. The stack ledger prices one `wf_walk` activation at
  1,744 bytes and the entry stack at **615,677** of them, so the program's own
  buffer binds 1,249x below the machine. Deleting the cap raised the reachable
  depth 30.8x and left three orders of magnitude between the program and the
  floor.
  The honest verdict is therefore narrower than the design's phrasing in one
  place and stronger in another. The cap **was** deletable, and deleting it
  converts a truncation indistinguishable from completeness into either a
  complete search or an error status. What the floor bought is not this
  program's ceiling but the right to have no hand-written one: before it, a
  program with no depth cap had no defined behaviour at its ceiling, and now
  `walk`'s ceiling is a number `--stack-ledger` prints and any death there is a
  record. A pathological-tree stack death is unreachable through wfgrep on this
  host, so the record half of the falsifier is carried by the floor's own
  probes rather than by the search program — re-derived at this tip below.
  One thing the deletion exposed and did not create: that thousand-byte path
  refusal sets the failure bit and breaks without writing anything to standard
  error, so a too-deep tree exits 2 with an empty stderr. It predates this batch
  (the 16-level cap kept every tree far away from it), it is a wfgrep defect
  rather than a floor one, and it is flagged here rather than fixed.
- Branch-tip gate, reported rather than worked around: `make check` fails at
  `research-tests`' `effect` target, and it fails identically with this
  executor's changes stashed. The cause is outside the repository — the
  installed `wasi-sdk` clang crashes in WebAssembly instruction selection on
  `adversarial-caller.ll` — so it is a host-toolchain failure, not a
  regression from this batch. Every other `make check` target is green:
  compiler gate, conformance adapter 500 pass / 1 skip, coverage 137/137,
  spec append-only, archive integrity, digest sync, approval history.

## Outcome

(Filled at closure.)
