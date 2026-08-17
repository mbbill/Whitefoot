# DELTA-DIAG1 — DIAG-1 enumerations to `wf-diag` rows

Status: CANDIDATE DELTA TEXT for the v0.31 candidate, batch 0070 (W5,
"DIAG-1 restructure"). Not a spec edit. The lead splices the hunks below into
the single v0.31 candidate; the spec file has one writer.

Base: `spec/kernel-spec.md` at v0.30, SHA-256
`5ed210190737b2aa53a91dc901f07d02344669eeb6d6660224602872331204d1`. DIAG-1
occupies lines 1546-1890 (42,060 B). Every line number below is a line of that
base file. A different base byte invalidates every hunk and ledger here.

Authority for the move: `research/investigations/spec-representation/DOSSIER.md`
§3(b) profile item 4 ("Data payloads: every machine-consumed span is a fenced
block with a `wf-<schema>` info string … First row of a table fence is its
column schema") and §4 Stage 1's "DIAG-1 enumerations to rows".

## 1. What this delta does and does not claim

It converts three of DIAG-1's enumerations into `wf-diag` fences. It is
**content-preserving by mechanical reconstruction**: every fence cell is a
verbatim substring of the base line it came from, and `verify-delta.rs`
reconstructs each base line byte-for-byte from its row plus one declared
reading template per fence. Nothing is reworded.

It does **not** reduce bytes. Measured below: the restructure is +386 B. The
DOSSIER's "−~12 KB DIAG-1/DIAG-2 prose scaffolding" estimate does not survive
arithmetic — markdown pipe syntax costs about as much per row as the prose
scaffolding it removes (see §4). The justification for these three fences is
addressability and Stage 2 extraction locks, not conciseness.

Six further candidate regions were examined and **rejected as not
content-preserving**; §5 records each with its reason. That is the substance of
this delta as much as the three hunks are.

## 2. The three conversions

### Hunk H1 — the location closed sum (base lines 1546-1554)

<<<hunk H1 1546 1554
[DIAG-1] Every source-language rejection cites exactly one numbered language rule and exactly one location from this closed sum:

```wf-diag location
| form | selected |
|---|---|
| `SourceBytes(SourceCoordinate)` | when no offending canonical-tree node exists or the defect belongs only to a source boundary |
| `SourceNode(NodePath, SourceCoordinate)` | when one source-backed canonical-tree node is the offending node |
| `BundleRoot(NodePath, BundleRootExtent)` | for a whole-unit defect with no offending source declaration |
```

Each row reads as its `form` followed by its `selected` condition.
`BundleRoot` requires the empty root `NodePath` and carries no source-local byte interval.
>>>

<<<ledger F1
template T1 {1} {2}
1549 | T1 | ; | `SourceBytes(SourceCoordinate)` | when no offending canonical-tree node exists or the defect belongs only to a source boundary
1551 | T1 | ; or | `SourceNode(NodePath, SourceCoordinate)` | when one source-backed canonical-tree node is the offending node
1553 | T1 | . | `BundleRoot(NodePath, BundleRootExtent)` | for a whole-unit defect with no offending source declaration
>>>

The three base lines 1548, 1550, 1552 hold only the list numerals `1.`, `2.`,
`3.` on their own lines — an artifact of the v0.30 one-sentence-per-line
migration applied to a numbered list. Row order carries the same enumeration,
so they are declared scaffold and vanish.

Order safety: the three forms are a closed sum selected by their own
conditions, not a priority ladder, and all three move together in base order.

### Hunk H2 — the declaration-inventory closed rank (base lines 1696-1704)

<<<hunk H2 1696 1704
Declaration inventory and FN-9 selector reservation create candidates under this closed rank:

```wf-diag inventory-rank
| rank | cites | candidate |
|---|---|---|
| 1 | FORM-3 | reserved-name violation defined by OP-1's derived set |
| 2 | OWN-3 | repeated REGIONID declaration within one function declaration or contract-member signature, parameters included |
| 3 | GRAM-10 | match-binder freshness violation |
| 4 | TYPE-6 | collision with PRE-1 |
| 5 | TYPE-6 | collision with an admitted system declaration [SYS-1] |
| 6 | TYPE-6 | compilation-root duplicate or same-lexical-scope redeclaration |
| 7 | TYPE-6 | nested declaration shadowing a live declaration |
```

Each row reads as its `rank`, the rule its `cites` cell names, and the `candidate` that rank admits; the rows retain rank order.
>>>

<<<ledger F4
template Ta {1}. a {2} {3}
template Tan {1}. an {2} {3}
1698 | Ta | ; | 1 | FORM-3 | reserved-name violation defined by OP-1's derived set
1699 | Tan | ; | 2 | OWN-3 | repeated REGIONID declaration within one function declaration or contract-member signature, parameters included
1700 | Ta | ; | 3 | GRAM-10 | match-binder freshness violation
1701 | Ta | ; | 4 | TYPE-6 | collision with PRE-1
1702 | Ta | ; | 5 | TYPE-6 | collision with an admitted system declaration [SYS-1]
1703 | Ta | ; and | 6 | TYPE-6 | compilation-root duplicate or same-lexical-scope redeclaration
1704 | Ta | . | 7 | TYPE-6 | nested declaration shadowing a live declaration
>>>

Two templates differ only by the English indefinite article, which carries no
normative content; both reconstruct their base lines exactly. Rank order is
normative ("the first applicable rank at that event", base line 1707) and all
seven items move together in base order.

### Hunk H3 — the lexical-use role-attribution table (base lines 1754-1766)

The eleven role-to-rule assignments are already a markdown table; the fence
gives extraction an info string to key on instead of a prose anchor. Every row
is byte-identical to its base line.

<<<hunk H3 1754 1766
```wf-diag role-attribution
| lexical-use role | rule cited by rank 1 or rank 3 |
|---|---|
| `type` TYPEID | TYPE-5 |
| contract bound or `conform_decl` contract TYPEID | FN-3 |
| `construct` constructor TYPEID, enum-variant-only `arm` TYPEID, or variant-form `ensures_selector` TYPEID | TYPE-6 |
| REGIONID use | OWN-3 |
| LABEL use | TYPE-6 |
| `const` IDENT | CONST-1 |
| `cvalue` IDENT | CONST-2 |
| `pbase` IDENT | TYPE-5 |
| IDENT or OPNAME `callee` | OP-1 |
| `fn_bind` right IDENT | FN-3 |
| FORM-5 generic-numeric TYPEID suffix | FORM-5 |
```
>>>

<<<ledger F6
template T1 | {1} | {2} |
1756 | T1 |  | `type` TYPEID | TYPE-5
1757 | T1 |  | contract bound or `conform_decl` contract TYPEID | FN-3
1758 | T1 |  | `construct` constructor TYPEID, enum-variant-only `arm` TYPEID, or variant-form `ensures_selector` TYPEID | TYPE-6
1759 | T1 |  | REGIONID use | OWN-3
1760 | T1 |  | LABEL use | TYPE-6
1761 | T1 |  | `const` IDENT | CONST-1
1762 | T1 |  | `cvalue` IDENT | CONST-2
1763 | T1 |  | `pbase` IDENT | TYPE-5
1764 | T1 |  | IDENT or OPNAME `callee` | OP-1
1765 | T1 |  | `fn_bind` right IDENT | FN-3
1766 | T1 |  | FORM-5 generic-numeric TYPEID suffix | FORM-5
>>>

The two references to this table in base lines 1750 and 1752 ("the
role-attribution table below") are untouched and still resolve.

## 3. Declarations the checker enforces

<<<scaffold
1548
1550
1552
>>>

<<<connective
1554 :: This form requires the empty root `NodePath` and carries no source-local byte interval. => `BundleRoot` requires the empty root `NodePath` and carries no source-local byte interval.
>>>

One connective substitution: base line 1554's "This form" referred to the
immediately preceding numbered item 3; after the move it is named explicitly.
No other word of DIAG-1 changes.

<<<new-prose
```wf-diag location
| form | selected |
|---|---|
```
Each row reads as its `form` followed by its `selected` condition.
```wf-diag inventory-rank
| rank | cites | candidate |
|---|---|---|
Each row reads as its `rank`, the rule its `cites` cell names, and the `candidate` that rank admits; the rows retain rank order.
```wf-diag role-attribution
>>>

Every line of new text in the three hunks is either a base line, a fence row
listed in a ledger, or one of the lines above. The two reading sentences are
the only new normative prose; each states the fence's single reading, which is
what makes the rows reconstructible.

## 4. Byte accounting

Measured by `verify-delta.rs` check C7 (see `PASS-EVIDENCE.md`):

| span | base B | new B | delta |
|---|---|---|---|
| H1 location sum (1546-1554) | 566 | 694 | +128 |
| H2 inventory rank (1696-1704) | 576 | 802 | +226 |
| H3 role-attribution table (1754-1766) | 521 | 553 | +32 |
| DIAG-1 body (1546-1890) | 42,059 | 42,445 | +386 |

The rule's full span including its trailing blank line (1546-1891) is 42,060 B
and becomes 42,446 B.

Why a table cannot shrink this text: per row, pipes and padding cost about
10 B while the removed sentence scaffolding (`cites [` … `] and spans` … `.`,
or `rejects at` … `and its complete extent.`) is 22-37 B — a net 12-27 B win
per row — and each fence pays roughly 120-140 B once for its info string,
column schema, separator, closing delimiter, and reading sentence. A fence
therefore breaks even at about six to ten rows and loses below that. Adding a
`rank` and a `cites` column, as H2 does, costs 16 B per row and reverses the
sign. This arithmetic holds for every region in §5 as well, so no
reorganization of DIAG-1's enumerations reduces its byte count materially.

## 5. Candidate regions examined and NOT converted

Each was tested against two conditions a content-preserving move must satisfy:
**uniformity** (all rows reconstruct through one declared reading, so the fence
has a single meaning) and **order safety** (no sentence that stays in prose is
ordered relative to a sentence that moves).

| region | base lines | size | verdict |
|---|---|---|---|
| Raw lexical and STRING defect assignments | 1572-1588 | 2,360 B | **Fails order safety.** Nine of fourteen sentences share the reading `{subject} cites [{rule}] and spans {span}.`; five do not (1573, 1583, 1585, 1587, and the scoping sentence 1580). The sequence is order-dependent — 1577 reads "Any **other** ASCII byte that cannot begin a specified token", which only means anything relative to the rows above it. Moving nine and leaving five would silently break that relation. |
| No-row attribution rows | 1607-1640 | 4,434 B | **Fails uniformity.** The spec already calls these "closed attribution rows … tested in order", but each row's citation sentence has row-specific phrasing ("that dotted call-or-targs spelling cites [FORM-3]", "the rejection cites that predicate's owner", "cites [FORM-1] as an unknown construct", "the expected-terminal set is replaced by only `SOURCE_END`, and the rejection cites the owner of `program`"). No single reading reconstructs all five, and rows 3 and 5 have no coordinate sentence at all. Row 3 alone is ~1.5 KB of definitional prose (the transparent mandatory-name path) that is not enumerable content. This is the largest enumerated region in DIAG-1 and the one most often assumed tabulable; it is not. |
| Lexical-use lookup closed rank | 1748-1752 | 637 B | **Fails uniformity, with a meaning-addition risk.** Ranks 1 and 2 end in a "carry …" clause; rank 3 does not. An empty payload cell would read as "carries nothing", which is stronger than the base text's silence. |
| Payload definitions (FORM-3, OWN-3, GRAM-10, TYPE-6, rank 1/2/3) | 1708-1732, 1768-1775 | 4,327 B | **Fails uniformity.** Each payload tuple is followed by rules about its members, ordering, and which conflicts a rank reports; the tuples cannot be separated from those rules without leaving dangling antecedents. |
| FN-3 contract-table locations | 1827-1839 | 1,813 B | **Fails uniformity and loses bytes.** Five of ten sentences share `{subject} rejects at {node} and its complete extent.`; 1828, 1831, 1836, 1837 each carry a different location form or an independent second clause. Five rows would net −110 B against ~140 B of fence overhead. |
| Callee-class citation and location assignment | 1841-1848 | 1,504 B | **Fails uniformity.** The genuinely tabular content is one sentence (1844: class → cited rule) with three items, and its third item inverts the phrasing of the first two ("for a table operation, the rule [OP-2] selects — OP-1 or TYPE-5"). A three-row fence costs more than the sentence. |
| FN-9 selector-admission locations | 1858-1866 | 1,889 B | **Fails uniformity.** Two of four location sentences share a reading; 1863 adds a coordinate clause and 1864 a payload clause. |

Total examined and rejected: 16,964 B of DIAG-1's 42,059 B. Adding the 1,663 B
of base text the three hunks cover, this accounts for every enumerated region in
the rule; the remaining ~23 KB is definitional and procedural prose with no
enumeration to move.

## 6. What the lead should decide

1. Whether three slightly-larger fences earn a place in the v0.31 exact-byte
   owner packet. Their value is that 21 rows of rule-citation assignment become
   extraction-keyable for the Stage 2 locks; their cost is +386 B and one
   approval item.
2. That the DOSSIER's DIAG-1 conciseness estimate (−12 KB) and its ≤300 KB
   ratchet target are not reachable by restructuring or by dereferencing
   restatement. `DELTA-RATCHET.md` §5 measures the whole-file upper bound.
3. Whether the base file's numbered-list scaffold lines (a bare `1.` on its own
   line, as at 1548-1552) should be normalized wherever they occur. They are a
   v0.30 migration artifact; H1 removes three of them as a side effect.
