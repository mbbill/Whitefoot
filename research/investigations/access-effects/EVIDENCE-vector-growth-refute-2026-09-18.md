# Evidence: adversarial check of the static-scope rule across vector reallocation (2026-09-18)

Method: four Opus attackers (soundness, performance loss, fact invalidation, slot states) each tried to break one claim under a frozen ten-rule candidate; one Opus judge re-traced every example against the rules literally. Workflow run `wf_d69b37c3-b12`. The candidate is the reference-restriction direction reached in the owner's 2026-09-17 session: references exist only as call arguments or block-scoped part-names, never stored or returned; effects are paths; Box lives on one global heap; Vector is user-level code over a runtime-length DynBox with per-slot full/empty state.

Claim under test: vector reallocation never needs runtime reasoning; any call that may reallocate declares `writes(v.buf)`, which overlaps every element path, so the checker rejects holding an element reference across it and accepts everything else, and every rejected program has an equal-performance rewrite.

## Verdict

The claim holds for what it names: no attacker produced a program in which reallocation itself forces runtime reasoning, and every "element reference across a push" rejection I re-traced has a rewrite whose only cost is a reload of v.buf's descriptor plus a scale-add that correct C++ must also emit — the defense is effect granularity (grow replaces the whole DynBox, so writes(v.buf) dominates every finer path) and never a question about whether a realloc occurs. I confirm zero accepted-yet-unsafe programs: all three use-after-free traces turn on readings the text permits rather than compels, so they are gaps, not breaks. The strongest surviving example is the one the claim's own wording concedes — writes(v.buf) "overlaps every v.buf[i] and every *v.buf[i]", yet grow moves only descriptors and never touches the pointees, so a `&*v.buf[i]` into a heap Node is rejected although it is provably safe; the Vector<Box<Node>> loop that also pushes then pays either two dependent, non-hoistable loads per iteration or a placeholder allocation, where C++ holds Node* in a register for free. Two further real costs are charged to R1 and R9, not R5: no variable-length inline tail (LLVM User+Use costs one extra dependent cache line per visit) and no protocol-based disjointness (lock-free SPSC degrades to fork-join plus double buffering and a per-buffer latency). R8's range-fact language forces Option<T> for data-determined occupancy, costing an open-addressed table about 1.41x bytes per slot and one unremovable discriminant branch per successful probe. Two attacker cost claims do not survive: the arena-hop and two-live-addresses cases are over-counted, because the base pointer is register-resident and the addresses are loop-invariant and hoistable under WF's own no-alias guarantee. The two highest-value gaps are the unstated struct-invariant binding rule — which lands three ways at once (R6 as written makes Vector write-only after any push; grow's body needs an ensures it lacks; the open-hole-across-push case is decided by it) and which is worth two dependent loads per iteration in the commonest heap-container shape — and the fact that R5 is a call-site rule with no statement-level prefix or move counterpart, a sentence the live project spec already carries ("Moving the backing owner or changing its storage descriptor remains forbidden while a descendant loan is live") and R1-R10 dropped. On the relayed Swift question: `inout` is a copy-in/copy-out parameter mode plus the Law of Exclusivity, enforced statically for locals and inout parameters but dynamically with a trap for class properties, globals and escaping captures, so WF must reject exactly where Swift traps; worth borrowing are the `_read`/`_modify` coroutine accessors (a user-definable `with`-producer, which R2 as written cannot express), the borrowing/consuming/~Copyable spelling, and the nonescapable Span plus lifetime-dependency work as the prior art to read first if R2's second-class restriction is ever relaxed — and not worth borrowing are dynamic exclusivity checks, copy-in/copy-out as specified semantics, and Array's copy-on-write uniqueness branch on every mutation.

## Confirmed accepted-yet-unsafe programs (0)

None.

## Confirmed rejections with real runtime cost (4)

### Reference to a Box pointee across a realloc (unsound Near-miss 2 / Accepted 3, slots G5)

```text
v: Vector<Box<Node>>
with &*v.buf[i] as node {
    for k in 0..n { node.count += 1; push(v, make_node()?)? }   // REJECTED
}
```

Rewrite:

```text
A: for k in 0..n { with &*v.buf[i] as node { node.count += 1 }; push(v, make_node()?)? }
B: reserve(v, v.len + n)?; b = take(&v.buf[i]); put(&v.buf[i], placeholder)
   with &*b as node { for k in 0..n { node.count += 1; push_within_capacity(v, make_node()?) } }
   x = take(&v.buf[i]); put(&v.buf[i], move(b))
```

Cost: Re-traced and confirmed: R3 folds pointee identity into the owner's path, so `*v.buf[i]` sits under push's writes(v.buf) and R5 rejects, although grow moves only the 8-byte descriptors and the Node never moves. The claim's own sentence concedes this ('overlaps every v.buf[i] and every *v.buf[i]'). Rewrite A costs, per iteration, a reload of v.buf.ptr + scale-add + one dependent load of the descriptor (not hoistable, because push writes v.buf) against a register-held Node* in C++ -- 2 dependent loads on a loop body whose useful work is one increment, and a cold miss on the first re-derivation after each growth. Rewrite B removes the per-iteration cost but needs a placeholder in slot i: for Box<Node> the only placeholder is another Box, i.e. one malloc, or a change of element type to Option<Box<Node>> (a discriminant test per access, size-neutral only if niche optimization is specified, which R1-R10 do not). Leaving the hole open instead costs nothing but is exactly what the unstated invariant-binding rule decides. No zero-cost rewrite exists under R1-R10 as written.

### Data-determined occupancy: open addressing must move occupancy into the data (slots G7)

```text
// wanted
struct Table<K,V> { ctrl: DynBox<u8>, slots: DynBox<Entry<K,V>> }
// invariant wanted but unstatable: forall i < cap: ctrl[i] is a hash byte <=> full(slots[i])
// forced
struct Table<K,V> { ctrl: DynBox<u8>, slots: DynBox<Option<Entry<K,V>>> }
```

Rewrite:

```text
None better than Option<Entry> as data. Sentinel keys work only for K with a reserved value; dropping ctrl and scanning Option discriminants replaces one SIMD group load with 16 strided loads, so it is worse.
```

Cost: Confirmed. Occupancy is an arbitrary hash-determined subset of [0,cap) with O(cap) simultaneous holes, so it is outside R8's range facts and past its static hole bound; R8's own escape names Option<T>. For a niche-free Entry=(u64,u64): 24 bytes/slot against 16+1, about 1.41x bytes and ~1.5x cache lines touched per probe sequence, on a structure whose cost is entirely misses. Plus one discriminant branch per successful probe that cannot be removed, because the fact that would remove it (ctrl[i] matched => slots[i] is Some) is a correlation between two paths at the same index and is not statable. hashbrown pays neither; safe Rust pays both, which is why hashbrown uses MaybeUninit. The missing ingredient is narrow and SMT-free (index-correlated invariants between sibling arrays); I am not adopting it.

### No variable-length inline tail: LLVM User+Use, sk_buff, attribute lists (perf lens)

```text
// C: struct { u32 n; T data[]; } -- one allocation, operands in the object's own cache line
// WF forced form:
struct Module { users: Vector<User>, ops: Vector<Handle>, free_runs: Vector<Run> }
struct User { opcode: u8, op_off: u32, op_len: u32 }
```

Rewrite:

```text
Global operand pool addressed by (op_off, op_len), as above. The per-User `ops: DynBox<Handle>` alternative is worse: one extra allocation per instruction, which is the cost hung-off-uses exists to avoid.
```

Cost: Confirmed, and charged to R1, not R5: a Box is one heap object and no aggregate's storage extends past its static size, so the flexible-array idiom has no WF spelling. Per User visited on a use-def walk, C++ touches one cache line for the object and its operands together; WF touches the User's line plus a second dependent line in m.ops at op_off. One extra dependent cache miss per hop on IR that exceeds L2, plus a free-run list and fragmentation C++ does not pay. I do not endorse the attacker's 15-30% figure (unmeasured), but the extra dependent line is real and unavoidable here.

### Protocol disjointness: lock-free SPSC ring (perf lens)

```text
par {
    while running { put(&ring.buf[ring.head & mask], item); release(ring.head + 1) }
    while running { t = acquire(ring.tail); use(&ring.buf[t & mask]) }
}
```

Rewrite:

```text
loop { par { fill(&front, 0, n/2); fill(&front, n/2, n) }; swap_buffers(&front, &back); consume_all(&back) }
```

Cost: Confirmed and correctly traced: R9 defers to R5, whose only disjointness evidence is distinct roots or proven-distinct indices/ranges; head != tail holds under a concurrent protocol that R1-R10 have no vocabulary for (no atomics, no memory model), and no fact carried across two concurrently executing arms can establish it. The only admitted shape is fork-join over statically partitioned ranges: per-item pipelining becomes per-buffer latency plus a second buffer and a barrier per buffer. For a 48kHz callback or a 120Hz compositor that is a missed deadline, not a percentage. Out of scope for the claim (rings do not grow) and a missing concurrency feature rather than a defect in the path rule.

## Rule gaps (missing or ambiguous rules, not filled here) (10)

### R5 is a call-site rule; no statement-level or prefix rule protects a live reference (unsound G1-A, G1-B)

```text
with &v.buf[0] as p { w = move(v); consume(move(w)); use(p) }
with &*e.A.0 as p { e = E::B(0); use(p) }
```

Gap: Re-traced and confirmed as a gap, not an acceptance. R5's conflict clause fires only on calls, so `w = move(v)` and `e = E::B(0)` are governed by nothing but R3's last clause, which names only Box content, only 'the owner', and only two of R4's four verbs. The deeper hole is that a move RE-ROOTS: after `w = move(v)`, consume's substituted effects are rooted at w and R5's 'different static roots' proves them disjoint from p's path rooted at v. Also uncovered: dropping a by-value owned parameter frees its storage while R4 says a by-value parameter's `reads` covers what it owns, so `consume(move(v))` has no declared WRITE for R5's part-name clause to catch. Missing rule (not filled here): writing, replacing, moving out of or freeing the storage at any PREFIX of a live reference's path is rejected, whether by call or by statement, and a move does not launder a live reference into a new root. Note the live project spec already carries this sentence -- range-loans DESIGN.md: 'Moving the backing owner or changing its storage descriptor remains forbidden while a descendant loan is live' -- so R1-R10 dropped it rather than lacking it.

### Struct invariants have no binding rule, and R6 as written makes Vector write-only (unsound G5, facts case 2, slots G2, facts case 10)

```text
push(&v, x)?            // writes(v.buf); ensures only v.len == old(v.len)+1
with &v.buf[i] as p { *p }   // needs full(v.buf[i]): R6 killed it, nothing re-established it

x = take(&v.buf[k]); push(v, y)?; put(&v.buf[k], x2)   // grow would take() an empty slot
```

Gap: Highest-value gap; three independent consequences confirmed. (1) R6 invalidates every fact MENTIONING a written path, so writes(v.buf) kills `full(v.buf[i])` for all i and push's ensures restores nothing: after any push the caller can never read an element. (2) The prompt's own grow carries no ensures, so inside push the facts `v.len < v.buf.cap` and `empty(v.buf[v.len])` die at the grow call and `put` is rejected -- the library the claim is built on does not check. (3) slots G2: I do NOT confirm it as an accepted unsafe program. Under a strictly literal reading grow's own body cannot prove `full(v.buf[i])` either, so grow is rejected, not the caller accepted; the unsafety needs the asymmetric reading 'invariant assumed at entry, not required at call sites'. The missing rule is where a type invariant holds: standing exit obligation vs R6-invalidatable fact, and per-callee `requires` vs implicit precondition of every call taking &v. That choice is worth measurable performance: per-callee requires admits the open-hole rewrite (zero cost) for the Vector<Box<Node>> case; the blanket reading forces re-derivation (2 dependent loads/iteration). It also decides facts case 10: if an invariant must hold after every statement, a two-vector struct cannot be pushed to at all and SoA collapses into AoS.

### R5 does not say its disjointness test applies among the callee's own substituted effects (unsound G3, Accepted 2 third arm)

```text
fn bad(outer: &Vector<Vector<Int>>, inner: &Vector<Int>) writes(outer.buf), writes(inner.len) { push(outer, ...)?; inner.len += 1 }
bad(&vv, &vv.buf[0])

fn both(a: &Vector<Int>, b: &Vector<Int>) writes(a.buf), writes(b.buf) { par { push(a,1)?; push(b,2)? } }
both(&v, &v)
```

Gap: Confirmed load-bearing and unstated. Each body checks fine because the parameters are distinct roots; safety rests entirely on comparing the substituted effect set against itself at the call site. R5's sentence placement (immediately after 'substitute actual arguments into the callee's effect paths') implies that reading, but the next sentence treats enclosing part-names as a separate clause, which weakens it. Without the intra-call reading, `bad(&vv, &vv.buf[0])` is a use-after-free and `both(&v,&v)` is a data race under R9. Missing sentence, not filled here: after substitution the callee's effect set must be internally pairwise-disjoint at every write, by the same criterion.

### Part-name transparency, and no exception for the part-name that IS the argument (unsound G4)

```text
with &v.buf[0] as p { set(p, 9) }      // set writes(x): R5's last clause rejects, literally
with &v as a { two(a, &v) }            // roots a and v differ: R5 proves disjoint, wrongly
```

Gap: Both halves confirmed by re-trace, and this is a defect in both directions. (a) p is a live part-name whose path is identical to the call's substituted write effect, so 'overlaps a write effect of the call' rejects the one idiom `with` exists for; every accepted example in all four lenses that calls a function on a part-name (the hash-map `put(&slot.key, ...)` rewrite, `set(p,9)`, `transform(&v.buf[i])`) depends on an exception that is not written. (b) R3 says paths are 'rooted at a static name' and does not say a part-name is not a root; under opacity a part-name laundered through `with` defeats every overlap check, and the same shape under R9 is an accepted race. Missing: a part-name is never a root, it denotes its defining path; and a live part-name that is an argument of the call, or under one, is not a conflict for that call.

### Index subexpressions in effect paths and part-name paths: entry-evaluated or re-read (perf reserve-then-push, unsound Accepted 1)

```text
fn push_within_capacity(v: &Vector<T>, x: T) requires v.len < v.buf.cap
    writes(v.buf[v.len]), writes(v.len)
with &out.buf[out.len - 1] as parent { for i in 0..k { push_within_capacity(out, child(i)); parent.child_count += 1 } }
```

Gap: Confirmed genuine and, on the cost axis, the highest-value wording question in the set. The effect path `v.buf[v.len]` subscripts storage the same call writes, and the part-name path `out.buf[out.len - 1]` does too. R4 defines effects as paths and R6 defines invalidation for facts; neither covers index expressions inside paths. Under entry evaluation both denote fixed slots, R5 proves the indices distinct from `parent_idx < out.len` (monotone via the ensures) and the whole reserve-then-hold-a-reference-across-a-hot-push-loop family is zero cost. Under the re-read reading the effect path is not a fixed location and every element reference into out.buf dies at every push, costing a reload of out.buf's descriptor (2 loads) plus a shift-add per iteration where C++ keeps Node* in a register. Not resolved here.

### Function and closure types carry no effect or ensures row (unsound G2, facts case 8)

```text
fn for_each<T>(v: &Vector<T>, f: &Fn(&T)) reads(v.buf) { ... with &v.buf[i] as e { f(e) } ... }
for_each(&v, &|e| { push(&v, *e)?; use(e) })

for_each_key(&t, &v, |w,k| push(&w,k)?); with &v.buf[i] as p { *p += 1 }   // i < v.len lost
```

Gap: Confirmed on both axes. R2 explicitly blesses a closure that captures a reference and is passed down, yet no rule puts an effect row in a function type, and none maps a closure's captured paths (caller's root namespace) onto the callee's parameter paths. 'No declared effects means unchecked' is a use-after-free (the closure's push reallocates under the live part-name e); 'no declared effects means no effects' is sound but bans every mutating visitor. Symmetrically, R6 consults a callee's ensures but nothing says a function TYPE or generic bound may carry one, so every higher-order growth site loses its bounds fact and pays one compare+branch that Rust recovers non-modularly by monomorphizing and inlining. The first-order workaround (inline the traversal) is zero cost for Copy T and costs a take/put pair per element otherwise; the loss is expressiveness, and the rule is missing either way.

### The fact language and its derivation procedure are unspecified (facts cases 3, 4, 9; slots G3, G6; perf insertion-sort)

```text
i = hash(k) & (t.cap - 1)     // needs i < t.buf.cap from is_pow2(cap): a bitvector fact
if c > T { push(&v,...)? }     // join: v.len == old  OR  v.len == old+1
put(&nb[i], take(&v.buf[i]))   // splice [0,m) and [m,len) range facts after a par
```

Gap: One gap with several faces, all confirmed. R1-R10 never state what arithmetic the checker does (the hash probe needs h & (cap-1) < cap, a bitvector step; the modulo formulation is provable but replaces the mask with a division, far worse than the branch it removes); never state how facts join at a merge (per-edge derivation then intersection keeps `i < v.len` and needs no case split; joining raw ensures equalities yields a disjunction and loses it); never state that loop-carried facts are inferred by fixpoint; never name the family behind R5's 'proven-distinct indices/ranges' (the half-split in slots G3 needs interval reasoning on affine endpoints); and R8's 'range facts' wording does not say whether a hole at a TRACKED runtime index is a range boundary, whether adjacent ranges splice, or whether pointwise state-preservation postconditions are admissible. Contingent cost if the language is linear-integer only and no lemma step exists: one compare+branch per probe iteration in the hottest loop most programs contain, which LLVM deletes for free via known-bits. The project's explicit finite `use` steps inside a local `invariant` close this at zero cost, but `use` is not part of R1-R10, so it is undecided as stated.

### The global heap is an unnamed root that no effect path can mention (unsound G6, facts case 12)

```text
par { push(&a, 1)?; push(&b, 1)? }    // both may call DynBox::empty and free
```

Gap: Confirmed: R1 puts every Box on one global heap, allocation and free mutate allocator state, and that state is not a path rooted at a static name, so R4 cannot declare it and R5/R9 cannot see it. R9 accepts by omission, which is the answer you want but not for a stated reason -- and it is the one place the design leans on a runtime property (allocator thread-safety) rather than a proof. Missing axiom, not filled here: the global heap is internally synchronized and allocation/free carry no declarable effect. The alternative (a named `heap` effect path) rejects every par containing two allocations, i.e. every parallel producer of heap data, with no expressible per-task arena escape.

### No variant-conditioned ensures, so every fallible mutator is under-specified (slots G1)

```text
fn push(v: &Vector<T>, x: T) writes(v.buf), writes(v.len) ensures v.len == old(v.len) + 1 {
    if v.len == v.buf.cap { grow(v)? }   // `?` returns Err; ensures is false on that path, and x is dropped
    put(&v.buf[v.len], move(x)); v.len += 1
}
```

Gap: Confirmed, small and closable at zero cost. The prompt's push has no `-> Result` yet uses `?`, and R4/R6 give no way to write `ok => len+1, err => len and buf unchanged`. Separately, on the error return the by-value x was never put anywhere, so R1's affinity auto-frees the caller's element: silent data loss, not memory unsafety. The repair (`-> Result<(), (OOM, T)>` with per-variant ensures, handing x back) is free on the hot path and matches Rust's try_reserve/push_within_capacity shape; ensures are erased before lowering. Not adopted here because variant-conditioned ensures are a rule addition.

