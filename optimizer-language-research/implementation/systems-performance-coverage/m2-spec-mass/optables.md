# Appendix S — Sealed-form catalog (normative op tables): `seq`, `table`, `conc_queue`

Status: catalog excerpt, authored under the sealed-form mechanism delta. Rule IDs are stable; diagnostics cite rule IDs [DIAG-1]. OPEN-FLAG items are recorded delta obligations [META-5], not normative content; a row conditioned on an OPEN-FLAG does not ratify before its flag is discharged.

## S.0 Conventions (every table row below fixes all nine columns)

[CAT-1] Columns. `signature`: explicit modes, region params, operands (op-table meta-notation, the OP-1 style; not `fn_decl` grammar). `own`: owned-value movement — which operands are consumed, which returns are own-out, which internal drops occur (all internal drops are compiler-derived and artifact-surfaced [STOR-3]). `loan`: `NONE`; `ISSUE &'r` / `ISSUE &uniq 'e` (the result is a kernel borrow-loan on the receiver); `ISSUE tok:t` (the result is an affine confined token of sealed type t carrying the stated loan); `CONSUME p` (the op consumes token operand p). `effects`: an [EFF-1] row in canonical order; `+ join(F)` per [CAT-5]. `failure`: `—` (total); `trap "msg"` (contract violation; deterministic abort [SCOPE-4, ERR-4], report cites this row's rule); `Result: E` (environmental outcome as a value). One row never mixes channels for one condition. `facts`: postconditions entering the requires engine. `kills`: fact families that die. `cg`: a codegen-contract mark defined in the form's CG list, or `—`.

[CAT-1a] Call-site grammar. The signature columns above are op-table meta-notation, not program text; a program call is spelled as follows. Type arguments go in angle brackets in the row's declaration order; region arguments appear in brackets only at a loan-minting call whose row has a dedicated result-region parameter, and are never written for a region the receiver binding already fixes; the only generics supplied are the op-level binders — receiver-fixed `K`, `V`, `h`, `N`, `M`, `IN` come from the receiver's type and are never respelled or reordered at the call; operand order equals the row's operand order. Worked examples, one per family: non-loan op `seq_push(s, move v)` (`'e`, `T`, `N` all fixed by `s`); lookup/loan op `tbl_get(t, k)` and `tbl_get_uniq(t, k)` (the value loan issues at the receiver's region — no region bracket, no `K`/`V`/`h` respelling); iteration/conformer op `seq_for_each(s, f)` with `f: &uniq 'v F` and `F` conforming `SeqVisit<T>`; dedicated-result-region op, the sole shape that writes a region argument, the C1 pattern `pool_entry<Entry, 'v>(np, h)` inside an enclosing `region 'v { }`. [M5-FIX-5]

Negative examples (each rejected; legal spelling follows) [M5R2-FIX-4]: ~~`seq_push<u64, 0>(s, move v)`~~ — respells the receiver-fixed `T`,`N`, which come from `s`'s type; write `seq_push(s, move v)`. ~~`seq_push['e](s, move v)`~~ — a region argument at a row whose loan column is `NONE`; region arguments appear only at a dedicated-result-region row (the `pool_entry<Entry, 'v>(np, h)` pattern), never here; write `seq_push(s, move v)`.

The receiver-fixed-generic suppression above applies to TABLE OPS ONLY [M5R3-FIX-4]. A call resolving to a user `fn` instead states ALL its type, region, and const arguments explicitly [TYPE-5] in one `<...>` targ list (generics and consts first, then regions, matching the `fn` declaration order `<generics>['regions]`), with named value arguments in declared order [GRAM-11]; see the worked example in cards.md.

[CAT-2] Fact families and notation. Facts attach to a live binding; moving or dropping the binding kills all its facts.

| family | form | meaning |
|---|---|---|
| LEN(x) | `len(x) = v` | length equals v |
| CAP(x) | `cap(x) = v` | capacity equals v |
| SLACK(x) | `cap_slack(x) >= n` | at least n further insertions need no growth/rehash and no allocation |
| INB(x, i) | `in_bounds(x, i)` | `i < len(x)` |
| ELEM(x) | content facts | facts the engine keys on element content reached through x |
| HAS(t, k) | `contains(t, k) = b` | key membership; k named per [TBL-6] |
| OCC(e) | `occupied(e) = b` | entry-token slot state |

`LEN(x) += 1` rewrites a live LEN fact; `SLACK dec1` rewrites each `cap_slack(x) >= n` to `>= n-1` (the `n = 0` form dies); `SLACK inc1` symmetrically. SLACK facts survive length-decreasing ops (monotone truth); INB facts survive length-increasing ops. A passed implicit check produces its fact on the dominated path exactly as [OP-5]. Proof may elide only what a row marks elidable; there is no source-level weakening (PATTERNS P8).

[CAT-3] Sealed opaque types are lowercase keyword-class type spellings (the `box`/`buffer` precedent [TYPE-2]). Companion result/error types are ordinary TYPEID enums so `match` dispatch [ERR-1] applies; their declarations below are counted catalog mass and their variant names obey [TYPE-6] global uniqueness.

[CAT-4] Tokens. `ISSUE tok:t` returns an affine own value of confined sealed type t holding the stated loan on the receiver; while the token is live the receiver is frozen per the per-binding loan judgment. Dropping an unconsumed token releases the loan, drops its captured owned operands (surfaced), and performs no data-structure write.

[CAT-5] Iteration protocol. Internal `for_each`-family ops with a monomorphized, must-inline visitor are the default; visitor `visit` returning `False()` stops iteration and the op returns `False()` (ran-to-end returns `True()`). OPEN-FLAG (EFFECT-JOIN) — narrowed by [CAT-5a](iii)/[M5R2-FIX-1]: the join is now defined (the op's stated row unioned with the conformer's declared member row) and writable by hand; what remains kernel-delta is only whether FN-3's checker computes that union automatically at the call site or the caller states the unioned row explicitly — the source spelling is identical either way.

