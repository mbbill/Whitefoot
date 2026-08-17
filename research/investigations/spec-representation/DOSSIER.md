# Specification representation — decision record

Status: LEAD-ADOPTED DESIGN, 2026-08-16, during the owner's overnight
delegation of 2026-08-15. Implementation proceeds on branch `spec-rework`
under task `docs/ongoing/0068`. The two protected packets it names (P1
gate wiring, P2 migration activation) bind nothing until explicit owner
approval; everything else is ordinary tooling.

Provenance: synthesized from three research inputs (external precedents;
AI/machine-consumption measurement; workflow cost mining over the six
v0.24-v0.29 activations) by research workflow `wf_559854f6-dd6`; load-bearing
current-state figures independently re-verified (file and line sizes, chain
state, zero-dependency Cargo.toml, scanner shape of `bin/spec.rs`).

# DECISION DOCUMENT (DRAFT) — Next representation of `spec/kernel-spec.md`

Verdict up front: adopt a **structured markdown profile of the existing single file** — same path, same byte-hash authority, same archive model — with a schema linter grown inside the existing `whitefoot-spec` binary, a candidate mode that kills the red window, and generation replacing every hand-mirrored scalar. Do not adopt a DSL, a sibling formal model, or a non-markdown container. Stage the migration as one mechanically-verified activation plus two ordinary follow-ups.

---

## 1. Problem inventory

Each entry is a mechanism (what structurally goes wrong) plus a real incident from the three research inputs or verified this session.

### A. Context / consumption (primary reader is AI)

- **A1 — The monolith does not fit a working context.** 431,650 B ≈ 105–123k tokens; it cannot sit beside code in a 128k window. A task's declared rule need is 0.4–9% of the file, but with no index the practical choice is "load everything or grep and hope." Incident: task 0047 needed GRAM-6 = 1,762 B (0.41%) and the agent had a 431 KB artifact to get it from.
- **A2 — Retrieval keys exist but resolve to nothing.** 100% of citations in 67 task records are rule ids or S-labels (219 bracketed ids, 56 `§N`, **zero** file:line). Both `bin/spec.rs:28` and `runner.py:136` already compute the full rule-id set — and discard the offsets. The id→location index is computed twice per gate run and thrown away twice.
- **A3 — The most-cited semantic content is unaddressable.** S1–S12 are cited 71 times across 20 task records but are markdown bullets with no id syntax (verified: single-line bullets at spec lines 1179–1189). No tool — `spec.rs`, `runner.py`, the conformance manifest — can name them. `entailment.rs:17-23` restates the whole list in English because nothing else can point at it.
- **A4 — Hub restatement inflates every read.** Rules average 4.17 outgoing refs; FN-8 is cited 34×, DIAG-1 (42 KB, 9.7% of the spec in one rule) 26×. The 1-hop closure of a small task is 2.5–48% of the file; 2-hop is 34–77%.
- **A5 — Namespace collision.** `PROOF-8`, `CAND-8`, `VERIFY-1`, `BOUND-1` are plan/outline ids in exactly the rule-id shape; a resolver finds nothing in the spec and gets no signal it looked in the wrong document (`docs/done/0044,0050,0051,0067`).
- **A6 — 49,153 B of dead header taxes every full read.** Lines 3–46 are per-version `Prior:` paragraphs (verified; the single worst line in the file is the 10,699 B v0.23 paragraph). Every one duplicates the Status paragraph frozen in the corresponding immutable `spec/kernel-spec-vN.md` archive and the digest in `governance/APPROVALS.md`.

### B. Iteration safety

