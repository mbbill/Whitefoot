# Whitefoot completion I/O implementation plan

Status: OWNER-ACCEPTED DIRECTION, IMPLEMENTED WORK-BRANCH CANDIDATE,
2026-08-26 through 2026-08-27.

This file supersedes the earlier mixed memory/world design in place. Git
history retains that discussion. The selected design has one logical state
model, one ownership system, and one `reads`/`writes` effect system. It has no
`world` domain, `external` effect, `blocks` effect, system-capability class,
logical-root registry, family relation, authority fragment, or `Ordered`
relation.

This file is an implementation plan, not the active language specification.
The active specification remains the one named by `docs/roadmap.md`. Work on a
branch may follow this plan, but merging any revision into `main` still
requires the exact owner approval and `make check` result required by the
repository rules.

The work-branch implementation now realizes this plan. It retained the useful
completion transport, removed the rejected split-state permission machinery,
and migrated the source, specification candidate, compiler, tests, and
maintained programs to one model. The earlier experimental code did not select
the result; every retained part had to satisfy the derivation below.

Call-site snippets in this document are semantic sketches. To keep the state
relation visible, some omit the written loan region after `&` or `&uniq`.
Normative Whitefoot source still writes a live REGIONID on every borrow
expression; this plan changes effect subjects, not borrow syntax.

## 1. Start with the operation the language must explain

Consider one finite write:

```whitefoot
let result = write_once(
  output: &uniq out,
  source: &payload,
  start: 0_u64,
  end: payload_end
);
```

The operation physically reads the payload, changes registers and stack
storage, changes a runtime completion record, submits work to a kernel queue,
changes a socket or file buffer, and may eventually change a device or remote
peer. Listing those physical locations in the source signature would expose
one target implementation and would still be incomplete on another target.

The writer observes a smaller and more stable statement:

```whitefoot
fn write_once['o, 's](
  output: &uniq 'o Output,
  source: &'s buffer<u8>,
  start: own u64,
  end: own u64
) -> result: own Result<u64, IoError>
reads(output, source), writes(output)
```

`source` and `output` name logical state. For a buffer, logical state is
implemented directly by Whitefoot-addressable bytes. For `Output`, logical
state may be implemented across a runtime, kernel, device, and peer. That
representation difference does not create a second language effect.

The signature says three independent things:

```text
&uniq output
    grants temporary exclusive permission to advance output

&source
    grants temporary shared permission to observe source

reads(output, source), writes(output)
    states what this function actually does with those permissions
```

The permission and the behavior must agree, but they are not interchangeable.
Ownership says what the call is allowed to touch and which uses may coexist.
The effect row says what the implementation actually touches.

## 2. The selected model

Whitefoot has one class of program state. A state value may be directly
represented as bytes or hidden behind an opaque system representation. The
language uses the same rules for both.

The complete source-level model is:

```text
own
    one binding owns the value and may move or consume it

&
    a shared loan permits observation and forbids mutation for its duration

&uniq
    an exclusive loan permits mutation for its duration

lifetime
    states how long a loan may remain live

reads(path), writes(path)
    state the exact incoming logical state observed or changed by a function
```

No additional source concept grants I/O permission. `File`, `Output`,
`Listener`, `Clock`, `Factory`, `Permit`, and `Source` are ordinary nominal
types. Their representations may be opaque and their constructors may be
restricted, but the compiler does not mark them as members of a separate
capability kind.

The same ordinary operations apply to them:

```whitefoot
struct Resources {
  input: ReadFile,
  output: Output,
}

let resources = Resources {
  input: move file,
  output: move out,
};
```

Move, borrow, field projection, enum construction, match, return, replacement,
and compiler-derived release retain their usual meaning. An opaque system
value does not acquire a second ownership graph merely because its physical
state crosses an ABI boundary.

This plan keeps two implementation questions separate:

```text
system mapping
    Does the API turn every observable host state and lifecycle transition
    into a sound arrangement of ordinary values, owners, and loans?

source call
    Given that API, does this call pass the language's ordinary type,
    ownership, effect, dataflow, and control-flow checks?
```

An "ordinary call" means the second line. It does not mean a blocking host
call. The source expression has no future, callback, task, worker, or I/O-only
permission rule. The compiler may lower that ordinary call into submission,
suspension, and completion after the normal language checks succeed.

Every host fact that a Whitefoot API lets the program observe belongs to the
logical state of some explicit input or returned value. The target may
encapsulate the physical kernel and device state that implements that value.
It may not expose an observable operation with no ordinary state route and
then repair the omission with backend metadata.

## 3. Why the old split must disappear completely

The rejected design described the same `Output` through two partially
overlapping systems:

```text
ordinary memory
    the bytes storing the handle

world state
    the output sequence behind the handle

capability relations
    a second graph deciding which operations may coexist
```

That split made a single source operation require several unrelated answers.
It also allowed the effect row, borrow checker, root/family analysis, and
runtime ordering machinery to disagree about the same value.

The unified design instead treats `Output` as one opaque state value:

```whitefoot
write_once(output: &uniq out, source: &payload, ...)
// &uniq out       gives exclusive permission
// writes(output)  records the state transition
```

The implementation may read a native handle while performing the transition.
That representation access is internal to `Output`; it is not a second state
domain. `reads` and `writes` remain separate exact facts. This operation
observes the current output state and changes it, so `output` appears in both.
An operation which overwrites state without observing its previous value may
declare only `writes(path)`.

The following constructs therefore have no place in the selected language or
compiler design:

- `memory reads(...)` and `world reads(...)` tags;
- `memory writes(...)` and `world writes(...)` tags;
- `external` and `blocks` effects;
- a system-capability type category;
- capability roots, families, fragments, or coexistence tables;
- `Free`, `Ordered`, and `Exclusive` operation relations;
- Output-specific ordering edges; and
- a hidden global identity used to repair an ownership-incomplete API.

Removing only the syntax while retaining one of these analyses internally
would preserve the same contradiction. The implementation must remove the
second graph as well.

## 4. Effect subjects are state paths, not lifetimes

The previous memory row used a lifetime as an indirect name for storage:

```whitefoot
fn copy['r](
  source: &'r Buffer,
  destination: &uniq 'r Buffer
) -> result: own unit
reads('r), writes('r)
```

Both loans use `'r`. The lifetime says how long they live; it cannot say which
parameter is read and which one is written. Giving every parameter a different
lifetime only hides the ambiguity:

```whitefoot
fn copy['s, 'd](
  source: &'s Buffer,
  destination: &uniq 'd Buffer
) -> result: own unit
reads('s), writes('d)
// 's and 'd are being forced to act as duplicate parameter names.
```

The selected form names the state directly:

```whitefoot
fn copy['r](
  source: &'r Buffer,
  destination: &uniq 'r Buffer
) -> result: own unit
reads(source), writes(destination)
```

One lifetime may now describe both loan durations without merging their
effects. An owned resource, which has no borrow lifetime at its boundary, is
also expressible:

```whitefoot
fn finish_file(
  output: own FileOutput
) -> result: own Result<unit, FinishError>
writes(output)
```

The source grammar accepts a statically resolved subset of ordinary place
syntax rooted at a formal parameter:

```text
state_path       := FORMAL_PARAMETER state_projection*
state_projection := "." STRUCT_FIELD
```

A bare parameter is the common case:

```whitefoot
reads(source)
writes(output)
```

Static fields preserve useful precision:

```whitefoot
struct Record {
  checksum: u64,
  payload: LargeBuffer,
  generation: u64,
}

fn refresh['r](
  record: &uniq 'r Record
) -> result: own unit
reads(record.payload), writes(record.checksum)
```

The first implementation must reuse the compiler's ordinary resolved-place
and projection machinery. It must not create a system-resource-only path
grammar. Enum payloads, dynamic or constant indices, and symbolic ranges need
a separate precision design. Until that design exists, the checker uses the
nearest nameable static ancestor. Exactness is judged at the precision of the
accepted path language.

Only formal-rooted paths appear in a callable boundary. Locals and the result
binder are not effect roots. A local state change is still present in the
function body and may still be observable, but it frames out of the incoming
state footprint in the same way as a write to a locally owned buffer.

## 5. Permission and exact behavior are different judgments

The `refresh` example obtains a unique loan of the whole record:

```whitefoot
fn refresh['r](
  record: &uniq 'r Record
) -> result: own unit
writes(record.checksum)
```

`writes(record.checksum)` does not shrink the loan. Another call cannot borrow
`record.payload` while `refresh` still holds `&uniq record`, because ownership
permission covers the whole argument place.

If the API intends field-level parallelism, it must borrow the field:

```whitefoot
refresh_checksum(value: &uniq record.checksum);
scan_payload(value: &record.payload);
// The two argument places are disjoint, so ordinary borrow rules may coexist.
```

This boundary is mandatory:

```text
ownership and borrows
    decide which state a function may access and which calls may coexist

effect paths
    record which part of that permitted state the function actually accesses
```

An effect never grants permission. It never narrows a loan. It never repairs
two overlapping `&uniq` arguments. It supplies behavior facts for call
checking, framing, proof invalidation, optimization, and scheduling after the
ordinary ownership judgment has succeeded.

A write requires an `own` or `&uniq` route to the affected state. A read may
use `own`, `&uniq`, or `&`. The body checker derives its actual path set and
checks the written row in both directions. Omitting an access and padding the
row with an access the function never performs are both errors.

## 6. Stateful observation also requires unique ownership

A clock cannot be presented as a stable shared object:

```whitefoot
fn now['c](clock: &'c Clock) -> result: own Instant
reads(clock)                         // invalid model
```

Two observations can return different instants because the clock is a state
machine. Presenting it through `&Clock` would allow state to change behind a
shared loan. The correct API advances the state explicitly:

```whitefoot
fn now['c](clock: &uniq 'c Clock) -> result: own Instant
reads(clock), writes(clock)
```

The same rule covers an RNG sequence, directory cursor, accept backlog,
receive stream, read-and-clear register, iterator, and factory. If observation
advances, consumes, acknowledges, clears, or otherwise changes state, the API
uses `own` or `&uniq` and declares `writes`.

A truly non-mutating observation may use a shared loan:

```whitefoot
fn read_at['f, 'd](
  file: &'f ReadFile,
  destination: &uniq 'd buffer<u8>,
  file_offset: own u64,
  start: own u64,
  end: own u64
) -> result: own ReadOutcome
reads(file, destination), writes(destination)
```

This contract is valid only for a qualified positioned-read resource whose
logical state is not advanced by the operation. A sequential file cursor uses
`&uniq` and `writes(file)` instead. MMIO, device files, virtual files, and
read-and-clear state cannot be silently admitted under the positioned-read
contract.

The effect row alone does not grant referential transparency to an opaque
operation. A transformation such as coalescing two target observations also
needs the operation contract and ordinary no-intervening-write proof. An API
whose observation itself changes state must never rely on this qualification;
it uses `writes` from the start.

## 7. There is no ambient state access

An operation which changes or observes process-visible state must receive the
ordinary state value that makes the action possible. These ambient forms are
not valid system APIs:

```whitefoot
now();
random();
open(path: path);
connect(address: address);
spawn(image: image);
notify(bytes: bytes);
```

The corresponding shapes carry explicit ordinary values:

```whitefoot
now(clock: &uniq clock);
reserve_open(factory: &uniq files, quota: &uniq file_quota);
connect(permit: move connect_permit, address: &address);
spawn(permit: move spawn_permit, image: &image);
notify(output: &uniq out, bytes: &bytes);
```

Entry points receive the initial `Args`, directories, streams, factories,
clocks, quotas, and other process state they are allowed to use. Functions may
move, borrow, split, aggregate, return, finish, or recycle those values using
ordinary language rules.

This does not require a `capability` keyword or type kind. An opaque constructor
and affine ownership already make a resource value unforgeable. `Permit` is an
ordinary own value whose API gives it meaning, not an element in a compiler
relation table.

The implicit process heap remains the known convenience exception. Local heap
allocation may use `allocates(heap)` without an incoming allocator value.
Unifying heap, arena, allocator, budget, and recycle semantics is deferred
until completion I/O is finished. The existing `box` exception is also not
expanded to solve I/O. Its design will be revisited separately.

## 8. The frame rule and local state

Callable effects describe incoming state. A write to a locally owned buffer
does not enter the caller-facing row:

```whitefoot
fn build() -> result: own Buffer
allocates(heap)
{
  let value = allocate_buffer(...);
  fill(destination: &uniq value);   // writes local value
  return move value;
}
```

The result carries the state to the caller. The function does not invent a
fictitious lifetime or local effect subject.

System resources follow the same rule, with one important optimizer
obligation. A target call which exhibits `writes(file)` remains a side-effecting
call even when `file` is currently local. The compiler may eliminate the call
only after an ordinary closed-state and escape proof establishes all of these
facts:

```text
the complete affected object is local
no observer can see any intermediate or final state
the object does not escape
the object is dead
the call has no independently observable result, allocation, trap, or release
```

Absence from the caller-facing row is not such a proof. The implementation
does not add `Published`, `Private`, or another hidden state category. System
target calls remain non-removable until the same general proof that justifies
dead local memory stores proves their removal sound.

This rule preserves notification:

```whitefoot
fn notify['o, 's](
  output: &uniq 'o Output,
  bytes: &'s buffer<u8>
) -> result: own unit
reads(bytes), writes(output)
```

The unused `unit` result does not make the call dead. `writes(output)` changes
incoming state and the optimizer cannot remove it. The compiler needs no
blanket `external` marker.

## 9. Existing move identity closes calls, returns, and release

No new language ability is required here. The ordinary type system already
says that `move` transfers one owned value and kills its old name. The compiler
must preserve that existing fact while it computes call effects and release;
it must not add a second resource identity beside the value.

Start with a pass-through function:

```whitefoot
fn pass_file(
  file: own ReadFile
) -> result: own ReadFile
pure
{
  return move file;
}
```

Now use its result:

```whitefoot
fn close_after_pass(
  file: own ReadFile
) -> result: own unit
writes(file)
{
  let same = pass_file(file: move file);
  return unit;   // compiler-derived release of same
}
```

The compiler must retain that `same` contains the state that arrived through
formal parameter `file`. Releasing `same` therefore contributes `writes(file)`
to `close_after_pass`.

The required internal bookkeeping follows existing value flow:

```text
move
    transfers the source provenance to the destination

borrow
    refers to the provenance of the borrowed place

struct or tuple construction
    stores provenance separately for each field

enum construction
    stores provenance in the selected payload

projection or match
    recovers the corresponding field or payload provenance

return
    maps result leaves back to the supplying formal leaves or to fresh values

compiler-derived release
    applies the type's release action to the provenance currently in the place
```

This bookkeeping never grants overlap, ordering, or access. It has no runtime
identity and adds no source rule. It exists only so ordinary call substitution
and compiler-derived release do not forget what the existing move and
aggregate rules already established.

Aggregates must preserve each leaf:

```whitefoot
struct Pair {
  first: ReadFile,
  second: ReadFile,
}
```

If `first` came from formal `left` and `second` came from formal `right`, moving
the `Pair` cannot flatten both fields into one anonymous origin. Projecting
`pair.first` later must recover `left`; projecting `pair.second` must recover
`right`.

Enums retain correlation with the discriminator. A `Left(file)` arm and a
`Right(file)` arm recover their respective formal origins. A control-flow join
may conservatively retain several possible suppliers, but it cannot call the
value fresh or effect-free merely because the exact route is unknown.

Dynamic affine containers retain their existing element-ownership and
occupancy rules. More precise effect projection through a dynamic element is a
later compiler-precision problem; it is not grounds for a new identity system.

### 9.1 Fresh results do not point back to a hidden parent

A factory operation performs its own explicit state transition:

```whitefoot
fn reserve_file['f](
  factory: &uniq 'f FileFactory
) -> result: own FilePermit
reads(factory), writes(factory)

fn open_read['d, 'p](
  permit: own FilePermit,
  root: &'d DirectoryRead,
  path: &'p RelativePath
) -> result: own Result<ReadFile, IoError>
reads(permit, root, path), writes(permit)
```

`reserve_file` writes `factory`. `open_read` writes the consumed `permit` and
returns a fresh ordinary owner on success. The directory and path are shared
selector inputs whose own Whitefoot-visible state does not change; the permit
carries the changing observation occurrence, so no mutation hides behind the
shared `DirectoryRead` borrow. Later writes to that local `ReadFile` do not
propagate back to `factory`. There is no fresh-child ancestry, common root, or
parent coordination domain hidden in the compiler.

