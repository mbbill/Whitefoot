# F-D4 — the three census flagships, rewritten fully claim-free

**Verdict: MIXED.** All three programs are now claim-free, compile at v0.39 with
the gate compiler, and are byte-identical to their originals on every input
tested (1,195 differential cases across the three). F-D4's stated refutation
condition — "the reduced probes do not survive contact with the whole sources"
— did fire once, on an effect row, and it is avoidable but unpriced. The larger
result runs the other way: **the design's corpus table understates what v0.39
already reaches by six sites**, and two of its route assignments are wrong.

Subject: `research/investigations/claim-model/DESIGN.md` at `236b837f`
(branch `batch/0106-claim-model-design`). Compiler: the gate build at
`/tmp/.../wf-0107-audit/target/gate/whitefootc`, spec `ACTIVE v0.39`.
Sources: `orig/` (copies of `tests/programs/*.wf`), `rw/` (the rewrites),
`probes/` (the negative probes), `harness/` (the differential drivers).

## Score in one table

| site | design's route (4.5.2) | what v0.39 actually needed | verdict |
| --- | --- | --- | --- |
| `percent_decode:16` | `[ENT-5.R]` + `[ENT-3.S5.O]` | loop-exit respelling, no rule | design understates |
| `percent_decode:18` | `[ENT-5.R]` alone | guard on `source_length` + the existing `requires`, no rule | design understates |
| `percent_decode:28,31` | the guard rewrite (t4) | exactly that | **holds** |
| `wfgrep:434` | the guard respelling (t10) | exactly that | **holds** |
| `wfgrep:469,495` | `[ENT-5.R]` + the `imin` image | `imin`+`ieq` respelled as one `ile`, no guard, no rule | **design wrong** |
| `wfgrep:553` | restructure (Q1 option b) | exactly that | **holds** |
| `wfgrep:556` | the unsigned-subtraction image (bucket P) | the same restructure as `:553`; bucket P is not reachable at v0.39 and buys nothing here | **design wrong** |
| `ipv4_checksum:19,22` | the pair guard (t8) | exactly that | **holds** |

Eleven claims deleted; zero added rules; zero behaviour changes.

## 1. `percent_decode.wf` — 4 claims deleted

**Diff summary.** `rw/percent_decode.wf`. The loop exit `ieq(input_index,
source_length)` becomes `ilt(input_index, source_length)` with the break on the
false edge, which is t4's shape and republishes the source bound at every head
(`:16`). The `remaining >= 3` test is replaced by two `+checked` additions and
one guard `ilt(last_index, source_length)` (`:28`, `:31`); the two `Err` arms and
the guard's false edge all fall into the literal-byte path the program already
had, so I hoisted the three identical write-and-step tails into one
`emitted`/`stride` pair written once at the end of the body. That is why the
line diff (83 removed / 88 added) is larger than the change: most of it is
re-indentation of the escape block under the two `match` arms. `:18`, the
coupled `output_index <= input_index` invariant, becomes a guard
`ilt(output_index, source_length)` whose false edge breaks the loop; the
contract's `requires ige(output_length, source_length)` carries it the rest of
the way to the output buffer's own length. `traps` leaves `decode` and `main`.

**Compile.** `whitefootc -o target/percent_decode_rw rw/percent_decode.wf`
succeeds with no diagnostic.

**Run.** Both self-checking `main`s exit 0. For a real differential I built
`harness/pd_driver_main.wf`, an argv-driven driver that decodes `argv[1]` and
publishes the produced count and the *whole* output buffer including its 204
sentinel fill, then linked the identical driver against the original `decode`
and the rewritten one. Over **691 inputs** — every string of length 1..3 over
`%0AfgZ`, thirteen hand-built escape shapes, 400 random strings over an
alphabet containing `%`, hex digits, non-hex letters and `\xff`, and 20 strings
of 200..1000 bytes — stdout and exit status are identical in every case.
`harness/pd_diff.py`: `cases=691 divergences=0`.

**Honesty.** Two unreachable false edges (the `+checked` `Err` arms) and one
more (`room_left`). None invents a value: an overflowing offset means "no escape
pair here", which is the literal-byte path, and an exhausted output means "stop
decoding", which is what `decode` returning a short count already means.

### 1a. The one interaction F-D4 asked for, and it is an effect row

My first rewrite of `:18` did the obvious thing and guarded against the output
buffer's own length:

```
let output_room = len(deref(out));
let room_left = ilt(output_index, output_room);
```

That is rejected — not for the bound, which it discharges, but by `[EFF-2]`:

```
[EFF-2] EffectMismatch expected_row: "reads(out, src), writes(out)"
                       found_row:    "reads(src), writes(out)"
                       missing:      ["reads(out)"]
```

