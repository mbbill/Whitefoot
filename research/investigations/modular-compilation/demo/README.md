# Two targets over one module graph

This is a complete **source design specimen** for the proposed module system:
every declared function has an implementation, both entries have complete
bodies, and all application dependencies are present. It is not currently an
executable Whitefoot project. The active compiler does not accept `.wfg`,
`.wfm`, qualified module paths, file-local aliases or role-specific field visibility.
The proposed forms are identified below; no build
command, successful compiler run or incremental timing is implied.

The application processes two jobs through a four-slot FIFO. The `kernel`
target keeps everything by value and requires `no_heap`. The `inspect` target
runs the same batch and copies its report into a heap cell. Both entries check
the resulting fields and return an `ExitStatus`; neither prints output. The
kernel entry is a small allocation-free workload, not an operating-system
boot image or a target-runtime qualification.

The specimen belongs to the [modular-compilation investigation](../DESIGN.md).
Its reader is evaluating source organization, interface completeness, proof
composition and edit impact. Keep the sources and this walkthrough together;
replace them when the selected syntax or application changes, and remove them
when they no longer expose a useful design question. If this becomes a formal
compiler regression, extract the needed cases into the maintained test system.
It is not a daily CI input.

## Read the project in this order

1. [modules.wfg](modules.wfg): the entire dependency architecture and both
   target declarations.
2. [data/module.wfm](data/module.wfm): two public data records and one generic
   allocating function. No field-access wrappers are required for these records.
3. [runtime/queue/module.wfm](runtime/queue/module.wfm): the complete queue
   definition with private storage, its getter and every operation's bounds.
4. [runtime/module.wfm](runtime/module.wfm): a client of that API which states
   its own precondition and accepts a function-kind argument with the queue's
   contract.
5. [kernel/start.wf](kernel/start.wf) and
   [tools/inspect/run.wf](tools/inspect/run.wf): complete uses of the same APIs,
   with different entry requirements and different local abbreviations.
6. [runtime/queue/storage.wf](runtime/queue/storage.wf),
   [runtime/queue/operations.wf](runtime/queue/operations.wf),
   [runtime/batch.wf](runtime/batch.wf),
   [runtime/report.wf](runtime/report.wf) and [data/heap.wf](data/heap.wf):
   the implementation behind those contracts.

The two entry interfaces, [kernel/module.wfm](kernel/module.wfm) and
[tools/inspect/module.wfm](tools/inspect/module.wfm), contain only their ordinary
public entry declarations. An entry is selected by the graph; it is not a special
function syntax or an implicit `main` name.

```text
demo/
  README.md
  modules.wfg
  data/
    module.wfm
    heap.wf
  runtime/
    module.wfm
    batch.wf
    report.wf
    queue/
      module.wfm
      storage.wf
      operations.wf
  kernel/
    module.wfm
    start.wf
  tools/
    inspect/
      module.wfm
      run.wf
```

## What the graph says

The directory containing the sole active `modules.wfg` is this project's
source root. No root declaration or project nickname is needed. The fixed
`pkg::` prefix means the current source package, whether it builds a library
or an executable. It does not mean the current directory of the compiler
process or the directory of the individual `.wf` file. This demo has one
source package containing five modules and two executable targets.

The five rows register five exact `module.wfm` interfaces. The filename is
always the same; the containing directory supplies the module name. An arrow
below means "may use that module's public API"; it is not a runtime call or
scheduling edge.

```mermaid
graph TD
  K[pkg::kernel] --> D[pkg::data]
  K --> Q[pkg::runtime::queue]
  K --> R[pkg::runtime]
  I[pkg::tools::inspect] --> D
  I --> Q
  I --> R
  R --> D
  R --> Q
  Q --> D
```

Every listed dependency is an earlier row. That local check certifies the DAG;
the compiler does not infer a desired order from source uses. These rows give
permission, not implicit name imports. An earlier row absent from a module's
list is still unavailable to that module's source.

`runtime` is both a module and a namespace parent. Its implementation consists
of **only** `runtime/batch.wf` and `runtime/report.wf`. It does not collect
`runtime/queue/*.wf`. The separately declared child precedes its parent in
the graph, and the parent's row explicitly grants access. Parenthood grants
no private-field access. `tools` has no `module.wfm` and is only a namespace
prefix. The demo root likewise has no `module.wfm` and is not an extra module.

