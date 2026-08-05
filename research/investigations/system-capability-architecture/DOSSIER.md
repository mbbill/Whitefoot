# System-capability architecture — design dossier

Status: DRAFT FOR OWNER REVIEW. This is design evidence for active BOUND-1,
not a numbered specification, compiler interface, or implementation authority.

Date: 2026-08-05

## 1. Proposed source model

Whitefoot system access should look like explicit typed inputs, not an ambient
standard library and not one mutable `Process` object. The command entry lists
exactly what it needs. Helper functions receive those values or resources as
ordinary parameters.

This is the proposed source shape; punctuation remains a specification task:

```wf
command fn main(
    command.args as args: own Args,
    command.cwd as cwd: own DirectoryRead,
    command.stdout as stdout: own Output,
    command.stderr as stderr: own Output,
) -> own ExitStatus
effects { allocates(heap), external, blocks, traps }
```

The left side of each parameter is the standard command input being requested.
The right side is its local name and type. This matters because `stdout` and
`stderr` have the same type but are different inputs, while `cwd` describes
where this particular `DirectoryRead` came from rather than inventing a
different filesystem type. An unused input is omitted. The implementation may
receive one private ABI record, but source code never owns or passes that
aggregate.

Only three source-level distinctions are needed:

| Kind | Examples | Access rule |
|---|---|---|
| Immutable value | `Args`, `HostString`, `RelativePath` | Shared borrows read it. It has no cursor or caller-visible mutation. Owning storage does not make it a system resource. |
| Shared capability | `DirectoryRead`, later `Network` and `MonotonicClock` | A shared borrow may observe outside state or create a new owned resource. The capability owns no cursor or sequence position that a later call consumes. |
| Stateful resource | `ReadFile`, `Output`, later `TcpListener` and `TcpStream` | An operation that advances state or fixes observable order needs `&uniq` or consumes the owner. |

These are type-contract distinctions, not new writer-visible keywords. Existing
`own`, `&`, and `&uniq` syntax expresses their use.

One additional construction rule covers deliberate sharing. If several workers
must use one logical service, source code explicitly converts its unique owner
into a controller plus independent owned ports or lanes. Each worker receives
one port; workers do not share a mutable `&uniq` value. The shared core and its
ordering, backpressure, completion, and failure rules belong to that operation's
contract. TCP receive/send halves and a future batched output publisher follow
this pattern.

This gives one uniform answer to the earlier `&uniq` problem:

- splitting a large `Process` object removes false serialization between
  unrelated things;
- it does not remove serialization that represents real state, such as one file
  cursor or one output order; and
- real parallelism comes from independent resources or an explicit split, not
  from declaring a stateful object shared because its native handle happens to
  be thread-safe.

The first `wfgrep` slice needs only the three basic kinds. The controller/port
rule is fixed now so later parallel output, networking, cancellation, and clocks
can be additive instead of replacing the command API.

The system surface uses ordinary free functions; Whitefoot does not need method
syntax, traits, or a generic `Reader`/`Writer` object:

```wf
args_count(args: &Args) -> u64
arg_get(args: &Args, index: u64) -> Result<HostString, ArgError>
host_bytes_len(value: &HostString) -> u64
relative_path(value: own HostString) -> Result<RelativePath, PathError>

host_copy_bytes(
    value: &HostString,
    destination: &uniq buffer<u8>,
    offset: u64,
    capacity: u64,
) -> Result<u64, CopyError>
effects { traps }

open_read(
    root: &DirectoryRead,
    path: &RelativePath,
) -> Result<ReadFile, IoError>
effects { external, blocks }

read_once(
    file: &uniq ReadFile,
    destination: &uniq buffer<u8>,
    offset: u64,
    capacity: u64,
) -> ReadOutcome
effects { external, blocks, traps }

write_once(
    output: &uniq Output,
    source: &buffer<u8>,
    offset: u64,
    count: u64,
) -> Result<u64, IoError>
effects { external, blocks, traps }
```

Outcomes are per-operation, never one shared union. An operation with exactly
two outcomes returns a prelude `Result` instantiation, so it declares no new
variant name; only `read_once` has three outcomes and therefore needs its own
enum. §6.7 gives the complete outcome inventory and the reason.

The sketches show the new system effects. Their full checked signatures also
carry the ordinary region `reads`/`writes` required by their borrows; `traps`
on read/write and on the buffer-copy operations covers invalid destination
ranges. No writer declares these built-ins or their effect rows.

Both the sketches above and the block below elide region identifiers, which the
language always writes. Accepted source writes a region into every borrow and
opens a `region` statement to introduce it. Three rules make the placement
structural rather than cosmetic: a borrow of an own-mode binding must use a
region introduced within that binding's scope, a borrow inside a labelled loop
may only name a region introduced inside that loop body, and region identifiers
are unique function-wide. Each structurally forced site therefore opens its own
region under a distinct spelling — in the block below, the loop body, and every
arm that borrows its own-mode binder, which is both arms passing an `IoError`
to `io_status`.

A sequential search then has this concrete shape:

```wf
let raw_arg: own Result<HostString, ArgError> =
    arg_get(args: &args, index: file_index);
let host_path: own HostString = match raw_arg {
    Ok(value: v) => { give move v; }
    Err(error: e) => { return usage_status(); }
}

let built: own Result<RelativePath, PathError> =
    relative_path(value: move host_path);
let path: own RelativePath = match built {
    Ok(value: v) => { give move v; }
    Err(error: e) => { return usage_status(); }
}

let opened: own Result<ReadFile, IoError> = open_read(root: &cwd, path: &path);
let file: own ReadFile = match opened {
    Ok(value: v) => { give move v; }
    Err(error: e) => { return io_status(error: &e); }
}

loop @scan {
    let step: own ReadOutcome = read_once(
        file: &uniq file,
        destination: &uniq input,
        offset: input_offset,
        capacity: input_capacity,
    );
    match step {
        ReadBytes(count: n) => {
            // Scan only those n bytes and append formatted matches to a
            // reusable OutputBatch; a full batch calls write_once.
        }
        ReadEnd() => { break @scan; }
        ReadFailed(error: e) => { return io_status(error: &e); }
    }
}
```

The three outcome dispatches are not incidental verbosity. Matching is
exhaustive with no wildcard arm, so they are the source cost §6.7 accounts for
when it gives each operation its own outcome type, and `io_status` is where the
thirty portable error classes are matched once instead of at every site. Both
`io_status` and `usage_status` are ordinary writer-defined helpers that build
their result with `exit_status`; neither is a system operation, and neither
adds anything to the slice in §9.

See what is and is not serialized: `cwd` is shared because opening a file does
not advance a source-visible cursor. The returned `file` is unique because each
read advances that file's cursor. `stdout` is unique because writes establish an
observable order. The input and output buffers are ordinary uniquely borrowed
Whitefoot memory. No helper receives a `Process`, and no helper can use a system
input absent from its parameters.

## 2. Current Whitefoot boundary

The active v0.17 language and compiler provide useful pieces but no system
interface:

- FN-7 fixes one no-argument main returning unit and permits only heap
  allocation and traps in its effect row.
- EFF-1 and EFF-2 describe only memory reads and writes, heap or arena
  allocation, and traps.
- affine ownership, explicit move, unique borrowing, initialized buffers,
  Result, and compiler-derived cleanup on normal control-flow edges can support
  a narrow class of owned resources and synchronous read-into operations;
- a process-aborting trap performs no language cleanup;
- CAP-1 only reserves the names Sendable and Shareable; it defines no thread
  construct or memory model;
- GATE-1 and LEDGER-1 reserve a trusted boundary definition route, but do not
  define system access, resource rights, a target ABI, or compiler
  implementation. That route is deliberately not the one this design takes:
  LEDGER-1 puts unsafe regions, FFI extern frames, and trusted primitive
  imports in one family sharing one obligation ledger, and §3.3 requires
  general FFI to stay separate from compiler-owned system operations. §11
  selects a distinct domain instead; and
- the backend always calls the Whitefoot main with no arguments and returns
  process status zero. Its only ordinary external I/O is a private
  write-to-stderr path used before aborting on a trap; allocation, release, and
  abort also cross the current compiler/runtime trust boundary.

Nine further active-specification rules are not gaps but constraints this
design must negotiate with. They are listed here because §11 gives each a
disposition:

- OP-1 closes callee resolution: an IDENT callee is either an operation-table
  family or a top-level source function, and absence from both is a hard error.
  Every system call in the first slice is currently neither.
- PRE-1 is a closed, normatively counted prelude, and TYPE-6 makes its members
  visible throughout every compilation unit. It is one candidate home for the
  slice's opaque types and outcome constructors, and the reason §11 does not
  select it is recorded there rather than left to inference.
- EFF-1 fixes the effect-row grammar and requires a mandatory canonical order,
  so a new category needs both a grammar production and an order position.
- FN-3 is the second rule that reads an effect row. Its contract-conformance
  equality normalizes a row to four capabilities, so two rows differing only in
  a category it does not know about compare equal.
- TYPE-6 makes enum constructor names unique across the whole compilation unit
  and context-free, so operation-specific outcome types compete for one flat
  namespace.
- STOR-5 keeps storage borrow-free and region-free, but tests for a borrow or a
  region-bearing type. An opaque backing lease is neither, so STOR-5 does not
  by itself forbid storing one.
- `slice<'r, T>` already carries finite storage-origin sets through binding,
  moving, passing, returning, and call substitution. It is the active
  language's existing answer to a pointer-and-length value whose backing must
  outlive it, and §5 records why the first slice does not reuse it.
- STOR-3 already derives deallocation on control-flow edges after checking.
  It is the rule the completion policies in §6.5 extend.
- ERR-2 requires every match to be exhaustive over declared variants with no
  wildcard arms, which sets the source cost of a wide portable error class.

Three questions therefore remain separate:

1. What may this code access? The explicit entry inputs and typed parameters say.
2. Can this call affect or wait on the outside world? Its effect row says.
3. How does the selected target perform the operation? A compiler-owned target
   implementation says.

GATE-1 can control the third. It does not by itself solve the first two.

## 3. Requirements

### 3.1 Safety and access

- Ordinary source cannot forge a resource, target implementation, system
  operation, access right, or external-effect fact.
- A program begins with no implicit system access. Its entry declaration names
  the inputs it requires.
- Runtime delegation is explicit movement or an approved narrowing operation.
  Duplication is never inferred from integer or handle copying.
- Every resource operation states ownership of inputs and outcomes, partial
  progress, recoverable failure, normal cleanup, and process-abort behavior.
- Resource handles do not grant arbitrary foreign memory access.

### 3.2 Performance

