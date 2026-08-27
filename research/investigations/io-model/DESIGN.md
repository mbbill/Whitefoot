# Whitefoot unified-state completion I/O design

Status: selected design, owner-confirmed 2026-08-26 and implemented as a
validated work-branch candidate on 2026-08-27. The full derivation is in
`FIRST-PRINCIPLES.md`; `IMPLEMENTATION-AUDIT.md` classifies the discarded
experiments; `RESULTS.md` records current validation and scoped measurements.

This design adds no I/O ownership form. Files, sockets, outputs, clocks,
factories, permits, Sources, buffers, and aggregates are ordinary Whitefoot
values. `own`, `move`, `&`, and `&uniq` provide all authority. A lifetime says
how long a loan lives. It never names the state an effect touches.

The only source-effect change is that `reads` and `writes` name formal state
paths instead of lifetimes:

```whitefoot
fn write_once['o, 's](
  output: &uniq 'o Output,
  source: &'s buffer<u8>,
  start: own u64,
  end: own u64
) -> result: own Result<u64, IoError>
reads(output, source), writes(output);
```

There is no `world`, `memory`, `external`, `blocks`, capability root, family,
fragment, `Ordered` relation, writer-visible future, callback, task, or await.

Call-site snippets below are semantic sketches. Some omit the mandatory live
REGIONID after `&` or `&uniq` so the state relation stays readable. Normative
Whitefoot source still writes that region; this design changes effect subjects,
not borrow syntax.

## 1. Permission and behavior are separate

Consider a function which receives an entire record but changes one field:

```whitefoot
fn refresh['r](record: &uniq 'r Record) -> result: own unit
reads(record.payload), writes(record.checksum);
```

The parameter mode grants permission over the complete record:

```text
&uniq record
    no overlapping access to any part of record while the loan is live
```

The effect row states the behavior the checked body actually exhibits:

```text
reads(record.payload)
writes(record.checksum)
    after the call, every other field is known unchanged
```

The narrower effect never narrows the loan. Two calls which both borrow the
whole record uniquely cannot overlap even if their written fields differ. To
obtain real field concurrency, the API accepts field borrows:

```whitefoot
update(value: &uniq pair.first);
update(value: &uniq pair.second);
```

The two actual places are disjoint under the existing place-overlap rule.
Their instantiated `writes(value)` paths are disjoint for the same reason.

## 2. Effect paths

The selected surface is:

```text
effect_path := FORMAL_PARAMETER ("." STRUCT_FIELD)*
```

Examples:

```whitefoot
reads(source)
writes(output)
reads(record.header, record.payload)
writes(record.checksum)
```

The root must be a formal value parameter. A bare root denotes the complete
state supplied by that parameter. A field suffix denotes one static structural
substate. Dynamic indices, ranges, enum payload selectors, dereference syntax,
locals, and result binders are not part of the first surface.

For a borrow parameter, the path denotes the referent rather than the local
reference representation. For a direct slice parameter, it denotes the viewed
backing state rather than the descriptor. For an `own` parameter, it denotes
the state moved into the call.

Calls substitute one formal path onto the corresponding actual place:

```text
callee effect     writes(value.inner)
actual argument   &uniq container.left
instantiated      writes(container.left.inner)
```

Reborrow and slice substitution reuse the existing resolved-place and origin
rules. Contract equality normalizes a path to parameter ordinal plus field
ordinals, so the contract and implementation may use different parameter
spellings.

`reads` and `writes` remain separate exact facts. An operation which observes
old state while changing it declares both. The checker derives the complete
row from body accesses, callees, and compiler-derived release, then compares
the written and derived rows in both directions.

## 3. Ordinary ownership closes the external mapping

An output is an ordinary opaque affine value:

```whitefoot
write_once(output: &uniq output, source: &bytes);
```

The unique loan prevents another operation from using the same output until
the completion contract returns that loan. Two distinct outputs remain
independent:

```whitefoot
write_once(output: &uniq stdout, source: &left);
write_once(output: &uniq stderr, source: &right);
```

A stateful observer follows the same rule:

```whitefoot
fn now['c](clock: &uniq 'c Clock) -> result: own Instant
reads(clock), writes(clock);
```

The clock is a changing state machine. Exposing it as `&Clock` would allow
hidden mutation behind a shared loan. Independent observations require
separate ordinary owned readers or permits produced by an ordinary factory.

Sequential input and output, random sequences, receive queues, listener
acceptance, directory iteration, and resource lifecycle all use `own` or
`&uniq` for the same reason. A position-explicit file read may use a shared
borrow only when its contract has no caller-visible cursor or other consumed
state:

```whitefoot
fn read_at['f, 'd](
  file: &'f ReadFile,
  destination: &uniq 'd buffer<u8>,
  file_offset: own u64,
  start: own u64,
  end: own u64
) -> result: own ReadOutcome
reads(file, destination), writes(destination);
```

