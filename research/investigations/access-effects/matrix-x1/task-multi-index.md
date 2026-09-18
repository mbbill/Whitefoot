# Engineering task under candidate x1 — multi-index object system

Derived against the frozen rule set in `CANDIDATE-X1.md` (2026-09-18) and only that
file. Engineering-task framing and the discriminating programs are quoted from
`PROGRAMS.md` where a step touches them (this task is that file's *Shared object
registry and indexes* row, and it re-uses P3's node-pool shape and P13's
stale-handle question). Nothing listed under "Not in this candidate" or "Deferred"
is used: no `with` blocks, no `uniq`/`mut` markers, no regions or store brands, no
`take`/`put`, no quantified facts, no channels or atomics, no destructors, no traps.
Rule 16 is used, and every place that depends on it is marked.

Cost convention from the protocol (point 7): runtime performance is the only cost.
Verbosity, extra parameters, duplicated code, contract size and proof work are not
costs and are recorded separately as observations.

---

## 0. The question this task tests

Can one logical object be reached through three independent keys (an id table, a
name table, and an ordering), mutated through one of those paths without
invalidating the others, and deleted so that every path to it is retired — under a
rule set where no aggregate may hold a reference (Rule 1), no reference may be
stored or returned (Rules 1, 4, 15), and no invariant may be stated over the
elements of an array (Rule 11)?

---

## 1. The strongest program first

The protocol (point 2) asks for the case where static analysis provably cannot
help, before the convenient one. For this task that case has four properties at
once: the slot touched is **data-determined** (it comes out of a hash table, so no
branch dominates it with a useful fact), the handle is held **outside** the store by
a caller the store was compiled without (**separate compilation**), the object's
slot is **reused** after deletion (so in-bounds does not mean valid), and the
operation **writes one index while reading another**.

```text
// P-hard: a caller that holds handles across a deletion that recycles the slot,
// and then mutates through a second key while a third key is being repaired.

fn hard(s: &Store, h_a: Handle, h_b: Handle, fresh: own Name) -> u64
    writes(s.gens), writes(s.live), writes(s.names), writes(s.payloads),
    writes(s.free), writes(s.by_id), writes(s.by_name), writes(s.order),
    reads(s.ids)
    contract {
        requires len_of(s.gens) == len_of(s.live);
        requires len_of(s.gens) == len_of(s.ids);
        requires len_of(s.gens) == len_of(s.names);
        requires len_of(s.gens) == len_of(s.payloads);
        requires len_of(s.by_id.e)   > 0;
        requires len_of(s.by_name.e) > 0;
        requires len_of(s.free) < cap_of(s.free);
        ensures  len_of(s.gens) == len_of(s.live);
        ensures  len_of(s.gens) == len_of(s.ids);
        ensures  len_of(s.gens) == len_of(s.names);
        ensures  len_of(s.gens) == len_of(s.payloads);
    }
{
    remove(&s, h_a)                       // retires slot h_a.slot; the slot goes on the free list
    n = insert(&s, 900, move fresh, empty_payload())   // may land in exactly that slot
    // h_a is now a stale handle that is still in bounds and names a different object.
    match resolve(&s, h_a) {
        Some(i) => { return 1 }           // must not happen: the generation moved
        None    => { }
    }
    // Mutate through the id path while the name path is repaired in the same call.
    match resolve(&s, h_b) {
        None    => { return 0 }
        Some(j) => {
            k_old = s.ids[j]              // read before any write to s.ids
            erase_entry(&s.by_id, k_old)  // writes(s.by_id): the id path is retired
            s.ids[j] = k_old + 1          // mutate the object through the slot path
            insert_entry(&s.by_id, s.ids[j], h_b)   // the id path is re-established
            reorder(&s.order, j, s.ids[j])          // the ordering path is repaired
            return 2
        }
    }
}
```

Everything hard about the task is in those fifteen lines: a stale in-bounds handle
that must not resolve, three write paths into one store in one call, a read of
`s.ids[j]` whose value feeds a call that writes a different field, and a slot index
`j` that no dominating branch predicts.

The trace in §3 shows this program is **accepted**, and names the exact clauses that
accept it. The costs it pays are in §5; the parallel work it gives up is in §4 and
§6.

---

## 2. The full system

### 2.1 Layout

Rule 1 decides the layout before anything else: *"Every struct, enum, tuple, array,
slice-like value, Box, DynBox, and generic instantiation holds only owned values.
This is recursive and closed under wrapping"*. So no index may hold an `&Object`.
Every index holds an owned handle, and a handle is two integers.

```text
type Name    = DynBox<u8>                  // affine
struct Payload { data: DynBox<u64>, weight: u64 }   // affine by containment (Rule 8)

struct Handle { slot: u64, gen: u64 }       // Copy: two u64 (notation: "Int and u64 are Copy")

// One hash-table entry. tag: 0 = never used, 1 = live, 2 = retired (tombstone).
// Occupancy is data, as Rule 12 requires: "Structures whose occupancy is decided by
// data keep that occupancy as data."
struct Entry { tag: u8, key: u64, val: Handle }      // Copy

struct Table {
    e:    DynBox<Entry>,    // DynBox::filled(n, EMPTY): cap_of == len_of == n, always
    live: u64,              // entries with tag == 1
    used: u64,              // entries with tag != 0   (live + tombstones)
}

// One ordering entry. The sort key is denormalized into the ordering index on
// purpose; see §3.7 — it removes a dependent load and a bounds test per comparison.
struct OrderEntry { key: u64, h: Handle }            // Copy

struct Store {
    // --- per-slot object state, one DynBox per field (see §3.5 for why) ---
    gens:     DynBox<u64>,       // generation of the slot's current occupant
    live:     DynBox<u8>,        // 1 if the slot holds a live object
    ids:      DynBox<u64>,       // primary key
    names:    DynBox<Name>,      // affine key material
    payloads: DynBox<Payload>,   // affine user data
    // --- slot recycling (Rule 16) ---
    free:     DynBox<u64>,       // stack of retired slot indices
    // --- the three access paths ---
    by_id:    Table,             // id   -> Handle
    by_name:  Table,             // hash -> Handle, confirmed against names[slot]
    order:    DynBox<OrderEntry>,// sorted by key; the ordering path
}
```

The five per-slot arrays are kept the same length. Candidate x1 has no struct
invariants, so that equality is carried in `requires`/`ensures` on every function
that touches the store — four extra equalities per contract. Rule 11 admits them:
*"Facts are the existing WF forms: affine comparisons over measures and integer
values"*, and `len_of` is one of the measures named in the notation section. This is
proof text, not runtime work, and by the protocol's point 7 it is not a cost.

### 2.2 Table operations

