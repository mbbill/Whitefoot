# Library modules: the prelude core and the standard library

This investigation decides how the prelude and a standard library become
ordinary modules, the work the owner selected after the modular compilation
PR ([todo item](../../../docs/todo.md)). It owns the design, its measurements and
its rejected alternatives; the decisions that survive go to the design tree,
and the rules they need go to the specification. Nothing here is implemented
yet, and nothing here is a design-tree decision until the owner rules on the
[amendments](../../../design/amendments/).

## Question

Four questions, fixed by the todo item:

1. Which prelude parts stay compiler-owned, and which become source modules?
2. How does a program name library modules? The name-resolution decision
   defers external dependency-name binding, and the modular design keeps the
   compiler-owned prelude out of an ordinary source package called `std`.
3. Do library modules join every check and closure, or only those that name
   them?
4. How are library verdicts, proof receipts and objects reused across
   programs?

## Current state

Facts at `9a858a4c` (main after the modular compilation merge).

**The prelude is 69 hand-written records plus a built-in catalog.**
`compiler/src/prelude.rs` holds 69 records: 18 opaque structs (the storage
shapes `Array`, `Slots`, `Ring`, the cell `Box`, and 14 host handles), two item
files (the structs `TcpConnection`, `AcceptedConnection`, `Inputs` and the
enums `ArgError` through `ListStop`), and 49 function signatures (29 host
functions, 9 construction functions, 9 window operations, `swap`,
`free_empty`). `Bool`, `Option`, `Result`, `Overflow`, `DivError`,
`NarrowError`, `Int` and `Float` are a separate 24-record built-in catalog
(`compiler/src/resolution/catalog.rs`), the only part a test compares with
specification section 14. The 69 records match that section's text, but no
test holds them to it.

**Every check re-parses and re-checks all of it.** `with_prelude_and_modules`
(`compiler/src/source.rs`) appends the 69 records to every module check,
composition and build; they belong to no module, sit in the unit's root scope
and are visible everywhere. The cache keys them only through the compiler's
own executable hash (`compiler/src/driver/cache.rs`).

**The host part is a library in all but name.** No rule outside section 14
names `IoError`, `HandleFactory`, `OutputStream` or any host function; only
the worked example of section 16 calls `exit_status` and returns
`ExitStatus`. Language rules do name `Bool` (conditions), `Option` and
`Result` (ERR-1, `propagate`), `Overflow`, `DivError` and `NarrowError` (the
checked operation table), the storage shapes and `Box` (TYPE-9, STOR-8) and
their construction and window operations (OP-10, OP-13). Beyond that example,
`ExitStatus` and `Inputs` are named only by the runner
(`compiler/src/driver/launcher.rs` matches both by spelling) and by PROG-3's
permission to use `Inputs`.

**The specification forbids external packages today.** PROG-1: "There is no
source include, external package, glob import, source-path search, dynamic
loading or reflection." MOD-8 makes a body-less interface declaration pending,
so no source form of a declaration whose definition the build supplies exists
outside the prelude.

**Libraries exist only as source-bundle text.** `lib/containers/` holds six
container libraries (1,871 lines: grow vector, deque, slab, hash map,
priority queue, ordered map), each a bundle with prefixed global names that a
caller concatenates before its own source. Apart from their own callers in
`tests/programs/containers/`, no program uses them. The I/O programs in
`tests/programs/` instead repeat the same helpers: an 88-line `IoError`
classifier in seven programs, a `write_once` retry loop in six, a `send_once`
retry loop in three, port parsing in four, and zero-filled byte windows
written inline 32 times. A module program cannot reach `lib/` at all: MOD-2
reads only records under its package root.

## Measurement E1: what the host rows cost every check

**Criterion, recorded before the run** (2026-09-25T07:25Z). Same source,
two compilers: (A) the gate build at `9a858a4c`; (B) the same tree with the
45 host records (14 handles, both item files, 29 host functions) removed from
`compiler/src/prelude.rs`, keeping the 24 storage, construction and window
records and the built-in catalog. Primary measure: callgrind instructions;
secondary: median wall time. If B executes at least 30% fewer instructions
than A on a small module check, the host rows are a material fixed cost of
every check; below 10%, they are not.

