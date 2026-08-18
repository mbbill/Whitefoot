# Directory traversal — exact v0.32-candidate rule edits (DRAFT, not a spec change)

Status: DRAFT delta text for batch 0071 (W1, outline:CAND-8), prepared for the
lead's single-writer integration into the one v0.32 candidate. Nothing here
edits `spec/kernel-spec.md`; every byte below awaits the owner's exact-byte
approval. Every anchor is against the active v0.31 file, SHA-256
`ea4b8ad4a56fbf43f3c98b91fc667da0b693c75b81807250a36454e03a197f1c`.

Evidence base: `research/investigations/wfgrep-traversal/RECON.md` (the gap
table and the design sketch). This delta closes exactly gaps 1, 2, 3, and 4/5
of that table and closes them the way the sketch proposed: entry names are
delivered by an [SYS-8]-style transfer into a caller-owned `buffer<u8>`, and
**no path value is ever formed**, so [HOST-3], [PATH-1], and [QUAL-2]'s
backing guarantee are untouched.

Implementation status: implemented end to end behind the one-line switch
`TRAVERSAL_SURFACE` in `compiler/src/resolution/catalog.rs` (default `false` =
the v0.31 inventory, byte for byte). The candidate inventory is selected by
`compile_with_traversal_surface`; a real recursive traversal program compiles,
links, and runs against a real directory tree under it (§6).

## 1. What changes (summary)

Three gaps block real recursive traversal in v0.31, and each is a missing
declaration rather than a compiler defect:

1. no operation produces a second `DirectoryRead`, so only `command.cwd` is
   ever reachable;
2. nothing enumerates a directory; and
3. an entry name has no route into program memory that does not first become a
   `HostString`, which [HOST-3] forbids for a producer whose backing is not
   command-lifetime.

The candidate adds **one opaque nominal type, one outcome enum with three
constructors, and three operations**:

- `DirectoryList` — one stateful directory enumeration with one entry cursor.
  Its target representation is one native descriptor, exactly like `ReadFile`;
  the whole cursor state is that descriptor's own position, so the value
  carries nothing else and no operation allocates.
- `ListOutcome` — `ListBytes(count:, entries:)`, `ListEnd()`,
  `ListFailed(error: IoError)`.
- `open_directory` — opens one child directory capability named by caller
  bytes.
- `open_list` — opens one enumeration handle over a held capability.
- `list_once` — one [SYS-8] one-attempt transfer that fills a caller-owned
  range with a portable entry-record sequence.

The decisive shape choice, and the reason the delta is this small: `list_once`
is a *transfer*, not an iterator. The host's own facility writes its native
records directly into the caller's range and the implementation rewrites them
in place as the portable form. Because the portable header (two bytes) is
smaller than any native record header, the rewrite only ever moves bytes
toward the front and no unread byte is overwritten. The consequences are that

- the enumeration handle needs no buffer, cursor field, or window of its own,
  so it is one descriptor and nothing else;
- the operation makes at most one host call, which is what [QUAL-3] requires
  of a synchronous transfer; and
- the whole entry-name delivery question (RECON gap 4) is answered by [SYS-8]
  machinery that already exists, with no second string type and no amendment
  to [HOST-3].

Names travel as bytes at the open sites too, so RECON gaps 5 and 6 — a route
from program bytes to a path value, and path composition — do not arise. The
cost is one new sentence in [PATH-2] admitting a single-component-name form of
a directory-relative operation.

## 2. The portable entry record

`list_once` writes a sequence of entry records into the caller's range. One
record is:

| byte | meaning |
|---|---|
| 0 | entry kind, one value of the closed set below |
| 1 | entry-name length in bytes, 1 through 255 |
| 2.. | exactly that many name bytes |

The closed kind set is `0` unknown, `1` regular file, `2` directory, `3`
symbolic link, `4` other. There is no multi-byte field, so the format has no
endianness and no alignment question, and a program reads it with the same
byte indexing it already uses on any other buffer.

