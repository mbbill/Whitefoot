# Whitefoot writer patterns

This is non-normative writer guidance. The active
[specification](../spec/kernel-spec.md) defines accepted source, conformance
evidence states what the compiler implements, and the
[constitution](constitution.md) defines the project objectives.

The examples below use the current source language. They are fragments unless
they name a maintained program. A pattern explains how to express an admitted
design; it grants no extra acceptance rule and makes no implementation-status
claim.

## P1. Put mutation in the reference parameter's effect row

A reference is a local name for a path. It has no shared or exclusive marker.
The callee's exact effect row says what it reads and writes, and the call site
proves that overlapping actual paths are safe [REF-1, EFF-1, EFF-5].

```whitefoot
struct Counter {
  value: u64;
}

fn store(counter: &Counter, next: own u64) -> result: own unit writes(counter.value) {
  set deref(counter).value = next;
  return unit;
}
```

An effect path is rooted at the bare parameter: write `writes(counter.value)`,
never `writes(deref(counter).value)`. Use the narrowest truthful path. Two
reads may overlap; a read/write or write/write pair must be proved disjoint.
For long call chains, compute owned commands in `pure` or read-only helpers and
apply them in one shallow writer. This keeps the mutation boundary visible in
signatures without an interior-mutability mechanism.

## P2. Choose the storage shape from its occupancy rule

The three storage shapes have different invariants [TYPE-9, WIN-1]:

- `Array<T, n>` is a full fixed run. Every element exists.
- `Slots<T, n>` is an inline prefix window with `len` and `cap`.
- `Ring<T, n>` is an inline wrapped window with `len`, `cap`, and `head`.
- Omitting `n` makes a runtime-capacity form. It may exist only as the
  `inner` content of a `Box`.

Use the construction and window operations instead of manufacturing a layout:

```whitefoot
let fixed = slots_new::<u8, 16>();
place_back(window: &fixed, value: 7_u8);

let growing = box_slots_new::<u8>(capacity: 64_u64);
place_back(window: &growing.inner, value: 9_u8);
```

Read measures as readonly fields: `fixed.len`, `fixed.cap`, and, for a ring,
`ring.head` [MSR-1, OP-15]. There is no `room` measure; write the needed
relation over `len` and `cap`. `Array` has only `len`.

Use `take_back`, `remove_at`, `insert_at`, `append`, `split_off`, `grow`,
`place_front`, and `take_front` for their declared transformations [OP-10]. A
source subscript always owes `index < run.len` [OP-4].

Growth policy can be ordinary source. The maintained
[grow-vector library](../lib/containers/grow-vector.wf) wraps
`Box<Slots<T>>` in `GrowVector<T, const ceiling: u64>`. The selected ceiling
supplies each concrete growth call's OP-9 bound; the policy doubles capacity
while it fits and otherwise saturates at that ceiling. A zero ceiling admits
an empty vector but no append. Reference-parameter contracts publish each
operation's length and capacity relationships. FN-9 does not publish the
constructor's measure through its aggregate result field. Its caller first
establishes that nested measure through ordinary control flow, as the
[program](../tests/programs/containers/grow-vector-program.wf) shows.

## P3. Reach heap content through `Box.inner`

`Box<T>` owns one heap cell. Its content is the ordinary field `inner`;
`deref` is only for a reference [TYPE-7, TYPE-9].

```whitefoot
nocopy struct Record {
  id: u64;
}

let record = Record(id: 7_u64);
let cell = box_new::<Record>(value: move record);
let id = cell.inner.id;
```

Moving an affine field consumes the whole owner, and the remaining owned parts
take their compiler-derived releases [WIN-3, STOR-3]. A direct
`let value = move cell.inner;` consumes the cell and frees it. A
runtime-capacity `Array<T>`, `Slots<T>`, or `Ring<T>` cannot be moved out of
its cell; leave it there, or empty a boxed window and consume it with
`free_empty` [TYPE-9, OP-14].

Use one `Box` for independently owned heap storage. Do not add a second
descriptor object around a runtime-capacity shape; the cell already owns that
shape.

## P4. Form a reference where its source path is visible

References may be local variables and call arguments. They are never stored in
aggregates and never returned [REF-3, TYPE-8]. Return an owned decision or
index, then let the caller form the reference from its own place.

```whitefoot
let index = choose_index(table: &table);
if index < table.len {
  let selected = &table[index];
  inspect(value: selected);
}
```

A write, move, or release of a proper prefix invalidates a reference. Moving
its owner also invalidates a reference to that owner itself. Form it again
after that event [REF-2]. A loop-carried reference can descend through owned
links [REF-1]. The checker summarizes unknown depth as a containing subtree;
the runtime reference remains an ordinary pointer. A selected payload
reference survives the selecting match's exit, but replacing its enum still
invalidates it. A single link-slot cursor can edit a list through an atomic
owned update. Independent cursors in the same subtree may invalidate each
other on structural writes; reform them after such an edit. The executable
[owned-link examples](../tests/programs/owned_link_cursors.wf) exercise list
walks, removal, tree descent and cursor resets.

