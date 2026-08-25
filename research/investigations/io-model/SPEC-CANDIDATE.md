# World-region I/O specification candidate against v0.36

Status: PHASE-A DRAFT, NON-AUTHORITATIVE. This file is the paper migration
required by `HANDOFF.md`. It changes no active rule and authorizes no compiler
or runtime behavior. If the owner approves the decisions and exact text, the
approved content is transcribed into `spec/kernel-spec.md` and this file is
deleted in the same branch change.

Base: Kernel Specification v0.36, SHA-256
`fd57cfc4bfcf685f14b073c98e149c8a44a201dc79fbd76075ebd49a87995c62`.

## 1. Owner decisions carried by this exact draft

The text below is complete under the five recommended selections. These are
recommendations, not decisions made by the implementation agent.

| ID | Recommended selection used below | Alternative and consequence |
|---|---|---|
| D1 | Conservative-first ordering. Every action replacing an `external` effect writes one command-wide world-order region. | A per-family weakening needs a complete replacement trace law and a new fence operation before migration. It is not drafted here. |
| D2 | Adopt T3's widened erroneous-execution clause. A schedule may select both the false claim named by the one record and the prefix of world effects performed before abort. | A trap-free world-window gate contradicts constitution T3 while W3's claim premise stands. |
| D3 | Reserve `WF_WORKERS=0` for sequential execution with no compute pool; a target may still initialize the I/O substrate its ordinary operations require. `WF_WORKERS=1` selects overlapped execution with one Whitefoot compute lane, no stealing worker, and the target's required completion endpoints; values of two or more select that many compute lanes plus those endpoints. The mapping change exposes all three deltas from DESIGN section 5: the winning false claim may differ, the pre-abort program-output prefix may differ under D2, and an overlapped clone uses the measured 48 B/level stack record rather than the sequential 16 B/level record. | Keeping `0` and `1` both sequential defers single-lane I/O overlap and all three stated deltas. |
| D4 | Keep bare `external` and `blocks` reserved after removing their effect productions. | Releasing either spelling changes the accepted IDENT set and requires positive and negative META-5 cases. |
| D5 | Rename PRV-1's unrelated input-provenance class to `boundary-derived` in this batch. | A separate batch leaves one spelling with two meanings during the migration; verdicts remain unchanged either way. |

Owner approval must name D1 through D5. Approval of D1 and D2 changes
specification bytes and semantics. D4 changes the accepted spelling set if the
alternative is selected. D5 changes specification terminology, compiler
metadata names, diagnostics, and conformance prose but no verdict. D3 is a
runtime-world mapping decision and is not a source-language construct; its
selected mapping is recorded in runtime documentation and tests, not
transcribed into the kernel's worker-independent observable semantics.

### Proposed activation identity and META-5 declaration

If approved, this candidate becomes Kernel Specification v0.37 and archives
the exact outgoing v0.36 bytes before identity generation. Its proposed
META-5 delta is:

- numbered rules +0/-0, with the same 137 rule IDs;
- grammar productions +0/-0, with the same 74 productions, while the `effect`
  production loses the `external` and `blocks` alternatives;
- unique fixed lowercase grammar atoms net -2; D4 adds both spellings to the
  explicit retired/reserved lowercase set, so neither becomes an IDENT;
- writer operation spellings +0/-0, runtime-trap families +0/-0, entry-form
  count +0/-0, contract-block forms +0/-0, and exception-clause count +0/-0;
- system operations +0 and system declaration records +47, from 199 to 246.

The accepted byte set changes deliberately. Former `external`/`blocks` effect
rows no longer parse; the four capability families require exact world
vectors; system operation calls require the expanded kinded region lists; a
command entry may declare world but not memory region parameters; and a
world-bearing capability may occupy a generic or stored position that [STOR-5]
otherwise admits. D4 keeps the two bare lowercase words rejected. No
conformance verdict changes, no required static check is removed, and correct
executions retain v0.36's cross-resource world order. Permission widens only
for windows whose complete memory, loan, exit, and world proofs pass. D2
widens only the specified observables of an erroneous execution.

## 2. Normative model added by the candidate

### 2.1 Two region kinds

A region declaration has exactly one kind after complete signature and body
formation:

- a **memory region** is an [OWN-3] lexical lifetime and may occur in `mode`,
  `slice`, `arena`, memory effects, and borrow expressions;
- a **world region** is an effect and may-alias identity and may occur in a
  system capability's declared world arguments and in world effects.

The same declaration may not occur in both sets. An unresolved or mixed-kind
declaration is rejected. Memory-region equality, liveness, and outlives never
consume a world region. World alias and ordering never consume a memory
region. A `region_stmt` introduces only a memory region. A function's
`region_params` may contain either kind; kind follows the complete declared
occurrences, not the spelling.

`reads(...)` and `writes(...)` retain one source grammar. Each referenced
REGIONID is kind-checked, and a normalized row retains separate memory-read,
memory-write, world-read, and world-write sets. Alpha equality preserves kind.

### 2.2 World identity and disjointness

Different capability values and different world-region spellings never by
themselves prove disjointness. Two world regions are disjoint only when a TCB
minting rule or a checked generativity derivation proves that every state facet
named by their footprints cannot alias. Without that proof they may alias.

Equality implies may-alias. Inequality without a proof also implies may-alias.
Native handle values, descriptor values, target table slots, source spellings,
paths, separate opens, and missing alias records prove nothing. `dup` preserves
every world region of its source. A capability-producing action may establish
a fresh result region only for state its contract proves fresh. Failure
produces no capability and no freshness fact. One static region identity used
by repeated executions denotes a may-alias class across those executions
unless an iteration- or instance-separation proof establishes more.

The first migrated system inventory records origin relations but grants no
cross-action overlap from handle, cursor, object, or sink inequality. Its one
positive ordering fact is that every world capability reachable in one command
carries the same command world-order region.

### 2.3 Capability world vectors

`own` remains payload-free. The four world-bearing opaque system families have
these exact type vectors:

| family | type | world facets |
|---|---|---|
| directory capability | `DirectoryRead<'q, 'h, 'd, 'f>` | command order, handle lifetime, directory/namespace object, conservative filesystem-object class |
| read handle | `ReadFile<'q, 'h, 'c, 'f>` | command order, handle lifetime, cursor sequence, conservative filesystem-object class |
| output handle | `Output<'q, 'o>` | command order, conservative output-sink class |
| enumeration handle | `DirectoryList<'q, 'h, 'c, 'd>` | command order, handle lifetime, enumeration cursor, directory object |

The type identity contains the nominal family and the complete world vector.
Thus `&uniq 'b Output<'q, 'o>` carries one memory-loan region `'b` and two
world regions `'q` and `'o`; none substitutes for another. `Args`,
`HostString`, `RelativePath`, `ExitStatus`, and all outcome-only system types
remain world-free.

World-bearing types are not memory-region-bearing under [STOR-5]. They may
occur inside `Result`, `Option`, user enum payloads, boxes, buffers, and direct
results without carrying a borrow lifetime. Every contained capability keeps
its complete world vector and release action.

### 2.4 Command root identities

A `command` entry may declare world-kind region parameters but no memory-kind
region parameter. Program start supplies their identities; they are not
runtime values and grant no ambient authority. Every declared entry world
region must occur in a selected input capability type, in a system operation's
explicit world argument, or in the exact exhibited world row. EFF-2 still
rejects declared-but-unexhibited effects.

For selected inputs, the closed root table is:

| label | written mode and type | root relation |
|---|---|---|
| `command.args` | `own Args` | no world region |
| `command.cwd` | `own DirectoryRead<'q, 'dh, 'd, 'f>` | carries the command order, one handle class, the conservative directory class, and the conservative file-object class |
| `command.stdout` | `own Output<'q, 'o>` | carries the command order and conservative output class |
| `command.stderr` | `own Output<'q, 'o>` | carries the same command order and output class as stdout when both are selected |

All selected world-bearing inputs use the same actual `'q`. Stdout and stderr
use the same actual `'o`, because redirection may make them one sink. No entry
identity proves the directory, file, and output facets disjoint from another
identity. A capability produced from a root inherits its `'q`; this version
mints no second command-order domain.

