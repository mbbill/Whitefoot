# System-capability architecture — design dossier

Status: DRAFT FOR OWNER REVIEW. This is design evidence for active BOUND-1,
not a numbered specification, compiler interface, or implementation authority.

Date: 2026-08-04

## 1. Decision this dossier prepares

Select the architectural boundary through which Whitefoot programs interact
with a host. The selected model must let the first wfgrep command read arguments,
files, and output without committing the language to a process-shaped subset
that later has to be replaced for networking, cancellation, or parallel work.

The recommendation is:

> A program selects a lifecycle profile and declares an exact static capability
> import set. The host grants those independent typed capabilities at entry,
> and operations create unforgeable affine runtime resources. Source semantics,
> observable external effects, and the compiler-gated provider implementation
> are separate layers. Native builds bind a static provider and lower hot
> operations directly.

The architecture can be selected only when every important system family has a
deliberate place, every cross-cutting ownership question has one answer, and an
exact first slice can be stated without deferring an API-shaping choice to its
implementation. Implementation remains staged. The first implementation must
be a true subset of this model, not a provisional API.

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
  define runtime authority, resource rights, a provider ABI, or compiler
  implementation; and
- the backend always calls the Whitefoot main with no arguments and returns
  process status zero. Its only ordinary external I/O is a private
  write-to-stderr path used before aborting on a trap; allocation, release, and
  abort also cross the current compiler/runtime trust boundary.

Three concepts therefore must not be conflated:

1. Authority: which outside object or operation family a running value permits.
2. Effect: which observable outside event a call may perform and how it orders.
3. Provider trust: which compiler-owned implementation is allowed to realize
   that semantic operation on a target.

GATE-1 can control the third. It does not by itself solve the first two.

## 3. Requirements

### 3.1 Safety and authority

- Ordinary source cannot forge a resource, provider identity, system operation,
  right, or external-effect fact.
- A program begins with no implicit mutable authority. Its entry world names the
  host capabilities it requires.
- Runtime delegation is explicit movement or an approved narrowing operation.
  Duplication is never inferred from integer or handle copying.
- Every resource operation states ownership of inputs and outcomes, partial
  progress, recoverable failure, normal cleanup, and process-abort behavior.
- Resource handles do not grant arbitrary foreign memory access.

### 3.2 Performance

- The normal byte path has no per-byte host call, mandatory whole-input
  materialization, whole-input zero fill, avoidable full copy, centralized
  provider lock, or global I/O fence.
- Synchronous input admits caller-owned initialized storage. The provider writes
  only within the stated capacity and returns the exact valid prefix length.
- The architecture has places for positioned and vectored I/O, batching,
  mapping, splice or forwarding, and owned asynchronous buffers.
- A static native provider can lower an operation to a direct call or target
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
- A new provider or target cannot change source semantics merely because its OS
  API differs.
- General FFI remains separate from the compiler-owned system provider.

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
| Raw syscalls and integer fds in source | Thin native ABI and complete OS access | Forgeable identities, implicit global fd table, manual close, weak effect precision, poor Windows portability, and an unchecked pointer wall | Reject as source semantics; permitted only inside a gated provider |
| Ambient functions such as args, open, stdin, and spawn | Small call sites and familiar APIs | Hidden authority and inter-function channels contradict FN-7's no-global rationale; capability use is invisible in signatures and hard to attenuate, test, or parallelize | Reject |
| One permanently retained affine Process capability | Explicit entry authority and simple bootstrap | Every operation needs the same unique holder, falsely serializing files, stdout, networking, clocks, and workers; a shared implementation requires a central lock or hidden aliasing | Reject as an ongoing source resource; a private entry envelope consumed immediately into independent inputs is compatible with the recommendation |
| Literal WASI 0.2 or 0.3 source API | Existing modular taxonomy and portability work | Unicode-only paths, no guaranteed caller buffer or zero-copy path, async tied to Component Model costs, incomplete threads/process surface, and semantics chosen for cross-language components rather than Whitefoot ownership | Reject as the language contract; retain as a possible target provider |
| Exact entry imports plus typed runtime resources and static providers | Explicit authority, fine resource ownership, modular effects, target portability, and no required dynamic dispatch | Requires new resource cleanup, external-effect, entry, provider, and later async/thread semantics | Recommend, subject to the closure gates in this dossier |

## 6. Proposed architecture

### 6.1 Layering

The boundary has four layers:

1. Source semantics. Fixed system operation and resource identities, ownership,
   outcomes, effects, and entry worlds.
2. Checked capability IR. Target-independent operations with exact resource
   types, event domains, memory operands, and derived cleanup.
3. Target provider. A compiler-selected, gated implementation for a target and
   world. It may use libc, direct system calls, OS libraries, or a WASI host.
4. Host ABI. Private calling conventions, handle tables, trampolines, and OS
   details. None grants source authority by itself.

An optimization may change layers 2 through 4 only while preserving layer 1.
A provider table used for embedding or deterministic tests is an implementation
choice; native commands do not pay for it when statically bound.

