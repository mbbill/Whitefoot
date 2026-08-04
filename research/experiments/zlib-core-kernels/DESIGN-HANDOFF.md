# Deferred compiler handoff: proof-selected DEFLATE kernels

Status: **DEFERRED RESEARCH RECORD — evidence and proposed pickup gates only**.

This document is not the Direction Outline, Current Plan, implementation
authorization, specification change, production fact channel, proof artifact,
or ratified addition to `docs/patterns.md`. It preserves the result of the zlib
core-kernel investigation so the work can be reconsidered if a selected project
needs the relevant proof and lowering capabilities.

## 1. The question this experiment answered

The original question was whether the obvious Whitefoot source shape can compete
with highly optimized zlib code. “Obvious” means that the writer expresses the
scalar semantics directly. The writer does not maintain distance-specific
vector cases, hand-unroll a decoder, rely on padding, or weaken a safety check.

The ordinary compiler result was negative:

- At match length 258 and distances 1–8, the canonical overlap-copy loop reached
  only 3.0%–9.0% of the pinned zlib-ng implementation.
- At distances 64 and above, where LLVM could treat the regions as separated,
  the same source reached 1.08x–1.28x zlib-ng.
- The prepared Huffman literal loop reached 313.100 million symbols/s with
  facts, versus 380.850 million symbols/s for the pinned zlib-ng projection.

The distance-64 crossover matters. It shows that safe buffers or Whitefoot's
source model are not an unavoidable performance tax. The missing capability is
a compiler strategy for short-period overlap.

The later compiler prototypes changed the answer:

- The periodic-copy lowering exceeded the zlib-ng wrapper median in all 14
  isolated cases. The smallest margins, 1.069x and 1.113x, should be read as
  parity rather than durable wins.
- The guarded six-symbol Huffman lowering reached 452.485 million symbols/s,
  1.309x the ordinary stage-0 lowering and 1.088x the zlib-ng projection by
  ratios of medians. The paired guarded/zlib-ng median ratio was 1.094 with a
  deterministic paired-bootstrap interval of [1.074, 1.105].

Both optimized experiments kept their Whitefoot source byte-identical.

The supported conclusion is therefore narrower than “the default lowering is
optimal”:

> For these two kernels, the obvious checked scalar source is a viable canonical
> writer shape when a finite machine-verified theorem licenses a
> strategy-changing compiler lowering.

This does not establish complete inflate performance, target portability, or a
general loop prover.

## 2. The evidence ladder

The experiment deliberately separated four questions.

| Step | What changed | What it established |
|---|---|---|
| Ordinary lowering | Nothing beyond the existing fact channels | The literal machine shape has large short-overlap and moderate Huffman gaps |
| Bounds-elision ceiling | All implicit kernel bounds checks were removed in an evidence snapshot | Check removal alone does not recover the missing algorithm |
| Manual strategy ceiling | Hand-written C materialized periods or decoded from a wide word | Competitive machine strategies exist on this host |
| Compiler-triggered prototype | Unchanged WF source selected the manual strategy after a closed check | A compiler consumer can bridge the isolated-kernel gap |

The bounds-elision ceiling is especially useful. At match length 258 it still
measured 2.325 versus 31.663 GiB/s for distance one, 0.893 versus 29.686 for
distance two, and 3.250 versus 27.407 for distance eight. Huffman improved to
383.257 million symbols/s but remained at 0.922x the 415.582 zlib-ng result.

In other words, `requires` and proof-elided bounds checks are necessary inputs,
but they do not invent periodic materialization or a multi-symbol word window.
The proof fact must authorize a different algorithmic lowering.

The raw artifacts for this ladder are in:

- `RESULTS.md` and the top-level JSON files;
- `bounds-elision-ceiling/`;
- `manual-lowering-ceiling/`;
- `PERIODIC-COMPILER-RESULTS.md`; and
- `GUARDED-COMPILER-RESULTS.md`.

## 3. The canonical source shapes

### 3.1 Periodic history expansion

`match_copy.wf` writes the sequential DEFLATE semantics:

```text
position starts at the initialized prefix end
for every output byte:
    value = out[position - distance]
    out[position] = value
    position += 1
```

