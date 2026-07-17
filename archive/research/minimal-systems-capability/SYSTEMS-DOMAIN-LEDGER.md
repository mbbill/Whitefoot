# Systems-Domain Ledger

Status: full-envelope research accounting, 2026-07-14. This artifact classifies
the stable public Rust 1.97.0 `core`, `alloc`, and `std` surface by
caller-observable systems domain. It selects no xlang mechanism, changes no
language rule, closes no capability family, and authorizes no implementation.

## 1. Purpose and claim discipline

The completed G0-Core synthesis concerns the **sequential, unique-owner
data-structure floor**. That floor is necessary for a general-purpose systems
language, but it is not the whole systems-language envelope. This ledger keeps
the two scopes separate:

- it assigns the current detailed floor and its protected prerequisites to
  `G0`;
- it assigns the remainder of the Rust stable systems surface to ordinary
  library work, a named trusted frame, a later family, a redundant surface, or
  an owner-ratified non-goal; and
- it records the claims that remain blocked even if every current data-
  structure family later passes.

The classification is an accounting destination, not a finding that xlang can
already derive the contract. In particular, `LIB` does not mean "derivable
today," `FRAME` does not authorize privilege, and `G0` does not mean "closed."
Every derivability, soundness, performance, and adoption claim still needs the
gate named by the research charter.

Rust is a finite external anchor, not a design oracle. Rust API names, traits,
representations, destructors, unsafe implementation techniques, and source
spellings carry no presumption for xlang. This ledger records needs and
boundaries only. It deliberately does not name a candidate xlang syntax, type,
operator, storage primitive, compiler intrinsic, or privileged container.

## 2. Pinned source universe

The anchor is Rust 1.97.0, released 2026-07-09:

- release tag: `1.97.0`;
- annotated tag object: `eca4cdea45792600b4275e9d4c64fd827d575a24`;
- peeled source commit: `2d8144b7880597b6e6d3dfd63a9a9efae3f533d3`;
- reference crates: stable public `core`, `alloc`, and `std` from that commit.

The checked-in mechanical inputs are:

- [the item inventory](RUST-1.97.0-API-INVENTORY.tsv), containing 17,135
  rendered rows, including 10,267 stable-safe and 560 stable-unsafe rows;
- [the module accounting](RUST-1.97.0-MODULE-ACCOUNTING.tsv), containing all
  297 reachable public modules, including collapsed target catalogs;
- [the extraction manifest](RUST-1.97.0-CENSUS-MANIFEST.json), pinning tool,
  source, policy, counts, and hashes; and
- [the census notes](RUST-CENSUS-NOTES.md), defining inclusion,
  canonicalization, and deduplication.

