# Provenance-saturation measurements under real system boundaries

Status: three completed frozen measurements. The 2026-08-06 wfgrep probe found
zero forced branches; the first 2026-08-09 boundary-fed DEFLATE walk was a
negative prerequisite because the held rule caught only two of three required
canonical-Huffman sites; the task-0046 A-only explicit-dataflow revision below
reaches three of three without activating or changing the language.

## Wfgrep baseline measurement (2026-08-06)

This was falsifier #3 of DOSSIER.md §6, executed against
`tests/programs/wfgrep.wf` (723 lines) — the corpus's largest real program
and, since the v0.18 system-capability work landed, one with a **genuine
declared boundary**: `command fn main` receives `Args`, `DirectoryRead`,
`Output`; file bytes arrive through `read_once`, argument bytes through
`host_copy_bytes`. This is the browser-fear stress case: a grep whose entire
working set derives from the environment.

## Provenance walk

External values entering the program: argument count (`args_count`), pattern
bytes and length (`host_copy_bytes`), file content bytes and per-read counts
(`read_once`), plus every IoError. Everything else is counter chains
(`scan`, `probe`, `carry`, `filled`, `moved`, `position`).

Key mechanics observed on real code:

- **Content never reaches a subject position.** wfgrep compares bytes
  (`ieq<u8>`) but never indexes BY a byte — the literal-match design has no
  content-indexed tables. Match cursors (`terminator`, `scan`) are clean
  +1-counter chains whose *stopping point* is content-controlled (control
  dependence — no taint) while their values stay internal.
- **Boundary-op postconditions are the load-bearing launderers.** The
  cursor arithmetic stays provable only because `read_once` returns
  `count <= capacity` and `host_copy_bytes` returns `copied <= capacity`.
  With those contracts, `carry`, `available`, `pattern_length` are bounded
  program invariants; without them, environment-magnitude counts flood every
  cursor computation and most sites degrade to claims-on-suspect values.
  **Spec consequence: the SYS boundary operations' count bounds must be
  normative postconditions** — the boundary analog of the `read_bits`
  ensures found in SIMULATION.md.
- **The corpus already exhibits the design's predicted helper equilibrium**:
  `append_slice` is total by design (self-guarded capacity loop — zero
  contract, interior fully proven at L0 from its own guards);
  `copy_range` and `line_matches` are contract-carrying (2 requires clauses
  each: range-within-source, pattern-length-within-pattern), every clause
  discharging at the call sites via the guards `main` already contains
  (`overfull`, `complete`, linear arithmetic on `stop <= available`).

## Classification result

| bucket | count |
|---|---|
| index/buffer_new sites proven at L0 (given boundary postconditions + 4 threading requires clauses) | all but 4 |
| structural claims needed | **1** (`carry`/`available <= 4096`, one loop-head claim covering the 4 remaining sites and feeding both call discharges; discharges at L1) |
| taint-forced branches | **0** |
| values still tainted past the parse/boundary layer that reach any trap subject | **0** |

Additional user-ensures case found: `append_slice` would need
`ensures result <= len(destination)` for `report_failure`'s accumulation to
discharge without claims — the second real ensures case (first:
`read_bits`), same shape: a result bound derivable inside the body.

## Verdict

The browser fear (DOSSIER §4.8) fails to materialize on the most
input-saturated real program available: **zero taint false positives, zero
forced branches, one structural claim in 723 lines.** The mechanism that
prevents saturation is exactly the dossier's §2.5 trio — control-dependence
exclusion, truthful metadata, relational branch-washing — plus one addition
this probe promotes to load-bearing status: **normative count-bound
postconditions on boundary operations.** Caveat: wfgrep is a literal
matcher; a regex engine with byte-indexed transition tables would exercise
the const-table/u8-type-range laundering paths instead (predicted provable
by type range: a u8 index into a 256-entry table needs no fact at all) —
untested here.

## Boundary-fed DEFLATE measurement (task 0041, 2026-08-09)

### Frozen basis and method