```text
fn hash(k: u64) -> u64                                    // pure, no reference parameters, no row

fn find(t: &Table, key: u64) -> Option<u64>
    reads(t)
    contract { requires len_of(t.e) > 0;
               requires t.used < len_of(t.e);             // at least one slot is not tag 1 or 2
               ensures when Some: result < len_of(t.e); }
{
    n = len_of(t.e)
    i = hash(key) % n                                     // n > 0 from the requires: the divisor is proved nonzero
    for step in 0..n {
        invariant b: i < n;                               // maintained by i = (i + 1) % n
        e = t.e[i]                                        // Rule 7: i < n == len_of(t.e)
        if e.tag == 0 { return None }
        if e.tag == 1 { if e.key == key { return Some(i) } }
        i = (i + 1) % n
    }
    return None
}

fn insert_entry(t: &Table, key: u64, v: Handle) -> Bool
    writes(t)
    contract { requires len_of(t.e) > 0;
               requires t.used + 1 < len_of(t.e); }
{
    n = len_of(t.e)
    i = hash(key) % n
    for step in 0..n {
        invariant b: i < n;
        e = t.e[i]
        if e.tag == 1 {
            if e.key == key { t.e[i] = Entry { tag: 1, key: key, val: v }; return false }   // replace
        } else {
            if e.tag == 0 { t.used = t.used + 1 }
            t.e[i] = Entry { tag: 1, key: key, val: v }
            t.live = t.live + 1
            return true
        }
        i = (i + 1) % n
    }
    return false                                          // unreachable under the requires
}

fn erase_entry(t: &Table, key: u64) -> Bool
    writes(t)
    contract { requires len_of(t.e) > 0;
               requires t.used < len_of(t.e); }
{
    match find_slot(t, key) {                             // the probe of `find`, inlined or shared
        None    => return false
        Some(i) => { t.e[i].tag = 2; t.live = t.live - 1; return true }   // tombstone, not tag 0
    }
}
```

