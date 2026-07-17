# Candidate C Hashbrown Calibration

Date: 2026-07-15

Status: Stage 1 complete; mandatory Gate 1 stop. This is a bounded structural
paper audit, not a safety proof, implementation, code-shape result, benchmark,
or production selection.

Gate 1 disposition: `C-REVISE`.

## 1. Source pin and audit boundary

- Project: official `rust-lang/hashbrown` repository.
- Tag: `v0.17.1`.
- Commit: `c62a63a61b7caf2de8f9ecb7b06a66b0ab6bdf3d`.
- `src/raw.rs` SHA-256:
  `0c8ad353ba95817e72b0a8fea48fa2599099ea3def374f254ab6402a9c468d22`.
- `src/map.rs` SHA-256:
  `b79497ce537ffc5ed4f8f3399434b9216c01e7927fdc434fee190e9e9ce2abb0`.
- `src/control/bitmask.rs` SHA-256:
  `5aaa52db03d2d17fe85fed67aa2bdc4ea376a0bf0e18010576e794b502d44689`.
- `src/control/mod.rs` SHA-256:
  `83fede19e9c5a26fd2c7372e6cf92547d32ade4cd69fde8a0eaaeb1be6ddc2ba`.
- `src/control/tag.rs` SHA-256:
  `691dc7aa8d720e6df59a82ac11d66c72802ffffc684620c8ff29523352aefd13`.
- `Cargo.toml` SHA-256:
  `3e1f9929f381f46f969ea49faab281dc3c6ee08b24ef41f3ac8e2eec209ca078`.

The frozen operations are exactly lookup, vacant insertion, replacement,
removal, and rehash. The audit excludes other traits and APIs, custom allocator
coverage, serialization, parallel features, compatibility behavior, the rest
of the repository, execution, compilation, generated code, and measurement.
The matrix contains 18 outcome/transition rows, below the 25-row ceiling.

## 2. Frozen reference representation

`RawTableInner` stores a bucket mask, a control pointer, `growth_left`, and an
item count. One allocation contains payload places followed by control bytes
and an additional replicated group-width control tail. A control byte is FULL
with a seven-bit hash tag, EMPTY, or DELETED. Payload places are not initialized
unless occupied. Group loads and tag matches avoid touching most payloads.

This representation is directly within C-4's intended sparse-family boundary:
representation-selected control metadata is charged only to sparse tables.
Nothing in the inspected slice requires fixed, full, or dense representations
to acquire control bytes, tombstones, sparse branches, or a Hashbrown identity.

## 3. Five-operation result

### Lookup

The reference probes control groups, invokes equality only for matching FULL
slots, and stops on EMPTY. C-4 is the correct storage family, but the frozen
candidate lacks the exact same-root, same-version rule by which a control byte
authorizes its payload place. C-7 result provenance and C-11 group-match,
occupancy, and probe-termination facts are also unenumerated. C0-10 has no exact
portable group/SIMD row with a scalar fallback. The route has no identified
structural tax, but it cannot receive paper authority.

### Vacant insertion

The reference obtains an EMPTY or DELETED index, updates control and counters,
and writes the key-value owner pair exactly once. EMPTY consumes growth budget;
DELETED does not. Entry insertion then returns a mutable value reference rooted
in the table, not in the temporary entry object. The frozen C-4/C-6/C-7/C-11
composition does not specify the versioned vacant token, commit sequence,
abandonment cleanup, result provenance, or fact synchronization. A growth path
also needs an exact C0-7 allocation/failure row. No zero fill or extra payload
copy is inherent in the observed representation.

### Replacement

Ordinary duplicate insertion keeps the stored key, consumes the offered key,
moves the new value into the slot, and returns the old value. The callback-based
replace path can instead temporarily take the whole owner and either restore a
replacement or leave the slot vacant. Both are finite protocols, but C-4/C-6
does not enumerate their owner disposition and abort behavior, and C-11 does
not distinguish occupancy facts that survive from value and borrow facts that
must invalidate.

### Removal

Removal moves the key-value owner out exactly once and marks the slot EMPTY or
DELETED. A group-local emptiness test selects DELETED when EMPTY would break a
probe chain; otherwise EMPTY restores growth capacity. The representation needs
no extra tombstone bit or whole-table scan beyond the reference algorithm. The
missing pieces are the exact C-4/C-6 transition, C-7 occupied-entry source map,
and C-11 reachability and invalidation rules.

### Rehash

Reserve chooses either in-place tombstone cleanup or resize. Resize allocates
one new root, visits FULL slots, raw-relocates each live payload once, commits by
swapping roots, and frees old raw storage without dropping moved payloads. Its
precommit guard frees the incomplete new raw table without treating copied bytes
as owners when hashing aborts.

In-place rehash temporarily changes all FULL controls to DELETED while their
payload owners remain live. It then restores same-group entries, moves an owner
to an EMPTY slot, or swaps through a DELETED chain. On callback abort, a guard
destroys each still-unprocessed DELETED-live owner exactly once while preserving
already restored FULL entries. This is a finite transition protocol, but normal
DELETED means vacant, so the phase-specific live interpretation must be confined
to an exact C-6 state machine. The frozen candidate does not contain that state
machine, its partial-progress cleanup, its re-rooting rules, or its fact schemas.

