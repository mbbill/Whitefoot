# Compiler Foundation Audit Receipt — 2026-07-21

Status: COMPLETE READ-ONLY RECEIPT. This records Decision 18's foundation
inspection at the lexical-observer checkpoint. It is evidence for the D25
handoff, not language, artifact, capability, or release authority.

## Snapshot identity and scope

The audit inspected the repository before any foundation migration:

- Git `HEAD`: `c1975d5d30f29a95647ff21d5e1895cad40adf0d`
- Git `compiler/` tree: `bead7377dc6b7c880d630d873143da79fadf5852`
- raw `git status --porcelain=v1 -z --untracked-files=all` SHA-256:
  `2aa3438ea56678b36f81a863b8e6e69aa0edcf81d274660655c855894b36e2d4`
  (eight records)

The content manifest enumerated bytewise-sorted paths from
`git ls-files -co --exclude-standard -z`. Its SHA-256 domain starts with
`whitefoot-content-manifest-v1` followed by a NUL. Each entry contributes a
big-endian 64-bit path length, exact path bytes, one `F` or `L` kind byte, a
big-endian 64-bit content or link-target length, and the exact bytes.

| Scope | Files | Bytes | Manifest SHA-256 |
|---|---:|---:|---|
| repository | 1,762 | 213,362,659 | `9f601d194f001b2a281037c5bb75cd8207d300ef0c4ef381f05f3e395434aa5d` |
| `compiler/` | 39 | 255,977 | `8e6fd6cd41347db5c010d6c3df4ea1f01c9d8c71f06d0013ad5bfa0d1ee080fc` |
| `whitefoot-contract` | 7 | 55,791 | `ea15d8402b174f486f1bc2a1d473ec54a7c7f5455b16cdbab01e93be0b470c00` |
| `whitefoot-frontend` | 9 | 58,402 | `ab3c05d993d62c126cd2daa79c5cf3d13f2aa889be3ede3592ed8dd72e0339ad` |
| `whitefoot-lexical-observer` | 5 | 27,925 | `1bd35522fca3e9777f43f8120b5d49ad7e7a4080cc53204718b71f74eb7cee34` |
| `whitefoot-verifier` | 3 | 12,709 | `e9c7a59d0bda2b5a6fb5467683f02f328eb1b0761cc1b6615a55420686374c77` |
| `compiler/tools` | 6 | 91,154 | `bff92805b62f4f174b3a5b392853faf842fabbdc8e0af449c32c879f0b2b7448` |
| root `tools/` | 27 | 555,517 | `efa31568d5d87921cf5d4ee75140bce3bc687d61d9e87ecded140d104432297a` |

At this snapshot `compiler/` had no tracked or untracked edit. The eight dirty
records were the two architecture/log modifications and six architecture
dossier files. The lexical observer was already preserved in commit `c1975d5`.
No user implementation edit overlapped the audited migration paths.

## Workspace and dependency topology

The safe-Rust workspace had four members and no third-party Rust dependency:

| Crate | Kind | Direct dependencies | Actual responsibility |
|---|---|---|---|
| `whitefoot-contract` | library | none | nominal digests, source carriers, spans, explicit limits, and version-1 source-binding bytes; no language judgment |
| `whitefoot-frontend` | library | `whitefoot-contract` | lossless shape lexer only; no classifier, parser, canonical tree, or language verdict |
| `whitefoot-verifier` | library | `whitefoot-contract` | exact source/spec candidate-binding equality audit only; no semantic artifact verification |
| `whitefoot-lexical-observer` | binary evidence tool | contract and frontend | bounded request/response observation for the lexer differential; no production API or authority |

The workspace forbids `unsafe`, pins Rust 1.91.1, keeps overflow checks enabled,
and denies broad Clippy and Rust warnings. `dependencies.json` records an empty
dependency set. Cargo/source-policy, exact toolchain, closed-environment,
reproducibility, specification-hash, catalog-hash, and workspace-topology gates
are real common infrastructure and should be retained.