The reservation is total because it is proof-only. It does not promise a
native descriptor, handle-table slot, kernel allocation, or host quota.
`ResourceExhausted` therefore remains a typed outcome of the open attempt. The
one-shot permit is consumed on success and recoverable failure in this first
slice, and the backend erases it before the native open ABI.

Two short unique factory loans may create two independent permit owners. Once
each inline reservation returns, neither factory loan remains attached to the
long operation. The two opens may then share one `DirectoryRead` and overlap:

```whitefoot
let left_permit = reserve_file(factory: &uniq files);
let right_permit = reserve_file(factory: &uniq files);

open_read(permit: move left_permit, root: &cwd, path: &left);
open_read(permit: move right_permit, root: &cwd, path: &right);
```

If a user helper accepts `&uniq FileFactory` and retains that parameter while
it performs a long open, the helper has explicitly selected a long exclusive
loan. The system operation has not hidden one. Parallel code instead reserves
before the helper and passes owned permits, or threads an owned factory through
an explicit result. Ordinary signature inspection exposes the difference.

An enclosing wrapper already receives the ordinary effects of the calls it
actually makes. If it reserves through an incoming factory, its row contains
`writes(factory)`. If it receives a permit directly, its row contains
`writes(permit)`. Subsequent changes to a locally owned result frame locally,
while the target calls remain side-effecting under the optimizer rule in
Section 8.

This is the point where provenance differs from the rejected root model.
Provenance follows the identity of an ordinary value through renaming and
release. It does not connect independent fresh values to an ancestor and then
use that ancestry to serialize them.

### 9.2 Release must be complete on every normal edge

Resource release is a real state transition:

```whitefoot
fn discard(
  file: own ReadFile
) -> result: own unit
writes(file)
{
  return unit;   // release is inserted by the compiler
}
```

The checker derives release effects for fallthrough, explicit return,
propagation, `give`, `break`, loop exits, match residuals, displaced values,
and partially built aggregates. Moving or returning the owner suppresses the
release on that edge. A normal edge may not lose an owner or release it twice.

A trap is fail-stop and performs no unwinding. Its impossible path does not
add release machinery to the correct path.

## 10. Ordinary ownership decides I/O concurrency

Two independent reads can overlap because their destination loans and file
owners do not conflict:

```whitefoot
let a = read_at(file: &file_a, destination: &uniq left_bytes, ...);
let b = read_at(file: &file_b, destination: &uniq right_bytes, ...);
// Both shared file loans and both unique destination loans are disjoint.
```

Two writes to distinct owners can overlap:

```whitefoot
let a = write_once(output: &uniq out, source: &left, ...);
let b = write_once(output: &uniq err, source: &right, ...);
// out and err are disjoint places.
```

Two writes to the same owner cannot hold unique loans simultaneously:

```whitefoot
let a = write_once(output: &uniq out, source: &left, ...);
let b = write_once(output: &uniq out, source: &right, ...);
// b waits until a's completion returns the uniq loan of out.
```

No `Ordered(OutputBytes)` relation is needed. The second call cannot be
submitted while the first target operation can still touch `out`.

An unrelated slow operation must not delay later reuse of `out`:

```whitefoot
let a = write_once(output: &uniq out, source: &left, ...);
let b = write_once(output: &uniq err, source: &slow, ...);
let c = write_once(output: &uniq out, source: &last, ...);
```

The required execution is:

```text
submit a using out
submit b using err

a completes
    the target returns the uniq loan of out
    c becomes ready immediately

b may still be running while c uses out
```

The scheduler needs ordinary dependency-driven activation rather than one
whole-batch join. It may encode dependencies internally, but those dependencies
come from dataflow, control flow, resolved places, and loans. It does not
collect an Output-specific sequence graph.

The complete overlap judgment remains the compiler's general one:

- the later call does not need an earlier result before submission;
- argument evaluation preserves the same observations;
- affected state paths do not conflict;
- loans that were sequentially live may legally be live together;
- consumed places remain single-use; and
- a typed early exit does not allow work that sequential execution would skip.

Written claims do not add a correct-path serialization condition. In a
correctly reviewed program a claim cannot fail. The impossible fail-stop path
does not select scheduling, completion order, cleanup, or rollback behavior.

Aliases introduced outside the mapped Whitefoot program do not merge ordinary
owners. If two paths are made hard links after entry, or stdout and stderr are
redirected to one host endpoint, operations through their independently
supplied owners may overlap. Every language faces that external alias. The
runtime itself may not knowingly manufacture unsound independent owners for
one lifecycle object; its constructors and split operations must satisfy their
ordinary type contracts.

## 11. Structure expresses independent resource parts

If a resource really contains independent state, the API returns ordinary
structure:

```whitefoot
struct TcpParts {
  receive: TcpReceive,
  send: TcpSend,
}
```

Receiving uniquely borrows `receive`; sending uniquely borrows `send`:

```whitefoot
receive_once(input: &uniq connection.receive, ...);
send_once(output: &uniq connection.send, source: &payload, ...);
```

The fields are disjoint places, so existing ownership permits overlap. Two
same-direction operations use the same field and therefore serialize.

A split is sound only if each returned owner has a complete lifecycle. If
shutdown, reset, or close can invalidate both parts, the API must retain an
owned parent that controls both, return a join value, or provide a consuming
reunite operation. The compiler does not repair an incomplete split with a
hidden common root.

Communicating endpoints are also ordinary owned state machines. A producer and
consumer may affect what the other later observes, so their type contract must
state capacity, ordering, overflow, and termination. Each operation still uses
`own` or `&uniq`; communication does not license shared interior mutation.