- The normal byte path has no per-byte host call, mandatory whole-input
  materialization, whole-input zero fill, avoidable full copy, centralized
  target lock, or global I/O fence.
- Synchronous input admits caller-owned initialized storage. The target code writes
  only within the stated capacity and returns the exact valid prefix length.
- The architecture has places for positioned and vectored I/O, batching,
  mapping, splice or forwarding, and owned asynchronous buffers.
- A static native implementation can lower an operation to a direct call or target
  intrinsic. Dynamic component dispatch is not required by source semantics.
- Independent resources and workers remain independent unless their real
  ordering or alias domain requires otherwise.

### 3.3 Completeness and evolution

- CLI context, filesystem, streams, clocks, randomness, sockets and DNS,
  waiting and cancellation, threads or tasks, child processes, signals, local
  IPC, memory mapping, and target or device access all receive an explicit v1,
  later, target-specific, or unsupported disposition.
- Synchronous operations do not preclude composable async or retain an ordinary
  borrow across an untracked suspension.
- Native paths and process arguments are not silently restricted to Unicode.
- A new target implementation cannot change source semantics merely because its
  OS API differs.
- General FFI remains separate from compiler-owned system operations.

## 4. Evidence from WASI

WASI is useful comparison evidence, not a source contract for Whitefoot.

### 4.1 What survives review

- WASI 0.1 established deny-by-default capability access through preopened
  directories and fd rights. That security principle survived later versions.
- WASI 0.2 separated link-time interface capabilities from unforgeable runtime
  resource handles and replaced one flat module with modular interfaces and
  worlds.
- owned and borrowed resource handles express transfer and temporary use much
  better than numeric fd values.
- relative filesystem operations rooted in directory capabilities provide a
  safer and more composable namespace boundary than ambient process cwd.
- separating system clock, monotonic clock, secure random, insecure random, and
  runtime seed capabilities avoids false equivalence among nondeterministic
  sources.

### 4.2 What Whitefoot must not copy

- WASI 0.1 is a C-oriented flat namespace over raw pointers and a global fd
  table. It is now legacy.
- WASI 0.2 pollable resources could not propagate wakeups through a composed
  A-to-B-to-host chain. WASI 0.3 moved async into the Component Model with
  async functions, streams, and futures to repair that sandwich problem.
- WASI filesystem paths use strings and therefore cannot represent every native
  Unix filename.
- caller-supplied buffers, stronger zero-copy routes, integrated cancellation,
  and cooperative then preemptive threads remain explicit items on the WASI
  0.3 roadmap.
- WASI 0.3 native async is not a stable portable guest thread system. Threads
  and parallel APIs remain proposal work.
- stable WASI CLI is an invocation environment, not a POSIX process subsystem:
  it has no complete fork, exec, spawn, wait, PID, or signal surface.

Primary sources:

- https://wasi.dev/releases
- https://wasi.dev/releases/wasi-p1
- https://wasi.dev/releases/wasi-p2
- https://wasi.dev/releases/wasi-p3
- https://wasi.dev/roadmap
- https://github.com/WebAssembly/WASI/blob/main/docs/Capabilities.md
- https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md
- https://github.com/WebAssembly/component-model/blob/main/design/mvp/Concurrency.md

## 5. Alternatives

| Alternative | Useful property | Fatal or material problem | Disposition |
|---|---|---|---|
| Raw syscalls and integer fds in source | Thin native ABI and complete OS access | Forgeable identities, implicit global fd table, manual close, weak effect precision, poor Windows portability, and an unchecked pointer wall | Reject as source semantics; permitted only inside compiler-owned target code |
| Ambient functions such as args, open, stdin, and spawn | Small call sites and familiar APIs | Hidden access and inter-function channels contradict FN-7's no-global rationale; system use is invisible in signatures and hard to narrow, test, or parallelize | Reject |
| One permanently retained affine `Process` object | Simple bootstrap | Every operation needs the same unique holder, falsely serializing files, stdout, networking, clocks, and workers; making it shared requires a central lock or hidden aliasing | Reject; a private entry ABI record projected into independent parameters is compatible with the recommendation |
| Literal WASI 0.2 or 0.3 source API | Existing modular taxonomy and portability work | Unicode-only paths, no guaranteed caller buffer or zero-copy path, async tied to Component Model costs, incomplete threads/process surface, and semantics chosen for cross-language components rather than Whitefoot ownership | Reject as the language contract; retain as a possible target implementation for the operations it can supply, not as a complete first-slice target — see §6.10 |
| Reuse `slice<'r, T>` region-carrying views for host strings and paths | Existing checked origin, overlap, and lifetime machinery; no new rule needed to keep backing alive | Regions become source-visible on every argument-derived value, and STOR-5 then forbids storing any such value inside a struct, enum payload, buffer element, or box | Reject for the first slice in favour of the opaque lease, whose safety rests on §6.8's command-lifetime premise; this remains candidate (a) for the general backing-lifetime rule §6.8 leaves open |
| Exact entry inputs plus typed values/resources and static target code | Explicit access, precise ownership, target portability, and no required dynamic dispatch | Requires new cleanup, external-effect, entry, and later async/thread semantics | Recommend, subject to the closure gates in this dossier |

## 6. Proposed architecture

### 6.1 Layering

The boundary has four layers:

1. **Source contract:** entry inputs, types, ownership, operation outcomes,
   effects, and cleanup.
2. **Checked IR:** fixed system-operation IDs, resource identities and alias
   relations, memory operands, and derived cleanup.
3. **Target implementation:** compiler-selected code that may use libc, direct
   system calls, OS libraries, or a WASI host.
4. **Private host ABI:** calling conventions, native handles, trampolines, and
   OS details that source code cannot observe or forge.

An optimization may change layers 2 through 4 only while preserving layer 1.
A function table used by deterministic tests is an implementation choice;
native commands do not pay for it when their operations are statically bound.

### 6.2 Program kinds and exact entry inputs

A program kind fixes its lifecycle and entry shape. Its entry declaration then
lists the exact standard inputs it needs. A command that does not use a clock or
network receives neither. If a declared input cannot be supplied, startup
fails before Whitefoot code runs. `Option` is used only when absence is part of
an operation's normal meaning, not to hide an undeclared input.

These are the kinds a program may declare, not a requirement that every unit
declare one. A unit that requests no standard inputs declares no kind and keeps
the unlabelled entry, which is the population §11.1's selection turns on.

The planned program kinds are:

- command: one entry invocation and one normal `ExitStatus` return;
- service: one long-running, process-owned invocation that receives its exact
  listener, clock, shutdown, and worker inputs and runs its own loop; arbitrary
  repeated host callbacks are a separate hosted-program design;
- embedded: a reserved future program kind. No embedded target or system
  operation is qualified by this decision until trap means halt or reset in a
  precisely defined way and interrupt, device, DMA, memory, and pending-I/O
  cleanup have a complete contract; and
- hosted library or arbitrary imported/exported foreign calls: BOUND-2, not a
  command or service variant.

Normal command status is the return value of the entry function, not a
`Process` object. Startup or target-qualification failure happens before
entry and therefore is not a source-returned status. A trap is abnormal
instance termination and also does not return an `ExitStatus`.

Entry inputs are independent parameters. `command.cwd` and `command.stdout` are
standard input labels, while `cwd: DirectoryRead` and `stdout: Output` are the
local bindings and reusable types. This separation avoids types such as
`ProcessCwdRead` and `CommandOutput` that mix where an object came from with
what operations it supports.

Arguments and the initial environment are immutable invocation snapshots.
Filesystem roots and network access are capabilities that create resources. A
clock is a shared observation/factory capability: reading it observes outside
state and creating a timer returns a new owned resource, but neither advances a
caller-owned clock cursor. Source order for clock calls follows the ordinary
`external` rule. Output, random streams, listeners, files, and workers with
queues or joins are live stateful resources unless their individual contracts
say otherwise.

### 6.3 Values, capabilities, resources, and ports

An immutable value can own storage without becoming a system resource. It is
safe to read through shared borrows because source-visible state does not
change. `Args`, `HostString`, and `RelativePath` use this rule.

A capability owns no caller-visible cursor or sequence position that a later
call consumes. Shared operations may observe outside state or create
independent owned resources.
Examples are:

- `open_read(&DirectoryRead, &RelativePath) -> ReadFile`; and
- future operations that observe `MonotonicClock`, create a socket from
  `Network`, create a timer from a clock, or create one shutdown watch per task
  from a shutdown input.

A resource identifies one live stateful object. It is opaque, unforgeable, and
affine. Movement transfers it. A shared borrow may inspect it without changing
its public state. An operation that advances a cursor, fixes output order,
accepts the next connection, or otherwise changes state uses `&uniq` or consumes
the resource. A duplicate or split operation exists only when its alias,
ordering, cleanup, and concurrent-use rules are complete.

Static system types encode standard operation sets instead of exporting
arbitrary integer right masks: examples include DirectoryRead,
DirectoryMutate, ReadFile, WriteFile, TcpListener, TcpStream, and Output. The
set is a closed compiler-owned lattice for each family, not one nominal type per
OS flag combination. A host may impose a narrower dynamic policy and return
PermissionDenied; a program cannot widen it. Attenuation consumes the broader
owner and returns a standard narrower resource. Retaining both values
requires a separate family operation whose alias, cleanup, and sharing contract
is explicit; source never edits or copies an integer mask.

Several workers do not share one stateful resource. A family that supports
parallel use exposes an explicit construction that consumes the original owner
and returns independent affine handles linked to a compiler-known shared core.
Each handle has one owner. For example:

- `split(move TcpStream)` returns one receive half and one send half;
- a future output-publisher builder consumes `Output` and creates one exclusive
  SPSC port per worker while a controller remains the only output owner; and
- a shutdown input creates an owned watch for each task instead of moving one
  affine token into many tasks.

The source type need not reveal whether the target uses a reference count,
mutex, kernel object, or no shared allocation at all. Those are implementation
details. The contract must still state ordering, backpressure, failure,
completion, and the pairs of operations that may run concurrently.

Revocation is not part of the first system interface. It requires shared state,
a concurrency memory model, and explicit stale-handle outcomes. The capability
map reserves it as later work instead of pretending affine movement solves it.

### 6.4 Resource contracts

Every compiler-owned resource family has one normative contract. Names in the
capability map are not considered designed until that contract
states:

1. states and legal transitions;
2. the semantic object, cursor, lane, and ordering aliases created by open,
   duplicate, split, move, and attenuation;
3. the disposition of every resource and data owner on every outcome;
4. implicit release, explicit finish or abandon, whole-process abort, and any
   separately selected contained-instance behavior;
5. operation pairs that may progress concurrently and the ordering they expose;
6. pending-operation, cancellation, and quiescence rules where applicable;
7. exact external effects, including compiler-inserted release; and
8. cross-platform guarantees that a target may reject as unsupported but may
   never silently weaken.