### 6.2 Entry profiles and exact imports

An entry profile fixes lifecycle and ABI shape; an exact static import set says
which system capabilities this particular program requires. `optional` does
not mean that every command receives a bag containing clocks, network, and
workers. If a program declares an import, the provider must grant it at startup
or refuse to start the instance. Runtime `Option` is used only when absence is
part of the operation's domain semantics, not to hide an undeclared authority.

The initial profiles are:

- command: one entry invocation and one normal `ExitStatus` return;
- service: a long-lived instance whose exact listener, clock, cancellation,
  and client imports are named statically; the inbound call ABI remains a
  later BOUND-1 lifecycle decision rather than a general foreign callback;
- embedded: one target-qualified instance with only its declared device,
  clock, interrupt, and memory capabilities; and
- hosted library or arbitrary imported/exported foreign calls: BOUND-2, not a
  system-interface profile disguised as a world.

Normal command status is the return value of the entry function, not a
`Process` capability. Startup or provider-qualification failure happens before
entry and therefore is not a source-returned status. A trap is abnormal
instance termination and also does not return an `ExitStatus`.

The exact source spelling is a specification decision. Logically, entry
capabilities are independent parameters. An implementation may pass one private
ABI record, but it must consume or project it immediately; source code does not
retain one unique aggregate whose borrow serializes all operations.

Snapshot data and live authority are distinct. Arguments and the initial
environment are immutable invocation snapshots. Filesystem, output, clocks,
randomness, network, and workers are live capabilities.

### 6.3 Interface capabilities and runtime resources

Entry interfaces grant the ability to ask for an operation or create a
resource. Runtime resources identify individual live objects.

Examples:

- a directory capability can open a ReadFile resource;
- a network capability can create an UnboundTcp resource;
- a worker capability can create a TaskGroup or JoinHandle;
- a monotonic clock interface can create a Timer or future completion; and
- stdout is already one output resource.

Resources are opaque and unforgeable. Their source-level owner is affine.
Movement delegates the authority. Borrowing permits temporary use without
delegation. An explicit duplicate operation exists only when its exact sharing,
cursor, offset, cleanup, and external-alias semantics are defined.

Static resource types encode standard operation sets instead of exporting
arbitrary integer right masks: examples include DirectoryRead,
DirectoryMutate, ReadFile, WriteFile, TcpListener, TcpStream, and Output. The
set is a closed compiler-owned lattice for each family, not one nominal type per
OS flag combination. A host may impose a narrower dynamic policy and return
PermissionDenied; a program cannot widen it. Attenuation consumes the broader
owner and returns a standard narrower resource. Retaining both authorities
requires a separate family operation whose alias, cleanup, and sharing contract
is explicit; source never edits or copies an integer mask.

Revocation is not part of the first system interface. It requires shared state,
a concurrency memory model, and explicit stale-handle outcomes. The capability
map reserves it as later work instead of pretending affine movement solves it.

### 6.4 Resource protocol descriptors

Every compiler-owned resource family has one normative protocol descriptor.
Names in the capability map are not considered designed until that descriptor
states:

1. states and legal transitions;
2. the semantic object, cursor, lane, and ordering aliases created by open,
   duplicate, split, move, and attenuation;
3. the disposition of every resource and data owner on every outcome;
4. implicit release, explicit finish or abandon, whole-process abort, and any
   separately selected contained-instance behavior;
5. operation pairs that may progress concurrently and the ordering they expose;
6. pending-operation, cancellation, and quiescence rules where applicable;
7. exact external events, including compiler-inserted release; and
8. cross-platform guarantees that a provider may reject as unsupported but may
   never silently weaken.

The descriptor is compiler-owned semantic data. A native fd, handle value, or
provider table entry is never the identity or alias proof exposed to analysis.
The four exemplar descriptors in section 7 are closure tests for the shared
model, not promises to implement those families together.

### 6.5 Resource completion and cleanup

Resource protocols use one of three completion policies:

1. **release-complete** — compiler-derived release is the complete language
   obligation. A ReadFile is the first example: losing a native close
   diagnostic cannot invalidate already observed bytes or promise durability.
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

A consuming close or finish invalidates the source handle on success and error.
In particular, a provider may not retry a numeric POSIX fd after `close`
reports `EINTR`, because the descriptor may already be closed and reused.
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

User-defined destructors and arbitrary writer-defined resource protocols are
not required. The first design uses fixed compiler-owned resource families with
complete gated contracts.

### 6.6 External effect domains

External effects are not memory-region effects and are not authorization. The
design keeps three identities separate:

- an authority origin says why an operation is permitted;
- a semantic alias or ordering domain says which outside events may interact;
  and
- a runtime resource identity says which affine owner is being used.

The minimum semantic categories are:

- `observes(domain)`: obtains outside or nondeterministic state;
- `changes(domain)`: advances a cursor, emits output, changes namespace or
  protocol state, consumes a nondeterministic sequence, or releases a resource;
- `blocks(domain)`: an ordinary call may block its current host thread; its
  stack and call-scoped borrows remain live only for that call;