## 12. Quota and permits expose real finite state

Recoverable, program-controlled capacity is state and must have an owner. A
hidden process-global pool would reintroduce mutation outside the function's
arguments.

The efficient shape separates short admission from long target work:

```whitefoot
let permits = reserve_connects(
  factory: &uniq network_factory,
  quota: &uniq socket_quota,
  count: 2_u64
);

let first = connect(permit: move permits.first, peer: &peer_a);
let second = connect(permit: move permits.second, peer: &peer_b);
```

Reservation writes `network_factory` and `socket_quota`. After it returns, the
two `Permit` values are independent ordinary owners. The long handshakes may
overlap without retaining either unique factory loan.

Every result accounts for its permit:

```whitefoot
match first {
  Connected { connection } => {
    let finished = finish(connection: move connection);
    // finished returns or consumes the embedded credit as declared.
  }
  Failed { error, permit } => {
    // The unused permit is available again.
  }
}
```

The target cannot promise that a logical permit reserves every host-wide
resource. File descriptor tables, kernel memory, ephemeral ports, and limits
changed by another process may still fail. An API distinguishes exactly
reserved Whitefoot/runtime capacity from honest target exhaustion.

Automatic release cannot secretly return credit to a separately held quota.
A resource contract chooses one explicit disposition:

- implicit release closes the resource and burns the embedded credit;
- explicit `finish` returns the credit as an owned result; or
- release sends the credit through an owned return endpoint with a matching
  collector.

Success, error, cancellation, partial progress, and release must dispose of
every owner and credit exactly once. This cost stays visible because the
computation really has that lifecycle.

## 13. Completion is an ownership transfer

An I/O call is not a blocking call disguised by the compiler. It is a
completion operation from the start. Before submission, the compiler-owned
frame holds one complete bundle:

```text
operation kind and typed metadata
resource owner or resource loan
payload owners or payload loans
inaccessible result storage
target admission storage
```

Submission changes the owner of that bundle:

```text
PREPARED
    writer frame owns every value and loan
        |
        | target accepts the complete bundle atomically
        v
ACCEPTED
    target owns the operation; writer cannot touch retained state
        |
        | target performs its final access and publishes the result
        v
TERMINAL
    result bytes are initialized; target will never touch retained state again
        |
        | runtime drains the completion
        v
DRAINED
    result and loans return to compiler-owned writer frames
```

Transient exhaustion of a completion record, submission queue entry, or
helper credit occurs before `ACCEPTED`. A failed admission transfers nothing.
The runtime waits for capacity and retries internally; it does not expose a
fake `WouldBlock` I/O result.

A qualified operation may publish finer monotonic facts when they improve
scheduling:

```text
result_ready(component)
loan_released(formal path)
terminal
```

`result_ready` means the named result bytes are initialized and immutable.
`loan_released(path)` means the target will never again use that borrowed
state path. Different payload and resource paths may be released at different
times.
`terminal` includes every required result and release fact and promises no
further publication.

For a sequential resource, release of its unique loan also means that the logical
state transition has reached the point after which a later operation cannot
overtake it. A backend may copy payload bytes into private kernel storage and
release the payload earlier. It may not return the unique resource loan while
the operation's position in that resource is still undecided. This condition
belongs to the ordinary resource-loan return contract; it is not a separate
ordering relation.

These milestones return authority previously transferred to the target. They
do not mint new permission. A later operation on the same resource can start
only after the earlier target has returned the required owner or unique loan.
This rule supplies sequencing without `Ordered`, an order-commit relation, or
a family-specific edge.

The source call exposes its ordinary result only when its ownership contract
is satisfied. Internal split milestones may wake unrelated dependent work
earlier, but they cannot let writer code touch a retained buffer or resource.

## 14. The target boundary follows the same ownership contract

Today, part of the runtime and target adapter is C. Treating it as though it
followed Whitefoot ownership is a trust obligation, not a language fiction.
The ABI contract must prove or test all of these statements:

- target acceptance receives each owner or loan exactly once;
- retained pointers remain valid until the declared release milestone;
- the target does not access payload or resource state after releasing it;
- result bytes are fully initialized before publication;
- one operation reaches terminal exactly once;
- cancellation and failure still dispose of every owner exactly once;
- a stale completion token cannot publish into a reused record; and
- completion never invokes writer code.

If the target layer were later rewritten in Whitefoot, the public API and
effect rows would remain unchanged. The compiler would prove rules that are
currently established by target contracts, audits, and tests.

Kernel rings, IOCP ports, helper mailboxes, interrupt state, and device queues
are target-private implementation state. The writer transfers an operation to
the target and holds no shared loan of this storage while the target changes
it. When multiple target actors communicate through one ring or queue, they
use a typed trusted atomic or channel protocol. That protocol belongs to the
general runtime trust boundary; it is never exposed as an ordinary mutable
slice behind a Whitefoot shared borrow.

This is not hidden writer-visible interior mutation. The writer no longer owns
the transferred operation, and the target-private concurrent structure has an
explicit trusted protocol. The existing `box` exception is not used to bless
arbitrary I/O objects.

## 15. Completion does not require a dedicated I/O thread

Completion describes ownership and notification, not a fixed thread layout.
On a single-threaded runtime with native kernel progress, execution can be:

```text
submit operation
run another ready Whitefoot frame
if no frame is ready, wait for target completion on the same OS thread
drain completion
make the waiting frame runnable
```

No dedicated user-space I/O worker is required. The kernel progresses the
operation while the Whitefoot thread computes or waits.

