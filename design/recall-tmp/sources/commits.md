## 65b3d24 2026-08-28 wip: TYPE-5 and OWN-10 publish the two sides they compared


## 43d6491 2026-08-28 wip: OWN-6 says the whole idiom; FORM-2 quotes the line its gap ends in


## 048ef6b 2026-08-28 wip: report every denied condition of a denied loop, honestly


## 423e85d 2026-08-28 wip: bring the test suite to the new payloads


## f4e6828 2026-08-28 wip: pin every new payload by its exact rendered text


## 67e9190 2026-08-28 wip: P15 states the walker resolution and drops the notice overclaim


## fd65823 2026-08-28 docs: record batch 0100


## c4ea04b 2026-08-28 docs: keep D2 open and say what item 9 actually closes


## 012ebb6 2026-08-28 style: rustfmt


## 33e93d8 2026-08-28 style: lift the FN-3 bound sentence out of the call


## 7aa6819 2026-08-28 diagnostics: spell a region in a rendered type the way the source spells it


## 917f79e 2026-08-28 docs: record the gate and CI results of the final commit


## 60cbc48 2026-08-28 record: state where the checks run, and what the init and clock comments really claim

WIP checkpoint (C7, C3 rest, C4/C5/C6 record side).

## cf4b325 2026-08-28 tests: require one branch of the demand-driven policy, not the decline specifically

The decline turns on a measurement, so requiring it requires an idle host:
the same sanitized binary declines 15814 of 16000 reads when this machine is
quiet and declines none while growing three helpers when it is loaded.

WIP checkpoint (C2 revision).

## 7773832 2026-08-28 record: trim the confirmed-table comparison to the page width


## f073dcf 2026-08-28 record: add the two test-design judgment calls and fix a cross-reference


## 261070c 2026-08-28 record: say where the two C.wide8.h4 before-cells come from


## 17cf599 2026-08-28 record: list the shutdown-order change with the other tip changes


## dfb0a30 2026-08-28 diagnostics: teach the four remaining bad defaults the verification writer met

FORM-3 name slots, GRAM-2's contract-block order, TYPE-6's four colliding
situations, and SYS-8's second residual now say what the writer has to do.
Work in progress: the payload tests come next.

## 0e0291a 2026-08-28 record: read the two cold rows from the confirmed draw the repair's own run produced

WIP checkpoint (handoff side).

## f0859f2 2026-08-28 record: carry the repaired runtime's draw into RESULTS as well


## 803db38 2026-08-28 record: point the earlier cache-label paragraph at the draw that grades the cold rows


## 124eaac 2026-08-28 record: count the level-or-ahead Linux readings correctly


## fc1b9b0 2026-08-28 tests: pin every diagnostic sentence by a probe that renders it

compiler/src/driver/pinned_sentences.rs is one table-driven test over a
corpus of minimal sources: each row names the rule its rejection must cite
and the exact rendered fragments it must carry. Adding a sentence means
adding a row. SYS-2 also stopped asking the resolver for a region use a
type or const argument never records, which turned an internal failure into
the source rejection the position always had.

## 39da854 2026-08-28 docs: quote the condition-3 sentence the compiler prints today

Also corrects three records of the batch that did not match the code: the
assert_rule_kind doc comment named a module that does not exist, the EFF-1
condition comment sat on the TYPE-5 constants and miscounted the conditions,
and SyntaxIssue::mechanical_fix had no caller.

## 4ff5ea8 2026-08-28 tests: name the caller's buffer in the two SYS-8 residual assertions


## a408772 2026-08-28 docs: record the second round of batch 0100


## ce47697 2026-08-28 docs: record the second round's gate and CI results


## c8b8618 2026-08-28 docs: correct the probe-corpus counts in the module header and the record


## 14af15c 2026-08-28 docs: say what the commits after the local gate run change


## efdf130 2026-08-28 docs: say why the literal count differs from the verifier's, and name the function correctly


## 7e15b73 2026-08-28 record: one mechanical table of every io-bench draw on this branch

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>

## 76a1271 2026-08-28 wip: rewrite the counting prose against the mechanical draw table


## 5e4bf20 2026-08-28 wip: correct the corpus completion-call census in both records


## 283be40 2026-08-28 record: grade the macOS cold rows against all nine draws


## cf1ae06 2026-08-28 record: point the handoff at the complete draw table for the macOS cold grade


## 34d7db8 2026-08-28 record: read the Linux draws in the handoff off the complete table


## dfd7a54 2026-08-28 record: rewrap the repaired-runtime draw paragraphs


## b7f9205 2026-08-28 record: attribute the cold column's stage sums to the cold column


## 15467d5 2026-08-28 record: fix the follow-up delta table and the counter and label trivia


## 0ad0b4b 2026-08-28 record: rewrap the edited paragraphs


## 9294224 2026-08-28 record: describe the default-route probe's negative control as measured

The record claimed that forcing the decline off fails the probe on every
run. The assertion is a disjunction, so removing one disjunct leaves the
other live: measured six runs of each of three builds under eight
spinning threads, the one-branch mutation passed one run of six on the
growth branch and only the two-branch mutation failed all six.

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>

## cc7affd 2026-08-28 record: read the handoff's Linux draw claims off the complete table

The handoff still counted draws in prose -- 'the two prior Linux draws',
'all three Linux draws report four CPUs', 'the fourth macOS draw'. Each
is replaced by what the complete draw table says: the eight-row warm
table with its narrow control, the CPU count of every Linux runner in
it, and the position of the a06c53f9 draw among the nine.

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>

## 8bf9710 2026-08-28 record: stop numbering draws in RESULTS' Linux and macOS follow-up prose

'a fourth macOS draw', 'all three Linux draws report 4 CPUs', 'two prior
draws that agreed' and the heading 'A fourth Linux draw' each counted
draws that the complete table above them already enumerates. Each now
cites the table's position, range or CPU column instead.

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>

## f2cd2b0 2026-08-28 docs: record the gate run at the branch tip and the repeated pinning proofs

`make check` is now green at `efdf130f` itself, not only at the revision two
doc comments earlier, so the Gate results section carries that run's stage
times and says which commit follows it.

Two further mutations of the pinning corpus are recorded beside the first:
shortening `GRAM2_CONTRACT_ORDER_FIX` and deleting the closing clause of
`COLLIDES_WITH_LIVE_OUTER` each fail the table naming their own probe. Three
literals in three files under three rules is the evidence that the table
fails on a wording change wherever the wording lives.

The coverage claim gains the four enumerations behind it: they disagree on how
many literals the batch adds — 88, 98, 99, 107 — because each counts a
different thing, and they agree on the set no test asserts, which is the seven
documented unreachable arms.

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>

## 55df38e 2026-08-28 docs: say the IR oracle was re-run at the branch tip and agrees to the number

The 630-file comparison against the base compiler was run again on the tip:
`files=630 exit-status-differences=0 ir-differences=0 stderr-differences=141`,
the same line all three rounds produce, and the `[RULE-N]` set of each of the
141 differing files is identical before and after. So every accepted source
under `tests/programs`, `tests/codegen`, and `tests/conformance/cases` still
emits the base's IR byte for byte, and the only thing this batch changed about
them is what the compiler says.

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>

## 1622821 2026-08-28 docs: cite the P15 ledger lines without unreachable probe coordinates

The two measured blocks quoted ledger lines by the file and line of probe
programs that are not in the repository, so a reader could not check the
coordinates. The file and line were never the point; the verdict text is.

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>

## d182898 2026-08-28 record: correct the round-4 log findings

Every Linux warm draw on this branch, not every one that exists (two
0092-era draws live under their own heading); the helper ceiling is the
fixed WF_BRIDGE_MAX_HELPERS of eight, not the operation bound; four
rounded ratios re-derived; three of six loaded runs took both branches;
the cold 4 KiB stage column exists and reads the same way.

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>

## 10b76c6 2026-08-28 Merge branch 'batch/0096-darwin-handoff' into integration/2026-08-28b

# Conflicts:
#	compiler/Makefile
#	compiler/src/backend/completion/bridge.c

## 8a18f65 2026-08-28 io: quiet the denied-loop notes under --no-overlap, and teach the buffer form

Two owner decisions on observations batch 0100 recorded and left open.

B5: an ordinary compile has printed every denied I/O-loop verdict to stderr
since 0099, and a build that wrote `--no-overlap` printed them too. That flag
has already said this build takes no overlap lowering, so a loop without a
pipeline is the build it asked for rather than news about the program. The
decision is by the flag, not by the text of a line: `io_notice_report` returns
the lines to print and returns none of them under it. `--par-ledger` is
untouched and prints the whole report under every lowering. Pinned by
`a_no_overlap_build_reports_no_denied_io_loop`, which compiles one
denied-output-loop source the three ways, asserts the notes are present by
default and under `--par` and absent under `--no-overlap`, and asserts the
three modules and the three verdict lists are identical -- the judgment is
pure, and a denied loop reaches the host through direct calls either way.

D1's writer half: `docs/patterns.md` P18 teaches the explicit form. Where a
writer would hold a `&uniq` resource inside a loop body, the loop holds a
buffer instead, each iteration folds its line into it with an element `set`,
and one write after the loop publishes it. The denied form's condition-3 line
and the buffered form's `permitted` line are quoted byte-exact from the branch
compiler; a helper that takes `&uniq` of the page is denied by condition 4
instead, which the entry states. The runtime half -- a final-stage write under
in-order commit -- belongs to 0095 Stage B and is untouched.

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>

## 100b37c 2026-08-28 Merge branch 'batch/0103-quiet-no-overlap' into integration/2026-08-28c


## 867fa55 2026-08-28 WIP: batch 0102 CLM-1 narrowing marker


## 061528d 2026-08-28 CLM-1: narrow claim authority's control dependence to selected definitions

Claim authority no longer marks every definition made under a
boundary-selected edge.  Each value component carries the identity of the
reaching definition that produced it, and a control-flow merge joins the
selector's witness into exactly those components whose incoming reaching
definitions are different definition occurrences.  A definition whose own
operands are all local is local wherever it stands.

Ten library tests that assert the repealed clause still fail; they are
rewritten with the specification amendment in the next commit.

## 25500d9 2026-08-28 spec: activate v0.39, narrowing CLM-1 claim authority to selected definitions

v0.38 is archived byte-exact as spec/kernel-spec-v0.38.md and the chain,
the generated identity, the transcribed literals, the qualification review
note, the derivation ledger, and every digest anchor name v0.39 at
4be4830fa87a534879de17524599b0919aef4dfab072dad823bf2f9b54d32d58.

## 86a4fd9 2026-08-28 conformance and tests for the v0.39 CLM-1 narrowing

Seven conformance cases: three accept (the differential-fuzz minimized pair
and a claim inside a selected arm) and four reject (a claim on the selected
payload, on a value_if delivery, on storage one arm wrote, and on
loop-carried state a boundary-selected loop updated). No existing verdict
changes.

Ten library tests that asserted the repealed position clause are rewritten
to the verdict v0.39 gives, and six new tests pin the selections the rule
retains.

## 3c0625c 2026-08-28 record, plan, and roadmap for the v0.39 CLM-1 narrowing

Adds docs/done/0102-clm1-narrow.md, the conformance boundary in the
merge-time record, the live version references, and a dated follow-up in
0097's record pointing at the ruling it asked for.

## 4dca391 2026-08-28 WIP 0105: bridge spin comment and shutdown precondition wording


## 537a05e 2026-08-28 spec: remove a contradiction in the amended CLM-1 paragraph

The sentence stating what stays Local named 'binder' and 'delivered value',
which the preceding sentence had just made unconditional selections. It now
names an ordinary binding, a computed value, or a storage write. The chain,
the two transcribed literals, the generated identity, and the six anchors
follow the new digest
b4d8e01eecd81bdda9c632093873d604ddfbd64d979a4884472907e456d69516.

## ee898f1 2026-08-28 WIP 0105: completion test for the post-shutdown guard refusals


## 4e7a03a 2026-08-28 patterns: P14 gains the v0.39 claim-locality narrowing


## 31e0c39 2026-08-28 WIP 0105: test-only WF_IO_NO_NATIVE_RING and the forced-adapter probe arm


## 227551f 2026-08-28 docs: batch 0105 record


## 2db3521 2026-08-28 record: the base-versus-new verdict table for the seven added cases


## 6bcfa6f 2026-08-28 runtime: measure the clock-failure spin on both Linux routes and say what the guard actually buys


## f66390f 2026-08-28 record: the regenerated fuzzer numbers, and drop the WIP marker

Base and narrowed compilers each ran a 200-program seed-1 campaign on this
host. The base refuses 5 of 208 attempts, all CLM-1; the narrowed compiler
refuses none of 203. Recompiling the 203 the narrowed compiler accepted with
the base compiler refuses exactly 5, all NonLocalClaim. Neither run found a
divergence. The minimized pair's admitted member publishes identical bytes
across 27 runs of the overlap matrix and --no-overlap.

## 2e85629 2026-08-28 CLM-1: discharge a selector's frame at the merge that joins its edges

The first cut of this judgment deleted frame discharge outright. That is
right for merges downstream of the construct and wrong for an enclosing
loop head, whose entry state predates it: with the frame retained, the loop
head attributed its own selection to a match that had already reconverged,
and tests/programs/wide_scan.wf lost a claim v0.38 admits.

A frame is now dropped at the merge that joins every edge its selector
produces, after that merge has read it. For an exhaustive reconvergence and
a counted loop with no escaping edge that is v0.38's condition verbatim.
For a match whose arms all break to one loop the joining merge is the
loop's exit, so that discharge moves there: walk_loop and walk_counted drop
every frame their own exit merge acquired.

## fbfe0a2 2026-08-28 record: note the campaign was repeated after the discharge repair


## c59fa63 2026-08-28 record: the make check result and the three environmental host failures


## 8ec4532 2026-08-28 docs: state the spin-bound measurements as draws and drop the route exemption

The gate verifier of this batch refuted the rewritten comment's own
route contrast: the forced-adapter route also hung an unbounded spin
(2 of its 44 runs, one STUCK at the probe's 180 s watchdog), and its
counters show helpers=0 in most runs, so the route usually finishes
because declined reads run inline, not because a helper publishes.
The spin comment now states the mechanism for both routes and labels
every count as a draw; the record carries both samples and the
corrected reasoning; the shutdown-header justification names the
lock-taking entry points instead of quantifying over all of them; the
section-2 heading no longer claims the ordering itself is test-covered
(its evidence remains the ThreadSanitizer report).

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 483a84c 2026-08-28 docs: the summary line carries the full refutation; name the operation path

The round-2 log verification confirmed every reworded claim and caught
the one sentence the reword had left behind: the record's summary still
narrowed the refutation to the attribution while section 1 now refutes
the route contrast too. The header's 'a delivered program reaches on
its own' also tightens to the writer's operation path, since the
bridge's own scheduler reaches the queue reader directly.

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 5e39b5b 2026-08-28 docs: justify the shutdown pair by its windows, not by reachability

The writer's operation path reaches five lock-taking entry points
through the join shims, so no reachability criterion picks out the two
the header details. The sentence now says what the two paragraphs
below demonstrate: these are the two windows worth spelling out, not
the only two a program can be inside.

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 660f73e 2026-08-28 docs: the decline check is described for its window's shape, not counted

Same defect one paragraph lower: 'the other entry a delivered program
reaches without holding anything' re-asserted the two-entry
reachability count the previous commit removed. The emitted join loop
reaches the queue reader and the progress entry the same way, so the
sentence now says only what distinguishes the paragraph: the window's
shape.

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 5e8a243 2026-08-28 docs: re-wrap the shutdown comment at the block's width

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## f642975 2026-08-28 record: correct the corpus-sweep scope, the discharge premise, and two citations

The gate verifier's five findings, none touching the judgment: the
program-corpus bullet now says which 26 files the implementer's sweep
compiled and that the verifier's own 120-file sweep found the same
identity; the analysis-time deferral no longer claims frames are never
removed (they are discharged at their construct's closing merge, the
ControlAuthority doc comment now says the same instead of the
opposite); the fuzzer table's narrowed wall clock is the post-repair
run's 3.0 minutes; the wide_scan citation points at the Rust harness
the program is embedded in. The skeptic's two latent findings
(DefinitionId's unenforced preconditions, the unpinned carrier
tie-break) are recorded under Not done.

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## e8a0ca3 2026-08-29 record: name the construct wide_scan actually holds, and the real scan cost

The record re-verification found the discharge counterexample's
structural description false against the artifact: wide_scan.rs holds
no counted loop and its arg_get sits ahead of every loop; the
structures that match the stated mechanism are the exhaustive
publish_all matches with no escaping edge inside the ordinary loop
@hostile_walk. The analysis-cost sentence now states the product shape
instead of calling the scan linear, and the DefinitionId bullet
attributes its finding to the skeptic's report, which is where it
lives.

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 7fa8f9b 2026-08-29 record: the discharge counterexample cites the mechanism, not a program

The record re-verification showed the cited program cannot exhibit the
first attempt's failure (its loops hold no downstream claim and its
in-loop matches mint no boundary witness), and no artifact of that
failure survives. The paragraph now states the mechanism alone, which
is verifiable from the code, and says the first failure left no
artifact. The cost sentence counts the scan per frame an edge carries,
and the two bullets are refilled at the file's width.

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 7393e62 2026-08-29 Merge branch 'batch/0102-clm1-narrow' into integration/2026-08-28c


## b1367c8 2026-08-29 Merge branch 'batch/0105-bridge-comment-and-shutdown-coverage' into integration/2026-08-28c


## fba8c0f 2026-08-29 io: the all-skipped exit says which reason it was

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 2d44e2a 2026-08-29 record: the two counts in the shortfall paragraph are the ones measured

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## fe4a877 2026-08-29 record: what made the shortfall commoner was contention, said exactly

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## e3c77c2 2026-08-29 record: which other movements weaken an answer, and why each is silent

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 3e51ab7 2026-08-29 record: the shortfall counts are the full sweep's

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 17fd7ec 2026-08-29 record: reflow the shortfall paragraph

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## eb3ba6c 2026-08-29 io: an open that took a returned descriptor is charged for it

WIP.

## c2ffaef 2026-08-29 io: three scripted cases for the charge rule

WIP.

## a3e1123 2026-08-29 io: the award mark never moves backwards, and a take is charged whether or not a waiter is registered

WIP.

## 6adbf70 2026-08-29 record: the award mark's two defects, the repair, and the sweep that measures it

WIP.

## d38e4af 2026-08-29 io: a returned descriptor is promised to exactly one consumer

The interleave probe's rare shortfall was the award mark being told two
different things and neither of them all of the truth.

The mark reopened on every waiter that found the order empty, back to a
`seen` its own attempt had read — and on the ring that is the moment of
submission, long before the kernel refuses the open and longer still
before the refusal is reaped.  Refused opens therefore arrive late and
one at a time, each finding the order empty because the one before it
has left to spend its award, and one returned descriptor was awarded to
three of them in turn.  The mark now only ever rises.

An open the host satisfies takes a descriptor whether or not it ever
waited for one, and it was charged for nothing, so a return it had
carried off was still on offer to a refused open.  Every satisfied open
now reports it, on all three routes, with the re-attempt a waiter makes
on an award charged once rather than twice.

Four scripted harness cases, and the shipped probe now checks the
ledger's promise beside its count of published opens.

## cf60e5e 2026-08-29 record: the charge's own adversarial schedule, built and measured

WIP.

## 2bed9cc 2026-08-29 wip: the pipeline descriptor carries a slot count and each block's slot parameter


## 0a58c51 2026-08-29 wip: a ring element is addressed from the slot the block names


## 7fe14be 2026-08-29 io: a staged region's completion storage is a ring, addressed by the slot its block names


## 0b7ea8c 2026-08-29 record: slot-indexed completion storage, and what Stage B still owes


## 2d57982 2026-08-29 record: a drain retires one element of each ring, and what that leaves the driver


## cbb867a 2026-08-29 test: the comment names the one handed-out site


## b750a43 2026-08-29 record: the ring changes no emitted module, measured against the pre-ring compiler


## c09c1ed 2026-08-29 record: what the emitter actually checks, what the ring tests actually show

Two shipped comments overclaimed and the branch's adversarial verification
established four things the record did not carry. No mechanism changes: the
diff outside docs/ is comment lines only.

- `BackendFailure::MisaddressedCompletionSlot`'s rustdoc said a slot is refused
  unless it is one of the naming block's own `u64` parameters. It is not:
  `validate_pipeline_slots` checks that the named value is a `u64` value of the
  function, and trusts both dominance and range against the ring width, as its
  own sibling comment already said honestly.
- The ring tests' helper doc said the descriptor is one a driver could produce.
  The slot it resolves is the probe's own `rounds` argument, loop-invariant and
  carried unchanged around the loop, and the staged module it emits does not
  pass `llvm-as`. The doc now says both, and says what the five tests do
  establish: the reservation shape, per-slot addressing, one-slot storage equal
  to the unstaged program's, and the three refusals.
- `docs/ongoing/0095-loop-pipeline.md` gains "What the Stage B verifiers
  established beyond the range": the failing verification reproduced here at
  this revision with its artifacts, the unbounded slot index, the per-operation
  SSA facts that are not ringed, and the untested staged-component ring — plus
  the range's own commit count and diffstat, and the plain statement that the
  driver is shelved by owner sequencing and this range has nothing for the
  io-bench runners to measure.

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## a7baf66 2026-08-29 record: say exactly what the helper resolves and what the emitter leaves

The previous round's honesty pass overclaimed in six places of its own. All of
these are comment, doc and record lines; the diff outside docs/ is `///` lines
only and no mechanism changes.

- `a_slot_index_for`'s doc said the probe has only one dominating `u64`
  parameter. The loop header has six parameters, five of them `u64` — the
  carried `rounds`, the running total, and the loop's index and bounds. The
  helper takes the *first* `u64` of that header, which is the carried copy of
  `rounds`; the doc now says that.
- `validate_pipeline_slots`'s doc said its two checks are between them the
  whole of what makes a run-time-chosen element safe. Range against the ring
  width is a third thing, trusted and unchecked: an out-of-range slot is an
  out-of-range `getelementptr inbounds` handed to the runtime as a write
  address. Both the summary and the trust paragraph now say so.
- `MisaddressedCompletionSlot`'s third refusal was broader than the code.
  `completion_entry_slot` refuses only with a ring in force and a carrying
  block; a one-slot descriptor has no ring element and refuses a missing slot
  nowhere. The rustdoc now carries both qualifiers.
- Record finding (c): the un-ringed SSA facts list omitted `DirectoryNext`'s
  destination. It is now exhaustive and cites the emitter lines.
- Record finding (c): the cited value names are the four-slot module's; the
  same three errors name `%t40`/`%t31` in the one-slot module. Both are now
  named, with the shape stated as identical.
- The "234 executions" figure was a formula no artifact prints. The logs hold
  270 md5 cells over 30 rows; 234 is the distinct-row count, now labelled as
  such. The matrix's identity is also narrowed to what it shows: the nine cells
  of a row agree, while the two `p06` rows at one descriptor limit do not agree
  across the two logs.
- Hygiene: the record no longer pins this host's ephemeral scratch root. The
  reproduction recipe and the bare artifact filenames stay.

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 6302aee 2026-08-29 record: the refusal bullet, the trust sentence, and the citation match the verifier's re-derivation

The three residual wording findings of the round-12 re-verification,
applied as stated: the record's third refusal bullet carries the same
ring-in-force and carrying qualifiers as the enum rustdoc, and states the
two unrefused cases (one-slot descriptor; non-carrying block under a
multi-slot descriptor); the trust sentence names range beside dominance
instead of claiming two halves are the whole; the DirectoryNext
destination citation narrows to its own line. The enum rustdoc gains the
same non-carrying sentence and loses the ragged wrap. Comment and record
lines only; no behavior.

## 6e93e82 2026-08-29 Merge branch 'batch/0095-loop-pipeline' into integration/2026-08-28c

One conflict, the shutdown paragraph of file_adapter.h: resolved to
0095's wording because the merged code is 0095's -- the decline check's
wf_file_adapter_queued is a plain atomic load of queue_count and takes
no lock (file_adapter.c), so 0105's takes-the-queue-lock sentence
describes code that no longer exists.

## 9cc3b90 2026-08-29 io: the interleave probe declares its locals on the branch that uses them

CI static (macos-14) refused the probe: main declared eight locals
before the non-Linux early return, so on a host with no kernel ring
every one of them was unused under -Werror (the miscounted counter is
round 10's, which is why this first fires now). The declarations move
into the Linux branch. Checked both preprocessor branches locally with
the gate's own warning set (-fsyntax-only, and -U__linux__ for the
non-Linux shape), and swept every completion .c the same way: clean.

## dd5a1cb 2026-08-29 Merge branch 'batch/0095-loop-pipeline' into integration/2026-08-28c

The macos-14 static fix: the interleave probe's locals move into the
Linux branch of main, closing the eight -Werror unused-variable errors
on the host with no kernel ring.

## d40ff17 2026-08-29 io: the interleave probe's helpers live inside the Linux guard

The second layer of the macos-14 static refusal: with main's non-Linux
branch a pure early return, the watchdog, the repetition body and the
warm-up became unused functions there, errors clang had suppressed
behind the first batch. The whole helper block through the overlay
helpers now sits in one #if defined(__linux__) region; the non-Linux
translation unit is the includes, the defines and the skipping main.
Checked with clang -c (not -fsyntax-only, which is why the first sweep
missed this: gcc's syntax-only pass skips end-of-unit unused-symbol
diagnostics) under -U__linux__ and natively, plus gcc natively, and the
same clang sweep over every completion source: clean.

## ddf358a 2026-08-29 Merge branch 'batch/0095-loop-pipeline' into integration/2026-08-28c

Second macos-14 static fix: the interleave probe's Linux-only helpers
move inside the Linux guard.

## 4b08c75 2026-08-29 Merge pull request #1 from mbbill/integration/2026-08-28c

Integration 2026-08-28c: batches 0103, 0102 (spec v0.39), 0105, 0095
## b4eddb1 2026-08-30 Complete the Windows IOCP runtime backend


## a8d9e7c 2026-08-30 Make Windows IOCP E2E failures observable


## 8e83c85 2026-08-30 Surface Windows driver link diagnostics in CI


## 1f05c39 2026-08-30 Preserve Whitefoot source bytes on Windows


## 853076d 2026-08-30 Qualify the native Windows runtime row


## f04e15c 2026-08-30 Launch the HostString probe through native Windows


## dbdbc30 2026-08-30 Record exact Windows runtime evidence


## c88baf0 2026-08-30 Add fail-closed Windows compute workers


## 0de3613 2026-08-30 Use the native Windows worker setting API


## 097a906 2026-08-30 Link Windows address-wait synchronization


## cac425a 2026-08-30 Resolve Windows address-wait APIs at startup


## 7d36e7f 2026-08-31 Preserve Windows worker failure diagnostics


## dd8c66e 2026-08-31 Resolve address waits through the Windows API set


## cf69135 2026-09-01 Implement source-carried proof compiler


## c273df8 2026-09-01 Remove stale claim guidance


## 010126f 2026-09-01 Preserve bounded affine facts across joins


## aaf25db 2026-09-01 Extend deterministic affine consequence checking


## d5fd209 2026-09-01 Derive unsigned literal division bounds


## df0c4e2 2026-09-01 Preserve bounds through killed graph vertices


## 9bc7751 2026-09-01 Add conformance cases for deterministic proof rules


## 0592f37 2026-09-01 Generalize source loop invariants


## 76bb3e7 2026-09-01 Harden source proof composition and joins


## 0962fd6 2026-09-01 Make affine pair inference monotone


## b1aa041 2026-09-02 Implement explicit local invariant certificates


## 65d81c4 2026-09-02 Add explicit loop invariant headers


## 4528e37 2026-09-02 Report exact loop invariant edge targets


## e5841b2 2026-09-02 Wire explicit loop headers into induction


## 0488118 2026-09-02 Wire local invariant certificates


## 4622735 2026-09-02 Finish loop invariant header integration


## 8099135 2026-09-02 Make Windows completion pressure fail closed


## 27e0f15 2026-09-02 Fix the Windows capacity gate link inputs


## eaa6e59 2026-09-02 Qualify the Windows IOCP runtime


## 1c8c596 2026-09-02 Implement deterministic source-carried proofs


## 92101e2 2026-09-02 Initialize every Windows IOCP entry


## 62068df 2026-09-02 Record source-proof candidate measurements


## bfa36c8 2026-09-02 Match Windows probes to production storage


## f42bcca 2026-09-02 Declare the Windows fail-stop dependency


## fd19068 2026-09-02 Preserve the Windows runtime staging tree


## 92a0683 2026-09-02 Match the Windows compute oracle bytes


## e6ddcb0 2026-09-02 Stabilize the Windows runtime qualification


## 97afc04 2026-09-02 Fix the Windows native-name IO benchmark


## 0d6df7c 2026-09-02 Optimize and harden the Windows IOCP runtime


## a7c49c4 2026-09-02 Fix the Windows bridge fault-probe ABI


## 6bcb68b 2026-09-02 Pin the ordinary-loop backedge guard contract