- `starts(domain)`: submits an operation that may progress after the call and
  returns an owner-accounted pending resource;
- `suspends(domain)`: suspends language execution with a continuation that owns
  all state retained across the suspension; and
- `spawns(domain)`: creates new language execution, distinct from starting
  kernel or provider I/O.

Cancellation is a change to a pending-operation domain; observing terminal
completion is another ordered event. One operation may have several categories.
A cursorful file read observes file contents, changes its cursor, and may block.
A positioned read does not change a cursor. Clock and random interfaces are
ordered observation streams: calls both observe and advance their sequence
domain, so swapping two calls cannot swap the values assigned by the program.

Rows remain exact and finite. Effects name resource or capability parameters.
A locally created resource projects its effects to the declared interface
parameter that created it until a later generative-domain calculus is selected.
This can be conservative, but it is nameable in a signature and cannot make an
open-read-close wrapper appear pure.

Sequential external calls remain in source program order, including calls on
different domains. Event domains initially support exact auditing, wrapper
projection, and declared-concurrency checks; they do not authorize automatic
external-call reordering. This conservative rule is not a global runtime lock:
explicit workers and independently owned resources can still execute
concurrently. A later verified disjointness fact may enable an optimization,
but facts-off compilation remains correct and accepted programs do not change.

Resource handle values, fd numbers, separate opens, or provider metadata never
prove external disjointness. Duplicate and split operations preserve explicit
alias relations. Independently opened paths may still meet through hard links,
mounts, devices, or redirection. Stdout and stderr conservatively share one
command-output ordering domain. A provider relation can improve lowering only
through a separately verified fact channel; it cannot alter source semantics or
acceptance.

Pure excludes every external category. Whether the eventual specification uses
these spellings is secondary to preserving these distinctions.

### 6.7 Data transfer and portable errors

The architecture uses operation-specific outcomes rather than forcing every
hot I/O operation through one large tagged union or scheduler.

The synchronous stream/file primitives are one-attempt operations:

    read-once(&uniq resource, &uniq initialized-buffer, checked-range)
        -> bytes(count) | end | error(io-error)

    write-once(&uniq resource, byte-view, checked-range)
        -> bytes(count) | error(io-error)

Both resource and buffer owners remain with the caller because the synchronous
operation holds only call-scoped borrows. On a read result, exactly `count`
bytes at the start of the requested range may have changed and the remaining
buffer is unchanged. The cursor advances by exactly `count`. A short success is
not EOF; only `end` states that no byte was available at the observed end. A
provider returning a count outside the checked range is a trusted-provider
violation, not a value the source program must defend against.

For a zero-length range, both operations return `bytes(0)` without issuing a
host transfer; a zero-length read is never reported as `end`. For a nonempty
range, read returns `bytes(n)` only for `n > 0`, and write never returns
`bytes(0)`: a host zero-write is `IoError(WriteZero)`. An error result from these
first primitives leaves the initialized buffer unchanged.

One source `read-once` or `write-once` maps to at most one provider-issued host
transfer attempt. If that attempt reports progress, the provider returns it
immediately; it does not hide a later error by looping. A reported interruption
is returned as Interrupted. `read_exact`, `write_all`, and retry policy are
ordinary library loops that can inspect cancellation or signal state and
accumulate `(progress, terminal reason)` themselves.
Stream zero progress, datagram zero length, closed peers, `would_block`, and UDP
truncation are family-specific outcomes fixed by their descriptors, not guessed
from a shared count convention.

`IoError` has this closed portable class set: NotFound, PermissionDenied,
AlreadyExists, NotDirectory, IsDirectory, DirectoryNotEmpty, ReadOnly,
ResourceBusy, InvalidInput, InvalidPath, Unsupported, Interrupted, WouldBlock,
TimedOut, BrokenPipe, WriteZero, UnexpectedEnd, ConnectionRefused,
ConnectionReset, ConnectionAborted, NotConnected, AddressInUse,
AddressUnavailable, ResourceExhausted, FileTooLarge, NoSpace, QuotaExceeded,
CrossDevice, DeviceFailure, and Other. It may carry opaque target detail for
diagnostics. Exhaustive portable control flow branches on the stable class; raw
errno or platform detail is not a portable semantic discriminator. A provider
may return Unsupported rather than silently weaken a guarantee. New native
errors map to Other until a later numbered specification deliberately adds a
portable distinction.

The first command slice fixes `ArgError` to InvalidIndex or
ResourceExhausted. Its text-conversion outcomes are InvalidText or
TooSmall(required). These smaller operation-specific outcomes do not masquerade
as host I/O failures.

An asynchronous operation cannot retain an ordinary Whitefoot borrow across
suspension. Submission therefore consumes an owned buffer or pool lease and a
resource lane into an affine `PendingOp`. Submit failure returns both owners.
While pending, the provider pins the data, registration, and lane; the original
file or socket lane cannot be moved, closed, or used through another owner.
Completion returns every owner plus one terminal outcome.

