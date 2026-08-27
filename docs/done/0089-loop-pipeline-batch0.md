# Batch 0089 — loop pipeline, batch 0: the three prerequisites

Branch: `batch/0089-loop-pipeline-batch0`, from `main` at `79b29665`.
Deliverables: three correctness fixes in `compiler/`, the design record at
`research/investigations/io-model/LOOP-PIPELINE.md` with the probe results
appended, this record.

## Charter

`research/investigations/io-model/LOOP-PIPELINE.md` §7 defines five batches.
This is batch 0, and it is the one the design says must not be cut: "every
later batch is silently wrong without it". Three things, none of which changes
a published byte or a permission verdict on any program in the bench corpus:

1. **The match-scrutinee gap** (§3.10). A may-suspend call written as a `match`
   scrutinee is the same call as one written as a `let` right-hand side, and
   the [PAR-1] judgment could not see it at all.
2. **Per-operation-record path storage** (§3.6 item 1), on every backend that
   stages a path, plus [SYS-2]'s `loan-released(path)` published while the open
   is still outstanding.
3. **Slot-indexed completion storage** (§3.6 item 2), so every per-site
   completion storage element belongs to one outstanding *operation* rather
   than to one written call.

The design's exit condition for the batch is that the probe numbers are
recorded and `C.wide8` is unchanged within spread. Both are below.

## Fix 1 — a call is a call wherever it is written

### The defect

`IrBuilder::lower_statements` populated its `call_results` map from the
`CheckedStatement::Let` arm alone, and `Program::candidate_of` in
`semantic/permission.rs` built a [PAR-1] candidate from `Let` and
`PropagateLet` alone. A `match` scrutinee is a call in call position with the
same [EFF-2] projection, the same loans and the same footprint, and neither
the judgment nor the lowering could reach it.

The consequence is not a wrong program; it is a program judged on its
spelling. Two sources that perform the same two operations in the same order
got different work:

```text
let first  = write_once<'out, 'bulk>(...);      let first = write_once<'out, 'bulk>(...);
let second = write_once<'err, 'marker>(...);    match write_once<'err, 'marker>(...) {
match second { Ok(...) => {} Err(...) => {} }     Ok(...) => {} Err(...) => {} }
```

Compiled by the base compiler at `79b29665`, the left form emits one
`wf__completion_file_write_submit` and one `wf__completion_file_join`; the
right form emits **zero** of each. With one candidate there is no window, so
no pair is judged, no chain is formed, and the permission ledger reports
nothing at all about a program that plainly performs two independent
operations.

### The fix

A candidate is now built from every written call position, and its identity is
the **call occurrence** rather than the binding a statement happens to define:
`PermissionSite::binding` becomes `Option<BindingId>`,
`PermissionCompletionStep::wait_for` names calls by `NodePath`, and
`IrBuilder::call_results` is keyed by `NodePath`. That is not bookkeeping — a
scrutinee call defines no binding at all, so a binding cannot be the identity
of a site.

One recorded fact distinguishes the two positions and the window rule does the
rest. `Candidate::result_read_by_own_statement` says that the statement's own
remainder — a dispatch and the arm it selects — reads the call's result, so
`Program::judge` starts the interposed window at the scrutinee's *own*
statement rather than after it. That statement is then classified exactly as
any other statement of its form is, and a `match` statement is a form this
judgment does not project, so a scrutinee call denies as a window's first
member without any rule of its own. It can only ever be a run's last member,
which is exactly the shape the IR requires: every member of a group must be
defined in one block, and a scrutinee's dispatch terminates its block.

### Tests

- `semantic::tests::permission::a_call_in_scrutinee_position_is_judged_as_the_bound_form_is`
  — two independent outputs, the second written as a scrutinee: the pair is
  eligible, the second site defines no binding, and the schedule has the bound
  call running while the scrutinee call is outstanding.