For example, `pkg::runtime::queue` always starts at `demo/` in these sources.
An external dependency would have its own explicit name, such as `std` or
`math`; source inside that selected dependency would use `pkg::` for its
own root. The compiler must retain that source ownership when checking an
imported library. It must not reinterpret a library's internal `pkg::`
paths against `demo/`, search for another active graph or infer missing edges.
This single-package specimen does not select the external dependency-binding
format or demonstrate cross-package reuse. External-library binding and
library-to-library dependencies are deferred beyond the next implementation;
this demo exercises one package's module DAG.

The tool reaches across directories to `runtime::queue` without moving that
module to a common ancestor. It lists both `runtime` and `runtime::queue`
because it directly uses both. Removing the latter edge is an error even
though the former still reaches the queue transitively. A future edge from
the queue back to its parent would fail the earlier-row check; retaining both
directions would be a cycle, and row reordering cannot make that legal.

## The public boundary and the private implementation

| Module | What the caller can read in its `.wfm` | Implementation ownership |
|---|---|---|
| `pkg::data` | All `Job` and `Report` fields; `boxed_copy`'s complete generic signature | `heap.wf` implements the callable and reuses the interface's record declarations |
| `pkg::runtime::queue` | Public capacity, the one complete `Queue` definition with private storage, ordinary runtime `len`, constructor, push and pop contracts | `storage.wf` owns the private constructor helper and getter body; `operations.wf` owns the other public bodies; both reuse the interface's type definition |
| `pkg::runtime` | `run_two` and the complete required contract of its `take` argument | `batch.wf` calls the private `summarize` in `report.wf` through the shared module inventory |
| `pkg::kernel` | The selected `start` callable | `start.wf` constructs jobs and a stack-resident queue |
| `pkg::tools::inspect` | The selected `run` callable | `run.wf` also invokes the shared heap helper |

Top-level declarations and struct fields are private unless marked `public`,
and only `.wfm` permits that modifier. Neither aliases nor implementation files
can publish a name. `Job` and `Report` explicitly publish both their types and
each field; their complete schemas make them copyable and droppable. Callers
construct them and inspect their fields directly. The interface schemas are
reused inside their owning module; implementation files
do not declare another `Job` or `Report`.

`Queue` has its one complete definition in `.wfm`: the type is public and its
unmarked `storage: Ring<Job, capacity>` field is private. The ordinary component
rules make this nongeneric type droppable, and `nocopy` forbids copying. There
is no second definition in `storage.wf` and no separate abstract capability
declaration to match. The representation is readable in the interface but
inaccessible to external source expressions. Callers may own it by value and
borrow it, but cannot construct it by fields or name
`storage`. There is no implicit allocation, handle, type invariant or
reference-returning getter. The existing `opaque` modifier is not used as
a privacy mechanism.

`new_storage` is called from a different implementation file without being
declared in `runtime/queue/module.wfm`. Likewise `summarize` is absent from
`runtime/module.wfm`.
All declarations in a module share its inventory, regardless of source-file
order. `capacity`, although declared in the interface, is reused by the
private storage field and implementation helpers; no duplicated constant can drift.

Each public callable's implementation repeats its full header and contract
without the interface-only `public` modifier.
In `runtime/queue/module.wfm` the job type is called `Job`; in `operations.wf`
it is called `Work`. Both aliases resolve directly to `pkg::data::Job`, so the declarations
must match after resolution. `runtime/module.wfm` uses an alias for `len`, while
`batch.wf` uses the module alias `fifo`; these also resolve to one callable.
Result and named-argument labels remain the same.

Every file declares its own aliases. An interface alias is not inherited by
its implementations, and `runtime` does not re-export `Queue`, `Job` or `len`.
Clients need their own direct graph edges and aliases. A canonical declaration
identity does not change when one local abbreviation changes.

For concurrent work, one agent can own the queue's interface and implementation
while another owns the batch consumer. An implementation-only edit with the
same checked boundary need not change the other agent's source. API changes
still require coordination with actual consumers; architectural edge changes
meet in `modules.wfg`. The small module sizes here expose those relationships
for reading and are not a recommendation to split production work this finely.
The parent's assigned files exclude its child module's directory. The fixed
interface basename keeps a module's own work together, but makes relative
paths important when several `module.wfm` files are open. Neither sibling
`data.wfm` nor repeated-name `data/data.wfm` is an alternative interface form.

This ownership boundary also does not prescribe one LLVM module or object
per source module. Body proofs and backend partitions have their own tracked
dependencies. Moving a private helper between source files creates no
writer-managed internal link boundary, and final native linking may run in
full. The edit scenarios below describe the desired finer reuse.

