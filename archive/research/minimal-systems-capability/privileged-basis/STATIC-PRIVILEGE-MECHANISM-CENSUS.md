# Static privilege-definition mechanism census

Status: D14 production-language evidence census, 2026-07-15. This report is
descriptive evidence for the next gate comparison. It does not select an xlang
mechanism, public capability basis, source spelling, or implementation.

## Question and method

The question is how a compiler, runtime, or official core implementation can
define semantics that an ordinary source definition cannot acquire merely by
choosing a package name, attribute, command-line flag, declaration shape, or
link symbol. The contrast set is deliberately small and mechanism-diverse:

- Rust 1.97.0 (`2d8144b7880597b6e6d3dfd63a9a9efae3f533d3`): compiler
  intrinsics, lang items, internal attributes, and an ordinary-user `unsafe`
  counterexample;
- Swift at `58c872384774c41f85c0e5c57b61c5416a16ec48`: a compiler-synthesized
  `Builtin` module plus standard-library compilation modes;
- Go at `3c2bdfff47b73889040c9152dd7f985c806ac15a`: exact
  architecture/package/function intrinsification, internal packages, and
  compiler directives; and
- .NET runtime at `2b4d69bfe02dc60350f7dc18cb5d04045a22cf69`:
  runtime-owned method identity, JIT intrinsic tables, and exact managed/native
  imports.

These cases cover the candidate classes named in `HANDOVER.md` without turning
the census into an API inventory. All implementation claims below are pinned to
official documentation or official source. The xlang consequences are
inferences from those facts and the binding P0, W1, and W3 rules.

## Observed production mechanisms

### Rust: compiler declarations and library-selected hooks

Rust separates several mechanisms rather than presenting one privilege route.
`core::intrinsics` declarations import implementations supplied by compiler
MIR, code generation, or constant evaluation. The documentation calls them
implementation details of `core`, directs users to stable wrappers, and notes
that a newly indirect const-stable intrinsic needs language-team approval
because users cannot reproduce it in stable Rust. Lang items let a library
definition supply an implementation for a compiler-selected operation.
Compiler-internal attributes add further exact hooks used by the runtime and
core library.

This is not a W3 precedent as a whole. On nightly Rust, ordinary source can
enable `lang_items`, `core_intrinsics`, and `rustc_attrs`; ordinary Rust source
can also use `unsafe` functions and blocks by design. The useful precedent is
the split between a compiler-owned semantic contract and safe public wrappers,
not Rust's admission boundary.

Authority and ownership:

- The compiler owns intrinsic meaning, lowering, and the table of recognized
  lang items and internal attributes.
- `core` owns declarations and safe or conditionally safe public wrappers.
- Backend and const-evaluation implementations may be distinct.
- A new compiler semantic hook normally changes compiler source and, when
  surfaced through `core`, official library source; it is not a zero-review
  library-only extension.

Cost and portability:

- Recognition itself has no runtime transition cost. A wrapper can inline to
  the intrinsic, but the body may lower to an instruction sequence or libcall.
- The language boundary can be backend-neutral while each backend and constant
  evaluator must implement the semantic operation.
- Intrinsic declarations can exist without a shipped standard library, but
  user-friendly safe wrappers then belong to ordinary libraries or a
  compiler-embedded interface.

Xlang consequence: compiler-owned semantics plus checked public wrappers can
serve P0, but copying nightly feature gates, user attributes, lang-item
declarations, or `unsafe` would directly fail W3 and add W1-visible modes.

### Swift: synthesized `Builtin` module and official-core mode

Swift's compiler constructs the `Builtin` module as an AST object and
synthesizes declarations for builtin types and operations. Standard-library
source imports and wraps those declarations. This cleanly separates compiler
semantic ownership from the public core-library API and permits wrappers to
have ordinary safe call shapes even when their implementation uses builtin or
runtime operations.

The physical design is stronger than name recognition: the compiler records a
specific `Builtin` module object, and module identity tests compare against that
object. An ordinary source module named `Builtin` is therefore not the same
compiler-owned module. However, the sampled toolchain exposes
`-enable-builtin-module`, while `-parse-stdlib` implicitly imports the module.
Those user-selectable modes are explicit counterexamples to xlang's required
admission policy. Swift also exposes writer-accessible unsafe facilities.

Authority and ownership:

- The compiler owns the module identity, synthesized declarations, type rules,
  and lowering.
- official core source owns wrappers and may also call runtime shims through
  exact external names;
- the runtime owns bodies that cannot be expressed as direct target lowering;
  and
- adding a primitive can require a compiler declaration/lowering change, a
  core wrapper, and possibly a runtime shim.

Cost and portability:

- A direct builtin call need not add a runtime boundary; wrappers can disappear
  after optimization.