[FN-7] admits exactly the [EFF-2]-exact row over the entry's declared world
parameters, together with `allocates(heap)` and `traps` when the body exhibits
them. It admits no memory read/write or arena-allocation entry because main has
no memory region parameter. A world row is not an allow-list: body calls and
compiler-derived releases must exhibit every declared world entry and the
declaration must contain every exhibited entry. `pure` is legal only when the
body and all normal-edge releases exhibit none of those categories.

A canonical complete-input example is:

```wf
command fn main['q, 'dh, 'd, 'f, 'o](command.args as args: own Args, command.cwd as cwd: own DirectoryRead<'q, 'dh, 'd, 'f>, command.stdout as out: own Output<'q, 'o>, command.stderr as err: own Output<'q, 'o>) -> status: own ExitStatus reads('d 'f), writes('q 'dh 'o), allocates(heap), traps {
```

The writer chooses binder names as today. The type vectors and alias equations,
not those spellings, are fixed.

### 2.5 Generated result identities and their binding mechanism

This draft selects the explicit side of the result-identity design: a
capability-producing operation declares result world formals, and the caller
passes writer-declared world parameters for them in the operation's existing
explicit region-argument list. There is no post-result unpack construct and no
compiler-hidden skolem. The written name is a potential identity, not a
capability or freshness assertion: failure associates no value with it, and
success establishes only the origin and freshness facts the operation contract
proves. A wrapper that exposes the result carries those world parameters in
its own signature. This keeps every resulting type finite and writable under
[TYPE-3], and repeated execution with the same written identity remains one
may-alias class.

System operation records distinguish input world formals, result world
formals, and inherited formals. Calls state all region arguments in declared
order under the existing `targs` grammar. A result formal used for a handle or
cursor may receive a freshness fact only when the system contract names that
facet fresh and the caller's selected identity has no incompatible live origin.
Otherwise the result remains in a conservative may-alias class.

The result's complete checked type retains those actual identities even when
the capability is nested in an outcome. A user function may pass, store,
return, or release that value under the ordinary type rules. A user declaration
cannot create a freshness fact by naming a different region. A user wrapper
may only propagate an origin relation supplied by a checked input, a checked
system result, or a verified callable result boundary.

### 2.6 World reads and writes

World read/read overlap is admitted only when each operation contract proves
source-order result attribution under overlap. An action that advances a
cursor, consumes input, samples an ordered sequence, accepts from a dynamic
set, or otherwise changes future observations writes the applicable world
region. Stdin consumption, entropy draws, monotonic-clock samples, `accept`,
file-cursor reads, and directory enumeration are therefore world writes unless
a later family contract proves a stronger attribution law. An idempotent
snapshot may use a world read.

A world write conflicts with a read or write of every may-alias world region.
Two world reads conflict when either contract lacks the source-attribution
proof above. Any unresolved origin, kind, projection, or may-alias relation
conflicts with every world access and denies overlap.

### 2.7 Integration with the existing region rules

The existing grammar continues to use REGIONID and `targs` for both kinds; no
new token or construct is added. The semantic split is exact:

1. An occurrence in `mode`, `borrow_expr`, `slice`, `arena`,
   `allocates(arena ...)`, or a `region_stmt` requires memory kind. An
   occurrence in one of section 2.3's system-capability vector slots or in a
   world formal slot of a user/system call requires world kind. A call actual
   inherits the corresponding formal's kind. A `reads` or `writes` occurrence
   uses the declaration's already established kind and does not by itself
   invent one.
2. Source world declarations are function `region_params`. They have ordinary
   lexical name scope but no OWN-3 liveness or outlives relation. A
   `region_stmt` always declares memory kind. Every unqualified region in
   [OWN-2] through [OWN-14], including loan liveness, holder suspension,
   storage duration, and loop restrictions, means memory region unless a
   sentence explicitly says world.
3. [OWN-12] requires a memory actual to be live and applies outlives and loan
   substitution only to memory formals. A world actual must be in scope, have
   world kind, and match every capability occurrence; it has no storage
   liveness test. [FN-1] retains the ordered kind vector as part of a callable
   signature, and [FN-2] substitutes each actual only for a formal of the same
   kind.
4. [STOR-5] `region-bearing` continues to mean that the complete type contains
   memory-bearing `slice` or `arena`. A capability world vector does not make
   its type region-bearing for storage or generic-argument purposes. It does
   remain part of exact type identity and recursive release.
5. Contract-member alpha equality compares the same ordered region kinds,
   types after kind-preserving substitution, and normalized memory/world rows.
   A kind mismatch is not alpha equality.

Kind constraints are solved over the complete resolved unit, including call
edges, before ordinary body checking. A forwarding cycle with no direct kind
anchor remains unresolved. Mixed kind is a hard [OWN-3] rejection at the first
source occurrence in canonical order that conflicts with the declaration's
earlier constraint; an unanchored parameter is an [OWN-3] rejection at its
declaration. The diagnostic names the region, both required kinds, and the
repair `split the memory lifetime and world identity into two region
parameters`. A wrong-kind user-call actual cites [FN-2], a wrong-kind system
operation or system nominal argument cites [SYS-2], an entry memory parameter
cites [FN-7], and a world region in `allocates(arena ...)` cites [EFF-1], each
at the complete argument or child that its existing rule names.

## 3. Exact effect and ordering replacement

### [EFF-1]

The row grammar becomes:

```wf-ebnf EFF-1
effects := "pure" | effect ("," effect)*
effect := "reads" "(" REGIONID+ ")" | "writes" "(" REGIONID+ ")" | "allocates" "(" ("heap" | "arena" REGIONID)+ ")" | "traps"
```

Categories remain in the canonical order reads, writes, allocates, traps and
appear at most once. A row normalizes to separate memory-read, memory-write,
world-read, world-write, arena-allocation sets, plus the heap-allocation and
`traps` flags. `pure` is the unique row with every set empty and both flags
false. The allocation category contains `heap` and memory-kind arena regions
only.

A world-kind entry states which conservative outside-state facet an action may
observe or change. Resource and family names are not effect spellings. Bare
`external` and `blocks` remain in the explicit retired/reserved lowercase set
but are no longer fixed grammar atoms or effect productions under D4;
REGIONID `'external` and LABEL `@blocks` remain legal.

Completion and blocking are trusted target-action metadata, not source effects
and not part of EFF-1 exactness.

### [EFF-2]

The existing body and release union remains exact in both directions. The
projection is extended as follows.

1. Memory entries retain the v0.36 projection through borrow modes, direct
   slices, allocation regions, and ultimate storage origins.
2. A world entry selects every matching occurrence in a capability type,
   whether the value parameter mode is `own`, shared, or unique. Selection is
   independent of memory-place projection.
3. Formal world regions substitute through user calls and system calls from
   the actual capability vectors and explicit result-region arguments.
4. World identities nested in `Result`, `Option`, or another outcome remain
   attached to the selected payload. A release projects from the complete
   released type, including a nested capability.
5. A compiler-derived release instantiates its world row from the released
   capability vector. Its target-action metadata is accumulated separately.
6. Contract-member equality compares kinded memory/world rows after
   positional alpha-renaming. A region-kind mismatch is never alpha-equal.
7. Missing or ambiguous occurrence selection, substitution, result identity,
   release identity, or alias information rejects EFF-2 or denies overlap; it
   never drops an access.

The body exhibits `traps` exactly as v0.36 does. It exhibits memory/world reads
and writes through the complete projection above. It exhibits allocation as
before. It does not exhibit target-action metadata. A contract block performs
no target action and contributes no memory/world effect, allocation, or trap.

The canonical release example becomes:

```wf
fn release_read_file['q, 'h, 'c, 'f](file: own ReadFile<'q, 'h, 'c, 'f>) -> result: own unit writes('q 'h) {
  return unit;
}
```

Its body has no syntactic effect. The release writes the command-order and
handle-lifetime world regions and separately carries completion/may-block
metadata. `pure` is an undeclared-but-exhibited EFF-2 rejection.

### [EFF-3] and [EFF-4]

