# Whitefoot completion I/O design

Status: selected implementation design for the v0.37 candidate. The complete
first-principles derivation remains in `FIRST-PRINCIPLES.md`; the rejected
experimental implementation is classified in `IMPLEMENTATION-AUDIT.md`; the
first clean-core measurements are recorded in `RESULTS.md`.

This document replaces the earlier world-region proposal in place. It contains
no writer-visible world lifetime, `external` effect, `blocks` effect, future,
pending value, callback, task, or blocking I/O family.

## 1. What must two writes mean

Consider two calls which use one output:

```whitefoot
let header = write_once(output: &out, source: &header_bytes, start: 0_u64, end: header_end);
let body = write_once(output: &out, source: &body_bytes, start: 0_u64, end: body_end);
```

The machine should make both operations pending as soon as their arguments are
ready. The output bytes must still contain the header before the body. Target
completion order must not choose byte order, and neither operation should
occupy a compute lane while the target is making progress.

Three different relations are present:

```text
shared borrow of out      keeps one logical output root alive
ordered reservations      assign header before body on that root
payload borrows           keep each byte buffer alive until its own release
```

One exclusive borrow cannot express this. It would reject simultaneous
operations before the output family could apply its ordering rule. One shared
borrow alone also cannot express it. It keeps the root alive but says nothing
about an outside write.

The API therefore uses an ordinary shared borrow plus a capability effect:

```whitefoot
fn write_once['o, 's](
  output: &'o Output,
  source: &'s buffer<u8>,
  start: own u64,
  end: own u64,
) -> result: own Result<u64, IoError>
reads('o 's), writes(output);
```

`reads('o 's)` describes ordinary Whitefoot storage and borrow lifetimes.
`writes(output)` says that the call changes authority supplied by the `output`
parameter. The Output family refines that write into one ordered put
reservation. Source code still uses the normal function-call grammar and the
normal ownership, effect, and typed-outcome rules.

## 2. Why an effect names a capability value

A borrow region answers how long a reference is usable. A capability value
answers which logical outside authority an operation uses. They cannot be one
identity.

An owned terminal operation makes the difference visible:

```whitefoot
fn finish_file(output: own FileOutput)
  -> result: own Result<unit, IoError>
writes(output);
```

There is no borrow region in this signature. Requiring `writes('loan)` would
either make the effect impossible to write or force a fictitious lifetime into
the type. Hiding the action entirely would make the signature claim `pure`
while the function flushes and closes a file.

The source effect grammar therefore admits a direct formal parameter beside a
REGIONID:

```ebnf
effect := "reads" "(" (REGIONID | IDENT)+ ")"
        | "writes" "(" (REGIONID | IDENT)+ ")"
        | "allocates" "(" ("heap" | "arena" REGIONID)+ ")"
        | "traps"
```

The first implementation accepts only a direct formal capability parameter.
A later field or payload path needs its own grammar and projection proof.

The checker derives capability effects from system calls, releases, and user
calls, then compares the derived and written rows in both directions. A writer
cannot omit, invent, or weaken one. `pure` remains an honest empty row.

## 3. Logical roots and family fragments

Every capability has one compiler-retained logical root. A successful factory
mints a fresh root unless its contract says that the result is a facet of an
existing root. Move preserves the root; borrow refers to it; a family split
preserves the common root and assigns distinct roles.

Environment aliasing does not alter this proof. Two opens mint two logical
roots even if a hard link makes them reach the same inode. Standard output and
standard error are two roots even if the host redirects both to one native
sink. That is the same boundary other languages use when the environment
changes underneath a legal program.

One authority use retained by the compiler has this conceptual form:

```rust
struct AuthorityUse {
    origin: CapabilityOrigin,
    family: FamilyId,
    fragment: FragmentId,
    access: ReadOrWrite,
}
```

The family contract assigns exactly one relation to two live fragments on the
same root:

```text
free       both may proceed; neither creates an order
ordered    both may be pending; family attribution fixes logical order
exclusive  the later fragment waits for authority release
```

Fragments on different logical roots are independent. Ordinary memory loans
remain a separate test. Two free file reads still cannot overlap if they both
write the same destination buffer.