**Workloads.** A one-function module (`pkg::a`, `public fn one() -> result:
u64 pure`); and the 32-module, 16-function chain that
[`modular-build-cost/run.sh`](../../experiments/modular-build-cost/run.sh)
generates, with the root entry returning `u64` so that B, which has no
`ExitStatus`, reads the same source. Machine: this container, 4 cores of an
Intel Xeon at 2.10 GHz; 15 wall runs for the small module, 11 for the chain
checks, 5 for `--check-modules`.

| Check | A instructions | B instructions | Fewer | A median | B median |
|---|---|---|---|---|---|
| One-function module, `--check-module` | 171,944,274 | 65,613,478 | 61.8% | 21.4 ms | 10.7 ms |
| Chain module `pkg::m16`, `--check-module` | 517,411,939 | 302,111,721 | 41.6% | 54.7 ms | 33.9 ms |
| Chain entry, `--entry app --check`, no cache | 21,572,999,804 | 13,780,549,497 | 36.1% | 1761.0 ms | 1113.9 ms |
| Chain, `--check-modules` | | | | 1852.3 ms | 1206.2 ms |

Process start alone is about 3 ms. Control: a `u32` literal edited into
`m16/body.wf` is rejected by both compilers with the same TYPE-5 diagnostic at
the same column, so B still checks what A checks.

**Result.** The criterion is met: the host rows are a material fixed cost,
paid by every module check and every composition whether or not the module
uses a single host declaration. The chain composition without a cache is 647
ms slower in A: it checks every closure module's verdict and then the
closure, each with the full prelude.

**What the cost is.** The saving grows with the module: 106 million
instructions on the one-function module, 215 million on the 16-function one.
Callgrind attributes it mostly to the checker's nominal passes; parsing is
18.6 million of the 215 million:
`reject_recursive_nominal_layouts` (57.8 million in A, 2.4 million in B),
`nominal_dependencies` (47.8 to 0.8), `ensure_nominals_in_node` (70.2 to
11.7), `instantiate_function_signature` (71.2 to 15.8) and
`validate_generic_templates` (88.7 to 36.8), with 28 million instructions of
`CheckedType` hash-set insertion that vanish in B. The 28-variant `IoError`
and the `Result<_, IoError>` signatures of the host functions feed these
passes, whose work grows with the prelude's nominals times the module's
functions. Part of the cost is therefore an implementation inefficiency that
a fix to those passes would reduce for every check; recorded in
[`docs/todo.md`](../../../docs/todo.md). Removing the rows from checks that do not
name them removes the cost entirely for those checks, and the design below
does both.

**Limits.** B is a probe, not the design: a module that names a standard
library module will process that module's interface, as it processes any
dependency's. The numbers bound what a check that names no host declaration
saves; they do not measure a check that names some.

## Design

### D1. The core stays compiler-owned; host declarations become `std` modules

The **core prelude** keeps exactly the declarations a language rule names:
`Bool`, `Option`, `Result`, `Overflow`, `DivError`, `NarrowError`, the bounds
`Int` and `Float`, the storage shapes `Array`, `Slots`, `Ring`, the cell `Box`,
the nine construction functions, the nine window operations, `swap` and
`free_empty`. It stays visible in every module without a name, because
conditions, `propagate`, the checked operations, TYPE-9 layouts and the
OP-10/OP-13 rows are the language's own meaning; a program cannot opt out of
them. The compiler keeps their closed operation identities as it does now
(`design/compiler/prelude-records`).

Every **host declaration** moves into a module interface of the standard
library: the 14 host handles, `Inputs`, `TcpConnection`, `AcceptedConnection`,
the eight host error enums and the 29 host functions, `ExitStatus` and
`exit_status` among them. Ground: no language rule names them, and the
constitution already requires that no declaration, type, ownership, effect,
proof, diagnostic or semantic rule distinguish a value or function by whether
its implementation crosses the host boundary. An ordinary public declaration
of an ordinary module interface is the most direct form of that requirement,
where a compiler-injected declaration visible in every module is not.

Initial layout, ordered by dependency:

| Module | Declarations |
|---|---|
| `std::io` | `HandleFactory`, `OutputStream`, `InputStream`, `IoError`, `ReadStop`, `write_once`, `read_next` |
| `std::text` | `Args`, `HostString`, `ArgError`, `CopyError`, `Utf8Error`, `Utf8CopyError`, `args_count`, `arg_get`, `host_bytes_len`, `host_copy_bytes`, `host_utf8_len`, `host_copy_utf8` |
| `std::fs` | `RelativePath`, `PathError`, `DirectoryRead`, `ReadFile`, `DirectorySource`, `ListStop`, `relative_path` and the file and directory functions |
| `std::net` | `SocketAddress`, `TcpListener`, `TcpReceive`, `TcpSend`, `TcpConnection`, `AcceptedConnection` and the socket functions |
| `std::process` | `Inputs`, `ExitStatus`, `exit_status` |

`ExitStatus` and `exit_status` are the one contested placement: 1,159 of the
1,307 conformance sources name `exit_status` and no other host declaration,
so moving them rewrites those sources (an alias or a qualified path each),
and the worked example's `main` changes with them. Keeping them in the core would
spare that rewrite but put a host handle among the language's own
declarations, with no rule naming it; the runner, which interprets a returned
value outside the source call (PROG-3), can recognize `std::process::ExitStatus`
by identity as easily as by spelling. Recommended: move them.

Rejected:

- Keeping the whole prelude implicit and adding a separate library beside it:
  the host rows would keep costing every check (E1), and host declarations
  would stay a second, compiler-injected declaration path.
- Moving the storage shapes, `Box` or their operations into `std`: their
  layout (TYPE-9), measures and allocation facts are language meaning the
  checker must know in every module; a module form would add a declaration
  path without removing any compiler knowledge.

### D2. A fixed `std` qualifier names the toolchain's standard library

The standard library is one source package shipped with the toolchain, with
its own graph registering its modules, and the fixed qualifier `std` names it,
as the fixed `pkg` names the program's own package. A module program's graph
row lists the standard library modules its module may name, beside its own
package's modules (`pkg::report: [std::io, pkg::data];`); MOD-5's single
access rule applies unchanged, so a record names `std::io` only when its row
lists it. A source bundle has no row, so it may name every standard library
module; its check and closure include only the modules it actually names,
with their dependencies. Inside the standard library's own records, `pkg`
names the standard library itself, which is the existing meaning of `pkg`:
the source's owning package.

Rejected:

- General external-package binding now (`package name = location;` in the
  graph): it answers third-party libraries too, but needs identity, version
  and renaming rules the owner has not selected, and the standard library,
  shipped with the compiler, needs none of them. It stays deferred, and a
  fixed `std` does not constrain it: a later binding form can treat `std` as
  one pre-bound name.
- An implicit standard prelude, every `std` declaration visible without a
  name in every module: it contradicts the single dependency authority and the
  refusal of glob imports, and E1 shows what injecting unused declarations
  into every check costs.

### D3. Library modules join only the checks and closures that name them

A module check reads the core prelude, its own records and the interfaces of
the modules its row lists, standard library modules included; a composition
selects the closure its entry reaches through rows. A module that names no
`std` module pays nothing for the standard library (E1 bounds the saving),
and a no-heap entry never selects a container module it does not reach. This
is D2's row rule applied to the closure; it needs no rule of its own beyond
MOD-8's existing "the interfaces of the modules it may name".

### D4. The build supplies the host definitions; the specification keeps their boundaries

A host declaration is a public interface declaration without a body, like any
pending declaration. Its definition is supplied by the build from the runtime
units, as the prelude's host rows are supplied today and as PROG-1 already
permits ("Build and link supply definitions for ordinary declarations").
Composition's requirement that every declared function have its definition
(MOD-8) is met by a Whitefoot body or by a definition the build supplies; which
standard library declarations the toolchain's runtime supplies is compiler
metadata keyed by their identities, like the compiler-owned PRE-1 rows, and
never a source marker.