The primary upstream sources are the exact-version
[`core`](https://doc.rust-lang.org/1.97.0/core/index.html),
[`alloc`](https://doc.rust-lang.org/1.97.0/alloc/index.html), and
[`std`](https://doc.rust-lang.org/1.97.0/std/index.html) documentation, plus the
pinned [`library/` source tree](https://github.com/rust-lang/rust/tree/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library).

### 2.1 Unit of accounting

The mechanical inventory preserves Rust renderings. The semantic unit in this
ledger is a normalized caller-observable contract cluster:

```text
pre-state + input ownership + behavior parameters
    -> post-state + result ownership + failure/destruction effects
```

Order, complexity, allocation, initialized and moved bytes, contiguity,
identity or address stability, invalidation, concurrency, platform dependence,
and fact-channel consequences are part of the contract. Two Rust spellings are
not merged when any of those observations differ.

Every canonical stable item receives one conservative declaration disposition
within the semantic partition of its domain row. A row with more than one code
contains an explicit partition; the codes are not alternatives for an
unclassified item. Reexports, aliases, prelude entries, and duplicated
primitive renderings route to the canonical domain before this classification
is applied.

The mechanical declaration classification records exactly one `domain_id`, one
`surface_evidence_status`, and one independent `need_route_kind` plus stable
`need_route_id` per canonical source declaration: 5,278 safe and 277 unsafe.
Every row retains Rust `caller_safety` without treating Rust-safe as approved
xlang surface. The surface axis distinguishes safe contract anchors, safe
boundary evidence, unsafe boundary evidence, and Rust-only namespace/source
surface. The need axis independently routes the underlying caller requirement
to `G0_CONTRACT`, `LIB_CONTRACT`, `LATER_FAMILY`, `FRAME`, `REDUNDANT`,
`NO_INDEPENDENT_NEED`, or a genuinely owner-authorized `NG`.

Boundary evidence never becomes `NG` merely because its Rust spelling uses a
raw pointer, manual lifetime, leak, uninitialized storage, or an unsafe call.
Such a row must name a safe displacement for the in-scope need, and any frame
on which that displacement depends. Only a need that conflicts with an owner-
ratified first-principles rule may use `NG`, with that authority recorded. The
545 detailed stable-safe seed declarations also carry an exact
`canonical_contract_id`; later-family declarations still require full contract
normalization before their domain can claim closure.

Thus Rust boundary and unsafe implementation requirements cannot disappear,
but they also cannot be smuggled into xlang as an unchecked escape or mislabeled
as evidence that the underlying systems need is absent.

## 3. Need-route codes

Each canonical declaration receives exactly one need route. Domain prose below
uses the shorter `G0`, `LIB`, `LATER`, and `RED` labels for readability; the
mechanical table uses the exact route names in this table.

| Route | Meaning | What the route does not claim |
|---|---|---|
| `G0_CONTRACT` | The contract is in the bounded G0-Core accounting: the sequential, unique-owner data-structure floor or a prerequisite that must be protected while evaluating it. | It does not close a family, select a mechanism, authorize implementation, or prove derivability. |
| `LIB_CONTRACT` | The contract belongs in an ordinary checked library after its dependencies close. | It does not assert current derivability, require an xlang standard library, or copy Rust's API shape. |
| `FRAME` | The declaration is itself a named compiler, runtime, allocator, ABI, OS, clock, scheduler, target, or MMIO boundary service. | It does not authorize a trusted path or allow writer-authored trust. A later contract that depends on a frame remains `LATER_FAMILY` with `required_frame_ids`. |
| `LATER_FAMILY` | The contract blocks a broader claim and requires its own registry, exact family lock, evidence, and owner adoption decision. | It may not inherit closure from G0 or from an adjacent family. |
| `REDUNDANT` | The declaration has a canonical contract elsewhere and no distinct observable ownership, failure, invalidation, cost, identity, address, or platform guarantee. | It requires an exact canonical replacement; name similarity is insufficient. |
| `NO_INDEPENDENT_NEED` | A Rust namespace, helper surface, or source-language artifact contributes evidence but no independent caller contract after its declarations are routed. | It does not erase any underlying declaration or implementation pressure. |
| `NG` | The underlying capability or behavior is an explicit owner-ratified non-goal with a recorded first-principles reason. | It is not a convenient deferral or a synonym for raw/unsafe surface. If a related caller need remains, its safe displacement is still recorded. |

Mixed domain descriptions remain semantic partitions, not multi-valued routes
for one declaration. Surface admissibility is the independent evidence axis;
for example, raw-pointer volatile access is unsafe boundary evidence, while its
underlying checked MMIO need is `LATER_FAMILY` and requires `F-MMIO`.

### 3.1 Frame accounting labels

`FRAME` rows use the following boundary labels. They name trust-accounting
subjects only; they do not declare that a frame exists, approve its ABI, or
select its implementation.

| Label | Boundary to be priced before use |
|---|---|
| `F-MEM` | Compiler/runtime target layout, relocation, copying, zeroing, initialization truth, and destruction glue. |
| `F-ALLOC` | Heap allocation-owner identity; checked owner transfer that does not relocate the owned allocation; growth, shrinkage, deallocation, alignment, actual acquired usable bytes and allocator rounding/slack, exhaustion, and allocation-failure mapping. |
| `F-TRAP` | Aborting trap entry, diagnostic payload attenuation, process termination, and optional stack capture. |
| `F-BUILD` | Immutable source/target configuration facts, package/build inputs, inclusion, linking, and compile-time environment. |
| `F-IO` | OS byte I/O, standard streams, partial progress, interruption, blocking, and descriptor/handle transfer. |
| `F-FS` | Filesystem calls, filesystem resource ownership, metadata, links, permissions, races, and platform path conversion. |
| `F-PROC` | Runtime environment access, process creation/control, pipes, waiting, termination, and exit status. |
| `F-ABI` | Narrow foreign-call ABI, OS handles, foreign buffers, platform strings, unwind-abort, fact attenuation, and reentry. |
| `F-NET` | Socket/resolver calls, network resource ownership, blocking, partial I/O, shutdown, and platform error mapping. |
| `F-CLOCK` | Monotonic/wall-clock reads, sleep/timer service, resolution, suspension behavior, and clock error mapping. |
| `F-THREAD` | Thread creation/join/parking, scoped entry, thread-local runtime state, and foreign-thread boundaries. |
| `F-SYNC` | Atomic machine operations, blocking/wakeup, scheduler interaction, and concurrent runtime bookkeeping. |
| `F-ASYNC` | Wakeup delivery, executor/scheduler interaction, async I/O/timers, cancellation boundary, and progress reporting. |
| `F-TARGET` | Target-feature truth, instruction selection, vector/architecture operations, and optimizer/compiler hint semantics. |
| `F-MMIO` | Device-memory authority, provenance, mapping lifetime, and invalidation; permitted widths, alignment, event identity, and atomicity/tearing; exact source-path access-event multiplicity; prohibition of speculative, duplicated, fused, split, widened, narrowed, or elided accesses unless the caller contract expressly permits that transformation; ordering against both relevant ordinary-memory and device accesses; external device mutation and attenuation of cached facts for device state and ordinary-memory regions reachable by external agents, including DMA; platform side effects; fault/trap behavior; and target/OS mapping. |

## 4. Exact scope boundary

### 4.1 Detailed G0 floor

`G0` contains registry-level accounting for the following contracts and no
broader systems claim:

- the scalar, checked-arithmetic, allocation, ownership, destruction, and
  recoverable-failure prerequisites exercised by the floor;
- fixed arrays and slices, unique heap ownership, dense owning sequences,
  deques, unordered and ordered association, heaps, append-only indexed pools,
  recyclable stable pools, and the registered graph, list, arena, and
  inline-small-sequence witnesses;
- byte sequences and incremental UTF-8 validation or sealing, but not complete
  Unicode text semantics;
- borrowed, unique, and owning synchronous iteration; and
- exact normal-exit ownership, destruction, invalidation, allocation failure,
  metadata-to-payload fact channels, and protected-baseline cost accounting for
  those contracts.

This is a floor of observable contracts, not a promise that one representation
or one language operation implements all of them. The current G0-Core research
can make later family locks eligible; it cannot close those families itself.

### 4.2 Whole systems envelope

The whole envelope additionally contains custom allocation, shared ownership,
interior mutation, pinning and address-sensitive values, reflection and build
integration, formatting, I/O, filesystems and paths, environment and process
control, OS resources and FFI, networking, time, threads and thread-local
state, atomics and synchronization, async execution and cancellation, target
facilities, volatile device-memory/MMIO access, full numeric and text behavior,
and compiler/runtime frames.

Those domains remain separately blocked below. Passing the detailed floor must
never be reported as passing the whole envelope.

## 5. Stable systems-domain ledger

### D01. Primitive values, arithmetic, and aggregate views

- **Rust families:** primitive integer, floating-point, Boolean, character,
  tuple, array, slice, and range renderings; `core`/`std::{array, num, f32,
  f64}`; `core`/`std::{i8, i16, i32, i64, i128, isize, u8, u16, u32, u64,
  u128, usize}`; numeric constants; applicable `primitive` pages.
- **Disposition:** `G0` for scalar values, checked integer behavior, bounds,
  lengths, indices, and aggregate views used by the detailed floor. `LIB` for
  remaining safe conversion and arithmetic conveniences once their semantics
  are available. `LATER` for complete floating-point, mathematical, constant-
  evaluation, and target-sensitive numeric contracts. Deprecated numeric
  module aliases and repeated primitive renderings are `RED`.
- **Blocked claim:** G0 cannot claim a complete numerics library, bit-exact
  cross-target floating behavior, a full math library, or arbitrary compile-
  time evaluation.

### D02. Core language and callable surface

- **Rust families:** root primitive and keyword pages; function pointers,
  closures, tuples, the never and unit values, control-flow/operator source
  forms, and callable portions of `core`/`std::ops`.
- **Disposition:** the already required value, call, control-flow, and static
  behavior subset is `G0`. Additional callable abstraction, dynamic dispatch,
  separate-compilation interaction, and source-organization contracts are
  `LATER`. Rust keyword pages and Rust-specific source spellings are `RED`.
- **Blocked claim:** a data-structure result says nothing about Rust-equivalent
  traits, object dispatch, closures, modules, crates, linking, or separate
  compilation.

### D03. Generic behavior, borrowing, conversion, comparison, and hashing

- **Rust families:** `core`/`std::{borrow, clone, cmp, convert, default, hash,
  marker, ops}` and `alloc::borrow`; the corresponding derives and primitive
  implementations.
- **Disposition:** `G0` for the exact equality, hashing, ordering, cloning,
  borrowing, and synchronous-iteration behavior parameters required by a
  registered floor contract. Remaining safe adapters and conveniences are
  `LIB`. Thread-transfer/shareability markers and pin-related markers are owned
  by D22 and D23 and are `LATER`; repeated trait impl renderings and derives are
  `RED`.
- **Blocked claim:** G0 does not provide an open-ended Rust trait system or
  certify arbitrary user behavior. Comparator/hash consistency and callback
  failure remain explicit family obligations.

### D04. Layout, raw memory, pointers, and provenance

- **Rust families:** `core::{alloc, mem, ptr}`, `std::{alloc, mem, ptr}`,
  `alloc::alloc`; raw-pointer and function-pointer primitive pages; layout,
  address, uninitialized-storage, manual-destruction, swap, replace, copy, and
  provenance-related declarations.
- **Disposition:** checked ownership, relocation, replacement, swap, layout,
  initialization, and destruction observations required by the floor are
  `G0`. Physical allocation, copying, zeroing, and target layout truth belong
  to `FRAME` `F-MEM`. User-selected/custom allocation and address/provenance contracts are
  `LATER`. The underlying volatile device-memory need routes to D24 `LATER`
  plus reviewed frame `F-MMIO`; Rust's raw-pointer volatile spelling remains
  inadmissible. Writer-visible raw dereference, unchecked layout, raw
  uninitialized payload sealing, manual lifetime forgery, and unchecked memory
  operations are inadmissible boundary evidence; their underlying checked needs
  retain the routes above.
- **Blocked claim:** G0 may claim only the reviewed checked transitions and
  facts needed by its registered contracts. It cannot claim raw-memory
  programming, custom allocators, general provenance control, stable addresses,
  or volatile device-memory access.

### D05. Optional values, recoverable errors, and error description

- **Rust families:** `core`/`std::{option, result, error}` and the prelude
  reexports of their canonical types.
- **Disposition:** the `Option`/`Result`-class ownership and recoverable-failure
  subset exercised by the floor is `G0`. General error composition,
  description, conversion, source chaining, and application-specific errors
  are `LIB`. Reexports and convenience aliases are `RED`.
- **Blocked claim:** G0 does not introduce exceptions or unwinding. Allocation,
  callback, I/O, OS, and cancellation failures remain owned by their domains.

### D06. Traps, assertions, panic behavior, and diagnostics

- **Rust families:** `core`/`std::panic`, `std::backtrace`, assertion and panic
  macros, panic payload and hook APIs, unwind boundaries, and backtrace capture.
- **Disposition:** xlang's aborting trap and the checks retained by the floor
  are `G0`. Trap reporting and stack capture cross `FRAME` `F-TRAP`; richer diagnostic
  presentation is `LIB` after that frame exists. Panic hooks, payloads, and
  backtraces are `LATER`. Source-visible unwinding, catching, resuming, or using
  panic as a recoverable value is `NG` under EFF-4 and ERR-1.
- **Blocked claim:** retained checks and aborting traps do not imply panic
  unwinding, cleanup-on-trap, exception safety, or portable backtraces.

### D07. Unique heap ownership and allocation service

- **Rust families:** `alloc`/`std::{boxed, vec}`, `core`/`std::alloc`,
  `alloc::alloc`, unique-owner constructors, allocation-error handling, and
  global/custom allocator declarations.
- **Disposition:** unique ownership and the allocation/growth/failure
  observations required by the detailed floor are `G0`. The actual allocator
  service is `FRAME` `F-ALLOC`. General boxed library conveniences are `LIB`. Allocator
  selection, custom allocators, alignment/layout control beyond the floor, and
  allocation policy are `LATER`. Raw allocator entry points and writer-authored
  allocator trust are inadmissible boundary evidence displaced by `F-ALLOC` and
  the later checked allocator family.
- **Blocked claim:** successful dense-family work cannot claim a custom-
  allocation API, allocator replacement, out-of-memory recovery policy, or
  arbitrary over-aligned storage.

### D08. Shared ownership and interior mutation

- **Rust families:** `core`/`std::cell`, `alloc`/`std::rc`, `alloc::sync`
  (`Arc` and related weak ownership), dynamic borrow cells, one-time cells, and
  reference-counted ownership.
- **Disposition:** the entire shared-ownership and dynamically checked interior-
  mutation capability is `LATER`; ordinary safe wrappers would be `LIB` only
  after that family closes. Atomic reference-count support also depends on
  `FRAME` `F-SYNC` in D22. Writer-visible `UnsafeCell`-class raw mutation
  authority is inadmissible boundary evidence for that later checked family.
- **Blocked claim:** unique-owner collection closure provides neither aliasing
  shared ownership nor interior mutation, cycle handling, weak ownership,
  atomic reference counts, or their destruction and overflow guarantees.

### D09. Sequential collections and topology witnesses

- **Rust families:** `core`/`std::{array, slice}`, `alloc`/`std::{boxed, vec,
  collections}`; `alloc::collections::{binary_heap, btree_map, btree_set,
  linked_list, vec_deque}`; `std::collections::{binary_heap, btree_map,
  btree_set, hash_map, hash_set, linked_list, vec_deque}`; collection-related
  primitive methods and the `vec!` macro's semantic contract.
- **Disposition:** normalized fixed, dense, deque, unordered, ordered, heap,
  append-only-pool, recyclable-pool, graph/list/arena/small-sequence witness,
  lifecycle, and failure contracts are `G0`. Safe convenience methods that
  introduce no new guarantee are `LIB` after their canonical family closes;
  aliases, reexports, and duplicate macro spellings are `RED`.
- **Blocked claim:** G0 accounting is not family closure. No one container,
  representation, language operation, or complexity promise is selected here.
  The census accounts unrestricted caller demand, while the first experimental
  base route is region-free and borrow-free. The exact payload-scope
  classification and 294-branch overlay keep `BR-STORED` as a named later
  family. Its six-state partition records 26 active, 138 deferred, 100 true
  no-complement, nine boundary-evidence, two frame-scope-deferred, and one
  delegated contract. `scope_owner_contract_ids` makes the singleton
  allocation-error delegation and evidence/frame authority explicit. Every
  deferred, delegated, boundary, or frame state blocks unrestricted `E`/`P`
  until its exact owner and branch obligations close, and excluded scope adds
  zero structural cost to the region-free/borrow-free default shape.
  The partition includes active `RangeBounds`/callable/protocol state in
  extract, splice, and filter rows; exact stored `BuildHasher` or caller
  `Hasher` roles for set relations/algebra and trait entrances; exact callable
  call-count partitions; and a singleton cached-key-array carrier at
  `VIEW-SORT-01`. Adjacent non-hash, non-callable, or ephemeral-key branches may
  not inherit those mechanisms or costs.

### D10. Synchronous iteration and ranges

- **Rust families:** `core`/`std::{iter, range}`, the legacy
  `core::ops::{Bound, Range, RangeBounds, RangeFrom, RangeFull,
  RangeInclusive, RangeTo, RangeToInclusive}` declarations,
  slice/string/collection iterator declarations, iterator adapters, range
  values, and owning, borrowed, or mutable cursor behavior.
- **Disposition:** borrowed, unique, and owning synchronous iteration required
  by registered floor contracts is `G0`. Remaining synchronous adapters are
  `LIB` once their callback, ownership, invalidation, and complexity contracts
  are derivable. Async iteration is D23 `LATER`. Only mechanically proven
  namespace, reexport, and duplicate helper spellings are `RED`; the legacy
  `core::ops` range state and queries remain independent D10 contracts where
  their caller observations differ.
- **Blocked claim:** index loops do not close iterator invalidation, escape,
  exact destruction, short-circuiting, double-ended, fused, or size-hint
  contracts unless those exact clusters pass.

### D11. Bytes, UTF-8, characters, ASCII, and strings

- **Rust families:** `core`/`std::{ascii, char, str}`, `alloc`/`std::{str,
  string}`, `alloc`/`std::ffi::c_str` where byte/text validity is the caller
  concern, and byte/string primitive methods.
- **Disposition:** byte storage plus incremental UTF-8 validation/sealing and
  boundary-safe edits required by the registered builder are `G0`. ASCII and
  already-valid UTF-8 library operations are `LIB` after their dependencies
  close. Complete Unicode normalization, segmentation, case mapping, locale,
  collation, and text-width semantics are `LATER`. Duplicate owned/borrowed
  reexports are `RED`.
- **Blocked claim:** a UTF-8 builder is not complete text support. Scalar-
  ordinal indexing receives no hidden O(1) promise.

### D12. Formatting and textual diagnostics

- **Rust families:** `core`/`alloc`/`std::fmt`; formatting traits, argument
  construction, formatting errors, and `format!`, `format_args!`, `write!`,
  `writeln!`, `print!`, `println!`, `eprint!`, `eprintln!`, and `dbg!`.
- **Disposition:** formatting contracts are `LIB`; output destinations depend
  on `FRAME` `F-IO` in D15. A complete formatting family, code-size policy, locale
  interaction, and failure behavior are `LATER`. Rust macro and derive
  spellings are `RED` after routing their semantic contract here.
- **Blocked claim:** byte/text builders do not imply generic formatting,
  console output, reflection-driven formatting, or zero-allocation formatting.

### D13. Runtime type identity and reflection

- **Rust families:** `core`/`std::any`, type identity and downcast operations,
  type-name and discriminant queries, and reflection-adjacent layout metadata.
- **Disposition:** runtime type identity, downcast, and general metadata access
  are `LATER`; no current rule establishes that they are required or rejects
  them as a permanent non-goal. Compile-time layout truth needed inside trusted
  code remains `FRAME` `F-MEM` in D04.
- **Blocked claim:** generic behavior parameters and static types do not imply
  reflection, dynamic typing, runtime downcast, dynamic loading, or a stable
  type-name ABI.

### D14. Source, configuration, build, and compile-time metadata

- **Rust families:** root configuration, inclusion, source-location,
  environment, derivation, test, and allocator attributes/macros; prelude
  editions; compiler-provided source metadata and conditional target facts.
- **Disposition:** Rust-specific attributes, derives, keyword pages, and macro
  spellings are `RED`; their semantic contracts route to the owning domain.
  Build configuration, package integration, source inclusion, compile-time
  environment, test discovery, and separate-compilation metadata are `LATER`.
  Compiler-supplied immutable target/source facts would cross `FRAME` `F-BUILD`.
- **Blocked claim:** the current closed compilation unit does not provide a
  package system, build-script contract, stable internal ABI, reflection, or
  dynamic loading. PROG-1 defines the current closed-unit kernel; dynamic
  loading remains a `LATER` build/ABI/frame contract unless the owner later
  rejects it explicitly.

### D15. Byte and structured I/O

- **Rust families:** `std::io`, `std::io::prelude`, stream traits, buffering,
  cursors, standard input/output/error, copy/read/write helpers, seek, and I/O
  error values.
- **Disposition:** pure in-memory stream and buffering logic is `LIB` after
  sequential storage closes. OS-backed I/O is `FRAME` `F-IO`; the complete I/O
  contract family, including partial progress, interruption, cancellation,
  deadlines, scatter/gather, buffering, and resource ownership, is `LATER`.
- **Blocked claim:** formatting and byte sequences do not imply I/O. No current
  result claims partial-write correctness, descriptor lifetime, blocking
  behavior, or platform error mapping.

### D16. Filesystems and paths

- **Rust families:** `std::{fs, path}` and filesystem/path extensions under
  `std::os::{darwin, linux, macos, unix, wasi, wasip2, windows}`.
- **Disposition:** platform-independent lexical path manipulation is `LIB` once
  byte/text and platform string contracts are fixed. Filesystem operations and
  metadata cross `FRAME` `F-FS`; the filesystem family is `LATER` and must cover
  handles, links, races, permissions, atomicity, partial failure, traversal,
  and platform differences.
- **Blocked claim:** string support is not path support, and path support is not
  filesystem support. No data-structure result claims race-free filesystem
  transactions or portable path encoding.

### D17. Runtime environment and process control

- **Rust families:** `std::env`, `std::env::consts`, `std::process`, and process
  extensions under `std::os::{unix, windows}`.
- **Disposition:** pure argument/environment parsing and command construction
  are `LIB`; reading or mutating process state, spawning, waiting, pipes,
  termination, and exit status cross `FRAME` `F-PROC`. The full environment/process
  family is `LATER`.
- **Blocked claim:** compile-time `env!`-class metadata is not runtime
  environment access. No current result claims process spawning, signal-safe
  cleanup, child ownership, pipe deadlock avoidance, or portable exit behavior.

### D18. FFI, platform strings, OS handles, and resource lifecycle

- **Rust families:** `core`/`alloc`/`std::ffi`, `core`/`alloc`/`std::ffi::c_str`,
  `std::ffi::os_str`, `std::os` and all stable platform submodules, C scalar
  types, C/OS strings, raw and owned descriptors/handles, and platform
  extensions.
- **Disposition:** the owner-ratified narrow outbound ABI and opaque OS/binary
  boundary is `FRAME` `F-ABI`; its complete resource, ownership, failure, buffer,
  pinning, unwind-abort, and fact-attenuation contract is `LATER`. Pure checked
  C/OS string transformations can be `LIB` after text and platform encoding
  close. Rich foreign object graphs are `LATER`; writer-visible raw FFI trust is
  inadmissible boundary evidence displaced by the checked ABI/resource family.
  FFI-in, callbacks, and foreign-thread entry are `LATER`, not silently covered.
- **Blocked claim:** unique ownership does not close resource handles. No
  current result claims descriptor/handle cleanup, ABI coverage, foreign
  concurrency, callbacks, pinning, or proof preservation across the wall.

### D19. Network values and network I/O

- **Rust families:** `core`/`std::net` value types and parsing; `std::net`
  sockets, addresses, DNS resolution, listener/stream/datagram operations, and
  OS-specific socket extensions.
- **Disposition:** pure address values, parsing, and classification are `LIB`.
  Socket and resolver operations cross `FRAME` `F-NET`; networking, including
  blocking, partial I/O, shutdown, timeout, name resolution, and resource
  lifetime, is `LATER`.
- **Blocked claim:** address parsing is not networking. No current result
  claims DNS, sockets, TLS, asynchronous network I/O, or cross-platform socket
  behavior.

### D20. Durations, clocks, and timers

- **Rust families:** `core`/`std::time`, `Duration`, `Instant`, `SystemTime`,
  elapsed-time arithmetic, sleeping, and timeout inputs in other domains.
- **Disposition:** pure duration arithmetic is `LIB`. Monotonic and wall clocks,
  sleeping, and timer service cross `FRAME` `F-CLOCK`; clock semantics, overflow,
  resolution, suspension behavior, deadlines, and timers are `LATER`.
- **Blocked claim:** benchmark timing infrastructure is not a language clock
  contract. No current result claims wall-clock correctness or timer service.

### D21. Threads and thread-local state

- **Rust families:** `std::thread`, thread builders/handles/scopes, parking,
  sleeping, identifiers, available parallelism, `thread_local!`, and platform
  thread extensions.
- **Disposition:** thread creation, joining, parking, and thread-local runtime
  service cross `FRAME` `F-THREAD`; the entire threading and thread-local family is
  `LATER`.
- **Blocked claim:** xlang v0 has no thread construct. G0 cannot claim send,
  share, join, scoped-thread lifetime, thread-local destruction, panic
  propagation, scheduling, or race-freedom beyond its single-threaded scope.

### D22. Atomics, synchronization, channels, and shared-state concurrency

- **Rust families:** `core::sync`, `core::sync::atomic`, `std::sync`,
  `std::sync::atomic`, `std::sync::mpsc`, mutexes, read/write locks, barriers,
  condition variables, once initialization, lazy values, channels, atomic
  values, and memory-order declarations.
- **Disposition:** pure safe library composition is `LIB` only after the
  concurrency substrate closes. Atomic instructions, blocking/wakeup, and
  scheduler interaction are `FRAME` `F-SYNC`; memory models, Sendable/Shareable laws,
  synchronization, poisoning policy, channels, concurrent destruction, and
  reclamation are `LATER`. Writer-visible unchecked synchronization or raw
  shared mutation is inadmissible boundary evidence displaced by the checked
  concurrency family.
- **Blocked claim:** sequential ownership does not establish a memory model,
  data-race freedom for shared state, atomic ordering, deadlock behavior,
  channel disconnect semantics, or concurrent reclamation.

### D23. Futures, tasks, async execution, cancellation, and pinning

- **Rust families:** `core`/`std::{future, task, pin}`, `alloc::task`, future
  polling, wakers, task contexts, `ready!`, `pin!`, and address-sensitive
  values.
- **Disposition:** pure future and adapter logic is `LIB` only after the family
  closes. Wakeup, scheduling, timer/I/O integration, and executor interaction
  cross `FRAME` `F-ASYNC`. Futures, tasks, pin/address sensitivity, async iteration,
  executor contracts, cancellation, and exact destruction are `LATER`.
  Writer-visible unchecked pin construction or address/lifetime forgery is
  inadmissible boundary evidence displaced by the later pin/address family.
- **Blocked claim:** synchronous iteration and callbacks do not imply async
  execution. No current result claims cancellation safety, wakeup correctness,
  executor progress, self-referential values, or address stability.

### D24. Architecture facilities, optimizer hints, and target facts

- **Rust families:** `core`/`std::arch`, stable target catalogs under
  `core::arch::{aarch64, wasm32, x86, x86_64}`, `core`/`std::hint`, target-
  feature detection macros, `core`/`std::ptr` volatile read/write evidence, and
  the collapsed architecture/intrinsic catalogs in the module accounting.
- **Disposition:** target discovery, feature dispatch, portable hint semantics,
  SIMD/vector contracts, target-specific operations, and the checked
  volatile/MMIO contract are `LATER`; compiler lowering and machine-feature
  truth are `FRAME` `F-TARGET`, while device-memory effects cross `FRAME`
  `F-MMIO`. A future checked MMIO family must freeze access authority,
  provenance, mapping/resource lifetime and invalidation; permitted widths,
  alignment, event identity, and atomicity/tearing; the exact number of access
  events on each executed source path;
  whether any speculation, duplication, fusion, splitting, widening,
  narrowing, elision, or reordering is permitted; ordering against relevant
  ordinary-memory as well as device accesses; fact attenuation for externally
  mutable device state and any ordinary-memory region reachable by an external
  agent, including DMA; platform side effects; and fault/trap behavior.
  Writer-visible unsafe intrinsics, unchecked
  reachability promises, raw-pointer volatile access, and raw target privilege
  are inadmissible boundary evidence displaced by the named later families and
  frames. Duplicate `std` reexports and feature-macro spellings are `RED`.
- **Blocked claim:** ordinary optimized scalar code does not imply portable
  SIMD, architecture intrinsics, target-feature dispatch, cache-control
  operations, stable optimizer-hint behavior, or device-memory/MMIO access.

### D25. Prelude, aliases, reexports, and documentation packaging

- **Rust families:** `core`/`std::prelude` and all edition/v1 submodules,
  `core`/`std::primitive`, crate-root reexports, deprecated numeric modules,
  aliases, and duplicate primitive renderings.
- **Disposition:** these surfaces are `RED` and route to the canonical semantic
  domain. Prelude contents do not create an independent capability.
- **Blocked claim:** `RED` is permitted only when canonicalization preserves all
  observable semantics and cost. A prelude or convenience form with a distinct
  guarantee must be moved back to its owning domain.

### D26. Compiler and runtime support beneath stable contracts

- **Rust families:** allocation and panic handlers, compiler-built operations,
  ABI glue, memory copying/zeroing, target layout, trap reporting, and the
  implementation needs evidenced by stable APIs and the unstable
  `intrinsics`/runtime modules.
- **Disposition:** every privileged implementation edge belongs to one or more
  of `F-MEM`, `F-ALLOC`, `F-TRAP`, `F-BUILD`, `F-IO`, `F-FS`, `F-PROC`,
  `F-ABI`, `F-NET`, `F-CLOCK`, `F-THREAD`, `F-SYNC`, `F-ASYNC`, and
  `F-TARGET`, or `F-MMIO`. An edge that cannot yet be assigned to an exact named frame
  remains `LATER`. Writer-callable trust, unchecked compiler facts, and raw
  intrinsic access are inadmissible boundary evidence, not independent need
  routes.
- **Blocked claim:** the presence of a compiler or runtime implementation is not
  a reviewed frame. Each fact, effect, resource, ABI, and target obligation must
  be named before it can support a shipped contract.

## 6. Stable module/path coverage crosswalk

This crosswalk accounts every module declaration marked stable by the pinned
inventory. Every item path routes to one canonical domain. A mixed module
family may appear in more than one line only where the member-level partition
is stated explicitly; that does not duplicate any item.

### 6.1 `core`

- D01: `core::{array, f32, f64, i8, i16, i32, i64, i128, isize, num, u8,
  u16, u32, u64, u128, usize}` and `core::{f32, f64}::consts`.
- D02: the `core::ops` namespace plus `ControlFlow`, `Fn`, `FnMut`, and
  `FnOnce` declarations.
- D03: `core::{borrow, clone, cmp, convert, default, hash, marker}` plus
  operator, `Deref`, and `Index` behavior declarations under `core::ops`.
- D04/D07: `core::{alloc, mem, ptr}` plus `core::ops::Drop` in D04, except the
  volatile device-memory evidence members of `core::ptr`, which route to D24.
- D05: `core::{error, option, result}`.
- D06: `core::panic`.
- D08: `core::cell`.
- D09: `core::{array, slice}`; `array` is listed with D01 at the module level,
  while its collection/view contracts route to D09 by member semantics.
- D10: `core::{iter, range}` plus
  `core::ops::{Bound, Range, RangeBounds, RangeFrom, RangeFull,
  RangeInclusive, RangeTo, RangeToInclusive}`.
- D11: `core::{ascii, char, str}`.
- D12: `core::fmt`.
- D13: `core::any`.
- D18: `core::ffi` and `core::ffi::c_str`.
- D19: `core::net`.
- D20: `core::time`.
- D22: `core::sync` and `core::sync::atomic`.
- D23: `core::{future, pin, task}` plus `core::ops::{AsyncFn, AsyncFnMut,
  AsyncFnOnce}`.
- D24: `core::{arch, hint}` and stable target families under
  `core::arch::{aarch64, wasm32, x86, x86_64}`.
- D25: `core::prelude`, `core::prelude::{rust_2015, rust_2018, rust_2021,
  rust_2024, v1}`, and `core::primitive`.

The `core` crate root itself contains reexports, macros, derives, attributes,
primitive renderings, and compiler-facing declarations. Reexports follow D25;
macros/derives/attributes follow Section 7; primitive members route by receiver
to D01, D02, D04, D09, or D11.

### 6.2 `alloc`

- D03: `alloc::borrow`.
- D07: `alloc::{alloc, boxed, vec}` for allocation and unique ownership.
- D08: `alloc::{rc, sync}`.
- D09: `alloc::{boxed, collections, slice, vec}` and
  `alloc::collections::{binary_heap, btree_map, btree_set, linked_list,
  vec_deque}`.
- D11: `alloc::{str, string}`.
- D12: `alloc::fmt`.
- D18: `alloc::ffi` and `alloc::ffi::c_str`.
- D23: `alloc::task`.

The repeated D07/D09 paths are partitioned by semantic member: allocation and
unique-owner lifecycle route to D07; sequential view and collection operations
route to D09. The `alloc` crate root's `format!` and `vec!` spellings route
through Section 7; root reexports are D25.

### 6.3 `std`

- D01: `std::{array, f32, f64, i8, i16, i32, i64, i128, isize, num, u8,
  u16, u32, u64, u128, usize}` and `std::{f32, f64}::consts`.
- D02: the `std::ops` namespace; its canonical members follow the `core::ops`
  semantic partition above.
- D03: `std::{borrow, clone, cmp, convert, default, hash, marker}`.
- D04/D07: `std::{alloc, mem, ptr}`, except the volatile device-memory
  evidence members of `std::ptr`, which route to D24.
- D05: `std::{error, option, result}`.
- D06: `std::{backtrace, panic}`.
- D07: `std::{boxed, vec}` for allocation and unique ownership.
- D08: `std::{cell, rc}`.
- D09: `std::{array, boxed, collections, slice, vec}` and
  `std::collections::{binary_heap, btree_map, btree_set, hash_map, hash_set,
  linked_list, vec_deque}`.
- D10: `std::{iter, range}`.
- D11: `std::{ascii, char, str, string}`.
- D12: `std::fmt`.
- D13: `std::any`.
- D15: `std::io` and `std::io::prelude`.
- D16: `std::{fs, path}` plus the filesystem extensions named under D18.
- D17: `std::env`, `std::env::consts`, and `std::process`.
- D18: `std::ffi`, `std::ffi::{c_str, os_str}`, `std::os`,
  `std::os::{darwin, fd, linux, macos, raw, unix, wasi, wasip2, windows}`,
  `std::os::darwin::fs`, `std::os::linux::{fs, net}`,
  `std::os::macos::fs`, `std::os::unix::{ffi, fs, io, net, prelude, process,
  thread}`, `std::os::wasi::{ffi, io, prelude}`, and
  `std::os::windows::{ffi, fs, io, prelude, process, raw, thread}`.
- D19: `std::net`.
- D20: `std::time`.
- D21: `std::thread`.
- D22: `std::sync`, `std::sync::{atomic, mpsc}`.
- D23: `std::{future, pin, task}`.
- D24: `std::{arch, hint}`.
- D25: `std::prelude`, `std::prelude::{rust_2015, rust_2018, rust_2021,
  rust_2024, v1}`, and `std::primitive`.

The `std` crate root is accounted exactly like the `core` root: canonical
reexports route to D25, macro/derive/attribute/keyword renderings route through
Section 7, and primitive members route by receiver semantics. Platform-gated
rendering does not promote a platform contract to a portable one.

## 7. Stable macro, derive, attribute, and keyword routing

The inventory includes these source surfaces because rustdoc exposes them.
Their Rust spelling is `RED`; any observable semantic contract routes as
follows:

| Semantic owner | Stable Rust source surfaces |
|---|---|
| D02/D03 | `matches!`; derives for `Clone`, `Copy`, `Default`, `Eq`, `PartialEq`, `Ord`, `PartialOrd`, and `Hash`. |
| D06 | `assert!`, `assert_eq!`, `assert_ne!`, `assert_matches!`, `debug_assert!`, `debug_assert_eq!`, `debug_assert_ne!`, `debug_assert_matches!`, `panic!`, `todo!`, `unimplemented!`, and `unreachable!`. |
| D09 | `vec!`. |
| D12 | `format!`, `format_args!`, `write!`, `writeln!`, `print!`, `println!`, `eprint!`, `eprintln!`, `dbg!`, and the `Debug` derive. |
| D14 | `cfg!`, `cfg_select!`, `compile_error!`, `concat!`, `env!`, `option_env!`, `include!`, `include_bytes!`, `include_str!`, `stringify!`, `file!`, `line!`, `column!`, `module_path!`, and the `derive`, `test`, and `global_allocator` attributes. The allocator attribute's operational contract also routes to D07. |
| D04 | `offset_of!`, `addr_of!`, and `addr_of_mut!`; their raw-address authority remains inadmissible boundary evidence under D04. |
| D21 | `thread_local!`. |
| D23 | `pin!` and `ready!`. |
| D24 | `is_aarch64_feature_detected!`, `is_loongarch_feature_detected!`, `is_riscv_feature_detected!`, `is_s390x_feature_detected!`, and `is_x86_feature_detected!`, including root reexports. |

All 40 stable Rust keyword pages are D25 `RED` documentation/source-language
surface: `Self`, `as`, `async`, `await`, `become`, `break`, `const`, `continue`,
`crate`, `dyn`, `else`, `enum`, `extern`, `false`, `fn`, `for`, `if`, `impl`,
`in`, `let`, `loop`, `match`, `mod`, `move`, `mut`, `pub`, `ref`, `return`,
`self`, `static`, `struct`, `super`, `trait`, `true`, `type`, `union`, `unsafe`,
`use`, `where`, and `while`. Their underlying language concepts are accounted
by the owning domain. The `unsafe` source capability is Rust-only surface or
inadmissible boundary evidence wherever it would grant unchecked authority;
the underlying need remains with D04, D18, D22, D23, or D24.

## 8. Unstable-only Rust 1.97 domains

Unstable items are evidence, not part of the stable completeness anchor. The
mechanical module accounting nevertheless preserves them so that future needs
do not disappear. Their current dispositions are:

- **D23 `LATER`:** `core`/`std::async_iter`.
- **D01 `LATER`:** `core`/`std::{autodiff, f16, f128}`, their constants, and
  unstable `f32::math`/`f64::math`.
- **D11 `LIB` or `LATER` after contract normalization:**
  `core`/`alloc`/`std::bstr` and `core`/`alloc`/`std::str::pattern`.
- **D02/D03 `LATER`:** `core::{contracts, field, from, index, pat}` and their
  `std` reexports.
- **D13 `LATER`:**
  `core`/`std::mem::type_info`.
- **D15/D18 `LATER` or `FRAME` `F-IO`/`F-FS`/`F-PROC`/`F-ABI`:** experimental
  `core::{io, os, process}`, Darwin Objective-C support, unstable platform
  filesystem/process modules, and `std::os::wasi::fs`.
- **D22 `LATER`:** `std::sync::{mpmc, nonpoison, oneshot, poison}`.
- **D24 `LATER`/`FRAME` `F-TARGET`, raw spelling inadmissible:**
  `core`/`std::{simd, intrinsics}`, `alloc::intrinsics`, MIR/scalable-SIMD
  submodules, unstable architecture catalogs, `core::{profiling, ub_checks}`,
  and `core`/`std::unsafe_binder`.
- **Separate future census:** `core`/`std::random`. Randomness semantics and OS
  entropy are not closed by a provisional unstable module.

No unstable-only row blocks the statement "the stable Rust 1.97 surface is
domain-accounted." It does block any stronger statement that the Rust snapshot
exhausts future systems needs.

For mechanical module coverage only, an otherwise unclassified module with
zero direct stable items receives the D26 holding route
`DOM-UNSTABLE-RUNTIME-HOLDING`. This prevents an unstable catalog from
disappearing while making no semantic classification claim. Stabilization or
use as evidence requires moving it to its exact owning domain before any
closure claim.

## 9. Exact claims allowed at each stage

### 9.1 After this ledger alone

The only permitted claim is:

> Every stable Rust 1.97.0 `core`, `alloc`, and `std` module/path family has a
> systems-domain destination, and the detailed sequential data floor is
> separated from the remaining systems envelope.

This is an accounting claim, not a derivability or completeness claim.

### 9.2 After G0-Core research closes

The permitted claim is limited to:

> The sequential, unique-owner data-structure research boundary, required
> contract families, prohibited simulations, global lifecycle obligations,
> protected baselines, and later-family boundaries are frozen well enough to
> request the first Family Lock A.

It does not claim that ordinary libraries can yet implement the floor.

### 9.3 After an individual family passes and is adopted

The claim names only that family and its exact frozen contracts. It may not
inherit adjacent rows, optional contracts, untested payloads/targets, or a whole
systems label.

### 9.4 After the complete detailed floor passes

Only after every mandatory baseline, witness, held-out, lifecycle, failure,
fact-channel, construction, and performance obligation for the floor closes may
the project seek the following claim:

> Ordinary no-unsafe xlang libraries can implement the registered sequential,
> unique-owner collection and topology contracts efficiently through public
> checked mechanisms.

Even that claim excludes custom allocators; shared ownership and interior
mutation; pinning and address-sensitive values; reflection; formatting and
build integration; I/O, filesystems, processes, OS resources, FFI, networking,
and clocks; threads, atomics, synchronization, and concurrent reclamation;
async execution and cancellation; complete Unicode; target SIMD/intrinsics;
volatile device-memory/MMIO; panic unwinding; complete numerics; and whole
systems-language completion.

### 9.5 Whole systems-language claim

A whole-envelope claim remains blocked until every `LATER` domain required by
the owner has its own registry, family lock, evidence, hostile review, and
adoption decision; every `FRAME` has an exact reviewed trust contract; every
`LIB` claim has an ordinary checked derivation; every `RED` mapping is proved
semantically and structurally lossless; and every `NG` has an owner-ratified
first-principles reason plus safe displacement whenever its caller need remains
in scope.

Rust's standard library is necessary but insufficient for that claim. At
minimum, the complement census must separately address common systems domains
not supplied as stable Rust `std` contracts, including cryptography and TLS,
memory mapping and virtual-memory control, signals, terminal control, dynamic
library policy, event multiplexing, production async runtimes, secure random
sources, serialization/protocol boundaries, and the held-out topology
witnesses.

## 10. Reopening and maintenance rules

1. **Pin before diffing.** A new census pins the Rust version, source commit,
   rustdoc toolchain, extraction policy, counts, and artifact hashes before any
   semantic update.
2. **Exactly one canonical destination.** Every stable item must resolve to one
   canonical detailed cluster or one named later-domain route and one primary
   disposition. Reexports, aliases, prelude entries, and generated impl
   duplication are never counted as new capabilities without a distinct
   observable contract.
3. **Unsafe is evidence only.** A newly stable unsafe API reopens the underlying
   domain analysis but never authorizes a writer-visible unchecked xlang
   surface.
4. **`G0` does not spread.** A G0 family can close only its frozen contracts.
   Cross-family dependencies require all implicated contracts to be frozen
   before candidate work.
5. **`LIB` requires an ordinary derivation.** A library row closes only after a
   no-unsafe, non-privileged implementation passes semantic, ownership,
   destruction, failure, invalidation, fact-channel, asymptotic, and structural
   cost review. A proof sketch, standard-library privilege, or derivation from
   an unclosed family is insufficient.
6. **`FRAME` requires a trust dossier.** A frame must name its ABI/platform,
   effects, owned resources, failure mapping, fact attenuation, target matrix,
   reentry/foreign-concurrency behavior, destruction boundary, and hostile
   review. An implementation detail is not automatically a frame.
7. **`LATER` cannot inherit closure.** Each later domain gets its own registry,
   Family Lock A, evidence, reviews, and owner adoption. Adjacent success does
   not close it.
8. **`RED` needs a canonical replacement.** Reclassification as redundant must
   cite the canonical cluster and show no loss of ownership, failure,
   invalidation, complexity, address/identity, platform, or structural-cost
   semantics.
9. **`NG` requires owner authority and a reason.** A non-goal must cite an
   owner ruling and a first-principles reason. If its caller need remains in
   scope, it must also cite a safe displacement. It reopens if the owner changes
   the goal or evidence shows that an in-scope displacement cannot meet the
   required contract or cost.
10. **Future stable Rust releases reopen only implicated domains.** Additions,
    removals, stabilizations, and semantic changes are mechanically diffed. An
    unrelated module does not invalidate closed evidence, but every affected
    domain and dependent claim reopens.
11. **Platform additions remain platform-scoped.** A newly rendered target or
    OS module reopens D18 or D24 for that platform; it does not silently widen a
    portable claim.
12. **External negative space remains live.** Because Rust `std` is not a
    completeness oracle, new cross-ecosystem witnesses or common systems needs
    can reopen the envelope even when the Rust snapshot is unchanged.
13. **Generativity remains mandatory.** Visible cross-ecosystem witnesses and
    training-excluded held-out structures retain their dependency budgets; a
    census-shaped special case cannot close a capability.
14. **Mechanism neutrality survives this artifact.** Any proposal that turns a
    row into syntax, a type, an operation, trusted state, or a compiler path is
    a new owner decision after the relevant family lock. This ledger supplies
    no such authorization.

## 11. Primary sources and controlling local evidence

- Rust 1.97.0 [`core` documentation](https://doc.rust-lang.org/1.97.0/core/index.html)
  and [source](https://github.com/rust-lang/rust/tree/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core).
- Rust 1.97.0 [`alloc` documentation](https://doc.rust-lang.org/1.97.0/alloc/index.html)
  and [source](https://github.com/rust-lang/rust/tree/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/alloc).
- Rust 1.97.0 [`std` documentation](https://doc.rust-lang.org/1.97.0/std/index.html)
  and [source](https://github.com/rust-lang/rust/tree/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/std).
- Official Rust module documentation for
  [`collections`](https://doc.rust-lang.org/1.97.0/std/collections/index.html),
  [`iter`](https://doc.rust-lang.org/1.97.0/std/iter/index.html),
  [`alloc`](https://doc.rust-lang.org/1.97.0/std/alloc/index.html),
  [`mem`](https://doc.rust-lang.org/1.97.0/std/mem/index.html),
  [`ptr`](https://doc.rust-lang.org/1.97.0/std/ptr/index.html),
  [`io`](https://doc.rust-lang.org/1.97.0/std/io/index.html),
  [`os`](https://doc.rust-lang.org/1.97.0/std/os/index.html),
  [`sync`](https://doc.rust-lang.org/1.97.0/std/sync/index.html),
  [`future`](https://doc.rust-lang.org/1.97.0/std/future/index.html), and
  [`arch`](https://doc.rust-lang.org/1.97.0/std/arch/index.html).
- [G0-Core research charter](G0-CORE-CHARTER.md), especially its finite-anchor,
  accounting, derivation, non-authorization, and claims rules.
- [General-purpose data-structure capability research](../general-purpose-data-structure-capability-RESEARCH.md),
  which defines the detailed sequential floor and its family boundaries.
- [`CONSTITUTION.md`](../../../CONSTITUTION.md),
  [`PATTERNS.md`](../../../PATTERNS.md), and
  [`kernel-spec-v0.6.md`](../../../spec/kernel-spec-v0.6.md), which control
  safety, proof-elision, traps, errors, compilation, and current capability
  semantics.
- [Owner directives](../../notes/user-directives.md), especially D4, D11, and
  D12.