With several scheduler threads, one or more threads may drain the native
completion source according to measured target policy. A completion first
publishes state and a wake epoch; a compiler-owned frame runs only after the
runtime drains that publication. The kernel or target never executes the
frame as a callback.

Some macOS file facilities or foreign libraries may require a bounded helper
because the available host call can block. A helper is a target implementation
choice under the same completion API. Linux io_uring and Windows IOCP use their
native completion paths where qualification succeeds. Source code does not
select a worker, readiness mode, blocking mode, or queue depth.

Depth-one direct or inline execution is a specialization of completion
semantics. The target may finish before the submit path returns, but it still
publishes the same typed result and ownership facts. Pure compute programs
must not link or initialize completion machinery they never use.

Whether completion outperforms the best blocking shape at depth one is an
experimental question. Submission, publication, and drain add real cost. The
implementation must measure that three-stage cost and retain a qualified
direct specialization when it wins without adding a second language API.

## 16. Runtime rules

The completion runtime is separate from the pre-existing compute-par runtime.
An I/O completion record is not a compute `par` slot, and a fixed record count
is not a source-language group limit.

The runtime must retain or implement:

- generation validation before result storage changes;
- exactly one terminal publisher;
- release publication before a waiting frame becomes runnable;
- acquire drain before result or returned loans become visible;
- an exact-token consume-to-park handshake with no lost wake;
- correct one-waiter and multi-waiter wake behavior;
- bounded queues and records with complete pre-accept rollback;
- per-component release only where the target contract proves last access;
- no writer function pointer in a target request; and
- zero-helper progress where the target can provide it.

A record may be reused only after terminal has been drained, the result has
moved to stable frame storage, and no target, event, waiter, or dependent frame
can still reach the old generation. Copied stale token bits may exist, but they
must fail generation validation before observing or modifying the new record.

Bounded capacity is runtime policy. The implementation must compare whole-batch
admission, streaming admission, per-lane caches, and batch refill. It may not
turn an experimental count such as 16 or 64 into a language rule.

The scheduler must release one completed dependency without waiting for an
unrelated member of the same submission group. Bounded successor activation
must also avoid holding completed records while waiting for new capacity. The
runtime may reserve successor capacity, move results into frame storage before
releasing records, reuse a record along a chain, or prove another bounded
protocol.

## 17. API patterns

### 17.1 Finite one-shot operations

Positioned reads, finite writes, datagram sends, one-shot timers, and finite
host queries use ordinary calls. The compiler may overlap them when dataflow,
state paths, ownership, loans, and typed exits permit.

### 17.2 Stateful sequential resources

Output streams, sequential files, clocks, RNGs, directory cursors, receive
streams, and other advancing machines use `own` or `&uniq`:

```whitefoot
read_next(file: &uniq file, destination: &uniq bytes, ...);
now(clock: &uniq clock);
send(output: &uniq connection.send, source: &payload, ...);
```

One explicit vector or batch operation may hold one unique loan while the
target performs several physical operations. This can reach `writev`, linked
requests, or another native batch without introducing shared ordered mutation.

### 17.3 Persistent sources

Listeners, periodic timers, file monitoring, signals, hotplug, multishot
receive, and unknown-size streams use an owned `Source` or `Subscription`:

```whitefoot
fn next['s](
  source: &uniq 's Source
) -> result: own NextOutcome
reads(source), writes(source)
```

The source owns its target registration, event capacity, overflow policy, and
terminal lifecycle. Each returned shot owns its payload and any recycle value.
`finish(source)` cannot free or reuse storage still held by an outstanding
shot. The API must require recycling, consume an explicit collection of
outstanding values, or specify an owned abandon path. A hidden shared reference
count does not replace that ownership.

### 17.4 Request and response

A host callback that needs a writer decision becomes an owned `Request` plus
an owned response value delivered through a `Source`. Writer code executes on
the Whitefoot scheduler. A foreign protocol that requires immediate reentrant
writer execution on the original callback stack must be adapted through an
explicit queue or remain unsupported.

### 17.5 Finish, recycle, and cancellation

Finish, close, durable commit, abandon, recycle, quota handback, peer
acknowledgement, and detach are separate consuming operations when their
results differ. Compiler-derived release cannot silently promise a stronger
successful transition than its type contract provides.

The default finite call has no writer-visible pending handle. Cancellation is
available only when an API returns or accepts ordinary owned cancellation
state. The target defines the race between cancellation and completion. Every
outcome, including partial progress, returns or consumes the payloads,
resource, cancellation state, and permits exactly once. An acknowledgement
that arrives before the target's last payload access cannot return the payload
loan.

### 17.6 Foreign libraries

A foreign adapter receives ordinary owned or borrowed values for every global,
registration, retained allocation, callback queue, and signal disposition it
uses. A synchronous adapter proves that it retains no pointer after return. An
asynchronous adapter receives a closed owned operation bundle and publishes
events through a `Source`.

Unregister is complete only after the foreign side can no longer call or retain
the pointer. If an in-process library cannot expose or satisfy that contract,
the mapping is unsupported or moved behind explicit process isolation.

## 18. Compiler implementation

### 18.1 Parser and callable boundary

1. Remove REGIONID operands from `reads` and `writes`.
2. Accept only formal-rooted static state paths.
3. Remove memory/world tags and the `external` and `blocks` alternatives.
4. Keep `allocates(...)` and `traps` as their existing non-state effects.
5. Carry the complete state row and compiler-derived result-state routing in
   every direct callable boundary. The current language has no indirect call.

### 18.2 Effect derivation and substitution

1. Reuse the resolved-place, borrow, reborrow, slice-origin, field-overlap, and
   aggregate projection machinery already required for ordinary memory.