Cancellation is only a request. A normal completion and cancel acknowledgement
race at a provider-defined linearization point, but exactly one terminal result
wins and reports any progress. No buffer is returned until the provider proves
that the kernel or device can no longer access it. `PendingOp` is
completion-required: normal code must await it or perform `cancel-and-reap`.
Whole-process abort relies on process teardown, not language cleanup. If a later
amendment selects contained-instance traps, its runtime orphan reaper inherits
the pending obligation. Dropping a pending token never frees or reuses its
buffer behind an active host operation.

Positioned variants do not advance a cursor. Vectored forms must not store
ordinary borrowed slices beyond the call; future owned segment chains have
their own descriptor. Mapping and splice are separate capabilities, not secret
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

Conversion to UTF-8 text is explicit and fallible. Native path output has a raw
lossless route; escaped and lossy display are separate presentation operations.
A directory entry name returned by enumeration round-trips component-for-
component into `open_child` without text conversion.

The path model distinguishes `PathComponent`, `RelativePath`, and an absolute
target path. NUL and target-invalid prefixes are rejected during construction.
Directory-relative operations never implement confinement with string-prefix
concatenation. They define whether `..`, a final symlink or reparse point, and
intermediate links are rejected or followed.

Two authority profiles are explicit:

- a process-equivalent namespace capability may intentionally follow native
  resolution anywhere its full namespace grant permits; and
- a confined directory capability guarantees that lexical traversal,
  symlinks/reparse points, mount transitions, and rename races cannot escape the
  granted root. A target unable to uphold that contract returns Unsupported.

Absolute paths, Windows drive or UNC prefixes, and cross-root operations require
the appropriate namespace capability; a Directory alone does not imply them.
The first command slice uses one process-equivalent current-directory
`ProcessCwdRead` and relative paths. A future confined root has a distinct
`ConfinedDirectoryRead` type and descriptor rather than dynamically changing
the containment promise of this operation.

The initial Args representation is an opaque immutable snapshot resource.
`arg_get` copies only the selected argument into an owned opaque `HostString`
with its native code units and provider-ready trailing sentinel. `HostString`
has the same target-independent semantic identity on every target but a
target-selected private representation; source cannot inspect or mix its u8 or
u16 units. This avoids an affine-element collection, stored slices, a Unix-only
public API, and a whole-argv copy.

Converting HostString to text is explicit and fallible through UTF-8 length and
caller-buffer copy operations. Converting it to RelativePath consumes the owner,
validates the exact first-slice path policy, and reuses its allocation without a
second copy. Failure also consumes and releases the HostString. A later
invocation-root borrowed view may add a zero-copy argument route without
replacing these owned operations.

### 6.9 Concurrency, waiting, and cancellation

Every resource family declares `Sendable`, `Shareable`, and its concurrent
operation matrix independently. A gated resource contract proves that a
provider representation satisfies those predicates; the language checker then
enforces movement, sharing, and non-interference. Neither side can infer them
from an fd or pointer.

- a cursorful file is movable to one worker but not implicitly shareable;
- positioned-read lanes may later be independently movable under an exact
  contract;
- a connected socket can be consumed by `split` into linked affine receive and
  send lanes so full-duplex operations do not require one unique whole-socket
  borrow; and
- stdout is not turned into a shared global lock. Parallel search gives one
  output owner or an explicitly specified publication mechanism.

The first-slice predicates are fixed now: Args, HostString, RelativePath, and
ProcessCwdRead are Sendable and Shareable under scoped shared borrows; ReadFile
and CommandOutput are Sendable but not Shareable. ReadFile's cursor and
CommandOutput's publication order therefore always have one mutable owner. A
future descriptor may add explicit lanes or a publisher, but does not
retroactively infer sharing from the native representation.

Thread or task creation is a machine/runtime authority, but concurrency is not
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
actually adopts, and that wakeup and owner lifetime are not provider-private.

The active language has no function values, stored borrows, shared-state memory
model, atomics, or task syntax. The first command slice does not pretend to
implement them; its resource protocol is chosen so they need additive
concurrency semantics rather than replacement file or output APIs.

### 6.10 Provider, versioning, and ABI

Each system operation has one target-independent compiler-owned semantic ID.
Its gated semantic record binds signature, outcomes, ownership transitions,
memory and external effects, cleanup, and required provider guarantees. The
checked IR carries only that semantic ID; it never carries a native provider
symbol or recognizes a source function name, path, project, test, or signature
lookalike.

A separate target-qualification table maps
`(spec version, semantic ID, target, entry profile)` to an approved provider
implementation version and private ABI symbol. A build fails when the mapping
is absent or incompatible. A provider upgrade cannot silently change semantics;
a semantic change requires a new specification identity and compatibility
review. A fixed Rust enum and table are sufficient—no WIT parser, semver
registry, dynamic loader, or plugin protocol is required.