### No definite-reinitialization obligation after moving out of a field (slots G6)

```text
free(move(v.buf))      // v.buf is now uninitialized storage
v.buf = move(nb)       // ... and nothing in R1-R10 requires this line to exist
```

Gap: Confirmed. R4 explicitly lists 'moving out of' under writes, so moving a struct field out through a second-class reference is permitted, but no rule states that the field must be re-initialized on every path before the function returns, or that the struct must not be observable in between -- including on an early Err return between the two statements, which would leave v.buf dangling. Ordinary deterministic flow analysis, no SMT, but unwritten. Writing grow as `old = replace(&v.buf, move(nb)); free(move(old))` sidesteps it at identical cost (the same stores), which is evidence the obligation is cheap to satisfy, not that it is stated.

## Zero-cost rewrites confirmed (11)

### Element reference held across push (the claim's own case; unsound Near-miss 1, slots G8)

```text
with &v.buf[i] as p { *p += 1 }
push(&v, x)?                       // ensures v.len == old(v.len)+1, so i < v.len survives R6
with &v.buf[i] as p2 { *p2 += 1 }
// Cost: one reload of v.buf.ptr + a scale-add. Rust's borrowck forces the identical re-derivation; correct C++ must reload too. No branch: the bounds fact is proved, not checked.
```

### Incremental lexer inspecting the previous token while pushing (perf lens)

```text
k = with &toks.buf[toks.len - 1] as prev { prev.kind }   // block closes, part-name gone
push(toks, scan(src, k))
// Cost: zero -- one load the C++ version also performs, and no re-derivation is needed because nothing is read after the push. The C++ original is UB the moment push_back reallocates.
```

### Self-appending string builder (perf lens)

```text
fn append_self(b: &Builder, lo: u64, hi: u64) requires hi <= b.len writes(b.buf), writes(b.len) {
    n = hi - lo
    if b.len + n > b.buf.cap { grow_to(b, next_cap(b.len + n))? }
    for i in 0..n { put(&b.buf[b.len + i], read(&b.buf[lo + i])) }   // lo+i < hi <= b.len <= b.len+i
    b.len += n
}
// Cost: zero. The disjointness is a linear endpoint comparison, exactly the form R5 admits and the form the project's range-loan work already carries with runtime endpoints. Instruction for instruction this is libstdc++'s self-append. The naive scratch-vector rewrite would cost one allocation plus an extra memcpy of n bytes, so this shape is a requirement, not a nicety.
```

### Hash-map get-or-insert / entry API (perf lens, unsound 'Not a cost after all')

```text
fn bump(map: &Map<K,u64>, k: K) writes(map.slots), writes(map.count) {
    if map.count + 1 > map.slots.cap * 7 / 8 { grow(map)? }   // the ONLY writes(map.slots) step
    j = probe(map, k)                                         // ensures j < map.slots.cap
    with &map.slots[j] as slot { if is_empty(slot) { ... }; slot.value += 1 }
}
// Cost: zero -- one hash, one probe, same as hashbrown; the load-factor branch is the same reserve check Rust performs inside entry(). The general principle: R2 forbids returning a reference, so every search returns an index, and an index re-derives in O(1). Hoist the only reallocating step in front of the reference.
```

### Adjacency list growing during traversal (perf lens; downgraded from 'real cost')

```text
if u == v { slow_path(g, u) } else {
    with &g.adj[u] as au { for k in 0..au.len { if needs_phi(au.buf[k]) { push(&g.adj[v], u) } } }
}
// Cost: one compare+branch per traversal, executed once outside the inner loop; the else arm gives R5 the fact u != v from the branch condition with no `use` step, and the inner loop keeps au's pointer and len in registers exactly as C++ does. The duplicated slow_path is code size, not runtime, and it is needed only where the C++ original is UB (self-loop).
```

### Two element references at unproven indices, merge(&v.buf[i], &v.buf[j]) (unsound 'Real cost 1'; downgraded)

```text
if i != j { merge(&v.buf[i], &v.buf[j]) } else { merge_self(&v.buf[i]) }
// Cost: one compare and one well-predicted branch per call -- the same runtime comparison Rust's get_many_mut/split_at_mut performs. R10 costs a source-level else body instead of Rust's panic edge: code size, not instructions. C++ without the test is silently wrong when i == j.
```

### par arm that pushes (slots G4)

```text
n = v.len
par { for i in 0..n/2 { transform(&v.buf[i]); if cond(i) { push(&extra_a, make(i))? } }
      for i in n/2..n { transform(&v.buf[i]); if cond(i) { push(&extra_b, make(i))? } } }
append(v, move(extra_a))?; append(v, move(extra_b))?
// Cost: zero against the correct baseline -- this is rayon's fold/reduce and every correct C++ parallel append; the first-class-reference alternative is a mutex around push (an atomic per push plus serialization at the realloc), which is strictly slower. `reserve(v, n + bound)?` before the par with per-arm disjoint index ranges is also accepted and fills in place.
```

### Editor cursor across a rope/B-tree split (perf lens), and R7's cost sentence

```text
struct Rope { leaves: Vector<Leaf>, free: Vector<u32>, index: BTreeIndex }   // cursor = (leaf_slot: u32, offset: u32)
for c in typed { insert_char(rope, cur, c); with &rope.leaves.buf[cur.leaf] as lf { lf.attr = mark } }
// Cost: effectively zero after the restructuring -- one indexed load per keystroke against a register-held Leaf*. But record the correction: through a Box chain, R7 re-derivation is an O(log n) re-descent with k dependent loads, not 'one address computation', because no Box handle can be cached. R7's cost sentence should read 'one address computation from a flat root'.
```

### grow needs an ensures for push's own body to check (facts case 2)

```text
fn grow(v: &Vector<T>) writes(v.buf) -> Result<(), OOM>
    ensures v.buf.cap > old(v.buf.cap)
         && forall i < v.len: full(v.buf[i])
         && forall v.len <= i < v.buf.cap: empty(v.buf[i])
// Cost: zero -- postconditions are erased before lowering. This is a defect in the prompt's library code, separable from the invariant-binding gap: even with standing invariants, grow must promise the capacity increase or push cannot prove v.len < v.buf.cap after it. Secondary: `v.buf.cap * 2` can overflow and no invariant bounds cap, so an explicit overflow arm is required -- one branch per growth, amortized O(log n), which Rust's Vec pays as 'capacity overflow'.
```

### Facts genuinely lost: pop, an index from the heap, an index produced in a data-dependent loop (facts cases 5, 6, 11)

```text
if i < v.len { with &v.buf[i] as p { *p += 1 } } else { /* recovery */ }
// Cost: exactly the bounds check Rust emits for v[i] and LLVM also cannot remove; WF replaces Rust's noreturn panic edge with an ordinary arm, gaining at most one phi when the arm must produce a value. All three rejections are semantically correct: after pop, i <= v.len is all that holds and the stronger fact is false for i == old(len)-1; a heap-loaded index never had a premise to invalidate. This is also the intended answer to 'realloc invalidates pointers': an Int is not a reference, so indices survive reallocation by construction.
```

### Two vectors kept in lockstep (facts case 10)

```text
struct Table<K,V> { keys: Vector<K>, vals: Vector<V>, invariant keys.len == vals.len }
fn push_pair(t: &Table<K,V>, k: K, x: V) writes(t.keys), writes(t.vals) ensures t.keys.len == old(t.keys.len)+1 {
    reserve(&t.keys, 1)?; reserve(&t.vals, 1)?
    push_nogrow(&t.keys, k); push_nogrow(&t.vals, x)
}
// Cost: zero, arguably negative -- splitting push into reserve + push_nogrow removes the second capacity branch rather than adding one, and it repairs an R10 hole where an OOM between the two pushes would break the invariant with k and x already consumed. The rejection was honest: nothing in the original program ever stated the lockstep.
```

## Attacker claims dismissed by the judge (8)