## 4. Files, directories, and output

### 4.1 Random-access files

A random read must not share an implicit cursor:

```whitefoot
fn read_at['f, 'd](
  file: &'f ReadFile,
  destination: &uniq 'd buffer<u8>,
  file_offset: own u64,
  start: own u64,
  end: own u64,
) -> result: own ReadOutcome
reads('f 'd file), writes('d);
```

Two `read_at` fragments on one file root are free. Destination loans decide
whether their memory writes can overlap. `ReadBytes(next)` states that the
destination prefix `[start, next)` was initialized. A short successful read is
`ReadBytes`; a failed outcome reports zero progress. `WouldBlock` and a
no-progress host interruption belong to the target adapter, not the writer
outcome.

### 4.2 Sequential files and directories

Sequential access is an owned Source because its cursor, read-ahead, capacity,
and finalization persist across outcomes:

```whitefoot
fn file_source(file: own ReadFile, offset: own u64)
  -> result: own FileSource
pure;

fn file_next['q, 'd](
  source: &'q FileSource,
  destination: &uniq 'd buffer<u8>,
  start: own u64,
  end: own u64,
) -> result: own ReadOutcome
reads('q 'd), writes(source 'd);
```

Several calls may hold ordered reservations. If the next physical offset is
not known until an earlier partial read completes, the later reservation waits
inside the Source without occupying a writer lane.

Directory lookup remains free on one `DirectoryRead` root. Enumeration is a
Source with ordered batches:

```whitefoot
fn open_read['c, 'p](root: &'c DirectoryRead, path: &'p RelativePath)
  -> result: own Result<ReadFile, IoError>
reads('c 'p root);

fn open_directory_source['c](directory: &'c DirectoryRead)
  -> result: own Result<DirectorySource, IoError>
reads('c directory);

fn directory_next['q, 'd](
  source: &'q DirectorySource,
  destination: &uniq 'd buffer<u8>,
  start: own u64,
  end: own u64,
) -> result: own ListOutcome
reads('q 'd), writes(source 'd);
```

The caller-buffer form is the first implementable storage shape. A later
source-owned pool can return owned initialized batches without changing the
completion or authority model.

### 4.3 Output and finished file output

`Output` is an ordered Sink. Its runtime may combine adjacent reservations
into `writev`; a partial physical write completes whole earlier reservations,
then records progress for at most one reservation, without changing logical
byte order.

Command output is release-complete and promises neither durability nor a
final close. A file created for output is different:

```whitefoot
fn open_output['d, 'p](directory: &'d DirectoryWrite, path: &'p RelativePath)
  -> result: own Result<FileOutput, IoError>
reads('d 'p), writes(directory);

fn write_file_once['o, 's](output: &'o FileOutput, source: &'s buffer<u8>, start: own u64, end: own u64)
  -> result: own Result<u64, IoError>
reads('o 's), writes(output);

fn finish_file(output: own FileOutput)
  -> result: own Result<unit, IoError>
writes(output);

fn abandon_file(output: own FileOutput)
  -> result: own unit
writes(output);
```

`FileOutput` is finish-required. `finish_file` drains ordered writes and
reports final flush and close diagnostics. It does not imply durable storage;
sync or transactional commit needs a separate operation and milestone.
`abandon_file` makes loss explicit instead of treating an affine drop as
successful finish.

## 5. Network and timer families

A TCP connection has one root with receive, send, and control fragments:

```text
Receive x Receive   ordered
Send x Send         ordered
Receive x Send      free
Control x any       exclusive
```

The first surface can keep those facets internal:

```whitefoot
fn tcp_connect['n, 'a](network: &'n Network, address: &'a SocketAddress)
  -> result: own Result<TcpConnection, IoError>
reads('n 'a), writes(network);

fn tcp_accept['l](listener: &'l TcpListener)
  -> result: own Result<TcpConnection, IoError>
reads('l), writes(listener);

fn tcp_receive_once['c, 'd](connection: &'c TcpConnection, destination: &uniq 'd buffer<u8>, start: own u64, end: own u64)
  -> result: own ReadOutcome
reads('c 'd), writes(connection 'd);

fn tcp_send_once['c, 's](connection: &'c TcpConnection, source: &'s buffer<u8>, start: own u64, end: own u64)
  -> result: own Result<u64, IoError>
reads('c 's), writes(connection);
```