For the first macOS/Linux provider, selection is static for the whole build.
The native hot-transfer gate is one direct statically bound provider call whose
body performs at most one libc/host transfer call, plus required bounds, count,
and error checks. There is no per-call vtable, handle-table lookup, provider tag,
domain metadata, heap allocation, data copy, or global provider lock. Wrapper
elimination is welcome but is not assumed; emitted code and measurement must
demonstrate the cost shape.

The command and process-isolated service specializations need no instance handle
table: a trap terminates the owning process, while normal compiler cleanup uses
direct opaque native values. If a later language amendment permits contained
in-process traps, that profile registers resource acquisition and pending
submission in a per-instance reaper. Registration stays outside each
synchronous transfer hot path, must not use one global provider lock, and
receives its own cost gate. Unselected containment machinery never taxes command
reads.

A deterministic provider for the first slice supplies only the arguments,
files, short reads, partial writes, redirects, and failures needed by its
contract tests. Later slices extend it only with their own operations. This is
not a general simulator or artifact-replay framework. Providers and the runtime
remain in the TCB; conformance and hostile tests provide evidence and catch
regressions but do not prove provider honesty.

A WASI target can later implement this provider model. Whitefoot source
semantics need not inherit every WASI limitation.

## 7. Protocol exemplars

These state machines test whether the cross-cutting rules are real. Entry/path,
ReadFile, and CommandOutput enter the initial implementation; TCP and Child
remain design exemplars.

### 7.1 Entry snapshots, HostString, RelativePath, and ProcessCwdRead

- Args is an immutable release-complete resource in states Live or Consumed.
  `args_count(&Args)` only borrows it. `arg_get(&Args, index)` leaves Args live;
  success allocates and returns one owned HostString, while InvalidIndex or
  ResourceExhausted returns no new owner.
- HostString is release-complete and owns target-native code units plus a private
  provider-ready sentinel. UTF-8 length and copy operations borrow it and leave
  it live on every outcome. A successful copy changes exactly the requested
  destination prefix; TooSmall(required) and InvalidText leave the buffer
  unchanged.
- `relative_path(HostString)` consumes HostString on success and error. Success
  retypes and reuses the same allocation as owned RelativePath; InvalidPath
  releases it and returns no owner. RelativePath is release-complete.
- `open_read(&ProcessCwdRead, &RelativePath)` uses scoped shared borrows, so both
  inputs remain live on success and error. Success creates one ReadFile;
  IoError creates none. ProcessCwdRead is release-complete, process-equivalent,
  Sendable, and Shareable for open operations; it is not a confinement claim.
- Args, HostString, RelativePath, and ProcessCwdRead release only their own
  compiler/provider storage. None stores a source borrow, and none needs a
  runtime handle-table lookup on an ordinary operation.

### 7.2 ReadFile

- `ProcessCwdRead.open_read(relative_path)` creates `ReadFile(Open)` with a
  cursor domain and a conservative filesystem-object alias domain. A separate
  open does not prove a separate object; no duplicate operation exists in the
  first slice.
- `read_once(&uniq file, &uniq buffer, range)` is call-scoped. `bytes(n)` leaves
  both owners live, changes exactly the first `n` requested bytes, and advances
  the cursor by `n`; `end` changes no byte; `error` changes no byte unless the
  operation returned progress as a different result. The first regular-file
  primitive never hides a second attempt.
- A later positioned-read lane observes the object without changing this
  cursor. Multiple lanes are allowed only through an operation whose descriptor
  creates and owns them; sharing the handle is not inferred.
- ReadFile is release-complete. Normal compiler release consumes it and may
  discard only a close diagnostic that carries no read or durability guarantee.
  Explicit close also consumes it on every outcome. Whole-process abort relies
  on OS teardown; only a separately selected contained-instance amendment would
  quiesce pending lanes and close the native object through a runtime reaper.

### 7.3 Command Output, stdout, and stderr

- `stdout` and `stderr` are separate affine CommandOutput owners but share one
  conservative command-output ordering domain because host redirection may make
  them the same sink.
- `write_once(&uniq CommandOutput, bytes, range)` performs one provider-visible output
  attempt. Calls made sequentially across either owner preserve source order.
  A successful count means that prefix was accepted by the host operation; it
  promises neither line atomicity nor storage durability.
- The first provider adds no hidden userspace buffering. CommandOutput is
  therefore
  release-complete: every material failure is reported by the write that sees
  it. Compiler release only detaches the source capability; it does not close or
  flush host stdout/stderr. Whole-process teardown later closes the native
  descriptors. A later buffered Output wrapper is completion-required and must
  expose flush/finish rather than inherit this policy.
- On macOS/Linux command targets, bootstrap owns the process and installs the
  profile's ignored-SIGPIPE disposition before entry, so a broken pipe reaches
  `write_once` as BrokenPipe without adding per-write signal operations. A
  hosted/service profile must receive an equivalent host guarantee or use a
  separately costed provider; it may not silently change a surrounding host
  process's signal policy.
- Concurrent publication has no implicit order. Parallel search gives one
  owner to an aggregator or uses a separately specified ordered publisher.