The contract is compiler-owned semantic data. A native fd, handle value, or
target table entry is never the identity or alias proof exposed to analysis.
The four exemplar contracts in section 7 are closure tests for the shared
model, not promises to implement those families together.

### 6.5 Resource completion and cleanup

Resource protocols use one of three completion policies:

1. **release-complete** — compiler-derived release is the complete language
   obligation. A ReadFile is the first example: losing a native close
   diagnostic cannot invalidate already observed bytes or promise durability.
   `DirectoryRead` and `Output` take the same policy, on the grounds their
   contracts give in §7.1 and §7.3, so every first-slice resource is
   release-complete and the first slice needs no exact-use checking.
2. **explicitly abandonable** — the type exposes a consuming `abandon` whose
   contract explicitly permits loss of unfinished external work. Abandonment is
   a source action, not an accidental affine discard.
3. **completion-required** — every normal or recoverable exit must consume the
   owner through `finish`, `cancel-and-reap`, `wait`, or another terminal
   transition. This needs a new exact-use checker obligation. Buffered output,
   atomic file replacement, pending I/O, and an unreaped child cannot silently
   use ordinary affine abandonment.

On every normal control-flow edge, the checker records exactly one owner
disposition: moved or returned, explicitly completed or abandoned, or released
by a permitted compiler-derived transition. These cases are mutually exclusive.
Every release that may run contributes its external effect to the function's
exact effect union; an owner consumed by `finish` is not released a second time.

The resulting legality rule is exact: a declared row is legal exactly when it
equals the union of the function's syntactic body effects and the effects of
every release that may run on any normal edge. This covers an owner moved on
one `match` arm and released on another, and release that runs only on some
paths; both take the union over all normal edges, so a conditionally released
resource still contributes its effect. The rule is deliberately limited to the
new resource families. v0.17's existing `buffer<T>`, `box<T>`, and arena
releases are not retrofitted into effect rows: retrofitting them would change
the legal row of every existing program and conformance case that owns one,
which no current experiment needs.

A consuming close or finish invalidates the source handle on success and error.
In particular, target code may not retry a numeric POSIX fd after `close`
reports `EINTR`, because the native descriptor may already be closed and reused.
`flush`, `sync_data`, `sync_all`, directory sync, atomic commit, socket
`finish_send`, and final handle release are different semantic operations. A
finish error does not roll back earlier writes and does not resurrect the owner.

The selected baseline preserves v0.17's **whole-process abort** law. A trap runs
no Whitefoot cleanup and returns no status. Command and service deployments
therefore terminate their owning process; a host that needs isolation runs the
Whitefoot instance in a separate process. The OS reclaims process-local memory
and handles, while external writes are not rolled back and persistent objects
or already spawned work retain the family semantics they had before the trap.

Host-surviving in-process trap containment is not selected by BOUND-1. It would
be a separate owner-approved language amendment. Such an amendment must give the
runtime an instance resource table, transfer every pending operation to a
reaper, and delay memory reclamation until every worker, continuation, and host
operation is quiescent. It may request cooperative cancellation but may not
asynchronously kill a language worker; quiescence can be unbounded. A profile
that cannot meet those rules must use process isolation or remain Unsupported.

User-defined destructors and arbitrary writer-defined resource contracts are
not required. The first design uses fixed compiler-owned resource families with
complete gated contracts.

### 6.6 Source effects and internal ordering facts

The first source contract needs only two new effect categories:

- `external`: the call may observe or change state outside ordinary Whitefoot
  memory, including file contents, cursors, output, namespaces, clock or random
  sequences, resource lifetime, and compiler-inserted resource release; and
- `blocks`: an ordinary call may block its current host thread.

`blocks` earns its own category because an operation that is `external` without
being `blocks` expresses something no other row can: a vDSO monotonic-clock
read, or a nonblocking socket call that returns `WouldBlock` instead of
waiting. Being honest about the first slice, no such operation exists in it
either, because §8 places clocks at `none; later additive interface`. The
category is reserved on the strength of a named successor operation rather than
a current need.

Within the first slice, therefore, no rule reads `blocks` and it is exactly
coextensive with `external`: every operation carrying one carries the other.

It is nevertheless fixed now rather than deferred, for one specific reason. A
declared row must equal the computed row, so introducing a category later is a
mechanical edit to every row that reaches the affected operations. That alone
does not distinguish `blocks` from the suspend and spawn categories deferred
below, which face the same migration. What distinguishes it is decidability:
whether an operation blocks is already determinable for every operation in the
first slice, while whether an operation suspends or spawns cannot be decided
until the async and task designs exist. Fix the category you can already
decide; defer the ones you cannot.

A row's shape is unchanged: it remains an exact union of categories with no
subtyping, exactly like the current memory, allocation, and trap effects. Its
derivation is not unchanged. The active EFF-2 attribution is syntactic over the
function declaration, and compiler-derived release has no syntactic occurrence,
so the first slice extends that attribution to cover release on normal
control-flow edges. §6.5 states the resulting legality rule and §11 records
the delta. EFF-2 is not the only rule that reads a row: FN-3 compares rows for
contract conformance through its own four-capability normalization, and §11
records that extension too, because a category FN-3 cannot see would let a
`pure` member bind an externally effectful function. `pure` excludes both new
categories. `open_read`, `read_once`,
`write_once`, and native resource release have `external`; each may also have
`blocks` when its contract permits synchronous waiting. In the first slice,
compiler-derived `DirectoryRead` and `ReadFile` release has both `external` and
`blocks`. Immutable `Args` and path operations have neither unless they
allocate ordinary Whitefoot memory.

The source row deliberately does **not** say `external(cwd)` or
`changes(file)`. Such parameterized effects would require every `Result`,
structure field, helper return type, move, and call substitution to preserve a
source-visible origin for each resource. The current design would then forbid
the optimizer from using that origin as a disjointness or reordering proof, so
the complexity would have no consumer.

Instead, explicit typed parameters say what a function can access. Ownership
and each resource contract say which calls may run concurrently. Checked IR
retains the actual resource IDs, alias links created by open/duplicate/split,
and operation order for auditing and lowering. These facts may become verified
optimization inputs later without adding dependent resource-origin types to the
first source language. The stdout/stderr may-alias record in §7.3 is the named
example: nothing in the first slice reads it, and it is retained so a later
cross-resource reordering fact cannot treat two separate `Output` owners as
disjoint sinks.

Sequential external calls remain in source program order, even when they use
different resources. This is a semantic rule, not a global runtime lock:
explicit workers and independent owned resources can still execute
concurrently. Native handle values, separate opens, or target metadata never
prove disjointness. A later verified fact may permit a transformation, but
facts-off compilation stays correct.

Starting background I/O, suspending a task, and spawning execution will need
their own control-effect decisions when those language features are designed.
They are not predeclared as first-slice vocabulary. A pending operation must in
all cases own every lane and buffer retained after the submitting call.

### 6.7 Data transfer and portable errors

The architecture uses operation-specific outcomes rather than forcing every
hot I/O operation through one large tagged union or scheduler.

Each operation therefore declares its own outcome type, and those types are
built to fit the active language's flat, whole-unit constructor namespace,
where every enum variant name is globally unique. Two rules do the work. An
operation with exactly two outcomes returns a prelude `Result<T, E>`
instantiation, which is the specified form for recoverable errors and declares
no new variant name at all. An operation with more than two outcomes gets one
bespoke enum whose variant spellings are prefixed by its operation, so no two
operations compete for a name. The complete first-slice inventory is:

| Operation | Outcome type |
|---|---|
| `arg_get` | `Result<HostString, ArgError>`, `ArgError { InvalidIndex(); }` |
| `host_bytes_len` | `u64` — total, no failure outcome |
| `host_utf8_len` | `Result<u64, Utf8Error>`, `Utf8Error { Utf8Invalid(); }` |
| `host_copy_bytes` | `Result<u64, CopyError>`, `CopyError { CopyTooSmall(required: u64); }` |
| `host_copy_utf8` | `Result<u64, Utf8CopyError>`, `Utf8CopyError { Utf8CopyTooSmall(required: u64); Utf8CopyInvalid(); }` |
| `relative_path` | `Result<RelativePath, PathError>`, `PathError { PathInvalid(); }` |
| `open_read` | `Result<ReadFile, IoError>` |
| `read_once` | `ReadOutcome { ReadBytes(count: u64); ReadEnd(); ReadFailed(error: IoError); }` |
| `write_once` | `Result<u64, IoError>` |

That is nine new variant names beside the thirty `IoError` classes and the
prelude's ten, with no collision among them. Distinct spellings also keep
`PathError`'s `PathInvalid` separate from `IoError`'s `InvalidPath`, which are
deliberately different failures.

One shared outcome union across these operations was considered and rejected.
Matching is exhaustive over an enum's *declared* variants with no wildcard arm,
so a shared union would force every call site to hand-write arms for outcomes
its own operation can never return — dead arms in the read loop and the
write loop, the two hottest sites in the program. Variant addition also
surfaces site-enumerated edit lists, so folding each later operation into a
shared union would turn an additive change into a whole-corpus edit.
Per-operation outcomes cost naming discipline instead, which a flat namespace
demands anyway.

Only the operations that share `IoError` can chain error propagation directly;
the distinct error types deliberately do not convert into one another, because
the language has no implicit conversions. In the first slice that means
`open_read` and `write_once` can chain inside a helper whose written result is
`own Result<U, IoError>`. Propagation is never available at the command entry
itself, which returns `own ExitStatus`, so every failure reaching the entry is
matched and mapped there.

The synchronous stream/file primitives are one-attempt operations:

    read_once(&uniq resource, &uniq initialized-buffer, checked-range)
        -> ReadBytes(count) | ReadEnd | ReadFailed(io-error)

    write_once(&uniq resource, byte-view, checked-range)
        -> Ok(count) | Err(io-error)

Both resource and buffer owners remain with the caller because the synchronous
operation holds only call-scoped borrows. On a read result, exactly `count`
bytes at the start of the requested range may have changed and the remaining
buffer is unchanged. The cursor advances by exactly `count`. A short success is
not EOF; only `ReadEnd` states that no byte was available at the observed end.
Target code returning a count outside the checked range violates its
compiler-owned contract; source code does not need to defend against it.

Every buffer range is validated before any target call or destination write.
Overflow in `offset + capacity`, an offset beyond the buffer, or a capacity that
extends past it traps with the existing bounds-check semantics and leaves the
resource and buffer unchanged. The target is never asked to validate a source
pointer.