`Entry` is Copy, so `t.e[i] = ...` is a plain store and none of Rule 6's
affine/linear assignment machinery is engaged. `t.e[i].tag = 2` writes a field below
an index; Rule 2 admits the path (*"continues through fields, `*` (Box content),
`[i]` (index, Rule 7)"*) and the old `u8` is Copy.

### 2.3 Resolving a handle

This is the only place where Rule 16 is load-bearing, and it is the function Rule 4
forces into this shape.

```text
fn resolve(s: &Store, h: Handle) -> Option<u64>
    reads(s.gens), reads(s.live)
    contract { requires len_of(s.gens) == len_of(s.live);
               ensures when Some: result < len_of(s.gens);
               ensures when Some: result < len_of(s.live); }
{
    if h.slot < len_of(s.gens) {                          // Rule 7: the test establishes the fact
        if s.gens[h.slot] == h.gen {
            if s.live[h.slot] == 1 { return Some(h.slot) }
        }
    }
    return None
}
```

Rule 4 forbids the C++ signature `Object* get(Store&, Handle)`: *"It cannot be
assigned into any aggregate (Rule 1), cannot be returned, and cannot be captured by
a function value that is stored or returned. A function that 'finds' something
returns an index or other owned data; the caller forms the reference."* The
`ensures when Some:` clauses are exactly Rule 4's own example shape, so the caller
re-forms `&s.names[i]` with no second bounds test.

### 2.4 Lookup through each of the three paths

```text
fn get_by_id(s: &Store, id: u64) -> Option<u64>
    reads(s.by_id), reads(s.gens), reads(s.live)
    contract { requires len_of(s.by_id.e) > 0;
               requires s.by_id.used < len_of(s.by_id.e);
               requires len_of(s.gens) == len_of(s.live);
               ensures when Some: result < len_of(s.gens); }
{
    match find(&s.by_id, id) {
        None    => return None
        Some(i) => return resolve(&s, s.by_id.e[i].val)   // i < len_of(s.by_id.e) from find's ensures
    }
}

fn get_by_name(s: &Store, q: &Name) -> Option<u64>
    reads(s.by_name), reads(s.names), reads(s.gens), reads(s.live), reads(q)
    contract { requires len_of(s.by_name.e) > 0;
               requires s.by_name.used < len_of(s.by_name.e);
               requires len_of(s.gens) == len_of(s.names);
               requires len_of(s.gens) == len_of(s.live);
               ensures when Some: result < len_of(s.gens); }
{
    n = len_of(s.by_name.e)
    i = hash_name(q) % n
    for step in 0..n {
        invariant b: i < n;
        e = s.by_name.e[i]
        if e.tag == 0 { return None }
        if e.tag == 1 {
            match resolve(&s, e.val) {                    // confirms the handle before touching names
                None    => { }                            // a tombstoned-but-live-tag entry cannot occur; harmless
                Some(j) => { if name_eq(&s.names[j], q) { return Some(j) } }
            }
        }
        i = (i + 1) % n
    }
    return None
}

// The ordering path: a binary search over the denormalized key, touching only s.order.
fn order_find(o: &DynBox<OrderEntry>, key: u64) -> Option<u64>
    reads(o)
    contract { ensures when Some: result < len_of(o); }
{
    lo = 0
    hi = len_of(o)
    while lo < hi {
        invariant a: lo <= hi;
        invariant b: hi <= len_of(o);
        mid = lo + (hi - lo) / 2                          // hi > lo, so mid < hi <= len_of(o)
        if o[mid].key < key { lo = mid + 1 } else { hi = mid }
    }
    if lo < len_of(o) { if o[lo].key == key { return Some(lo) } }
    return None
}
```

### 2.5 Insert

```text
enum Inserted { Added(Handle), Duplicate, Oom }

fn insert(s: &Store, id: u64, name: own Name, p: own Payload) -> Inserted
    writes(s.gens), writes(s.live), writes(s.ids), writes(s.names), writes(s.payloads),
    writes(s.free), writes(s.by_id), writes(s.by_name), writes(s.order)
    contract {
        requires len_of(s.gens) == len_of(s.live);
        requires len_of(s.gens) == len_of(s.ids);
        requires len_of(s.gens) == len_of(s.names);
        requires len_of(s.gens) == len_of(s.payloads);
        requires len_of(s.by_id.e)   > 0;  requires s.by_id.used   + 1 < len_of(s.by_id.e);
        requires len_of(s.by_name.e) > 0;  requires s.by_name.used + 1 < len_of(s.by_name.e);
        ensures  len_of(s.gens) == len_of(s.live);
        ensures  len_of(s.gens) == len_of(s.ids);
        ensures  len_of(s.gens) == len_of(s.names);
        ensures  len_of(s.gens) == len_of(s.payloads);
        ensures  len_of(s.gens) >= len_of(deref(entry(s.gens)));
        ensures when Added: result.slot < len_of(s.gens);
    }
{
    match find(&s.by_id, id) { Some(i) => { return Duplicate } None => { } }
    // `name` and `p` are affine and unconsumed on this arm; Rule 8 releases them at
    // scope exit. Nothing leaks and no destructor runs.

    reuse = false
    i     = 0
    if len_of(s.free) > 0 {                               // Rule 6 pop: requires len_of > 0
        c = pop(&s.free)
        if c < len_of(s.gens) { i = c; reuse = true }      // §3.4: this test cannot be discharged statically
    }
    if reuse == false {
        i = len_of(s.gens)
        if append_slot(&s) == false { return Oom }         // grows all five arrays together
        // append_slot's ensures gives len_of(s.gens) == len_of(deref(entry(s.gens))) + 1,
        // hence i < len_of(s.gens).
    }

    s.ids[i]      = id
    s.names[i]    = move name                             // Rule 6: the old affine Name is released
    s.payloads[i] = move p                                // Rule 6: the old affine Payload is released
    s.live[i]     = 1
    h = Handle { slot: i, gen: s.gens[i] }

    insert_entry(&s.by_id,   id,               h)
    insert_entry(&s.by_name, hash_name(&s.names[i]), h)
    if order_insert(&s.order, OrderEntry { key: id, h: h }) == false { return Oom }
    return Added(h)
}
```

`append_slot` is the Rule 6 `Vector` growth pattern applied five times:

```text
fn append_slot(s: &Store) -> Bool
    writes(s.gens), writes(s.live), writes(s.ids), writes(s.names), writes(s.payloads)
    contract { requires ...the four length equalities...;
               ensures  ...the four length equalities...;
               ensures when true: len_of(s.gens) == len_of(deref(entry(s.gens))) + 1; }
```

Each array grows independently: *"`Vector<T>` is library code: a DynBox plus a
`push` that, when `len_of == cap_of`, allocates a larger DynBox, moves `[0, len_of)`
across, and replaces the old one."* (Rule 6). Five arrays means five allocations at
a growth event instead of C++'s one; see §5.

### 2.6 Mutate through one path, keep the others consistent

Two representative mutations. `rename` changes the name key; `rekey` changes the id
key, which also moves the object in the ordering.

```text
fn rename(s: &Store, h: Handle, new: own Name) -> Bool
    writes(s.names), writes(s.by_name), reads(s.gens), reads(s.live)
    contract { requires len_of(s.gens) == len_of(s.live);
               requires len_of(s.gens) == len_of(s.names);
               requires len_of(s.by_name.e) > 0;
               requires s.by_name.used + 1 < len_of(s.by_name.e);
               ensures  len_of(s.gens) == len_of(s.names); }
{
    match resolve(&s, h) {
        None    => return false                           // `new` released at scope exit (Rule 8)
        Some(i) => {
            old = hash_name(&s.names[i])                  // read of s.names[i], before any write
            erase_entry(&s.by_name, old)                  // writes(s.by_name) only
            s.names[i] = move new                         // writes(s.names[i]); old Name released
            insert_entry(&s.by_name, hash_name(&s.names[i]), h)
            return true
        }
    }
}

fn rekey(s: &Store, h: Handle, new_id: u64) -> Bool
    writes(s.ids), writes(s.by_id), writes(s.order), reads(s.gens), reads(s.live)
    contract { requires len_of(s.gens) == len_of(s.live);
               requires len_of(s.gens) == len_of(s.ids);
               requires len_of(s.by_id.e) > 0;
               requires s.by_id.used + 1 < len_of(s.by_id.e);
               ensures  len_of(s.gens) == len_of(s.ids); }
{
    match resolve(&s, h) {
        None    => return false
        Some(i) => {
            old = s.ids[i]
            if old == new_id { return true }
            erase_entry(&s.by_id, old)                    // id path retired
            s.ids[i] = new_id                             // the object mutates
            insert_entry(&s.by_id, new_id, h)             // id path re-established
            reorder(&s.order, old, new_id, h)             // ordering path repaired
            return true
        }
    }
}

fn reorder(o: &DynBox<OrderEntry>, old_key: u64, new_key: u64, h: Handle) -> Bool
    writes(o)
{
    match order_find(o, old_key) { Some(k) => { order_erase_at(o, k) } None => { } }
    return order_insert(o, OrderEntry { key: new_key, h: h })
}
```

`order_insert` and `order_erase_at` shift the array. §3.6 shows why the shift must
be written as an element loop rather than as a range-to-range helper call.

```text
fn order_erase_at(o: &DynBox<OrderEntry>, k: u64)
    writes(o)
    contract { requires k < len_of(o);
               ensures len_of(o) == len_of(deref(entry(o))) - 1; }
{
    n = len_of(o)
    j = k
    while j + 1 < n {
        invariant a: j < n;
        invariant b: n == len_of(o);
        o[j] = o[j + 1]                                   // a statement, not a call: see §3.6
        j = j + 1
    }
    x = pop(&o)                                           // requires len_of(o) > 0, from k < n
}
```

### 2.7 Delete and retire every path

```text
fn remove(s: &Store, h: Handle) -> Bool
    writes(s.gens), writes(s.live), writes(s.names), writes(s.payloads),
    writes(s.free), writes(s.by_id), writes(s.by_name), writes(s.order),
    reads(s.ids)
    contract { requires ...the four length equalities...;
               requires len_of(s.free) < cap_of(s.free);
               requires len_of(s.by_id.e)   > 0;  requires s.by_id.used   < len_of(s.by_id.e);
               requires len_of(s.by_name.e) > 0;  requires s.by_name.used < len_of(s.by_name.e);
               ensures  ...the four length equalities...; }
{
    match resolve(&s, h) {
        None    => return false
        Some(i) => {
            // (1) retire the id path
            erase_entry(&s.by_id, s.ids[i])
            // (2) retire the name path
            erase_entry(&s.by_name, hash_name(&s.names[i]))
            // (3) retire the ordering path
            match order_find(&s.order, s.ids[i]) {
                Some(k) => { order_erase_at(&s.order, k) }
                None    => { }
            }
            // (4) retire every handle that names this slot, including handles the
            //     store has never seen (Rule 16's generation-as-data mechanism)
            s.live[i] = 0
            s.gens[i] = s.gens[i] +wrap 1
            // (5) the object's heap blocks: see §3.8. They are retained until the
            //     slot is reused, where `s.names[i] = move name` releases them.
            push_nogrow(&s.free, i)
            return true
        }
    }
}
```

After `remove`, all three tables have been repaired and any `Handle { slot: i, gen:
g }` held anywhere in the program — in a local, inside another struct, in a
different compilation unit — fails `resolve` on its generation compare. That is the
"retire every path to it" requirement, discharged.

---

## 3. Rule-by-rule trace at the interesting points

### 3.1 No index may hold an object reference — Rule 1, and why it is not a cost here

Rule 1: *"Every struct, enum, tuple, array, slice-like value, Box, DynBox, and
generic instantiation holds only owned values. This is recursive and closed under
wrapping: there is no type parameter, wrapper, or variant payload through which a
reference can be stored."* The C++ baseline `unordered_map<u64, Object*>` is
therefore rejected outright, as is `DynBox<&Object>` and `Option<&Object>` (Rule 1's
own third rejected line).

The rewrite is `Entry { tag, key, val: Handle }` in one `DynBox<Entry>`. Note what
this does to the memory hierarchy: the C++ map node holds key and pointer adjacently
(one cache line), then dereferences the pointer (a second line). The x1 probe reads
one `Entry` (tag, key and handle in one line), then reads the field array at
`handle.slot` (a second line). **Same number of dependent cache lines.** Rule 1
costs nothing on the lookup path as long as the tag is carried inside the entry
rather than in the separate `tags: DynBox<u8>` array of Rule 12's sketch — Rule 12
permits either, since its requirement is only that *"Structures whose occupancy is
decided by data keep that occupancy as data."*

### 3.2 The stale handle — Rule 16, and the one thing it makes a logic error

In `hard`, `h_a` is used after `remove(&s, h_a)` and after an `insert` that may have
recycled exactly that slot. Rule 16: *"A stale index that is still in bounds names
the current occupant of that slot: a logic error, not a memory error. Programs that
need to detect it keep a generation number as data."*

So the memory-safety question has no runtime component at all: `s.gens[h_a.slot]`
is a read of a live `u64` in a live block, which Rule 7 admits once
`h_a.slot < len_of(s.gens)` is tested. The *identity* question is answered by data:
`remove` incremented `s.gens[i]`, so `s.gens[h_a.slot] == h_a.gen` is false and
`resolve` returns `None`. This is the whole of the task's "delete an object and
retire every path to it" requirement, and it is one compare.

**This is the dependence on Rule 16.** Without Rule 16 the slot-reuse pattern is not
available at all and the store must either never reuse a slot (unbounded growth) or
find another admitted mechanism; candidate x1 offers none, since `swap_remove` (Rule
6) moves a different object into the hole and thereby breaks *its* handles instead.

### 3.3 Three write paths into one store in one call — Rule 9 and Rule 10 clause 1

`remove` declares eight write paths and one read path, all rooted at the reference
parameter `s`. Rule 9 admits this: *"An effect row lists `reads(path)` and
`writes(path)` where each path starts at a reference parameter and may continue
through fields, `*`, and whole-index or range positions supplied as arguments"*, and
its own example line `fn update(o: &Obj, c: Bool) writes(o.a), writes(o.b) //
member paths are allowed`.

Inside `remove`, the call `erase_entry(&s.by_id, s.ids[i])` substitutes to
`writes(s.by_id)` plus, by Rule 10 clause 2 (*"A by-value argument contributes a
consumption (`move`) or a read (copy) of its place to this comparison"*), a read of
`s.ids[i]`. Rule 10 clause 1 then compares `writes(s.by_id)` with `reads(s.ids[i])`:
different field selectors from the same root, so they do not overlap, exactly as in
Rule 10's example `two(&a.x, &a.y) // accepted: distinct fields`. **Accepted.**

The same comparison accepts `order_remove`-style calls that take three fields of one
store at once, and it is what makes the whole task expressible in one function: a
single `writes(s)` row would have been legal too, but would have destroyed every
fact and reference in the caller (§3.4). The finer row is free.

### 3.4 A fact about one index survives maintenance of another — Rule 11 and Rule 3

This is the acceptance that makes "mutate through one path and keep the others
consistent" cheap. In `rename`:

```text
Some(i) => {                              // fact F: i < len_of(s.names)   (resolve's ensures + the length equality)
    old = hash_name(&s.names[i])
    erase_entry(&s.by_name, old)          // writes(s.by_name)
    s.names[i] = move new                 // needs F
```

Rule 11: *"A fact that mentions a path is invalidated when that path is written (by
statement or call) unless the callee's `ensures` re-establishes it."* `F` mentions
`s.names`; the call writes `s.by_name`. `s.names` is not written, so `F` survives
with **no re-test and no re-derivation**.

The same holds for a *reference* rather than a fact. If the writer had bound
`p = &s.names[i]` before the call, Rule 10 clause 3 applies: *"A live reference
outside the call whose path has a proper prefix among the call's write paths becomes
invalid after the call (Rule 3)."* The proper prefixes of `s.names[i]` are `s` and
`s.names`; the call's write path is `s.by_name`, which is neither. `p` stays valid.

Compare Rust: a method taking `&mut self` cannot be called while `&mut
self.names[i]` is live, and the standard workaround is to destructure `self` into
per-field borrows at every call site. Compare C++: nothing is checked at all. The x1
row is *finer* than Rust's `&mut self` and *checked*, unlike C++'s. On this point the
candidate is better than both baselines at zero runtime cost.

### 3.5 Why the object is stored as five arrays, not one array of structs

The task's central operation is "mutate through one path": replacing the `Name` of a
live object. The array-of-structs form of that write is

```text
slots: DynBox<Object>                     // struct Object { id: u64, name: Name, payload: Payload, ... }
s.slots[i].name = move new                // ← the old value is affine
```

Rule 6 states the release rule for exactly one shape: *"Assigning `buf[k] = x` where
the old value is affine releases the old value; where it is linear the assignment is
rejected unless written as the atomic update whose function consumes the old
value."* That sentence is about `buf[k]`. Nothing in the rule file states what
`buf[k].f = x` or `o.f = x` does when the old value at `.f` is affine: whether the
old `Name`'s block is released, leaked, or the assignment rejected. This is a **rule
gap** (§7, gap 1), and the protocol forbids me from resolving it.

The struct-of-arrays form is inside the stated sentence verbatim: `s.names[i] = move
new` is `buf[k] = x` with `buf = s.names` (a path to a DynBox, admitted by Rule 2)
and an affine old value. So the SoA rendering is derivable under the rules as
written, with no appeal to the gap, which is why §2 uses it.

The alternative that avoids both the gap and SoA is Rule 6's atomic update on the
whole element:

```text
s.slots[i] = with_name(s.slots[i], move new)     // "the old value goes into f by value,
                                                 //  f's result is committed, no program
                                                 //  point lies between"
fn with_name(o: own Object, n: own Name) -> own Object
{ return Object { id: o.id, name: move n, payload: move o.payload } }   // ← move o.payload
```

`move o.payload` is a partial move out of an owned local, and Rule 6 says *"There is
no `take` operation and no partial move out of any place"* while Rule 8's example
line `fn use_it(c: own Conn) { ...; close(move c.f) }` does exactly that. That
tension is already recorded as the 8-8 cell's gap and is not re-derived here; I note
only that it blocks this escape route for an object with **two or more** non-Copy
fields, which the task's object has (`Name` and `Payload`). With one non-Copy field
the escape route works, at the price of two `sizeof(Object)` copies per mutation
(§5).

**Consequence recorded for the owner:** under x1 as written, a multi-index object
with several non-Copy fields is naturally stored struct-of-arrays. The runtime price
is §5's "one extra cache line per additional field touched". The proof price (four
extra length equalities in every contract) is not a cost by the protocol.

