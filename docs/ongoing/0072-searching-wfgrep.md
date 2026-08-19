# 0072 — The searching wfgrep

Owner: lead. Workspace: `batch-0072` branch with executor worktrees on
file-disjoint briefs. Base: main at the v0.32 activation.
Registered: 2026-08-18 under the ACTIVE Current Plan (W1, W3, W4, W5).

## Authority

The ACTIVE `docs/current-plan.md` (owner direction 2026-08-18).
Protected surfaces produced here — v0.33 candidate bytes, conformance
families — are marked candidates awaiting the owner's exact-byte
approval; nothing activates on this branch.

## Scope

- W1: the file-open-by-name operation (spec delta + implementation
  behind a default-off switch + running evidence), then wfgrep's real
  recursive search over the v0.32 traversal surface.
- W3: one v0.33 candidate composed by the lead — strict-in-U clause
  retirement, the Linux enumeration disposition, and W1's surface.
- W4: the migrate-tool deletion, deferred renames, attribution rulings.
- W5: adversarial exit audit and the owner packet.

W2 (flagship re-attribution) opens only after W1's search runs, and may
ride this batch or the next depending on what W1 measures.

## Out of scope

Everything the plan excludes; any activation; any unmarked protected
change; merging to main. Executors do not edit `spec/kernel-spec.md`,
`tests/conformance/`, `governance/`, or the plan and roadmap.

## Morning review (2026-08-19, batch end)

Branch `batch-0072`, 24 commits over main. Full `make check` exit 0 at the
tip (verified by exit code, not by reading the log tail).

### W1 — the searching wfgrep

**READ THIS FIRST — the search is bounded, and the bounds truncate
silently.** `walk` retains at most 64 entries per directory, in the
host's enumeration order, before it sorts them; and it leaves any
subtree deeper than 16 levels unsearched. Both bounds return normally:
exit status 0, empty stderr, output that looks like a clean sorted
prefix and is not one. The exit audit found this; it was in no doc
clause, no delta, and no test, while the batch's own headline claimed
agreement with `grep -rn`. The doc clauses now state both bounds.

Two consequences the owner must weigh before W2 runs:
- The frozen flagship corpus is 4,096 files in ONE directory
  (`research/experiments/wfgrep-baseline/PROTOCOL.md`). **The
  re-attribution against ripgrep cannot run honestly until the bound is
  gone.**
- Raising 64 to a larger constant would move the lie, not remove it.
  The real repair is to collect entries through the growable layer this
  project already has, so the only bound is memory. That is the first
  item of the next batch, not a patch here.

`tests/programs/wfgrep.wf` was an argv-fed searcher: an outside caller
passed the file list. It is now a real recursive search. The entry takes
the working-directory capability, `open_list`/`list_once` enumerate,
`open_directory` descends, and the new `open_file` opens each regular
file by its enumerated name bytes. Two independent oracles check it: a
reference search written in the harness, and the host's own `grep -rn`
over the same fixture tree. A symbolic-link case pins that an enumerated
link is not followed.

`open_file['c, 'n](root: &'c DirectoryRead, name: &'n buffer<u8>,
offset: own u64, count: own u64) -> own Result<ReadFile, IoError>` is
`open_directory`'s twin: the same [SYS-8] range validation (trapping),
the same single-component content validation (recoverable `InvalidPath`),
the same darwin binding route; only the product differs. It sits behind
`OPEN_BY_NAME`, default false, and its delta is drafted at
`research/investigations/searching-wfgrep/SPEC-DELTA.md`.

UNDER ACTIVE v0.32 WFGREP DOES NOT COMPILE, and a test pins that
rejection. The search exists only under the candidate inventory, which is
the honest state until the owner approves the bytes.

### W3 — v0.33 residue

Strict-in-U retirement: nine probes failed to reach the [OP-4]/[OP-2]
strict rejections, so the three diagnostic variants left the public
surface while the U-obligation scan stayed as an internal-consistency
guard — violating it is now a compiler failure, not a source rejection,
and the accepted set does not move. Linux enumeration: the
recommendation is to keep it unmapped, and the investigation corrected
the stated reason — Linux does supply [QUAL-2]'s third guarantee
(`getdents64` is exactly a bounded batch advancing the directory's own
position); what is absent is the approved-implementation row, which
[QUAL-1] makes a different stop. The compiler reports `UnmetGuarantee`
where it should report `MissingMapping`, its own comment already says
`MissingMapping`, and the derivation-ledger row repeats the conflation.
That correction is NOT landed: it belongs with the spec bytes.

The division goal-matching question the owner raised became a probe
report rather than a delta, per his deferral of arithmetic-trap work
until a systematic audit of every trap site.

### W4 — residue