The load happens before the store. Forward overlap is intentional: a later
iteration may read a byte produced by an earlier iteration. This recurrence is
the semantic information the compiler needs. The source does not spell a fill,
broadcast, shuffle, or wide copy.

The entry contract proves the complete API boundary used by the experiment:

- `1 <= distance <= min(seed_len, 32768)`;
- `3 <= match_len <= 258`;
- `match_len * repeats` does not overflow; and
- `seed_len + match_len * repeats <= len(out)` without overflow.

### 3.2 Checked table bitstream decoding

`huffman_literals.wf` writes one literal at a time:

```text
refill the scalar accumulator by bytes until nine bits are available
entry = fixed_lencode[hold & 511]
check that entry is a literal
checked-convert the entry value to u8
store one output byte
discard entry.used bits
advance output by one
```

The immutable 512-entry table is the exact pinned zlib-ng fixed literal table.
The source performs the kind and conversion checks before the output store and
state update. It does not choose a word width or batch size.

The entry contract establishes the representable worst-case input and output
capacity for this isolated all-literal API. A complete decoder should usually
treat truncated input or output shortage as ordinary data failure rather than
strengthening `requires` to exclude it. A local fast-region guard can select a
batch while retaining the checked scalar fallback.

## 4. What `requires` contributes—and what it does not

A `requires` block is an executed function-entry check. When it passes, its
predicate is a dominated fact. It is not a writer-authored assumption and it is
not, by itself, a proof of every loop access.

A production optimizer must keep two analyses independent:

1. Derive a named obligation from the function body and the exact sites that
   would consume it.
2. Normalize the checked requirement separately and prove that it supplies the
   obligation's premises.

This direction matters. Matching an expected contract first and then searching
for code to optimize can hide missing obligations. A body-derived obligation
instead explains which premise is absent and why an access or strategy remains
checked.

The temporary prototypes do not yet implement this production architecture.
They preserve the checked entry condition, but their closed recognizers are
feasibility triggers rather than canonical proof objects.

## 5. Periodic-copy theorem schema

Let:

- `S` be the initialized prefix length;
- `D` be the stable distance;
- `M` be the match length;
- `R` be the repeat count;
- `N = M * R`; and
- `E = S + N`.

A finite production verifier must establish:

1. `D > 0`.
2. `D <= S`.
3. `N` and `E` are computed without unsigned overflow.
4. `E <= len(out)`.
5. The nested loops execute exactly `N` successful byte iterations.
6. At iteration `k`, the destination is exactly `S + k`.
7. The generated-history read is exactly `out[S + k - D]`.
8. The only output mutation in the certified region is the corresponding store
   to `out[S + k]`.
9. `D`, `S`, `M`, and `R` remain stable.
10. No call or alias can mutate the output between the certified load and store.
11. The governing entry check or local guard dominates every credited site.

For `0 <= k < N`, define `p = S + k` and `q = p - D`.

- `D <= S` proves `q >= 0`.
- `D > 0` proves `q < p`.
- If `k < D`, then `q` lies in the initialized prefix.
- If `k >= D`, then `q = S + (k - D)` was written by an earlier iteration.
- `p < E <= len(out)` proves the destination is in range.
- The non-overflow premises make these ordinary integer equalities, not merely
  equalities modulo 2^64.

Induction therefore establishes:

```text
out[S + k] = out[S + k - D]
```

This theorem proves safe periodic expansion. It does not automatically prove a
particular SIMD implementation. Each lowering strategy must additionally prove
that its lanes reproduce the recurrence and that every wide load reads already
initialized storage.

Possible target strategies include:

- distance one: repeated-byte fill;
- short power-of-two periods: broadcast;
- other short periods: vector-table permutation or rotation;
- sufficiently separated regions: ordinary wide copy; and
- unsupported or short cases: the scalar recurrence.

Those categories belong to lowering, not writer guidance.

### 5.1 Current periodic prototype

The archived `democ.patch`:

- runs only after ordinary checker acceptance;
- identifies roles by checked AST structure rather than identifier spelling;
- recognizes one exact recurrence and loop topology;
- alpha-normalizes one exact contract;
- preserves the explicit source entry check; and
- emits a call to `__wf_periodic_copy_u8_repeated`.