### 3.6 The ordering shift cannot be a helper call — Rule 10 clause 1

The ordering index is a sorted array, so insertion and deletion shift a run of
elements. The natural library form is a range-to-range copy:

```text
fn copy_run(dst: &DynBox<OrderEntry>, src: &DynBox<OrderEntry>)  writes(dst), reads(src)
copy_run(&s.order[k+1 .. n+1], &s.order[k .. n])      // ← rejected
```

Rule 10 clause 1: *"Two effects on overlapping paths where at least one is a write
must be proved disjoint (different roots, or indices or ranges proved distinct);
otherwise the call is rejected."* The two ranges overlap by construction — that is
what a shift is — so the call is rejected. There is no exemption for a
memmove-shaped helper, and nothing in the candidate provides one as a primitive.

The rewrite is the element loop in §2.6. Rule 10 is *"the call-site rule"*; a plain
assignment `o[j] = o[j + 1]` is a statement, not a call, so no disjointness
obligation arises, and the loop is accepted with `invariant a: j < n` discharging
Rule 7's bounds requirement at both indices.

Cost: the writer hands the backend a scalar element loop where C++ hands it
`memmove`. If the backend idiom-recognizes the loop, zero; if not, the shift runs at
one element per iteration instead of a vector width. For a 16-byte `OrderEntry` and
a 32-byte vector unit that is up to a 2-4x difference on the shift, and more against
a tuned `memmove` that uses non-temporal stores for long runs. This is the one place
in the task where the rules, not the layout, cost real time, and §6 makes it the
counterexample's second half.