`0` is a real outcome, not a placeholder: some hosts and filesystems classify
no entry at enumeration time. A program that must know the kind of an unknown
entry probes it with an ordinary open, exactly as `wfgrep.wf` already
distinguishes a directory today.

## 3. Exact rule edits

### 3.1 [SYS-2] — the inventory

**(a)** Line 2157, the opaque sentence. Replace

> Seven opaque nominal types: `Args`, `HostString`, `RelativePath`,
> `DirectoryRead`, `ReadFile`, `Output`, and `ExitStatus`.

with

> Eight opaque nominal types: `Args`, `HostString`, `RelativePath`,
> `DirectoryRead`, `ReadFile`, `Output`, `ExitStatus`, and `DirectoryList`.

**(b)** Line 2165. Replace `Seven enum nominal types with thirty-nine variant
constructors:` with `Eight enum nominal types with forty-two variant
constructors:`.

**(c)** In the enum code block, after the closing brace of `enum IoError`,
append

```
enum ListOutcome {
  ListBytes(count: u64, entries: u64);
  ListEnd();
  ListFailed(error: IoError);
}
```

**(d)** Line 2229. Replace `Eleven operations, each one complete signature
record in the [GRAM-2] `fn_sig` shape:` with `Fourteen operations, each one
complete signature record in the [GRAM-2] `fn_sig` shape:`.

**(e)** In the operation code block, after the `exit_status` line, append the
three rows exactly as written:

```
fn open_directory['c, 'n](root: &'c DirectoryRead, name: &'n buffer<u8>, offset: own u64, count: own u64) -> own Result<DirectoryRead, IoError> reads('c 'n), external, blocks, traps;
fn open_list['c](directory: &'c DirectoryRead) -> own Result<DirectoryList, IoError> reads('c), external, blocks;
fn list_once['l, 'd](list: &uniq 'l DirectoryList, destination: &uniq 'd buffer<u8>, offset: own u64, capacity: own u64) -> own ListOutcome reads('l 'd), writes('l 'd), external, blocks, traps;
```

The two-row `reads`/`writes` derivation is unchanged and mechanical: every
borrow parameter of formal region `'r` contributes `reads('r)`, and every
`&uniq 'r` parameter contributes `writes('r)` as well.

**(f)** Line 2245, the counted totals. Replace

> The inventory is therefore exactly fourteen nominal types, thirty-nine
> enum-variant constructors, sixty-four variant fields, eleven operations,
> fourteen operation region parameters, and twenty-five operation value
> parameters.

with

> The inventory is therefore exactly sixteen nominal types, forty-two
> enum-variant constructors, sixty-seven variant fields, fourteen operations,
> nineteen operation region parameters, and thirty-four operation value
> parameters.

**(g)** Line 2280. Replace `one hundred and sixty-seven declaration records`
with `one hundred and ninety-two declaration records`. The preorder sentence
itself is unchanged.

**(h)** The `wf-prov` table gains three rows, appended after `exit_status`:

```
| `open_directory` | `Ok(value:)` external; `Err(error:)` external | — |
| `open_list` | `Ok(value:)` external; `Err(error:)` external | — |
| `list_once` | `ListBytes(count:, entries:)` internal; `ListFailed(error:)` external; `ListEnd()` carries no result component | `destination` external; `list` external |
```

Both transferred counts are internal for the same reason `ReadBytes(count:)`
is: they are program-bounded by the caller's own validated range, and the
environment's choice of a position inside that bound does not make them
external. The sentence at line 2273 already states this and needs no edit.

`No system operation allocates.` is unchanged and remains true: the
enumeration handle is a descriptor and the records land in caller storage.

**Ordinals.** The two new nominal types occupy table positions 14 and 15, so
every constructor and operation ordinal shifts by two, and the three new
constructors shift every operation ordinal by a further six. This is ordinary
version data — an ordinal is an identity inside one specification version
[SYS-1] — but it is why the implementation switch selects one whole inventory
rather than patching the other (§5).