For a zero-length range, both operations report a count of zero without issuing
a host transfer; a zero-length read is never reported as `ReadEnd`. For a
nonempty range, read returns `ReadBytes(n)` only for `n > 0`, and write never
returns `Ok(0)`: a host zero-write is `Err(WriteZero)`. A failure result from
these first primitives leaves the initialized buffer unchanged.

One source `read_once` or `write_once` maps to at most one host transfer
attempt. If that attempt reports progress, the target code returns it
immediately; it does not hide a later error by looping. A reported interruption
is returned as Interrupted. `read_exact`, `write_all`, and retry policy are
ordinary library loops that can inspect cancellation or signal state and
accumulate `(progress, terminal reason)` themselves.
Stream zero progress, datagram zero length, closed peers, `would_block`, and UDP
truncation are family-specific outcomes fixed by their contracts, not guessed
from a shared count convention.

`IoError` has this closed portable class set: NotFound, PermissionDenied,
AlreadyExists, NotDirectory, IsDirectory, DirectoryNotEmpty, ReadOnly,
ResourceBusy, InvalidInput, InvalidPath, Unsupported, Interrupted, WouldBlock,
TimedOut, BrokenPipe, WriteZero, UnexpectedEnd, ConnectionRefused,
ConnectionReset, ConnectionAborted, NotConnected, AddressInUse,
AddressUnavailable, ResourceExhausted, FileTooLarge, NoSpace, QuotaExceeded,
CrossDevice, DeviceFailure, and Other. A class may carry fixed-size inline
target detail — an errno-sized code and a small target discriminator — for
diagnostics only. That detail is copy data: it allocates nothing, owns nothing,
and has no release action, so `IoError` takes no row in §9's release table and
the operation rows stay allocation-free. A message, a buffer, or any heap-backed
payload is excluded, because it would allocate on every failing call and
invalidate those rows under §6.5's row-equality rule. A target that cannot fit
its detail in that space maps to `Other` and reports the rest through its own
diagnostics.

A payload-carrying variant is affine under the ordinary ownership rules, so
`IoError` with detail and `ReadOutcome` are moved or matched rather than copied.
Neither needs a release action and neither takes a release row; the affinity is
a source-form consequence, not a cleanup obligation. A richer diagnostic payload
remains possible but is a different design and must be priced as one: it would
need a release row in §9, `allocates(heap)` on all three failing operations,
and a Sendable/Shareable judgment in §6.9.

Exhaustive portable control flow branches on the stable class; raw
errno or platform detail is not a portable semantic discriminator. A target
may return Unsupported rather than silently weaken a guarantee. New native
errors map to Other until a later numbered specification deliberately adds a
portable distinction.

Failure to establish stable immutable argument backing is not one of these
outcomes at all: it is a startup failure before entry, and §6.10 makes it a
target-qualification failure rather than something source code observes. The
argument, path, and text failures in the table above stay deliberately small
and separate so they cannot masquerade as host I/O failures. Because a
30-class `IoError` must be matched exhaustively without a wildcard arm, the
slice expects one `io_status(error: &IoError) -> own ExitStatus` helper written
once, so
those arms appear in exactly one place rather than at every call site.

An asynchronous operation cannot retain an ordinary Whitefoot borrow across
suspension. Submission therefore consumes an owned buffer or pool lease and a
resource lane into an affine `PendingOp`. Submit failure returns both owners.
While pending, the target code pins the data, registration, and lane; the original
file or socket lane cannot be moved, closed, or used through another owner.
Completion returns every owner plus one terminal outcome.

Cancellation is only a request. A normal completion and cancel acknowledgement
race at a contract-defined linearization point, but exactly one terminal result
wins and reports any progress. No buffer is returned until the target code proves
that the kernel or device can no longer access it. `PendingOp` is
completion-required: normal code must await it or perform `cancel-and-reap`.
Whole-process abort relies on process teardown, not language cleanup. If a later
amendment selects contained-instance traps, its runtime orphan reaper inherits
the pending obligation. Dropping a pending token never frees or reuses its
buffer behind an active host operation.

Positioned variants do not advance a cursor. Vectored forms must not store
ordinary borrowed slices beyond the call; future owned segment chains have
their own contract. Mapping and splice are separate capabilities, not secret
implementations of read:

- splice transfers bytes between compatible resources without materializing
  them in source memory; and
- Mapping owns the mapping lifetime and is the provenance root of every view.
  Truncation, invalidation, dirty-page, sync, and fault behavior must be fixed
  before mapping can enter a numbered specification.

### 6.8 Paths and invocation strings

Arguments, environment entries, and native paths are lossless target-indexed
host strings, not automatically text. Their underlying code-unit width is not a
portable source layout:

- Unix-family targets preserve arbitrary non-NUL bytes; and
- Windows-family targets preserve native 16-bit code units, including values
  that are not valid Unicode scalar sequences.

Those two families are the closed set. A target outside both must define its own
lossless code-unit family, or it fails qualification for the host-string and
path operations rather than narrowing them to what its own string domain can
carry.

Conversion to UTF-8 text is explicit and fallible. Native path output has a raw
lossless route; escaped and lossy display are separate presentation operations.
A directory entry name returned by enumeration round-trips component-for-
component into `open_child` without text conversion.

The path model distinguishes `PathComponent`, `RelativePath`, and an absolute
target path. NUL and target-invalid prefixes are rejected during construction.
Directory-relative operations never implement confinement with string-prefix
concatenation. They define whether `..`, a final symlink or reparse point, and
intermediate links are rejected or followed.

Two filesystem-root contracts are explicit:

- a process-equivalent namespace capability may intentionally follow native
  resolution anywhere its full namespace grant permits; and
- a confined directory capability guarantees that lexical traversal,
  symlinks/reparse points, mount transitions, and rename races cannot escape the
  granted root. A target unable to uphold that contract returns Unsupported.

Absolute paths, Windows drive or UNC prefixes, and cross-root operations require
the appropriate filesystem input; `DirectoryRead` alone does not imply them.
The `command.cwd` entry binding supplies one process-equivalent `DirectoryRead`:
`..` and native links may resolve outside the initial directory within the
process namespace. A future confined root has a distinct
`ConfinedDirectoryRead` type and contract rather than changing this promise at
runtime.

`Args` is an opaque immutable entry value. Its backing remains valid until the
command invocation ends. Native Unix and Windows targets use the existing argv
backing directly; a target without stable native backing snapshots all arguments
once before entry or refuses startup.

`arg_get(&Args, index)` returns a small opaque `HostString` lease containing a
private pointer and length into that immutable backing. It performs no heap
allocation or byte copy. Several leases may refer to the same immutable bytes.
The source value owns the lease, not unique argument storage, and cannot expose
or mix the target's u8 or u16 code units. `InvalidIndex` is its only source-level
failure.

Every first-slice lease rests on one explicit premise: its backing is the
command-lifetime argv snapshot, fixed before entry and valid until the
invocation ends. That premise, not a checker rule, is what makes the first
slice safe. Because the backing strictly outlives every value derived from it,
no lease can dangle however it is moved, returned, stored, matched, or retyped,
and the compiler needs no lifetime relation between a lease and its backing.
§6.10 makes the premise a required target guarantee so a target that cannot
supply it fails qualification instead of silently invalidating every lease.

The premise does not extend to a producer whose backing is not
command-lifetime — future directory enumeration is the obvious case, where the
names live in an iterator batch that can be reused or released. Such a producer
needs a rule the first slice does not have, and selecting it is an open
decision, not a detail. There are two candidates and they differ in kind:

- **(a) a region-bearing lease**, which reuses the existing region-carrying
  view machinery so the checker relates a lease to its backing exactly as it
  already relates a slice to its storage. This buys enforcement at source
  level, but regions become visible on every argument-derived value and the
  borrow-free storage rule then forbids storing any such value; and
- **(b) an owned-backing resource type**, which gives the value its own
  backing and a real release action. This keeps values storable, but it makes
  the type a resource rather than an immutable value, with the release,
  aliasing, and sharing contract that implies (§7.1).

Neither is selected here. What matters for the owner decision is that the
first slice is sound without either, and that the choice is a source-language
decision: it cannot be discharged by facts recorded after acceptance.

Converting `HostString` to text is explicit and fallible through UTF-8 length
and caller-buffer copy operations. Converting it to `RelativePath` consumes and
validates the lease, then transfers the same backing obligation into a new
opaque type. The argv fast path remains an inline retype with no copy. Invalid
path syntax consumes the input and returns no path.

### 6.9 Concurrency, waiting, and cancellation

Every resource family declares `Sendable`, `Shareable`, and its concurrent
operation matrix independently. Its compiler-owned contract proves that a
target representation satisfies those predicates; the language checker then
enforces movement, sharing, and non-interference. Neither side can infer them
from an fd or pointer.

- a cursorful file is movable to one worker but not implicitly shareable;
- positioned-read lanes may later be independently movable under an exact
  contract;
- a connected socket can be consumed by `split` into linked affine receive and
  send lanes so full-duplex operations do not require one unique whole-socket
  borrow; and
- stdout is not turned into a shared global lock. Parallel search gives one
  output owner to an aggregator or consumes it into one controller plus one
  exclusive port per worker.

The first-slice predicates are stated literally, because every first-slice type
permits shared borrows and that is therefore not what separates them. `Args`,
`DirectoryRead`, and `ExitStatus` are Sendable and Shareable, the last because
it is an immutable command code with no interior state. The command-lifetime
argv-backed `HostString` and `RelativePath` are Sendable and Shareable because
their backing is immutable and outlives the invocation; a later owned-backing
string type re-derives both predicates from its own representation and does not
inherit them. `ReadFile` and `Output` are Sendable and not Shareable. A file
cursor and output publication order therefore each have one mutable owner. A
future contract may add explicit lanes or consume `Output` into a publisher,
but does not retroactively make either original type shared.

Thread or task creation needs an explicit runtime input, but concurrency is not
merely another system-call family. Closure/resource capture and move, join and
trap propagation, atomics, the memory model, cancellation, and deterministic
failure are cross-cutting language semantics jointly owned by PAR-4 and
BOUND-1. OS threads and async tasks are not treated as interchangeable.

A scoped TaskGroup is one future candidate, not a selected runtime architecture.
Whatever model is selected must account for every child and resource before a
normal scope exit. Arbitrary asynchronous thread kill is not a baseline
operation. Async I/O uses the `PendingOp` ownership protocol above regardless of
the eventual task syntax.

WASI 0.2 pollables demonstrate a composition hazard, but Whitefoot's current
closed unit does not by itself rule out a simple readiness or wait-set design.
The future wait representation remains unselected. Its gate is that timers,
I/O, task join, and cancellation compose through every boundary Whitefoot
actually adopts, and that wakeup and owner lifetime are not private target
implementation details.