**Reading `len` of a `&uniq` parameter is a read of it.** The original never
read `out` in the body — the length reached it only through the contract's
`define output_length = len(deref(out))` — so the guard route on a
write-only output parameter *widens the function's published effect row*, and
the widening propagates to every caller that declares one. That is exactly the
class F-D4 names ("an interaction with an effect row") and the design does not
mention it anywhere: not in 3.12's route menu, not in 6.2's mechanical fix
channel, which would hand the writer precisely this version, and not in 11.1's
bill of what the deletion costs.

It is avoidable here: guarding `output_index` against `source_length` instead,
and letting the `requires` carry `source_length <= output_length`, keeps the
row at `reads(src), writes(out)`. That is what `rw/percent_decode.wf` does. But
the escape depends on the function already having a contract that relates the
two lengths. A function guarding a `&uniq` output parameter with no such
contract has no escape, and the design owes that case a sentence.

The interaction also runs the other way, in the deletion's favour, and the
design does not count this either: a claim puts `traps` in its function's
effect row and `traps` propagates to every transitive caller. Deleting the
claims deleted `traps` from `decode` and `main` here, and from `search_file`,
`walk` and `main` in `wfgrep` — three functions whose own claim count is zero.

## 2. `ipv4_checksum.wf` — 2 claims deleted

**Diff summary.** `rw/ipv4_checksum.wf`. t8's shape, verbatim in spirit: the
`offset == length` exit and the two claims are replaced by `offset +checked 1`
and one guard `ilt(low_offset, length)`. Both subscripts discharge from the one
guard because the checked addition makes `low_offset` an exact successor of
`offset`. The guard's false edge is the odd tail; I gave it the RFC 1071 fold
(`sum += last << 8`, then the carry fold) rather than t8's plain add, so the
value is right and not merely defined. The `requires ieq(iand(length,1),0)` and
its two caller-side parity guards in `main` go with the claims, as 4.5.2 says.
`traps` leaves the function and `main`. The `u16` narrowing is kept, so `main`
is otherwise unchanged.

**Compile.** `whitefootc -o target/ipv4_checksum_rw rw/ipv4_checksum.wf`
succeeds with no diagnostic.

**Run.** Both self-checking `main`s exit 0 over both the const header and the
copied runtime header (the zero-bearing canonical header, which argv cannot
carry). `harness/ipv4_driver_main.wf` is the argv driver; over **276 even-length
headers** — 30 random each at lengths 2, 4, 6, 8, 10, 20, 40, 100 and 500, plus
all-`0xff` and all-`0x01` at 2, 4 and 20 — the two builds publish identical
checksum bytes with identical status, and every one of them agrees with an
independent RFC 1071 reference written in the harness.
`harness/ipv4_diff.py`: `even cases=276 divergences=0 reference_mismatch=0`.

The odd tail is a widening, not a change: the original refuses odd lengths at
its `requires`, so nothing it accepts moves. Driving the rewrite alone over
**160 odd-length headers** at lengths 1, 3, 5, 7, 9, 21, 101 and 255, every
checksum agrees with the RFC 1071 reference. The design's sentence that t8's
"false edge is the odd-tail case the RFC actually specifies" is confirmed by
execution, which it was not before.

**One caller-side detail worth the row.** Deleting the `requires` removes a
real cost the design records only as "artificial". With it in place a caller
must establish `ieq(iand(len(view),1),0)` against *the slice's own length*;
guarding the buffer length it was built from is not enough, and my first driver
was rejected by `[FN-8]` with residual `ieq(iand(len(view), 1_u64), 0_u64)`.

## 3. `wfgrep.wf` — 5 claims deleted, in three functions

**Diff summary.** `rw/wfgrep.wf`, 37 lines removed / 33 added over 1,417.

*`append_trailing_newline` (`:434`).* t10's respelling, and the design is exactly
right about it. The original tested `ile(carry, input_room)` and then
`ieq(input_room -wrap carry, 0)`; together those reject precisely
`carry >= input_room`. One guard `ilt(carry, input_room)` says that, and the
write discharges from it. Two tests become one; the claim goes.

*`scan_line` (`:469`, `:495`).* **The design's route is wrong and the site is
easier than it says.** `imin(available, input_room)` followed by
`ieq(available, bounded_available)` is the long spelling of one comparison:
`a == min(a, r)` exactly when `a <= r`. Written as `ile(available, input_room)`
the admission test *is* a published guard, `bounded_available` is just
`available`, and both reads discharge by transitivity through the existing loop
guard `ilt(probe, available)` and the existing `ilt(spot, available)`. **No new
rule, no `imin` image, no added guard, no added branch** — the two claims are
simply deleted along with their `probe_ok`/`spot_ok` bindings.