### 3.2 [SYS-4] — kind and capability predicates

Add one row to the `wf-sys` table, after `ExitStatus`:

```
| `DirectoryList` | stateful resource | yes | no |
```

Add one sentence to the paragraph that explains `ReadFile` and `Output`:

> `DirectoryList` is not Shareable on the same ground: an enumeration cursor
> has exactly one mutable owner, and two lanes over one enumeration would
> observe each other's advance.

### 3.3 [SYS-5] — completion policy and release

Extend the release-complete sentence to name `DirectoryList`, and add one row
to the release table after `ExitStatus`:

```
| `DirectoryList` | at most one native close attempt | `external, blocks` |
```

Extend the outcome-release sentence to name `ListOutcome` among the outcome
types that have no release action and take no row in the table.

### 3.4 [SYS-6] — outcome types

**(a)** Replace

> The one operation with more than two outcomes declares one enum whose
> variant spellings carry its operation prefix, so no two operations compete
> for a constructor name in the whole-unit constructor domain [TYPE-6].

with

> Each operation with more than two outcomes declares its own enum whose
> variant spellings carry that operation's prefix, so no two operations
> compete for a constructor name in the whole-unit constructor domain
> [TYPE-6].

**(b)** Add three rows to the outcome inventory:

```
| `open_directory` | `own Result<DirectoryRead, IoError>` |
| `open_list` | `own Result<DirectoryList, IoError>` |
| `list_once` | `own ListOutcome` |
```

**(c)** After the `ReadBytes`/`ReadEnd`/`ReadFailed` sentence, add

> `ListBytes(count, entries)`, `ListEnd`, and `ListFailed(error)` are
> [SYS-8]'s three enumeration outcomes; `count` is the exact byte length of
> the portable entry-record prefix written into the requested range and
> `entries` is the exact number of complete records that prefix holds.

**(d)** The `propagate` sentence names the operations that share `IoError`.
Replace `that is exactly `open_read` and `write_once`` with `that is exactly
`open_read`, `write_once`, `open_directory`, and `open_list``. `list_once` is
not in that set, because its outcome type is `ListOutcome` rather than a
`Result`.

### 3.5 [SYS-7] — no change

`IoError` is reused unchanged, and every failure below is one of its existing
thirty classes. The two new `origin` discriminators are target data inside
[QUAL-1] records, not specification text: both opens report the target's
directory-open facility and `list_once` reports its enumeration facility.

### 3.6 [SYS-8] — one-attempt transfers

**(a)** Replace the opening enumeration with

> `read_once`, `write_once`, `list_once`, `host_copy_bytes`, and
> `host_copy_utf8` are one-attempt operations over a caller-owned initialized
> `buffer<u8>` and a caller-written range.

**(b)** In the range-validation paragraph, add `for `list_once` the range is
`offset` and `capacity`;` beside the `read_once` clause. Everything else about
range validation is unchanged and applies as written: overflow of the
mathematical sum, an offset past the runtime length, or a range extending past
it traps as the operation-internal contract check [OP-4, ERR-4] before any
host transfer and before any write of the destination.

**(c)** In the zero-length paragraph, add

> For a zero-length range `list_once` reports `ListBytes(0, 0)` and issues no
> host transfer, and a zero-length enumeration is never reported as
> `ListEnd`.

**(d)** In the at-most-one-attempt paragraph, add `list_once` beside
`read_once` and `write_once`, and add

> `list_once` returns `ListBytes(count, entries)` for the records one attempt
> reported, `ListEnd` exactly when the host reported that the directory holds
> no further entry, and `ListFailed(error)` otherwise. A batch smaller than
> the requested range is not the end of the directory; only `ListEnd` states
> that. A range too small for the target's own next record is reported as a
> recoverable failure in that target's class rather than as a truncated or
> partial entry, and the cursor does not advance, so the same handle with a
> larger range reports the same entries. No entry is ever split across two
> attempts and no record is ever reported without its complete name.

