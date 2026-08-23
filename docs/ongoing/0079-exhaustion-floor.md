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

## Outcome

(Filled at closure.)