- `semantic::tests::permission::a_scrutinee_call_denies_against_a_later_call_it_is_read_before`
  — the same two calls with the scrutinee first: denied with
  `Denial::InterposedForm { side: Between(0), form: "a match statement" }`, and
  no schedule is formed.
- `backend::tests::completion::a_call_in_scrutinee_position_is_handed_out_exactly_as_a_bound_call_is`
  — the IR-shape assertion (both spellings emit the same submit/join/direct
  counts) plus a run of each emitted program under `WF_IO_HELPERS` 0, 1 and 4
  with both published streams byte-checked.
- `backend::tests::completion::a_scrutinee_call_before_an_independent_call_stays_sequential`
  — the scrutinee-first program emits no submission and no join, and still
  publishes the same bytes in all three helper configurations.

### What `many_files_narrow.wf` does after the fix

**It emits zero submissions, exactly as before, and its permission ledger is
empty, exactly as before.**

"Submission" here means one `call ... @wf__completion_file_*_submit` site in
the emitted module — a hand-out the *lowering* performs. Counted that way, at
`79b29665` and on this branch alike:

```text
program                 open_at_submit   pread_submit   join sites   --par-ledger lines
many_files_narrow.wf           0              0             0                0
many_files_wide.wf             3              3             6               11
many_files_wide8.wf            7              7            14               23
```

That count is not the same thing as a host operation. The probes measured that
`many_files_narrow.wf` performs **8,192 io_uring operations** on the Linux
container today — every one of its 8,192 `read_at` calls goes through the
completion bridge, is submitted to the ring, and is joined immediately by the
same thread. Those are *direct* calls: the bridge routes a direct read through
the ring on Linux whether or not the lowering handed anything out. So the
narrow program submits nothing and still performs 8,192 ring round trips, and
the two numbers must not be confused. The design's falsifier F2, which says
`wf__completion_file_submissions()` is 0 today for the narrow program, is wrong
for that reason and is corrected in §9.6 of the design record.

Why the fix does not change the narrow program: a window is formed from
candidates that are **consecutive statements of one block**, and narrow's two
system calls are not in one block. `match open_file(...)` is the only statement
of its `region 'n` block, and `match read_at(...)` sits inside that match's
`Ok` arm — a different statement list one level deeper. One candidate per block
is no window, so no pair, so no schedule. §3.10 says the spelling is "part of
why" narrow submits nothing; this batch removes that part, and the remaining
part is the loop and the nesting, which is what batches 1 to 3 are for.

The wide programs are unaffected because they already wrote their calls as
`let` bindings.

## Fix 2 — a submitted open owns its path bytes

### The defect

`bridge.c` stored the caller's pointer into the request
(`request.operation.open_at.path = path`), the bounded POSIX adapter copied
that pointer into its queue entry, and the Linux ring copied it into the SQE
(`submission->addr = (uint64_t)(uintptr_t)entry->request.buffer.path`) where
the kernel keeps it until the name is resolved. A submitted open outlives the
call that formed it: the caller regains its name buffer the moment submission
returns and may rewrite it while the host is still resolving. Both adapters
were resolving storage they did not own.

Nothing in a shipped program hits it today, because one call site holds at most
one hand-out and the emitter's `%component` buffer is not written again before
the join. That is exactly why it had to be fixed now: the barrier the pipeline
removes is what has been hiding it, and the failure mode is a silently wrong
file, not a crash.

### The fix

`wf_file_work` and `wf_linux_io_uring_entry` each gained
`char path_storage[WF_FILE_PATH_CAPACITY]`, and each stages the name into it at
submission and repoints the request at the copy. `WF_FILE_PATH_CAPACITY` is
1024: the widest component an admitted open can name is Darwin's 1023 bytes
(`qualification.rs` fixes the Darwin-family limit at 1023 and the Linux family
at 255), and 1024 holds that with its terminator. A work record is copied
whenever it moves, so `wf_file_work_bind_path` rebinds the pointer at every
copy and the invariant is that a record's open never names bytes outside
itself.

