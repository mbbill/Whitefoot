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
  abort; a fault the floor does not own keeps its own signal, status, and
  core dump. All 176 corpus observable rows byte-identical to the pre-change
  compiler's.
  The discrimination band (2026-08-23, superseding this entry's first form
  and `research/investigations/exhaustion/DESIGN.md:38-39`). As first landed
  both said unconditionally that a wild fault keeps exit 139 and its core.
  Measured false by the batch audit: the handler accepted any fault in
  `[stack_low - 1 MiB, stack_high)`, so a corruption fault anywhere in a
  megabyte below any thread's stack — a write *or* a read, which can only be
  wild — was converted to exit 134 and `{"resource":"stack"}`, positively
  misattributing corruption to exhaustion. That is the misdirection
  `wf_floor.c`'s own header says a diagnostic must not commit, and the
  regression sampled one far address (`0xdeadb000`) and asserted only that
  the process did not return, so nothing could see it.
  The band is now the probe's geometry: one page-walk stride plus the ABI red
  zone (16,512 bytes on this host), read once outside signal context. Every
  generated definition carries `probe-stack`, so a descent touches its pages
  on the way down and the first touch below the stack is at most one stride
  under it; below that only a leaf's red zone is reachable. Nothing wider is
  slack — every extra byte is a range of wild faults reported as exhaustion,
  and the old band was roughly 64x the stride and about 128,000x the
  eight-byte distance a legitimate overflow actually lands at.
  A second defect in the same path, found by the audit's refuter and
  reachable from an ordinary accepted program with no wild pointer and no
  FFI: the non-guard path restored `SIG_DFL` and returned, which is
  per-signal and process-wide where the classification above it is
  per-thread. An externally delivered SIGBUS — a supervisor, a job
  controller, a harness — arrives there with a null `si_addr`, was swallowed,
  and left the floor disarmed for every thread, so the next overflow was a
  bare host signal with zero bytes. Measured at `d3eee546`: a program that
  takes one external SIGBUS and then descends 400,000,000 frames printed
  `SURVIVED COMPLETED` and exited 0 at depth 1,000, and exited 138 with zero
  bytes at depth. The handler now re-raises after restoring, so the process
  cannot outlive the restore; the same program exits 138 in both rows.
  Regressions:
  `only_a_fault_within_the_probe_stride_is_read_as_an_exhausted_stack` faults
  at a controlled distance below its own thread's stack and pins both sides
  of the boundary — half a page and one page below give the record and
  SIGABRT, four pages, 64 KiB and 16 MiB below give SIGSEGV and zero bytes;
  `an_externally_delivered_signal_does_not_disarm_the_floor` runs the matrix
  above; and `a_fault_that_is_not_exhaustion_keeps_its_own_disposition` now
  asserts SIGSEGV rather than only "did not return", which is the difference
  between a core dump of the corruption and a core dump of `abort`. All three
  fail against `d3eee546`'s floor.
  Also fixed here, from the audit's handler review: the non-Darwin bounds
  capture leaked its `pthread_attr_t` when `pthread_getattr_np` succeeded and
  `pthread_attr_getstack` failed. Untested on this host.
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
  writes a record", so the two classes share one writer.
  One latch, not two (2026-08-23, superseding this entry's first form and the
  F8 justification bullet). As first landed, the sentence "which is what
  makes 'no execution produces both records' a mechanism rather than an
  argument" was false, and the audit named the reason: sharing the *writer*
  serialized the module's three record classes against each other and against
  nothing else. The floor's signal handler wrote the stack record through a
  latch of its own in `wf_floor.c`, so two threads dying of different
  resources at once each won a latch and each wrote. Neither the finder nor
  its refuter could demonstrate an interleaving — the window between a
  winner's `write` and its `abort` is microseconds, and 200 tuned runs
  produced 103 heap, 97 stack, and 0 double records — but the recipe states
  it unconditionally and the owner is asked to ratify those bytes, so the
  repair is the mechanism rather than the sentence: there is now one latch in
  the process. `wf_floor.c` owns it and exports its address; the emitted
  writer asks for it, and a module linked without that translation unit falls
  back to one of its own through the same `weak` definition the floor entry
  already uses. `the_floor_and_the_module_share_one_record_latch` claims the
  latch and then runs out of stack: shared, the floor writes nothing; separate,
  it writes the stack record and aborts. `a_module_that_writes_a_resource_
  record_and_hands_a_call_out_is_latched` covers the widened condition itself,
  which the audit found untested — narrowing it back to claims would put every
  `--par` heap-only module on the unlatched writer with nothing failing.
  Cost: five `--par` corpus modules (`dir_walk`, `generic_instances`,
  `par_layout`, `recursive_tree`, `wfgrep`) are no longer byte-identical to
  `d3eee546` — they are the five that emit a latch, and the diff is exactly
  the accessor and the two uses. All 160 observable rows (20 units, two
  worlds, four worker settings) remain byte-identical.
  Measured with the optimizer-defeating shape from e3 (a read at
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
  The buffer arm (2026-08-23, superseding this entry's first form). As first
  landed, F5 declared a `buffer` in a cleanup cycle deliberately out of
  scope on the ground that "no program can reach one — a nominal recursive
  through a buffer has no selected target layout and is refused before
  emission", and had `DropPlan::of` refuse the case as a compiler invariant.
  That ground was false and the batch audit falsified it in one program:
  `box` supplies the indirection the target layout needs while the buffer
  stays inside the cycle, so
  `enum Chain { Nil(); Cons(kids: box<buffer<Option<Chain>>>); }` closes the
  cycle `Chain -> box<buffer<Option<Chain>>> -> buffer<Option<Chain>> ->
  Option<Chain> -> Chain`. That program compiles and runs on the pre-change
  compiler and was refused here with a bare `Backend/Backend: InvalidIr` —
  a silent acceptance regression, and a compiler-invariant failure shown to
  the writer with no rule id, no source coordinate, and no statement of what
  was unsupported.
  The traversal now has the second arm. A `box` names one content, so one
  entry carries the whole edge and the block is released as the entry is
  taken. A buffer names many elements whose order [STOR-3] fixes — each
  element's drop in ascending index order *followed by* that same one heap
  free — so its step pushes one entry for the block and then one per element
  from the last index down, and the last-in first-out worklist takes them
  back in exactly the order the rule fixes: element 0 first, the free last.
  Nothing resumes; the step returns after recording where the elements are.
  This is the one place the preceding paragraph's "both untouched" needs
  qualifying: ascending index order among buffer elements is no longer only
  the straight-line helper's loop, it is also produced by the push order
  here, and the free stays after every element rather than moving ahead of
  them the way a box's does.
  Measured at this tip: the audit's program compiles and runs (exit 0, empty
  stderr, default and `--par`); a 5,000,000-level chain of one-element
  buffers completes at exit 0 in 276 MB where the pre-batch compiler's
  cyclic glue (`wf.drop.t0 -> wf.drop.buffer.t4 -> wf.drop.t4 ->
  wf.drop.t0`) exits 139 with zero bytes. All 40 emitted corpus modules (20
  standalone units x two worlds) are byte-identical to `d3eee546`, because a
  program with no recursive buffer registers no new entry kind.
  Regressions: `no_compiler_derived_drop_reaches_itself` walks the emitted
  module's own drop-glue call graph and fails on any cycle, so a new nominal
  shape is covered without anyone extending a table (it fails against the
  pre-change compiler, whose `wf.drop.t0` calls `wf.drop.t0`), and it now
  runs over both indirections rather than only the boxed spine;
  `an_ownership_chain_keeps_its_straight_line_drop` fails an emitter that put
  every program on a worklist; `a_deep_boxed_spine_is_reclaimed_without_a_
  record` runs the traversal end to end;
  `a_cleanup_cycle_through_a_buffer_is_accepted_and_runs` is the audit's
  program, so the acceptance regression cannot recur silently;
  `a_deep_cleanup_cycle_through_a_buffer_is_reclaimed_without_a_record` runs
  the buffer arm at 1,000,000 levels;
  `a_buffer_in_a_cleanup_cycle_is_walked_in_the_order_the_rule_fixes` pins
  the push order where the order is chosen, since [STOR-3] gives memory
  reclamation the empty effect row and nothing downstream can see it; and
  `a_buffer_block_outlives_the_elements_the_traversal_takes_from_it` runs the
  wide shape under a scribbling host allocator, which is the half of that
  order a running program can be made to notice — pushing the block last
  turns every element load into a scribbled tag and the enum's invalid-tag
  abort fires (verified by ablation: both cases fail, the second with a
  signal instead of exit 0).
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
- F8's routed third-class decision, landed rather than only specified — the
  target-domain ceiling guard gets its own class, `{"resource":"target-domain"}`,
  and both `buffer.rs` edges reach `@wf_target_domain_abort` instead of a bare
  `@abort()`. Measured on a `buffer_new` of ten quintillion `u8` — inside
  `buffer_fits`'s language ceiling of `(2^64 - 1)/stride` and outside the
  target's `i64::MAX`, which is the whole reachable band of that guard — the
  same source moved from exit 134 with **zero bytes** to exit 134 with exactly
  the 29-byte record, verified against a rebuild of the pre-change compiler.
  The choice is argued from the specification's own distinction rather than
  from taste. [DIAG-1] lists "resource failure" and "target-layout failure
  [STOR-6], target-qualification failure [QUAL-1]" as separate members of one
  non-rejection sum, and `spec/kernel-spec.md:719` routes the dynamic guard down
  the same non-continuing path without merging it into the resource class. Same
  death, different condition, so: same path, distinct class. The arena goes the
  other way and folds into `heap`, because an arena node is that one supply
  refusing; a target-domain refusal is not, because the byte count has no exact
  target value and a host with more memory refuses it identically. A reader who
  sees `heap` goes and looks at the machine's memory, and for this failure that
  is the wrong place to look.
  Spelling: `target-domain`, not the brief's suggested `target-ceiling`, so the
  class name is the specification's own noun for the condition — `:719` reads "a
  failed dynamic target-domain guard". Recorded as a deviation from the brief.
  Gating is by operation, not by type: the constant and the helper follow a
  `BufferFill` or `BufferVacant` in the program, so a module carrying buffers it
  never allocates emits neither. Regressions:
  `a_request_past_the_target_ceiling_writes_its_own_resource_record` runs it end
  to end, and `every_target_domain_guard_reaches_the_target_domain_abort` holds
  both edges to it for the same completeness reason the refusal edges are held —
  one edge left calling `@abort` still dies with zero bytes, and it is the edge
  nobody was looking at.
  One existing compiler test required rewriting rather than deleting, flagged
  the way F5 flagged its own:
  `backend::tests::buffers::target_domain_failure_aborts_before_allocation_without_a_language_record`
  asserted the edge called `@abort()` and that stderr was empty. Its claim —
  no *language* record — has not moved and is now pinned at the new mechanism:
  the bytes on standard error are exactly the resource record and carry none of
  `rule_id`, `function`, `node_path`, or `message`. Not protected conformance
  evidence; recorded because the change forced it.
- Branch-tip gate, reported rather than worked around: `make check` fails at
  `research-tests`' `effect` target, and it fails identically with this
  executor's changes stashed. The cause is outside the repository — the
  installed `wasi-sdk` clang crashes in WebAssembly instruction selection on
  `adversarial-caller.ll` — so it is a host-toolchain failure, not a
  regression from this batch. Every other `make check` target is green:
  compiler gate, conformance adapter 500 pass / 1 skip, coverage 137/137,
  spec append-only, archive integrity, digest sync, approval history.

## F9 — what changes by default, and what a reader must not assume

Five disclosures. Every one is a default-build consequence, not an opt-in, and
the merge packet is incomplete without them.

**1. Exhaustion paths now write bytes to standard error.** This is the batch's
whole point and it is also its most visible behaviour change. Before, every
resource death — an overflowed stack, a refused allocation, a byte count past
the target's domain — produced zero bytes; a supervising process saw only a
signal or exit 134. Now each writes exactly one record and aborts: 20 bytes for
`{"resource":"heap"}`, 21 for `{"resource":"stack"}`, 29 for
`{"resource":"target-domain"}`. Anything downstream that compared a program's
standard error against empty now compares it against a record. Nothing on a
normal path changed: a program that does not exhaust a resource writes nothing
new, and all 176 corpus observable rows are byte-identical to the pre-batch
compiler's.

The change is sharpest under `--par`, and this continues the disclosure batch
0076 made about the pool default. A program that overflows under the pool and
not sequentially used to differ only in its exit signal. It now also differs in
its stderr bytes. The dependency is not new — the outcome already differed —
but it became visible, which is the improvement and the disclosure at once.

**2. The parallel default multiplies peak live heap by the lane count.** This
is the heap twin of 0076's depth flag and it is not new to this batch either;
what is new is that it is measured at this tip rather than carried over.
`pk_peak.wf` — a depth-4 binary recursion whose leaf holds a transient 128 MiB
buffer — three runs at each setting, byte-stable at every one except the
eight-lane cell, which varied by 16 KB across three runs:

| build | peak footprint | ratio |
|---|---|---|
| sequential | 135,708,672 | 1.00x |
| `--par WF_WORKERS=2` | 269,991,936 | 1.99x |
| `--par WF_WORKERS=4` | 538,542,080 | 3.97x |
| `--par WF_WORKERS=8` | 1,075,609,600 | 7.93x |
| `--par` default (10 lanes on this host) | 1,344,176,128 | **9.91x** |

Peak is exactly `lanes x 134,217,728` plus a base of 1.49 MB that grows about
51 KB per lane. The lane count is `hw.logicalcpu`, a property of the machine
and not of the program, so the same binary on the same input dies on a big-core
laptop and survives on a two-core VM. What the floor changes is that it now
dies with `{"resource":"heap"}` instead of silently — where the host delivers
the refusal at all, which item 5 bounds.

The policy question is the owner's and no executor should settle it: document
only; bound the number of concurrently live handed-out allocations; or let the
default lane count consider memory as well as processors.

**3. At `-O2` the optimizer deletes allocations, so an OOM test can test
nothing.** Every Whitefoot executable links at `-O2` and there is no other
level. LLVM forwards a fill value to the loads and removes a `malloc`/`free`
pair whose contents nothing observes, taking the refusal edge with it — the
research measured four probes asking for 16 TiB and 4 EiB and returning
normally. This is semantically fine and operationally decisive: **no test may
assume the refusal edge exists in an optimized binary.** The shape that defeats
it is the one both refusal cases use — read the buffer at an index the compiler
knows only through a type range, so the load cannot be decided and the
allocation cannot be deleted. A naively written case asks for sixteen terabytes,
exits 0, and passes.

The target-domain guard is not covered by that warning in the same way. Its
comparison is on the byte count rather than on the buffer's contents, so a
constant length lets LLVM fold the branch and keep the abort — which is why the
existing `buffers.rs` case survives `-O2` with an unused buffer. A runtime
length keeps the guard for the ordinary reason. The rule is still worth stating
as one sentence: an exhaustion test proves nothing until it has been seen to
fail against the build without the floor.

**4. The floor's own cost, corrected from the research's estimate.** The
dossiers priced the runtime-owned entry stack at "0 ms, 0 RSS". Measured here,
that holds for the 1 GiB *reservation* and not for the mechanism: the floor
costs **+0.078 ms per process and +65,560 bytes of peak footprint**. An 8 MiB
and a 4 GiB entry thread measure the same, and a 16 KiB and a 64 KiB alternate
signal stack measure the same, so neither the reservation nor the handler stacks
are the cost — it is creating the entry thread at all. The number is small and
it is not zero, and a packet that repeated "0 ms, 0 RSS" would be wrong.

**5. Sixteen lanes reserve sixteen gibibytes of address space, and it stays
address space.** Each lane now gets the entry's stack byte for byte, so a
sixteen-worker run reserves 16 x 1 GiB. Re-derived at this tip on
`par_layout --par`, three runs each: 1,720,320 bytes peak at four workers and
2,179,072 to 2,195,456 at sixteen. Twelve extra lanes carrying twelve extra
gibibytes of reservation cost about 459 KB of resident memory, roughly 38 KB a
lane — thread bookkeeping and the pages a signal stack touches, not the
reservation. If the reservation were resident the sixteen-lane run would show
sixteen gibibytes. The mechanism is confirmed; the absolute numbers sit 0.08 to
0.23 MB above F7's own (1.80 MB and 1.95 MB) because this is a later build on a
machine doing other work, and the difference is reported rather than reconciled.

## F8 — the merge-time application recipe: resource death in [SCOPE-3]

**Nothing below is applied to `spec/kernel-spec.md` on this branch.** The
in-tree file is untouched at ACTIVE v0.35,
`645b22b19bdfcf51683b9b10c7fd9109fc4029e9687df30e09e871daf84eb769`, 3,443
lines, 437,084 bytes, so the landed-archive gate stays green here. Every line
number quoted below is that file's own.

The candidate was produced by applying the exact edits below to a scratch copy
and hashing the result. It is reproducible from this record and nothing else,
and that was checked rather than assumed: each edit block below was extracted
from this file and diffed against the corresponding lines of the hashed
candidate, and all six are identical.

| candidate | SHA-256 | size |
|---|---|---|
| v0.36, ACTIVE header | `ee50a356a392294c8ef5c79e78e658f75269c5948f3d120cf7fd2570a6509cfe` | 441,249 bytes, 3,459 lines, 137 rules |

The digest is of the bytes that land on `main` — header and status already
reading v0.36 ACTIVE, since the activation commit is where they take effect. An
owner who applies the text as CANDIDATE first and flips the header afterwards
produces a different intermediate digest for the same final bytes.

**Grammar verification.** The recipe adds no production, token, or spelling.
The native verifier, which reuses the compiler's own lexer and parser, confirms
it:

```
$ whitefoot-grammar spec/kernel-spec.md <candidate>
grammar-preserving candidate verified by the active compiler: 74 productions,
93 decisions, 105 terminal predicates
```

identical to the installed inventory.

### Why [SCOPE-3] and not a new rule

The condition already belongs to [SCOPE-3]: `:726` and `:959` both point at it
by name. A new `[RES-n]` rule would cost a rule-count move, a derivation-ledger
row, a protected coverage annotation, and a conformance denominator change, for
a statement that is an obligation on the implementation rather than a new
language fact. [SCOPE-4] is the precedent for the shape — it owns the trap
class *and* the trap's reporting obligation ("before aborting, the runtime
attempts to write the exact [DIAG-3] trap record"), in one rule. [SCOPE-3] now
owns the resource-death class and its reporting obligation the same way.
`:726` and `:959` are deliberately left byte-identical: they already say the
right thing and they already cite the rule that now defines it.

### Edit 1 — [SCOPE-3] gains the resource-death definition

Insert the following fifteen lines immediately *after* the file's line 25 (`This
is the Layer-4 envelope statement; violations of (a)/(b) are outside the
language's guarantee.`), with no blank line between:

```
An execution that exhausts a resource the trusted computing base supplies reaches the edge of that envelope without leaving it, and this rule fixes what happens there.
Such an execution is fail-stop: it performs no further operation of the program, produces no external effect after the operation that exhausted the resource, and neither continues, retries, unwinds, nor runs language cleanup.
It is contained: it writes nothing outside the storage the program already owns, so no resource death is a memory-safety event and the freedom from undefined behavior above survives one.
Before the process ends, the implementation writes exactly one resource record to standard error; no execution writes a second one, and no record is partial or interleaved with another.
The record's bytes are fixed by the exhausted resource class alone, and it names no source construct, rule identifier, function, node path, worker, host thread, dynamic call stack, address, depth, or size.
It is not a [DIAG-3] record and never precedes, follows, or replaces one; the absence of that record's fields is what distinguishes the two, and an execution in which no executed `claim` is false produces no [DIAG-3] record whether or not it exhausted a resource.
The classes are exactly three, and an implementation may not report one of them as another.
The first is the stack an execution runs on.
The second is the storage supply an allocation draws on, which is one class for a heap box, a `buffer`, and an arena's backing alike, because a refusal of any of them is that one supply refusing.
The third is the target's own representable byte-count and address-index domain [QUAL-1, STOR-6], which is not that supply: the byte count it refuses has no exact value there, and a host with more memory refuses it identically.
This specification fixes neither a spelling for those class names nor an exit status for the death, because the record's presence and its class are what a reader, a test, and a supervising process tell apart, and a second byte-fixed mandatory runtime report would make this a language output instead of the trusted computing base reporting its own limit [DIAG-3].
Writing the record is a quality obligation on the implementation rather than a language guarantee, and its coverage is exactly the conditions the implementation can observe: an allocation the allocator refuses, an execution that runs past the stack it was given, and a byte count a target-domain guard refuses.
Where the host ends the process without delivering the condition to it — an external kill, or an operating system that grants more memory than it holds and later reclaims it by killing the process — no record is possible and none is required; on such a host the observed refusal is the rarer case rather than the typical one, and this rule promises nothing about the other.
A permitted overlap [PAR-1, PAR-2] may raise the call depth an execution reaches and may not lower it below the depth the source-order execution reaches, so taking the permission cannot turn a completing execution into an exhausted one by depth alone.
It fixes no such relation for allocated storage: overlapping raises peak simultaneous demand by the number of executions in flight, so an overlapped execution may reach a refusal the source-order execution does not.
```

Sentence by sentence, what each is for and what it is measured against:

- *Fail-stop* and *contained* are the two halves of the charter. Containment is
  the safety half and the only sentence here that is not about reporting: it is
  what F1's `probe-stack` restores, and without it an accepted program can walk
  a large frame past the guard region into a neighbouring thread's live stack.
  The `.wf`-level reproduction is in the F1 follow-up entry above — ablate the
  attribute and the program runs with frames past the end of its own lane stack
  and returns a normal answer, 10/10.
- *Exactly one record* is a mechanism, not an aspiration: both writers share the
  emitted module's one first-writer-wins latch, and the floor's runtime carries
  its own for the signal path.
- *The absence of that record's fields* is the load-bearing distinction and the
  reason the record's shape is what it is. Two unrelated constraints force a
  fixed constant independently — a signal handler may only reach
  async-signal-safe facilities, and [PAR-1] requires identical observables under
  every permitted schedule — so this sentence costs nothing the implementation
  was not already paying.
- *The classes are exactly three* carries the routed decision. The arena folds
  into the allocation supply; the target domain does not, argued from the
  specification's own separation of "resource failure" from "target-layout
  failure" in [DIAG-1] `:1643`.
- *Fixes neither a spelling nor an exit status* is what keeps this from becoming
  a second [DIAG-3]. A byte-exact second mandatory report would make the record
  a language output, and it would close the class set against a future target
  with a genuinely different resource. It also declines the research's own
  recommendation of a distinct exit status: today a trap and a resource death
  are both 134, and the record is what tells them apart, so the status buys a
  second discriminator for the case that already has one.
- *Where the host ends the process without delivering the condition* is the
  honest limit, named in the rule rather than in a footnote. On an overcommitting
  host the allocator's refusal is the rare case: the measured death is a SIGKILL
  at 78 GB of footprint with nothing on standard error, and no clause can promise
  a record for a signal that cannot be caught.
- *A permitted overlap may raise the depth and may not lower it* is F7's residual
  turned into an obligation. It is what makes taking the permission safe rather
  than a gamble: before F7 a lane's stack came from `RLIMIT_STACK` and a deep
  recursion at eight workers failed 27 times in 30. After it, every lane gets
  the entry's stack byte for byte and the same probe passes 30/30 at all seven
  settings — re-derived at this tip at 35/35 over seven settings. The final
  sentence refuses to extend the guarantee to storage, because F9's item 2
  measures the opposite there and a rule that covered both would be false.

### Edit 2 — [ERR-4] names the class its enumeration has no slot for

Replace the file's line 1471 with:

```
[ERR-4] Classification: expected environment and input failures are values (`Result`); unproved function, operation-domain, allocation-fit, bounds, and system-range obligations are source rejections; a false executed `claim` traps [SCOPE-4]; and exhaustion of a resource the trusted computing base supplies is none of those three, but the contained fail-stop resource death with its own record that [SCOPE-3] fixes.
```

[ERR-4] is the exhaustive classification of what a failure *is* in this
language, and runtime resource failure had no slot in it. That gap is why a
reader could reasonably have assumed a resource death was either a trap or a
rejection; it is now neither, explicitly.

### Edit 3 — [DIAG-3] says the resource record is not its own and not developer output

Insert one line immediately *after* the file's line 1993 (`An implementation may
provide additional developer output only on a separately selected channel that
cannot alter, prefix, suffix, or replace the mandatory trap record.`):

```
The resource record [SCOPE-3] requires of an exhausted execution is neither this record nor that additional developer output: it is written on this record's own channel, is fixed by its exhausted resource class alone, and no execution produces both records.
```

This sentence is necessary rather than decorative. Line 1993 as it stands
confines *additional developer output* to a separately selected channel, and the
resource record is on standard error — the trap record's own channel. Without
this sentence the two rules contradict each other the moment the floor ships.
[DIAG-3]'s exclusivity sentence at `:1986` is deliberately **not** touched: it
already lists "resource failure" and "target-qualification failure" among the
things that produce no DIAG-3 record, it is already correct, and it is the
sentence that keeps the two classes apart.

### Edit 4 — the META-5 delta declaration

Replace the file's line 6 with:

```
META-5 delta declaration: numbered rules +0/-0 (137 remain); grammar productions +0/-0 (74 remain); unique fixed lowercase grammar atoms net +0; writer operation spellings +0/-0; runtime-trap families +0/-0; entry forms +0/-0; contract block forms +0/-0; system operations +0 and declaration records +0; exception clauses +0/-0. [SCOPE-3] gains the definition of resource death: an execution that exhausts a trusted-computing-base resource is fail-stop and contained, writes exactly one record naming only the exhausted resource class, and is not a [DIAG-3] event; [ERR-4] and [DIAG-3] name that class where their own enumerations had no slot for it. No construct is added, no accepted program changes, no verdict changes, and no required check is removed.
```

Note that "runtime-trap families +0/-0" is exact and load-bearing: the resource
record is not a trap and this change adds no trap family. If a future reader
finds a delta declaration claiming a new trap family for this, the change was
misread.

### Edit 5 — the selection ground

Append one sentence to the end of the file's line 7, separated by a single
space:

```
The resource-death definition [SCOPE-3] states is selected on the same ground by the resource-exhaustion investigation of batch 0079, whose measured death table, containment reproduction, and record evidence are recorded in `research/investigations/exhaustion/` and `docs/done/0079-exhaustion-floor.md`, under the owner's chartering direction of 2026-08-23; it states an implementation obligation and no writer-facing construct, so the accepted language is byte-identical across it.
```

### Edit 6 — the header

Line 1 becomes `# Kernel Specification v0.36` and line 3 becomes
`Status: ACTIVE v0.36`.

### Impact inventory

`[SCOPE-3]`'s extent moves from lines 24-25 and 335 bytes to lines 24-40 and
3,654 bytes. `[ERR-4]` moves from line 1471 to line 1486 and gains 174 bytes.
`[DIAG-3]` gains one line at what becomes line 2009. Line-initial rule
definitions are **137 before and 137 after**, so `RULE_COUNT` does not move.

**Bracketed rule-token occurrence counts move, in both directions.** Under the
single-token convention: `SCOPE-3` 10 to 14, `DIAG-3` 14 to 19, `ERR-4` 4 to 5,
`SCOPE-4` 7 to **6**, `PAR-1` 5 to **4**, `PAR-2` 3 to **2**. The three
decreases are all one edit: the outgoing META-5 delta declaration named
`([PAR-1], [PAR-2]; 137 remain)` and "in the sense [SCOPE-4] fixes", which is
correct for v0.35's delta and wrong for v0.36's, so the replacement drops them.
No rule loses its last reference, no rule becomes unreferenced, and the set of
cited rule ids is identical before and after — checked over the whole file, not
over the changed lines.

**The counting convention, stated because the numbers do not reproduce without
it:** every count above is of the single-token citation `[X]` only. Under the
all-citation convention, which also counts `[A, B]` forms, the same six move
plus `QUAL-1` 12 to 13 and `STOR-6` 10 to 11 — Edit 1 adds one `[QUAL-1,
STOR-6]` — while `PAR-1` and `PAR-2` come out unchanged, because each loses one
single-token citation and gains one inside Edit 1's `[PAR-1, PAR-2]`.

The recipe adds non-ASCII: Edit 1's "Where the host ends the process" line
carries two U+2014 em dashes, taking the file from 98 lines with non-ASCII bytes
to 99. The specification already carries non-ASCII on 98 lines and no gate
forbids it; this is disclosure, not a defect.

### Derived material the activation change must carry

- `spec/kernel-spec-v0.35.md`: the outgoing ACTIVE bytes archived flat,
  digest `645b22b1…`. Append-only and hook-enforced thereafter.
- `compiler/src/spec_identity.rs`: regenerated rather than hand-edited
  (`cargo run --bin whitefoot-spec -- --emit-identity src/spec_identity.rs`),
  taking `SPEC_SHA256_HEX` to `ee50a356…` and `ACTIVATION_CHAIN_LENGTH` from 27
  to 28. `RULE_COUNT` stays 137.
- `compiler/src/spec.rs`: the transcribed digest literal moves with it.
- `governance/APPROVALS.md`: one chain record,
  `ACTIVE-SPEC: v0.36 ee50a356a392294c8ef5c79e78e658f75269c5948f3d120cf7fd2570a6509cfe 645b22b19bdfcf51683b9b10c7fd9109fc4029e9687df30e09e871daf84eb769`.
- `spec/derivation/derivation-ledger.md`: **no new row and no status change.**
  [SCOPE-3] stays `✅ derived`; the added text is derived from the same R4
  premise its existing row already cites — silent corruption is the forbidden
  failure mode — and states no new fact source. A v0.36 delta section records
  rules +0, productions +0, and totals unchanged. While writing that section,
  fix the drift noted under audit dispositions below.
- Conformance corpus: **zero cases.** The change adds no construct, changes no
  accepted program, and changes no verdict, so no case and no expected verdict
  moves and no coverage annotation is needed. This is the reason the batch
  touches no protected conformance evidence at all.

### Deliberately not in this recipe

- **A distinct exit status for a resource death.** The research asked for one
  (a trap and a resource death are both 134 today). Declined and stated as
  declined in Edit 1: the record already discriminates, and a status change
  would be a second mechanism for a question that now has an answer — and
  moving off `abort` would cost the core dump that a wild fault still produces.
- **Byte-exact record bytes in the specification.** Declined for the reason
  Edit 1 gives: it would create a second mandatory language runtime report
  beside [DIAG-3] and close the class set against a target with a genuinely
  different resource.
- **Any change to `:726` or `:959`.** Both already cite [SCOPE-3] for exactly
  this condition and both remain correct.
- **The [SYS-5] release-order question F5 raised.** Deliberately left out of
  this clause and carried to the owner instead; see the open items below. It is
  not an exhaustion matter, and folding an unrelated ordering rule into an
  exhaustion clause would put the mechanism in the wrong home.

### Where the rule is deliberately wider than the implementation

The rule requires one record naming the class; this implementation additionally
makes the record byte-identical for a given class and makes the sequential
schedule deterministic. The rule requires a record for the conditions the
implementation can observe; this implementation observes all three. Neither
gap costs anything — an implementation never has to take the room a rule leaves
it — and both avoid a further [META-5] amendment when the implementation
widens, for instance to a target where a fixed-capacity pool is a fourth class.

## Outcome

Closed 2026-08-23 on `exhaust/floor`, nine items and one falsifier, all
lead-verified or re-derived at the branch tip by the closing executor.

**The charter is answered on its own terms, and the answer has a shape.** A
correct Whitefoot program can still die of exhaustion — nothing here raises a
ceiling — but it can no longer die *silently*. Before this batch the only
abnormal end a correct program could reach was the only one with zero
diagnostic bytes, while a false claim, which a reviewed program cannot reach at
all, got a byte-exact record. That asymmetry is gone. And one thing that was
worse than the owner's complaint is gone with it: an accepted program could
walk a large frame past its guard region into a neighbouring thread's live
stack and return a normal answer for a computation that never fit. That is
silent corruption, not a segfault, and F1's `probe-stack` closes it — measured
on a `.wf`-level reproduction, exit 0 ablated against exit 134 with the record
probed, 10/10 each way.

Landed commits, in order:

| commit | item |
|---|---|
| `f089aa4f` | F1 — `probe-stack` on every generated definition |
| `178d4f69` | F2+F3 — the runtime-owned stack and the defined death |
| `feef8658` | F4 — the four allocation-refusal edges reach one record |
| `23279a52` | F1 follow-up — the `.wf`-level containment reproduction |
| `dc8cf1a3` | F5 — iterative drop glue for recursive nominals |
| `1a9b4c45` | F7 — a lane's stack is the entry's stack |
| `87afd8db` | F6 — `--stack-ledger` |
| `a4b4b4a1` | the batch falsifier — wfgrep's depth cap deleted |
| `5c95580d` | F8's routed third class — `{"resource":"target-domain"}` |
| this commit | F8 recipe, F9 disclosures, closure |

**Verification at the tip.** `make -C compiler check` green before and after
every change in this executor's scope, 1,235 library cases and 56 program
cases. `make check` reaches the same single stop the branch has carried since
it opened: `research-tests`' `effect` target fails because the installed
`wasi-sdk` clang crashes in WebAssembly instruction selection on
`adversarial-caller.ll`, identically with the branch's changes stashed. It is a
host-toolchain defect outside the repository, not a regression, and it is an
open item for the packet rather than a finding. Every other target is green:
repository invariants (`AGENTS.md` and `CLAUDE.md` byte-identical), approval
history, spec append-only, archive integrity over 36 recorded specifications,
digest sync against the chain tail, the conformance harness at 29 cases, and
coverage 137/137 with 0 uncovered. `conformance-run` sits behind
`research-tests` in the target list and so never runs in a failing `make check`;
run alone at this tip it reports **Pass=500 Skip=1**, which is the same figure
the earlier entry recorded and is the check that the corpus delta really is
zero.

**The falsifier's verdict: the cap was deletable, and the honest statement is
narrower than the design's.** wfgrep now searches to the depth of the tree.
The capped build, on the same 300-level tree the new regression uses, finds the
shallow match, exits 0, and writes nothing — a complete-looking successful
search that missed half its input. What deleting the cap does *not* do is
exercise the floor: `walk`'s own thousand-byte display path stops the descent at
493 levels, bisected, while the stack holds 615,677 activations of it, so the
program's own buffer binds 1,249x below the machine. The floor is what makes
having no hand-written cap defensible, not what raises this program's ceiling.
The record half of the falsifier is carried by the floor's own probes, which is
where it was always going to be carried.

**Audit-lite dispositions.** Six landings, load-bearing numbers re-derived
independently at the tip rather than read from the logs:

- F1 — `par_layout`'s emitted module: 22 definitions, 22 carrying `#0`, one
  attribute group naming this host's `__chkstk_darwin`. Completeness confirmed.
- F2 — the 2,000,000-frame recursion completes under `ulimit -s 1024`, exit 0.
  The compiler's number, not the shell's, confirmed.
- F5 — the drop-glue call graph of `recursive_tree`'s emitted module,
  re-extracted with correct function boundaries: `wf.drop.t0` to
  `wf.drop.step.0` and `wf.drop.run`, `wf.drop.run` to `wf.drop.step.0`,
  `wf.drop.step.0` to `wf.drop.push`. Acyclic. (A first extraction that
  misattributed `wf_main`'s three calls to the preceding drop definition
  appeared to show a cycle; the finding did not survive being done correctly,
  and is recorded because a reviewer running the same careless one-liner will
  see the same false positive.)
- F6 — the ceiling model re-derived by hand: 1,073,741,824 / 1,744 = 615,677,
  exactly what `--stack-ledger` prints for `wf_walk`. The `(stack - 6144)`
  term is correctly gone, because the entry no longer runs on the process
  stack.
- F7 — the 2,000,000-deep recursion, five runs at each of `WF_WORKERS`
  0/1/2/4/8/16 and the shipped default: **35/35**. The coin flip is gone.
- F2/F3 headline — an unbounded recursion at this tip: exit 134, stderr exactly
  `{"resource":"stack"}`. F4's twin and F8's third class confirmed by hand the
  same way, the latter against a rebuild of the pre-change compiler to establish
  that it wrote zero bytes before.

One finding, inherited rather than introduced. `spec/derivation/derivation-ledger.md`'s
v0.35 section still reads "84 derived · 52 existence-only · 0 underived across
136 rules" and still names a candidate binding at
`73d647c8945ad3d51eea3ed030714b433d6171e0d36b0869dd91366238cbd8f5`, which is no
current file. The ledger carries 137 rule rows including [PAR-2]'s, and the
0078 record predicted the corrected line verbatim — "84 derived · 53
existence-only · 0 underived across 137 rules" — so the v0.35 activation added
the row and left the totals sentence and the candidate paragraph behind. It is
on `main` at `c704b9e6`, not introduced here. Not repaired in this batch because
the ledger is spec-adjacent evidence and the v0.36 activation touches that same
section anyway; the recipe's derived-material list carries the fix.

**Open for the merge packet.**

1. **The [SCOPE-3] recipe application.** One owner transcription, six edits,
   candidate `ee50a356a392294c8ef5c79e78e658f75269c5948f3d120cf7fd2570a6509cfe`.
   Approval covers exactly those bytes.
2. **The [SYS-5] release-order question, an owner decision this batch declines
   to make.** F5's iterative drop glue changes one reachable shape: an enum
   variant declaring a system-resource field *after* a recursive `box` field
   has its chain of `close` calls run shallowest-first where they used to run
   deepest-first. No corpus type has that shape and the specification fixes no
   order within one value, so today's rule permits both. But [SYS-5]'s `close`
   is the one release action with a non-empty effect row — it is externally
   observable — and an unfixed order over an observable action means two
   conforming implementations can publish different external-effect orders for
   the same program, which is the kind of thing [EFF-5] otherwise nails down.
   Fixing it deepest-first is not free: it would require the worklist to defer a
   node's own cleanup behind its children, which is exactly the trade F5 made to
   keep the pending list the size of the traversal's frontier instead of the
   depth it has reached. Three options, none taken here: leave it unfixed and
   say so in [STOR-3]; fix it deepest-first and pay the worklist cost; or fix it
   shallowest-first and ratify what the implementation now does.
3. **The third record class.** `{"resource":"target-domain"}` landed on the
   branch with its two regressions and its argument, and Edit 1 states the
   three-class partition normatively. If the owner prefers the guard folded into
   `heap`, both the code and the clause move together and the record above says
   why that would be the wrong reading.
4. **The `wasi-sdk` research-test host defect**, carried from `main`: the
   installed toolchain crashes on `adversarial-caller.ll` and `make check`
   cannot be fully green on this host until it is replaced. Reproduced with the
   branch's changes stashed.
5. **The parallel heap multiplier's policy question** (F9 item 2): document
   only, bound concurrently live handed-out allocations, or let the default
   lane count consider memory as well as processors.
6. **wfgrep's thousand-byte path refusal**, exposed by the falsifier and
   predating it: a too-deep tree exits 2 with an empty standard error. A wfgrep
   defect, unowned by this batch.

**Deferred with named re-entry conditions**, unchanged from the design synthesis
except where this batch moved one: the static acyclic stack bound (a batch of
its own, and the doctrinally right end-state); proof-derived stack segmentation
(re-evaluate reach now that the claim-in-closure exclusion is deleted); prologue
depth checks (only on a target without guard pages, and only gated on a
per-function ledger check); the typed allocation-failure surface (reopen on a
target where allocation failure is truthful — strict overcommit, a fixed-capacity
pool, WASM linear memory with a declared maximum, or no virtual memory at all);
routing `buffer_fits` to bounded-arena facts, which is the only mechanism on the
list that *prevents* a heap death rather than reporting it; and depth-bound
proofs past the acyclic tier, which the research measured as a mirage — the loop
splitter's own ten-frame theorem is free to print as a bound and the general
question waits for a program it blocks.

F6 reports that free bound as an ordinary cycle row today, because the channel
by which the compiler would tell the ledger "I created this recursion and I know
its depth" does not exist. Inventing one for a report would have cost more than
the report is worth. It is the one place the ledger is wrong in spirit, and its
own module says so.