`pure` excludes every memory/world read and write, allocation, and trap. The
existing transformation licenses and abort rule are otherwise unchanged.

### [EFF-5]

Sequential world actions preserve the v0.36 source-order promise through one
conservative command-order domain.

Every system operation and compiler-derived release whose v0.36 row contained
`external` writes the `'q` member of its capability vector. A user wrapper
projects that write through its callable boundary. All world capabilities
reachable in one command carry the same `'q`; a produced capability inherits
it. Consequently every pair of former-external actions in one command has
conflicting world footprints regardless of resource, family, owner, native
handle, path, or source spelling.

For two such calls or releases ordered on one normal path, the earlier action
reaches its family completion point before the later action reaches its
submission point. Submission means the request becomes visible to its world
provider. Completion means the result and every borrowed buffer are published
to the caller and the action's world outcome is fixed; it does not imply
durability unless that family says so. A compiler-derived release occupies its
normal-edge position after preceding reverse-declaration-order releases. A
callee's world actions remain inside the boundary position of its call site.

This ordering holds at facts-off and every optimization level. It is not an
optimizer fact. No native or target fact relaxes it. Reordering, eliminating,
deduplicating, coalescing, hoisting, sinking, or speculating an action with a
nonempty world footprint remains unlicensed.

This candidate intentionally does not expose a cross-region fence, because it
does not yet remove any v0.36 cross-resource order. A later family-specific
narrowing must add the exact fence operation and define whether its point is
submission, completion, visibility, or durability in the same specification
amendment.

## 4. Exact overlap and trap replacement

### [PAR-1] world condition

The v0.36 memory footprints, argument-expression reads, loans, interposed-form
checks, and exit checks remain. Each call footprint additionally contains its
projected world reads and writes. A world conflict is judged by section 2.6 and
denies exactly like an unresolved memory footprint. Compiler-derived releases
that may run inside the window contribute their instantiated world accesses.

Each member remains an `ordinary_let_rhs`, but its selected call may resolve
either to a declared user function or directly to one [SYS-2] operation. A
user-call footprint comes from its kinded callable boundary; a system-call
footprint comes from its fixed operation record. A call written between the
members is classified through the same two-target path. Unknown target class,
missing operation data, or an expression shape whose complete footprint is not
computed denies permission.

The old condition that rejects `external`, `blocks`, and every system
operation is deleted. Target-action metadata alone never denies permission.
A call or direct operation requiring completion may be actualized only by an
I/O-frame lowering that preserves the common completion contract. An
implementation that does not select such a lowering executes the legal window
sequentially. A selected target that cannot execute the operation even
sequentially reports an unsupported backend capability, never a source
rejection.

Under D1, two former-external actions conflict through `'q`, so the migration
does not reorder them. A world action may overlap independent compute when all
memory, loan, exit, and world conditions hold.

### [PAR-1] observables and erroneous execution

For an execution in which every executed `claim` is true, every observable is
the observable of source-order execution: every binding and place, normal or
trap outcome, exact [DIAG-3] bytes, and the complete world trace [EFF-5]
requires. This holds in every execution, conditional on TCB contract
compliance.

An execution in which an executed `claim` is false is erroneous. It retains no
undefined behavior and no unproved memory or world overlap. The process writes
exactly one complete [DIAG-3] record naming one false executed claim, then
aborts the whole process without unwinding or language cleanup. No second,
partial, or interleaved record is written.

The schedule may select both which false claim the record names and which
world effects were performed before the abort. The selected world prefix must
obey each submitted action's family semantics and every [EFF-5] order among
actions that reached their ordering points. The trap latch admits no new
submission after it wins. An already-submitted action retains its family
semantics; abort does not wait for it to reach terminal state. The mandatory
record uses one TCB-serialized write whose bytes no in-flight source output can
split or interleave. Failure of that TCB channel may terminate the process
before explicit abort but never permits execution to continue.

No permission, optimization, or fast path is withheld from a correct program
to stabilize an erroneous execution's claim choice or partial world trace.
There is no trap-free closure gate. This sentence carries constitution T3's
yield direction into the permission rule.

When a member does not reach its continuation, what survives is exactly the
one complete record, whole-process abort without cleanup, and the family
semantics of world work submitted before the latch. No source-order value is
promised at an abandoned point. An implementation may expose a source-order
execution control for reproducing a defect, but that control fixes only the
program schedule. A live filesystem, clock, or network remains
nondeterministic without a recorded-world backend. The concrete D3
`WF_WORKERS` mapping is an implementation contract, not a source-language
observable or a sentence in this rule.

### [PAR-2]

The counted-loop memory, loan, accumulator, endpoint, exit, and combination
rules remain. Its body receives the same projected world accesses, conflict
test, target-action metadata rule, and fail-closed unresolved treatment as
[PAR-1]. Under D1, two iterations containing former-external actions conflict
through `'q`. Correct executions preserve the index-order world trace.
Erroneous executions receive exactly [PAR-1]'s widened claim-and-world-prefix
guarantee; the old promise of no observable after the first abandoned
iteration is replaced by that rule.

## 5. Target-action metadata replacing `blocks`

Every target action carries this trusted record outside EFF-1:

| field | values | meaning |
|---|---|---|
| dispatch | `inline`, `completion` | whether the action completes on the executing lane or through a completion source |
| host wait | `never`, `may-block` | whether its target adapter may block a host thread |
| loan end | `call-return`, `terminal` | when transferred buffers and capability state return to the caller |

System operations, compiler-derived releases, close adapters, completion waits,
and backend adapter actions all declare the record. Closed-world analysis
derives the union of reachable target actions for each user wrapper. It is a
lowering summary, not a source effect and not contract equality. A backend must
route a `may-block` action so it cannot stall a Whitefoot compute lane required
for progress. An unavailable route is an unsupported backend capability or
target-qualification failure, not a source rejection.

The union is componentwise and conservative: `completion` dominates `inline`,
`may-block` dominates `never`, and `terminal` dominates `call-return`; the
empty summary is `(inline, never, call-return)`. It includes every system call
and compiler-derived release reachable on the conservative structural graph.
Recursive call components take the least fixed point of that finite join.
Missing callable or release metadata yields the dominating conservative
record rather than an empty one.

For the first migrated inventory, the seven host-facing operation rows and
three native-close releases use `(completion, may-block, terminal)`. All other
system operations and logical releases use `(inline, never, call-return)`.

## 6. Exact [SYS-2] migration

### 6.1 System nominal records

The opaque inventory remains eight names, but four names now require the world
vectors in section 2.3. A bare `DirectoryRead`, `ReadFile`, `Output`, or
`DirectoryList`, a wrong argument count, a non-world argument, or a different
argument order is a [SYS-2] rejection. The other twelve system nominal types
remain bare and reject `targs`.

The inventory count becomes sixteen nominal types, fourteen owner-local system
nominal world parameters, forty-two constructors, sixty-seven fields, fifteen
operations, fifty-four operation region parameters, and thirty-eight operation
value parameters: 246 declaration records in [SYS-2] preorder. System nominal
world parameters and operation parameters are owner-local and never enter
source lookup.

The preorder is exact. Visit the sixteen nominal types in table order; emit
each nominal record and then that nominal's world-parameter records in vector
order. Next visit the forty-two constructors in table order and emit each
constructor followed by its fields. Finally visit the fifteen operations in
table order and emit each operation followed by its region parameters and
then its value parameters. Only nominal, constructor, and operation records
enter source lookup. The four nominal vectors spell their owner-local formals
exactly as section 2.3 does.

### 6.2 Complete operation signatures

The complete fifteen-operation inventory is:

```wf
fn args_count['a](args: &'a Args) -> result: own u64 reads('a);
fn arg_get['a](args: &'a Args, position: own u64) -> result: own Result<HostString, ArgError> reads('a);
fn host_bytes_len['v](value: &'v HostString) -> result: own u64 reads('v);
fn host_copy_bytes['v, 'm](value: &'v HostString, destination: &uniq 'm buffer<u8>, start: own u64, end: own u64) -> result: own Result<u64, CopyError> reads('v 'm), writes('m);
fn host_utf8_len['v](value: &'v HostString) -> result: own Result<u64, Utf8Error> reads('v);
fn host_copy_utf8['v, 'm](value: &'v HostString, destination: &uniq 'm buffer<u8>, start: own u64, end: own u64) -> result: own Result<u64, Utf8CopyError> reads('v 'm), writes('m);
fn relative_path(value: own HostString) -> result: own Result<RelativePath, PathError> pure;
fn open_read['b, 'p, 'q, 'dh, 'd, 'f, 'rh, 'rc](root: &'b DirectoryRead<'q, 'dh, 'd, 'f>, path: &'p RelativePath) -> result: own Result<ReadFile<'q, 'rh, 'rc, 'f>, IoError> reads('b 'p 'dh 'd 'f), writes('q 'rh 'rc);
fn read_once['b, 'm, 'q, 'h, 'c, 'f](file: &uniq 'b ReadFile<'q, 'h, 'c, 'f>, destination: &uniq 'm buffer<u8>, start: own u64, end: own u64) -> result: own ReadOutcome reads('b 'm 'h 'f), writes('b 'm 'q 'c);
fn write_once['b, 's, 'q, 'o](output: &uniq 'b Output<'q, 'o>, source: &'s buffer<u8>, start: own u64, end: own u64) -> result: own Result<u64, IoError> reads('b 's), writes('b 'q 'o);
fn exit_status(code: own u8) -> result: own ExitStatus pure;
fn open_directory['b, 'n, 'q, 'dh, 'd, 'f, 'rh](root: &'b DirectoryRead<'q, 'dh, 'd, 'f>, name: &'n buffer<u8>, start: own u64, end: own u64) -> result: own Result<DirectoryRead<'q, 'rh, 'd, 'f>, IoError> reads('b 'n 'dh 'd), writes('q 'rh);
fn open_list['b, 'q, 'dh, 'd, 'f, 'lh, 'lc](directory: &'b DirectoryRead<'q, 'dh, 'd, 'f>) -> result: own Result<DirectoryList<'q, 'lh, 'lc, 'd>, IoError> reads('b 'dh 'd), writes('q 'lh 'lc);
fn list_once['b, 'm, 'q, 'h, 'c, 'd](list: &uniq 'b DirectoryList<'q, 'h, 'c, 'd>, destination: &uniq 'm buffer<u8>, start: own u64, end: own u64) -> result: own ListOutcome reads('b 'm 'h 'd), writes('b 'm 'q 'c);
fn open_file['b, 'n, 'q, 'dh, 'd, 'f, 'rh, 'rc](root: &'b DirectoryRead<'q, 'dh, 'd, 'f>, name: &'n buffer<u8>, start: own u64, end: own u64) -> result: own Result<ReadFile<'q, 'rh, 'rc, 'f>, IoError> reads('b 'n 'dh 'd 'f), writes('q 'rh 'rc);
```

The first seven rows and `exit_status` carry `(inline, never,
call-return)`. `open_read`, `read_once`, `write_once`, `open_directory`,
`open_list`, `list_once`, and `open_file` carry `(completion, may-block,
terminal)`. No operation carries `traps` or allocates.

For calls, all region arguments remain explicit and follow declaration order.
Memory formals must receive live memory regions. World formals must receive
world regions. The checker also verifies that every actual capability vector
matches the supplied world actuals exactly; a call cannot re-label a
capability by supplying a different identity.

### 6.3 Origin and alias records

The seven world-facing rows carry these compiler-owned origin relations:

| operation | authority and inherited origins | result origins |
|---|---|---|
| `open_read` | borrows one `DirectoryRead`; inherits `'q` and `'f` | result handle `'rh` and cursor `'rc` are fresh state on success when the caller identity is eligible for minting; failure mints nothing |
| `read_once` | borrows one `ReadFile`; no capability result | preserves every input origin; writes cursor `'c` |
| `write_once` | borrows one `Output`; no capability result | preserves `'q` and `'o` |
| `open_directory` | borrows one `DirectoryRead`; inherits `'q`, `'d`, and `'f` | result handle `'rh` is fresh on success; the directory object may alias the input or another open |
| `open_list` | borrows one `DirectoryRead`; inherits `'q` and `'d` | result handle `'lh` and enumeration cursor `'lc` are fresh state on success |
| `list_once` | borrows one `DirectoryList`; no capability result | preserves every input origin; writes cursor `'c` |
| `open_file` | borrows one `DirectoryRead`; inherits `'q` and `'f` | same handle/cursor rule as `open_read` |

Separate opens, same or different paths, `.` or `..`, symbolic or hard links,
and distinct target handles never prove distinct directory or file objects.
All `Output` values may alias in this target policy. A duplicate operation, if
later added, preserves every source origin. A network family, if later added,
may mint a fresh connection origin only when its TCB contract proves a new
connection; duplicate handles retain the same connection origin.

### 6.4 Operation points, outcomes, and progress

Each world-facing semantic ID fixes all seven closure fields required by the
I/O design:

| semantic ID | submission and linearization | completion and loan release | outcome and progress |
|---|---|---|---|
| `sys.open_read` | submit after both authority borrows are established; linearize when the target fixes success with one provisional handle or one failure | terminal after result publication and cleanup of any unpublished handle | existing `Result<ReadFile, IoError>`; may wait without a language deadline |
| `sys.read_once` | submit after both exclusive loans and range proof; linearize when the target fixes the transferred prefix and cursor advance | terminal after buffer bytes, result, and cursor state are release-published; both loans end then | existing `ReadOutcome`; peer/device removal completes with a typed outcome when the target can report one; a lost completion is a target defect |
| `sys.write_once` | submit after source/output loans and range proof; linearize when the host accepts the returned prefix | terminal after result publication and loan return; not visibility to another machine and not durability | existing `Result<u64, IoError>`; may wait without a language deadline |
| `sys.open_directory` | submit after component validation; linearize when target resolution fixes one directory handle or failure | terminal after result publication and cleanup of any unpublished handle | existing `Result<DirectoryRead, IoError>`; may wait |
| `sys.open_list` | submit after directory authority is borrowed; linearize when target fixes enumeration handle or failure | terminal after result publication and cleanup of any unpublished handle | existing `Result<DirectoryList, IoError>`; may wait |
| `sys.list_once` | submit after both exclusive loans and range proof; linearize when target fixes the complete portable record prefix and cursor advance | terminal after in-place record conversion, result/cursor publication, and both loan returns | existing `ListOutcome`; may wait; lost completion is a target defect |
| `sys.open_file` | submit after component validation; linearize after open and required descriptor-status classification fix success or one failure | terminal after publication and cleanup of any unpublished handle | existing `Result<ReadFile, IoError>`; may wait |

No cancellation is exposed in this version. Every normal or recoverable window
exit observes the terminal state of every submitted action in that window.
A cancel request is not terminal. From submission through terminal state, a
borrowed buffer does not move, free, or become reusable; an exclusive input
loan also excludes the submitting lane. The frame and its generation-tagged
completion node remain live and cannot return to a free list.

Typed world failures remain values. A failed proof remains a static rejection.
A false claim remains the sole language trap. TCB allocation or thread
exhaustion remains resource death. A lost, duplicate, or malformed completion,
an impossible target count, and a violated release/acquire publication contract
are target defects under [SCOPE-3], never `IoError` and never a source
rejection.

### 6.5 Release table

The consuming release table becomes:

| type | release action | instantiated world row | target-action metadata |
|---|---|---|---|
| `Args` | logical consume | `pure` | inline/never/call-return |
| `HostString` | logical consume | `pure` | inline/never/call-return |
| `RelativePath` | logical consume | `pure` | inline/never/call-return |
| `DirectoryRead<'q, 'h, 'd, 'f>` | at most one native close attempt | `writes('q 'h)` | completion/may-block/terminal |
| `ReadFile<'q, 'h, 'c, 'f>` | at most one native close attempt | `writes('q 'h)` | completion/may-block/terminal |
| `Output<'q, 'o>` | logical source detach | `pure` | inline/never/call-return |
| `ExitStatus` | logical consume | `pure` | inline/never/call-return |
| `DirectoryList<'q, 'h, 'c, 'd>` | at most one native close attempt | `writes('q 'h)` | completion/may-block/terminal |