A name that does not fit is refused **before** an operation is claimed —
`wf_file_path_fits` in `wf__completion_file_open_at_submit`,
`wf_bridge_submit_linux_open_at`, `wf_file_adapter_submit` and
`wf_linux_request_valid` — so the caller falls back to its direct open, which
resolves its own buffer inside its own call and needs no copy. That is a
throughput fallback of the same class as a full queue, never a changed
outcome, and no generated open can reach it.

Two backends stage a path, and they are all of them. The Windows IOCP adapter
carries transfers only: `windows_completion.{c,h}` contains no open request
shape, and `bridge.c` has no `_WIN32` path at all, so an open on Windows goes
through the same bounded POSIX adapter this batch fixed.

### `loan-released(path)` while the operation is outstanding

`wf_completion_publish_milestone` is a new non-terminal route in the completion
core. It publishes one milestone fact of an operation the target is submitting
or already holds, accumulating it with `fetch_or`; it writes no result byte,
does not move the phase and raises no completion event, so a one-shot operation
still has exactly one event and exactly one terminal. `WF_COMPLETION_TERMINAL`
is refused outright: a terminal fact without the result bytes it stands for
would let a consumer read an empty slot. The terminal route may keep its plain
store because it is already checked to carry the complete
`WF_COMPLETION_OWNERSHIP_COMPLETE` product, which is a superset of anything
published earlier.

Both adapters publish `WF_COMPLETION_PAYLOAD_RELEASED` inside the submitting
call, after the target accepts and before submission returns — so the fact
holds before the caller can regain control, which is the guarantee that
matters. In the bounded adapter the copy happens under the queue lock, before
any helper can see the entry; on the ring it happens before the SQE exists.

### Tests

- `test_submitted_open_owns_its_path_bytes` in
  `compiler/src/backend/completion/harness.c` — writes two one-byte marker
  files with equal-length names, submits an open of the first, and **rewrites
  the caller's buffer to the second name immediately after every submit**, the
  way a loop reusing one scratch buffer would. It then reads the descriptor and
  requires the marker to be `'A'`. It runs both legs: the bounded POSIX adapter
  driven by the test thread alone (so the host call demonstrably runs after the
  rewrite), and the shipped bridge on whichever target it selects. It checks
  `open_directory` the same way, by a marker file that exists only under the
  first directory. It observes `WF_COMPLETION_PAYLOAD_RELEASED` set,
  `WF_COMPLETION_TERMINAL` clear and the phase still `IN_FLIGHT`. Under
  `WF_REQUIRE_LINUX_IO_URING` it also requires the ring's submission counter to
  have advanced by two, so the leg cannot pass on a silent fallback.
- `probe_open_stages_its_own_path_case` in `native_adapter_probe.c`, which
  links only the core and the ring — asserts `entry->request.buffer.path ==
  entry->path_storage` and the staged bytes, then observes the release
  milestone, then scribbles over the caller's buffer and drives the operation
  to terminal. It asserts the property of the entry rather than of the outcome
  deliberately, and that choice was checked rather than assumed: with the
  staging removed *and* the structural assertions removed, so that only the
  behavioural half runs, the probe **passes** against a caller-owned pointer on
  this kernel. `IORING_OP_OPENAT` copies the name during the `io_uring_enter`
  that the submit performs, so an outcome assertion would catch nothing today
  and would only begin failing once the doorbell is deferred. The property that
  has to hold is that the record owns the bytes, so that is what is checked.

**Both tests were falsified before being trusted.** With the staging removed
and everything else unchanged, the harness fails at `harness.c:1308: check
failed: marker == 'A'` on macOS and, with the ring required, inside the Linux
container; the ring probe fails at `native_adapter_probe.c:484:
entry->request.buffer.path == entry->path_storage`. With the staging in place
both pass on macOS and in the container with io_uring required.