This second measurement applies the held provenance candidate literally to a
closed program that now has a real boundary origin. It is pinned to:

- activation `f4c7e60c47bdea620eea5a00be89ff54d7678cc9` and active-spec
  SHA-256 `53495b9c47b92942876c90931d0296c752855954564ebf7435a549c48cb2dc86`;
- held-candidate SHA-256
  `62f9fbb98d69777f5cacacb8f63fd4a922eed4bef6e49d7a66f71df7827fb47b`;
- the compilation-unit order `raw_deflate.wf`, `raw_deflate_dynamic.wf`,
  `raw_deflate_dynamic_decode.wf`, `raw_deflate_boundary.wf`, whose respective
  SHA-256 digests are `c8fa0d58301e5346041c1886eaa3e277f9d3926212b6a5420e52b22eada300f0`,
  `cca35bbd3c5985c1e6753e0b0ca5311be7287d2021c01b46f14506b06734fcee`,
  `56c3bc84858849a27e4d493e6db0445056d36e2a7b3e864bb86d35bb22b792b7`,
  and `3fbd1281b1e9f4f9a161cf7d846622ae277611eaf9d34ce3ba576f3a81d140c4`.

The denominator is static, not the one dynamic fixture that happens to run in
the test. The entry accepts an arbitrary file, so stored, fixed, and dynamic
branches all belong to the checked program: the decoder has 21 claim
declarations and 29 subscript obligations; the boundary helpers add two
claims and four obligations, for **23 claims and 33 obligations**. The
dynamic-only supplemental denominator remains 16/24, or 18/28 with the
boundary helpers.

The walk reconstructed the candidate's PRV-1 least fixed point, PRV-2 columns,
and PRV-3 unasserted state from the checked tree and a source crosswalk. The
current checker retains a deterministic checked structure and traversal order,
plus run-local dense function and binding IDs and ordered claim/obligation
records. It does **not** retain statement identities, durable binding names,
provenance predecessors, claim-to-obligation support edges, or PRV-2 columns,
so this report is not an export of an existing provenance summary. Two
independent source walks reproduced the counts below. The five v0.24 redundant
advisories were also rechecked: `count_slot_in_counts`,
`validate_slot_in_counts`, `offsets_slot_in_offsets`,
`offsets_slot_in_counts`, and `ordered_symbol_in_lengths`. No scratch analyzer
or tracked artifact remains.

### Boundary and fixed-point lineage

`read_once` writes external content into `scratch`; `copy_range`'s derived
write column carries the external source byte into `stream`; `inflate` thus
receives an external `input`. `read_bits` reads that root, returns external
bits, and writes an external value into `InflateState.hold`. Because PRV-1 is
flow-insensitive per whole storage root, that write classifies every read of
the `InflateState` root, including its offset fields, as external.

The canonical-table path exposes the candidate's other decisive choice.
External code lengths choose the address written in `counts`, but the stored
right-hand side is still an internal counter. PRV-1 propagates neither control
dependence nor the provenance of a write address into its value, so `counts`,
then `offsets`, and finally `destination = offsets[count_index]` remain
internal. This is the laundering path the held candidate predicted its
measurement might find.

### Complete claim classification

`GATE` means that the subject is external and the unasserted state cannot
prove the obligation. `PASS` means that the subject is external but S1 or
S4–S10 proves it without S2/S3. `INTERNAL` means PRV-3 does not attach. The
line numbers are fixed by the source digests above.

