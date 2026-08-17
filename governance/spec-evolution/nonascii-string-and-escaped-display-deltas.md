# Two deferred-item deltas for the v0.31 candidate

Status: DELTA TEXT FOR LEAD INTEGRATION, 2026-08-17, batch 0070 (W5). These are
proposals, not spec bytes. `spec/kernel-spec.md` has one writer; the lead folds
these into the single v0.31 candidate, and the owner approves the exact bytes.

Two DEFERRED markers in v0.30 are lifted here:

1. **D1 — non-ASCII diagnostic text** (`[FORM-5]`, active spec line 90:
   "non-ASCII diagnostic text is DEFERRED").
2. **D2 — escaped host-string display** (`[HOST-2]`, active spec line 2278:
   "Escaped, quoted, and lossy display of a host string are a DEFERRED separate
   presentation family with their own delta [META-5]").

Delete this file once the v0.31 candidate carries both, or once the owner
declines one.

---

# D1 — non-ASCII diagnostic text

## What blocks it

The restriction is grammatical, not implementational: `[FORM-5]` fixes the
STRING interior to ASCII-printable bytes, and `[DIAG-1]`'s raw-scanning
paragraph makes a valid non-ASCII scalar inside a STRING a `FORM-5` rejection.
`compiler/src/lexer/scanner.rs::string` implements exactly that. Lifting it
without moving spec bytes is impossible.

## Design: an encoding-level admission with no Unicode tables

The trap in this item is the normalization question — an earlier note
(`research/notes/batch1-spec-deltas.md`, item 5) framed the choice as
"ASCII-only, or card UTF-8/NFC with a pinned Unicode version". Pinning a Unicode
version is the wrong price: NFC needs decomposition and combining-class tables,
and `compiler/Cargo.toml` has zero dependencies by deliberate policy.

The sound lift needs no tables at all, because a diagnostic STRING is carried,
never compared:

- Admit any scalar whose complete well-formed UTF-8 encoding appears literally,
  restricted by **codepoint range only**: `U+00A0..U+10FFFF`. Surrogates are not
  scalar values, so they are already excluded; overlong encodings are already
  excluded by well-formedness. Both are pure range/shape checks the scanner
  already performs.
- Exclude `U+0000..U+001F`, `U+007F`, and `U+0080..U+009F` — C0 controls, DEL,
  and C1 controls. This keeps the emitted `[DIAG-3]` JSON record valid without
  adding a fourth escape (§"Derived material" below), and keeps every admitted
  non-ASCII scalar at or above `U+00A0`, which is a comparison, not a lookup.
- State explicitly that no normalization, case folding, or equivalence
  judgment exists. Two STRINGs differing only by normalization are different
  STRINGs. This is the same stance `[HOST-1]` already takes for host strings:
  lossless carriage, not text semantics. It is what makes the Unicode-version
  pin unnecessary — nothing in the language ever asks whether two diagnostic
  strings mean the same thing.
- Keep "each character has exactly one spelling": for an admitted non-ASCII
  scalar the raw encoding is the only spelling, since the three escapes stay
  ASCII-only and no `\u` form is added.

## Delta

### Change 1 — `[FORM-5]`, active line 89 (one sentence)

Before:

```
`unit`; STRING `"..."` whose interior is a sequence of items, each one raw ASCII-printable byte in U+0020..U+007E other than `"` and `\`, or one of exactly three escapes `\\ \" \n`; no other byte is legal, and each character has exactly one spelling (the escape where one is defined, the raw byte otherwise).
```

After:

```
`unit`; STRING `"..."` whose interior is a sequence of items, each one raw ASCII-printable byte in U+0020..U+007E other than `"` and `\`, one raw scalar value in U+00A0..U+10FFFF written as its complete well-formed UTF-8 encoding, or one of exactly three escapes `\\ \" \n`; no other byte sequence is legal, and each character has exactly one spelling (the escape where one is defined, the raw encoding otherwise).
```

### Change 2 — `[FORM-5]`, active line 90 (one sentence, replacing the DEFERRED marker)

Before:

```
STRING appears only in `doc` entries, `check` messages, and `claim` justifications; non-ASCII diagnostic text is DEFERRED.
```

After:

```
STRING appears only in `doc` entries, `check` messages, and `claim` justifications; its interior is a carried byte sequence, and no rule normalizes, case-folds, reorders, or compares it, so two STRINGs whose encodings differ are different STRINGs however they render and this admission fixes no Unicode version.
```

### Change 3 — `[DIAG-1]` raw scanning, active line 1585 (one sentence)

This is the sentence that currently rejects an admitted scalar.

Before:

```
At any other STRING cursor, if the actual byte sequence beginning there does not begin one complete well-formed UTF-8 encoding of a Unicode scalar value, [FORM-2] spans its first byte; a valid non-ASCII scalar instead cites [FORM-5] and spans its complete UTF-8 encoding.
```

After:

```
At any other STRING cursor, if the actual byte sequence beginning there does not begin one complete well-formed UTF-8 encoding of a Unicode scalar value, [FORM-2] spans its first byte; a valid scalar in U+0080..U+009F cites [FORM-5] and spans its complete UTF-8 encoding, and a valid scalar at or above U+00A0 is an admitted STRING item.
```

Unchanged and deliberately so: line 1573 (an ill-formed encoding inside a STRING
still cites `FORM-2` at its first byte — the "no truncation mid-encoding"
guarantee), line 1578 (a non-ASCII scalar *outside* a STRING still cites
`FORM-1`), lines 1581–1583 (a backslash followed by a non-ASCII scalar remains a
`FORM-5` rejection spanning the backslash and the complete encoding, because no
`\u` escape is added), and line 1584 (a raw ASCII byte outside the permitted set
still cites `FORM-5`).

## Derived material this delta obliges, in the same work

| surface | change |
|---|---|
| `compiler/src/lexer/scanner.rs::string` (lines 228–296) | the `_ if !byte.is_ascii()` arm stops rejecting: accept a scalar at or above `U+00A0` by advancing `utf8_scalar_len`; keep `InvalidUtf8` for an ill-formed sequence and `InvalidStringByte` for `U+0080..U+009F` |
| `compiler/src/lexer/tests/hostile.rs` | boundary cases at `U+007F`, `U+0080`, `U+009F`, `U+00A0`, plus a truncated 3-byte encoding and a lone continuation byte, each asserting which of `FORM-2`/`FORM-5` cites |
| generated syntax data (`compiler/src/syntax/grammar/generated.rs` via `grammar_tables`) | regenerate; `[GRAM-1]`'s STRING terminal predicate widens |
| native grammar verifier | rerun; the STRING predicate is a frontend contract |
| `tests/conformance/` cases and manifest | protected; a new `FORM-5` case family for the boundaries above needs its own owner approval, separate from the spec packet |
| `compiler/src/backend/emitter.rs::json_string` | **already landed in this batch** (commit `4d4195c`): the encoder preserved only ASCII, expanding every non-ASCII byte into its Latin-1 scalar and doubling it. The regression `backend::tests::a_diag3_record_preserves_the_exact_utf8_bytes_of_its_message` covers it |

The C1 exclusion is what keeps `json_string` correct with three escapes: JSON
requires escaping `U+0000..U+001F`, and after this delta no such scalar can
reach a record field.

## Status of D1 in this batch

Blocked on spec bytes, by design — the compiler's frontend contract cannot lead
the grammar. The one part that could land landed: the emitter defect that would
have corrupted every multi-byte diagnostic the moment the grammar admitted one.

---

# D2 — escaped host-string display

## What blocks it

`[HOST-2]` defers the presentation family; `[SYS-2]` is the inventory that would
have to carry the operations. The compiler's inventory is extraction-locked
against the specification's own bytes, so a compiler-first addition fails.
Reproduced this batch by adding the two rows below to
`compiler/src/resolution/catalog.rs` and running
`cargo test --profile gate --lib resolution::catalog`:

```
resolution::catalog::tests::system_inventory_matches_the_sys2_counted_totals FAILED
  left: 13   right: 11                                     (catalog.rs:1049)
resolution::catalog::tests::system_entities_are_recovered_from_preorder_ordinals FAILED
  left: (14, 39, 13, 111)   right: (14, 39, 11, 103)       (catalog.rs:1217)
resolution::catalog::tests::system_inventory_matches_independent_extraction_from_exact FAILED
  the extracted eleven-operation list vs the catalog's thirteen (catalog.rs:1328)
```

The probe was reverted. This is the lock working as designed, and it is the same
class of lock this batch extended to `wf-ops` and `wf-prov`.

## Design: one measure, one copy, mirroring the two existing routes

`[HOST-2]` already has exactly two routes with a settled shape: a length
operation plus a one-attempt copy over a caller-owned `buffer<u8>` and a
caller-written range. The presentation family is a **third route of the same
shape**, so it adds no new mechanism:

| existing | lossless | text | new: escaped |
|---|---|---|---|
| measure | `host_bytes_len` | `host_utf8_len` | `host_escaped_len` |
| copy | `host_copy_bytes` | `host_copy_utf8` | `host_copy_escaped` |

Design decisions, each chosen to avoid inventing machinery:

- **Total, not fallible in a new way.** The escaped rendering is defined for
  *every* code-unit sequence — that is the whole point of a display route, and
  it is why the text route cannot serve the purpose. So `host_escaped_len`
  returns `own u64` with no failure outcome, exactly like `host_bytes_len`, and
  `host_copy_escaped` reuses `CopyError` with `CopyTooSmall(required)` as its
  only recoverable failure. **No new enum, no new constructor spelling, no new
  `IoError` class** — `[SYS-6]`'s inventory grows by two rows that name existing
  types.
- **ASCII-only output.** The rendering emits only bytes in `U+0020..U+007E`.
  A code unit that is an ASCII printable other than `\` renders as itself; `\`
  renders as `\\`; every other code unit renders as `\xHH` with two uppercase
  hex digits, four bytes. On a 16-bit family a code unit above `0xFF` renders as
  `\uHHHH`, six bytes. This is a fixed, target-family-dependent function of the
  code units and nothing else — no Unicode tables, no locale, no replacement
  scalar.
- **Lossy is the same operation, not a fourth one.** "Lossy" in the deferred
  marker's sense (never fails, never refuses) is satisfied by totality above;
  there is no separate lossy route to add.
- **Quoting is not an operation.** Surrounding quotes are two bytes the caller
  writes. Adding a quoting variant would double the family for zero semantic
  content.
- **`host_escaped_len` is the exact length `host_copy_escaped` needs**, so the
  caller's sizing loop is the same one it already writes for the other routes,
  and `[SYS-8]`'s "reports the exact length the destination range must have"
  contract carries over verbatim.

Deliberately still deferred and stated as such: any operation that decomposes,
enumerates, or joins a path (`[PATH-1]` already defers display), and any
conversion between the escaped rendering and a source text type.

## Delta

### Change 1 — `[HOST-2]`, active line 2278 (replace the DEFERRED sentence)

Before:

```
Escaped, quoted, and lossy display of a host string are a DEFERRED separate presentation family with their own delta [META-5], not a mode of either route.
```

After:

```
The display route renders the code units as one ASCII presentation of them, is total over every code-unit sequence, and is not a mode of either route above: it reports and copies a rendering, never the code units themselves, so no program recovers the original sequence from it and no rule admits it where text is required.
Quoting is not an operation: a caller that wants delimiters writes them itself.
```

### Change 2 — `[HOST-2]`, active line 2279 (extend to three routes)

Before:

```
The exact operation names, signatures, buffer and range preconditions, and outcome types of both routes are [SYS-2] inventory data, with their transfer semantics in [SYS-8] and their outcome types in [SYS-6].
```

After:

```
The exact operation names, signatures, buffer and range preconditions, and outcome types of all three routes are [SYS-2] inventory data, with their transfer semantics in [SYS-8] and their outcome types in [SYS-6].
```

Also change line 2275 ("Exactly two routes exist.") to "Exactly three routes
exist: two conversions and one display." The display sentence above replaces the
DEFERRED sentence in place, so it stays inside `[HOST-2]`'s route paragraph.

### Change 3 — `[SYS-2]` operation block, two rows after `host_copy_utf8`

```
fn host_escaped_len['v](value: &'v HostString) -> own u64 reads('v);
fn host_copy_escaped['v, 'd](value: &'v HostString, destination: &uniq 'd buffer<u8>, offset: own u64, capacity: own u64) -> own Result<u64, CopyError> reads('v 'd), writes('d), traps;
```

### Change 4 — `[SYS-2]` counted totals (active line 2200)

Before: "exactly fourteen nominal types, thirty-nine enum-variant constructors,
sixty-four variant fields, eleven operations, fourteen operation region
parameters, and twenty-five operation value parameters."

After: "…fourteen nominal types, thirty-nine enum-variant constructors,
sixty-four variant fields, **thirteen** operations, **seventeen** operation
region parameters, and **thirty** operation value parameters."

### Change 5 — `[SYS-2]` declaration-record count (active line 2235)

Before: "one hundred and sixty-seven declaration records".
After: "one hundred and seventy-seven declaration records".

Derivation, so the lead can re-check rather than trust: 14 + 39 + 64 + 13 + 17 +
30 = 177; owner-local records 64 + 17 + 30 = 111; resolver-visible 14 + 39 + 13 =
66. These are the exact numbers the reverted probe produced from the compiler's
own preorder walk — `(14, 39, 13, 111)` at `catalog.rs:1217`.

### Change 6 — `[SYS-2]` `wf-prov` table, two rows

```
| `host_escaped_len` | plain result external | — |
| `host_copy_escaped` | `Ok(value:)` internal; `Err(error:)` external | `destination` external |
```

Rationale for the classes, matching the existing pairs exactly:
`host_escaped_len`'s result is a function of target-supplied code units, so it is
external like `host_bytes_len`'s; `host_copy_escaped`'s success count is bounded
by the caller's own `capacity`, so it is internal like `host_copy_bytes`'s, while
its `Err` payload measures target data and is external.

### Change 7 — `[SYS-6]` `wf-sys` outcome table, two rows

```
| `host_escaped_len` | `own u64`; total, no failure outcome |
| `host_copy_escaped` | `own Result<u64, CopyError>` |
```

`[SYS-6]`'s prose already covers `CopyTooSmall(required)`; extend the sentence
"On a successful `arg_get` …" only to name the escaped copy's `Ok` payload as
the exact rendered length.

### Change 8 — `[SYS-8]` (three sentences)

- Opening sentence: add `host_copy_escaped` to the list of one-attempt
  operations, and to "for the two copy operations it is `offset` and
  `capacity`" → "for the three copy operations".
- Postcondition sentence: add "on a successful `host_copy_escaped` the copied
  length is at most the requested `capacity`".
- New paragraph after the "two copy operations differ only after range
  validation" paragraph:

```
`host_copy_escaped` measures the complete rendering, returns `CopyTooSmall(required)` without writing any byte when the destination range is smaller than that exact rendered length, and otherwise copies the complete rendering.
The rendering is fixed by the target's code-unit family [HOST-1]: a code unit that is an ASCII printable in U+0020..U+007E other than `\` renders as that one byte; `\` renders as the two bytes `\\`; every other code unit of an 8-bit family renders as the four bytes `\xHH` with two uppercase hexadecimal digits; and a 16-bit family code unit above 0xff renders as the six bytes `\uHHHH` with four uppercase hexadecimal digits.
Every rendered byte is therefore in U+0020..U+007E, the rendering is total over every code-unit sequence, and `host_escaped_len` reports exactly the length `host_copy_escaped` writes for the same value.
```

### Change 9 — `[ENT-3.S10]`

`host_copy_escaped`'s `Ok(value: w)` count is bounded by its `capacity`
parameter on exactly the same footing as `host_copy_bytes`, so add it to S10's
operation list and to the `capacity` bounding-parameter list. Without this the
boundary fact is lost and every caller re-checks a bound the operation already
guarantees.

Note for the lead: this touches a rule whose sub-id set is now extraction-locked
(`semantic::tests::entailment_sources`), but only S10's prose changes, not the
label set, so no lock moves.

## Derived material D2 obliges, in the same work

| surface | change |
|---|---|
| `compiler/src/resolution/catalog.rs` | `SYSTEM_OPERATIONS` 11 → 13 with the two rows above (exact literals are in the reverted probe); the three count assertions follow the spec's new totals |
| `compiler/src/semantic/provenance.rs` | `system_result_provenance` and `system_external_writes` gain ordinals 6 and 7 with the rest shifting by two; **the `wf-prov` lock landed this batch (`f431b3b`) will catch a mis-shift** |
| `compiler/src/backend/qualification.rs` | two semantic ids, `wf.sys.host_escaped_len.v1` and `wf.sys.host_copy_escaped.v1`; the existing indices shift |
| `compiler/src/backend/emitter/system.rs` | two emitters. `host_escaped_len` is a counting loop over the lease; `host_copy_escaped` reuses `range_validation` and then a render loop. Both are ordinary emitted IR with no new host symbol and no allocation |
| `compiler/src/backend/tests/system.rs`, `system_io.rs` | round-trip tests: an all-printable value renders unchanged; a value containing `\`, `0x00`, `0x7f`, and `0xff` renders to the exact expected bytes; `capacity` one short returns `CopyTooSmall(required)` with the exact length and leaves the buffer unchanged; `host_escaped_len` equals the copied length |
| `compiler/src/semantic/entailment/flow/sources.rs` | `BOUNDARY_COUNTS` (line 39) gains `("host_copy_escaped", "capacity", "Ok")`; the array widens 4 → 5. It already resolves the bounding actual by parameter name from the catalog row, so nothing positional moves |
| `tests/conformance/` | protected; a case family for the display route needs its own owner approval |
| `spec/derivation/derivation-ledger.md`, `compiler/src/spec_identity.rs` | regenerate as the activation normally does |

## Status of D2 in this batch

Design complete, delta text complete, implementation **blocked on spec bytes**
and reported rather than forced. The blocker is the extraction lock, not a
missing decision: every open question above (failure vocabulary, totality,
quoting, lossy-vs-escaped, the 16-bit family) is answered, and the exact numbers
the counted totals need were produced by the compiler's own preorder walk.
