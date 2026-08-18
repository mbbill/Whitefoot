# wfgrep recursive traversal — system-surface reconnaissance

Status: RECONNAISSANCE. This is design evidence for batch 0071 W1
(outline:CAND-8). It records what the landed wfgrep program and the v0.31
system declaration domain actually provide, and what a real recursive
directory traversal would additionally require. It proposes no bytes, adds no
declaration, and carries no authority: every gap named here is a
specification surface the lead alone may fold into a candidate.

Base: `main` at `3a9204bf`, active specification v0.31 at `spec/kernel-spec.md`
SHA-256 `ea4b8ad4a56fbf43f3c98b91fc667da0b693c75b81807250a36454e03a197f1c`.

## 1. The wfgrep program as it exists today

Source: `tests/programs/wfgrep.wf`, 607 lines, SHA-256
`fb2f3b44160a947d7adca9fc9b5af851b446a7bcfc179ede4f8c689b21033904`.
Harness: `compiler/tests/programs/wfgrep.rs` (ten scenario tests over one
shared build), with fixture and invocation support in
`compiler/tests/programs/support.rs`.

### What it receives

The entry is

```
command fn main(command.args as args: own Args, command.cwd as cwd: own DirectoryRead, command.stdout as out: own Output, command.stderr as err: own Output) -> own ExitStatus allocates(heap), external, blocks, traps
```

so the program's complete input surface is the four [FN-7] standard inputs.
Everything it searches arrives by exactly one route:

1. `args_count` gives the argument count; `arg_get(position:)` leases argument
   *n* as a `HostString`.
2. Argument 1 is the pattern. Its bytes are transferred into a program-owned
   `buffer<u8>` of 4096 by `host_copy_bytes` — the lossless route, so a
   non-UTF-8 pattern survives (`a_path_that_is_not_text_travels_the_lossless_route_unchanged`).
3. Arguments 2.. are file names. Each is consumed by `relative_path` into a
   `RelativePath`, then opened with `open_read(root: cwd, path:)`.
4. File content is pulled with `read_once(file:, destination: input, offset:,
   capacity:)` into the same 4096-byte program buffer, one attempt at a time,
   with a manual carry/shift for lines that straddle a read boundary.

There is no other input. Nothing is injected by the harness into the program's
address space: the Rust harness only writes fixture files into a temp
directory (`FixtureDirectory`), then runs the built executable with
`current_dir(fixture)` and the file names as `argv[2..]`
(`CompiledProgram::run`). The program does the real `openat`/`read`.

### What it emits

`write_once(output:, source:, offset:, count:)` against `command.stdout` for
matched lines (batched through a 4096-byte buffer) and against
`command.stderr` for `wfgrep: NAME: reason` diagnostics. The process result is
`exit_status(code:)`: 0 matched, 1 no match, 2 error.

### The honest characterization

wfgrep today is a **multi-file literal search over an externally supplied file
list**, resolved relative to the invocation directory. It is not a recursive
searcher and contains no traversal code. This is not a new finding: the frozen
`research/experiments/wfgrep-baseline/PROTOCOL.md` already classifies its
`many` case as "512 files, 1 line each … many small files, no traversal" over
an "explicit argv list", and states that "the `many` list is explicit because
the frozen slice has no directory" enumeration.

## 2. The complete v0.31 system surface

Normative: [SYS-1] .. [SYS-13], [HOST-1] .. [HOST-3], [PATH-1], [PATH-2],
[QUAL-1] .. [QUAL-3], [FN-7]'s standard-input table. The per-type family
contracts are [SYS-10] `DirectoryRead`, [SYS-11] `ReadFile`, [SYS-12]
`Output`, [SYS-13] `ExitStatus`.
Implementation: `compiler/src/resolution/catalog.rs`
(`SYSTEM_NOMINALS`, `SYSTEM_CONSTRUCTORS`, `SYSTEM_OPERATIONS`,
`system_declarations`), `compiler/src/backend/qualification.rs`
(semantic-ID to symbol map), `compiler/src/backend/emitter/system.rs`
(the emitted shims).

Counts, which any candidate must recompute: 14 nominal types, 39 enum-variant
constructors, 64 variant fields, **11 operations**, 14 operation region
parameters, 25 operation value parameters — 167 declaration records.

Nominal types: `Args`, `HostString`, `RelativePath`, `DirectoryRead`,
`ReadFile`, `Output`, `ExitStatus` (opaque); `ArgError`, `Utf8Error`,
`CopyError`, `Utf8CopyError`, `PathError`, `ReadOutcome`, `IoError` (enums).