2. Attribute a direct access to the current formal-rooted state path, or frame
   it locally when its ultimate owner is local.
3. At a call, substitute each callee formal path with the resolved actual place
   and then project it to the current function's formals.
4. Reuse the existing ownership value flow through move, aggregate, enum,
   match, result, replacement, recursion, and release when attributing effects.
5. Include compiler-derived release on every conservative normal-control edge.
6. Check the written row against the derived row in both directions.
7. Surface unsupported provenance or projection precisely; do not fall back to
   one global state or an empty row.

### 18.3 Permission and scheduling

1. Remove capability-root, family-fragment, relation-pair, and `Ordered`
   permission checks.
2. Use ordinary places, borrow modes, consumed values, effects, dataflow,
   control flow, and early exits for direct system calls and user calls alike.
3. Let completion return exact owners and loans at qualified milestones.
4. Activate a newly unblocked call without waiting for unrelated in-flight
   work.
5. Keep the fail-stop claim path out of correct-path scheduling and runtime
   state.

### 18.4 Lowering

1. Keep completion enabled independently of compute `--par`.
2. Generalize stackless suspension across branches, loops, multiple suspension
   sites, and non-tail suspended children.
3. Preserve one direct source-call model while selecting native completion,
   bounded helper, readiness, or inline target lowering.
4. Fuse same-owner sequences only while one operation owns the unique state
   and the target preserves exact partial and error semantics.
5. Erase proof-only ownership facts and avoid passing one native handle twice
   under different conceptual roles.
6. Preserve pure-compute zero-link and zero-loop-tax behavior.

## 19. Target implementation and evidence

### 19.1 Linux

Use real io_uring submission and completion where the operation qualifies.
Test stale generations, result publication, short I/O, cancellation, queue
pressure, and wake races against the actual kernel path. Compare registered
resources, linking, batching, multishot, and direct depth-one execution.

### 19.2 macOS

Select the best qualified native or bounded-helper path per operation. A host
call that can block the sole scheduler while another Whitefoot action is
needed to unblock it cannot run inline. Helper count and drain ownership are
target policy, not source modes. Compare readiness, dispatch, bounded helper,
direct, and completion paths with the same source API.

### 19.3 Windows

Use IOCP and `OVERLAPPED` only after real Windows execution validates retained
storage, cancellation, terminal publication, wake behavior, and multi-waiter
progress. Cross-compilation is useful evidence but is not runtime
qualification.

### 19.4 The C trust boundary

Build target probes around the exact C ABI contract. Compile with strict
warnings and run ASan, UBSan, and TSan where supported. The evidence must cover
the ownership transfer, not merely successful bytes on a happy path. A future
Whitefoot target implementation should be able to replace C without changing
source signatures or effects.

## 20. Verification plan

### 20.1 Functional API correctness

Every shipped API receives a contract matrix covering:

```text
success and empty input
full, short, partial, EOF, and terminal progress
typed host and resource failure
invalid target representation
exact result initialization
destination bytes and untouched tails
owner and permit disposition on every result
finish, close, abandon, recycle, and implicit release
```

File tests cover positioned read, sequential read, write, open/create,
metadata promised by the API, finish, and close. Network tests cover TCP and
UDP construction, connect, listen, accept, send, receive, shutdown, peer
termination, and partial progress. Source-style APIs test event delivery,
overflow, recycling, unregister, and terminal behavior. Clock, timer,
directory, process, and foreign mappings receive the same lifecycle treatment
when they enter scope.

### 20.2 Ownership and concurrency safety

Positive tests prove that independent owners and disjoint fields overlap.
Negative tests prove that one resource cannot acquire two live unique loans,
that retained payloads cannot be mutated, that moved owners cannot be reused,
and that finish cannot race an outstanding operation.

Deterministic hostile schedules cover:

```text
completion before wait
completion between the wait check and park
stale generation publication
duplicate or post-terminal publication
result publication before initialization
payload or resource release before last target access
one-waiter and multi-waiter wake
record and queue capacity exhaustion
partial I/O and cancellation races
single-thread and multiple-thread progress
dependent activation while unrelated work remains in flight
```

Long randomized stress supplements these schedules. It does not replace them.

### 20.3 Performance

Compare against the best native C or Rust shape, not a deliberately weak
blocking loop:

```text
depth 1 through target saturation
small through large payloads
single and multiple scheduler threads
independent resources and same-resource batches
direct, inline, native completion, readiness, and bounded helper paths
registered and unregistered resources
batch, linked, and multishot operations where applicable
CPU cycles, atomics, cache misses, syscalls, and context switches
p50 and p99 latency
live retained bytes and buffer-byte-time
frame size, ABI arguments, and register spills
```

The experiments must isolate the cost of prepare, submit, publish, drain, and
resume. If completion misses the native ceiling, the responsible API,
ownership granularity, lowering, or runtime handoff is reopened. Supporting
only completion semantics does not predetermine the winning target mechanism.

## 21. Current experimental implementation

### Retain where evidence confirms it

- completion-only source behavior;
- ordinary call syntax with typed results;
- no writer callback;
- no correct-path false-claim cost;
- generation-safe completion records;
- one terminal publication and drain-before-resume;
- lost-wake and multi-waiter fixes;
- pure-compute completion-runtime link boundaries;
- useful Linux, macOS, and Windows target work; and
- measurements that compare direct, helper, completion, and wake paths.

### Delete or replace

