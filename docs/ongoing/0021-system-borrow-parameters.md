# 0021 — Borrow-mode parameters for system nominal types

This is a temporary live coordination record, not execution authority.

- **Status:** `IN PROGRESS`
- **Authority:** `ACTIVE` `docs/current-plan.md` Work item 2 (unsupported
  specified capability, task-0015 finding)
- **Owner / workspace:** executor agent / `worktree-agent-a69e30a5a0e00b887`,
  lead-reviewed
- **Base revision:** `d84643b`

## Goal

Implement `&`/`&uniq` parameters of system nominal types on the normal path
(v0.19 restricts nothing here: SYS-4 kinds take shared/unique borrows;
`semantic/check/types.rs` currently admits non-own parameters only for
buffer/slice/struct/box), then decompose `tests/programs/wfgrep.wf`'s
~500-line `main` into helpers as the composition witness: same oracle, same
§9.1 cost gates, behavior byte-identical. The 35 task-0019 `pending` cases
this capability gates flip back runnable and must pass.

## Progress

- Claimed at `d84643b`; authorities read (v0.19 SYS-2/SYS-4/OWN/FN-1/EFF-2,
  task-0015 finding 1, the §9.1 cost-shape gates).
- **Done — capability** (`ea06412`). Two sites: `semantic/check/types.rs`
  admits the system-resource nominal beside buffer/slice/struct/box, and
  `semantic/check/borrows.rs` gives the OWN-6 child reborrow the case the
  plain borrow already had. Nothing else needed changing — effect
  projection, the checked program, and the backend already treated a
  borrowed opaque resource as the inline value. Regressions in
  `semantic/tests/borrows.rs` (admission, checked representation, both
  row directions) and `semantic/tests/system_effects.rs` (a borrowed
  resource contributes no release row, against the canonical owned case).
- **Done — witness** (`d7a5781`). `wfgrep` gains `publish_all`
  (`&uniq 'o Output`, `&'s buffer<u8>`; the write-until-accepted loop, once,
  five call sites) and `report_failure` (`&'a Args`, `&uniq 'e Output`,
  `&uniq 'r buffer<u8>`). `main` ~500 → ~300 lines; all nine oracle cases
  pass unchanged.
- **§9.1 gates re-derived, not relaxed.** `publish_all` costs 245 against
  the host inliner's 225 threshold and stays out of line at four of its
  five call sites, so level-2 evidence now reads the program's own code
  rather than `main` alone, and the row that counted five emitted `@write`
  sites counts five publication entries (still 2 stdout / 3 stderr). Per
  task 0016's recorded caveat. Rationale in the module doc.
- **Finding — task-0019 cross-link, unresolved.** The 35 cases 0019 flips
  to `pending` naming this capability are **not** gated on it. A/B over the
  corpus adapter (capability reverted vs. present) is byte-identical:
  Pass=242 Fail=123 Skip=14 both ways, same 35 `RegionsAndBorrows`
  failures. None of the 35 names a system type. 29 need borrow-mode
  parameters and `let` borrows of **scalars** (`&'r i32`, `&uniq 'r u64`);
  6 need them of **enum nominals** (`State`, `Result`, `Cell`, `Coin`,
  `Sign`). Both are outside this task's written scope, so they were not
  absorbed. Reproductions: `fn observe ['r](value: &'r i32)` stops at
  `check/types.rs` `parse_parameters_with`; `let p: &'r i32 = &'r a;` stops
  at `check/borrows.rs` `check_borrow`'s trailing arm. The flip-back the
  plan sequences will not turn these green; it needs its own task.

## Validation, stop, and closure

Oracle and cost_shape gates green on the refactored program; the 35 cases
runnable and passing; unpiped gates. Any semantics question the spec does
not settle stops the task. Close to done.

Validation run: `make check` green (compiler gate + corpus structure +
spec append-only), unpiped. `make conformance-run` unchanged at
Pass=242 Fail=123 Skip=14 — the 123 pre-existing failures are exactly the
task-0014 set, neither reduced nor increased. The "35 cases runnable and
passing" closure condition is **not met** and cannot be met by this task;
see the finding above.