- **unsound 'Real cost 2': shared/cyclic structures pay an arena indexing per hop**: Cost over-counted. In a traversal that does not write the arena, `arena.buf`'s base stays register-resident, so `i = arena.buf[i].next` is a single scaled load -- the same memory access as C++'s `p = p->next`, in one addressing mode -- and u32 handles shrink the node relative to 8-byte pointers, improving density. The residual real costs are narrow: an imul when the node size is not a power of two, and a non-hoistable base reload only when the traversal interleaves with insertions that declare writes(arena.buf). R1's genuine cost is the inline-tail case (confirmed separately), not per-hop pointer chasing.
- **facts case 13: alternating two element references across a loop**: Cost over-counted. The loop body writes only *p and *q; nothing in it writes v.buf, so both addresses (v.buf.ptr + i*8 and + k*8) are loop-invariant and LICM hoists them into registers -- and R2/R5 guarantee v is unaliased for the loop's duration, so the hoist is legal without a restrict annotation C++ needs. Net against safe Rust it is negative (two bounds checks saved, since i < v.len and k < v.len are static facts here). The two-block form is the rewrite and it is zero cost.
- **unsound Accepted 1: push_fast with writes(v.buf[v.len]) and a live &v.buf[0]**: Not a counterexample; I confirm the acceptance and that it is safe (the `requires v.len < v.buf.cap` makes growth unreachable, and 0 < v.len gives proven-distinct indices). Worth keeping as the counterweight: reallocation is blocked by ordinary effect granularity, not by a blessed mechanism, and this is a case where WF is strictly more permissive than Rust's field-granular borrow checker at equal safety. Its acceptance is contingent on the effect-path index ambiguity listed as a gap.
- **unsound Accepted 2: par over provably distinct slots**: Not a counterexample; acceptance re-traced and correct. The false-sharing remark is not a cost of the rules -- nothing forces the writer to partition below a cache line. The third arm (`both(&v,&v)`) is real but is the G3 gap, listed there.
- **unsound Accepted 3 / arena handles / swap_remove: cases where WF is level or ahead**: Not counterexamples, and I confirm all three traces. Moving the Box out gives it a new static root and R5 accepts the push (this is the real answer to 'I need a pointer that survives a realloc': move the ownership out, do not hold a reference). u32 handles beat 8-byte pointers on density and carry a kind tag in a register where `Expr*` must load one. `swap_remove` with a known i < j emits no branch where safe Rust keeps its bounds check, because i < j is a stated fact LLVM cannot learn.
- **unsound Near-miss 3: open hole across grow**: Rejected, and correctly, but for one reason rather than three. R5 alone suffices: slot's path v.buf[0] overlaps push's writes(v.buf). The attacker's second reason (push's implicit precondition from the Vector invariant) is precisely the unstated binding rule, so it cannot be cited as an independent defense; the third (grow's take on an empty slot) is a consequence, not a gate. Zero cost: the correct program closes the hole before the push, one placeholder store, same as Rust's mem::replace.
- **facts case 1: touch_after_grow -- the bounds fact survives grow untouched**: Not a counterexample, and a correct sharpening of the claim. grow writes v.buf, not v.len, so R6 never touches `i < v.len` and no ensures is consulted. The claim's 'bounds fact carried by ensures' is needed only for push, which changes the length; reallocation alone is fact-transparent.
- **facts cases 4 and 7, perf insertion-sort hole, slots G3 and G8: accepted-fine controls**: Not counterexamples; acceptances re-traced. The loop fixpoint (case 4) and the two-half parallel grow (slots G3) both depend on the unstated derivation procedure, so they are evidence for that gap rather than against the claim. The insertion-sort moving hole has exactly one hole, so it is inside R8's static bound however 'unrelated' is read; only the three-range wording is at issue. slots G8 is the claim's own case and it holds: R5 rejects without asking whether the realloc occurs, and the rewrite costs a shift-add plus a reload C++ must also perform. The zero runtime cost of R8 is the positive finding to keep: slot states are erased, so grow lowers to a memcpy with no per-slot tag and free's 'all slots empty' needs no scan.

## Attacker verdicts and example index

- `unsound`: holds, 16 examples
  - [undecided-rule-gap] G1-A (headline). Re-root the vector by a plain move, then free through the new root while a part-name into the old root is live
  - [undecided-rule-gap] G1-B (second witness for the same gap). Overwrite an enum whose payload owns a Box, while a part-name into the payload is live
  - [undecided-rule-gap] G3. The same vector through two parameters -- R5's intra-call disjointness is load-bearing but unstated
  - [undecided-rule-gap] G2. A mutating closure passed down into a traversal -- higher-order effects are entirely unspecified
  - [undecided-rule-gap] G4. Is a part-name transparent? R5's conflict clause, read literally, rejects the core `with` idiom
  - [undecided-rule-gap] G5. Struct invariants at an Err exit -- R6 makes the system safe and simultaneously unusable
  - [undecided-rule-gap] G6. The global heap is an unnamed shared root that R9 has to exempt
  - [rejected-zero-cost] Near-miss 1. Element reference held across push (the claim's own case)
  - [rejected-zero-cost] Near-miss 2. Reference to Box CONTENT inside Vector<Box<T>> across grow -- rejected, and genuinely conservative
  - [rejected-zero-cost] Near-miss 3. Open hole across grow -- take/put interleaved with reallocation
  - [accepted-fine] Accepted 1. Narrow-effect push with a proven-distinct index -- the system is not merely conservative
  - [accepted-fine] Accepted 2. par over provably distinct slots, and the aliasing version R9 must reject
  - [accepted-fine] Accepted 3. Move the Box out of the vector to get a pointer that survives reallocation
  - [rejected-real-cost] Real cost 1. Two references into one vector at runtime indices -- R10 forces the duplicate branch into the source
  - [rejected-real-cost] Real cost 2. Shared or cyclic structures cannot hold pointers, so every hop pays an arena indexing
  - [rejected-zero-cost] Not a cost after all. Get-or-insert (the entry API) -- the obvious rewrite is zero-cost once effects are index-granular
- `perf`: holds, 10 examples
  - [rejected-zero-cost] Incremental lexer: inspect previous token while pushing the next (the baseline case)
  - [undecided-rule-gap] Reserve-then-push: parser holding an element reference across pushes that provably cannot reallocate
  - [rejected-zero-cost] String builder appending a slice of itself (self-aliasing write, the case R5 rejects on paths alone)
  - [rejected-zero-cost] Hash map entry API: get-or-insert with one hash and one probe (interner, word count, dataflow fixpoint)
  - [rejected-real-cost] Adjacency list growing during traversal: SSA phi-edge insertion while walking a predecessor list
  - [accepted-fine] Arena-allocated IR with stable node references (rustc &'tcx, Clang Stmt*, V8 Node*)
  - [rejected-real-cost] Variable-length tail inline in an object: LLVM User with its operand array, kernel sk_buff, DOM attribute list
  - [accepted-fine] One moving hole at a runtime index: insertion-sort inner loop on a non-Copy element (the hot loop inside every hybrid sort)
  - [rejected-real-cost] Pipelined producer/consumer over a shared buffer: compositor display list, io_uring SQ/CQ, audio callback ring
  - [rejected-zero-cost] Editor cursor held across a B-tree/rope leaf split (re-derivation is O(log n), not 'one address computation')
- `facts`: holds, 13 examples
  - [accepted-fine] Growth alone is fact-transparent: the bounds fact was never the thing at risk
  - [undecided-rule-gap] push does not check against its own body: grow carries no ensures
  - [undecided-rule-gap] Conditional growth: the fact survives only if the join intersects derived facts, not raw ensures
  - [accepted-fine] Data-dependent pushes in a loop, index fixed before the loop
  - [rejected-zero-cost] Index produced inside the data-dependent loop: rejection is semantically correct
  - [rejected-zero-cost] pop before re-derivation: the fact is genuinely and correctly unrecoverable
  - [accepted-fine] swap_remove with a known index ordering: WF removes a branch Rust keeps
  - [undecided-rule-gap] Growth inside a generic callback: the modularity gap, and the load-bearing one
  - [undecided-rule-gap] Hash probe after rehash: the fact language, not the invalidation, is the wall
  - [rejected-zero-cost] Two vectors in lockstep: rejection is honest, and the rewrite is faster than the original
  - [rejected-zero-cost] Index loaded from the heap and revalidated later
  - [undecided-rule-gap] Growth inside par: the allocator is not a path
  - [rejected-real-cost] Alternating two element references across a write that cannot be proven disjoint
- `slots`: broken, 8 examples
  - [undecided-rule-gap] G1. grow's OOM path: v survives, but push's contract and the moved-in element do not
  - [undecided-rule-gap] G2. A hole at a runtime index across push — as written, R1-R10 accept an uninitialized read
  - [accepted-fine] G3. Parallel grow: the copy loop splits, and R8 costs nothing at runtime
  - [rejected-zero-cost] G4. par over two halves where one branch can push
  - [rejected-real-cost] G5. Vector<Box<Node>>: a reference to the node survives realloc physically, but R3/R5 kill it
  - [undecided-rule-gap] G6. What DynBox::resize(&buf, cap) must promise
  - [rejected-real-cost] G7. Unbounded holes at data-dependent indices: open addressing, and why Option is not a free rewrite
  - [accepted-fine] G8. Control: the case the claim is actually about, and it does hold

## Attacker examples in full

### Lens: unsound (verdict: holds)

VERDICT: holds, but weakly. I could not produce an accepted-yet-unsafe program traced rule by rule, yet three of the six gaps below are places where a narrow-but-defensible reading of the text AS WRITTEN yields a use-after-free or a data race. The rule set is sound under the intended reading and under-specified as written.

RANKED GAPS (close these before anything else)

1. G1 -- R5 is the only rule with a "live reference vs. write" conflict check, and it is written for CALLS ONLY. Plain statements that destroy storage under a live part-name (`w = move(v)`, `e = E::B(0)`) are governed by nothing except one under-scoped sentence at the end of R3 ("moving or freeing the owner while such a reference is live is rejected"), which names only Box content, only the immediate owner, and only two of R4's four verbs. Worse, G1-A exposes the deeper issue: a move RE-ROOTS a value, and R5's disjointness test is "different static roots" -- so once `v` becomes `w`, every later call looks disjoint from a reference whose path is still rooted at `v`. Required rule: writing, replacing, moving out of, or freeing the storage at ANY PREFIX of a live reference's path is rejected, by call or by statement.

2. G3 -- R5 must apply its disjointness test among the CALLEE'S OWN substituted effects, not only between the call and the enclosing context. If it does not, `f(&v, &v)` breaks everything. The reading is load-bearing in three independent places (two parameters, recursion, par-inside-a-function) and is never stated.

3. G2 -- higher-order effects are entirely unspecified. R2 explicitly blesses closures that capture references and are passed down, but no rule puts an effect row in a function type or explains how to map a closure's captured paths (caller's root namespace) onto the callee's parameter paths. Either reading is a real decision: "no declared effects means unchecked" is unsound; "no declared effects means no effects" is sound but bans every mutating visitor unless the higher-order function pre-declares which of ITS parameters the closure may write.

4. G4 -- R3 says paths are "rooted at a static name" without saying a part-name is not a root. Under opacity, `with &v as a { two(a, &v) }` slips a write and a read of the same vector past R5. Separately, R5's conflict clause read literally rejects `with &v.buf[0] as p { set(p, 9) }`, the core idiom, because there is no exception for the part-name that IS the argument.

5. G5 -- R6's blunt invalidation accidentally saves the broken-invariant-on-Err-path case, but it also means that after any `push` (declared writes(v.buf)) the caller has lost `full(v.buf[i])` and can never read an element. As written, Vector is write-only. Invariants must be standing exit obligations, not R6-invalidatable facts.

6. G6 -- the global heap is an unnamed shared root that R9 exempts only by omission. State the axiom.

Minor, one line each: R1-R10 never say whether user-defined destructors exist (R8's empty-slot requirement closes the "drop runs user code during free" hole, but only for DynBox); and R4 never says whether index subexpressions in effect paths like `writes(v.buf[v.len])` are evaluated at entry or quantified.

ON THE CLAIM UNDER TEST. It holds, for a reason narrower than it states. Reallocation is defended not by anything specific to reallocation but by EFFECT GRANULARITY: grow must declare writes(v.buf) at field granularity because it replaces the whole DynBox descriptor (R4's "replacing"), and every element path and every Box-content path sits under that field, so the coarse effect dominates every fine reference without R5 ever asking whether a realloc occurs. The rewrite really is zero-cost -- one base reload plus a scale-add, which Rust and correct C++ also emit -- and the strongest objection ("a reference obtained by a search is not cheaply re-derivable") dies because R2 forbids returning references, so searches return indices and indices re-derive in O(1). Two corrections to scope: (a) the claim holds only because push is DECLARED coarsely; the narrow push_fast is also safe but for a different reason (its precondition makes growth unreachable), and that case is MORE permissive than Rust; (b) reallocation is not where the danger is. The unsafety lives in `free` and in `move` -- in G1, not in grow.

ON SWIFT `inout` (answering the relayed question, since it bears directly on the vector-realloc problem)

`inout` is a parameter MODE, not a reference type. Its specified semantics are copy-in/copy-out (call-by-value-result): the argument is copied in at entry, the callee mutates its own copy, and it is written back at return. The implementation may pass an address instead whenever that is observably equivalent, which it usually is. The model matters because it defines aliasing away rather than answering it -- if another path read the original storage mid-call it would see a stale value, and Swift's response is simply to forbid the overlap.

The forbidding is the Law of Exclusivity (SE-0176): two accesses to the same variable overlapping in time, at least one a write, are an error. That is R5. Here is the part worth taking: Swift enforces it STATICALLY for local variables and inout parameters, but DYNAMICALLY -- runtime begin_access / end_access markers and a TRAP -- for class instance properties, global and static variables, and variables captured by escaping closures. Those are exactly the cases the static analysis cannot do. Swift's answer is a runtime check. WF forbids traps (R10), so WF must reject exactly where Swift traps. That is the design fork, stated cleanly: Swift bought expressiveness with a runtime check; WF has already decided not to, so it pays in rejections instead. Every case above where I wrote "rejected, rewrite" is a case Swift compiles and then either traps on or does not, at runtime's discretion.

Three things worth borrowing:

(a) `_read` / `_modify` coroutine accessors. A `_modify` accessor yields an inout reference to its caller for the duration of a scope and regains control afterwards. That is literally `with &path as p { ... }`, and it is how Swift's Array subscript gives in-place element mutation without copy-in/copy-out. Strong independent evidence that the `with`-block shape is right, and it also shows the shape scales to user-defined containers rather than just built-in indexing. Worth checking whether R2 can express a user-defined part-name producer -- R1-R10 currently give you `with &path` over syntactic paths only, so a WF Vector cannot expose its own yielding accessor.

(b) The Swift 5.9-era ownership vocabulary: `borrowing`, `consuming`, `inout` as explicit parameter modes, plus `~Copyable` types. Same lattice as WF's reads / by-value / writes, arrived at independently, and the naming is better than most.

(c) Most relevant to your actual question: `Span` / `RawSpan` (SE-0447) and `~Escapable` types with compiler-checked lifetime dependencies (`@lifetime`). This is Swift's direct answer to "the buffer moved, so every pointer into it must die", and it is the closest existing prior art to R2+R3. A Span is a non-escapable value carrying a static lifetime dependency on the container it came from; mutating the container ends the dependency and the compiler rejects later uses. Two things to notice: they made non-escapability a TYPE property rather than a syntactic restriction on where references may appear, which is strictly more expressive than R2 (a Span can be stored in a local, passed around, even returned, as long as the dependency is tracked), and they still needed the lifetime annotations -- they did not get it for free from path structure. If WF ever wants to relax "references are second-class", that is the design to read first, and the price of the relaxation is exactly the annotation burden `@lifetime` represents.

What NOT to borrow: dynamic exclusivity enforcement (incompatible with R10), and copy-in/copy-out as the SPECIFIED semantics -- it keeps the model simple but means a program's meaning can depend on whether the optimizer chose the address-passing form, which for a language whose whole point is machine-checked source proofs is the wrong kind of slack.

#### G1-A (headline). Re-root the vector by a plain move, then free through the new root while a part-name into the old root is live

Kind: undecided-rule-gap

```text
v: Vector<Int>            // v.len == 3, v.buf.cap == 4
with &v.buf[0] as p {
    w = move(v)               // plain assignment statement, not a call
    consume(move(w))          // fn consume(x: Vector<Int>) reads(x)  -- takes ownership, frees x.buf at end of body
    use(p)                    // fn use(q: &Int) reads(q)
}
```

Trace: R2: p is a block-scoped part-name, legal. R3: p denotes the path v.buf[0], rooted at the static name v.

Statement 2, `w = move(v)`: this is an assignment, not a call. R5's conflict clause is worded exclusively for calls -- 'Substitute actual arguments into the callee's effect paths ... Any live part-name from an enclosing `with` block whose path overlaps a write effect OF THE CALL is a conflict.' There is no call here, so R5 by its own text does not fire. R4 defines what `writes(path)` covers ('writing, replacing, moving out of, or freeing the storage at that path') but R4 is a rule about checking a function body against its DECLARED effects; it does not by itself say that a statement-level move conflicts with a live part-name. The only sentence in R1-R10 that could reject this is the last clause of R3: 'A reference to Box content is a part of the Box owner's path: moving or freeing the owner while such a reference is live is rejected.' Two things must be read in for that to catch this program: (a) that a DynBox slot `v.buf[0]` counts as 'Box content' (R3's grammar lists `*b` for Box content and `v.buf[i]` separately, so this is not stated), and (b) that 'the owner' means any prefix of the reference's path, not just the immediately-enclosing Box -- here the thing moved is `v`, two levels above the slot.

Statement 3, `consume(move(w))`: now the check IS a call check, and it is the point of the attack. The live part-name is p with path `v.buf[0]`, root `v`. The callee's effect after substitution is on root `w`. R5's disjointness criterion is 'different static roots, or proven-distinct indices/ranges'. v and w ARE different static roots. So R5, read exactly as written, proves them disjoint and ACCEPTS the call. consume's body drops its by-value Vector, freeing the DynBox heap block that p points into.

Statement 4, `use(p)`: reads(v.buf[0]) -- freed memory.

The whole program turns on one under-scoped sentence in R3. If that sentence is read narrowly (Box content only, immediate owner only), R1-R10 accept a use-after-free. I do not call this 'broken' because the sentence exists and can plausibly be read to reject; per the instructions this is a rule gap, not a traced acceptance. But it is the sharpest one I found: the entire path-overlap machinery is defeated by the fact that A MOVE CHANGES THE ROOT NAME, and R5 only compares root names.

Rewrite:

```text
The missing rule, stated: 'Writing, moving out of, replacing, or freeing the storage at any PREFIX of a live reference's path is rejected, whether by a call or by a statement.' With that rule the program is rejected at statement 2 (`move(v)` writes the prefix `v`), and the rewrite is to close the block first: read v.buf[0] into a local; end the block; then `w = move(v); consume(move(w))`.
```

Cost: With the rule added: zero. The value at v.buf[0] is copied to a register before the move, which is what a first-class-reference language emits anyway once the owner is consumed. Without the rule added: the program is a silent use-after-free, a soundness failure rather than a cost.

#### G1-B (second witness for the same gap). Overwrite an enum whose payload owns a Box, while a part-name into the payload is live

Kind: undecided-rule-gap

```text
enum E { A(Box<Int>), B(Int) }
e: E = E::A(Box::new(7))
with &*e.A.0 as p {          // requires the fact is_A(e), which holds here
    e = E::B(0)              // plain assignment; the old payload's Box is dropped, heap block freed
    use(p)                   // reads freed memory
}
```

Trace: R3 admits `*b` (content of Box b) as a path form, so p's path is `*e.A.0` rooted at e. R3's final clause is squarely on point here -- e IS the Box owner and the assignment replaces it -- so this witness is closer to being caught than G1-A. But the clause says the check applies when 'moving or freeing the owner'; `e = E::B(0)` is a REPLACEMENT, and R4's vocabulary explicitly distinguishes 'writing, replacing, moving out of, or freeing'. R4's four-verb list is scoped to the meaning of a declared `writes` effect, not to R3's liveness clause, which names only two of the four verbs.

R6 does fire, but for the wrong thing: the fact is_A(e) is invalidated because e was written. That invalidates the WELL-FORMEDNESS of re-deriving `&*e.A.0` afterwards. It says nothing about the liveness of the already-materialized part-name p. Nothing in R6 links fact invalidation to reference invalidation.

I include this second witness because it shows G1-A is not a one-off about `move`: the general shape is 'a non-call statement destroys storage under a live part-name, and R5 -- the only rule with a liveness-vs-write conflict check -- is written to fire on calls only.'

Rewrite:

```text
Same repair as G1-A: the liveness conflict check must be a statement-level rule over path prefixes, using all four verbs from R4, not a call-site rule.
```

Cost: Zero once repaired: `x = *e.A.0; end block; e = E::B(0)` -- one load that was going to happen anyway.

#### G3. The same vector through two parameters -- R5's intra-call disjointness is load-bearing but unstated

Kind: undecided-rule-gap

```text
fn bad(outer: &Vector<Vector<Int>>, inner: &Vector<Int>)
    writes(outer.buf), writes(outer.len), writes(inner.len)
{
    push(outer, Vector::new())?     // writes(outer.buf), writes(outer.len)
    inner.len += 1                  // writes(inner.len)
}

// call site
vv: Vector<Vector<Int>>             // vv.len >= 1
bad(&vv, &vv.buf[0])
```

Trace: Body check (R4): push's effects {writes(outer.buf), writes(outer.len)} are covered by bad's declared set, and the write to inner.len is covered by writes(inner.len). At the `push(outer, ...)` call inside the body, is there a live reference that overlaps writes(outer.buf)? `inner` is a parameter reference with root `inner`; `outer.buf` has root `outer`. R5: 'different static roots' -> proven disjoint -> the body is ACCEPTED. That is correct in isolation: for a caller passing two genuinely different vectors the body is safe.

Call site: substitute actuals into bad's effect paths -> writes(vv.buf), writes(vv.len), writes(vv.buf[0].len). The first and third OVERLAP (the third is under the first) and both are writes. Whether the program is rejected depends entirely on whether R5's sentence 'Two effects on overlapping paths where at least one is a write must be proven disjoint' applies AMONG THE CALLEE'S OWN SUBSTITUTED EFFECTS, or only between the call's effects and the enclosing context's live part-names. The sentence sits immediately after 'Substitute actual arguments into the callee's effect paths', which is the reading that saves the system; but it is never said outright, and the next sentence treats enclosing part-names as a separate clause, which weakly suggests the first sentence was about the context too.

If the intra-call reading is NOT intended, this program is accepted, `push(outer, ...)` reallocates vv.buf while `inner` points into the old block, and `inner.len += 1` writes freed memory -- a traced acceptance of an unsafe program.

This same reading is load-bearing in three independent places: here; in the `par` version below; and in any recursion where a vector reaches itself through two parameters. That convergence is why I treat it as an omission rather than a real hole -- but it is the single most important sentence to make explicit.

Rewrite:

```text
State it: 'After substitution, the callee's effect set must be internally pairwise-disjoint at every write, by the same criterion.' The call site then passes an index instead of a second reference: `bad_idx(&vv, 0)` with effects writes(vv.buf), writes(vv.len), writes(vv.buf[0].len), re-deriving `&vv.buf[i]` after the push and carrying `i < vv.len` from push's ensures.
```

Cost: Zero. The re-derivation is one reload of vv.buf's base pointer plus a scale-add -- exactly what Rust emits after a potential realloc, and exactly what correct C++ must emit too.

#### G2. A mutating closure passed down into a traversal -- higher-order effects are entirely unspecified

Kind: undecided-rule-gap

```text
fn for_each<T>(v: &Vector<T>, f: &Fn(&T)) reads(v.buf) {
    i = 0
    while i < v.len {
        with &v.buf[i] as e { f(e) }       // <-- the call whose effects are unknown
        i += 1
    }
}

// caller
v: Vector<Int>
for_each(&v, &|e| { push(&v, *e)?; use(e) })   // closure captures &v; its effects are writes(v.buf), writes(v.len)
```

Trace: R2 explicitly blesses the shape: 'A closure capturing a reference is itself second-class (may only be passed down as an argument).' So the caller's line is exactly what R2 contemplates, and capturing `&v` is legal because the closure is only passed down.

Inside for_each, R4 says 'calling g inside f is allowed only if f's declared effects cover g's effects after substituting f's arguments.' For `f(e)` this requires knowing f's effects. R1-R10 contain no rule that puts an effect row in a function or closure TYPE, and no rule for substituting a closure's captured paths -- which are paths in the CALLER's root namespace (`v` in main) -- into the callee's root namespace (`v` the parameter of for_each). The two happen to be the same object here and different objects in a benign call like `for_each(&v, &|e| push(&w, *e)?)`; nothing in the rules lets the checker tell those apart.

If `&Fn(&T)` carrying no effects silently means 'unchecked', the program is accepted: `push(&v, ...)` reallocates v.buf while the live part-name `e` (path v.buf[i]) points into the freed block, and the closure's own `use(e)` after its push reads freed memory.

If instead it means 'no effects', the program is REJECTED at the call site because the actual closure has effects and the parameter type admits none -- sound, but then no mutating closure can ever be passed to any higher-order function unless that function's signature pre-declares, in terms of its OWN parameter paths, what the closure may write (e.g. `f: &Fn(&T) writes(v.buf)`), and the call site must check the actual closure's captured roots against that. That is expressible and is what I would build, but it is a rule that does not exist yet, and it drags a limited form of cross-frame root correspondence back into a design whose premise was second-class references precisely to avoid that.

Rewrite:

```text
First-order rewrite, available today: do not pass the closure. Inline the traversal and re-derive each iteration -- `i = 0; while i < v.len { x = v.buf[i]; push(&v, x)?; i += 1 }` -- which R5 accepts because the `with` block is closed before the push.
```

Cost: For T: Copy, zero -- the inlined loop is what a first-class-reference language compiles the closure down to anyway, and Rust rejects the closure version too. For T non-Copy the value must be taken and put back (R8), costing one extra pair of moves per touched element versus C++ passing `T&` into a std::function. The real loss is expressiveness, not instructions: no iterator or visitor abstraction over a structure you also mutate.

#### G4. Is a part-name transparent? R5's conflict clause, read literally, rejects the core `with` idiom

Kind: undecided-rule-gap

```text
// (a) the idiom the whole design exists to support
with &v.buf[0] as p {
    set(p, 9)              // fn set(x: &Int, y: Int) writes(x)
}

// (b) the aliasing question
with &v as a {
    two(a, &v)             // fn two(x: &Vector<Int>, y: &Vector<Int>) writes(x.buf), reads(y.buf)
}
```

Trace: (a) R5's last clause: 'Any live part-name from an enclosing `with` block whose path overlaps a write effect of the call is a conflict: rejected.' At `set(p, 9)`, p is a live part-name from an enclosing `with`, its path is v.buf[0], and the call's write effect after substitution is v.buf[0]. They overlap -- they are identical. Read literally, R5 REJECTS the one thing `with` blocks exist for. The clause needs an exception for the part-names that are themselves arguments of the call (and for part-names whose path is under one), but that exception is not written.

(b) The substitution in (a) only works if a part-name used as an argument is EXPANDED to its defining path. If instead a part-name is a fresh static name in its own right -- R3 does say paths are 'rooted at a static name', and a part-name is a static name -- then in (b) the substituted effects are writes(a.buf) and reads(v.buf), whose roots `a` and `v` differ, R5 proves them disjoint, and the call is accepted with a live write and a live read of the same vector. That is an aliasing unsoundness, and the same shape inside `par` would be an accepted data race under R9. The system is sound only under transparency: p is never a root, it is always textually its defining path. R5's own last clause presupposes transparency ('a live part-name whose PATH...'), so transparency is clearly intended -- but R3's 'rooted at a static name' is the sentence that would have to say so, and does not.

Rewrite:

```text
Two sentences. 'A part-name is not a root; every use of a part-name denotes its defining path.' And: 'A live part-name that is an argument of the call, or whose path lies under such an argument's path, is not a conflict for that call.'
```

Cost: Zero either way -- this is a specification defect, not a performance question. Note it is a defect in both directions: (a) is over-rejection of the core idiom, (b) is under-rejection of aliasing.

#### G5. Struct invariants at an Err exit -- R6 makes the system safe and simultaneously unusable

Kind: undecided-rule-gap

```text
fn grow_bad(v: &Vector<T>) writes(v.buf) -> Result<(), OOM> {
    nb = DynBox::empty(v.buf.cap * 2)?
    for i in 0..v.len { put(&nb[i], take(&v.buf[i])) }
    audit_log(v.len)?                 // second fallible op, AFTER the moves
    free(move(v.buf))
    v.buf = move(nb)
}

// caller
if grow_bad(&v).is_err() { x = v.buf[0] }     // invariant claims full(v.buf[0]); it is empty
```

Trace: On the `audit_log(...)?` early return, v.buf's slots [0, len) have all been taken and are empty, while Vector's invariant says `forall i < len: full(buf[i])`. Two rules bear on it and they give different, both-safe answers.

Path 1 (R8): the live `nb` is affine and auto-freed at the early return. R8: 'free(box) requires every slot empty when T is non-Copy' -- nb's slots [0, len) are full, so for non-Copy T the checker REJECTS grow_bad at its Err path. But for T: Copy the requirement does not apply, nb is freed with full slots (harmless for Copy), and this path catches nothing.

Path 2 (R6), which is what saves the Copy case: grow_bad declares writes(v.buf), so at the call site R6 invalidates every caller fact mentioning v.buf -- including `full(v.buf[0])` inherited from the invariant -- on BOTH the Ok and Err paths, and grow_bad's absent `ensures` re-establishes nothing. So `x = v.buf[0]` is rejected for want of a fullness fact. Safe.

But R6 is too blunt to live with. Apply the same reasoning to the GOOD `push`: it declares writes(v.buf) and its only `ensures` is `v.len == old(v.len) + 1`. By R6, after any push the caller has lost `full(v.buf[i])` for every i and can never read an element again. As literally written, R1-R10 make Vector write-only. The fix is to make a type invariant a standing obligation re-checked at every function exit (including Err exits) rather than an R6-invalidatable fact -- and once it is, grow_bad is rejected directly at the Err return for all T, which is the answer you want. That is a rule about where invariants are checked, and R8's phrase 'maintained as invariants' is its only hint.

Worth recording: the GIVEN grow is safe by luck of ordering -- its single fallible operation is the first statement, before any take, so no broken intermediate state can escape. That is a property of the code, not of the rules.

Rewrite:

```text
State: 'A type invariant is an obligation at every exit of every function that can write a value of that type, including early and Err returns; it is not a fact subject to R6 invalidation.' grow_bad must then restore the invariant before returning Err -- `for i in 0..v.len { put(&v.buf[i], take(&nb[i])) }; free(move(nb)); return Err(e)` -- or move audit_log before the first take.
```

Cost: Moving the fallible call before the loop: zero. Writing the rollback loop: one extra copy pass of len elements, on the error path only, which a strongly-exception-safe C++ implementation also pays.

#### G6. The global heap is an unnamed shared root that R9 has to exempt

Kind: undecided-rule-gap

```text
par {
    push(&v, 1)?;          // may call DynBox::empty -> touches the global heap
    push(&w, 1)?           // may call DynBox::empty -> touches the same global heap
}
```

Trace: R9 allows the block iff A's writes are disjoint from B's reads and writes 'by the same path-overlap check'. Effects are declared as paths rooted at static names (R3, R4). R1 says Box is 'an owning pointer to one heap object on a single global heap'. The allocator's own mutable state is not a path rooted at any static name, so it cannot appear in any effect declaration, so the path-overlap check cannot see it, so R9 accepts. That is the right answer, but it is right only because the heap is invisible to the effect language -- an exemption by omission rather than by rule. The same silent exemption covers concurrent `free`. R1-R10 nowhere state the required axiom: 'the global heap is internally synchronized; allocation and free carry no declarable effect.' That axiom does real work and should be written down, not least because it is the one place the design depends on a runtime property (allocator thread-safety) rather than a proof.

Cost: Zero if the axiom is stated. If the heap were instead modelled as `writes(heap)`, every `par` block containing two allocations would be rejected -- a large and pointless loss of parallelism.

#### Near-miss 1. Element reference held across push (the claim's own case)

Kind: rejected-zero-cost

```text
with &v.buf[0] as p {
    push(&v, 42)?          // writes(v.buf), writes(v.len)
    use(p)
}
```

Trace: R3: p's path is v.buf[0]. R5, last clause: p is a live part-name from an enclosing `with`; push's substituted write effect is v.buf; v.buf[0] is under v.buf, so they overlap and one is a write; disjointness would require 'different static roots' (both are v -- no) or 'proven-distinct indices/ranges' (one path is an index, the other the whole field, so there are no two indices to distinguish -- no). REJECTED.

This is the mechanism the claim rests on and it does work, for a reason worth naming precisely: `grow` must declare `writes(v.buf)` at FIELD granularity, because it replaces the whole DynBox descriptor and R4's `writes` covers 'replacing ... the storage at that path'. Every element path and every Box-content path sits under that field, so the coarse effect dominates every fine reference without R5 ever asking whether a reallocation actually occurs (R5's final sentence).

Rewrite:

```text
with &v.buf[0] as p { x = *p }
push(&v, 42)?              // ensures v.len == old(v.len) + 1, so 0 < v.len still holds
with &v.buf[0] as p2 { use(p2) }
```

Cost: Zero. Re-deriving costs one reload of the DynBox base pointer plus a scale-add. Rust emits the identical reload because push may realloc and the borrow checker forces the same re-derivation; correct C++ does too. The bounds fact `0 < v.len` is carried by push's ensures, so R7's 'bounds fact proved, not checked' holds and no branch is emitted.

#### Near-miss 2. Reference to Box CONTENT inside Vector<Box<T>> across grow -- rejected, and genuinely conservative

Kind: rejected-zero-cost

```text
v: Vector<Box<Int>>
with &*v.buf[0] as p {         // p: &Int, the heap Int owned by the Box in slot 0
    push(&v, Box::new(9))?     // writes(v.buf)
    use(p)
}
```

Trace: R3: 'A reference to Box content is a part of the Box owner's path', so p's path is `*v.buf[0]`, under v.buf[0], under v.buf. R5: overlaps push's writes(v.buf), not disjoint, REJECTED.

This is the one place where R5 is strictly more conservative than the machine needs. grow moves Box DESCRIPTORS between DynBox blocks (`put(&nb[i], take(&v.buf[i]))`); the heap object each descriptor points to is never touched. So `*v.buf[0]` names the same address before and after grow, and the original program is in fact memory-safe. R3's decision to fold Box content into the owner's path throws that away. I think it is the right trade -- the alternative is a second notion of identity for heap objects, which needs exactly the runtime reasoning the design forbids -- but it should be recorded as a deliberate loss rather than assumed free.

Rewrite:

```text
See 'Accepted 3': take the Box out of the vector first, giving it its own root, after which the reference survives the push.
```

Cost: Zero when the take-out rewrite applies (one 8-byte descriptor move each way; Rust needs the same rewrite). Non-zero only when the vector's invariant must hold across the push, which forces a placeholder store into the hole -- one extra store.

#### Near-miss 3. Open hole across grow -- take/put interleaved with reallocation

Kind: rejected-zero-cost

```text
with &v.buf[0] as slot {
    b = take(slot)             // full -> empty; v's invariant `forall i < len: full(buf[i])` now broken
    push(&v, Box::new(9))?     // reject?
    put(slot, move(b))
}
```

Trace: Three independent rules stop this, which is reassuring. (1) R5: slot's path v.buf[0] overlaps push's writes(v.buf) -- rejected immediately. (2) If the `with` block were closed first, R8's Vector invariant no longer holds (`empty(v.buf[0])` with 0 < len), so push's implicit precondition fails. (3) If both were somehow bypassed, grow's own loop executes `take(&v.buf[0])` on an empty slot, violating take's full -> empty precondition.

Note also that R8's 'free(box) requires every slot empty when T is non-Copy' is what makes grow's `free(move(v.buf))` legal and, more importantly, means no user code ever runs during a free -- the classic 'destructor reaches back into the container mid-realloc' hole is closed by construction. Caveat: R1-R10 never say whether user-defined destructors exist at all. If they do, this closure argument needs re-checking.

Cost: Zero -- the correct program closes the hole before the push, one extra store of the placeholder, and Rust's equivalent (mem::replace or Option::take) pays the same store.

#### Accepted 1. Narrow-effect push with a proven-distinct index -- the system is not merely conservative

Kind: accepted-fine

```text
fn push_fast(v: &Vector<T>, x: T) requires v.len < v.buf.cap
    writes(v.buf[v.len]), writes(v.len)
{ put(&v.buf[v.len], move(x)); v.len += 1 }

with &v.buf[0] as p {          // forming this requires 0 < v.len
    push_fast(&v, 7)
    use(p)                     // still valid
}
```

Trace: R4: push_fast's body writes only one slot and the length, so the narrow declaration is legal -- it never calls grow, because its `requires` rules out that branch. R5 at the call: the live part-name p has path v.buf[0]; the substituted write effect is v.buf[v.len]. Disjointness by 'proven-distinct indices': forming p required 0 < v.len, hence v.len != 0, hence the indices differ. ACCEPTED, and safe, because no reallocation is reachable.

This is the useful counterweight to the claim: reallocation is prevented not by one blessed mechanism but by ordinary effect granularity. One ambiguity to flag -- the effect path `v.buf[v.len]` MENTIONS v.len and the same call WRITES v.len. R1-R10 do not say whether an effect path's index subexpressions are evaluated at entry (old(v.len)) or are quantified. Everything above assumes entry evaluation.

Cost: Zero, and strictly better than the coarse-effect version: the reference survives, so no base-pointer reload is needed. This is something Rust cannot express -- Rust's borrow checker is field-granular, not index-granular, so `v.push` is rejected here while WF accepts. A case where WF is more permissive than Rust at equal safety.

#### Accepted 2. par over provably distinct slots, and the aliasing version R9 must reject

Kind: accepted-fine

```text
// accepted
par {
    set(&v, 0, 1);         // writes(v.buf[0])
    set(&v, 1, 2)          // writes(v.buf[1])
}

// rejected
par {
    push(&v, 1)?;          // writes(v.buf), writes(v.len)
    push(&v, 2)?           // same paths
}

// rejected only via the G3 reading
fn both(a: &Vector<Int>, b: &Vector<Int>) writes(a.buf), writes(b.buf) {
    par { push(a, 1)?; push(b, 2)? }
}
both(&v, &v)
```

Trace: First: R9 with R5's 'proven-distinct indices' -- accepted, and safe (distinct slots, no reallocation reachable from set's declared effects). Second: identical paths, not disjoint, rejected. Third: inside `both`, a and b are different static roots so R9 accepts the par; the aliasing is caught only at the call site, and only under the G3 reading that the callee's substituted effect set must be internally disjoint. That is the third independent place the reading is load-bearing, and here the consequence of getting it wrong is a data race rather than a sequential use-after-free.

Cost: Zero. Note the accepted case permits two threads writing adjacent slots in one cache line -- false sharing, a performance issue rather than a safety one, and no rule addresses it.

#### Accepted 3. Move the Box out of the vector to get a pointer that survives reallocation

Kind: accepted-fine

```text
v: Vector<Box<Int>>
with &v.buf[0] as slot {
    b = take(slot)                // b: Box<Int>, an ordinary variable -- a NEW static root
    put(slot, Box::new(0))        // restore the invariant so the vector is usable again
}
with &*b as p {                   // path *b, rooted at b
    push(&v, Box::new(9))?        // writes(v.buf), writes(v.len) -- root v
    use(p)                        // ACCEPTED
}
```

Trace: R5: p's path `*b` is rooted at b; push's effects are rooted at v; different static roots -> proven disjoint -> accepted. And it is genuinely safe: b owns its heap object outright, the object's address is fixed under R1 (a Box's descriptor moves, its pointee does not), and grow never touches objects reachable through descriptors it does not hold. R8's hole is closed before the push so the Vector invariant holds and push's precondition is satisfied.

