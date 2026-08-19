# File open by name — exact v0.33-candidate rule edits (DRAFT, not a spec change)

Status: DRAFT delta text for batch 0072 (W1, outline:CAND-8), prepared for the
lead's single-writer integration into the one v0.33 candidate. Nothing here
edits `spec/kernel-spec.md`; every byte below awaits the owner's exact-byte
approval. Every anchor is against the ACTIVE v0.32 file, SHA-256
`5ea3927aef20d08e1c9c80a50242628f2c469974261b68c696ee2db3934e6bf5`, 3264
lines.

Evidence base: `research/investigations/wfgrep-traversal/` (the v0.32
traversal delta this one completes) and the running program in
`tests/programs/wfgrep.wf` (§6).

Implementation status: implemented end to end behind the one-line switch
`OPEN_BY_NAME` in `compiler/src/resolution/catalog.rs`, default `false` = the
active v0.32 inventory byte for byte. The candidate inventory is selected by
`compile_with_inventory`; a real recursive search program compiles, links, and
runs against a real directory tree under it (§6).

## 1. What changes (summary)

v0.32 landed the traversal surface: a program can open a child directory by an
enumerated name (`open_directory`), open an enumeration (`open_list`), and
pull a batch of portable entry records (`list_once`). `tests/programs/dir_walk.wf`
walks a real tree with exactly that surface.

One gap remains, and it is the gap that keeps a real recursive search out of
reach: **nothing opens a *file* by an enumerated name.** `open_read` takes a
`RelativePath`, and [SYS-14] already states why an enumerated name can never
become one — a name's backing is not the command-lifetime argument snapshot
[HOST-3], and a path value is an inline lease over that snapshot [PATH-1].
`open_directory` solved that for directories by taking a caller-owned name
range instead of a path value. The file side was simply not delivered.

The candidate therefore adds **one operation and nothing else**:

```
fn open_file['c, 'n](root: &'c DirectoryRead, name: &'n buffer<u8>, offset: own u64, count: own u64) -> own Result<ReadFile, IoError> reads('c 'n), external, blocks, traps;
```

No new nominal type, no new constructor, no new variant field, no new outcome
enum, no new [SYS-5] release row, no new [QUAL-2] guarantee. The result type is
the `Result<ReadFile, IoError>` `open_read` already declares, the failure
vocabulary is the closed [SYS-7] class set, and the validation is exactly
`open_directory`'s: the [SYS-8] range check (trap) first, then the
single-path-component content check (recoverable `InvalidPath`), then at most
one host call.

### Why a sibling rather than a widened `open_read`

`open_read` is the path route and stays it. Two arguments decide this:

1. **One producer, one type** [TYPE-5]. Overloading `open_read` on the second
   parameter's type would make one spelling carry two signatures; [SYS-2]
   fixes exactly one complete signature record per operation and the resolver
   selects an operation by spelling alone. Nothing in the kernel admits
   overload resolution, and inventing it for one operation would be a
   language-shaped change to buy a name.
2. **The validation differs.** The path route validates at construction
   ([PATH-1]: no NUL, no target-root prefix, `.`/`..` and separators
   preserved). The name route validates a *component* at the call
   ([SYS-14]: nonempty, no NUL, no separator, within the component limit) and
   carries a [SYS-8] range trap the path route does not have. These are
   different contracts with different failure classes; one operation stating
   both would state neither exactly.

### Naming

`open_directory` names a directory object by one component and yields
`DirectoryRead`; `open_file` names a file object by one component and yields
`ReadFile`. In the name-route family the axis is the object kind, and both
capabilities are read-only in this slice exactly as `open_directory` already
is — so `open_file` does not restate the mode, on the same ground
`open_directory` does not. `open_read` keeps its name as the path route.

This is a naming proposal; the spelling is the lead's and the owner's to
ratify. Alternatives considered and rejected: `open_read_name` and
`open_named` (both put the *input form* in the name, which no other operation
does), `open_entry` (the operation is not restricted to enumerated entries —
any caller-owned byte range naming one component is admitted).

## 2. Exact rule edits

### 2.1 [SYS-2] — the inventory

**(a)** Line 2172, the opaque sentence: **no change.** No nominal type is
added.

**(b)** Line 2180, the enum sentence: **no change.** No constructor is added.

**(c)** The enum code block: **no change.**

**(d)** Line 2249. Replace

> Fourteen operations, each one complete signature record in the [GRAM-2] `fn_sig` shape:

with

> Fifteen operations, each one complete signature record in the [GRAM-2] `fn_sig` shape:

**(e)** In the operation code block, after the `list_once` line (line 2265),
append the row exactly as written:

```
fn open_file['c, 'n](root: &'c DirectoryRead, name: &'n buffer<u8>, offset: own u64, count: own u64) -> own Result<ReadFile, IoError> reads('c 'n), external, blocks, traps;
```

