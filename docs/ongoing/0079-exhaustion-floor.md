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

## Outcome

(Filled at closure.)
