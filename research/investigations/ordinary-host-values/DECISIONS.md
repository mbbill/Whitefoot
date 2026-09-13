# Selected ordinary interfaces

The owner selected all seven recommendations for C2, with PAR-3 deleted
outright and no suspension classification retained. The alternatives below
preserve the reasons for those selections. Baseline source coordinates use
`20043721`; [DESIGN.md](DESIGN.md) fixes the common ordinary-value boundary.
The combined publication is v0.55 over main's v0.54. No option below reinstates
an external declaration class, semantic ID, finalizer, suspension contract or
trusted proof source.

## 1. Which objects must be consumed?

| Current nominal | Recommendation | Other ordinary choice and cost |
| --- | --- | --- |
| `ReadFile`, `DirectoryRead`, `DirectorySource`, `TcpListener`, `TcpReceive`, `TcpSend` | Declare `linear`; explicit consuming close, or transfer to another owner, on every normal exit | Affine with empty drop permits losing the native handle. It is safe only for an API whose intentional outcome is abandonment; it cannot silently mean close. |
| `TcpConnection` | Ordinary struct of the two linear direction owners; linearity follows its fields | An opaque linear connection hides the fields and requires ordinary splitting operations; see decision 4. |
| `Args`, `HostString`, `RelativePath` | Affine, empty drop, immutable invocation backing valid for the whole invocation | Independently owned dynamic text requires ordinary store-backed data and explicit release; do not reuse the current program-lifetime lease representation for it. |
| `InputStream`, `OutputStream` | Affine, empty drop for borrowed standard-stream access; this API does not own the process stream's close | Make a distinct owning stream linear if an API later transfers responsibility for closing it. Do not infer that distinction from an OS descriptor. |
| `SocketAddress`, `ExitStatus` | Keep the current affine classification and empty drop for this removal | A later ordinary API can use transparent copy data or return `u8`; changing copyability is unnecessary to C1. |
| `HandleFactory` | Affine, empty drop if it is only the existing quota/accounting value; it never closes outstanding owners | If the selected implementation makes it own reclaimable backing, represent that backing through the ordinary store model, or make the factory linear with explicit consuming shutdown. No implicit opaque cleanup. |
| `HandlePermit` | Retire with decision 2's simple interface | If retained, make it an ordinary linear reservation: explicit return/cancel or consumption by open. Dropping it must not return credit implicitly. |

Current evidence: all opaque types are affine at
`spec/kernel-spec.md:2740–2750`; their different native release actions are at
`:3025–3067`. Ordinary modifier linearity, its closure through owned fields,
and the scope-exit obligation already exist at `:882–930` (PROV-6). This
selection changes source cleanup obligations substantially: adding `linear`
without migrating every outcome and partial move is not an implementation.

## 2. Factory parameter or one-shot permits?

**Recommend direct `&uniq HandleFactory` on open and close for the first
ordinary API.** The factory is an explicit state operand, not a special
capability kind. Refusal returns its temporary borrow and leaves the existing
accounting state available; consuming close also receives the factory and
returns an ordinary outcome. The selected native implementation must make
both transitions true of that passed value. A moved handle has no inferred
parent link to the factory.

The alternative is an ordinary reservation API: a short exclusive factory
call produces a linear reservation, opening consumes it, refusal returns it,
and an explicit cancellation/close operation returns credit to a passed
factory. This can shorten the factory loan and permit concurrent opens. It
costs additional source outcomes and explicit state plumbing. A reservation
accounts for the resource it actually represents; it is not evidence that
two accesses to some other shared mutable namespace are independent.

The direct form holds the factory loan until the ordinary call returns. Two
opens through that factory therefore serialize under OWN-5, even if their
rows name different selectors. That is a real performance cost and must be
measured against the two-permit witness
`tests/conformance/cases/accept-sysfile-two-permits-shared-directory.wf:1`.
It is not evidence that the value model cannot express opening a file.
If concurrency is required at this interface now, select reservations before
implementation; do not retain SYS-2's early loan-release milestones.