- A runtime shim adds an ABI transition unless optimized away.
- The abstract builtin surface can be target-neutral, but target-specific
  availability and lowering still require compiler/backend work.
- A synthesized module works when no standard library ships. The compiler must
  then expose any intended public primitive contract itself or allow ordinary
  checked libraries to wrap an unforgeably identified public declaration.

Xlang consequence: a compiler-created declaration identity is a strong W3
building block and can keep public wrappers regular for W1. A flag-selected
privileged mode is not acceptable, and using a module name rather than the
compiler-created identity would lose the relevant property.

### Go: exact intrinsic keys, official package boundaries, and directives

The Go compiler's intrinsic registry keys entries by architecture, package, and
function. It registers exact operations from packages including `runtime`,
`math/bits`, `sync/atomic`, and `internal/runtime/atomic`; some mappings fall
back to ordinary calls when an architecture or feature does not supply the
intrinsic. Public packages therefore provide normal call surfaces while the
compiler can replace selected declarations with target operations.

The `go` command prevents imports of an `internal` package from outside the
parent tree, and standard-library import paths are resolved specially rather
than fetched as ordinary modules. These are useful distribution boundaries,
but a package path or function name alone is not a sufficient xlang authority
identity: the compiler registry visibly begins from names, and an alternative
front end or raw compilation path must preserve the official-package
provenance check.

Go is also a W3 counterexample. Ordinary source can import `unsafe`, and
`//go:linkname` can alias an object-file symbol across package encapsulation and
even type safety. This shows why exact symbol imports cannot themselves grant
trusted semantics.

Authority and ownership:

- The compiler owns the intrinsic registry and target-dependent replacements.
- official library packages own public wrappers and fallback implementations.
- internal runtime packages own low-level declarations and helpers.
- New optimized operations usually change the compiler registry and official
  package; a new runtime operation may additionally change assembly or runtime
  source.

Cost and portability:

- Successful intrinsification has no call-transition requirement; an
  unsupported mapping may remain a normal call.
- Per-architecture registry entries make backend coverage explicit but grow a
  cross-product of operations and targets.
- The mechanism can work without a shipped standard library only if the
  compiler owns stable declaration identities or public declarations. Merely
  reserving import strings would make authority depend on package resolution.

Xlang consequence: exact operation tables and safe wrappers are favorable to
P0 and auditability. Name/path recognition and writer-accessible directives are
W3 failures if treated as authority, while architecture-specific tables impose
a visible TCB and maintenance cost.

### .NET: runtime-owned method identity, JIT tables, and native imports

.NET's core library contains an internal `IntrinsicAttribute`, but the JIT does
not infer privilege from source spelling alone. During import it receives a
runtime method handle and runtime-supplied flags, including the intrinsic flag,
then selects from a closed named-intrinsic table. Core-library methods such as
`RuntimeHelpers` expose ordinary managed signatures, often with a managed
fallback, while recognized forms may be expanded by the JIT.

For operations implemented in the native runtime, CoreCLR distinguishes
`InternalCall`/FCall and QCall. Both use exact runtime-owned registration.
QCall uses normal platform calling conventions and marshaling; FCall is a
shorter but more error-prone transition. Official guidance prefers private
managed wrappers around these native entries.

This arrangement is not a W3 precedent for the full language. Ordinary C# can
enable unsafe blocks, and .NET exposes ordinary native interop. The relevant
precedent is that source attributes and method names are advisory until the
runtime binds a core-library method identity that the user assembly does not
own.

Authority and ownership:

- The runtime owns core-library assembly/type/method identity and intrinsic
  flags.
- The JIT owns the closed intrinsic table and optimized expansion.
- CoreLib owns managed public wrappers and fallbacks.
- The native runtime owns registered QCall/FCall bodies.
- A new primitive may require coordinated CoreLib, runtime/JIT, and native
  changes.

Cost and portability:

- JIT-expanded intrinsics need not incur a call transition.
- QCall/FCall operations do incur a managed/native boundary unless special
  optimization removes it; the two routes have different performance and
  safety tradeoffs.
- Managed fallback bodies improve backend portability, while JIT and native
  implementations enlarge the runtime TCB.
- The identity mechanism presupposes a toolchain-owned core assembly. A
  no-standard-library language would need the analogous identities embedded in
  the compiler or runtime rather than supplied by a distributable user library.

Xlang consequence: toolchain-owned declaration identity is stronger than an
attribute spelling and supports auditable exact matching. Multiple JIT and
native import routes increase TCB branches, and a mandatory runtime transition
would need a P0 justification for each primitive.

## Mechanism-class comparison facts