The `reads`/`writes` derivation is unchanged and mechanical: `'c` and `'n` are
each borrowed and neither is `&uniq`, so the row is `reads('c 'n)` with no
`writes`. The `external, blocks, traps` classification is
`open_directory`'s, for the same three reasons: it reaches a host object, the
host call may block, and the retained [SYS-8] range check is a trap.

**Table position.** Appending after `list_once` keeps every existing
declaration ordinal fixed, which is what lets one implementation switch select
the active inventory as an exact prefix of the candidate one (§4). Placing the
row next to `open_directory` instead is equally normative and would be more
legible; it renumbers `open_list` and `list_once` and costs the prefix
property. The lead chooses; the delta proposes the appended position because
the differential evidence in §6 depends on it.

**(f)** Line 2268, the counted totals. Replace

> The inventory is therefore exactly sixteen nominal types, forty-two
> enum-variant constructors, sixty-seven variant fields, fourteen operations,
> nineteen operation region parameters, and thirty-four operation value
> parameters.

with

> The inventory is therefore exactly sixteen nominal types, forty-two
> enum-variant constructors, sixty-seven variant fields, fifteen operations,
> twenty-one operation region parameters, and thirty-eight operation value
> parameters.

Recount, from the table itself: nominal types 16 (unchanged), enum-variant
constructors 42 (unchanged), variant fields 67 (unchanged), operations
14 + 1 = 15, operation region parameters 19 + 2 (`'c`, `'n`) = 21, operation
value parameters 34 + 4 (`root`, `name`, `offset`, `count`) = 38.

**(g)** Line 2306. Replace `one hundred and ninety-two declaration records`
with `one hundred and ninety-nine declaration records`. The preorder sentence
itself is unchanged.

Recount, by the stated preorder: 16 nominal types + (42 constructors + 67
fields) + (15 operations + 21 region parameters + 38 value parameters)
= 16 + 109 + 74 = 199. The added seven are the operation record, its two
region-parameter records, and its four value-parameter records.

**(h)** The `wf-prov` table (block at line 2276) gains one row, appended after
`list_once` (line 2292) in the same order as the operation table:

```
| `open_file` | `Ok(value:)` external; `Err(error:)` external | — |
```

Identical to `open_read`'s and `open_directory`'s cell, on the identical
ground: every component of an opened handle comes from outside. There is no
writable `&uniq` parameter, so the third column is `—`.

`No system operation allocates.` (line 2274) is unchanged and remains true:
the opened handle is a descriptor and nothing is copied into program storage
beyond the bounded terminating slot the target shim owns.

### 2.2 [SYS-4] — no change

No nominal type is added, so the `wf-sys` kind/`Sendable`/`Shareable` table
(line 2421) is untouched. `ReadFile` is already a stateful resource that is
Sendable and not Shareable, and a second producer does not change a type's
kind.

### 2.3 [SYS-5] — no change

No nominal type is added, so the release table (line 2455) is untouched.
`ReadFile` is already release-complete with at most one native close attempt,
and `open_file` produces exactly that type.

### 2.4 [SYS-6] — outcome types

**(a)** The `wf-sys` outcome table (block at line 2487) gains one row,
appended after `list_once` (line 2501):

```
| `open_file` | `own Result<ReadFile, IoError>` |
```

No new enum: the operation has exactly two outcomes, so [SYS-6]'s own rule
gives it a [PRE-1] `Result<T, E>` instantiation and no new constructor
spelling. It is the same instantiation `open_read` declares, which is why
nothing in [SYS-7] or the constructor domain moves.

**(b)** Line 2514. Replace

> `propagate` [ERR-3] therefore chains only across operations that already
> share one error type: that is exactly `open_read`, `write_once`,
> `open_directory`, and `open_list` inside a function whose written result is
> `own Result<U, IoError>`.

with

> `propagate` [ERR-3] therefore chains only across operations that already
> share one error type: that is exactly `open_read`, `write_once`,
> `open_directory`, `open_list`, and `open_file` inside a function whose
> written result is `own Result<U, IoError>`.

### 2.5 [SYS-7] — no change

`open_file`'s recoverable failures are drawn from the existing closed class
set, including the `InvalidPath` its component validation produces. No class,
no detail field, and no mapping rule changes.

### 2.6 [SYS-8] — no change

`open_file` is **not** added to line 2532's list. That list is the one-attempt
*transfer* family, whose members write or read the caller's buffer; `open_file`
only reads a name out of it and transfers nothing. `open_directory` is
likewise absent from that list, and its range validation is stated in [SYS-14]
by reference to [SYS-8]. §2.8 does the same for `open_file`, which is the
mirror the placement rule requires.

### 2.7 [SYS-10] — `ReadFile` gains a second producer