## 4. Structural account

| Dimension | Reference requirement | Candidate C finding |
|---|---|---|
| Initialization/zeroing | Vacant payload remains uninitialized. | No forced initialization or zeroing identified. |
| Payload copy/move | Insert writes once; replacement exchanges one value; removal reads one owner; resize relocates each live payload once; in-place rehash uses branch-specific move or swap. | No forced extra payload traffic identified; exact transition authority is absent. |
| Allocation | Lookup, no-grow insertion, replacement, removal, and in-place rehash allocate nothing; resize allocates one new root. | C0-7 exact row and failure contract are absent; no extra allocation is inherent. |
| Metadata | Sparse control bytes, counters, and replicated control tail are part of the reference representation. | C-4 can charge them only to the selected sparse representation; no unrelated-shape metadata tax identified. |
| Indirection | Control and payload use calculated places inside one allocation. | No forced extra indirection identified. |
| Branch/check | Reference probes, tag tests, equality, tombstone choice, and rehash branches remain. | No forced extra check identified; compiler code shape is not established. |
| Scan | Probe scans groups; in-place rehash scans control; resize visits occupied slots. | No additional whole-table scan identified. |
| Atomic/fence | None in the frozen single-table slice. | None required. |
| Machine event | Group/SIMD operations and resize allocation/release. | Exact C0-10 and C0-7 rows are absent. |
| Code size | Specialized monomorphic paths exist in the reference. | Not evaluated; no generated-code evidence. |
| Asymptotic behavior | Expected probe behavior and linear rehash are preserved by the paper shape. | No asymptotic tax identified; no performance claim follows. |

## 5. Safety and fact obligations

1. FULL must authorize exactly its corresponding live payload under one root
   relation and version; EMPTY and ordinary DELETED must not.
2. Group matches are only candidates. Equality establishes a key hit, and EMPTY
   terminates only the current valid probe relation.
3. Control bytes, payload liveness, counters, and access facts must change as
   one logical transition even when reference writes are physically sequenced.
4. A vacant or occupied entry token cannot escape its root/version or survive a
   mutation, relocation, removal, or rehash.
5. Insertion, replacement, removal, failure, and abandonment give every offered,
   stored, displaced, and returned owner exactly one disposition.
6. Resize preserves old owners until commit, re-roots all survivors at commit,
   and releases old storage without destroying moved owners.
7. In-place rehash's DELETED-live phase is visible only inside its fixed
   protocol. Normal sparse operations may never interpret DELETED as live.
8. Abort cleanup destroys each unprocessed live place exactly once and leaves
   each documented survivor valid; no transition fact escapes.
9. C-11/C0-5 must enumerate occupancy, vacancy, group-match, probe-termination,
   rehash-phase, and final-state facts with exact invalidators.
10. Fixed, full, dense, ring, and unrelated sparse representations acquire no
    Hashbrown-specific state, branch, row, or project-name recognition.

## 6. Proposed bounded repairs, not applied

The evidence supports repairing existing categories, not admitting a new named
container family:

1. C-4: enumerate a finite selected sparse relation between control state and
   corresponding payload liveness, including phase-local interpretations.
2. C-4/C-6: enumerate insertion, replacement, removal, resize, and in-place
   rehash transitions with commit, abandonment, partial progress, and cleanup.
3. C-4/C-7: enumerate same-root versioned vacant, occupied, bucket, and returned
   reference provenance and invalidators.
4. C-11: enumerate sparse vacancy, occupancy, group-match, probe, relocation,
   and rehash fact schemas and their transfer/invalidation rules.
5. C0-10: add an exact portable group-operation row with target SIMD lowering
   and a behaviorally identical scalar fallback.
6. C0-7: add the exact checked growth allocation, layout, failure, root-transfer,
   and release row selected by the sparse family.

The first four items may form one bounded sparse-family semantic package rather
than four public features. The last two are exact machine leaves, not container
families. This single project may expose a gap but cannot satisfy the frozen
two-independent-demand admission rule for a new family.

## 7. Gate 1 reasoning

Candidate C is not falsified by this slice. The reference mechanism fits C-4's
already frozen sparse category, uses a finite set of local state transitions,
and shows no need for project, API, path, or container identity. No unavoidable
initialization, zeroing, extra copy, extra allocation, unrelated metadata,
extra indirection, additional whole-table scan, synchronization event, or
asymptotic tax was identified.

The frozen candidate also cannot pass. Every operation directly exercises one
or more of the six definitions that Stage 0 deliberately left absent. The
matrix therefore records explicit `COMPOSITION-GAP` and `C0-GAP` outcomes. It
contains no `UNKNOWN`, but absence of uncertainty does not supply missing
authority. There is no generated-code or measured-performance evidence.

The exact Gate 1 result is therefore `C-REVISE`: retain Candidate C as the first
validation hypothesis, repair the existing reusable sparse-family semantics on
paper before another audit, retain B as the later compression challenge, and
retain A as the generality fallback. This report does not apply any repair.

Stage 2 is not authorized. Work stops here pending an owner decision.