The active language has no function values, ordinary source stored borrows, shared-state memory
model, atomics, or task syntax. The first command slice does not pretend to
implement them; its resource contract is chosen so they need additive
concurrency semantics rather than replacement file or output APIs.

### 6.10 Target implementation, versioning, and ABI

Each system operation has one target-independent compiler-owned semantic ID.
Its gated semantic record binds signature, outcomes, ownership transitions,
memory and external effects, cleanup, and required target guarantees. The
checked IR carries only that semantic ID; it never recognizes a source function
name, path, project, test, or signature lookalike as a system operation.

That identity is not a naming rule and does not answer how a writer is
permitted to spell a system call. Which declaration domain admits these names
is a separate source-language question, selected in §11.1.

A separate target-qualification table maps
`(spec version, semantic ID, target, program kind)` to an approved
implementation version and private ABI symbol. A build fails when the mapping
is absent or incompatible. A target-code upgrade cannot silently change semantics;
a semantic change requires a new specification identity and compatibility
review. A fixed Rust enum and table are sufficient—no WIT parser, semver
registry, dynamic loader, or plugin protocol is required.

Command-lifetime argument backing is one such required target guarantee, bound
to the command entry and `arg_get`. A target qualifies only if it supplies
either stable native argv backing or the pre-entry snapshot described in §6.8.
A target that can supply neither fails qualification and the build fails; it
never silently invalidates the premise every derived lease depends on. This is
the enforcement point for that premise precisely because argument lifetime is a
property of the target, not of source: there is nothing in a program to check.

For the first macOS/Linux implementation, selection is static for the whole
build. A hot transfer lowers to required bounds/address checks, one direct
libc/host call, a count check, and a cold error mapper. Qualification must either
inline the compiler wrapper or show that any remaining call is immaterial.
There is no per-call vtable, handle-table lookup, operation-ID switch, target
tag, heap allocation, data copy, or global system lock.

The command and process-isolated service specializations need no instance handle
table: a trap terminates the owning process, while normal compiler cleanup uses
direct opaque native values. If a later language amendment permits contained
in-process traps, that profile registers resource acquisition and pending
submission in a per-instance reaper. Registration stays outside each
synchronous transfer hot path, must not use one global system lock, and
receives its own cost gate. Unselected containment machinery never taxes command
reads.

A deterministic test implementation for the first slice supplies only the arguments,
files, short reads, partial writes, redirects, and failures needed by its
contract tests. Later slices extend it only with their own operations. This is
not a general simulator or artifact-replay framework. Target code and the runtime
remain in the TCB; conformance and hostile tests provide evidence and catch
regressions but do not prove their honesty.

A WASI target can later implement this model's resource, effect, ownership, and
lowering rules, and can qualify for the operations whose semantics it is able to
supply. It does not currently qualify for the lossless host-string and path
operations: under §6.8's code-unit families, WASI's string domain cannot
represent every native filename. That is a qualification failure for those
operations, not a licence to narrow them. Whitefoot source semantics do not
inherit a target's limitation.

## 7. Protocol exemplars

These state machines test whether the cross-cutting rules are real. Entry/path,
ReadFile, and Output enter the initial implementation; TCP and Child
remain design exemplars.

### 7.1 Entry snapshots, HostString, RelativePath, and DirectoryRead

- `Args` is an immutable entry value. `args_count(&Args)` only borrows it.
  `arg_get(&Args, index)` leaves it live and returns one inline opaque
  `HostString` lease without allocation or byte copying; `InvalidIndex` returns
  no value. Native backing, or one pre-entry snapshot, remains valid for the
  whole command invocation.
- `HostString` refers to immutable target-native code units. Length and copy
  operations borrow it, in a lossless byte form and a text form. A successful
  copy changes exactly the requested destination prefix and leaves the rest of
  the buffer unchanged; every recoverable failure leaves the whole buffer
  unchanged. Both copy operations first validate `offset` and `capacity`, so an
  overflow or out-of-bounds range traps before reading the HostString or
  writing the destination. `host_copy_bytes` then copies the native bytes with
  no further failure mode beyond `CopyTooSmall(required)`. `host_copy_utf8`
  instead validates and measures the encoding, returns `Utf8CopyInvalid` or
  `Utf8CopyTooSmall(required)` without a write, and only then copies the
  complete encoding.
- `relative_path(move HostString)` consumes the lease on success and error.
  Success validates and retypes the same inline representation as
  `RelativePath`; `PathInvalid` returns no value. Neither path allocates.
- `open_read(&DirectoryRead, &RelativePath)` uses scoped shared borrows, so both
  inputs remain live on success and error. Success creates one `ReadFile`;
  `IoError` creates none. The `command.cwd` instance is process-equivalent and
  shareable for open operations; it is not a confinement claim.
- `DirectoryRead`'s own contract is short because its machine is. It is live
  from its entry binding until release, with no other transition: the first
  slice defines no attenuation, duplicate, or split operation, so no other
  state is reachable. Opening a file creates an independent `ReadFile` with its
  own cursor domain and does not alias the capability; two `DirectoryRead`
  values may denote the same directory object, and nothing infers separateness
  from a native handle. It is release-complete on ReadFile's ground — losing a
  close diagnostic on a read-only directory capability cannot invalidate an
  already opened file or promise durability — and the first slice exposes no
  explicit close. Any number of `open_read` calls may progress concurrently
  through shared borrows of one `DirectoryRead`, exposing no ordering relative
  to one another: each either creates its own `ReadFile` or fails, and none
  observes another's effect. That is the pair §10.2 step 1 depends on. A target
  that cannot open relative to a directory capability fails qualification
  rather than emulating it through an ambient cwd.
- The first slice fixes exactly one `HostString`, backed by the command-lifetime
  argv snapshot and released by logical consume with no target call; `Args` and
  `RelativePath` release the same way. A producer with separately owned backing
  does not reuse this type. It introduces a distinct owned-backing string
  resource with its own release action and its own family contract, because
  storage class is a function of type and one type cannot carry two release
  actions. Any conversion between the two is a later slice's explicit
  operation, never an implicit retype. Releasing `DirectoryRead` follows its
  resource contract. None stores an ordinary source borrow or needs a runtime
  handle-table lookup.

### 7.2 ReadFile

- `open_read(&DirectoryRead, &RelativePath)` creates `ReadFile(Open)` with a
  cursor domain and a conservative filesystem-object alias domain. A separate
  open does not prove a separate object; no duplicate operation exists in the
  first slice.
- `read_once(&uniq file, &uniq buffer, range)` is call-scoped. `ReadBytes(n)`
  leaves both owners live, changes exactly the first `n` requested bytes, and
  advances the cursor by `n`; `ReadEnd` changes no byte; `ReadFailed` changes no
  byte, because an attempt that made progress reports it as `ReadBytes`
  instead. The first regular-file primitive never hides a second attempt.
- A later positioned-read lane observes the object without changing this
  cursor. Multiple lanes are allowed only through an operation whose contract
  creates and owns them; sharing the handle is not inferred.
- ReadFile is release-complete. Normal compiler release consumes it and may
  discard only a close diagnostic that carries no read or durability guarantee.
  The first slice has no separate explicit-close operation. A later consuming
  close may expose its diagnostic, but it must consume the owner on every
  outcome and may not change derived-release semantics. Whole-process abort
  relies on OS teardown; only a separately selected contained-instance amendment
  would quiesce pending lanes and close the native object through a runtime
  reaper.

### 7.3 Output, stdout, and stderr

- `command.stdout` and `command.stderr` supply separate affine `Output` owners.
  Sequential writes across both preserve source order under §6.6's blanket
  external-call ordering rule, not through any aliasing analysis. Checked IR
  additionally records the conservative fact that redirection may make the two
  owners the same sink; that record has no first-slice consumer and is retained
  so a later verified cross-resource reordering fact fails closed on this pair.
- `write_once(&uniq Output, bytes, range)` performs at most one host output
  attempt. Calls made sequentially across either owner preserve source order.
  A successful count means that prefix was accepted by the host operation; it
  promises neither line atomicity nor storage durability.
- The first target implementation adds no hidden userspace buffering, so every
  failure the host write itself reports reaches `write_once`. `Output` is
  therefore release-complete: compiler release only detaches the source
  capability and reports nothing; it does not close or flush host
  stdout/stderr, and whole-process teardown later closes the native
  descriptors. A failure that a host surfaces only at descriptor close or
  writeback — delayed allocation, NFS, a late `ENOSPC` — is outside the first
  slice's error model and can be lost, so a redirected `wfgrep` may exit 0
  after a failed writeback. That is a stated limitation under §6.4 item 8, not
  a silently weakened guarantee. A later buffered or durable Output wrapper is
  completion-required and must expose flush/finish rather than inherit this
  policy; that is the named path for strengthening the guarantee.
- On macOS/Linux command targets, bootstrap owns the process and installs the
  command kind's ignored-SIGPIPE disposition before entry, so a broken pipe reaches
  `write_once` as BrokenPipe without adding per-write signal operations. A
  hosted or service program must receive an equivalent host guarantee or use a
  separately costed target implementation; it may not silently change the
  surrounding host process's signal policy.
- Sequential `wfgrep` formats into one reusable `OutputBatch` and calls
  `write_once` only for full or terminal batches. Partial writes are handled by
  an ordinary `write_all` loop; recycling changes only the logical used length
  and never clears or reallocates the buffer.
- Concurrent publication has no implicit order. A future
  `publication_new(move output)` consumes the original owner. A linear builder
  creates one exclusive SPSC port and two reusable batches per worker, then a
  completion-required controller becomes the sole `Output` owner. Workers move
  full batches through their own ports; the controller performs ordered writes
  and returns empty batches. The default order is completion order. A distinct
  ordered publisher pays the storage and head-of-line cost of file order.
- This shape pays synchronization and normally one host write per batch, not per
  match. It has no shared output lock, per-write reference count, or mandatory
  whole-output materialization.
- Terminal control, color, and console modes are separate capabilities. A trap
  diagnostic uses a mandatory runtime channel, never flushes ordinary `Output`,
  and cannot be used by source code.

### 7.4 TCP, split lanes, and pending receive

- A TCP resource moves through fixed states such as Unbound, Bound, Listening
  or Connecting, Connected, and Consumed. Unsupported target transitions fail
  rather than emulate weaker semantics.
- `split(Connected)` consumes the whole stream and returns linked affine RxHalf
  and TxHalf lane owners. This permits one receive and one send to progress
  concurrently without sharing one mutable whole-stream handle.