### 3.7 Denormalizing the sort key removes the dependent load and the bounds test

The obvious ordering index is `DynBox<Handle>` sorted by the objects' ids, whose
comparator reads `s.ids[o[mid].slot]`. That form costs, per binary-search step, one
random dependent load into `s.ids` **and** one bounds test, because
`o[mid].slot < len_of(s.ids)` is not derivable: Rule 11 says *"There are no
quantified facts over array elements ("for all i ...") and no per-slot occupancy
facts; occupancy that is determined by data is stored as data."* The invariant "every
slot stored in the ordering index is in range" is precisely a quantified fact over
elements, so it cannot be stated, and the test is per step — about 20 tests and 20
random loads for a million objects.

Storing the key in the entry (`OrderEntry { key, h }`) removes both: `order_find`
touches only `o`, whose bounds come from the loop invariants at zero runtime cost,
and the search never leaves the ordering array. This is not merely parity with C++ —
a C++ ordering index of `Object*` sorted by id chases a pointer per comparison; the
x1 form does not. On this operation the candidate's restrictions push the writer
toward the faster layout.

The residual cost is that the key is duplicated, so `rekey` must repair the ordering
array — which it must do anyway, since the object moved in the order.

### 3.8 Freeing the object's heap blocks at delete — what Rule 8 gives and what it does not

Rule 8: *"Affine values are consumed at most once; at scope exit the compiler
releases their memory recursively (Box, DynBox) and runs no user code."* **Scope
exit**, not overwrite. A slot inside a `DynBox` is not a scope, so a deleted
object's `Name` and `Payload` blocks are not released by `remove`'s step (4). Rule 6
forbids emptying the slot instead: *"No slot ever carries a tag, and no program
point can observe a slot inside the window as empty"*, and *"There is no `take`
operation and no partial move out of any place"*.

Two admitted renderings, both gap-free:

**(a) Deferred release (what §2.7 does).** The dead object's blocks stay in the slot
until `insert` reuses it, where `s.names[i] = move name` releases them under Rule
6's affine-assignment sentence. Runtime cost: **zero**. Memory cost: the aggregate
capacity of the dead objects' blocks is retained while their slots sit on the free
list. For a store that shrinks permanently, that memory is held until the store
itself goes out of scope. C++'s destructor frees at delete.

**(b) Eager release.** Overwrite each with a freshly built empty value:

```text
s.names[i]    = empty_name()?             // DynBox::new<u8>(0)?   — Rule 14: allocation returns Result
s.payloads[i] = empty_payload()?          // one more allocation
```
Runtime cost: **two allocations per delete** (Rule 14: *"Allocation returns a
`Result` and never traps"*, so `remove` also grows an `Oom` arm) against C++'s two
frees. This is strictly worse than (a) unless the retained memory matters.

Note what is *not* available: there is no `free` or `drop` operation on a place. The
only release mechanisms in the candidate are scope exit (Rule 8), the affine
assignment (Rule 6), and window shrinkage (`pop`, `swap_remove`, `truncate`, Rule
6). A pool that wants eager release must therefore pay an allocation to obtain the
value it overwrites with.

### 3.9 The free list's element cannot be proved in bounds — Rule 11 again

`insert` pops a slot index off `s.free` and must establish `i < len_of(s.gens)`
before writing `s.ids[i]` (Rule 7: *"Every index must be proved in bounds; when the
proof is unavailable the program tests the measure, which is ordinary data"*). The
fact that every index ever pushed onto `s.free` was in range is a quantified fact
over `s.free`'s elements, which Rule 11 forbids stating. So the program tests, and
Rule 7's own line covers the shape: `if i < len_of(buf) { use(&buf[i]) } // the test
establishes the fact; one compare, no trap`.

The `else` arm is written honestly rather than as dead code: §2.5 falls through to
the append path, so an out-of-range free-list entry costs a slot and nothing else.
Cost: one compare and one perfectly predicted branch per insert that reuses a slot,
against a C++ free list of raw pointers that tests nothing. Against `Vec`-backed
Rust or a safe C++ `at()`, zero.

### 3.10 Growth invalidates every reference into the store — Rule 3 and Rule 6

`append_slot` writes `s.gens`, `s.live`, `s.ids`, `s.names`, `s.payloads`. By Rule
3, *"[validity] is invalidated when any proper prefix of p's path is written, moved
out of, replaced, or freed, by a statement or by a call"*, any live `p =
&s.names[i]` dies, because `s.names` is a proper prefix. This is true even when the
push did not reallocate — `push_nogrow`'s row is `writes(buf)` for the whole DynBox,
so the header write invalidates references into the payload.

For this task the consequence is mild, because Rule 4 already pushed the code into
an index-passing style: `resolve` returns an index, the caller forms `&s.names[i]`
at the point of use, and the *fact* `i < len_of(s.names)` survives the growth via
`append_slot`'s `ensures len_of(s.gens) >= len_of(deref(entry(s.gens)))` combined
with the caller's own `i < n` where `n` was bound to a local before the call — the
mechanism Rule 12's first example blesses: *"after the join: len_of(buf) >= n, where
n was len_of before — facts about indices below n survive"*. So re-forming the
reference costs one load of the (possibly new) base pointer and no test.

### 3.11 The affine argument on a failing arm — Rule 8, no destructor, no leak

`rename`'s `None` arm and `insert`'s `Duplicate` arm both drop an `own Name` and an
`own Payload` that were never stored. Rule 8: *"Affine values are consumed at most
once; at scope exit the compiler releases their memory recursively (Box, DynBox) and
runs no user code."* No drop flag, no destructor, no leak, and no branch: the arm is
statically known to hold an unconsumed affine value. C++ parity.

Had `Payload` been declared linear (Rule 8's `linear type File` shape), those arms
would be rejected until the writer consumed it explicitly — which is the desired
behavior for a registry of external resources and costs nothing at runtime.

### 3.12 The generation bump

`s.gens[i] = s.gens[i] +wrap 1` is a `u64` increment. Candidate x1 says nothing
about integer overflow; its only statement about these types is the notation line
*"`Int` and `u64` are Copy."* The totality of the increment is therefore governed by
the existing specification (P15's `+wrap` is the wrapping form; P17 is the
partial-operation witness), not by this candidate. Under the wrapping form the cost
is zero and a generation collides only after 2^64 reuses of one slot — a logic
error under Rule 16's own framing, not a memory error. Recorded as gap 3 (§7)
because a reader of the rule file alone cannot settle it.

---

## 4. Parallel opportunities: admitted and refused

Rule 13: *"`par { A; B }` is accepted when A's write paths are disjoint from B's
read and write paths and vice versa, using the same path-overlap and
index/range-disjointness judgment as Rule 10. Read/read overlap is allowed."*
`PROGRAMS.md` notes that this is overlap of otherwise sequential computation, not a
writer-visible thread facility, so "refused" below means the overlap is not admitted,
not that a thread is blocked.

### Admitted

1. **Rebuilding the two key indexes in parallel.**
   ```text
   par { rebuild(&s.by_id, &s.ids, &s.gens, &s.live);
         rebuild(&s.by_name, &s.names, &s.gens, &s.live) }
   ```
   `writes(s.by_id)` against `writes(s.by_name)`: distinct fields, disjoint by the
   Rule 10 judgment. Both arms read `s.ids`/`s.names`/`s.gens`/`s.live`: *"Read/read
   overlap is allowed."* Both allocate their new tables: Rule 14, *"Allocation and
   release carry no effect entry and never make two parallel arms conflict."*
   **Accepted, and statically proved** — where the C++ equivalent is correct only by
   the programmer's say-so.

2. **A per-object kernel over the payloads, by ranges.**
   ```text
   par { work(&s.payloads[0..mid], &s.ids[0..mid]);
         work(&s.payloads[mid..n], &s.ids[mid..n]) }
   ```
   Rule 13's own example `par { kernel(&v[0..mid], &out[0..mid]); kernel(&v[mid..n],
   &out[mid..n]) } // disjoint ranges: accepted`. Dead slots are skipped by reading
   `s.live`. **Accepted at zero cost.**

3. **Parallel read-only traversal of all three indexes at once.** All arms are
   `reads`. **Accepted.**

4. **A gather driven by handles, writing to a private output.**
   ```text
   par { gather(&s.order[0..mid], &s.ids, &s.names, &out[0..mid]);
         gather(&s.order[mid..k], &s.ids, &s.names, &out[mid..k]) }
   ```
   Both arms read the whole store and write disjoint ranges of a different root.
   **Accepted** — this is the important one, because it means "follow handles from
   any index into any object" parallelizes as long as the parallel work only
   *reads* the store. C++ parity, with a proof.

5. **Two mutations through two handles, after one compare.**
   ```text
   if ha.slot != hb.slot { par { bump(&s.ids[ha.slot]); bump(&s.ids[hb.slot]) } }
   else { bump2(&s.ids[ha.slot]) }
   ```
   Rule 10's `two(&v[i], &v[j]) // accepted only with the fact i != j`, and Rule 11's
   *"refinement facts from a dominating branch"* supplies `i != j`. Cost: one
   compare. C++ needs the same compare to be correct under aliasing. **Accepted at
   zero cost.**

6. **Sharded parallel index maintenance.** If `by_id` is `DynBox<Table>` with `k`
   shards, `par { insert_entry(&s.by_id[x], ...); insert_entry(&s.by_id[y], ...) }`
   is accepted once `x != y` is tested, by the same clause as (5). The shard index
   is data (a hash), so the test is runtime; the fallback arm is sequential.

### Refused

7. **Two mutations through the same index path.**
   ```text
   par { rename(&s, ha, n1); rename(&s, hb, n2) }      // both write s.by_name: refused
   ```
   Rule 13, since `writes(s.by_name)` overlaps `writes(s.by_name)` — the same
   rejection shape as its example `par { push(&v, 1); stats(&v) } // rejected`.
   Rewrite: shard the table (6). There are no atomics (listed under "Not in this
   candidate"), so a lock-free shared table is not expressible; the "Known costs"
   section already records *"Lock-free rings are not expressible; batched fork-join
   with two buffers is the available form."*

8. **Parallel scatter into the object arrays at data-determined slots.** Two arms
   each writing `s.names[d1]` and `s.names[d2]` for data-determined `d1`, `d2`
   cannot be admitted, because pairwise `di != dj` for a whole batch is not one
   dominating-branch fact and Rule 11 has no quantified form to state it. Rewrite
   and its price: §6.

9. **Concurrent insert/delete from two independent agents.** Out of scope for the
   candidate: `par` is not a thread facility and channels and atomics are listed
   under "Not in this candidate" and "Deferred to a future concurrency and layout
   round".

---

## 5. Cost table against an idiomatic C++ or Rust implementation

Baselines: **C++-ptr** = `boost::multi_index`-style, objects in a node pool,
`unordered_map<u64, Object*>`, an intrusive ordering list, raw pointers everywhere.
**Rust-slot** = `slotmap` plus `HashMap<u64, Key>` plus a sorted `Vec`, i.e. the safe
form of the same design. Per-operation, extra work only.

| Operation | C++-ptr | Rust-slot | x1 (this rendering) | Extra vs C++-ptr | Extra vs Rust-slot |
|---|---|---|---|---|---|
| lookup by id | probe node (1 line), deref ptr (1 line) | probe, then slotmap get: bounds + version check | probe `by_id.e` (1 line), `resolve`: bounds compare + gen compare, then field array (1 line) | +2 compares, +2 predicted branches, +1 L1 load for `len_of` from the block header; **0 extra dependent cache lines** | 0 |
| lookup by name | probe, deref, string compare | same + version check | probe, `resolve`, `name_eq(&s.names[j], q)` | +2 compares, +1 branch | 0 |
| ordered iteration (k items) | list hop: 1 dependent load per hop, payload on the same node | `Vec` scan + slotmap get per item | sequential scan of `s.order` (prefetchable) + 1 dependent load per item | −1 serial dependency per hop (the scan prefetches; the list hop does not); +1 compare per item | 0 |
| ordered search | O(log n) pointer-chasing comparisons | same | O(log n) comparisons inside `s.order` only (§3.7) | −1 dependent random load per comparison | −1 dependent load per comparison |
| ordered insert / erase (shift of m) | `memmove` (vectorized) or O(1) list splice | `Vec::insert` (`memmove`) | scalar element loop (§3.6, Rule 10 clause 1) | **up to 2-4x on the shift** if the backend does not idiom-recognize the loop; O(m) vs O(1) against the list form | same 2-4x risk |
| mutate one non-Copy field | in-place store + free old | in-place store + free old | `s.names[i] = move new` (SoA) | 0 | 0 |
| mutate two fields of one object | 1 cache line | 1 cache line | 2 arrays: 2 cache lines | **+1 cache line per additional field touched** | +1 line |
| mutate one non-Copy field, AoS variant | in-place | in-place | atomic update: 2 × `sizeof(Object)` copies, and blocked by the §3.5 gap for ≥2 non-Copy fields | +2 × `sizeof(Object)` byte copies | same |
| insert, slot reused | pop free ptr | pop free key + checks | pop index + 1 bounds compare (§3.9) | +1 compare, +1 branch | 0 |
| insert, growth event | 1 `realloc` | 1 `realloc` per container | **5 allocations + 5 block moves** | +4 allocations per growth; total bytes moved is equal or lower (no struct padding) | +4 allocations |
| delete, deferred release | 3 erases + destructor frees 2 blocks | same | 3 erases + `live` store + gen bump + free push; **the 2 blocks are retained until slot reuse** | −2 frees at delete; **retained memory ≤ Σ capacities of dead objects' blocks** | −2 frees, same retention |
| delete, eager release | 3 erases + 2 frees | same | 3 erases + **2 allocations** (§3.8b) + 2 frees | +2 allocations, +1 `Oom` arm | +2 allocations |
| detect a stale handle | undefined behavior (or `weak_ptr`: 2 atomic RMWs) | version compare | gen compare | +1 load +1 compare vs raw ptr; **−2 atomic RMWs** vs `weak_ptr` | 0 |
| parallel index rebuild | correct by convention, unproved | `rayon::join` on split borrows | `par` accepted by Rule 13 §4(1) | 0 | 0 |
| parallel batch delete of b objects | scatter under one index lock | same | sort the batch by slot + partition into contiguous ranges + 1 compare per element; index removals stay per-shard | **+O(b log b) sort, +b compares, index phase serialized per shard** | same |

Two whole-system observations the per-operation table hides:

- **The lookup path is not slower.** Rule 1's ban on stored references is fully paid
  for by carrying the key inside the table entry. The extra cost of a lookup is two
  integer compares against two well-predicted branches, which sit in the shadow of
  the two dependent cache misses that dominate the operation on any realistic store.
- **The layout tax is the real one.** Struct-of-arrays is forced (or at least
  strongly preferred, pending gap 1) by the affine-assignment sentence, and an
  operation touching f fields of one object touches f cache lines instead of one.
  For a registry whose hot operation reads one field, that is a win; for one whose
  hot operation reads four, it is a 4x increase in touched lines on a random access.

---

## 6. The strongest counterexample I can construct

**Parallel batch retirement with a shift-heavy ordering index.** This is the
operation where x1 loses real time and the loss cannot be rewritten away.

```text
// C++: a batch of b handles to delete, out of n objects, with an intrusive ordering list.
//   parallel_for(batch) { o = *h; o->dead = true; free_payload(o); }     // distinct objects: safe
//   for (h : batch) { id_index.erase(...); name_index.erase(...); order.unlink(o); }  // O(1) unlink
```

Under x1:

```text
fn retire_batch(s: &Store, batch: &DynBox<Handle>) -> u64
    writes(s.gens), writes(s.live), writes(s.names), writes(s.payloads),
    writes(s.free), writes(s.by_id), writes(s.by_name), writes(s.order), reads(batch)
```

Three separate refusals stack up:

1. **The retirement phase cannot be parallel as written.** Two arms writing
   `s.live[d1]` and `s.live[d2]` for data-determined `d1`, `d2` need the pairwise
   distinctness of a whole batch. Rule 11: *"There are no quantified facts over array
   elements ("for all i ...")"*, so "the batch contains no repeated slot" is not
   statable, and Rule 13's index-disjointness judgment has nothing to consume. Best
   rewrite: sort the batch by slot, split it at a slot boundary, and hand each arm a
   **range reference** into the object arrays plus its sub-batch —
   `par { retire(&s.live[0..mid], &s.names[0..mid], &batch[0..p]);
          retire(&s.live[mid..n], &s.names[mid..n], &batch[p..b]) }` — which Rule 13
   admits by its adjacent-range example. Price: an O(b log b) sort C++ does not
   need, plus one bounds compare per element inside each arm to relate the stored
   slot to the arm's range offset (Rule 7; the relation `slot - lo < len_of(part)` is
   again a per-element data fact that Rule 11 cannot quantify).

2. **The index phase stays serial.** Every arm would write `s.by_id` and
   `s.by_name`, which Rule 13 refuses on the same-path overlap. Sharding recovers
   parallelism only between shards, and the shard of each key is data, so the
   batch must be bucketed by shard first — another pass over the batch.

3. **The ordering phase is asymptotically worse.** C++ unlinks b nodes from an
   intrusive list in O(b). The x1 ordering index is an array, and §3.6 showed the
   shift must be a scalar element loop, so b removals cost O(b·n) scalar element
   copies in the naive form. The available rewrite is a single compaction pass —
   mark the b entries, then one left-to-right filter over `s.order` — which is O(n)
   scalar copies, once. The alternative rewrite is an index-linked ordering
   (`next: u64, prev: u64` stored per slot), which Rule 1 permits because indices are
   not references and which restores O(1) unlink, but then every hop of an ordered
   traversal pays the §3.9 bounds compare, because "every stored `next` is in range"
   is exactly the forbidden quantified fact. So the writer chooses between O(n)
   compaction per batch and one compare per traversal hop; C++ pays neither.

**Force of the counterexample:** moderate, not fatal. The measured loss is an
O(b log b) sort, one pass of scalar compaction, and either a compare per traversal
hop or O(n) per batch — against a C++ design that is itself undefined behavior if any
handle in the batch is stale, which is precisely the failure mode this task exists to
prevent. Against **Rust-slot**, which must do the same checks, the counterexample has
no force at all except the §3.6 scalar shift.

**A second counterexample worth recording, weaker but sharper.** A registry whose hot
loop reads four fields of one object found by id — `for each q in queries { i =
get_by_id(s, q); f(s.ids[i], s.names[i], s.payloads[i].weight, s.gens[i]) }` —
touches four cache lines per query under the forced SoA layout where C++-ptr touches
one. At a million objects, that is three extra random misses per query, roughly a 3x
increase in memory stalls on the hot path. This one is a real, unavoidable cost under
the SoA rendering, and it is unavoidable only because of gap 1: if `s.slots[i].name =
move new` is admitted with affine release, the AoS layout is available and this cost
disappears entirely.

---

## 7. Rule gaps

**Gap 1 — affine overwrite of a field, as opposed to a whole element.**

> "Assigning `buf[k] = x` where the old value is affine releases the old value; where
> it is linear the assignment is rejected unless written as the atomic update whose
> function consumes the old value." (Rule 6)

Missing: whether the same holds for a field path — `o.f = x` through a reference
parameter, or `buf[k].f = x` — when the old value at `.f` is affine. Rule 9 makes
`writes(o.f)` a legal effect (*"member paths are allowed"*) and Rule 2 makes the
path legal, but no sentence says whether the old affine value is released, leaked,
or the assignment rejected. This decides array-of-structs against struct-of-arrays
for this task, and with it the second counterexample of §6. Marked
`undecided-rule-gap`.

**Gap 2 — is a variant payload in place a path?**

> "A path starts at a local variable or a parameter and continues through fields,
> `*` (Box content), `[i]` (index, Rule 7), or `[lo..hi]` (range, Rule 7)." (Rule 2)

A variant payload projection is not in that list. Yet Rule 12 recommends
`slots: DynBox<Option<Entry>>` for non-Copy payloads, and the "Known costs" section
records *"Open-addressing hash tables with non-Copy payloads pay one null check per
hit"* — both of which presuppose that the `Some` payload of an element can be
reached in place without moving the element out. Under the narrow reading a
`DynBox<Option<Name>>` element is unreadable except through the whole-element atomic
update (two copies per read), which would make the Option-based tombstone rendering
expensive. §2 avoids the question by not using `Option` in the store, but any
implementation of Rule 12's own recommended layout meets it. Marked
`undecided-rule-gap`; it does not change this task's verdict.

**Gap 3 — the arithmetic rule is not stated in the candidate.**

> "`Int` and `u64` are Copy." (notation section — the candidate's only statement
> about these types)

Missing: whether `s.gens[i] + 1` requires a proof of non-overflow, and whether a
wrapping form is available. The generation bump in `remove` needs one of the two.
This is governed by the existing specification rather than by candidate x1, so it is
recorded as an absence in the rule file rather than an inconsistency.

---

## 8. Verdicts

**Rules consistent for this task: yes.** No two sentences of the rule file give
opposite answers for any program in §2. Rules 1, 2, 3, 4, 6, 7, 9, 10, 11, 12, 13,
14, 15 and 16 were each used at least once and none of them contradicted another.
Gap 1 and gap 2 are silences, not contradictions: they leave a cheaper rendering
undecided, they do not reject the task. (The candidate's one known internal tension,
between Rule 8's `close(move c.f)` example and Rule 6/12's no-partial-move sentences,
is reached only by the AoS escape route in §3.5 and is recorded in the 8-8 cell; it
is not re-derived here.)

**Task achievable: yes, with cost.** All four requirements of the engineering task
are discharged by §2: one object is reachable through an id table, a name table and
an ordering; `rename` and `rekey` mutate through one path and repair the others in
the same call, with the other paths' facts and references surviving by Rule 10
clause 3; `remove` retires all three stored paths eagerly and every handle held
anywhere in the program lazily, by the Rule 16 generation compare. The costs are
enumerated in §5; against the safe Rust baseline they are near zero, and against the
raw-pointer C++ baseline they are two integer compares per lookup, one extra cache
line per additional object field touched, four extra allocations per growth event,
the §3.6 scalar shift, and the §6 batch-parallel losses.

**Parallelism: preserved for this task's real opportunities.** The two index
rebuilds run in parallel with a static proof (§4.1), ranged per-object kernels and
handle-following read-only gathers are accepted at zero cost (§4.2, §4.4), and
two-handle mutation needs one compare (§4.5). The refusals are parallel writes
through one index path (§4.7, recoverable by sharding plus one compare) and parallel
scatter into the object arrays at data-determined slots (§4.8, recoverable by a sort
plus ranges, at the price named in §6). No opportunity is lost outright; two are
recovered at a stated price.

**Dependence on Rule 16: yes, essential.** The entire "delete an object and retire
every path to it" requirement rests on *"A stale index that is still in bounds names
the current occupant of that slot: a logic error, not a memory error. Programs that
need to detect it keep a generation number as data."* Without Rule 16 the slot-reuse
pool is not available under this candidate, and the remaining rendering — never
reuse a slot — makes the store grow without bound across a delete-heavy workload.
`swap_remove` is not an escape: it relocates a *different* object and breaks that
object's handles instead.

**Verdict: `accepted-fine`** for the multi-index object system as rendered in §2.
Two subsidiary verdicts: `undecided-rule-gap` for the array-of-structs rendering of
"mutate one non-Copy field of an object" (gap 1), which is what forces the
struct-of-arrays layout and its per-field cache-line cost; and `rejected-real-cost`
for the overlapping-range shift helper of §3.6, whose element-loop rewrite risks a
2-4x loss on ordered insertion and deletion against a vectorized `memmove`.