**(e)** In the buffer-disposition paragraph, add

> On `ListBytes(count, entries)` exactly the first `count` bytes of the
> requested range may have changed, every other byte of the buffer is
> unchanged, and the enumeration cursor advances past exactly the entries
> those records name. On `ListEnd` and on `ListFailed` no byte of the buffer
> changes and the cursor does not advance.

and add `on `ListBytes(count, entries)` the count is at most the requested
`capacity`` to the list of retained bounds.

### 3.7 [SYS-10] — `DirectoryRead` gains a second producer

**(a)** Replace

> It is live from its entry binding until its release and has no other
> transition: this specification declares no attenuation, duplicate, split, or
> explicit close operation for it, so no other state is reachable.

with

> It is live from its entry binding or from the `open_directory` that created
> it until its release, and has no other transition: this specification
> declares no attenuation, duplicate, split, or explicit close operation for
> it, so no other state is reachable.

**(b)** In the downward-aliasing paragraph, after the `open_read` sentence,
add

> `open_directory` creates an independent `DirectoryRead` naming the child
> directory object, and `open_list` creates an independent `DirectoryList`
> with its own entry cursor; neither aliases the capability it was opened
> against, and releasing either leaves that capability live.

**(c)** In the concurrency paragraph, replace `Any number of `open_read` calls`
with `Any number of `open_read`, `open_directory`, and `open_list` calls`, and
`Each either creates its own `ReadFile` or fails` with `Each either creates
its own `ReadFile`, `DirectoryRead`, or `DirectoryList`, or fails`.

**(d)** Add one paragraph at the end:

> A capability `open_directory` returns names the object the target's own
> directory-relative resolution reached for that component, with the process
> equivalence and the deferred confinement [PATH-2] already fixes. Two
> `DirectoryRead` values may denote the same directory object however they
> were produced, and a program that descends must exclude the self and parent
> components itself: nothing in this specification detects a cycle.

### 3.8 [SYS-14] — the new family contract

Insert immediately after [SYS-13], in the [SYS-10] .. [SYS-13] house style:

> [SYS-14] `DirectoryList` is a stateful resource with one state.
> `open_list` creates it live, with one entry-cursor domain over the
> directory object the capability it was opened against names. A separate
> `open_list` on the same capability creates a separate cursor and does not
> prove a separate directory object, and this specification declares no
> duplicate, split, rewind, or positioned-lane operation, so multiple lanes
> over one enumeration are not reachable.
>
> `list_once` is call-scoped and leaves both owners live on every outcome; its
> transfer, cursor, and buffer semantics are [SYS-8]. It reports the entries
> the host reported, in the host's own order: this specification fixes no
> enumeration order, promises no stability across two enumerations of the same
> directory, and states no relationship to a concurrent change of that
> directory's content. A program that needs a deterministic order sorts what
> it collected.
>
> The reported entries are exactly what the target's directory holds,
> including the self and parent entries when the target's directory holds
> them. They are not filtered, because filtering them would cost a second host
> call in the batch that held only them [QUAL-3], and a program that descends
> must exclude them anyway to terminate.
>
> One entry record is one kind byte, one name-length byte, and exactly that
> many name bytes. The closed kind set is `0` unknown, `1` regular file, `2`
> directory, `3` symbolic link, and `4` other; `0` states that the target
> classified the entry at enumeration time as nothing more specific, not that
> the entry is absent or unreadable. A name is one path component: it is never
> empty, never longer than the target's component limit, and contains no NUL
> and no target separator, so no record a program reads can name more than one
> component.
>
> An entry name reaches source only as those bytes. This specification
> declares no operation turning an enumerated name into a `HostString` or a
> `RelativePath`, because a name's backing is not the command-lifetime
> argument snapshot [HOST-3] and a path value is an inline lease over that
> snapshot [PATH-1]. `open_directory` therefore takes a caller-owned name
> range rather than a path value, and path composition remains the DEFERRED
> addition [PATH-1] states.
>
> `open_directory` validates that name range before any host call: a range
> that is empty, longer than the target's component limit, or containing a NUL
> or a target separator yields `InvalidPath` with both detail fields zero
> [SYS-7], no host call, and no capability. A valid range that names no
> directory yields the target's own failure class — `NotFound`,
> `NotDirectory`, `PermissionDenied`, and the rest of the closed set — exactly
> as `open_read` does.
>
> `DirectoryList` is release-complete [SYS-5]. Compiler-derived release
> consumes the resource and may discard only a close diagnostic, which carries
> no guarantee about entries already observed. This specification declares no
> separate explicit-close operation, and a deep traversal therefore holds one
> descriptor per live level. Whole-process abort relies on operating-system
> teardown [SYS-5].