| # | claim declaration | constrained subject and lineage | v0.24 disposition | held result |
|---:|---|---|---|---|
| 1 | `raw_deflate.wf:245 stored_header_zero_in_input` | external `state.input_offset` | retained | GATE |
| 2 | `:248 stored_header_one_in_input` | external `state.input_offset + 1` | retained | GATE |
| 3 | `:251 stored_header_two_in_input` | external `state.input_offset + 2` | retained | GATE |
| 4 | `:254 stored_header_three_in_input` | external `state.input_offset + 3` | retained | GATE |
| 5 | `:294 stored_copy_in_input` | external `state.input_offset + copied` | retained | GATE |
| 6 | `raw_deflate_dynamic.wf:20 length_symbol_in_tables` | external decoded `symbol - 257`; protects two table reads | retained | GATE |
| 7 | `:76 match_copy_in_history` | external state offset minus decoded distance | retained | GATE |
| 8 | `:108 count_symbol_in_lengths` | literal-seeded `symbol` counter | retained | INTERNAL |
| 9 | `:121 count_slot_in_counts` | external `cvt(lengths[symbol])`; protects a read and write | redundant | PASS |
| 10 | `:141 validate_slot_in_counts` | literal-seeded length counter | redundant | INTERNAL |
| 11 | `:180 offsets_slot_in_offsets` | literal-seeded length counter | redundant | INTERNAL |
| 12 | `:184 offsets_slot_in_counts` | literal-seeded length counter | redundant | INTERNAL |
| 13 | `:198 order_symbol_in_lengths` | literal-seeded symbol counter | retained | INTERNAL |
| 14 | `:206 order_slot_in_offsets` | external `cvt(lengths[symbol])`; protects an offsets read and write | retained | **GATE — canonical hit** |
| 15 | `:210 destination_in_symbols` | `offsets[external index]` yields internal under P2 | retained | **INTERNAL — canonical miss** |
| 16 | `:248 walk_length_in_counts` | literal-seeded bit-length counter | retained | INTERNAL |
| 17 | `:259 ordered_in_symbols` | external bits → code → offset → ordered | retained | **GATE — canonical hit** |
| 18 | `raw_deflate_dynamic_decode.wf:13 distance_position_in_lengths` | external `position - literal_count` | retained | GATE |
| 19 | `:69 code_index_in_order` | literal-seeded counter; external count affects only control | retained | INTERNAL |
| 20 | `:87 ordered_symbol_in_lengths` | internal const-table element | redundant | INTERNAL |
| 21 | `:238 end_symbol_in_literals` | literal `256` | retained | INTERNAL |
| 22 | `raw_deflate_boundary.wf:53 copy_read_in_source` | cursor seeded by actual `0`; external source is not the subject | retained | INTERNAL |
| 23 | `:56 copy_write_in_destination` | target seeded by actual `0` | retained | INTERNAL |

Seven obligations have no named claim. They complete the 33-site denominator
and supply the important branch-proven external controls:

| obligation | subject | held result |
|---|---|---|
| `raw_deflate.wf:38 input[state.input_offset]` | external whole-state offset; the preceding `input_done` false edge establishes the exact bound | PASS |
| `raw_deflate.wf:123 out[state.output_offset]` | external whole-state offset; the preceding `output_full` false edge establishes the exact bound | PASS |
| `raw_deflate_dynamic.wf:46 distance_bases[distance_symbol]` | external decoded symbol; the invalid-symbol false edge establishes `< 30` | PASS |
| `raw_deflate_dynamic.wf:48 distance_extras[distance_symbol]` | same subject and branch fact | PASS |
| `raw_deflate_dynamic_decode.wf:8 literal_lengths[position]` | literal-seeded position counter | INTERNAL |
| `raw_deflate_boundary.wf:33 text[taken]` | literal-seeded counter behind its own branch | INTERNAL |
| `raw_deflate_boundary.wf:34 destination[at]` | caller-seeded counter behind its own branch | INTERNAL |

The obligation-level cross-check is **18 external subjects out of 33**. Six
external obligations already prove in the unasserted state. The other **12
obligation nodes are rejected and belong to 10 distinct claims**; the two
extra nodes are the second table read protected by `length_symbol_in_tables`
and the second offsets access protected by `order_slot_in_offsets`. The other
15 obligations have internal subjects. This is why a first emitted diagnostic
is not a measurement denominator.