Different destination places can overlap in execution. One shared file value
does not create a special free-read relation; ordinary shared borrowing already
permits the observations. A file operation which advances a cursor instead
uses an owned state machine and `&uniq`.

## 4. Factories and permits are ordinary values

No observable system action may obtain ambient authority:

```whitefoot
now();
open(path: path);
connect(address: address);
random();
```

Each operation instead receives an ordinary object which owns the required
state:

```whitefoot
fn reserve_file['f](factory: &uniq 'f FileFactory)
  -> result: own FilePermit
reads(factory), writes(factory);

fn open_read['d, 'p](
  permit: own FilePermit,
  root: &'d DirectoryRead,
  path: &'p RelativePath
) -> result: own Result<ReadFile, IoError>
reads(permit, root, path), writes(permit);
```

`reserve_file` itself contributes `writes(factory)` to every wrapper which
calls it. `open_read` consumes and writes the permit on success and failure.
The directory and path are stable selector inputs, so they use shared borrows;
the changing observation occurrence is the consumed permit and is not hidden
mutation of `DirectoryRead`. A later operation on the local file does not need
to be traced backward and relabelled as another factory write.

Moving an incoming owner to another local name still denotes the same value:

```whitefoot
let same = pass(file: move file);
close(file: move same);
```

This is existing `move` semantics. The compiler must not forget it while
checking effects across calls or releases, but the language gains no new
identity, lineage, or capability feature.

The command entry may receive `command.files as files: own FileFactory`.
Factories produce several independent permits through short unique loans.
Long operations then consume different owned values and may overlap through
shared loans of one `DirectoryRead`:

```whitefoot
let first_permit = reserve_file(factory: &uniq files);
let second_permit = reserve_file(factory: &uniq files);

open_read(permit: move first_permit, root: &cwd, path: &left_path);
open_read(permit: move second_permit, root: &cwd, path: &right_path);
```

This first file slice burns each permit on success or recoverable failure.
Reservation is total and proof-only: it reserves no native descriptor, kernel
memory, or host quota. Host exhaustion remains `IoError.ResourceExhausted`
from the attempted open. The permit is erased before the native open ABI, so
the ownership proof adds no native hot-path argument.

A helper which itself accepts `&uniq FileFactory` keeps that caller loan for
the helper call's full duration. If the helper also performs the open, it has
deliberately chosen serialization even though the system API did not require
it. Code that needs overlap reserves permits before entering the long helper,
passes owned permits, or threads an owned factory through an explicit result.
This is the ordinary borrow rule made visible, not a hidden factory exception.

A later API whose finite logical budget must be reused returns an owned permit
from explicit close or finish. An automatic release cannot mutate a separately
owned shared pool behind the writer's back.

## 5. Finite calls and persistent Sources

A finite one-shot operation uses the ordinary call surface. The result becomes
an ordinary value only at the operation's ownership-complete milestone. The
compiler may run independent calls while the target owns the operation.

A persistent or unbounded relation is an ordinary owned state machine:

```whitefoot
fn next['s](source: &uniq 's ConnectionSource)
  -> result: own AcceptOutcome
reads(source), writes(source);
```

Listeners, periodic timers, signals, directory iteration, and unknown-size
streams use this form when their capacity, cursor, coalescing, pool, and finish
state persists across deliveries. They do not use a callback. Target code
publishes an owned outcome; the Whitefoot scheduler later runs writer code.

Pending identity becomes writer-visible only when the program must store,
return, cancel, query, or select it independently. Such an identity is a
family-specific ordinary affine object, not a universal future wrapper.

## 6. Completion ownership

Before target acceptance, the runtime owns one closed operation bundle:

```text
stable operation record
resource owner or live resource loan
payload owners or live payload loans
target metadata
uninitialized result storage
```

Submission has three honest outcomes:

```text
rejected before ownership
    target received nothing; runtime keeps the complete bundle

inline terminal
    result and every retained owner or loan return immediately;
    no later event can arrive

accepted
    target owns the bundle until its contract publishes release
```

The internal operation record distinguishes at least:

```text
result_ready
loan_released(path)
terminal
```

A simple operation may publish them together. Zero-copy and multishot target
operations may release different paths at different times. Cancellation
requests do not return any retained owner or loan; only the target's terminal
or exact `loan_released(path)` milestone can do that.

These facts describe when existing ordinary ownership returns. They do not
mint authority and do not appear in writer syntax.

## 7. Dependency-driven overlap

Consider three calls:

```whitefoot
let first = write_once(output: &uniq out, source: &a, ...);
let middle = write_once(output: &uniq err, source: &b, ...);
let last = write_once(output: &uniq out, source: &c, ...);
```