- Terminal control, color, and console modes are separate capabilities. A trap
  diagnostic uses a mandatory runtime channel, never flushes ordinary
  CommandOutput,
  and cannot be used by source code.

### 7.4 TCP, split lanes, and pending receive

- A TCP resource moves through fixed states such as Unbound, Bound, Listening
  or Connecting, Connected, and Consumed. Unsupported target transitions fail
  rather than emulate weaker semantics.
- `split(Connected)` consumes the whole stream and returns linked affine RxHalf
  and TxHalf lane owners. This permits one receive and one send to progress
  concurrently without sharing one mutable whole-stream handle.
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
- UDP uses a separate datagram descriptor: zero-length datagrams are data,
  receive reports truncation explicitly, and a send is one datagram rather than
  a byte-stream prefix.

### 7.5 Child process

- ProcessSpawn authority selects an executable relative to an explicit
  executable or namespace capability. Arguments, environment, cwd, and each
  stdio transfer are explicit. No ambient fd/handle inheritance occurs. Signal
  mask and disposition are also explicit child-start state: the portable default
  resets SIGPIPE and other resettable dispositions for the child; inheritance
  requires a separately named spawn option and provider support.
- Spawn returns a stable Child resource, not a reusable PID. Its states are
  Running, ExitedUnreaped, and Consumed/Reaped; optional stdin/stdout/stderr
  pipes are separate owners with declared alias and cleanup rules.
- Child is completion-required. Normal code must wait/reap or explicitly detach
  where the entry profile permits it. Kill requires a separate right and is only
  a termination request: it retains the Child owner and reports whether the
  request was issued, was unnecessary because exit was already observed, or
  failed. Wait/reap consumes Child and returns terminal exit. Detach also
  consumes Child, returns no exit status, and transfers eventual reaping/orphan
  responsibility to the declared runtime/profile policy. Whole-process abort
  performs no language wait/reap and does not promise that the child is killed
  or rolled back; the target's documented child-orphan policy applies. A future
  contained-instance amendment would need an explicit reaper policy.
- Signals are typed event or cancellation inputs, not arbitrary asynchronous
  handlers. Broken pipe is an Output error. A higher-level retry helper may
  retry Interrupted only when it is not suppressing an observable signal,
  timeout, or cancellation event.

## 8. Capability-family map

| Family | Entry authority and resources | Architectural rule | First implementation |
|---|---|---|---|
| Command context | exact command imports; Args, HostString, Environment, Stdin, stdout, stderr | snapshots are distinct from live streams; normal status is entry return | Args, HostString, stdout/stderr CommandOutput, `ExitStatus` |
| Filesystem | namespace or Directory grants; Directory, files, cursor, mapping, lock | lossless paths, explicit containment profile, operation-specific mutation/durability, descriptor per resource | ProcessCwdRead, RelativePath, relative open, ReadFile |
| Clocks and timers | SystemClock or MonotonicClock; Timer/Pending | distinct ordered observation domains; monotonic deadlines never derived from wall clock | none; later additive interface |
| Randomness | SecureRandom, InsecureRandom, RuntimeSeed | distinct ordered sources; seeded deterministic generator is a value, not secure authority | none; later additive interface |
| Network | Network/Resolver grants; TCP states, UDP, listener, split lanes | protocol state machines, stream/datagram distinction, backpressure, pending ownership, half-close | none; TCP exemplar closes substrate decisions |
| Wait and cancellation | Wait authority; PendingOp, timer, cancellation source | cancel is request; one terminal outcome; quiescence before owner return; wait representation unselected | none; future API must use descriptor rules |
| Threads | Worker authority; JoinHandle or selected scope resource | same authority/resource substrate, plus separately selected memory, capture, join, trap, and failure semantics | none; PAR-4 co-design required |
| Async tasks | execution authority; task/completion resources | not interchangeable with OS threads; continuation owns suspended state | none; task syntax/runtime unselected |
| Child processes | ProcessSpawn; Child and explicit pipe resources | capability-relative executable/cwd, no implicit inheritance, stable identity, mandatory reap/detach disposition | none; Child exemplar closes substrate decisions |
| Signals | typed signal-event grant | event stream or cancellation input; no arbitrary async source handler | none; later additive interface |
| Local IPC | explicit local namespace grant; pipe/local socket/shared-memory resource | family-specific stream, datagram, alias, and shared-memory rules | none; later |
| Memory mapping | Directory/File plus mapping support; Mapping | mapping is provenance root; invalidation, faults, dirty state, and sync are explicit | none; later |
| Target/device | embedded exact imports; typed device resources | target-qualified semantics, never a generic syscall escape | none; target-specific |
| General FFI | separate gated foreign capability | opaque ABI, callbacks, loading, and foreign threads are BOUND-2 | outside this design |

Architectural membership does not promise simultaneous implementation. A later
family may add operations and resource types, but it must use the same authority,
owner-accounting, effect, process-abort, future-containment, qualification, and
provider rules.

## 9. Exact first command slice

The first implementation target is deliberately small but not provisional. Its
semantic operations are:

| Item | Exact semantic shape | Ownership / effects |
|---|---|---|
| command entry | exact imports: Args, one ProcessCwdRead, stdout CommandOutput, stderr CommandOutput; returns `ExitStatus` | imports are independent; normal cleanup precedes status mapping; trap returns no status |
| `args_count` | shared-borrowed immutable Args snapshot → count | Args remains; snapshot read; no live host event |
| `arg_get` | shared-borrowed Args + index → owned HostString, InvalidIndex, or ResourceExhausted | Args remains; success has `allocates(heap)` and copies only that argument including private sentinel; errors create no owner; no live host event |
| `host_utf8_len` | shared-borrowed HostString → encoded length or InvalidText | HostString remains; no allocation or external event |
| `host_copy_utf8` | shared-borrowed HostString + unique initialized buffer + range → bytes(exact encoded length), TooSmall(required), or InvalidText | success copies the entire UTF-8 encoding, returns exactly `host_utf8_len`, and leaves the rest of the supplied range unchanged; failures leave the whole buffer unchanged; owners remain |
| `relative_path` | consuming HostString → owned RelativePath or InvalidPath; rejects NUL and absolute/target prefixes, preserves and accepts `.` and `..` components | success reuses the owned native allocation; failure consumes/releases it; no second allocation; first-slice open follows intermediate and final native links and permits escape above cwd within the process-equivalent grant |
| `open_read` | shared-borrowed ProcessCwdRead + shared-borrowed RelativePath → owned ReadFile or IoError | inputs remain; success acquires one resource; observes namespace, changes the resource-lifetime domain, may block; no ambient cwd lookup |
| `read_once` | unique ReadFile + unique initialized buffer + range → bytes/end/IoError | one host attempt; observes file, changes cursor, may block; owners remain |
| `write_once` | unique CommandOutput + byte view + range → bytes/IoError | one host attempt; changes shared command-output domain, may block; owners remain |
| `ExitStatus` | portable command code 0–255; wfgrep uses 0, 1, 2 | provider maps exactly on qualified targets; startup failure and trap are outside it |

Compiler-derived release is also an exact semantic operation:

| Resource | Consuming release action and exact effect |
|---|---|
| Args | logical consume; no host call or external effect |
| HostString / RelativePath | free compiler-owned heap storage; ordinary memory cleanup, no external event |
| ProcessCwdRead | one native close attempt; `changes(cwd-resource-lifetime)` and `blocks(cwd-resource-lifetime)`; discard only the close diagnostic and never retry the fd |
| ReadFile | one native close attempt; `changes(file-resource-lifetime)` and `blocks(file-resource-lifetime)`; discard only the close diagnostic and never retry the fd |
| CommandOutput | logical source-capability detach; no close, flush, provider call, or external effect; OS process teardown owns descriptor close |

Environment, stdin, directory enumeration, absolute namespaces, file mutation,
buffered output, async, network, and workers are later additive imports and
operations. None requires replacing the operations above.

### 9.1 Native cost shape to verify

| Path | Required native shape | Evidence still required |
|---|---|---|
| provider selection | one link-time target table decision | inspect checked IR and final symbols; no runtime provider tag |
| selected argument | one O(length) opaque allocation/copy with provider-ready sentinel, only when requested | compare with native argv access; no whole-argv copy or handle lookup |
| UTF-8 text conversion | length pass plus caller-buffer encode/copy only for arguments used as text | inspect Unix fast path and Windows conversion; invalid text is explicit |
| RelativePath construction | in-place validation and type transition over consumed HostString | verify no second allocation/copy and no stored source borrow |
| `open_read` | one direct provider call and one native open-relative operation | inspect call path and target error mapping |
| `read_once` / `write_once` | one direct provider call, at most one host transfer call, bounds/count/error checks | inspect generated code; count calls, copies, allocations, and locks |
| ProcessCwdRead/ReadFile release | one direct native close attempt | verify `changes(resource-lifetime)` plus `blocks(resource-lifetime)` is in every enclosing exact row; never retry an ambiguous fd close |
| Args/HostString/RelativePath release | logical consume or compiler-owned heap free only | verify no external event, provider call, handle lookup, or hidden source borrow |
| CommandOutput release | logical capability detach only; no close or flush | verify no external event/provider call and that OS process teardown owns native descriptor close |
| initialized buffer | one initialization on allocation, then reuse across reads | measure against a raw initialized-buffer control; stop for a proof-gated initialization design if this is material |

These are gates, not claims that LTO or the OS will remove the cost.

The macOS/Linux command bootstrap performs its one-time SIGPIPE normalization
before timing the source entry body, but end-to-end wfgrep measurement includes
bootstrap. The emitted hot `write_once` path still contains no per-call signal
mask operation. Hosted/service targets require separate qualification.

## 10. Witness traces

### 10.1 Sequential wfgrep

1. The command profile grants exactly Args, one ProcessCwdRead, stdout, and
   stderr as independent inputs.
2. `arg_get` obtains opaque native HostStrings. Pattern conversion to UTF-8 is
   explicit; `relative_path` consumes the path argument without a second copy,
   and ProcessCwdRead opens it as ReadFile.