All state operands require an alias audit. For example, `tcp_accept` must
advance an exclusive listener (or an explicit queue state), and a read that
samples changing contents must advance an exclusive read state. A shared
selector can remain shared only where its declared state stays stable during
the loan. The current `tcp_accept` and `read_at` signatures at
`spec/kernel-spec.md:2864,2879` cannot be copied as evidence that the ordinary
boundary has been checked. Native physical aliases grant no additional
language permission; an API needing shared changing state must pass that
state explicitly. Test two handles to one file and redirected streams.

## 3. Entry arguments

**Recommend one ordinary prelude `Inputs` struct for the six non-provider
inputs, plus a separate ordinary `Heap` parameter where a program needs it.**
Use ordinary fields `args`, `cwd`, `stdout`, `stderr`, `handles`, and `stdin`.
The build's launcher constructs these arguments and calls the selected
ordinary function. There is no `command`, label ordinal, special main type,
entry-only contract ban, or source-call ban. A program needing only a subset
can use an ordinary launcher wrapper that handles the remaining owners.

The alternative is separate ordinary parameters throughout. It minimizes
aggregate moves and ownership plumbing for small programs, but repeats the
input list. Neither option requires source grammar beyond GRAM-2's existing
struct and function declarations. Selection of a symbol and its native
launcher ABI belongs to the build, not FN-7 acceptance.

The current seven-input table is at `spec/kernel-spec.md:1740–1769`.
`Heap` cannot be placed in the struct under the ordinary provider/storage
rules (`:850–857,1291–1306`), and a view cannot be smuggled into it either.
The entry-heap brand must use the ordinary function region instantiation;
remove the unspellable, command-selected exception, including BLK-4's check
at `:1186`. Keep the existing provider-availability and confinement checks.

## 4. TCP pairing

**Recommend ordinary public `TcpConnection` fields and explicit consuming
direction closes**, each with the required ordinary host state parameter.
Any convenience `close_connection` is an ordinary WF wrapper over those
calls. Its behavior must work for any well-typed pair, not only a pair minted
by one native call. Native shared descriptor bookkeeping is implementation
state of the explicit host object; it cannot be a source pairing invariant.

The alternative is an opaque linear connection and an ordinary operation
that splits it into two direction owners. It removes public reconstruction,
but still needs total cleanup after splitting. No option gets a special
constructor prohibition on a public struct.

The current rule permits fields but installs no constructor
(`spec/kernel-spec.md:2759–2767`), and close assumes one matched connection
(`:3331–3335`; `compiler/src/backend/emitter/system.rs:3584`). The falsifier
is to destructure two connections and construct `TcpConnection` with the
first receive half and the second send half. Ordinary OWN-1 allows those two
moves. Returning exactly one pair's credit for that mixed object is an API
bug. It is not a counterexample to ordinary structs.

## 5. What completion machinery remains?

**Recommend retaining the native completion engines and generic scheduler
behind the ordinary call ABI, with loans lasting until return.** The compiler
must emit the same direct call/task form for a WF body and a linked body
with the same signature. Runtime suspension is an implementation of that
call; it does not create an acceptance summary or an early source result.

The simpler alternative is initially blocking calls on the executing worker,
with ordinary PAR-1/PAR-2 scheduling where applicable. It cuts the migration
surface but can lose throughput or exhaust workers. This is a build/runtime
choice; neither choice changes source acceptance.

Delete PAR-3's current special cut at the first `may-suspend` operation and
its SYS-8 byte coverage/resource retry rules (`spec/kernel-spec.md:2610–2636`).
Preserve PAR-1/PAR-2's ordinary interference conditions after removing target
summary requirements. Generalizing staged loop overlap to a source-call
boundary is a new parallelism design, so C1 does not introduce it. Losing
that optimization is an explicit cost of this recommendation, to be measured
on wfgrep and the existing staged witnesses, not hidden by changing their
source verdicts. Private continuation/frame code may be reused only if it
does not recover the deleted semantic permission.