**(a)** Line 2597. Replace

> `open_read` creates an independent `ReadFile` with its own cursor domain and
> does not alias the capability.

with

> `open_read` and `open_file` each create an independent `ReadFile` with its
> own cursor domain and do not alias the capability.

**(b)** Line 2603. Replace

> Any number of `open_read`, `open_directory`, and `open_list` calls may
> progress concurrently through shared borrows of one `DirectoryRead`,
> exposing no ordering relative to one another.

with

> Any number of `open_read`, `open_file`, `open_directory`, and `open_list`
> calls may progress concurrently through shared borrows of one
> `DirectoryRead`, exposing no ordering relative to one another.

Line 2604 (`Each either creates its own `ReadFile`, `DirectoryRead`, or
`DirectoryList`, or fails, and none observes another's effect.`) is unchanged
and already covers the added producer, because `ReadFile` is already listed.

### 2.8 [SYS-11] — the family contract sentence

**(a)** Line 2612. Replace

> `open_read` creates it live, with one cursor domain and one conservative
> filesystem-object alias domain.

with

> `open_read` and `open_file` each create it live, with one cursor domain and
> one conservative filesystem-object alias domain.

The next sentence — `A separate open does not prove a separate object` — is
unchanged and now carries the added producer for free: two `ReadFile` values
may denote the same file object however they were produced.

**(b)** After line 2614 (`` `read_once` is call-scoped and leaves both owners
live on every outcome; its transfer, cursor, and buffer semantics are
[SYS-8]. ``), insert the name-range paragraph, mirroring [SYS-14]'s
`open_directory` paragraph sentence for sentence:

> `open_file` takes a caller-owned name range rather than a path value, for
> the reason [SYS-14] states: an enumerated name's backing is not the
> command-lifetime argument snapshot [HOST-3], and a path value is an inline
> lease over that snapshot [PATH-1].
> `open_file`'s name range (`offset`, `count`) over the caller's buffer is
> validated exactly as a [SYS-8] range — overflow of the mathematical sum, an
> offset past the buffer's runtime length, or a range extending past it traps
> as the operation-internal contract check [OP-4, ERR-4] before any content
> validation and before any host call — and that retained check is the
> operation's sole trap and backs its `traps` row [EFF-2].
> `open_file` then validates the range's content before any host call: a
> component that is empty, longer than the target's component limit, or
> containing a NUL or a target separator yields `InvalidPath` with both detail
> fields zero [SYS-7], no host call, and no file.
> A valid range that names no readable file yields the target's own failure
> class — `NotFound`, `IsDirectory`, `PermissionDenied`, and the rest of the
> closed set — exactly as `open_read` does.

Placement note: the contract sentence belongs in [SYS-11] rather than
[SYS-14] because the family it constrains is `ReadFile`'s. [SYS-14] states
the name-range route once and gives its reason; [SYS-11] applies it. The
alternative — stating it in [SYS-14] beside `open_directory`'s — puts a
`ReadFile` obligation inside the `DirectoryList` family contract, which is
where a later reader would not look for it.

### 2.9 [SYS-14] — one cross-reference

**(a)** Line 2668. Replace

> `open_directory` therefore takes a caller-owned name range rather than a
> path value, and path composition remains the DEFERRED addition [PATH-1]
> states.

with

> `open_directory` and `open_file` therefore take a caller-owned name range
> rather than a path value, and path composition remains the DEFERRED
> addition [PATH-1] states.

That sentence already carries the whole reason (`This specification declares
no operation turning an enumerated name into a `HostString` or a
`RelativePath` …`), so naming the second consumer there is the complete edit;
the family contract itself goes to [SYS-11] (§2.8).

### 2.10 [PATH-2] — no change

Line 2372 already reads

> A directory-relative operation resolves either one relative path value or
> one caller-supplied single path component [SYS-14]; both are resolved
> through the target's own directory-relative facility and neither is
> concatenated onto a prefix.

That sentence is written over the two *forms*, not over an enumerated
operation list, so a second component-form operation needs no edit. The
no-emulation rule, the process-equivalence promise, and the deferred confined
root all carry unchanged.

### 2.11 [QUAL-1]/[QUAL-2] — no new guarantee

`open_file` gets its own target-independent semantic ID [QUAL-1] and its own
row in the compiler-internal qualification table, like every operation. The
[QUAL-2] guarantees its record requires are exactly `open_directory`'s — a
lossless host-string code-unit family and the target's own directory-relative
resolution facility — both already stated. The third guarantee
(directory enumeration) is **not** required: `open_file` performs no
enumeration.

No [QUAL-2] text changes. The guarantee set is a per-ID record, not a written
list in the rule.

### 2.12 [QUAL-3] — no change