- **B1 — Hand mirrors without extraction locks drift silently.** Only column 1 of the 126-row op table is test-locked; `domain`/`signature`/`effects` are re-encoded by hand (`model.rs::traps()`, `scalar_result_type()`). Consequence found: **16 unreachable match arms** in `calls.rs:112-145` naming pre-respelling operation names (`iadd.wrap` — 0 hits in the spec). The PRV-1 table is hand-transcribed as numeric ordinals in `provenance.rs:477-528` with **no extraction test at all**.
- **B2 — Prose-anchored extraction is one edit away from silent loss.** `ebnf.rs` anchors on literal strings (`fenced_after("[GRAM-2]")`, `"Seven opaque nominal types: "`); 4 of 73 productions live *inline in prose* and the extractor's own doc comment confesses: "Scraping only fenced blocks silently loses those four." Task 0031 hand-added 84 follow rows, 70 underivable, **undetected for a full version**.
- **B3 — Byte-anchored deltas are version-brittle.** E3: the ENT-5 delta was a whole-line patch cut against v0.22, invalidated by v0.23, re-cut, re-verified, and the **owner approval re-obtained**. E2: held v0.24-era prose had to be re-derived against v0.26 — a whole task (0050).
- **B4 — Hand-transcribed identities.** E1: a hand-copied 40-hex commit id named a nonexistent object. E12: the active digest exists in **11 places**, 2 machine-guarded.
- **B5 — Hand-bumped scalar assertions.** `Ok(19)→Ok(20)→Ok(21)` chain length, `Ok(132)→Ok(133)` rule count, `Pass=409→423→432`, three `!= "v0.29"` string guards — ~8 hand edits per activation, each a latent wrong-constant.
- **B6 — Stale prose survives green gates.** E7: the derivation ledger *still at HEAD* says "the installed v0.28 authority" after v0.29 activated; `whitefoot-spec` checks only that a row exists per rule.

### C. Process cost

- **C1 — The 21 h 36 m red window.** From candidate freeze to activation, `make check` fails on ≥8 independent checks (empirically re-executed on the v0.28 candidate tree: `spec-archive-integrity` exits 2; identity, chain, title, rule-count tests all red). Consequence: candidates live on never-merged held branches or as *uncommitted worktree state* (v0.25–v0.27 have no candidate commit on any ref); the only verification available is hand-picked focused tests; the **first genuine full green gate runs after activation** (≈50 min per run); one activation had to be discarded and re-materialized to keep held commits off `main` (E16).
- **C2 — Verifier chicken-and-egg.** `whitefoot-grammar` returns `ChangedFrontendContract` unless the candidate equals the bytes the compiler already embeds — it can structurally never bless the thing it exists to check (`docs/done/0058` records exactly this).
- **C3 — Review does arithmetic a tool should do.** v0.28: "43 mechanically reproducible zero-context hunks", "three independent frozen-byte reviews" doing "rule arithmetic, grammar arithmetic" by hand.
- **C4 — Owner packets are hand-assembled.** 14 hand-transcribed source SHA-256s in the v0.28 packet.
- **C5 — Permanent per-version machinery.** `whitefoot-migrate` (~93 KB, v0.22→v0.23 spelling) still runs in every gate, two releases stale — exactly the "script forked per spec version" CLAUDE.md forbids.

### D. Conciseness