PRV-2 folds those ten declarations into 14 rejecting call statements and 21
external required-argument atoms: 1/1 for `inflate`, 2/2 for `decode_length`,
2/4 for `copy_distance`, 3/3 for `build_huffman_table`, 3/8 for
`decode_table_symbol`, and 3/3 for `store_dynamic_length`. The candidate's
column stores only a set of required parameter positions, while its diagnostic
requires a callee obligation. It does not say how one top-level argument that
protects several obligations selects or orders those diagnostics. That is a
second review defect, separate from the classification result.

### Canonical result, precision, and repair cost

The required canonical result is **2/3, not 3/3**:

- `order_slot_in_offsets` is gated;
- `ordered_in_symbols` is gated;
- `destination_in_symbols` is a false negative. External `count_index` only
  controls where internal count values are written; those internal values feed
  `offsets`, and P2 classifies `offsets[count_index]` from the root alone.
  The external allocation size of `symbols` is only the bound, which PRV-3
  deliberately excludes.

False-positive reporting needs two explicit lenses because the candidate does
not define an independent semantic oracle. Under its own whole-root,
flow-insensitive definition, every GATE row has a real external propagation
path, so there are **zero formal classification false positives**. Under the
site-local precision standard used by the saturation probe, the five stored
block claims are **five precision false positives**: external writes to a
sibling field and a later external update contaminate the whole state root
across paths and time, while each indexed offset at the claim point is a
program-maintained counter behind an aggregate guard. More broadly, eight of
the ten gated declarations are outside the three pre-registered canonical
sites, covering nine obligation nodes; this is reported as precision spill,
not silently relabelled as eight proven false positives.

Nine gated claim families could be replaced locally by exact value branches
returning the existing `Truncated`, `InvalidHuffmanCode`, `InvalidDistance`,
or `InvalidHuffmanTree` outcomes. `distance_position_in_lengths` is materially
more expensive: `store_dynamic_length` returns `unit`, so an honest repair
changes it to `Result<unit, InflateError>` and propagates that result through
three callers, or duplicates a domain branch at all three callers. The missed
`destination_in_symbols` receives no diagnostic at all; it first needs a rule
revision or a manually added branch. O3 also remains open: moving any gated
access into a helper whose `requires` performs the check bypasses PRV-3 more
cheaply than the honest Result repair.

### Verdict

This is a **negative prerequisite result and a successful measurement**. The
held rule finds live instances, but it misses one of the three sites its own
§8 requires, exhibits whole-root precision spill, leaves PRV-2 diagnostic
multiplicity undefined, and still has the material O3 `requires` bypass. The
candidate therefore returns to rule review, not to corpus rewriting or
activation. The active v0.24 specification, compiler, and protected corpus are
unchanged by this measurement.

## Revised explicit-dataflow remeasurement (task 0046, 2026-08-09)

### Frozen basis and one-rule delta

This is a source walk of one held-design revision, not a specification or
compiler result. It keeps the earlier measurement's exact basis:

- activation `f4c7e60c47bdea620eea5a00be89ff54d7678cc9` and active-spec
  SHA-256 `53495b9c47b92942876c90931d0296c752855954564ebf7435a549c48cb2dc86`;
- the compilation-unit order `raw_deflate.wf`, `raw_deflate_dynamic.wf`,
  `raw_deflate_dynamic_decode.wf`, `raw_deflate_boundary.wf`, with SHA-256
  digests `c8fa0d58301e5346041c1886eaa3e277f9d3926212b6a5420e52b22eada300f0`,
  `cca35bbd3c5985c1e6753e0b0ca5311be7287d2021c01b46f14506b06734fcee`,
  `56c3bc84858849a27e4d493e6db0445056d36e2a7b3e864bb86d35bb22b792b7`,
  and `3fbd1281b1e9f4f9a161cf7d846622ae277611eaf9d34ce3ba576f3a81d140c4`;
  and
- the static denominator of **33 subscript obligations and 23 claim
  declarations**, including every stored, fixed, and dynamic branch.