Investigated the report that an ordinary `loop ( invariant ... ) { }` header
wrongly fails its Backedge obligation when the body increments a cursor under
a source guard. The rejection is not an implementation defect: it is what the
active specification's fixed AUTO family yields.

A source guard such as `ilt(cursor, limit)` is an ordinary L0 relation over
the mutable cursor place. The increment's own SET-1 commit kills every fact
whose support names that place and establishes only ENT-3.S5's applicable
post-write image, which an arithmetic right-hand side does not have. The
L0-to-affine index that ENT-6 gives AUTO's final route is an ephemeral view of
the current difference-bound state, explicitly not a published affine premise,
so nothing relates the pre-write cursor image to the limit on the backedge.
Only an invariant conclusion is a theorem over immutable value images, which
is why INV-1 states that a complex backedge is stated by an `invariant_stmt`
on the reaching body path where its local premises are live.

The counted form behaves identically for the same source shape; a counted
header works only because its compiler-owned binder is never written by the
body, so the S11 true-header bound survives to the backedge while the target
is rendered over the `binder + 1` image. Ordinary and counted headers are
therefore already symmetric, and no compiler behavior changed here.

Added the coverage that was missing for that contract: unit tests for a
published guard discharging a guarded cursor increment, an unguarded increment
failing at Backedge, an unpublished guard failing at Backedge, a body
invariant after the write coexisting with a proved header, and a break-only
body leaving the preservation batch vacuous; plus one runnable conformance
case for the accepted ordinary-loop guarded-cursor form.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 639e7c0 2026-09-03 Render an invariant-free loop header on one line

FORM-2 broke a `for` header apart unconditionally, so a counted loop with
nothing but its binding still cost three lines. The break exists to set
`header_invariant` clauses on their own lines; with no invariant there is
nothing to set apart, and the three-line spelling only made the common
counted walk harder to read.

A `for` whose header carries no invariant now renders `for (i in a..b) {`
and `for @label (i in a..b) {` on one line, and that one-line form is the
only canonical one. A header with one or more invariants keeps the existing
layout: `for (`, one item per line at depth plus one with a comma after all
but the last, then `) {` at the original depth. An ordinary `loop` is
unchanged — no parentheses without invariants, the same multi-line header
with them.

The gap builder now counts `header_invariant` children rather than all
header items, and marks the header breaks only when that count is nonzero;
the space before `(` is marked either way, since it overrides the generic
right attachment in both forms.

Every Whitefoot source in the repository whose `for` header has no
invariant is rewritten to the one-line form: 33 `.wf` files under
`tests/programs/` and `tests/conformance/cases/`, the embedded sources in
20 Rust test and driver files, and the examples in `docs/patterns.md` and
`docs/current-plan.md`. Prose in `docs/` that called the counted header
multiline is corrected to describe the invariant-carrying form. One driver
ledger assertion follows its fixture's shifted line number.

Pre-v0.41 `.wf` records under `research/experiments/` keep the older
paren-free header; they do not parse under the current grammar either way
and are left as the dated evidence they are.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 8ebde13 2026-09-03 Drop local timings and trim the agent charter

Remove the laptop-measured compile-cost and parallel-execution tables, the
host and toolchain description, and the measured source identity list from the
candidate measurement section of the plan. Performance for this candidate is
measured by the gate runner stages, where the per-stage breakdown printed by
canonical `make check` can be compared against the base branch; the section now
says that and keeps the structural erasure and sequential-fallback evidence.

Record two known gaps under the source-proof direction: non-affine addressing
such as `r * cols + c` sits outside the fixed affine fragment, so the writer
uses a literal stride or a flat scan today; and the pre-kill closure path
materializes the complete L0 closure before every kill event, with projecting
only through the dying term left as a measured-performance candidate.

Return the agent charter to charter-level rules by removing the bullets that
restate specification content (AUTO's affine boundary, the canonical loop
header) or narrate current implementation scope (proof-service framing,
`.wfproof` and proof-cache deferral, `par` fact sourcing). AGENTS.md and
CLAUDE.md stay byte-identical.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 842efa4 2026-09-03 Admit the integer tightening of an accumulated affine candidate

A written certificate and the automatic route could form 2*(mid - hi) <= -1
for a binary-search midpoint but never divide that inequality, so the halved
relation mid < hi stayed unproved and the subscript was rejected.

Every affine atom denotes a mathematical integer, so an accumulated
k * v <= u with a positive integer k dividing every coefficient also proves
v <= floor(u / k) with the floor taken toward negative infinity. The checker
now forms exactly two such factors for one candidate and its target -- the
multiple relating the two coefficient vectors, and the greatest common
divisor of the candidate's coefficients -- and checks the ordinary DIRECT
residual against each tightened candidate. Both factors are read from the two
vectors in checked i128 arithmetic; no factor is searched for, no premise is
added, and the fixed candidate families are unchanged.

[PRF-1]'s final residual and every AUTO family route through the one new
helper, so the certificate and the automatic route admit the same integer
consequence. The specification states the step in [ENT-6]'s AUTO description
and in [PRF-1]; no rule is added or removed. New unit tests cover the divided
vector, the floored bound at its exact boundary, a candidate no factor
divides, an unrepresentable divisor, the midpoint certificate, the automatic
midpoint bound, and a signed certificate whose halved bound is negative and
odd. Two conformance cases run the proved midpoint index.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 26467e8 2026-09-03 Merge fix/pr4-loop: ordinary-loop [INV-1] backedge coverage


## 4a1cf91 2026-09-03 Merge fix/pr4-tighten: deterministic integer tightening in the affine residual check


## 5476195 2026-09-03 Merge fix/pr4-format: one-line counted for header without invariants

Resolved compiler/src/spec.rs and compiler/src/spec_identity.rs by taking the
merged spec/kernel-spec.md and re-deriving both digests from it: spec.rs's
recorded constant and measured-digest literal were set to the independently
measured sha256sum of the merged file, and spec_identity.rs was regenerated
with the whitefoot-spec binary.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## eb436ec 2026-09-03 Merge fix/pr4-docs: drop locally measured numbers and agent-authored guidance bullets


## 767f20e 2026-09-03 Renumber the source-proof candidate from v0.41 to v0.40

The v0.40 activation record this branch carried was never merged to `main` and
was never approved, so `main`'s active specification is still v0.39 at
b4d8e01eecd81bdda9c632093873d604ddfbd64d979a4884472907e456d69516. The branch
therefore stops presenting an intermediate activated v0.40 and presents one
candidate, v0.40, superseding active v0.39.

governance/APPROVALS.md is restored to `main`'s exact bytes: the unapproved
"v0.40 source-carried proof" record and its `ACTIVE-SPEC: v0.40 ...` line are
removed, so the chain tail is v0.39 again and this branch adds nothing to the
ledger until activation. `git diff origin/main -- governance/APPROVALS.md` is
empty.

spec/kernel-spec.md is titled v0.40 and declares
`CANDIDATE v0.40 supersedes v0.39 <v0.39 digest>`. Its META-5 line is now one
delta from v0.39, recounted from the [GRAM-*] blocks and from
spec/kernel-spec-v0.39.md rather than composed arithmetically from the two
former deltas:

- numbered rules +2/-9, 131 remain (added INV-1, PRF-1; retired CLM-1, CLM-2,
  CLM-3, DIAG-3, PRV-1, PRV-2, PRV-3, SCOPE-4, TRAP-1);
- grammar productions +8/-1, 82 remain (added affine_add_op, affine_expr,
  affine_factor, affine_term, for_binding, header_invariant, invariant_stmt,
  proof_use; removed claim_stmt). The former records' +7/-1 then +3/-2 do not
  compose to this, because two productions added by the earlier draft were
  removed again by the later one;
- unique fixed lowercase grammar atoms +2/-4, 54 remain (added `invariant`,
  `use`; removed `because`, `claim`, `deny_claims`, `traps`);
- runtime-trap families +0/-1, 0 remain, and entry forms +0/-1, 1 remains,
  both consequences of retiring TRAP-1 and the `deny_claims` prefix.

The retired rule ids are written unbracketed: a bracketed [CLM-1] in the
specification text is a rule reference, and `whitefoot-spec` rejects a
reference with no definition.

Documentation now names v0.39 as the active authority with its digest, and
describes v0.40 only as the pending candidate: README.md, compiler/README.md,
docs/roadmap.md, docs/current-plan.md, docs/patterns.md, and
spec/derivation/derivation-ledger.md. The ledger's amendment section becomes a
candidate amendment, and its `prove`/`use` wording becomes the `invariant`/`use`
form the candidate actually defines. compiler/src/spec.rs's version label and
digests, compiler/src/backend/qualification.rs's review log and REVIEWED_FOR,
and the generated compiler/src/spec_identity.rs all follow.

Verified by running: make repository-invariants approval-history-integrity
spec-append-only spec-digest-sync; make -C compiler spec format lint docs;
whitefoot-spec --check-identity; make conformance (131/131 rules covered); and
cargo test --lib (1464 passed).

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 548ca47 2026-09-03 Skip an unrepresentable integer-tightening factor instead of aborting the list

`integer_tightenings` propagated both factor derivations with `?`, so an
`ArithmeticOverflow` while reading or verifying the target-multiple factor,
or a capacity failure while dividing it out, abandoned the whole list and
suppressed the coefficient-divisor candidate that could still have been
representable. Under [ENT-1] and AUTO's tightening rule an unrepresentable
candidate is skipped and may not suppress another candidate.

Derive and divide out each factor on its own, keeping the fixed order
(target multiple, then coefficient divisor) and the existing deduplication.
The function can no longer fail, so it returns the candidate list directly;
the single caller drops its `unwrap_or_default`, and no `?` can reintroduce
a whole-list abort.

Three unit tests cover the skip: a candidate whose target-multiple
verification overflows i128 while the divisor candidate succeeds, a
candidate whose multiple and divisor are both unrepresentable and yield a
plain empty list, and a tightening that exceeds a checker input capacity.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## d9943da 2026-09-03 Give a set commit the value image its let spelling has

An ordinary loop whose body incremented a cursor under a source guard was
rejected at the [INV-1] backedge when written `set i = i + 1_u64;`, and
accepted when written `let j = i + 1_u64; set i = j;`. One value, two
spellings, two verdicts: the commit's image depended on the statement's shape
rather than on the value's, which the compiler rules forbid.

The cause was in the entailment walk of a `set`. The target kill removed the
guard relation `i < n`, and only [ENT-3.S5]'s three-row copy image was
established afterwards, which an arithmetic right-hand side does not have, so
nothing related the new value of `i` to `n`. The `let` spelling worked because
the binder gave the value its own term before the kill, letting [ENT-5]'s
pre-kill closure carry `j <= n` across the write.

A commit now evaluates its right-hand side to that occurrence's own commit
value: a compiler-owned [ENT-2] term identified by the statement's NodePath
and the value's fragment type, which source cannot name and no event kills.
The `let` sources and the commit share one routine over that destination —
image, closure, kill, then the S5 equality naming the value — so every S5, S6,
S7, and S9 row a `let` initializer receives, a commit receives. Wrapping,
saturating and checked forms therefore receive exactly what they receive at a
`let`, and nothing more: an unproved wrapping range still leaves the value
with only its type range. Only a direct fragment place forms a commit value,
since no other target can receive the image.

The specification states the same in [ENT-2] and [ENT-3.S5], with the
regenerated identity and transcribed digest. Rule, production, and atom counts
are unchanged.

Tests: the unit test added with the previous commit pinned the defective
rejection; it is now an acceptance test with an honest name, beside one that
asserts both spellings of the increment reach one verdict. The S5 test that
pinned an unimaged `+wrap 0` commit is likewise now a parity test, joined by
two tests that hold the line where no image exists: a right-hand side outside
every source, and an unproved wrapping subtraction whose acceptance would read
one past the end. One conformance case runs the plain `set` form of the loop
that the published-guard case already covers. The wfgrep proof-size guard is
raised to the new deterministic counts: one more term per fragment-place
commit, the same cost shape.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## ed99c05 2026-09-03 Merge fix/pr4-tighten2: skip an unrepresentable integer tightening

`integer_tightenings` propagated both factor derivations with `?`, so an
unrepresentable target-multiple factor abandoned the whole list and suppressed
the coefficient-divisor candidate that could still have been representable.
Each tightening is now formed on its own, in the same fixed order, and an
unrepresentable one is skipped rather than removing the other tightening or
the untightened candidate.

The only textual overlap with the branch tip was the doc comment and the call
site of `affine_candidate_residual_proof` in
`compiler/src/semantic/entailment/flow.rs`, which merged without conflict; the
commit-value work on this branch and the tightening fix are independent.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 909ec42 2026-09-03 Say what forms a commit value, and what an unimaged commit leaves

A review of the previous commit found three overreaching claims about the new
[ENT-2] commit value and one stale conformance rationale. Nothing here changes
compiler behaviour.

[SET-2] admits only an affine, region-free target, so a `replace` right-hand
side never has a fragment type and never forms a commit value. [ENT-2] clause
(f), the [ENT-5] (a) kill clause, and the doc comments repeating them
nevertheless named `replace` beside `set` as a source of one. Each now names
only an admitted [SET-1] `set` whose right-hand side has one fragment type.
Clause (a) keeps `replace` where it is true — a replace commit is still a kill
event — and only the commit-value sentence inside it was narrowed.

[ENT-3.S5] said a right-hand side matching no image row leaves v with only its
own type range, which presumes a v that is never formed: no source recognizes
such a right-hand side, so no commit-value term is interned and the commit
publishes no post-write equality at all. The sentence now says that, scoped to
S5, because a call right-hand side may still publish a receiver relation over
the post-write target through its own route. A row whose form matches while its
side condition fails is untouched and already covered by the parity sentence
above: the commit value exists and carries only its type range, exactly as the
same initializer's binder does at a `let`.

One static term per statement is sound because the forward flow visits each
statement once — a loop body once from the head state formed before that walk,
each match arm over its own statements, no statement twice — so a commit value
denotes that statement's value in the one abstract evaluation performed, as a
counted header image denotes the binder's value in an arbitrary iteration.
[ENT-2] and the `CommitValue` doc comment now state that identity discipline.

The conformance case inv1-pos-ordinary-loop-guarded-cursor still explained its
published guard by a rationale the previous commit retired: the plain `set`
spelling is accepted now, which its neighbour
inv1-pos-ordinary-loop-cursor-set-image runs. Its doc says what the case
actually shows — a writer-published body invariant on the reaching path
discharges the backedge on its own, alongside the automatic commit-value route.
The id, rules, expectation, and status are unchanged.

The regenerated identity and both transcribed digests follow the new bytes.

Tests: the specification binary 18 passed, `--lib spec` 13 passed, `--lib
entailment` 192 passed, and the conformance adapter 490 pass with 1 skip;
`make spec-digest-sync` passes.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 471b103 2026-09-02 Drop the premature v0.39 archive until v0.40 activates


## 0b847a3 2026-09-03 Activate kernel specification v0.40

v0.39 is archived byte-exact as spec/kernel-spec-v0.39.md, and the activation
chain, the generated identity, the transcribed literals, the qualification
review note, the derivation ledger, and every live digest anchor name v0.40 at
15ec2f6f475a7b70fb2654026ec3b6ef79afca3bd588fb38f22005d6637c0168.

The appended governance/APPROVALS.md record states the exact specification
bytes and the exact conformance content of this activation. It becomes
effective only when the owner approves this exact revision for merge into
main; no approval is claimed here.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 35b396c 2026-09-02 Merge pull request #4 from mbbill/codex/source-proof

Implement source-carried proof compiler
## 7a3c3bc 2026-09-03 Let the gate name main on a push to main

Every gate job on a push to main failed at "Name main as a local ref":
`git branch -f main origin/main` refuses to move the branch the checkout
has out, so main's gate never ran past checkout (runs 45, 104, 151, 192).
The step now creates the local ref only when the checkout is not already
on main; pull-request runs are unchanged.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 29d523f 2026-09-02 Merge pull request #5 from mbbill/batch/0113-gate-main-ref

Let the gate name main on a push to main
## 631961a 2026-09-02 Merge origin/main into codex/windows-iocp-runtime


## ddcfdc4 2026-09-03 Merge pull request #3 from mbbill/codex/windows-iocp-runtime

Complete the Windows IOCP runtime backend
## 6f667df 2026-09-03 docs: the join-image rule for a conditionally advanced loop binding; conformance for its two verdicts and the &uniq buffer length gap

The 2026-09-03 scenario sweep judged nine mismatches. Four of the seven finder
expectation errors were one shape: a loop header invariant over a binding only
one arm of a body join advances. The compiler and the specification agree there
and the rejections are mandatory, so the gap was in the pattern doctrine.

docs/patterns.md
- P19 states the rule in one line and shows the three accepted body shapes
  (identical update on every arm, arms adding different constants, the choice
  lifted into the addend with a bound on it) and the two repairs (a dominating
  guard after the join, which alone compiles a whole binary search; or removing
  the join). Every fragment is taken verbatim from a program compiled with the
  sweep binary.
- P8 gains the sentence the p12 misdiagnosis needed: a bound stated relative to
  the loop counter carries no overflow conclusion until the counter itself is
  bounded.
- P14 gains the [DIAG-1] warning that one rule and one location are reported
  per rejection, so a probe invariant that draws no message of its own has not
  been shown to hold; and the note that AUTO's pair family includes the
  self-pair, which makes a doubling target automatic and a use block for it a
  redundant-block rejection.

tests/conformance
- ent6-neg-join-one-arm-advances-accumulator (reject INV-1) and
  ent6-pos-join-value-if-lifted-addend (run, exit 0) are the same fold before
  and after the lift, so the doctrine's two verdicts are corpus evidence.
- ent5-neg-callee-uniq-buffer-replace-kills-length records the sweep's unsound
  accept as a tracked gap: the specification requires reject at OP-4, the
  current compiler classifies every callee write through a &uniq buffer<T>
  actual as an element write and keeps the caller's stale length fact, so an
  out-of-bounds heap read is accepted. Status xfail, with the reason stating
  the mechanism; it becomes an XPASS the moment the classification is fixed.

Verified with the sweep binary (whitefootc-909ec42e) and in this worktree:
make conformance green, canonical corpus green, and the native adapter reports
Pass=492 Xfail=1 Skip=1.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 808d1a1 2026-09-03 Re-close a fact state after a kill and re-emit [ENT-2] implicit bounds

`FactState::kill` and `FactState::kill_goals` removed live facts without
clearing `closed_term_count`, the marker that says the state already is its
own [ENT-4] closure. An endpoint projection also deletes the materialized
copy of the [ENT-2] implicit bounds of the killed term, including the
`len(P) = N` equality of an `array<T, N>` place, so the next query took the
closure fast path, returned the post-kill maps verbatim, and never re-seeded
that equality. Acceptance then depended on whether a join had happened to
stamp the marker at the current interned-term count: writing an array's root
binding after a join over an element read left `0_u64 < len(arr)`
undischarged, and inserting an unrelated `let` before or after the write
flipped the verdict.

Both removal endpoints now invalidate the marker, so a state that lost facts
is closed again before the next query answers. The [ENT-2] rule table moves
into one `for_each_implicit_bound`, which all three closure entry points --
the proof closure, the proof-free contradiction closure, and the fast path --
re-emit; implicit facts are a function of the term table and the place's type,
so they are regained at every query rather than having to survive in a map.
This makes the `length_term` promise in flow.rs true as written.

A killed established bound stays killed: closure over survivors plus the
implicit axioms cannot rederive it, which the new negative unit test and the
unchanged `ent5-neg-kill-on-write` conformance verdict both pin.

Evidence: 1484 lib tests pass; clippy --all-targets and cargo fmt --check are
clean; the conformance adapter reaches Pass=491 Skip=1 (490 before this
change, plus the case added here) and the canonical-corpus gate passes. Over a
665-program sweep only two verdicts move: the reduced repro now accepts, and
its full sibling advances past the wrong subscript rejection to a genuine
undischarged exact multiplication it had been shadowing. No accepted program
became rejected, and wfgrep.wf compiles in the same time as before.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## e19da58 2026-09-03 Report the body failure that breaks a loop backedge, not the backedge

[DIAG-1] admits exactly one rule and one location per rejection, and the
checker chose that one rejection by written position: the smallest failing
node path won. A loop-header invariant is written before its body, so its
backedge failure beat every failure inside the body — including the failure
that caused it. When a body operation cannot discharge its domain obligation,
[ENT-6] forms the binding it initializes as a fresh full-range atom, and that
demoted value is exactly what leaves the header relation unproved at the next
header. The writer saw only the [INV-1] effect and never the [OP-2], [OP-4],
or [FN-8] cause. A body-end `invariant` probe was hidden the same way, so a
probe that failed read as a probe that passed, and two separate reviews drew
false conclusions from it.

Select the reported rejection by the order in which the checker decides
obligations rather than by where they are written. Every judgment but one is
decided where it stands, so its node path is already that order. INV-1's
backedge is the exception: it is proved only after the whole loop body has
been walked, so it is positioned immediately after its loop statement's
subtree — after every failure the body holds, and still before the loop's
following siblings. INV-1's base judgment is decided at the header and stays
there, so a failed base still precedes the body failures it causes, and a
nested loop's backedge still precedes the enclosing loop's. The order is a
plain comparison over syntax child ordinals with an explicit end-of-subtree
position, and no step reads a hash.

`an_unguarded_cursor_increment_fails_the_ordinary_loop_backedge` pinned the
masked order: its unbounded `limit` made `cursor + 1_u64` fail OP-2 first, and
the test passed only because the header failure was reported ahead of it.
Bounding `limit` by a written requirement keeps that test's subject the
backedge, and its doc comment records why the bound is load-bearing.

Verified: 1483 lib unit tests pass, clippy --all-targets and fmt --check are
clean, and the conformance adapter is unchanged at Pass=490 Skip=1 — every one
of the 491 conformance cases produces byte-identical output to the previous
build. Across a 665-file experiment corpus nine rejected programs now cite the
body failure (seven OP-2, one OP-4, one FN-8) instead of the INV-1 backedge it
produced, and no program changed between accept and reject.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 488c531 2026-09-03 Merge fix/sweep-d2


## 5b8bcf1 2026-09-03 Merge fix/sweep-d3


## 239879a 2026-09-03 Merge fix/sweep-d4


## a40c7e7 2026-09-03 Merge pull request #6 from mbbill/batch/0114-sweep-fixes

Sweep fixes: re-close after kills, causal diagnostics, join-image pattern
## e336db7 2026-09-03 Land the snapshot corpus as its own gate stage

tests/snapshot/ records what this compiler does with 491 programs from the
665-program sweep. index.tsv is the single source of truth; the gate compares
accept/reject and nothing else, and the cited rule is carried for a reader
only. README.md says in its first paragraph what this is - a snapshot of one
compiler, not specification evidence, outside the conformance boundary - and
states the condition for deleting the directory again.

Every verdict and rule was re-derived with the compiler on this branch rather
than the binary the sweep ran, so the corpus already carries the D2 and D3
fixes: kills__writer-r2__06_chain_middle_replace-min now accepts, and ten
rejections cite a different rule than the sweep recorded.

Selection, from the 621 sweep candidates:

  - the 129 rows whose author stated no expectation are out. That includes
    the sweep's known unsound accept, which belongs to the conformance xfail
    case ent5-neg-callee-uniq-buffer-replace-kills-length rather than to a
    corpus that only remembers verdicts.
  - kills__adversary-r1__p11_whole_buffer_replace_kills_length is out: it
    stops in the parser (its effect clauses are not comma separated), so it
    records nothing about the checker.

That leaves 491 rows: 303 accept, 188 reject, 478 agreeing with their author
and 13 not. Each disagreeing row's doc says what the author expected, what the
compiler does, and the rule that decides it.

Thirteen case files carried a doc line asserting an expectation this compiler
does not meet. Those lines now state the snapshot verdict and what the author
expected; no program was otherwise changed.

Wiring: snapshot-run follows conformance-run in CHECK_STAGES with its own
timing line, joins the CI conformance job, and tests/snapshot.rs joins
compiler/Makefile's INTEGRATION_TARGETS so test-partition still matches the
directory. The driver is #[ignore]d for cost exactly as the conformance
adapter is, and compiles only - no link, no execution. A rejection reached
before resolution is reported rather than counted, because such a case has no
semantic verdict to record.

make check green: snapshot-run 1 s, Pass=491 Flip=0.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## ebee9fb 2026-09-03 Merge pull request #7 from mbbill/batch/0115-snapshot-corpus

Land the snapshot corpus as its own gate stage
## 5d80d31 2026-09-03 Record the evidence behind the container and resource redesign