`whitefoot-migrate` deleted (2503 lines, 5 files, 36 unit tests, its
Cargo.toml entry and four `migrate: keep` site markers), with no live
consumer anywhere. Six conformance cases renamed with every verdict
verified unchanged. The adapter's ignore-reason tally corrected from the
stale 460 to the measured 489; the executor flagged that it moved one
more token than instructed, because 489 was never measured at the v0.31
candidate and correcting only the number would replace one false
statement with another. Attribution inventory recorded.

### Evidence changes: the lead's were reverted, the executor's stand

The lead made three test-evidence edits believing the executor had died
mid-task. It had not; it finished and took the option the lead had
preferred, so all three lead edits are reverted and the executor's
replace them:

- The frozen A10 section is KEPT, not removed. The executor rebuilt
  `report_failure` to assemble one diagnostic and clamp it with a
  `value_if`, so the postcondition delivery route keeps its real-program
  witness. The shape earns itself independently: one host write instead
  of three, so the pieces cannot separate in a shared sink. THERE IS NO
  COVERAGE LOSS.
- The frozen receiver-route count is 11, derived from the restored
  source's eleven `append_slice` call sites — not the lead's 10, which
  was derived before the restoration.
- wfgrep leaves the earlier-program inventory differential in the
  executor's own commit, for the same reason: it now requires the
  traversal rows.

The lead's premature edits and their reverts are on the branch as
history rather than rewritten, so the exit audit can see both.

### Honest state

W5's audit is the two finders running at closure. The v0.33 candidate is
NOT composed: three delta documents exist (open_file, strict retirement,
Linux disposition) and are held so the contract-surface decision can
compose into the same candidate rather than forcing two activations.

## Owner approval packet

THE SINGLE ACT: approve batch 0072 = merge `batch-0072` to main. No
specification byte moves; nothing activates.

Decisions folded into that review:

1. **The silent search bounds** (above). Ruling wanted: merge with the
   bounds disclosed and repair them first in the next batch, or hold the
   merge until `walk` collects through the growable layer.
2. **A misattributed failure class.** `io_class` maps `BrokenPipe` to 4,
   and `report_failure` names 1, 2, 3, 5, 6 — so 4 falls through to
   "cannot read". A STANDARD OUTPUT write failure inside `search_file`
   is therefore reported as the INPUT FILE being unreadable. The two
   retired diagnostics (`broken pipe`, `write error`) went with the
   publication loop they belonged to. Ruling wanted: accept the coarser
   reporting, or restore the classes.
3. **Two wfgrep diagnostics retired.** `wfgrep: broken pipe` and
   `wfgrep: write error` went with the publication loop they belonged
   to; a failed publication now reads `wfgrep: PATH: cannot read`. Their
   constants were deleted rather than left unread. Ruling wanted: accept
   the coarser error reporting, or restore the two cases.
4. **Protected surface:** six conformance renames (verdicts verified
   unchanged) and the adapter ignore-reason tally correction.
5. **The Linux qualification correction** is specified but not landed;
   it rides the v0.33 candidate.
6. **The attribution gaps** the batch inventoried: OWN-6 has zero
   reject-citing cases while its `InvalidChildReborrow` surface has three
   separable conditions pinned only by lib tests; OP-5 has one citing
   case while 42 cases exercise its condition judgment. Both need case
   additions the lead did not make unilaterally.

7. **Un-scoped research landed on this branch, and it now gates a scoped
   deliverable.** Five contract-surface documents (~300 KB) were written
   under owner direction mid-batch, under no plan item, and the record
   then cites the pending contract decision as the reason W3's v0.33
   candidate was not composed. The documents are split onto main; the
   sequencing decision is the owner's: compose v0.33 now from the three
   deltas that exist, or hold it for the contract surface.
8. **A compiler-ahead-of-spec window.** ACTIVE v0.32 still states the
   strict-in-U OP-2 and OP-4 rejections in writing; the batch removed
   the arms that emit them and kept the invariant as an internal
   consistency guard. The accepted set cannot move, but a program v0.32
   requires be rejected as invalid source would now surface as a
   compiler failure. The window closes when the v0.33 candidate lands.

### Known defects carried, not fixed

- `report_failure` open-codes an 18-line copy loop that duplicates
  `copy_range`, which is declared in the same file and already used for
  exactly this job. The repair touches the proof structure the A10
  witness depends on and needs its own verification round.
- The `value_if` clamp's else branch is unreachable at runtime: the copy
  loop already bounds the length. It is a proof device for the following
  `append_slice`, which is legitimate, but the batch described the shape
  as better code without saying so.
- Five pre-batch wfgrep test cases retired with no replacement; the
  behaviours they covered were verified still working, so this is
  coverage loss rather than regression.