*`shift_input_tail` (`:553`, `:556`).* The same respelling, plus Q1 option (b).
`source_index` is guarded directly against `input_room` before the read, and
`moved` before the write. Each false edge sets `produced = moved` and breaks,
so the returned `tail` is the count actually moved on every edge — no invented
value, no dead assignment.

**Compile.** `whitefootc -o target/wfgrep_rw rw/wfgrep.wf` succeeds with no
diagnostic, in 10 seconds. `traps` leaves all three functions and also
`search_file`, `walk` and `main`.

**Run.** `harness/wfgrep_diff.py` runs both builds side by side in the same
fixture tree and compares stdout, stderr and exit status. **228 cases, zero
divergences**: the test suite's own search tree under seven patterns including
the empty and the total hit set; the 300-level deep tree; a sweep of every
read-boundary offset from 4056 to 4135; lines of 4095, 4096, 4097, 8192 and
12295 bytes, each also unterminated; the empty tree, the empty file, the
missing root, the missing argument, the non-text pattern, the symlink case,
the unreadable directory and the unreadable file; 120 randomised trees of 1..4
files at depths 0..2 with sizes clustered on the buffer boundaries; and an
80-entry directory past the 64-entry truncation bound. Spot checks confirm the
shift path is genuinely exercised: a 40-buffer file with 42 matches produces
byte-identical output from both builds, and that output equals host `grep -rn`.

## 4. Q1 — the answer is no, and the good finding does not materialise

**Restructuring `wfgrep:553` does not change what `wfgrep` does.** Two
independent grounds.

*By proof.* On the guarded path `available <= input_room`, `scan <= available`,
and the loop guard gives `moved < tail` where `tail = available - scan` with no
wrap. So `source_index = scan + moved < scan + (available - scan) = available <=
input_room`. The guard's false edge is unreachable, and `produced` therefore
always equals `tail`. Same for `moved < input_room`.

*By execution.* 228 differential cases, byte-identical on stdout, stderr and
exit status, including every read-boundary offset in an 80-byte window around
the 4096-byte buffer and five over-long-line sizes — the shapes that drive
`shift_input_tail` at all.

So the design's flag — "if the rewrite turns out to change what `wfgrep` does,
that is a finding worth more than the rule" — does not collect. Q1's
recommendation (b) is confirmed and Q1 can be closed.

**Q1's diagnosis of *why* is also confirmed, and I pushed on it hard.** The
chain needs two sums, and v0.39 does not compose them even when every term is
exact. `probes/p1_shift_respelled.wf` (respelling alone), `p2` (`+checked` on
the offset), `p4` (`-checked` on the tail as well), and `p5` (both checked and
the whole loop inside the `Ok` arm, so no mutation stands between the
subtraction and the use) all fail identically:

```
[OP-4] UndischargedBoundsObligation residual: "source_index < len(deref(input))"
```

There is no arithmetic route at v0.39. `probes/p6_shift_restructured.wf` is
option (b) in isolation and compiles.

**And a second, unflagged residue rides with it.** `probes/w_nowrite.wf` is the
full rewrite with only the `moved` guard removed; it fails with residual
`moved < len(deref(input))`. So `:556` is not closed by the respelling either —
at v0.39 it needs the same restructure as `:553`, not the unsigned-subtraction
image 4.5.2 assigns it. Under the design's endpoint bucket P would close it,
but bucket P is a 108-claim purchase justified elsewhere; at *this* site the
two lines that `:553` forces anyway close it for free, which is the same
reasoning the audit used to demote the `+-wrap` rows.

## 5. The impossible-else bill, measured

11.1 predicted three tiers and said the corpus bounds how often the third bites.
Over eleven claims in three programs the bill is:

- **Tier zero (statement-position guard, empty or already-existing false edge):
  8 sites** — `percent_decode:16,28,31`, `wfgrep:434,469,495`,
  `ipv4_checksum:19,22`. Of these, three (`wfgrep:434,469,495`) cost *nothing at
  all*: they are respellings that delete a test rather than add one.
- **Tier one (a `break`): 3 sites** — `percent_decode:18`, `wfgrep:553,556`.
- **Tier three (an invented value or a widened signature): 0 sites.**

11.1's sentence "No corpus claim was found whose successor is a value-position
invented return" is now backed by the experiment it named, on the three
flagships, and it held. That is the single most load-bearing result here and it
is a pass.

Three false edges are unreachable (`percent_decode`'s `room_left`, `wfgrep`'s
`read_ok` and `write_ok`). None is a dead guard in the dishonest sense 11.1
warns about: each has a true total meaning on its own terms — "the output is
full, stop", "only this many bytes moved, report that" — and none returns a
plausible-looking wrong value. The `+checked` `Err` arms in both
`percent_decode` and `ipv4_checksum` are likewise honest: an offset that
overflows means there is no pair to read, which is already a case the program
handles.

## 6. What was run