The only place-read propagation change is the A revision of P2: a place read
joins the class of its storage root with the class of every subscript-offset
atom in the resolved place. Fields preserve the accumulated class, and
`len(P)` remains internal. Writes still classify a storage root from the
written right-hand side, not from the address selected for the write. Branch,
match, and loop control still transfer no provenance. No whole-root address
taint and no path- or flow-sensitive storage rule was added.

The revised datum also makes the held result/payload projection explicit. A
payload-carrying enum retains one class for each direct variant payload, with
the aggregate derived as the join of those projections. A direct projection
holds the aggregate class of its field atom. Constructing or returning a
variant contributes only to that variant's corresponding payload; joining
alternative deliveries unions corresponding projections componentwise. A
`match` binder takes its selected variant payload's projection, not the
aggregate join. If the payload is itself a payload-carrying enum, that one
outer class conservatively seeds every direct projection of the inner binder;
the model does not invent recursive payload-path selectors. A `value_match` or
`value_if` binding likewise takes the componentwise union of the values its
`give` paths deliver; the scrutinee, condition, and selected control edge
contribute no provenance. On `propagate`, the normal binder takes the
`Ok(value:)` projection and the error return takes the `Err(error:)`
projection, applying the same nested-binder seeding rule when needed.
Dispatching on the tag never joins one sibling payload into another. This
structural datum is required to reproduce the held [SYS-2] table's internal
transfer-count payloads beside external error payloads; it is not enum-tag or
control-flow taint. The aggregate is a derived view of the projections, not
another independently counted result-column membership.

### Obligation and claim result

Only one of the 33 obligation subjects changes. At
`raw_deflate_dynamic.wf:207`, `destination = offsets[count_index]` now joins
the external `count_index`; the fixed point also makes `offsets` external via
the selected `counts` values described below. The following
`destination_in_symbols` claim therefore changes from `INTERNAL` to `GATE`.
Its unasserted state does not prove `destination < len(symbols)`. Every other
row in the complete task-0041 claim and unclaimed-obligation tables remains
byte-for-byte applicable.

| measure | held P2 | revised P2 |
|---|---:|---:|
| external obligation subjects | 18/33 | **19/33** |
| external subjects proved without S2/S3 | 6 | **6** |
| rejected obligation nodes | 12 | **13** |
| distinct rejected claim declarations | 10 | **11** |
| internal obligation subjects | 15/33 | **14/33** |
| canonical-Huffman declarations gated | 2/3 | **3/3** |

The canonical declarations are now all gated:

- `order_slot_in_offsets`, through the external code length used as its
  subscript;
- `destination_in_symbols`, through the external subscript of the `offsets`
  read; and
- `ordered_in_symbols`, through the decoded-bit path to `ordered`.

The 15 claims in the five original boundary-bearing programs still gain no
gate: wfgrep remains 0/8, `run-sysfile-multichunk` 0/4, and the three
copy-toosmall/invalid cases 0/1 each. Their subjects are still literals,
program counter chains, or bounded transfer counts; none is a newly external
place-read offset.

### Complete binding and storage-root delta

The table below enumerates every binary binding or root classification change
on the frozen boundary-fed unit. A name not listed retains its held class.
Result-projection and write-column membership are listed separately in the next
section.