The native close attempt remains one attempt with discarded diagnostics and no
retry. It changes the handle-lifetime facet, not the persistent object facet.
A logical release performs no world access. Whole-process abort runs no
release.

## 7. [TRAP-1] and the diagnostic channel

After a false claim wins the process-wide trap latch, no backend accepts a new
world submission. Work already submitted retains its family semantics, may
finish before or after the diagnostic write, and is not rolled back. The abort
path does not wait for terminal states. Hosted teardown holds or transfers all
kernel references until the host can no longer touch submitted buffers. A
bare-metal target must quiesce, quarantine, or reset DMA before memory reuse;
this is target qualification, not language cleanup.

The diagnostic channel is TCB-owned and distinct from every source
capability. One globally serialized host write emits the complete [DIAG-3]
record. A source output action that may reach the same host sink linearizes
wholly before or wholly after that write and cannot split or interleave the
record bytes. D2 permits such a whole source write when it belongs to the
selected submitted-work prefix; it never becomes part of the record. This
serialization cost exists only on the trap path. A channel failure may
terminate the process before explicit abort and never resumes execution.

## 8. [QUAL-*] and gated calls

Each semantic ID's qualification record additionally binds its world vector,
origin/mint relation, footprint, submission/linearization/terminal points,
target-action metadata, publication memory order, frame lifetime, progress
contract, and abort teardown. Backend selection remains a build and TCB matter
and never appears in source or the checked-program semantic ID.

A gated foreign signature must name a complete typed capability/world
footprint. When its reach cannot be classified, qualification charges the call
to one compiler-owned top-world domain that may alias every world region and
writes the command-order domain. Missing footprint data never implies purity
or disjointness. The current kernel exposes no writer-callable gated foreign
signature.

The compiler's `--io-ledger` output is a deterministic audit surface beside
`--par-ledger`. For every world-bearing source call, compiler-derived release,
and permission site it reports the stable semantic ID, instantiated world
vector, inherited/generated origin relations, normalized world footprint,
target-action summary, permission result, and whether the selected lowering
actualized or left the site sequential. It is diagnostic output only, reads no
optimizer fact, and changes neither source acceptance nor checked-program
identity. Unresolved data is printed as an explicit conservative denial, never
as an omitted field.

## 9. PRV-1 terminology under D5

The input provenance class formerly called `external` becomes
`boundary-derived`; `unconditional-external` becomes
`unconditional-boundary`, and the implementation field becomes
`unconditionally_boundary_derived`. This class continues to seed only the
PRV-1 data-dependency judgment. It does not inspect an effect row, world
region, capability origin, or target-action record. All PRV-2 and PRV-3
verdicts remain byte-for-byte the same apart from terminology in diagnostics
or explanatory prose.

Conformance case IDs and writer-chosen source identifiers remain stable: they
are historical test identity and ordinary source spelling, not names of the
metadata class. Every normative sentence, diagnostic phrase, manifest or case
`doc`, compiler type/variant/field, and internal explanatory comment that names
the class uses `boundary-derived` after D5.

If D5 is deferred, every clause in this section instead retains its v0.36
spelling and the effect migration must explicitly state that the provenance
homonym is unrelated. No world rule consumes it in either choice.

## 10. Per-row disposition of `reviews/spec-sweep.md`

Every table row in the sweep is dispositioned below. `Replaced` names the
candidate section containing the complete replacement rule. `Preserved` means
the v0.36 sentence remains semantically unchanged. `D5 rename` means only the
independent PRV terminology changes, with no effect or verdict change.

### 10.1 Effect, entry, ordering, and trap rows

| sweep row | disposition |
|---|---|
| K30 | Replaced by sections 4 and 7: distinct TCB diagnostic channel, one serialized write, no resume. |
| K672 | Replaced by sections 5 and 6.5: instantiated world row plus target-action metadata for every release. |
| K1089 | Replaced by sections 2.1 and 3: normalized memory/world read/write sets, allocations, and traps. |
| K1092 | Deleted; section 3 defines kinded contract equality and excludes action metadata. |
| K1184 | Replaced by section 2.4: main admits world-kind parameters and exact world rows, but no memory-kind parameter. |
| K1208 | Replaced by section 2.4's complete-input header and root alias equations. |
| K1343 | Replaced by section 3's four-category order. |
| K1347 | Replaced byte-for-byte by section 3's EFF-1 fence. |
| K1351 | Replaced by section 3: `pure` empties every memory/world set, allocation, and trap. |
| K1356 | Replaced by sections 2.3, 2.6, and 3: capability facets and world does-ness. |
| K1357 | Replaced by section 5's trusted metadata. |
| K1358 | Replaced by sections 2.1 and 3: existing `reads`/`writes` carry kinded REGIONIDs. |
| K1361 | Resolved by D4: bare spellings remain reserved without effect productions. |
| K1362 | Preserved explicitly in section 3. |
| K1365 | Replaced by section 3's complete memory/world projection and separate action summary. |
| K1368 | Replaced by section 3: no memory/world effect and no target action in erased contracts. |
| K1372 | Replaced by section 3: hidden counted-loop mechanics remain effect- and target-action-free. |
| K1408 | Replaced by section 3's own/shared/unique capability occurrence projection. |
| K1419 | Replaced by section 3's exact `ReadFile` release example. |
| K1426 | Replaced by section 3's [EFF-3] clause. |
| K1427 | Replaced by sections 3 and 6: every world action has a nonempty world row. |
| K1432 | Replaced by section 3's conservative command-order rule. |
| K1433 | Replaced by the common `'q` relation in sections 2.4 and 3. |
| K1434 | Replaced by section 3's submission-before-completion ordering points. |
| K1435 | Preserved in behavior by the common `'q` write across every resource and family. |
| K1436 | Replaced by sections 3 and 6.5: instantiated release write at the normal-edge position. |
| K1437 | Replaced by section 3's nested-call boundary rule. |
| K1438 | Preserved explicitly in section 3 for facts-off and every optimization level. |
| K1440 | Replaced by section 3's world-action trace. |
| K1441 | Replaced by section 3: command-wide order is not a global runtime lock; no narrowing occurs yet. |
| K1442 | Replaced by section 2.2: ownership alone never proves world disjointness. |
| K1444 | Replaced by section 2.2's closed proof sources. |
| K1445 | Preserved explicitly in section 2.2. |
| K1446 | Replaced by section 3's nonempty-world-footprint transformation prohibition. |
| K1447 | Preserved; any later optional fact family remains separately approved and fail-closed. |
| K2012 | Replaced by section 4: projected world conflicts plus an I/O-frame actualization requirement; metadata alone does not deny. |
| K2016 | Replaced by sections 2.6, 3, and 4: correct runs preserve the exact world trace and read attribution. |
| K2017 | Preserved in section 4's TCB-conditional statement. |
| K2018 | Preserved in section 4 for every correct execution. |
| K2019 | Preserved with submitted-work semantics fixed by sections 4 and 7. |
| K2020 | Preserved with one serialized record and whole-process abort in sections 4 and 7. |
| K2021 | Preserved and strengthened operationally by section 7's single-write channel. |
| K2022 | Replaced under D2: schedule selects claim identity and the pre-abort world prefix. |
| K2023 | Replaced under T3: no trap-free gate; memory/world conflict safety remains and the erroneous promise widens. |
| K2024 | Preserved for correct executions; erroneous world-prefix selection is the explicit exception fixed beside it. |
| K2025 | Preserved: permission remains optional. |
| K2026, first sentence | Replaced by section 4's exact surviving record, abort, and submitted-family semantics. |
| K2039 | Replaced by section 4's loop world-footprint and action-lowering condition. |
| K2042 | Replaced by section 4's index-order world trace and read-attribution requirement. |
| K348 | Replaced by sections 2.1, 2.5, and 6.2: call arguments remain explicit, but each written region actual is checked against the formal memory/world kind. |
| K684 | Amended by section 2.3: STOR-5's `region-bearing` predicate remains the memory-lifetime predicate over `slice` and `arena`; a world-bearing capability may occur in stored or generic payloads and retains its vector and release. |
| K1182 | Replaced by section 2.4: main remains nongeneric and contract-free, admits world-kind region parameters, and still rejects memory-kind region parameters. |
| K1211 | Preserved by section 2.4: every world authority still originates in an explicitly selected entry capability and reaches helpers only through written parameters; no ambient route is added. |

