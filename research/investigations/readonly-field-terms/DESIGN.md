# Readonly fields below subscripts as terms

## Question and scope

Specification v0.69 made a subscripted place an [ENT-2] term only through the
prelude's measures: clause (b) named "a place whose final step selects a
readonly field", but then required the prefix to be a measured type having
that measure, and the endpoint paragraph confined the subscript admission to
measure places. A writer's own readonly field reached through a subscript,
such as `deref(nodes)[i].count` in an index-based tree, was therefore no term:
it could not bound a counted loop, appear in a `requires`, or be copied with a
`let` equal to its source.

This investigation decides, from the kill semantics and the index-based tree
and layout use cases, whether that field should be a term, and bounds what the
change reaches. The owner's 2026-09-20 ruling ("a place with subscripts is a
term when its last step is a readonly field") is the history of the question,
not its ground.

## Soundness: every change is an overlapping write

A term over storage is sound when every event that can change the value the
term denotes kills every fact supported by it. [ENT-5] kills a fact when a
write overlaps its support under [OWN-7], or when a support binding is
consumed or leaves scope. For a place `P[i].f` whose last step selects a
readonly field `f`, [TYPE-2] makes `f` never a `set` target and never an
argument at a written parameter. The complete set of events that change it:

| event | written place | why it overlaps `P[i].f` |
| --- | --- | --- |
| `set P[j] = v`, `replace`, `update` of an element | `P[j]` | prefix of the term's place unless `j != i` is proved |
| `set P = v`, `set owner = v` | a prefix | prefix |
| `swap(first: &P[a], second: &P[b])` | `P[a]`, `P[b]` | prefix unless both offsets are proved distinct from `i` |
| `remove_at`, `insert_at` on a window | `P.filled` | [WIN-2]: every live index overlaps `filled` |
| `place_front`, `take_front` | `P` | prefix |
| `take_back` | `P.last` | overlaps `P[i]` unless `i != P.len - 1` is proved |
| call whose row writes `P` or a prefix | projected actual | prefix; a range actual reaches its element storage [CALL-3] |
| write to the offset binding `i` | `i` | the offset's support is part of the term's support |
| rebinding a reference the place reads through | the holder | the holder is a support member |
| consume or scope exit of the root or offset | — | kills (c) and (d) |

A write to a sibling field `P[i].g` or to an element at a proved-distinct
offset overlaps none of these and correctly keeps the fact. Measures over
subscripted places already relied on exactly this table, so the extension adds
no kill rule. The specification now states the ground once, in [ENT-2]'s
clause (b) paragraph, and [ENT-5]'s support sentence covers every clause (b)
term: the storage of its readonly field and the support of every offset in its
place.

Verification in the compiler found one kill the general term needed and the
measure path already had: the tracked-place arm of `event_kills_term` checked
only resolved-place overlap and consumption, not the support of offsets in
the term's own spelled path. Clause (a) places have no offsets, so this was
not a defect before; a subscripted place term without it would have survived
`set i = 2` and discharged a subscript at the new element (reproduced before
the fix). The arm now also applies `event_kills_offset_support`. No kill was
missing for existing measure terms.

Evidence: the conformance cases
`ent5-neg-readonly-field-offset-reassigned-in-loop`,
`ent5-neg-readonly-field-element-replaced`,
`ent5-neg-readonly-field-element-swapped`,
`ent5-neg-readonly-field-window-shift` and
`ent5-neg-readonly-field-callee-writes-base` each reject a program whose
runtime would read out of bounds; `ent5-pos-readonly-field-sibling-and-distinct-writes`
keeps the fact across non-overlapping writes. Reference-holder rebinding and
unproved-distinct element writes were checked with the same shape during the
change.

## Alternatives

| alternative | sound | reaches the use cases | term growth |
| --- | --- | --- | --- |
| A. No change: bind a `let` copy first | yes | local loops only | none |
| B. Readonly fields below subscripts (selected) | yes, no new kill rule | loops, requirements, copies | only readonly fields a program reads below subscripts |
| C. Every integer field below subscripts | yes, same kills | same, plus mutable fields | every such read, killed by every unproved-distinct element write |
| D. Quantified per-element facts | needs instantiation at every read and re-proof at every write | yes | unbounded family |

A fails the requirement use case: a copy is detached from the storage it came
from, so a callee cannot state a relation between one element's field and
another storage, and a caller's fact about an element never reaches the
callee. B covers that case with no new mechanism. C differs from B only for
fields a program assigns in place; in a loop that updates `nodes[j].x`, every
`nodes[i].x` fact dies unless `i != j` is proved, so those terms rarely survive
to be useful while every element write pays the overlap test against them.
Its closure-size cost is unmeasured; it stays the recorded follow-up in
`docs/todo.md`. D is refused by the checks-and-proofs node's existing
decision on quantified storage facts.

Restricting B to the prelude's measures is the v0.69 state. A writer's
readonly field changes by exactly the events of the table above, so the
restriction was a special case with no soundness difference.

## Boundary of this change

The term joins the L0 fragment wherever [ENT-2] terms are consumed: comparison
origins and branch facts [ENT-3.S1], requirements and their caller discharge
[FN-8, ENT-3.S4], `let` copy equalities [ENT-3.S5] and the S7 rows that read a
term, and counted endpoints [ENT-3.S11].

It deliberately does not reach three surfaces, each for its own reason:

- [FN-9] relation datums. Publishing a relation over a formal-subscripted place
  needs the formal offset substituted on both the body and the caller side. The
  existing measure path reads such an offset as an unknown capture (a relation
  over `deref(rows)[i].len` renders as `[?]`), which is only unusable today
  because nothing is provable about that term; admitting readonly fields there
  without fixing both substitutions would make it unsound. FN-9's datum list is
  unchanged and `fn9-neg-readonly-field-below-subscript-relation` pins it.
- [INV-1] affine atoms and [ENT-6] automatic images. Invariant atoms admit no
  field selection at all except measure members, and ENT-6's candidate set is
  specified over measure terms. A readonly field joins as clause (a) field
  places do: L0 only. A relation such as `first + count <= kids.len` therefore
  still needs own parameters or a measure.
- Tracked-place offsets with projections. [ENT-2] admits `nodes[n.parent].count`,
  but the compiler captures only literal, const and bare-binding offsets. Such a
  place forms no term (under-derivation), and as a counted endpoint it is
  reported as an unsupported compiler capability, never as a source verdict. A
  measure read with such an offset was already unsupported.

`docs/todo.md` records each with its validation criterion.

## Checking cost

New terms arise only where a program reads a readonly integer field below a
subscript; each distinct spelling is one term, exactly as a measure over the
same place is, and dies with its support. Before this change only a handful of
conformance cases and no maintained program or library performed such a read,
so existing checks change cost negligibly. No timing experiment was run:
nothing here selects between alternatives by cost, and C's cost question is
left open with its own criterion.
