# Evidence: boundary surveys, 2026-09-16

Three research reports for owner decisions O7, O12 and O6 of
[VERDICT-CORE.md](VERDICT-CORE.md): how languages with mutable value
semantics and projection-only references meet stored-reference needs in
practice (Hylo, Swift, data-oriented practice); which data structures in
kernels, compilers and browsers have a production-proven reference-free form;
and a token census of the specification and pattern cards by rule family
against the M8 budget. Citations marked "(unverified)" were not confirmed.
Evidence, not decisions. Remove with the verdict it supports.


# File: mvs-in-practice.md

# Mutable Value Semantics in Practice: how reference-free languages meet "stored reference" needs

Evidence survey for the Whitefoot capability-boundary decision: is a core model with
mutable value semantics, **no first-class references**, second-class projections,
pool handles and index-based access an acceptable *permanent* boundary?

Scope: the seven needs that fail under that model today.

| Id | Need |
|----|------|
| N1 | Stored cursor / iterator over a **growing** container (Whitefoot P4) |
| N2 | Two long-lived writable names for one slot (Whitefoot P7) |
| N3 | Callback holding a reference |
| N4 | `split_at_mut` across a call boundary |
| N5 | Element-yielding iterator (yields *mutable* element access) |
| N6 | Intrusive structure (doubly-linked list, tree with parent links) |
| N7 | Back-pointer (child → parent) |

Legend for the "escape?" column:

- **none** — expressible in the checked, safe subset with no dynamic check beyond
  what the model already implies.
- **dynamic** — expressible safely, but correctness rests on a *runtime* check
  (bounds precondition, exclusivity trap, generation tag) rather than a static proof.
- **unsafe** — requires the language's unsafe subset (raw pointers) or a reference type.
- **N/E** — not expressible; the pattern is refused and the program must be rewritten.

Every claim below was fetched and read unless explicitly marked **(unverified)**.

---

## 1. Hylo (formerly Val)

### 1.0 The model, in the team's own words

Hylo has **no first-class reference**. `let` / `inout` / `sink` / `set` are passing
conventions; subscripts and properties *project* rather than return; projections and
values carrying `remote` parts may not escape their local scope.