The hidden fragment summary distinguishes send from receive even though both
source boundaries write `connection`. A listener is a persistent Source with
bounded preposted accepts. When its ready queue is full, the adapter stops
posting more accepts and lets the host backlog provide pressure. It never
builds an unbounded queue.

TCP send completion means that the local stack accepted the reported prefix.
It does not mean that the peer acknowledged it. A higher-level protocol that
requires acknowledgement owns a different receipt and finish contract.

A one-shot timer is a normal finite call; a periodic timer is a Source:

```whitefoot
fn timer_until['c](clock: &'c MonotonicClock, deadline: own MonotonicInstant)
  -> result: own Result<MonotonicInstant, TimerError>
reads('c clock);

fn periodic_next['t](timer: &'t PeriodicTimer)
  -> result: own TickOutcome
reads('t), writes(timer);
```

A periodic source retains one coalesced shot and reports an `expirations`
count. It neither allocates an unbounded backlog nor silently pretends every
period was delivered separately.

## 6. Completion milestones and capacity

Every operation record has distinct facts even when the first implementation
publishes them together:

```text
accepted
result-ready
payload-released
authority-released
terminal
```

A normal call exposes its result only at ownership-complete, which is
result-ready plus every payload and authority release the caller needs. A
zero-copy family whose result becomes ready before its payload is reusable
must return a family-specific affine receipt. It cannot make an ordinary
result usable early and hide the remaining loan.

Capacity exhaustion has one internal outcome, `wait-capacity`. The target does
not own the operation yet; the runtime retains its complete bundle and runs
other ready frames. `WouldBlock` is not a writer scheduling outcome. A typed
`ResourceExhausted` result is reserved for a real host resource-creation
failure after admission rules have run.

Cancellation request is not terminal. A cancellable family keeps the payload,
fragment, target token, and unique terminal witness until normal completion or
confirmed cancellation wins. Dropping an active operation is impossible.

## 7. Compiler representation

The compiler keeps memory and authority separate:

```text
memory row          region reads/writes and allocations
capability row      formal capability reads/writes
family summary      root, role, fragment, relation, attribution, milestones
target summary      never-suspends or may-suspend
```

System contracts seed all four. User calls substitute formal regions and
capability parameters into caller places. Capability origin follows moves,
returns, admitted aggregate projections, fresh results, and family facets. An
unknown origin or fragment fails closed for overlap and optimization.

Callable result origin is a closed-world fixed point, not the absence of an
edge. A result which can carry at most one runtime root retains three
independent finite facts: it may be absent on a variant such as `Err`, it may
be freshly minted, and it may come from each listed formal. Enum root
cardinality is computed per variant before taking the type-wide minimum and
maximum. This makes `Result<ReadFile, IoError>` a zero-or-one-root value while
distinguishing it from a struct that simultaneously owns two roots. A pass-through,
choice, recursive wrapper, loop update, or release therefore cannot turn an
existing caller root into a fresh local root or an empty effect.

The current executable compiler closes that representation for values whose
maximum is one. It does not pretend that one flat origin set describes a
multi-root product: after every ordinary source judgment, such a value stops as
the explicit `CapabilityResultOrigin` unsupported capability. The next
representation step is an origin tree keyed by owned release-leaf paths. Until
then, preliminary checking may use conservative scratch to finish ordinary
source judgments, but no multi-root product publishes a checked program or
reaches authority lowering and code generation as though its roots were absent.

The overlap judgment first applies existing dataflow, ordinary memory,
operand-read, loan, and exit tests. It then compares family fragments:

```text
free       permit
ordered    permit and retain the family attribution order
exclusive deny this overlap window
unknown    deny this overlap window
```

Denial never rejects source. The ledger reports the concrete roots, fragments,
relation, milestones, and whether lowering actually used the permission.

## 8. Runtime and lowering