The first and middle calls can overlap because their actual output places are
different. The last call cannot obtain `&uniq out` until the first operation
returns that loan. It has no dependency on `middle`.

The compiler therefore builds ordinary value, control, read/write, and loan
release dependencies. When `first` returns the output loan, `last` may submit
even if `middle` is still in flight. There is no Output-specific edge, ordered
batch, adjacency scan, or whole-group join.

The same mechanism handles memory, files, sockets, clocks, permits, and
compiler-derived release. A fine effect path may preserve facts across a call,
but it never overrides a conflicting loan.

## 8. Runtime and target boundary

The generic completion core retains:

- finite generation-checked operation records;
- exactly one terminal publication;
- release/acquire result visibility;
- drain before a writer frame becomes runnable;
- announce, recheck, then park;
- one scheduler sleeping decision for compute and completion;
- target callbacks which never execute writer code; and
- pure-compute binaries which link no completion machinery.

The old bridge machinery is deleted:

- logical roots and family identifiers;
- Free, Ordered, and Exclusive tables;
- Output byte-order edges and ordered batches;
- all-or-none language-like group limits; and
- whole-group waits which delay an unrelated successor.

Kernel and runtime shared storage remains private to the target adapter.
Linux io_uring SQ/CQ pages, IOCP ports, helper mailboxes, MMIO, and device-owned
DMA state cannot appear as ordinary Whitefoot buffers. The adapter represents
them as atomic or channel protocols. C currently implements that trusted
boundary under the same ownership contract. A future Whitefoot runtime uses
the language's separately justified atomic/channel substrate; it does not
weaken ordinary borrow rules.

## 9. Platform shape

Linux uses real io_uring operations where qualified. The runtime owns the user
endpoint, the kernel owns the other endpoint, and CQ publication returns
operation results and loans.

Windows uses stable operation records containing `OVERLAPPED` and retains
every pointer until the actual IOCP completion. `CancelIoEx` is only a request.
Immediate success is terminal only when the selected native mode proves that
no later packet will arrive.

macOS may use readiness for sockets and a bounded target-only helper for
regular files. A helper owns one typed operation bundle and never receives a
writer function pointer. Its blocking call must finish before it returns any
retained loan.

Target selection does not change source semantics. A depth-one direct call is
an optimized lowering of the same completion call, not a second blocking API.

## 10. Optimizer rules

A call with a nonempty `writes` row is not removed merely because its result is
unused. The compiler may eliminate it only after ordinary closed-state and
escape proof establishes that every affected object is local, unobserved, and
dead. This is the same proof used for dead local memory stores. No system type
gets an `externally_observed` tag in the language.

Repeated `reads` may be combined only when ownership and ordinary state proof
show that the observed state cannot change between them. A live changing
state machine uses `&uniq` and `writes`, so Clock, entropy, receive, cursor, and
similar APIs do not require an environment-driven-read exception.

The backend must lower physical effects honestly. A buffer effect may support
argument-memory attributes. `writes(output)` must not be lowered as though the
bytes storing a native handle were the only memory changed. The target call
remains side-effecting and may access target-private state.

## 11. Implementation boundary

Retain from the current candidate:

- completion-only calls and typed outcomes;
- finite operation records, generation checks, milestones, drain, and wake;
- selective stackless lowering and direct/inline specialization;
- Linux io_uring, macOS typed fallback, and Windows IOCP foundations;
- target code which receives typed operation bundles rather than writer code;
- correctness, hostile-race, sanitizer, and performance harnesses.

Replace in the current candidate:

- REGIONID and capability operands with one formal state-path grammar;
- separate memory and capability effect sets with one path set;
- capability result origins with ordinary move/result/place flow;
- family relation permission with ordinary ownership and loan overlap;
- shared Output and DirectorySource APIs with `own` or `&uniq` state machines;
- ordered batches with dependency-driven loan release; and
- legacy bridge APIs which encode roots, families, or fixed group sizes.

The implemented first slice proves:

1. two parameters sharing one lifetime remain distinct effect subjects;
2. an `own` resource parameter can appear in `reads` and `writes`;
3. field paths preserve unaffected facts without narrowing a loan;
4. two different outputs overlap while two uses of one output do not;
5. a later same-output operation starts when the earlier loan returns, without
   waiting for unrelated I/O;
6. a moved formal owner is still attributed to that formal at release;
7. completion-before-wait, stale generation, duplicate terminal, cancellation,
   and capacity races preserve every owner and loan;
8. pure compute links no completion runtime; and
9. Linux, macOS, and Windows report honest qualification status.

The implementation remains a work-branch candidate. Every component behind
the specification archive gate passes independently; canonical `make check`
intentionally stops because v0.37 is still marked `CANDIDATE`. Activation and
merge require owner approval of the exact revision, after which that exact
revision must pass canonical `make check` with an `ACTIVE` specification
identity.