This is the real answer to 'I need a pointer that outlives a reallocation': you do not hold a reference across the realloc, you MOVE THE OWNERSHIP OUT so there is nothing left to invalidate. Which is what R1's relocatability discipline is for -- the thing that survives is an owned descriptor, not a reference.

Cost: Zero relative to Rust: Rust forbids holding `&mut v[0]` across a push too, and its escape (`std::mem::replace(&mut v[0], placeholder)`) is byte-identical -- one 8-byte descriptor load, one placeholder store. Relative to C++ holding a raw pointer: one extra placeholder store, ~1 instruction, plus the requirement that a placeholder value exist.

#### Real cost 1. Two references into one vector at runtime indices -- R10 forces the duplicate branch into the source

Kind: rejected-real-cost

```text
// wanted
merge(&v.buf[i], &v.buf[j])       // fn merge(x: &T, y: &T) writes(x), reads(y)

// rewrite when i != j is not provable
if i != j {
    merge(&v.buf[i], &v.buf[j])
} else {
    merge_self(&v.buf[i])         // a second, specialized body
}
```

Trace: R5: substituted effects writes(v.buf[i]) and reads(v.buf[j]) overlap in root and one is a write, so disjointness must come from 'proven-distinct indices'. In a sort's partition loop the invariant i < j discharges it and the call is accepted at zero cost. Where the indices come from data -- a permutation array, a user-supplied pair, a graph edge list -- no fact is available and the call is rejected. R10 forbids a trap, so the `else` arm must exist in the source and be a real, separately-checked function.

Rewrite:

```text
Shown above.
```

Cost: One compare and one well-predicted branch per call, plus a duplicated specialized body (code size, and a second function to keep correct). Versus Rust: equal on instructions -- get_many_mut / split_at_mut perform the same runtime comparison -- but Rust spends its else-arm on a panic, so WF pays the extra source-level body that R10 demands. Versus C++ `merge(v[i], v[j])`: one extra compare and branch, and the C++ is silently wrong when i == j.

#### Real cost 2. Shared or cyclic structures cannot hold pointers, so every hop pays an arena indexing

Kind: rejected-real-cost

```text
// impossible under R1: aggregates never contain references; Box is unique-owning, so no back-pointer or cross-link
struct Node { next: &Node, prev: &Node }        // R1: rejected outright

// forced form
struct Node { next: Int, prev: Int, payload: T }
arena: Vector<Node>
// traversal hop:
i = arena.buf[i].next
```

Trace: R1 rejects the reference-in-aggregate form with no appeal, and Box's affinity rejects the two-owners form. The cost is specific to SHARED structures: a singly-linked list or a tree is fine with `Option<Box<Node>>` -- one load per hop, traversal by recursion since R2 forbids a reference in a loop variable, and the recursion is tail-shaped so it can be compiled to a loop. It is doubly-linked lists, intrusive lists, and graphs that must go through an arena.

Note the interaction with reallocation: `arena.buf`'s base pointer cannot be hoisted across any call declaring writes(arena.buf), so a traversal interleaved with insertions reloads the base every hop.

Cost: Per traversal hop versus C++ `p = p->next`: one extra load (the arena base) plus one scale-add, and one more register live across the loop -- roughly 2 extra instructions and one extra L1 access per hop, with the base load not hoistable across insertions. No rewrite under R1-R10 recovers the raw pointer. This is the honest headline cost of the rule set, and it is not a reallocation cost -- it is the price of R1.

#### Not a cost after all. Get-or-insert (the entry API) -- the obvious rewrite is zero-cost once effects are index-granular

Kind: rejected-zero-cost

```text
// naive rewrite: two probes on the miss path
idx, found = probe(m, h, k)
if found { m.buckets[idx].v += 1 }
else { insert(m, h, k, 1)? }              // writes(m.buckets) -- may resize, re-probes from h

// zero-cost rewrite
idx, found = probe(m, h, k)               // ensures idx < m.buckets.cap
if found                    { m.buckets[idx].v += 1 }       // writes(m.buckets[idx])
else if m.len < m.threshold { insert_at(m, idx, k, 1) }     // writes(m.buckets[idx]); requires m.len < m.threshold
else                        { resize(m)?; insert(m, h, k, 1)? }   // writes(m.buckets) -- re-probe only here
```

Trace: R2 forbids returning a reference, so probe must return an Int -- which is the crucial enabler, because an index is re-derivable in O(1) whereas a found reference is not. The naive rewrite loses one probe on every insert because `insert` declares writes(m.buckets) (it may resize) and so cannot reuse the vacant index across itself. The second rewrite splits the resizing case out: `insert_at` has a precondition that rules out resizing and therefore a narrow effect, and the index survives it because an Int is not a reference and nothing needs invalidating. The `idx < m.buckets.cap` fact comes from probe's ensures and survives the intervening branch condition, which reads but does not write m.

I include this because it is the strongest case I could find against the claim's cost story, and it does not survive. The claim's 'end the block, re-derive' recipe looks expensive whenever the reference was obtained by a SEARCH, but R2's ban on returning references forces searches to return indices, and index re-derivation is O(1). The pattern generalizes.

Cost: Zero. The extra `m.len < m.threshold` test is a branch Rust's HashMap performs too. Versus Rust's entry API: identical instruction counts on both the hit path and the no-resize insert path.

### Lens: perf (verdict: holds)

VERDICT REASONING

The claim as literally stated -- "Vector reallocation never needs runtime reasoning, and every rejected program has an equal-performance rewrite" -- holds. I pushed on the eight candidate programs the lens named plus two more, and could not produce a case where the *reallocation* rule costs measurable runtime against a correct Rust or C++ program. The reason is structural and worth stating plainly: holding an element reference across a push is already illegal in Rust and already undefined behaviour in C++, so the honest baseline is the program a careful C++ author actually writes, and that program already uses an index or a copy. In several cases (arena handles, per-kind AST pools) the WF shape is strictly cheaper than the pointer shape, because a u32 handle beats an 8-byte pointer and carries its kind tag in a register instead of in memory.

Where real cost does appear, it is charged to rules other than R5, and the design conversation should move there:

1. R1's Box shape is the expensive rule, not R5. A struct cannot have a runtime-length inline tail, so `struct { u32 n; T data[]; }` -- LLVM's User+Use, sk_buff, DOM attribute lists, V8's in-object slots -- has no WF spelling. The best rewrite (global operand pool + a range stored in the object) costs one extra cache line per object visit on a pointer-chasing pass, plus a free-run list and fragmentation. I would estimate 15-30% on a memory-bound LLVM-style pass, and it is unavoidable under R1. This is the single strongest finding in the run.

2. R9 has no vocabulary for runtime-protocol disjointness, so lock-free SPSC (compositor display lists, io_uring rings, audio callbacks) is inexpressible. The only admitted shape is fork-join over statically partitioned ranges, which converts per-item pipelining into per-buffer latency plus double buffering. Genuine lost parallelism, but out of scope for the reallocation claim: rings do not grow. It is a missing feature, not a defect in the path rule.

3. R7's cost sentence is understated. "One address computation" is true from a flat root; through a Box chain (rope, B-tree, trie) re-derivation is a full O(log n) re-descent with k dependent loads, because no Box handle can be cached anywhere. The rope case recovers to near-zero only by restructuring into a flat leaf pool. Recommend restating R7 as "one address computation from a flat root; a re-descent through a Box chain otherwise."

TWO GENUINE AMBIGUITIES -- reported as undecided, not resolved

A. Effect paths containing a mutable subscript. `push_within_capacity(v) requires v.len < v.buf.cap writes(v.buf[v.len]), writes(v.len)` -- is the `v.len` inside the effect path the caller's pre-call value, or a symbol re-read after the declared write to `v.len`? R4 and R5 define effects as paths and R6 defines invalidation for facts, but nothing covers index expressions inside effect paths. This one ambiguity decides whether the entire "reserve once, then hold an element reference across a hot push loop" family is zero-cost or costs 2-3 L1 loads per iteration. It is the highest-value thing to settle in this rule set, and it is purely a specification-wording question, not a mechanism question.

B. R8's "range facts" and the moving hole. Read narrowly as prefix quantifiers (`forall i < len`), R8 rejects the insertion-sort inner loop on non-Copy elements and forces the `Option<T>` fallback R8 itself names -- 50% cache footprint on a 16-byte payload plus a tag test per comparison, or a swap-based rewrite at 3x the inner-loop memory traffic. Read as three ranges with a runtime endpoint (`[0,j)` full, `[j,j+1)` empty, `[j+1,len)` full), it accepts at zero cost, and that reading is consistent with the runtime-endpoint ranges already established in <repository>/research/investigations/range-loans/DESIGN.md. Hoare partition needs the same reading with five ranges and a static bound of 2. Recommend R8 say explicitly that a hole at a *tracked* runtime index is a range boundary, and that the static bound governs only holes at *unrelated* indices.

ANSWERING THE USER'S SWIFT QUESTION (the user's own voice; not covered by the computed task)

Swift's `inout` is formally copy-in / copy-out: the callee gets a mutable local, and on return the value is written back to the caller's storage. The compiler implements it as pass-by-address whenever that is observationally equivalent, so it is usually free, but the *semantics* are copy-in/copy-out. That is why `inout` arguments never alias, and why `&x` in Swift is not a reference-forming operator at all -- it is an argument-passing marker. Swift enforces non-overlap with the Law of Exclusivity (SE-0176): static enforcement for local variables and `inout` parameters, and *dynamic* enforcement with a runtime trap for class properties and globals.

Three things in Swift are worth borrowing for WF, and one is worth explicitly refusing:

- Borrow (strongly recommended): the `_read` / `_modify` coroutine accessors. A user-defined container declares `subscript(i: Int) -> T { _modify { yield &storage[i] } }`, and the caller writes `c[i].field += 1` with no reference ever becoming a first-class value -- the yield is scoped to the caller's access and the accessor resumes afterwards. This is exactly WF's `with &path as p { }` turned inside out, and it is the missing piece that would let a *user-level* Vector expose `v[i]` syntax without violating R2. It also gives a principled place to attach the `writes(v.buf[i])` effect from ambiguity A above.