The helper in `periodic_copy_helper.c` is hand-written ARM NEON C. Its
disassembly contains repeated-byte duplication, `tbl` permutations, vector
loads/stores, and a scalar fallback. The ordinary assembly retains a dependent
`ldrb`/`strb` loop and two in-loop bounds branches.

This is strong feasibility evidence, but not a complete proof. The external
helper's equivalence and memory behavior are manually argued. The prototype
does not issue a production proof report or independently represent the body
obligation and requirement. It is target-specific, and the repeated-identical-
match API amortizes work that a mixed token stream may not.

## 6. Guarded bit-window theorem schema

### 6.1 Table certificate

For the archived fixed-table experiment:

- lookup width `L = 9`;
- table length `2^L = 512`;
- mask `511`;
- word width `W = 64`; and
- batch width `B = 6`.

The verifier must bind a certificate to the exact immutable table and establish:

1. There are exactly 512 entries.
2. Lookup uses exactly the low nine bits.
3. Every accepted literal entry has `1 <= used <= 9`.
4. Every accepted literal value is representable as `u8`.
5. The kind and conversion checks happen before the store and state advance.
6. A nonliteral preserves the scalar program's failure behavior and output
   prefix.
7. Any table replacement, mutation, or new table version invalidates the fact.

A dynamic table cannot inherit this certificate from a name or type. It needs a
checked construction certificate bound to that exact table version.

### 6.2 State invariant

Let `u_i` be the bits consumed by successful literal `i` and define:

```text
C_k = sum(u_i for i in 0 .. k)
produced_k = k
byte_k = floor(C_k / 8)
offset_k = C_k mod 8
```

After `k` successful literals, the verifier must establish that:

- output `[0, k)` equals the scalar loop's output prefix;
- `byte_k` and `offset_k` identify the same next input bit as the scalar
  `(hold, bits, input_pos)` state;
- the available low accumulator bits equal the bitstream suffix starting at
  `C_k`;
- all earlier kind and conversion checks occurred in source order; and
- no cursor arithmetic wrapped.

The scalar-to-window equivalence is load-bearing. Bounds alone do not prove it.

### 6.3 Six-symbol window

The fast-region guard establishes:

```text
remaining_symbols >= 6
byte_cursor <= len(src)
len(src) - byte_cursor >= 8
produced <= len(out)
len(out) - produced >= 6
```

At batch entry, `offset <= 7`. Before the sixth lookup, at most five earlier
entries have consumed nine bits each, and the sixth lookup needs at most nine:

```text
offset + 5 * max_used + lookup_bits
<= 7 + 5 * 9 + 9
= 61
<= 64
```

One guarded 64-bit window therefore contains all bits required by six
source-ordered lookups. If lookup `j` is nonliteral, exactly the preceding `j`
literal outputs have been written, preserving the scalar output prefix and trap
order.

After six successes:

```text
sum = offset + used_0 + ... + used_5
next_byte = byte + floor(sum / 8)
next_offset = sum mod 8
next_produced = produced + 6
```

These equations re-establish the invariant.

The first production lowering should enter the original checked scalar code for
a false guard and for the tail. The prototype's specialized two-byte tail is
possible, but it adds another equivalence and over-read proof that should be
earned separately.

### 6.4 Current guarded prototype

The archived `democ.patch`:

- runs the ordinary checker first;
- computes an alpha-normalized whole-function AST digest;
- certifies the constant table and the `61 <= 64` batch bound;
- emits a guarded six-symbol LLVM loop directly;
- preserves source-order literal checks and stores; and
- enters a checked scalar tail.

Hostile tests cover guard pages, all tail remainders, table-mask drift,
zero-progress entries, out-of-range literal values, and nonliteral failures in
both bulk and tail paths. The ARM64 disassembly shows one `ldr x` word load,
six sequential mask/table/store sequences, and an `ldrh` scalar tail.

The digest is still only a trigger. The state recurrence, table-to-output
semantics, and lowering refinement are manually encoded rather than represented
as production proof objects. The path covers only fixed-table literals and says
nothing about length/distance dispatch, dynamic tables, block transitions, or a
complete decoder.