## Module and public-boundary inventory

### `whitefoot-contract`

- `digest.rs` provides nominal `SpecHash`, `CatalogHash`, and `Sha256Digest`
  identities. Equality proves identity only, not validity or completeness.
- `source.rs` provides `SourceId`, `ByteOffset`, `SourceSpan`, `LogicalPath`,
  `SourceInput`, `SourceFile`, `SourceBundle`, and explicit source limits and
  errors. Bundle order currently binds transport/source identity only. It does
  not define the normative compilation unit while A-10 is open.
- `binding.rs` provides `BoundSource`, `SourceBinding`, and the exact version-1
  `WFSOURCE` codec. The wire format binds ordered source bytes and specification
  identity only. It is not a checked semantic artifact.

The crate-level documentation overstates that the later frontend can issue the
exact FORM-2 diagnostic. The current lexer deliberately does not own a
normative diagnostic ABI or full source-form judgment. That claim must be
narrowed without changing behavior.

### `whitefoot-frontend`

- `token.rs` defines source-borrowing, bundle-local shape tokens and the token
  tape. They are not portable IDs or canonical syntax nodes.
- `scanner.rs` implements the two-pass lossless scanner and `lex_v0_8`.
- `outcome.rs` keeps complete, source-issue, resource-failure, and invariant-
  failure outcomes distinct.
- focused tests cover lexical behavior, hostile seams, and resource ceilings.

The implementation owns only lexical shape. Its crate name is broader than its
responsibility and invites parser accumulation. Rename it to `whitefoot-lexer`;
do not change the scanner's authority and do not create a syntax crate until
the grammar entrance gate closes.

### `whitefoot-verifier`

- `source_binding.rs` compares a candidate's exact specification hash, source
  count, ordered logical paths, and raw bytes with invocation inputs before it
  constructs `VerifiedSourceBinding`.

That check is complete for source-binding equality, but neither the crate nor
the result verifies a semantic artifact. Rename the crate to
`whitefoot-source-audit`, retain the narrow equality capability, and ensure all
documentation says exactly what was checked. Artifact replay belongs to the
future semantic component, not this crate.

### `whitefoot-lexical-observer`

- `protocol.rs` decodes a closed, capped request and encodes the existing lexer
  outcome families.
- `projection.rs` projects the borrowed token tape under one exact response
  ceiling.
- `main.rs` reads one request and writes one transient response.

This binary and its Python model/runner are evidence only. They must remain
outside the production dependency direction and cannot issue a verdict,
receipt, capability, artifact, or facet-closure claim.

## Reachable allocation audit

### Paths already explicit and controlled

- The lossless scanner validates and counts without allocation, computes exact
  counts, uses fallible reservation before emission, checks second-pass count
  invariants before each push, and publishes only a complete tape.
- Observer ingress and response byte ceilings are explicit; its response
  projection computes a checked exact size before one fallible reservation.
- Hash values and source/span coordinates are fixed-size values.

These claims begin after the caller has constructed the owned source bundle.
They cannot hide failures in that earlier construction.

### Hidden infallible owned construction to repair

- `LogicalPath::parse(impl Into<Box<str>>)` can allocate before returning a
  validation result.
- `SourceInput::new(impl Into<Box<str>>, Vec<u8>)` can hide path allocation and
  exposes no construction-failure family.
- `SourceBundle::with_limits` performs owned `Vec`, `BTreeMap`, `Arc`, boxed
  slice, and clone/conversion allocations on an API that reports source limits
  but not allocation failure.
- `BoundSource::new(impl Into<Arc<[u8]>>)` and
  `SourceBinding::{new,from_bundle}` contain conversion, collection, and clone
  paths that are not audibly fallible.