- Both halves carry a compiler-owned connection identity. It is not a static
  type distinction, since every RxHalf and TxHalf share their types, so
  `reunite` compares the identity at runtime and reports a mismatch as an
  ordinary outcome: a future
  `reunite(move rx, move tx) -> Result<Connected, ReuniteError>`, with
  `ReuniteError { ReuniteMismatch(rx: RxHalf, tx: TxHalf); }`, returns both
  halves unchanged and live when they do not match, so no owner is lost on
  either outcome. That outcome type is exemplar design and does not enter
  §6.7's first-slice inventory; it shows the inventory's two-outcome rule
  extending to a multi-owner failure. Connection-wide operations either consume
  a reunited owner or are explicitly listed on the half types; split never
  erases this decision.
- `start_receive(RxHalf, BufferLease)` returns either both owners on submission
  failure or `PendingReceive`, which exclusively owns the lane, buffer,
  registration, and connection-core reference. Terminal completion returns the
  RxHalf, BufferLease, progress, and exactly one of peer-end, error, or
  cancellation acknowledgement. Timeout is a higher-level timer/wait race whose
  winning branch requests cancel-and-reap; it is not built into receive.
- `finish_send(TxHalf)` is graceful half-close and consumes that half on every
  outcome. Dropping the last halves is best-effort connection release and may be
  abortive; it is not a synonym for graceful shutdown. Peer EOF, reset, partial
  send, local close, whole-process abort, and any future contained-instance
  reclaim are distinct transitions.
- UDP uses a separate datagram contract: zero-length datagrams are data,
  receive reports truncation explicitly, and a send is one datagram rather than
  a byte-stream prefix.

### 7.5 Child process

- `ProcessSpawn` access selects an executable relative to an explicit
  executable or namespace capability. Arguments, environment, cwd, and each
  stdio transfer are explicit. No ambient fd/handle inheritance occurs. Signal
  mask and disposition are also explicit child-start state: the portable default
  resets SIGPIPE and other resettable dispositions for the child; inheritance
  requires a separately named spawn option and target support.
- Spawn either returns a stable Child resource, not a reusable PID, or fails and
  returns every transferred stdio owner unchanged, so a failed spawn loses
  nothing the caller handed it. Its states are Running, ExitedUnreaped, and
  Consumed/Reaped; optional stdin/stdout/stderr pipes are separate owners with
  declared alias and cleanup rules.
- Child is completion-required. Normal code must wait/reap or explicitly detach
  where the program kind permits it. Kill requires a separate right and is only
  a termination request: it retains the Child owner and reports whether the
  request was issued, was unnecessary because exit was already observed, or
  failed. Wait/reap consumes Child and returns terminal exit. Detach also
  consumes Child, returns no exit status, and transfers eventual reaping/orphan
  responsibility to the declared runtime/program-kind policy. Whole-process abort
  performs no language wait/reap and does not promise that the child is killed
  or rolled back; the target's documented child-orphan policy applies. A future
  contained-instance amendment would need an explicit reaper policy. Wait and
  pipe drainage are not independent operations: a blocking wait may not progress
  concurrently with an undrained output pipe owned by the same program, because
  once the pipe buffer fills the child blocks on write and the parent blocks on
  wait. A real contract must therefore either provide a non-blocking or
  multiplexed wait, or require the pipes to be drained or released first. The
  first slice implements none of them.
- Signals are typed event or cancellation inputs, not arbitrary asynchronous
  handlers. Broken pipe is an Output error. A higher-level retry helper may
  retry Interrupted only when it is not suppressing an observable signal,
  timeout, or cancellation event.

## 8. Capability-family map

| Family | Entry inputs, values, and resources | Architectural rule | First implementation |
|---|---|---|---|
| Command context | exact standard inputs; Args, HostString, Environment, Stdin, stdout, stderr | snapshots are distinct from live streams; normal status is entry return | Args, HostString, stdout/stderr `Output`, `ExitStatus` |
| Filesystem | namespace roots, DirectoryRead/DirectoryMutate, files, iterators, mappings, locks | lossless paths, explicit containment type, operation-specific mutation/durability, one contract per resource | `command.cwd` as DirectoryRead, RelativePath, relative open, ReadFile |
| Clocks and timers | SystemClock or MonotonicClock; Timer/Pending | wall and monotonic time remain distinct; shared clock reads or timer creation do not imply one mutable cursor | none; later additive interface |
| Randomness | SecureRandom, InsecureRandom, RuntimeSeed | secure streams remain unique stateful resources; a seeded deterministic generator is an ordinary value | none; later additive interface |
| Network | Network/Resolver grants; TCP states, UDP, listener, split lanes | protocol state machines, stream/datagram distinction, backpressure, pending ownership, half-close | none; TCP exemplar closes substrate decisions |
| Wait and cancellation | PendingOp, timer, shutdown root, per-task watch | cancel is request; one terminal outcome; quiescence before owner return; one owned watch per task | none; wait representation unselected |
| Threads | worker input; JoinHandle or selected scope resource | explicit capture/move, memory model, join, trap, and failure semantics | none; PAR-4 co-design required |
| Async tasks | task-group and completion resources | not interchangeable with OS threads; continuation owns suspended state | none; task syntax/runtime unselected |
| Child processes | ProcessSpawn; Child and explicit pipe resources | capability-relative executable/cwd, no implicit inheritance, stable identity, mandatory reap/detach disposition | none; Child exemplar exercises owner accounting and names the wait/pipe hazard, and does not close the family's substrate decisions |
| Signals | typed signal-event grant | event stream or cancellation input; no arbitrary async source handler | none; later additive interface |
| Local IPC | explicit local namespace grant; pipe/local socket/shared-memory resource | family-specific stream, datagram, alias, and shared-memory rules | none; later |
| Memory mapping | Directory/File plus mapping support; Mapping | mapping is provenance root; invalidation, faults, dirty state, and sync are explicit | none; later |
| Target/device | future embedded exact inputs; typed device resources | target-qualified semantics, never a generic syscall escape | none; embedded program kind remains reserved |
| General FFI | separate gated foreign capability | opaque ABI, callbacks, loading, and foreign threads are BOUND-2 | outside this design |

Architectural membership does not promise simultaneous implementation. A later
family may add operations and resource types, but it must use the same explicit
access, owner-accounting, effect, process-abort, future-containment, and target-
qualification rules.

## 9. Exact first command slice

The first implementation target is deliberately small but not provisional. Its
semantic operations are:

| Item | Exact semantic shape | Ownership / effects |
|---|---|---|
| command entry | exact bindings: `command.args` as Args, `command.cwd` as DirectoryRead, `command.stdout` and `.stderr` as Output; returns `ExitStatus` | inputs are independent; language cleanup runs on the return edge, and the target then maps the returned `ExitStatus` to the process status; trap returns no status |
| `args_count` | shared-borrowed immutable Args snapshot → `u64` count | Args remains; snapshot read; no live host event |
| `arg_get` | shared-borrowed Args + index → `Result<HostString, ArgError>` | Args remains; no allocation or byte copy; the command-lifetime backing remains; no external effect |
| `host_bytes_len` | shared-borrowed HostString → `u64` native byte length | total, with no failure outcome; HostString remains; no allocation, copy, or external effect |
| `host_copy_bytes` | shared-borrowed HostString + unique initialized buffer + range → `Result<u64, CopyError>` carrying the exact byte length | the lossless route: no validation and no Unicode restriction; invalid range traps before reading source or writing destination; `CopyTooSmall(required)` leaves the whole buffer unchanged; owners remain |
| `host_utf8_len` | shared-borrowed HostString → `Result<u64, Utf8Error>` encoded length | text-facing use only; HostString remains; no allocation or external event |
| `host_copy_utf8` | shared-borrowed HostString + unique initialized buffer + range → `Result<u64, Utf8CopyError>` carrying the exact encoded length | text-facing use only; invalid range traps before reading source or writing destination; success copies the entire encoding and leaves the rest unchanged; recoverable failures leave the whole buffer unchanged; owners remain |
| `relative_path` | consuming HostString → `Result<RelativePath, PathError>`; rejects NUL and absolute/target prefixes, preserves and accepts `.` and `..` components | success validates and retypes the inline lease; failure consumes it; no allocation or copy; first-slice open follows native links and may escape above the initial cwd within the process namespace |
| `open_read` | shared-borrowed DirectoryRead + shared-borrowed RelativePath → `Result<ReadFile, IoError>` | inputs remain; success acquires one resource; `external, blocks`; no ambient cwd lookup |
| `read_once` | unique ReadFile + unique initialized buffer + range → `ReadOutcome` | at most one host attempt; advances cursor; `external, blocks, traps` plus ordinary region effects; owners remain |
| `write_once` | unique Output + byte view + range → `Result<u64, IoError>` | at most one host attempt; fixes output order; `external, blocks, traps` plus ordinary region effects; owners remain |
| `exit_status` | `u8` code → `own ExitStatus` | total and pure; every `u8` is a valid command code, so there is no failure outcome; no allocation, host call, or external effect |
| `ExitStatus` | portable command code 0–255; wfgrep uses 0, 1, 2 | target maps it exactly; startup failure and trap are outside it |

The raw byte pair is what keeps §3.3's promise that process arguments are not
silently restricted to Unicode. Unix targets already preserve arbitrary non-NUL
bytes, so withholding the bytes would have been an omission in the operation
set rather than a representation limit, and a pattern or path argument that is
not valid UTF-8 would have been unobtainable by source. It is also what makes
§12.2's required non-text argument and path test executable; UTF-8 conversion
remains available for diagnostics and for the Windows 16-bit path, but it is no
longer the only route to a HostString's contents.

`ExitStatus` is an opaque type with one total constructor rather than a bare
`u8` alias. There are no implicit conversions, so without a stated constructor
the entry's return value would be unwritable; keeping the type distinct also
prevents an arbitrary integer from being returned as a command status, and
matches how every other first-slice type is fixed.

Compiler-derived release is also an exact semantic operation:

| Resource | Consuming release action and exact effect |
|---|---|
| Args | logical consume; no host call or external effect |
| HostString / RelativePath | logical consume of an inline lease; no host call or external effect |
| DirectoryRead | at most one native close attempt; `external, blocks`; discard only the close diagnostic and never retry an ambiguous fd close |
| ReadFile | one native close attempt; `external, blocks`; discard only the close diagnostic and never retry an ambiguous fd close |
| Output | logical source detach; no close, flush, target call, or external effect; OS process teardown owns descriptor close |
| ExitStatus | logical consume; no host call or external effect |

Environment, stdin, directory enumeration, absolute namespaces, file mutation,
buffered output, async, network, and workers are later additive imports and
operations. None requires replacing the operations above.

### 9.1 Native cost shape to verify