### 10.2 Gated, system, release, and contract rows

| sweep row | disposition |
|---|---|
| K2080–2084 | Replaced by section 8: complete typed footprint or conservative top-world charge. |
| K2250 | Replaced by section 6.2 `open_read`, including inherited and result world identities. |
| K2251 | Replaced by section 6.2 `read_once`, with handle/file reads and cursor/order writes. |
| K2252 | Replaced by section 6.2 `write_once`, separating borrow lifetime from order/sink identities. |
| K2254 | Replaced by section 6.2 `open_directory`, with inherited object classes and a result handle. |
| K2255 | Replaced by section 6.2 `open_list`, with handle/cursor result identities. |
| K2256 | Replaced by section 6.2 `list_once`, with directory read and cursor/order writes. |
| K2257 | Replaced by section 6.2 `open_file`, matching `open_read` origin rules. |
| K2263 | Replaced by sections 5 and 6.2: fixed memory/world row plus separate target-action record. |
| K2265 | Replaced by sections 2.6 and 5. |
| K2340 | Replaced: a logical lease consume has no world-region access. |
| K2366 | Replaced by section 8's expanded semantic-ID qualification record. |
| K2398 | Replaced by sections 4 and 7: completed writes remain and submitted work retains family semantics. |
| K2440 | Replaced terminologically with unfinished world-submitted work; no policy class changes. |
| K2452 | Replaced by section 6.5 `DirectoryRead` row and metadata. |
| K2453 | Replaced by section 6.5 `ReadFile` row and metadata. |
| K2456 | Replaced by section 6.5 `DirectoryList` row and metadata. |
| K2459 | Replaced: logical consume performs no world-region access. |
| K2466 | Replaced by sections 6.5 and 7: no release on abort; submitted work is not rolled back. |
| K2616 | Preserved in behavior by stdout/stderr sharing `'q` and `'o`; same-sink source order remains mandatory. |
| K2634 | Replaced: `exit_status` has no world-region access. |
| K2685 | Replaced: a claim predicate admits no operation with a world access or target action. |
| K2163 | Replaced by sections 2.3 and 6.1: the eight opaque names remain, with exact world vectors on four capability families. |
| K2166 | Replaced by sections 2.3 and 6.1: four opaque families require world `targs`; the remaining twelve system nominal types remain bare. |
| K2240 | Preserved in count and replaced in content by section 6.2's complete fifteen-operation table. |
| K2260 | Replaced by section 6.1's exact inventory counts and 246-record preorder. |
| K2295 | Preserved and kinded by section 6.2: system operations remain nongeneric, and every `targ` remains a region actual. |
| K2306 | Replaced by sections 5, 6.1, 6.2, and 6.3: each operation record additionally fixes region kinds, world footprints, origin relations, and target-action metadata. |
| K2405 | Preserved and applied by section 2.6: cursor or sequence consumption is a world write even when payload bytes flow toward the caller. |
| K2432 | Preserved by sections 2.2 and 6.3: no duplication operation is added, and any future duplicate preserves every source world identity. |
| K2444 | Replaced by section 6.5's complete instantiated release table. |
| K2545 | Preserved by section 6.4: endpoint, buffer, and cursor disposition remains exact through terminal completion. |
| K2588 | Replaced by section 6.3's separate handle/cursor result origins and conservative persistent-object origin. |
| K2590 | Preserved by sections 2.2 and 6.3: separate directory capabilities may denote one object. |
| K2600 | Preserved by sections 2.2 and 6.3: descent and separate opens never prove a distinct directory object. |
| K2603 | Replaced by sections 2.3 and 6.3: `ReadFile` carries distinct handle, cursor, and conservative filesystem-object facets. |
| K2613 | Preserved by sections 2.3 and 6.3: `Output` remains one stateful family, and all values conservatively may alias one sink domain. |
| K2614 | Preserved: stdout and stderr remain separate affine owners while section 2.4 gives them the same order and sink identities. |
| K2617 | Replaced by section 2.4's mandatory common `'o` identity; the same-sink relation now participates directly in world conflicts rather than remaining an unused fact. |
| K2644 | Replaced by sections 2.3 and 6.3: `DirectoryList` carries a fresh handle/cursor result pair and the inherited conservative directory-object identity. |

### 10.3 Provenance rows

| sweep row | disposition |
|---|---|
| K943 | D5 rename only: `unconditionally boundary-derived`. |
| K2271 | D5 rename only for the `args_count` result class. |
| K2272 | D5 rename only for `arg_get` payload classes. |
| K2273 | D5 rename only for `host_bytes_len`. |
| K2274 | D5 rename only for `host_copy_bytes` result and destination classes. |
| K2275 | D5 rename only for `host_utf8_len`. |
| K2276 | D5 rename only for `host_copy_utf8` result and destination classes. |
| K2277 | D5 rename only for `relative_path`. |
| K2278 | D5 rename only for `open_read`. |
| K2279 | D5 rename only for `read_once`; dependent/internal distinctions stay fixed. |
| K2280 | D5 rename only for `write_once`. |
| K2282 | D5 rename only for `open_directory`. |
| K2283 | D5 rename only for `open_list`. |
| K2284 | D5 rename only for `list_once`; dependent/internal distinctions stay fixed. |
| K2285 | D5 rename only for `open_file`. |
| K2290 | D5 rename of class and unconditional bit. |
| K2292 | D5 rename only; no unlisted component inherits the class. |
| K2738 | D5 rename only; claim authority remains unchanged. |
| K2837 | D5 rename only in the amendment-level component-class clause. |
| K2969 | D5 rename only for endpoint dependency prose. |
| K3151 | D5 rename only in the protected-family repair. |
| K3188 | D5 rename only for the system-write seed. |
| K3190 | D5 rename only for system result seeding. |
| K3203 | D5 rename only; local initialization remains separate. |
| K3205 | D5 rename only and preserved as the explicit proof that provenance ignores effect rows. |
| K3242 | D5 rename only for the local PRV-3 candidate bit. |
| K3251 | D5 rename only for the list of metadata not added by facts. |
| K3258 | D5 rename only for direct-demand composition. |
| K3264 | D5 rename only: no synthetic boundary parameter datum. |
| K3272 | D5 rename only for terminal treatment of the bit. |
| K3273 | D5 rename only for command-entry inputs. |
| K3285 | D5 rename only for the component pair. |
| K3287 | D5 rename only for the concrete class evaluation. |
| K3289 | D5 rename only for labelled command input initialization. |
| K3291 | D5 rename only for the closed origin set. |
| K3307 | D5 rename only for non-subject uses. |
| K3318 | D5 rename only for origin carriers. |
| K3325 | D5 rename only for the no-synthetic-datum invariant. |
| K3329 | D5 rename only for `Targets(c, q)`. |
| K3362 | D5 rename only for the restructuring repair. |
| K3363 | D5 rename only; `claim` remains no repair. |
| K3370 | D5 rename only for the PRV-3 rejection premise. |
| K3371 | D5 rename only for ordered diagnostic explanations. |
| K3377 | D5 rename only for entry-local leaves. |
| K3383 | D5 rename only for claim authorization. |
| K3386 | D5 rename only for the diagnostic payload and repair text. |

### 10.4 Homonyms and `pure`-dependent rows

| sweep row | disposition |
|---|---|
| K7 | Preserved: ordinary-English external review. |
| K19 | Preserved: ordinary-English external approval judgment. |
| K66 | Preserved: brace blocks. |
| K140 | Preserved: parser external terminal. |
| K145 | Preserved: parser external-terminal predicate. |
| K1029 | Preserved: branch blocks. |
| K1479 | Preserved: foreign-code boundary; section 8 separately fixes world authority. |
| K1492 | Preserved: externally invoked entry. |
| K1571 | Preserved: parser external terminal. |
| K1602 | Preserved: parser external predicate. |
| K1615 | Preserved: parser external predicates. |
| K1643 | Preserved: external tool failure. |
| K1654 | Preserved: parser external terminal. |
| K1802 | Preserved: external tool failure. |
| K6 | Replaced by the eventual META-5 delta; this migration changes source bytes and permitted overlap. |
| K1094 | Replaced by section 3's kinded row equality and nonempty world footprint. |
| K1346 | Preserved byte-for-byte. |
| K1353 | Deleted because the two effect categories are removed. |
| K1355 | Replaced: a world payload names a conservative alias facet. |
| K1359 | Replaced: world rows feed only the enumerated alias, order, and overlap judgments. |
| K1416 | Preserved. |
| K1421 | Preserved against the new `writes('q 'h)` release example. |
| K1424 | Preserved because every world-touching action has a nonempty world row. |
| K1425 | Preserved. |

All sixteen amendments at the end of the sweep are therefore represented:
sections 2.2, 2.1, 2.3, 2.4, 2.5, 6.3, 6.3, 2.6, 3, 4, 3, 5, 6, 3/8,
9, and 11 respectively.

## 11. Conformance migration ledger

The parent search names exactly 42 case files. Every expected verdict below is
retained. `syntax` means capability types, explicit world arguments, and effect
rows are migrated together. `PRV only` means source behavior is unchanged and
only D5 terminology in a `doc` string moves. `boundary` means the case pins one
of the seven EFF-2/release judgments and is rewritten to test the same side of
the new rule.

| case | parent and candidate verdict | migration |
|---|---|---|
| `accept-syseff-conditional-release-union` | accept | boundary: outcome becomes `Result<ReadFile<'q, 'h, 'c, 'f>, IoError>` and the declared release union is `writes('q 'h)` |
| `accept-syseff-pure-immutable-only` | accept | boundary: remains `pure`; prose says no world access rather than no external effect |
| `accept-sysentry-command-all-inputs` | accept | syntax: add entry world binders/vectors; exact release row is `writes('q 'dh)` |
| `accept-sysrelease-return-unit-declared` | accept | boundary: exact section 3 release example |
| `prv1-pos-control-write-address-nontaint` | accept | PRV only |
| `prv1-pos-payload-sibling-isolation` | accept | PRV only |
| `prv2-neg-direct-system-result` | reject PRV-2 | PRV only |
| `prv2-neg-entry-system-result-bridge` | reject PRV-2 | PRV only |
| `prv2-neg-mutual-demand` | reject PRV-2 | PRV only |
| `prv2-neg-recursive-demand` | reject PRV-2 | PRV only |
| `prv2-neg-two-hop-bridge` | reject PRV-2 | PRV only |
| `prv2-pos-seedless-mutual` | accept | PRV only |
| `prv3-neg-read-offset-taint` | reject PRV-3 | PRV only |
| `prv3-pos-external-bound-only` | accept | PRV only; the stable case ID remains, while its `doc` terminology moves under D5 |
| `prv3-pos-external-branch` | accept | PRV only; the stable case ID remains, while its `doc` terminology moves under D5 |
| `prv3-pos-internal-claim` | accept | PRV only |
| `reject-sys14-list-end-beyond-buffer` | reject SYS-8 | syntax: entry and list operation world arguments/row migrate; failing range judgment stays SYS-8 |
| `reject-syseff-conditional-release-narrow` | reject EFF-2 | boundary: still omits `writes('q 'h)` from a one-arm release |
| `reject-syseff-declared-unexhibited` | reject EFF-2 | boundary: use a well-bound `Output<'q, 'o>` parameter and declare an unexhibited `writes('o)` |
| `reject-syseff-pure-member-binds-release` | reject FN-3 | boundary: pure member still cannot bind a function whose `ReadFile` release exhibits `writes('q 'h)` |
| `reject-syseff-return-unit-pure` | reject EFF-2 | boundary: owned `ReadFile` release remains the omitted effect |
| `reject-sysentry-input-type-mismatch` | reject FN-7 | syntax: the selected Args row is still written as the wrong capability type, now with a complete world vector |
| `reject-sysentry-label-out-of-order` | reject FN-7 | syntax: add valid world vectors without repairing label order |
| `run-sysdir-open-notfound` | run exit 0 | syntax: migrate entry, operation result identities, and row; runtime outcome unchanged |
| `run-sysfile-empty` | run exit 0 | syntax: migrate file identities and `read_once`; runtime outcome unchanged |
| `run-sysfile-exact` | run exit 0 | syntax; runtime endpoint unchanged |
| `run-sysfile-multichunk` | run exit 0 | syntax; repeated calls reuse the static may-alias class and runtime bytes stay unchanged |
| `run-sysfile-short` | run exit 0 | syntax; short-read outcome unchanged |
| `run-sysout-basic-write` | run exit 0 | syntax: `Output<'q, 'o>` and `writes('q 'o)`; bytes unchanged |
| `run-sysout-redirect-same-sink-order` | run exit 0 | syntax: stdout/stderr share `'q` and `'o`; combined bytes remain exactly `AABB` |
| `sys14-directory-release` | run exit 0 | syntax: release becomes `writes('q 'h)`; close behavior unchanged |
| `sys14-entry-kind-closed` | run exit 0 | syntax: valid command root vector; program-kind closure unchanged |
| `sys14-list-handle-affine` | reject OWN-1 | syntax: world-bearing handle remains affine; duplicate use still rejects |
| `sys14-list-handle-unique` | reject OWN-5 | syntax: world vector does not weaken exclusive memory loans |
| `sys14-list-outcome-exhaustive` | run exit 0 | syntax; outcome constructors unchanged |
| `sys14-list-zero-range` | run exit 0 | syntax; zero-range outcome unchanged |
| `sys14-no-path-from-bytes` | reject TYPE-5 | syntax: capability vectors migrate; forbidden conversion remains TYPE-5 |
| `sys14-open-directory-component` | run exit 0 | syntax; component resolution unchanged |
| `sys14-open-directory-empty-name` | run exit 0 | syntax; `InvalidPath` outcome unchanged |
| `sys14-open-directory-success` | run exit 0 | syntax: result handle identity supplied explicitly; runtime directory unchanged |
| `v033-run-open-file-directory` | run exit 0 | syntax; `IsDirectory` outcome unchanged |
| `v033-run-open-file-regular` | run exit 0 | syntax; successful file/read behavior unchanged |

The seven verdict-sensitive manifest records are exactly
`accept-sysrelease-return-unit-declared`,
`reject-syseff-return-unit-pure`,
`reject-syseff-declared-unexhibited`,
`accept-syseff-conditional-release-union`,
`reject-syseff-conditional-release-narrow`,
`accept-syseff-pure-immutable-only`, and
`reject-syseff-pure-member-binds-release`. Their expected kinds and cited
rules do not move. In particular, `reject-syseff-declared-unexhibited` does
not delete the old atoms and become vacuous; it declares one bound but
unexhibited world write.

The 27 files whose source, after removing string literals, actually writes an
`external` effect row are exactly:

| | | |
|---|---|---|
| `accept-syseff-conditional-release-union` | `accept-sysentry-command-all-inputs` | `accept-sysrelease-return-unit-declared` |
| `reject-sys14-list-end-beyond-buffer` | `reject-syseff-declared-unexhibited` | `reject-syseff-pure-member-binds-release` |
| `reject-sysentry-input-type-mismatch` | `reject-sysentry-label-out-of-order` | `run-sysdir-open-notfound` |
| `run-sysfile-empty` | `run-sysfile-exact` | `run-sysfile-multichunk` |
| `run-sysfile-short` | `run-sysout-basic-write` | `run-sysout-redirect-same-sink-order` |
| `sys14-directory-release` | `sys14-entry-kind-closed` | `sys14-list-handle-affine` |
| `sys14-list-handle-unique` | `sys14-list-outcome-exhaustive` | `sys14-list-zero-range` |
| `sys14-no-path-from-bytes` | `sys14-open-directory-component` | `sys14-open-directory-empty-name` |
| `sys14-open-directory-success` | `v033-run-open-file-directory` | `v033-run-open-file-regular` |

The 27 files with an actual source row containing `external` receive the full
syntax migration. The remaining files contain only release-boundary prose or
PRV terminology. No manifest expectation, rule citation, case status, runtime
byte expectation, or fixture arrangement changes. The same-sink witness stays
mandatory and is the conservative-alias positive control.

The 42-case review count is not the complete repository syntax surface.
Nineteen further tracked `.wf` programs use a world-bearing family or operation
and migrate in place so their preserved workload remains compilable:

| repository role | exact files |
|---|---|
| canonical test programs | `tests/programs/byte_string.wf`, `tests/programs/dir_walk.wf`, `tests/programs/par_layout.wf`, `tests/programs/raw_deflate_boundary.wf`, `tests/programs/wfgrep.wf` |
| current experiments | `research/experiments/buffer-initialization-cost/drain.wf`, `research/experiments/wfgrep-double-walk/shapes/s1-hoisted-first-byte.wf`, `research/experiments/wfgrep-double-walk/shapes/s2-fused-scan-match.wf`, `research/experiments/wfgrep-double-walk/shapes/s3-swar-newline-scan.wf` |
| durable parallelism loop probes | `research/investigations/proof-derived-parallelism/loop/probes/p4_split_equiv.wf`, `research/investigations/proof-derived-parallelism/loop/probes/p5_float_split.wf`, `research/investigations/proof-derived-parallelism/loop/probes/r2_grid_loop_d21_w256.wf` |
| durable parallelism pair probes | `research/investigations/proof-derived-parallelism/probes/bt.wf`, `research/investigations/proof-derived-parallelism/probes/min_stack.wf`, `research/investigations/proof-derived-parallelism/probes/p1a.wf`, `research/investigations/proof-derived-parallelism/probes/p1b.wf`, `research/investigations/proof-derived-parallelism/probes/p6.wf`, `research/investigations/proof-derived-parallelism/probes/q4.wf`, `research/investigations/proof-derived-parallelism/probes/zero_elig.wf` |

`reject-sysname-collision-in-kind-unit.wf` contains the spelling `open_read`
as the deliberately colliding source declaration, not as a system operation;
it receives no syntax rewrite and must retain its TYPE-6 verdict. Compiler unit
tests and embedded source strings migrate with the implementation and are
closed by a repository-wide old-surface search before activation.

## 12. Compiler-side work sizing

This sizing identifies invariant-bearing implementation units; it is not
authority to activate the candidate. The migration has two substantial pieces
and seven mechanical or downstream units.

| unit | implementation boundary | principal files |
|---|---|---|
| **1. Region-kind judgment** | Add `Memory`/`World` checked kinds; collect direct anchors from modes, `slice`/`arena`, capability-vector slots, allocation rows, and entry roots; propagate kind constraints through user/system call actuals; reject mixed or unresolved declarations; keep `region_stmt` memory-only. Contract alpha-renaming includes kind. | `compiler/src/semantic/model.rs`, `compiler/src/semantic/check/borrows.rs`, `compiler/src/semantic/check/generics.rs`, `compiler/src/semantic/check/types.rs`, `compiler/src/semantic/check/contracts.rs` |
| **2. Capability world-region representation (hard piece)** | Key each world-bearing opaque instance by `(system family, complete world vector)` while retaining one normal nominal/type path. Parse and kind-check system nominal `targs`; substitute vectors recursively through user-call parameters/results and `Result`, `Option`, user aggregates, buffers, and boxes; keep STOR-5's memory-region-bearing test limited to `slice`/`arena`. | `compiler/src/resolution/catalog.rs`, `compiler/src/semantic/model.rs`, `compiler/src/semantic/check.rs`, `compiler/src/semantic/check/nominals.rs`, `compiler/src/semantic/check/nominal_instances.rs`, `compiler/src/semantic/check/types.rs`, `compiler/src/semantic/check/expressions/calls/user.rs` |
| **3. EFF-2 projection (hard piece)** | Split normalized rows into memory/world read/write sets. Preserve memory projection through loans and storage origins; substitute world formals directly through every matching capability occurrence independent of `own`/shared/unique mode; instantiate system result vectors and recursively instantiate release rows from contained capabilities. Missing occurrence, substitution, or kind data fails closed. | `compiler/src/semantic/check.rs`, `compiler/src/semantic/model.rs`, `compiler/src/semantic/check/expressions/calls/user.rs`, `compiler/src/semantic/check/expressions/calls/system.rs`, `compiler/src/semantic/check/cleanup.rs`, `compiler/src/semantic/check/contracts.rs` |
| **4. System declaration data** | Replace the mode-derived-only row with explicit kinded memory/world accesses, nominal vector arities, result-vector mappings, origin/mint relations, and target-action records. Preserve exactly fifteen operations and make the resolver preorder total 246 records. | `compiler/src/resolution/catalog.rs`, `compiler/src/resolution/engine.rs`, catalog/resolution tests |
| **5. Permission** | Add world accesses to pair and loop footprints; world writes conflict with every may-alias world access and unresolved alias data conflicts universally. Generalize one call projection over user and direct system targets, then remove the `external`/`blocks` and blanket-system gates. Target-action metadata does not decide permission; lack of an I/O lowering route leaves the window sequential. | `compiler/src/semantic/permission.rs`, `compiler/src/semantic/loop_permission.rs`, permission ledgers and tests |
| **6. Entry and syntax** | Admit only world-kind entry region parameters; check exact cwd/output vectors and stdout/stderr root equations; update the canonical header. Remove the two effect alternatives while retaining their explicit lowercase reservation under D4, then regenerate parser tables. | `compiler/src/semantic/check/entry_form.rs`, `compiler/src/syntax/terminal.rs`, `compiler/src/bin/grammar.rs`, `compiler/src/grammar_tables/model.rs`, `compiler/src/syntax/grammar/generated.rs` |
| **7. Target-action lowering summary and ledger** | Carry dispatch/host-wait/loan-end metadata on system operations and releases, derive the transitive closed-world wrapper summary, and require a completion-frame route at actualization without turning an unavailable route into source rejection. Emit deterministic `--io-ledger` rows from the same checked records; the ledger never re-derives semantics. | checked-call/release metadata, driver/ledger rendering, lowering, `compiler/src/backend/par_runtime.c`, the completion translation units |
| **8. Provenance under D5** | Rename only the independent PRV data-dependency class and implementation fields; do not read world rows or change any PRV verdict. | PRV model/checker/diagnostics and the provenance conformance prose |
| **9. Activation mechanics** | Migrate all 42 review cases without verdict drift, the nineteen additional syntax-bearing `.wf` workloads, and every compiler fixture string; archive v0.36 byte-exactly; update the digest chain, generated identity, six digest anchors, active version literals, META-5 delta, and dated backend review marker. | `spec/`, `tests/conformance/`, `tests/programs/`, the named research workloads, `compiler/src/spec_identity.rs`, `compiler/src/spec.rs`, `compiler/src/backend/qualification.rs`, `governance/APPROVALS.md` |

The focused compiler test matrix is:

1. region-kind positives and negatives: memory/world call substitution, mixed
   use, unresolved use, a world name in `region_stmt`, a memory name in a
   capability slot, and kind-sensitive contract alpha-equality;
2. capability identity: exact/wrong/bare vectors, unequal vectors, vectors
   inside `Result`, `Option`, a user payload, a buffer element, and a box, plus
   repeated calls using one static may-alias class;
3. both EFF-2 directions through own/shared/unique parameters, nested results,
   user wrappers, system calls, conditional release unions, and the three
   native-close rows;
4. permission positives for world-plus-independent-compute and negatives for
   same order/sink/object identities, unequal-but-unproved identities,
   unresolved projection, direct system actions in loops, and releases in a
   window; claim-bearing closures remain eligible under T3;
5. parser cases proving bare `external`/`blocks` no longer form effects while
   REGIONID `'external` and LABEL `@blocks` remain legal under D4;
6. an old/new conformance verdict diff over all 42 migrated files, the seven
   exact manifest records, unchanged PRV verdicts, and exact `AABB` bytes for
   the same-sink runtime witness.