3. wfgrep reuses one initialized caller-owned buffer. `read_once` changes only
   the returned prefix, reports a short count separately from end, and advances
   the cursor by exactly that count.
4. Matching consumes only the returned prefix. Boundary overlap remains in
   ordinary Whitefoot storage.
5. `write_once` publishes a checked range. Its short result is handled without
   assuming failure atomicity, and stdout/stderr calls preserve source order.
6. Normal return derives release before mapping `ExitStatus` 0, 1, or 2.

No whole-file allocation, per-byte call, Unicode path conversion, raw fd,
hidden global, or temporary Process API is required.

### 10.2 Parallel file search

1. Later directory traversal creates independent ReadFile resources and owned
   work records; a language storage design must make those records representable
   before this witness becomes executable.
2. Each ReadFile and buffer moves to one worker through a checked declared
   parallel construct. Cursorful handles are not shared.
3. Workers return owned match batches. One output owner or specified publisher
   determines publication order; no worker borrows a global Process or stdout.
4. The selected join/scope mechanism accounts for every task, owner, and
   deterministic failure before normal exit.

The resource API survives this step, but task syntax, affine-element storage,
Sendable/Shareable judgments, memory model, worker provider, and failure
selection remain explicit PAR-4 work.

### 10.3 Cancellable network service

1. The service profile statically imports listener, monotonic clock,
   cancellation, and execution authority.
2. Accept yields owned TCP resources. `split` creates Rx/Tx lane owners; each
   connection and owned buffer lease moves into the selected task construct.
3. A pending receive owns its Rx lane, buffer, registration, and core reference.
   Submit failure returns them; completion or cancel-and-reap returns them only
   after quiescence.
4. A timer races receive through the future selected wait model. Exactly one
   terminal result owns partial progress and every submitted owner.
5. Half-close, last-owner release, task failure, and whole-process abort are
   distinct transitions. The baseline service is process-isolated; any future
   host-surviving containment must satisfy the separate quiescence gate.

This witness fails any design that retains statement-scoped borrows across
suspension, uses one Process holder, frees a cancelled buffer before quiescence,
or leaves wakeup lifetime private to the provider.

## 11. Required language and compiler deltas

The exact first slice requires:

- a command entry profile with exact imports and `ExitStatus`, replacing the
  current FN-7 ceiling;
- fixed opaque Args, HostString, ProcessCwdRead, RelativePath, ReadFile, and
  CommandOutput types;
- release-complete compiler-derived cleanup on every normal edge;
- the exact operation-specific outcomes and portable error class above;
- external event categories and conservative exact rows distinct from memory;
- compiler-owned semantic operation IDs plus a static target qualification
  table and provider; and
- checked IR resource/effect identities and first-slice conformance tests.

The first slice does not require completion-required checking, stored borrows,
partial initialization, D17 representation proofs, mapping, async syntax, wait,
cancellation, atomics, threads, function values, revocation, dynamic loading,
callbacks, or general FFI. If initialized-buffer cost is material, work stops
for a separately proved initialization model before wfgrep grows around it.

Later families have named protocol obligations and exemplar compatibility, not
completed numbered-spec semantics. Their implementation remains conditional on
the project and on the separate concurrency decisions they actually need.

## 12. Performance and hostile-review gates

The architecture is rejected if any witness requires:

- one host call per byte or field;
- a full input copy or materialization not required by the operation;
- a unique global context or centralized provider lock on independent work;
- dynamic dispatch or handle-table lookup on the static native hot path;
- a Unicode-only conversion before native path access;
- a borrow retained across suspension without tracked ownership;
- resource cleanup or pending-operation reaping by writer convention;
- handle identity or provider metadata used as a disjointness proof;
- a hidden external effect inside pure or memory-only rows; or
- a source-recognized primitive lookalike.

The first provider slice must test empty, short, exact, multichunk and changing
files; non-text argument/path values; `..`, absolute, NUL, and symlink policy;
open/read errors; short output and broken pipes; stdout/stderr redirected to one
sink; normal/recoverable/fatal cleanup; close error and no-fd-retry behavior;
effect omission and addition; primitive lookalikes; chunk and host-call counts;
peak storage; initialization; emitted calls; allocations, copies, and locks.

## 13. Recommendation and owner decision

Hostile review supports the exact-import plus typed-resource plus static-provider
direction and rejects the three smaller source models in section 5. This file
remains a candidate until the owner accepts its semantic choices; it is not yet
specification authority.

Owner acceptance would select:

1. exact static capability imports under a lifecycle profile;
2. affine typed resources with mandatory protocol descriptors and three
   completion policies;
3. conservative program-ordered external effects, with blocking, independent
   progress, suspension, and spawning kept distinct;
4. operation-specific one-attempt I/O, lossless target paths, portable error
   classes, and the existing whole-process abort law;
5. target-independent semantic operation IDs with a static qualified provider;
   and
6. the exact first command slice and native cost gates in section 9.

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