| Mechanism class | Authority decision | Why ordinary source cannot define an equivalent in a W3-compatible form | Safe public call model | Extension path | Runtime/code-shape cost | No-stdlib and backend consequences | Independent-route and audit consequence |
|---|---|---|---|---|---|---|---|
| Compiler-hard-coded public operation | Parser/checker or semantic operation table recognizes a closed operation identity | The operation is not a source declaration and accepts no writer-supplied proof or implementation | Ordinary checked call or operation with compiler-defined preconditions and effects | Compiler, checker, lowering, constant evaluator, and tests change for each new operation | No inherent transition; direct lowering can be optimal | Works without a library; every backend must lower it or use a specified fallback | One authority route, but the compiler table and per-operation code grow |
| Compiler-only intrinsic/builtin definition form | Compiler accepts a declaration form only in a toolchain-owned artifact | The decisive condition must be artifact identity, not a token, attribute, path, or flag ordinary source can reproduce | Checked wrappers call the privileged declaration; callers receive no definition authority | Compiler contract/lowering plus the owned declaration artifact; runtime only when needed | Wrapper can inline away; body may be instruction sequence or libcall | Works without a shipped library if declarations are embedded; backend-neutral contracts still need backend implementations | One primary definition route if all privileged declarations use it; the privileged form and loader become TCB |
| Sealed compiler-embedded core module | Compiler constructs the module and compares unforgeable in-process declaration/module identity | A user module with the same name is a different object; no user-selectable loader path may substitute for it | The embedded module may expose safe primitives directly or through ordinary wrappers | Compiler-owned module manifest plus lowering and tests; optional runtime body | No inherent transition; wrapper cost depends on optimization | Naturally no-stdlib; embedded interface increases compiler resident surface | One auditable registry can enumerate all authority if it does not delegate to separate name-based routes |
| Specially compiled official core source mode | Driver/frontend enables extra semantics while building official source | A W3-compatible version requires an internal immutable build action; a public flag, path, package name, or attribute fails this condition | Official source publishes checked wrappers | Compiler mode and official source change together; bootstrap and release pipeline also change | Usually no inherent transition | Can build a core artifact, but contradicts a strict no-shipped-library deployment unless only an embedded result remains | Adds a second language mode and release-pipeline trust; W1 and audit burden rise |
| Exact native runtime import | Compiler/runtime binds an already-authorized declaration identity to an exact native body | Symbol text alone is forgeable; the declaration must first be owned by the compiler/runtime registry | Private native entry behind a checked public wrapper | Declaration registry, ABI description, runtime implementation, and target support change | ABI transition, marshaling, and unwind behavior can be material | Useful only for irreducible runtime/OS leaves; every target/runtime must implement or reject it | A physical body mechanism, not sufficient semantic authority by itself; each ABI route enlarges TCB |

## Cross-cutting answers to the ten census questions

1. **Authority granted.** The observed mechanisms let the toolchain attach
   semantics, lowering, ABI behavior, or runtime identity that an ordinary
   function body does not establish.
2. **Source non-equivalence.** The strongest observed discriminator is
   toolchain-owned declaration/module/method identity. Names, attributes,
   import paths, symbols, and flags are insufficient unless resolution first
   proves that identity.
3. **Safe calls.** Every contrast supports normal public wrappers. Callability
   and definition authority are separable.
4. **Ownership split.** Compilers own recognition and static contracts;
   official libraries own wrappers and portable fallbacks; runtimes own only
   bodies requiring runtime state, OS entry, JIT knowledge, or target support.
5. **Runtime cost.** Recognition and direct lowering have no necessary runtime
   transition. Native shims do, and fallback calls may. The mechanism class
   does not guarantee zero cost for every semantic operation.
6. **Extension.** Every credible route requires a toolchain change. This is a
   desired review boundary for xlang, not an extensibility defect. The minimum
   changed components depend on whether the new operation needs runtime state.
7. **No standard library.** Compiler-hard-coded operations and embedded
   declarations work directly. Source-mode and core-assembly mechanisms require
   either a build-only official artifact or an embedded result. Public wrappers
   can otherwise be ordinary checked libraries.
8. **Backend independence.** Contracts can be backend-neutral, but lowering is
   not free: each backend needs an implementation, a specified runtime/libcall
   fallback, or an explicit unsupported-target result.
9. **Route count.** Rust and .NET demonstrate how intrinsic, library-hook, and
   native-import routes accumulate. A single xlang admission decision should
   not be confused with a single physical lowering or runtime body.
10. **P0/W1/W3/TCB.** Direct lowering and safe wrappers support P0; one regular
    public call model supports W1; only toolchain-owned identity with no
    user-selectable entry supports W3. Every additional parser mode, resolver
    exception, intrinsic table, backend implementation, and runtime ABI entry
    is auditable TCB and must earn its place.

## Falsifiers carried into the gate comparison

The next gate comparison must reject or explicitly discharge each of these
failure modes:

- an ordinary dependency can acquire authority by choosing a reserved name or
  path;
- source can spell an internal attribute, declaration form, or module import;
- a command-line flag turns arbitrary input into official-core input;
- a linker symbol or native import name grants semantics before declaration
  identity is authorized;
- a user-supplied opcode, formula, contract, or state-machine descriptor is
  interpreted as trusted semantics;
- a backend silently implements weaker checks or effects than the abstract
  contract;
- no-stdlib operation accidentally makes a distributable library artifact the
  root of authority; or
- a second intrinsic, runtime, loader, or JIT route bypasses the primary audit
  registry.

## Primary-source ledger

All links are official project documentation or source. Source links are pinned
to the revisions named above.

- Rust intrinsic contract:
  <https://doc.rust-lang.org/1.97.0/core/intrinsics/index.html>
- Rust lang items and nightly user reachability:
  <https://doc.rust-lang.org/1.97.0/unstable-book/language-features/lang-items.html>
- Rust `unsafe` source reachability:
  <https://doc.rust-lang.org/1.97.0/reference/unsafe-keyword.html>
- Rust compiler-internal attribute registry:
  <https://github.com/rust-lang/rust/blob/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/compiler/rustc_feature/src/builtin_attrs.rs>
- Rust lang-item registry and dependency lookup:
  <https://github.com/rust-lang/rust/blob/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/compiler/rustc_hir/src/lang_items.rs>
- Swift builtin synthesis:
  <https://github.com/swiftlang/swift/blob/58c872384774c41f85c0e5c57b61c5416a16ec48/lib/AST/Builtins.cpp>
- Swift builtin-module construction and identity:
  <https://github.com/swiftlang/swift/blob/58c872384774c41f85c0e5c57b61c5416a16ec48/lib/AST/ASTContext.cpp>
  and
  <https://github.com/swiftlang/swift/blob/58c872384774c41f85c0e5c57b61c5416a16ec48/lib/AST/Module.cpp>
- Swift standard-library wrappers:
  <https://github.com/swiftlang/swift/blob/58c872384774c41f85c0e5c57b61c5416a16ec48/stdlib/public/core/Builtin.swift>
- Swift privilege-relevant command-line modes:
  <https://github.com/swiftlang/swift/blob/58c872384774c41f85c0e5c57b61c5416a16ec48/include/swift/Option/Options.td>
  and
  <https://github.com/swiftlang/swift/blob/58c872384774c41f85c0e5c57b61c5416a16ec48/lib/Frontend/ModuleInterfaceSupport.cpp>
- Go intrinsic registry:
  <https://github.com/golang/go/blob/3c2bdfff47b73889040c9152dd7f985c806ac15a/src/cmd/compile/internal/ssagen/intrinsics.go>
- Go standard-package resolution:
  <https://go.dev/ref/mod#module-path>
- Go internal-package enforcement:
  <https://github.com/golang/go/blob/3c2bdfff47b73889040c9152dd7f985c806ac15a/src/cmd/go/internal/load/pkg.go>
- Go compiler directives and the `go:linkname` counterexample:
  <https://pkg.go.dev/cmd/compile#hdr-Compiler_Directives>
- .NET intrinsic attribute:
  <https://github.com/dotnet/runtime/blob/2b4d69bfe02dc60350f7dc18cb5d04045a22cf69/src/libraries/System.Private.CoreLib/src/System/Runtime/CompilerServices/IntrinsicAttribute.cs>
- .NET closed named-intrinsic table and JIT import:
  <https://github.com/dotnet/runtime/blob/2b4d69bfe02dc60350f7dc18cb5d04045a22cf69/src/coreclr/jit/namedintrinsiclist.h>
  and
  <https://github.com/dotnet/runtime/blob/2b4d69bfe02dc60350f7dc18cb5d04045a22cf69/src/coreclr/jit/importercalls.cpp>
- .NET managed intrinsic wrappers:
  <https://github.com/dotnet/runtime/blob/2b4d69bfe02dc60350f7dc18cb5d04045a22cf69/src/libraries/System.Private.CoreLib/src/System/Runtime/CompilerServices/RuntimeHelpers.cs>
- .NET QCall/FCall ownership and transition design:
  <https://github.com/dotnet/runtime/blob/2b4d69bfe02dc60350f7dc18cb5d04045a22cf69/docs/design/coreclr/botr/corelib.md>
- C# ordinary-user unsafe counterexample:
  <https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/unsafe-code>

## Census boundary

This census establishes comparison facts, not a recommendation. In particular,
it does not prove that a sealed embedded module, an intrinsic declaration form,
or any combination is minimal for xlang. That decision requires an explicit
architectural comparison against the falsifiers above, followed by hostile
review. Only after that review may the public capability basis be derived.