## Fix 3 — completion storage belongs to an operation, not to a call

Every per-site completion storage element — token, result slot, raw value, raw
error, an open's outcome, a directory cursor's position, and the `%component`
path buffer — was a bare `alloca` in the function entry block, shared by every
hand-out of that site. The target writes the result into it and reads the
staged path out of it *while the operation is outstanding*, so two operations
of one site that are outstanding together need one element each.

`FunctionEmitter::indexed_entry_slot` reserves `count` slots of one type in the
entry block and returns element `index`; both the storage and the element
pointer are entry-block definitions, so the name dominates every block exactly
as a plain `entry_slot` name did. `completion_entry_slot` wraps it with the
site's outstanding-operation count and the index of the hand-out being
emitted. Both answer one and zero today, and the doc comments say why: a
completion schedule is submitted and joined inside the block that formed it, so
a site never holds a second hand-out — which is why one shared element has
never been wrong, and exactly why it would become wrong, silently and without a
compile error, the first time a schedule outlives its block. The count and the
index are now the whole of what a deeper schedule changes.

### Tests

- `backend::tests::completion::every_completion_storage_element_is_indexed_by_its_hand_out`
  — every `alloca` in the handed-out probe function is `[1 x T]` and is reached
  through `getelementptr inbounds [1 x T], ptr %storage, i64 0, i64 0`. Before
  the change they are bare `alloca i64`, `alloca i32`, `alloca [2 x i64]`, so
  the assertion is a real one.
- `many_files_wide8.wf` byte check: the emitted binary publishes
  `17098009301725298919 00000000000071024640`, the same line every other line
  of the bench corpus publishes.

## Parity

### The compiler half is settled without a stopwatch

Every bench program's writer module compiles to **byte-identical arm64
assembly** before and after, at the `-O2` the driver actually passes to the
host compiler. LLVM folds a `[1 x T]` alloca plus a constant-zero
`getelementptr` back into the bare alloca, so the only IR difference this batch
makes to a writer's module cannot survive to machine code.

```text
program                     LLVM IR diff lines   assembly bytes      assembly diff
many_files_narrow.wf                 0            68,545 / 68,545          0
many_files_wide.wf                 672           234,966 / 234,966         0
many_files_wide8.wf              1,754           638,637 / 638,637         0
pipe_relay.wf                       56            87,058 / 87,058          0
many_files_wide.wf   --no-overlap    —                  —                  0
many_files_wide8.wf  --no-overlap    —                  —                  0
many_files_narrow.wf --no-overlap    —                  —                  0
```

The IR differences are entirely the indexed allocas and the temporary
renumbering they cause. `many_files_narrow.wf` is identical even in IR, because
fix 1 changes nothing about it.

What is left that could move a number is the C completion runtime, and its cost
is arithmetic rather than mystery. Per **submitted open**: one `strlen` and one
`memcpy` of the component (about ten bytes for this workload), plus one
`wf_completion_publish_milestone` — a per-slot mutex acquire/release and an
atomic `fetch_or`. `many_files_wide8.wf` submits 7,168 opens, so that is 7,168
extra uncontended mutex round trips in a program whose median run is hundreds
of milliseconds.

### The wall-clock table, and why it does not decide anything

macOS 26.5.2, Apple M4, 10 cores. Sixteen interleaved passes, medians of
eleven or fifteen recorded runs after two warm-ups, both lines of each pair in
one runner process, and the pass order reversed on alternate passes. Every
recorded run byte-checked against
`17098009301725298919 00000000000071024640`; a line that published anything
else could not have reported a time at all.

```text
line                med of 16 pass medians   min pass   max pass   user med
C.wide.before                    1087.2        485.8     2866.0      133.3
C.wide.after                     1071.4        472.4     2254.3      133.1
C.wide8.before                   1005.3        671.3     1861.8      138.6
C.wide8.after                    1053.1        533.1     3207.7      140.2
```

