# Linux directory enumeration — the disposition, and the delta for it

Status: DELTA TEXT FOR LEAD INTEGRATION (batch 0072, W3, executor G2). This
file is delta input to one v0.33 candidate; nothing here changes
`spec/kernel-spec.md`, and every byte lands only through the owner's
exact-byte approval. Basis revision: ACTIVE v0.32 at `spec/kernel-spec.md`,
SHA-256 `5ea3927aef20d08e1c9c80a50242628f2c469974261b68c696ee2db3934e6bf5`.

The choice put to this task was:

- **(a)** map Linux with `getdents64`, specifying its portable-record rewrite
  exactly as darwin's is specified, and stating honestly that it cannot be
  executed here; or
- **(b)** keep it unmapped and state the qualification failure in the spec's
  own [QUAL-2] terms so a reader learns it from the specification rather than
  from a research file.

## Recommendation: (b), with one correction that (b) as briefed forces

Keep Linux unmapped. But the investigation found that the reason the tree
currently gives for the stop is **the wrong reason**, and correcting it is
what makes (b) actually teachable from the specification. The delta in §4 is
that correction. It is small: two sentences in [QUAL-2] and one derivation-
ledger cell.

### The finding

`spec/kernel-spec.md:2395` states the third guarantee as:

> The third is a directory-enumeration facility for the enumeration semantic
> IDs [SYS-14]: one host call that reports a bounded batch of the entries of
> an open directory and advances that directory's own enumeration position.

Linux **supplies** that. `getdents64(2)` is exactly one host call that reports
a bounded batch of the entries of an open directory descriptor and advances
that descriptor's own position. So under v0.32's own definition Linux is a
*qualifying* target for the enumeration guarantee. What is missing is the
approved-implementation row — the `(specification version, semantic ID,
target, program kind)` mapping [QUAL-1] requires — because nobody has
transcribed and tested it.

[QUAL-1] already distinguishes these two stops precisely, at
`spec/kernel-spec.md:2384`:

> Compilation stops when the mapping is absent, when the approved
> implementation is incompatible with the selected target or program kind, or
> when a required target guarantee is unmet.

Absent mapping and unmet guarantee are the first and third arms of that
sentence, and they mean different things about the target. The compiler
currently reports Linux under the third.

### Where the tree says the wrong thing

`compiler/src/backend/qualification.rs:667`:

```rust
TargetGuarantee::DirectoryEnumeration => self.directory_enumeration.is_some(),
```

`directory_enumeration` is an `Option<DirectoryEnumeration>` holding the
target's ABI record — symbol, declaration, and the five `struct dirent`
offsets. For the Linux triples it is `None`
(`compiler/src/backend/qualification.rs:704–710`), so `supplies()` answers
"this target has no directory-enumeration facility", and `operation_row`
raises `QualificationFailure::UnmetGuarantee { guarantee: DirectoryEnumeration }`
for `open_list` (ordinal 12) and `list_once` (ordinal 13).

One `Option` is being asked to answer two different questions: *does this
target have such a facility* (a property of the target, which [QUAL-2] owns)
and *have we transcribed and approved its record* (a property of our table,
which [QUAL-1] owns). Linux answers yes to the first and no to the second, and
the code reports the first.

`spec/derivation/derivation-ledger.md:151` repeats the same conflation in its
Form cell: "Linux enumeration is deliberately unmapped and fails qualification
rather than emulating [QUAL-2]." The word *unmapped* is right; the citation is
not.

The compiler's own comment at `compiler/src/backend/qualification.rs:697–703`
already has it right — "this target fails qualification for the [SYS-14]
enumeration IDs — a target-qualification failure, not a source rejection
[QUAL-1]" — so the code comment and the code disagree.

No test pins any of this: the only Linux-triple test in the tree is
`every_portable_class_is_mapped_exactly_once_in_inventory_order`
(`compiler/src/backend/tests/system_io.rs:1169`), which exercises the error
tables and never touches enumeration.

## Why not (a)

Four reasons, in order of weight.

**1. A qualification row is a promise, and this one cannot be kept from
here.** The host is macOS (Darwin 25.5.0); there is no Linux CI in the tree
and no way to execute a single emitted `getdents64` call. Landing an approved
implementation row would assert a `(version, semantic ID, target, kind)`
mapping that nothing has ever run. [QUAL-3] additionally fixes the *emitted
shape* of a synchronous transfer — at most one direct host call, one count or
outcome check, a cold outcome mapper, no allocation, no copy of the
transferred data, the wrapper inlined or shown immaterial — "as a condition of
qualification", and states that the evidence for it is "inspection of emitted
code and symbols". That inspection cannot be performed for code that is never
emitted for a runnable target.

**2. The Linux rewrite is not darwin's rewrite with different constants.**
Darwin's record carries an explicit name-length byte (`d_namlen` at offset 18)
which the in-place portable rewrite reads directly; the portable form
[SYS-14] fixes is `[kind][name length][name bytes]`, so darwin's shim copies a
length it was handed. `struct linux_dirent64` has **no name-length field** —
the name is NUL-terminated and padded out to `d_reclen`. A Linux shim must
therefore *derive* the name length with a bounded scan inside each record
before it can write the portable one. That is a different rewrite with a loop
in it, not a table of five offsets, and it interacts with [SYS-14]'s "never
empty, never longer than the target's component limit, contains no NUL and no
target separator" record contract in a way that wants testing rather than
assertion. (Recorded from documentation knowledge, **not measured**: no Linux
host was available to this task. That is precisely the problem.)