- `SourceBinding::encode_canonical` computes a bounded size but uses
  infallible `Vec::with_capacity`; `decode_canonical` grows a vector and creates
  owned path/source storage without allocation-failure results.
- Source-binding mismatch errors clone logical paths even though comparison can
  report stable source-local information without infallible owned copies.
- Derived `Clone` implementations on owned aggregate carriers expose additional
  unreported allocations.

The migration should use explicit fallible owned constructors, exact
prevalidation/precalculation, and `try_reserve` before any result-visible
mutation. Allocation failure stays distinct from malformed input, a language
rejection, and an invariant failure. Version-1 `WFSOURCE` bytes must remain
unchanged.

### Tool-only or future paths

- The observer ultimately writes a transient response to stdout. It is
  bounded, but it is not transactional durable publication and must not be
  described as such.
- Python evidence runners and Cargo gates allocate through their host runtimes.
  They are bounded test/tool processes, not production compiler data paths.
- There is no current parser, canonical tree, semantic kernel, artifact codec,
  optimizer overlay consumer, backend, object writer, linker orchestrator, or
  durable output publication path to audit. Their absence must not be filled by
  placeholders.

## Publication and failure atomicity

- Source and binding constructors currently return only complete owned values,
  but their allocation failures can abort the process; that boundary requires
  repair before a controlled-failure claim.
- Lexer success publishes a complete immutable token tape; every reported
  lexical/resource/invariant failure publishes no tape.
- Source-binding audit constructs its capability only after all equality checks
  succeed.
- The observer writes only transient evidence output. It publishes no compiler
  artifact and grants no authority.
- No current path publishes semantic, lowering, object, or executable output.

## Disposition

| Surface | Disposition | Handoff action |
|---|---|---|
| toolchain, policy, and reproducibility gates | keep | retain and update exact topology pins after reviewed renames |
| nominal digest types | keep | preserve identity-domain separation |
| source carriers and version-1 source binding | keep and refactor | make owned allocation explicit and fallible; preserve exact wire bytes; keep bundle ordering nonnormative until A-10 |
| lossless scanner and token tape | keep and narrow | rename crate to `whitefoot-lexer`; preserve shape-only responsibility |
| lexical observer and independent Python model | keep as evidence | no production dependency or authority |
| source-binding equality audit | keep and narrow | rename crate to `whitefoot-source-audit`; correct claims; do not grow it into artifact replay |
| catalog, capability overlay, and discrepancy tools | keep outside dispatch | identities and obligation evidence only; no handler selection |
| parser, tree, node-path, and canonical syntax schema | defer | wait for terminal partition, node/location, A-10, and pre-tree diagnostic gates |
| semantic, CFG, proof, and artifact schema | defer | wait for applicable language decisions and concrete contract approval |
| target, ABI, lowering, and publication | defer | wait for target/resource profiles and accepted-compilation boundary |
| archived compilers | keep inert | no active build, import, semantic, or release dependency |
| deletion set | none | no audited component requires deletion |

## Approved migration order and stop conditions

D25 authorizes this preparation in cohesive tranches:

1. correct responsibility and authority claims;
2. make owned source construction fallible;
3. make binding codec and source-audit construction fallible while preserving
   exact version-1 bytes;
4. rename only the two overbroad crates and update topology/policy/evidence
   references;
5. rerun the lexical differential and all repository gates.

Stop and return to the owner if the migration would change version-1 wire
bytes, a numbered specification, a protected expected result, the static
catalog, a capability claim, A-10 behavior, lexer outcome semantics, or an
entrance-gated production schema. The audit authorizes no such change.

## Receipt conclusion

The current foundation is small, testable, and mostly architecture-independent.
It should not be deleted. Its two material defects are hidden infallible owned
allocation and names/documentation that overstate responsibility. Repairing
those defects and preserving the observer as evidence leaves a suitable base
for the first new-plan implementation tranche: the standalone grammar-change
verifier and non-authoritative successor-specification evidence preparation.
