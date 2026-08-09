# Provenance-saturation measurements under real system boundaries

Status: two completed frozen measurements. The 2026-08-06 wfgrep probe found
zero forced branches; the 2026-08-09 boundary-fed DEFLATE measurement is a
negative prerequisite because the held rule catches only two of three required
canonical-Huffman sites.

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
