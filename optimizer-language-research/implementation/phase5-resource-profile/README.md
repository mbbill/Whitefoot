# Phase 5 resource-profile evidence

Status: EVIDENCE PROTOCOL CANDIDATE. This directory is non-authoritative. It
does not select a hard profile, activate v0.10, change a source verdict, or
authorize compiler implementation.

The owner-approved Phase 5 successor proposal fixes the resolver's fifteen
resource fields but deliberately supplies no numerical maxima. Decision 15
also requires those fields to be borrowed from one versioned invocation-wide
`ResourceProfile`, while the current frontend still accepts separate raw limit
structures. Therefore approving fifteen resolver values plus one mixed-tree
value would recreate the rejected standalone-limits design.

This evidence step has three jobs:

1. freeze the first complete invocation-profile schema, covering ingress
   through resolution and no later stage;
2. define the host, workload, work-accounting, layout, and peak-memory
   evidence required to select its hard maxima; and
3. produce an exact owner-review packet only after independent measurements
   and hostile review agree.

The initial audit returned **NO-GO for numerical approval**. Current
conformance, code-generation, and experiment sources are useful regression
floors but are not a realistic large-project demand set. Existing Rust test
limits, lexical-observer caps, `REPRESENTABLE`, and the proposal's abstract
77-case diagnostic evidence are expressly not production sizing evidence.

## Exact scope of profile version 1

Version 1 ends at complete lexical resolution. It contains no zero,
unbounded, reserved, or placeholder field for typing, CFG/ownership, effects,
artifacts, backends, external tools, publication, or optional facts. Those
views do not exist in version 1. Before implementing any such stage, a later
profile version must add its exact fields and separately approved hard maxima.

This is how one invocation-wide versioned profile remains honest while the
compiler is built in gated stages. It does not waive any later Decision 15
field.

All maxima are inclusive `u64` values. The canonical field order is defined
once by `schema.py`:

- ingress: sources, logical-path bytes, per-source bytes, total source bytes,
  and binding bytes;
- lexical: token bytes, tokens, lexemes, and charged scan work;
- syntax: classified tokens, production nodes, mixed elements, tree depth,
  simultaneous parser stack entries, list members, expected terminals,
  cumulative syntax work, and charged retained-tree bytes; and
- resolution: the exact fifteen R-04 fields in their approved order.

The resolution typed view names its last field `max_work`, as required by
R-04. The canonical parent encoding names it `max_resolution_work` so the
lexical, syntax, and resolution work counters cannot be confused.

Current parser, finalizer, and canonical-audit raw limit structures are
migration inputs, not the public profile schema. The complete operational
field meanings, checked derivations, migration table, actual-count receipts,
and closed failure order live only in `SCHEMA-SEMANTICS.md`; this overview does
not redefine them.

The generated grammar identity fixes the local-shape task ceiling. It is not a
caller-selectable program dimension. Parser tasks and open frames share one
simultaneous stack-entry budget. One cumulative syntax meter crosses terminal
classification, parser, finalizer, and canonical audit capabilities; each
action still belongs to a named stage and has one exact charge schedule.

`max_tree_bytes` is a stable charged-storage budget. Each retained record
family has an approved byte charge and a compile-time obligation that the
concrete safe-Rust size and alignment fit it. The charge is not `Vec`
capacity, allocator bookkeeping, or whatever `size_of` happens to report in
one build. Element ceilings still remain independently enforced.

## Identity and tightening

The approval packet will pin one canonical hard-profile byte string. Its exact
big-endian codec is implemented and mutation-tested in `schema.py`. It contains
the fixed 16-byte hard-profile domain, `u16` schema version, a zero 32-byte
parent slot, raw 32-byte digests for the exact successor specification,
schema/count semantics, work schedules, and storage/peak model, a `u16`-length-
prefixed closed graphic-ASCII host-class identifier, a `u16` field count, and
every ordered `(u16 field tag, u64 maximum)` pair. Its SHA-256 digest is the
hard-profile identity. `meaning.py` gives the exact domain-separated digest
construction: the semantics identity binds the complete schema descriptor,
`SCHEMA-SEMANTICS.md`, and approved proposal `7fc48cc3...`; the work identity
binds `WORK-SCHEDULE.md` and that same proposal; the storage identity binds
`STORAGE-MODEL.md`.