## The proof and execution story

The following is the **intended argument**, not a compiler verification result.
The selected erased private-field and state-transfer judgments are described
in the next section; their compiler implementation remains outstanding.

| Point in either entry | Queue length known from the public contracts | Why the next operation is permitted |
|---|---|---|
| `new` returns | 0 | `0 < capacity`, where the public constant is 4 |
| First `push` returns | 1 | There is room for the second job |
| Second `push` returns | 2 | `run_two` requires at least two jobs |
| First `take` inside `run_two` returns | At least 1 | The first call's length relation proves the second call's precondition |
| Second `take` returns | Entry length minus 2 | This proves `run_two`'s own length relation; these entries reach 0 |

`pop` satisfies the function-kind formal's exact type, effect and length
contract. Supplying `fn fifo::pop` is an ordinary compile-time argument under
the proposed module names; it does not allocate a function object or introduce
dynamic dispatch. Both targets select the same concrete
`runtime::run_two<fn runtime::queue::pop>` instance.

Inside the queue, `push` and `pop` directly name the private Ring length in
requires/ensures, just as their bodies use the ordinary window operations.
The Ring has capacity 4. The external `run_two` interface and its function-kind
formal repeat the same private paths under erased annotation visibility; no
executable access to Queue's storage is granted.

`len` is an ordinary getter with a written `result == private length`
postcondition proved by its body. Its call in `runtime/batch.wf` after both
removals supplies the report's `remaining` field. Normal-return substitution
carries that relation to the caller; there are no getter calls in contracts.
The `writes(queue.storage)` row kills overlapping live facts before the verified
postcondition supplies the new state's facts. An old runtime getter result
remains an integer, not a reference to the future length.
`deref(entry(queue)).storage.len` is the frozen entry datum in an `ensures`,
with no runtime snapshot. `made.storage.len` describes the constructor's
returned value and is transported through construction, return and binding.

The FIFO bodies select jobs `(tag: 7, payload: 250)` and
`(tag: 9, payload: 10)` in that order. The private report helper computes an
explicit `+wrap` checksum: `(250 + 10) mod 256 = 4`. The intended report is:

```text
Report(first_tag: 7, second_tag: 9, digest: 4, remaining: 0)
```

Both entries should return exit code **0**. Codes 1, 2, 3 and 4 denote a
mismatched first tag, second tag, digest or remaining count respectively.
These checks test application results; they do not repair a failed source
proof. The written contracts state occupancy safety, not a formal sequence
model of FIFO contents or the exact report fields. The latter expectations
follow from these complete bodies and the ring operations and still need
execution tests once the module path exists.

The kernel's source composition includes `data`, the queue, `runtime` and
`kernel`. Every definition in those modules receives ordinary checking,
including the allocating generic helper's schema. Its conservative execution
closure never selects `boxed_copy`, `Box<Report>` or their release path, so
that helper does not impose a heap requirement on this target. The separate
tool module is not part of the kernel's selected source composition.

The inspect target selects the same three library modules and its own entry.
It instantiates `boxed_copy<Report>`, allocates one report cell and releases it
on every return edge. Both `boxed_copy` and the entry write `pure`, which in
WF does not mean allocation-free. `no_heap` belongs to the kernel target in
the graph, not to any of these reusable modules or functions.

Generated object selection and native/runtime supplies must respect that
distinction as well. An object emitted for the shared module must not force
the kernel to resolve an unused allocator symbol merely because the tool's
helper was compiled. This specimen states that required result; it does not
demonstrate a linker or backend achieving it.

## Proposed notation used here

These sources follow the selected [boundary rules](../LANGUAGE.md) and
[qualified syntax](../SYNTAX.md). The grammar qualification does not execute
the specimen or establish a compiler implementation.

| Form | Meaning |
|---|---|
| `pkg::`, ordered graph rows and named targets | One implicit source root, exact earlier dependencies and target requirements |
| `directory/module.wfm` | Complete declarations, one complete public representation, explicit `public`, no executable function bodies |
| File-local `alias` headers | Abbreviations with the original identities and direct-edge checks |
| `deref(queue).storage.len` in an annotation | A typed private-field path; it adds no runtime access or effect |
| `reads(queue.storage.len)` / `writes(queue.storage)` | Exact structural effects, also writable in external wrapper/formal annotations |
| Ordinary `fn len(...)` with a result-to-field `ensures` | Runtime getter; its verified postcondition is available after normal return |
| `deref(entry(queue)).storage.len` | Frozen mathematical entry value, independent of later mutation |
| `made.storage.len` | Aggregate result projection transported through construction, return and caller binding |