```text
comparison                       wall           user
many_files_wide    before->after   -1.5%         -0.2%
many_files_wide8   before->after   +4.8%         +1.2%
```

**This table is reported so that nobody mistakes it for evidence.** The host
carried load averages between 6 and 33 throughout — background `cloudd`,
`CacheDelete`, `XprotectService` and the endpoint-security stack, and, for part
of it, another worktree running its own compiler gate *and another agent
benchmarking a program out of the same generated file tree*. Single recorded
runs reached 8, 11 and 45 seconds against sub-second medians, and a line whose
quiet median was 392 ms in the probe run three hours earlier has a pass-median
range here of 485 to 2,866 ms. Under those conditions a ±5 % reading is a
reading of the machine, and the two programs pointing in opposite directions is
what noise looks like rather than what a mechanism looks like.

The assembly identity above is the parity claim. This table is the honest note
that the stopwatch could neither confirm nor refute it on this host today, and
that nobody stopped rerunning when a favourable pair appeared: all sixteen
passes are in the aggregate.

The condition the design set — "`C.wide8` is unchanged within spread" — is met
in the only sense this host can support: each line's pass medians lie inside
the other's range, and the aggregate difference is a small fraction of the
spread.

### One flake, recorded rather than hidden

The first `make check` run on this revision failed one test:
`backend::tests::completion::independent_io_reaches_the_second_operation_before_the_first_unblocks`,
at its three-second `recv_timeout` on the first of its four helper
configurations. It passed three times out of three in isolation immediately
afterwards, at a host load average of 32 with another worktree's suite running,
and the whole gate passed on the re-run.

It is recorded because "it passed the second time" is not by itself an
argument. The mechanism argument is: that test drives two `write_once` calls on
pipes and performs no open, so neither the path staging nor the release
milestone — the only two things this batch adds to a running program's
critical path — is on it, and the storage indexing compiles to identical
machine code. What the batch does add to *every* file operation is a larger
`wf_file_work` copy, quantified under "Judgment calls" at about two
milliseconds across an eight-thousand-file program; it cannot produce a
three-second stall. The test spawns a child and waits three seconds for a
thread to be scheduled, on a machine that was running two full gates at once.

### Bytes

`many_files_narrow`, `many_files_wide`, `many_files_wide8` and their
`--no-overlap` builds all publish
`17098009301725298919 00000000000071024640` before and after, and every
recorded bench run is byte-checked by the runner, so a line that published
anything else could not have reported a time at all.

## The flagged specification sentence

`spec/kernel-spec.md` was **not** edited. This batch makes the following [SYS-2]
sentence true in the implementation, and the design (§4.3) states it as the
sentence the spec needs. It is recorded here, verbatim, as a flagged item for
whoever owns the [SYS-2] table:

> `open_file`'s and `open_directory`'s `name` borrow is released before target
> transfer: forming the request copies the admitted `[start, end)` range into
> compiler-owned storage, and that copy is the operation's last access to the
> caller's buffer. Every other retained borrow of a `may-suspend` operation is
> released at `terminal`.

Both halves now hold. The emitter already copies the admitted `[start, end)`
range into `%component` before the submit; this batch makes the adapter copy
`%component` into the operation record, so after `wf__completion_file_open_at_submit`
returns, neither the writer's buffer nor the compiler's staging buffer is
retained by anything. Until this batch the sentence would have been false, which
is why the design says to land the two together.

## The design record and the probes

`research/investigations/io-model/LOOP-PIPELINE.md` is the design's §§0-8
verbatim, with one edit — an absolute scratch path in the provenance line
replaced by the branch name — plus a header and an appended §9.