Use a range reference for one contiguous run [REF-4]:

```whitefoot
let part = &bytes[first..end];
let count = deref(part).len;
let byte = deref(part)[offset];
```

Formation proves `first <= end <= bytes.len`. The range's own `len` is
`end - first`, and every subscript is relative to that range. A range reference
over a `Ring` is refused because a wrapped window need not be one extent.

## P5. Use indices as durable pool handles

A pool is a `Slots` window plus owned indices used as handles [OP-13]. Keep the
payloads in one storage object, return indices from searches, and form short
references only at the access site. This avoids stored references and
per-element heap cells.

An index is not a generation-safe handle by itself. If a slot can be removed
and reused, store a generation in the program's data and compare it. A stale
index that remains in bounds names the new occupant; bounds safety does not
make it the old object.

For sparse ownership, make vacancy a value such as `Option<T>` and use `swap`
to exchange a complete element without creating a hole [OP-11, WIN-3]. The
maintained examples in [option_slots.wf](../tests/programs/option_slots.wf) and
[block_pool.wf](../tests/programs/block_pool.wf) show those two shapes.

## P6. Ask for the capabilities a generic body uses

Copy and drop are structural capabilities [OWN-1, PROV-6]. Choose a bound from
the body:

- `<T>` requires no capability. The body must consume each `T` exactly once.
- `<T: drop>` accepts droppable values. The body may leave one for its derived
  release.
- `<T: copy>` accepts copyable values. The body uses them bare and may use one
  more than once.

```whitefoot
fn forward<T>(value: own T) -> result: own T pure {
  return move value;
}

fn discard<T: drop>(value: own T) -> result: own unit pure {
  return unit;
}

fn duplicate<T: copy>(value: own T) -> (left: own T, right: own T) pure {
  return value, value;
}
```

`move` on a copy value is an error. A generic `move` written under `<T>` or
`<T: drop>` remains that generic body's consuming spelling even when a concrete
argument happens to be copy.

A declaration may remove a capability. `nocopy` removes copy; `nodrop` removes
drop and copy. `nodrop` is valid on a struct or enum, including a tag-only enum:

```whitefoot
nodrop enum Ticket {
  Open();
  Closed();
}

fn spend(ticket: own Ticket) -> result: own unit pure {
  match move ticket {
    Open() => {
      return unit;
    }
    Closed() => {
      return unit;
    }
  }
}
```

A `nodrop` value must be consumed explicitly on every exit. Its modifier states
an ownership obligation; it does not attach a finalizer.

## P7. Package compile-time behavior with `interface` and `binding`

An `interface` is an ordered group of function signatures. A `binding` supplies
one concrete function for every member [FN-3, FN-4]. Both are compile-time
declarations; no dictionary, closure, or dynamic dispatch value is formed.

```whitefoot
interface Identity<T> {
  fn same(value: own T) -> result: own T pure;
}

fn same_u8(value: own u8) -> result: own u8 pure {
  return value;
}

binding ByteIdentity : Identity<u8> {
  same = same_u8;
}

fn apply<interface Identity<T>>(value: own T) -> result: own T pure {
  return Identity<T>::same(value: move value);
}
```

Every group introduction writes the marker, including a zero-argument group:
`fn read<interface Source>()`. Forwarding, a concrete generic argument, and a
qualified member call do not repeat it:

```whitefoot
fn forward<interface Identity<T>>(value: own T) -> result: own T pure {
  return apply::<Identity<T>>(value: move value);
}

let result = forward::<ByteIdentity>(value: 9_u8);
```

The member's full modes, types, effects, requirements, and postconditions are
the generic caller's boundary. A binding may refine that boundary only as
[FN-4] permits. Calls retain their ordinary syntax; `interface` and `binding`
replace the retired group-declaration keywords, not the call form. See
[grow-vector.wf](../lib/containers/grow-vector.wf) and
[grow-vector-program.wf](../tests/programs/containers/grow-vector-program.wf) for a behavior
that consumes owned elements while updating an environment.

## P8. State maintained arithmetic at its boundary

Every partial operation must have its domain proved before lowering. Put a
loop relation in the loop header so the checker proves its base and every
reachable backedge [INV-1]:

```whitefoot
invariant capacity: output.cap >= limit;

for (
  at in 0_u64..limit,
  invariant filled: output.len == at
) {
  place_back(window: &output, value: 0_u8);
}
```

The counted loop supplies `at < limit` in the body. Derived expressions still
owe their own exact integer and subscript obligations. Header invariants have
no `use` block.

Use a local invariant for a relation proved at one program point. When the
fixed automatic families cannot combine the needed premises, direct the finite
proof with explicit `use` steps [PRF-1]:

```whitefoot
invariant total_limit: first + second + third <= first_limit + second_limit + third_limit {
  use first_bound;
  use second_bound;
  use third_bound;
}
```