| Path | Required native shape | Evidence still required |
|---|---|---|
| target selection | one link-time table decision | inspect checked IR and final symbols; no runtime operation switch or target tag |
| selected argument | one inline pointer/length lease over immutable command backing | verify no allocation, byte copy, or handle lookup in `arg_get` |
| raw argument bytes | length pass plus caller-buffer byte copy | verify no validation, allocation, or Unicode rejection on the Unix path; a non-UTF-8 argument reaches source unchanged |
| UTF-8 text conversion | length pass plus caller-buffer encode/copy only for arguments used as text | inspect Unix fast path and Windows conversion; invalid text is explicit |
| RelativePath construction | validation and type transition over consumed HostString lease | verify no allocation/copy and no exposed native units |
| `open_read` | one direct compiler wrapper or intrinsic and one native open-relative operation | inspect call path and target error mapping |
| `read_once` / `write_once` | bounds/address checks, at most one host transfer, count check, cold error mapping | inspect generated code; no material wrapper, allocations, copies, lookups, or locks |
| DirectoryRead/ReadFile release | at most one direct native close attempt | verify `external, blocks` in every enclosing exact row; never retry an ambiguous fd close |
| Args/HostString/RelativePath/ExitStatus release | logical consume only | verify no external event, target call, handle lookup, or byte copy |
| Output release | logical detach only; no close or flush | verify no external event/target call and that OS process teardown owns native descriptor close; record that a failure surfaced only at close or writeback is outside this slice's error model |
| output batching | one reusable fixed buffer; normally one host write per full batch | count calls and copies; reject syscall-per-match and buffer reinitialization after flush |
| buffer initialization reuse | one initialization on allocation, then reuse across reads | compare against an equivalent Whitefoot loop; reject any per-read re-initialization or reallocation after flush |
| initialization cost | the one-time fill at allocation is not material to steady-state throughput | compare steady-state throughput with buffer reuse against an equivalent native read loop over *uninitialized* storage, counting the one-time fill at allocation; stop for a separately proved initialization model only on a material loss |

These are gates, not claims that LTO or the OS will remove the cost. The two
buffer rows need different controls and answer different questions: the first is
a structural check that initialization happens once, and an initialized control
answers it; the second asks whether paying initialization at all is material,
which only an uninitialized control can answer, because an initialized control
carries the same cost on both sides and cancels it. The remaining `material`
judgments here and in §6.10 are structural inspections rather than quantitative
gates, and carry no threshold by design.

The macOS/Linux command bootstrap performs its one-time SIGPIPE normalization
before timing the source entry body, but end-to-end wfgrep measurement includes
bootstrap. The emitted hot `write_once` path still contains no per-call signal
mask operation. Hosted/service targets require separate qualification.

## 10. Witness traces

### 10.1 Sequential wfgrep

1. The command entry requests exactly Args, `command.cwd`, stdout, and stderr as
   independent inputs with reusable types.
2. `arg_get` creates zero-copy HostString leases. The pattern is copied into
   Whitefoot storage as raw bytes with `host_copy_bytes`, which performs no
   validation and imposes no Unicode restriction, so a pattern that is not
   valid UTF-8 is searched exactly as given; `relative_path` validates and
   retypes a path lease without copying, and DirectoryRead opens it as
   ReadFile.
3. wfgrep reuses one initialized caller-owned buffer. `read_once` changes only
   the returned prefix, reports a short count separately from end, and advances
   the cursor by exactly that count.
4. Matching consumes only the returned prefix. Boundary overlap remains in
   ordinary Whitefoot storage.
5. Matching and formatting append to one reusable output batch. Full or final
   batches use `write_once`; a library loop handles partial writes. This avoids
   one syscall per match while keeping every flush failure explicit.
6. The entry returns `ExitStatus` 0, 1, or 2; language cleanup runs on that
   return edge, and the target then maps the returned status to the process
   status.

No whole-file allocation, per-byte call, Unicode conversion of any argument or
path, raw fd, hidden global, per-argument allocation, or temporary Process API
is required.

### 10.2 Parallel file search

1. Later directory traversal creates owned path jobs; workers use a scoped
   shared DirectoryRead factory to open independent ReadFile resources. This
   step has three explicit dependencies. It needs `DirectoryRead: Shareable`,
   which §6.9 grants; a scoped parallel construct whose join precedes the
   shared borrow's scope exit, where no suspension is involved so the
   applicable gate is step 4's scope and join accounting rather than §12.2's
   borrow-across-suspension gate; and the backing-lifetime decision §6.8 leaves
   open, because enumeration is exactly the non-command-lifetime producer that
   premise excludes. The path-job records face two separate obstacles. They
   need affine-element storage to be representable at all, and under §6.8
   candidate (a) they additionally become region-bearing, which STOR-5 forbids
   from any struct field, enum payload, or buffer element; candidate (b)
   instead makes the names resources with a release action, changing what an
   owned path job is and adding release effects to the traversal's rows.
2. Each ReadFile and buffer moves to one worker through a checked declared
   parallel construct. Cursorful handles are not shared.
3. The sequential Output owner is consumed into a controller. Each worker owns
   one SPSC port and two reusable output batches, moving full batches without
   copying bytes or sharing `&uniq Output`. The default publisher emits in
   completion order; deterministic file order is an explicitly costed variant.
4. The selected join/scope mechanism accounts for every task, owner, and
   deterministic failure before normal exit.

The system API survives this step, but task syntax, affine-element storage, the
Sendable/Shareable judgments for the types this witness introduces beyond the
seven fixed ones — path-job records, per-worker SPSC ports and output batches,
and the workers themselves — the scoped shared-capability borrow in step 1,
memory model, worker implementation, and failure selection remain explicit
PAR-4 work. The seven first-slice types are not among them; §6.9 states all
seven. The backing-lifetime rule is not among them: it is a BOUND-1
source-language decision that PAR-4 cannot absorb, and it gates this witness
independently of every item in that list.

### 10.3 Cancellable network service

1. One process-owned service invocation receives its listener, monotonic clock,
   shutdown root, and worker input. Repeated host callbacks are not this program
   kind.
2. Accept yields owned TCP resources. `split` creates Rx/Tx lane owners; each
   connection and owned buffer lease moves into the selected task construct.
3. The shutdown root creates one owned watch per task, and the clock creates an
   owned timer before suspension; no task retains an ordinary parent borrow.
4. A pending receive owns its Rx lane, buffer, registration, and core reference.
   Submit failure returns them; completion or cancel-and-reap returns them only
   after quiescence.
5. A timer races receive through the future selected wait model. Exactly one
   terminal result owns partial progress and every submitted owner.
6. Matching live Rx/Tx halves can reunite for connection-wide operations.
   Half-close, last-owner release, task failure, and whole-process abort are
   distinct transitions. The baseline service is process-isolated; any future
   host-surviving containment must satisfy the separate quiescence gate.

This witness fails any design that retains statement-scoped borrows across
suspension, uses one Process holder, frees a cancelled buffer before quiescence,
or leaves wakeup lifetime private to target code.

## 11. Required language and compiler deltas

The exact first slice requires:

- a command entry form with exact standard input labels, independent typed
  parameters, and `ExitStatus`, raising the current FN-7 ceiling. The
  unlabelled `fn main() -> own unit` entry remains admissible: a unit that
  requests no standard inputs keeps it and therefore declares no program kind.
  That is what makes the kindless population non-empty by construction, which
  is the premise §11.1's selection rests on. Every entry in the active corpus is
  currently unlabelled, which illustrates that population rather than
  establishing it;
- fixed opaque Args, HostString, DirectoryRead, RelativePath, ReadFile, Output,
  and ExitStatus types, including the compiler-owned immutable-backing lease
  rule for HostString and RelativePath;
- exactly one first-slice string type: HostString is backed by the
  command-lifetime argv snapshot, and a producer with separately owned backing
  introduces a distinct resource type rather than changing this one's release;
- release-complete compiler-derived cleanup on every normal edge;
- an EFF-2 extension: effect attribution becomes the union of syntactic body
  occurrences and compiler-derived release on normal control-flow edges, so a
  legal row depends on owner disposition as well as written calls. Existing
  `buffer`, `box`, and arena releases are not retrofitted;
- an EFF-1 row-grammar extension: the two categories join the row grammar and
  take a defined position in its mandatory canonical order;
- a STOR-3 extension: compiler-derived release becomes per-type table data
  rather than the rule's fixed enumeration of memory-reclamation actions, and
  one admitted action may perform a host call carrying an external effect. This
  is the premise the EFF-2 extension attributes — there is no release effect to
  attribute until a release action exists that has one. The no-finalizers
  clause is scoped rather than removed: it continues to forbid
  writer-registered destructors, which §6.5 preserves, and does not forbid a
  compiler-owned release action fixed by a resource family's contract;
- an FN-3 normalization extension: contract-conformance equality currently
  normalizes a row to four capabilities, so two rows differing only in
  `external` or `blocks` compare equal. The normalization must compare the new
  categories the way it compares `traps`, by presence, or `fn_sig` members must
  be forbidden from carrying them. Without one of the two, a `pure` member can
  bind an externally effectful function, which §12.2 lists as an
  architecture-rejecting condition;
- the exact operation-specific outcomes and portable error class above;
- exact `external` and `blocks` effects distinct from memory effects, without
  parameterized resource-origin syntax;
- a declaration home for the compiler-owned system types and operations. The
  active rules admit an IDENT callee only as an operation-table family or a
  top-level source function, and admit a nominal type or constructor only from
  a source declaration or the prelude, so the slice's opaque types, outcome
  constructors, and operation names currently have no domain. §11.1 states the
  selection, its amendment surface, and the owner fork it depends on;
- compiler-owned system-operation IDs plus a static target-qualification table
  and direct native lowering, including command-lifetime argument backing as a
  required target guarantee so a target that cannot supply it fails
  qualification; and
- checked IR resource/effect identities, preservation and cleanup of opaque
  backing leases across move/match/store/return/call, and first-slice
  conformance tests. That checked-IR retention serves auditing and lowering; it
  is not the rule that refuses a dangling lease, because a fact recorded about
  an accepted program cannot reject one.

The nine active-specification rules named in §2 have these dispositions. EFF-1,
FN-3, and STOR-3 are extensions the specification proposal must write, listed
above; none is satisfied by any other change here, FN-3 is the one whose
omission would silently admit the program §12.2 rejects, and STOR-3 is the one
the EFF-2 extension depends on. OP-1 gains a third admitted callee source under
the selection in §11.1, and PRE-1 is deliberately left untouched there. ERR-2 is
the reason a wide portable error class is matched through one shared status
helper rather than at every site, and the reason §6.7 gives each operation its
own outcome type instead of one shared union. TYPE-6 takes two dispositions.
Its whole-unit constructor uniqueness is satisfied without amendment:
two-outcome operations reuse the prelude `Result`, and the one multi-outcome
operation gets operation-prefixed variant spellings, so the slice's nine new
constructor names collide with nothing. Its declaration-domain rows separately
gain a third admitted source under the selection in §11.1. The reuse of
`slice<'r, T>` is rejected in §5 for the reason recorded there.