[CAT-5a] Effect-row rules (restating [EFF-1] for catalog writers). (i) Canonical clause order is `reads, writes, allocates, traps`; `pure` is the empty row [EFF-1]; a row out of order is malformed. (ii) A `fn`'s declared row must cover the union of the rows of every op and fn it calls: every callee `reads`/`writes`/`allocates`/`traps` appears in the caller's row (a trapping op contributes `traps`), while effects confined to a region minted by an internal `region 'x { }` block do not escape into the row. (iii) Conformer rows (RESOLVED — replaces the former OPEN-FLAG (EFFECT-JOIN)). Every contract member in S.1/S.2 declares a CEILING effect row [EFF-1]. A `conform` binds a `fn` whose declared row (a) equals exactly what its body exhibits [EFF-2, both directions] and (b) is contained in that member's ceiling — conformance is by subsumption, the "at most" discipline of [FN-7], so a conformer may exhibit LESS than the ceiling but never more (a non-trapping predicate conforms to a `traps`-carrying ceiling; correcting the round-2 "exact-match" statement). An op row carrying `+ join(F)` has effective row at a call site = its stated clauses (dropping `+ join(F)`) unioned with the CONCRETE conformer's declared row, known at monomorphization [FN-2]; the caller writes that union as its own declared row. Region names in any row obey the binding discipline of cards.md C1 [M5-FIX-3]. [M5R2-FIX-1] [M5R3-FIX-7]

[CAT-6] OPEN-FLAG (ALLOC-ERR): kernel v0.6 treats OOM as TCB-level [OP-9, SCOPE-3]. The two recoverable-admission ops (`seq_reserve`, `tbl_reserve`) return `Result<unit, AllocError>`, requiring the prelude addition `enum AllocError { AllocFailure(); }` and an owner ruling. Every other allocating row keeps the TCB stance: its OOM is not a language trap and not a Result.

[CAT-7] Every op IDENT in this appendix is RESERVED per the [OP-1] discipline.

[CAT-7a] Scalar ops are not re-listed here: integer arithmetic, comparison, bitwise, and count ops are rows of the kernel operation tables [OP-2, OP-7] and are callable directly. The subset visible in the cards (`ieq`, `ige`, `iand`, `ipopcount`, `iadd.wrap<u64>`, `isub.wrap<u64>`) is illustrative, not the closed set: the kernel defines the full compare set `ieq ine ilt ile igt ige` (dotless, `(T, T) -> own Bool`), the moded arithmetic families `iadd`/`isub`/`imul` with `.wrap`/`.trap`/`.checked`/`.sat` suffixes and `idiv`/`irem` with `.trap`/`.checked` (there is no bare `iadd`/`isub` — every integer add or subtract carries an explicit overflow mode), bitwise `iand ior ixor`, and counts `ipopcount iclz ictz`. Strict-less-than is `ilt` written directly. An op spelling in neither the kernel tables nor this appendix is a hard error [DIAG-1], never a license to invent. [M5-FIX-2]

[CAT-7b] Dotless is not typeless [M5R4-FIX-2]. A receiver-less generic scalar op (`ieq`, `ine`, `ilt`, `ile`, `igt`, `ige`, `iadd.wrap`, `isub.wrap`, `iand`, ...) has NO receiver to fix its type argument, so it ALWAYS takes an explicit `<T>` at the call — `ieq<u64>(n, 0_u64)`, never `~~ieq(n, 0_u64)~~` [TYPE-5, FN-2: call sites state all type args explicitly]. The [CAT-1a] receiver-fixed-generic suppression is TABLE-OPS-ONLY (it drops only the `T`/`N`/`K`/`h` a CONTAINER receiver already fixes); it never licenses dropping a scalar op's type arg. The absence of a `.mode` suffix (a "dotless" op name) is a mode-axis fact and says nothing about type arguments — dotless is not typeless.

---

## S.1 `seq<T, inline N>` — growable sequence

[SEQ-1] Form. `seq<T, N>`: T any owned type (affine legal); N a constant-expression [CONST-1] inline capacity (0 legal). Spare capacity is uninitialized form-internal storage: an index `>= len` is not a place, so uninitialized reads are unrepresentable.

[SEQ-2] Inline mode and spill. `cap(s) >= N` always. While unspilled, the store is frame-resident [STOR-1]. The first growth past the current capacity moves all elements to one heap store (spill). Spill is one-way: `pop`/`truncate`/`clear` never return to inline; `seq_take_all` transfers the spilled store to its result and resets the receiver to the empty inline state (len 0, cap N).

[SEQ-3] Growth policy. A required capacity `c > cap` sets the new capacity to the least power of two `>= max(2 * cap, c, 4)`. The byte size `cap * sizeof(T)` is computed in u64; overflow traps `"seq capacity overflow"` before allocating (the [OP-9] discipline). OOM per [CAT-6].

[SEQ-4] Amortized-cost contract. Total element moves due to growth are `<= 2 x` the number of elements ever appended; per-op cost beyond that is O(1) except: `insert_at`/`remove_at` O(len - i) element moves, `truncate` O(drops), `drain`/`extend` O(elements processed).

[SEQ-5] Drop. Dropping a `seq` drops live elements in descending index order, then frees the spilled store; all surfaced [STOR-3].

[SEQ-6] Fact discipline. Structure facts (LEN/CAP/SLACK/INB) are killed only as rows state; element-writing, element-moving, or element-dropping rows kill ELEM(s). Growth inside a row is value-preserving, so ELEM facts survive reallocation.