The target is published only after every use and the final combination have
been checked. Proofs are erased and add no runtime branch.

## P9. Put a contract on a true API requirement

Use `requires` when every valid caller must establish the condition, and
`ensures` when a callee can prove a relation every normal caller may use
[FN-8, FN-9]. State window transitions with entry and exit measures:

```whitefoot
fn pop<T, const n: u64>(window: &Slots<T, n>) -> value: own T writes(window.last), writes(window.len) contract {
  requires deref(window).len > 0_u64;
  ensures deref(window).len + 1_u64 == deref(entry(window)).len;
} {
  let value = take_back(window: window);
  return move value;
}
```

Contracts are proof-only. They do not insert a check or an alternate result.
If insufficient capacity, malformed input, or another false condition is an
expected outcome, branch before the partial operation and return an ordinary
`Result`, `Option`, or domain enum. The true edge supplies the fact:

```whitefoot
if index < input.len {
  return Some<u8>(value: input[index]);
}
return None<u8>();
```

Do not turn recoverable input failure into an invariant or a requirement merely
to make a later operation compile.

## P10. Transform owned state without a hole

Use the operation that owns the complete transition:

- `swap(first: &a, second: &b)` exchanges two places and permits the same place
  twice [OP-11].
- Window operations move their declared slots and measures [OP-10].
- `set place = value;` releases the displaced affine value. It refuses an
  overwrite of a linear value [SET-1, WIN-3].
- `set place = transform(value: move place);` is an atomic in-place update only
  under [OP-12]'s result and row conditions.

```whitefoot
set state = advance(state: move state);
```

No program point contains a hole during an admitted atomic update. A complete
binding whose value was consumed may be reinitialized by assigning that whole
binding; a projected, dereferenced, or subscripted place below a dead root may
not [SET-1, LIV-1].

At a branch or loop join, every outer binding must have the same live/dead
status on every incoming edge [LIV-1]. Move a value on every arm and publish one
replacement, or keep the move inside the arm that exits.

## P11. Consume must-use resources explicitly

A `nodrop` owner must be consumed on every exit [PROV-6]. Destructure a
non-opaque aggregate whole when its parts need different consumers:

```whitefoot
let Inputs(args: args, cwd: cwd, stdout: out, stderr: err, handles: factory, stdin: input) = move inputs;
close_directory(factory: &factory, directory: move cwd);
```

Host failures are ordinary `Result` values. Match them or use `propagate` in a
function returning the same error type [ERR-1, ERR-3]. A helper that acquires a
linear handle closes it or returns it on every path. Passing a handle by
reference does not consume it; passing it as `own` does.

The maintained [stdin_echo.wf](../tests/programs/stdin_echo.wf) shows an inline
window passed to `read_next` and `write_once`, with the invocation's owners
consumed explicitly.

## P12. Use fixed arrays for immutable tables

A named const may contain primitives, const-eligible structs, and
constant-capacity `Array` values [CONST-2]. A full array literal writes every
element:

```whitefoot
const digits: Array<u8, 4> =[48_u8, 49_u8, 50_u8, 51_u8];
```

Borrow and subscript it under the ordinary rules. Const storage is immutable;
`Slots`, `Ring`, `Box`, and runtime-capacity arrays are not const-eligible.

## P13. Keep parallelism a proved implementation permission

Write the correct source-order program first. [PAR-1] may overlap independent
adjacent statements, and [PAR-2] may map or reduce a counted loop only after
the checker has retained every required disjointness, bounds, and arithmetic
proof. Failure to derive permission keeps sequential lowering and never changes
source acceptance.

For a map, partition one origin into proved-disjoint range references and make
each helper's declared row stay within its actual range. For a reduction, keep
one associative and commutative accumulator update in the admitted operation
family. Do not add locks, scheduling calls, or runtime alias tests to seek
permission.

Maintained examples live under
[tests/programs/compute](../tests/programs/compute) and
[tests/programs/parallel](../tests/programs/parallel); their source contracts
and ordinary sequential behavior remain the authority.

## P14. Keep branchless classifier state in `Bool`

For byte classification and scanner state, keep predicates in `Bool`, combine
them with `band`, `bor`, `bxor`, and `bnot`, and update the state directly.
When a numeric contribution is needed, select it with a value-producing
`match`:

```whitefoot
let increment = match starts_word {
  True() => {
    give 1_u64;
  }
  False() => {
    give 0_u64;
  }
}
```

This states Boolean dataflow directly and leaves control flow for genuine
program alternatives. Use an exact integer operation when overflow is excluded
by proof, and a `.wrap` operation only when modular arithmetic is the intended
result [OP-2].

## Known gaps

Current unresolved language and compiler questions are recorded in
[todo.md](todo.md) and the relevant investigation directories. A missing
pattern does not authorize retired syntax or a new mechanism. Reduce the need
to a small source case, identify the specification rule that admits or refuses
it, and record measured cost only when performance selects between alternatives.
