# Fixed-resource execution

This investigation asks how Whitefoot can establish that a no-heap program
finishes within a declared resource budget. Bounded recursion is an admissible
candidate: qualification must prove peak stack bytes fit a capacity supplied
in advance; a recursion-depth bound is only intermediate evidence. The initial
probes use main `f3cf41d4`, specification v0.62, and the compiler built for the
[cleanup investigation](../access-effects/cleanup-continuations/README.md).
The [implementation contract](DESIGN.md) proposes source rank clauses and
an internal proof consumer; the latest completed study covers [stack bytes](STACK.md).
The whole topic is deferred. The checkpoint below preserves where to resume;
whole-program work estimation additionally needs a concrete cost-budget consumer.
The active specification and compiler are unchanged by this investigation;
the former amendments are retained as [research drafts](DESIGN.md#retained-design-drafts),
outside the live tree and the active amendment process.

## Deferred work and resumption

The single fixed-resource topic in [TODO](../../../docs/todo.md) is the entry
point for future work. This investigation retains the unfinished design and
its reproducible evidence; no implementation work is currently selected.
The goal remains normal completion without heap acquisition within supplied
storage capacities. Stack capacity is in bytes, not a chosen activation cap.

| Area | Evidence retained | Still missing |
|---|---|---|
| Stack frames and peak paths | [STACK.md](STACK.md): static-report undercounts, entry alignment, generated helpers, and manually inspected 4 KiB invocations | Automatic complete machine inventory, instruction accounting and actual linked-image qualification |
| Recursive completion and depth | [Bounded recursion](bounded-recursion.wf), rank controls and [DESIGN.md](DESIGN.md) | Checked entry ranks, complete recursive-edge coverage and a valid mapping to optimized machine paths |
| Loop completion | [Rank-obligation probes](rank-obligations.wf) and the proposed header snapshots | Every continuing backedge and nested/concrete callee covered; a counted outer loop alone is insufficient |
| Other storage and heap exclusion | Source `program no_heap;` trials and the region/closure analysis below | Complete buffers, statics and runtime-state accounting, plus allocation evidence for every linked helper |
| Entry and environment | A synchronous fixed-stack adapter and two inspected native entry paths | General native progress/closure evidence; blocking operations, concurrency, callbacks and interrupts need additional contracts |

The stack probes use compiler main `7127bcb6`; the earlier source and cleanup
probes use `f3cf41d4`. Their recorded outputs are dated evidence, not fresh
measurements of a later main or another toolchain. Neither the local INV-1
controls nor the successful fixed-stack runs implement a termination or
resource checker. The active specification and the atomic update contract
have not been amended by these proposals.

When this topic is explicitly resumed:

1. Read this checkpoint, [STACK.md](STACK.md) and [DESIGN.md](DESIGN.md), then
   the affected current specification and design owners. Recheck the relevant
   probes against the then-current compiler, target and native dependencies
   before importing their observations into an implementation.
2. Start with the proposed arm64 Darwin inventory for one synchronous entry
   and a complete acyclic machine call graph. Account for below-SP accesses,
   every admitted entry alignment, calls/tail edges and linked helpers;
   unknown evidence stays a gap. Use the [target admission cases](DESIGN.md#implementation-admission-and-evidence),
   including oversized frames, small fitting frames, exact byte thresholds,
   missing callees, changed images and changed frames under the same depth proof.
3. Add the source rank/coverage consumer and recursive byte composition, then
   complete the no-heap/native/region boundary. Preserve reset-before-decrement,
   missing-backedge and nonterminating-callee controls. Stack fit, termination
   and full deployment qualification remain separate claims.

Three former amendments are retained as research drafts:
the [proof-boundary replacement](DESIGN.md#proof-boundary)
for `language/checks-and-proofs`, the [progress proposal](DESIGN.md#progress-and-storage)
for `language/checks-and-proofs/resource-bounds`, and the
[compiler consumer proposal](DESIGN.md#compiler-resource-consumer)
for `compiler/resource-bounds`. The owner withdrew them from active amendment
consideration so this research can be preserved on main. Their technical
content is not adopted or rejected. Reassess their grounds on resumption;
any selected tree revision needs a fresh amendment and owner ruling.

Related work stays with this topic. [Recursive cleanup](../access-effects/cleanup-continuations/README.md)
still has no general constant-stack lowering; its continuation-layout study
reopens for a concrete heap-using ownership depth that misses its stack budget
or a measured cleanup-cost obstacle. Validate release order, all continuation
states, representation cost and native regressions, or establish a sufficient
depth-dependent bound for that consumer. Mutual tail transfers reopen only
for a real mutually recursive program: FN-10 currently covers direct self
transfers, and a different callee needs a matching ABI and target evidence.
Neither extension is a prerequisite for the first no-heap calculation.

Whole-program work estimation remains deferred until a concrete execution-cost
budget needs it. Tighter storage-path estimates reopen when a conservative
bound prevents a real kernel from fitting. Wider ranks, symbolic/amortized
costs, hardware deadlines and asynchronous/parallel/service contracts each
need a concrete consumer and their own validation before extending the domain.

## Question and initial scope

For every input satisfying an entry contract, establish normal completion or
an ordinary declared result, no dynamic heap acquisition, and an upper bound
on all required storage under an explicit target/runtime
model. Exhaustion followed by an abort does not establish this property.
The first research baseline is a sequential, bounded-input computation;
interrupts, parallel execution, blocking I/O and hardware deadlines require
additional environment contracts before the claim can cover them.

The existing declaration is `program no_heap;` [GRAM-2, STOR-8]. It excludes
Box and runtime-capacity shapes at the source boundary. Source no-heap,
termination, maximum simultaneous stack use, total work, and a real-time
deadline are different properties and must not stand in for one another.

## Criteria recorded before the new probes

1. Prefix maintained finite computation programs with the existing no-heap
   declaration in scratch; preserve their algorithms and oracles. Compile
   and execute ordinary sequential lowering and inspect the resulting module
   and post-codegen stack ledger. This tests the existing source/storage
   route, not whole-runtime resource closure.
2. Compile a bounded, genuinely non-tail recursive computation with an
   explicit input cap and decreasing argument. Compare all admitted depths
   in a small input fixture with an iterative value oracle. If optimized
   assembly retains recursive calls, derive a conservative module-only
   stack bound conditional on the source depth argument and measured frame
   costs. If it eliminates the calls, record that outcome rather than
   forcing recursion with an artificial target edit.
3. Change only the recursive actual to stop decreasing, retaining the input
   cap and ordinary memory-safety obligations. Compile but never execute
   this negative control. Its acceptance would demonstrate that an input
   bound and successful existing source checks are not a termination proof.
4. Account for the code outside the emitted module: startup, linked bodies,
   compiler helpers, exit, signal stacks and all execution contexts. Missing
   call targets, stack costs, progress arguments or allocations stay unknown;
   neither a successful run nor a generous reservation fills them in.
5. Compare a restricted counted-loop route with checked recursion/loop ranks
   and a general resource-analysis route. State which obligations can use
   the current deterministic proof engine and which need a new judgment.
   No SMT solver, timeout-selected acceptance, runtime fuel counter, or
   assumed writer-supplied bound is a candidate proof authority.

This directory owns the current fixed-resource question; the older
[containers-and-resources design](../containers-and-resources/DESIGN.md)
describes a different language and allowed
nonterminating services while excluding depth certificates. Its obsolete
surface and restrictions do not govern this study. The cleanup study keeps
its reproduction and layout results, but general constant-stack destruction
is no longer the prerequisite for this no-heap objective.

## The property to establish

Given an entry contract, an exact linked image, a target/ABI and an environment
contract, resource evidence needs to establish all of the following:

| Obligation | Required conclusion | Insufficient substitute |
|---|---|---|
| Ordinary safety | All Whitefoot value, ownership and memory obligations hold | A resource estimate |
| Progress | Every admitted invocation reaches its declared completion | Finite stack, tail calls or an exhaustion abort |
| Storage | Peak live storage fits each supplied memory region | Sum of source local-variable sizes |
| Allocation closure | No heap or unbudgeted acquisition in the covered execution | No `box_new` in one module |
| Correspondence | Bounds describe the delivered code, helpers and runtime | A report for another build or target |
| Environment | Supplied memory, execution progress and external responses meet a stated contract | Assuming an OS always provides them |

Deterministic behavior means the language-determined result for the same
admitted inputs and modeled external responses. Resource bounds can be
conservative maxima; executions need not take identical numbers of steps.
A wall-clock deadline additionally needs a hardware and scheduling model.
Termination under continued execution does not prove that a preempted task
will run again or an external read will return. A perpetual service could
later have bounded response per request, but does not meet this study's
whole-invocation termination goal.

The strict memory objective covers startup through completion. A deployment
may supply fixed memory before the invocation; its capacity, alignment and
handoff point must be named, rather than hiding arbitrary allocation in
initialization. Bare-metal startup can use statically reserved regions. A
hosted component can claim only the boundary whose caller-supplied resources
and adapter it accounts for.

## What exists and what is missing

The [active specification](../../../spec/kernel-spec.md) remains authoritative.

| Current owner | Available basis | Missing for this objective |
|---|---|---|
| GRAM-2, STOR-8 | `program no_heap;` excludes heap-owned source forms and allocating prelude rows | Evidence for linked bodies, startup and helpers |
| FN-1 | Counted loops capture endpoints and advance a fixed binder | Completion of bodies/callees; bounds for ordinary `loop` |
| FN-8, FN-9, ENT-4, ENT-6, INV-1, PRF-1 | Checked contracts, affine relations and finite local certificates | Rank/depth judgments and resource-summary composition |
| FN-10 | Marked self transfers retain no caller activation | Termination, argument/release cost and complete stack bound |
| STOR-6 | Target layout and complete generated frames must be representable | Proof that deployment supplies enough storage |
| `backend/stack_ledger.rs` | Frame costs and call edges from the same optimized host compilation | Runtime/external coverage, proved cycle counts and final-image certification |
| SCOPE-3 | Explicit compiler/runtime/OS trust and availability boundary | Deployment evidence that establishes sufficient resources |

A nonempty release graph currently has a Box leaf [PROV-6, STOR-3].
Consequently the no-heap source subset has no value-depth-dependent automatic
Box destruction. Fixed-capacity arrays of non-Box contents add no such release.
Index-linked trees in fixed arrays can still have recursive *algorithms*:
their depth and progress need proofs. A future bounded-heap profile could
include automatic-release depth in its stack budget, but that is a different
profile from the strict no-heap goal.

The ordinary POSIX floor uses `pthread_create`, a 1 GiB entry-stack reservation
and a 64 KiB alternate-stack mapping per attaching thread. It retains the
original thread and can fall back to it if entry-thread setup fails
([source](../../../compiler/src/backend/wf_floor.c)). The
[stack ledger](../../../compiler/src/backend/stack_ledger.rs) explicitly excludes
runtime translation units. These are concrete coverage gaps, not defects in
the current source no-heap judgment. Allocation and progress of linked native
bodies are not described by their `pure` or `reads`/`writes` rows [EFF-1].

## Candidate proof decomposition

The following decomposition motivates the [concrete source proposal](DESIGN.md).
Neither adds a current acceptance judgment.

### Source progress and work

A counted loop with captured endpoints L and U enters its body at most
`max(0, U-L)` times, provided each entered body completes. Nested loops and
calls need their own completion and cost evidence; a finite outer counter
does not make a diverging inner call terminate.

For an ordinary loop, a first candidate is an integer rank R with a proved
nonnegative lower bound, a finite entry cap B, and `R_next < R_head` on every
continuing backedge. Strict integer decrease gives at most B backedges and
B+1 head visits under this convention. Exits by break, return or propagation
owe no decrease, but their calls and cleanup still need coverage. Inner loops cannot
silently reset an outer rank. These are mathematical value images; machine
wraparound is not a decreasing proof.

For direct recursion, prove `0 <= R <= B` on entry and a strictly smaller
nonnegative rank at every recursive actual. At most B+1 activations of that
recursive chain can then coexist. Nonrecursive work in each activation, its
loops and outgoing callees must also terminate. A bounded parameter alone
does not establish this. Mutual-call components would later need a compatible
component argument; self recursion is sufficient for the first experiment,
and no mutual-tail lowering is required.

The existing proof context can discharge local affine comparisons once a
consumer supplies precise head/entry/actual identities and obligations. It
does not currently check that a proposed rank covers every cycle. Coverage
and the induction rule are new language/proof design work. Rank evidence
would be erased, with no runtime countdown or trap. Agent-supplied evidence
must be checked, just as written PRF-1 steps are.

Depth and work differ. Two calls at rank r-1 give depth r+1, but a full binary
call tree has `2^(r+1)-1` nodes. Sequential work adds; alternatives take a
maximum; a loop's work multiplies a body bound by its entry bound. An initial
experiment can compose closed numeric bounds under bounded entry contracts.
Symbolic products, logarithmic depth and amortized costs need separately
justified certificate rules, not silent widening of ENT-6 or budgeted search.

### Target storage and correspondence

For sequential execution, stack is the maximum weighted *simultaneously
active* call path. With F_f the maximum local frame cost and H_fg only ABI
cost not already included in those frames, the acyclic case is:

```text
S_f = F_f + max(0, max over reachable calls f -> g of (H_fg + S_g))
```

A direct self-recursive component with at most D simultaneous activations
(D >= 1), local frame upper bound F and remaining recursive-edge ABI cost
H_ff contributes at most `D * F + (D - 1) * H_ff`. Add the complete caller
path and worst outgoing callee path, including their boundary edges only
where not already charged. If an architecture-corrected per-activation cost
already includes that edge cost, H_ff is zero and the component product is
`D * F`. This conservative account neither adds sibling calls nor counts a
return address twice. General components need path-weighted bounds; a level
need not visit every SCC member once.

Frames include spills, alignment, outgoing arguments, temporaries and target
conventions such as red zones. Dynamic frames require a verified maximum or
remain uncertified. Tail transfers improve a bound only when lowering/target
evidence establishes them. A source depth proof must remain a valid
machine-path bound after inlining, cloning and other transformations.
Establishing that correspondence is a main experiment, not something the
current ledger already proves.

The memory account includes code/read-only storage, mutable statics, every
live stack, supplied buffers and runtime state, respecting each region's
alignment and capacity. Overlaid storage uses a maximum only with proved
lifetime exclusion. A sequential loop reuses frame storage: iterations affect
work, not the sum of every iteration's stack allocation. Multiple contexts
and interrupts need budgets for every simultaneously live stack and bounded
nesting.

Evidence is tied to the source/checked instances, toolchain and flags,
target/ABI, linked bodies, runtime and image. Hashes identify inputs; they do
not prove bounds. Unknown calls or unsupported stack adjustments stay gaps,
with no default byte allowance interpreted as proof.

### Runtime and completion boundary

A first complete target should be one sequential entry on a supplied fixed
stack, with no helper threads, lazy mappings, heap calls or unbounded waits
in its reachable closure. This is a candidate qualification requirement, not
a change to the ordinary runtime now. Startup, exit conversion and generated
library calls still need evidence. An external declaration needs progress,
storage and allocation evidence for its actual linked definition; a writer's
signature is not permission to assume those properties.

The ordinary source checker remains the one semantic path. Resource evidence
would refine a checked program's deployment guarantee without changing its
result or relaxing its safety checks. The proposed rank form and source
interfaces are specified in DESIGN.md; complete target qualification still
needs its correspondence and coverage evidence. Existing accepted source
without the future certificate remains ordinary source; this study invents
no current rejection rule or compiler flag.

## Native probes, 2026-09-22

The implementation-readiness probes in this dated study distinguished three
unresolved choices. Their criteria were recorded before those probes ran:

1. Submit the proposed local descent inequalities through ordinary INV-1,
   using explicit immutable copies only as an experiment stand-in for erased
   rank snapshots. Check direct descent and a loop backedge, then change the
   recursive actual and the backedge separately. A reset-before-decrement
   example must distinguish comparison with the function-entry value from
   comparison with the value immediately before the call. These tests assess
   reuse of the existing proof engine, not implemented termination checking.
2. Compose the same generated object with a small sequential adapter and a
   caller-supplied fixed stack. Inspect every reachable machine call, static
   frame and linked section. If ordinary optimization preserves enough call
   identity, keep it; if it does not, test a narrowly constrained optimization
   boundary. A successful run is supporting evidence only. Dynamic loader and
   host startup remain outside this component experiment's handoff boundary.
3. Check closed numeric work/stack composition against independent small
   examples, including binary recursion, zero work, and enormous numeric
   depths. Evaluation must depend on the representation size of the bounds,
   not unroll the admitted number of runtime iterations. Any clipped result
   must mean that this upper bound cannot certify the requested budget, never
   that the program necessarily uses that many resources.

The compiler sources are unchanged from `f3cf41d4`. The host is arm64 macOS
26.6.2 with Apple clang 21.0.0, ordinary `-O2`, and `--no-overlap`.
Each positive probe was compiled to an ordinary native executable and exited
0 with its value checks intact. Compilation and execution were separate
guarded commands; these are correctness observations, not timing benchmarks.

| Probe | Source change / check | Post-codegen module observation |
|---|---|---|
| [IPv4 checksum](../../../tests/programs/ipv4_checksum.wf) | Add only `program no_heap;` in scratch; retain both checksum checks | No recursive cycle; largest reported acyclic root chain 80 B |
| [SHA-256](../../../tests/programs/sha256_abc.wf) | Add only the declaration in scratch; retain the existing expected-result check | No recursive cycle; largest reported acyclic root chain 320 B |
| [Bounded non-tail recursion](bounded-recursion.wf) | Compare every count 0..32 for one 32-byte fixture with an iterative oracle | `wf_fold_hash` retains its self-call; 32 B/frame, 48 B `wf_main`, 64 B `wf__main_body` |
| Nondecreasing recursive control | Change only recursive actual `remaining: next` to `remaining: remaining` | Accepted by the current compiler; compile only, never executed |

The emitted modules contain no calls to malloc, calloc, realloc or free.
This is only an emitted-module observation: all ordinary executables still
link the standard runtime. A missing allocation call in that module is not
proof of no allocations in the execution closure.

For the recursive probe, ordinary source checking proves the subtraction's
domain, indexing and the recursive callee's input requirements. A separate
mathematical argument is that remaining starts at most 32, decreases by one
at every recursive call, and returns immediately at zero. It gives at most
33 simultaneous source activations. For the test entry's 33 invocations, the
helper is entered `1 + 2 + ... + 33 = 561` times and the iterative oracle
executes `0 + 1 + ... + 32 = 528` iterations. These are deductions from the
probe, not results of an implemented termination/resource checker.

The retained optimized assembly contains:

```asm
cbz x2, base
stp x20, x19, [sp, #-32]!
stp x29, x30, [sp, #16]
sub x19, x2, #1
mov x20, x0
mov x2, x19
bl _wf_fold_hash
```

The later rotate/add and loads of saved registers prevent a tail transfer.
The final linked executable's disassembly has the same 32-byte stack
adjustment and recursive call. Combining the source depth argument with the
maximum frame gives a conservative `33 * 32 = 1,056 B` bound for this helper
chain alone. The zero case actually skips its frame; the conservative product
does not rely on that refinement. Caller frames, runtime, other stacks and
exit paths are excluded, so this is **not** a certified program-stack budget.
The ledger's reservation divided by 32 remains no substitute for a source
depth argument.

The nondecreasing control obeys the same no-heap declaration, input cap and
ordinary safety obligations. With a positive remaining count it never reaches
the base case, unless the external resource boundary stops it first. Its
successful compilation distinguishes existing partial correctness from the
requested totality guarantee without changing either language expectation.

### Reproduce the probes

Use the guarded compiler build if its sources changed. Scratch receives all
generated sources, binaries and reports; no gate reads these research inputs.
The allocator-call search should print no matches (rg status 1).

```sh
make -C compiler build
resource_study=research/investigations/fixed-resource-execution
resource_scratch=$(mktemp -d /tmp/whitefoot-resource-study.XXXXXX)
for sample in ipv4_checksum sha256_abc; do
  awk 'BEGIN { print "program no_heap;\n" } { print }' \
    "tests/programs/$sample.wf" > "$resource_scratch/$sample.wf"
  perl .github/run-check.pl resource-ledger \
    compiler/target/gate/whitefootc --no-overlap --emit-llvm --stack-ledger \
    -o "$resource_scratch/$sample.ll" "$resource_scratch/$sample.wf"
  perl .github/run-check.pl resource-native-build \
    compiler/target/gate/whitefootc --no-overlap \
    -o "$resource_scratch/$sample" "$resource_scratch/$sample.wf"
  perl .github/run-check.pl resource-native-run "$resource_scratch/$sample"
done
perl .github/run-check.pl bounded-resource-ledger \
  compiler/target/gate/whitefootc --no-overlap --emit-llvm --stack-ledger \
  -o "$resource_scratch/bounded.ll" "$resource_study/bounded-recursion.wf"
perl .github/run-check.pl bounded-resource-build \
  compiler/target/gate/whitefootc --no-overlap \
  -o "$resource_scratch/bounded" "$resource_study/bounded-recursion.wf"
perl .github/run-check.pl bounded-resource-run "$resource_scratch/bounded"
rg -n 'call .*@(malloc|calloc|realloc|free)' \
  "$resource_scratch/bounded.ll" "$resource_scratch/ipv4_checksum.ll" \
  "$resource_scratch/sha256_abc.ll"
sed 's/remaining: next/remaining: remaining/' \
  "$resource_study/bounded-recursion.wf" > "$resource_scratch/nondecreasing.wf"
perl .github/run-check.pl nondecreasing-compile-only \
  compiler/target/gate/whitefootc --no-overlap --emit-llvm \
  -o "$resource_scratch/nondecreasing.ll" "$resource_scratch/nondecreasing.wf"
perl .github/run-check.pl bounded-resource-assembly \
  /usr/bin/clang -x ir "$resource_scratch/bounded.ll" -S \
  -o "$resource_scratch/bounded.s" -fstack-usage -Wno-override-module -O2
cat "$resource_scratch/bounded.su"
otool -tV "$resource_scratch/bounded"
```

The last command is the recorded macOS disassembly check. The experiment does
not establish a cross-target frame figure, a real-time bound, complete runtime
closure, or a general recursion certificate. The compiler itself checked all
source obligations it currently specifies; the new rank interpretation above
is an argument to qualify in a future proof consumer.

### Implementation-readiness results

The [rank-obligation probe](rank-obligations.wf) uses ordinary value copies and
INV-1 inequalities to test the proposed arithmetic obligations on the current
checker. It compiles and checks all counts 0..32 natively. Its ordinary loop
and non-tail recursion both agree with their expected results. These copies
are experiment inputs, not an implementation of erased rank snapshots.

Four compile-only changes distinguish the proof boundaries:

| Change | Current result |
|---|---|
| Set recursive `next` to unchanged `remaining` | INV-1 rejects `local_progress` |
| Leave the loop's `remaining` unchanged | INV-1 rejects `backedge_progress` |
| Reset `remaining` to 32, then decrement | INV-1 rejects `entry_progress` |
| Same reset, remove only `entry_progress` | Accepted, never run |

The last case keeps a proved local decrease but repeats rank 31 forever for
positive inputs. It selects an activation-entry snapshot over a call-local
comparison. The loop case selects a per-iteration snapshot. Existing
`entry(reference)` cannot supply either: MSR-3 admits it only for written
reference-parameter measures in ensures, not as a scalar/body snapshot.

The [bound algebra model](bound-algebra.rs) checks clipped affine-map
exponentiation against an independent level-sum oracle on 3,150 combinations
of depth, branching, local cost and budget. Four full-u64-rank controls cover
linear growth, no children, zero local cost and budget overflow. B+1 uses
u128; B=`u64::MAX` takes 65 exponent-bit steps rather than unrolling a runtime
depth. At depth 33, one-child activation count is 33 and two-child count is
8,589,934,591, while a 32-byte recursive frame still has the same conservative
1,056-byte chain bound. This validates the small algebra model, not a resource
consumer integrated into the compiler.

The [fixed-stack adapter](fixed-stack.S) replaces the module's weak floor
entry for one research link, invokes the unchanged WF entry on a statically
reserved, 16-byte-aligned 4,096-byte region and restores the caller's stack
on return. The two exit conversion functions are extracted verbatim from
`compiler/src/backend/ordinary_values.c`; no scheduler or ordinary exhaustion
floor is linked. The same generated module is compiled to assembly with its
stack report, and that exact assembly is assembled for the final link.

The native program exits 0. Final disassembly confirms the entry-to-helper
path uses 32 bytes in the adapter, 64 in `wf__main_body`, and 32 per recursive
`wf_fold_hash` activation. The two exit helpers use no stack and make no calls.
The source `wf_main` was inlined into the entry body and its separate 48-byte
definition is not on this invocation path. Thus this particular inspected
path has a conservative `32 + 64 + 33*32 = 1,152 B` stack bound, below the
4,096-byte reservation. The base case's skipped frame leaves this deliberately
loose. A requested budget below 1,152 would fail this bound, not prove that
actual execution needs 1,152 bytes.

`llvm-nm --undefined-only` prints no symbols. Every call in the linked
disassembly resolves to the recursive helper, entry body or the two exit
helpers; there are no allocator, stack-probe or lazy-binding calls in this
closure. No extra optimization restriction was needed to inspect this
particular graph. It does not establish general optimizer correspondence.
An additional exploratory leaf-frame check reported 16 bytes and its
assembly adjusted SP by exactly 16; no general red-zone claim follows.

The linked section payloads are 456 bytes of text, 32 of constants, 112 of
compact unwind data, 104 of unwind frames and 4,096 of BSS. Their 4,800-byte
sum is not a deployment memory budget: the Mach-O image maps three 16 KiB
segments (`__TEXT`, `__DATA`, `__LINKEDIT`) and reserves an inaccessible
`__PAGEZERO`; the host loader, libraries, suspended caller stack and asynchronous
activity are outside this component experiment. A complete deployment must
account for its actual supplied regions and environment, not add section
payloads and call that whole-process memory.

The experiment narrowed the implementation proposal: the existing checked body and
ABI can run inside a small explicit resource boundary, and current ProofContext
can discharge the required local comparisons. The missing pieces are the
erased snapshot/coverage consumer and general target qualification. The
[implementation contract](DESIGN.md#implementation-admission-and-evidence)
states the first source milestone and the stronger deployment milestone
separately.

To reproduce the additional probes, use the existing compiler and the variables
from the earlier block, or create a fresh scratch directory first:

```sh
perl .github/run-check.pl rank-build \
  compiler/target/gate/whitefootc --no-overlap \
  -o "$resource_scratch/rank" "$resource_study/rank-obligations.wf"
perl .github/run-check.pl rank-run "$resource_scratch/rank"
rustfmt --check --edition 2024 "$resource_study/bound-algebra.rs"
perl .github/run-check.pl bound-algebra-build \
  rustc --edition 2024 -O "$resource_study/bound-algebra.rs" \
  -o "$resource_scratch/bound-algebra"
perl .github/run-check.pl bound-algebra-run "$resource_scratch/bound-algebra"
ruby - "$resource_study" "$resource_scratch" <<'RUBY'
study, scratch = ARGV
source = File.read(File.join(study, 'rank-obligations.wf'))
reset = source.sub('let next = remaining - 1_u64;',
                   "set remaining = 32_u64;\n  let next = remaining - 1_u64;")
variants = {
  'unchanged-call' => [source.sub('let next = remaining - 1_u64;',
                                'let next = remaining;'), 1],
  'unchanged-loop' => [source.sub('set remaining = remaining - 1_u64;',
                                'set remaining = remaining;'), 1],
  'reset-with-entry' => [reset, 1],
  'reset-local-only' => [reset.sub("  invariant entry_progress: next < start;\n", ''), 0]
}
variants.each do |name, (text, expected)|
  input = File.join(scratch, name + '.wf')
  File.write(input, text)
  system('perl', '.github/run-check.pl', name,
         'compiler/target/gate/whitefootc', '--no-overlap', '--emit-llvm',
         '-o', File.join(scratch, name + '.ll'), input)
  raise "unexpected status for #{name}" unless $?.exitstatus == expected
end
native = File.read('compiler/src/backend/ordinary_values.c')
first = native.index('void wf_exit_status(')
last = native.index('void wf_socket_address_v4(', first)
raise 'missing exit definitions' unless first && last
File.write(File.join(scratch, 'exit-adapter.c'),
           "#include \"ordinary_values.h\"\n#include <string.h>\n\n" + native[first...last])
RUBY
perl .github/run-check.pl fixed-body-assembly \
  clang -x ir "$resource_scratch/bounded.ll" -S \
  -o "$resource_scratch/fixed-body.s" -fstack-usage -Wno-override-module -O2
perl .github/run-check.pl fixed-exit-assembly \
  clang -I compiler/src/backend "$resource_scratch/exit-adapter.c" -S \
  -o "$resource_scratch/exit-adapter.s" -fstack-usage -O2
perl .github/run-check.pl fixed-link \
  clang "$resource_scratch/fixed-body.s" "$resource_scratch/exit-adapter.s" \
  "$resource_study/fixed-stack.S" -Wl,-map,"$resource_scratch/fixed.map" \
  -o "$resource_scratch/fixed"
perl .github/run-check.pl fixed-run "$resource_scratch/fixed"
cat "$resource_scratch/fixed-body.su" "$resource_scratch/exit-adapter.su"
llvm-nm --undefined-only "$resource_scratch/fixed"
llvm-objdump --disassemble --no-show-raw-insn "$resource_scratch/fixed"
llvm-size --format=sysv "$resource_scratch/fixed"
otool -l "$resource_scratch/fixed"
```

The adapter commands target arm64 macOS and require LLVM inspection tools on
PATH. They deliberately do not run any compile-only divergent variant. The
three added probes have an explicit caller here and are replaced or removed
when maintained resource-proof and target regressions supersede them. The
one-shot scratch Makefile used to group native build/run commands was removed
after use; it is not a new repository runner or gate input.

## Alternatives and useful prior work

| Route | Benefit | Cost and research disposition |
|---|---|---|
| Counted loops and acyclic calls only | Small first complete proof boundary | Useful control; unnecessarily excludes bounded recursion as a final rule |
| Checked ranks plus target bounds | Retains recursion where its demand fits | Preferred next research route; needs coverage, composition and code correspondence |
| General inferred/amortized resource types | Tighter bounds for richer programs | Defer until simple ranks fail a real consumer; checking must fit deterministic finite proof rules |
| Transform recursion into iteration | Can improve particular stack budgets | Independent optimization; does not prove termination or close runtime resources |
| Runtime fuel, larger stacks or measured high-water marks | Operational limits or diagnostic evidence | Does not establish the requested static proof |

[SPARK's subprogram variants](https://docs.adacore.com/spark2014-docs/html/lrm/subprograms.html#subprogram-variant-aspects)
and [loop variants](https://docs.adacore.com/spark2014-docs/html/lrm/statements.html#loop-statements)
put progress conditions at recursive calls and backedges. Its assertion policy
and verification machinery differ from Whitefoot's; the useful comparison
does not import runtime assertions or solver-based acceptance.
[GNATstack](https://docs.adacore.com/live/wave/gnatstack/html/gnatstack_ug/The_GNATstack_Tool.html)
separates compiler frame/call information from composition and exposes unknown
external calls, dynamic frames and cycles. Coverage is the useful lesson,
not accepting supplied missing numbers on trust.
[AARA's work on non-monotone resources](https://arxiv.org/abs/2106.13936)
addresses peak resources such as stack and tree-depth bounds; it motivates a
later precision comparison rather than adding an inference engine now.
[AbsInt's WCET analysis](https://www.absint.com/aiT_WCET.pdf) includes binary
control flow, loop bounds and cache/pipeline models. This study accordingly
keeps a finite work bound distinct from a hardware deadline.

## Design suitability on resumption

The closed compilation unit, concrete instantiation, checked contracts,
fixed-capacity storage and erased finite proofs provide suitable foundations.
Resource consumers should use the shared checked representation, followed by
evidence for the target closure. A second source checker or container-specific
proof path would duplicate semantic responsibility. The main uncertainty is
preserving and checking source progress/storage evidence through optimization and
linking.

On resumption, the first target task is the complete stack inventory described
in the [stack study](STACK.md). The later source rank/coverage consumer and storage
composition are specified in [DESIGN.md](DESIGN.md).
An acyclic stack calculation can be implemented independently of rank syntax;
recursive path bounds and progress coverage follow as separate proof inputs.
The former amendments remain research drafts; no tree revision is pending.
The first complete target milestone takes
the supplied byte budget as an input and must distinguish one oversized frame
from many small frames that fit, as well as the exact budget boundary. It also
must distinguish an unaccounted native callee, missing source/machine mapping,
an insufficient budget and a changed image before publishing a complete
qualification. The manually inspected adapter above is evidence for that
boundary, not an implemented general certificate checker.

General constant-stack cleanup is deferred as an optimization for heap-using
programs; reopen it when a concrete ownership depth fails its stack budget or
cleanup cost is the measured obstacle. Hardware timing, interrupts, parallelism
and long-lived services each need a stated contract and consumer before
extending the initial certificate. The single maintained TODO topic points to
the checkpoint and these reopening conditions. No live design decision, source
rule, ordinary runtime or acceptance result changes in this research.

The retained source probe serves the bounded-recursion comparison and belongs
beside these results. Replace or remove it when maintained resource-proof
regressions supersede the question. The study reuses existing program fixtures
and introduces no formal test or research dependency into correctness gates.