## 7. What a complete production proof means

For either transform, “complete” covers the entire authority chain:

1. Exact declaration, place, operation, type, effect, and control identities are
   checked.
2. The body independently produces a named obligation.
3. Checked premises are independently normalized and matched to it.
4. A finite theorem proves bounds, initialization, progress, arithmetic, and
   relevant failure order.
5. The selected lowering refines the scalar semantics for every admitted value.
6. Every removed implicit check names the proof record that discharged it.
7. Explicit checks remain present.
8. Facts-off compilation grants no optimization authority.
9. Report/no-report compilation emits identical program bytes.
10. Failure to prove a premise retains the checked scalar path and reports the
    first mechanical repair.
11. The fact records its producer, consumers, invalidators, exact site set,
    dependency cone, and table/version identity when applicable.
12. A hostile fact-channel review attacks every premise and invalidator before
    the consumer ships.

The two temporary prototypes do not cover all twelve items. They are
performance-and-feasibility evidence, not machine-verified production proofs.

## 8. Candidate writer-pattern guidance

The writer should be taught the semantic scalar pattern, not the machine
schedule. Otherwise the writer is merely recreating zlib's hand-maintained
optimization matrix and the default-shape claim has not been tested.

These are candidate experiment cards only. They have no P-number and do not
modify `PATTERNS.md`.

### Canonical periodic history expansion

Write one stable-distance forward byte recurrence with one monotone destination
cursor, exact unit increments, checker-proven exclusive output access, and the
weakest true initialized-history/capacity contract.

Do not write distance switches, repeated-byte fills, shuffle tables, explicit
SIMD, wide stores, or intentional writes into padding. The compiler selects
those strategies after proving the recurrence.

The first schema should reject mutable or zero distance, additional output
writes, intervening effectful calls, non-unit cursor steps, unmodeled early
exits, or missing non-overflow/history/capacity premises.

### Canonical guarded table bitstream decode

Write one checked scalar table decode with one canonical state update, an
immutable bounded table, kind and conversion checks before the store, and
ordinary checked behavior for incomplete input or unsupported entries.

Do not unroll six symbols, select a machine refill width, depend on padding, or
write a hand-maintained tail. The compiler chooses batch width and word loads
under a local guard.

The first schema should reject table mutation, mask/width mismatch, zero
progress, unrepresentable accepted values, reordered checks/stores, hidden
effects, unproved dynamic tables, or a word load not dominated by its local
availability guard.

### Context and diagnostics

Writer context should contain:

1. An always-present compact catalog index.
2. The complete task-relevant card for compression, decoding, scanning, or
   another recognized problem family.

Each card should have a copyable source skeleton, weakest contract, permitted
variations, invalidators, named proof fact, lowering family, near-miss
diagnostics, and positive/hostile examples. The same stable pattern identifier
should appear in teaching material, compiler diagnostics, proof reports, and
performance gates.

Unsafe shortcuts and writer-authored optimizer promises remain
unrepresentable. A safe program outside the catalog remains legal and receives
checked scalar lowering. For an owner-selected hot root, CI may separately
require the expected proof record so a performance regression cannot pass
silently.

This turns the optimizer problem into membership in a finite family:

```text
canonical body + checked premises
    -> bounded structural membership check
    -> pre-proved theorem instance
    -> strategy-selecting lowering
```

It does not require a general affine, Presburger, or loop theorem prover.

## 9. Production `wfc` readiness

Production implementation is premature in this record. The Direction Outline
owns current status and an `ACTIVE` Current Plan owns execution sequencing;
this experiment changes neither.

Before these consumers can exist, production `wfc` needs:

- complete facts-off body semantics and whole-unit lowering;
- `requires` parsing, checking, lowering, and independent obligation accounting;
- executable proof artifacts and per-site report schemas;
- relational recurrence and table-certificate fact storage;
- facts-off and report/no-report identity controls;
- ordinary lowering for these scalar kernels;
- a portable periodic lowering with a scalar fallback;
- guarded wide-load, table, and loop emission; and
- target-feature selection separated from the semantic theorem.