- Borrow: `borrowing` / `consuming` parameter modifiers (SE-0377) and non-copyable `~Copyable` types (SE-0390). Swift reached explicit affine/linear parameter ownership from the opposite direction than Rust, and the spelling is cleaner for effect-annotated signatures of R4's shape.

- Borrow, with interest: `Span` / `RawSpan` (SE-0447) plus the lifetime-dependency annotations (SE-0456, `@lifetime(borrow x)`). Swift is currently building a non-escaping slice type -- a second-class reference in all but name -- precisely because it does not want first-class references either. WF and Swift 6 are converging on the same answer from different starting points, and Swift's written rationale for rejecting full first-class lifetimes is directly relevant prior art.

- Refuse: dynamic exclusivity enforcement. Swift traps at runtime when static enforcement cannot decide. WF's premise forbids that, so where Swift traps, WF must reject at compile time. The practical consequence is that WF must be *more* restrictive than Swift on exactly the cases Swift punts to runtime -- overlapping access through class properties and globals -- which in WF terms means R5 must reject two `&` arguments reaching the same storage through the same Box chain. By path comparison it does, but the rule text never works an example through a Box; worth adding one.

ON THE VECTOR-GROWTH QUESTION THE USER ASKED DIRECTLY

The user's framing was: reallocation invalidates every outstanding pointer, that is a runtime event, so how can a static checker cope? The answer this rule set gives is a good one and worth saying back plainly: the checker never models the event. `grow` is reachable from `push`, so `push` declares `writes(v.buf)`, and `writes(v.buf)` is a *path* that contains every `v.buf[i]` as a sub-path. Any live reference into the buffer is under that path, so R5 kills it whether or not reallocation actually occurs on that call. The conservatism is deliberately placed where it is free: it costs the writer a re-derivation (one shift-add from a flat root) and costs nothing at runtime, because the reference was a compile-time path and never a stored pointer to begin with. What the writer loses is the ability to say "I reserved, so this push cannot realloc" -- and ambiguity A is exactly whether the rules let them say it by narrowing the effect path to `v.buf[v.len]`. That is the question to settle next.

No files were written and no repository state was changed; the only files I read were <repository>/research/investigations/range-loans/DESIGN.md and a grep over <repository>/research/investigations/containers-and-resources/DESIGN.md.

#### Incremental lexer: inspect previous token while pushing the next (the baseline case)

Kind: rejected-zero-cost

```text
// WF, rejected
fn lex(src: &Str, toks: &Vector<Token>) writes(toks.buf), writes(toks.len) {
    while more(src) {
        with &toks.buf[toks.len - 1] as prev {
            t = scan(src, prev.kind)        // merge decision needs the previous token
            push(toks, move(t))             // REJECTED here
        }
    }
}

// C++ shape
for (;;) { Token& prev = toks.back(); toks.push_back(scan(src, prev.kind)); }
```

Trace: R2 makes `prev` a block-scoped part-name over path `toks.buf[toks.len-1]`. R4 gives `push` the effects `writes(v.buf), writes(v.len)` after substituting `toks` for `v`. R5: the live part-name path `toks.buf[...]` overlaps the write path `toks.buf` (prefix containment), one side is a write, and no disjointness proof is possible because they share the root and one path is a prefix of the other. Rejected. Note the C++ version is undefined behaviour the moment push_back reallocates; Rust's borrowck rejects it too. The honest baseline is the C++ program that is already wrong.

Rewrite:

```text
fn lex(src: &Str, toks: &Vector<Token>) writes(toks.buf), writes(toks.len) {
    while more(src) {
        k = with &toks.buf[toks.len - 1] as prev { prev.kind }   // block ends, part-name gone
        t = scan(src, k)
        push(toks, move(t))
    }
}
// or, with no block at all, because the argument is evaluated before the call:
//   push(toks, scan(src, kind_at(toks, toks.len - 1)))
```

Cost: Zero. `prev.kind` is one load that the C++ version also performs. R7's re-derivation is not even needed because nothing is read after the push. R6: the bounds fact `toks.len - 1 < toks.len` is re-established by push's `ensures v.len == old(v.len) + 1`. Against a *correct* C++ program (which must copy the token or use std::deque), WF is equal or cheaper.

#### Reserve-then-push: parser holding an element reference across pushes that provably cannot reallocate

Kind: undecided-rule-gap

```text
// The thing every fast parser/arena does: reserve once, then push in a hot loop
// while keeping a pointer to a slot that was reserved earlier.
fn parse(p: &Parser, out: &Vector<Node>) writes(out.buf), writes(out.len) {
    reserve(out, 1024)?                        // ensures out.buf.cap >= out.len + 1024
    with &out.buf[out.len - 1] as parent {     // the node we are filling in
        for i in 0..k {                         // k <= 1024 known
            push_within_capacity(out, child(p, i))
            parent.child_count += 1             // in-place field update, no re-derivation
        }
    }
}

fn push_within_capacity(v: &Vector<T>, x: T)
    requires v.len < v.buf.cap
    writes(v.buf[v.len]), writes(v.len)
    ensures v.len == old(v.len) + 1
{ put(&v.buf[v.len], move(x)); v.len += 1 }
```

Trace: This is the decisive case for the claim's cost model, and R4/R5 as written do not decide it. R4 lets a signature declare effects as *paths*, and R3 admits `v.buf[i]` as a path, so `writes(v.buf[v.len])` is syntactically a legal effect. R5 would then compare the live part-name path `out.buf[out.len-1]` against the write path `out.buf[out.len]`; from the entry fact `out.len - 1 < out.len` these indices are provably distinct, so R5 would ACCEPT. But the same signature declares `writes(v.len)`, and the effect path `v.buf[v.len]` contains a subscript naming storage the same call writes. R4 and R5 never say whether a subscript inside an effect path is evaluated at the caller's pre-call state (`old(v.len)`) or is a symbol re-read after the write, and R6's invalidation rule is stated for *facts*, not for the index expressions inside effect paths. Under the pre-call reading the case is accepted and zero-cost; under the re-read reading the path is not a fixed location at all and every element reference into `out.buf` dies at every push. UNDECIDED -- I am not resolving it.

Rewrite:

```text
Under the pessimistic reading the rewrite is R7 re-derivation inside the loop: `for i in 0..k { push_within_capacity(out, child(p,i)); with &out.buf[parent_idx] as parent { parent.child_count += 1 } }`.
```

Cost: Under the optimistic reading: zero. Under the pessimistic reading: per iteration, reload `out.buf`'s descriptor (data pointer and cap, 2 loads) plus a shift-add, where C++ keeps `Node* parent` in a register across the whole loop. For a parser loop whose body is ~8 instructions that is roughly 2-3 extra L1 loads per token, call it 10-25% on the token-emission loop. The gap is worth closing explicitly: this single ambiguity separates 'reserve makes element references free' from 'element references never survive a push'.

#### String builder appending a slice of itself (self-aliasing write, the case R5 rejects on paths alone)

Kind: rejected-zero-cost

```text
// WF, rejected
fn dup_tail(b: &Builder, lo: u64, hi: u64) writes(b.buf), writes(b.len) {
    append(b, &b.buf[lo..hi])      // REJECTED
}
fn append(b: &Builder, s: &[u8]) reads(s), writes(b.buf), writes(b.len) { ... }

// C++: legal and defined, one memcpy, realloc-aware
s.append(s, lo, hi - lo);   // libstdc++/libc++ copy from the old block before freeing it
```

Trace: R5 substitutes actuals into `append`'s effects: `reads(b.buf[lo..hi])` and `writes(b.buf)`. The two paths share root `b` and `b.buf` is a prefix of `b.buf[lo..hi]`; one is a write; R5's only disjointness evidence is distinct roots or proven-distinct indices/ranges, and neither is available against a whole-field write. Rejected. R5's stated posture is exactly right here and exactly the reason it rejects: it 'never asks whether a runtime event actually happens', so it cannot lean on the fact that the C++ implementation handles the overlap internally.

Rewrite:

```text
// Vector is user code (that is the whole point), so the aliasing case becomes its own kernel:
fn append_self(b: &Builder, lo: u64, hi: u64)
    requires hi <= b.len
    writes(b.buf), writes(b.len)
{
    n = hi - lo
    if b.len + n > b.buf.cap {
        nb = DynBox::empty(next_cap(b.len + n))?
        for i in 0..b.len { put(&nb[i], take(&b.buf[i])) }
        free(move(b.buf)); b.buf = move(nb)
    }
    // now a single-box copy between two ranges of ONE box:
    invariant { use hi <= b.len; use b.len <= b.len + n }   // [lo,hi) disjoint from [b.len, b.len+n)
    for i in 0..n { put(&b.buf[b.len + i], read(&b.buf[lo + i])) }
    b.len += n
}
```

Cost: Zero. The disjointness `hi <= b.len <= b.len + i` is a range-endpoint comparison, exactly the form R5 admits and exactly the form the range-loans work already carries with runtime endpoints. Instruction for instruction this is the libstdc++ self-append path: at most one reallocation, exactly one copy of the n bytes. The naive rewrite a writer might reach for first -- copy [lo,hi) into a scratch Vector, append the scratch, drop it -- would cost one allocation plus one extra memcpy of n bytes, so the zero-cost version is a real design requirement, not an accident.

#### Hash map entry API: get-or-insert with one hash and one probe (interner, word count, dataflow fixpoint)

Kind: rejected-zero-cost

```text
// Rust: the Entry API is the reason HashMap is fast. ONE hash, ONE probe.
*counts.entry(word).or_insert(0) += 1;

// Naive WF, rejected
with &map.slots[probe(map, k)] as slot {
    if is_empty(slot) { insert(map, k, 0) }     // REJECTED: insert declares writes(map.slots)
    slot.value += 1
}
```

Trace: R2 admits the part-name over path `map.slots[j]`. R4 gives `insert` the effect `writes(map.slots)` because insert may resize. R5: the live part-name `map.slots[j]` is under the write path `map.slots`; prefix containment, one is a write, no proof available. Rejected. This is the sharpest on-claim case because the entry API's entire performance argument is that the probe result survives the insert.

Rewrite:

```text
// Hoist the only reallocating step out of the reference's life, then probe once.
fn bump(map: &Map<K,u64>, k: K) writes(map.slots), writes(map.count) {
    if map.count + 1 > map.slots.cap * 7 / 8 { grow(map)? }   // the ONLY writes(map.slots) step
    j = probe(map, k)                                         // reads(map.slots); ensures j < map.slots.cap
    with &map.slots[j] as slot {
        if is_empty(slot) { put(&slot.key, move(k)); put(&slot.value, 0); map.count += 1 }
        slot.value += 1
    }
}
```

Cost: Zero. One hash, one probe, identical to hashbrown. The load-factor branch is not extra: Rust's `entry()` performs the same reserve check, just inside the call. `probe` returning an index rather than a bucket pointer is also not extra -- hashbrown's own `find_or_find_insert_slot` returns a bucket that is immediately turned into an address by the same shift-add. The restructuring the rules force (pull the reallocating step in front of the reference) is the shape a hand-tuned map already has.

#### Adjacency list growing during traversal: SSA phi-edge insertion while walking a predecessor list

Kind: rejected-real-cost

```text
// C++ shape, in every SSA constructor and every incremental dominator update
for (Node* w : g.adj[u]) { if (needs_phi(w)) g.adj[v].push_back(u); }

// WF, rejected
with &g.adj[u] as au {
    for k in 0..au.len {
        w = au.buf[k]
        if needs_phi(w) { push(&g.adj[v], u) }    // REJECTED
    }
}
```

Trace: R4 gives `push` the effects `writes(g.adj[v].buf), writes(g.adj[v].len)`. R5 compares those against the live part-name path `g.adj[u]`. Same root `g`, same field `adj`, subscripts `u` and `v` are runtime values with no fact relating them. R5 requires proven-distinct indices; none exists. Rejected. The rejection is correct and the C++ program above is undefined behaviour exactly when u == v, which for a self-loop or a block that is its own predecessor is a real input, not a hypothetical.

Rewrite:

```text
// (a) If the program genuinely guarantees u != v, make the guarantee a fact once, outside the loop:
if u == v { slow_path(g, u) } else {
    invariant { use u != v }                 // now R5 proves the subscripts distinct
    with &g.adj[u] as au { for k in 0..au.len { ... push(&g.adj[v], u) ... } }
}
// (b) If u == v is genuinely possible, re-derive per iteration (R7):
for k in 0..len_of_adj(g, u) { w = elem(g, u, k); if needs_phi(w) { push(&g.adj[v], u) } }
```

Cost: Rewrite (a) is zero-cost in the loop: one runtime compare and branch executed once per outer traversal step, and the inner loop keeps `au`'s data pointer and len in registers exactly as C++ does. Rewrite (b) costs, per iteration, a reload of the inner Vector's descriptor: `g.adj.buf` (register-resident), then `g.adj.buf[u].buf` and `.len` -- 2 dependent L1 loads and a shift-add, where C++ hoists an iterator pair into registers. In a DFS inner loop of ~6 instructions that is a real 20-30%. The honest accounting: WF makes you pay (b) only in exactly the situation where the C++ code was already wrong, and gives you (a) for free where the C++ code was right. The residual real cost is the outer branch in (a), which is noise.

#### Arena-allocated IR with stable node references (rustc &'tcx, Clang Stmt*, V8 Node*)

Kind: accepted-fine

```text
// Rust/C++: alloc hands back a reference that stays valid across further allocs,
// because arena chunks never move. This is rustc's entire TyCtxt design.
let ty: Ty<'tcx> = tcx.mk_ref(region, TypeAndMut { ty: inner, mutbl });  // &'tcx TyKind
match ty.kind() { ... }   // one dependent load

// WF: impossible to even write.
fn alloc(a: &Arena<Node>, n: Node) -> &Node   // REJECTED at the signature
```

Trace: R2 forbids a reference as a return value outright -- references exist only as a call argument or a `with` part-name. R1 additionally forbids any aggregate from containing a reference, so no IR node can hold `&Node` children. This is rejected by the rule set's spine, not by the reallocation rule, and no `with` block can span the lifetime of an IR node that outlives the function that made it.

Rewrite:

```text
// Handles. Homogeneous flat pool:
struct Arena<T> { items: Vector<T> }
fn alloc(a: &Arena<T>, x: T) -> u32 writes(a.items.buf), writes(a.items.len) { push(&a.items, move(x)); a.items.len - 1 }
// Heterogeneous AST: per-kind pools + a packed tagged handle (6 bits kind, 26 bits index).
struct Ast { ints: Vector<IntLit>, calls: Vector<CallExpr>, bins: Vector<BinOp>, ... }
```

Cost: Zero to negative, and this surprised me. The flat-pool handle is a u32 against an 8-byte pointer: the IR shrinks, which is why rustc's own newtype-index/IndexVec style and LLVM's value numbering already prefer indices. Deref is `base + idx*size` with `base` register-resident across a whole pass -- the same address computation the pointer version does, minus 4 bytes per edge. The heterogeneous case is also fine: the kind tag rides in the handle register, so the jump table dispatches with no load, while `Expr*` must load a kind field from memory first. The one thing WF genuinely cannot do is a heterogeneous *byte* arena (`base + byte_offset` reinterpreted as a node), because there is no type punning without unsafe -- but per-kind pools dominate it anyway. I could not construct a cost here.

#### Variable-length tail inline in an object: LLVM User with its operand array, kernel sk_buff, DOM attribute list

Kind: rejected-real-cost

```text
// C++/LLVM: the Use array is allocated immediately before the User object, in the
// SAME allocation and usually the same cache line. op_begin() is pointer arithmetic
// off `this`, with ZERO loads and ZERO extra cache lines.
Use *User::op_begin() { return hung_off_uses ? ... : (Use*)this - NumUserOperands; }
for (Use &U : I->operands()) { visit(U.get()); }   // the hottest loop in LLVM

// WF: a struct cannot have a runtime-length tail.
struct User { opcode: u8, ops: DynBox<Handle> }    // ops is a SEPARATE heap block, always
```

Trace: R1 defines `Box<T>` as an owning pointer to one heap object and forbids any aggregate from containing a reference; R8's DynBox is likewise one heap block. Nothing in R1-R10 provides a flexible array member or lets a struct's storage extend past its static size. So the C idiom `struct { u32 n; T data[]; }` -- one allocation, one cache line, zero indirection -- has no WF spelling. This is not the reallocation rule; it is R1's shape of Box. The WF program is accepted by the checker, but the only representable shape carries a cost.

Rewrite:

```text
// Best available: one global operand pool, and the User stores a range, not a box.
struct Module { users: Vector<User>, ops: Vector<Handle>, free_runs: Vector<Run> }
struct User { opcode: u8, op_off: u32, op_len: u32 }
fn operand(m: &Module, u: u32, k: u32) -> Handle
    requires k < m.users.buf[u].op_len
{ with &m.users.buf[u] as uu { m.ops.buf[uu.op_off + k] } }
```

Cost: Real and unavoidable. Per User visit: C++ touches one cache line for the User and its operands together; WF touches the User's line plus a second line in `m.ops` at `op_off`, a separate dependent miss on any IR that does not fit in L2. On a memory-bound LLVM pass walking use-def chains that is the difference between one and two cache misses per hop -- I would expect 15-30% on such a pass, and it is not recoverable, because R1 forbids the inline tail and forbids storing the operand reference in the User. The naive alternative (`ops: DynBox<Handle>` per User) is worse: one extra *allocation* per instruction, which is precisely the cost LLVM invented hung-off-uses to avoid. The pool rewrite also drags in a free-run list and fragmentation that C++ does not pay. This is the strongest cost I found in the whole exercise, and it is charged to R1, not to R5.

#### One moving hole at a runtime index: insertion-sort inner loop on a non-Copy element (the hot loop inside every hybrid sort)

Kind: accepted-fine

```text
// The inner loop of std::sort / pdqsort / slice::sort for T = String, Box<T>, etc.
// ONE hole, at a runtime index, moving left. Rust: one move per step.
fn insert_one(a: &DynBox<T>, len: u64, i: u64)
    requires i < len
{
    tmp = take(&a[i])                       // hole opens at i
    j = i
    invariant {
        // hole at runtime index j, expressed as THREE ranges with runtime endpoints
        forall p < j: full(a[p]);  empty(a[j]);  forall j < p < len: full(a[p])
    }
    while j > 0 && greater(&a[j-1], &tmp) {
        put(&a[j], take(&a[j-1]))           // hole moves j -> j-1
        j -= 1
    }
    put(&a[j], move(tmp))                   // hole closes
}
```

Trace: R8 says slot-state facts are range facts and that the number of simultaneously open holes at unrelated runtime indices is statically bounded, with `Option<T>` as the fallback beyond the bound. The reading that matters: this loop never has a hole at an *unrelated* index. It has exactly one hole whose position is a tracked runtime value, and the state is a three-range decomposition `[0,j)` full, `[j,j+1)` empty, `[j+1,len)` full with runtime endpoint j. Runtime range endpoints are exactly what R5's 'proven-distinct ranges' already compares, and the project's range-loan work already carries runtime endpoints through formation and overlap. So the invariant is in the admitted vocabulary and the loop is ACCEPTED. Hoare partition (two holes at unrelated runtime indices i < j) is the same argument with five ranges and needs only a static bound of 2. R8's *wording* is what misleads: 'range fact' read narrowly as the prefix form `forall i < len` would reject this loop. The wording, not the mechanism, is the gap.