### 3.9 [PATH-2] — the component-name form

After the first sentence, add

> A directory-relative operation resolves either one relative path value or
> one caller-supplied single path component [SYS-14]; both are resolved
> through the target's own directory-relative facility and neither is
> concatenated onto a prefix.

Nothing else in [PATH-2] changes. In particular the process-equivalence
promise, the no-emulation qualification rule, and the deferred confined root
apply unchanged to the new operations, so an enumerated symbolic link that
`open_directory` follows may reach an object outside the directory the
capability names — the same promise `open_read` already makes.

### 3.10 [QUAL-2] — the third guarantee

Replace `Two guarantees are stated here` with `Three guarantees are stated
here`, and add after the lossless-code-unit sentence:

> The third is a directory-enumeration facility for the enumeration semantic
> IDs [SYS-14]: one host call that reports a bounded batch of the entries of
> an open directory and advances that directory's own enumeration position. A
> target with no such facility fails qualification for those IDs rather than
> emulating them, and in particular never substitutes a scan built out of
> other operations.

## 4. What this delta deliberately does not add

- **No file open by name.** A traversal that searches file content still needs
  an `open_read` whose name arrives as bytes. It is the obvious next
  operation, it reuses this delta's name validation exactly, and it is left
  out because nothing in this batch exercises it. Adding it later is one
  operation row and one qualification row.
- **No metadata, size, mode, or identity operation.** RECON gap 8's
  cycle-detection route (device and inode identity) stays closed; a traversal
  terminates by excluding the self and parent components and by its own depth
  bound.
- **No no-follow open flag.** A program that must not follow a symbolic link
  uses the `3` kind and does not descend.
- **No rewind, no positioned enumeration, no ordering promise.**
- **No second string type.** [HOST-3]'s owned-backing string remains the
  DEFERRED addition it already is.

## 5. Implementation

Switch: `pub const TRAVERSAL_SURFACE: bool = false` in
`compiler/src/resolution/catalog.rs`. `false` admits exactly the v0.31
inventory; `true` admits the candidate one. `compile` reads the constant;
`compile_with_traversal_surface` selects the inventory explicitly, and
`resolve_with_traversal_surface` carries the selection into the resolved unit
so the checker cannot disagree with the resolver about which inventory the
declaration ordinals came from. Activation flips the constant and deletes both
entries.

| file | change |
|---|---|
| `compiler/src/resolution/catalog.rs` | the switch; the three tables extended in place with active-prefix accessors `system_nominals`/`system_constructors`/`system_operations`; every ordinal-to-index map takes the inventory state; `SystemResourceType::DirectoryList` and its [SYS-5] contract |
| `compiler/src/resolution/engine.rs`, `mod.rs` | `resolve_with_traversal_surface`; the resolved unit records its inventory state |
| `compiler/src/semantic/check*.rs` | the checker reads that state for every ordinal lookup; no other semantic change — a system call is already checked entirely from its catalog row |
| `compiler/src/semantic/provenance.rs` | the three `wf-prov` rows |
| `compiler/src/backend/qualification.rs` | `DirectoryEnumeration` (facility symbol, native record layout, native kind values); the `DirectoryEnumeration` [QUAL-2] guarantee; rows for semantic IDs 11, 12, 13; the `DirectoryList` resource row |
| `compiler/src/backend/emitter/system.rs` | `emit_open_directory`, `emit_open_list`, `emit_list_once`, `list_outcome_shape`, the shared component-name validation |
| `compiler/src/driver.rs` | `compile_with_traversal_surface` |