The current fixed-capacity structure-of-arrays compiler representation can
express these finite verifiers. No maps, recursive proof search, SMT, or general
abstract interpreter is inherently required. The larger risks are complete
provenance/accounting and portable strategy lowering.

Relative difficulty after the shared infrastructure exists:

| Work | Difficulty |
|---|---|
| Periodic recurrence verifier | Medium |
| Portable periodic lowering | High |
| Guarded bit-window verifier | High |
| Guarded bit-window lowering | Medium |
| General affine or loop prover | Out of scope |

A rough planning estimate for both production channels, including reports,
hostile tests, portable fallbacks, and target validation, was 8–12 or more
engineer-weeks with roughly 2x uncertainty after the prerequisites exist. This
is an estimate, not a schedule or authorization.

## 10. Pickup gates

Reopen only when an owner-selected milestone in `docs/current-plan.md` cites
this evidence. Re-audit the following historical technical gates against the
current compiler and active specification; they do not themselves create work
or prerequisites.

### Shared entry gates

1. Facts-off self-hosting and whole-unit lowering are complete.
2. Production `wfc` supports `requires` end to end.
3. Proof artifacts and reports exist.
4. Facts-off and report/no-report controls are established.
5. A proof record can name theorem, premises, sites, provenance, consumers,
   invalidators, dependency cone, and first failure.
6. Both scalar functions lower normally without either new fact.
7. This directory's sources, raw results, patches, IR, assembly, and hashes
   remain internally consistent.

### Per-pattern proof gates

1. Freeze one finite theorem and its admitted variations.
2. Derive the obligation from the body independently of source requirements.
3. Add one hostile negative for every premise and invalidator.
4. Test zero, maximum, and wrap arithmetic boundaries.
5. Compare production accounting with an independent bounded reference checker.
6. Run report-only before any fact can change lowering.
7. Obtain hostile fact-channel review before enabling authority.

### Lowering gates

1. Preserve the ordinary checked scalar fallback.
2. Prove output equivalence over every admitted parameter class.
3. Preserve explicit entry checks and required failure/output-prefix behavior.
4. Use protected pages to expose over-read and over-write.
5. Pin IR and assembly on each supported target family.
6. Measure code size, short inputs, tails, and guard-false paths.
7. Grant no production credit to an unreviewed external helper.
8. Keep target capability selection separate from the semantic proof.

### Performance gates

1. Reproduce the isolated results.
2. Add a mixed literal/match replay using frozen token, length, and distance
   distributions from real compressed corpora.
3. Measure ARM64 and every named deployment target, including x86-64 when
   required.
4. Compare facts-on, facts-off, and pinned reference bytes from identical source.
5. Report setup and capacity-check amortization separately.
6. Make no whole-decoder claim without a complete decoder measurement.
7. Ratify a writer pattern only if the mixed workload confirms that the
   canonical source plus compiler strategy is efficient.

## 11. Recommended implementation order

1. Implement periodic obligation production and verification in report-only
   mode.
2. Compare its proof record with an independent bounded checker.
3. Enable one portable periodic strategy with scalar fallback.
4. Add target-specialized strategies one at a time.
5. Implement the guarded bit-window theorem in report-only mode.
6. Enable the guarded batch while retaining the original scalar tail.
7. Optimize the tail only if measurement earns its separate proof.
8. Run the mixed token replay.
9. Seek a separate owner decision for production fact channels and candidate
   pattern cards.

Stop and retain checked scalar lowering if the schema starts requiring a
general solver, its admitted variations lose a finite boundary, a real decoder
needs an over-restrictive entry contract, target lowering cannot be validated
independently, failure order cannot be preserved, or mixed replay does not
retain the benefit.

## 12. Final deferred conclusion

The experiments refute the claim that literal lowering of the obvious scalar
shape is already optimal. They support a more useful design claim:

> The obvious checked scalar shape can be the optimal writer-facing shape even
> when it is not the optimal machine shape, provided the compiler recognizes a
> small closed theorem schema and selects a proved machine strategy.

For periodic copy, the finite recurrence proof appears easier than the portable
lowering. For guarded Huffman decode, the lowering is mechanically plausible,
while the complete state and failure-order proof is the larger task. Neither
requires or justifies a general prover.