| scope | newly external bindings or roots | closure and unchanged neighbors |
|---|---|---|
| `decode_length` | `length_base_word`, `length_base`, `length_extra_byte`, `length_extra_count` | The two constant-table reads now join external `length_index`. `length_extra` and `copy_length` were already external through `read_bits`. |
| `copy_distance` | `distance_base_word`, `distance_base`, `distance_extra_byte`, `distance_extra_count` | The two constant-table reads now join external `distance_symbol`. `distance`, `source_index`, and the actual boundary-path `byte` were already external. |
| `build_huffman_table` | `previous_count`; the `counts` root; the `count` at validation; `left`, `oversubscribed`, `incomplete`; the `count` used to build offsets; `offset`; the `offsets` root; `destination`; `destination_ok` | `previous_count = counts[count_index]` is the first new edge. Its external value is written back into `counts`; the later read makes `offset` external and writes it into `offsets`. `symbols` is **not** tainted by its external write address because its RHS `symbol` is internal. |
| code-table result of `build_huffman_table` | returned `Ok(value: table)` payload and caller binding/root `code_table` | The code table was internal under held P2 and becomes external through its `counts` field. `propagate` projects that `Ok` payload into `code_table`; it does not join the distinct error payload into the binding. The literal and distance table `Ok` payloads were already external because their external `symbol_count` actuals size `symbols`; they gain the new `lengths` lineage but no binary-class change. All three builds nevertheless change their internal `counts` and `offsets` roots as above. |
| `decode_table_symbol` at the code-table call | `table` actual, `empty`, `count`, `first_after_count`, mutable `first`, mutable `symbol_index`, `last_length`, mutable `decoded` | The newly external `code_table` root affects its field reads. `offset` and `ordered` were already external through the bits read from `state`/`input`; both `len` results stay internal. Literal- and distance-table invocations were already external-root cases. |
| `decode_dynamic` code-length path | `decoded_symbol` match binder, `length_symbol`, `direct_length`, direct `length` match binder, mutable `previous_length`, `repeat_previous`, `short_zero`, `long_zero` | These close from the revised code-table `Ok` result projection and revised selected-symbol result. The distinct error payloads remain in their own projections and do not inherit the successful payload's class. `repeat_bits`, `repeat_count`, `literal_lengths`, and `distance_lengths` were already external. The latter two roots are external from `buffer_new` with external sizes even before their content writes. |

The pre-construct code-table `symbols` allocation remains internal, while the
literal- and distance-table `symbols` allocations remain external through
their external allocation sizes. A construct joins its fields, so the newly
external code-table `counts` field is sufficient to make the returned
`code_table` root external; this does not retroactively classify the write
address as written content.

### Signature columns, protected leaves, and diagnostics

Exactly three `Ok(value:)` result-projection memberships and four write-column
memberships are added by written propagation edges. Every parameter dependency
named in this frozen delta uses that parameter's `plain` selector; no
enum-payload selector occurs here.

| derived column | added parameter datums | exact edge |
|---|---|---|
| `decode_length` `Ok(value:)` result | `(symbol, plain)` | `symbol` selects both constant-table reads that feed the returned length payload. |
| `build_huffman_table` `Ok(value:)` result | `(lengths, plain)` | a length-derived offset selects `counts`; the selected count reaches the returned table payload's `counts`. |
| `decode_table_symbol` `Ok(value:)` result | `(state, plain)`, `(input, plain)` | `read_bits(state, input)` reaches `ordered`, which selects the returned `symbols` element payload. |
| `decode_length` `state` write | `(symbol, plain)` | the selected `length_extra_count` is the `count` actual of `read_bits`, whose hold/bits update depends on it. |
| `copy_distance` `state` write | `(distance_symbol, plain)` | the selected `distance_extra_count` is the `count` actual of `read_bits`. |
| `copy_range` destination write | `(from, plain)` | `cursor`, seeded by `from`, selects the source element written to the destination. |
| `copy_distance` `out` write | `(state, plain)`, `(input, plain)`, `(distance_symbol, plain)` | those parameters reach `source_index`, which selects the history byte written back to `out`. |

`read_bits` gains another explicit edge from `state.input_offset` to its input
read, but `state` was already in both its `Ok(value:)` result projection and
state-write dependencies, so no column membership changes. No `Err(error:)`
result projection changes:
the three result additions above are confined to the successful payloads, and
an external aggregate join does not reclassify a distinct error projection.
That invariance follows from the structural projection rule above rather than
from a hand-assigned exception in the source walk. `store_dynamic_length`
still derives writes from `value`, not from `position` or `literal_count`;
`emit_byte` still derives the output write from `value`, not from
`state.output_offset`. No other direct or call-composed result projection or
write set expands. In particular, the external address of
`symbols[destination] = symbol` does not invent a `symbols` write dependency.