§9, "Probe results (qemu container, provisional until the real-Linux CI run)",
summarizes the five probes of §5.5 that were run against this same base
revision: probe A's deferred doorbell (`C.wide8` sys 54.7 -> 39.1 ms, wall 98.5
-> 84.1, `io_uring_enter` calls 15,360 -> 2,048), probe B's hand-written ceiling
(45.7 ms at depth 32 against `N.pool2`'s 31.7, and 45.1 ms for the same shape
with no ring at all, so the binding constraint is the fold's load balance and
not the transport), probe C (helpers instead of the ring are slower; the ring
off with *zero* helpers takes `C.wide8` from 98.5 to 66.0 ms and `C.narrow`
from 285 to 62, Linux-only and not to be generalised), probe D (the fold is
40-45 ms of user CPU on both hosts), probe E (both fold spellings summarize;
the real W1 exposure is an extent test moved one call deep, which today's
[ENT-6] refuses), and the five corrections to §0.1 the probes' counters
produced.

The section states plainly that every Linux number in it comes from the
qemu-virtualised container, that by the owner's ruling those are not
performance evidence, and that the two policy decisions the probes raise —
whether the Linux ring should carry this workload at all, and whether the
direct depth-one path should bypass the ring — wait for the real-Linux CI
numbers. Nothing in this batch depends on either answer.

## Judgment calls

- **The call occurrence, not the binding, is a site's identity.** A scrutinee
  call defines no binding, so making `PermissionSite::binding` optional and
  keying `call_results` by `NodePath` is not a widening for convenience — it is
  the only identity every written call position has. The alternative, a
  synthetic binding for scrutinee results, would have put a fiction into the
  checked model to keep a map key.
- **One recorded fact, not a rule per spelling.** `result_read_by_own_statement`
  moves the window's start; everything else — the denial of a scrutinee as a
  first member, the refusal of a `match` between two calls — falls out of the
  classification that already existed. A `match`-specific denial rule would have
  been a second mechanism saying the same thing.
