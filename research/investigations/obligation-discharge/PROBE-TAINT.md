# Taint-saturation probe: wfgrep under real system boundaries

Status: falsifier #3 of DOSSIER.md §6, executed 2026-08-06 against
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