**Target binding, stated honestly.** Darwin binds `__getdirentries64`, the one
libSystem entry that reports a batch of 64-bit-inode directory records against
an open descriptor and advances it. It was chosen after measuring the
alternatives on the build host: `getdirentries` does not link on a 64-bit-inode
target (`_getdirentries_is_not_available_when_64_bit_inodes_are_in_effect`),
and `opendir` plus `readdir` is two calls with an allocation, which [QUAL-3]
excludes on a transfer path. The record layout in the qualification row —
`d_reclen` at 16, `d_namlen` at 18, `d_type` at 20, `d_name` at 21 — was
measured on the build host rather than transcribed.

The too-small-range behavior of §3.6(d) was measured on the same host rather
than assumed: `__getdirentries64` with a range of 8 or 40 bytes returns `-1`
with `EINVAL` (which the Darwin table maps to `InvalidInput`), and a following
call with a 4096-byte range then reports every entry of the directory — so the
failing attempt advanced nothing. That is the whole basis for the
cursor-does-not-advance sentence; it is a target-observed property the
qualification record requires, not a promise derived from documentation.

**Linux is deliberately unmapped.** `getdents64` has a different arity and a
different record layout, and no evidence in this tree exercises either. A
qualification row is a promise, so the Linux targets supply no enumeration
facility and fail qualification for the two enumeration semantic IDs — a
target-qualification failure, which [QUAL-1] already distinguishes from a
source-language rejection. `open_directory` needs no enumeration facility and
is mapped on both families. Mapping Linux is a small change that should land
with a Linux host that can run these same tests, not before.

**Memory safety of the in-place rewrite.** The emitted `list_once` validates
every native header field against the reported extent before using it: the
record length must be at least the header plus the name length, must not
exceed the bytes remaining in the batch, and must be nonzero, and the name
length must not exceed the component limit. A record that fails any of those
ends the walk and the operation reports the records it already normalized.
The reported extent is itself clamped to the caller's validated capacity, so a
host that over-reports cannot make the walk read past the range. The
destination cursor is provably behind the source cursor at every step, because
a portable record is two bytes of header against the native record's larger
one, so the forward copy never overwrites an unread byte.

## 6. Evidence

Program: `tests/programs/dir_walk.wf` — a recursive walk of the invocation
directory that collects every entry's kind and relative path into the growable
byte-string layer, sorts the collected records, and publishes them. It
descends with `open_directory`, enumerates with `open_list` and `list_once`,
excludes the self and parent components, bounds its depth, and forms no path
value anywhere.

Harness: `compiler/tests/programs/traversal.rs`, nine cases, all passing under
`cargo test --profile gate --test programs`:

| case | direction |
|---|---|
| `the_traversal_program_walks_a_real_tree_and_publishes_it_sorted` | compiles, links, runs against a real three-level fixture tree |
| `an_empty_tree_publishes_nothing_after_the_self_and_parent_entries_are_skipped` | the self and parent entries reach source and the program skips them |
| `an_unreadable_subdirectory_is_recorded_without_descending_into_it` | a mode-000 subdirectory is enumerated as an entry and its failed descent is recoverable, not a trap |
| `the_candidate_inventory_leaves_every_v031_program_byte_identical` | `wfgrep.wf`, `byte_string.wf`, and `growable_vec.wf` emit identical modules under both inventories |
| `the_traversal_source_is_undeclared_without_the_candidate_inventory` | with the switch off every traversal spelling is an undeclared name |
| `an_enumeration_handle_is_not_usable_after_it_is_moved` | the handle is affine like every other system resource |
| `program_bytes_still_cannot_become_a_path_value` | `relative_path` over a `buffer<u8>` is still rejected with the surface admitted |
| `an_enumeration_match_that_omits_an_outcome_is_rejected` | portable control flow over `ListOutcome` is exhaustive |
| `the_component_validation_precedes_every_host_call` | the emitted rejection path builds the portable class and returns without reaching the directory-relative open |