- **D1 — Unbounded growth loci.** The Status header grows ~2–5 KB per activation forever (49 KB now). DIAG-1 alone is 42 KB. Rules are single lines up to 2.6 KB (OP-4) with no sub-structure to prune against.
- **D2 — Restatement instead of reference** (A4's flip side): because refs are unchecked at write time, drafting agents restate hub content defensively, and nothing pushes back.

### E. Normative authority

- **E1 — The structure is real but latent.** Machines semantically consume 3.07% of the bytes (13,262 B); the other 97% is convention-carried prose. Everything in §B exists to re-derive structure that was flattened into prose.
- **E2 — Foreign numbering leaked into the normative text.** Four parallel systems ([FAM-n], S-labels, §sections, leaked plan Stage ids) with no declared relationship.
- **E3 — Intra-file authority is ambiguous.** The op-table op column is extraction-locked (normative-as-data); its other three columns are prose-normative only; nothing states which wins when a cell and a sentence disagree.

---

## 2. Weighted criteria (derived from the owner's requirements)

| # | Criterion | Weight | Source in owner intent |
|---|---|---|---|
| K1 | AI context economy: per-task retrieval cost, addressability, index-ability | 25 | "must fit AI context; PRIMARILY READ BY AI" |
| K2 | Iteration safety: drift is unrepresentable or build-breaking, never silent | 25 | "maintenance slow, painful, error-prone" |
| K3 | Process cost per change: red window, hand-syncs, review/approval toil | 20 | "change process complex and time-consuming; iteration simple, safe, reliable" |
| K4 | Conciseness of the artifact itself | 10 | "the spec itself stays CONCISE" |
| K5 | Authority continuity: exact-byte approval, SHA chain, immutable archives survive unchanged | 10 | Constitution + APPROVALS model (hard constraint, also scored) |
| K6 | Migration cost/risk: no new toolchain deps, no Python, bounded one-shot work | 10 | Project law (no speculative frameworks; Rust-native; verified: Cargo.toml has zero deps) |

"Constraints embedded in the structure, not bolted-on tools" is not a separate criterion; it is the definition of a good K2 score.

---

## 3. Options matrix

### (a) Status quo text + tooling only (index/linter, zero spec-byte changes)

Emit the id→offset index and 1-hop ref graph from the scanners `bin/spec.rs` already contains; generate the identity scalars; add candidate-mode gates. **What it cannot fix, structurally:** giant-line diffs (B3, C3), unaddressable S-rules (A3 — adding ids *is* a spec-byte change), full-row table extraction (B1 — needs controlled cell vocabulary in the spec), the inline-production trap (B2), the 49 KB header (A6), conciseness (D). 
- Migration cost: ~2–4 h tooling. Iteration-safety gain: moderate (B4/B5 die; B1/B2/B3 remain). Token cost of result: unchanged 431 KB; per-task reads drop via index. Review-diff quality: unchanged (single-line rules diff as whole-rule replacement). Risk: lowest; also lowest ceiling — the error class that forced an owner re-approval (E3) survives intact.

### (b) Structured single-file markdown profile — RECOMMENDED

The spec stays **one markdown file at the same path**, hashed and archived exactly as today, but conforming to a machine-checked profile. This is the ecmarkup-shaped move the external report identifies as "the proven cheap step past free text," executed with structure the file mostly already has.

**Format selection.** Considered: TOML, JSON, YAML, Rust-macro DSL, custom-block-in-markdown.
- **JSON/TOML/YAML**: all require a parser crate — `compiler/Cargo.toml` has **zero dependencies today** (verified), and a hand-rolled YAML/TOML parser is its own project. Prose becomes escaped or indent-scoped strings: diff churn, token overhead, and LLMs read it worse than markdown. Rejected on K6 + K1 + review-diff.
- **Rust-macro DSL** (spec as Rust source the compiler consumes): collapses the spec into the compiler, inverting project law ("compiler behavior … do[es] not" define the language); the compiler-independent Python conformance runner could no longer read the normative artifact without rustc. Rejected on K5.
- **Custom-block-in-markdown**: zero new deps (extends the existing ~400-line line/bracket scanner in `bin/spec.rs`, verified plain `str` scanning); preserves every existing anchor (`^\[ID\]` line starts keep `rule_definitions`, `runner.py`'s regex, and `semantic/tests.rs::definition_rank` working); markdown is the format LLMs read natively; diffs become per-sentence. **Selected.**

**The profile, exactly:**

1. **File skeleton**: title line; ONE `Status:` paragraph for the current version; one pointer line ("Prior versions: the immutable `spec/kernel-spec-vN.md` archives and the `ACTIVE-SPEC:` chain in `governance/APPROVALS.md`."); then sections. All `Prior:` paragraphs deleted — they duplicate the archives byte-for-byte by construction (each archive froze its own Status paragraph).
2. **Rule block**: `[OP-4]` alone on its line; body = **one sentence per line**, blank line between paragraphs, block ends at next `^\[` or `^##`. No body line may begin with `[` (linter; rewrap as "Per [X], …"). No metadata keys unless a machine consumer exists (law: earn its place).
3. **Sub-rules**: `[ENT-3.S10]` at line start; addressable by the same scanner with the extended id shape. Envelope keys (`views:`, `point:`, `event:`) allowed **only** on S-rules because a compiler test will assert them. Parent carries `retired: S8` so non-reuse is machine-enforced, not a sentence.
4. **Data payloads**: every machine-consumed span is a fenced block with a `wf-<schema>` info string — `wf-ebnf` (including the 4 currently-inline productions of CONST-1/CONST-2/EFF-1), `wf-ops`, `wf-prelude`, `wf-sys`, `wf-prov`, `wf-diag`. Extraction keys on the info string, never on prose anchors. First row of a table fence is its column schema; enumerable cells use controlled tokens whose definitions live once in the owning rule's prose.

**Sample rule — OP-4, complete, faithfully transformed from the current single 2,580-byte line (verified at spec line 421):**

```markdown
[OP-4]
A subscript `p[i]` selects one element place of an indexable base: the base place `p`'s final selected type must be `array<T, N>`, `slice<'r, T>`, or `buffer<T>`, and the subscripted place's selected type is exactly that element type T — derived from the base place's already-fixed type [TYPE-5] — written where the binding carries an annotation, derived at a body `let` — by the same declared-type selection that types a field suffix, never from expected type or cross-statement inference; a subscript whose base's final selected type is not one of the three indexable types is a hard error citing OP-4 at that subscript's `psuffix` node.
The subscript carries the bounds obligation `i < len(p)` [ENT-6].
A discharged subscript reads or writes with no runtime bounds check in every build mode, and its checked-program disposition records the discharging derivation [DIAG-2].
Base discharge is judged before provenance: a subscript whose obligation the complete fact state does not discharge is a compile-time rejection citing OP-4 at that subscript's `psuffix` node, carrying the residual obligation rendered exactly per [ENT-6]; it forms no [PRV-2] or [PRV-3] candidate and publishes no checked program.
Its mechanical fix is a dominating `claim` of the residual [CLM-1] or a dominating branch establishing it [ENT-3].
Only after complete-state discharge succeeds may the constrained-subject gate replace that success with a [PRV-3] local-leaf rejection or retain a downstream demand for [PRV-2].
Discharge is a deterministic checker derivation [ENT-1]; a solver result never participates.
A `buffer<T>` obligation is over the runtime length term.
The offset atom has exact value mode and type `own u64`; after the [TYPE-7] implicit-read exclusivity, any other offset mode or type is a hard error citing OP-4 at the offset `atom` node, with `SourceCoordinate` equal to that atom's complete checked half-open source extent.
A subscript in a [SET-1] target forms the selected place without reading its stored value; its base and offset are evaluated during target evaluation, and its discharge judgment is identical in target position.
A successful bounds judgment neither narrows nor authorizes narrowing the offset or its scaled byte offset; target address formation additionally obeys [STOR-6].
The range validation of the system transfer operations [SYS-8] is an operation-internal contract check with table-fixed trap semantics [ERR-4] whose trap record uses the operation `call` node [DIAG-3]; the discharge judgment does not apply to it.
```

Byte-identical content modulo newline placement; the diff unit becomes the sentence (the v0.28 delta's "40 anchored edits across twenty existing rules" would have been ~40 one-line diffs).

**Sample S-rule — ENT-3.S10 (from the current single-line bullet at spec line 1187):**

```markdown
[ENT-3.S10]
views: complete unasserted s4-blinded
point: match-arm-entry
event: S10
For a `match_stmt` or `value_match` whose scrutinee is directly a call to `read_once`, `write_once`, `host_copy_bytes`, or `host_copy_utf8` [SYS-2, SYS-8], or a bare IDENT naming a `let` binding of the call's outcome type initialized by such a call under the same no-kill, no-`set` path discipline as S7's checked-arithmetic origin: with k the actual bound to the call's bounding parameter — `capacity` for `read_once`, `host_copy_bytes`, and `host_copy_utf8`; `count` for `write_once` — read as a term or constant, where no [ENT-5] kill event applies to a fact supported by k on the path to the match, the `ReadBytes(count: w)` arm of a `read_once` match and the `Ok(value: w)` arm of the other three establish w <= k at arm entry; every other arm establishes nothing.
These facts carry the same trust class as S6's allocation-length equality — a declared operation contract, never a writer statement.
The three [SYS-9] relations are retained checked-program facts and are not L0 fact sources in this version.
```

The envelope encodes exactly the schema the measurement report says is realistic — `(label, view mask, establishment point, event kind)` — and a new compiler test asserts it against `FlowEventKind` (`state.rs:68-87`) and `ProofView` (S12's `event:` maps to the `postcondition` group since no `S12` variant exists). Guards and relation construction stay prose, per the measured verdict "Term grammar: yes. Fact sources: no." This folds the sound half of option (c) into (b). S-rules become citable by conformance manifests and task records with real ids.

**Sample op-table rows (from spec lines 350–359):**

````markdown
[OP-1]
Every computation is a call naming one operation from the operation table; …
Domain tokens: `int` is every integer type T; `sint` is every signed integer type T; `float` is f32 and f64; `tag-enum` is one exact nominal tag-only enum T (every variant nullary), including `Bool`; …

```wf-ops
| op | domain | signature | effects |
| `+checked` `-checked` `*checked` | int | `(T, T) -> own Result<T, Overflow>` | pure |
| `ineg.checked` | sint | `(T) -> own Result<T, Overflow>` | pure |
```
````

All four columns become extraction-locked: the catalog test extends from column 1 to the full row, and `model.rs::traps()` / `scalar_result_type()` / `checked_error` become derived-or-locked instead of hand-mirrored — the 16-dead-arm class (B1) becomes a compile failure. Same treatment: PRV-1 (`wf-prov`, gains its first extraction test), PRE-1, SYS-2, DIAG-1's enumerations (`wf-diag` rows).

**Linter** (~250 lines added to `bin/spec.rs`, same scanning style it already uses): id/sub-id uniqueness and shape; every `[REF]` resolves (exists today) including sub-ids; no body line starts `[`; one sentence per line (line ends `.`/`:`/table row; warn >500 B as a conciseness ratchet); every fence info string is a known `wf-` schema and its column header matches; controlled-vocabulary cells draw from the owning rule's declared token list; retired labels never redefined; per-family rule counts, production counts, table-row counts **emitted** (killing hand arithmetic C3/B5); Status paragraph shape. Plus `--index`: JSON `{rule: {start, end, refs: […]}}` on stdout — a query, not a committed file (repo-hygiene law: no unowned artifacts).

- Migration cost: one mechanical transform (sentence split + header deletion + fence moves + sub-id insertion + vocab tokens), a one-shot verifier proving old→new content identity modulo the enumerated moves, one activation, one owner packet; extractor updates (`ebnf.rs` keys on fences) in the same batch. Estimate: one focused day of work plus the normal activation.
- Iteration-safety gain: B1, B2, B4, B5 dissolve; B3 shrinks to sentence-level anchors; B6 shrinks (ledger counts generated).
- Token cost of result: ≈375 KB immediately (−47 KB header, −~12 KB DIAG restructure, +~3 KB newlines/markers) ≈ 107k tokens; per-task effective reads 5–50 KB via index; ratchet target below.
- Review-diff quality: approval unit = sentence/row = semantic unit (the SpecTec lesson, at zero DSL cost).
- Risk: the migration activation is a large byte diff requiring exact owner approval — bounded by the mechanical-transform verifier; protected-surface edits (runner regex) enumerated in §6.

### (c) Executable-core split (L0 fragment as code/data, prose shell)

The measured evidence is against the strong form: S-rule fact sources need recursive closure queries, path-condition kills, five distinct establishment points, per-rule view masks, and load-bearing dispatch order — "not expressible as a row" (measurement §3). A core artifact beside the prose with its own lifecycle is precisely the RISC-V/KJS configuration the external report shows always drifts then freezes ("no part of one artifact is generated from any other… you write the same thing four times"). The defensible parts — term grammar as data, S-envelope schema, table extraction — are exactly option (b)'s fences and envelopes.
- Migration cost: high (entailment restructure). Safety gain: negative at the new prose↔core seam unless extraction-locked, at which point it *is* (b). Token cost: similar. Review: owner must approve two artifacts of different kinds. Risk: highest drift risk of any option. **Fold into (b); reject as standalone.**

### (d) Full DSL (SpecTec-shaped)

Wasm's precondition — rules were formal-first, prose derived — is absent here: Whitefoot's rules are prose-first with embedded near-formal fragments. SpecTec cost multiple expert-years and paid off because W3C requires four redundant artifacts per feature; Whitefoot maintains one. Its two genuinely valuable components (definition-use type checking; executable link to a conformance gate) are already delivered more cheaply by (b)'s linter + the existing compiler-and-conformance gate — Whitefoot's compiler *is* its ESMeta/meta-interpreter, already wired as the activation gate.
- Migration cost: multi-expert-year, new toolchain to maintain ("some expertise will have to grow… to maintain it over time"). Safety gain: highest ceiling, eventually. Token cost: best (Wasm 2.0 = 2,957 DSL LoC). Review: good (diff'able ASCII) but resets the whole approval-anchor history. Risk: the KJS/Standard-ML freeze — full formality with insufficient tooling ownership correlates with a frozen language. **Reject now; define a revisit gate (§4, Stage 3).**

### Scores (1–5 × weight)

| | K1×25 | K2×25 | K3×20 | K4×10 | K5×10 | K6×10 | Total /500 |
|---|---|---|---|---|---|---|---|
| (a) tooling only | 4 | 2.5 | 3.5 | 1 | 5 | 5 | 342 |
| **(b) markdown profile** | **5** | **4.5** | **4.5** | **4** | **5** | **3.5** | **452** |
| (c) core split | 3.5 | 3 | 3 | 3 | 3 | 2 | 302 |
| (d) full DSL | 4 | 5 | 2 | 5 | 2 | 1 | 345 |

---

## 4. Recommendation (staged)

### Stage 0 — this week, before any spec-byte change
1. `--index` and count-report emission in `whitefoot-spec` (offsets/refs it already computes and discards).
2. Generated identity: `whitefoot-spec --emit-identity` writes `compiler/src/spec_identity.rs` (version, hash bytes, chain length, rule count) with a `--check` byte-compare in `make -C compiler check` — the exact pattern `grammar_tables` already uses for `generated.rs`. Kills B5. The three `qualification.rs` tripwires collapse to one deliberate `REVIEWED_FOR` constant beside a single site (they are intentional per-activation review forcers; keep one, not three).
3. A `make` target that greps the remaining prose digest sites (README, roadmap, patterns, ledger) against the chain tail — checking, not generating, so docs stay ordinary prose. Kills E12's 8 unguarded sites.
4. **Candidate mode** (protected change, owner packet P1): `Status: CANDIDATE vN+1 supersedes vN <sha256>` recognized by `bin/spec.rs` and `spec-archive-integrity` — self-consistency always required; chain-tail equality required when ACTIVE, replaced by "supersedes == chain tail" when CANDIDATE; `runner.py` reads its pin from the APPROVALS chain tail (one protected edit now, zero per activation forever) and accepts a declared candidate the same way. Result: **`make check` runs fully green on a candidate branch before approval** — the single largest process-cost fix available (C1, C2 via fence-keyed contract extraction, E16).
5. Docs hygiene: plan/outline item ids stop using the bare `NAME-N` rule-id shape (prefix them, e.g. `plan:PROOF-8`) — fixes A5 at zero spec cost.

### Stage 1 — one migration activation (v0.30, owner packet P2)
The profile of §3(b), applied mechanically: sentence-per-line; `Prior:` deletion with pointer line; `[ENT-3.S1]`…`[ENT-3.S12]` sub-ids + `retired: S8`; all machine-read payloads into `wf-` fences (including the 4 inline productions — B2's confessed trap closes); op-table controlled vocabulary; DIAG-1 enumerations to rows; leaked plan Stage ids removed from spec text. Extractors re-keyed to fences in the same batch. The packet carries, as law requires, the exact SHA-256, diff, impact inventory, and verifier results — plus the one-shot transform verifier's proof that new content = old content modulo the enumerated moves (then the verifier is deleted, per one-shot law). **No semantic edits ride this activation.**

### Stage 2 — next 2–3 ordinary activations
Full-row extraction locks (op table columns 2–4, PRV-1, DIAG rows) as ordinary compiler tests; S-envelope assertion against `FlowEventKind`/`ProofView`; conciseness ratchet (below). Optional protected follow-up: conformance manifests may cite `ENT-3.S10`-grade ids. Separately authorized cleanup: delete `whitefoot-migrate` (C5).

### Stage 3 — explicitly gated, default no
L0 term grammar as canonical `wf-` data, or any DSL move, **only if** the proof-certificate direction someday needs machine-consumed rule ASTs — and then only in the generate-or-gate configuration; never a sibling artifact. Until that consumer exists, project law says it does not ship.

### The four required answers

**Normative when representations disagree.** There is exactly one normative representation: the structured `spec/kernel-spec.md` bytes, hash-chained as today. Every derived view (index output, identity module, grammar tables, catalogs, ledger join) is generated with byte-compare `--check` gates — disagreement is a red build, never a judgment call. Within the file, a `wf-` fence is normative for what it enumerates; the owning rule's prose is normative for meaning; a fence/prose conflict is a spec defect that stops the affected work (the existing spec/compiler-discrepancy rule applied intra-file), and the linter makes the mechanical class of such conflicts unrepresentable.

**How approval/digest attach.** Unchanged in every particular: exact-byte owner approval of the candidate file; SHA-256 `ACTIVE-SPEC:` chain in `governance/APPROVALS.md`; flat immutable archives; pre-commit hook. The profile is *inside* the bytes the owner already approves. Candidate mode changes when gates can be green, not what the owner approves.

**How 431 KB becomes concise.** Where the bytes go: −47 KB `Prior:` paragraphs (to the immutable archives that already hold them verbatim — pure deletion, supersede-in-place); −~12 KB DIAG-1/DIAG-2 prose scaffolding (to `wf-diag` rows); +~3 KB newlines/markers → **≈375 KB (~107k tokens) at Stage 1, mechanically**. Then a linter-enforced ratchet: untouched rules may not grow; touched rules replace restated hub content with bare refs (safe once refs are machine-resolved) → **target ≤300 KB (~86k tokens) within three activations**, long-run ≤250 KB. The bigger number for the primary reader: per-task context drops from "the 431 KB file" to the measured cited+1-hop set, **5–50 KB, retrieved by id through `--index`**.

**End-to-end change flow, and why it is simpler, step by step.**

| Step | Today (measured, v0.28) | New |
|---|---|---|
| Draft | Edit spec → tree red on ≥8 checks for 21 h 36 m; candidate lives on a never-merged held branch or as uncommitted state | Edit rule blocks; `spec-lint` green locally; commit `Status: CANDIDATE` on an ordinary branch — tree green |
| Verify | Focused tests only; grammar verifier structurally rejects the candidate (`ChangedFrontendContract`); first full gate only after activation | `make spec-sync && make check` — the full 50-min gate runs **before** review; verifier reads the candidate's own fences |
| Review | 43 zero-context hunks; three frozen-byte reviews doing rule/grammar arithmetic by hand | Per-sentence/per-row diffs; counts and impact inventory emitted by the linter |
| Approve | Packet with 14 hand-transcribed SHAs | Same law, tool-emitted digests and inventory; owner still approves exact bytes |
| Activate | 99-file commit; 11 digest sites + ~8 scalars by hand; branch-topology dance, one attempt discarded | Flip CANDIDATE→ACTIVE, append chain line, archive outgoing bytes, run `--emit-identity` — one commit, green on both sides, zero hand-computed values |

Each step loses its dominant toil term; no step gains a new artifact to maintain.

---

## 5. What NOT to adopt

1. **SpecTec-style DSL cutover** — law: no frameworks a current experiment doesn't need; the formal-first precondition is absent; the payoff existed only under a four-redundant-artifact standards burden Whitefoot doesn't carry.
2. **A prose-generation backend** (English pseudocode from rules) — the primary reader is AI; regenerating human prose is the most expensive SpecTec component and serves no reader here.
3. **A sibling formal model** (Coq/Lean/K/Sail-style, or a second "executable spec" in Rust) — every negative case in the external evidence (KJS, JSCert, Wasm's provers, Cerberus, SML) is the same failure: a parallel formal artifact under separate cadence drifts, then freezes. Anything formal must generate or gate from day one; the compiler + conformance gate already occupy that role.
4. **JSON/YAML/TOML container** — new parser dependency in a zero-dependency workspace (verified), escaped/indent-scoped prose ruins diffs and tokens, and markdown is the LLM-native surface.
5. **Rust-macro DSL spec inside the compiler** — inverts "the specification defines the language; compiler behavior does not"; the compiler-independent conformance runner could no longer read the normative artifact.
6. **A second machine-readable database beside the spec** (riscv-unified-db pattern) — the structured file *is* the database; the derivation ledger's mapping becomes a generated join, not a rival source of truth.
7. **Committed index/atlas files without `--check` wiring** — repo-hygiene law: material with no owner and no reader is rot; the index is a query (`--index`), not a file.
8. **Semantic rewording inside the migration activation** — the migration must be provably meaning-preserving or the owner cannot review it; conciseness arrives via the ratchet, rule by rule.
9. **File-per-rule or per-family splitting** — it multiplies the digest/approval surface, breaks every `^\[ID\]` anchor and the flat-archive model, and solves nothing the index doesn't solve at 133 rules.
10. **Touching the archive-immutability model** — it is the one representation-independent piece with a perfect record (zero landed digest mistakes in the mined window). Keep the hook, the flat archives, and the chain exactly as they are.

---

## 6. Protected-approval inventory for the lead (to batch owner packets)

- **P1 (gates/wiring, before Stage 1):** candidate mode in `compiler/src/bin/spec.rs` + `Makefile spec-archive-integrity`; `runner.py` pin-from-chain + rule-id regex extended for `[FAM-N.Sk]` sub-ids. All are canonical-gate wiring → owner approval with exact before/after.
- **P2 (spec bytes, Stage 1):** the v0.30 migration activation — full SHA-256, diff, impact inventory, grammar-verifier result, transform-verifier proof of content preservation.
- **Ordinary (no owner packet):** index/identity emission (additive flags — flag to owner inside P1 anyway since they touch a gate binary), extraction-lock compiler tests, S-envelope assertions, linter checks, `spec-lint` target.
- **Optional later protected:** conformance-manifest citation of S-sub-ids; `whitefoot-migrate` deletion (gate-wiring removal).

Evidence provenance: current-state numbers and incident table from the three supplied research inputs; independently re-verified this session — `wc -c spec/kernel-spec.md` → 431650; longest line = line 15 at 10,699 B (v0.23 `Prior:` paragraph); OP-4 full text at line 421 (2,580 B single line); S1–S12 bullets at lines 1179–1189 + S8 retirement at 1191; op-table rows at lines 350–363; `compiler/Cargo.toml` → no `[dependencies]` section; `bin/spec.rs:28-70` → dependency-free line/bracket scanners. Key file paths: `/Users/bytedance/code/Whitefoot/spec/kernel-spec.md`, `/Users/bytedance/code/Whitefoot/compiler/src/bin/spec.rs`, `/Users/bytedance/code/Whitefoot/compiler/src/bin/grammar_tables/ebnf.rs`, `/Users/bytedance/code/Whitefoot/compiler/src/resolution/catalog.rs`, `/Users/bytedance/code/Whitefoot/compiler/src/semantic/provenance.rs`, `/Users/bytedance/code/Whitefoot/compiler/src/semantic/entailment/flow/sources.rs`, `/Users/bytedance/code/Whitefoot/tests/conformance/runner.py`, `/Users/bytedance/code/Whitefoot/Makefile`, `/Users/bytedance/code/Whitefoot/governance/APPROVALS.md`.

## Correction (2026-08-18, measured)

The conciseness ratchet's targets are RETIRED. The batch-0070 measurement
(786-check verifier, spec-ratchet investigation): cross-rule verbatim
restatement is 1.1% of the spec, tabulating DIAG-1's enumerations grows it
(+386 B), and the defensible whole-pass yield was -161 B. The balloon rules
are dense original content; this document's §D2 byte story ("agents restate
hub content defensively") is refuted as a size driver. Size control remains
a growth-watch (per-rule top movers in review packets), not a shrink
program. The ≤300 KB and -12 KB DIAG-1 figures above are superseded.