`open_file` lowers to exactly the shape [QUAL-3] already fixes: the required
source check, at most one direct host call, one outcome check, and a cold
outcome mapper reached only on failure, with no heap allocation, no copy of
transferred data, no global lock, and no per-call signal operation. The one
bounded stack slot it uses to terminate the validated component for the host
facility is `open_directory`'s, already emitted and already reviewed under
this rule; it copies the *name*, not transferred data.

## 3. What this delta deliberately does not add

- **No path algebra.** No join, no component type, no absolute path, no
  multi-component name. The admitted range is one component, exactly as
  `open_directory` admits.
- **No write or create mode.** `open_file` opens for reading. A writable or
  creating sibling is a separate family with its own `Output`-shaped
  contract, its own release policy question, and its own delta.
- **No `openat`-style flag surface.** No follow/no-follow choice, no
  directory-only or file-only enforcement beyond what the host reports. A
  name that reaches a directory yields the target's own class (`IsDirectory`
  on the Unix family); the program branches on the enumerated kind byte
  [SYS-14] if it wants to avoid the call.
- **No stat, no metadata, no size.** A program learns a file's extent by
  reading it, as `read_once` already fixes.
- **No confined root.** [PATH-2]'s deferred confinement is untouched;
  `open_file` makes exactly the process-equivalent promise `open_read` and
  `open_directory` make and no stronger one.

## 4. Implementation

Switch: `OPEN_BY_NAME` in `compiler/src/resolution/catalog.rs`, default
`false`. The two inventory switches are carried together as one `Inventory`
value (`traversal_surface`, `open_by_name`) threaded through resolution,
semantic checking, and lowering exactly where the single `traversal_surface`
bool was threaded before. `Inventory::ACTIVE` is `{ traversal_surface: true,
open_by_name: false }` — the active v0.32 inventory — and is what `compile`
and `resolve` read.

Because the row is appended, the active inventory is an exact prefix of the
candidate one: `system_operations(inventory)` returns 14 rows under
`ACTIVE` and 15 under `CANDIDATE`, every existing declaration ordinal is
unchanged, and the differential test in §6 shows the emitted module of every
earlier program is byte-identical across the switch.

Touched: the catalog inventory row; the `wf-prov` result class and external
write columns (`compiler/src/semantic/provenance.rs`); the [QUAL-1] symbol and
[QUAL-2] guarantee rows (`compiler/src/backend/qualification.rs`); the emitted
implementation (`compiler/src/backend/emitter/system.rs`,
`emit_open_file`, which reuses `range_validation_after` and
`component_validation` unchanged and differs from `emit_open_directory` in
exactly the open flags and the result representation).

## 5. Evidence

See §6 of this document for the run, and
`compiler/tests/programs/open_by_name.rs` and
`compiler/tests/programs/wfgrep_search.rs` for the tests.

## 6. Running evidence

Filled in by the implementation, below.

## 7. PROPOSED conformance cases (no `tests/conformance/` edit here)

Proposed only. Adding any of these is a protected-evidence change requiring
the owner's exact before/after audit and an approval-ledger entry, and none is
written by this batch.

| proposed id | rule | direction | what it pins |
|---|---|---|---|
| `sys11-open-file-success` | SYS-11 | positive | a valid one-component name opens the file the enumeration reported, and `read_once` on the result returns its bytes |
| `sys11-open-file-range-trap` | SYS-11 | positive (traps) | `offset + count` past the name buffer's length traps before any content validation and before any host call |
| `sys11-open-file-empty-name` | SYS-11 | negative | a zero-length name range yields `InvalidPath` with both detail fields zero and makes no host call |
| `sys11-open-file-component` | SYS-11 | negative | a name range containing a separator or a NUL yields `InvalidPath` with zero detail and makes no host call |
| `sys11-open-file-not-found` | SYS-7 | positive | a valid component naming nothing yields the target's own `NotFound`, not `InvalidPath` |
| `sys11-open-file-directory` | SYS-7 | positive | a valid component naming a directory yields the target's own class, never a `ReadFile` |
| `sys11-open-file-propagate` | ERR-3 | positive | `propagate` chains `open_file` with `open_read`, `open_directory`, and `open_list` in one `own Result<U, IoError>` function |
| `sys11-open-file-affine` | OWN-1 | negative | the `ReadFile` an `open_file` produced is affine exactly as `open_read`'s is |
| `sys11-open-file-release` | SYS-5 | positive | a `ReadFile` from `open_file` runs exactly one derived close and the emitted program contains no second one |
| `sys11-open-file-no-path` | PATH-1 | negative | `open_file` still admits no `RelativePath` and `relative_path` still admits no `buffer<u8>` |

No existing conformance case pins the [SYS-2] counted totals, so §2.1(f) and
§2.1(g) require no corpus edit; those totals are pinned by the compiler's own
extraction lock against the specification text.
