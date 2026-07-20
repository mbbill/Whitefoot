# Whitefoot production compiler

This is the permanent safe-Rust implementation of the Whitefoot compiler. Its
exact initial language target is `spec/kernel-spec-v0.8.md`, SHA-256
`d04336f7fa8d1a6a0f03fe58a17f972b658217a73a3dff91a906b4ba295328a8`.

The workspace is in Phase 2. It establishes production contracts and
verification boundaries; it is not yet a source-to-code compiler and makes no
conformance or release claim.

## Current crates

- `whitefoot-contract` owns deterministic, judgment-free data contracts: the
  exact specification and static-catalog identities, ordered source bundles,
  source identities and bundle-bound spans, explicit resource ceilings, and
  the first canonical source-binding framing. The catalog identity is a nominal
  byte identity only; it does not claim implementation completeness.
- `whitefoot-frontend` depends only on `whitefoot-contract`. A complete lexer
  result partitions the exact source bytes while classifying v0.8 lexical
  shapes. Source issues, resource exhaustion, and compiler failures remain
  distinct internal outcomes. Lexer-owned output allocation is fallible and
  explicitly bounded; this does not extend that claim backward to source-bundle
  construction. This is lossless frontend infrastructure, not a
  language-acceptance, semantic-capability, or catalog-facet claim.
- `whitefoot-verifier` depends only on `whitefoot-contract`. Its first real
  judgment verifies that an artifact is bound to the expected specification
  and exact ordered source bytes. Verified state has no public constructor.

Semantic, normative diagnostic, and command-line modules will be added only
when they contain real production behavior. The independent verifier remains a
separate crate and has no dependency on the frontend. Lowering becomes a
separate crate when a real backend exists, so the dependency graph can prevent
raw frontend state from reaching it.

No active code imports the retired implementations under `archive/`.

## Capability audit boundary

The implementation overlay lives at `../capabilities/whitefoot-rust/v0.8/`,
outside this Cargo workspace. It is audit metadata, not a compiler input. The
workspace policy rejects direct contiguous static-catalog facet-ID source
occurrences, scans every `.rs` file under `compiler/crates/`, forbids `#[path]`,
source-splicing `include!`, and local `macro_rules!`, rejects compile-time
environment macros and aliased data macros, limits conditional compilation to
canonical `#[cfg(test)]`, and permits only the exact specification and static
semantic-catalog lock files as source-level included data. Exact
package and dependency checks keep the overlay outside crate APIs. Crate doctest
targets are disabled and gate commands forbid explicit doctest execution;
active compiler Cargo configuration and every rustfmt or Clippy configuration
discoverable from source ancestry are forbidden. Every
workspace-resolving or compiling gate Cargo command uses isolated Cargo and
process homes, fresh target and temporary directories, a configuration-free
working directory, closed environment, exact toolchain, and explicit manifest;
Make variables cannot replace that runner. No evidence replay provider exists
yet, so the overlay cannot close any semantic obligation.

When an executable conformance adapter is added, its gate must compare identical
results with the overlay and derived report absent from the filesystem, working
directory, environment, arguments, and compile-time inputs, and under hostile
overlay mutations. The compiler, verifier, lowerer, runtime, diagnostics, and
adapter test selection must behave identically without that metadata.

`static-semantic-catalog-v0.8.sha256` mirrors the compiler-independent lock at
`../facets/v0.8/static-catalog.sha256`. The root audit rebuilds the canonical
catalog and requires the independently derived hash, hardcoded reviewed value,
root lock, and compiler mirror to agree. Compiler policy pins the mirror's exact
value, and the Rust unit test binds that mirror to the nominal `CatalogHash`
constant. Rust does not parse the catalog or capability overlay. The version-1
`WFSOURCE` codec remains exactly source plus specification identity; catalog,
tree, and proof identities belong to a future checked-artifact envelope after
their canonical schemas and real consumer exist.

The ordered multi-file bundle is a toolchain input envelope around v0.8's one
closed program, not a new language rule. Each file will contain complete
top-level items; combined declaration order is file order followed by item
order, while per-file UTF-8 and canonical formatting remain FORM-2 frontend
judgments. Invalid raw bytes therefore survive bundle construction.

## Gate

Run:

```sh
make -C compiler check
```

The gate verifies the exact Rust toolchain fingerprint, v0.8 bytes, and static
catalog identity, enforces
the closed three-package dependency graph and inherited lint policy, rejects
symlinks, build scripts, procedural macros, unapproved dependencies, and paths
outside this workspace, then checks formatting, builds, lints, tests, and
rustdoc. The reproducibility layer copies the complete compiler source to two
different paths and physical files, builds both release graphs, and compares
every Cargo-declared workspace artifact under collision-checked logical keys.

Absolute checkout paths otherwise enter stable-rustc library metadata and make
all six current release artifacts differ across source copies. The gate uses
stable `--remap-path-prefix` flags for both the copied source root and its target
directory. The two builds never share a checkout or target directory, and the
gate uses no nightly compiler option.

The toolchain alias is `1.91`; `toolchain-lock.json` closes that alias to exact
rustc and Cargo releases and commits, LLVM, rustfmt, Clippy, profile, and
component set. Dependency resolution and builds run with Cargo's locked,
offline controls (`--locked --offline`), so the exact toolchain must already be
installed. The current graph has no external package. Before one is admitted,
the clean reproducibility builds need a reviewed vendoring or deterministic
source-population strategy; an ordinary caller cache is insufficient. This is
an offline Cargo guarantee, not a claim that rustup can install a missing
toolchain without network access. `dependencies.json` binds its complete
external-package allowlist to the exact `Cargo.lock` bytes.
