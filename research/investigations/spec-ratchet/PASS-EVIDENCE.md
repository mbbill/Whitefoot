# PASS evidence — DIAG-1 restructure and ratchet deltas

Batch 0070, W5. Base `spec/kernel-spec.md` at v0.30, SHA-256
`5ed210190737b2aa53a91dc901f07d02344669eeb6d6660224602872331204d1` (resolved
with `shasum -a 256`, not copied forward).

Disposition of this directory: the ratchet deltas R1-R4 integrated into the
v0.31 candidate, and `verify-delta.rs` — a one-shot pinned to the digest above
and to those hunks — was deleted with them, as its own docstring required. The
`DELTA-DIAG1.md` fences are the part still awaiting an owner decision, so this
record and that file stay until it is made; delete both in the change that
lands or abandons the fences. Nothing outside this directory reads these files.

## Commands (as run, before the verifier was deleted)

```
rustc -O --edition 2021 -o /tmp/verify-delta research/investigations/spec-ratchet/verify-delta.rs
/tmp/verify-delta spec/kernel-spec.md \
  research/investigations/spec-ratchet/DELTA-DIAG1.md \
  research/investigations/spec-ratchet/DELTA-RATCHET.md
```

Exit code read directly (`echo "EXIT=$?"`, not through a pipe): `EXIT=0`.

## What a green run does and does not establish

Establishes, mechanically:

- **C1** each of the 21 moved base sentences is reconstructed byte-for-byte from
  its new `wf-diag` row plus one declared reading template and one declared
  trailing list token. No cell is a paraphrase; every cell is a verbatim
  substring of its base line.
- **C2** every ledger row actually occurs in the new text, so the ledger cannot
  prove a row that was never written.
- **C3** every one of the 345 base lines of DIAG-1 is accounted for as moved,
  declared scaffold, declared connective, or carried through verbatim. Zero
  unclassified. This is the check that sees silent loss.
- **C4** every line of new text is a base line, a ledger row, or one of the
  declared new-prose lines. Zero undeclared. This is the check that sees
  silently added normative text.
- **C5/C6** the multiset of rule citations (41 distinct ids, 178 occurrences)
  and of location forms is identical before and after, comparing against the
  base with the one declared connective substitution applied — so the declared
  change cannot mask an undeclared one.
- **C8/C9** all four ratchet cut targets and all 33 quoted base fragments match
  the file's own bytes, so no evidence in `DELTA-RATCHET.md` is
  hand-transcribed.

Does **not** establish: that moving a mapping from prose into a fence is the
right design; that the fence's declared reading is the reading a human infers
from the fence alone; that the four ratchet cuts are wise rather than merely
equivalent; or anything at all about the seven regions §5 of `DELTA-DIAG1.md`
declines to convert. It also does not establish that the deltas apply cleanly to
any file other than the pinned base.

## Negative controls

The reconstruction proof would be decorative if no perturbation could break it,
so two distinct corruptions were run. Each must fail, and each must be caught by
a check that is not merely a restatement of another.

`--negative-control` (drops the last character of one row cell):

```
failures           3
C1 line 1549: reconstruction differs
    base: `SourceBytes(SourceCoordinate)` when no offending canonical-tree node exists or the defect belongs only to a source boundary;
    new : `SourceBytes(SourceCoordinate)` when no offending canonical-tree node exists or the defect belongs only to a source boundar;
C2 line 1549: row absent from new text: | `SourceBytes(SourceCoordinate)` | when no offending canonical-tree node exists or the defect belongs only to a source boundar |
C4 undeclared new line: | `SourceBytes(SourceCoordinate)` | when no offending canonical-tree node exists or the defect belongs only to a source boundary |
NEGATIVE CONTROL FAILED AS REQUIRED — the checks have teeth.
```

`--negative-drop` (deletes one carried-through sentence, base line 1662 — the
non-language-failure sentence — from the new text):

```
failures           3
C3 line 1662 is unclassified (dropped?): An input-envelope failure, resource failure, target-layout failure [STOR-6], target-qualification failure [QUAL-1], compiler-invariant failure, unsupported compiler capability, backend failure, or external-tool failure is not a source-language rejection, cites no language rule, and carries no expected-terminal set.
C5 citation QUAL-1: base 1, new 0
C5 citation STOR-6: base 2, new 1
NEGATIVE CONTROL FAILED AS REQUIRED — the checks have teeth.
```

The second control matters more than the first: a dropped sentence is the
failure whose symptom is success. C3 caught it, and C5 caught it independently
through the lost citations — two checks with different mechanisms, not one check
counted twice.

## Full green output