Because those definitions are trusted, not checked, the specification keeps
stating their boundaries: section 14 becomes the core prelude plus the exact
interface text of the host modules of D1. The standard library's Whitefoot
modules, such as the containers of D5, are checked code and are not
specification text; their interfaces are library documentation.

Rejected:

- A source modifier (`extern`, `native`, `host`) on declarations the build
  supplies: it distinguishes declarations by implementation origin, which the
  constitution and `language/system-interface/declaration-home` refuse.
- Moving the host interfaces out of the specification with the rest of the
  library: the safety guarantees depend on the supplied definitions honoring
  their boundaries (SCOPE-3), and the constitution requires the specification
  to state the external conditions its guarantees depend on.

### D5. The container libraries become the first Whitefoot-bodied `std` modules

`lib/containers/` moves into the standard library as modules
(`std::collections::vector`, `deque`, `slab`, `hash_map`, `priority_queue`,
`ordered_map`), each with a `module.wfm` publishing its operations and the
fields its callers read; their existing callers in `tests/programs/containers/`
and their allocation ledgers become the regression suite of the move. Helpers
the corpus repeats (a `write_once` loop, an `IoError` classifier, byte-window
filling, decimal parsing and formatting) are library candidates with measured
duplication, but each needs its own interface decision; they follow once the
package works.

An earlier container investigation recorded the owner's lean toward having no
standard library at all
([containers-and-resources](../containers-and-resources/DESIGN.md)). The owner's
selection of this work supersedes that lean; the libraries it produced are the
first content.

### D6. Verdicts, receipts and objects are reused by identity, across programs

A standard library module's verdict depends only on its own records, the
standard library graph, the core prelude and its dependencies' interfaces
(MOD-8), never on the program that names it, so its cache key is independent
of the consumer: two programs sharing a `--cache` directory check each
standard library module once. Proof receipts are keyed by canonical inputs
already. Stable LLVM names derive from module-qualified spellings, so a
non-generic standard library function has one name in every program and its
fragment object is reused wherever its text is equal; a generic instance is
reused wherever its concrete arguments are equal. A toolchain-supplied warm
cache is possible later and needs no new key.

## Specification changes

One amendment, following the `spec-amendment` skill, after the owner rules on
D1 to D4:

- PRE-1: the core list of D1; the host records leave it, and its diagnostic
  preorder shrinks.
- Section 14: the host modules' interface text (D4).
- PROG-1: the standard library is the one package outside the program that a
  program may name; no other external package.
- PROG-2: a source bundle may name every standard library module.
- MOD-1, MOD-5: a row may list standard library modules; `std` is a module
  prefix beside `pkg` and aliases.
- MOD-4: an alias may target a standard library path.
- MOD-8: a definition the build supplies meets composition's definition
  requirement.
- TYPE-2: a value of an opaque struct is formed only by a definition the build
  supplies, the construction rows or a host function.
- PROG-3 and the worked example (section 16): `std::process::ExitStatus` and
  `Inputs` if D1's recommendation holds.

## Implementation plan

1. The standard library package: its root in the toolchain, the `std`
   qualifier in paths, rows and aliases, module identities that include the
   package, standard library records in module checks and closures, and
   consumer-independent cache keys.
2. The host modules: the host records leave `compiler/src/prelude.rs` for
   `std` interface files; supplied-definition metadata replaces the prelude
   origin test; runtime symbols stay as they are through that metadata; the
   runner recognizes `Inputs` and `ExitStatus` by identity.
3. The corpus: conformance cases and test programs name the host modules
   through aliases or qualified paths, with every verdict and runtime result
   unchanged.
4. The containers: `lib/containers/` becomes `std::collections`, with its
   callers and allocation ledgers.
5. Measurements: E1 again with the real split; E2, a second program reusing
   the first one's standard library verdicts through a shared cache; the
   chain and wfgrep front-end times before and after.

## Open points

- Whether the standard library is versioned apart from the compiler. It is
  not, for now: its records are inputs to the compiler identity's cache scope
  like any other records, and it ships with the compiler.
- The standard library module granularity of D1 is a proposal; D2 to D4 do
  not depend on it.
- The nominal passes' growth with prelude size is recorded separately; this
  design does not wait for it.
