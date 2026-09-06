# 0011 — Target qualification and argument-path lowering

Frozen coordination history. This record reports how the task was carried
out; it is not authority.

- **Status:** `DONE` — landed on main at `61936d6` (ff), 2026-08-06
- **Authority (historical):** `ACTIVE` `docs/current-plan.md` Work item 2,
  wave 5

## Outcome

Whitefoot programs with system interfaces now compile, link, and run. The
static QUAL-1 qualification table (fixed Rust data: spec version × semantic
ID × target × program kind → approved implementation + private ABI symbol,
plus per-type representation and release rows) is consulted once before
layout; an absent/incompatible row or unmet QUAL-2 guarantee is a
TargetQualification failure citing no language rule. The macOS/Linux command
bootstrap establishes command-lifetime argv backing from the stable native
vector (refusing startup otherwise), installs ignored-SIGPIPE once, opens
`command.cwd`, supplies the two Output owners, invokes the entry once, and
maps ExitStatus exactly. Eight operations emit as alwaysinline private
wrappers with one direct call per site (args_count, arg_get under v0.19's
`position`, both byte routes, both UTF-8 routes with one shared complete
validator, relative_path, exit_status); releases emit per SYS-5
(logical/detach = no code; DirectoryRead/ReadFile = one direct close).
§9.1 verified on the optimized module: no surviving wrapper call on lease
paths, no malloc/free/memcpy, no indirect call. Three reviewed rulings:
SYS-8's per-site trap record added `TrapSite` through checked program and IR
(synthesizing a DIAG-3 record would have weakened protected diagnostics);
`command.args` carries the complete native vector including position 0
(HOST-1 losslessness; 0014 reconciles the arrange.argv doc); start failure
exits 71 silently as target-defined behavior the spec deliberately leaves
target-side. Five accept cases compiled, linked, ran, and flipped runnable.

## Evidence and validation

- Landed commits: `c4acbc4` (claim), `61936d6` (implementation); the lead's
  closure change completes the v0.19 derived sweep the 0018 activation
  missed (`tests/conformance/runner.py` corpus pin → v0.19). Both gates
  green by unpiped exit codes; lib tests 391 → 404; witnesses 18/18;
  manifest runnable 338 → 343, pending 14.

## Follow-ups

- 0012 owns the last three operations (open_read/read_once/write_once), the
  sole remaining UnsupportedSystemInterface stop; `OperationRow::NotImplemented`
  marks them, ReadFile's descriptor representation and close already exist.
- 0013 consumes `SystemTarget::probe` and the guarantee-withholding path.
- 0014 reconciles arrange.argv ("after the program name") with the complete
  native vector, and owns the start-failure/status-71 runtime case if the
  catalog wants one.