Two evidence files under research/investigations/containers-and-resources:
the owner's rulings and accepted conclusions from the 2026-08-31 design
discussion, quoted verbatim with line references, and the sweep defect D1
(a callee's write through a uniquely borrowed buffer never kills the
caller's length fact) with its minimal program, mechanism, cited rules,
and the signature-visibility lesson it establishes.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 9c9ad3a 2026-09-03 Draft the container model: owners, affine views, three call rules

CONTAINERS.md states the laws (a view is a value; length is a type or
contract fact; no hidden growth; the initialized prefix is the only
initialization state; capacity is a proof term), the rules for owners,
views, calls, the sequence operation table and the par builder, the
fact discipline that makes D1 inexpressible, a migration of four
existing programs, writer walkthroughs with drafted diagnostics, seven
open questions, and a verified-versus-reasoned register: thirteen probe
programs were run against the gate binary; every introduced syntax is
labelled design text.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 594af3a 2026-09-03 Draft the resource model: providers, the envelope, resource-closed

RESOURCES.md states the laws (the envelope is a promise; no ambient
resource; no silent failure; the runtime lives inside the envelope;
shape not bytes; lowering before judgment; success-assuming demand;
stock not flow), the rules for capability values ([PROV]), the covered
set and envelope judgment ([RES]), the stack ([STK]) and the runtime
closure ([RUN]), how the envelope is computed, the startup protocol,
writer walkthroughs, a kernel scheduler with a page pool and a hosted
program with Heap, ten open questions, and a verified-versus-reasoned
register from probes run against the gate binary.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 922be29 2026-09-03 Integrate the container and resource design into DESIGN.md

DESIGN.md is the single design: the two owner goals and D1 as the
concrete failure, fifteen merged laws, the rule families [PROV], [RES],
[STK], [RUN], [CNT], [VIEW], [CALL], [SEQ], [BLD] with one amendment
register against v0.40, a kernel-flavoured program with the heap absent
and a hosted line collector over Heap, seventeen open questions with a
single recommendation each, a verified-versus-reasoned register from
probes re-run against the gate binary, and an implementation order.
Q6 is recorded as a specification gap: loop-exported invariants and
invariant statements publish affine facts while [FN-9] reads the L0
closure, and the affine-to-L0 bridge is granted per consumer family;
the recommendation is one numeric goal disposition shared by every
consumer. RESOURCES.md and CONTAINERS.md are reduced in place to the
rationale and probe registers DESIGN.md does not carry.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 849603d 2026-09-03 Repair the design after falsifier round 1

Second draft of DESIGN.md. The measures len, cap and room become one
algebra of one-place terms with affine value images, so a push inside
a loop is provable; one numeric goal disposition replaces the
per-family proof route grants; [LIV-1] join-checked affine liveness
replaces [OWN-11]'s move prohibition so value-in value-out loops are
writable; the Builder family is deleted in favour of filled
construction, MutSpan and subscript writes under [PAR-2]; view values
hold their own loan; the resource-closed judgment is split into a
source-stage judgment and a target-stage materialization of E; every
provider-owned release declares writes on its provider; tail position
is defined by caller-frame deadness; checked acquisition results are
typeable; a kernel main loop without break is representable; a
FixedRing owner, seq_take_at and seq_exchange cover what a stack
prefix cannot. Section 6.5 records each round-1 finding and the rule
that now refuses it. CONTAINERS.md and RESOURCES.md are brought into
agreement with the new text.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 05a20a8 2026-09-03 Consolidate the design after falsifier round 2

Third draft of DESIGN.md. Six concepts replace eighteen scattered
clauses: provenance sets generalised from [OWN-5] to every value taken
from a provider and to views; linear release for by-value reclaimed
values, matched by provenance rather than type (law L13 restated);
compiler-owned measure data at function entry and before each transfer,
so a moved value's measures reach the callee's contract and a
reinitialised binding is a fresh term; descriptor-precise measure
support and kills stated by effect rows; region-local reservation with
frame and extent forms; and the `update p by op(args);` statement as
the one receiver-threading spelling. Rule count falls from 65 to 53,
every rule states Judgment, Publishes, Amends and Law, and the
amendment register is assembled from the Amends lines. Execution
contexts and interrupts are declared a follow-on design with the
interface this design imposes on it. Both worked programs are rewritten
and walked against the rules. Section 6.6 records each round-2 finding
and the rule that refuses it now.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 5ecb31a 2026-09-03 Say that the third draft's probe sources are not in the repository

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 4faf6d2 2026-09-03 Judge SET-2 and STOR-5 targets by the region-bearing relation

[SET-2] makes a region-bearing replace target a hard error and [STOR-5]
defines region-bearing as containing slice<'r, T> or arena<'r, T> at
any depth, but the checker tested only the Slice constructor at the
replace target and at box_new's derived referent, so replacing one
arena descriptor with another and boxing an arena descriptor were both
accepted. The relation is now computed once from the checked type's
structure (checked_type_is_region_bearing: slices, arena instances, and
anything reaching one through array or buffer elements, box referents,
struct fields, or enum payloads, with a visited set for nominal cycles)
and used at the replace target, at box_new, and in the generic
substitution arm of the written-type predicate. The SET-2 diagnostic
carries the spec's restructuring text naming the arena case.

Conformance: set2-neg-arena-replace-target (reject SET-2),
set2-pos-box-descriptor-replace (positive control, exit 0),
stor5-neg-box-new-arena-content (reject STOR-5). Snapshot corpus
unchanged: it holds no arena program.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 909626c 2026-09-03 Brand every store in its type; fourth draft after falsifier round 3

A store's identity is the region that names it, and that region is a
type argument of every value taken from the store (HeapBox<'s, T>,
ArenaBox<'s, T>, PoolSlot<'s, T>, PoolVector, ArenaVector, Arena<'s,
BYTES, ALIGN>), so a value's relationship to its backing survives fields,
container elements, enum payloads, multi-return, joins and calls by type
formation alone; provenance sets remain only for the three loan-bearing
views. Reservation is an event of the region block with one reservation
point per region and a reset at exit. Disposal is one `dispose p using
(...)` statement, structural and closed under containment, and propagate
with a live linear binding is an error. Measure data are keyed on program
point, place and measure; every operation that writes a measured place
publishes each of its measures; the arena carries its extent in its
type. [CNT-5] is deleted, box_new and arena_new leave [OP-1], the
contract surface gets its own clause_expr production, seq_vacant and
`update ... into` are added, [RES-7]'s exclusion list becomes a property
test. Section 6.7 records each round-3 finding and the rule that refuses
it now. Companions brought into agreement.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 5fa851f 2026-09-03 Draft the v0.41 candidate: comparison symbols and the call-site `::` delimiter

Carry the owner rulings of 2026-09-03 on the FLOOR-5 comparison row through
the specification, the compiler, and every derived artifact in one batch.

Specification (CANDIDATE v0.41 supersedes v0.40): the six integer
comparisons are respelled `== != < <= > >=` as one `compare_op` class of
`infix_tail`, integer-only rows exactly as the arithmetic symbols are; a
call writes its type and region arguments after the `::` delimiter
(`cvt::<u8, u32>(w)`), so `IDENT "<"` begins only a comparison and every
grammar decision keeps its two-token bound; `!` enters the token alphabet
inside `!=` only; the four ordered symbols replace `ile ilt ige igt` as the
relation of a header invariant, a local invariant, and a relation-form use
step, and a multiplied relation-form use step is parenthesized. Twenty-one
rules move at thirty-eight verbatim-anchored sites; the derivation ledger
and `governance/spec-evolution/comparison-symbols-v041-candidate.md` record
the delta, the rulings, and the rejected alternatives.

Compiler: five compound tokens and the `!` byte in the lexer; five fixed
terminals; grammar tables and spec identity regenerated; DIAG-1's row-1 and
row-2 attributions keyed on `::`; the FORM-2 renderer and audit give `::`
both attachments and a `compare_op` `<`/`>` neither; the resolver drops the
invariant-carrier role; the checker reads infix comparisons, invariant
relations, and use steps from the `compare_op` node; the operation catalog,
goal and invariant renderings follow the new spellings.

Corpus and docs: every test case, the snapshot index, the conformance
manifest, the embedded test programs, and the current documentation examples
respelled mechanically by a one-shot token rewriter (not shipped); verdicts,
rule citations, statuses, and coverage rows are unchanged. The design memory
and the roadmap's FLOOR-5 entry record the rulings; the SWEEP record gains
the re-examination and its corrections.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01W5Gx4bfx6tiozYwhVSTQBC

## b20e44d 2026-09-03 Review the qualification table for v0.41 and pin the regenerated grammar counts

The backend's command-entry row is a deliberate per-activation review
tripwire keyed on the specification version; the v0.41 candidate reports
every command entry unmapped until the table is re-reviewed. The review is
recorded beside the pin: the respelled comparison rows lower to the same
signed or unsigned `icmp` predicates, the `::` delimiter is a front-end
spelling, and no system operation, resource representation, release row,
result shape, entry form, or host ABI mapping changes, so the v0.40 mapping
carries forward complete.

The grammar consistency test pins the regenerated inventory: 83 productions,
109 decisions, 106 terminal predicates.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01W5Gx4bfx6tiozYwhVSTQBC

## 4bf5623 2026-09-03 Space the multiplied use relation, respell the remaining fixtures, and add conformance cases for v0.41

FORM-2 gains the stated space between a use step's multiplier and the `(`
that delimits its relation, `use 3 * (a <= b);`, overriding the generic
right attachment of `(` as the `for` header already does; the renderer and
audit apply it before the block-bearing walk, where a line-bearing
`proof_use` is otherwise skipped. The candidate digest, its ledger binding,
and the spec identity follow the sentence.

The one-shot rewriter now covers a dotted operation callee, so the eight
negative fixtures that still spelled `imul.wrap<i32>(...)` take the `::`
delimiter and reach their intended rule again; the deliberately undelimited
fixture is restored. Test sources that generated `cvt<...>(` through format
strings, the whitespace-mangled round-trip inputs, and the tests that used
a comparison name as their reserved-name sample move to `cvt`, the
retired names are asserted free, the lexer edge cases admit `!=`, and the
expected-spelling prefix, invariant citations, and family counts follow the
new grammar.

Five conformance cases pin the new rules: the six operators with a delimited
call, the undelimited type application (attributed to FORM-3 by DIAG-1 row
3), the bare multiplied use relation, the parenthesized multiplied use
relation, and `==` in invariant position.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01W5Gx4bfx6tiozYwhVSTQBC

## 8ab1fef 2026-09-03 Pin the v0.41 candidate identity and inventory in the compiler's own tests

The embedded active-specification constants name the candidate's version
and its independently measured SHA-256, the grammar tests pin the
regenerated inventory (83 productions, 109 decisions, 4,305 select rows,
106 terminal predicates, `compare_op` at its specification-order index), and
the postcondition tests take `cvt` as their reserved-name sample now that
the retired comparison names are free identifiers.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01W5Gx4bfx6tiozYwhVSTQBC

## a97edf9 2026-09-03 Merge pull request #8 from mbbill/batch/0117-set2-region-bearing-targets

Judge SET-2 and STOR-5 targets by the region-bearing relation
## 7e30954 2026-09-03 Merge remote-tracking branch 'origin/main' into claude/ecstatic-archimedes-t3mzp9


## b046cfe 2026-09-03 Respell the live research programs and the conformance cases merged from main

The io-hosts workflow compiles the io-completion-bench component-open
program on every push, and the io-bench workflow builds the rest of that
directory, so the programs a maintained runner still compiles take the
v0.41 spelling: the ten `research/experiments/io-completion-bench/programs`
benchmarks and `research/experiments/buffer-initialization-cost/drain.wf`,
rewritten by the same one-shot token rewriter and compiled through the
branch compiler afterwards. Programs that already failed to parse on main
for an unrelated reason (the `for @label` loop form in `many_files_loop.wf`
and the three wfgrep-double-walk shapes) and the dated evidence under
`research/investigations` and `research/experiments/blind-writer` keep the
spelling of the specification they were written against.

The conformance cases that reached main after this branch was cut are
brought to the candidate spelling on the merge: two delimited calls in
`set2-neg-arena-replace-target` and two comparisons in
`set2-pos-box-descriptor-replace`; verdicts and rule citations are
unchanged. The candidate record's derived-material section records both.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01W5Gx4bfx6tiozYwhVSTQBC

## bb0cb47 2026-09-03 Partition the design into kernel and library; fifth draft

The kernel specification admits only what wf cannot express. Section 3
is split into 3.K, the kernel rules (measures and the one goal
disposition; brand, reservation, reset and disposal; the branded slot
block with its initialized-prefix typestate and three per-slot
operations; views and loans; liveness, reinitialization and in-place
exchange; the call rules; envelope, resource-closed, stack and runtime
closure; system operations over views: 48 rules in nine families, six
nominal types, fourteen operations) and 3.L, the library written in wf
against the kernel (vacant and filled construction, take_at, clear and
truncate, FixedRing, HeapVector growth, a block pool, collect and append
helpers, keyed families, update as a set admission). Every library item
was written and its proof obligations walked; the seven kernel additions
the partition forced are named with the function that needs each. The
[CNT] family, [SEQ-0], AppendView and absorb, and law L14 are removed;
laws L3, L7, L12, L13, L15 and L17 are closed over activation, release,
linearity, partial moves and byte reclamation; L18 states minimality.
Bound tables and the operation inventory move to a generated-data
appendix. Region elision is assumed as a separate amendment with the one
property this design needs: the spelling is decidable from declaration
text alone. Section 6.8 records each round-4 finding and the rule that
refuses it now. Companions reduced to agree.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## a392e6c 2026-09-03 Respell the merged SET-2 replace test's embedded programs

The region-bearing replace test that reached main after this branch was cut
spells its `array_new` and `arena_new` calls without the `::` delimiter, so
under the v0.41 candidate the fixture no longer parses and the test cannot
reach its SET-2 judgment. The four calls take the delimiter; the expected
rule, issue, and mechanical fix are unchanged.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01W5Gx4bfx6tiozYwhVSTQBC

## 710d8ce 2026-09-03 Activate kernel specification v0.41

The owner approved the v0.41 candidate on 2026-09-03. The outgoing v0.40
bytes are archived byte-for-byte at `spec/kernel-spec-v0.40.md`; the stable
file's status line flips from CANDIDATE to `ACTIVE v0.41` and no other byte
changes, so the active identity is SHA-256
899437ecf48691b9bc436c86a56ccc2a47fc4eb9290d546010296db7808c5761 superseding
15ec2f6f475a7b70fb2654026ec3b6ef79afca3bd588fb38f22005d6637c0168.

`governance/APPROVALS.md` gains the merge-time record for the merge that
carries this revision: the specification record names the exact bytes, the
conformance record names the five added and 275 respelled cases, the manifest
before/after digests, and the unchanged runner and adapter files, and the
`ACTIVE-SPEC: v0.41` line extends the chain. The generated identity, the
transcribed digest in `spec.rs`, the derivation ledger's binding and
statistics, the candidate record's status, and the live documents that quote
the active digest follow the activation.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01W5Gx4bfx6tiozYwhVSTQBC

## 3060291 2026-09-03 Merge pull request #9 from mbbill/claude/ecstatic-archimedes-t3mzp9

Activate kernel specification v0.41: integer comparison symbols and the call-site `::` delimiter
## 79ad38f 2026-09-03 Merge remote-tracking branch 'origin/main' into batch/0116-containers-and-resources


## b2bbd4d 2026-09-03 Sixth draft: inputs and outputs, derived linearity, surface proposals

Parameters are inputs and results are outputs: the &uniq container
parameters and the exit datum are withdrawn, helpers are value-in and
value-out with an ordered result list and the exchange admission of
set, and a contract relates results to inputs with no entry or exit
convention. Linearity is derived: a value whose release requires a
capability is linear, closed under containment, and no writer marks a
store-derived type; the linear modifier is for logical obligations
only, with a writer guideline. dispose is a consume and a write. The
block typestate becomes an initialized window with the prefix as its
head-zero case. Accounting is closed: a domain is a store, the map
carries a retained label and a reset transfer, and derived columns
replace hand-filled ones. A notion table states the closure property
of each notion. Every language-surface addition the design relies on
is listed as PROPOSED in a dedicated section with alternatives; nothing
is written as decided. Respelled to the v0.41 surface with citations
re-pointed. Section 6.9 records each round-5 finding.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## b9dd2c7 2026-09-03 Bring the research programs and wfgrep bundles to spec v0.41

Four research programs still carried the v0.25 counted-loop header and the
pre-v0.41 comparison and call-site spellings, so the current compiler
rejected them although their Makefiles build them:
io-completion-bench/programs/many_files_loop.wf and the three
wfgrep-double-walk shapes. The labelled `for` was never retired: v0.40 moved
the binding into parentheses and made the label optional, and v0.41
respelled the six integer comparisons as symbols and delimited call-site
type arguments with `::`. The four files are respelled by exactly those
three rules (4 headers, 184 comparisons, 108 call sites) and compile; the
io-completion-bench verify line publishes the expected checksum.

The wfgrep bundle family (baseline, double-walk, wide-scan-lowering) is
made honest about HEAD. Their subject was the tests/programs/wfgrep.wf of
2026-08-06, which became a recursive search with PATH:LINE:TEXT output on
2026-08-18, so their verify phases cannot match the pins from HEAD, and
commit c4e82fba rewrote path strings inside their committed raw evidence
after RESULTS.md pinned its digest. Each RESULTS.md gains a dated replay
status naming both facts. The double-walk driver and runner are reduced to
what HEAD can still assert: the three shape sources, kept on the active
specification, are compiled with the gate-profile compiler and verified
byte for byte against the inherited manifest (all fifteen shape-case pins
match). The baseline and wide-scan drivers stay as the frozen runs' record
and write replay logs to the scratch root instead of appending to the
committed raw files. The experiments index gains the two missing bundles.

Two v0.17-era floor studies, literal-line-floor and wfgrep-scan-floor, had
Makefiles that fed kernels the current compiler rejects (unnamed result
bindings; the `traps` effect retired in v0.40). Those Makefiles are removed,
each RESULTS.md records why and where the freeze commit is, and the index
moves both bundles to a frozen v0.17 section.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>

## 5333a0c 2026-09-03 Force region elision: kernel specification v0.42 candidate

[FORM-8] Canonical region spelling. A region name is written exactly
where the document does not otherwise fix the region denoted and is
absent everywhere else, so each region position has one legal spelling.
A declaration writes a region name only to relate two of its own
positions or to name a result region no input determines; every other
position is unnamed and distinct. A borrow writes its region only when
it is not the innermost region block's; a region block is named only
when the name occurs inside it; a call writes only the region arguments
no actual determines, as the leading members of its `::` application,
and no application at all when none remains. Every clause is decided
from the owning declaration's own text.

The grammar gains five optional REGIONID decisions and no production;
resolution mints unspellable synthetic regions for unnamed positions;
the checker infers determined call regions from the actuals and rejects
the other spelling under FORM-8 with the spec's restructuring text.
Every program in tests, docs and live research is respelled
mechanically: 2623 region tokens across the four test corpora become
260, all of them relating two positions or naming a result region.
Eight conformance cases pin the rule; three "wrong region-argument
count" cases retire because a rule with one legal list cannot express
that fault. The specification is a v0.42 CANDIDATE over v0.41; no
archive is written and no ACTIVE-SPEC line is added; every check stage
except the candidate's expected spec-archive-integrity stop is green.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## ac05408 2026-09-03 Merge pull request #10 from mbbill/claude/kind-hamilton-d81821

Bring the research programs and wfgrep bundles to spec v0.41
## 2e8a96f 2026-09-03 Merge origin/main into batch/0118-region-elision

Main respelled the research programs to v0.41 (PR #10). The four
programs both sides touched take main's v0.41 text and are re-elided
under [FORM-8]; the v0.42 gate compiler accepts each of them.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## efab880 2026-09-03 Seventh draft: one set commit rule, R1 as a rule, publication

Owner decisions recorded as decided: the `linear` modifier beside
derived linearity; one commit rule for `set` (right-hand side fully
evaluated, targets dead during it, all targets reinitialised at the
commit, n-ary, value lists admitted, no exchange operation anywhere);
`dispose p;` resolving the capability from the brand; the shared slice
is a copy value and mut_slice is affine; the remaining surface entries
adopted, so §3.S is a decision record. R1 becomes [BLK-4]'s fourth
clause: no &uniq parameter may be or reach a container or loan-bearing
type. [CALL-6] states publication, how a declared relation becomes a
fact; [CALL-7] states completeness for functions returning measured
values. Measure data gain placements at rebinding, enum payload
binding and destructuring binders; linearity closes under ownership;
dispose refuses declaration-linear leaves; ensures may not name a
&uniq parameter's measure; [RES-10] charges consumable domains by
trips and refuses unbounded loops. Three new surface proposals stay
PROPOSED: on_propagate blocks, seq_rebase, and the [SYS-8] range
operations over views. Section 6.10 records each round-6 finding.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## c168ab2 2026-09-03 Activate kernel specification v0.42

The status line flips from CANDIDATE to ACTIVE v0.42 and no other
specification byte changes; the resulting identity is
6b935d2ea7729876fc96533b5559f6f58598e335b4b5cffad86cc4782c0eed26. The
outgoing v0.41 bytes (899437ec...) are preserved byte-for-byte as
spec/kernel-spec-v0.41.md. governance/APPROVALS.md carries the
activation record and the ACTIVE-SPEC chain line, spec_identity.rs is
regenerated by whitefoot-spec, spec.rs carries the digest, and the
README, plan, roadmap, patterns and derivation ledger name v0.42 as
active. Canonical make check is green on this revision, including
spec-archive-integrity.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 6c4e30e 2026-09-04 Loop bodies are region blocks; associative image joins: v0.43 candidate

Two amendments over v0.42. First, every loop body is itself a region
block [OWN-11]: a borrow of an outer binding inside a loop body is
written bare and lives in the body's own unnamed per-iteration region,
so the guarantee that outer bindings are written again between
iterations is unchanged; a region block that is a loop body's only
statement restates what the loop already says and is a [FORM-8]
rejection, while a block beside other statements keeps its distinct
[OWN-6] statement scope and stays legal. Second, [ENT-6]'s control-flow
join folds every delta atom an earlier join minted back into its
interval before comparing images and takes the hull, so nested
conditionals reach exactly the image one flat match over the same
branches reaches; acceptance no longer depends on the shape of the
control join, and delta atoms stay ordinary shared atoms between joins.
The corpus loses its four sole-statement loop region blocks; six
conformance cases pin the two rules; conformance Pass=512, snapshot
Pass=491 Flip=0, every check stage green except the candidate's
expected spec-archive-integrity stop.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## b9a00c4 2026-09-04 Eighth draft: linearity by scope, one denotation rule, consolidation

Owner decisions recorded: a store-backed value is linear only in a
scope that does not hold the capability its release needs, and a
scope that holds it derives the release on every leaving edge charged
to writes(heap) (108 dispose statements leave the worked programs);
on_propagate is rejected and removed; seq_rebase withdraws to the
library with its memory cost stated; the seven system range operations
take views; type and const arguments stay written; loop bodies are
their own region blocks. Every denotation is keyed on parameter mode,
a published relation is established at the routed arm and only there,
[RES-10] tests only terms and compile-time integers, brand resolution
is one rule, and the release graph is bounded. Section 6's five
per-round tables collapse into one; the file falls from 7821 to 6161
lines at 51 kernel rules. Three surface items remain proposed:
seq_reslice, a linearity bound on generic parameters, and a
ReserveOutcome result for reserve_file.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 874aff6 2026-09-04 Record the join-shape root cause and the landed loop-body criterion

The eighth draft's own probe pair did not reproduce the join-shape
asymmetry; the orchestrator's re-run of the round-7 programs did, and
the investigation on main found the cause in [ENT-6]'s join and the
repair in the v0.43 candidate. The loop-body region rejection is stated
with the criterion that candidate implements: a region statement that
is the body's only statement, unless a type argument inside it must
write its name.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## e8e3426 2026-09-04 Merge remote-tracking branch 'origin/main' into batch/0116-containers-and-resources


## 285594c 2026-09-03 Merge pull request #11 from mbbill/batch/0118-region-elision

Force region elision: kernel specification v0.42 (activated)
## 38bc692 2026-09-03 Merge pull request #12 from mbbill/batch/0116-containers-and-resources

Design: containers and resources (eighth draft, for owner review)
## 23a2de3 2026-09-04 Merge remote-tracking branch 'origin/main' into batch/0120-loop-body-region


## 63c6c97 2026-09-03 Merge pull request #14 from mbbill/batch/0120-loop-body-region

Loop bodies are region blocks; associative image joins: v0.43 candidate
## 3703c81 2026-09-04 Design: record the owner's rulings on the last three proposals

S31 is not adopted as an operation: a shared slice formed from a
mut_slice is the ordinary shared child reborrow of a unique loan that
[OWN-6] already admits for places (a v0.42 probe accepts
peek(x: &deref(x)) where x is &uniq), so [VIEW-6] states that rule
applied to views and the seq_reslice row disappears from the kernel
vocabulary. S32, a linearity bound on a generic parameter, is adopted
and read by [PROV-6] and [BLK-4]. S33, reserve_file returning a
three-variant ReserveOutcome, is adopted in place of the Result form,
and [RES-6] publishes room(factory) = 0 on the Exhausted arm. The
proposed list in §3.S is now empty; §7's tests are updated for B5, B7,
B8 and B10.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 55af97d 2026-09-04 Activate kernel specification v0.43

The status line flips from CANDIDATE to ACTIVE v0.43 and no other
specification byte changes; the resulting identity is
037c9e69b271a7ae212bd71fa2e79c74a3bf4b2115c0418f4908a24b0a9f6951. The
outgoing v0.42 bytes (6b935d2e...) are preserved byte-for-byte as
spec/kernel-spec-v0.42.md. governance/APPROVALS.md carries the
activation record and the ACTIVE-SPEC chain line, spec_identity.rs is
regenerated by whitefoot-spec, spec.rs carries the digest, and the
README, plan, roadmap, patterns and derivation ledger name v0.43 as
active. Canonical make check is green on this revision.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## ac0ec12 2026-09-04 Append the v0.43 activation record instead of rewriting the candidate's

The candidate record of 2026-09-03 already exists on main, and
governance/APPROVALS.md is append-only over main's records, so the
activation is recorded as a new record after it, with the ACTIVE-SPEC
chain line; the candidate record is restored byte-for-byte.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 966bf4c 2026-09-03 Merge pull request #15 from mbbill/batch/0123-design-s31-s33

Design: record the rulings on S31 to S33 (docs only)
## 6977159 2026-09-03 Merge pull request #16 from mbbill/batch/0122-activate-v043

Activate kernel specification v0.43 (restores main's static gate)
## 1a91c0d 2026-09-04 The fact machinery: measure operands, call data, publication (v0.44 candidate)

Batch B1 of the container-and-resource design, as a v0.44 CANDIDATE
over v0.43. [MSR-5]: a requires or ensures clause takes a clause_expr
whose operands may be calls, so len(P) of any place [ENT-2] admits a
length term for derives directly on either side of a comparison, with
no define hoisting. [MSR-3]: the mode-keyed denotation table; an own
operand of an established relation denotes the call datum, a
compiler-owned immutable term minted at the transfer point, and a
&uniq parameter's measure is inadmissible in a source-declared ensures.
[CALL-4]: the single-result route, with the measured-result, result-
measure and any-variant routes deferred with their deltas. [CALL-6]
with [ENT-3.S13] (call datums): how a declared relation is
instantiated at the call and established on the normal continuation,
restricted to its routed arm, and the refusal of a contract block whose
published relations are contradictory. Six conformance cases, unit
tests, the grammar tables regenerated, docs/patterns.md P16 and P21.
wfgrep.wf and raw_deflate_boundary.wf pass the capacity they used to
publish through a &uniq measure as an own operand with a requires
clause, the restructuring the new diagnostic names; no behaviour
changes. Conformance Pass=518, snapshot Pass=491 Flip=0, every check
stage green except the candidate's expected spec-archive-integrity
stop.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 31aaa1c 2026-09-04 Record B1's landed form in the containers design

B1 (the v0.44 candidate, PR #17) implemented [MSR-3], [MSR-5], [CALL-4]
and [CALL-6] in a narrower form than the eighth draft wrote, and exposed
four defects in the design text. This brings DESIGN.md to what landed:

- [MSR-5]: the production is now
  `clause_expr := (atom | call | construct) ((infix_op | compare_op)
  (atom | call | construct))?`, with the Bool root carried by the
  [OP-5] judgment. The draft's `affine_expr compare_op affine_expr`
  dropped every Bool-rooted clause the corpus writes and contradicted
  [FN-8]'s retained `.defined` admission; a dated correction records
  this. The affine widening stays [MSR-4]'s in B2.
- [MSR-3]: the table row now says a `&uniq` parameter's measure, as
  its Judgment line always did; a non-measure `&uniq` operand stays
  admissible in an ensures, with the L11 reason.
- [CALL-4]: the measured result, a measure over a result place and a
  route over any variant are recorded as deferred to B7; [S16]'s result
  list and its destinations go to a new B1b entry in section 7.
- [ENT-3.S13]: the population is stated honestly. No v0.44
  compiler-owned row carries a declared relation set, so what landed is
  the call-datum substitution half at ordinary source calls; B7 extends
  the population to [BLK-0] rows.

Section 6 gains 6.0 listing the six conformance cases with their
verdicts and the corpus consequence for `append_slice`. CONTAINERS.md's
S13 bullet notes the source-call-only population.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 7e3866d 2026-09-04 Merge pull request #18 from mbbill/batch/0124-design-b1-defects

Record B1's landed form in the containers design (docs only)
## 9c73825 2026-09-04 Merge pull request #17 from mbbill/batch/0121-fact-machinery

B1 fact machinery: v0.44 CANDIDATE (needs activation before merge)
## 51ae8c7 2026-09-04 Activate kernel specification v0.44

The v0.44 candidate merged to main in PR #17 without activation, so the
canonical gate stops main at spec-archive-integrity. This commit activates
it and changes nothing else in the language:

- spec/kernel-spec.md: the status line flips from
  `CANDIDATE v0.44 supersedes v0.43 037c9e69...` to `ACTIVE v0.44`; no
  other byte changes. The active bytes hash to
  5ef144bfa9f85e9d2a412e053e43b83d250b804acb2f3d409f4d4367301fa049.
- spec/kernel-spec-v0.43.md: the outgoing v0.43 bytes archived
  byte-for-byte (037c9e69b271a7ae212bd71fa2e79c74a3bf4b2115c0418f4908a24b0a9f6951).
- governance/APPROVALS.md: one appended activation record and the
  `ACTIVE-SPEC: v0.44` chain line; the candidate record and every earlier
  record are untouched.
- compiler/src/spec.rs and spec_identity.rs: the embedded identity and
  the chain length (36), regenerated with `whitefoot-spec --emit-identity`.
- README.md, compiler/README.md, docs/current-plan.md, docs/roadmap.md
  (revision 68), docs/patterns.md, spec/derivation/derivation-ledger.md:
  v0.44 named as the active authority. The ledger's v0.44 section is
  bound to the activated bytes and its preamble corrected to what its own
  table says: two rules derived (MSR-3, CALL-6), two existence-only
  (MSR-5, CALL-4); 81 derived, 55 existence-only, 0 underived over 136.

Canonical `make check` passes on this revision: conformance Pass=518
Xfail=1 Skip=1, snapshot Pass=491 Flip=0.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 9d2f824 2026-09-04 Merge pull request #19 from mbbill/batch/0125-activate-v0.44

Activate kernel specification v0.44 (turns main's static gate green)
## e60ee2a 2026-09-04 Retire the candidate spec state and stop quoting the identity in prose

Two simplifications of the specification gate, decided by the owner on
2026-09-04 after the v0.43 and v0.44 candidates each merged to main and
each needed a zero-content activation PR (#16, #19) to turn main green.

1. No candidate state. `spec/kernel-spec.md` always declares
   `Status: ACTIVE vN` and hashes to the chain tail. An amendment lands
   in one change: the amended file, the archive of the outgoing bytes,
   the appended approval record with its `ACTIVE-SPEC:` line, and the
   regenerated `compiler/src/spec_identity.rs`. `spec-archive-integrity`
   now requires the ACTIVE status line and drops its candidate branch
   and the `spec-candidate-integrity` alias; `whitefoot-spec` parses
   only the ACTIVE form, and its candidate lineage tests go with the
   state they tested.

2. Prose stops quoting the identity. The chain tail and the generated
   identity module are the two machine-checked homes of the version and
   digest. `spec-digest-sync`, which forced six documents to quote the
   digest and an "active vN" sentence at every activation, is replaced
   by `spec-prose-integrity`, which fails when live prose quotes a
   64-hex digest or names a version as the active authority. README,
   compiler/README, WORKFLOW, current-plan, roadmap (revision 69),
   patterns, and the derivation ledger's header now point at the chain
   instead; the ledger's per-version amendment bindings are frozen
   history and stay, so the ledger is held to the phrase check only.

Also in the same spirit: `compiler/src/spec.rs` derives the version and
hash constants from `spec_identity.rs` (a compile-time hex decode)
instead of a hand-transcribed literal. The literal's test, which said
the same thing the chain check already says, is retired with that
explanation; the independent `shasum` measurement still enters through
the approval record.

Canonical `make check` passes on this revision: conformance Pass=518
Xfail=1 Skip=1, snapshot Pass=491 Flip=0.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 68b7193 2026-09-04 Merge pull request #20 from mbbill/batch/0126-gate-simplify

Retire the candidate spec state and stop quoting the identity in prose
## 2748fcc 2026-09-04 Design: retire array (S34) and capitalize the view names (S35)

Two surface decisions the owner took on 2026-09-04 after asking why
`array<T, n>` had survived the container redesign.

S34: `array<T, n>` retires with its `array_new` row. It was exactly the
`len = cap = n`, `head = Z` case of a run, which A.1 already tabulated
as four exact constants; a `FixedVector<T, n>` whose four measures are
standing facts is that case with no runtime descriptor word, so a const
of `FixedVector<T, n>` type with exactly n literal entries is the
const-eligible form, lowers to element storage only, and materializes
its descriptor at each use. One fixed run, one spelling.

S35: every compiler-owned container, store and view nominal is
capitalized: Vector, FixedVector, Heap, Arena, Slice, MutSlice. Only the
primitive types stay lowercase. This supersedes S5's kept name and S6's
spelling; the operation names seq_slice and seq_mut_slice are unchanged.

The 3.S table and grounds, 3.K.10's naming table, [BLK-1], the A.1
measure and layout tables, and the amendment register ([TYPE-2],
[OP-7], [CONST-1]) are updated; every normative section now writes
Slice and MutSlice, while section 6 keeps the old spelling in its
probe quotes. The 3.S note records why the omission survived seven
falsifier rounds: none asked whether the old surface was fully replaced.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 82f6d6a 2026-09-04 Merge pull request #21 from mbbill/batch/0128-design-array-views

Design: retire array (S34) and capitalize the view names (S35)
## 60e6a2a 2026-09-05 The completion path has one arm: submit, then join

Design §8. The seven submits return nothing; an emitted module that
submits requires the runtime, so the weak fallbacks that answered "run
it directly" go, with the inline arm, the offered phi, the eligibility
branches and the Windows verdict fork (the capacity wait the emitted
fork covered is now the Windows bridge's own until its record port).
The one arm that produces an outcome without a submission, an invalid
component of an open by name, builds the typed outcome in place with
no call. The bridge answers an offset beyond INT64_MAX with EINVAL, as
the direct path did, and completes an empty enumeration range inline;
the directory mapper gained the empty-range arm the transfers had.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## cc1fc49 2026-09-05 evidence: what binary arithmetic in the proof surface actually costs

Four independent sweeps against the v0.44 compiler: three constructed
programs in their own domain (loop-carried bounds, multi-dimensional and
strided access, codecs and capacity arithmetic), one counted the 1229 .wf
files already in the tree. Every verdict recorded here was compiled.

All four converged on one sentence rather than on prover strength. ENT-6
computes the four endpoint products of a non-constant multiplication,
proves all four are in range, and discards them, so the multiply is
admitted and its result carries no bound. E1 accepts the product, E5
rejects the add that follows it, and supplying exactly the discarded
bound as a written guard turns E5 into an accept.

The same measurement found a second gap that is not about nonlinearity:
a contract clause admits no binary arithmetic at all. `count * 2_u64` and
`count + 2_u64` are the same FN-8 rejection, so a size precondition is
unwritable and a widened multiplication would not fix it.

The corpus count (1.7% of files) and the constructed count (9 of 11
natural forms rejected) disagree because the corpus cannot measure this:
it is written in the language that has the gap, and one of its own files
records picking a power-of-two grid width to dodge the family.

This is selection ground only. No spec byte, compiler code, or
conformance content changes here.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## af7164f 2026-09-05 evidence: correct the contract-clause reading, and add its probes

The first reading of the contract surface was wrong in a way that changes
what the fix is. A clause does admit binary arithmetic, nonlinear included:
`define needed = count *sat count;` forms, and so does `count +wrap 2_u64`.
Both fail later, at the caller, Unproved. What a clause refuses is the
exact rows — requires.rs:813-816 rejects any operation whose is_exact()
holds — so the divide is exact against total, not addition against
multiplication. A second and independent wall is arity: clause_expr admits
one operator, so `requires a >= b + 1_u64;` is a GRAM-5 parse rejection
whatever the row.

The consequence is unchanged: `requires len(out) >= 2 * len(src)` is
unwritable. The cause is a formation-time policy over one operator set,
grounded in a clause being runtime-typed where an affine expression is
mathematical, so the repair is an allow-list plus a spec amendment rather
than a second relation model.

That also reorders the work. Widening a written surface before the fact
base can hold what it states turns a clean rejection into a silent
non-fact: checked_affine_relation_l0 publishes only unit-coefficient
relations, and a measure has no affine image at all. The fact base is fed
first.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## effe2e2 2026-09-05 One lowering for every I/O operation; the direct family leaves

Design §8. Every ordinary I/O wrapper in the system emitter reserves
the record in its own alwaysinline frame, submits, joins and maps the
completion, and derived release closes through one shared helper that
does the same. The four text builders the handed-out path and the
wrappers share are the one spelling of submit, transfer target and
retirement. The direct family (pread, write, open_at, status, close,
directory_next), the qualification rows that named it and every
declaration of a _direct symbol are gone from the compiler, the bridge,
the Windows runtime and the harness. The open_file and open_directory
kind check and its close on mismatch are the runtime's, decided from
expected_kind, so no emitted code holds a struct stat. The
deterministic test target takes the same shape with submit and join
stubs. A thread joining its own queued submission runs that record
itself, so an ordinary write no longer waits behind an unrelated
blocked one when helpers are pinned.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## 7b9d41f 2026-09-05 Bring the io-hosts workflow to the runtime it tests

The Linux native-adapter probe step lists its sources by hand and had
no scheduler core, which the record-by-address runtime links since
3acc3e9; the Makefile's own probe build had it. The Windows blocking
probe's expected line still named the retries and attempts the
descriptor retirement ledger counted, deleted on this branch at
554f1d9, and the resource-attempt interleave probe it scripted was
deleted with the ledger, so its two steps go. Both jobs had been red
on this branch for those reasons and no other that this change knows.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## 566e517 2026-09-05 Record slice 3's order and the shared-code decision for Windows

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## f14a70f 2026-09-05 Order the ring's reaper after the submitter with one release-acquire pair

The record carries an `issued` word: the ring's submit stores it with
release after every other word of the record is written, and the CQE
reaper loads it with acquire before reading any of them. The kernel
carries the record's address from the SQE to the CQE, but a mapped
ring is not a C11 synchronization, and the entry pool's state word that
used to give this ordering went with the pool. The thread sanitizer on
the real Linux host reported exactly that gap in the default-route
probe; it is quiet on the core/read, default-route and bridge-and-ring
stages now. A CQE naming a record that was never issued is a protocol
failure.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## 69cec6c 2026-09-05 Publish the interval an admitted product already proved: v0.45

[ENT-6]'s fixed interval-product rule proves an inclusive interval for
each operand of a non-constant multiplication, forms the four products of
their endpoint pairs, and admits the multiplication exactly when all four
lie in the result type. It then discarded them. So the multiply was
admitted and the value it bound carried no bound at all, and the `+` that
followed was refused [OP-2] for want of a premise the checker had just
computed.

[ENT-3] gains source S14, which establishes the least and greatest of
those four products on the value the multiplication binds. Nothing about
the derivation widens: both published relations are constant bounds
against the distinguished zero term, so they are ordinary L0 facts whose
[ENT-5] support is the bound value alone. That is what they mean — they
describe the value already produced, so a later write to an operand
leaves them true while a write to the bound place kills them under the
ordinary rule. No relation over the operands, no new term, and no
automatic route by which a product enters a premise family is added; a
written `use` remains the only way a product participates in a
certificate.

The measurement moves into affine_integer_product_interval, which the
domain decision and S14 both read, so the admitted range and the
published bound cannot disagree. The interval travels out of the
judgment on ProofResult and waits keyed by the operation's own carrier
node, because the domain is judged while the initializer is walked and
S14 establishes at the binding that walk then reaches; a domain
discharged by the finite L0 or affine-clause route carries nothing and
publishes nothing.

Evidence: E1_bounded_product.wf is accepted because the interval rule
discharges `r * w`; E5_product_then_add.wf is the same program plus
`base + c` and was refused. Both now accept, as does an invariant stated
over the product's result. Over the 55 constructed multi-dimensional and
strided programs exactly two verdicts move, both from refusal to
acceptance, and the guarded flattened-index programs relocate from [OP-2]
to the [OP-4] subscript bound they were always about. The snapshot corpus
holds at 491 pass, 0 flips, and conformance covers 136/136 rules.

The backend qualification tripwire is reviewed and carried forward: the
amendment is front-end fact publication over erased proof state, admits
only programs previously refused, and introduces no operation kind, entry
form, result shape, or ABI mapping.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## 58a9e27 2026-09-05 Bring the derived documents to v0.45

The derivation ledger binds the amendment: [ENT-6] and [ENT-3] change in
place, no rule is added or retired, so the statistics line carries over
from v0.44 unchanged and the new section records why withholding the
interval was a publication choice rather than a soundness boundary. R4
ranks a static rejection above a runtime check, and a writer whose
admitted product carried no bound had to hand the checker back the
identical interval as an executed guard; P0 reads the same guard as an
emitted comparison and an unreachable arm on the hottest index path.

The plan and the outline carry the version narrative, including what S14
does not reach: it states a constant interval, so a bound relative to
another runtime value stays outside it.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## 1895516 2026-09-05 Record what L1 costs: the clause surface needs a measure atom first

Built the smaller half of L1 and ran it rather than reasoning about it.
Admitting the exact affine rows in clause position is 22 lines, and with
them an expansion codec's precondition forms where v0.45 refuses it at
formation. It then fails at every caller:

  [FN-8] UndischargedCallRequirement
    instantiated_goal: "len(out) >= len(src) * 2_u64"
    disposition: Unproved

The call site is len(out)=8 and len(src)=4, so the goal is true. Neither
fact domain can hold it: L0 is a two-term difference bound and carries no
coefficient, and the affine layer has no measure atom — affine_term_value
images Zero, Constant, and unprojected binding places, and every length
term falls to the None arm, which is what spec:1429 means by "not yet an
affine atom".

So the smaller half alone would ship a precondition a writer can state
and no caller can meet. That is the failure the contract scoping
predicted for widening a surface ahead of the fact base, so the patch is
recorded here and not merged. L1 is both halves in one amendment: the
exact rows read mathematically, and len(P) admitted as an affine atom.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## 9051576 2026-09-05 A mixed overlap group hands out both its kinds

The emitter's constructor dropped every group holding a completion
member, so a run that mixed compute and I/O lost all of its compute
hand-outs and ran them as plain calls. Owner ruling 2026-09-05: I/O and
compute members overlap in one group. A submitting completion member
needs no lane frame and is the one member of a group allowed to
suspend; a group whose join site itself submits keeps today's lowering.
The join site's joins run through compute_join_order (compute members
newest first, completion members where they were published, design
§4), and block_exit_label settles the label on the last member that
opens a block. Two fixtures pin the two orders, joined C2, IO, C1 and
IO, C2, C1, at WF_WORKERS 0, 1 and 4.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## cb0e599 2026-09-05 Record the second L1 attempt: eight pieces, still refused

Built the measure atom rather than reasoning further about it. The state
field, the kill and join and scope-exit rules, the mint in the one
mutable window before the proof, the read over both the bare and the
deref spelling, the L0 bridge that tightens an atom minted as an unknown,
and the clause allow-list all compile. The expansion codec's precondition
is still Unproved with the same instantiated goal.

The affine route is wired for call goals and the literal-sided multiply
arm exists, so both ends of the shape are present. What is not
established is whether the measure atom is found after
admitted_call_goal_expression substitutes the actual, which here is a
borrow expression: if substitution leaves the measured operand as
anything other than a place datum, every piece is correct and
unreachable. That is one instrumented run, and it belongs before more of
this is written rather than after.

The patch is recorded beside the finding and not merged. L1 is not a
follow-on to L0: L0 published a fact the checker had already proved into
a domain that already held that shape, while this introduces a new kind
of affine atom whose identity has to mean the same thing at every layer
it crosses.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## 9613e6d 2026-09-05 A clause states an affine relation, and the atom that discharges it: v0.46

Three sentences move together because each alone is inert, which the
build proved twice before the third was found.

FN-8 admits exact addition, subtraction and multiplication in a clause
and reads them over the mathematical integers — the carve-out INV-1
already gives an affine expression. A clause is erased before lowering
and evaluates nothing, so a row total over the mathematical integers
states a relation where it would otherwise request an operation, and no
domain obligation arises to discharge. Division, remainder, negation,
absolute value and the shifts stay inadmissible, each having an input its
own relation cannot state its way out of.

ENT-3 makes a measure term an affine atom: one per measured place,
identified by that place's root binding, minted at its full u64 range,
with the L0-to-affine index ranging over measure terms so what is known
about an object tightens it. This is the admission v0.44 recorded as
DEFERRED and did not take, and it is sound by ENT-5's own support rule
rather than a new one — a measure is fixed at creation and an element
write never moves it, so only a write to the root removes the atom, and a
join keeps it where every input agrees.

ENT-6's affine route discharges a comparison goal whose normalization is
affine whether or not it also projects to L0. The target is the goal's
own comparison normalized, so proving it proves the goal; the projection
was what the retained evidence named, not a premise. Without one the
derivation is the affine consequence alone, the shape the interval-product
rule already uses for its endpoint proofs.

Evidence: `requires len(out) >= 2 * len(src)` is the precondition of
every expansion codec and v0.45 refuses it at formation. The exact rows
alone make it form and leave it unprovable at every caller, at a call
site where the goal reads 8 >= 4 * 2. The atom alone leaves the route
unreachable: the affine target is built and never consulted. With all
three it compiles and discharges. Across 186 constructed probes the
accepted set moves by four programs, each a contract the previous version
could not state; the snapshot corpus holds at 491 pass and zero flips.

Conformance: fn8-neg-requires-partial-op keeps its id, rules and reject
verdict and moves from an exact addition to an exact division, because
the amendment deliberately admits the row it used; the boundary is
recorded in APPROVALS. fn8-pos-requires-affine-row is added for the
behaviour this version introduces. One pinned-sentence probe reads its
actual from a buffer element instead of a literal, because a conversion
of a known constant is now discharged; the sentence it pins is unchanged.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## 5cfec1e 2026-09-05 The named const is the number it names: v0.47

[ENT-2] clause (c) already names the mathematical value of an integer
literal and of an integer-typed named const in the same breath, so a
const was a constant term everywhere except in the relations written
about it. [INV-1] excluded it from the affine atoms, which left one
declared value with two spellings: the name in the body, the repeated
digits in every invariant and every `use` that reasoned about it. R3
admits one way to say anything, and W1 says which spelling to keep — the
duplicated one is the worse, because a limit declared once then has to be
maintained at every site, and a stale digit is a silent divergence
between what the code enforces and what the proof states.

The const folds to its value at formation, so the admission carries no
consequence downstream: no atom kind, image, kill, or join changes. The
check is that the two spellings agree — the same loop written over `cap`
and over `255_u64` produces the same required relation byte for byte,
including its rendering in a failure's residual.

A const-generic parameter is symbolic rather than closed and is not this
admission: an affine factor is a number, and a symbolic constant has no
number to fold to. It is registered as open.

Conformance adds inv1-pos-named-const-atom and modifies nothing; the
boundary is recorded in APPROVALS. Coverage is 136/136 before and after,
the snapshot corpus holds at 491 pass and zero flips, and the qualification
tripwire is reviewed and carried forward.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## b735515 2026-09-05 Record what shipped against what the evidence asked for

Three amendments landed in the order the sweeps ranked them. Seven of the
186 constructed probes move across v0.45, v0.46 and v0.47, every one from
refusal to acceptance, and the snapshot corpus holds at 491 pass with zero
flips through all three.

L2 is not taken and the note says why rather than leaving it open: the
measured case is matrix multiply's inner index, whose residual cancels to
an affine premise already held, so what is missing is a representation for
the polynomial the step passes through. AffineInequality segregates its
constant as `upper`, and multiplying a premise by a term makes that
constant a polynomial, so the constant folds into an empty monomial before
any of it is expressible.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## 2f37957 2026-09-05 L2 is a spelling decision, and the machinery is already right

The matmul inner index needs one term-scaled premise and one plain one.
Worked through, the degree-2 monomials cancel exactly and the residual is
the constant zero, so DIRECT closes it with nothing further. The same
shape at a literal stride compiles today — recorded as
matmul_literal_stride_certificate.wf, which accepts — so the certificate
pipeline, the scaled sum, the residual check and the diagnostics are all
already correct for this class. What is missing is only that the
multiplier may not be a term.

That also settles a scoping question. The kernel-side reading put the
break in AffineInequality's segregated `upper`, which cannot hold `m * u`
when a premise is scaled by a term. True of a design where the polynomial
lives in the shared type, and avoidable: if a written certificate must
reduce to an affine residual — which this class does, to zero — the
monomials exist only inside the certificate's own accumulator and
AffineForm, the fact state, the kill and the join are untouched.

The grammar is where it stops. Both candidate productions were written,
the tables regenerated, and the parser run; both raise PredictiveConflict.
`proof_use` already admits a bare affine relation as a source and
`affine_term` is `affine_factor ("*" affine_factor)?`, so `use a * b <= c;`
is legal and after `use IDENT *` the distinguishing token is arbitrarily
far away. The tables are strong-LL(2). The literal multiplier escapes only
by lexing: a bare decimal with no type suffix is a different token class
from an affine literal, which must carry one.

So admitting a term multiplier needs a new fixed grammar atom, and which
spelling is a taste question R3 settles by evidence measured under W1, not
by the compiler. Four candidates and their costs are in L2-SPELLING.md;
the only one that leaves one spelling for one concept is also the only one
that breaks every `use 3 * X;` already written. The rest of L2 is
understood and sized and deliberately unwritten, because writing it before
the spelling is chosen would fix the spelling by default.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## 92b19e1 2026-09-05 Slice 3 step (iii): the scheduler core becomes the POSIX runtime

sched/entry.c and entry.h are the platform layer over core.c: the one
wf__sched_core, the WF_WORKERS and WF_STACKS policy, the worker threads
with their reserved host stacks and rendezvous, and the wf__par_* module
ABI as thin functions over the core. wf_floor.c runs wf__main_body on a
pool stack whose bottom is the scheduler loop and returns the posted
status on the host stack. Every I/O join runs the core's rule when the
caller is on a pool stack and waits in place otherwise.

par_runtime.c, completion/writer_scheduler.c and writer_scheduler.h are
deleted; the core is staged at every link site under the union of the
parallel and completion predicates. The Windows units keep the retired
writer ABI behind _WIN32 until step (iv).

Two defects found on the way: the bridge's wake seam answered "not
mine" before the bridge initialized, so a park before the first
operation could sleep on the wrong condition and miss its wake; the
wake epoch now initializes under its own pthread_once. The atexit
shutdown no longer destroys the ring while the pool is running.

Worker start measured before chosen: eager start at the core's entry
cost 2.7x on the io bench (0.2851 s against 0.1063 s before), lazy
start at the first lane acquisition is neutral (0.1083 s), so the start
stays lazy as design §5 says. The par_layout regression at W=4 and W=8
is park-on-miss itself and is recorded as §12's first item for slice 4.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## babf5c7 2026-09-05 Slice 3 step (iv): Windows takes the shared runtime

One scheduler core, one entry, one bridge, one file adapter and one
record on every platform; a platform is only the leaves that call the
host. sched/prim_windows.c is prim_host.c's twin: fibers for the
switch, _beginthreadex with a reserved host stack, an SRWLOCK and
condition variable behind the same wake seam, an address-only
VirtualAlloc reservation with one committed page per slot for the
core's state header. sched/entry.c carries one startup policy with no
platform conditional: thread creation, the thread's attach and detach
for the switch, the CPU count, the once and the rendezvous are all
behind prim.h, the last two over the core's own atomics and yield.

The bridge has one ring seam with three arms (io_uring, IOCP, none);
the IOCP ring finds the record by its embedded OVERLAPPED and its park
is the completion port itself, a wake being one posted packet per
announced sleeper. The file adapter is shared, with file_posix.c and
file_windows.c as its host leaves; the wait set is an opaque block the
platform's wait_host.c or wait_windows.c fills, so contract.h names no
host threading API. The record grows from 128 to 160 bytes to hold the
ring's state in a union and the open's descriptor class.

One rule for WF_WORKERS, WF_STACKS and WF_IO_HELPERS on both platforms:
unset is the caller's default, 0 through the ceiling is that number,
anything else ends the run before the program body with one diagnostic
line. WF_WORKERS 0 or 1 stays the sequential opt-out.

Deleted: par_runtime_windows.c and its probe, writer_scheduler_windows.c,
windows_completion.c/.h and its probe, windows_bridge.c,
windows_blocking.c/.h and its probe, the capacity probe and observer,
native_completion_api.h, and the capacity corpus program. The
completion-windows job is rebuilt around the shared probes, the grant
observer is one file both platforms link, and a new step requires the
floor to classify an overflow on a pool fiber. mingw-w64 and wine are
wired as optional local proxies (completion-windows-cross,
completion-windows-wine), outside check.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## 086d251 2026-09-05 Read runtime settings through the platform layer

The MSVC C runtime marks getenv deprecated and the Windows job builds
with -Werror, so the shared units that read a setting failed on the
real host. prim.h gains P4, one setting read into the caller's buffer:
getenv on POSIX, GetEnvironmentVariableA on Windows. entry.c's setting
rule, the bridge's two environment reads and the default-route probe
go through it; no shared unit names an environment API of its own.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## 3683d84 2026-09-05 Stage every header on every platform; name every fail-stop

The driver's staged tree carries every header of the runtime on both
platforms and only the compiled unit list differs, so the include
closure of a shared unit such as bridge.c is the same fact on every
host; the three test stagings follow. The ring-refusal reader moves
under the two ring arms, since a target with no ring is already that
route (Darwin built it unused).

Every abort in the bridge, the IOCP ring, the Windows host runtime and
the Windows floor writes one line naming its site before it ends the
process; the Windows job shows both channels and the exit status of
every run before judging it, and requires the bridge's own line from
the initialization fail-stop. The port association asks for the
skip-on-success mode before binding the handle, because a handle cannot
leave a port and a refused handle must still be usable by the adapter.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## 58d39d6 2026-09-05 The Windows host runtime no longer reaches the bridge

An open records the descriptor's class in the runtime's own table and
stops there; the port association stays where the ring already does it
lazily, at the first offer of a record on that descriptor. A program
that submits nothing links the floor and the host runtime with no
completion runtime, which the real host's HostString step proved and
completion-windows-cross now checks by symbol. HOST_LINK_LIBRARIES is
imported only where it is read.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## 29a48ac 2026-09-05 Bind a handle to the port exactly once, under the table's lock

Every lane of a program may offer its first record on one descriptor
at the same moment; the check-then-bind ran outside the descriptor
table's lock, so the losing lane's record fell to the file adapter,
whose overlapped read on the now-bound handle posted a packet the
reaper could not account for. The association is now one critical
section in windows_runtime.c, with the bind supplied by the bridge so
the host runtime still reaches nothing of the completion runtime. The
Windows file leaf sets the low bit of its OVERLAPPED event so its reads
post no packet to any port the handle is on, which also covers a shape
the ring refuses on a bound descriptor. The ring's fail-stops carry the
error code.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## 0627bf2 2026-09-05 Harden the grant observer's exit path and add a control to the Windows pool step

The observed --par executable is the one Windows link that calls the
C runtime from an exit handler while detached workers still run their
loops; the observer now formats its line itself and makes one fputs.
The workflow step records that its link set is exactly what the driver
stages for this module, and on failure runs the same link without the
observer as a control, then an AddressSanitizer build and cdb when
present, so the host names the fault.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## 427b9aa 2026-09-05 Register the grant observer's report through atexit

On the MSVC target a destructor attribute becomes a C-runtime
terminator, walked by _initterm inside common_exit after the stream
locks are released; the observed --par executable died in fputs there
with the program's own bytes already written and the workers parked
where they belong. The report is now registered with atexit from a
constructor, which runs before any terminator with the streams intact
on both platforms. The sanitizer diagnostic finds its runtime DLL in
clang's own directory.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## 20905c7 2026-09-05 Record the real host's verification of step (iv) in the plan

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## 8e43c54 2026-09-05 Slice 4a: the park-on-miss measurements, nothing chosen

research/experiments/park-on-miss-measurements/ holds the method, every
table of design §12 and the plan's added choices, the exact commands and
the bars; run.sh reproduces them behind make park-on-miss-measurements,
outside check. Each alternative is a compile-time define read by the C
unit and carried into every gate build by SCHED_VARIANT_DEFINES, so the
enumerator judges a form on the shipped form's terms. Four of the six
behavioural variants are rejected by the enumerator and one passes it
and then hangs par_layout at two or more workers; only the lane slot
count is separable, and 2 slots costs 12 percent on the grid at W=8.
The compute-miss regression stands at 45 and 103 percent, the park and
publish round trip at 4.4 microseconds best on a host whose own park
and wake is 16.2, the record grows by exactly 32 bytes per outstanding
operation, and every hand-out entry's chain bound is at most 80 bytes.
io-completion-bench gains the four-stage chain in its four shapes.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## 44745ff 2026-09-05 Slice 4b: close batch 2 except the compute-miss decision

The six measurement variants leave core.c, core.h, bridge.c and the
Makefile; the shipped form is the only text again, and the measurement
bundle keeps every table as the record with one sentence per retired
section. The enumerator records each schedule's external inputs, the
completion order and the heads its stubs delivered, and replays them
from a fresh core, comparing every stack phase change, every lane slot
swing, the terminal statistics and the enabled process at each step
(design §11 item 24); the enumerate line reports the replay counts.
LOOP-PIPELINE §3.4 and the roadmap's stackless and Windows items are
corrected in place; the handoff is folded into the plan and deleted;
docs/done/0107-park-on-miss.md is the batch record. The measurement
runner no longer hard-codes a checkout path, which repository-invariants
refused on 8e43c54. Design §12 item 1 stays open in the plan.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## 0d23cd2 2026-09-05 Bring the Windows qualification bench to the shared runtime

windows-bench.ps1 pinned the lowering slice 2 replaced and linked the
units slice 3 deleted, which is the one red check on the pull request.
The IR assertion now pins what the emitter writes for the mixed
program: the first read in flight across the compute member and the
source-last read in wf_main, and the lane protocol inside
wf_compute_pair (design §4, §8). The observed link is the driver's
Windows unit set plus the grant observer, and the retired probe
counters are replaced by the grants line and the bridge's own
statistics; the ring evidence is WF_REQUIRE_WINDOWS_IOCP with exit zero.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## 1a7048d 2026-09-05 Record the owner's reading of the compute-miss fallback as the colouring question

The plan's open item states that the fallback is the colouring question
and is accepted if it is what recovers the performance, sequenced after
the first API that waits, and notes that [FN-1] already derives the
never-suspends or may-suspend summary per function, never written, so
the hand-out's target-action bit would be that summary rather than a
new spelling.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## 0b8e406 2026-09-05 Propose streams and TCP under T4 as the first waiting API

The owner revised the order on 2026-09-05: the network is concurrency's
real demand and a loopback is a controlled peer, so TCP comes first,
with command.stdin as the first instance of the stream design and the
standard streams renamed InputStream and OutputStream. NETWORK.md
carries the T4 accounting of every socket resource, the types and
operations in the specification's own forms, the runtime routes per
platform, the control test, the slices and the decisions the owner
still has to take; the plan points at it.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## 5c31b74 2026-09-05 Revise the network proposal: two halves from the start, the factory rename, the bar

Full duplex is designed in rather than added later, because a later
split would need second receive and send signatures; a connection is a
receive half and a send half from accept or connect, the pair close
returns the credit, and derived release of a half is the host's
half-close. The descriptor factory is renamed with the spelling left to
the owner; the operation names follow the stream pair (read_next and
receive_next, write_once and send_once); the reference for the control
test is the fastest existing solution regardless of language, a
hand-written io_uring server with multishot accept and receive, with an
epoll server as the second reference.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## 6884dd2 2026-09-05 Network proposal: the connection is a two-field system struct; every decision settled

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## d7073df 2026-09-05 A use cites one premise: `times` and delimited premises

The Farkas coefficient was spelled with `*`, which is a category error: it
is the multiplicity of a premise, not a multiplication, and its right
operand is a relation rather than a number. Three defects followed from
that one pose — the form read as `n * bool`, a term multiplier was
unparseable because strong-LL(2) cannot separate `use IDENT *` from an
affine relation, and telling the two apart needed a whitespace rule the
parser cannot see.

    proof_use   := "use" (("[0-9]+" | IDENT) "times")? use_premise ";"
    use_premise := IDENT | "(" affine_expr compare_op affine_expr ")"

A `use` now cites exactly one premise — an invariant name or a delimited
relation — optionally prefixed by `N times` to state its multiplicity.
`times` is evidence-selected rather than chosen: it has zero uses as an
identifier in the corpus, and four of its fifteen doc-string appearances
are the corpus explaining this very construct in prose. Every form is
decided within two tokens of `use`, and the whitespace rule is replaced by
an ordinary keyword-paren space of the kind the `for` header already has.

Parentheses become mandatory on the relation form. Today's split — bare
when unmultiplied, parenthesized when multiplied — was the second scar from
the same ambiguity, and with `*` gone it has no reason to exist. A
certificate is a list of premises with multiplicities and `use` names one
line of it, so a premise deserves one shape.

The premise becomes its own production node, so the role and check layers
read it there: a `proof_use`'s own direct IDENT can now only be a term
multiplicity, and the `use_premise`'s is the invariant it cites.

The term multiplier itself is admitted by the grammar and resolves as a
proof value; the certificate arithmetic that consumes it follows.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## c17dc52 2026-09-05 A use may be scaled by a value: v0.48

The certificate step `use n times (p < k);` is the one thing the L2 evidence
asked for and the only thing that was missing: the identical certificate at a
literal stride already compiled. A matrix multiply's inner index at a runtime
stride, `n*p + j < n*k`, is one term-scaled premise plus one plain one, and
its residual cancels to zero.

Scaling a premise by a value makes the accumulated sum nonlinear, and the
question was where those monomials live. They live nowhere: the accumulator
starts affine, becomes a degree-two polynomial at the first term multiplicity,
and every nonlinear monomial must fold back to the value image an admitted
exact multiplication already bound before the residual is formed. What comes
out is an ordinary affine inequality, so the residual, its integer tightenings
and the L0 route are the same ones a bare-decimal certificate uses. No fact,
published conclusion, invariant target or `affine_expr` can hold a nonlinear
term, and a sum that keeps one rejects with its own diagnostic rather than
being measured against a weaker target.

Folding the sum rather than expanding the target is the direction that cannot
lose: expanding rewrites a proposition that is already affine and can turn a
provable residual unprovable, while folding only ever removes monomials.

The multiplicity must be unsigned. Nonnegativity is exactly what makes `m*p
<= 0` follow from `p <= 0`, and taking it from the written type keeps it
structural instead of becoming an obligation the certificate has to discharge
before it can start.

What an admitted product equals is recorded as value identities — the atom the
multiplication bound and its two operand atoms — so the map needs no kill and
no join: a write mints a new atom that is simply absent, while the old atoms
keep denoting the old values. The same walk now drops a stale measurement
before it judges, so a loop body's second pass cannot read the first pass's
operands.

Conformance adds four cases and modifies eleven, all recorded in APPROVALS;
one of the eleven, gram4-neg-multiplied-use-relation-bare, keeps its rejection
by moving its bare relation after `times` rather than acquiring the
parentheses the migration would have given it. Coverage is 136/136 before and
after and the qualification tripwire is reviewed and carried forward.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## c128e08 2026-09-05 Record what v0.48 moved, and what it did not

The two probes that wrote a term multiplicity were written to demonstrate the
grammar's refusal, and both carry targets AUTO proves on its own, so neither
becomes an accept now that the form parses. Saying "two probes moved" would
have been the convenient reading rather than the measured one. What the
capability is measured by instead is two probes that state a real target: the
matmul inner index at a runtime stride, and the same certificate citing a
named premise rather than a written relation.

`grid/G5_scaled_use_runtime_factor.wf` keeps its place in the grid sweep with
its doc string corrected and a re-measurement appended to that sweep's recorded
verdicts, rather than being edited into agreement with the new version.

L2-SPELLING recorded four candidate spellings and called the decision open;
what shipped is none of them verbatim, and the note now says so: the fourth
candidate's goal — one spelling for one concept — was reached by moving the
marker in front of the premise instead of adding one to the literal form. It
also records the one design detail settled during implementation, that the sum
folds down to admitted products rather than the target expanding up.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## 270cf9d 2026-09-05 Fold to the product the target names

Two bindings can hold the same product, and the least-atom tie-break refused
the case where the target names the other one — a rejection with no reason a
writer could see, since the two values are equal. The target's own text picks
among them now: a monomial folded to the value the target already names
cancels against it, while one folded to an equal value under another name does
not. Still one pass and one winner per operand pair, so it is a canonical form
rather than a search.

Found by probing the fold adversarially rather than by a failing test. The same
probes confirm the two rebinding cases stay refused: a write to an operand or
to the product mints a new atom, the monomial no longer matches what was
recorded, and the residual does not close.

Adds prf1-pos-term-multiplicity-shared-product; the specification sentence, its
digest, the approval record and the identity move with it.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## 4fca327 2026-09-05 Say why folding is the direction, not just that it is safer

The reason was written as a preference between two symmetric choices. It is
not one: folding is bounded and expanding is not. A sum holds finitely many
monomials and each fold removes one, so folding terminates in the degree-two
domain it started in. An operand of a product may itself be a product, so
expanding `let a = n * p; let b = a * q;` reaches `n*p*q` — a degree nothing
in [PRF-1] can hold. Folding is closed under this domain; expanding is not.

Comment and derivation row only; the rule and its bytes are unchanged.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## 644f090 2026-09-05 Compile the three pose writings instead of predicting them

PROOF-SURFACE claimed only one of the three plausible-but-wrong writings the
`times` pose invites becomes a parse error, and that the other two are caught
after parsing. Both halves are wrong, and wrong in the direction that
flattered the note: two are parse errors, and the third is a resolution
failure naming the invariant-name domain, which is the right answer rather
than a confusing one.

All three now carry the verdict they actually produce. What remains a residue
is narrower than the note claimed: the value-spelling case rejects correctly
but no diagnostic says why that position wants an invariant name.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## ce95928 2026-09-05 One of the two registered restrictions is not a gap

The v0.48 row registered two limits on the fold. The second — that a product
admitted through the finite L0 route rather than an affine one records
nothing — reads like a hole in the feature and is not one. A product that
route admits has at least one compile-time-known operand, so its value image
is already a constant or an affine scaling and no nonlinear monomial is ever
formed; with both operands constant the target needs no certificate at all
and a written one is the redundant-block rejection. Both cases compiled.

Registering a case that cannot arise costs a future reader the time to find
that out, so the row now says what was measured instead. The first
restriction, a derived operand expression such as `(n + 1) * p`, stands.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## db9892c 2026-09-05 Repair the old spelling where it was quoted inside prose

An audit of the migration diff turned up a third failure mode neither of the
two already repaired covers. The regex rewrote code lines correctly and left
untouched the same syntax quoted inside an adjacent English doc string, so two
snapshot cases still describe an `1 *` multiplier that their own code no
longer writes. Both keep their verdict and cited rule; only the sentence a
reader trusts was wrong. The mirrored doc column in the corpus index moves
with them.

The same staleness reached one standing fact in `mcts_mem`, which is the part
of that file read as current rather than as history, and which had it that
only a multiplied relation premise is parenthesized. It now says what the
grammar says, with a dated Moves entry recording the supersession as that
file's own convention requires.

Snapshot holds at 491 pass, 0 flips.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## a72c845 2026-09-05 Cite the position's own rule, and stop the note claiming what it ran

Fuzzing the new grammar surface turned up a diagnostic this batch broke and a
specification sentence it left contradicting the compiler.

`use 3_u64 times pair;` cited CONST-1, whose text is about `array<T, N>` sizes
and `const` targs. `raw_restriction_owner` names a production's own rule
rather than a lexical class, and had been written when `const` was the sole
production using the grammar's pattern predicate; v0.48 removed the
bare-relation alternative that used to route a suffixed literal down the
expression path, so the heuristic started answering for a position it does not
know. Measured against a pre-v0.48 build, the old citation was GRAM-5 by that
other path — so this is new, not inherited. The arm now recognizes the `const`
position by its own two alternatives and otherwise leaves the ordinary parse
rule to answer, with a note that a third sharer needs the production threaded
rather than a second shape test.

[INV-1]'s `compare_op` sentence said `==` in a relation-form premise cites
INV-1, while [PRF-1] says a relation source is owned diagnostically by PRF-1
and the compiler follows the second. One restriction, stated once; only the
citation follows the owning position, and the sentence now says so.

PROOF-SURFACE opened by claiming every verdict in it was compiled. Two were
not: a `[GRAM-4]` for a suffixed multiplier that is really GRAM-5, and a
position table whose clause row read as writability when it records row
admission — `requires a * b <= c;` is GRAM-5 for arity, which the
investigation's own README says and the table did not. Both re-measured, and
the header now says which verdicts were run when.

Adds gram4-neg-typed-multiplicity so the citation cannot drift back.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## b50d21f 2026-09-05 Record the affine-nesting crash where gaps are recorded

Fuzzing found the driver aborting with a stack overflow and no diagnostic on a
proof expression nesting parentheses about 1400 deep. It is not this batch's:
a build from before the v0.48 amendment crashes at the same depth, and it is
reachable from an invariant target as well as from a `use` premise, so it sits
in the shared affine handling.

Not fixed here. Nothing a writer writes nests 1400 parentheses, and the repair
wants a bisect between the parser and the semantic former plus, most likely, a
normative sentence for the new limit — which is a version of its own rather
than a rider on this one. What it must not do is go unrecorded, so it is
written where the roadmap says implementation gaps belong, with the
reproduction, the measurement that dates it, and the condition that removes it.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## d2f2239 2026-09-05 Measure how far a composite multiplicity reaches

A sweep reported that `m = a + 1_u64` and `m = 2_u64 * a` fold and close while
`m = a - 1_u64` does not, and read that as an asymmetry worth a compiler look.
Held to one target it is not one: addition and subtraction behave identically,
and the line runs between a pure scaling of the atom and an image carrying a
constant term. The sweep's three probes carried three different targets, so
the multiplicity was not the variable under test.

The rejections are also correct rather than incomplete. Scaling `p - k + 1` by
`a + 1` leaves the residual `k <= p + a`, which the premises do not imply and
`p = 0, k = 1000, a = 2` falsifies.

Recorded beside the L2 probes with the six-row table, and with the axis the
ledger already registers — a composite in the product's own operand, which
records no atom and stops at NonlinearCertificateSum — separated from it,
because the two are easy to conflate and fail differently.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## 8cd4bae 2026-09-05 Batch 3 slice 1: specification v0.46, streams and TCP under T4

One amendment, landed as one change: the active specification is v0.46
over v0.45's archived bytes, the approval record is appended, the
identity module regenerated, the qualification review moved. Output is
OutputStream; FileFactory, FilePermit, command.files and reserve_file
are HandleFactory, HandlePermit, command.handles and reserve_handle,
because sockets draw from the same descriptor table. command.stdin at
ordinal 5 supplies an InputStream and read_next reads it through the
one submit-then-join lowering: the ring's IORING_OP_READ at offset -1
on Linux, the shared adapter elsewhere, ReadFile on Windows. TCP enters
the language whole: SocketAddress with its two pure constructors,
TcpListener, TcpConnection as a system-declared struct of TcpReceive and
TcpSend, the three outcome enums handing the permit back, tcp_listen,
tcp_accept, tcp_connect, receive_next, send_once, close_connection and
close_listener under [SYS-15] to [SYS-18]; their runtime routes are
slice 2 and a module that submits one stops at target qualification
with a missing mapping, never a source rejection. Eleven conformance
cases are added and thirty-one respelled with their verdicts unchanged.

Two defects found on the way: the lowering decoded a system constructor
ordinal against the active inventory rather than the unit's own, which
a second nominal-record block exposed; and no checked borrow form
carried a field path for a non-buffer type, which &uniq
connection.receive needed.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## 33bb75f 2026-09-05 The registered remedy does not work, and the restriction is wider than recorded

A stride sweep put v0.48's term multiplicity against the domain it was written
for and found it refuses essentially every natural stride. Held to one shape,
a stride copied from a parameter accepts and `width + padding`, `width + 4`,
and `2 * width` all reject: any derivation kills the fold, and a stride in
this domain is definitionally derived.

Two things in this batch's own record were wrong. The ledger offered "the
writer must bind the sum first" as the remedy, but `let stride = width +
padding;` is binding it and the binding's image is still the sum, so no
product is recorded; the only workaround is a whole extra function to make the
checker hold the operand opaquely. And the compiler's mechanical fix told a
writer who had already bound the product to bind the product. Both now name
the real condition.

The ledger also said no corpus program had asked for it. A sweep of nineteen
padded-bitmap, DIB, alignment, tiling, volume and audio programs has now asked
for it, so that sentence is replaced by what was measured.

Not repaired here. What it points at is not more proof power: [PRF-1] resolves
a named premise by declaration identity rather than by re-deriving its value,
one sentence away from where the multiplicity is expanded into its image
instead. Applying the same rule to the multiplicity and to a product's
operands is the candidate, and it is a version of its own.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## 60a4f88 2026-09-05 Say what PRF-1 admits, and stop borrowing a narrower term

A sentence-by-sentence audit against the compiler found two places where the
v0.48 text says something the implementation does not.

PRF-1 enumerated three declaration syntaxes for a named multiplicity — local,
parameter, const. The implementation admits any live own-mode unsigned integer
value binding, and a counted loop binder and a match binder both go through.
The rule now names the class rather than a list, which is how [INV-1] already
words the same admission one page away. The moved-local clause moves with it:
every admitted type is copy, so [OWN-1] rejects a moved one first and the
clause described a case that cannot be built.

The fold's precondition said the multiplication's domain "discharged through
an affine route". This specification uses that phrase for the constant-operand
normalization specifically, and contrasts it with the interval-product rule —
which is the only route the flagship non-constant case can take. The sentence
now says what actually fixes the images a fold names, and keeps the one
exclusion that is real: the finite L0 route reads no affine image and records
nothing.

Adds prf1-pos-binder-multiplicity, which had no coverage in either suite.

Also records two pre-existing costs beside the nesting crash, both verified
against a pre-v0.48 build: a runtime-sized allocation fails at TargetLayout
with no rule and no location, which a 22-program sweep hit in all 22 and which
is why the corpus carries `1000_u64` ceilings; and a proof block costs about
eight times per doubling of its entries, putting PRF-1's 4096 ceiling — which
the rule calls "not a work or time budget" — many hours out of reach.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## 4a21668 2026-09-05 Arm every pool fiber's emergency stack from inside the fiber

The io-hosts overflow proof passed on 5c31b74 and segfaulted on 6884dd2
with the same runtime bytes, and again on 8cd4bae: the floor set the
stack guarantee on each thread's host stack, but Windows keeps a
guarantee for the calling thread or fiber, and a fiber takes it only
when set from inside that fiber. A pool fiber therefore overflowed with
nothing under the handler but what was left of the guard page, and
whether that was enough depended on where in the page the descent
stopped, which the steals decide. wf_prim_fiber_main now calls the
floor's attach as every pool fiber's first frame, so the sixty-four
kilobytes are there on every stack a program runs on; prim_windows.c
carries the weak answer for a core linked without the floor, as it does
for the bounds seam. The io-hosts step runs each configuration five
times, because one draw is not evidence against an intermittent defect.
The design and the plan record the platform fact in place.

Verified here: completion-windows-cross and completion-windows-wine on
the clean revision, and repository-invariants. The real host is the
judge; wine does not swap the guarantee between fibers.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## 2d455e5 2026-09-05 Reach the floor's attach through the primitive layer, one weak answer per link

The real host's linker refused 4a21668 at the first link: LNK1227, a
conflicting weak default for wf__floor_attach_thread, because entry.c
and prim_windows.c each carried one. Moving the weak answer into the
platform leaves alone then failed the mingw proxy the other way: GNU ld
satisfies a PE weak default only for references from its own unit, so
entry.c's strong reference went undefined. So the per-thread half of the
floor is now reached the way the per-stack half already is: entry.c
calls wf_prim_floor_attach, P2's second name in prim.h, and each
platform leaf is the one unit of a link that names the floor's attach
and the one that carries its weak answer; the Windows leaf calls it on
every pool fiber's first frame as before.

Verified on the clean revision: completion-windows-cross and
completion-windows-wine, make completion-test, and cargo test --profile
gate --lib backend (302 passed). The real host is the judge.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## 6311482 2026-09-05 A bounded spin before the idle park, measured

The Windows qualification bench runs to its bar on the real host and
misses two of them: the unified --par build of windows_runtime_mixed.wf
at 1.06 times its completion-only build against 0.95, on a four-vCPU
Windows VM where a park is a completion-port sleep and both wakes on
the loop's critical path, the idle worker's to steal the hand-out and
the publisher's when the stolen job readies its joiner, are kernel
round trips. The retired runtime found both by spinning before it
slept; the core parked the moment its last look missed.

wf_sched_idle_step now repeats its own looks before it parks, for
WF_SCHED_IDLE_SPIN_ROUNDS rounds of the new wf_prim_pause and then
WF_SCHED_IDLE_YIELD_ROUNDS rounds of wf_prim_yield. Where the spin sits
is most of the result: in front of the drain it delays every I/O
completion by its own length (16 rounds cost many_files_wide 19
percent, 4096 cost it forty-one times); after the drain and the window's
own last look, immediately before the park, the I/O line is flat across
the whole sweep. The epoch capture does not move, so section 6's
lost-wake argument is unchanged. The constants are 256 and 16, about 11
microseconds of looks against this host's 16.2 microsecond park-and-wake,
from a sweep of seven pause counts by three yield counts over the mixed
program's three builds, par_layout and the grid at four worker counts,
many_files_wide and the park-and-publish round trip, CPU beside wall on
every line; the addendum in the measurement bundle holds the tables and
run.sh reproduces them. Under the enumerator a round is a step, so the
enumerate build pins one spin round and no yield round, and the Makefile
and tests/sched.rs say why a yield round contradicts that model. Section
12 item 1's compute-miss regression narrows to 11 and 17 percent on
this host; the Windows job is the judge of the bar.

Verified on the clean revision: format and lint, completion-test with
the four enumerator configurations, cargo test --profile gate --lib
(1493 passed), completion-windows-cross and -wine, conformance-run
(Pass=531 Xfail=1 Skip=1), snapshot-run (Pass=491 Flip=0),
repository-invariants.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## 8cd617d 2026-09-05 Keep the measurement runner executable

6311482 staged run.sh from a blob and lost its mode; the Makefile's
park-on-miss-measurements target runs it directly.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## 436222c 2026-09-05 Record the Windows judge's table for the idle spin

bench-windows-qualified on 6311482 met every bar with the chosen 256 and
16: mixed-full 0.6920 and mixed-total 0.6878 against 0.95, the whole
p10..p90 band under 0.70, which is the ratio the Linux host gives. The
run just before the spin met the bars without it at 0.86 on another
runner, with the completion-only reference itself 12 percent slower, so
the record states that the Windows VM moves between runners by more
than the bar's margin and that the spin's evidence is its 0.69 beside
the 0.86 and 1.06 of the runs without it.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## debece8 2026-09-05 Batch 3 slice 2: the TCP runtime on POSIX, and loopback tests

Every TCP operation lowers and runs on POSIX. qualification.rs maps
ordinals 22 through 28 on the native column and leaves them unmapped on
the Windows one, so a Windows submission still stops at qualification.
The emitter carries the seven wrappers in the one submit-then-join
shape, with the three outcome enums built as FileOpenOutcome is and the
permit handed back in every failed variant. wf_file_request gains six
kinds inside the record's 160 bytes: listen, accept, connect, receive,
send and the half-close; the accept's peer record lives in the arm the
accept alone uses, because twenty-four more bytes on the shared result
head would put the record past the block an emitted frame reserves. A
SocketAddress crosses the ABI as the three scalars of its emitted
layout. file_posix.c executes every kind with socket, bind, listen,
accept4, connect, recv, send, shutdown and close, IPv4 and IPv6,
close-on-exec, no SO_REUSEADDR; linux_io_uring.c carries accept,
connect, receive and send on the ring, with a connect's socket and
native address made in the submitting call; file_adapter.c keeps the
pair's two-count so the second release of a connection closes the
object and spends the credit. tests/programs gains tcp_echo, tcp_client,
tcp_fanout and tcp_refused, each run on both routes against a std::net
peer, the adapter probe gains a loopback round trip on the ring, and
the harness the pair's accounting. The five systcp-* conformance cases
move from unsupported to accept because their subject did.

Two compiler changes were needed: a permitted may-suspend call of an
operation the typed adapter has no hand-out form for keeps its
qualified wrapper instead of reaching an internal error, and a
system-declared struct keeps its catalog identity into the IR so the
backend resolves close_connection's parameter type.

Open and recorded in the plan: the fixed-trip accept loop is denied
both actualizations by the permission judgment, so four peers are
served one at a time; the server-loop shape is the language work
NETWORK.md section 6 names.

Verified on the clean revision: format and lint, completion-test, the
five sanitizer targets, cargo test --profile gate --lib (1493), --bins
(11), --test programs (62 passed; the three root-permission cases pass
without cap_dac_override), completion-windows-cross and -wine,
conformance-run (Pass=531 Xfail=1 Skip=1), snapshot-run (Pass=491
Flip=0), repository-invariants.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## 1ad703b 2026-09-06 Read the peer's family through the host's record; stage the accept loop; report the core's counters

The macOS gate failed debece8 twice over. The completion harness's
accept published the zero address because wf_socket_address_from_native
read the family as a sixteen-bit number at the record's first byte,
which is sa_family on Linux and sa_len beside a one-byte sa_family on
Darwin; it now copies the host's own struct sockaddr and reads the
member. The sampling case that expects a --par binary with no worker
setting to be granted lanes has failed on the macOS runner since the
idle spin landed, while passing on Linux at three cores here; the core
now formats its summed counters as one line when WF_SCHED_REPORT asks,
read at the core's entry under the one settings rule, the observer
prints it after the grant line, and the case shows it when it fails, so
the next macOS run says what the threads did rather than the zero alone.

tcp_fanout.wf's accept loop is staged by PAR-3 as written now: the Err
arm of the permit reservation returns from the prologue instead of
writing a status from a path that never submits, and the scratch is the
iteration's own. The ledger test pins the four dispositions the loop
satisfies. What the permission grants the lowering does not yet take: a
may-suspend user call at the staged point keeps its wrapper, so the four
peers are still served in turn; the plan and NETWORK.md name the
hand-out form for that call as the next compiler work.

Verified on the clean revision: format and lint, completion-test,
network and parallel test modules, snapshot-run (Pass=491 Flip=0),
repository-invariants, completion-windows-cross and -wine.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## ae7c508 2026-09-06 Batch 3 slice 3: the TCP routes on Windows

The six socket operations reach the same two Windows engines the file
operations reach.  On the adapter route `file_windows.c` gains listen,
connect, accept, transfer, and shutdown arms over Winsock, with a
blocking `accept` on the helper thread.  On the default route
`windows_iocp.c` issues connect through ConnectEx (resolved once by
WSAIoctl), receive and send through WSARecv and WSASend, and refuses
accept on port because an AcceptEx address pair is 88 bytes and the
record has none to give, so the qualification table sends accept to the
adapter on Windows.  Listen, bind, and shutdown stay on the adapter on
every target, as on POSIX.

The socket address vocabulary moves out of `file_posix.h` into
`socket_address.h`, shared by both platforms, carrying the family read
through the host's record verbatim.  `windows_runtime.c` learns a socket
descriptor class, Winsock startup, and the socket error mapping, so one
descriptor table closes files and sockets alike.

`bridge_default_probe.c` performs a loopback round trip and reports
`tcp-port=` and `tcp-ring=`; the io-hosts workflow runs the network
programs and the refused connection on both routes on the real Windows
host with `-lws2_32` on the link line.  The qualification test checks
that every target column maps the TCP rows to one ABI symbol.

Verified in a clean worktree on Linux: cargo lib, bins, and program
tests; completion-test, sanitize, tsan, the Windows cross build, and the
Wine run; conformance, snapshot, and repository invariants.  The real
Windows host is the io-hosts run on this commit.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## 29f4e14 2026-09-06 Read the port argument through the text route in the four TCP programs

io-hosts on ae7c508: the echo server on the real Windows host never
listened on the port the step passed it, was still running when the
thirty-second wait ended, and wrote nothing on either channel, while the
bridge probe's own loopback round trip passed on both routes in the same
job. The programs copied the argument with host_copy_bytes, which is the
lossless route of [HOST-2]: on a UTF-16 host family it hands the program
the native code units, so "60527" arrived as 36 00 30 00 35 00 32 00 37
00, parse_port ended its scan at the first zero byte, and the server
listened on port 6 and blocked in its accept. The four programs now copy
the argument with host_copy_utf8, the text route, whose bytes are the
decimal digits on every host family; the Err arm is unchanged because it
reads nothing of the error. The step's never-listened branch now lists
the process's own TCP endpoints and stops it, so a wrong port names
itself the next time.

Verified here: cargo test --profile gate --test programs network, 12
passed on both routes. The real Windows host is the judge.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## 7dbbf2e 2026-09-06 Hand a staged may-suspend user call to a compute lane

A [PAR-3] loop whose staged call is a user function the checker judged
may suspend was permitted and then served in turn: the backend handed
out only system operations with a typed adapter, and a permitted user
call kept its wrapper, so tcp_fanout.wf's four peers waited on each
other. The staged point now takes the grant. The bounded-batch
recognizer in lowering/builder/loops.rs admits a second staged tail, the
call bound by a `let` with the statements after it as the remainder,
and selects the lane form by the cut's kind alone: a may-suspend user
call. A system operation bound by a `let` is an explicit decline that
keeps the loop it always had, because a submitted operation's drain
publishes its outcome at the block boundary and a remainder in the same
block would read a value that does not exist yet; the decline is
exercised by a test whose loop the ledger really stages at open_file.

The pipeline carries the form as one mechanism with the ring: the slot
holds a lane frame's address instead of a completion record, the
iteration's own bindings ride the same per-slot storage as captured
scalars, and their compiler-derived releases run in the drain on the
value that iteration allocated. The emitter acquires a frame, fills it
inside the granted edge and publishes it, or on a refused acquisition
runs the same call where it is written and leaves its answer in the same
ring element, so the drain has one thing to do either way: a null frame
loads the answer, otherwise join, load the result field, release. The
window ceiling for the lane form is the lane's own slot count,
LANE_SLOTS restating WF_SCHED_LANE_SLOTS beside LANE_FRAME_BYTES and
pinned to core.h by a test; the runtime's answer for the fanout is the
trip count, four. The sequential clone and a frame the slot cannot hold
plan no frame and run the sequential schedule.

Only tcp_fanout.wf emits the new form; the other corpus programs emit no
par.staged. text and every --par byte-identity case is unchanged. The
four-peers concurrency test passes on both routes, which is the proof
the plan named for this work; the plan, NETWORK.md section 6 and
LOOP-PIPELINE.md section 3.5 record it in place.

Verified in a clean worktree: format and lint, cargo test --profile
gate --lib, --bins and --test programs (the three root-permission cases
excepted), completion-test, sanitize, tsan, the Windows cross build and
the Wine run, conformance-run (Pass=531 Xfail=1 Skip=1), snapshot-run
(Pass=491 Flip=0), repository-invariants.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## 95e85a2 2026-09-06 Leave a peer-bound request to a helper, never to a scheduler thread

The gate on 7dbbf2e was red on the three-core macOS runner:
tcp_fanout.wf's fourth peer was not answered while the first three were
silent. Reproduced on Linux with WF_WORKERS=3 on the adapter route and
read from a hung server with gdb: every worker was inside
wf_bridge_run_own, the joining thread's own claim, blocked in the host's
recv on a silent peer, and the fourth connection had no thread left to
accept it. The adapter's helper pool grew only on the measured wait
verdict, which cannot see a wait that has not happened yet, and a pool
stack claimed its own record at the join before anything asked whether
that record could block the thread.

A request whose kind waits on a peer, an accept, a receive, a connect or
a send, is now peer-bound, decided by kind and nothing else. Enqueueing
one grows a helper whenever the cap allows, with neither the depth nor
the verdict term; a scheduler thread's progress pass takes only requests
no peer can hold up, from the end from_head names, while a pool may
exist, and takes anything under a pinned zero pool because nothing else
could run it; a pool stack withholds its own peer-bound claim at the join
once a helper exists, while a thread waiting in place keeps it because it
has nothing else to do; and peer-bound executions are not sampled into
the wait mean, which one silent peer would otherwise push past every
file operation's threshold. Adapter-route socket concurrency is thereby
bounded by WF_BRIDGE_MAX_HELPERS, and a pinned pool below the peer count
is now honestly the bound rather than silently widened by the workers'
own waits. The readiness-driven adapter, a poll, kqueue or WSAPoll over
the queued descriptors inside the park, is the ring-less host's proper
engine and is recorded in NETWORK.md section 5 as the open item.

The harness gains a case that fills a cap-two pool with accepts no peer
has reached and checks the growth, the untaken queued record, the
completion once peers connect, and the pinned-zero path. The four-peers
case spawns its server with WF_WORKERS=3 on both routes so the proof no
longer depends on the host's core count: before this change it failed
seven of twelve runs here with the gate's exact message, after it twelve
of twelve and the program alone twenty of twenty.

Verified in a clean worktree: format and lint, completion-test,
sanitize, tsan, the Windows cross build and the Wine run, cargo test
--profile gate --lib, --bins and --test programs (the three
root-permission cases excepted), conformance-run, snapshot-run,
repository-invariants.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## 493a5b1 2026-09-06 Read "the pool started" from the started-worker count, not from a steal

The macOS gate's sampling job failed the absent-worker-setting case on
95e85a2, and the counters the case prints since 1ad703b say what the
threads did: threads=3 workers_started=2 parks=0 steals=0 inline_runs=63.
The pool started, and the program ran to its end before either of its
two workers was scheduled at all on a three-core runner saturated by
the suite's sibling cases. The case read "the pool started" from the
grant count, which is a steal, a scheduling event that needs a pool
thread to be given a CPU while the offering lane still holds the work;
on that host a program this short reaches its end first, at every one
of the observation runs. That is the default doing what it should with
the CPU it was given, not the path being off.

The case now reads the property it states from the core's own count of
the pool threads it started: at least one under an absent setting, none
under either opt-out, beside the opt-outs' zero grants it already
required. The existential observation that the default build can be
granted lanes stays where it was, in the WF_WORKERS=4 case that
re-observes it over the sample of runs and passed on the same runner.

Verified here: format and lint, the case once free and once pinned to
three cores.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## c966ee0 2026-09-06 Link Winsock into the Windows bench's observed mixed executable

io-bench on 95e85a2: the bench-windows-qualified job stopped at its one
hand-written link, "link the observed mixed executable", with twenty
unresolved externals, WSARecv, WSASend, WSAStartup and the rest, all from
completion-windows_iocp.o and windows_runtime.o. Slice 3 put Winsock on
whitefootc's own Windows link and on io-hosts.yml's hand-written ones and
missed this script's. The link line now names ws2_32 as those do.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## d18b2e9 2026-09-06 Fold by the declaration a writer named: v0.49

v0.48 admitted a term multiplicity and then refused it wherever it mattered. It
folded a nonlinear monomial by the operand images a multiplication was formed
over, and a local's image is transparent: `let stride = width + padding;` gives
`stride` the image `width + padding`, so a product over `stride` and a
certificate scaling by `stride` reached the fold as different arithmetic and
nothing matched. Held to one shape with only the derivation varying, a stride
copied from a parameter accepted while `width + padding`, `width + 4`, and
`2 * width` all rejected. A stride is definitionally derived — padding, DIB
rounding, tile area, channel count, alignment — so the feature reached matrix
multiply, where the stride happens to be a parameter, and nothing else in the
domain a nineteen-program sweep wrote for it.

An operand and a multiplicity name the same value when they name the same
declaration, not when their images coincide. That is the rule PRF-1 already
applies to a named premise one sentence away, which resolves to its declaration
identity and is never reparsed from the current value of its spelling. Each
such binding now contributes one opaque handle that both sides name.

Two designs were built before this one, and each is the obvious first idea:

  Publishing the handle's defining equality as a fact keeps everything
  provable about `width + padding` provable about `stride` — and is invisible
  to the residual, which is the direct L0 route by rule. The sum folded and the
  residual came out as exactly that equality instead of zero.

  Replacing the binding's image with the handle makes every reader agree and
  the residual close, at the cost of the other side: an ordinary premise about
  the binding then needs the equality to prove, and two of the four rows moved
  from a fold failure to a premise failure. Transparency helps premises and
  opacity helps the fold; choosing one globally breaks the other.

What ships keeps the handle between the fold and the residual and unfolds it
before anything is proved, so every other premise, the target, and the residual
are written in exactly the terms they were written in before. The domain
judgment still reads the transparent images, because those intervals are what
admit the multiply at all — reading handles there was also tried and costs the
interval, failing the multiplication's own OP-2.

Six unsoundness attacks that this batch built against v0.48 still reject,
including the sharpest, where the target names a product formed under an
operand's old value. Snapshot holds at 491 pass and zero flips; conformance is
528 pass with the one pre-existing xfail.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## 866a3c5 2026-09-06 Withdraw a sentence measurement could not reach

v0.49 landed with a normative sentence saying that an operand which is not a
plain read of a binding names no declaration, contributes no handle, and makes
a certificate scaling by it reject. That is what the code does, and I wrote it
from reading the code rather than from running one — the same shortcut this
batch has already had to correct three times.

Run, it does not hold as stated, because a writer cannot get there. A
field-read operand's multiplication does not discharge its own OP-2 domain in
the first place: `shape.stride *defined k` is undischarged with both operands
bounded by `requires`, so the program stops well before any fold is attempted.
Four other shapes were tried for the same sentence and each stopped on an
unrelated rule.

A rule states what a writer can observe. This one is withdrawn from the
specification and registered in the ledger as what it is: settled about the
code, unsettled about whether it is reachable, with the measurement that says
so. The conformance case drafted for it is dropped with it rather than left
asserting a verdict for the wrong reason.

Coverage stays 136/136, conformance 528, snapshot 491 with zero flips.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## fdaff6f 2026-09-06 Pin the terminal inventory the new keyword actually made

`times` is the 99th fixed terminal, and two places still carried the
count from before it existed.

`ALL_TERMINAL_PREDICATES` was sized 106 with the eight external
predicates written at hardcoded indices 98..=105. The array initializer
copies the fixed inventory first and then overwrites those slots, so a
99th fixed terminal did not extend the array — it collided with it:
`predicates[98] = Identifier` landed on `Fixed(Times)` and the keyword
vanished from the inventory the diagnostic order is checked against.
`verify_compiler_grammar` reported the inventory and the order as
differing in length, which is exactly the mismatch it exists to catch.

The `active_compiler_grammar_is_consistent` counts were the same
staleness one layer up: 84/120/106 describe the grammar before the
`times` production and its decision were added.

Both are now the inventory `times` actually produces: 107 predicates
with the external ones at 99..=106, and 85 productions, 121 decisions,
107 terminals.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## 8a5e913 2026-09-06 Derive the external predicate positions instead of writing them

The collision the previous commit repaired was possible because two
things that must agree were written twice: `ALL_TERMINAL_PREDICATES`
copies the fixed inventory in and then places the external predicates at
literal slots, and `TerminalPredicate::index` answers with those same
literals. Nothing tied either to `ALL_FIXED_TERMINALS.len()`, so a new
keyword moved the boundary and neither followed.

Both now read `EXTERNAL_TERMINAL_BASE`, which is that length, and the
array is sized from the two lists rather than counted by hand. A new
fixed terminal shifts the external predicates instead of overwriting one.

Two tests pin what the arithmetic alone cannot promise. The inventory
must hold every fixed terminal and every external predicate exactly
once — that is the property the collision broke, and counting occurrences
catches a loss and a duplicate alike. `index` must give each predicate
its own bit inside the `u128` a `TerminalSet` is stored in; it is not an
inventory position, because the fixed terminals answer with declaration
order while the inventory is in first-occurrence order, and the bit space
has 21 free positions left. Both tests were run against an injected
off-by-one in the base and both fail on it.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## a6b31b5 2026-09-06 Batch 3 slice 4: the TCP echo control test against io_uring and epoll

The control test NETWORK.md section 6 asks for, in io-completion-bench:
uring_echo.c is the io_uring reference, raw ABI, multishot accept and
multishot receive into a registered buffer ring, one ring and one
SO_REUSEPORT listener per thread, the echo sent out of the kernel-filled
buffer with one sendmsg over the queued buffers, single-issuer deferred
task running; epoll_echo.c is the second reference, one edge-triggered
epoll and listener per thread; netload.c is the one load generator every
server is measured with, and linux-net-bench.sh the protocol, with
net-tools, net-verify and linux-net in the bundle's Makefile and one
step in io-bench's Linux job. programs/tcp_echo_server.wf is the
Whitefoot line: the connection count arrives as an argument and bounds
the staged for, which PAR-3 stages as it does a literal, and each
callee's receive-and-send loop is that connection's own sequential loop.

The bounds are set to the test's own numbers: the lane holds 1024 slots
and the bridge's default window is 1024, so a fixed-trip accept loop
naming 1024 connections keeps every one in flight, and WF_STACKS may
name 2048 stacks, which the protocol sets to 1100. The harness pins the
window at the lane's slot count.

The test found a stall in the ring at exactly 129 connections: the
completion queue of a depth-64 ring holds 128 entries, the 129th
completion went to the kernel's overflow list under IORING_FEAT_NODROP,
only an io_uring_enter moves it into the queue, and no submission was
coming because every callee was parked on one of those completions; the
descriptor stays readable for that reason, so every park returned at
once and the reaper, reading only the mapped queue, reaped nothing while
four threads spun. linux_io_uring.c now maps the submission ring's
flags word, treats IORING_SQ_CQ_OVERFLOW as a non-empty queue in the
park, and enters for no minimum inside the non-waiting progress pass
while the flag is raised; the completion queue is the caller's size
through IORING_SETUP_CQSIZE, 2048 for the bridge, 16 in the native
adapter probe, whose new case stages twenty-four reads against it and
reaps them without a wait. The server then completes 129, 256 and 1024
connections.

The result, on this four-core host, is the batch's: the Whitefoot line
is about a tenth of either reference at 64 and 1024 connections and
about half at one, and the table in the bundle's README and the batch
record says so line by line. 8192 connections in flight is outside the
shapes and the stack pool and the table stops at 1024.

Verified in a clean worktree: format and lint, completion-test with the
overflow case, sanitize, tsan, the Windows cross build and the Wine run,
cargo test --profile gate --lib, --bins and --test programs (the three
root-permission cases excepted), conformance-run, snapshot-run,
repository-invariants; net-verify and the protocol here.

The crate root exports LANE_SLOTS so the program test that pins the
staged window reads the ceiling rather than a literal, and RESULTS.md
carries the table beside the file tables.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## c59213b 2026-09-06 Keep the gate's docs and the Darwin pin with the lane's 1024 slots

The gate on a6b31b5 failed twice over the raised lane slot count. The
crate root now exports LANE_SLOTS for the program test that pins the
staged window, and its documentation linked LANE_FRAME_BYTES, a private
item, which rustdoc under -D warnings refuses for a public one; the
mention is plain code now. And the case that pins a submitted operation
to the two-element ring asserted that no "[1024 x " array appears in the
module, which on Darwin every path buffer is, PATH_MAX being 1024 there;
it names the lane ring's whole shape, an array of that many pointers.

Verified here: format and lint, make docs, the parallel test module;
in the clean worktree: lint, docs, cargo test --profile gate --lib and
the network program tests.

The batch record and RESULTS.md gain the runner's own table from
io-bench on a6b31b5 beside this host's.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## 519a228 2026-09-06 Evict the version commentary the spec was never meant to keep

The v0.30 representation rework evicted twenty-one accumulated header
paragraphs on the stated ground that history lives in the archives and
the chain, and retired the tooling's hard requirement for the two
phrases with them — `compiler/src/bin/spec.rs` still carries the comment
saying the delta inventory belongs to the approval record and "never in
the normative bytes". Nothing enforced it afterwards, and [META-5] still
read "in this document's status header", so the practice walked straight
back. This branch put five paragraphs back, one per amendment, each with
a tail reciting every prior version's selection ground.

The header is now the title, the status line, and one line naming where
prior versions and every version's declaration live. [META-5] moves its
own declaration to the change's `governance/APPROVALS.md` record and
states the property positively, so the rule itself refuses the next
paragraph rather than inviting it: the specification states the language
and carries no commentary about its own versions, and a version's own
such text is not retained after it activates.

v0.49's delta declaration moves to its approval record verbatim, beside
the description and selection ground already there. The derivation
ledger's META-5 row said candidates put both declarations in the status
header; it now says where they go. Nothing else in the live tree assumed
the old home.

The five removed blocks are preserved byte-for-byte in
`spec/kernel-spec-v0.44.md` through `v0.48.md`, which is where the
file's own second line has always sent a reader.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## c3420aa 2026-09-06 Record the adapter route's window bound and the two steady states

The control test's adapter-route line held for two minutes at 64
connections and every connection came back reset when the run was
ended. The trace says why: eight accept4 calls in all, every helper
inside a recvfrom. wf__completion_window caps a staged loop at
WF_BRIDGE_MAX_HELPERS when no ring is the engine, so the server keeps
eight connections open, the generator closes nothing until every
connection has finished, the ninth accept is never issued, and the
listener's pending connections are reset when the process ends. The
same holds in the sequential world at a window of one, and four peers
pass because four is under eight. NETWORK.md section 5 and the record's
open items say so beside the readiness-driven adapter that removes it.

The record also names the Whitefoot line's two steady states, 95 and 36
thousand round trips a second for one configuration, as the first
variable to isolate before the runtime's structure is changed, and
NETWORK.md's status closes the batch.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## 59892e9 2026-09-06 Make the outline a map again, not a log

`How to read this outline` says what this file is for — what the project
can already do, what is missing before a direction advances, updated in
place, with detail linked rather than copied. I added a hundred lines
that are none of those: five paragraphs reciting what each of v0.45
through v0.49 changed, copied out of the specification's own amendment
prose. Four such paragraphs were already here for v0.40 and v0.42
through v0.44, and I extended the row instead of reading the rules above
it.

The cost is not tidiness. `outline:PROOF-1`'s `Current` — the field this
file calls its canonical status summary — still described only the exact
L0 boundary and the `requires` axiom, with nothing about the affine
layer, measure atoms, named consts, the published product interval, the
written certificate, or how a multiplicity folds. The log at the top
grew and the sentence that was supposed to carry the state went stale
behind it.

All nine paragraphs go. What was still landscape in the v0.40 one — the
proof pass running in the ordinary semantic compiler, what it checks,
what it erases, and that it carries no writer-reachable runtime
assertion — stays as a present-tense statement.

`outline:PROOF-1` now states what the proof surface reaches and names
premise products as the unstarted piece no evidence has asked for.
`outline:FLOOR-5` loses its chronological narration and, with it, a
sentence this batch had falsified: it still said a multiplied `use`
relation is parenthesized, when every relation premise is. `Current
baseline` drops its version framing the same way.

The rule is now written where the next paragraph would be added, with
the failure mode named so it is not a matter of remembering.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## ec7c691 2026-09-06 Stop reciting version history in the plan document

The five paragraphs I put in `roadmap.md` for v0.45 through v0.49 are
here too, byte-identical — I checked them against the copy I had just
removed. With the specification's status header and the approval record,
the same amendment prose was living in four places at once, which is the
one thing `roadmap.md`'s own reading rules forbid outright: detailed
semantics stay in their canonical owner and are linked, not copied.

The header sentence had the same shape at smaller scale, a chain reading
"v0.41 added ..., v0.42 the ..., v0.43 the ..." that I extended by five
clauses. It now points at where that history lives instead of repeating
it, and says that what the project can do now is `roadmap.md`'s job.

Seven paragraphs go, 475 lines to 353. The plan's own body — the
outcome, the determinism boundary, the automatic-route boundary, the
source forms, and the induction semantics — is untouched: it is what the
plan said, and it is not a status field pretending to track the present.

Not fixed here, because it is the owner's call: this file is titled
`Current Plan` and its status line reads IMPLEMENTED AND ACTIVATED as
v0.40, nine versions back. That gap is most likely why the recitation
started — a finished plan that nobody replaced stays "current" by having
each new version appended to it.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## e134360 2026-09-06 Reap sixty-four completions a pass and complete an answered transfer where it was submitted

The TCP echo control test, isolated one variable at a time on the
development host at 64 connections (docs/done/0108-streams-and-tcp.md
section 6):

- The progress pass a scheduler thread makes on every idle turn reaped
  exactly one completion, taking the ring's submission lock to kick and
  its completion lock to read one entry; four threads doing that for 64
  connections made 720 thousand futex calls a run. WF_BRIDGE_REAP_BUDGET
  is 64: 19 thousand futex calls, 35 to 69 thousand round trips a second.
  Worker count 2, 4 and 8 measured flat before and after, so the reap was
  a serial resource and not the core count; 1024 measures the same as 64.

- A socket send the host accepts at once and a receive whose bytes have
  already arrived were submitted, parked and woken like a transfer that
  waits. wf_file_transfer_now in the platform leaf asks once with
  MSG_DONTWAIT; an answer that is the operation's own outcome completes
  the record on the submitting thread as the bounded adapter already does
  for a positioned read, and only "the host would wait" leaves the record
  for the engine. 69 to 205 thousand round trips a second; the parks fall
  from 256 thousand to 107 thousand, one per receive that had to wait.
  The Windows leaf answers that every transfer would wait.

The protocol on this host after both: 1.08 of the io_uring reference at
one connection (was 0.54), 0.68 at 64 (0.11), 0.48 at 1024 (0.08), 0.78
on the 64 KiB payload (0.31). The record, RESULTS.md and the plan carry
the new table beside the first reading, the isolation table with every
variable tried including the ones that moved nothing, and what remains
on the ring: the receive whose peer has not sent yet, one submission,
park, reap and wake each on one ring under two locks for the pool.

Verified in a clean worktree at this revision: make format lint;
completion-test, completion-sanitize, completion-tsan,
completion-windows-cross, completion-windows-wine; cargo test --profile
gate --lib --bins and --test programs; conformance-run, snapshot-run,
repository-invariants.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## b7110ba 2026-09-06 Retire the plan document and the per-batch record

`docs/current-plan.md` and `docs/done/` move to `archive/`, filenames
unchanged. `mcts_mem/` is now the single place a decision is recorded.

The plan document delivered its plan as v0.40 and was never replaced, so
it stayed "current" by having each later version appended to it. Work has
not been planned in a document for a long time: this batch ran from
`research/investigations/binary-arithmetic/` to a branch to the
specification and the approval record, and never wrote to it. The
standing rule that created it — "keep one rolling Current Plan as the
sole source of plan-derived execution authority and sequencing" — lived
in `mcts_mem/whitefoot/development-workflow.md`, which is where its
replacement, the two facts behind it, and the move are now recorded.

The per-batch record goes with it. A finished task is not evidence, so
every live citation of one is removed rather than repointed: the
roadmap's `Facts:` lists keep their durable sources and lose the batch
links, the derivation ledger cites the differential-fuzz experiment
directory that actually holds the campaign, `patterns.md` states its
finding without the batch numbers, and the completion adapter's comment
gives its reason without them. One citation was also wrong — the
roadmap said batch 0084 enumerated open API and target items, and it
enumerates measurement caveats; the open items were already stated in
the same paragraph.

`governance/APPROVALS.md` keeps its seven `docs/done/` mentions. It is
append-only and those bytes are in `main`.

CLAUDE.md and AGENTS.md now say where a claim's support comes from —
the specification, a conformance case, a measured result, a design, or a
decision in `mcts_mem/` — so the next citation of a finished task is
refused by the rule rather than by memory.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## d1e69b1 2026-09-06 Add the pull-request checklist

Every failure this branch produced was a step nobody was asked about.
The specification's status header grew five version paragraphs because
nothing asked whether the file carries version commentary. The roadmap's
`Current` field went stale for a month because nothing asked whether the
affected item was brought to the present tense. `mcts_mem` has no entry
for any of the five amendments because nothing asked. And a real defect
sat behind a failing stage for fifteen commits because "the gate is
green except a known environment failure" was accepted as a report,
when `check` stops at the first failure and the stages after it had
never run.

None of that wants a script. It wants the question asked where the work
is handed over, so a skipped step has to be written down as skipped.
Hence the rule that a line is answered "n/a" with its reason rather than
deleted, and the stage table rather than one green line.

The derivation section is first because it is the one that decides
whether the rest is worth reviewing, and it says plainly that an
underived change is allowed while a silent one is not — a checklist that
cannot be answered honestly gets answered dishonestly.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## d013621 2026-09-06 Scope the derivation question to language changes

The constitution governs what the language is. It says nothing about how
the project is run, and it should not: P0, P1, the standing theorems and
R1-R6 are all about syntax, semantics, and what a program may be
admitted for. `[META-6]` already scopes the derivation ledger the same
way, to numbered rules and nothing else.

As written the checklist asked every change for a constitutional
premise, including changes to CI, a gate, the repository layout, or the
compiler's internals, which have none. A required field that is
truthfully "n/a" most of the time teaches the next author to write
"n/a", and then it is not a question any more. It now says to skip the
section outright unless the change touches `spec/`, the grammar, or what
the checker accepts, and names where the other class of decision is
settled instead: the project goal and priority order in `CLAUDE.md`.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## d2c1e2e 2026-09-06 Answer the checklist once, at review request, against a named commit

A pull request worked as a draft moves under its own description. The
checklist as written invited exactly that: filled in at open, true for
one commit, then read as current for every commit after it. That is the
defect this branch has been repairing everywhere else, reintroduced in
the document meant to prevent it.

The list is now answered once, when review is requested, and names the
head commit it was answered against. Naming the revision is what makes
staleness visible rather than merely warned about: a reviewer compares
that commit to the head and knows immediately whether the answers
describe the pull request or its history. It is also the discipline the
repository already uses everywhere it matters — a digest pins the
specification's bytes, and rules 2 and 3 pin approval and `make check` to
an exact revision.

Filling it in during draft is called out as worse than leaving it empty,
because empty reads as unanswered and stale reads as answered. The gate
table is pinned to the same commit.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## d016afb 2026-09-06 Set the ring up with COOP_TASKRUN and print its counters beside the core's

The second single-variable series on the TCP echo control test, at 64
connections on the development host (docs/done/0108-streams-and-tcp.md
section 6): the idle spin holds no completion (E8), an io_uring_enter at
every submission halves the rate (E9), the enter outside the submission
lock changes nothing (E10), a kick once eight entries are staged is worse
(E15), IORING_RECVSEND_POLL_FIRST measures nothing (E16), and a progress
pass before every ready stack when the queue holds a completion changes
nothing (E17). IORING_SETUP_COOP_TASKRUN moves 200 to 225 thousand round
trips a second to 241 to 251 thousand with the same parks and enters: a
completion the kernel finishes on the submitting thread's behalf is posted
at that thread's next syscall instead of by an interrupt. A kernel that
refuses the flag gets the ring without it.

Where the remaining margin is, measured under a compile-time counter that
is not shipped: the enter that hands staged receives to the kernel costs a
microsecond an entry and 28 microseconds a call, 22 percent of the wall
time, serialized on the pool's one ring; the reap is four percent; two
workers reach 85 percent of four and eight reach less. The reference arms
one multishot receive per connection on a ring per thread and submits no
receive at all. A ring per thread and a receive that completes more than
once change what a record is for the emitter, the core and the bridge;
the record carries that as the reason and the data for the next version.

The grant observer prints the ring's counters after the core's under
WF_SCHED_REPORT (wf__bridge_report: submissions, enters, completions,
kernel waits and wakes, host wake writes, overflow flushes, runtime parks,
inline completions), so the next series reads them without a hand-linked
build.

The protocol on this host: 1.26 of the io_uring reference at one
connection, 0.74 at 64, 0.58 at 1024, 0.86 on the 64 KiB payload. The
record, RESULTS.md and the plan carry the table and both series.

Verified in a clean worktree at this revision: make format lint;
completion-test, completion-sanitize, completion-tsan,
completion-windows-cross, completion-windows-wine; cargo test --profile
gate --lib --bins and --test programs (the three root-permission cases
excepted); conformance-run, snapshot-run, repository-invariants.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## 59cd5f7 2026-09-06 Split the checklist by what completeness is owed

An earlier draft asked each decision to name the future constructs that
would invalidate it. That set is unbounded and unenumerable, so nobody
can tell a thorough answer from a lazy one, which makes the question
worthless however honestly it is asked.

Checking the change against the rules that already exist inverts the
quantifier onto a finite set. `spec/derivation/derivation-ledger.md`
holds 143 status-bearing rows; "did you read every row" has an answer.

Completeness is demanded for language changes and nowhere else, because
the specification is the object whose internal consistency is the
product. Two rules that contradict each other fail no test — they make
some program's acceptance arbitrary while the suite stays green. A
compiler defect fails a test. The sweep is required exactly where
testing cannot substitute for it, and skipped where it can.

The sweep is worth what the ledger is worth: 86 rows are derived and 57
are existence-only, and a row that states no premise cannot be
contradicted. Those are listed by ID rather than counted as checked, so
the part of the sweep that learned nothing stays visible instead of
reading as coverage.

The owner clause is stated too: choosing among options that all derive
is a decision to make, and choosing one that contradicts is not.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## d27addc 2026-09-06 Give the Windows bridge its own report stub

d016afb defined wf__bridge_report twice in the ring-less section and not
at all in the Windows one, so the macOS gate refused the redefinition and
the Windows io-hosts observed link had no symbol. One stub per platform
section now; the Linux section keeps the ring's line.

Verified here: format lint, the Windows cross build and the Wine run, and
the preprocessed unit under each platform section holds one definition.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## 9bec22b 2026-09-06 Retire the approval ledger and its activation chain

`governance/APPROVALS.md` moves to `archive/`. Its purpose was an
asynchronous loop — research a set of plans, approve them one at a time,
then execute the approved ones — and that loop is gone: work runs
research to implementation to specification, and nothing had written a
plan to it in a long time. The merge-time prose it carried is read by
nothing.

The `ACTIVE-SPEC:` chain goes with it, and that is the part that was
costing something. It verified a total order over every specification
version — each record's `superseded` field had to equal the previous
record's digest — which no decision anywhere consumed;
`ACTIVATION_CHAIN_LENGTH` was read by one test comparing it against
itself. What the order did do is make two concurrent specification
branches structurally impossible: the second to merge had to renumber,
recompute its digest against the new base, re-parent its record, and
re-archive bytes it never held. That is not a merge conflict to resolve,
it is the governance layer redone, and it was forced every time.

Nothing that protects anything is removed. A version's identity is its
own bytes; the generated identity module names them and `whitefoot-spec`
rejects a disagreement; `make spec-append-only` forbids a released
archive from changing at all, which is stricter than hashing it against a
record. `approval-history-integrity` and `spec-archive-integrity` read
only the retired ledger and go with it, leaving eight gate stages.

Rule 4 no longer demands a record in a file no one reads: a merge that
changes the specification or conformance evidence states what changed and
its selection ground in the pull request, answered against the exact
revision being merged, where the checklist already asks for it. [META-5]
says the same: the declaration lives in the change that makes it.

The conformance runner hashed the specification to compare it against the
chain tail; with the bytes as their own identity there is nothing to
agree with, so that check and the four tests that exercised it are gone
rather than rewritten to compare a value with itself.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## ab454f2 2026-09-06 Derive the specification identity at build time

`spec_identity.rs` was a generated file committed beside the source, with
a test proving the committed copy matched a fresh generation. That is a
second copy of a fact the crate already embeds — the specification's
bytes are right there in `include_str!` — and the test existed only to
police the copy. An amendment that forgot to regenerate turned the gate
red rather than the build, which is the wrong end: this branch amended
the specification three times and paid the regeneration step each time.

`build.rs` derives it instead, from the same bytes, on any build that
touches them. Verified end to end: appending one byte to the
specification changes the reported digest with no other action, and
removing it changes it back. Nothing can go stale because there is no
second copy to be stale.

It emits only the two values that must exist in constant position.
`RULE_COUNT` is gone — its one consumer was a test comparing the live
scan against the constant, so the test now compares the scan against
itself and says so. `--emit-identity`, `--check-identity`,
`identity_module`, and `committed_identity_is_fresh` go with it.

The SHA-256 is reused rather than reimplemented: `build.rs` includes
`src/spec/sha256.rs` directly. That file's `//!` header had to move to
the `mod sha256;` declaration, because an inner attribute cannot appear
in an `include!` expansion; the declaration says why, so the next reader
does not helpfully move it back and break the build.

`docs/WORKFLOW.md` also states how a version number is settled now that
the activation chain no longer forces a total order. Two branches
amending the specification archive the same outgoing bytes under the same
name, so only the new number collides, and the second to merge retitles
two lines and rebuilds.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## f22c244 2026-09-06 Separate the four rules from the practice around them

`docs/WORKFLOW.md` opened with the four branch-and-main rules copied
verbatim from `CLAUDE.md`, and five of its nine section headings said
"(not workflow)" in the title. The document was mostly not workflow, and
the part that was duplicated the file every agent already reads.

The rules keep one home. What they mean exactly — work branch, exact
revision, all repository tests, conformance evidence — moves to
`CLAUDE.md` beside them, because a definition of a rule is part of the
rule. The specification-amendment mechanics move with it, now including
how a version number is settled at merge. "Conformance integrity" and
"Specification identity" are dropped: `CLAUDE.md` already carries the
first and the second restated what the compiler and its README now say.

What remains is 110 lines of how to do the work, and it is not folded in.
One section of it, "The failures that look like success", is the most
useful writing in the repository — the mask corollary alone describes
what happened on this branch, where an environment failure in an early
gate stage hid a real defect for fifteen commits and got read as
confirmation rather than as a probe. Compressing that to fit an
instruction file every agent loads would damage it, and `CLAUDE.md` is
already 244 lines.

So it becomes `docs/practice.md`, named for what it is, opening with a
sentence saying it contains no workflow at all. References follow it:
`README.md`, three in `docs/roadmap.md`, `[META-5]`'s pointer in the
specification, two derivation-ledger rows, and the conformance
manifest's META-5 reason.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## c6f654e 2026-09-06 Take the outline out of the working loop

The outline claimed to record what the project can already do, and that
claim is what rotted: `outline:PROOF-1`'s `Current` line described only
the exact-L0 boundary for a month while five amendments landed above it,
because the truth lives in the specification and `compiler/README.md`
and the outline was keeping a copy.

It is now reference. Long-range directions and candidate projects, read
for orientation, with nothing waiting on it and no step updating it. The
checklist line that required an item be brought to the present tense is
removed, since that line was what tied it to the loop.

The honest consequence is stated in the file: a `Current` line that
disagrees with the specification, the compiler README, or the tests is
this file being stale, which it is allowed to be. A document nobody
depends on may lag; the failure before was that it lagged while
presenting itself as a status report.

Also repairs a sentence an earlier edit in this batch left broken, where
removing the activation chain spliced two clauses together.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## d422c9b 2026-09-06 Let the reset test's peer close only after the echo has arrived

a_peer_that_resets_reaches_the_program_as_its_own_outcome_on_both_routes
closed the peer as soon as its payload was written. On the macOS runner
the close raced ahead of the program's echo, so the peer's receive queue
held nothing at the close and the host sent a graceful end instead of a
reset; the program read the direction's end and exited zero, which the
test refused. A receive the submitting thread answers at once (e134360)
made that race real. The peer now peeks one echoed byte before it closes,
so the queue holds data at the close on every host and the close is a
reset by the host's own rule. Verified here on both routes.

The batch record's section 6 gains the third isolation series and its
result: the ring per thread, the kernel submission thread and the armed
multishot receive were each built and measured and left the rate where it
was, the profile puts the remaining cost in the scheduler's shape rather
than the ring, and none of the three is kept; RESULTS.md and the plan say
the same in one sentence each.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## 6d86245 2026-09-06 Record the compute line against main at the end of the batch

par_layout compiled by each tree's own whitefootc --par and run
alternately on the development host: this branch is within two percent
of main at one and two workers and eight to nine percent behind at four
and eight, the narrowed form of the compute-miss item the batch left
open, with the same output bytes.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## 76adab5 2026-09-06 Let CI's static job read the stage list instead of restating it

Retiring `approval-history-integrity` and `spec-archive-integrity` left
`.github/workflows/gate.yml` naming both, so the `static` job failed on
every host while the local gate was green. The workflow had its own copy
of the repository-level stage list, and I updated the Makefile's and not
that one. My own gate script was a third copy, which is why running it
reproduced the oversight instead of catching it.

The list now lives once. A root `static` target names the stages that
read the tree without running a compiled program, and the workflow runs
`make static && make -C compiler static`. A stage retired or added from
here on reaches CI without anyone remembering to.

Also drops the retired stage from the checkout comment beside it, which
explained why `main` is fetched as a local ref.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## 043a721 2026-09-06 Merge pull request #22 from mbbill/batch/0129-binary-arithmetic

Binary arithmetic in the proof surface (v0.45-v0.49), and the workflow that carried it
## b21c0a3 2026-09-06 Merge origin/main (043a721) into io/t4-resource-relations

The specification amendment of this branch lands as v0.50 over main's
v0.49: main's v0.49 bytes are archived as spec/kernel-spec-v0.49.md, the
header carries no version commentary, and the delta declaration and
selection ground move to the pull request as [META-5] now requires. The
branch's own v0.45 and v0.46 archives give way to main's, whose v0.45
through v0.49 amendments are front-end and touch no rule this one
touches; the derivation ledger carries the amendment as two halves of
v0.50 with 140 rules, 84 derived and 56 existence-only.

The identity module is gone with main's build.rs deriving it from the
bytes; the qualification review carries both histories under one v0.50
stamp; the conformance manifest is the union; the three batch records,
the approval ledger and the plan follow main into archive/ and the files
this branch owns point at the new homes. The three isolation series of
the TCP echo control test move from the archived record into
research/investigations/io-model/RESULTS.md, where a measured result
lives, and the decisions of this landing are written into
mcts_mem/whitefoot/system-interface.md and parallelism.md.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## 2620c41 2026-09-06 Record what v0.45 through v0.49 settled

The five amendments landed without an entry in `mcts_mem/`, which is the
gap the merged batch flagged against itself: `checks-and-proofs.md`
stopped at 2026-08-05 and `surface-form.md` at 2026-08-09 while the whole
proof surface moved.

`certificate-fold` is its own node rather than a bullet, because the
decision has three alternatives that were actually built and a measured
table separating them. Each rejected one keeps its own node under
`certificate-fold.alt/` with the measurement that killed it:

  fold-by-operand-image — v0.48's rule. A local's image is transparent,
  so a product over `stride` and a certificate scaling by `stride`
  arrived at the fold as different arithmetic. One of four rows passed.

  handle-equality-as-fact — the equality was published, true, and
  unreachable: the residual takes the direct L0 route, which does not
  consult a published affine fact. A fact the consumer cannot read is
  indistinguishable from no fact, and the symptom reads like an
  arithmetic bug rather than a routing one.

  opaque-binding-image — closes the fold and relocates the cost: an
  ordinary premise about the binding then needs the equality, and two of
  four rows moved from a fold failure to a premise failure.

Two of the facts added to `checks-and-proofs.md` are method rather than
semantics, because both cost this batch real work. Four statements it
wrote about itself were false and all four were written from reading the
code instead of running it — one reached the specification as a
normative sentence describing a case a writer cannot construct. And the
corpus cannot measure a gap in the language it is written in: it
reported 1.7% of files against 9-of-11 constructed refusals, with one
corpus file recording its own dodge in its source.

The `surface-form.md` pitfall is stated as the property rather than the
incident: a keyword is a position in a table, two facts that must agree
were written twice, and the fix derives the second from the first.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## 39ad816 2026-09-06 Correct what an existence-only ledger row is, and re-ground the workflow rules

The merged pull-request template told every future change that an
existence-only row "states no premise, so nothing can conflict with it and the
sweep learns nothing there", and to list such rows by ID instead of reading
them. Measurement says otherwise: of the 57 existence-only rows none has a
Notes field under 60 characters, the median is 337, and 31 name an explicit
condition under which the row would be promoted. The status separates a
derived need from a minimality-selected form. It does not mark an empty row.

So the sweep asks two questions at such a row, and the template now says both:
does this change contradict the row, and does this change's evidence meet the
promotion condition the row already names. v0.45 through v0.49 is the case
that shows the cost of asking only the first. [PRF-1] holds its form "until
wider source compares proof length, diagnostic quality, and missing
expressivity"; that batch produced wider source, measured proof length, and a
documented missing-expressivity case in the derived stride, and recorded none
of it against the row.

The workflow node had the same shape of defect one level down. The 2026-09-06
ruling landed there as a Fact while the standing rules above it still routed
what landed to `governance/APPROVALS.md` and language change to a guarded
branch in `docs/WORKFLOW.md`, both retired by that ruling. The rules now state
the loop that runs, and the drift is recorded as its own pitfall: appending a
Fact is easy and rewriting a rule is not.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## 2e84ff4 2026-09-06 Merge pull request #13 from mbbill/io/t4-resource-relations

The I/O model under T4 (v0.50): the backed file permit, park-on-miss on the scheduler core, streams and TCP
## 109af00 2026-09-06 B7c4b: buffers off the buffer surface, in the part that has a twin

Nine cases move to the run: the invariant-erasure walk over a view, the
OP-4 refusal of an out-of-range element commit, the empty take, both OP-9
allocation-fit refusals, the nested-struct release order, the projected
move's residual siblings, the explicit release on return and break edges,
and the trivially droppable element that keeps one release. Each keeps its
assertion, and the two counts that changed are re-derived at the
assertion: a store take is refusable where buffer_new aborted, so main
also carries what each refusal arm holds. The reverse-order property moved
into a releaser that holds the owner and nothing else, so it is still four
releases on one edge.

Five are left on buffer<T> deliberately and each says why at the test: two
target-boundary cases the run cannot carry because a store take reaches no
arm of backend::target's allocation validation, the OOM-abort edge a
refusable take replaces with a source-level arm, a target-guard absence
that would pass vacuously over a run, and the &uniq struct pointer [BLK-4]
refuses over a run.

Two more still hold embedded buffer sources and are not yet migrated:
projected_buffer_target_is_formed_once_before_rhs and
affine_element_buffers_construct_replace_vacate_and_drop_per_element.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 1a00279 2026-09-06 Make the denied-path cases check the denial they depend on

Three corpus cases set a fixture path to mode 000 and then assert that the
walking program reports it as unreadable. Mode bits do not deny a process
holding CAP_DAC_OVERRIDE, which uid 0 carries by default, so under a container
that runs the gate as root the scenario is unconstructible: the walk reads the
path it was told was closed and the case fails on its output comparison. That
failure reads like a traversal defect and is not one, which is how three
permanently red cases become background noise.

`close_path` now establishes the mode and confirms the denial actually reaches
this process, so the failure names its cause and its remedy instead. The
opposite risk is covered by the same check: without it a case could report
success in an environment where the path it describes was never closed.

This is not a skip and does not make the gate green here. Verified both ways
on this commit: as uid 0 all three fail at the precondition with the new
message, and the same binary run under setpriv --reuid=65534 passes all three,
which is also what CI reports.

The three inline permission blocks collapse into `close_path`/`reopen_path` in
`support.rs`, so the precondition cannot be established in one case and skipped
in another.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## 426be7a 2026-09-06 Merge remote-tracking branch 'origin/main' into batch/0130-decision-memory


## f8965cb 2026-09-06 Give the two Moves that name a missing node their node

Every `[[link]]` in `mcts_mem/` now resolves. Two did not: `operation-spelling`
records replacing `multiplied-use-star` and `development-workflow` records
replacing `rolling-current-plan`, and neither replaced design existed as a
node, so the reason each was rejected lived only inside the sentence that
retired it.

That is the failure the `.alt/` convention exists to prevent. A Move says a
design was replaced; the node it names is where the design and the measurement
that killed it are readable. A Move pointing at nothing records that something
was rejected without recording what, which is the shape of a decision record
that cannot be audited.

Both nodes are written from evidence that already existed rather than recalled:
`multiplied-use-star` from the `*` spelling's category error, its strong-LL(2)
ambiguity, and the whitespace rule proposed to rescue it; `rolling-current-plan`
from the plan document's own accretion history and the 2026-09-06 ruling that
retired it.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

## b0778e2 2026-09-06 B7c4b: the whitefootc driver fixtures off the buffer surface

Both embedded sources publish a view of a run where they published a borrowed
buffer: the stackless-and-compute fixture keeps its writer-scheduler selection
and the denied I/O loop keeps its [PAR-3] denial under all three lowerings.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 9008f94 2026-09-06 Merge branch 'batch/0127-b-backend' into batch/0127-containers


## 8a5ad14 2026-09-06 Merge pull request #23 from mbbill/batch/0130-decision-memory

Record v0.45 through v0.49, and correct four artifacts that read as current and are not
## d72ee4b 2026-09-06 Merge main (2e84ff44) into batch/0127-containers

Main's PR #13 landed the io_uring/Windows completion runtime, the system
renames, the standard-input stream, and the TCP surface; this branch is the
containers redesign. Both touch the same rows, so the merge decided each
overlap once and applied that decision everywhere.

Spellings. Main's names win everywhere: HandleFactory, HandlePermit,
OutputStream, InputStream, command.handles, reserve_handle, the four open
outcome enums, and the five explicit closes. The branch's view operand classes
win at every range-bearing system row, including the three rows main added:
read_next and receive_next take &uniq MutSlice<u8> and send_once takes
&Slice<u8>, with the same transitional buffer<u8> arm and the same viewed-
storage call classification read_at and write_once already had. The catalog
rows say so; nothing selects a class by operation name.

Entry. The seven-input command table is main's six rows plus this branch's
command.heap as heap: own Heap at ordinal 6. The entry-form table, its tests,
the spec's ordinal prose, both bootstrap emitters, and the conformance case
that enumerates every input all carry the seventh row.

Specification. v0.51 is the containers amendments over main's v0.50; main's
outgoing bytes are archived as spec/kernel-spec-v0.50.md and the derivation
ledger records 101/60/161. Two prose statements that the two batches left
inconsistent are corrected here: [FN-7] now calls command.heap ordinal 6, and
[SYS-18] writes send_once's source as &Slice<u8>, which is what its own row
already said.

Backend. Main's completion runtime wins wholesale, including the retirement of
the stackless plan: emitter/stackless.rs and its tests are gone and this branch
drops its edits to them. qualification.rs keeps main's reviews, then this
branch's containers review, then a v0.51 note stating why the two batches are
disjoint at the target table; REVIEWED_FOR is v0.51.

Tests and corpus. Main's rewrites win for structure and spelling; this branch's
verdict-preserving migrations off buffer<T> and onto the view forms are
re-applied on main's text wherever the property survives, with every assertion
re-derived rather than relaxed. The retired allocates(heap) atom is dropped
from every source main left carrying it, because the ambient heap contributes
nothing to a written row on this branch, and the measure readers are spelled
len_of. Every conformance verdict is the one the manifest records, and every
program keeps the exit codes and outputs its own test pins. cost_shape's
release-site count is re-derived on the merged wfgrep and is eighteen, which is
neither side's number: main's completion close became a submit-and-join pair
the optimizer can no longer fold, and the store surface gave main more ways to
leave. Four manifest doc strings are corrected to describe the merged sources.

Follow-up work named by the merge review lands as its own commits after this
one: the frame-slot predicate's missing SliceFromRun arm, the [OWN-5] hole at a
&uniq borrow of a frozen parent view, and the migration of main's new stream
and TCP programs off buffer_new.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 8373e7b 2026-09-06 Read a run's storage from its shape, and pin a view of one across a suspend

The frame-slot planner decided whether a run keeps its slots inline by naming
`IrType::FixedVector` at the site, while the emission that consumes the slot
decides the same question through `RunShape::of`. Two readings of one fact go
out of step at the third run storage, and the reviewer's finding on this branch
was exactly that class: a stack-bound referent enumerated by name, with a view
of an inline run outside the enumeration. The planner now asks the shape table,
which is the one authority on what a run's slots are behind.

Behaviour is unchanged today, and deliberately so: `RunShape::Inline` is
exactly `FixedVector` and `Descriptor` is every `Vector`, whatever its release,
so the two readings agree on the run types that exist. What changes is that
they cannot stop agreeing.

The reviewer's own repair was written against `emitter/stackless.rs`, which
main deleted with the stackless plan; its defect — a frame pointer surviving
into a heap-allocated continuation — has no expression left, because the
completion record is now the frame's own block. What survives is the property
worth pinning, so a backend test takes a view of a frame-resident run across a
`write_once`, asserts the view indexes the run's own frame slot rather than a
descriptor's pointer word, and runs the program for its bytes.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 47bec3d 2026-09-06 The child's freeze on its parent view reaches the parent's holder

[S31, OWN-5] a shared child reborrow of an exclusive view freezes the
range while the child lives. The check that enforced this asked only
about an element *write* through the parent: `check_loan_access` sees
claims on the origin storage, and `&uniq writer` names the view
descriptor rather than that storage, so no claim on the origin saw it.

A `&uniq` of an exclusive view is the borrow through which the frozen
element write is made, and a `move` of one hands that write to a callee
outright. Both are the access the element write already is, taken one
indirection earlier, and both were accepted with the child live:

    let writer = mut_slice_of(&uniq bytes);
    let view = slice_of(&bytes);
    let done = consume(view: view, output: &uniq writer);   // accepted

That program compiled and ran. It now stops at the `&uniq` operand with
[OWN-5] BorrowConflict, which is the same rule and the same diagnostic
the element write already gets.

The refusal is one loan path, not a second rule: a unique borrow or move
whose place root holds an exclusive slice loan asks the existing
`check_child_reborrow_freeze` about that loan's origin place, so the
freeze is a property of the loan rather than of the one statement form
that happened to check it.

Liveness needed one statement of its own. The child and the borrow are
two operands of one call, and document order puts the child's use
*before* the borrow, which would read the child as already dead. This
checker states no program point between two operands of one statement —
the very reason a loan no binding holds keeps its whole region extent —
so a use anywhere in the access's own `let`, `set`, expression, or
`return` statement is simultaneous with the access. `Simultaneity` names
which of the two readings a caller wants; every pre-existing caller
keeps `Sequential`, the reading its diagnostics are pinned to.

Corpus: 727 pass / 3 skip (was 725/3, the two cases below), snapshot 484
pass / 0 flip, compiler suite green. No other case changes verdict.

Cases added:
  own5-neg-a-unique-borrow-of-a-parent-view-while-its-child-lives
    reject OWN-5 — the borrow above.
  own5-pos-a-unique-borrow-of-a-parent-view-after-its-child
    run exit 0 — the same borrow after the child's last use.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 443759a 2026-09-06 stdin_echo off the buffer surface and onto a run with two views

Main's standard-input program read into `buffer_new(4096, 0)` through
`&uniq buffer<u8>` and published from the same buffer. On this branch a
destination is a view: `read_next` takes `&uniq MutSlice<u8>` and
`write_once` takes `&Slice<u8>`.

The chunk becomes a store-resident `Vector<u8>` from `heap_vector` at
`command.heap`, filled once by a counted `place_back` loop, and the two
system calls reach it through views of their own strength. The views
live in *sibling* regions, not nested ones: an exclusive view for the
read, ended at its brace, then a shared view for the publish. A shared
child of a live `MutSlice` would freeze the parent, and a second
exclusive view of a live one is refused outright, so one run viewed one
way at a time is the only shape [VIEW-2, OWN-5].

The read's `end` is the run's own length, read once as
`let held = len_of(chunk)` before the loop, rather than the literal
4096. Both views carry their origin's length, so `available <= held`
from the `ReadBytes` edge and `len_of(payload) == held` discharge
`publish_all`'s `requires length <= len_of(deref(source))` at the one
live point, and the literal no longer has to be kept in step with the
allocation. `main` gains `command.heap as heap: own Heap` at ordinal 6
for the store.

Behaviour is unchanged. A 10,000-byte payload comes back byte for byte
through a pipe and through a redirected regular file, on the native ring
and through the file adapter, and an empty standard input still reaches
its end publishing nothing — exit 0 in every case, before and after.

`the_stream_read_lowers_through_the_one_submit_and_join_shape` pinned
the entry's call as `@wf_main(i32 1, i32 0)`. The seventh row's store
operand now follows the two descriptors, so the assertion pins the
prefix; what it checks — descriptor 1 for `command.stdout`, descriptor 0
for `command.stdin`, and no open for either — is what it checked before.

docs/patterns.md P34 cites this program as its evidence, so its code
block and its length-bound bullet carry the same form, and the
sibling-region rule joins them as the pattern's first bullet.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 177a27c 2026-09-06 The TCP programs off the buffer surface, as far as the judgments reach

Main's four TCP programs read and wrote `buffer_new` runs through
`&buffer<u8>` and `&uniq buffer<u8>`. On this branch a range-bearing row
takes a view: a destination is `&uniq MutSlice<u8>` and a source is
`&Slice<u8>`.

Three of the four migrate whole. `digits`, `chunk`, and `payload` become
runs, and each system call reaches its run through a view of its own
strength, formed in a region that ends before the other view forms — one
run is viewed one way at a time [VIEW-2, OWN-5].

  tcp_client   `digits` (16) and `chunk` (4096) are store-resident
               vectors at `command.heap`; `payload` (8) is an inline
               `FixedVector<u8, 8>`, which only ever needs the shared
               view `send_once` takes. `main` gains `command.heap`.
  tcp_echo     `digits` and `chunk` the same, at `command.heap`.
  tcp_refused  `digits` the same, at `command.heap`.

`byte_at`, `parse_port`, `send_all`, and `publish_all` take `&Slice<u8>`
in all three, and each read is bounded by the run's own measured length
rather than by the literal the allocation was written with: `let room =
len_of(digits)` and `let held = len_of(chunk)` before the loop, and the
system call's `end` is that. A view carries its origin's length, so the
copy bound and each helper's `requires length <= len_of(deref(source))`
discharge from the same term, and no literal has to be kept in step with
an allocation.

tcp_fanout migrates as far as its own judgment allows, and no further.
Its argument scratch becomes an arena extent — `arena_frame` in main's
own region, viewed both ways — and `byte_at` and `parse_port` take
`&Slice<u8>`. The per-connection scratch stays `buffer_new(256, 0)` and
`serve_one` keeps `&uniq buffer<u8>`, because that program exists to
hold the staged permission its fixed-trip accept loop is granted
[PAR-3], and the staged judgment refuses every replacement:

  - a store-resident run reaches its store through `&uniq heap`, a
    borrow of storage the iteration does not introduce, which the
    judgment refuses outright;
  - an arena extent per iteration is an `Access::Arena` the judgment
    resolves to no disposition, which denies on condition 7; and
  - a `&uniq MutSlice<u8>` destination is a view descriptor, which the
    judgment resolves to no place at all — also condition 7 — so the
    denial does not depend on where the run lives.

`buffer_new`'s `BufferFill` is the one iteration-own construction that
judgment classifies. Making the containers forms visible to it is real
compiler work with its own evidence, not a program edit, so the program
says in its own doc why its scratch is spelled the way it is. Its
ledger is unchanged: staging permitted at `serve_one`, four places
classified, `&uniq handles` serialized-P, `&bound` read-only, `set
outcome = reported;` serialized-E, and the scratch replicated.

Every program keeps the exit codes and bytes its own test pins, checked
against loopback peers before and after on both runtime routes: the
client sends `ABCDEFGH` and publishes `abcdefgh` at exit 0, the echo
returns every byte at exit 0, the refused connect exits 0 on the second
attempt with the first's own permit, and the fanout answers four peers
at exit 0. `programs::network` and `programs::stream` are green, the
fanout's permission-verdict and lane-hand-out cases included, with no
assertion changed.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 04e60bd 2026-09-06 Main's seven new conformance cases off the buffer surface

This branch migrated 73 of the corpus's 74 `buffer_new` cases onto the
view forms before the merge; main's PR added seven more, and the merge
left them on the transitional surface. They move now, by the same
transformation and with no verdict, rule list, arrangement, or manifest
doc touched:

  run-sysfile-close-returns-permit          run exit 0
  run-sysfile-failed-open-returns-permit    run exit 0
  run-sysin-read-to-end                     run exit 0
  sysin-read-outcome-exhaustive             reject ERR-2
  sysin-read-zero-range                     run exit 0
  systcp-connection-field-effect-paths      accept
  systcp-connection-two-halves              accept

Each `buffer_new(n, 0)` becomes a store-resident `Vector<u8>` from
`heap_vector` at `command.heap`, filled by the counted `place_back` loop
the corpus already uses, and each `&uniq page`/`&uniq chunk`/`&uniq
scratch` destination becomes `&uniq window` over `mut_slice_of` of that
run. Every `main` gains `command.heap as heap: own Heap` at ordinal 6
and the `70_u8` store-refusal arm the corpus's other store cases carry.

Two cases needed more than the mechanical move:

`sysin-read-zero-range` reads `chunk[0]` back after its second
`read_next`. The read moves out of the exclusive view's region and after
it, so the run is viewed one way at a time and the origin read is the
ordinary one. What it asserts is unchanged: outcome stays 0 only when
the empty range answered `ReadBytes(2)` without moving the position and
the following nonempty read then observed the first byte.

The two `systcp` helpers take `&uniq MutSlice<u8>` rather than `&uniq
buffer<u8>`, and `pump` sends back through `slice_of(&deref(scratch))` —
the destination view's own shared child [OWN-6] — instead of naming the
buffer place. The effect rows they exist to state, `link.receive` and
`link.send` leaf by leaf and the two disjoint field loans, are the same
rows over the same paths.

Corpus: 727 pass / 3 skip, unchanged from before this commit. The one
`buffer_new` case left in the corpus is fn8-pos-requires-affine-row,
which this branch retained deliberately before the merge.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 825efae 2026-09-06 The one writer-facing name the merge's rename sweep missed

Main renamed the handle surface — HandleFactory, HandlePermit,
command.handles, reserve_handle — and the merge carried those names
everywhere the compiler and the corpus call them. One doc string was
left behind: accept-par3-staged-loop-with-prologue-break describes "the
short unique factory loan that reserve_file takes" while its own body
calls reserve_handle. A case's doc is the writer-facing statement of
what the case shows, so it names the operation the case calls.

Nothing else in the sweep's scope is stale. The `Inventory::FilePermits`
variant in resolution/catalog.rs is main's own name for a catalog
generation and not a writer spelling; the qualification.rs comment
describing the rename is history and reads correctly as history; and the
containers investigation's DESIGN.md carries `reserve_file`,
`FileFactory`, `FilePermit`, and `own Output` inside its dated amendment
register and its design drafts, where they record what was proposed
under the name it was proposed under.

The case's verdict, rules, and manifest row are unchanged.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## a8838c2 2026-09-06 B7c4b: A view argument is a footprint on the storage it was formed over

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 2912e6d 2026-09-06 B7c4b: A run taken from a store inside the iteration is iteration-own storage

The staged judgment now projects a kernel row's own footprint: its `&uniq`
provider operand holds the serialized store access and the run it hands back
is storage the iteration introduces. The handed-out completion transfer takes
a view destination beside a buffer one, both being the same contiguous
descriptor.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 0f31df6 2026-09-06 B7c4b: The store's run and cell rows are validated against the selected target

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 8d7ac6b 2026-09-06 B7c4b: The store surface's own alignment boundary, and what the target no longer stops

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 945b0a0 2026-09-06 B7c4b: Three conformance cases for the view footprint and the store's iteration-own run

Also models the give-back a general store's run owes on the iteration's own
exit, which a walk over statements alone never reaches.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 6deecf4 2026-09-06 B7c4b: The last patterns off the buffer, and what the remaining test modules are still for

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 9d61f73 2026-09-06 B7c4b: The staged hand-out keeps its buffer scratch, and the reason is recorded

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 98abed4 2026-09-06 B7c4b: The batch's verdicts, as the gate measured them

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## c6536e5 2026-09-06 B7c4b: The fanout program's doc says why its scratch stays, which is no longer the judgment

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 029ac1f 2026-09-06 B7c4b: The fanout program leaves the buffer, and the staged walk stops counting a release that performs nothing

`tests/programs/tcp_fanout.wf` was the last `tests/programs` source on the
retiring surface. Its per-connection scratch is now `serve_one`'s own bump
extent, taken as a run and written through the exclusive view the run hands
back, and the caller's fixed-trip loop keeps its staged hand-out: the ledger
reports the loop permitted, `@wf__par_publish` is in the module, and
`four_peers_are_served_at_once_under_par_on_both_routes` passes on both
routes.

The scratch could not stay in the loop and reach the callee. A borrow of a run
is the address of the run's own storage, and the staged lowering refuses an
addressed binding the prologue declares, because four in-flight iterations
cannot share one frame slot; and an owned run parameter occurring at one
position of a declaration may carry no region name, therefore no affine bound
[FORM-8], so its store region is the fail-closed general one and the callee
may not release it [PROV-6]. The `replicated` disposition the ledger printed
for the buffer scratch is therefore gone from this program, and the assertion
that pinned it is retired at the test with that reason written out; the
classification itself stays covered by
`par3-pos-a-per-iteration-run-from-the-store-is-iteration-own`.

Separately, `direct_staged_tail` walked into a region only when its
fallthrough drops were empty, which counted a record for a value that releases
nothing — a view owns no storage and a bump extent's run is reclaimed by its
own region reset — as work a split body would lose. The walk now asks whether
each record performs anything, through one reading of that question shared
with the target stage: `type_derives_release` moves into `lowering.rs` and
`type_requires_cleanup` delegates to it.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 2d88fe9 2026-09-06 B7c4b: The retirement's specification text and the first compiler cut, recorded and then reverted

This commit DOES NOT BUILD and the next commit reverts it in full. It is kept
in history because it is the measured content of the retirement's first two
steps, and the batch stopped on evidence rather than on effort: keeping the
work costs nothing and re-deriving it would cost a day.

What is here. `spec/kernel-spec.md` with `array<T, N>`, `box<T>`,
`arena<'r, T>` and `buffer<T>` removed from [GRAM-3], [TYPE-2], [TYPE-5],
[TYPE-7], [SET-1], [SET-2], [CONST-1], [CONST-2], [OWN-1], [OWN-10], [LIV-2],
[PROV-6], [BLK-4], [VIEW-2], [STOR-1] through [STOR-6], [OP-1], [OP-4],
[OP-5], [OP-9], [EFF-1], [EFF-2], [FN-2], [FN-9], [DIAG-2], [PAR-1], [SYS-2],
[SYS-8], [MSR-1], [ENT-2], [ENT-3], [ENT-5] and [ENT-6]; `allocates(arena
REGIONID)` retired with `arena<'r, T>`; the allocation-fit predicate respelled
`fits::<T>(n)`; `slice_of` and `mut_slice_of` moved into the kernel IDENT
domain; the ambient heap gone from [PROV-6] and [EFF-1]. The grammar tables
regenerated from that text by `whitefoot-grammar-tables`, which is what makes
the four words ordinary IDENTs. In the compiler, the four type productions
gone from the type parser, the seven retired rows gone from the operation
family table, and the `Array` and `Buffer` variants gone from `CheckedType`,
with 54 build errors remaining at the point the batch stopped.

Why it stops here is in the record and in the report, not in this message.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 2c117b5 2026-09-06 B7c4b: Revert the partial retirement, so the branch tip is a tree that builds and passes

The previous commit's inverse, byte for byte. The retirement cannot be
partial: the moment `buffer<T>`'s production leaves [GRAM-3], every one of the
923 remaining occurrences of the four retiring type spellings and their seven
formation rows across `compiler/src` stops parsing, and 600-odd of those are
embedded test sources whose migration is a hand judgment each. The branch tip
is therefore the fanout commit, which builds, passes, and is mergeable on its
own.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 081d172 2026-09-06 B7c4b: The record of what this batch landed and why it stopped, and four ledger rows that had gone stale

`DESIGN.md` gains 6.0y in 6.0x's style: the fanout migration, the staged
walk's release question, the two findings that stopped the batch, and the
measured size of what the retirement still owes. No "transitional" sentence of
6.0q-6.0x or of section 7's B7 is superseded, because nothing they describe
retired.

The derivation ledger's sweep found four rows describing a compiler that has
not existed since B7b and B7a6, and each is corrected where it stands: [BLK-2]
said four formation rows where the active rule has six; [BLK-0], [BLK-2] and
[BLK-3] each registered that a call to one of their rows is an explicit
unsupported capability, which nine programs falsify; [BLK-1] registered that a
run of runs is not expressible and that a run value is unsupported at
execution, which B7a6 and B7b falsify; and [MSR-1] and [MSR-5] spelled the four
measures `len(P)`, `cap(P)`, `room(P)` and `head(P)`, which [S36] superseded.
Each correction names the superseded reading rather than deleting it.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## eff095c 2026-09-06 B7c4b: The fill helper's doc names the reason it is a helper, which moved with the scratch

The scratch is now reserved inside `serve_one`, so the fill is no longer a loop
at a staged call site and the doc's [PAR-3] reason had gone stale. What makes
it a helper is its contract: `ensures len_of(result) >= 256_u64;` is what a
caller reads instead of re-deriving the opened length from three loop
invariants [FN-9].

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

## 0f537c7 2026-09-06 Record first-principles container architecture reassessment


## d998dd0 2026-09-06 Select container storage architecture from executable experiments


## ecca644 2026-09-06 Record successful container experiment gate validation


## 2e37ff8 2026-09-06 Distinguish container demand from language-constrained representations


## 4f8fb9d 2026-09-06 Ground container follow-ups in six external workload traces


## f5dab70 2026-09-06 Implement typed container places and aggregate destinations

Checkpoint the ordinary storage path, value snapshots, deterministic slot reuse, target evaluation order and executable ownership evidence. Keep the remaining parallel aggregate ABI adapter as an inert review patch; parallel integration, final measurements and the full gate remain pending.

## ebcd7f6 2026-09-06 docs: align project guidance and remove retired workflow rules


## 15400e2 2026-09-06 Measure owned storage and record remaining frame costs

Retain the same 168-sample matrix alongside its baseline. Record executed scalar/wide/view checks, 1488 passing unit tests and the incomplete root gate. Distinguish scalar element-loop work from the remaining one-time aggregate copy and conservative frame storage.

## bb8eb30 2026-09-06 Fix aggregate system operands and read-out diagnostic priority

Keep TYPE-2 precedence for repeated affine element moves without changing normative cases. Render ordinary system arguments from materialized aggregate snapshots. Preserve FIR runtime checks while observing direct fields, and correct stale heap capability documentation. Native conformance passes 732 cases with one pending; parallel aggregate integration remains incomplete.

## 7868e5f 2026-09-06 docs: repair design memory and require its maintenance skill

Repair missing provenance, misplaced evidence, rationale in Items, and inconsistent replacement pairs. Preserve existing alternative conclusions and add mirror entries with source commits. Remove one Move that recorded node construction rather than a design decision.

Normalize malformed committed entries as explicitly requested by the owner. This bounded history repair is intentional; subsequent maintenance follows the append-only rule.

## 9296344 2026-09-07 Review container foundation before further call adapters


## de0e47d 2026-09-07 Specify checked storage roles and destination normalization boundary


## f41fef7 2026-09-07 Retain checked source signature modes in typed IR


## 3024612 2026-09-07 docs: add completion review and keep active PRs updated


## 8bbe521 2026-09-07 Retain checked user-call uses and borrow origins


## 8830c07 2026-09-07 docs: simplify PR template to change and review summary


## bf8cdc5 2026-09-07 Preserve aggregate results and staged owned storage through retirement


## 7ce43dd 2026-09-07 Record the complete parallel integration gate result


## 6cc0098 2026-09-07 Merge pull request #27 from mbbill/codex/docs-ai-workflow

docs: clarify project guidance, add completion review, and repair memory
## a880a42 2026-09-07 Merge remote-tracking branch 'origin/main' into codex/container-continuation

# Conflicts:
#	compiler/README.md
#	docs/patterns.md
#	governance/hooks/pre-merge-commit
#	mcts_mem/whitefoot/data-model.md
#	spec/derivation/derivation-ledger.md

## d5722f1 2026-09-07 Release owned places without whole-aggregate cleanup snapshots


## 37b0be9 2026-09-07 Align nested residual cleanup expectation with PROV-6


## 13fef48 2026-09-07 Share the typed internal ABI across function and parallel call emission


## d5c0bb8 2026-09-07 Make stack probe control deterministic and refresh benchmark view parameters


## aad08d2 2026-09-07 Measure container storage after shared ABI and place cleanup


## c4964ce 2026-09-07 Construct fresh aggregate bindings in their planned destinations


## ea979ba 2026-09-07 Record direct construction costs and conservative placement boundaries


## 8eb9f56 2026-09-07 docs: ground Whitefoot in the AI agent harness premise


## a96be5e 2026-09-07 Research non-escaping control-header loan boundaries


## d1aec0d 2026-09-07 Enforce statement loans and retire completed owned control headers


## 9e9edb5 2026-09-07 Reconcile container capability evidence with current implementation


## f2a2986 2026-09-07 Merge pull request #25 from mbbill/codex/container-continuation

Implement store-branded containers and typed owned storage
## 1fec60a 2026-09-07 research: propose an evidence-guided decision workflow


## c509608 2026-09-07 Make constitutional grounds and decision updates explicit


## 7f096b9 2026-09-07 Integrate container main and carry decision index to v0.52


## 217b2d8 2026-09-08 Rewrite constitutional commitments and retire inherited clause labels


## 7060831 2026-09-08 Clarify human direction and delegated agent implementation


## 46a608a 2026-09-09 Reduce decision workflow to four explicit occasions


## 8da2c5c 2026-09-09 Record constant initialization alternatives and probe criteria


## fa5e800 2026-09-09 Repair legacy memory metadata with explicit historical traceability


## 35fe3b8 2026-09-09 Reassess constant initialization grounds with omission and value controls


## cdbd16c 2026-09-09 Keep memory verification instructions owned by the skill


## 6f5a4e8 2026-09-09 Clarify source proof grounds within documentation scope


## 868354f 2026-09-09 Assess active rule grounds and retain concrete design questions


## 3016842 2026-09-09 Merge pull request #29 from mbbill/codex/agent-harness-constitution

Rewrite the constitution and make decision grounds maintainable
## 6c281c8 2026-09-10 Bring the I/O bench programs to the current language and let the gate see them

The containers batch (PR #25) moved the language under the measurement
bundle: `len` became the measure reader `len_of`, the ambient heap stopped
being an effect root so `allocates(heap)` no longer resolves, and a callee's
unique borrow of a whole buffer now kills the caller's length fact, so a
name buffer handed to `name_at(&uniq name)` could no longer discharge the
`0..10` range of the `open_file` that followed. Ten of the twelve programs
in research/experiments/io-completion-bench/programs stopped compiling.
Nothing noticed: the io-bench workflow's steps run `sh ... | tee`, so the
step's status was tee's, and the summary step printed "the network table
did not run" into a green job. The read and network tables on main have
been empty since f2a29866.

The programs follow the pattern read_heavy_wide8_4k.wf already carried:
`len_of`, no allocation clause, and the two unique-borrow helpers take a
`&uniq MutSlice<u8>` lent through `mut_slice_of` in a nested region, so the
buffer keeps its length. Sizes, offsets, checksums and the printed line are
unchanged; the file, read and network protocols publish the same bytes as
before (verify, read-verify and the network protocol's correctness pass all
pass on this tree).

Two guards so it cannot go quiet again. The bundle gains `programs-check`,
which compiles every program and nothing more, and the root gate runs it as
the `bench-programs` stage, in its own CI job so no stage grows past its
budget (about fifty seconds of compilation on four processors). The
io-bench workflow sets `shell: bash` as its default, which GitHub runs as
`bash -eo pipefail`, so a protocol that fails behind `tee` fails its step.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## 6816e9b 2026-09-10 Restore ordinary-stack compute execution independently of I/O waits

Extract the retained runtime from PR #28 at 70aa8e5 onto main 3016842.
Use native thread stacks, current-stack join/help/steal, atomic deque cells
and claims, lane-owned task frames, and the native exhaustion floor.
Keep main's compiler publication policy and vectorization unchanged.

Typed I/O retains submit-then-join. May-suspend user calls stay on their
caller's stack, so connection-level suspended-handler concurrency is
temporarily unsupported. Source-order server loops wait for the current
handler; no restoration mechanism is chosen. WF_STACKS is inert.

Retire sched-enumerate and its enumerate.c, enumerate.h, schedules.c and
switch.h machinery because they enumerate the removed managed-stack state
machine. Remove its four thread/stack configuration tests and the explicit
state-search equivalence test with that implementation. The native deque
probe exercises 200000 tasks with eight reusable slots, four participants
and a concurrent counter observer, with statistics on and off.
sched-deque-tsan detects races in these observed executions; it does not
exhaustively enumerate interleavings or prove weak-memory ordering,
liveness, completion wakeups or stack-floor behavior. Native startup/join,
completion and exhaustion checks remain separate. Do not present this as
verification coverage equivalent to the retired enumerator.

Retire the reverse-order four-peer concurrency assertion and its staged
lane-layout checks with suspended-user-call fanout. Preserve four-peer
source-order execution, native/helper I/O routes, ordinary-call IR checks,
result bytes, mutation, error and cleanup checks. Replace the completion
join's scheduler-reentry assertion with the independent completion boundary.
The conformance adapter only drops the removed switch header; no source
case, verdict, manifest or case selection changes.

Keep the scalar-leaf, sequential-refusal and recursive-frontier compiler
controls, research bundle, native comparisons and compute-bench workflow
on the existing evidence branch. This change makes no cross-library
performance claim.

## 8498c77 2026-09-10 Merge pull request #31 from mbbill/io/bench-programs-current-language

Bring the I/O bench programs to the current language and let the gate see them
## 8f0460e 2026-09-10 Merge remote-tracking branch 'origin/main' into compute/current-stack-runtime


## 731836d 2026-09-10 Document deferred connection concurrency and select runnable network references

Append a dated boundary for the superseded park-on-miss scheduler and describe the TCP program as source-order ordinary calls. Select uring and epoll for the CI network protocol because its concurrent four-peer correctness pass cannot complete with the current Whitefoot server. The WF program still compiles under programs-check; this table makes no WF network execution or performance claim. Preserve pipefail and the existing reference checks.

## 33ed2c0 2026-09-10 Merge pull request #32 from mbbill/compute/current-stack-runtime

Restore current-stack compute execution with independent I/O waits
## 4df2456 2026-09-10 Port compiler compute offer controls onto the current-stack runtime

Extract scalar-leaf suppression, sequential refusal and recursive frontier controls from PR #28 at 70aa8e5. Keep current main runtime, acceptance, joins, ordinary signatures and non-par lowering unchanged. Share the existing iterative graph decomposition with frontier specialization and stack attribution.

Retain the provisional scalar-leaf default of 16 after a five-pair ordinary-compiler quadrature comparison on main runtime 33ed2c00. Include only that numerical program with a normal command entry, analytic oracle and reproduction commands. Record the owner-reported Windows mixed-total tail as an open measurement.

## cce8256 2026-09-10 Clarify policy-dependent reports and clone routing after review


## 268f370 2026-09-10 Measure Windows qualification with one fewer compute worker


## dcfb85c 2026-09-10 Balance worker comparison positions and label diagnostic rows


## 6e95208 2026-09-10 Record Windows worker qualification and its throughput tradeoff


## a93e4c1 2026-09-10 Compare Windows variability with a native multithread control


## 145c0f7 2026-09-10 Record native-relative qualification and preserve fractional margins


## 677dd3e 2026-09-11 Add the design tree skill and a pilot migration of checks-and-proofs

Introduce design/ as the live design-decision record that will replace
mcts_mem/: a concept-organized tree of decisions with their reasons and
rejected alternatives, a change log with one entry per approved tree diff,
and a project-independent skill holding the procedure, templates, check
prompts, and a structural lint.

The pilot migrates the root and the checks-and-proofs subtree from
mcts_mem/ at 3016842: nine nodes, 33 decisions, 21 rejected alternatives,
no dated history. mcts_mem/ stays in place until the remaining twelve
subtrees are migrated; design/README.md states the cutover condition.

The lint checks form only (node template, decision markers, universal-scope
instance lists, link resolution, name uniqueness, ASCII-only text, no
history sections, log-entry structure, and one log entry naming every node
changed since main). It is exposed as `make design-lint` and is not yet a
`make check` stage.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01DmvaM2XFNArVZGTAk9hNHr

## 5dba7e3 2026-09-11 State the real reasons for the optional-optimizer-facts decision

The migrated root decision restated its rule as its reason. Split it into
two decisions carrying the four recorded reasons: optimizer-independent
acceptance, the facts-off build as the reference that exposes a wrong
emitted attribute, per-family attribution against that reference, and no
debug-versus-release split. Logged as an owner discussion.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01DmvaM2XFNArVZGTAk9hNHr

## f761005 2026-09-11 Name what the acceptance half of the optimizer-facts rule constrains

The acceptance half of the root decision on optional optimizer facts did
not say whom it bound. It now states its subjects and its testable form:
same accepted programs and verdicts with facts on or off, so the checker's
fact sources are closed to optimizer output and no rule makes acceptance
depend on a derivable optional fact. Logged as an owner discussion with
the two existing instances assigned to their future subtrees.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01DmvaM2XFNArVZGTAk9hNHr

## 7607e2b 2026-09-10 Merge pull request #34 from mbbill/codex/windows-bench-stability

Stabilize Windows qualification with W=3 and native controls
## 597ba84 2026-09-11 Node layout: scope first, blank lines between fields, reasoned rejections

Fields are separated by blank lines so each renders as its own paragraph,
Scope: sits directly under the title because it says what the node governs,
and every Rejected: line carries its reason behind the same marker a
decision uses. Template, procedure, check prompt G1, and the lint change
together; the nine nodes were converted mechanically with no decision
content changed. Logged as an owner discussion.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01DmvaM2XFNArVZGTAk9hNHr

## b736d47 2026-09-11 Write decisions for a reader who has not seen the record

The contract-block node's reasons were compressed from the memory record's
terms of art and could not be read on their own. Rewrite its five
decisions and two rejected lines in plain words with no change in content,
and make readability without the source record a standing rule in the
procedure, the bootstrapping step, and design-gate check G1. Logged as an
owner discussion.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01DmvaM2XFNArVZGTAk9hNHr

## f13dd48 2026-09-11 Merge origin/main (7607e2be) into codex/compiler-scheduling-controls

Both sides appended entries to mcts_mem/whitefoot/parallelism.md; the merge
keeps both sets in date order.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

## 395042f 2026-09-10 Merge pull request #33 from mbbill/codex/compiler-scheduling-controls

Add compiler compute scheduling controls
## e8bf625 2026-09-11 Remove fields and mechanisms the agent added without discussion

The owner did not ask for and did not know about several things the skill
carried: an Origin: label and a Code: line on log entries, Instances: lists
with statuses and the "all X" scope marker, Applies-to: links, a pending
node status, a per-commit check, and the checks that depended on them (G2
scope test, C5 universal coverage). All are removed from the procedure,
templates, check prompts, lint, and the existing log entries. The
consistency scan is restated in the owner's form: assume the tree is
consistent, read the ancestor chain and same-scope nodes, extend as needed,
report what was read. The workflow is the five steps the owner approved,
with the correspondence checks at pull-request time only. No tree node
changed.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01DmvaM2XFNArVZGTAk9hNHr

## e89ee75 2026-09-11 Remove the Scope field from nodes, lint, procedure, and template

A field with no rule that maintains it will rot. Scope restated what a
node's title and position already imply and nothing updated it. Deleted
from every node; the lint no longer requires or orders it. No decision
content changed.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01DmvaM2XFNArVZGTAk9hNHr

## f3e5b35 2026-09-11 Apply the no-maintainer rule across the design skill and wire the lint in

Anything with no rule that maintains it will rot. Removed: node title lines
(the file name is the name), the lapses-when clause on rejected lines, the
add/change/remove prefixes on log Nodes lines, design/README.md (the root
README names the directory), the templates (the lint is the format), the
separate checks file (merged into the procedure), two duplicate
descriptions of what the lint checks, and the one-time bootstrapping
section. Added: design-lint as a make check and make static stage, since a
gate target outside the gate has no maintainer.

The tree root no longer restates the constitution's principles. The four
concrete decisions the constitution carried (no runtime trap as a language
feature, no bypass of required proof, no exponential checking work,
migration cost not a ground before real users) now live in the tree root
with their reasons, and the constitution keeps purpose, objectives, and
priorities. No live document quoted the moved sentences.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01DmvaM2XFNArVZGTAk9hNHr

## 5a5a62d 2026-09-11 Split the design tree into language and compiler trees; retire mcts_mem writes

Language decisions are checked against the specification and compiler
decisions against the code, so they are two trees: design/language and
design/compiler. The pilot subtree moves under language and every decision
that restated a rule's semantics is cut to the decision itself; the
specification owns the semantics. The checker-is-TCB rule moves to the
compiler root, which is seeded from the toolchain and fact-channels memory
nodes with reasons taken from their replacement records.

CLAUDE.md/AGENTS.md, README, practice, review-checklist, and roadmap no
longer direct decisions into mcts_mem/, which stays as a frozen record until
its content is moved. The review checklist's memory section becomes the
design-tree section (tree before code, correspondence, form). Known
compiler defects move from compiler/README to docs/todo.md and the run
instructions to the root README, ahead of that file's retirement.

The derivation ledger is not deleted here: the active specification's
META-6 requires it and the whitefoot-spec gate reads it, so its retirement
is a specification amendment awaiting the owner's approval.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01DmvaM2XFNArVZGTAk9hNHr

## 97ff4a5 2026-09-11 Migrate the remaining memory subtrees and retire the compiler README

Move every remaining mcts_mem subtree into the two design trees: 47 nodes
under design/language and design/compiler, each holding only decisions
with their reasons and the refused alternatives. Reasons come from each
record's replacement entries and recorded rationale; dated measurements,
claim-era alternatives, and the development-workflow record stay out of
the trees because the guidance files own the process.

Retire compiler/README.md. Its running and checking commands move to the
root README, its known defects to docs/todo.md, its decisions to the
compiler tree, and its implemented-surface statements to the conformance
results. Every live reference is repointed, the file leaves the
spec-prose-integrity lists, and the memory-era wording in the agent
instructions, practice guide, review checklist, research indexes, and
roadmap now names the design trees instead.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01DmvaM2XFNArVZGTAk9hNHr

## 7203e33 2026-09-11 Merge origin/main into the design tree branch

Keep both gate additions in CHECK_STAGES, design-lint from this branch and
bench-programs from main, and keep compiler/README.md retired; the
paragraphs main added to it move to their new homes in the next change.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01DmvaM2XFNArVZGTAk9hNHr

## 207860a 2026-09-11 Bring the compiler tree up to the current-stack runtime and grain controls

Main merged the current-stack runtime and three opt-in --par grain
controls after this branch retired compiler/README.md and migrated the
memory tree. Carry that material to its new homes: six decisions and one
rejected alternative in the compiler tree's parallel-lowering nodes, the
flags and runtime settings in the root README, the connection-concurrency
gap and the inert WF_STACKS setting in docs/todo.md, and the scalar-leaf
remeasurement protocol in the proof-derived-parallelism investigation.
Mark design-lint phony and point the log's format note at the skill.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01DmvaM2XFNArVZGTAk9hNHr

## 9b7e0d6 2026-09-11 Add a temporary Chinese rendering of the design trees for review

design/zh-tmp mirrors design/ in Chinese for the owner's review of the
migrated trees. The English files stay the only maintained ones; this
directory is deleted after the review and nothing links to it.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01DmvaM2XFNArVZGTAk9hNHr

## 80a9212 2026-09-11 Apply the pull-request check findings to the language tree

The procedure's pull-request checks ran over the language tree against the
specification. Reorder the system-interface root's flaw list to match its
alternatives, spell out the terms that needed the retired record to be
understood, tell the requires-and-ensures block apart from the trait-like
contract, give the unfiltered directory entries the specification's own
reason, and remove two restatements of rules the root already states. The
findings left to the owner are listed in the log entry.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01DmvaM2XFNArVZGTAk9hNHr

## f454a87 2026-09-11 Apply the pull-request check findings to the compiler tree

The procedure's pull-request checks ran over the compiler tree against the
code. The cleanup-traversal node had been migrated from a memory record the
owner's 2026-09-04 ruling had already superseded; rewrite it from the
emitter's recorded ruling, with the worklist and the cycle refusal as
rejected alternatives. Repair wording in six other nodes without changing
their meaning; the log entry lists each repair.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01DmvaM2XFNArVZGTAk9hNHr

## 56a7144 2026-09-11 Refresh the Chinese rendering after the review repairs

Redo the 21 files of design/zh-tmp whose English sources changed under
the pull-request check findings, so the rendering the owner reads matches
the current trees and log.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01DmvaM2XFNArVZGTAk9hNHr

## 3039639 2026-09-11 Add plain-language explanations to the Chinese rendering

Every decision, rejected alternative, and log entry in design/zh-tmp now
carries a line beginning with a plain-language explanation for the
owner's review. The explanations are review aids only, have no English
counterpart, and leave the rendering's own lines unchanged.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01DmvaM2XFNArVZGTAk9hNHr

## 8c4c253 2026-09-12 Review the compiler root with the owner

Cut the compiler root from eleven decisions to four under the owner's
rulings: the optimizer-fact decisions go, since the intent was never a
facts-on and facts-off pair of builds but the absence of any
behavior-changing build mode; the tautological, historical, and project
direction entries go or move (the ripgrep target to the research
experiments index); the remaining decisions merge into research
instrument, rules not shapes, explicit unsupported, and deterministic
rule-citing rejection. The language root's optimizer-facts decision
becomes the one-source-one-program decision. The agent instructions'
compiler-rules section now points at the tree, and its two repository
rules move to the hygiene section. The Chinese rendering follows.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01DmvaM2XFNArVZGTAk9hNHr

## 1ec22a9 2026-09-12 Review cleanup-traversal and derived-totality with the owner

Drop the buffer release-loop description from cleanup-traversal and delete
derived-totality, whose decisions were the withdrawn optimizer-facts rule
and a design for a fact that does not exist; the emitted-attribute test
keeps pinning that no module promises termination. Drop the roadmap's link
to the deleted node and update the Chinese rendering.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01DmvaM2XFNArVZGTAk9hNHr

## bfc3b09 2026-09-12 Review the parallel-lowering subtree with the owner

Remove the default-off decision for compute actualization, since --par is
an implementation choice that does not affect correctness; shorten the
startup decision to its content; give the clone-set decision its own
reason; merge the I/O-join decision into the current-stack runtime
decision; and fold lane-stack into parallel-runtime. The Chinese
rendering follows.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01DmvaM2XFNArVZGTAk9hNHr

## fd87c60 2026-09-12 Complete the parallel-lowering review edits and log entry

The previous commit landed only the first half of the subtree review:
finish the clone-set reason in two-worlds, merge the I/O-join decision
into the current-stack runtime decision, carry lane-stack's decision and
rejected alternative into parallel-runtime, add the log entry, and bring
the Chinese rendering along.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01DmvaM2XFNArVZGTAk9hNHr

## 8e676ac 2026-09-12 Review the remaining compiler leaves with the owner

Confirm resource-exhaustion-floor, tag-only-lowering, and
wide-probe-lowering; reword the exhaustion decision for length without
changing its content. The compiler tree review is complete.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01DmvaM2XFNArVZGTAk9hNHr

## b85faa9 2026-09-12 Review the language root and checks-and-proofs with the owner

Restate the one-behavior decision in general terms, widen the
migration-cost decision into the merits-not-cost-or-corpus rule, put the
no-SMT decision first in checks-and-proofs, drop the reason the root
already gives and the runtime-fallback restatement, and reword the second
rejected alternative. The Chinese rendering follows.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01DmvaM2XFNArVZGTAk9hNHr

## 180affe 2026-09-12 Review the checks-and-proofs children with the owner

Trim certificate-fold's rejected list, drop obligation-discharge's
restatements of the roots and fold writer-trap-surface into it, give the
contradictory-requirements decision its real reason, fold
requirement-enforcement into requires-entry-contract, and drop the
claim-era and recognizer rejections the roots already cover. The Chinese
rendering follows.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01DmvaM2XFNArVZGTAk9hNHr

## 1cab839 2026-09-12 Review contracts, data-model, effects, and name-resolution with the owner

Drop the circular effect-subtyping rejection, restate stable identity and
generalize the relocation rule in data-model, fold the two data-model
decisions of container-representation into it and delete that node, drop
the optimizer-fact clause from tag-only-equality, give the effect-row
exactness decision an honest reason, and add that pure promises nothing
about termination. The Chinese rendering follows.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01DmvaM2XFNArVZGTAk9hNHr

## c31d437 2026-09-12 Complete the data-model, effects, and tag-only edits and their log entry

The previous commit landed only part of this review round: finish the
Chinese data-model rendering, drop the optimizer-fact clause from
tag-only-equality, give the effect-row exactness decision its honest
reason, add that pure promises nothing about termination, and write the
log entry.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01DmvaM2XFNArVZGTAk9hNHr

## 9185a06 2026-09-12 Review the law decision and the ownership subtree with the owner

Restate the contracts law decision, drop the future clause from
affine-replacement, and fold control-header-temporary-loans into
no-reborrow. The Chinese rendering follows.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01DmvaM2XFNArVZGTAk9hNHr

## e8b61ba 2026-09-12 Complete the ownership review edits and their log entry

The previous commit deleted control-header-temporary-loans before its
decision and rejected alternative had been folded into no-reborrow, and
left the affine-replacement trim, the Chinese law decision, and the log
entry unapplied. Finish all of them.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01DmvaM2XFNArVZGTAk9hNHr

## 0c48999 2026-09-12 Review the parallelism subtree and pattern-doctrine with the owner

Give the permission decision a reason without optimizer facts, drop the
scheduling-edge restatement and the out-of-date auto-parallelization
decision, fold loop-permission's one language sentence into parallelism,
and fold pattern-doctrine's writer-trial decision into the language
root. The Chinese rendering follows.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01DmvaM2XFNArVZGTAk9hNHr

## 1d6f122 2026-09-12 Review the surface-form subtree with the owner

Drop the two root-covered evidence-policy decisions, fold
binding-annotation and iteration-forms into the surface-form root, and
remove the rejected lists that only restated their decisions, keeping
the one independent alternative. The Chinese rendering follows.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01DmvaM2XFNArVZGTAk9hNHr

## ef7c7bf 2026-09-12 Review the system-interface subtree with the owner

Strengthen the ownership decision so that system objects are ordinary
owned objects and no language mechanism is invented for system state,
drop the effects duplicate and the runtime measurement, merge the entry
form with the entry decision from requires-entry-contract, shorten
declaration-home, and drop directory-enumeration's restating rejection.
The language tree review is complete. The Chinese rendering follows.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01DmvaM2XFNArVZGTAk9hNHr

## 2c1bd39 2026-09-12 Retire the derivation ledger: kernel specification v0.53

Amend the specification to v0.53, archiving the outgoing v0.52 bytes:
[META-6], which required a rule-to-ground index at
spec/derivation/derivation-ledger.md, is removed (rules -1, tokens 0,
spellings 0, exceptions 0; selection ground: the owner ruled that the
specification and the design tree are the two records and must be
consistent with each other, so a third record that nothing consumes only
drifts). Delete the ledger and its v0.2 predecessor, move the
Featherweight-Rust reconciliation memo to research/notes, and drop the
META-6 conformance manifest row with the rule. The whitefoot-spec gate
keeps its identity, unique-rule-id, and cross-reference checks and loses
the index coverage checks; the qualification tripwire moves to v0.53.
Guidance that pointed at the index now points at the design trees, and
the language root records the ruling.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01DmvaM2XFNArVZGTAk9hNHr

## 5e576c7 2026-09-12 Classify changed regions instead of matching every one to a node

The owner ruled that review is reading and judgment: the correspondence
check now classifies each changed region as needing no decision, with a
one-line statement the owner reads, or as a choice that gets a node; the
delete test is dropped, and CI owns the rest. Open a temporary recall
workspace for recovering past decisions into the trees.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01DmvaM2XFNArVZGTAk9hNHr