Executable callers still cannot read, write, borrow, construct or destructure
private Queue storage. A caller may name it in requires/ensures, invariants,
explicit proof premises and effect rows. That distinction lets a wrapper or
manual proof state its own obligations without a runtime getter solely for
proof naming. An annotation still checks path types, domains, state validity
and direct module edges; it cannot assume its own conclusion.

Private paths mentioned by a contract or client proof become semantic
dependencies. A relevant representation edit may require changing that proof,
while an ordinary getter body edit with unchanged verified claims need not
reprove its callers. No `observe`, `use view`, footprint declaration, trusted
axiom, implicit type invariant, runtime snapshot or mandatory box is needed.

Implementing these rules must make both entries check and execute, qualify the
rejection probes below, and include the larger GrowVector wrapper/function-kind
witness from LANGUAGE.md. The FIFO alone is not evidence for all containers,
precise effect combinations or incremental performance.

## Edits to try while reading

These are predicted consequences to qualify in the future implementation,
not measured invalidation results. Each experiment starts from this specimen.

| Edit | Intended source result | Incremental work that must follow |
|---|---|---|
| Rewrite `summarize`'s body with equivalent operations and unchanged header | Public APIs and permissions remain unchanged | Recheck that helper and any affected private summary users; update code importing its implementation, then link. Do not reprove every library solely because one file changed |
| Move `summarize` to another direct `runtime/*.wf` file, preserving its resolved aliases | Same private declaration identity; still callable from `batch.wf` | Refresh file membership and locations; reuse semantic work only when the resolved declaration/context and dependencies are unchanged |
| Rename the `Work` alias in `report.wf`, updating its uses | Same canonical type, no change in other files | Refresh that file's formation/resolution; normalized semantic results may remain reusable |
| Change the private `Queue.storage` declaration in `.wfm` | Executable access stays private; contracts/client proofs naming its paths, derived capabilities or layout can change | Recheck actual path/contract, capability/ownership, getter, body, layout/release/ABI and optimizer consumers; source proofs reuse only when their consumed facts are unchanged, not merely because the field is private |
| Change `Report`'s public field schema | Callers may need source changes | Revalidate schema, field/type/ownership users and layout/codegen consumers; publication is a real dependency |
| Delete `pkg::kernel`'s direct queue edge while keeping its runtime edge | Reject the kernel's queue aliases/uses | Revalidate the edge/lookup consumers; transitive reachability grants no source permission |
| Change only the tool's aliases or body | Kernel source proofs are unchanged when their actual inputs are unchanged | Recheck tool consumers and its changed specialization/optimizer dependencies; no blanket graph-file or target key should reprove all shared bodies |
| Add `no_heap;` to `target inspect` | Reject its reachable `boxed_copy<Report>`/`Box<Report>` heap requirement | Recheck target composition against shared heap summaries; do not reinterpret every function's ordinary proof |
| Add the same storing call to `kernel/start.wf` | Reject the kernel's reachable allocation before optimization | Recheck the changed body, concrete execution closure and target requirement; optimizer deletion is not permission |
| Change an unselected allocating helper to contain an unproved partial operation | Reject the selected module's ordinary source checking | No-heap reachability is not an exemption from source safety |

For additional rejection probes, try calling `fifo::new_storage` externally,
reading `pending.storage` in an executable expression, changing only `push`'s implementation contract,
placing the getter body in `.wfm`, adding `public` to any `.wf` declaration,
removing `public` from `Job.tag` while its external uses remain, repeating the
`Queue` definition in `.wf`, rebinding `pkg`, moving an interface to
the former sibling location, or adding an edge to a later graph row.
Each should fail for that specific boundary; an unrelated earlier syntax
failure is not evidence for it. These are review exercises until the real
compiler implements the proposed forms.

A future CI review aid should compare the resolved public API, not only changed
lines containing `public`. Changing a published field type or a getter contract
without editing its modifier must still be reported. Private storage changes
that alter public capabilities or proof paths also matter; layout-only effects can be reported
separately. No comparison script or new review gate is implemented here.

The eventual executable qualification must check both targets, the rejection
probes, no allocator dependency in the kernel artifact, and cold/incremental
agreement for these edits. The current validation is source/design review
and repository structural checks only. There are no module compilation,
execution, runtime-performance or incremental-work measurements here.