Rewrite:

```text
n/a
```

Cost: Zero, under the three-range reading. Under the narrow reading the fallback R8 itself names costs a great deal, and is worth stating so the wording gets fixed: `Option<(u64,u64)>` is 24 bytes against 16, a 50% cache-footprint increase on hash entries, plus a tag test on every comparison. And a swap-based rewrite that avoids holes entirely costs three element moves per step against one: on `String` that is 72 bytes of traffic per inner step instead of 24, a 3x in the hottest loop of the sort. Neither fallback is needed, but the rule text should say so.

#### Pipelined producer/consumer over a shared buffer: compositor display list, io_uring SQ/CQ, audio callback ring

Kind: rejected-real-cost

```text
// C++/Rust: producer writes ring[head], consumer reads ring[tail], disjoint at runtime
// because head != tail is maintained by acquire/release counters.
par {
    produce: while running { put(&ring.buf[ring.head & mask], item); release(ring.head + 1) }
    consume: while running { t = acquire(ring.tail); use(&ring.buf[t & mask]); ... }
}
```

Trace: R9 admits `par { A; B }` iff A's writes are disjoint from B's reads and writes by the same path-overlap check as R5. A writes `ring.buf[head & mask]`, B reads `ring.buf[tail & mask]`. Same root, same field, subscripts are runtime values whose distinctness is established by an atomic protocol. R5 admits distinctness only from distinct static roots or proven-distinct indices/ranges, and R1-R10 contain no atomics, no memory model, and no vocabulary in which `head != tail` could be a carried fact across two concurrently executing arms. Rejected, and there is no `use` step that can rescue it, because the fact does not hold in the sequential sense R6 describes -- it holds under a concurrent protocol the rules do not model.

Rewrite:

```text
// The only shape R9 admits: static partition, hence fork-join, hence double buffering.
loop {
    par { fill(&front, 0, n/2); fill(&front, n/2, n) }   // proven-disjoint ranges
    swap_buffers(&front, &back)                          // join
    consume_all(&back)
}
```

Cost: Lost pipelining, the one cost on the lens's list I could not rewrite away. Latency goes from one item to one full buffer period: for a 48kHz audio callback or a 120Hz compositor that is not a percentage, it is a missed deadline. Plus a second buffer's allocation and a barrier per buffer. I am reporting this as OUT OF SCOPE for the claim under test -- nothing here is about reallocation, and a ring buffer does not grow -- but it is the place where the path-comparison discipline runs out: R5/R9 can express 'these ranges are disjoint because their endpoints compare' and cannot express 'these indices are disjoint because a protocol keeps them so'. If WF wants lock-free SPSC it needs a concurrency vocabulary, not a better reallocation rule.

#### Editor cursor held across a B-tree/rope leaf split (re-derivation is O(log n), not 'one address computation')

Kind: rejected-zero-cost

```text
// WF, rejected
with &rope.root.*.kids[i].*.leaf.buf[j] as cur {
    insert_char(rope, c)          // may split the leaf AND the root: writes(rope.root)
    cur.attr = mark               // REJECTED
}
```

Trace: R3 admits the path through Box contents (`*b`) and makes the element a part of the owner's path. R4 gives `insert_char` the effect `writes(rope.root)` because a split can replace the root Box. R5: the live part-name's path is rooted at `rope.root` and the write path `rope.root` is its prefix. Rejected. This case matters because it falsifies the claim's *cost sentence*, not its verdict: the claim says re-derivation is 'one address computation'. Here R7 re-derivation is a fresh descent through k Box indirections -- k dependent loads, likely k cache misses -- because a Box handle cannot be cached anywhere (R1 forbids storing the reference, and caching the *index path* still requires reloading each child Box pointer on the way down).

Rewrite:

```text
// Flatten the leaves into one pool with stable slot indices; the cursor is a pair.
struct Rope { leaves: Vector<Leaf>, free: Vector<u32>, index: BTreeIndex }
// cursor = (leaf_slot: u32, offset: u32); insert_char does writes(leaves.buf) freely
for c in typed { insert_char(rope, cur, c); with &rope.leaves.buf[cur.leaf] as lf { lf.attr = mark } }
```

Cost: Effectively zero after the restructuring: re-derivation drops from O(log n) dependent loads to `leaves.buf` (register) plus one indexed load, against the C++ version's register-held `Leaf*`. So one L1 load per keystroke -- unmeasurable at typing rates, and unmeasurable even in a scripted bulk edit. But note what paid for it: the rules did not accept the natural nested-Box rope, they forced a specific representation (flat leaf pool with a free list). Since ugliness is free here that is an acceptable trade, and it happens to be the representation fast ropes already use. The finding to carry forward is that R7's cost claim should read 'one address computation *from a flat root*', because through a Box chain it is a full re-descent.

### Lens: facts (verdict: holds)

VERDICT RATIONALE (holds, not broken)

No case in this lens produced a proven unavoidable extra branch. The decided cases split cleanly into three groups: (1) facts that survive growth untouched because grow writes v.buf and not v.len -- the claim's core is stronger than the claim states; (2) facts recoverable by linear arithmetic over ensures, per-edge derivation with intersecting joins, and a fixpoint over a finite candidate-fact set -- all deterministic and terminating, so compatible with the no-SMT constraint; (3) facts genuinely lost, where the residual branch is exactly the bounds check Rust also keeps and LLVM also cannot remove. One case (swap_remove with a known i < j) is strictly better than safe Rust.

Three cases would flip the verdict, and all three are rule gaps rather than decided defects. In descending order of how much rides on them:

1. ENSURES IN FUNCTION TYPES (case 8). R4 declares effects on parameters, R6 consults a callee's ensures, and nothing says a function TYPE -- a generic bound, or the type of a stored closure per R1 -- may carry an ensures, nor whether a closure literal's inferred ensures flows into its bound. Every higher-order growth site depends on this. Rust recovers these facts non-modularly, by monomorphizing and inlining; WF's modular checking cannot, so without ensures in function types the rules pay one branch per higher-order growth site. This is the highest-value thing to settle.

2. THE FACT LANGUAGE (case 9). A hash probe needs `h & (cap-1) < cap`, a bitvector fact. R1-R10 never state what arithmetic the checker does and give no lemma facility. If it is linear-integer only, the hottest loop in most programs gains one compare+branch per probe that LLVM deletes for free via known-bits. The project's `use`-step mechanism closes this at zero cost, but `use` is not part of the rule set under test.

3. JOIN SEMANTICS OF R6 (case 3). Per-edge derivation followed by per-fact intersection keeps everything and needs no case split; joining raw ensures equalities produces a disjunction the core is not stated to handle and loses the fact. R6 does not say which.

Two further gaps found that are not cost questions but should be settled: grow as written carries no ensures, so push's own body does not check under R6 (case 2, zero-cost fix); and R9's disjointness check is over declared paths while R1 puts every allocation on one global heap that no path names, so two concurrently-allocating par arms are declared disjoint by a rule that cannot see the allocator (case 12).

SWIFT `inout`, since it was asked directly

`inout` is a parameter modifier, not a reference type: semantically copy-in / copy-out (call-by-value-result), lowered to pass-by-address when the compiler can prove that is observationally equivalent. `&x` at the call site is a required marker, not an address-of operator. An inout binding cannot be stored, returned, or captured by an escaping closure. That is exactly R2(a), shipped and used at scale for a decade, which is decent evidence that "reference only as a call argument" has an acceptable ergonomic ceiling -- especially for WF, where an AI writes the code and ugliness does not count.

Three things in Swift are directly worth borrowing or studying:

- `withUnsafeMutablePointer(to:_:)` and `array.withUnsafeMutableBufferPointer { ... }` are R2(b)'s `with` block, down to the shape: a scoped, non-escaping part-name, with the standard library itself written against it. Prior art that the block form is usable for real container internals.

- Swift's exclusivity rule (SE-0176) is R5's overlap check: overlapping accesses to the same storage where at least one is a modification are an error. The important lesson is a warning, not an endorsement -- Swift enforces it STATICALLY only for local variables and inout parameters, and falls back to a DYNAMIC check that traps for class properties, globals and statics (on by default in release since Swift 5). WF forbids traps, so WF must reject wherever Swift falls back. Swift is therefore not evidence that the static half suffices; its dynamic half carries the remainder. Also note Swift's path granularity is coarser than R3/R5: `foo(&a[i])` takes an exclusive access to the whole array `a` for the duration of the call, with no element-level or proven-distinct-index precision. WF's path model with proven-distinct indices is strictly more precise than shipped Swift.

- The recent direction is the real signal: `borrowing`/`consuming` parameter modifiers (SE-0377), noncopyable `~Copyable` types (SE-0390), nonescapable types with lifetime dependencies, and `Span`/`RawSpan` as safe borrowed views over contiguous storage. Read that as the experiment already run: Swift concluded that second-class `inout` plus scoped `withUnsafe...` was NOT enough for a zero-cost array/buffer API and is now adding lifetime-dependent borrows -- i.e. the first-class-reference-with-lifetimes answer WF is trying to avoid needing. Whatever forced Swift there is the pressure WF should expect, and the cases above suggest where it will arrive first (higher-order code and stored views).

What is not worth borrowing: Swift's Array is copy-on-write, so every mutation entry point pays an `isKnownUniquelyReferenced` branch plus retain/release traffic, and its answer to "realloc invalidates pointers" is "CoW makes it invisible". That is precisely the per-mutation branch this verdict is hunting for, paid unconditionally. WF's value semantics plus affine Box delete it, and that is the clearest place where WF is ahead of Swift rather than behind it.

#### Growth alone is fact-transparent: the bounds fact was never the thing at risk

Kind: accepted-fine

```text
fn touch_after_grow(v: &Vector<Int>, i: Int) writes(v.buf) {
    // entry fact from caller: i < v.len
    grow(&v)?                          // declared: writes(v.buf)
    with &v.buf[i] as p { *p += 1 }    // needs: i < v.len, i < v.buf.cap, full(v.buf[i])
}
```

Trace: R4 gives grow the effect set {writes(v.buf)}. R6 invalidates a fact only when a path the fact MENTIONS is written. The fact `i < v.len` mentions the local `i` and the path `v.len`; neither is in grow's write set, so R6 leaves it standing untouched. No ensures clause is consulted, no arithmetic is needed. This is stronger than the CLAIM states: the claim says the bounds fact is 'carried by ensures', but for a pure reallocation there is nothing to carry -- reallocation writes the buffer, not the length. Every genuine fact loss in this lens comes from length-CHANGING operations (push/pop/opaque callbacks), not from reallocation. What grow does invalidate is the v.buf-side facts (`i < v.buf.cap`, `full(v.buf[i])`), which is the subject of the next case. The address must be recomputed because v.buf.ptr changed, but that is a data dependency, not a proof obligation.

Cost: Zero on the fact side. One reload of v.buf.ptr plus one lea for the re-derived address -- identical to what Rust emits after `v.reserve(..)` because the same base pointer was invalidated.

#### push does not check against its own body: grow carries no ensures

Kind: undecided-rule-gap

```text
fn push(v: &Vector<T>, x: T) writes(v.buf), writes(v.len) ensures v.len == old(v.len) + 1 {
    if v.len == v.buf.cap { grow(v)? }     // writes(v.buf), no ensures
    put(&v.buf[v.len], move(x))            // needs v.len < v.buf.cap AND empty(v.buf[v.len])
    v.len += 1
}
```

Trace: R6 kills every fact mentioning `v.buf` at the grow call. That set includes the Vector invariant's second conjunct `forall len <= i < cap: empty(buf[i])` (R8 range fact, mentions v.buf) and the capacity fact `v.len < v.buf.cap`. grow's declared interface is `writes(v.buf) -> Result<(), OOM>` with no ensures, so under R6 nothing re-establishes either one, and `put` is rejected on the then-branch. The else branch is fine (`v.len != v.buf.cap` plus the struct invariant gives `v.len < v.buf.cap`), and the if-join then loses it again for the same reason as the conditional-growth case below. So the library code the CLAIM is built on does not typecheck under the rule set as literally written. This is a gap, not a defect: R8 says slot-state facts are 'maintained as invariants' while R6 says facts die on a write to a mentioned path, and R1-R10 never say whether a struct invariant is automatically re-asserted at a callee's return or must be restated in ensures.

Rewrite:

```text
fn grow(v: &Vector<T>) writes(v.buf) -> Result<(), OOM>
    ensures v.buf.cap > old(v.buf.cap)
         && forall i < v.len: full(v.buf[i])
         && forall v.len <= i < v.buf.cap: empty(v.buf[i])
```

Cost: Zero. Postconditions are erased before lowering, so restating the invariant in grow's ensures costs no instruction. Secondary note in the same function: `v.buf.cap * 2` can overflow, and R1-R10 give no invariant bounding cap, so an explicit overflow arm is required -- one branch per growth, i.e. amortized O(log n) total, and Rust's Vec pays the same 'capacity overflow' check.

#### Conditional growth: the fact survives only if the join intersects derived facts, not raw ensures

Kind: undecided-rule-gap

```text
fn bump(v: &Vector<Cell>, i: Int) writes(v.buf), writes(v.len) {
    // i < v.len
    c = with &v.buf[i] as p { p.count + 1 }
    if c > THRESHOLD { push(&v, Cell{count: 0})? }   // taken: v.len == old+1 ; skipped: v.len == old
    with &v.buf[i] as q { q.count = c }              // needs i < v.len at the merge
}
```

Trace: The split itself is forced and correct: holding `p` across `push` is a live part-name on path v.buf[i] against a `writes(v.buf)` effect, rejected by R5, exactly as Rust's borrow checker rejects it. The interesting residual is the merge. Taken edge: `i < old(v.len)` and `v.len == old(v.len)+1` give `i < v.len` by linear arithmetic. Skipped edge: the fact was never invalidated. So `i < v.len` holds on both predecessors -- but only if derivation happens PER EDGE and the merge intersects the resulting fact sets. If instead the checker carries the raw ensures equalities to the merge and joins those, it gets `v.len == old OR v.len == old+1`, a disjunction that a no-SMT, no-case-split core is not stated to handle, and `i < v.len` is lost. R6 says nothing about the join. Per-edge derivation followed by intersection is deterministic, terminating, and needs no case split, so it is compatible with the no-SMT constraint -- but it is a reading, not a stated rule.

Rewrite:

```text
None needed if the join is per-edge-then-intersect. If the join is over raw ensures, the only rewrite is `if c > THRESHOLD { push(..)?; with &v.buf[i] as q { q.count = c } } else { with &v.buf[i] as q { q.count = c } }` -- duplicate the tail into both arms so no merge is crossed.
```

Cost: Zero under the good reading. Under the bad reading with the duplication rewrite: code-size duplication of the tail, still zero branches. Under the bad reading WITHOUT the rewrite (runtime revalidation): one compare+branch beyond Rust -- LLVM after inlining push has `len` in the range [old, old+1] and kills the bounds check with no annotation at all.

#### Data-dependent pushes in a loop, index fixed before the loop

Kind: accepted-fine

```text
fn scan(v: &Vector<Int>, src: &Vector<Int>, i: Int) writes(v.buf), writes(v.len) {
    // i < v.len
    for j in 0..src.len {
        x = with &src.buf[j] as s { *s }
        if x > 0 { push(&v, x)? }
    }
    with &v.buf[i] as p { *p += 1 }        // needs i < v.len
}
```

Trace: R6 kills `i < v.len` at the push inside the body, so recovery has to be a loop invariant rather than a straight-line derivation. `i < v.len` holds on the entry edge, is preserved on the push edge (v.len == old+1, plus the merge of the previous case), and is preserved on the skip edge trivially. The checker must therefore run a fixpoint over a candidate fact set across the back edge. That is terminating and deterministic: the candidate predicates are generated from the index expressions occurring in the program (a finite set) and the lattice over them is finite, so no work budget or solver state selects acceptance. R6 does not say the checker does this, but nothing in R1-R10 forbids it and no explicit `use` step is required -- the invariant is the fact itself.

Cost: Zero branches. The address &v.buf[i] after the loop costs one reload of v.buf.ptr plus one lea, because v.buf was written; Rust reloads the same base pointer for the same reason and additionally keeps its bounds check on `v[i]` unless it can prove the range, which it can here after inlining. Call it equal.

#### Index produced inside the data-dependent loop: rejection is semantically correct

Kind: rejected-zero-cost

```text
fn last_pushed(v: &Vector<Int>, src: &Vector<Int>) writes(v.buf), writes(v.len) -> Int {
    last = 0
    for j in 0..src.len {
        x = with &src.buf[j] as s { *s }
        if x > 0 { push(&v, x)?; last = v.len - 1 }
    }
    with &v.buf[last] as p { *p }          // needs last < v.len
}
```

Trace: On the push edge `v.len == old+1` and `last = v.len - 1` give `last < v.len`, and later iterations only grow v.len so the fact is preserved; the skip edge preserves it unchanged. The fixpoint nevertheless fails because on the LOOP ENTRY edge the candidate invariant is `0 < v.len`, which is false when v is empty. This is not a checker weakness: the program really is wrong when v is empty and no x is positive. R6 rejects for the right reason.

Rewrite:

```text
found = false; last = 0
for j in 0..src.len {
    x = with &src.buf[j] as s { *s }
    if x > 0 { push(&v, x)?; last = v.len - 1; found = true }
}
if found { with &v.buf[last] as p { *p } } else { 0 }
```

Cost: One compare+branch after the loop. Rust writes `v[last]` and pays exactly one cmp/jae into a panic block that LLVM cannot remove (`last` is loop-carried and data-dependent). WF pays one cmp/jb plus a cold else-arm that must produce a value instead of diverging. Extra cost over Rust: at most one register carrying `found` across the loop, and even that usually folds into an existing flag. No extra hot-path branch.

#### pop before re-derivation: the fact is genuinely and correctly unrecoverable

Kind: rejected-zero-cost

```text
fn drop_last_then_touch(v: &Vector<Int>, i: Int) writes(v.buf), writes(v.len) {
    // i < v.len
    pop(&v)                                 // ensures v.len == old(v.len) - 1
    with &v.buf[i] as p { *p += 1 }         // needs i < v.len
}
```

Trace: `i < old(v.len)` together with `v.len == old(v.len) - 1` yields only `i <= v.len`. No ensures clause can strengthen this, because for `i == old(v.len)-1` the desired conclusion is false. R6 refuses, correctly. Same for truncate (`v.len <= old(v.len)`) and for any helper whose ensures permits shrinking.

Rewrite:

```text
if i < v.len { with &v.buf[i] as p { *p += 1 } } else { /* caller-chosen recovery */ }
```

Cost: One compare+branch -- bit-for-bit the same test Rust emits for `v[i]`, which LLVM also cannot remove here (i opaque, len reduced by an amount it must still track). The only structural difference is the failure edge: Rust's is `panic!`, marked noreturn, carrying no live state; WF's must be an ordinary arm. If that arm is `return Err(..)` or `continue` the cost is identical; if it must produce a merged value, WF gains one phi node. No extra hot-path instruction.

#### swap_remove with a known index ordering: WF removes a branch Rust keeps

Kind: accepted-fine