**3. The call itself has an unsettled shape.** Darwin's
`__getdirentries64(int, void *, size_t, off_t *)` takes four arguments
including the position slot; the Linux call takes three and has no position
argument, and the libc wrapper `getdents64` was only exposed from glibc 2.30
— before that the call site must go through `syscall(SYS_getdents64, …)`.
Choosing between a libc wrapper with a version floor and a raw syscall is a
portability decision with [QUAL-3] consequences, and it needs a Linux host to
settle, not a guess.

**4. Nothing in the current plan needs it.** W1's searching wfgrep runs on the
one target that has an enumeration row. Adding an unverified second target row
is exactly the robustness work the project rules defer.

The honest position is therefore not "Linux cannot enumerate" — it plainly
can — but "we have not mapped it, and an unmapped target stops compilation
rather than getting a guess." Option (b) says that; the current text does not.

## 4. Exact edits

### 4.1 [QUAL-2] — distinguish the two stops

`spec/kernel-spec.md:2396`, current:

```
A target with no such facility fails qualification for those IDs rather than emulating them, and in particular never substitutes a scan built out of other operations.
```

Replacement (two sentences in place of one):

```
A target with no such facility fails qualification for those IDs rather than emulating them, and in particular never substitutes a scan built out of other operations.
A target that has such a facility but for which the table [QUAL-1] holds no approved implementation is a different stop with the same effect: compilation stops for an absent mapping, the target is not thereby declared unqualified, and no implementation is improvised for it in either case.
```

That added sentence is the whole content of disposition (b), stated where a
reader meets the guarantee. It names no target, which [QUAL-1] requires:
"The table is compiler-internal data" (`spec/kernel-spec.md:2388`), so the
specification cannot and must not say *which* targets are mapped. What a
reader can learn from the specification is the distinction — and today they
cannot.

### 4.2 [QUAL-1] — no change

`spec/kernel-spec.md:2384` already enumerates both stops. Nothing is added
there; §4.1 only makes [QUAL-2] stop implying that the absent-mapping arm is
the unmet-guarantee arm.

### 4.3 [SYS-14] — no change

SYS-14 is target-independent throughout and names no host call. It stays
byte-identical.

### 4.4 `spec/derivation/derivation-ledger.md:151` — correct the Form cell

Current fragment:

```
Linux enumeration is deliberately unmapped and fails qualification rather than emulating [QUAL-2].
```

Replacement:

```
Linux enumeration is deliberately unmapped: the target has the facility (`getdents64`) but no approved implementation row was transcribed or tested, so compilation stops for an absent mapping [QUAL-1] rather than emulating or guessing a record layout — `linux_dirent64` carries no name-length field, so its portable rewrite is a different rewrite from darwin's rather than a different constant table.
```

The derivation ledger is not the active specification, so this is an ordinary
documentation correction that may land with or without the [QUAL-2] byte.

## 5. Compiler correction — identified, not landed

The classification defect in §"Where the tree says the wrong thing" is a
defect **against ACTIVE v0.32**, not an anticipation of this delta: under
v0.32's own [QUAL-2] wording Linux supplies the guarantee, so
`UnmetGuarantee` is the wrong arm today. It is reported here rather than
landed because this brief authorised delta documents, the fix is one sentence
of the same story as §4.1, and splitting it from the spec bytes would leave
the two halves of one correction in different changes. The lead should land
them together.

Exact shape, for whoever lands it:

- `compiler/src/backend/qualification.rs`: separate the guarantee bit from the
  ABI record. Give `SystemTarget` a `directory_enumeration_facility: bool`
  answering [QUAL-2]'s third guarantee, keep
  `directory_enumeration: Option<DirectoryEnumeration>` as the transcribed
  row, set the bool `true` for both the darwin and the Linux triples, and make
  `operation_row` return `QualificationFailure::MissingMapping(facility)` for
  ordinals 12 and 13 when the bool holds and the record is `None`.
  `supplies()` then reads the bool.
- The `REVIEWED_FOR` tripwire at `compiler/src/backend/qualification.rs:59`
  is untouched: no [SYS-2] operation, resource, guarantee, or entry contract
  changes, only which arm of [QUAL-1] one absent row reports.
- Regression, none of which exists today: for both Linux triples, assert
  `open_list` and `list_once` stop with `MissingMapping`, and assert both
  darwin triples still qualify. This is the first test of any kind over the
  Linux enumeration disposition.

Nothing about the accepted source language changes: a qualification stop is
not a source-language rejection and cites no language rule
(`spec/kernel-spec.md:2385`), so no program's acceptance moves either way.

## 6. Impact inventory

- **Accepted set:** unchanged. Both edits are about target qualification,
  which runs after complete source acceptance.
- **Rejected set:** unchanged.
- **Emitted code:** unchanged. No target gains or loses a row.
- **Rule inventory:** 135, unchanged. No rule id is added or removed;
  [QUAL-2] gains one sentence.
- **Grammar:** untouched; the native grammar verifier is not implicated.
- **Diagnostics:** the failure *class* reported for an unmapped-but-capable
  target changes from `UnmetGuarantee` to `MissingMapping` once §5 lands.
  No source diagnostic changes.
- **Conformance corpus:** no case is added, changed, or re-verdicted.
  Target qualification is not exercised by the corpus.

## 7. What this leaves open

Linux support remains a real gap, and this delta does not close it — it
records the gap accurately so that closing it later is a bounded job. The
conditions under which (a) becomes the right answer are concrete and worth
stating: a Linux host or CI runner able to execute the tree's own traversal
tests, at which point the mapping is a transcription plus a
`linux_dirent64` name-length scan plus the [QUAL-3] emitted-shape inspection,
each of which can then be evidenced rather than asserted.