An invocation may supply every effective maximum at or below its hard value.
It may not omit, reorder, add, or loosen a field. The effective profile uses
its distinct fixed domain, places the selected hard-profile digest in the
parent slot, repeats every meaning-bearing digest and host identity, and
encodes all effective values in the same order. Its identity is the SHA-256
digest of those exact bytes. Stage views are
unforgeable borrows carrying both identities and the exact specification
identity. No CLI, environment variable, cache entry, artifact, or source file
can construct a view or loosen a value.

Zero is a valid tightening and admits only actual zero. An internally
inconsistent tightening is still a valid restrictive profile; it can only
cause an earlier resource failure and cannot change language meaning.

## Evidence required before values can be approved

### Demand workloads

The demand set must include all eligible active sources plus two named,
canonical, compiler-independent workload families:

- a compiler-shaped closed unit with declarations, deep and wide scopes,
  labels, regions, long common-prefix names, same-spelling candidates,
  forward function calls, and many lexical uses; and
- a production-software-shaped codec unit with structs, enums, contracts,
  functions, matches, loops, calls, constants, and requires blocks.

Before numerical approval, both independent routes must show that every
success-demand bundle passes FN-8, complete declaration inventory, and complete
lexical resolution with no unresolved or conflicting name. After production
resolution exists, the same bytes must reach `Complete(ResolutionCompleteUnit)`
before the implementation confirmation gate can pass. Later typing may still
reject a resource-sizing skeleton, but that limitation must be explicit.
Generated scale points must be named future-capacity scenarios, not unexplained
percentage headroom.

The hostile set must independently exercise flat width, nesting depth, maximum
dual-entry struct declarations, long equal prefixes, repeated invisible
origins, diagnostic paths, sibling scopes, unrelated owner partitions, labels,
and an early FN-8 defect followed by the maximum scanned tail. It must also
cover large low-topology strings and alternating one-byte lexical partitions.

`workloads.py` currently supplies only deterministic, canonical smoke seeds
for the two named families. The independent source route has established that
the scale-1, scale-2, and scale-17 seeds pass FN-8, complete declaration
inventory, and complete lexical resolution. The independent analytic route
also selects `Complete` from its separate construction relation. These seeds
are not yet the complete demand or hostile set and do not select a value.

`evidence_manifest.py` is the workload producer's neutral manifest codec. Its
canonical v1 bytes contain only the closed family, scale units, exact generator
revision, construction dimensions, and ordered logical-path/length/source-hash
records. They contain no expected role, count, work, diagnostic, or grammar
result. Each measurement route must decode those bytes independently and may
not import the producer codec or the other route.

### Independent counters

Two evidence routes may share only the exact source bytes, specification
bytes, neutral profile schema, and generator parameter manifest. They may not
share an extracted grammar, role table, count formula, expected count, work
implementation, or diagnostic selector:

1. a source-to-role route that independently lexes/parses canonical source and
   projects the complete approved role and scope matrices; and
2. a separately represented analytic or relational route that derives the
   same counts and exact work from workload construction parameters.

They must agree on every profile field, every derived count, the selected
diagnostic, and exact charged work. The proposal's existing role-stream models
remain valuable differential evidence but cannot replace either complete
source-to-role route.

The current non-authoritative foundation implements both routes in
`source-route/` and `analytic-route/`. `cross_route_agreement.py` invokes the
workload producer and both routes as separate processes. For both families at
scale 1, 2, and 17, it requires exact agreement on all 33 field states, the 27
currently available field values, every analytic derived count reconstructed
from independently reported source-route facts, the `Complete` result, the
same neutral-manifest identity, and the same ordered source identities.