- REGIONID effect subjects;
- mixed REGIONID and parameter operands;
- memory/world effect tags;
- `external` and `blocks`;
- system-capability type flags and shape checks;
- logical roots and fresh-child parent ancestry;
- family fragments and pairwise coexistence relations;
- shared Output or shared mutable directory/source APIs;
- Output-specific `Ordered` batches and sequence edges;
- special limits derived from fixed completion record arrays; and
- group joins that retain unrelated completed ownership.

### Generalize

- resolved state paths across ordinary memory and opaque resource values;
- ordinary ownership value flow retained through every affine language
  construct for effect and release attribution;
- compiler-derived release attribution;
- stackless suspension and dependency-driven activation;
- exact per-component completion release;
- factory, permit, quota, Source, finish, and recycle lifecycles; and
- API, runtime, and target testing across every result and platform.

## 22. Implementation sequence

1. Rewrite the active design and derived documentation so no implementer can
   mistake the mixed state model for the selected one.
2. Change the effect grammar to formal-rooted static state paths and regenerate
   parser and syntax evidence.
3. Generalize resolved-place substitution so ordinary memory and opaque
   resource arguments take the same path.
4. Make effect and release attribution preserve the ownership identity already
   established by move, results, aggregates, branches, and containers.
5. Remove capability roots, families, fragments, relation tables, and
   Output-specific ordering from semantic checking and IR.
6. Correct every system API to use ordinary `own`, `&`, and `&uniq` state
   machines, starting with positioned file read and finite write.
7. Carry exact result and loan release through callable summaries and
   stackless lowering.
8. Retain and harden the generic completion record, admission, drain, and wake
   protocol while deleting the rejected bridge machinery.
9. Qualify Linux and macOS paths, then execute and finish the x86-64 MSVC
   Windows row. Native run
   [33304333316](https://github.com/mbbill/Whitefoot/actions/runs/33304333316)
   closes that row for the current file-operation slice; it does not claim a
   Windows compute pool or performance result.
10. Complete functional matrices, hostile schedules, stress, sanitizers,
    maintained programs, and native performance comparisons.
11. Run canonical `make check` on the exact proposed revision and obtain owner
    approval before any merge to `main`.

The branch may remain temporarily inconsistent while these steps are in
progress. Unsupported intermediate combinations must fail explicitly. They
must never be accepted through an empty effect, a global fallback state, or a
name-based special case.

## 23. Remaining API and implementation work

No open question below reopens the unified state model. Each question may
block one API or optimization while the core implementation proceeds.

1. Reusable quota, finish, and credit-return signatures beyond the fixed first
   file slice. That slice already has total `reserve_file`, one-shot burned
   `FilePermit`, shared `DirectoryRead` selectors, typed host exhaustion, and
   proof-only native erasure.
2. The structural parent, join value, or reunite operation for long-lived TCP
   send and receive parts.
3. Persistent `Source` pool ownership, outstanding-shot recycling, overflow,
   and terminal contracts.
4. Cancellation and deadline result shapes, partial-progress accounting, and
   exact owner disposition.
5. Dynamic affine-container provenance and the useful precision boundary for
   data-dependent indices and ranges.
6. Complete direct-call result-state routing and per-path completion release.
7. Bounded record and successor admission without hold-and-wait.
8. Stackless frame placement, reuse, direct-path erasure, and ABI cost.
9. Exact target choices and measured thresholds on Linux, macOS, and Windows.
10. Foreign callbacks, signals, MMIO, DMA, GPU, and shared-memory mappings.
    Until a typed trusted atomic or channel protocol owns their concurrent
    access, they are not exposed as ordinary Whitefoot borrows.
11. The later redesign of `box` and the later unification of heap, arena,
    allocator, budget, and recycle semantics.

## 24. Completion criteria

The I/O design is implemented only when all of these statements are true:

- source has one untagged `reads`/`writes` state row;
- effects name formal-rooted static state paths and never lifetimes;
- file opens consume one-shot ordinary permits, share stable directory
  selectors, retain no factory loan, and pass no permit to the native ABI;
- `own`, `&`, and `&uniq` provide all writer-visible authority;
- every stateful observation, including clocks and cursors, uses `own` or
  `&uniq` and declares `writes`;
- no system API obtains ambient state without an ordinary input owner or loan;
- no capability type class, root, family, fragment, or `Ordered` relation
  participates in checking, IR, lowering, or runtime permission;
- effect attribution does not forget the ownership identity already established
  by move, result, aggregate, branch, container, and release;
- fresh factory results are independent ordinary owners and do not propagate
  later child effects back to a hidden parent;
- compiler-derived release contributes the exact formal state paths on every
  normal edge;
- a target call is not removed merely because its result is unused or its
  current state object is local;
- ordinary ownership alone prevents overlapping mutation and releases newly
  available state without waiting for unrelated work;
- completion transfers every owner and loan to the target and returns each one
  exactly once after last access;
- target-private concurrency uses qualified atomics or channels and never
  exposes mutable storage behind a writer shared loan;
- no target invokes writer code;
- no correct execution pays for a false claim path;
- pure compute links and executes no completion machinery;
- all three hosted targets report honest qualification based on native
  execution evidence;
- API correctness, hostile concurrency, stress, sanitizer, conformance, and
  maintained-program tests pass;
- measured fast paths match the best relevant native shape or reopen the
  responsible design; and
- the exact revision passes `make check` and receives owner approval before it
  reaches `main`.

## 25. Deferred successor: allocation and `box`

The implicit heap and the existing `box` rule are the two deliberate exceptions
left outside this I/O closure. They are not precedents for hidden I/O state.
After completion I/O closes, the next investigation should unify heap, arena,
explicit allocator state, fallible capacity, budgets, recycling, and the
remaining interior-mutation exception using the same first-principles method.
