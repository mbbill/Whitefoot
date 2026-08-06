# 0014 — First-slice conformance execution

Frozen coordination history. This record reports how the task was carried
out; it is not authority.

- **Status:** `DONE` — landed on main, 2026-08-06; the corpus-wide green
  lane is handed to the owner (pre-existing material, outside task
  authority)
- **Authority (historical):** `ACTIVE` `docs/current-plan.md` Work item 2,
  wave 7

## Outcome

The native execution adapter wires the corpus to the real compiler
(`compiler/tests/conformance*`): same manifest bytes as `runner.py`, stops
mapped honestly (Source → reject+rule, TargetQualification → unsupported,
compiler-unsupported → unsupported so a runnable case fails rather than
passes, everything else a non-verdict), `arrange` realized as a real
invocation (raw-byte fixtures and argv, piped stdin, sink labels as dup'd
descriptions). argv[0] reconciled toward losslessness: `arrange.argv` is
the complete native vector, position 0 included (documented in the schema
comment; no existing entry reinterpreted). 22 runtime cases added, all 22
passing (argument/byte-route/copy-refusal/range-trap, path, open-error with
the full 30-arm match, read shapes including the untouched-suffix witness,
write and one-sink EFF-5 order, exit statuses). Coverage by case 90 → 100
of 119. `make conformance-run` drives the adapter and reports the complete
tally.

## Owner-level finding (first full corpus run: Pass=242 Fail=123 Skip=14)

All 123 failures pre-existing, four buckets, no executor-available move:
1. **45** rejections carry no rule id (rule_id populated only for semantic
   stops; Resolution/Parsing/Lexing/CanonicalSource plumbing) — compiler
   work, next-plan candidate.
2. **41** protected case sources are incomplete units (no `fn main`); FN-7
   masks their declared rules and contradicts two `accept` cases. Owner
   ruling: amend the 41 sources mechanically (protected change) or rule on
   DIAG-1 ordering. The 0017-flagged latent instance was 41.
3. **35** `runnable` overclaims (`RegionsAndBorrows` unsupported);
   per-case evidence exists for a status correction to `pending` — a
   protected status change requiring owner agreement.
4. **2** real divergences to investigate: `gram5-pos-recursive-place-projection`
   (expects run 0, gets TYPE-5 reject) and `type7-neg-propagate-box-holder`
   (expects TYPE-7, gets ERR-3).
The corpus test is `#[ignore]`d with this blocker as its reason; nothing
was flipped, excluded, or weakened.

## Evidence and validation

- Landed commits: `37b3d8c` (claim), `ce7cab9` (adapter + lane), `68117a1`
  (record). Both gates green by unpiped exit codes; `make conformance-run`
  exit 2 reporting the tally above.
- Harness-convention note: 0015's `CompiledProgram::run` uses argv[1..];
  unification is a small follow-up when either is next touched.

## Follow-ups

- The four-bucket owner packet rides the checkpoint report.
- Withheld cases with reasons recorded: open-isdirectory (native open of a
  directory returns Ok; spec fixes no EISDIR surface — lead call on an
  ENOTDIR substitute), nontext filename (APFS EILSEQ), NUL argv (HOST-1
  excludes it), QUAL unsupported (test-only column unreachable from the
  corpus), stdin absence (no stdin operation in SYS-2).