That gate deliberately reports `trace-incomplete`. Both routes independently
withhold fields 9, 14 through 17, and 33: lexical scan work, parser stack,
selected list members, expected terminals, cumulative syntax work, and
resolution work. Aggregate formulas are forbidden substitutes. A separate
action-schedule replay must close those six gaps before the routes can satisfy
the complete-counter requirement above.

### Work and scaling

Every work unit must name one constant-bounded action. A unit may not hide a
loop, allocation, clone, formatting pass, hash, parent walk, range scan, or
dependency call. Scaling evidence must double declaration/use width, spelling
prefix length, scope depth, diagnostic-origin/path volume, unrelated owners,
and FN-8 tail size. It must detect insertion shifts, per-use scope walks,
unrelated-owner scans, obsolete event sorts, repeated node-path walks, or more
than the approved two diagnostic-origin scans.

The evidence must establish both sides of each selected work maximum:

```text
work required by every named demand workload
    <= selected maximum
    <= work completed inside the supported service envelope
```

### Layout and peak memory

The approval packet fixes a 64-bit compiler-host class, Rust toolchain and
allocator assumptions, minimum available memory, and supervised RSS/service
envelope. Every count must fit `usize`, `isize::MAX`, checked byte products,
and `Layout::array` on every supported compiler host.

Because resolver records do not exist yet, the packet must approve a
conservative byte charge and alignment ceiling for each future storage. The
implementation stops if any concrete record exceeds its charge. The report
must evaluate all twelve lifetime peaks in `STORAGE-MODEL.md`, not only success
and diagnostic end states, and select the global maximum. Until finite
allocator slack is pinned, requested-byte accounting and actual supervised RSS
remain separate claims.

`layout-witness/` is the current non-authoritative layout foundation. It checks
the 25 candidate record charges with checked `u64`, `usize`, `isize`, and
`Layout::array` arithmetic; measures the public active frontend layouts it can
name; encodes all twelve peak categories as seventeen closed ledger rows; and
pins the candidate proposal, specification, storage model, executable, host,
and toolchain identities. It explicitly leaves private frontend layouts,
future resolver records, allocator slack, populated peak rows, and externally
supervised RSS unproved. Those omissions prevent numerical approval.

### Boundaries and failure authority

Evidence covers exact and one-over behavior for every field and derived
quantity, checked overflow, host conversion, layout failure, zero-capacity
allocation dormancy, every storage-order allocation injection, competing
failures, deterministic repetition, and failure atomicity.

A bound hit is only a resource failure. It never becomes a language rejection,
`Unsupported`, truncated diagnostic, skipped rule, weakened coverage, or
changed protected verdict.

## Mandatory cross-field receipts

The exact frontend and resolver equations, including per-role lookup insertion
multiplicity, split source/PRE-1 diagnostic origins, exact path sums, spelling
components, and checked derived elements, live only in
`SCHEMA-SEMANTICS.md`. Work may stop a hostile componentwise combination before
other maxima; the packet reports that effective envelope rather than claiming
that every simultaneous maximum is admitted.

## Approval boundary

The eventual owner-review packet must pin exact profile bytes and hash,
successor proposal and candidate hashes, host identity, workload and report
hashes, record charges, peak ledgers, work/service ledger, reproducible
commands, claim limits, and independent hostile GO.

Owner approval must explicitly select the exact v1 field and count semantics,
merged parser-stack budget, cumulative syntax meter, work schedules,
storage/stride and peak model, codec and identity construction, raw-limit and
failure migration, host class, and numerical values. It is not approval of
numbers detached from their meaning. Guarded v0.10 installation and successor-
frontend reproduction would still be next. Resolver implementation would
remain a separate authorization after those gates pass.