- **`WF_FILE_PATH_CAPACITY` is 1024 and the storage is inline.** A submission
  may not allocate, so the record has to carry the bytes; 1024 is the widest
  admitted component (Darwin's 1023) plus its terminator, and also Darwin's
  whole `PATH_MAX`, which is what the harness's absolute scratch paths need.
  The cost is 64 KiB of static data in the bridge queue on every platform and
  another ~70 KiB in the ring's entry table on Linux, which is visible as the
  emitted binary's `__DATA` growing from 32 KiB to 96 KiB.
- **A work record is still copied whole.** `wf_file_work` grew from about 80
  bytes to about 1.1 KiB, and the bounded adapter copies one whole record into
  the queue at submission and one out at execution — for *every* file
  operation, not only opens. For `many_files_wide8.wf` on macOS that is 7,168
  submitted opens plus 7,168 submitted reads, each copied into the queue and
  out of it: about 14,300 x 2 x 1.1 KiB ≈ 31 MiB of `memcpy`, on the order of
  1 to 2 ms against a several-hundred-millisecond program. A kind-aware partial copy
  would remove it in about ten lines, and it is deliberately not written: the
  cost is unmeasured on this host, the whole-struct assignment cannot silently
  miss a field a future change adds, and the project's rule is to fix measured
  resource problems rather than imagined ones. It is recorded here so the next
  person finds the arithmetic instead of rediscovering it.
- **The milestone is published after `target_accepted`, not literally at
  `begin_submit`.** The design says "at `begin_submit`"; what is observable and
  what matters is that the fact holds before the submitting call returns, which
  both adapters now guarantee. Publishing between `begin_submit` and
  `target_accepted` would announce a released loan for an operation the target
  might still refuse, and the refusal path would then have to retract it.
- **`wf_completion_publish_milestone` refuses `WF_COMPLETION_TERMINAL` rather
  than trusting its callers.** A terminal fact carries result bytes; a route
  that writes none may not publish it. The terminal route keeps its plain store
  of the milestone word because it is already checked to be the complete
  ownership product.
- **The indexed storage ships with a count of one.** Emitting `[1 x T]` and a
  constant-zero `getelementptr` is machine-code-identical to a bare alloca, so
  the shape costs nothing and the test pins the invariant as a number rather
  than as an unwritten assumption. The alternative — leaving the bare allocas
  and a comment — is the shape that made fix 2 necessary.
- **Parity is claimed from the assembly, not from the stopwatch.** The host was
  too loaded for a wall-clock table to resolve a fraction of a percent. Rather
  than run until a favourable pair appeared, the assembly identity is reported
  as the claim and the noisy table is reported as a noisy table.
- **The probe numbers went into the design record's §9, not into
  `RESULTS.md`.** The design's §7 says to write them into `RESULTS.md`.
  `RESULTS.md` is the project's record of measurements taken as evidence, and
  every Linux number the probes produced comes from a qemu-virtualised
  container that the owner has ruled is not performance evidence. Putting them
  beside real measurements would have made them look like ones. They are
  appended to the design they were run for, labelled provisional, with the
  real-Linux CI run named as what replaces them.
- **The design body was landed verbatim except for one absolute scratch path.**
  A committed document should not point at a personal scratch directory, and
  the repository gate refuses one; the branch and revision the design was
  written against are named instead. The probe
  results are appended as §9 rather than edited into §0.1, so a reader can see
  which predictions held.

## Not done

- **Nothing from batches 1 to 4.** No staged permission, no pipeline, no
  privatization, no `wf__completion_window`, no deferred doorbell, no
  retire-and-retry, no `RESULTS.md` program-level update. Batch 0 is
  prerequisites only, and the design's exit condition for it is that no
  published byte and no permission verdict on the bench corpus moves.
- **`many_files_narrow.wf` still submits nothing.** Fix 1 removes one of the
  two reasons; the other is that its two system calls are in different
  statement blocks, which is what batches 1 to 3 address.
- **The `RESULTS.md` `--no-overlap` claim is still wrong on Linux.** Probe C
  found that the S line still routes every read through io_uring, so the bench
  README's "every I/O call is an ordinary direct call" is false there. It is
  recorded in the design record's §9.6 and left for the batch that owns
  `RESULTS.md`; correcting it here would have meant editing a measurement
  section this batch did not measure.
- **Real-Linux numbers.** Everything Linux in this batch is the qemu container:
  it shows that the code builds, that the ring path works, and that the new
  tests fail without their fixes. It is not performance evidence and no
  performance claim is made from it.
- **A quiet macOS parity run.** The host did not become quiet during the batch.
  What was measurable — the emitted assembly, the published bytes, the
  arithmetic bound on the runtime's added work — is above; the wall-clock table
  is reported with its spread and is not load-bearing.

## Approval classes

Under the four rules in `AGENTS.md`:

- **Rule 2** — the exact revision to be merged needs owner approval, as every
  merge does.
- **Rule 3** — the exact revision must pass `make check`. It does: the gate
  ends `== WHITEFOOT ALL TESTS GREEN ==` on this revision, and
  `cargo clippy --all-targets --profile gate -- -D warnings` and
  `cargo fmt --all -- --check` are clean.
- **Rule 4 does not apply.** This merge changes neither `spec/kernel-spec.md`
  nor conformance evidence: nothing under `tests/conformance/` is touched, no
  conformance case, manifest, adapter, runner, collection wiring or
  gate-integrity test is added, modified, deleted or renamed, and the
  specification file is byte-unchanged. No `governance/APPROVALS.md` record is
  therefore required by this merge. The [SYS-2] sentence above is flagged, not
  applied.

The changed paths are `compiler/src/{backend,lowering,semantic}/**`,
`compiler/src/backend/completion/*.{c,h}`,
`research/investigations/io-model/LOOP-PIPELINE.md` (new),
`docs/done/0089-loop-pipeline-batch0.md` (new), and two link lines in
`docs/roadmap.md`.