| artifact | evidence |
| --- | --- |
| `rw/percent_decode.wf` | compiles clean; `pd_diff.py` 691 cases, 0 divergences |
| `rw/ipv4_checksum.wf` | compiles clean; `ipv4_diff.py` 276 even cases, 0 divergences, 0 reference mismatches; 160 odd cases match RFC 1071 |
| `rw/wfgrep.wf` | compiles clean; `wfgrep_diff.py` 228 cases, 0 divergences |
| `probes/p1,p2,p4,p5` | the `:553` arithmetic route does not exist at v0.39 |
| `probes/p6` | Q1 option (b) compiles in isolation |
| `probes/w_nowrite.wf`, `w_noread.wf` | both `shift_input_tail` guards are load-bearing |
| `rw/percent_decode.wf` first draft | the `[EFF-2]` widening (section 1a) |

**Limits.** argv cannot carry a NUL, so the `percent_decode` and
`ipv4_checksum` differentials use NUL-free inputs; the zero-bearing canonical
IPv4 header is covered by each program's own `main` instead. The `wfgrep`
differential compares two builds against each other, not against the reference
oracle in `compiler/tests/programs/wfgrep.rs`; since the original is the build
that oracle already certifies and the two agree byte for byte, that is
equivalent, but it is a step of indirection. Nothing here was run through
`make check`, and nothing was committed.

## 7. Design sentences that must move

Seven, all in 4.5.2, 10 and 11.1. Line numbers are `DESIGN.md` at `236b837f`.

1. **4.5.2, line 3405.** `| wfgrep.wf:469, :495 | probe index inside the input |
   [ENT-5.R] + the imin image | the imin row is 3.5.1 |` — **wrong.** Both sites
   dissolve at v0.39 with no rule, no guard and no branch, by respelling
   `imin(a,r)` + `ieq(a, ·)` as `ile(a, r)`. The row belongs beside `:434` as
   "the guard respelling", with `rw/wfgrep.wf` as its evidence.

2. **4.5.2, line 3407.** `| wfgrep.wf:556 | tail <= bounded_available | the
   unsigned-subtraction image (bucket P) | 3.5.1 |` — **wrong at v0.39.** The
   respelling does not reach it (`probes/w_nowrite.wf`, residual
   `moved < len(deref(input))`). It needs the same restructure as `:553`, which
   `:553` forces anyway. Move it into the `:553` row and stop counting it as a
   bucket-P customer.

3. **4.5.2, line 3401.** `| percent_decode.wf:16 | variable-stride cursor bound |
   [ENT-5.R] + [ENT-3.S5.O] | as utf8parse:18 |` — **understated.** Respelling
   the loop exit `ieq(input_index, source_length)` as `ilt(...)` closes it today
   with no rule; the design's own probe `t4` does exactly this and the table
   does not say so. Note this does *not* transfer to `utf8parse:18` untested.

4. **4.5.2, line 3402.** `| percent_decode.wf:18 | writes <= scan | [ENT-5.R]
   alone | as utf8parse:20 |` — **understated.** Guarding `output_index` against
   `source_length` and letting the existing `requires ige(output_length,
   source_length)` carry it closes the site today, at the price of one `break`.

5. **4.5.2, line 3415, the Score paragraph.** "Four sites are compiled claim-free
   today (`percent_decode:28,31`, `wfgrep:434`, `ipv4_checksum:19,22`) … **One —
   `wfgrep.wf:553` — has no compiled route and two ordinary repairs.**" — the
   count is now ten of ten across the three flagships, and `:553` has a compiled
   route: option (b), compiled in `probes/p6_shift_restructured.wf` and shipped
   in `rw/wfgrep.wf`. What survives verbatim is the narrower true statement:
   `:553` has no *arithmetic* route, confirmed by `probes/p1,p2,p4,p5`.

6. **10, Q1, line 4207.** "**If the rewrite turns out to change what `wfgrep`
   does, that is a finding worth more than the rule.**" — it does not. Q1
   resolves to (b) on both proof and 228 differential cases; the sentence should
   record the answer rather than the open bet.

7. **11.1, line 4297.** "That is evidence, not proof, and F-D4 is the
   experiment." — F-D4 has been run on the three flagships and the tier-three
   count is zero; the tier tally is 8 / 3 / 0. Beside it, 11.1 must gain the
   sentence it is missing: **a guard on a `&uniq` output parameter reads that
   parameter, so `[EFF-2]` widens the function's declared row and the widening
   reaches every caller** (section 1a). The escape — guard a term the contract
   already relates to the parameter's length — exists only for a function that
   has such a contract, and 6.2's mechanical fix channel currently points the
   writer at the widening version.

Nothing found here refutes the design's principle, the deletion, or any of the
four routes it names as compiled (`t4`, `t8`, `t10`, and Q1's option b). Every
one of those held under contact with the whole sources.