Operations, verbatim from [SYS-2]:

```
fn args_count['a](args: &'a Args) -> own u64 reads('a);
fn arg_get['a](args: &'a Args, position: own u64) -> own Result<HostString, ArgError> reads('a);
fn host_bytes_len['v](value: &'v HostString) -> own u64 reads('v);
fn host_copy_bytes['v, 'd](value: &'v HostString, destination: &uniq 'd buffer<u8>, offset: own u64, capacity: own u64) -> own Result<u64, CopyError> reads('v 'd), writes('d), traps;
fn host_utf8_len['v](value: &'v HostString) -> own Result<u64, Utf8Error> reads('v);
fn host_copy_utf8['v, 'd](value: &'v HostString, destination: &uniq 'd buffer<u8>, offset: own u64, capacity: own u64) -> own Result<u64, Utf8CopyError> reads('v 'd), writes('d), traps;
fn relative_path(value: own HostString) -> own Result<RelativePath, PathError> pure;
fn open_read['c, 'p](root: &'c DirectoryRead, path: &'p RelativePath) -> own Result<ReadFile, IoError> reads('c 'p), external, blocks;
fn read_once['f, 'd](file: &uniq 'f ReadFile, destination: &uniq 'd buffer<u8>, offset: own u64, capacity: own u64) -> own ReadOutcome reads('f 'd), writes('f 'd), external, blocks, traps;
fn write_once['o, 's](output: &uniq 'o Output, source: &'s buffer<u8>, offset: own u64, count: own u64) -> own Result<u64, IoError> reads('o 's), writes('o), external, blocks, traps;
fn exit_status(code: own u8) -> own ExitStatus pure;
```

Three facts follow directly and are the whole story:

- **No operation returns `DirectoryRead`.** The only value of that type a
  program can ever hold is the one bound by `command.cwd` ([FN-7] table
  ordinal 1; [SYS-2]: opaque values "are produced only by the operations in
  this rule and by the command entry's standard input bindings").
- **No operation enumerates anything.** There is no directory stream, no entry
  type, no metadata or `stat`, and no file-type predicate.
- **`RelativePath` can only be built from an argument.** `relative_path`
  consumes a `HostString`, and [HOST-3] fixes a host string's backing as *the
  command-lifetime argument snapshot*; [PATH-1] retypes that same inline lease
  with no copy. There is no route from program bytes to a path.

## 3. Gap table — what real recursive traversal needs

"Present" means a v0.31 program can do it today. "Exact spec surface" names
what the addition would touch; nothing here is proposed as bytes.

| # | Needed capability | Present? | Exact specification surface it would require |
|---|---|---|---|
| 1 | Obtain a directory capability for a subdirectory | **Absent.** Only `command.cwd` yields `DirectoryRead`; no operation produces one. [SYS-10]: it "is live from its entry binding until its release and has no other transition", and "Opening creates aliases only downward. `open_read` creates an independent `ReadFile` … and does not alias the capability." | One new [SYS-2] operation row returning `Result<DirectoryRead, IoError>`; [PRV-1] provenance row; [QUAL-1] semantic ID + `qualification.rs` symbol. No new nominal — [SYS-4] and [SYS-5] already carry `DirectoryRead` (shared capability; "at most one native close attempt", `external, blocks`), and [SYS-10] already admits that "Two `DirectoryRead` values may denote the same directory object". But **[SYS-10]'s entry-binding-only liveness sentence and its downward-aliasing paragraph must both state the new producer.** |
| 2 | Enumerate the entries of a directory | **Absent.** No stream, cursor, or entry operation exists. | One new opaque nominal (a stateful directory stream) → [SYS-2] nominal table, [SYS-4] kind/Sendable/Shareable row (stateful resource, not Shareable — it owns a cursor), [SYS-5] release row, and one new per-type family contract in the [SYS-10] .. [SYS-13] series; an open-stream operation and a next-entry operation → two [SYS-2] rows, [PRV-1] rows, two [QUAL-1] semantic IDs. |
| 3 | Represent one enumeration step's three outcomes (entry / end / failure) | **Absent.** `ReadOutcome` is `read_once`'s own type; [SYS-6] states "the one operation with more than two outcomes", singular. | One new enum with three operation-prefixed variants → [SYS-2] enum table (+3 constructors, +N fields); **[SYS-6]'s singular sentence must change**; [SYS-7] is untouched because the failure payload is still `IoError`. |
| 4 | Read an entry's name as bytes the program owns | **Absent, and the obvious reuse is blocked.** [HOST-3]: "A producer whose backing is not command-lifetime yields no value of this type: it introduces a distinct owned-backing string resource with its own release action and its own family contract." A directory entry name is not command-lifetime backed, so it may **not** be delivered as a `HostString`. | Two disjoint options. (a) Deliver the name by an [SYS-8]-style one-attempt transfer into a caller-owned `buffer<u8>` — adds no type, extends [SYS-8]'s enumeration of transfer operations, and keeps [HOST-3] untouched. (b) Introduce a second string type with owned backing — new nominal, new [SYS-4] row, new [SYS-5] release row and release effect, new conversion contract, plus the [HOST-3] "explicit later operation with its own delta [META-5]" it already anticipates. |
| 5 | Turn an entry name into something openable | **Absent.** `relative_path` takes only a `HostString`; a path is an inline lease over command-lifetime backing ([HOST-3], [PATH-1]). Constructing one over a program buffer would break that representation invariant and the [QUAL-2] backing guarantee. | Either (a) the directory-relative operations of gaps 1–2 accept `(&'c DirectoryRead, &'b buffer<u8>, offset, count)` directly and **no path value is ever formed** — the smallest honest shape, and it leaves [PATH-1]'s deferral intact; or (b) [PATH-1] and [HOST-3] are amended to admit a second path representation whose backing is a borrowed program buffer, which drags in region-bearing path types under [STOR-5] and reopens [QUAL-2]. |
| 6 | Compose a child path from a parent path and a name | **Absent, explicitly deferred.** [PATH-1]: "A path component type, an absolute path type, and every operation that decomposes, enumerates, joins, or displays a path are DEFERRED additions with their own deltas [META-5]." | Not required if traversal descends by opening a child capability per level (gap 1), because every name stays a single component. Otherwise it is a full path-algebra delta. |
| 7 | Distinguish a directory from a regular file | **Indirect only.** The `IsDirectory` class is reachable and wfgrep already maps it to status 3, but not where one would guess: `open_read` emits `openat` with `file_open_flags: 0` (`O_RDONLY`, no `O_DIRECTORY`), and on Unix-family targets opening a directory read-only *succeeds* — the `EISDIR` arrives at the following `read(2)`, so source sees `ReadFailed(IsDirectory)` from `read_once`. Either way it costs a real open per entry, sees no symlink, and names no other file type. No harness case currently covers it. | Either accept the probe-by-open-then-read cost (no spec change), or carry a file-type discriminant on the entry outcome of gap 3 (one `u8` field on the entry variant, plus the normative statement of its closed value set). |
| 8 | Symbolic-link and cycle policy | **Absent, and the existing promise is deliberately weak.** [PATH-2] fixes process-equivalent resolution: `.`, `..`, symlinks, and mount transitions resolve exactly as the surrounding namespace does, "so a resolved object may lie outside the directory that capability names", and confinement is a deferred distinct type. The harness test `open_read_follows_a_symbolic_link_out_of_the_directory_it_names` pins that today. | A traversal that follows links needs either a no-follow open flag (a distinct semantic ID with its own [QUAL-1] record) or an identity value to detect revisits (device/inode — a new metadata surface). A traversal that does **not** follow links needs only the file-type discriminant of gap 7. |
| 9 | Deterministic output order | Present as ordinary source work. The OS enumeration order is unspecified, so the program must collect and sort names itself. | None. This is exactly what the byte-string/collection layer of this batch is for. |
| 10 | Bounded recursion state | Present as ordinary source work: a hand-rolled explicit stack over the collection layer, or recursion (`recursive_tree.wf` shows derived cleanup for recursive nominals). | None. But note each open directory level holds a `DirectoryRead` and a stream; release is compiler-derived [SYS-5] and a deep tree holds one descriptor per live level. |
| 11 | Reading file bytes, writing results, exit status, error classes | **Present and sufficient.** `open_read`, `read_once`, `write_once`, `exit_status`, the 30 `IoError` classes. | None. |

### Verdict

**Real recursive directory traversal is not expressible in v0.31.** The
blocking gaps are 1, 2, 3, and 4/5 together; each is a specification addition,
not a compiler defect and not a source-shaping problem. No arrangement of
existing operations enumerates a directory or produces a second directory
capability.

What *is* possible today is **harness-simulated traversal**: an outside agent
(the Rust test harness, a shell, or `find`) walks the tree and passes the file
list on `argv`, and wfgrep searches exactly that list. That is what the ten
existing wfgrep tests and the `many` baseline case already do. It is a
legitimate way to exercise the search core at scale, and it is *not* traversal —
the program contributes no directory logic at all, and calling it a
"recursive search" would be a false claim in a record or an owner packet.

## 4. Traversal slice, grounded in what exists

If the lead folds a traversal delta into a candidate, the smallest shape that
touches the fewest existing invariants is:

- **Keep `DirectoryRead` as the only capability type.** Add one operation that
  opens a child directory relative to a held capability. It reuses the
  existing [SYS-4] kind row and the existing [SYS-5] release row, and every
  piece of its lowering already exists in tree: the command bootstrap opens
  the working directory with `open(".", O_RDONLY|O_DIRECTORY)` using
  `SystemTarget::directory_open_flags`, and `emit_open_read` already emits
  `openat` + one branch + the shared cold errno mapper. A child open is
  `openat(dirfd, name, directory_open_flags)` — one host call, satisfying
  [QUAL-3]'s "at most one direct host call", with the `DirectoryRead`
  representation (a native descriptor) unchanged.
- **Add exactly one new opaque nominal** for the enumeration cursor, because a
  cursor is state and [SYS-4] requires a stateful resource for anything a later
  call advances. Its release is a native close attempt and its family contract
  mirrors [SYS-11] `ReadFile` almost sentence for sentence: created live by one
  operation, one cursor domain, call-scoped advance leaving both owners live,
  release-complete, no duplicate/split/positioned-lane operation.
- **Deliver entry names by transfer, never as a string or path value.** The
  next-entry operation writes the name bytes into a caller-owned `buffer<u8>`
  under the [SYS-8] one-attempt contract, exactly like `read_once`, and returns
  `EntryBytes(count:, kind:)` / `EntryEnd()` / `EntryFailed(error:)`. This is
  the decisive choice: it keeps [HOST-3]'s command-lifetime backing invariant,
  [PATH-1]'s deferral of path algebra, and [QUAL-2]'s backing guarantee all
  untouched, and it costs one enum instead of a string family.
- **Take the child name as bytes at the open sites too**, so no `RelativePath`
  is ever constructed from program memory. The cost is that the two new
  directory-relative operations take `(root, buffer, offset, count)` rather
  than a path; the benefit is that gaps 5 and 6 disappear entirely.
- **Do not follow symbolic links in v1.** Use the `kind` discriminant to
  descend only into directories, and leave [PATH-2]'s confinement deferral and
  the existing link-following behavior of `open_read` exactly as they are.

That is one new nominal, one new enum with three constructors, and three new
operations. Everything above the system boundary — the worklist, the sort, the
name buffers, the per-level state — is ordinary source over the collection and
byte-string layer this batch is building.

### Lowering note

`open_read` today emits an always-inlined `openat` + one branch + a cold errno
mapper (`compiler/src/backend/emitter/system.rs::emit_open_read`). A directory
stream is the one place where [QUAL-3]'s "at most one direct host call" needs a
deliberate choice: the raw enumeration syscall (`getdents64` on Linux) is one
call and qualifies, while the libc `opendir`/`readdir` pair allocates and is
not a single call. Any candidate should state which it binds, per target,
inside the [QUAL-1] record.

## 5. What the byte-string program already supplies

`tests/programs/byte_string.wf`, landed with this report, is the source-side
half of the traversal slice and is independent of every gap above. It provides
construction from literal arrays, append and concat by buffer growth, length,
a bounds-safe `Option<u8>` accessor, a naive substring search, and decimal
formatting, over the v0.31 `buffer<u8>` and the [SET-2] affine replace. Its
search path (`bs_len`, `bs_byte`, `bs_find`) is marked `deny_claims`, so the
whole read side of a searcher is proved rather than asserted: the accessor
discharges its bounds obligation by a real value branch, and
`compiler/tests/programs/heap.rs` pins both directions — deleting the branch
is an [OP-4] rejection, and injecting one provable claim into the strict
search is a [CLM-3] rejection.

A traversal built on the gaps above needs exactly these pieces for its
worklist, its entry-name buffers, its deterministic sort, and its path
assembly. None of that work is blocked; only the system boundary is.

## 6. What this report is not

It is not a specification proposal, not a plan item, and not authority to add
a declaration. Gaps 1–8 are exactly the surfaces a v0.32 (or later) candidate
would have to state, count, verify, and take to the owner as exact bytes.
