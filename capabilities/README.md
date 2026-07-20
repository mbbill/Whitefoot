# Compiler capability audit

`whitefoot-rust/v0.8/*.json` is the implementation-owned audit overlay for the
safe-Rust compiler. It lives outside the Cargo workspace, binds the exact
compiler-independent semantic catalog, and is never a compiler input or a
source of language behavior.

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
release decision. No evidence replay provider exists in the foundation phase,
so every receipt reference remains unresolved and grants nothing. A later
class-specific provider must independently replay and validate its exact
receipt before any referenced evidence can contribute to closure.

The audit derives missing handlers, unexercised lanes, missing evidence classes,
and live discrepancy blockers. No status, completion, waiver, fallback,
expected-verdict, or release field exists. The static catalog and discrepancy
sidecar are rebuilt and revalidated on every repository audit.

The overlay remains outside the exact Cargo workspace and outside production
crate dependencies. Compile-time textual `include*` and `#[path]` inputs cannot
escape `compiler/`, and production Rust may not contain facet IDs. When an
executable adapter exists, its gate must run an identical-result differential
with the overlay and derived report absent from the filesystem, working
directory, environment, arguments, and compile-time inputs, plus hostile overlay
mutations. Neither the overlay nor its report may enter checking, verification,
lowering, runtime behavior, test selection, or diagnostics.