## 6. Owned effects and function formals

**Recommend ordinary storage-place effects, with no value-history effect
root.** A callee's row projects onto actual places. A value moved into local
storage does not make that storage another name for an earlier effect root;
borrow/view referent provenance continues to resolve real storage normally.
No summary follows an owned result back to an input. This applies uniformly
to a `Box`, a run, and an opaque nominal.

There is no compliant alternative that keeps FN-1's routing under another
name. The choice for the owner is to adopt this consequence together with
the API migration, or to identify a concrete ordinary program whose required
behavior needs a different *existing* interface shape. An ordinary `&uniq`
parameter keeps the mutation at a stable caller place without a round trip.
At `replace deref(dst) = move src;` SET-2 still exhibits a read and write of
`dst`; subsequently reading it adds no `src` effect merely because of value
history. Pure owned conversions remain legitimate value transformations.

Remove FN-4's non-copy-formal-result freshness restriction and selected
actual referent-routing check (`spec/kernel-spec.md:1713–1715`), which otherwise
require the summary being deleted. Keep exact signatures, formal-row coverage,
structural contract equality, ordinary borrow result provenance and finite
instantiation. Prelude `fn_sig`s must be eligible behavior arguments by those
same checks, without the current source-body-only exclusion. A bodyless
prelude declaration supplies its PRE-1 contract; C1 does not pretend that
FN-9 verified a nonexistent WF body.

Remove EFF-3's suspension qualifier, but keep its ordinary ownership,
termination, effects and surviving-observer conditions. A local effect
framing out is not itself a proof that a call can be removed. No native tag
may restore that distinction for the optimizer. Validate effect attributes
against opaque ordinary declarations and WF definitions with the same
interfaces, including wrappers and moved owners.

## 7. Opaque layout and range results

**Recommend retaining OP-9's existing `(32,16)` ceiling as the uniform ceiling
for an ordinary opaque nominal.** Concrete layout must fit that ceiling under
the ordinary all-stored-types target-layout obligation
(`spec/kernel-spec.md:1563,1571`). A link-selected native size must not select
`buffer_fits` acceptance. Removing opacity's ceiling instead would make
ordinary allocation-fit queries over these values unavailable; deriving it
from a native catalog would preserve an external exception.

For operation postconditions, recommend existing `Result.Ok` integer routes
and ordinary multi-results (DESIGN.md), replacing `ReadBytes`/`ListBytes` as
compiler-recognized proof sources. Keeping those enums as ordinary data is
also possible, but then callers must establish their bounds with intended
runtime outcomes; an impossible-case branch added solely to satisfy the
checker is not an acceptable migration. Do not generalize FN-9's route
grammar during this removal.

The selected enumeration API uses three ordinary results:
`(Result<unit, ListStop>, next: u64, entries: u64)`. It promises
`start <= next <= end` on every return. Its implementation returns
`next == start` on an error; CALL-4's deferred non-Ok routes stay deferred,
so this convention is tested without adding a selected-Err proof premise.
This matters to the directory walker: it binds the batch, closes the view's
loan, and then matches the outcome. CALL-4 does not carry an enum-bound
endpoint fact through that later owned-local match. The separate numeric
result uses the existing FN-9/CALL-6 multi-result publication path and keeps
the endpoint available without an impossible bounds-failure branch. The
ordinary WF `provide` witness in
[`ordinary_numeric_result_facts_survive_a_later_outcome_match`](../../../compiler/src/semantic/tests/ordinary_effects.rs)
returns an outcome, endpoint and count with an unconditional endpoint
contract; its caller proves the endpoint after matching the saved outcome.
This selects an API shape, not a new fact rule.
