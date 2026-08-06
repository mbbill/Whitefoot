# Codegen probe: check shape parity and the real cost of retained checks

Status: falsifier #4 of DOSSIER.md §6, executed 2026-08-06 on the real
toolchain (whitefootc `--emit-llvm` → `/usr/bin/clang -O2`, aarch64-darwin).

## Method

`tests/programs/sha256_abc.wf` compiled to textual IR. Two variants built at
-O2: baseline, and a hand-edited IR with all 9 conditional bounds-check
branches (`br i1 … label %array.*.trap.*`) rewritten to unconditional
fall-through — simulating full L1 discharge. Main-loop count raised
1024 → 1048576 in both (identical workload; final checksum check then traps
after the loop, which does not affect loop timing). Alternating runs, 3 each.

## Results

1. **Shape parity is exact by construction.** Today's emitted check is
   `icmp` + `br i1` to a per-site cold `noreturn` trap block — precisely the
   claim construct's compiled form. An adjacent claim compiles to the same
   instructions as today's retained check; the dossier §4.5 fusion
   requirement is trivially met at the adjacent position.
2. **clang -O2 already deletes all nine in-loop bounds checks by its own
   induction (SCEV/indvars).** The optimized baseline contains 5 conditional
   branches — exactly the five loop guards — and a single surviving abort
   path: the program's deliberate final `check` (which is, note, exactly a
   claim). The two binaries differ only in block layout.
3. **Runtime is indistinguishable**: baseline 3.56/3.25/3.32 s vs nochecks
   3.50/3.40/3.27 s over 2^20 hash iterations — noise.

## Implications for the design

- **The design's runtime-cost story is safe at both ends.** Adjacent claims
  cost exactly what today's checks cost, and on induction-friendly scalar
  code the backend already removes what the source-level checker cannot yet
  prove — so migrating to obligation-discharge semantics cannot regress
  scalar performance on this shape.
- **Source-level discharge (L1) buys certificates, not scalar seconds, on
  this workload.** The backend's elision is invisible to the language: it
  yields no trap-freedom judgment, no deny-claims partition membership, and
  it evaporates the moment the loop shape defeats SCEV. What L1 adds over
  the backend is the *machine-checked semantic guarantee* plus freedom for
  transforms the trap-ordering semantics currently blocks (vectorization /
  check motion — the roadmap's DIAG-3 amendment territory), not straight-line
  speed.
- Caveat: one kernel, one target, aarch64 branch predictors eat
  never-taken checks; memory-bound or vector-blocked shapes may differ.
  The claim-consolidation dynamic win measured in SIMULATION.md (5→1 checks
  per extend iteration at L0) is likewise absorbed by the backend here; its
  value case is shapes where SCEV fails.