```text
fn compact_touch(v: &Vector<Int>, i: Int, j: Int) writes(v.buf), writes(v.len) {
    // facts: i < j, j < v.len
    swap_remove(&v, j)                      // ensures v.len == old(v.len) - 1
    with &v.buf[i] as p { *p += 1 }         // needs i < v.len
}
```

Trace: From `i < j` and `j < old(v.len)` we get `i + 1 <= j <= old(v.len) - 1 == v.len`, hence `i < v.len`. Pure linear integer reasoning over one ensures equality and two premises: no case split, no quantifier instantiation, no SMT. R6 restores the fact and R5 sees no live part-name across the call, so the `with` is accepted with no runtime test.

Cost: Negative: WF emits no branch here. Safe Rust writes `v[i]` and keeps the bounds check, because `i < j` is a fact the source never states and LLVM has no way to learn it when i and j come from opaque sources (two `find` calls, say). The effect/ensures discipline buys back one branch that Rust pays. Worth recording because the rest of this lens is a cost hunt: the tax is not uniform.

#### Growth inside a generic callback: the modularity gap, and the load-bearing one

Kind: undecided-rule-gap

```text
fn for_each_key<F>(t: &Vector<Int>, v: &Vector<Int>, cb: F)
    reads(t), writes(v.buf), writes(v.len)
    where F: fn(&Vector<Int>, Int) writes(v.buf), writes(v.len)
{
    for j in 0..t.len { cb(&v, with &t.buf[j] as s { *s }) }
}

fn caller(v: &Vector<Int>, t: &Vector<Int>, i: Int) writes(v.buf), writes(v.len) {
    // i < v.len
    for_each_key(&t, &v, |w, k| { push(&w, k)? })      // appends only, never shrinks
    with &v.buf[i] as p { *p += 1 }                    // needs i < v.len
}
```

Trace: R4 checks bodies modularly against declared effects, so at the call site only for_each_key's declaration is visible. It declares writes(v.len) and no ensures, so R6 kills `i < v.len` and nothing restores it. The concrete closure's behaviour is not admissible evidence, because for_each_key was checked once against its declaration, not per instantiation. Rejected. Rust, by contrast, monomorphizes on the closure type, LLVM inlines both levels, sees only `len += 1`, derives `len >= old(len)`, and deletes the bounds check with no annotation. So under R1-R10 as written this is a real branch beyond Rust at every higher-order growth site. The rewrite that makes it free requires a postcondition to appear in a FUNCTION TYPE (a generic bound, or the type of a closure stored per R1), and requires a closure literal's inferred ensures to flow into that bound. R4 says signatures declare effects on parameters; R6 mentions a callee's ensures. Neither says a function type may carry an ensures. That is the gap, and the CLAIM's zero-cost property for all higher-order code depends on how it is closed.

Rewrite:

```text
fn for_each_key<F>(t: &Vector<Int>, v: &Vector<Int>, cb: F)
    reads(t), writes(v.buf), writes(v.len)
    ensures v.len >= old(v.len)
    where F: fn(&Vector<Int>, Int) writes(v.buf), writes(v.len) ensures v.len >= old(v.len)
{ ... }
// then the loop fixpoint of the earlier case carries i < v.len through the call
```

Cost: Zero if ensures may appear in function types (postconditions are erased). One compare+branch beyond Rust per higher-order growth site if they may not. Separate, harder residual worth stating: a callback that can genuinely shrink, e.g. `|w,_| { if w.len > 100 { pop(&w) } else { push(&w,1) } }`, is unreachable by any ensures -- but there the branch is semantically required (the index really can go out of range) and safe Rust pays it too unless constant propagation happens to resolve the test. Hand-specializing the callback at the call site makes WF and Rust identical.

#### Hash probe after rehash: the fact language, not the invalidation, is the wall

Kind: undecided-rule-gap

```text
struct Table<K,V> {
    buf: DynBox<Entry<K,V>>
    cap: Int
    len: Int
    invariant cap == buf.cap;  invariant is_pow2(cap)
}

fn insert(t: &Table<K,V>, k: K, x: V) writes(t.buf), writes(t.cap), writes(t.len) {
    if t.len * 2 >= t.cap { rehash(&t)? }        // writes(t.buf), writes(t.cap)
    i = hash(k) & (t.cap - 1)                     // needs i < t.buf.cap
    loop {
        with &t.buf[i] as s { if empty_slot(s) { put(s, Entry{k,x}); break } }
        i = (i + 1) & (t.cap - 1)                 // needs i < t.buf.cap, every iteration
    }
}
```

Trace: The invalidation half is fine: R6 kills `cap == buf.cap` and `is_pow2(cap)` at the rehash call, and rehash can restate both in its ensures at zero runtime cost. The wall is the DERIVATION: from `is_pow2(c)` conclude `h & (c-1) < c` for arbitrary runtime h and c. That is a bitvector fact, not a linear-integer one. R1-R10 never specify the fact language and give no lemma facility. The surrounding project text says harder proofs arrive as explicit finite `use` steps inside a local `invariant`, which would close this cleanly -- but a `use` step is not one of R1-R10, so under the rule set as stated this is undecided. This is the single case in this lens that would make the verdict `broken`.

Rewrite:

```text
With a lemma step: `invariant { use pow2_mask(t.cap, hash(k)) }` before the probe, restoring `i < t.cap`, erased at lowering, zero cost. Under R1-R10 alone I could not find a zero-cost rewrite. Storing the mask instead of the capacity (`invariant mask + 1 == buf.cap`) relocates but does not discharge the same bitvector step; a modulo formulation (`i = hash(k) % t.cap` with `cap > 0`) is provable by linear reasoning but replaces the mask with a division, which is far worse than the branch it removes.
```

Cost: If the fact language is linear-integer only and no lemma step exists: one compare+branch per probe iteration, in the hottest loop most real programs contain, plus a cold out-of-range arm that must produce a value -- so the probe loop gains a merge point and stops being a three-instruction cycle. Rust/LLVM removes this check for free via known-bits (`x & (c-1) <= c-1 < c`). That is a genuine extra branch beyond Rust, contingent entirely on how the fact language is specified.

#### Two vectors in lockstep: rejection is honest, and the rewrite is faster than the original

Kind: rejected-zero-cost

```text
i = find(&keys, k)?              // ensures i < keys.len
push(&vals, x)?                  // writes vals.*, says nothing about keys
with &vals.buf[i] as p { *p = y }   // needs i < vals.len
```

Trace: The only fact in scope is `i < keys.len`. `keys.len` and `vals.len` have distinct static roots; R6 has no rule relating them and no ensures on push can create one, because push knows nothing about keys. Rejected. The rejection is honest rather than conservative: nothing in the program ever states the lockstep, so the checker is refusing an unstated assumption, not failing to find a proof.

Rewrite:

```text
struct Table<K,V> { keys: Vector<K>, vals: Vector<V>, invariant keys.len == vals.len }

fn push_pair(t: &Table<K,V>, k: K, x: V) writes(t.keys), writes(t.vals) -> Result<(), OOM>
    ensures t.keys.len == old(t.keys.len) + 1
{
    reserve(&t.keys, 1)?                  // both growths up front
    reserve(&t.vals, 1)?
    push_nogrow(&t.keys, k)               // cannot fail, no capacity branch
    push_nogrow(&t.vals, x)
}
// then i < t.keys.len together with the invariant gives i < t.vals.len
```

Cost: Zero, arguably negative: reserve performs the same capacity compare push already did, and splitting push into reserve+push_nogrow removes the second capacity branch rather than adding one. It also repairs an R10 hole in the naive version, where an OOM between the two pushes would leave the invariant broken with k and x already consumed. Sub-gap worth flagging: R8 shows invariants on a struct but R1-R10 never say WHEN an invariant must hold. If it must hold after every statement, push_pair is unwritable (it is broken between the two push_nogrow calls) and the only legal representation is Vector<(K,V)>. That is a real, measurable cost, not a rewrite annoyance: forcing AoS makes a key-only probe loop touch sizeof(K)+sizeof(V) bytes per probe instead of sizeof(K), roughly 8x the cache lines for an 8-byte key against a 56-byte value.

#### Index loaded from the heap and revalidated later

Kind: rejected-zero-cost

```text
for j in 0..jobs.len {
    id = with &jobs.buf[j] as s { s.idx }     // arbitrary Int out of memory
    push(&v, compute(id))?                     // may reallocate
    with &v.buf[id] as p { *p += 1 }           // needs id < v.len
}
```

Trace: R6 is not even engaged: `id` is a loaded value and no static fact ever constrained it, so the write to v.len is irrelevant -- the fact never existed to be invalidated. Rejected for absence of a premise. Note that storing the index in a struct is perfectly legal under R1/R2 (an Int is not a reference), so this pattern is the language's intended answer to 'pointer invalidated by realloc': indices survive reallocation by construction, which is the whole reason value semantics buy anything here.

Rewrite:

```text
if id < v.len { with &v.buf[id] as p { *p += 1 } } else { continue }
```

Cost: One compare+branch per iteration -- exactly the bounds check Rust emits for `v[id]`, which LLVM cannot remove for precisely the same reason. Equal. Worth stating because this is the common case in real code and the no-trap rule costs nothing here: it replaces Rust's panic edge with a continue edge.

#### Growth inside par: the allocator is not a path

Kind: undecided-rule-gap

```text
par {
    push(&a, x)?      // writes(a.buf), writes(a.len) ; may call DynBox::empty and free
    push(&b, y)?      // writes(b.buf), writes(b.len)
}
```

Trace: R9 defers to the same path-overlap check as R5. `a.*` and `b.*` have distinct static roots, so the two arms are declared disjoint and the par is accepted. But R1 places every Box and DynBox on ONE global heap, and DynBox::empty and free mutate that heap's allocator state. No effect path under R4 names it, so R5 and R9 are structurally unable to see the conflict. The rules do not resolve this, and it is a soundness question rather than a cost question. Two consistent resolutions: (a) allocation is a runtime-provided atomic or locked operation outside the effect system, which costs exactly what Rust's thread-safe global allocator costs, i.e. nothing extra; or (b) the heap becomes a named effect path, in which case every concurrently-allocating par is rejected -- including parallel map producing vectors, the single most common parallel pattern -- and the only rewrite is per-task arenas, which is not expressible without a second heap root. The straightforward conflicting case (`par { read(&v.buf[0..k]); push(&v, x) }`) is correctly rejected by R9 on overlapping v.buf, matching Rust's refusal to share &v and &mut v across threads, so it costs nothing relative to Rust.

Cost: Under resolution (a): zero relative to Rust. Under resolution (b): total loss of parallelism for any parallel producer of heap data, which is the largest single cost anywhere in this lens -- but it is an R9/R1 interaction, not a fact-invalidation defect.

#### Alternating two element references across a write that cannot be proven disjoint

Kind: rejected-real-cost

```text
fn alternate(v: &Vector<Int>, i: Int, k: Int, n: Int) writes(v.buf) {
    // i < v.len, k < v.len, both from runtime data, i != k not provable
    for _ in 0..n {
        with &v.buf[i] as p { *p = f(*p) }
        with &v.buf[k] as q { *q = g(*q) }
    }
}
```

Trace: A single with-block covering both part-names is rejected by R5, which needs proven-distinct indices and does not have them. Splitting into two sequential blocks is accepted: each block re-derives its address from v.buf.ptr and its index. R6/R7 are satisfied trivially (no length changes), so this is purely the R7 re-derivation cost that the CLAIM calls 'one address computation'.

Rewrite:

```text
The two-block form above is already the best available under R1-R10. There is no formulation that keeps both addresses live, because R2 forbids a reference in an ordinary variable and R5 forbids two overlapping unproven part-names.
```

Cost: Two extra address computations per iteration (ptr + i*8 and ptr + k*8) where C++ holds T* p and T* q in registers across the whole loop: 2-4 instructions per iteration, no branch, no allocation, no lost parallelism. Against SAFE Rust the cost is zero -- safe Rust cannot hold two &mut into one Vec either, re-indexes identically, and additionally pays two bounds checks per iteration that WF does not, since i < v.len and k < v.len are static facts here. Mitigation worth noting: because R2/R5 guarantee v is unaliased for the loop's duration, the WF backend may hoist v.buf.ptr into a register for the whole loop, which C++ may not do without restrict. Net: WF is level with ordinary C++ and behind restrict-annotated C++ by the two lea instructions.

### Lens: slots (verdict: broken)

SCOPE OF THE VERDICT. The claim as literally stated — "Vector reallocation never needs runtime reasoning; every rejected program has an equal-performance rewrite" — holds for a Vector<T> with T stored inline (G8, G2 rewrite 1, G4). R5's path comparison rejects exactly the dangling cases without ever asking whether a realloc occurs, R7's re-derivation costs one address computation that a correct C++ program also pays, and R8 is proved entirely at compile time so grow lowers to a memcpy with no per-slot runtime tag and free's "all slots empty" needs no scan. That positive result is real and is the strongest thing in this lens.

I set the verdict to broken for two findings that the lens's own mandated cases produced, neither of which is about realloc timing:

1. UNAVOIDABLE RUNTIME COST (G7). R8's fact language is range facts only, so occupancy determined by data (open addressing, slab free lists, any struct-of-arrays split of a tagged union) must fall back to Option<T> as data. For a niche-free element that is +41% bytes per slot versus a control-byte layout, roughly 1.5x cache lines touched per probe, plus one unremovable discriminant branch per successful probe. The missing ingredient is narrow and SMT-free: an index-correlated invariant between sibling arrays, `forall i: ctrl[i] matched <=> full(slots[i])`, invalidated by a write to either path. The language already lets a discriminant on the *same* path license an unwrap; R8 refuses the same licensing across two paths at the same index. Until that exists, the standard hash map cannot be written in WF at C++ performance, which is the exact outcome the no-unsafe-escape promise is supposed to prevent.

2. AN ACCEPTED UNSAFE PROGRAM UNDER THE LITERAL RULES (G2). The struct `invariant:` clause has no stated binding rule. push and grow declare no `requires`, R5 only compares paths and sees no conflict, so `take(&v.buf[k]); push(v, y); put(&v.buf[k], x)` is accepted and grow then calls take on an empty slot — an uninitialized read, and for T = Box<Node> a duplicated owner. The intended reading obviously rejects it; the rule set just does not contain the sentence. I am reporting it as undecided rather than resolving it, but it is the single most important sentence missing.

THE ONE DECISION THAT IS WORTH MEASURABLE PERFORMANCE. When the invariant binding is written down, the choice is between "the struct invariant is an implicit precondition of every call taking `&v`" and "each callee states the slot-state facts it needs" (grow needs full[0,len); a capacity-guaranteed append needs only empty(buf[len])). The first reading forces G5's rewrite A and costs two dependent loads per iteration in the Vector<Box<Node>> loop, the commonest heap-container shape in the language. The second reading admits G5's rewrite B at zero cost. Recommend per-callee requires.

OTHER GAPS FOUND, ALL CLOSABLE WITHOUT RUNTIME COST. (a) R4/R6 cannot express a variant-conditioned `ensures`, so every fallible mutator — starting with push itself, whose declared `ensures v.len == old(v.len)+1` is false on the OOM path — is under-specified; and the prompt's push destroys the moved-in element on failure instead of returning it (G1). (b) R5 says "proven-distinct indices/ranges" without naming the fixed family; the half-split in G3 needs interval reasoning on affine endpoints, which is decidable and deterministic but must be specified. (c) R8 does not say whether adjacent range facts splice automatically; if not, one erased `use` step suffices (G3). (d) R4 admits "moving out of" a field through a second-class reference (grow's `free(move(v.buf))`) but states no definite-reinitialization obligation; writing grow with `replace` avoids the question at identical cost (G6).

DESIGN SUGGESTION, NOT ADOPTED HERE. G5/G6 both want an effect strength between reads and writes — `relocates(p)`: the bytes at p may move, every Box descriptor reachable at p is moved, none freed or overwritten. A reference whose path crosses a deref strictly below p survives it; one stopping at or above the deref does not. grow and DynBox::resize would declare it, and a `&*v.buf[i]` into a heap Node would legitimately survive the realloc that does not move the Node. It stays a pure path comparison, no runtime cost, no SMT, and R1's guarantee that no value contains a reference is what makes it sound.

ON THE USER'S SWIFT QUESTION (one line, since it is adjacent to this lens): Swift `inout` is copy-in/copy-out by specification, not a pointer — the callee gets a local copy and the value is written back at return, with the compiler free to pass an address as an optimization when it can prove that is observationally equivalent, and exclusive-access enforcement (static where possible, a dynamic flag otherwise) preventing overlapping inout arguments. The borrowable part for WF is that `&path` at the call site with no first-class reference type is precisely Swift's shape, and that Swift's exclusivity rule is R5's path-overlap check with a runtime fallback WF does not want. The part not to borrow is the write-back itself: it costs a copy in each direction for large values, which is the kind of cost this design is trying to avoid.

#### G1. grow's OOM path: v survives, but push's contract and the moved-in element do not

Kind: undecided-rule-gap

```text
fn push(v: &Vector<T>, x: T) writes(v.buf), writes(v.len)
  ensures v.len == old(v.len) + 1
{
    if v.len == v.buf.cap { grow(v)? }     // <-- `?` returns Err out of push
    put(&v.buf[v.len], move(x))
    v.len += 1
}

// caller
let n1 = Box::new(Node{..})?               // T = Box<Node>, non-Copy, owns heap
push(v, move(n1))                          // signature says it cannot fail

```

Trace: R10 makes DynBox::empty return Result, so grow's failure point is the single allocation at the top, before any take/put. v.buf is untouched, every slot state is unchanged, and the struct invariant `forall i<len: full` still holds. That part is clean: there is no partially-planned state to unwind, because grow plans nothing before it allocates. Two things do break, both in the prompt's own code. (a) push is declared with no `-> Result` yet its body uses `grow(v)?`; either push is ill-typed or its `ensures v.len == old(v.len)+1` is false on the error path. R4 defines effects and R6 defines fact invalidation/re-establishment, but nothing in R1-R10 says an `ensures` may be conditioned on which Result variant is returned. So the rule set cannot express `ok => len+1, err => len unchanged`. (b) x was moved into push by value. On the error return x is not put anywhere; under R1's affine Box it is auto-freed, so the caller's Node is silently destroyed by a failed push. Not memory-unsafe, but it is data loss that a correct API must not have.

Rewrite:

```text
fn push(v: &Vector<T>, x: T) writes(v.buf), writes(v.len)
  -> Result<(), (OOM, T)>
  ensures ok  => v.len == old(v.len) + 1
  ensures err => v.len == old(v.len) && v.buf == old(v.buf)
{
    if v.len == v.buf.cap {
        if grow(v) is Err(e) { return Err((e, move(x))) }   // hand x back
    }
    put(&v.buf[v.len], move(x))
    v.len += 1
}
```

Cost: Zero on the hot path. The error path moves sizeof(T) bytes back out to the caller and the call site tests one discriminant; Rust's `Vec::try_reserve`/`push_within_capacity` has exactly the same shape, and C++ `push_back` avoids it only by throwing. The gap to close is in the rule set, not in codegen: R4/R6 must be extended with variant-conditioned `ensures`, otherwise every fallible mutator in the language is under-specified. Flagged undecided rather than resolved here.

#### G2. A hole at a runtime index across push — as written, R1-R10 accept an uninitialized read

Kind: undecided-rule-gap