The revised internal-required column is a finite relation from a **parameter
datum** `(position, selector)` to a protected leaf. The selector is the
parameter's sole `plain` datum for a non-payload type or one exact
`(variant, payload field)` projection for a payload-carrying enum. An enum
aggregate expands to the union of its projections and is not an independent
selector. The leaf identity is the concrete [FN-2] instantiation, exact
[ENT-6] obligation occurrence, and normalized conjunct ordinal; every current
obligation has one conjunct at ordinal zero. Direct edges retain the leaf in
the function that contains it. At a call, each callee datum selects the actual
atom's corresponding plain or payload projection, then substitutes the
caller parameter datums in that selected component's dependency while
preserving the leaf. This componentwise substitution never replaces a payload
selector with the actual's aggregate. Union to a least fixed point over the
finite `(function, parameter datum, leaf)` domain handles ordinary, recursive,
and mutually recursive calls.

The reconstruction seeds each ordinary parameter component with its own
parameter datum; an E1 entry input is unconditionally external. Each explicit
`return` contributes its plain datum or direct projections componentwise to the
result column, multiple returns union, and `propagate` contributes its
automatic error-return projection through the same result edge. These base
generators are required for the three revised `Ok(value:)` result memberships
below; they are not inferred from discovery order.

The write column likewise has explicit generators: a direct `set` contributes
its right-hand-side aggregate to every overlapped formal `&uniq` root, an E3
system write contributes unconditional external content, and a user call
substitutes the callee write component through [EFF-2]'s boundary projection.
The target address by itself contributes nothing. These are the edges used by
the four revised write memberships below.

All parameter selectors participating in the frozen `Req` projection are
`plain`. Lifting the key from a position to a datum therefore
changes neither of the two `Req` deltas below, nor the fourteen rejecting call
statements or 24 external required-argument atoms. Adding another protected
leaf to an already-required datum changes only that datum's diagnostic targets
and does not narrow acceptance again. Adding a distinct payload datum at the
same parameter position can narrow that previously unrequired component.

Two internal-required changes must be kept distinct. In
`build_huffman_table`, `lengths` was already required for the two
`order_slot_in_offsets` obligation leaves. Its `(lengths, plain)` datum
therefore does not grow as a domain member, but the relation now also retains
the `destination_in_symbols` leaf and composes that leaf through callers. In
`copy_distance`, by contrast, `distance_symbol` newly reaches
`source_index` through the selected distance-base and distance-extra entries,
so `(distance_symbol, plain)` and its `match_copy_in_history` leaf are a
genuinely new pair. `decode_table_symbol` already required the plain datums
of `table`, `state`, and `input` for `ordered_in_symbols`; the revised result
creates a new external table actual at the code-table call, not a new callee
datum.

The frozen diagnostic projection is therefore **14 rejecting call statements
and 24 external required-argument atoms**, versus 14/21 under held P2. Three
atoms are new: `code_table` at the first `decode_table_symbol` call, whose
`state` and `input` atoms were already external, and `distance_symbol` at each
of the two `copy_distance` calls. The three `build_huffman_table` calls still
contribute one external `lengths` atom each, but that one argument now protects
the two order-slot leaves and the new destination leaf.

Diagnostics are selected only after relation convergence. At one call, the
compiler unions the protected leaves for every required datum at an argument
position whose corresponding actual component is external, then emits one
event at that argument atom rather than one event per datum or leaf. Witness
reconstruction searches finite `(function, parameter datum, leaf)` states,
minimizes call-boundary count, then lexicographically orders the complete
sequence of call/argument node paths and callee/caller parameter datums,
followed by leaf node path and conjunct ordinal. Parameter positions use
declaration order; `plain` precedes enum selectors, which follow variant and
payload-field declaration order.
A residual tie between concrete instantiations of the same source route uses
[DIAG-1]'s implementation-defined but executable-stable order, never
hash/worklist or first-discovered order;
visited states terminate a recursive cycle. The direct,
multiple-leaf, one-call, self-recursive, and mutually recursive control shapes
all reach the same relation independent of visit order. A direct edge yields
one zero-call witness; multiple leaves remain distinct before one witness is
chosen; an ordinary call preserves the callee leaf; and revisiting a recursive
state adds neither a relation pair nor an infinite witness.