Visitor contracts (normative bytes; each member declares a CEILING effect row per [CAT-5a]/[M5R2-FIX-1] — a conformer's declared row is contained in the ceiling (subsumption, [FN-7]) and equals what its body exhibits; the `traps` in the visit/keep ceilings lets a conformer use a bounds-checked `index` [M5R3-FIX-7]; OPEN-FLAG (SELF-SPELLING): FN-3 does not yet fix the conforming-type spelling `Self` inside `fn_sig`):

```
contract SeqVisit<T> { fn visit['v](env: &uniq 'v Self, item: &'v T) -> own Bool reads('v), writes('v), traps; }
contract SeqVisitUniq<T> { fn visit['v](env: &uniq 'v Self, item: &uniq 'v T) -> own Bool reads('v), writes('v), traps; }
contract SeqSink<T> { fn take['v](env: &uniq 'v Self, item: own T) -> own unit writes('v), allocates(heap), traps; }
```

| op | signature | own | loan | effects | failure | facts | kills | cg |
|---|---|---|---|---|---|---|---|---|
| `seq_new` | `() -> own seq<T, N>` | returns own | NONE | pure | — | LEN(r)=0; SLACK(r)>=N; CAP(r)=N | — | — |
| `seq_with_cap` | `(n: u64) -> own seq<T, N>` | returns own | NONE | allocates(heap), traps | trap `"seq capacity overflow"` [SEQ-3] | LEN(r)=0; SLACK(r)>=n | — | — |
| `seq_reserve` | `['e](s: &uniq 'e seq<T, N>, n: u64) -> own Result<unit, AllocError>` | — | NONE | writes('e), allocates(heap), traps | trap `"seq capacity overflow"`; Result: AllocError [CAT-6] | Ok arm: SLACK(s)>=n | CAP(s) | — |
| `seq_push` | `['e](s: &uniq 'e seq<T, N>, v: own T) -> own unit` | consumes v | NONE | writes('e), allocates(heap), traps | trap `"seq capacity overflow"` (growth path only) | LEN(s)+=1; from live `len(s)=v`: INB(s, v) | CAP(s); SLACK dec1 | CG-PUSH |
| `seq_pop` | `['e](s: &uniq 'e seq<T, N>) -> own Option<T>` | Some payload own-out | NONE | writes('e) | — (empty is `None()`, a value) | None arm: LEN(s)=0; Some arm: LEN(s)-=1, SLACK inc1 | INB(s,\*); ELEM(s) | CG-INL |
| `seq_len` | `['r](s: &'r seq<T, N>) -> own u64` | — | NONE | reads('r) | — | LEN(s)=r | — | CG-INL |
| `seq_cap` | `['r](s: &'r seq<T, N>) -> own u64` | — | NONE | reads('r) | — | CAP(s)=r | — | CG-INL |
| `seq_get` | `['r](s: &'r seq<T, N>, i: u64) -> &'r T` | — | ISSUE &'r (element) | reads('r), traps | trap `"seq index out of bounds"`; elidable by INB(s, i) proof [OP-4] | INB(s, i) on the pass path | — | CG-GET |
| `seq_get_uniq` | `['e](s: &uniq 'e seq<T, N>, i: u64) -> &uniq 'e T` | — | ISSUE &uniq 'e (element) | writes('e), traps | trap `"seq index out of bounds"`; elidable by INB(s, i) proof | INB(s, i) | ELEM(s) | CG-GET |
| `seq_set` | `['e](s: &uniq 'e seq<T, N>, i: u64, v: own T) -> own unit` | consumes v; prior element dropped (surfaced) | NONE | writes('e), traps | trap `"seq index out of bounds"`; elidable by INB(s, i) proof | INB(s, i) | ELEM(s) | CG-INL |
| `seq_replace` | `['e](s: &uniq 'e seq<T, N>, i: u64, v: own T) -> own T` | consumes v; prior element returned own-out | NONE | writes('e), traps | trap `"seq index out of bounds"`; elidable by INB(s, i) proof | INB(s, i) | ELEM(s) | CG-INL |
| `seq_swap` | `['e](s: &uniq 'e seq<T, N>, i: u64, j: u64) -> own unit` | — | NONE | writes('e), traps | trap `"seq index out of bounds"` (either index); elidable by INB(s, i) and INB(s, j) proofs | INB(s, i); INB(s, j) | ELEM(s) | CG-INL |
| `seq_insert_at` | `['e](s: &uniq 'e seq<T, N>, i: u64, v: own T) -> own unit` | consumes v | NONE | writes('e), allocates(heap), traps | trap `"seq position out of bounds"` (i > len); trap `"seq capacity overflow"` | LEN(s)+=1 | CAP(s); ELEM(s); SLACK dec1 | — |
| `seq_remove_at` | `['e](s: &uniq 'e seq<T, N>, i: u64) -> own T` | removed element own-out | NONE | writes('e), traps | trap `"seq index out of bounds"`; elidable by INB(s, i) proof | LEN(s)-=1; SLACK inc1 | INB(s,\*); ELEM(s) | — |
| `seq_truncate` | `['e](s: &uniq 'e seq<T, N>, n: u64) -> own unit` | surplus elements dropped, descending index (surfaced) | NONE | writes('e), traps | trap `"seq truncate beyond length"` (n > len); elidable by proof `n <= v` from live LEN(s)=v | LEN(s)=n | INB(s,\*); ELEM(s) | — |
| `seq_clear` | `['e](s: &uniq 'e seq<T, N>) -> own unit` | all elements dropped, descending index (surfaced) | NONE | writes('e) | — | LEN(s)=0; from live CAP(s)=v: SLACK(s)>=v | INB(s,\*); ELEM(s) | — |
| `seq_drain` | `<F: SeqSink<T>>['e, 'v](s: &uniq 'e seq<T, N>, a: u64, b: u64, f: &uniq 'v F) -> own unit` | each element in [a, b) own-out to `F::take`, ascending; tail shifts down | NONE | writes('e), writes('v), traps + join(F) [CAT-5] | trap `"seq drain range invalid"` (unless a <= b <= len) | LEN'(s) = LEN(s) - (b - a) | INB(s,\*); ELEM(s) | CG-ITER |
| `seq_take_all` | `['e](s: &uniq 'e seq<T, N>) -> own seq<T, N>` | whole contents own-out; s reset to empty inline state [SEQ-2] | NONE | writes('e) | — | LEN(s)=0; SLACK(s)>=N; CAP(s)=N; from live `len(s)=v`: LEN(r)=v | all prior facts on s superseded; INB(s,\*); ELEM(s) | — |
| `seq_as_slice` | `['r](s: &'r seq<T, N>) -> own slice<'r, T>` | — | ISSUE &'r (whole initialized prefix) | pure | — | LEN(r)=LEN(s) (link) | — | CG-INL |
| `seq_as_uniq_slice` | `['e](s: &uniq 'e seq<T, N>) -> own uslice<'e, T>` | — | ISSUE &uniq 'e (whole initialized prefix) | writes('e) | — | LEN(r)=LEN(s) (view is fixed-length) | ELEM(s) | CG-INL |
| `seq_extend_move` | `['e](s: &uniq 'e seq<T, N>, src: own seq<T, M>) -> own unit` | consumes src; all elements moved in order; src store freed (surfaced) | NONE | writes('e), allocates(heap), traps | trap `"seq capacity overflow"` | from live LEN facts: LEN'(s)=LEN(s)+LEN(src) | CAP(s); SLACK dec len(src) (killed when unknown) | CG-BULK |
| `seq_extend_copy` | `['e, 'x](s: &uniq 'e seq<T, N>, src: slice<'x, T>) -> own unit` | T copy required (compile reject otherwise, cites this row) | NONE | reads('x), writes('e), allocates(heap), traps | trap `"seq capacity overflow"` | from live LEN facts: LEN'(s)=LEN(s)+LEN(src) | CAP(s); SLACK dec len(src) (killed when unknown) | CG-BULK |
| `seq_for_each` | `<F: SeqVisit<T>>['r, 'v](s: &'r seq<T, N>, f: &uniq 'v F) -> own Bool` | — | NONE | reads('r), writes('v) + join(F) [CAT-5] | — | — | — | CG-ITER |
| `seq_for_each_uniq` | `<F: SeqVisitUniq<T>>['e, 'v](s: &uniq 'e seq<T, N>, f: &uniq 'v F) -> own Bool` | — | NONE | writes('e), writes('v) + join(F) [CAT-5] | — | — | ELEM(s) | CG-ITER |

OPEN-FLAG (UNIQ-VIEW): `seq_as_uniq_slice` requires a fixed-length unique view type `uslice<'e, T>`; kernel v0.6 defines no uniq-mode slice type. The row is conditional on that delta.

CG marks (all CI-pinned in the codegen corpus; a pin regression fails the gate):
- **CG-PUSH** (`seq_push`, designated hot op): guaranteed-inline at every call site. Fast path is exactly: one slack compare + branch, one element store (memcpy of `sizeof(T)` for composite T), one length increment. Growth is one out-of-line cold call. Under a live `SLACK(s) >= 1` fact the compare + branch are elided (proof-elision of a non-safety branch; the safety story is unchanged).
- **CG-GET** (`seq_get`, `seq_get_uniq`): guaranteed-inline; bounds compare + trap branch + address computation; under a discharged INB proof, address computation only.
- **CG-INL**: guaranteed-inline; exact shape pinned per op.
- **CG-ITER**: the visitor member is monomorphized and must-inline (no call instruction survives); the loop is single-latch; with a trap-free, `pure`-joined visitor the loop is vectorization-eligible (P8: no willreturn poison).
- **CG-BULK**: element transfer lowers to one contiguous copy region (memcpy shape) after at most one growth.

---

## S.2 `table<K, V, h>` — SwissTable map

[TBL-1] Form and key kinds. `key_kind` is derived from K, closed: **copy** (K a primitive or tag-only enum) or **bytes** (K = `buffer<u8>`); any other K is a compile-time reject citing TBL-1. Spellings used below: KIN (lookup key) — copy: `k: K` (bare copy); bytes: `k: slice<'k, u8>` (rows taking KIN add `['k]`). KOWN (stored key) — copy: `k: K`; bytes: `k: own buffer<u8>`. Key equality is value equality (copy) or byte equality (bytes). V is any owned type (affine legal).

[TBL-2] Hasher. `h` is a closed set: `fold`, `sip_keyed`, `identity`, `crc`. There is no default (META-2): h is always written. `identity` is legal only for copy integer K (compile reject otherwise, cites TBL-2). `sip_keyed` is the only hasher with instance state (k0, k1 supplied at creation). Hash of equal keys is equal within an instance. Iteration order is deterministic for identical instance histories [DIAG-1] but is not a fact channel and not stable across catalog revisions; a program encoding order assumptions is defective.

[TBL-3] Layout and probe (normative for CG-PROBE). Bucket counts are powers of two `>= 16`. Each bucket has one control byte; H2 = low 7 hash bits (stored in the control byte); H1 = remaining bits, selecting the start group; a group is 16 control bytes; probing is the triangular group sequence. An empty table owns no allocation (a shared static empty group); the first insert allocates.

[TBL-4] Rehash. An insertion that would push occupied control bytes (live + tombstones) past `floor(7/8 * buckets)` rehashes first: in place (tombstone purge) when `live <= buckets / 2`, else into `2 * buckets`. `remove` writes a tombstone; storage never shrinks in place. Insert and lookup are amortized O(1) [SEQ-4 discipline]. Rehash preserves membership: HAS facts survive rehash; CAP/SLACK die per rows. The bucket-array byte size is computed in u64; overflow traps `"table capacity overflow"` before allocating.

[TBL-5] `cap_slack(t) >= n` means: n further inserts proceed with no rehash and no allocation. An insert that lands on a tombstone may reuse it; SLACK dec1 remains conservative.

[TBL-6] Fact keys. A HAS fact names its key through a live binding; the fact dies when that binding dies or its bytes are written. A bytes key moved into `tbl_insert`/`tbl_entry` is consumed, so no HAS fact is producible from that operand (rows state "if keyable").

[TBL-7] Entry-token drop. Dropping an unconsumed `entry` token releases the loan and drops the captured key (surfaced); it writes nothing [CAT-4].

Visitor contracts (KIN/KOWN abbreviate the TBL-1 spellings; OPEN-FLAG (SELF-SPELLING) as in S.1):

```
contract TblVisit<K, V> { fn visit['v](env: &uniq 'v Self, key: KIN<'v>, value: &'v V) -> own Bool reads('v), writes('v), traps; }
contract TblVisitUniq<K, V> { fn visit['v](env: &uniq 'v Self, key: KIN<'v>, value: &uniq 'v V) -> own Bool reads('v), writes('v), traps; }
contract TblRetain<K, V> { fn keep['v](env: &uniq 'v Self, key: KIN<'v>, value: &'v V) -> own Bool reads('v), traps; }
contract TblSink<K, V> { fn take['v](env: &uniq 'v Self, key: own KOWN, value: own V) -> own unit writes('v), allocates(heap), traps; }
```

| op | signature | own | loan | effects | failure | facts | kills | cg |
|---|---|---|---|---|---|---|---|---|
| `tbl_new` | `() -> own table<K, V, h>`; for h = sip_keyed: `(k0: u64, k1: u64) -> own table<K, V, h>` (table data per TBL-2) | returns own | NONE | pure | — | LEN(r)=0 | — | — |
| `tbl_with_cap` | `(n: u64) -> own table<K, V, h>`; sip_keyed adds `k0, k1` | returns own | NONE | allocates(heap), traps | trap `"table capacity overflow"` [TBL-4] | LEN(r)=0; SLACK(r)>=n | — | — |
| `tbl_reserve` | `['e](t: &uniq 'e table<K, V, h>, n: u64) -> own Result<unit, AllocError>` | — | NONE | writes('e), allocates(heap), traps | trap `"table capacity overflow"`; Result: AllocError [CAT-6] | Ok arm: SLACK(t)>=n | CAP(t) | — |
| `tbl_insert` | `['e](t: &uniq 'e table<K, V, h>, k: KOWN, v: own V) -> own Option<V>` | consumes k and v; prior value own-out as `Some`; on occupied, the stored key is retained and the incoming bytes key is dropped (surfaced) | NONE | writes('e), allocates(heap), traps | trap `"table capacity overflow"` (rehash growth only) | HAS(t, k)=True (if keyable [TBL-6]); None arm: LEN(t)+=1, SLACK dec1; Some arm: LEN survives | CAP(t) (None arm); ELEM(t); other-key HAS survive | CG-PROBE |
| `tbl_contains` | `['r](t: &'r table<K, V, h>, k: KIN) -> own Bool` | — | NONE | reads('r) | — | HAS(t, k)=r | — | CG-PROBE |
| `tbl_get` | `['r](t: &'r table<K, V, h>, k: KIN) -> &'r V` | — | ISSUE &'r (value) | reads('r), traps | trap `"table key absent"`; elidable by HAS(t, k)=True proof (this row's [OP-4] analog) | HAS(t, k)=True on the pass path | — | CG-PROBE |
| `tbl_get_uniq` | `['e](t: &uniq 'e table<K, V, h>, k: KIN) -> &uniq 'e V` | — | ISSUE &uniq 'e (value) | writes('e), traps | trap `"table key absent"`; elidable by HAS(t, k)=True proof | HAS(t, k)=True on the pass path | ELEM(t); structure facts survive (value writes cannot change membership) | CG-PROBE |
| `tbl_entry` | `['e](t: &uniq 'e table<K, V, h>, k: KOWN) -> own entry<'e, K, V>` | captures k (consumed into the token) | ISSUE tok:`entry<'e, K, V>` (unique loan on t; t frozen while live) | reads('e) | — | — | — | — |
| `entry_occupied` | `['x](e: &'x entry<'e, K, V>) -> own Bool` | — | NONE | reads('x) | — | OCC(e)=r | — | CG-INL |
| `entry_fill` | `(e: own entry<'e, K, V>, v: own V) -> own unit` | consumes v; inserts captured key + v | CONSUME e | writes('e), allocates(heap), traps | trap `"entry occupied"`; elidable by OCC(e)=False proof; trap `"table capacity overflow"` | LEN(t)+=1; SLACK dec1; HAS(t, k)=True (if keyable) | CAP(t); ELEM(t) | — |
| `entry_replace` | `(e: own entry<'e, K, V>, v: own V) -> own V` | consumes v; prior value own-out; stored key retained; captured key dropped (surfaced) | CONSUME e | writes('e), traps | trap `"entry vacant"`; elidable by OCC(e)=True proof | LEN survives | ELEM(t) | — |
| `entry_remove` | `(e: own entry<'e, K, V>) -> own V` | value own-out; stored key and captured key dropped (surfaced); tombstone written [TBL-4] | CONSUME e | writes('e), traps | trap `"entry vacant"`; elidable by OCC(e)=True proof | LEN(t)-=1; HAS(t, k)=False (if keyable) | ELEM(t) | — |
| `tbl_remove` | `['e](t: &uniq 'e table<K, V, h>, k: KIN) -> own Option<V>` | value own-out as `Some`; stored key dropped (surfaced); tombstone written | NONE | writes('e) | — (absent is `None()`, a value) | HAS(t, k)=False; Some arm: LEN(t)-=1 | ELEM(t); other-key HAS survive; SLACK survives [TBL-5] | CG-PROBE |
| `tbl_len` | `['r](t: &'r table<K, V, h>) -> own u64` | — | NONE | reads('r) | — | LEN(t)=r | — | CG-INL |
| `tbl_retain` | `<F: TblRetain<K, V>>['e, 'v](t: &uniq 'e table<K, V, h>, f: &uniq 'v F) -> own unit` | `keep = False()`: stored key and value dropped (surfaced), tombstone written | NONE | writes('e), writes('v) + join(F) [CAT-5] | — | — | LEN(t); HAS(t, \*); ELEM(t); SLACK survives | CG-ITER |
| `tbl_drain` | `<F: TblSink<K, V>>['e, 'v](t: &uniq 'e table<K, V, h>, f: &uniq 'v F) -> own unit` | every key and value own-out to `F::take`, scan order [TBL-2]; control bytes reset, buckets retained | NONE | writes('e), writes('v) + join(F) [CAT-5] | — | LEN(t)=0 | HAS(t, \*); ELEM(t) | CG-ITER |
| `tbl_for_each` | `<F: TblVisit<K, V>>['r, 'v](t: &'r table<K, V, h>, f: &uniq 'v F) -> own Bool` | — | NONE | reads('r), writes('v) + join(F) [CAT-5] | — | — | — | CG-ITER |
| `tbl_for_each_uniq` | `<F: TblVisitUniq<K, V>>['e, 'v](t: &uniq 'e table<K, V, h>, f: &uniq 'v F) -> own Bool` | — | NONE | writes('e), writes('v) + join(F) [CAT-5] | — | — | ELEM(t) | CG-ITER |

[TBL-8] Entry occupied-path value access (interim, pending the loan-judgment mechanism behind OPEN-FLAG (BORROW-OPTION)). v0 exposes no one-probe accessor that reads or mutates the value already stored at an occupied entry: `entry_occupied` returns only a `Bool`, and `entry_replace`/`entry_remove` consume the token and bind the incoming value before the stored one is readable. Until a borrowing occupied-value row lands, do NOT trap, no-op, or invent a row — the sanctioned fallback is the two-probe spelling: `tbl_contains(t, k)`, then on `True` `tbl_get_uniq(t, k)` to read or mutate the value in place (the second probe's `"table key absent"` trap is elided by the `HAS(t, k)=True` fact the first produced [OP-4]); when the new value is computed from the old, `tbl_remove(t, k)` (or `entry_remove`) followed by `tbl_insert`/`entry_fill` with the recomputed value. This is a documented working shape, not a new op. [M5-FIX-1]

OPEN-FLAG (BORROW-OPTION): a miss-tolerant borrowing lookup (`Option` of a borrow) is unspellable in kernel v0.6 — mode is not a `targ` [GRAM-3], so `Option<&'r V>` is not a type. The miss channel is therefore trap-guarded-by-fact (`tbl_contains` then `tbl_get`, one probe each; or `tbl_entry`). A borrow-carrying-enum delta would add an `Option`-returning lookup row; until then this partition is normative.

CG marks:
- **CG-PROBE** (designated hot ops): guaranteed-inline for `tbl_contains`/`tbl_get`; pinned probe shape: one H2 broadcast into a 16-lane vector (machine core), then per group step exactly one 16-byte control load, one lane compare, one mask extract; the hit path touches at most one value cache line before the value access. Pinned in the codegen corpus.
- **CG-INL**, **CG-ITER**: as defined in S.1.

---

## S.3 `conc_queue<T, ep, K>` — bounded concurrent queue

> **RED BOX — brand discipline [M5R3-FIX-2].** `'q` is NOT a region: it never appears in `region_params`, in a parameter type, or in your effect row. There is no brand-parameter spelling, and a brand can never instantiate a region argument (CQ-2). ALL endpoint use must live in the single function scope that called `cq_new` (workaround (a) below); a helper `fn` receives an already-minted endpoint value or a pre-taken borrow of one, never the brand. Threading `'q` through any `fn` signature is rejected.

Single-scope pipeline, complete — mint, split, send, and receive in the scope that owns `'q`; nothing crosses a `fn` boundary. (Rides OPEN-FLAG (BRAND-MINT) for the fresh `'q` binder on `cq_new`'s result and OPEN-FLAG (SEALED-MATCH) for `match ends`; the `'q` effects stay confined, so they never appear in the row — the RED BOX in action.)

```
fn roundtrip(v: own u64) -> own Result<u64, CqRecvFail> allocates(heap), traps {
  doc "Everything touching 'q lives here; no endpoint or brand crosses a fn boundary.";
  region 'e {
    let ends: own cq_ends<'q, u64> = cq_new<u64>();
    match ends {
      QueueEnds(tx: t, rx: r) => {
        let s: own Result<unit, CqSendFail<u64>> = cq_try_send(&uniq 'e t, v);
        match s {
          Ok(value: u) => {
            return cq_try_recv(&uniq 'e r);
          }
          Err(error: f) => {
            check False() else trap "roundtrip: send failed on an empty queue";
          }
        }
      }
    }
  }
}
```

cq_new instantiation spelling [M5R4-FIX-6]: `conc_queue<T, ep, K>` fixes the payload `T`, the endpoint discipline `ep` (`spsc`/`mpmc`), and the const capacity exponent `K`; the intended `cq_new` call is `cq_new<Record, spsc, 10>()` — payload `Record`, `spsc`, `2^10` slots — minting a fresh brand `'q`. TWO attestation flags: (a) the `ep` tag (`spsc`/`mpmc`) and the hasher `h` in `table<K, V, h>` are lowercase closed-set tags, which GRAM-3 `targ := type | REGIONID | const` does NOT admit (a TYPEID is uppercase-initial), so the tag-argument position is a SPEC GAP pending a kernel delta (admit closed-set tag identifiers as a targ sort); (b) the `'q` binder rides OPEN-FLAG (BRAND-MINT) — the mint-position binder has no prior name. Do not guess a different spelling; the `<T, ep, K>` arity above is the attested intent.

Blessed affine-FIFO route [M5R4-FIX-6] (resolving the seq-vs-C2 choice): a single-threaded FIFO of an AFFINE payload is C2's pool-handle ring (or the two-seq flip queue), NEVER front-removal from a `seq` — `seq_remove_at(s, 0_u64)` is O(len) per pop (SEQ-4) and is the WRONG_CHOICE anti-pattern C2 supersedes. Cross-thread FIFO is this `conc_queue` form. Copy payload single-threaded is C2's masked ring.

[CQ-1] Form. `ep` closed set: `spsc`, `mpmc`. `const K: u64`, `0 <= K <= 32` (compile-time reject outside, cites CQ-1 [DIAG-1]); capacity is `2^K` slots. T must conform `Sendable` at instantiation (compile reject otherwise; [CAP-1] vocabulary). Slot storage is one heap allocation at `cq_new`; its byte size is a monomorphization-time constant, and overflow is a compile-time reject.

[CQ-2] Brand. `cq_new` mints the instance brand `'q`: an identity tag, not a lexical region — no outlives relation, no storage bound; a brand unifies only with itself, so endpoints of distinct instances never mix. A brand is not a lexical region and may never instantiate a region parameter; a row generic over a region rejects a brand argument. [M5-FIX-7] OPEN-FLAG (BRAND-MINT): TYPE-5 requires the binder's full type, but a fresh brand has no prior name; the mint-position binder spelling is kernel-delta work. Rows below assume it exists.

OPEN-FLAG (BRAND-CROSS-FN) [M5R2-FIX-6]: the full statement and the sanctioned single-scope workarounds are now the RED BOX at the head of S.3 [M5R3-FIX-2]. In brief: binding a brand across a user-`fn` boundary is unspellable in v0; workaround (a) `conc_queue` — consume the endpoints in the scope that minted `'q`, pass helper fns pre-minted endpoint values or borrows, never the brand; workaround (b) `arena<'r, T>` — keep arena code in one fn or pass the `arena` value (not its region/brand) and re-borrow in the callee.

[CQ-3] Endpoints. `cq_tx<'q, T>` and `cq_rx<'q, T>` are affine own values. Every endpoint op takes `&uniq` on the endpoint, so per-endpoint use is single-threaded by construction; concurrency is distinct endpoints on distinct threads. spsc: exactly the two minted endpoints; the clone rows are a compile-time reject citing CQ-3. mpmc: `cq_tx_clone`/`cq_rx_clone` mint peers. Endpoints conform `Sendable` iff T does; endpoints are never `Shareable`. OPEN-FLAG (THREADS): v0 defines no thread construct [CAP-1]; the cross-thread clauses bind when the concurrency layer lands; single-threaded use is legal now.

[CQ-4] Ends destructure. `cq_ends<'q, T>` is a brand-parameterized sealed enum admitted to `match` with the single variant `QueueEnds(tx: cq_tx<'q, T>, rx: cq_rx<'q, T>)`; one `match` moves both endpoints out in one arm [OWN-13], which OWN-1's whole-binding-death-on-partial-move makes the only sound two-affine-field extraction. OPEN-FLAG (SEALED-MATCH): the sealed-form mechanism declares forms opaque; a matchable sealed enum and enum brand-parameterization [GRAM-2 has no region gparam] are kernel-delta work.

[CQ-5] Close and teardown. The queue is closed-for-send when the last `cq_rx` drops, and closed-for-recv when the last `cq_tx` drops AND the queue has drained (receivers get all in-flight items first, then `RecvClosed()`). Close directions [M5-FIX-7]:

| trigger | resulting state |
|---|---|
| last `cq_rx` drops | closed-for-send |
| last `cq_tx` drops AND queue drained | closed-for-recv (receivers drain all in-flight items first, then `RecvClosed()`) |

Send-family ops on a closed queue return the item back inside the `Err` payload — no owned value is ever lost to a failure. When the last endpoint overall drops: remaining items are dropped in FIFO order, then storage is freed; every endpoint drop is a surfaced compiler-derived drop [STOR-3] whose sealed body performs the counting and drain (drain-on-drop).

[CQ-6] Ordering and facts. Items from one producer are received in that producer's send order; spsc is globally FIFO; mpmc cross-producer interleaving is unspecified. Every handoff is release/acquire: reads of a received T are data-race-free (D1). The form exports NO occupancy facts — any occupancy predicate can be invalidated by another endpoint before use, so the fact channel fails closed here by design. The only static fact is the type-level capacity `2^K`. All kill columns are `—`: no fact-bearing state crosses these ops.

[CQ-7] Blocking. A blocking row spins a bounded count, then parks on the platform futex-equivalent; it wakes on space, item, or close. OPEN-FLAG (PARK): the parking primitive is a TCB/runtime delta. OPEN-FLAG (BLOCKS-EFFECT): EFF-1's closed row grammar has no blocking word; blocking rows below are marked `[blocks]` pending that kernel delta. OPEN-FLAG (BRAND-EFFECTS): rows write `reads('q)`/`writes('q)` treating the brand as an effect region; brand-in-row spelling is kernel-delta work.

Companion enums (normative bytes; the `QueueFull`/`RecvEmpty` variants are statically unreachable from the blocking rows and retained for regularity — the DivError precedent [OP-2]):

```
enum CqSendFail<T> { QueueFull(value: T); QueueClosed(value: T); }

enum CqRecvFail { RecvEmpty(); RecvClosed(); }

enum CqBatchOut<T, const IN: u64> { BatchOpen(sent: u64, rest: seq<T, IN>); BatchClosed(sent: u64, rest: seq<T, IN>); }
```

| op | signature | own | loan | effects | failure | facts | kills | cg |
|---|---|---|---|---|---|---|---|---|
| `cq_new` | `() -> own cq_ends<'q, T>` (mints `'q` [CQ-2]) | returns own (both endpoints inside) | NONE | allocates(heap) | — | capacity is the static type-level `2^K` [CQ-6] | — | — |
| `cq_send` | `['e](tx: &uniq 'e cq_tx<'q, T>, v: own T) -> own Result<unit, CqSendFail<T>>` | consumes v on Ok; Err returns it (`QueueClosed(value)`); `QueueFull` unreachable (blocking) | NONE | reads('q), writes('q) [blocks, CQ-7] | Result: CqSendFail (closed) | — | — | CG-QOP |
| `cq_recv` | `['e](rx: &uniq 'e cq_rx<'q, T>) -> own Result<T, CqRecvFail>` | item own-out; `RecvEmpty` unreachable (blocking) | NONE | reads('q), writes('q) [blocks, CQ-7] | Result: CqRecvFail (`RecvClosed` only, per CQ-5) | — | — | CG-QOP |
| `cq_try_send` | `['e](tx: &uniq 'e cq_tx<'q, T>, v: own T) -> own Result<unit, CqSendFail<T>>` | consumes v on Ok; `QueueFull(value)` / `QueueClosed(value)` return it | NONE | reads('q), writes('q) | Result: CqSendFail | — | — | CG-QOP |
| `cq_try_recv` | `['e](rx: &uniq 'e cq_rx<'q, T>) -> own Result<T, CqRecvFail>` | item own-out | NONE | reads('q), writes('q) | Result: CqRecvFail (`RecvEmpty` when open and empty; `RecvClosed` when closed and drained) | — | — | CG-QOP |
| `cq_send_batch` | `['e](tx: &uniq 'e cq_tx<'q, T>, batch: own seq<T, IN>) -> own CqBatchOut<T, IN>` | consumes batch; blocks until all sent or closed; `rest` returns the unsent suffix in order (FIFO preserved); `BatchOpen` has `sent = len`, empty `rest` | NONE | reads('q), writes('q) [blocks, CQ-7] | — (closure is the `BatchClosed` value arm) | — | — | CG-QBATCH |
| `cq_try_send_batch` | `['e](tx: &uniq 'e cq_tx<'q, T>, batch: own seq<T, IN>) -> own CqBatchOut<T, IN>` | consumes batch; admits the longest sendable prefix immediately; unsent suffix in `rest` | NONE | reads('q), writes('q) | — | — | — | CG-QBATCH |
| `cq_recv_batch` | `['e, 'd](rx: &uniq 'e cq_rx<'q, T>, dst: &uniq 'd seq<T, M>, max: u64) -> own Result<u64, CqRecvFail>` | appends 1..=max items own into dst (blocks for the first); returns count; `RecvEmpty` unreachable | NONE | reads('q), writes('q), writes('d), allocates(heap), traps [blocks, CQ-7] | trap `"seq capacity overflow"` (dst growth [SEQ-3]); Result: CqRecvFail (`RecvClosed` when closed and drained) | Ok arm: LEN(dst) += r (when keyable) | CAP(dst); SLACK(dst) dec r (killed when unknown) | CG-QBATCH |
| `cq_try_recv_batch` | `['e, 'd](rx: &uniq 'e cq_rx<'q, T>, dst: &uniq 'd seq<T, M>, max: u64) -> own Result<u64, CqRecvFail>` | appends 0..=max items own into dst; `Ok(0)` when open and empty | NONE | reads('q), writes('q), writes('d), allocates(heap), traps | trap `"seq capacity overflow"`; Result: CqRecvFail (`RecvClosed` only when closed and drained) | Ok arm: LEN(dst) += r (when keyable) | CAP(dst); SLACK(dst) dec r (killed when unknown) | CG-QBATCH |
| `cq_tx_clone` | `['x](tx: &'x cq_tx<'q, T>) -> own cq_tx<'q, T>` | new endpoint own-out | NONE | reads('x), writes('q) | — (mpmc only; spsc is a compile-time reject citing CQ-3) | — | — | — |
| `cq_rx_clone` | `['x](rx: &'x cq_rx<'q, T>) -> own cq_rx<'q, T>` | new endpoint own-out | NONE | reads('x), writes('q) | — (mpmc only; spsc is a compile-time reject citing CQ-3) | — | — | — |

CG marks (CI-pinned; a pin regression fails the gate):
- **CG-QOP**: spsc — no read-modify-write ops on any path; one acquire load of the opposite cursor, amortized below one per op by cursor caching; one release store to publish; the fast path is guaranteed-inline. mpmc — Vyukov-style per-slot sequence numbers; exactly one successful CAS per admitted item on the unbatched path.
- **CG-QBATCH** (designated hot ops, the batched contract shape): one cursor reservation per batch (plain store for spsc; one CAS loop for mpmc), one contiguous slot-copy region (memcpy shape for copy T), one release publish per batch. Per-item amortized instruction budget pinned in the codegen corpus. Performance pin: sustained throughput `>= 80,000,000 items/s` (spsc, T = u64, batch >= 64, reference pin host) as a CI perf pin.