```text
// k is a runtime index, k < v.len, T non-Copy
let x = take(&v.buf[k])       // form (a): the reference dies at the call; buf[k] now empty
let y = transform(move(x))    // work that produces a new element to push
push(v, move(y))              // <-- may call grow
put(&v.buf[k], move(x2))

```

Trace: Check R5 first. `&v.buf[k]` appears only as a call argument, so no part-name from a `with` block is live at the push. R5's clauses are: substitute arguments into callee effect paths, reject overlapping write/write or read/write pairs among the *call's own* effects, and reject a live `with` part-name overlapping a write. None of them fires: push's effects are writes(v.buf), writes(v.len), and nothing else is live. R5 accepts.

Now R8. push may call grow, and grow's loop does `put(&nb[i], take(&v.buf[i]))` for i in 0..v.len. With i == k it calls take on an empty slot. take is specified full -> empty; on an empty slot it moves out bytes that were already moved out, i.e. an uninitialized read, and for T = Box<Node> it hands out a second owner of a freed or never-owned pointer. So this is an accepted memory-unsafe program.

What should have stopped it is the struct `invariant: forall i < len: full(buf[i])`. But R1-R10 never state the binding rule for a struct invariant: nothing says it is an implicit precondition and postcondition of every function whose parameter path is rooted at that struct, and neither push nor grow carries a `requires` clause. R8 only says slot-state facts are 'maintained as invariants'. Under the intended reading (invariant is an implicit pre/post at every call taking `&Vector`) the push is rejected, because `full(v.buf[k])` is false at the call. Under the literal reading of R1-R10 it is accepted and unsound. This is the sharpest gap the lens found: it is one unwritten sentence, not a design flaw, but as stated the rule set does not have it.

Rewrite:

```text
Under the invariant-as-precondition reading the writer has three moves, in increasing cost:

(1) Do not open a hole at all. Mutate in place through a block, close it, then push:
    with &v.buf[k] as e { e.field = f(e.field) }
    push(v, move(y))?
    with &v.buf[k] as e { e.other = g(e.other) }   // re-derive: one shift-add + one load of v.buf.ptr

(2) If the value genuinely must leave the slot while pushes happen, reserve first so no grow can occur, and use a capacity-guaranteed append whose effect is the single slot:
    reserve(v, v.len + n)?                          // one allocation, hoisted
    let x = take(&v.buf[k])
    for j in 0..n { push_within_capacity(v, f(j)) } // writes(v.buf[v.len]), writes(v.len)
    put(&v.buf[k], move(x2))
  This only typechecks if the invariant is a precondition of the callees that need it (grow needs full[0,len); push_within_capacity needs only empty(buf[len])) rather than of every call taking `&v`. That distinction is exactly the undecided point above, and it is worth deciding in favour of per-callee `requires`.

(3) Close the hole by shrinking: swap the last element into k and drop len by one (swap_remove).
```

Cost: (1) costs one address recomputation and one reload of v.buf.ptr after the push — and a correct C++ program must reload too, because push_back may have reallocated, so this is parity, and WF additionally omits the bounds check that safe Rust's `v[k]` pays. (2) costs zero on the hot path; the reserve is an allocation a good C++ program also hoists. (3) costs one move of sizeof(T) plus a len adjustment. Note that Rust's borrow checker rejects the original shape too (`&mut v[k]` live across `v.push`), and the C++ analogue is undefined behaviour after realloc, so there is no correct first-class-reference baseline that the rewrites lose against.

#### G3. Parallel grow: the copy loop splits, and R8 costs nothing at runtime

Kind: accepted-fine

```text
fn grow(v: &Vector<T>) writes(v.buf) -> Result<(), OOM> {
    let m = v.len / 2
    let nb = DynBox::empty(v.buf.cap * 2)?
    par {
        for i in 0..m       { put(&nb[i], take(&v.buf[i])) }   // inv: nb[0,i) full, buf[0,i) empty
        for i in m..v.len   { put(&nb[i], take(&v.buf[i])) }   // inv: nb[m,i) full, buf[m,i) empty
    }
    // needs: forall i < len: full(nb[i]) and empty(v.buf[i])
    free(move(v.buf))
    v.buf = move(nb)
}
```

Trace: R9 asks for disjointness by the R5 path check. Branch A writes nb[0,m) and v.buf[0,m); branch B writes nb[m,len) and v.buf[m,len). The roots coincide, so acceptance depends on R5's 'proven-distinct indices/ranges'. R5 does not name the decision procedure; for two half-open ranges whose endpoints are affine in the same locals this is interval reasoning, decidable, deterministic and SMT-free, so it plausibly sits inside the fixed automatic family — but R1-R10 do not say so, and the boundary of that family is where the real engineering is. Treating the split as in-family, the par is accepted and there is no race and no reallocation inside it, because grow's own realloc happens before the par (nb is already allocated) and free/assign happen after.

The second obligation is fact splicing: after the par the writer needs `forall i < len: full(nb[i])` from the two per-branch range facts over [0,m) and [m,len). R8 says slot-state facts are range facts but does not say whether adjacent-range conjunction is automatic. If it is not, the preamble's escape applies: an explicit finite `use` step inside a local `invariant` discharges it, and proofs are erased before lowering, so the cost is zero.

The important positive result is what R8 costs at runtime: nothing. Slot states are static facts, not per-slot runtime tags, so `put(&nb[i], take(&v.buf[i]))` lowers to a byte move with no state store, the whole loop is a memcpy candidate, and `free(move(v.buf))`'s 'every slot empty' obligation is discharged statically with no runtime scan. A design that carried a full/empty bit per slot would pay one byte per element plus a scan per free; this one pays zero. That is the main thing R8 buys.

Cost: Zero versus a hand-written parallel memcpy in C++. The only residual question is whether the writer must supply one erased `use` step to splice the two range facts.

#### G4. par over two halves where one branch can push

Kind: rejected-zero-cost

```text
par {
    for i in 0..v.len/2       { transform(&v.buf[i]) }            // writes(v.buf[i])
    for i in v.len/2..v.len   { transform(&v.buf[i])
                                if cond(i) { push(v, make(i))? } } // writes(v.buf), writes(v.len)
}
```

Trace: R9 via R5. Branch B's push carries writes(v.buf), which overlaps branch A's writes(v.buf[i]) for every i — v.buf is a prefix of v.buf[i], so no index reasoning can separate them. Rejected. Also branch A reads v.len as its loop bound while B writes v.len: read/write overlap on the same root, rejected again. Both rejections are correct and physical: a realloc inside the par frees the array branch A is walking, which is a use-after-free in C++ and a data race on v.len in any language. R5 gets there without ever asking whether the realloc actually happens at runtime, which is the claim under test working exactly as advertised.

Rewrite:

```text
let n = v.len                                  // hoist the bound out of the par: one register
let mut extra_a = Vector::new()
let mut extra_b = Vector::new()
par {
    { for i in 0..n/2   { transform(&v.buf[i]); if cond(i) { push(&extra_a, make(i))? } } }
    { for i in n/2..n   { transform(&v.buf[i]); if cond(i) { push(&extra_b, make(i))? } } }
}
append(v, move(extra_a))?                      // after the par
append(v, move(extra_b))?
```

Cost: Two thread-local vectors plus a bulk memcpy of the appended elements at the merge. This is identical to what rayon's fold/reduce and every correct C++ parallel-append do; the alternative in a first-class-reference language is a mutex around push, which is strictly slower (an atomic per push, plus serialization at the realloc). So relative to the *correct* baseline the cost is zero, and relative to the naive shared-push baseline WF's rewrite is faster. If the writer instead wants a single shared buffer, `reserve(v, n + upper_bound)?` before the par plus a per-branch disjoint index range gives an in-place parallel fill with `writes(v.buf[i])` effects only — also accepted, also zero cost.

#### G5. Vector<Box<Node>>: a reference to the node survives realloc physically, but R3/R5 kill it

Kind: rejected-real-cost

```text
// v: Vector<Box<Node>>. grow moves the 8-byte descriptors; the Node objects never move.
with &*v.buf[i] as node {
    for k in 0..n {
        node.count += 1
        push(v, make_node()?)?      // writes(v.buf) -> may realloc the descriptor array
    }
}
```

Trace: R3: a reference to Box content is a part of the Box owner's path, so `node` has path v.buf[i] (via the deref). R5: push declares writes(v.buf); the live part-name `node` has a path overlapping v.buf; conflict, rejected. The rejection is sound under the rules but physically unnecessary: grow relocates only the descriptor array, and each Node stays at its heap address for the whole operation, since grow moves descriptors rather than freeing them. This is the one place in the lens where a demonstrably safe program is refused, and the refusal is structural: R3 deliberately folds the pointee's identity into the owner's path so that moving or freeing the owner can be caught, and that same folding cannot distinguish 'the owner's bytes moved' from 'the pointee died'.

Rewrite:

```text
Rewrite A (always legal, the claim's prescribed one): end the block, push, re-derive.
    for k in 0..n {
        with &*v.buf[i] as node { node.count += 1 }
        push(v, make_node()?)?
    }

Rewrite B (zero cost, but legal only under the per-callee-precondition reading of G2): hoist the Box out of the vector so the reference is rooted at a local, and make sure no grow can run while the slot is empty.
    reserve(v, v.len + n)?               // no grow inside the loop
    let b = take(&v.buf[i])              // 8-byte descriptor move; slot i is now a hole
    with &*b as node {                   // root is the local b, disjoint from every v.* path
        for k in 0..n {
            node.count += 1
            push_within_capacity(v, make_node()?)?   // writes(v.buf[v.len]), writes(v.len)
        }
    }
    put(&v.buf[i], move(b))
  R5 accepts the inner push because `node`'s root is the local b and the callee's paths are rooted at v: different static roots, disjoint. What decides B's legality is whether the hole at v.buf[i] blocks a call that does not need `full(v.buf[i])`.
```

Cost: Rewrite A: per iteration, one reload of v.buf.ptr plus a shift-add plus one dependent load of the descriptor — two dependent loads that C++ (`Node* n = v[i].get();` hoisted, well-defined across push_back because unique_ptr pointees do not move) pays zero of. On an L1 hit that is roughly 8-10 cycles per iteration on a loop body whose useful work is one increment; after a realloc memcpy the descriptor array is cold, so the first re-derivation after each growth is a cache miss. This is a real, unavoidable cost *under rewrite A*. Rewrite B recovers it entirely: two 8-byte moves hoisted out of the loop, zero per-iteration overhead. The decision that determines which one the writer gets is the undecided invariant-binding question in G2 — that question is therefore not cosmetic, it is worth two dependent loads per iteration in the most common heap-container shape in the language.

Design observation (not a rule I am adopting): the clean fix is a third effect strength between reads and writes — call it `relocates(p)` — meaning 'the bytes at p may move; every Box descriptor reachable at p is moved, none is freed or overwritten'. Under it, a reference whose path crosses a deref strictly below p stays live, while a reference stopping at or above the deref dies. grow would declare relocates(v.buf) instead of writes(v.buf), rewrite A becomes unnecessary, and the check stays a pure path comparison with no runtime cost and no SMT. R1's no-references-inside-values guarantee is what makes the byte move type-correct in the first place, so the ingredient is already present.

#### G6. What DynBox::resize(&buf, cap) must promise

Kind: undecided-rule-gap

```text
fn resize(b: &DynBox<T>, newcap: Int) writes(b) -> Result<(), OOM>
  requires newcap >= b.cap
  ensures  b.cap == newcap
  ensures  forall i < old(b.cap): state(b[i]) == old(state(b[i]))   // state-preserving, per slot
  ensures  forall i < old(b.cap): b[i] == old(b[i])                 // ghost value identity
  ensures  forall old(b.cap) <= i < newcap: empty(b[i])
  // failure: b unchanged in cap, states and contents

// then grow collapses to:
fn grow(v: &Vector<T>) writes(v.buf) -> Result<(), OOM> {
    resize(&v.buf, v.buf.cap * 2)?
}
```

Trace: Three things the contract must carry, and one it must not need. (1) State preservation must be *pointwise*, not a range fact: resize has to be callable on a buffer whose occupancy is full[0,k) + empty at k + full(k,len), and the postcondition `state(b[i]) == old(state(b[i]))` quantifies over a per-slot equality rather than asserting a shape. R8 says slot-state facts *are* range facts; a pointwise state-equality postcondition is not obviously inside that language, and whether it is admissible is undecided. This is also where R8's 'statically bounded number of simultaneous holes' comes from: each hole at an unrelated runtime index splits the range facts in two, so the bound is a consequence of the fact language, not an arbitrary limit. (2) Value identity across the move must be a ghost equality on non-Copy T (it is erased, so no Eq requirement leaks into the runtime). (3) On failure it must promise b entirely unchanged, so the `?` in grow leaves v's invariant intact — same shape as G1.

What it does *not* need is any new rule about references. R1 guarantees no value contains a reference, which is exactly the precondition that makes moving the bytes type-correct — this is the payoff R1 was bought for. R5 then handles the reference side for free: resize declares writes(b), so every live part-name under b is a conflict and dies before the call. The two rules compose with nothing added. The one wart is inherited from G5: because R3 roots `&*b[i]` at b, writes(b) also kills references to pointees that resize demonstrably does not move; a `relocates` strength would let resize declare the weaker effect honestly.

Separately, note that the prompt's hand-written grow does `free(move(v.buf)); v.buf = move(nb)`. R4 explicitly lists 'moving out of' under writes, so moving a struct field out through a second-class reference is permitted — but R1-R10 never state the matching definite-reinitialization obligation (that v.buf must be re-initialized on every path before grow returns, and that v must not be observable in between). The obligation is ordinary deterministic flow analysis, but it is unwritten. Writing grow as `let old = replace(&v.buf, move(nb)); free(move(old))` sidesteps it at identical cost (the same stores).

Cost: resize replaces the take/put loop with a single realloc, which for a bit-copyable T lets the allocator extend in place and skip the copy entirely — a genuine win over the loop, which always copies. Since slot states are erased, the loop version was already a memcpy candidate, so the difference is the in-place extension, not the copy. No runtime cost either way; the open questions are all in the fact language.

#### G7. Unbounded holes at data-dependent indices: open addressing, and why Option is not a free rewrite

Kind: rejected-real-cost

```text
// The smallest realistic algorithm whose occupancy is an arbitrary runtime subset.
struct Table<K, V> {
    ctrl:  DynBox<u8>       // per slot: EMPTY | DELETED | h2(hash)
    slots: DynBox<Entry<K,V>>
    // wanted invariant: forall i < cap: ctrl[i] is a hash byte  <=>  full(slots[i])
}
fn lookup(t: &Table<K,V>, k: &K) reads(t) -> Option<&V> {
    let mut i = h1(k) & (t.cap - 1)
    loop {
        if ctrl_group_match(&t.ctrl, i, h2(k)) {      // 16 ctrl bytes in one SIMD load
            let e = &t.slots[i]                        // requires full(slots[i])
            ...
        }
        ...
    }
}
```

Trace: Occupancy here is determined by hash values, so it is an arbitrary subset of [0, cap) that no range fact can describe, and the number of simultaneously empty slots is proportional to cap, i.e. not statically bounded. R8's own escape applies verbatim: 'beyond that the writer must use Option<T> as data'. So `slots: DynBox<Option<Entry<K,V>>>` with every slot statically full and occupancy carried in the data.

The precise thing R8 cannot state is the invariant in the comment above: a *correlated* fact between two arrays at the same index, `forall i: ctrl[i] is a hash byte <=> full(slots[i])`, invalidated whenever either path is written. That is not a range fact in R8's sense (it is a biconditional between a data value and a slot state), yet it is deterministically checkable without SMT — it is an ordinary loop/structure invariant. Worth noting the inconsistency: the language must already let `if opt is Some { ... }` license the unwrap, i.e. a data value licensing an initialization fact on the *same* path; R8 simply refuses the same licensing when the data lives on a *different* path at the same index. Correlated parallel arrays (SwissTable, ECS component arrays, slotmaps, arena free lists, any struct-of-arrays split of a tagged union) are the dominant systems idiom, so this restriction is not a corner case.

Rewrite:

```text
struct Table<K, V> {
    ctrl:  DynBox<u8>
    slots: DynBox<Option<Entry<K,V>>>    // every slot statically full; occupancy is data
}
// probe: SIMD-match on ctrl as before, then
//   match &t.slots[i] { Some(e) => ..., None => unreachable-but-must-be-handled }
```

Cost: Two distinct costs, and neither has a zero-cost escape under R1-R10 as written.

(1) Layout. For a niche-free Entry — say (u64, u64) — Option<Entry> is 24 bytes against 16 + 1 ctrl byte for hashbrown's layout: +41% bytes per slot. With 64-byte lines that is 2.67 slots per line instead of 4, so a probe sequence touches about 1.5x the cache lines, on the single data structure whose cost is entirely cache misses. If Entry contains a Box the niche makes Option free and this cost vanishes, so the tax lands exactly on the small-POD tables that are the common case. Keeping ctrl separate does preserve the SIMD group scan (the alternative, dropping ctrl and scanning interleaved Option discriminants, costs 16 strided loads per group instead of one — do not do that).

(2) The redundant test. Having matched ctrl[i], the writer still has to get past Option's discriminant to reach the Entry. Without the correlated fact `ctrl[i] matched => slots[i] is Some`, that is a branch per successful probe that hashbrown does not have. It predicts well, but it is a real branch, it blocks the load from issuing early, and the compiler cannot remove it because the proof it would need is precisely the fact the rule set cannot state.

The C++/Rust-with-unsafe baseline pays neither. Plain safe Rust pays both, which is why hashbrown is written with MaybeUninit — and 'the standard hash map needs unsafe' is exactly the outcome WF has promised not to accept. The minimal rule addition that removes both costs, without SMT and without a runtime tag, is index-correlated facts between sibling arrays: a declared invariant relating a data path and a slot-state path at the same index, invalidated by a write to either, re-established by the writes that maintain both together. I am flagging it, not adopting it. The same shape recurs in a slab allocator's free list (slot holds either a T or the next free index) and in any SoA split of an enum.

#### G8. Control: the case the claim is actually about, and it does hold

Kind: accepted-fine

```text
with &v.buf[i] as e {
    e.count += 1
    push(v, x)?          // rejected here
    e.count += 1
}
// rewrite
with &v.buf[i] as e { e.count += 1 }
push(v, x)?              // ensures v.len == old(v.len) + 1, so i < v.len survives (R6)
with &v.buf[i] as e { e.count += 1 }
```

Trace: R5's part-name clause rejects the original: `e` has path v.buf[i], push writes v.buf, overlap with a write, rejected — without ever asking whether the realloc happens. R7 gives the rewrite: the part-name is gone at the end of the block, re-deriving costs one address computation, and the bounds fact `i < v.len` needed to re-derive is preserved across the push because R6 lets push's `ensures v.len == old(v.len) + 1` re-establish it (i < old(len) and len == old(len)+1 gives i < len, a linear step). This is the claim under test, and for a Vector<T> with T stored inline it works exactly as advertised: the rejection is total, the rewrite is mechanical, and the cost is one shift-add plus a reload of v.buf.ptr that a correct C++ program must also perform after push_back. Safe Rust additionally pays a bounds check that WF proves away.

Cost: Zero to negative versus C++, negative versus safe Rust. The claim holds for inline T. It is only when the element is itself an owning pointer (G5) or when occupancy is data-dependent (G7) that the path-only check over-rejects or forces a layout change.