`never-suspends` functions keep the current direct ABI. A `may-suspend`
function receives selective stackless start/resume lowering. Its frame stores
only values live across a suspension point, the resume state, owned operation
bundles, and milestone requirements. It is not a copied native stack.

Before target handoff, one finite operation has stable bounded storage and an
immutable token containing its captured generation. Submission returns:

```text
inline-terminal     all promised milestones published; no future packet
target-owned        target now owns progress
wait-capacity       runtime owns the unsubmitted bundle
```

A target publisher validates the captured token before changing result
storage, writes the result, release-publishes milestone facts, and leaves one
bounded completion event. A scheduler lane drains that exact event before it
injects the dependent writer frame. The resumed frame can therefore consume
immediately. Enqueueing the frame advances the shared compute/completion epoch;
target and drain code never invoke writer code.

An ordinary blocking join uses the same rule without a writer frame. Only
after its final source recheck does it register the exact token it may sleep
for. A different lane that drains that token clears the registration and
publishes the new consumable state. A drain with no registered token owner
performs no second epoch update, which keeps the uncontended round trip on the
measured fast path.
A blocking helper is allowed only as one target's bounded adapter and executes
only a typed target operation.

Compute publication and completion publication share one wake epoch and one
announce, recheck, then park decision. Completion-before-wait produces no
wake. One POSIX waiter receives one signal; several already-announced waiters
receive one broadcast for the epoch transition. A frame can resume on any
eligible lane; its originating lane is an affinity hint, not a correctness
condition. Queue draining and helping are bounded.

No submission, completion, frame, scheduler, or target path reads a trap latch
or carries trap-specific state. A correct program cannot execute a false
claim, so that impossible path receives no normal-path budget.

## 9. Implemented v0.37 candidate boundary

The candidate implements the general compiler model and one deliberately
finite actualization slice:

- direct independent `read_at`/`write_once` groups of 2–64 reserve completion
  slots all-or-none, then submit every member including the source-last call;
- 2–16 direct same-block Output writes reserve all operation capacity or none,
  submit every member, and commit their `OutputBytes` attribution before the
  first physical write;
- one single-block root suspension can cross a zero-state tail-wrapper chain
  to a file leaf and resume on any scheduler lane;
- empty file ranges complete inline or complete their already-reserved token
  without a host transfer;
- branch, loop, multi-suspension, indirect, and non-tail suspended shapes keep
  the synchronous ABI; and
- DirectorySource retains `DirectoryEntries` edges but is not actualized yet.

Linux parks on one epoll set containing the io_uring fd and a broadcast
eventfd for compute, target, admission, and capacity facts. The eventfd remains readable until every
already-announced waiter leaves, so one waiter cannot consume another token
owner's wake. macOS uses the bounded typed fallback for regular files and the
same internal EINTR/readiness retry for directory batches. Windows has a real
IOCP/OVERLAPPED implementation and a Win32-native core, but remains
fail-closed until the probe executes on Windows and its wake packet path gains
the same bounded persistent multi-waiter guarantee.

## 10. Platform shape and evidence

Linux should submit real operations through io_uring where the qualified
kernel supports them. Windows should use overlapped operations associated with
IOCP. macOS may use kernel completion or readiness where it is real and a
bounded target-only helper for regular-file operations which lack a better
facility. None of those mechanisms changes source semantics.

The first end-to-end evidence must include:

1. independent output writes simultaneously pending;
2. two writes on one Output pending with source-order bytes;
3. two `read_at` operations on one file and disjoint destinations overlapping;
4. overlapping destinations denied by ordinary memory proof;
5. TCP receive and send on one root overlapping;
6. pure code linking no completion runtime;
7. an unused outside write remaining in the module;
8. completion-before-wait producing no wake;
9. a stale captured token failing before result storage changes; and
10. no claim-dependent instruction or field on a correct submission path.

Performance remains an experiment, not a premise. Each platform compares
matched direct blocking and completion implementations at depth one and across
increasing concurrency, including inline completion, bounded helper fallback,
native completion, Source/Sink batching, and scheduler resume latency. A
measured loss reopens the responsible representation or adapter. It never
introduces a second writer-visible blocking API by default.