### Hostile boundary controls

These controls separate the selected explicit-dataflow rule from the stronger
whole-root or noninterference rules that task 0046 did not choose:

| control | revised classification | reason |
|---|---|---|
| internal root, external read offset | external | the read joins its subscript offset |
| internal root, internal read offset | internal | neither root nor offset is external |
| external root, internal read offset | external | the existing root dependency remains |
| nested place with two offsets | join of root and both offsets | fields preserve, rather than reset, the accumulated class |
| `len(P)` for any root or offsets | internal | the explicit metadata special case is unchanged |
| constant table indexed by an external value | external selected value | this is the `length_bases`, `length_extras`, `distance_bases`, and `distance_extras` shape |
| literal selected only by an external branch | internal | branch choice transfers no provenance; this remains parser laundering by design |
| guarded external write address, internal RHS, later fixed read | internal root and internal fixed read | write-address implicit flow remains outside the classifier |
| checked arithmetic over an external operand | external `Ok(value:)`; internal `Err(error:)`; external derived aggregate | the success value has an explicit operand edge, while the tag-only arithmetic error is selected only by control |
| `Result<T, E>` with external `Ok(value:)` and internal `Err(error:)` | external aggregate; external `Ok` binder; internal `Err` binder | match extracts the corresponding projection instead of copying the aggregate join into both payloads |
| `propagate` of that same result | external normal binder; internal propagated error payload | the `Ok` and `Err` edges project their own payloads, and tag-selected control transfers no provenance |
| user helper forwards that same result | external `Ok`, internal `Err`, external derived aggregate at the caller | the helper's result column and call substitution map each payload selector independently instead of collapsing both through the aggregate |
| outer `Err(error:)` external where `error` is itself a payload enum | every direct projection of the matched error binder is external | the outer projection carries the nested atom's aggregate; finite one-level selectors do not pretend to retain deeper precision |

The last control is an intentional, observable limit rather than a neutral
case. Let an internal zero-filled `a` be updated only under a bounds guard by
`set a[external_i] = 1`. The RHS is internal, so `a` remains internal; a later
`fixed = a[0]` also remains internal. If `fixed` is used to index a one-element
buffer behind a supporting claim, the claim is legal under the revised gate,
yet an environment choice of `external_i == 0` makes `fixed == 1` and can make
that claim fire. Catching this would require write-address or control-flow
taint, or path-sensitive storage, none of which the A-only revision claims to
provide. Conversely, reading `a[external_i]` directly is external because the
read offset itself is joined.

### Precision and disposition

The site-local precision accounting is unchanged. The five stored-block claims
remain precision false positives under the site-local lens because whole-root,
flow-insensitive `InflateState` contamination is unchanged. Eight of the 11
gated declarations remain outside the three canonical declarations, the same
noncanonical precision-spill count as before. The revised rule adds exactly the
pre-registered canonical miss rather than relabelling either precision lens.

The honest repair count advances with that one new diagnostic. Ten of the 11
rejected claim families, now including `destination_in_symbols`, can become a
local value branch returning an existing domain error. The remaining
`distance_position_in_lengths` family is still expensive:
`store_dynamic_length` must return `Result<unit, InflateError>` through three
callers, or those callers must each carry the branch. Moving either access
behind `requires` remains the cheaper O3 bypass, not a repair.

This is a **positive remeasurement of the two task-0046 review defects**, not
an activation result. The direct place-read rule reaches canonical 3/3, and
the finite parameter-datum-to-leaf relation makes multiple protected
obligations and their diagnostic witness deterministic. The active v0.24
specification, compiler, protected corpus, and installed identity are
unchanged. The revised design remains held: O3 still permits a gated obligation
to move behind a callee `requires`, so stage 7 must close that bypass before
any exact-byte specification candidate or provenance activation may be
proposed.
