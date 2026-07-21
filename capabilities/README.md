# Compiler capability audit

`whitefoot-rust/v0.9/*.json` is the active implementation-owned audit overlay
for the safe-Rust compiler. It lives outside the Cargo workspace, binds the
exact compiler-independent v0.9 semantic catalog, and is never a compiler input
or a source of language behavior. `whitefoot-rust/v0.8/` remains an immutable
version-bound snapshot; activating v0.9 does not rewrite historical evidence.

The initial overlay is intentionally empty. Source binding is real production
infrastructure, but it does not implement a semantic facet. A generic handler
or evidence reference lands only with its first coherent semantic tranche and
hostile review of the exact claim.

Each canonical fragment has only these fields:

- `format`, fixed to `whitefoot-capability-fragment-v1`;
- `implementation_id`, fixed to `whitefoot-rust`;
- `catalog_sha256`, the exact canonical static-catalog identity;
- `handlers`, where one generic handler owns explicit facets in exactly one
  required pipeline lane; and
- `evidence`, containing only receipt IDs and exact receipt digests.

The overlay never carries evidence class, exercised lanes, a verdict, or a
release decision. No evidence replay provider exists in the current
grammar-evidence tranche, so every receipt reference remains unresolved and
grants nothing. A later class-specific provider must independently replay and
validate its exact receipt before any referenced evidence can contribute to
closure.

The audit derives missing handlers, unexercised lanes, missing evidence classes,
and live discrepancy blockers. No status, completion, waiver, fallback,
expected-verdict, or release field exists. The static catalog and discrepancy
sidecar are rebuilt and revalidated on every repository audit.

The overlay remains outside the exact Cargo workspace and outside production
crate dependencies. The workspace policy scans every `.rs` file under
`compiler/crates/`, rejects direct contiguous facet-ID source occurrences,
forbids `#[path]`, source-splicing `include!`, and local `macro_rules!`, rejects
compile-time environment macros and aliased data macros, limits conditional
compilation to canonical `#[cfg(test)]`, and permits only the exact
specification and static semantic-catalog lock files as source-level included
data. The catalog lock carries identity only; it is not capability metadata and
cannot authorize a semantic result. Crate doctest
targets are disabled and gate commands forbid explicit doctest execution. Active
compiler Cargo configuration and every rustfmt or Clippy configuration
discoverable from source ancestry are forbidden. Every
workspace-resolving or compiling gate Cargo command uses isolated Cargo and
process homes, fresh target and temporary directories, a configuration-free
working directory, closed environment, exact toolchain, and explicit manifest;
Make variables cannot replace that runner. When an
executable adapter exists, its gate must run an identical-result differential
with the overlay and derived report absent from the filesystem, working
directory, environment, arguments, and compile-time inputs, plus hostile overlay
mutations. Neither the overlay nor its report may enter checking, verification,
lowering, runtime behavior, test selection, or diagnostics.