> "You can't return a reference to a part of an object. However, you can **project** it.
> This mechanism plays a big role in the elimination of lifetime annotations because
> **a projected value can never escape**."
> — kyouko-taiga (Dimi Racordon), [hylo-lang discussion #788](https://github.com/orgs/hylo-lang/discussions/788)

> "Basically, the rules for instances of types with remote parts are the same as the rules
> for bare local '2nd-class references': **they aren't allowed to escape their local scope**,
> which allows us to always reason about lifetime and access in a context where the
> necessary information is available."
> — Dave Abrahams, [hylo-lang discussion #754](https://github.com/orgs/hylo-lang/discussions/754)

The specification makes the exclusivity rule explicit:

> "If a projection `p` projects an object `o` immutably, `o` is immutable for the duration of
> `p`'s lifetime. If a projection `p` projects an object `o` **mutably, `o` is inaccessible**
> for the duration of `p`'s lifetime."
> — [specification/spec.md § Projections, item 8](https://github.com/hylo-lang/specification/blob/main/spec.md)

And Hylo *does* have an unsafe subset — `Pointer<T>` / `PointerToMutable<T>` are `Regular`
(copyable, storable) values whose `unsafe[]` subscript is guarded only by a doc comment:

```hylo
// StandardLibrary/Sources/Core/Pointer.hylo
public type Pointer<Pointee>: Regular {
  internal var base: Builtin.ptr
  /// - Requires: `self` is the address of an object of type `Pointee` and its storage
  ///   is accessed only through this projection during the projection's lifetime.
  public subscript unsafe(): Pointee {
    yield base as* (remote let Pointee)
  }
}
```
[hylo/StandardLibrary/Sources/Core/Pointer.hylo](https://github.com/hylo-lang/hylo/blob/main/StandardLibrary/Sources/Core/Pointer.hylo)

### 1.1 Hylo per need

| Need | How Hylo meets it | Escape? | Citation |
|------|-------------------|---------|----------|
| N1 stored cursor over growing container | `Collection.Position` is a plain *value* (`Int` for `Array`). The cursor is copied, not borrowed, so growing the container is legal — but validity is re-established at each access by a **runtime precondition**. Alternatively `Slice<access, Base>` carries a `remote` part and then **cannot be stored** (not `Sinkable`). | dynamic (index) / N/E (slice) | [Collection.hylo](https://github.com/hylo-lang/hylo/blob/main/StandardLibrary/Sources/Core/Collection.hylo), [discussion #742](https://github.com/orgs/hylo-lang/discussions/742) |
| N2 two writable names for one slot | Refused by design: a mutable projection makes its source *inaccessible*. You get one writable name at a time, scoped. | N/E | [spec § Projections 8–9](https://github.com/hylo-lang/specification/blob/main/spec.md) |
| N3 callback holding a reference | A closure's `let`/`inout` captures *are* remote parts, so such a closure cannot escape. Only `sink` captures (ownership transfer) make a closure escapable. | N/E (borrowing closure) / none (sink closure) | [discussion #754](https://github.com/orgs/hylo-lang/discussions/754), [language tour](https://docs.hylo-lang.org/language-tour/functions-and-methods) |
| N4 split across a call | Two `inout` arguments are admitted only when their *paths* are provably disjoint. Statically distinct constant indices pass; computed indices do not. Real splitting is done with raw pointers. | unsafe | [JOT 2022 §3](https://www.jot.fm/issues/issue_2022_02/article2.pdf), [Array.hylo `swap_at`](https://github.com/hylo-lang/hylo/blob/main/StandardLibrary/Sources/Array.hylo) |
| N5 element-yielding iterator | `Iterator.next()` yields **by value** (`Optional<Element>`), consuming. Mutable element access is via `MutableCollection`'s `inout` subscript, one element at a time, scoped. The team calls C++-style iterators an anti-pattern. | none (value iterator); N/E (mutable-reference iterator) | [Iterator.hylo](https://github.com/hylo-lang/hylo/blob/main/StandardLibrary/Sources/Core/Iterator.hylo), [discussion #754](https://github.com/orgs/hylo-lang/discussions/754) |
| N6 intrusive structure | Nodes in an internal array, links as offsets. The whole owns all nodes; a node owns neither neighbour. Measured **1 order of magnitude faster** than the reference version. | none (in Swift's safe subset) | [Racordon, *Who Owns the Contents of a Doubly-Linked List?*, Programming 2025](https://doi.org/10.4230/OASIcs.Programming.2025.25) |
| N7 back-pointer | Identity value (index / key) stored in a side table owned by the whole; access is `whole[id]`. | dynamic | same paper, §2.2 |

### 1.2 Code shapes

**N1/N5 — Hylo's Collection is index-based; Iterator yields values.**

```hylo
// StandardLibrary/Sources/Core/Collection.hylo
public trait Collection {
  type Element
  type Position: Regular                        // a *value*, not a reference
  fun start_position() -> Position
  fun end_position() -> Position
  fun position(after i: Position) -> Position
  subscript(_ i: Position): Element { let }     // projects, does not return
}

// StandardLibrary/Sources/Core/Iterator.hylo
public trait Iterator {
  type Element
  fun next() inout -> Optional<Element>         // by value; takes ownership
}
// doc comment: "An `Iterator` typically does not model a `Collection`."
```

Walking a collection therefore needs the whole collection in hand at every step:

```hylo
public fun main() {
  let s = "abc"
  var i = s.start_position()
  while i != s.end_position() { print(s[i]); &i = s.index(after: i) }
}
```
([discussion #742](https://github.com/orgs/hylo-lang/discussions/742))

**N2/N4 — the standard library's own `swap_at` uses two raw pointers.**

```hylo
// StandardLibrary/Sources/Array.hylo — MutableCollection conformance
public fun swap_at(_ i: Int, _ j: Int) inout {
  precondition((i >= 0) && (i < count()), "position is out of bounds")   // runtime trap
  precondition((j >= 0) && (j < count()), "position is out of bounds")
  if i == j { return }
  var p = pointer_to_element(at: i)
  var q = pointer_to_element(at: j)
  &p.unsafe[].exchange(with: &q.unsafe[])        // two live mutable accesses: unsafe
}
```

Two *safe* writable names for one array cannot coexist; the `inout` subscript itself is
implemented over an unsafe pointer:

```hylo
public subscript(_ position: Int): Element {
  inout {
    precondition((position >= 0) && (position < count()), "position is out of bounds")
    yield &(pointer_to_element(at: position).unsafe[])
  }
}
```

Algorithms that would use two mutable references in C++/Rust are written against
positions plus a whole-collection method instead — `rotate` in `MutableCollection.hylo`
is written entirely with `swap_at(s1, m1)` and `position(after:)`, never with two
simultaneous element references.
([MutableCollection.hylo](https://github.com/hylo-lang/hylo/blob/main/StandardLibrary/Sources/Core/MutableCollection.hylo))

**N2 — one writable name at a time (spec example).**

```hylo
fun f1() {
  var x = 42
  inout y = x // mutable projection begins here
  print(x)    // error: 'x' is projected mutably
  x += 1      // error: 'x' is projected mutably
  print(y)    // mutable projection ends afterward
}
```
([spec § Projections, item 9](https://github.com/hylo-lang/specification/blob/main/spec.md))

**N3 — capture kind decides escapability.**

```hylo
var sum = 0
[1, 2, 3].for_each(fun(_ n) { &sum += n })   // inout capture == remote part: cannot escape

var counter = fun[var i = 0]() inout -> Int { // sink capture: owns its state, CAN escape
  defer { &i += 1 }
  return i.copy()
}
```
> "Lambdas can be returned from functions **if they don't capture `let` or `inout` bindings**."
> — kyouko-taiga, [discussion #788](https://github.com/orgs/hylo-lang/discussions/788)

### 1.3 What the Hylo team says users cannot write

| Claim | Quote | Source |
|-------|-------|--------|
| No returned reference to a part | "You can't return a reference to a part of an object." | [#788](https://github.com/orgs/hylo-lang/discussions/788) |
| Rust's `split_at_mut`-style selective borrow | asked directly whether Val users could write the Rust examples: "**Indeed, they can't.**" | [#788](https://github.com/orgs/hylo-lang/discussions/788) |
| Deliberate expressiveness loss | "there's no way in Val to say that you definitely won't use parts from `cache` in the projection … Therefore, unlike in Rust, you won't be able to use it while the projection is live. **That's the kind of hit to expressiveness we're willing to take in exchange for a simpler type system.**" | [#788](https://github.com/orgs/hylo-lang/discussions/788) |
| A work-around exists, outside safety | "It is possible to work around the type checker, **but not in the safe subset of the language**." | [#788](https://github.com/orgs/hylo-lang/discussions/788) |
| Storing a projection-carrying view | "as of now, it doesn't let you 'store' a string view inside another collection" — `error: type 'StringView<let>' does not conform to 'Sinkable'` | [#742](https://github.com/orgs/hylo-lang/discussions/742) |
| Wrong-collection index is not caught | "nothing statically guarantees that we're using indices in the correct collection", with a compiling program that indexes `s2` with `s1`'s indices | [#742](https://github.com/orgs/hylo-lang/discussions/742) |
| The "context piping problem" | "we always need explicit access to the base collection … becomes quite inconvenient when [we] want to pass things around in functions." Proposed fix: **implicit parameters**, "not high on the priority list". | [#742](https://github.com/orgs/hylo-lang/discussions/742) |
| Intrusive structures are built on unsafe parts | "we always want to encapsulate them inside data structures with value semantics, e.g. in the implementation of a doubly-linked list type, whose public API does not expose references. … At that point, **we are not afraid to tell people to build a safe abstraction using unsafe parts (plain pointers).**" | Dave Abrahams, [#754](https://github.com/orgs/hylo-lang/discussions/754) |
| Rc/Arc deliberately excluded | "we're reluctant to have them in the standard library. For us, proliferating references (to shared mutable state) is an anti-pattern…" | Dave Abrahams, [#754](https://github.com/orgs/hylo-lang/discussions/754) |
| C++ iterators are an anti-pattern | "We consider those to be an anti-pattern also… **A Val iterator would simply store the whole collection as a remote part, much the way Swift iterators store a (CoW'd) copy of the collection.**" | Dave Abrahams, [#754](https://github.com/orgs/hylo-lang/discussions/754) |
| Today's practical answer for views | "You can go a long way if you're willing to **use the unsafe API to create a safe abstraction around raw pointers.**" | kyouko-taiga, [#742](https://github.com/orgs/hylo-lang/discussions/742) |

### 1.4 The MVS paper (Racordon, Shabalin, Zheng, Abrahams, Saeta — JOT 21(2), 2022)

DOI [10.5381/jot.2022.21.2.a2](http://dx.doi.org/10.5381/jot.2022.21.2.a2) ·
[PDF](https://www.jot.fm/issues/issue_2022_02/article2.pdf)

Abstract, verbatim (opening):

> "Mutable value semantics is a programming discipline that upholds the independence of values
> to support local reasoning. **In the discipline's strictest form, references become
> second-class citizens: they are only created implicitly, at function boundaries, and cannot
> be stored in variables or object fields. Hence, variables can never share mutable state.**"

§1:

> "MVS does not surface references as a first-class concept in the programming model. As such,
> they can neither be assigned to a variable nor stored in object fields, and **all values form
> disjoint topological trees rooted in the program's variables**."

§2.5 "Representing other relationships" is the paper's own answer to N6/N7 — it is the
pool/handle idiom, stated as the design:

> "any arbitrary graph can be represented as an adjacency list. For example, a vertex set might
> be represented as an array, each element of which contains an array of outgoing edge
> destination indices. This approach can be seen as **decoupling the two roles of first-class
> references: inner array elements represent relationships *without* conferring direct access
> to the related data**, which is only available through the object of which it is a part."

and it names the cost honestly:

> "Naturally, losing the ability to directly access data through references changes the way
> programs are written. For example, **traversing an arbitrary graph requires access to the whole
> graph at each step**, rather than just a single vertex and its outgoing edges. In exchange, we
> get improved expressiveness, correctness, and even performance."

§3 gives the exclusivity rule and the exact boundary for N4:

> "Unlike generalized borrows, `inout` parameters are second-class citizens: they have
> lexically-bounded lifetimes and must 'appear in person'. These restrictions ensure that aliases
> can be prevented simply by verifying that the **'path' to a value … never appears twice in
> `inout` arguments to a single function call.**"
>
> "The type system allows the same array to be indexed more than once by `inout` arguments only
> if it can conclude that **the indices cannot overlap**. For example, given an array `x`, the
> expression [`f(&x[0], &x[1])`] is well-typed, but [`f(&x[0], &x[f(0)])`] is not. In the second
> case, the type system conservatively assumes that `f(0)` could be evaluated as any value."

This is the precise statement of what MVS can do about `split_at_mut`: **constant, syntactically
distinct paths only.** A run-time-computed split point is refused.

### 1.5 Racordon, *Who Owns the Contents of a Doubly-Linked List?* (Programming 2025)

[OASIcs.Programming.2025.25](https://doi.org/10.4230/OASIcs.Programming.2025.25) ·
artifact: [kyouko-taiga/vimpl-2025-artifact](https://github.com/kyouko-taiga/vimpl-2025-artifact)

This is the strongest single piece of evidence that N6/N7 have a **reference-free, safe,
and faster** answer. Abstract, verbatim:

> "Despite their popularity, systems enforcing full ownership guarantees such as Rust leave many
> users frustrated with the inability to represent notionally self-referential data structures —
> e.g., doubly-linked lists — using first-class references. … In this paper, we take a look at the
> way value-oriented languages address this issue and **study representations of arbitrary
> graph-like data structures without references.**"

Method and result:

> "we implement directed graphs and a doubly-linked list in Swift **without using first-class
> references and without escaping the safe subset of the language**."

> "a reasonable way to emulate pointers is to use **indices into an array**. From there, one can
> implement a doubly-linked list by replacing pointers with indices. … The addresses returned by
> `insert` are indeed **stable — i.e., they are not invalidated by removes or other insertions,
> even if the internal array must be reallocated.**"

> "Who owns the contents of a doubly-linked list? **The list itself of course! Crucially, a node
> owns neither its successor nor its predecessor.** Instead, the nodes forming a list are all
> parts of that whole while links between these nodes are peer relationships that can be
> represented independently."

Measured (Apple M1, 10-iteration averages):

> "The results show that the **value-based approach significantly outperforms its reference-based
> counterpart, by one order of magnitude.** The biggest contributor of this difference is cache
> locality."

> graph algorithms (Tarjan SCC, Dijkstra): "The results show **no noticeable difference** between
> the two graph implementations."

Costs the paper itself concedes:

> insertion is "in **amortized** constant time rather than O(1), due to the cost of reallocating
> the internal array."
>
> "One may object that this approach could suffer from **the misuse of a URL, e.g., by calling
> `addLink` on a page contained in a different graph.** Note however that such a danger was
> already present in [the reference version]."

The artifact's handle type is literally Whitefoot's pool handle, without a generation tag:

```swift
// Sources/Value/List+Value.swift
public struct List<T> {
  public struct Address: Hashable {
    internal let rawValue: UInt32                 // an index, not a reference
    internal var offset: Int { Int(rawValue) }
  }
  private struct Node { var predecessor: Address; var successor: Address; var value: T }
  private var contents: [Node] = []               // the pool
  private var head = Address(.max)
  private var tail = Address(.max)

  public subscript(_ a: Address) -> T {           // access only through the whole
    get { contents[Int(a.rawValue)].value }
    set { contents[Int(a.rawValue)].value = newValue }
  }
}
```

Note the two dynamic obligations this shape leaves open: `contents[...]` is a
bounds-checked Swift `Array` subscript (a trap on failure), and `Address` carries no
generation, so a stale address silently reads a recycled node.


---

## 2. Swift

### 2.0 Headline: Swift no longer lives without first-class references

The JOT 2022 paper studies **Swift** as *the* language "based on that discipline". As of
**Swift 6.4**, Swift has added safe, first-class reference types to the standard library.
An argument of the form "Swift chose to live permanently without first-class references"
is historically true and **currently false**.

**[SE-0519 `Ref` and `MutableRef` types for safe, first-class references](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0519-ref-mutableref-types.md)**
(Joe Groff, Alejandro Alonso) — *Status: Implemented (Swift 6.4)*. Motivation, verbatim:

> "It would be useful to be able to form these sorts of references outside of the confines of a
> function call, as local variable bindings, as members of other types, as elements of generic
> containers, and so on. **Developers can use classes to box values and pass references to a
> common holder object around, but in doing so they introduce allocation, reference counting, and
> dynamic exclusivity checking overhead. `UnsafePointer` is, of course, unsafe**, and interacts
> awkwardly with Swift's high-level semantics, requiring extreme care to use properly."

The proposal also gives the single clearest argument in the whole survey for why a
*reassignable* reference variable — not just a scoped binding — is what graph work needs:

> "Since a reference binding's name refers to the target value … **the reference itself cannot be
> reassigned. In a graph of values that refer to each other through `Ref` or `MutableRef`
> references, a traversal loop can update a `Ref` variable in-place as it advances through the
> graph, but not a `borrow` binding.**"

What it took to get there: `~Copyable` (5.9) → `~Escapable` (6.2) → lifetime dependencies →
`borrow`/`mutate` accessors (6.4). The lifetime-dependency layer **still has no accepted
Evolution proposal**: every shipping proposal spells the attribute `@_lifetime` (underscored),
and [swift-evolution PR #2750](https://github.com/swiftlang/swift-evolution/pull/2750)
("Lifetime dependencies") is closed and unmerged. Swift shipped first-class references on an
unratified foundation.

### 2.1 Proposal ledger (verified against `swiftlang/swift-evolution`)

| ID | Exact title | Status |
|----|-------------|--------|
| SE-0176 | Enforce Exclusive Access to Memory | implemented, Swift 4.0 |
| SE-0377 | `borrowing` and `consuming` parameter ownership modifiers | implemented, 5.9 |
| SE-0390 | Noncopyable structs and enums | implemented, 5.9 |
| SE-0446 | Nonescapable Types | implemented, 6.2 |
| SE-0447 | Span: Safe Access to Contiguous Storage | implemented, 6.2 |
| SE-0467 | MutableSpan and MutableRawSpan: delegate mutations of contiguous memory | implemented, 6.2 |
| SE-0474 | Yielding accessors | **Accepted, not shipped** (flag `-enable-experimental-feature CoroutineAccessors`) |
| SE-0507 | Borrow and Mutate Accessors | implemented, 6.4 |
| SE-0516 | `Iterable` (file `0516-borrowing-sequence.md`) | implemented, 6.4 |
| SE-0517 | UniqueBox | implemented, 6.4 |
| SE-0519 | `Ref` and `MutableRef` types for safe, first-class references | implemented, 6.4 |
| SE-0527 | UniqueArray (file `0527-rigidarray-uniquearray.md`) | implemented, 6.4 |
| — | Lifetime dependencies (`@_lifetime`) | **no SE number; PR #2750 closed unmerged** |

No vision document titled "A Roadmap for Improving Swift Performance Predictability" was
found **(unverified)**; the accessors vision lives on a fork branch linked from SE-0474.

### 2.2 The two rules that decide everything

**SE-0176, enforcement split** (this is where the dynamic escape lives):

> "* Local variables, `inout` parameters, and struct properties can generally enforce the rule
> **statically**. …
> * Class properties and global variables will have to enforce the rule **dynamically**. …
> **Local variables will sometimes have to use dynamic enforcement when they are captured in
> closures.**
> * **Unsafe pointers will not use any active enforcement**; it is the programmer's
> responsibility to follow the rule."

and the reason classes can never be static:

> "we never try to enforce exclusivity of access on the whole object at all; we only enforce it
> for individual stored properties. … it's inherent to the nature of reference types that
> references can be copied pretty arbitrarily throughout a program."

Runtime checks are **on in Release since Swift 5**; the failure is a trap
(`Simultaneous accesses to …, but modification requires exclusive access`).
([swift.org blog](https://www.swift.org/blog/swift-5-exclusivity/))

**SE-0446, the storage rule for nonescapable values** — the viral bit:

> "**Stored struct properties and enum payloads can have nonescapable types if the surrounding
> type is itself nonescapable. Equivalently, an escapable struct or enum can only contain
> escapable values.**"
> "**Nonescapable values cannot be stored as class properties**, since classes are always
> inherently escaping."
> "they cannot be stored in global or static variables."
> "**Escaping closures cannot capture nonescapable values.**"

So a `Span`/`Ref`/`MutableRef` *can* be stored in a struct — and that struct is thereby
`~Escapable` and can never reach a class property, a global, an escaping closure, or a `Task`.

### 2.3 Swift per need

| Need | How Swift meets it | Escape? | Citation |
|------|--------------------|---------|----------|
| N1 stored cursor over growing container | `IndexingIterator` stores a **copy** of the collection (`let _elements`) plus an index; COW makes the copy cheap. A cursor over a *growing* container is therefore semantically a cursor over a *frozen snapshot*. A true reference cursor (`MutableRef` field) is possible in 6.4 but makes the cursor `~Escapable`. | none (snapshot) / viral `~Escapable` (real ref) | [Collection.swift](https://github.com/swiftlang/swift/blob/main/stdlib/public/core/Collection.swift), [SE-0519](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0519-ref-mutableref-types.md) |
| N2 two writable names for one slot | **Refused — this is the axiom, not a gap.** The Law of Exclusivity *is* the ban. Residual cases (classes, globals, captures) are permitted only under a **runtime trap**. | class + dynamic trap, or `UnsafeMutablePointer` | [SE-0176](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0176-enforce-exclusive-access-to-memory.md) |
| N3 callback holding a reference | `inout` cannot be captured by an escaping closure; the book prescribes copy-in / `defer` copy-out. The local so captured is heap-boxed under dynamic enforcement. `Ref` cannot rescue it (SE-0446 bans nonescapable captures in escaping closures). | copy (semantics change) or class + dynamic | [TSPL Declarations](https://github.com/swiftlang/swift-book/blob/main/TSPL.docc/ReferenceManual/Declarations.md), [SE-0446](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0446-non-escapable.md) |
| N4 split across a call | **Not expressible safely.** `MutableSpan.extracting` is `mutating` and freezes the parent; it re-slices, it does not split. `split(at:)` is an unimplemented Future Direction blocked on noncopyable tuples. | `withUnsafeMutableBufferPointer` | [SE-0467](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0467-MutableSpan.md) |
| N5 mutable element-yielding iterator | No such protocol exists. The idiom is to iterate **indices** and re-project each element through the `_modify` coroutine. SE-0516 added borrowing iteration — read-only, and by `Span` **batch**, not by element. Mutating iteration is an unproposed Future Direction. | none needed, but no reference is ever named | [SE-0516](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0516-borrowing-sequence.md) |
| N6 intrusive structure | A **class** (ARC + per-property dynamic exclusivity + `weak`/`unowned` to break cycles), or an **arena of indices**, or unsafe pointers. `UniqueBox`/`UniqueArray` (6.4) give noncopyable heap storage but express an ownership *tree*, not a back-edge. | class + dynamic, or index arena (unchecked indices) | [SE-0519 Motivation](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0519-ref-mutableref-types.md), [SE-0517](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0517-uniquebox.md) |
| N7 back-pointer | `weak`/`unowned` class references. `unowned(safe)` traps on use-after-free; `weak` costs an ARC side-table entry; `unowned(unsafe)` is a true unsafe escape. No safe value-type back-pointer exists. | class + ARC runtime mechanism | [TSPL ARC](https://github.com/swiftlang/swift-book/blob/main/TSPL.docc/LanguageGuide/AutomaticReferenceCounting.md) |

### 2.4 Code shapes

**N1/N5 — what the standard library actually does: snapshot + index.**

```swift
// stdlib/public/core/Collection.swift
@frozen
public struct IndexingIterator<Elements: Collection> {
  @usableFromInline internal let   _elements: Elements   // a COPY of the collection (COW-shared)
  @usableFromInline internal var   _position: Elements.Index
}

extension Collection where Iterator == IndexingIterator<Self> {
  public __consuming func makeIterator() -> IndexingIterator<Self> {
    return IndexingIterator(_elements: self)
  }
}
```

`_elements` is `let`. Mutating the original during iteration triggers COW separation and the
cursor keeps walking the old buffer. Swift makes the aliasing question *disappear* rather than
answering it — at the cost that "cursor over a growing container" is not what you get.

**Mutable element access is per-element re-projection, not a reference:**

```swift
// stdlib/public/core/Array.swift — the safe element projection is a coroutine over a raw pointer
public subscript(index: Int) -> Element {
    get { ... }
    _modify {
      _makeMutableAndUnique()
      _checkSubscript_mutating(index)
      let address = unsafe _buffer.mutableFirstElementAddress + index
      defer { _endMutation() }
      yield unsafe &address.pointee      // Swift's projection primitive, on an unsafe base
    }
}

for i in xs.indices { xs[i] *= 2 }       // the only safe mutable-iteration idiom
// `for inout e in xs { e *= 2 }` does not exist and has no proposal
```

SE-0516 states the reason the old protocol cannot be fixed:

> "an iterator's `next()` operation **returns** an `Element?`. For a sequence of noncopyable
> elements, this operation could only be implemented by **consuming** the elements of the
> iterated sequence … *borrowing* iteration … **cannot be supported by the existing `Sequence`**."

and it *considered and rejected* the Hylo/Rust shape, `next() -> Ref<Element>`, "because the
bulk iteration aspect of the proposed `Iterable` protocol is a critical part of improving
performance". Swift concluded per-element reference yielding loses to **span batching**.

**N4 — the safe path does not split; the unsafe path does.**

```swift
// SAFE: extracting() is `mutating`. The parent is frozen, not split.
var array = [1, 2, 3, 4, 5]
var span1 = array.mutableSpan
var span2 = span1.extracting(3..<5)
// neither array nor span1 can be accessed here      <- SE-0467's own comment
span2.swapAt(0, 1)
_ = consume span2                 // must END span2 before span1 is usable again
span1.swapAt(0, 1)

// UNSAFE: the only way to get two disjoint mutable views live across a call
a.withUnsafeMutableBufferPointer { buf in
  let lo = UnsafeMutableBufferPointer(rebasing: buf[..<3])
  let hi = UnsafeMutableBufferPointer(rebasing: buf[3...])
  process(lo, hi)                 // disjointness is entirely on the programmer
}
```

SE-0467, Future Directions, verbatim:

> ```swift
> public mutating func split(at index: Index) -> (part1: Self, part2: Self)
> ```
> "Unfortunately, **tuples do not support non-copyable or non-escapable values yet.** … destructuring
> the non-copyable constituent part remains a challenge. **Solving this issue for `Span` and
> `MutableSpan` is a top priority.**"

Verified absent from current `main` (`stdlib/public/core/Span/MutableSpan.swift`): nine
`mutating func extracting(...)` overloads, no `split`. Relatedly, `swapAt` exists precisely
*because* `swap(&a[i], &a[j])` violates exclusivity — the stdlib added a **fused operation**
rather than a way to name two disjoint slots. (Hylo made the identical choice; see §1.2.)

**N2 — the ban, and the only workaround.**

```swift
var v = 0
func f(a: inout Int, b: inout Int) { a += b }
f(a: &v, b: &v)            // error: cannot pass the same value to multiple in-out parameters

// SE-0176's own admitted false positive — dynamic enforcement is per-property:
let object = Paired()
swap(&object.pair.x, &object.pair.y)          // runtime exclusivity failure
modifying(&object.pair) { pair in swap(&pair.x, &pair.y) }   // documented workaround
```

> "Attempting to make dynamic enforcement aware of the fact that these accesses are modifying
> different sub-components of the property would be prohibitive." — SE-0176

**N3 — the book's own prescribed workaround is a copy.**

```swift
func multithreadedFunction(queue: DispatchQueue, x: inout Int) {
    // Make a local copy and manually copy it back.
    var localX = x
    defer { x = localX }
    queue.async { someMutatingOperation(&localX) }
    queue.sync {}
}
// return { a + 1 }  =>  error: escaping local function captures 'inout' parameter 'a'
```

> "**A closure or nested function that captures an in-out parameter must be nonescaping.**" — TSPL

**N6/N7 — classes, with an ARC mechanism for the back-edge.**

```swift
final class Node {
  var value: Int
  var next: Node?
  weak var prev: Node?        // weak/unowned required to break the ARC cycle
  init(_ v: Int) { value = v }
}
// value-type alternative: an arena; indices stand in for pointers, validity unproved
struct Arena { struct Node { var value: Int; var next: Int?; var parent: Int? }
               var nodes: [Node] = [] }
```

### 2.5 Swift's own statements about what could not be expressed

- **SE-0519 Motivation** (quoted in §2.0) — the canonical "here is the hole", from the proposal
  that fills it: classes cost allocation + refcounting + dynamic exclusivity; `UnsafePointer`
  is unsafe.
- **SE-0519 Future Directions** — even `Ref`/`MutableRef` is "an **unsatisfying endpoint** to
  the local reference bindings story"; primitive `borrow x = y` binding syntax is still wanted.
- **SE-0519 admitted limits** — no `Ref` to a `~Escapable` type, i.e. **no reference to a
  reference**: "the current model lacks the ability to track multiple lifetimes per value."
  And references derived from nontrivial accesses (get/set pairs, `yielding` coroutines,
  `didSet`) cannot outlive the caller; the proposed fix is generalized single-yield
  coroutines — *Hylo-style projections as ordinary functions* — listed as a Future Direction.
- **SE-0377** — `borrowing` is not a no-copy guarantee: "there are circumstances where the
  compiler defaults to copying when it is theoretically possible to borrow, particularly when
  working with shared mutable state such as global or static variables, escaped closure
  captures, and class stored properties."
- **SE-0517 (UniqueBox)** — on why classes are the wrong tool for unique ownership: "this
  construct is more of a shared pointer than a unique one. **Classes also come with their own
  overhead on top of the pointer allocation.**"
- **Ownership Manifesto** — why static enforcement can never cover classes: "It cannot be used
  for ordinary reference-type properties because there is no way to prove in general that a
  particular object reference is the unique reference to an object."

### 2.6 SE-0474 / SE-0507 — Swift's version of Hylo subscripts

[SE-0474 "Yielding accessors"](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0474-yielding-accessors.md)
(Accepted, not shipped) officialises the long-underscored `_read`/`_modify`:

> "This feature has been available (but not supported) since Swift 5.0 via the `_modify` and
> `_read` keywords."
> "will use a new contextual keyword `yield` to pause the coroutine and lend access of a value
> to the caller. When the caller ends its access to the lent value, the coroutine's execution
> will continue after `yield`."
> "When the value being mutated is noncopyable, the common `get`/`set` pattern often becomes
> impossible to implement: **the very first step of mutation makes a copy!**"

```swift
struct GetMutate {
  var x: String = "👋🏽 Hello"
  var property: String { yielding mutate { yield &x } }
}
```

[SE-0507 "Borrow and Mutate Accessors"](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0507-borrow-accessors.md)
(Implemented, 6.4) overtook it with a **non-coroutine** variant:

```swift
struct RigidWrapper<Element: ~Copyable>: ~Copyable {
    var _element: Element
    var element: Element { borrow { return _element }
                           mutate { return &_element } }
}
```

> "Unlike `yielding borrow` and `yielding mutate` accessors, borrowing accessors do not require
> the overhead of a coroutine … [but] can only expose values guaranteed to remain valid until
> the next mutation, whereas yielding accessors can construct temporary values."

**Design note for Whitefoot:** Swift ended up needing **two** projection forms, split exactly
on "does the projection need a finalization phase after the yield". Hylo has only the
coroutine form.

---

## 3. Control case: Rust, the language that *does* have first-class references

Worth stating plainly because it bounds what "first-class references would fix" can mean.
Rust has `&mut`, lifetimes, and a borrow checker — and still **cannot express N4 or N5 in its
own safe subset**. Both are safe *APIs* over an unsafe *kernel*.

**N4 — `split_at_mut` is `unsafe` inside:**

```rust
// core/slice/mod.rs
pub const unsafe fn split_at_mut_unchecked(&mut self, mid: usize) -> (&mut [T], &mut [T]) {
    let len = self.len();
    let ptr = self.as_mut_ptr();
    // SAFETY: Caller has to check that `0 <= mid <= self.len()`.
    //
    // `[ptr; mid]` and `[mid; len]` are not overlapping, so returning a mutable reference
    // is fine.
    unsafe {
        (from_raw_parts_mut(ptr, mid),
         from_raw_parts_mut(ptr.add(mid), unchecked_sub(len, mid)))
    }
}
```
The safe `split_at_mut` is a wrapper that `panic!`s on `mid > len` — i.e. the bound is a
**runtime trap**, and the disjointness is a hand-written SAFETY argument, not a checked proof.
([doc.rust-lang.org/src/core/slice/mod.rs.html](https://doc.rust-lang.org/src/core/slice/mod.rs.html))

**N5 — `IterMut` stores raw pointers, not a reference:**

```rust
// core/slice/iter.rs
pub struct IterMut<'a, T: 'a> {
    /// The pointer to the next element to return, or the past-the-end location
    /// if the iterator is empty.
    ptr: NonNull<T>,
    /// For non-ZSTs, the non-null pointer to the past-the-end element.
    end_or_len: *mut T,
    _marker: PhantomData<&'a mut T>,
}
```
([doc.rust-lang.org/src/core/slice/iter.rs.html](https://doc.rust-lang.org/src/core/slice/iter.rs.html))

**Consequence for the boundary decision:** N4 and N5 are met by *no surveyed system* inside a
checked safe subset. Rust, Swift and Hylo all answer them the same way — an unsafe kernel with
a hand-argued safety comment, wrapped in a safe signature. Adding first-class references to
Whitefoot would therefore **not** by itself make N4/N5 provable; it would only move the
unsafe kernel from "pool/index" to "pointer".

---

## 4. Making handles statically checkable: branded indices and GhostCell

Both Hylo's team and Racordon's 2025 paper concede the same two residual hazards of the
pool/handle idiom — *the compiler does not know a handle belongs to this container*, and
*access is bounds-checked at runtime*. There is verified prior art that removes both, and it
is directly relevant to a language that forbids runtime traps.

### 4.1 Branded ("generative") indices — bounds-check-free, container-bound

The [`indexing` crate](https://github.com/bluss/indexing) ("Sound unchecked indexing using
'generativity'; a type system approach to indices, pointers and ranges that are **trusted to be
in bounds**") ties every index and range to a `'id` lifetime brand that the borrow checker will
not unify with any other, so an index is usable **only** with the container that minted it.

```rust
use indexing::scope;

fn lower_bound<T: PartialOrd>(v: &[T], elt: &T) -> usize {
    scope(v, move |v| {                      // 'id brand created here, unforgeable
        let mut range = v.range();
        while let Ok(range_) = range.nonempty() {
            let (a, b) = range_.split_in_half();      // <- a branded, proved-disjoint split
            // Access uses no runtime bounds checking and is guaranteed to be in bounds
            if v[b.first()] < *elt { range = b.tail(); } else { range = a; }
        }
        range.first().integer()
    })
}
```
([docs.rs/indexing](https://docs.rs/indexing/latest/indexing/);
[`generativity` crate](https://docs.rs/generativity/latest/generativity/): "a unique lifetime
that the Rust borrow checker will not unify with any other lifetime, which can be used to brand
types such that you know that **you, and not another copy of you**, created them")

Two things to note for the decision:

1. `v[b.first()]` **compiles to an unchecked index**. The static brand discharges the bounds
   obligation — exactly the shape Whitefoot needs, since it cannot trap.
2. `split_in_half()` is a **branded range split**, i.e. N4 answered in the *index* world rather
   than the pointer world, with disjointness carried in types rather than a SAFETY comment.

### 4.2 GhostCell — one permission for a whole aliased collection

[GhostCell: Separating Permissions from Data in Rust](https://plv.mpi-sws.org/rustbelt/ghostcell/)
(Yanovski, Dang, Jung, Dreyer, ICFP 2021 / PACMPL 5(ICFP)), abstract verbatim:

> "The Rust language offers a promising approach to safe systems programming based on the
> principle of *aliasing XOR mutability* … **However, to implement pointer-based data structures
> with internal sharing, such as graphs or doubly-linked lists, we need to be able to mutate
> aliased state.** To support such data structures, Rust provides a number of APIs that offer
> so-called *interior mutability* … Unfortunately, the existing APIs sacrifice flexibility,
> concurrent access, and/or performance, in exchange for safety.
> In this paper, we propose a new Rust API called **GhostCell** which avoids such sacrifices by
> *separating permissions from data*: it enables the user to safely synchronize access to a
> **collection** of data via a **single permission**."

Soundness is machine-checked in Coq by extending RustBelt. The mechanism — a brand plus one
token that owns write permission for the entire collection — is structurally the same as
"the whole owns all nodes; access goes through the whole", but *statically enforced and
erased*, with no per-node runtime check.

**(unverified)** the paper's specific claims about runtime overhead and about which operations
remain `unsafe` inside the GhostCell implementation; only the abstract above was read in full.

---

## 5. Other languages: how "stored reference" needs are met without references

| Language | Is there a *storable* reference? | What it does for N6/N7 | Citation |
|----------|----------------------------------|------------------------|----------|
| **Mojo** | **Yes, but split.** `ref` is a *binding/parameter/return* form only. `Pointer[T, origin]` is the storable form and carries the origin: "When used this way, the `Pointer` type **carries the origin of the value it points to. It can be used to store a reference in a struct field.**" | The manual's own self-referential-struct guidance **drops the origin**: `comptime NodePointer = UnsafePointer[Self, MutUntrackedOrigin]`. Mojo's static origin system does not cover the stored-cyclic case. | [Mojo manual: pointers](https://docs.modular.com/mojo/manual/pointers/), [lifetimes/origins](https://docs.modular.com/mojo/manual/values/lifetimes/), [self-referential structs](https://mojolang.org/docs/manual/structs/reference/) |
| **Koka (Perceus)** | No. In-place update is *inferred* from uniqueness, not named. | Cycles are not collected. "(cycle-free) programs are garbage free." "we leave the responsibility to the programmer to **break cycles by explicitly clearing a reference cell** that may be part of a cycle." | [Perceus, PLDI 2021](https://doi.org/10.1145/3453483.3454032), [PDF](https://xnning.github.io/papers/perceus.pdf) |
| **Lean 4 (FBIP)** | No. | "**Because the verifiable fragment of Lean cannot create cyclic data, the Lean runtime does not have a technique to detect it.**" Cyclic structure is simply outside the model. | [Lean reference manual: Reference Counting](https://lean-lang.org/doc/reference/latest/Run-Time-Code/Reference-Counting/), [Counting Immutable Beans](https://arxiv.org/pdf/1908.05647) |
| **Carbon** | **Yes — pointers, deliberately.** "Unlike C++, Carbon does not currently have reference types. The only form of indirect access are pointers." | Two arguments transferable here: one indirection construct because "N ways × M parameters" explodes API surface; and pointers rather than references because "**Carbon's indirection mechanism retains the ability to refer distinctly to the point*er* and the point*ee* … This ends up critical for supporting rebinding.**" | [carbon-lang docs/design/values.md](https://github.com/carbon-language/carbon-lang/blob/trunk/docs/design/values.md) |
| **Zig** | Yes (raw pointers, no lifetimes). | Pointer stability is **per-method documented folklore**: `ArrayList.items` carries "Pointers to elements in this slice are invalidated by various functions of this ArrayList in accordance with the respective documentation", and ~40 methods each carry an "Invalidates …" line. Zig's own compiler AST consequently uses `u32` indices. | [lib/std/array_list.zig](https://github.com/ziglang/zig/blob/master/lib/std/array_list.zig), [lib/std/zig/Ast.zig](https://github.com/ziglang/zig/blob/master/lib/std/zig/Ast.zig) |
| **Odin** | Yes (pointers); `#soa` is a language feature. | Community idiom is explicit: "The handle can be used as a permanent reference to the items in the map. In other words, **the handle can be used where you would normally store a pointer or index.**" | [Zylinski, Handle-based maps](https://zylinski.se/posts/handle-based-maps-three-implementations/), [Odin overview](https://odin-lang.org/docs/overview/) |
| **Jai** | — | **(unverified)** — no citable first-party design document found; recommend not citing. | — |

Carbon is the most pointed contrast: a *new* systems language, designed with full knowledge of
Rust and Swift, concluded it needed one storable indirection primitive, and gave *rebinding* as
a first-order reason.

---

## 6. Data-oriented practice: pools + generational handles

### 6.1 The idiom is universal, and predates Rust

Catherine West, RustConf 2018 closing keynote
([written version](https://kyren.github.io/2018/09/14/rustconf-talk.html),
[video](https://www.youtube.com/watch?v=aKLntZcp27M)) — the strongest evidence that this is an
industry idiom rather than a borrow-checker workaround:

> "why am I bothering with this `EntityId` stuff here? This is C++, why can't we use pointers?
> Well, it turns out that doing this is very unsafe, so the pattern that **all game engines I've
> ever [seen] (in languages without a fancy garbage collector) … adopt** is actually to have some
> kind of map from some form of 'entity id' to an actual entity pointer … if they kept raw
> pointers, they would be continually invalidated and this tends to lead to hard to solve
> ephemeral bugs and so more or less nobody does this."

> "I genuinely think that most of the time when you find yourself running into self borrowing
> with Rust, plain `Vec`s and indexes should [be the] first tool you reach for. … Self borrowing
> solutions like `rental` are tools of last resort."

and her own statement of the plain-index hazard, and the fix:

> "We can delete an entity which frees up a slot in a `Vec`, but then it's possible that the very
> next allocated entity will use the same index. … We'll get a 'random other entity' in its place
> **without being able to tell that it was in fact removed out from under us!**"

```rust
pub struct GenerationalIndex { index: usize, generation: u64 }
struct AllocatorEntry { is_live: bool, generation: u64 }
pub struct GenerationalIndexAllocator { entries: Vec<AllocatorEntry>, free: Vec<usize> }
impl GenerationalIndexAllocator {
    pub fn allocate(&mut self) -> GenerationalIndex { ... }
    pub fn deallocate(&mut self, index: GenerationalIndex) -> bool { ... }
    pub fn is_live(&self, index: GenerationalIndex) -> bool { ... }
}
```

### 6.2 The crate family and its stated rationale

`generational-arena` module doc, verbatim — this is the canonical statement of why handles
replace references:

> "Imagine you are working with a graph and you want to add and delete individual nodes at a
> time, or you are writing a game and its world consists of many inter-referencing objects with
> dynamic lifetimes that depend on user input. These are situations where matching Rust's
> ownership and lifetime rules can get tricky.
> It doesn't make sense to use shared ownership with interior mutability (i.e. `Rc<RefCell<T>>`
> or `Arc<Mutex<T>>`) nor borrowed references (ie `&'a T` or `&'a mut T`) for structures.
> **The cycles rule out reference counted types, and the required shared mutability rules out
> borrows.** … In these situations, it is tempting to store objects in a `Vec<T>` and have them
> reference each other via their indices."

and its statement of the ABA hazard the generation exists to close:

> "* `obj1` references `obj2` at index `i`
> * someone else deletes `obj2` from index `i` …
> * a third thing allocates `obj3`, which ends up at index `i` …
> * `obj1` attempts to get `obj2` at index `i`, but **incorrectly is given `obj3`**"

```rust
pub struct Index { index: usize, generation: u64 }
pub struct Arena<T> { items: Vec<Entry<T>>, generation: u64,
                      free_list_head: Option<usize>, len: usize }
let mut arena = Arena::new();
let rza = arena.insert("Robert Fitzgerald Diggs");
assert_eq!(arena[rza], "Robert Fitzgerald Diggs");
arena.remove(rza);
```
([docs.rs/generational-arena](https://docs.rs/generational-arena/latest/generational_arena/))

| Crate | Handle | `size_of::<Index>()` | Generation? |
|-------|--------|----------------------|-------------|
| `thunderdome` | index + gen | 8 | yes |
| `slotmap` | `idx: u32, version: NonZeroU32` | 8 | yes (wraps after 2³¹) |
| `generational-arena` | `index: usize, generation: u64` | 16 | yes |
| `slab` | key only | 8 | **no** — "keys may be reused" |
([thunderdome README](https://github.com/LPGhatguy/thunderdome),
[slotmap](https://docs.rs/slotmap/latest/slotmap/),
[slab](https://github.com/tokio-rs/slab/blob/master/src/lib.rs))

### 6.3 ECS engines: `Entity` *is* a generational index; parent links *are* IDs

**Bevy** — `crates/bevy_ecs/src/entity/mod.rs`:

```rust
#[repr(C, align(8))]
pub struct Entity {
    // ordering is explicitly used by repr(C) to make this struct equivalent to a u64
    #[cfg(target_endian = "little")] index: EntityIndex,
    generation: EntityGeneration,
    #[cfg(target_endian = "big")]    index: EntityIndex,
}
```
> "Unique identifier for an entity in a `World`. **Note that this is just an id, not the entity
> itself.** Further, the entity this id refers to may no longer exist in the `World`."
> "# Aliasing … it is possible for a later entity to be spawned at the exact same id! … **Aliasing
> can happen without warning.**"

N7 is a component holding an `Entity`, and the inverse index is maintained by machinery:

```rust
#[relationship(relationship_target = Children)]
pub struct ChildOf(#[entities] pub Entity);
// "When ChildOf is inserted on a 'source' entity, the 'target' entity will automatically
//  (and immediately, via a component hook) have a Children component inserted"
let root       = world.spawn_empty().id();
let child1     = world.spawn(ChildOf(root)).id();
let grandchild = world.spawn(ChildOf(child1)).id();
```
([bevy_ecs/entity/mod.rs](https://github.com/bevyengine/bevy/blob/main/crates/bevy_ecs/src/entity/mod.rs),
[hierarchy.rs](https://github.com/bevyengine/bevy/blob/main/crates/bevy_ecs/src/hierarchy.rs))

Design-relevant cost: **a forward handle cannot be traversed backwards, so bidirectional links
need a maintained inverse index.** Bevy pays for this with component hooks.

**EnTT** (C++) packs both into one `u32`:

```cpp
template<> struct entt_traits<stl::uint32_t> {
    using entity_type  = stl::uint32_t;
    using version_type = stl::uint16_t;
    static constexpr entity_type entity_mask  = 0xFFFFF;  // 20 bits index
    static constexpr entity_type version_mask = 0xFFF;    // 12 bits version
};
```
> "when an identifier is released, the registry can freely reuse it internally. In particular,
> **the version of an entity is increased**."
([entt/entity/entity.hpp](https://github.com/skypjack/entt/blob/master/src/entt/entity/entity.hpp),
[docs/md/entity.md](https://github.com/skypjack/entt/blob/master/docs/md/entity.md))

### 6.4 Compilers do this too — and they fixed the type-confusion hazard in the type system

**rustc.** `rustc_index::IndexVec` doc, verbatim:

> "An `IndexVec` allows element access only via a specific associated index type, meaning that
> **trying to use the wrong index type (possibly accessing an invalid element) will fail at
> compile time.**"

```rust
fn f<I1: Idx, I2: Idx>(vec1: IndexVec<I1, u8>, idx1: I1, idx2: I2) {
    &vec1[idx1]; // Ok
    &vec1[idx2]; // Compile error!
}
```
([rustc_index::IndexVec](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_index/vec/struct.IndexVec.html);
`DefId`/`HirId`/`NodeId` in the [rustc dev guide glossary](https://rustc-dev-guide.rust-lang.org/appendix/glossary.html))

**Zig.** `lib/std/zig/Ast.zig` uses `u32` indices, and has since upgraded them from bare `u32`
to **non-exhaustive enums** to recover a type distinction between index kinds:

```zig
pub const TokenIndex = u32;                 // index into `tokens`
pub const NodeList   = std.MultiArrayList(Node);
pub const Index = enum(u32) { root = 0, _, ... };   // was a bare u32
```
Measured (Zig 0.9.0 release notes, primary source):
> Bootstrap compiler: "wall clock: 4.17 seconds / peak RSS: 439 MiB"
> Self-hosted compiler: "wall clock: 2.76 seconds / peak RSS: 236 MiB"
> "it's 1.5x faster and 0.53x as much memory."
([ziglang.org 0.9.0 release notes](https://ziglang.org/download/0.9.0/release-notes.html),
[Ast.zig](https://github.com/ziglang/zig/blob/master/lib/std/zig/Ast.zig))

Andrew Kelley's Handmade Seattle 2021 talk
([media.handmade-seattle.com](https://media.handmade-seattle.com/practical-data-oriented-design/))
gives per-structure numbers (tokens 64 B → 5 B; AST nodes 120 B → 15.6 B; 39% wall-clock
reduction) — **(unverified: these figures come from a third-party transcript, not the video.)**

Andre Weissflog, ["Handles are the better pointers"](https://floooh.github.io/2018/06/17/handles-vs-pointers.html) (2018), the origin of the slogan:
> "Since only the system knows the array base pointers, it's **free to move or reallocate the
> item arrays at will without invalidating existing index handles.**"
> "Array indices need fewer bits than full pointers, and a smaller data type can be picked for
> them, which in turn allows tighter packing of data structures and better data cache usage."
Conceded downside: it "doesn't fit very well into a classical OOP world."

### 6.5 Where practitioners report pain

| Pain | Quote | Source |
|------|-------|--------|
| **Wrong-arena / dangling index** | "either you reuse the node/edge indices … or else you leave a placeholder. The former approach leaves you vulnerable to '**dangling indices**', and the latter is a kind of leak. **This is basically exactly analogous to malloc/free.** Another similar problem arises if you use the index from one graph with another graph (you can mitigate that with fancy type tricks, but **in my experience it's not really worth the trouble**)." | [Matsakis, *Modeling graphs in Rust using vector indices*](https://smallcultfollowing.com/babysteps/blog/2015/04/06/modeling-graphs-in-rust-using-vector-indices/) |
| same | "We could take an `Idx` returned from one arena and use it with another, resulting in either the wrong data or an out-of-bounds error." The branded-lifetime fix costs: the lifetime cannot leave the scope, and the arenas/indices "can no longer be `Send` nor `Sync`". | [llogiq, *Arenas vs. Indices*](https://llogiq.github.io/2019/04/06/arena.html) |
| **"You reinvented a GC"** | "**With refcounting, the accounting needed for lifetime tracking happens when you copy a pointer.** So copying pointers is expensive, but dereferencing them is cheap … **With a generational arena, the accounting … happens when you access an index. So copying indices is cheap, but dereferencing them is expensive because you have to check whether the index is valid.** … There ain't no such thing as a free lunch." | [icefox on Lobsters](https://lobste.rs/s/2rgojg/single_ownership_memory_safety_without) |
| **Loss of static borrow + type checking** | "`generational-arena` … essentially **defers borrow analysis to runtime**." "The code is a bit ugly. **You lose static borrow checking AND static type checking.**" "there's no way to statically assert to the compiler that the arena will live longer than all of its allocations *without* using `unsafe`." | [users.rust-lang.org, *Using an Arena without passing it around everywhere*](https://users.rust-lang.org/t/using-an-arena-without-passing-it-around-everywhere/41831) |
| **Context piping / verbosity tax** | "there's a **huge syntax tax** I'd like to avoid … I can't think of a way to use an Arena without polluting all the interfaces in my code base … **Imagine if every function had to pass a 'malloc' function around…**" | same thread |
| **No methods on the node** | `c.move_by(1,1)` becomes `c_idx.move_by(&mut arena, 1, 1)`; "this change requires that I **define a new trait for every existing trait** due to the change in each function signature." | [users.rust-lang.org, *Extending object traits to an arena type data structure*](https://users.rust-lang.org/t/extending-object-traits-to-an-arena-type-data-structure/54572) |
| **Index ≠ aliasing control** | "By using indirect reference (i.e. index) to an object, I can safely and freely distribute copies of the reference, but **it also allows any consumer of any object data to have mutable access to all of data.**" | same thread |
| **Counterpoint** | "We still need to avoid `&mut self` methods, and each function has an extra `people` argument. But **aliasing mistakes are compiler errors instead of panics**, and there's no risk of memory leaks." … but "we can't delete anything from the `Vec` without messing up the indexes of other elements." | [O'Connor, *Object Soup is Made of Indexes*](https://jacko.io/object_soup.html) |
| **Gamedev critique** | generational arenas "[allow] a language like Rust to **completely side-step the borrow checker**"; "one of the big annoying downsides is that one has to define a variable and a type for every arena they intend to use." | [LogLog Games, *Leaving Rust gamedev after 3 years*](https://loglog.games/blog/leaving-rust-gamedev/) |

**Generation wraparound is a real soundness boundary in every implementation**: `slotmap`
wraps after 2³¹ reuses of a slot ("*After 2^31 deletions and insertions to the same underlying
slot the version wraps around and such a spurious reference could potentially occur"), EnTT's
default version is **12 bits**, and Bevy's wraps and therefore deliberately does not implement
`Ord` on `EntityGeneration`. A design that claims a handle is *proved* valid cannot rest on a
wrapping counter.

---

## 7. Cross-cutting summary

Cell format: **how it is met** / *escape required*.

| Need | Hylo | Swift (6.4) | Rust | Pool + handle idiom |
|------|------|-------------|------|---------------------|
| **N1** cursor over growing container | `Position` value + whole-collection access / *runtime bounds precondition* | snapshot the collection (`let _elements`) + index / *none, but it is a frozen cursor*; real `MutableRef` cursor / *viral `~Escapable`* | index; `&mut` cursor is impossible while growing / *none* | stable generational handle / *validity check* |
| **N2** two writable names, one slot | refused (mutable projection makes source inaccessible) / **N/E** | refused — this *is* the Law of Exclusivity / *class + runtime trap* | refused (aliasing XOR mutability) / *`RefCell` runtime, or `unsafe`* | indices alias freely but give no write control / *no static guarantee at all* |
| **N3** callback holding a reference | borrowing closure cannot escape; only `sink` capture escapes / **N/E** | copy-in/`defer` copy-out, or class / *semantics change, or heap box + dynamic enforcement* | **meets it** with `&'a mut T` + lifetime parameters / *none* | callback holds a handle; container passed at call time / *none* |
| **N4** split across a call | constant distinct paths only; stdlib uses raw pointers / **unsafe** | `extracting` freezes the parent; `split(at:)` unimplemented / **unsafe** | safe API, `unsafe` kernel with a hand-written SAFETY comment / **unsafe** | branded range `split_in_half()` / *statically proved (research/library)* |
| **N5** mutable element-yielding iterator | `next()` yields by value; per-element `inout` subscript / **N/E** as a reference | no protocol; index loop + `_modify`; `next() -> Ref<Element>` considered and **rejected** for perf / **N/E** | `IterMut { ptr: NonNull<T>, end_or_len: *mut T }` / **unsafe** | index loop / *per-step bounds check* |
| **N6** intrusive structure | node array + offsets; **1 order of magnitude faster** than references | class + ARC + `weak`, or index arena | `Rc<RefCell<>>` (runtime), or arena | **the standard answer**, universally adopted |
| **N7** back-pointer | identity value in a table owned by the whole | `weak`/`unowned` class ref / *ARC runtime check* | index, or `Weak<T>` / *runtime* | entity id + maintained inverse index (Bevy `ChildOf`/`Children`) |

Two patterns fall out immediately:

- **N4 and N5 are met by an unsafe kernel in *every* production language surveyed, including
  Rust.** Adding first-class references does not make them checkable; it only relabels where
  the unsafe kernel sits.
- **N6 and N7 have a clean, reference-free answer with *measured* advantages** and are the
  overwhelming industry default in engines, compilers and arena libraries.

---

## ASSESSMENT

*This section is judgement over the evidence above, clearly separated from it.*

### A. Needs with a clean reference-free answer

**N6 (intrusive structure) and N7 (back-pointer) are settled.** This is the strongest result in
the survey and it is not close. Racordon (Programming 2025) implemented a doubly-linked list and
a web graph in Swift's safe subset with no first-class references, got *stable* addresses across
reallocation, and measured the value version **one order of magnitude faster** on insert,
traverse and remove, with **no noticeable difference** on Tarjan SCC and Dijkstra. Independently:
Bevy, EnTT, rustc, the Zig self-hosted compiler and the Odin/handle community all converge on
the same representation, and West's keynote establishes that C++ engines did this *before* Rust
existed because raw pointers "would be continually invalidated." A permanent boundary that
refuses stored references for graph-shaped data is refusing something the industry already
refuses.

**N1 (cursor over a growing container) is a false attribution.** No language in this survey lets
a *reference* cursor survive container growth — Rust's `&mut` cursor exclusively borrows the
container so growth is impossible, and Swift's `IndexingIterator` sidesteps the question by
storing a COW snapshot, which means a Swift cursor is over a *frozen* container, not a growing
one. The universal answer is a value handle. P4 is a cost of *growth*, not a cost of the
reference-free model, and first-class references would not fix it.

**N3 has a reference-free answer that is a real design change, not a workaround**: either the
callback *owns* its state (Hylo's `sink` capture, which is escapable), or it holds a handle and
receives the container at call time. That is a genuine expressiveness loss relative to Rust,
but it is the same loss Swift takes, and Swift's own answer (copy-in/`defer` copy-out, or a
class with a heap box under dynamic enforcement) is worse than the handle answer.

### B. Needs that every surveyed system meets only with an escape

**N4 (split across a call) and N5 (mutable element-yielding iterator).** These are met nowhere
in a checked safe subset:

- Rust: `split_at_mut_unchecked` is `unsafe` with a prose SAFETY comment; the safe wrapper
  `panic!`s. `IterMut` stores `NonNull<T>` and `*mut T`, not a reference.
- Swift: `MutableSpan.split(at:)` does not exist; SE-0467 calls solving it "a top priority";
  the only path is `withUnsafeMutableBufferPointer`. There is no mutable-element iteration
  protocol and SE-0516 *rejected* the `next() -> Ref<Element>` design on performance grounds.
- Hylo: refuses computed-index `inout` pairs by the path rule; its own `Array.swap_at` is two
  raw pointers and `.unsafe[]`.

This is the decisive finding for the boundary question. **Whitefoot cannot adopt any surveyed
system's answer to N4/N5, because every one of them is an unsafe escape or a runtime trap, and
Whitefoot admits neither.** So for these two needs the choice is not "references or no
references" — it is "refuse, or discharge by proof".

**N2 (two long-lived writable names for one slot) is not a gap; it is the shared axiom.** Rust's
aliasing-XOR-mutability, Swift's Law of Exclusivity and Hylo's projection rule are three
statements of the same ban. Nobody meets N2 statically. Swift's escape is a class under
*runtime* exclusivity checks that are enabled in Release and trap; Rust's is `RefCell`, also a
runtime check. Adding first-class references to Whitefoot would not deliver N2 — Rust has
first-class references and still refuses it. P7 should be reclassified from "capability gap" to
"the property the model is buying".

### C. What is genuinely unmet without first-class references

Being strict about "unmet in *every* surveyed system, and attributable to the absence of
first-class references rather than to the aliasing discipline":

1. **An escaping callback that holds a live mutable borrow of caller state (N3).** Rust meets
   this with lifetime parameters. Hylo and Swift do not, and both say so explicitly. This is the
   one need where the second-class boundary, and not the exclusivity axiom, is the binding
   constraint. The theoretical statement is precise: second-class values "cannot be returned
   under any circumstances, **even when this would be sound**"
   ([Osvald et al., OOPSLA 2016](https://www.cs.purdue.edu/homes/rompf/papers/osvald-oopsla16.pdf);
   the reachability-types line of work exists to relax exactly this).
2. **A *reassignable* reference variable for graph traversal.** SE-0519's own Future Directions
   make this argument better than I can: scoped bindings cannot serve, because "the reference
   itself cannot be reassigned. In a graph of values that refer to each other … a traversal loop
   can update a `Ref` variable in-place as it advances through the graph, but not a `borrow`
   binding." Under a pool model the equivalent is a mutable *handle* variable, which Whitefoot
   already has — so this is met, at the cost of re-indexing the pool each step.
3. **Nothing else.** N1, N6 and N7 are met better without references. N2, N4 and N5 are met by
   nobody without an escape.

### D. The two obligations the reference-free model actually creates

Both are named by the primary sources as *conceded* costs, and both are the places where
Whitefoot's proof system changes the answer.

**1. Handle validity becomes a dynamic question in every existing system.** Hylo's `Array`
subscript is a `precondition` (a trap). Swift's `Array` subscript traps. Rust's `split_at_mut`
`panic!`s. `slotmap` wraps after 2³¹ reuses of a slot; EnTT's version field is **12 bits**;
Bevy's generation wraps and therefore refuses to implement `Ord`. icefox's critique is
technically correct as stated: "**with a generational arena, the accounting needed for lifetime
tracking happens when you access an index** … dereferencing [is] expensive because you have to
check whether the index is valid." A design that claims a handle is *proved* valid cannot rest
on a wrapping generation counter. But note what this means: **the entire cost of the pool idiom,
as the critics state it, is a check that a proof system can discharge.** That is the case for
believing Whitefoot's boundary is better than the boundary of every system surveyed, not merely
equal to it.

**2. Wrong-arena confusion is real, and the type system already fixes it.** Matsakis judged the
"fancy type tricks" not "worth the trouble" in 2015; rustc disagreed and built `newtype_index!`
and `IndexVec` so that "trying to use the wrong index type … **will fail at compile time**"; Zig
upgraded its AST indices from bare `u32` to `enum(u32)` for the same reason; Racordon's artifact
wraps the offset in a nominal `Address` type. Three serious compilers independently reached for
a type-level distinction. Whitefoot can make handles container-bound by construction rather than
by library convention, and the branded-index / generativity work
([`indexing`](https://docs.rs/indexing/latest/indexing/),
[GhostCell, ICFP 2021](https://plv.mpi-sws.org/rustbelt/ghostcell/)) shows the stronger form —
`v[b.first()]` with "**no runtime bounds checking**", and a branded `split_in_half()` — is
already sound and machine-checked in Coq.

### E. The cost the sources agree on and nobody has solved: context piping

Every reference-free system hits it and names it:

- Hylo: "we always need explicit access to the base collection … becomes quite inconvenient when
  [we] want to pass things around in functions. I call this the **'context piping problem'**."
  Proposed fix — implicit parameters — "not high on the priority list."
- The MVS paper: "traversing an arbitrary graph **requires access to the whole graph at each
  step**, rather than just a single vertex and its outgoing edges."
- Rust arena users: "**Imagine if every function had to pass a 'malloc' function around…**";
  `c.move_by(1,1)` becomes `c_idx.move_by(&mut arena, 1, 1)`; "this change requires that I
  define a new trait for every existing trait."
- Bevy pays for the *inverse* direction with component hooks that maintain `Children` whenever
  `ChildOf` is inserted.

This is an ergonomic tax, not an expressiveness limit, and it is the honest cost of the
boundary. It is also the one place where the "written by AI agents" framing cuts differently
from the human-ergonomics literature: threading a container parameter is verbose to write and
*easier* to check, and an agent pays the verbosity cost far more cheaply than a human does.

### F. The one fact that most complicates the "permanent boundary" framing

**Swift — the language the MVS paper itself studies as the exemplar of the discipline — has
added first-class references.** SE-0519 `Ref`/`MutableRef` is *Implemented (Swift 6.4)*, and its
Motivation is precisely the Whitefoot question: "It would be useful to be able to form these
sorts of references outside of the confines of a function call, as local variable bindings, as
members of other types, as elements of generic containers … Developers can use classes to box
values … but in doing so they introduce allocation, reference counting, and dynamic exclusivity
checking overhead. `UnsafePointer` is, of course, unsafe."

Three qualifications keep this from being decisive against the boundary:

1. It took four stacked features (`~Copyable` → `~Escapable` → lifetime dependencies →
   `borrow`/`mutate` accessors) and the **lifetime-dependency layer still has no accepted
   Evolution proposal** — the attribute ships as `@_lifetime` and PR #2750 is closed unmerged.
   That is the complexity price of first-class references, paid in public.
2. The result is admittedly incomplete: SE-0519 calls `Ref`/`MutableRef` "an **unsatisfying
   endpoint** to the local reference bindings story", cannot express `Ref` to a `~Escapable`
   type ("the current model lacks the ability to track multiple lifetimes per value"), and
   still wants Hylo-style yielding functions as a Future Direction.
3. `~Escapable` is **viral upward through storage** — one nonescapable field makes the whole
   struct nonescapable, barring it from classes, globals, escaping closures and Tasks. N3, N6
   and N7 all die on that one rule. Swift bought reference *bindings*, not stored references in
   long-lived aggregates.

Read carefully, Swift's move is evidence that the boundary binds in practice — and also evidence
about what relaxing it costs.

### G. Bottom line for the decision

The evidence supports the boundary as a **permanent capability boundary**, on these terms:

- Refusing stored references costs **N3** (escaping borrowing callback) and nothing else that
  another system delivers safely.
- **N2, N4, N5** are refused or escaped by every system surveyed, Rust included. They are not
  arguments for first-class references.
- **N1, N6, N7** are met better without references, with measured evidence.
- The two real costs — dynamic handle validity and wrong-container confusion — are exactly the
  two obligations a machine-checked proof system can discharge and every surveyed system cannot.
  That is the differentiating argument, and it should be the one the decision rests on.
- The one unfunded cost is **context piping**, which no surveyed system has solved, and which
  the AI-writer framing makes cheaper than the literature assumes.


# File: systems-code-references.md

# Evidence survey: do kernels, compilers and browsers need stored references?

Question for the owner: can a core model with **no first-class references** —
call conventions `let`/`inout`/`sink`, second-class projections passed down
calls but never stored or returned, pool handles for shared or cyclic
structure, index-based access — express the data structures that kernels,
compilers and browsers actually need? Or is a *stored* reference
indispensable somewhere?

Survey date: 2026-09-16. Everything below was fetched and read during this
survey unless it carries the marker **(unverified)**. Quotes are verbatim from
the cited URL. Where a claim rests on a secondary source that paraphrases a
primary one, the row says so.

Terminology used throughout:

| term | meaning here |
|---|---|
| stored reference | an address of another object held in a field of a live object, dereferenceable without further authority |
| second-class projection | an access passed into a call and not storable or returnable (Whitefoot `inout`/`&uniq`, Hylo projections, SPARK borrow) |
| pool + handle | values live in one owning container; other values hold a plain copyable index (optionally index + generation) |
| index-linked | a pointer field replaced by an integer offset into a known container |

---

## Master table: data-structure needs

"Production" below means shipped, load-bearing software, not a demo or a
benchmark. Where the reference-free form exists only in research or in a
non-production library, the row says so.

| # | need | pointer form stores | reference-free form | production use of the reference-free form | cost reported | citation |
|---|---|---|---|---|---|---|
| 1 | **intrusive doubly linked list** (O(1) unlink given the element, no allocation at insert) | `struct list_head { struct list_head *next, *prev; }` embedded in the object; container recovered by `container_of` offset subtraction | node record with `prev`/`next` **indices** into the owning container plus a free list; the container owns its nodes | **Yes.** SPARK's formally verified `Doubly_Linked_Lists`: `Prev : Count_Type'Base; Next : Count_Type;`, `Cursor` is a node index. Also Racordon's Swift list. | bounded capacity (SPARK's is `List (Capacity : Count_Type)`); insertion becomes amortized O(1) if the array grows; unlink still needs the container in scope, so `list_del(entry)` with no list handle has no direct analogue | [types.h](https://raw.githubusercontent.com/torvalds/linux/master/include/linux/types.h), [container_of.h](https://raw.githubusercontent.com/torvalds/linux/master/include/linux/container_of.h), [SPARKlib .ads](https://raw.githubusercontent.com/AdaCore/SPARKlib/master/src/spark-containers-formal-doubly_linked_lists.ads), [OASIcs.Programming.2025.25](https://drops.dagstuhl.de/entities/document/10.4230/OASIcs.Programming.2025.25) |
| 2 | **tree with parent links** (rb-tree, DOM, scene graph) | `struct rb_node { unsigned long __rb_parent_color; struct rb_node *rb_right, *rb_left; }` — parent pointer with colour in the low bits | `parent/prev/next/first/last : Option<NodeId>` in one `Vec`; all five relations O(1) | **Yes, outside browsers.** `ego-tree` (behind `scraper`); `blitz-dom` over `slotmap`. **No production browser engine.** Blink, Gecko, WebKit, Servo are all pointer-based. | ego-tree: "Nodes can be detached (orphaned) but not removed" — no reclamation. blitz stores no sibling links, so `nextSibling` degrades to a linear scan of the sibling list. Blink keeps parent **uncompressed** because it is hot. | [rbtree.rst](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/plain/Documentation/core-api/rbtree.rst), [ego-tree](https://raw.githubusercontent.com/rust-scraper/ego-tree/master/src/lib.rs), [node.h](https://raw.githubusercontent.com/chromium/chromium/main/third_party/blink/renderer/core/dom/node.h) |
| 3 | **general graph** (cycles, sharing, multiple in-edges) | `Rc<RefCell<Node>>` / `shared_ptr` with cycles, or raw back-pointers | vertices in a pool; edges are handles or keys; "peer relationships represented independently" | **Yes.** petgraph is the Rust idiom; rustc, Cranelift, Zig, Carbon all do this for IR graphs. Measured: Tarjan SCC and Dijkstra show "no noticeable difference between the two graph implementations". | removal invalidates indices unless generational (petgraph: "Removing a node will force the last node to shift its index"); cross-arena index mixup is undetected without branding, and branding costs `Send`/`Sync` | [petgraph](https://docs.rs/petgraph/latest/petgraph/graph/struct.Graph.html), [Racordon 2025](https://drops.dagstuhl.de/entities/document/10.4230/OASIcs.Programming.2025.25), [llogiq](https://llogiq.github.io/2019/04/06/arena.html) |
| 4 | **wait queue** (sleeper enqueues itself, waker walks the list) | `struct wait_queue_entry { unsigned int flags; void *private; wait_queue_func_t func; struct list_head entry; }` — a stack-allocated entry with a task pointer **and a function pointer**, linked into a head owned by someone else | index-linked waiter pool + a closed enum tag instead of `func`; the `void *private` becomes a task handle | **Partly.** io_uring's SQ/CQ rings are pure index rings in shared memory, no pointers at all. But no production waitqueue-equivalent with an index-linked, stack-allocated entry was found. | the stack-allocated entry is the hard part: the entry's storage outlives the enqueue call but not the waiting frame. Whitefoot's `dispatch.md` already forbids the `func` pointer, so the callback half is out of the model regardless. | [wait.h](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/plain/include/linux/wait.h), [io_uring(7)](https://man7.org/linux/man-pages/man7/io_uring.7.html) |
| 5 | **RCU-protected pointer** (lock-free read of a field being replaced) | a pointer field published with `rcu_assign_pointer`, read with `rcu_dereference`; reclamation deferred via an embedded `rcu_head` + callback | an index field published atomically; reclamation deferred by **epoch**, or avoided entirely by never recycling | **Yes, for the reclamation half.** crossbeam-epoch: "it is inserted into a pile of garbage and marked with the current epoch." **Not found** for a production index-based RCU-style publish inside a kernel. | the epoch scheme is about reclamation, not addressing; append-only avoids it but never reclaims. A published index read concurrently still needs the same release/acquire ordering a pointer does. | [whatisRCU.rst](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/plain/Documentation/RCU/whatisRCU.rst), [crossbeam-epoch](https://docs.rs/crossbeam-epoch/latest/crossbeam_epoch/) |
| 6 | **arena IR / AST** (compiler nodes referring to nodes) | `llvm::Use { Value *Val; Use *Next; Use **Prev; User *Parent; }` — 4 × 64-bit per def-use edge; `ilist` intrusive links in `Instruction`/`BasicBlock` | flat arrays addressed by a newtyped **u32** index | **Yes — this is the dominant shape in new compilers.** rustc (`IndexVec<BasicBlock, _>`, `newtype_index!` "uses a u32"), Cranelift (`entity_impl!` over `u32`), Zig (`enum(u32)`, `MultiArrayList`), Carbon (`IdBase` over `int32_t`). | debugging needs context-aware dumpers (Carbon built `Dump(ctx, id)`, a custom LLDB command and `--sem-ir-crash-dump`); raw integers lose type safety unless newtyped; secondary maps silently return defaults; the arena lifetime threads everywhere. LLVM's counter-case: intrusive lists buy "guaranteed to support a constant-time splice operation" and polymorphic storage. | [entities.rs](https://raw.githubusercontent.com/bytecodealliance/wasmtime/main/cranelift/codegen/src/ir/entities.rs), [MIR](https://rustc-dev-guide.rust-lang.org/mir/index.html), [Carbon README](https://raw.githubusercontent.com/carbon-language/carbon-lang/trunk/toolchain/docs/README.md), [LLVM Use.h](https://raw.githubusercontent.com/llvm/llvm-project/main/llvm/include/llvm/IR/Use.h), [Carbon debugging.md](https://raw.githubusercontent.com/carbon-language/carbon-lang/trunk/toolchain/docs/debugging.md) |
| 7 | **DOM** (spec-mandated tree, plus a JS heap that can cycle with it) | Blink: 4 traced `Member<T>` links + parent; Servo: 6 `MutNullableDom` links; both under a tracing GC | node pool + `NodeId` handles; `html5ever`'s `TreeSink` already abstracts the handle to `type Handle: Clone` | **Partly.** `scraper` (`Handle = NodeId`) and `blitz-dom` (slotmap + `NodeId`) run the real HTML5 tree-construction algorithm on indices. **No production browser** does. | the unsolved part is **lifetime across the JS boundary**, not addressing: "The C++ reference counting will never destroy a cycle, and the JavaScript garbage collector can't trace through the C++ pointers." The index engine (blitz) has no JS heap — an honest confound. Costs of the pointer answers are measured: MiraclePtr +4.5-6.5% browser memory, ~7% main-thread contention, ~50% of UaFs made non-exploitable. | [DOM Standard](https://dom.spec.whatwg.org/), [TreeSink](https://raw.githubusercontent.com/servo/html5ever/main/markup5ever/interface/tree_builder.rs), [Servo GC](https://research.mozilla.org/2014/08/26/javascript-servos-only-garbage-collector/), [MiraclePtr](https://security.googleblog.com/2022/09/use-after-freedom-miracleptr.html) |
| 8 | **iterator / traversal with mutation** | an iterator storing a reference (or raw node pointer) into the collection it mutates | index cursor + **deferred mutation queue** applied at one exclusive point | **Yes, universally.** Bevy `Commands` + `ApplyDeferred`; Unity structural changes restricted to the main thread. Whitefoot's own P1 command buffer is the same mechanism. | Unity states it plainly: "Structural changes to the data in ECS are the primary cause of sync points"; "You can't make structural changes directly in a job because it might invalidate other jobs that are already scheduled." Bevy RFC 53: "Updates are still not immediately visible within a stage"; "Hierarchy updates are now single threaded." Also Borretti's named limit of second-class refs: "The main pain point is iterators." | [Commands](https://docs.rs/bevy_ecs/latest/bevy_ecs/system/struct.Commands.html), [structural changes](https://docs.unity3d.com/Packages/com.unity.entities@1.3/manual/concepts-structural-changes.html), [RFC 53](https://raw.githubusercontent.com/bevyengine/rfcs/main/rfcs/53-consistent-hierarchy.md), [Borretti](https://borretti.me/article/second-class-references) |
| 9 | **two-element borrow** (swap, compare-and-move two slots) | two `&mut` into the same collection | index pair with a **proved** disjointness obligation, or a split view | **Yes, but the proof is runtime in every production system found.** Rust: `split_at_mut` (static, positional only); `get_disjoint_mut` — "This method does a O(n^2) check to check that there are no overlapping indices". Whitefoot is the outlier: LIV-2 discharges `left < right` / `right < left` **statically** in the multi-target commit. | std's workaround paragraph appears three times in the slice docs; unchecked variants exist because the check cannot be elided; positional splitting cannot express "these two runtime indices differ" | [std slice](https://doc.rust-lang.org/std/primitive.slice.html), `design/language/ownership.md` (LIV-2 decision) |
| 10 | **device / driver back-pointer, resource identity** | `struct device *parent; struct device_driver *driver; void *driver_data;` | an integer handle validated against a table | **Yes.** Zircon: "In user mode a handle is simply a specific number returned by some syscall", validated against the process handle table. POSIX fds are the same shape. DMA: devices get a `dma_addr_t`, never a CPU pointer. | a handle table lookup per use; process-local meaning (a handle number is meaningless in another process) | [device.h](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/plain/include/linux/device.h), [Zircon Handles](https://fuchsia.googlesource.com/fuchsia/+/refs/heads/main/docs/concepts/kernel/handles.md), [dma-api-howto](https://docs.kernel.org/core-api/dma-api-howto.html) |
| 11 | **persistent / on-disk structure** (for contrast) | — | page numbers, inode numbers: pointers have never been usable here | **Yes, universally.** SQLite b-tree interior cells hold "A 4-byte big-endian page number which is the left child pointer"; the freelist is a page-number chain. | none attributable to the index form; this is the baseline case where index-linking is the only option and nobody complains | [SQLite file format](https://www.sqlite.org/fileformat.html) |



---

## 1. Linux kernel: how much is "a pointer stored inside an object"

Almost all of it, and the pointers point into the *interior* of other objects.

| structure | what is stored inside the object | source |
|---|---|---|
| `struct list_head` | `struct list_head *next, *prev;` — two absolute pointers to *embedded nodes*, not to the containing objects | [types.h](https://raw.githubusercontent.com/torvalds/linux/master/include/linux/types.h) |
| empty-list init | `#define LIST_HEAD_INIT(name) { &(name), &(name) }` — self-referential; the object cannot be moved after init | [list.h](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/plain/include/linux/list.h) |
| `container_of` | nothing stored; recovers the container by subtracting a static offset: `(type *)((void *)(ptr) - offsetof(type, member))` | [container_of.h](https://raw.githubusercontent.com/torvalds/linux/master/include/linux/container_of.h) |
| `struct rb_node` | `unsigned long __rb_parent_color;` (parent pointer with the colour bit packed into the low bits) plus `rb_right`, `rb_left` | [rbtree_types.h](https://raw.githubusercontent.com/torvalds/linux/master/include/linux/rbtree_types.h) |
| `struct hlist_node` | `struct hlist_node *next, **pprev;` — `pprev` is the address of the pointer *slot* that points at this node | [types.h](https://raw.githubusercontent.com/torvalds/linux/master/include/linux/types.h) |
| `struct wait_queue_entry` | a `list_head`, a `wait_queue_func_t func` callback pointer, and `void *private` holding the `task_struct *` | [wait.h](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/plain/include/linux/wait.h) |
| `struct rcu_head` | `struct callback_head *next; void (*func)(struct callback_head *head);` embedded in the object it reclaims | [types.h](https://raw.githubusercontent.com/torvalds/linux/master/include/linux/types.h) |
| `struct device` | `struct device *parent;`, `struct device_driver *driver;`, `void *driver_data;` — up-pointer, back-pointer and untyped escape hatch | [device.h](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/plain/include/linux/device.h) |
| `struct pci_dev` | embeds `struct device dev;` by value; recovered with `#define to_pci_dev(n) container_of(n, struct pci_dev, dev)` | [pci.h](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/plain/include/linux/pci.h) |

The kernel states the intrusive design as a deliberate performance decision:

> "The Linux rbtree implementation is optimized for speed, and thus has one
> less layer of indirection (and better cache locality) than more traditional
> tree implementations. Instead of using pointers to separate rb_node and data
> structures, each instance of struct rb_node is embedded in the data structure
> it organizes."
> — [Documentation/core-api/rbtree.rst](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/plain/Documentation/core-api/rbtree.rst)

`hlist`'s one-word head has its own stated reason:

> "Double linked lists with a single pointer list head. Mostly useful for hash
> tables where the two pointer list head is too wasteful. You lose the ability
> to access the tail in O(1)."
> — [list.h](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/plain/include/linux/list.h)

RCU is the case where a *stored* pointer is read concurrently with its update:

> "The reader uses the spatial rcu_dereference() macro to fetch an
> RCU-protected pointer, which returns a value that may then be safely
> dereferenced."
> "Note that the value returned by rcu_dereference() is valid only within the
> enclosing RCU read-side critical section."
> — [Documentation/RCU/whatisRCU.rst](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/plain/Documentation/RCU/whatisRCU.rst)

### The stated reason intrusive lists exist — the strongest single finding

Rust-for-Linux answers the design question directly, and the answer is *not*
"because pointers are natural". It is **allocation**:

> "The kernel uses a lot of intrusive linked lists, which are extremely rare in
> userspace Rust. This is a consequence of a unique limitation in kernel code
> related to memory allocations: Memory allocations are always fallible and
> failures must be handled gracefully. When you are in an atomic context (e.g.
> when holding a spinlock), you are not allowed to allocate memory at all."
> "This means that we need data structures that do not need to allocate. Or
> where the allocation and insert steps are separate. Imagine a map protected
> by a spinlock. How do you implement that if insert simply cannot allocate
> memory?"
> "Then, given an `Arc<MyValue>`, you can insert that into a linked list
> without having to allocate memory. The only thing you have to do is adjust
> the next/prev pointers. Additionally, there are a bunch of C APIs that work
> using the same principle, so we are also forced into this pattern when we
> want to use those C APIs."
> — [Arc in the Linux kernel](https://rust-for-linux.com/arc-in-the-linux-kernel)

This matters for the decision because a **fixed-capacity pool satisfies the
same requirement**: insertion into a pre-sized pool with a free list allocates
nothing either. The kernel's requirement is "no allocation at insert", not
"stored pointer". The second half of the quote — forced compatibility with
existing C APIs — is a binding constraint for a kernel written in Whitefoot
only at the FFI boundary.

### What Rust-for-Linux had to build

| mechanism | why it exists | source |
|---|---|---|
| `Pin` / `pin-init` | "In the kernel many data structures are not allowed to change address, since there exist external pointers to them that would then be invalidated." The running example is `list_head`. | [The Safe Pinned Initialization Problem](https://rust-for-linux.com/the-safe-pinned-initialization-problem) |
| `ListArc` | A plain `Arc` cannot prove who may write `next`/`prev`. "The `ListArc` type can be thought of as a special reference to a refcounted object that owns the permission to manipulate the `next`/`prev` pointers stored in the refcounted object." | [rust/kernel/list/arc.rs](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/plain/rust/kernel/list/arc.rs) |
| `unsafe trait ListItem`, `HasListLinks`, `impl_has_list_links!` | the `container_of` round trip is not checkable; the macro statically proves only that the field path follows no pointer (`if false { let _: usize = ::core::mem::offset_of!(Self, $($field).*); }`) | [impl_list_item_mod.rs](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/plain/rust/kernel/list/impl_list_item_mod.rs) |
| `unsafe fn remove` | the type system cannot prove *which* list an element is in: "`item` must not be in a different linked list (with the same id)." | [rust/kernel/list.rs](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/plain/rust/kernel/list.rs) |
| `RBTreeNodeReservation` | the Rust rbtree did *not* go intrusive; it owns separately allocated nodes and therefore needs an explicit reservation type "when the insertion context does not allow sleeping, for example, when holding a spinlock" | [rust/kernel/rbtree.rs](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/plain/rust/kernel/rbtree.rs) |

LWN's framing of the same problem:

> "The kernel features a number of self-referential structures, such as doubly
> linked lists. Sharing these structures with Rust code poses a problem: moving
> a value that refers to itself (including indirectly) could cause the
> invariants of this kind of structure to be violated. For example, if a doubly
> linked list node is moved, node->prev->next will no longer refer to the right
> address. In C, programmers are expected to just not do that."
> — Daroc Alden, [How to write Rust in the kernel: part 3](https://lwn.net/Articles/1026694/), 2025-07-18

And the author of `kernel::list` on the cost side:

> "Linked lists are famously hard to implement in Rust given the cyclic nature
> of the pointers, and indeed, this implementation uses unsafe to get around
> that."
> "Linked lists aren't great for cache locality reasons, but it can be hard to
> avoid them for cases where you need data structures that don't allocate. ...
> The linked list is chosen over Vec in this case so that I don't have to worry
> about reducing the capacity of the vector."
> — Alice Ryhl, cover letter archived at [LWN](https://lwn.net/Articles/984613/), 2024-08-06

**(unverified)**: no statement was found anywhere in Rust-for-Linux material
that index/arena/slotmap data structures were *considered and rejected*. The
rationale given is always the positive one above.

### The kernel's own reference-free subsystems

Two production Linux mechanisms in the same tree are built with no stored
pointers at all, which bounds the claim that kernels require them.

| mechanism | reference-free form | source |
|---|---|---|
| io_uring SQ/CQ rings | shared-memory ring of **integer indices**; "While the CQ ring directly indexes the shared array of CQEs, the submission side has an indirection array between them. The submission side ring buffer is an index into this array, which in turn contains the index into the SQEs." | [io_uring(7)](https://man7.org/linux/man-pages/man7/io_uring.7.html) |
| DMA to devices | devices never see CPU pointers; drivers hand them a `dma_addr_t` handle. "The driver can use virtual address X to access the buffer, but the device itself cannot because DMA doesn't go through the CPU virtual memory system." | [dma-api-howto](https://docs.kernel.org/core-api/dma-api-howto.html) |
| Zircon (Fuchsia) kernel objects, for contrast outside Linux | "In user mode a handle is simply a specific number returned by some syscall" — a 32-bit index into the process's handle table, validated by the kernel on every syscall | [Zircon Handles](https://fuchsia.googlesource.com/fuchsia/+/refs/heads/main/docs/concepts/kernel/handles.md) |

### eBPF: the closest production precedent for the whole question

eBPF is a verified, in-kernel language with **no arbitrary stored pointers**.
Programs could originally only reach complex state through *maps* — keyed
tables, i.e. exactly "pool + handle". That model held for years and then hit a
wall; in 2022-2023 the kernel added owning/non-owning references with real
linked lists and rbtrees.

> "As the complexity of BPF programs grows, though, so does the demand for
> advanced data structures." ... when a node is allocated with `bpf_obj_new()`
> the program owns it; adding it to an rbtree transfers ownership; "If the
> program removes a node from the tree, it must, once again, take
> responsibility for disposing of it." ... "accessing the tree (with
> bpf_rbtree_first(), for example) can create 'non-owning' references that must
> all be invalidated when a node is freed."
> — Jonathan Corbet, [Red-black trees for BPF programs](https://lwn.net/Articles/924128/), 2023-02-27

The patch series' own motivation: allow programs to "allocate their own
objects, build their own object hierarchies, and use the basic building blocks
provided by BPF runtime to build their own data structures flexibly", with
packet queueing (linking `sk_buff`/XDP frames into custom structures) as the
immediate use case — i.e. a limitation of *predefined map schemas*, not of
indices as such ([Local kptrs, BPF linked lists](https://lwn.net/Articles/910840/)).
The roadmap in that series lists, verbatim, "Introduce bpf_refcount for local
kptrs, shared ownership" and "Introduce shared ownership linked lists"
([LWN](https://lwn.net/Articles/914833/)).

The kernel documentation for the result defines precisely the second-class
discipline Whitefoot is considering:

> An **owning reference** "controls the lifetime of the pointee" and
> "Ownership of pointee must be 'released' by passing it to some graph API
> kfunc, or via `bpf_obj_drop`." A **non-owning reference** "does not own the
> pointee", cannot add nodes or drop, and is valid only inside the critical
> section of the root's `bpf_spin_lock`.
> — [BPF graph data structures](https://docs.kernel.org/bpf/graph_ds_impl.html)

Read for the decision: BPF's non-owning reference *is* a second-class
projection with a lock-scoped extent. BPF did not conclude that stored
references were needed for addressing; it concluded it needed **ownership
transfer into a container plus a scoped borrow out of it**.

---

## 2. Compilers: index-linked arenas dominate new compilers

Four independently developed modern compilers store IR nodes in flat arrays
addressed by a **32-bit newtyped index**, and say why in their own sources.

| compiler | representation | stated reason |
|---|---|---|
| rustc | "Nobody ever references a basic block directly: instead, we pass around `BasicBlock` values, which are newtype'd indices into this vector."; `IndexVec<BasicBlock, BasicBlockData<'tcx>>`; `newtype_index!` "uses a u32" | allocation volume, cheap interned equality, and — for `HirId` — incremental compilation: ids force an observable dependency edge ([MIR](https://rustc-dev-guide.rust-lang.org/mir/index.html), [memory.html](https://rustc-dev-guide.rust-lang.org/memory.html), [HIR](https://rustc-dev-guide.rust-lang.org/hir.html), [newtype_index!](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_index/macro.newtype_index.html)) |
| Cranelift | `Value`, `Inst`, `Block` are `u32` newtypes indexing tables in `Function` | **the single most on-point quote in this survey**, below |
| Zig | `MultiArrayList` columns; `Zir.Index` / `Zoir.Index` are `enum(u32)`; AST `TokenIndex = u32` | "This allows for memory savings if the struct or union has padding, and also improves cache usage if only some fields or just tags are needed for a computation." ([multi_array_list.zig](https://raw.githubusercontent.com/ziglang/zig/master/lib/std/multi_array_list.zig), [Zir.zig](https://raw.githubusercontent.com/ziglang/zig/master/lib/std/zig/Zir.zig)) |
| Carbon | "Data is stored in vectors and flyweights are passed around, avoiding more typical heap allocation with pointers."; `Parse::Node` wraps `int32_t`; `IdBase`/`IndexBase` wrap `int32_t` | "Vectorization both minimizes memory allocation overhead and enables better read caching because adjacent entries will be cached together." ([toolchain/docs/README.md](https://raw.githubusercontent.com/carbon-language/carbon-lang/trunk/toolchain/docs/README.md), [idioms.md](https://raw.githubusercontent.com/carbon-language/carbon-lang/trunk/toolchain/docs/idioms.md)) |

Cranelift states all three reasons in one sentence:

> "Entity references in instruction operands are not implemented as Rust
> references both because Rust's ownership and mutability rules make it
> difficult, and because 64-bit pointers take up a lot of space, and we want a
> compact in-memory representation. Instead, entity references are structs
> wrapping a `u32` index into a table in the `Function` main data structure.
> There is a separate index type for each entity type, so we don't lose type
> safety."
> — [cranelift/codegen/src/ir/entities.rs](https://raw.githubusercontent.com/bytecodealliance/wasmtime/main/cranelift/codegen/src/ir/entities.rs)

and the crate doc adds the size argument:

> "Smaller indexes. The normal `usize` index is often 64 bits which is way too
> large for most purposes. The entity reference types can be smaller, allowing
> for more compact data structures."
> — [cranelift/entity/src/lib.rs](https://raw.githubusercontent.com/bytecodealliance/wasmtime/main/cranelift/entity/src/lib.rs)

### LLVM is the pointer-linked counterexample, and it has reasons

| item | what it stores | source |
|---|---|---|
| `llvm::Use` | `Value *Val; Use *Next; Use **Prev; User *Parent;` — one def-use edge costs four 64-bit pointers. "This is notionally a two-dimensional linked list." | [Use.h](https://raw.githubusercontent.com/llvm/llvm-project/main/llvm/include/llvm/IR/Use.h) |
| `llvm::Value` | "Every value has a 'use list' that keeps track of which other Values are using this Value."; `Use *UseList = nullptr` | [Value doxygen](https://llvm.org/doxygen/classllvm_1_1Value.html) |
| `llvm::ilist` | "ilist<T> implements an 'intrusive' doubly-linked list. It is intrusive, because it requires the element to store and provide access to the prev/next pointers for the list." Kept because "it can efficiently store polymorphic objects, the traits class is informed when an element is inserted or removed from the list, and ilists are guaranteed to support a constant-time splice operation." "These properties are exactly what we want for things like Instructions and basic blocks" | [LLVM Programmer's Manual](https://llvm.org/docs/ProgrammersManual.html) |

The LLVM position is the honest opposing case: **O(1) splice and identity
stable under arbitrary insertion/removal** are what the pointer form buys.
A flat index-linked list gives stable identity (append-only or generational)
and O(1) unlink, but splicing a *range* between two containers is not free.

### Reported costs of the index form, from the projects that pay them

| cost | evidence |
|---|---|
| raw integers lose type safety; every project bought a newtype layer back | rustc `IndexVec`: "trying to use the wrong index type (possibly accessing an invalid element) will fail at compile time"; Cranelift: "Improved type safety"; Carbon `IdBase`: "to provide a measure of type-checking"; matklad, [Newtype Index Pattern](https://matklad.github.io/2018/06/04/newtype-index-pattern.html): "if indexes are used for two types of objects, like `Foo` and `Bar`, you may end up with `thing: usize`" |
| debugging: an id is meaningless without its store | Carbon: "Since most data in the toolchain is referenced by id, this ends up being a very frequent task"; they built `Dump(<context>, <id>)`, a custom LLDB `dump` command, and `--sem-ir-crash-dump` ([debugging.md](https://raw.githubusercontent.com/carbon-language/carbon-lang/trunk/toolchain/docs/debugging.md)) |
| indirection traded against node size | Carbon SemIR: "This balances the size of `SemIR::Inst` against the overhead of indirection." ([check/README.md](https://raw.githubusercontent.com/carbon-language/carbon-lang/trunk/toolchain/docs/check/README.md)) |
| removal invalidates indices | petgraph: "Removing nodes or edges may shift other indices. Removing a node will force the last node to shift its index to take its place." ([petgraph::Graph](https://docs.rs/petgraph/latest/petgraph/graph/struct.Graph.html)) |
| a missing entry silently reads as a default | Cranelift `SecondaryMap`: "it does not keep track of which entities have been inserted. Instead, any unknown entities map to the default value." |
| the arena lifetime threads through everything | rustc: "The lifetime of that buffer is `'tcx`. Our types are tied to that lifetime" |

Niko Matsakis' 2015 post is the canonical Rust-side statement of the idiom and
its limit:

> "unlike an `Rc` pointer, an index alone is not enough to mutate the graph:
> you must use one of the `&mut self` methods" ... "the overall data structure
> is very compact. There is no need for a separate allocation for every node"
> ... "graphs implemented this way can easily be sent between threads and used
> in data-parallel code"
> — [Modeling graphs in Rust using vector indices](http://smallcultfollowing.com/babysteps/blog/2015/04/06/modeling-graphs-in-rust-using-vector-indices/)

**(unverified)** — explicitly not established: no controlled published
benchmark isolates "same compiler, pointers vs indices". Zig's often-quoted
RSS reduction compares bootstrap methods, not data-structure choice, and the
"35% faster" figure circulating for Zig's data-oriented rewrite comes from a
third-party summary, not a developer statement. Do not cite either as an A/B
measurement.

---

## 3. Browsers: the DOM needs relations, not necessarily stored links

### What the spec actually requires

The DOM Standard defines the tree by **relations derived from `parent` plus an
ordered set of `children`**, not by stored links:

> "An object that participates in a tree has a parent, which is either null or
> an object, and has children, which is an ordered set of objects."
> "The previous sibling of an object is its first preceding sibling or null if
> it has no preceding sibling."
> "The index of an object is its number of preceding siblings, or 0 if it has
> none."
> — [DOM Standard §1.1](https://dom.spec.whatwg.org/)

Event dispatch and removal both need the *upward* edge, but the spec phrases it
as an operation:

> "A node's **get the parent** algorithm, given an event, returns the node's
> assigned slot, if node is assigned; otherwise node's parent."
> "To remove a node node ... 1. Let parent be node's parent. 2. Assert: parent
> is non-null."
> — [DOM Standard](https://dom.spec.whatwg.org/)

### What engines store

| engine | stored per node | note |
|---|---|---|
| Blink | `TaggedParentOrShadowHostNode parent_or_shadow_host_node_;`, `Member<Node> previous_; Member<Node> next_;`, `Member<LayoutObject>`, `Member<NodeRareData>`; `ContainerNode` adds `Member<Node> first_child_` | **four links, not six.** `lastChild` is derived: "We do not store lastChild() explicitly; it is stored in `first_child_->previous_`" (circular prev list). Parent and `tree_scope_` are deliberately **uncompressed**: "Both tree_scope and parent are hot accessed members. Keep them uncompressed for performance reasons." ([node.h](https://raw.githubusercontent.com/chromium/chromium/main/third_party/blink/renderer/core/dom/node.h), [container_node.h](https://raw.githubusercontent.com/chromium/chromium/main/third_party/blink/renderer/core/dom/container_node.h)) |
| Servo | `parent_node`, `first_child`, `last_child`, `next_sibling`, `prev_sibling`, `owner_doc`, all `MutNullableDom<T>` | six traced GC references ([node.rs](https://raw.githubusercontent.com/servo/servo/main/components/script/dom/node/node.rs)) |
| Stylo `TNode` (shared by Servo and Firefox) | the six relations as **operations** on a `Copy` handle, saying nothing about representation | [style/dom.rs](https://raw.githubusercontent.com/servo/stylo/main/style/dom.rs) |
| `ego-tree` (index-based, behind `scraper`) | `parent: Option<NodeId>, prev_sibling, next_sibling, children: Option<(NodeId, NodeId)>` in a `Vec`; `NodeId(NonZeroUsize)` | "Node parent, next sibling, previous sibling, first child and last child can be accessed in constant time"; "All methods perform in constant time." Cost: "Nodes can be detached (orphaned) but **not removed**" — no reclamation. ([ego-tree](https://raw.githubusercontent.com/rust-scraper/ego-tree/master/src/lib.rs)) |
| `blitz-dom` (index-based rendering engine, Stylo + Taffy) | `parent: Option<NodeId>`, `children: ThinVec<NodeId>`, nodes in a `slotmap::SlotMap` | stale ids "no longer resolve ... instead of aliasing the new occupant". Cost: no sibling links, so `child_index()` is a linear scan of the sibling list, turning a spec-O(1) operation into O(siblings). |

**The HTML5 tree construction algorithm is already generic over the handle
representation.** `html5ever`'s `TreeSink` bounds a node reference by `Clone`
alone:

> "`Handle` is a reference to a DOM node. The tree builder requires that a
> `Handle` implements `Clone` to get another reference to the same node."
> `type Handle: Clone;`
> — [markup5ever tree_builder.rs](https://raw.githubusercontent.com/servo/html5ever/main/markup5ever/interface/tree_builder.rs)

and two shipping sinks instantiate it both ways: `markup5ever_rcdom` with
`type Handle = Rc<Node>`, and `scraper` with `type Handle = NodeId` — a plain
`Vec` index ([scraper tree_sink.rs](https://raw.githubusercontent.com/rust-scraper/scraper/master/scraper/src/html/tree_sink.rs)).

### What the browser engines say the hard part is — and it is not addressing

Both engines converged on a **tracing GC**, and the stated reason in both cases
is **cross-language cycles**, not tree navigation:

> "When the event fires, the handler adds a property on the `Event` which
> points back to the `Element`. We now have a cross-language reference cycle,
> with an `Element` pointing to an `Event` within C++, and an `Event` reflector
> pointing to the `Element` reflector in JavaScript. The C++ reference counting
> will never destroy a cycle, and the JavaScript garbage collector can't trace
> through the C++ pointers, so these objects will never be freed."
> — [JavaScript: Servo's only garbage collector](https://research.mozilla.org/2014/08/26/javascript-servos-only-garbage-collector/)

> "Blink, being forked from WebKit, originally used reference counting ...
> Reference counting is supposed to solve memory management issues but is known
> to be prone to memory leaks due to cycles. On top of this inherent problem,
> Blink also suffered from use-after-free issues as sometimes reference
> counting would be omitted for performance reasons."
> — [Oilpan library, V8 blog](https://v8.dev/blog/oilpan-library)

**(unverified / searched for and not found):** no statement by Servo or
Chromium that an index or arena DOM was considered and rejected. The published
argument is against *reference counting*. Manish Goregaokar, a Servo GC author,
names arenas as generally acceptable: "petgraph or an arena are often
acceptable solutions for this kind of pattern, but not always, especially if
your data is super heterogeneous"
([A tour of safe tracing GC designs in Rust](https://manishearth.github.io/blog/2021/04/05/a-tour-of-safe-tracing-gc-designs-in-rust/)).

### What each approach cost, with numbers

| approach | cost reported |
|---|---|
| Servo's GC + lints | a bespoke rustc driver (`crown`) with `register_tool` on nightly and two lints; and the team's own admission: "Ad-hoc extensions to a type system cannot easily guarantee soundness" and wrapping foreign pointers is "still a source of bugs and one of our largest areas of unsafe code" ([Experience Report, arXiv:1505.07383 §4.3, §4.7](https://ar5iv.labs.arxiv.org/html/1505.07383)) |
| Servo, positive result | "in the more than two years since Servo has been under development, we have encountered zero use-after-free memory bugs in safe Rust code" (same report) |
| Servo layout | "Because the script task's GC does not trace layout, node data cannot be safely stored in layout data structures" — the GC choice constrains a whole other subsystem ([OpaqueNode](https://raw.githubusercontent.com/servo/stylo/main/style_traits/dom.rs)) |
| Oilpan | every non-stack pointer must be a traced handle: "all pointers except on-stack pointers must be wrapped with Oilpan's handles"; `Persistent<T>` cycles still leak; "Persistents have a small unavoidable overhead because they require maintaining a list of all persistents"; `CrossThreadPersistent` assignment "requires a global lock" ([BlinkGCDesign.md @96](https://chromium.googlesource.com/chromium/src/+/refs/tags/96.0.4664.45/third_party/blink/renderer/platform/heap/BlinkGCDesign.md), [BlinkGCAPIReference.md](https://raw.githubusercontent.com/chromium/chromium/main/third_party/blink/renderer/platform/heap/BlinkGCAPIReference.md)) |
| MiraclePtr / `raw_ptr<T>` | "MiraclePtr increased the memory usage of the browser process 4.5-6.5% on Windows and 3.5-5% on Android"; "an increase of the main thread contention (~7%)" on Windows, ~6% on Android plus "First Input Delay (~1%), Input Delay (~3%) and First Contentful Paint (~0.5%)"; protects "~50% of use-after-free issues"; "MiraclePtr currently protects only class/struct pointer fields, to minimize the overhead" ([Use-after-freedom: MiraclePtr](https://security.googleblog.com/2022/09/use-after-freedom-miracleptr.html), 2022-09-13) |
| the problem being paid for | "around 70% of our high severity security bugs are memory unsafety problems (that is, mistakes with C/C++ pointers). Half of those are use-after-free bugs." (912 high/critical bugs since 2015) — [Chromium memory safety](https://www.chromium.org/Home/chromium-security/memory-safety/) |

The honest confound: **the one engine in this table that uses indices
(`blitz-dom`) is also the one with no JavaScript heap.** Whether an
index-addressed DOM survives contact with a scripting language that holds
arbitrary references to nodes *and is held by them* is not answered by any
source found.

---

## 4. Data-oriented and ECS practice: the reference-free idiom in production

### The mechanism, and who ships it

| system | entity/handle form | source |
|---|---|---|
| Bevy | `Entity` = index + generation. "Note that this is just an id, not the entity itself. Further, the entity this id refers to may no longer exist in the `World`." Freed: "The `Entity::generation` is bumped, which makes all existing `Entity` references with the previous generation 'invalid'." | [bevy_ecs::entity](https://docs.rs/bevy_ecs/latest/bevy_ecs/entity/index.html) |
| Unity DOTS | "An `Entity` struct refers to an entity, but isn't a reference. Rather, the `Entity` struct contains an `Index` that you can use to access entity data, and a `Version` that you can use to check whether the `Index` is still valid." | [Unity.Entities.Entity](https://docs.unity3d.com/Packages/com.unity.entities@1.3/api/Unity.Entities.Entity.html) |
| EnTT | "Entities are represented by entity identifiers. An entity identifier contains information about the entity itself and its version." | [EnTT wiki](https://github.com/skypjack/entt/wiki/Entity-Component-System) |
| slotmap | "once a key is removed, it stays removed, even if the physical storage inside the slotmap is reused for new elements." | [slotmap](https://docs.rs/slotmap/latest/slotmap/) |
| SoA storage | Unity: "All entities and components with the same archetype are stored in uniform blocks of memory called chunks ... A chunk contains an array for each component type." Bevy: "`Table` — A column-oriented structure-of-arrays based storage". | [archetypes](https://docs.unity3d.com/Packages/com.unity.entities@1.3/manual/concepts-archetypes.html), [bevy storage](https://docs.rs/bevy_ecs/latest/bevy_ecs/storage/index.html) |

The founding argument, Catherine West's RustConf 2018 keynote (canonical URL
is now [kyju.org](https://kyju.org/blog/rustconf-2018-keynote/); the
`kyren.github.io` URL is dead):

> "Generational indexes are never re-used because the generation will always
> increment, yet the 'real indexes' will always be 'small' ... This way, you
> can use fast indexing into a `Vec` without many of the bad 'pointer-like'
> properties of simple indexes!"
> "You can do a LOT with indexes into Vecs. This is way easier than self
> borrowing or `Rc<RefCell>`." ... "Self borrowing solutions like rental are
> tools of last resort. It is not worth it, move on with your life, and just
> use Vecs and indexes."

and the plain-index failure the generation fixes:

> "We can delete an entity which frees up a slot in a `Vec`, but then it's
> possible that the very next allocated entity will use the same index. ...
> We'll get a 'random other entity' in its place without being able to tell
> that it was in fact removed out from under us!"

### Where practitioners report it fails

| failure | what they actually do | source |
|---|---|---|
| **borrow two elements of one collection** | `split_at_mut`; the std docs carry the same "To work around this, we can use `split_at_mut`" paragraph three times (for `clone_from_slice`, `copy_from_slice`, `copy_within`). `get_disjoint_mut` checks disjointness **at runtime**: "This method does a O(n^2) check to check that there are no overlapping indices". Unchecked siblings exist because the check cannot be elided. | [std slice](https://doc.rust-lang.org/std/primitive.slice.html) |
| **mutation during iteration** | deferred command queues. Bevy: "Since each command requires exclusive access to the `World`, all queued commands are automatically applied in sequence when the `ApplyDeferred` system runs". Unity states the cost outright: "You can't make structural changes directly in a job because it might invalidate other jobs that are already scheduled, and creates a synchronization point"; "Structural changes to the data in ECS are the primary cause of sync points." | [Commands](https://docs.rs/bevy_ecs/latest/bevy_ecs/system/struct.Commands.html), [structural changes](https://docs.unity3d.com/Packages/com.unity.entities@1.3/manual/concepts-structural-changes.html) |
| **cross-structure links / hierarchies** | hand-maintained bidirectional invariants, then a language-level relationship feature. Bevy RFC 53: "This introduces state where both components on different entities are out of sync, and relies on the maintenance system to make it eventually consistent ... the subject of much frustration." Bevy 0.16's fix picks one side as authoritative: "We use this 'source of truth' model instead of allowing both components to 'drive' for performance reasons. Allowing writes to both sides would require expensive scanning during inserts." Still one-to-many only. | [RFC 53](https://raw.githubusercontent.com/bevyengine/rfcs/main/rfcs/53-consistent-hierarchy.md), [Bevy 0.16](https://bevy.org/news/bevy-0-16/), [Relationship](https://docs.rs/bevy_ecs/latest/bevy_ecs/relationship/trait.Relationship.html) |
| **specialized structures inside ECS** | do not put them there. flecs FAQ: "Things that ECS implementations are generally not good at are queries or operations that require highly specialized data structures, such as binary trees or spatial structures." | [flecs FAQ](https://www.flecs.dev/ecs-faq/) |
| **stale handle after free** | generation check — which is a runtime check that **can still alias**. Bevy: "Eventually, generations can (and do) wrap or alias. This can cause `Entity` and `EntityGeneration` values to be equal while still referring to different conceptual entities." slotmap: after 2³¹ deletions on one slot "the version wraps around and such a spurious reference could potentially occur." floooh: "it isn't waterproof because the same combination of array index and 'unique pattern' will be created sooner or later." | [EntityGeneration](https://docs.rs/bevy_ecs/latest/bevy_ecs/entity/struct.EntityGeneration.html), [slotmap](https://docs.rs/slotmap/latest/slotmap/), [floooh](https://floooh.github.io/2018/06/17/handles-vs-pointers.html) |
| **deferred reclamation for concurrent readers** | epochs. crossbeam-epoch: "When an element gets removed from a concurrent collection, it is inserted into a pile of garbage and marked with the current epoch." (Note: this is reclamation only; the crate docs never mention RCU — the analogy is folklore.) | [crossbeam-epoch](https://docs.rs/crossbeam-epoch/latest/crossbeam_epoch/) |
| **arena in scope everywhere** | pass the context, or give up. A user asking for a way out: "there's a huge syntax tax I'd like to avoid ... I can't think of a way to use an Arena without polluting all the interfaces in my code base ... Imagine if every function had to pass a 'malloc' function around". The answer: "`generational-arena` ... essentially, defers borrow analysis to runtime ... You lose static borrow checking AND static type checking." | [users.rust-lang.org thread](https://users.rust-lang.org/t/using-an-arena-without-passing-it-around-everywhere/41831) |
| **cross-arena index mixup** | brand the arena, which then cannot leave its scope. llogiq: "We could take an `Idx` returned from one arena and use it with another, resulting in either the wrong data or an out-of-bounds error." and of the branding fix, "Another downside is that our indices and arenas can no longer be `Send` nor `Sync`." | [Arenas vs. Indices](https://llogiq.github.io/2019/04/06/arena.html) |

### The idiom's own proponent, eight years later

Catherine West's 2026 retrospective is the strongest single statement of the
cost, because it comes from the person who popularized the idiom
([The Edge of Safe Rust](https://kyju.org/blog/tokioconf-2026/)):

> "Eventually you will figure out (or someone will tell you) that the proper
> way to solve this problem is by encoding your graph using a `Vec` with
> indexes. This is honestly a great solution, especially when the problem is
> otherwise simple! ... The only real change is having to pipe through a
> 'pointer context' of sorts (access to the underling storage vector) and the
> syntactic change from `node.field` to `nodes[node_idx].field`."
> "All of the bad qualities of our 'pointers' are extremely similar to problems
> that arise using real machine pointers."
> "If you accidentally free an index that still has a live reference somewhere,
> then allocate another value and it happens to get the same index, accessing
> the 'dead' index will both succeed and give you an effectively random, valid
> value of whatever your `T` type is. Though unlikely, this could even turn out
> to be as bad as something like Heartbleed."
> "Unfortunately, there is no simple solution to statically preventing errors
> (panics, whatever) when accessing deleted indexes."
> "the 'use Vecs and indexes' solution to (safe) Rust lacking more capable
> pointers is remarkably similar to just wholesale reinventing the pointer
> without the Rust enforced limitations. I'm not saying this is even a bad
> thing! ... but we need to be clear eyed about it."

and the one property indices buy that references cannot:

> "unlike a real reference, `usize` will never be `!Send` even if the referent
> isn't `Sync`, which makes the entire container able to be `Send` when any use
> of real references might not allow this! ... This one can be almost unsolvable
> in a safe way and force you into an index-based strategy for soundness."

Measured cost of the generation check itself, from a language that made it the
core safety mechanism: **+10.84% over unsafe** on the BenchmarkRL terrain
generator, versus +25.29% for reference counting — with the author's own
caveats that this is basic generational references with heap-only allocation
([Vale, Evan Ovadia](https://verdagon.dev/blog/generational-references)).
Vale's stated downsides: "It occasionally adds an 8-byte generation number to
the top of allocations" and "There will be `__check`s at run-time, unless the
programmer chooses to optimize them away."

---

## 5. Prior art: proof-carrying languages that banned stored references

This is the closest precedent to Whitefoot's actual question, and no other
section of this survey is as directly on point.

| system | what it did | outcome |
|---|---|---|
| **SPARK (Ada subset), ~1988-2019** | shipped for three decades in avionics and rail with **no access types at all**. "SPARK originally excluded pointers because absence of aliasing is a key assumption of the SPARK analysis, and removing it would induce so much additional annotation burden for users, that it would make the tool hardly usable." | real certified systems were written. The language restriction that survives: "Aliasing of names is not permitted." ([Using Pointers in SPARK](https://www.adacore.com/blog/using-pointers-in-spark), Claire Dross, 2019-06-06; [Language Restrictions](https://docs.adacore.com/spark2014-docs/html/ug/en/source/language_restrictions.html)) |
| **SPARK's reference-free doubly linked list** | the formally verified container is an **array of nodes with integer index links and a free list**: `type Node_Type is record Prev : Count_Type'Base; Next : Count_Type; Element : aliased Element_Type; end record;` and `type Cursor is record Node : Count_Type := 0; end record;` | **a production, formally verified, index-linked doubly linked list exists.** ([SPARKlib spark-containers-formal-doubly_linked_lists.ads](https://raw.githubusercontent.com/AdaCore/SPARKlib/master/src/spark-containers-formal-doubly_linked_lists.ads)) |
| **SPARK 2019+, ownership pointers** | added move/borrow/observe, Rust-inspired. Why: "Users particularly needed pointers to store indefinite types (like unconstrained arrays and strings) inside data structures, since indefinite types cannot be stored directly due to their unknown size at compile time." | the stated need was **variable-size storage**, not graphs. And even with pointers: "Pointer-based data structures can be defined in SPARK as long as they do not introduce cycles. For example, singly-linked lists and trees are supported whereas doubly-linked lists are not." ([Pointer-Based Data Structures in SPARK](https://www.adacore.com/blog/pointer-based-data-structures-in-spark), Claire Dross, 2019-10-08) |
| **eBPF** | verified in-kernel language, originally map-only (pool + key). Added owning/non-owning references with lists and rbtrees in 2022-2023. | see §1. The verifier's non-owning reference is a lock-scoped second-class projection. |
| **Hylo / mutable value semantics** | "In the purest form of mutable value semantics, references are second-class: they are only created implicitly, at function boundaries, and cannot be stored in variables or object fields." | research language; the doubly-linked-list result below is its strongest evidence. |

### The single most on-point publication

Dimi Racordon, **"Who Owns the Contents of a Doubly-Linked List?"**, OASIcs
Programming 2025, article 25, DOI 10.4230/OASIcs.Programming.2025.25
([Dagstuhl](https://drops.dagstuhl.de/entities/document/10.4230/OASIcs.Programming.2025.25),
PDF read and text-extracted in this survey).

Abstract, verbatim: "Despite their popularity, systems enforcing full ownership
guarantees such as Rust leave many users frustrated with the inability to
represent notionally self-referential data structures — e.g., doubly-linked
lists — using first-class references. ... In this paper, we take a look at the
way value-oriented languages address this issue and study representations of
arbitrary graph-like data structures without references."

Its method and results, verbatim from the PDF:

> "we argue that the percieved limitations of full ownership are due to
> unfamiliarity rather than lack of expressiveness."
> "a reasonable way to emulate pointers is to use indices into an array. From
> there, one can implement a doubly-linked list by replacing pointers with
> indices. ... The twist here is that these references are represented as
> offsets into the internal array rather than actual references, thereby
> satisfying unique ownership requirements."
> "The addresses returned by insert are indeed stable — i.e., they are not
> invalidated by removes or other insertions, even if the internal array must
> be reallocated."
> "the list itself is essentially an allocator for its own nodes. A direct
> consequence is that one can apply well-known approaches to more cleverly
> (re-)use the internal array and avoid leavage after the removal of an
> element. For example, one could keep a free list identifying the positions of
> the nodes that have been removed."
> "Who owns the contents of a doubly-linked list? The list itself of course!
> Crucially, a node owns neither its successor nor its predecessor. Instead,
> the nodes forming a list are all parts of that whole while links between
> these nodes are peer relationships that can be represented independently."

Measured, in safe Swift, on an Apple M1:

> Graphs (Tarjan SCC, Dijkstra): "The results show no noticeable difference
> between the two graph implementations, suggesting that representing peer
> relationship separately, without first-class references, does not negatively
> impact performance."
> Doubly-linked list (append, traverse, random removal, traverse-after-removal):
> "The results show that the value-based approach **significantly outperforms
> its reference-based counterpart, by one order of magnitude.** The biggest
> contributor of this difference is cache locality."

Stated cost: insertion is "in amortized constant time rather than O(1), due to
the cost of reallocating the internal array", and the graph case used string
keys — "A more careful choice of identities — e.g., using integers to avoid the
cost of hashing character strings — would likely improve on performance at the
cost of some convenience and ease of implementation."

### The known limit of second-class references

Fernando Borretti, [Second-Class References](https://borretti.me/article/second-class-references)
(2023-06-12), states the model exactly as Whitefoot proposes it — "Can't be
returned from functions. Can't be stored in data structures. Can only be
created at function call sites, as a special parameter-passing mode." — with
the benefit "because references can only be created at function calls, cannot
be stored (leaked) anywhere, and cannot be returned, borrow checking becomes
trivial" and "lifetimes become redundant: you no longer need lifetime
annotations *anywhere*". His named limitation is not graphs:

> "The main pain point is iterators. In Rust, iterators are (far and above) the
> main place where you find yourself storing a reference in a struct."

with coroutines or typed indices as the alternatives. In his later survey he
repeats it: "Not being able to store references in data structures makes it
hard to implement iterators."
([Type Systems for Memory Safety](https://borretti.me/article/type-systems-memory-safety), 2023-07-22)

---

## 6. Existing Whitefoot doctrine, for comparison

`why-whitefoot.md` §10 "Handles and copies instead of references"
(`/private/tmp/whitefoot-access-effects-research/docs/why-whitefoot.md`),
verbatim:

> "*Current status: append-only index-linked SoA has a seeded pattern and
> current witness. Recyclable generational pools, stale-handle checks, and
> check-elision schemes below are proposed or historical, not current
> language/compiler capability.*"
> "Node links in a tree or graph are handles into the pool, not pointers or
> references."
> "**The writer's burden:** most code holds no loans at all, so the borrow
> rules bite only at the few sites that point into something. The
> self-referential-struct wall that pushes real Rust projects through `Pin`,
> `unsafe`, or index-arena workarounds does not exist, because structs store
> values, not borrows. The problematic program is not painful to write; it is
> impossible to state."
> "**Safety of stale handles:** a future recyclable pool would pair each slot
> with a generation and expose mismatch as a typed lookup outcome. ... There is
> no dedicated hidden handle trap. Check-free schemes (loans that freeze reuse
> for a scope, affine owned handles, proof-discharged repeat checks) remain an
> active research track."
> "In Rust a stale unchecked index may silently select a reused slot. The
> proposed Whitefoot generational form would close that hole; the current
> append-only pattern avoids reuse instead and makes no generational-pool
> claim."

`design/language/data-model.md` (read-only; the live tree) already rules on
the recycling question:

> "Decision: Append-only stable identity is its own contract that pays no
> generation or recycling cost, and a recyclable stable identity, if the
> language ever admits one, is a separate contract rather than a cost every
> pool pays, because **a well-typed slot-recycling use-after-free is
> unrepresentable when indices never recycle** and access is then a bare bounds
> check, instead of one pool contract that pays for recycling everywhere."
> "Decision: Any operation that moves owners between slots, such as a
> compaction or a removal shift, is an ownership relocation in which every
> retained owner reaches exactly one destination and a moved-from slot is never
> read or dropped."
> "Decision: Struct-of-arrays is the taught default layout for bulk data while
> array-of-structs stays a genuine need ..."

Two further live-tree facts bear on rows below:

- `design/language/dispatch.md`: "The kernel has no function values and no
  dynamic dispatch, exhaustive match over closed sum types being its only
  dispatch" — so a wait-queue-style stored **callback pointer** is already out
  of the model independently of this decision.
- `spec/kernel-spec.md` [STOR-5] already enforces borrow-free storage: an owned
  aggregate cannot contain a `Slice`/`MutSlice`, so "structs store values, not
  borrows" is current spec, not aspiration.

`docs/patterns.md` records the pattern and, importantly, the open gaps:

> "P2. Struct-of-arrays pool (append-only, index-linked) ... a node is a `u64`
> index; indices never recycle; the whole pool drops at once. ... Replaces:
> `Rc<RefCell<Node>>` graphs, pointer-linked heap nodes, and Rust's Vec-index
> arena WITH free-lists (STOR-1 rejects recycling: stale indices are well-typed
> UAF)."
> "Known gaps (findings, not yet patterns): In-place mutation interleaved with
> traversal of the same structure (graph rewriting while walking). Restructure
> via P1/P2 or reject (OWN-8 posture) ... Long-lived borrows stored in data
> (self-referential structs): structurally unrepresentable in v0 (structs store
> values, not borrows); the index pool (P2) is the blessed encoding."

That known gap — **mutation interleaved with traversal** — is precisely the
failure mode §4 documents in production ECS, where the answer is a deferred
command queue, which is also Whitefoot's P1. The doctrine and the external
evidence agree on the mechanism; what the doctrine lacks is the free-list /
generational story that every production reference-free system eventually
needed.

---

# ASSESSMENT

*This section is the surveyor's judgement over the evidence above, clearly
separated from it. Every claim of fact points back to a cited row.*

## A. Needs with a production-proven reference-free form

| need | proof | strength |
|---|---|---|
| **arena IR / AST** | four independently developed compilers — rustc, Cranelift, Zig, Carbon — chose u32-indexed flat arrays and state the reasons in their own sources. This is not a workaround; it is the dominant shape in new compiler design. | **strongest**. No hedge needed. |
| **general graph** | petgraph is the Rust idiom; the same four compilers do it for IR graphs; and Racordon 2025 measured Tarjan SCC and Dijkstra on a reference-free Swift graph with "no noticeable difference". | **strong**, with measurement. |
| **doubly linked list** | SPARK ships a *formally verified* index-linked doubly linked list with a free list, in an ecosystem used for certified avionics; Racordon measured an index-linked list at **one order of magnitude faster** than the reference version. | **strong**. Note this is the exact structure Rust-for-Linux needed `unsafe` and `Pin` for. |
| **iterator/traversal with mutation** | deferred command queues are the universal answer in production ECS (Bevy, Unity) and are already Whitefoot's P1. | **strong**, but see cost below. |
| **resource / device identity** | Zircon handles, POSIX fds, `dma_addr_t`. Integer-handle resource identity is the norm, not an experiment. | **strong**. |
| **persistent structures** | every on-disk b-tree in existence, SQLite included. | trivially strong; included because it shows index-linking is the *default* wherever pointers were never available. |
| **queue / ring between mutually distrusting parties** | io_uring's SQ/CQ are pure index rings in shared memory. | **strong**. |
| **tree with parent links, outside a scripting heap** | ego-tree reaches all five DOM relations in O(1) with indices; blitz-dom runs the real HTML5 tree-construction algorithm plus Stylo cascade on `NodeId`s. | **moderate**: real software, not browser-scale. |

## B. Needs with NO production-proven reference-free form

| need | what is missing | how load-bearing |
|---|---|---|
| **intrusive list node removable with only the element in hand** | `list_del(entry)` needs no list handle and no allocation. Every index form needs the owning container in scope to unlink. No production system was found that does kernel-style "remove me from whatever list I am on" without a stored pointer. | **Real, and it is a shape constraint, not a capability gap.** The index form can always unlink if the container is a parameter — which is exactly what the P1 command-buffer architecture already forces. What is lost is the *ambient* removal available at arbitrary call depth. |
| **stack-allocated node linked into someone else's list** (wait-queue `DEFINE_WAIT`) | the entry lives in the waiter's frame and is linked into a head the waiter does not own. A pool-allocated waiter record is a different lifetime story. | **Real.** Not fatal: the waiter can own a pool slot instead of a stack entry, at the cost of a pool. No production reference-free precedent found. |
| **DOM under a scripting heap** | the unsolved problem is the *cross-language cycle*, and it is about lifetime, not addressing. Both Servo and Blink converged on tracing GC for exactly this. The one index-based engine has no JS heap. | **Real and unresolved by this survey.** An index pool does not reclaim; a DOM that never reclaims leaks a browsing session. Any reference-free DOM needs a reachability story over handles, which is a tracing collector over indices rather than pointers. |
| **RCU-style publish of a link read concurrently under no lock, inside a kernel** | epochs solve reclamation; no production index-based in-kernel RCU publish was found. | **Probably not real.** Nothing in RCU depends on the published value being an address rather than an index; the ordering requirements are identical. Absence of precedent, not evidence of impossibility. Marked here to keep the ledger honest. |
| **slot reuse with a statically proved stale-handle impossibility** | every production generational scheme checks at runtime and **can still alias**: Bevy's generations "can (and do) wrap or alias"; slotmap wraps at 2³¹; floooh's tag "isn't waterproof". kyren, the idiom's own advocate: "there is no simple solution to statically preventing errors ... when accessing deleted indexes." | **This is the sharpest finding for Whitefoot.** No production system has what `data-model.md` would need if it ever admits recycling. Whitefoot's current answer — append-only, never recycle, so a well-typed use-after-free is unrepresentable — is *stronger* than every production alternative, and its cost is that memory is never reclaimed within a pool. |
| **O(1) splice of a sublist between containers** | LLVM keeps intrusive lists partly for this: "ilists are guaranteed to support a constant-time splice operation". An index-linked list can splice in O(1) only within one pool. | **Real but narrow.** Affects IR block/instruction reordering; cross-pool splice becomes a copy. |

## C. What the evidence does *not* say

Three claims that would be convenient and are **not supported**:

1. **No source found says arenas/indices were considered and rejected** by
   Rust-for-Linux, Servo, or Chromium. Their published arguments are positive
   arguments for what they chose (no allocation in atomic context; cycles
   across the JS boundary), never a rejection of the index form. Do not cite
   any of them as "the experts tried indices and they failed".
2. **No controlled A/B benchmark exists** isolating pointers vs indices inside
   one compiler. The Zig memory numbers compare bootstrap methods; the "35%"
   figure is a third-party summary. The only clean measurement in this survey
   is Racordon 2025 (Swift, M1, one order of magnitude for the linked list, no
   difference for graphs) — and it is a short paper on two case studies.
3. **The reference-free form is not automatically safer.** Its own proponents
   say the opposite about the *unchecked* version. kyren 2026: a stale index
   "will both succeed and give you an effectively random, valid value ... this
   could even turn out to be as bad as something like Heartbleed." The safety
   comes from the generation check or from never recycling — not from using an
   integer.

## D. The load-bearing conclusion

**The evidence supports the reference-free core model for kernels and
compilers, and leaves the browser DOM open.**

The decisive reframing comes from the two best sources. Rust-for-Linux says
intrusive lists exist because **you cannot allocate in atomic context** — a
requirement a fixed-capacity pool meets equally well. Racordon says the
frustration with full ownership is "due to unfamiliarity rather than lack of
expressiveness", and demonstrates the doubly linked list and the general graph
in safe Swift, faster. Neither is an argument that a stored address is
necessary; both are arguments about allocation, ownership, and habit.

What is genuinely unresolved is **lifetime, not addressing**. Three of the six
"no production form" rows (DOM under a JS heap, slot reuse, stack-allocated
waiters) are all the same question: when is a pool slot dead? Pointers do not
answer that question either — that is why browsers have a GC and Chromium pays
5% of browser memory for MiraclePtr. Whitefoot's append-only answer is a real
answer, and its cost is stated: pools grow monotonically. The decision the
owner faces is therefore not "references or handles" but **"what reclaims a
pool slot, and what proves the reclamation safe"**.

One asymmetry favours the reference-free model beyond expressiveness, and it is
the one a proof-carrying language should want most. kyren: "unlike a real
reference, `usize` will never be `!Send` even if the referent isn't `Sync`,
which makes the entire container able to be `Send` when any use of real
references might not allow this! ... This one can be almost unsolvable in a
safe way and force you into an index-based strategy for soundness." Whitefoot's
whole parallel-independence story rests on the same property.

## E. What an AI writer would have to be taught

Ordered by how often the evidence shows a human getting it wrong.

| # | lesson | evidence it is needed |
|---|---|---|
| 1 | **A link is a value, and the container is a parameter.** `node.next` becomes `pool.nodes[i].next`; any function that follows a link takes the pool. kyren names this the entire cost: "The only real change is having to pipe through a 'pointer context' of sorts ... and the syntactic change from `node.field` to `nodes[node_idx].field`." | the users.rust-lang.org thread is a working programmer failing at exactly this: "I can't think of a way to use an Arena without polluting all the interfaces in my code base ... Imagine if every function had to pass a 'malloc' function around." |
| 2 | **Handles are typed, and mixing two pools is the new type error.** Every production system independently re-invented a newtype layer: rustc `newtype_index!`, Cranelift `entity_impl!`, Carbon `IdBase`, floooh's per-type handle. | llogiq: "We could take an `Idx` returned from one arena and use it with another, resulting in either the wrong data or an out-of-bounds error." matklad: "you may end up with `thing: usize`." |
| 3 | **Deep code returns write intents; one shallow owner applies them.** This is not a style preference, it is how every production ECS handles mutation during traversal, and it is what makes parallel iteration sound. | Unity: structural changes "are the primary cause of sync points"; Bevy `ApplyDeferred`. Whitefoot P1 already teaches it. |
| 4 | **Peer links are not ownership.** The container owns its nodes; a link is a peer relationship. This is the single reframing that makes a doubly linked list expressible. | Racordon: "a node owns neither its successor nor its predecessor. Instead, the nodes forming a list are all parts of that whole while links between these nodes are peer relationships." |
| 5 | **Bidirectional links need one authoritative side.** Writers will try to maintain both directions and produce transient inconsistency. | Bevy RFC 53: "relies on the maintenance system to make it eventually consistent ... the subject of much frustration". Bevy 0.16's fix: "The `Relationship` component is the 'source of truth' ... Allowing writes to both sides would require expensive scanning during inserts." |
| 6 | **Do not reach for a linked list at all.** The reference-free idiom's own culture says so: "99% of the time you should just use a Vec (array stack)." Blink already derives `lastChild` rather than storing it; the spec defines sibling relations as *derived*. Teach "which relations do you need in O(1)?" before "which links do you store?" | [too-many-lists](https://rust-unofficial.github.io/too-many-lists/); Blink `container_node.h`; DOM Standard §1.1 |
| 7 | **A stale handle is a logic error, and the language must make it a checked outcome or an impossibility.** An AI writer coming from Rust folklore will write a free list and reuse slots, which in an append-only model is the one thing that reintroduces the hazard. | kyren 2018: "We'll get a 'random other entity' in its place without being able to tell that it was in fact removed out from under us!" `data-model.md` already rules on this; the writer must be told *why*. |
| 8 | **Two indices are not known to differ.** Swap, partition and heap code must discharge a disjointness obligation, not assume it. | Rust's escape hatch is a runtime O(n²) check; Whitefoot's LIV-2 proves it statically. This is a Whitefoot-specific advantage the writer has to be taught to use. |
| 9 | **The pool is an allocator, and its growth and reclamation policy is a design decision made once, up front.** | Racordon: "the list itself is essentially an allocator for its own nodes"; TigerBeetle's static-allocation discipline ("TigerBeetle allocates no memory after startup") is the shipped version of this habit. |

## Citation index for the three strongest sources

1. **Dimi Racordon, "Who Owns the Contents of a Doubly-Linked List?", OASIcs
   Programming 2025, DOI 10.4230/OASIcs.Programming.2025.25** — peer-reviewed,
   implements a doubly linked list and arbitrary graphs *without first-class
   references* in safe Swift, and measures the index form at one order of
   magnitude faster for the list and at parity for graph algorithms. Directly
   answers the owner's question.
2. **Rust for Linux, "Arc in the Linux kernel"** — states, in the kernel
   community's own words, *why* intrusive lists exist: no allocation is
   permitted in atomic context, and existing C APIs impose the pattern. Reframes
   the requirement from "stored pointer" to "no allocation at insert".
3. **Catherine West, "The Edge of Safe Rust" (2026)** — the index idiom's
   originator, eight years on, stating both its ceiling ("remarkably similar to
   just wholesale reinventing the pointer"; "there is no simple solution to
   statically preventing errors ... when accessing deleted indexes") and the one
   property it uniquely buys (`Send`-ness across a container whose contents are
   not `Sync`).

Runner-up, and the best evidence that a *proof-carrying* language can live
without stored references: **SPARK** — thirty years of certified systems with
no access types at all, a formally verified index-linked doubly linked list in
its own container library, and, when it finally added ownership pointers, a
model that still cannot express a doubly linked list.



# File: spec-token-census.md

# Spec token census: where the taught surface spends its budget

Measured 2026-09-16 against the worktree `/private/tmp/whitefoot-access-effects-research`
(branch `research/access-effects`), read-only. Sources measured:

- `spec/kernel-spec.md` — ACTIVE v0.57, 533,492 bytes / 80,763 words / 3,595 lines.
- `docs/patterns.md` — 89,380 bytes / 14,125 words / 1,795 lines.
- `research/experiments/blind-writer/2026-08-28/programs/*.wf` — the re-declared prelude.

Method. The specification was split at every line-initial bracketed rule id
outside a fenced block (`^\[FAM-N\]`), the text from one id to the next being that
rule's text; a heading run immediately preceding a rule id (with its blank lines)
was attributed to the rule it precedes, per the census request. 129 rules split
this way; the 10 `[ENT-3.Sn]` sub-step labels are proof steps inside ENT-3, not
separate rules, and stay inside ENT-3. The 328 bytes before the first rule id are
reported as FRONTMATTER. Per-rule byte and word sums reconcile exactly with `wc`
on the whole file (533,492 / 80,763).

Two token estimates are reported throughout: **TOK_B4 = bytes / 4** and
**TOK_W13 = words x 1.3**. **Totals in this document use TOK_B4.** It is the
larger of the two (133,373 vs 104,992 for the specification), it reproduces the
"about 130k tokens" figure the requirement discussion already uses, and for dense
technical prose with heavy punctuation, backticked identifiers and EBNF fences a
BPE tokenizer runs closer to 4 bytes/token than to 0.77 words/token. Read TOK_W13
as the optimistic bound. No tokenizer was run; owner decision **O6** in
`VERDICT-CORE.md` is precisely the open question of which tokenizer fixes K.

Evidence files beside this one: `spec-rules.tsv` (per-rule spans and counts),
`spec-density.tsv` (per-rule borrow/region vocabulary density),
`cluster-marked.tsv`, `patterns-cards.tsv`, `patterns-terms.tsv`, and the awk
scripts that produced them.

---

## 1. Specification by rule family, sorted by tokens

| Family | Rules | Bytes | Words | TOK_B4 | TOK_W13 | % of spec | % of K=48k |
|---|---:|---:|---:|---:|---:|---:|---:|
| ENT | 6 | 75275 | 11604 | **18819** | 15085 | 14.1% | 39.2% |
| DIAG | 2 | 53086 | 7369 | **13272** | 9580 | 10.0% | 27.6% |
| FN | 9 | 47745 | 7001 | **11936** | 9101 | 8.9% | 24.9% |
| MSR | 6 | 36197 | 5636 | **9049** | 7327 | 6.8% | 18.9% |
| BLK | 5 | 29594 | 4421 | **7398** | 5747 | 5.5% | 15.4% |
| OP | 9 | 28468 | 4428 | **7117** | 5756 | 5.3% | 14.8% |
| TYPE | 7 | 27415 | 4109 | **6854** | 5342 | 5.1% | 14.3% |
| OWN | 14 | 27021 | 4164 | **6755** | 5413 | 5.1% | 14.1% |
| FORM | 8 | 24369 | 3813 | **6092** | 4957 | 4.6% | 12.7% |
| PROV | 2 | 19783 | 3302 | **4946** | 4293 | 3.7% | 10.3% |
| PAR | 2 | 19680 | 3044 | **4920** | 3957 | 3.7% | 10.2% |
| GRAM | 11 | 18971 | 2836 | **4743** | 3687 | 3.6% | 9.9% |
| STOR | 6 | 17492 | 2556 | **4373** | 3323 | 3.3% | 9.1% |
| CALL | 6 | 13965 | 2242 | **3491** | 2915 | 2.6% | 7.3% |
| PRE | 1 | 11818 | 1494 | **2954** | 1942 | 2.2% | 6.2% |
| EFF | 4 | 11568 | 1694 | **2892** | 2202 | 2.2% | 6.0% |
| VIEW | 4 | 11089 | 1792 | **2772** | 2330 | 2.1% | 5.8% |
| INV | 1 | 10491 | 1632 | **2623** | 2122 | 2.0% | 5.5% |
| SET | 2 | 9124 | 1418 | **2281** | 1843 | 1.7% | 4.8% |
| PRF | 1 | 9075 | 1426 | **2269** | 1854 | 1.7% | 4.7% |
| LIV | 2 | 8969 | 1473 | **2242** | 1915 | 1.7% | 4.7% |
| CONST | 2 | 5638 | 864 | **1410** | 1123 | 1.1% | 2.9% |
| GIVE | 1 | 3990 | 632 | **998** | 822 | 0.7% | 2.1% |
| ERR | 4 | 3144 | 457 | **786** | 594 | 0.6% | 1.6% |
| SCOPE | 3 | 2817 | 380 | **704** | 494 | 0.5% | 1.5% |
| PROG | 3 | 2514 | 373 | **628** | 485 | 0.5% | 1.3% |
| META | 5 | 1466 | 208 | **366** | 270 | 0.3% | 0.8% |
| EX | 1 | 1007 | 158 | **252** | 205 | 0.2% | 0.5% |
| LEX | 1 | 779 | 103 | **195** | 134 | 0.1% | 0.4% |
| CAP | 1 | 614 | 90 | **154** | 117 | 0.1% | 0.3% |
| FRONTMATTER | 1 | 328 | 44 | **82** | 57 | 0.1% | 0.2% |
| **TOTAL** | **130** | **533492** | **80763** | **133373** | **104992** | **100.0%** | **278%** |

Reading of the family table. Nothing in the specification is small. The three
largest families — `ENT` (entailment), `DIAG` (diagnostics) and `FN` (functions
and contracts) — are 44,027 tokens between them, 92% of the whole budget K, and
none of them is ownership. The ownership cluster is spread across `OWN`, `PROV`,
`PAR`, `VIEW`, `LIV`, `EFF`, `CAP` and parts of `STOR`, `FORM`, `TYPE`, `BLK`,
`SET`, `MSR`, `CALL` and `ENT`, so the family table alone understates it;
section 2 measures the cluster directly.

---

## 2. The ownership cluster, rule by rule, with the retire marks

Marks are against `research/investigations/access-effects/VERDICT-CORE.md`
section 1.3, whose signature paragraph says a signature's vocabulary is "three
conventions (`let`, `inout`, `sink`); types; value refinements with `old()`;
outcome types; an `origin` set on a yielded projection; the pool identity
parameter on `Pool` and `Handle`; the plane map at the type declaration" and
whose **Absent** list is "region parameters, lifetime parameters, entry/exit
storage states, loan clauses, memory effect rows, modes."

- **R — retired.** The rule's whole subject is one of the absent constructs.
- **P — partly retired.** The judgment survives but is restated over resolved
  paths, conventions and footprints; the borrow/region apparatus inside it goes.
- **K — kept.** The rule carries no reference machinery and survives essentially
  as written.

`A%` is a measured upper bound on the borrow/region/loan/view text inside each
rule: the share of the rule's bytes sitting on lines that mention `region`,
`borrow`, `reborrow`, `loan`, `outliv`, `&uniq`, `Slice`/`MutSlice`, `arena`,
`Heap`/`Arena`, `lifetime`, or a REGIONID. Lines in this specification are
sentence-sized (171 bytes average), so `A%` is a sentence-granularity mention
density, not a proof that every one of those bytes disappears.

| Rule | Mark | Bytes | Words | TOK_B4 | TOK_W13 | A% | Absent construct | What happens to it |
|---|:--:|---:|---:|---:|---:|---:|---|---|
| OWN-1 | **P** | 2740 | 455 | **685** | 592 | 32.2% | borrow modes | owner + copy/affine classification kept (BIND, sink); its borrow/view/arena members go |
| OWN-2 | **R** | 247 | 39 | **62** | 51 | 85.4% | borrow modes; region parameters | the mode vocabulary own / & / &uniq and the region on a mode |
| OWN-3 | **R** | 1090 | 161 | **272** | 209 | 99.9% | region parameters; lifetime parameters | lexical regions, region_stmt, region_params |
| OWN-4 | **R** | 385 | 64 | **96** | 83 | 99.7% | lifetime parameters; loan clauses | loan liveness and the outlives-or-equals test |
| OWN-5 | **P** | 6190 | 959 | **1548** | 1247 | 90.0% | borrow modes; loan clauses | exclusivity survives as the four-cell conflict table over resolved paths; holder suspension and the four reborrow flavours go |
| OWN-6 | **R** | 5489 | 807 | **1372** | 1049 | 96.2% | loan clauses | holder, resolution, statement-scoped child reborrow |
| OWN-7 | **K** | 2262 | 337 | **566** | 438 | 38.3% | - | prefix overlap + range disjointness = PATH / EXT, carried over |
| OWN-8 | **K** | 201 | 28 | **50** | 36 | 0.0% | - | reject-when-unsure |
| OWN-9 | **R** | 577 | 83 | **144** | 108 | 99.8% | borrow modes | the &uniq-unaliased optimizer consequence, replaced by the LLVM mapping table |
| OWN-10 | **R** | 544 | 86 | **136** | 112 | 99.8% | lifetime parameters | borrow-storage duration |
| OWN-11 | **R** | 2145 | 366 | **536** | 476 | 93.3% | region parameters | loop body is a region block |
| OWN-12 | **R** | 861 | 128 | **215** | 166 | 99.9% | region parameters; memory effect rows | call-site region substitution, argument-borrow overlap, effect-row overlap |
| OWN-13 | **P** | 2022 | 310 | **506** | 403 | 86.4% | borrow modes | match consumption kept (BIND); the derived borrow-mode binders go |
| OWN-14 | **R** | 2268 | 341 | **567** | 443 | 100.0% | loan clauses | non-argument reborrow disposition and the returned reborrow |
| LIV-1 | **K** | 1836 | 298 | **459** | 387 | 36.3% | - | join-checked liveness = BIND joined by conjunction |
| LIV-2 | **P** | 7133 | 1175 | **1783** | 1528 | 77.8% | loan clauses | the one-commit rule is kept; its loan and borrow clauses go |
| PROV-1 | **P** | 4106 | 668 | **1026** | 868 | 98.3% | region parameters | store identity becomes the pool identity parameter P; the region spelling, brand elision and outlives rules go |
| PROV-6 | **P** | 15677 | 2634 | **3919** | 3424 | 75.4% | region parameters | linearity / reclamation kept (affine release at sink); region-bearing and capability clauses restated |
| VIEW-1 | **R** | 1872 | 324 | **468** | 421 | 74.9% | loan clauses | Slice / MutSlice and two loan strengths |
| VIEW-2 | **R** | 6720 | 1052 | **1680** | 1368 | 75.0% | loan clauses | view formation and the loan the formed value holds |
| VIEW-4 | **R** | 966 | 165 | **242** | 214 | 93.2% | loan clauses | a commit may not displace a live loan |
| VIEW-6 | **R** | 1531 | 251 | **383** | 326 | 99.9% | region parameters; loan clauses | view-result origin and two results may not share a region |
| STOR-5 | **R** | 3797 | 580 | **949** | 754 | 99.9% | region parameters | region-bearing storage and the store brand |
| EFF-1 | **R** | 4843 | 728 | **1211** | 946 | 49.3% | memory effect rows | the reads / writes / allocates row grammar |
| EFF-2 | **R** | 5541 | 804 | **1385** | 1045 | 75.8% | memory effect rows | the exhibited-effect union over the body |
| EFF-3 | **R** | 798 | 112 | **200** | 146 | 0.0% | memory effect rows | the pure row and what it licenses |
| EFF-4 | **K** | 386 | 50 | **96** | 65 | 0.0% | - | no writer-reachable abort effect or runtime fallback |
| CAP-1 | **P** | 614 | 90 | **154** | 117 | 42.3% | borrow modes; memory effect rows | the no-capability statement is kept; its vocabulary list is restated over conventions and footprints |
| PAR-1 | **P** | 7177 | 1121 | **1794** | 1457 | 65.1% | memory effect rows; borrow modes | statement-pair permission is kept, restated over footprints and PATH / EXT |
| PAR-2 | **P** | 12503 | 1923 | **3126** | 2500 | 75.3% | memory effect rows; borrow modes | loop-iteration permission is kept, restated over footprints, EXT and ProvedRangePartition |
| CALL-6 | **K** | 4327 | 683 | **1082** | 888 | 54.1% | - | the publication route for a declared relation |
| ENT-5 | **P** | 19546 | 3008 | **4886** | 3910 | 41.4% | loan clauses; memory effect rows | support and death of a fact; the frame rule replaces call-driven killing and the borrow/box/arena support clause goes |

### Subtotals

| Group | Rules | Bytes | TOK_B4 | TOK_W13 | % of cluster | % of K=48k |
|---|---:|---:|---:|---:|---:|---:|
| Retired outright (R) | 17 | 39674 | **9918** | 7918 | 31.4% | 20.7% |
| Partly retired (P) | 10 | 77708 | **19427** | 16046 | 61.5% | 40.5% |
| Kept (K) | 5 | 9012 | **2253** | 1815 | 7.1% | 4.7% |
| **Cluster total** | **32** | **126394** | **31598** | **25779** | **100.0%** | **65.8%** |
| _of which measured apparatus inside the P rules_ | - | 52045 | _13011_ | - | 41.2% | 27.1% |

**Retirable subtotal.** Lower bound (R only) **9918 TOK_B4**; mid estimate (R + the measured apparatus share of the P rules) **22930 TOK_B4**; upper bound (R + all of P) **29346 TOK_B4**. Kept: **2253 TOK_B4**.

### Region and borrow text outside the named cluster

The 32 cluster rules are 126,394 bytes (31,598 TOK_B4, 23.7% of the
specification). The other 97 rules are 406,770 bytes (101,692 TOK_B4), and
43.4% of that text — 176,737 bytes, **44,184 TOK_B4** — sits on lines that
mention the same borrow/region/loan/view vocabulary. That is a mention density,
not a retirable count, but it locates where the rewrite lands. The concentrated
non-cluster rules:

| Rule | Bytes | TOK_B4 | A% | What is region/borrow in it |
|---|---:|---:|---:|---|
| FORM-8 | 11322 | 2830 | 95.4% | canonical region spelling: where a REGIONID is written and where it is elided — the whole rule |
| TYPE-2 | 5177 | 1294 | 85.6% | region arguments of source nominals, view and provider types |
| BLK-4 | 2344 | 586 | 84.2% | kernel-domain region parameters |
| FN-2 | 4525 | 1131 | 77.5% | region parameters and modes in a declaration |
| SET-2 | 3812 | 953 | 71.9% | commit through borrows and loan-bearing targets |
| MSR-2 | 3447 | 862 | 71.9% | measure support over borrowed and store-backed places |
| CALL-5 | 2019 | 505 | 71.2% | region arguments at a call |
| FN-4 | 2228 | 557 | 71.1% | law premises over borrowed parameters |
| BLK-0 | 11319 | 2830 | 69.8% | the kernel declaration domain's signatures, all written with modes, regions and effect rows |
| BLK-2 | 6924 | 1731 | 69.7% | reserving occurrences and bump extents |
| FN-3 | 3965 | 991 | 64.1% | declaration/definition agreement over modes and regions |
| TYPE-5 | 8270 | 2068 | 64.0% | region substitution and invariant brand positions |
| SET-1 | 5312 | 1328 | 63.3% | commit targets rooted in borrows |
| PRE-1 | 11818 | 2954 | 58.3% | the prelude's own signatures, in modes, regions and effect rows |
| FN-1 | 15262 | 3816 | 54.9% | parameter modes, result regions, the slice-result ceiling |
| ENT-3 | 22288 | 5572 | 49.0% | the entailment procedure's steps over borrowed and store-backed places |

Non-cluster rules whose text is 90%+ region/borrow mentions total 14,373 bytes
(**3,593 TOK_B4**); at 60%+, 79,361 bytes (**19,840 TOK_B4**). Those are the
rules that would be rewritten rather than edited.

---

## 3. `docs/patterns.md` by pattern card

34 cards plus a preamble and a "Known gaps" section, split at `## ` headings
outside fences. Per-card counts reconcile with `wc` on the file
(89,380 bytes / 14,125 words).

The mark is a mechanical score: occurrences of `&uniq`, a `&x` borrow form, the
word `region`, a REGIONID (possessives excluded), `Slice`/`slice_of`, and
`reborrow`, summed over the card.

- **SUBJECT** (score >= 10) — the card's worked example is written in borrow,
  region, view or reborrow forms and cannot survive unrewritten.
- **TOUCH** (score 1-9) — the card uses the forms incidentally; its lesson
  survives a rewrite of its code.
- **CLEAR** (score 0) — no borrow, region or view form at all.

| Card | Bytes | Words | TOK_B4 | TOK_W13 | uniq | &x | region | REGIONID | Slice | reborrow | Score | Mark |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|:--:|
| P1. Command buffer (write intents) | 1121 | 162 | **280** | 211 | 1 | 1 | 0 | 0 | 0 | 0 | 2 | TOUCH |
| P2. Struct-of-arrays pool (append-only, index-linked) | 1027 | 134 | **257** | 174 | 1 | 1 | 0 | 1 | 0 | 0 | 3 | TOUCH |
| P3. Region staircase + static nursery (lifetime shape) | 3384 | 494 | **846** | 642 | 2 | 2 | 15 | 13 | 0 | 0 | 32 | **SUBJECT** |
| P4. Linear threading (exclusive access through a call chain) | 1955 | 298 | **489** | 387 | 2 | 3 | 3 | 0 | 0 | 3 | 11 | **SUBJECT** |
| P5. Env-struct behavior parameterization (FN-5) | 1439 | 194 | **360** | 252 | 0 | 0 | 2 | 1 | 0 | 0 | 3 | TOUCH |
| P6. Behavior laws are not safety premises (FN-4) | 788 | 111 | **197** | 144 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | CLEAR |
| P7. Branchless classifier (i1 dataflow) | 913 | 134 | **228** | 174 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | CLEAR |
| P8. State a proof at the boundary that maintains it | 2931 | 467 | **733** | 607 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | CLEAR |
| P9. Exact capacity contract or recoverable shortage | 1869 | 265 | **467** | 344 | 0 | 0 | 1 | 0 | 0 | 0 | 1 | TOUCH |
| P10. Direct returned view | 1378 | 212 | **344** | 276 | 1 | 1 | 4 | 3 | 2 | 0 | 11 | **SUBJECT** |
| P11. Counted half-open range | 1477 | 226 | **369** | 294 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | CLEAR |
| P12. External constrained subject takes a value path | 1300 | 204 | **325** | 265 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | CLEAR |
| P13. Return the decision, not the access | 2770 | 461 | **692** | 599 | 6 | 8 | 6 | 11 | 0 | 1 | 32 | **SUBJECT** |
| P14. Guide a larger affine proof with `use` | 2537 | 403 | **634** | 524 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | CLEAR |
| P15. Own the handles and storage for the whole call | 2650 | 391 | **662** | 508 | 3 | 4 | 2 | 0 | 0 | 0 | 9 | TOUCH |
| P16. One length fact above the writes | 9478 | 1537 | **2370** | 1998 | 7 | 7 | 2 | 8 | 4 | 0 | 28 | **SUBJECT** |
| P17. Commit the transformed value back into the place it came from | 4783 | 762 | **1196** | 991 | 6 | 6 | 1 | 2 | 0 | 0 | 15 | **SUBJECT** |
| P18. Build a result locally, then publish it | 1137 | 163 | **284** | 212 | 2 | 2 | 0 | 0 | 0 | 0 | 4 | TOUCH |
| P19. Advance a tracked binding the same way on every arm | 4721 | 783 | **1180** | 1018 | 0 | 0 | 0 | 2 | 0 | 0 | 2 | TOUCH |
| P20. The loop body is already the region | 2281 | 383 | **570** | 498 | 2 | 3 | 13 | 0 | 0 | 1 | 19 | **SUBJECT** |
| P21. Mutate through an exclusive parameter and state both measures | 2210 | 305 | **552** | 396 | 3 | 3 | 1 | 0 | 0 | 0 | 7 | TOUCH |
| P22. Write `linear` for a logical obligation, and never for a storage one | 2344 | 407 | **586** | 529 | 0 | 0 | 0 | 1 | 0 | 0 | 1 | TOUCH |
| P23. Take the whole value apart in one statement | 1472 | 248 | **368** | 322 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | CLEAR |
| P24. `dispose` is the early release, not a free | 1814 | 331 | **454** | 430 | 0 | 0 | 0 | 2 | 0 | 0 | 2 | TOUCH |
| P25. Name a generic store brand; elide an already-determined brand | 2463 | 385 | **616** | 500 | 0 | 0 | 11 | 6 | 0 | 0 | 17 | **SUBJECT** |
| P26. Reserve the extent in the outer block and take inside an inner one | 1801 | 298 | **450** | 387 | 5 | 5 | 12 | 11 | 0 | 0 | 33 | **SUBJECT** |
| P27. Choose a type parameter's bound from what the body does with the value | 2517 | 432 | **629** | 562 | 0 | 0 | 8 | 4 | 0 | 0 | 12 | **SUBJECT** |
| P28. Borrow a contained run or take ownership of it | 4311 | 725 | **1078** | 942 | 7 | 8 | 3 | 7 | 0 | 0 | 25 | **SUBJECT** |
| P29. Give a nominal the store its contents live in | 3967 | 645 | **992** | 838 | 3 | 3 | 10 | 20 | 0 | 0 | 36 | **SUBJECT** |
| P30. Swap two elements in one commit | 1475 | 249 | **369** | 324 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | CLEAR |
| P31. Write through a view with `mut_slice_of`, read with `slice_of` | 5295 | 830 | **1324** | 1079 | 5 | 6 | 2 | 2 | 18 | 0 | 33 | **SUBJECT** |
| P32. Pass the destination on, and hand its reader back as the child | 2355 | 359 | **589** | 467 | 6 | 6 | 3 | 9 | 7 | 1 | 32 | **SUBJECT** |
| P33. A full fixed run of literals is a `const` | 1161 | 198 | **290** | 257 | 0 | 1 | 0 | 0 | 3 | 0 | 4 | TOUCH |
| P34. Read a stream to its end, and publish through a helper | 4247 | 663 | **1062** | 862 | 8 | 10 | 7 | 0 | 3 | 0 | 28 | **SUBJECT** |
| Known gaps (findings, not yet patterns) | 653 | 88 | **163** | 114 | 1 | 1 | 0 | 0 | 0 | 0 | 2 | TOUCH |
| Preamble (lines 1-23) | 1356 | 178 | **339** | 231 | - | - | - | - | - | - | - | CLEAR |
| **TOTAL (34 cards + gaps + preamble)** | **89380** | **14125** | **22345** | **18362** | | | | | | | | |

### Pattern subtotals

| Group | Cards | Bytes | TOK_B4 | TOK_W13 | % of file |
|---|---:|---:|---:|---:|---:|
| SUBJECT | 15 | 52985 | **13246** | 11029 | 59.3% |
| TOUCH | 12 | 22146 | **5536** | 4447 | 24.8% |
| CLEAR | 8 | 12893 | **3223** | 2655 | 14.4% |
| Preamble | 1 | 1356 | 339 | 231 | 1.5% |
| **TOTAL** | **37** | **89380** | **22345** | **18362** | **100.0%** |

Within SUBJECT, eleven cards exist *because of* the borrow/region model and have
no subject left without it — **P3** (region staircase), **P4** (linear threading
through reborrows), **P10** (direct returned view), **P13** (return the decision,
not the access), **P20** (the loop body is already the region), **P25** (name a
generic store brand), **P26** (reserve the extent in the outer block), **P28**
(borrow a contained run), **P29** (give a nominal the store its contents live
in), **P31** (write through a view with `mut_slice_of`), **P32** (pass the
destination on, hand its reader back as the child) — 31,960 bytes, **7,990
TOK_B4**. The remaining four SUBJECT cards (**P16**, **P17**, **P27**, **P34**)
teach a lesson that survives — a length fact above the writes, commit the
transformed value back, choose a bound from what the body does, read a stream to
its end — in code that would be rewritten.

---

## 4. The prelude cost: what every file must re-declare

`[PROG-1]` states that one closed compilation unit defines every language name,
"by the prelude `[PRE-1]`, or by the kernel declaration domain `[BLK-0]`", and
that this version has "no source include, import, module, source-path lookup, or
separate-compilation form." So a single-file writer re-declares every helper in
every file. Two measurements:

**(a) The measured re-declared prelude.** The blind-writer experiment
(`research/experiments/blind-writer/2026-08-28/REPORT.md` finding 3) names seven
helpers — `digit_byte`, `byte_at`, `put_byte`, `put_text`, `put_range`,
`put_decimal`, `emit_all` — as "roughly 110 lines that are byte-identical in four
of the five programs — 30-60% of each file before `main`." Extracting those seven
declarations from `p1_tree_wc.wf`:

| Declaration | Lines | Bytes | Words | TOK_B4 | TOK_W13 |
|---|---:|---:|---:|---:|---:|
| `byte_at` | 9 | 316 | 45 | 79 | 59 |
| `digit_byte` | 12 | 294 | 46 | 74 | 60 |
| `emit_all` | 25 | 852 | 110 | 213 | 143 |
| `put_byte` | 10 | 411 | 57 | 103 | 74 |
| `put_decimal` | 38 | 1105 | 157 | 276 | 204 |
| `put_range` | 11 | 528 | 74 | 132 | 96 |
| `put_text` | 12 | 495 | 70 | 124 | 91 |
| **Prelude, per file** | **117** | **4001** | **559** | **1000** | **727** |

The byte counts are identical function-for-function across `p1`, `p2`, `p3`, `p4`
and `p5` (an md5 over the five shared declarations is the same in `p1`, `p3`,
`p4` and `p5`), confirming byte-identical duplication rather than similar code.
`p2` omits `digit_byte` and `put_decimal` and carries 67 lines / 2,602 bytes;
`p3`, `p4` and `p5` omit `byte_at` and `put_range` and carry 97 lines / 3,157
bytes. **The per-file prelude cost is 1,000 TOK_B4 (727 TOK_W13), 117 lines**,
and the five programs paid it five times: 16,074 bytes of the 62,374-byte
five-program corpus, 25.8%. (p1 4,001 + p2 2,602 + p3 3,157 + p4 3,157 + p5
3,157.)

**(b) The same effect in `tests/programs`.** Across the 39 `.wf` programs
(431,808 bytes), 14 top-level declarations appear byte-identically in two or more
files — `factory_refusal` in 6 files (2,701 bytes each), `parse_port` in 4 (850),
`byte_at` in 4 (320), `socket_class` in 2 (2,069), `publish_all` in 2 (916),
`copy_range` in 2 (843), `send_all` in 2 (820) and seven more. Redundant bytes:
**24,786, or 6,197 TOK_B4** across the corpus, 5.7% of it. This corpus is a
deliberately varied compiler test set rather than five programs by one writer, so
its duplication rate is a floor, not the writer's experience.

**(c) The specification's own prelude rules.** `[PRE-1]` is 11,818 bytes
(**2,954 TOK_B4**) and `[BLK-0]` is 11,319 bytes (**2,830 TOK_B4**) — 5,784
TOK_B4, 12% of K, for the declaration domain alone. These are already counted
inside the 133,373 of section 1 and are not added again below. 58.3% of PRE-1's
text and 69.8% of BLK-0's sits on lines carrying mode, region or effect-row
vocabulary, because every signature in both is written in it.

---

## 5. Grand totals against K = 48,000

| Surface | Bytes | Words | TOK_B4 | TOK_W13 | x K (B4) |
|---|---:|---:|---:|---:|---:|
| `spec/kernel-spec.md` v0.57 | 533,492 | 80,763 | **133,373** | 104,992 | 2.78x |
| `docs/patterns.md` | 89,380 | 14,125 | **22,345** | 18,362 | 0.47x |
| Prelude, re-declared per file | 4,001 | 559 | **1,000** | 727 | 0.02x |
| **Taught surface total** | **626,873** | **95,447** | **156,718** | **124,081** | **3.26x** |
| **Budget K** | - | - | **48,000** | 48,000 | 1.00x |
| **Overage** | - | - | **+108,718** | +76,081 | **+2.26x** |

On the optimistic TOK_W13 estimate the surface is 124,081 tokens, still 2.59x K.
**K is missed by a factor between 2.6 and 3.3 on the current surface, and the
specification alone misses it by 2.2-2.8x before a single pattern card is
counted.** A no-reference core has to remove far more than the borrow rules for
M8 to be met; the borrow rules are the largest single removable block, not a
sufficient one.

---

## 6. What a no-reference core would cost — estimate

**This section is an estimate, not a measurement.** It prices the constructs the
core-model verdict itself names (section 1.3's Names table, plane map, judgment
table, conflict table, frame rule, operation table, join/loop-head rules,
signature vocabulary and erasure table; grafts G1-G6; repairs P-a..P-d; and the
additions section 5 marks required for D7, D9 and D12) as *prose lines this
specification would need*, at this specification's own measured density of
170.9 bytes and 25.9 words per non-blank line — **42.7 TOK_B4 (33.7 TOK_W13) per
line**. Nothing has been drafted; each line count is a judgement about how many
sentences the construct needs when written in the style of the rules measured
above, where one judgment such as `[OWN-5]` runs 33 lines and `[VIEW-2]` runs 26.

| # | Construct the core model introduces | Source | Est. lines | Est. TOK_B4 |
|---|---|---|---:|---:|
| 1 | Three conventions `let` / `inout` / `sink` and what each grants | 1.3 signature vocabulary | 14 | 598 |
| 2 | The plane map: `backing`, `plane<T, len n, align k>`, stride/offset/alignment, SoA/AoS, struct fields as one-element planes | G3 | 38 | 1623 |
| 3 | Projection: `f(inout c.a[i])`, the `with ... as` block, and the second-class discipline (down only, never up, never stored, never returned) | 1.3 Names, D4 primary | 40 | 1708 |
| 4 | Pools: `Pool<P,T>`, `Handle<P,T>`, the identity parameter, ghost `sigma`/`gamma`, `Live(h)`, insert/remove/compact, one liveness clause per pool at a loop head | G1 | 48 | 2050 |
| 5 | The frame rule and the two-part footprint | G2, P-a | 26 | 1110 |
| 6 | The four-cell read/write/end conflict table | P-c | 14 | 598 |
| 7 | `PATH`: prefix overlap over the containment tree root -> plane -> extent -> element | 1.3 judgments | 18 | 769 |
| 8 | `EXT`: half-open extents, the fixed linear fragment, the enclosing branch condition as a premise, `ProvedRangePartition` and its three side conditions | 1.3 judgments, D5 | 42 | 1793 |
| 9 | `BIND`: two-point lattice, join by conjunction, loop-head fixpoint of height 2, the arm-naming diagnostic | 1.3 judgments, join rules | 24 | 1025 |
| 10 | `INV`: syntactic instantiation of a written pool invariant at a written `use` step | 1.3 judgments, O15 | 20 | 854 |
| 11 | `NoReach(T)` as a checked type predicate, and distinctness as a corollary of `PATH` | G5 | 14 | 598 |
| 12 | `old()`-indexed exit refinements with a fixed chaining rule | P-b | 20 | 854 |
| 13 | `Foreign<T>` and the foreign kill rule as the stated exception to footprint identity | P-d | 14 | 598 |
| 14 | The `origin` set on a yielded projection and the yield-once `subscript` form | D7 additions | 22 | 939 |
| 15 | The four rejection shapes with their payloads | G6, D-5, M7 | 20 | 854 |
| 16 | Lowering commitments: no implicit deep copy, retain/release or COW; in-place `inout` and NRVO as lowering rules | 1.3 commitment | 12 | 512 |
| 17 | Erasure and the LLVM mapping: the block-head scope declaration, one `!alias.scope` per plane, `initializes`, `noalias` return | G4, 1.3 erasure table | 18 | 769 |
| 18 | The byte-extent `split` primitive, the one enumerated M3 entry | D9, O17 | 10 | 427 |
| 19 | Elision of the pool identity parameter | D12 "newly wanted" | 10 | 427 |
| | **Estimated new specification text** | | **424** | **18,106** |

---

## 7. Reading

**What a no-reference core saves and what it costs.** The ownership cluster is
31,598 TOK_B4, 23.7% of the specification and 66% of K by itself; of that, 9,918
retires outright (the 17 R rules: every region rule, every borrow-liveness and
reborrow rule, all four VIEW rules, the store-brand rule and the three effect-row
rules), a further 13,011 is the measured borrow/region apparatus inside the ten
rules whose judgment survives but must be restated over resolved paths,
conventions and footprints (`OWN-5`, `LIV-2`, `PROV-1`, `PROV-6`, `PAR-1`,
`PAR-2`, `ENT-5` and three smaller ones), and only 2,253 — `OWN-7`'s prefix
overlap, `OWN-8`'s reject-when-unsure, `LIV-1`'s join-checked liveness, `EFF-4`
and `CALL-6` — survives untouched, which is exactly the set the verdict keeps as
`PATH`, `BIND` and the no-fallback rule. Outside the cluster a further 3,439
TOK_B4 sits in rules that are 90%+ region text (`FORM-8`'s canonical region
spelling alone is 2,830) and 14,962 in rules that are 60%+, so a defensible
whole-specification retirement is roughly **26,000-30,000 TOK_B4**, plus 7,990 in
the eleven pattern cards that exist only because of the model. Against that, the
verdict's own construct list prices at an estimated **18,100 TOK_B4** of new
specification text — the plane map, projections and the `with ... as` block,
pools with branded handles and ghost slot/generation, the frame rule with
two-part footprints, the conflict table, `PATH`/`EXT`/`BIND`/`INV`/`NoReach`,
`old()` chaining, the foreign kill rule, `origin` sets and `subscript`, the four
rejection shapes and the erasure mapping — plus an estimated 5,700 for the eight
to ten new pattern cards those constructs need at the file's current 629-TOK_B4
average. **Net expected saving: on the order of 12,000-14,000 TOK_B4 on the
specification and 2,300 on the pattern cards, taking the taught surface from
156,718 to roughly 140,000 — 2.9x K rather than 3.26x.** The prelude is
untouched by the choice: 36.9% of its 4,001 bytes carry region, mode or
effect-row syntax and its seven signature lines (965 bytes) would roughly halve,
worth about 120 TOK_B4 per file, but its 1,000-token cost is a missing module
system, not an ownership model. The conclusion the numbers force is that
**retiring the borrow rules is the single largest available reduction and is
still nowhere near sufficient for K = 48,000**: the three largest families
(`ENT` 18,819, `DIAG` 13,272, `FN` 11,936) are 44,027 TOK_B4 of non-ownership
text that no D1-D4 choice touches, and `ENT` plus `DIAG` alone would consume two
thirds of K. Either K rises, or the taught surface has to be cut somewhere the
access-effects question does not reach — which is what owner decision **O6** is
actually asking, and why the verdict records that independent construct counts
for the same model ran from 11 to 50 and calls its own count of 13
"unfalsifiable".