Actual run output of the traversal program against the fixture tree
(`a.txt`, `z.txt`, `sub/b.txt`, `sub/deeper/c.txt`), exit status 0:

```
1 a.txt
2 sub
1 sub/b.txt
2 sub/deeper
1 sub/deeper/c.txt
1 z.txt
```

Switch-off evidence: the whole existing suite is unchanged and green — 916
library tests, 43 program tests, the canonical corpus, the conformance
structure target, `cargo fmt --check`, `cargo clippy -D warnings`,
`cargo doc -D warnings`, and `whitefoot-spec`.

Two properties this program establishes that a smaller probe would not:

- **The bounds are proved, not claimed.** Every index derived from an
  enumeration record is external data under [PRV-1], so [PRV-3] refuses a
  claim about it; the walk discharges each bound with a real value branch
  instead. The first version of the program used claims and was rejected —
  which is the check working.
- **The traversal is real.** No fixture list, argument vector, or harness
  injection reaches the program: it opens, enumerates, and descends through
  the host itself, and the sorted listing is derived entirely from what the
  enumeration reported.

## 7. PROPOSED conformance cases (no `tests/conformance/` edit here)

Proposed only. Adding any of these is a protected-evidence change requiring
the owner's exact before/after audit and an approval-ledger entry, and none is
written by this batch.

| proposed id | rule | direction | what it pins |
|---|---|---|---|
| `sys14-list-outcome-exhaustive` | SYS-6 | positive | a `match` over `ListOutcome` with its three constructors is exhaustive, and omitting one is an [ERR-2] rejection |
| `sys14-list-range-trap` | SYS-8 | positive (traps) | `list_once` with `offset + capacity` past the buffer length traps before any host call, leaving the buffer unchanged |
| `sys14-list-zero-range` | SYS-8 | positive | a zero-length range yields `ListBytes(0, 0)`, never `ListEnd` |
| `sys14-open-directory-component` | SYS-14 | negative | a name range containing a separator or a NUL yields `InvalidPath` with zero detail and makes no host call |
| `sys14-open-directory-empty-name` | SYS-14 | negative | an empty name range yields `InvalidPath` |
| `sys14-list-handle-affine` | OWN-1 | negative | a moved `DirectoryList` is not usable |
| `sys14-list-handle-unique` | SYS-4 | negative | two live `&uniq` borrows of one `DirectoryList` are rejected |
| `sys14-no-path-from-bytes` | PATH-1 | negative | `relative_path` over a `buffer<u8>` is still a type rejection |
| `sys14-directory-release` | SYS-5 | positive | a `DirectoryList` that leaves scope runs exactly one derived close and the emitted program contains no second one |
| `sys14-entry-kind-closed` | SYS-14 | positive | every reported kind byte is one of the five closed values |

No existing conformance case pins the [SYS-2] counted totals — the corpus's
`sys*` cases are behavioral, and a grep of `tests/conformance/manifest.jsonl`
finds no declaration count — so §3.1(f) and §3.1(g) require no corpus edit.
Those totals are pinned instead by the compiler's own extraction lock
(`system_inventory_matches_independent_extraction_from_exact`), which reads
the specification text directly and is bound to the *active* inventory, so it
keeps checking v0.31's numbers until activation and then checks v0.32's.