```
== base ==
spec               spec/kernel-spec.md
expected digest    5ed210190737b2aa53a91dc901f07d02344669eeb6d6660224602872331204d1
spec lines         3118
DIAG-1 span        1546-1890  (42059 B)

== C1 reconstruction of moved base lines ==
  ok    1549  trail=";"    F1
  ok    1551  trail="; or" F1
  ok    1553  trail="."    F1
  ok    1698  trail=";"    F4
  ok    1699  trail=";"    F4
  ok    1700  trail=";"    F4
  ok    1701  trail=";"    F4
  ok    1702  trail=";"    F4
  ok    1703  trail="; and" F4
  ok    1704  trail="."    F4
  ok    1756  trail=""     F6
  ok    1757  trail=""     F6
  ok    1758  trail=""     F6
  ok    1759  trail=""     F6
  ok    1760  trail=""     F6
  ok    1761  trail=""     F6
  ok    1762  trail=""     F6
  ok    1763  trail=""     F6
  ok    1764  trail=""     F6
  ok    1765  trail=""     F6
  ok    1766  trail=""     F6

== C2 each row present in the new text ==
  21 rows checked, 0 absent

== C3 coverage of base lines 1546-1890 ==
  moved 21  scaffold 3  connective 1  retained 320  unclassified 0

== C4 provenance of new lines ==
  356 new lines, 0 undeclared

== C5 rule-citation multiset ==
  ok    41 distinct ids, 178 occurrences, identical before and after

== C6 location-form multiset ==
  ok    BundleRoot         6
  ok    BundleRootExtent   2
  ok    NodePath           16
  ok    SourceBytes        4
  ok    SourceCoordinate   8
  ok    SourceNode         24

== C7 byte accounting ==
  H1  base 1546-1554  566 B -> 694 B  (+128 B)
  H2  base 1696-1704  576 B -> 802 B  (+226 B)
  H3  base 1754-1766  521 B -> 553 B  (+32 B)
  DIAG-1 total        42059 B -> 42445 B  (+386 B)

== C8 ratchet cuts against the base file ==
  ok   R1 line  2701 ref  2698  217 B -> 82 B  (-135 B)
  ok   R2 line  2745 ref  1303  346 B -> 170 B  (-176 B)
  ok   R3 line  1937 ref  2816  276 B -> 158 B  (-118 B)
  ok   R4 line  2949 ref  2911  258 B -> 140 B  (-118 B)
  ratchet total -547 B

== C9 quoted base sentences ==
  33 quotations checked, 0 not found

== verdict ==
checks run         786
failures           0

PASS
```

Line-count reconciliation, as an independent arithmetic check on C3/C4: the base
span is 345 lines (21 moved + 3 scaffold + 1 connective + 320 retained) and the
new text is 356. The difference of 11 is exactly the three fences' structure —
H1 +3, H2 +6, H3 +2 lines of info string, column schema, separator, delimiters,
and reading sentence. A duplicated base line would have to be offset by a
dropped one, which C3 catches.

Note on C6: `BundleRoot` occurs 6 times in the new text against 5 in the raw
base, which is the declared connective substitution on base line 1554 ("This
form" becomes "`BundleRoot`"). The check compares against the base with that one
declared substitution applied, which is why it is green; the raw-base comparison
was red before the substitution was declared, and that is the behaviour wanted.

## Repository gate state at the time of this run

Branch `caps-batch` tip `029f31e`. This work adds only files under
`research/investigations/spec-ratchet/` and moves no specification byte; the
`.rs` file is outside every Cargo workspace (there is no root `Cargo.toml`, and
`compiler/Cargo.toml` declares no members), so it enters no build graph.

Gate legs that could be affected were run:

```
make repository-invariants spec-append-only spec-archive-integrity spec-digest-sync
spec append-only: no released kernel specification was modified or removed
spec archive integrity: 31 recorded specifications hash as recorded
spec digest sync: docs/current-plan.md does not quote the active digest (v0.30 5ed2101907...)
spec digest sync: docs/current-plan.md does not name v0.30 as the active authority
make: *** [spec-digest-sync] Error 1
```

That failure is **pre-existing on the branch and not caused by this work**:
`docs/current-plan.md` is unmodified here (`git diff --name-only HEAD -- docs/current-plan.md`
is empty) and the two strings `spec-digest-sync` requires — the active digest,
and the literal `Active language authority: v0.30` — are absent from the plan
file as rewritten by `eb012d2`. It is reported to the lead rather than fixed,
because the Current Plan is the lead's file and editing it is outside this
scope.

The compiler and conformance legs were not re-run: nothing here reaches them.