STOR-5 is the one rule left open, deliberately. It does not see an opaque
backing lease, because such a lease is neither a borrow nor region-bearing, so
nothing in the active language forbids storing one. The first slice is sound
regardless, because §6.8's command-lifetime premise makes the backing outlive
every derived value and §6.10 enforces that premise at target qualification.
No source-level rule enforces backing lifetime in the first slice, and none is
needed there. Selecting one is required before any producer outside that
premise is added; §6.8 names the two candidates and the choice is a
source-language decision, not something later machinery can absorb.

The first slice does not require completion-required checking, writer-visible
stored borrows,
partial initialization, D17 representation proofs, mapping, async syntax, wait,
cancellation, atomics, threads, function values, revocation, dynamic loading,
callbacks, or general FFI. If initialized-buffer cost is material, work stops
for a separately proved initialization model before wfgrep grows around it. That
condition triggers on the §9.1 initialization-cost row, whose control is an
equivalent native read loop over uninitialized storage.

Later families have named protocol obligations and exemplar compatibility, not
completed numbered-spec semantics. Their implementation remains conditional on
the project and on the separate concurrency decisions they actually need.

### 11.1 Declaration home for system types and operations

Two rules leave the slice's names homeless. OP-1 admits an IDENT callee only as
an operation-table family or a top-level source function; TYPE-6 admits a
nominal type or constructor only from a source declaration or the prelude. The
slice's operations and opaque types are neither, so every call in §1 and §9 is
currently an OP-1 rejection.

GRAM-11 settles one axis before the comparison starts. It partitions call
spelling by declaration domain: a callee resolving to a function takes named
arguments, while an operation-table callee takes positional unnamed operands.
Every system call written in this dossier uses named arguments, so the home
must be signature-shaped and the operation table is excluded by spelling rather
than by preference.

Three routes remain, at these amendment surfaces:

- **Route A — extend PRE-1.** TYPE-6's nominal and constructor domain rows are
  unchanged, because prelude members are already an admitted source; its
  lexical-IDENT callee row still gains one. Its "exactly twenty-four
  declaration records" sentence, the stated preorder, and the DIAG-1 payloads
  keyed to those ordinals all move, but no new collision rank or origin kind is
  needed: rank 4 absorbs the new members and the ordinals simply grow. PRE-1
  gains roughly fourteen nominals, thirty-nine constructors, and — new in kind
  — function signatures, which the prelude has never carried. OP-1 still needs a
  third admitted callee source. PROG-1, GATE-1, LEDGER-1, and the SCOPE rules
  are untouched. Standing cost: prelude members are visible throughout every
  unit, so those spellings are reserved even in a unit that declares no program
  kind.
- **Route B — LEDGER-1's gated boundary family.** Rejected. LEDGER-1 puts
  unsafe regions, FFI extern frames, and trusted primitive imports in one
  family sharing one obligation ledger, so system operations would share both
  with general FFI — which §3.3 forbids and the current plan makes binding.
  PROG-1's name-definition sentence is also touched, since its "only external
  boundary" clause is about boundaries rather than name sources. This route
  fails on LEDGER-1 alone; the SCOPE-1 and SCOPE-3 objections often raised
  against it do not hold cleanly and are not relied on here.
- **Route C — a distinct compiler-owned system-declaration domain**,
  signature-shaped, neither PRE-1 nor the gated family. TYPE-6 gains a third
  admitted source in three rows: nominal type, constructor, and the
  lexical-IDENT row that admits a callee. OP-1 gains a third admitted callee
  source. PROG-1 names a third source of language names. DIAG-1 needs a new
  collision rank and a new origin kind, because a user/system collision is
  neither rank 4 nor representable in its two-member origin sum. PRE-1, GATE-1,
  LEDGER-1, and the SCOPE rules are untouched.

Route C is selected, on one ground: it is the only route whose name visibility
can be conditional. Its domain is admitted only in a unit that syntactically
declares a program kind, so a unit that declares none reserves nothing, where
Route A reserves in that same unit. Both sides of the comparison therefore
range over one population, and §11's entry delta is what makes that population
non-empty: a unit requesting no standard inputs keeps the unlabelled entry and
declares no kind. The trigger must be the program-kind declaration alone and
not the entry's input types, because diagnostics admit names before resolution
and keying visibility on resolved types would be circular. That
conditional-visibility rule is new machinery, and it is the whole of Route C's
advantage — the FFI separation is not a ground, since Route A preserves it
equally.

The reservation cost Route C avoids is prospective, and the evidence should be
read at that strength. Route A's reservation set is sixty-four spellings:
fourteen nominals, thirty-nine constructors, and the eleven operation names,
which become prelude members and so exclude a user top-level function of the
same spelling. None of the sixty-four collides with anything in the current
active sources, so the cost falls on future writers and on the conformance
corpus rather than on code that exists today. The eleven operation names are
the part most likely to be wanted: they are ordinary lowercase verbs such as
`open_read` and `read_once`, unlike the prefixed constructor spellings.

That makes the selection an owner fork rather than a settled fact. If the owner
accepts the syntactic conditional-visibility mechanism, Route C stands. If the
owner declines to add that mechanism, the selection falls back to Route A
rather than to an unconditional Route C: without conditional visibility Route C
carries Route A's reservation cost and three more rule amendments, so Route A is
then strictly cheaper.

## 12. Cross-review results and implementation gates

### 12.1 Review results

The revised source model passed three independent reviews:

- **Semantic review — pass after simplification.** Existing `own`, `&`, and
  `&uniq` are sufficient; the category names need no source keywords. Replacing
  parameter-named external domains with `external` and `blocks` avoids a new
  resource-origin type calculus without weakening access control, ownership,
  purity boundaries, cleanup, or conservative call order. Final review also
  required trap-before-access range validation, one shared clock
  classification, exact release effects, and an explicit statement that
  writer-visible ReadFile close is later; those are closed above. The general
  opaque-backing lease rule it asked for is deliberately not closed: §6.8
  states the command-lifetime premise the first slice actually rests on and
  records a general rule as an open decision between two named candidates,
  because nothing below the acceptance contract can supply one.
- **Performance review — pass after two required changes.** `arg_get` now returns
  a zero-copy command-lifetime lease instead of allocating per argument.
  Sequential output uses reusable source-level batches, and the future parallel
  shape uses one exclusive SPSC port per worker plus one controller. Native hot
  operations must lower directly without dispatch or a material wrapper.
- **Evolution review — pass after narrowing the selected program kinds, not
  blanket approval of future
  APIs.** A service is one process-owned run loop; repeated host callbacks are a
  separate program kind. The entire embedded kind remains reserved until
  halt/reset, interrupt, device, DMA, memory, and pending-I/O cleanup are
  defined. A shutdown input creates one owned watch per task, clocks create
  owned timers before suspension, and split TCP halves preserve a connection
  link and may reunite.

The exact command-entry punctuation still belongs to the specification proposal.
The general backing-lifetime rule for string producers outside the
command-lifetime premise, task syntax, wait representation, complete
networking, and embedded teardown remain later designs. None is required to
implement or validate the first command slice.

### 12.2 Implementation rejection gates

The architecture is rejected if any witness requires:

- one host call per byte or field;
- a full input copy or materialization not required by the operation;
- a unique global context or centralized system lock on independent work;
- dynamic dispatch or handle-table lookup on the static native hot path;
- a Unicode-only conversion before native path access;
- a borrow retained across suspension without tracked ownership;
- resource cleanup or pending-operation reaping by writer convention;
- handle identity or target metadata used as a disjointness proof;
- a hidden external effect inside pure or memory-only rows; or
- a source-recognized primitive lookalike.

The first target slice must test empty, short, exact, multichunk and changing
files; non-text argument/path values; `..`, absolute, NUL, and symlink policy;
invalid/overflowing buffer ranges and no-call/no-write behavior; open/read
errors; short output and broken pipes; stdout/stderr redirected to one sink; an
output sink that fails only at close or writeback; normal, recoverable, and
fatal cleanup; close error and no-fd-retry behavior; effect omission and
addition, including a function consuming an owned `ReadFile` whose body is
exactly `return unit;`, which must declare `external, blocks` because its only
effect is the derived release on the return edge; primitive lookalikes; chunk
and host-call counts; peak storage; initialization; emitted calls; allocations,
copies, locks, and the absence of per-argument allocation and per-match output
calls.

## 13. Recommendation and owner decision

Cross-review supports exact entry inputs, immutable values, stateless factories,
unique stateful resources, explicit controller/port splitting, and static target
lowering. It rejects the smaller source models in section 5. This file remains a
candidate until the owner accepts it; it is not yet specification authority.

Owner acceptance would select:

1. exact standard entry inputs under a declared program kind, with a unit that
   requests no standard inputs declaring no kind and keeping the unlabelled
   entry — the population the item 5 fork turns on;
2. ordinary `own`, `&`, and `&uniq` over opaque values and resources, with
   compiler-owned resource contracts and three
   completion policies;
3. exact source effects `external` and `blocks`, conservative source ordering,
   and detailed identity/alias facts kept in checked IR;
4. operation-specific one-attempt I/O, lossless target paths, portable error
   classes, and the existing whole-process abort law for command and
   process-owned service programs;
5. target-independent system-operation IDs with static qualified native code,
   plus the §11.1 declaration home for the system names themselves. That item
   carries one open fork the owner must settle: accepting a new
   conditional-visibility mechanism selects the distinct compiler-owned domain,
   while declining it selects the prelude extension instead and reserves the
   system spellings in every program;
6. zero-copy command argument leases, reusable input/output buffers, and the
   exact first command slice and cost gates in section 9; and
7. later parallel sharing only through one of three explicit mechanisms — a
   family-defined split, a controller/port construction, or a shared borrow of
   a `Shareable` capability inside a scoped parallel region — never by making a
   cursor or Output implicitly shared.

The later wait/task syntax, thread memory model, complete network surface,
mutation/durability API, and general FFI are not smuggled into this decision.
Their designs must satisfy the selected substrate and exemplars, but they still
require their own project-driven specification proposals.

## 14. Stop condition

Stop this investigation after another hostile review confirms internal
consistency and the owner accepts or rejects the architecture. Do not edit the
numbered specification or compiler under this dossier. On acceptance, record a
live design-memory decision and replace the Current Plan with one exact
command-slice specification proposal and project-independent tests, then return
to the same wfgrep checkpoint.
