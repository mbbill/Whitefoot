# 0041 — measure the provenance gate on the boundary-fed deflate sites

This is a temporary live coordination record, not execution authority.

- **Status:** `IN PROGRESS`
- **Authority:** the ACTIVE stage-5a plan derived from Direction Outline
  revision 22; owner approval 2026-08-07 (provenance gate advanced in
  priority); and the held candidate
  `governance/spec-evolution/provenance-gate-candidate.md`. This record
  carries forward the measurement half of task 0037, which is PARKED with its
  driver delivered.
- **Owner / workspace:** Codex lead /
  `/Users/bytedance/do_not_scan/wf-0045-final-activation`, branch
  `codex/0045-ent5-activation`
- **Base revision:** `e5db43d`
- **Dependency:** `SATISFIED` by terminal task 0045, activation `f4c7e60`, and
  the canonical post-activation confirmation in
  `research/investigations/obligation-discharge/ACCEPTANCE.md`; this claim is
  refreshed onto the closure commit.

## Goal

Apply the held provenance-gate rule to the deflate decoder's sites now that
they carry real external provenance, and report which existing claims the gate
makes illegal, whether the three canonical-Huffman sites are among them, and
what the honest repair costs.

Task 0037 removed the reason this could not be measured: the deflate programs
used to build their compressed input synthetically, so no value in them had a
boundary origin and the gate had zero live instances. It now has instances —
`tests/programs/raw_deflate_boundary.wf` reads the stream through the SYS read
path, so the decoder's table indices are externally provenanced.

The deliverable is evidence, not a language change. **A finding that the gate
does NOT fire on those sites is a successful outcome and must be reported as
plainly as a finding that it does.**

## Progress

- Completed: v0.24 ENT-5 activation, installed frozen acceptance, and the S10
  evidence disposition are terminal at `e5db43d`.
- Current: enumerate and classify every relevant boundary-fed deflate claim
  subject under the held rule.
- Next: record the reproducible table, repair costs, limitations, and resulting
  prerequisite disposition in the canonical probe and outline.

## Direction, method, and scope

Apply the held rule exactly to the frozen boundary-fed deflate path. Enumerate
every relevant claim subject in deterministic source order and retain the
boundary-origin and constraint lineage needed to reproduce the classification.
Report all gated, ungated, ambiguous, false-positive, and false-negative
findings, plus the honest repair each positive would require. Explicitly name
the three canonical-Huffman sites.

This task may use hand analysis or one disposable scratch probe. Its expected
tracked touch set is this task record,
`research/investigations/obligation-discharge/PROBE-TAINT.md`, and
`docs/roadmap.md`. It does not edit the specification, compiler, protected
corpus, active plan, or held candidate, and it cannot activate the gate.

## Satisfied ordering constraint (verified, not assumed)

The measurement may start only because ENT-5 is active and the existing S10
facts were revalidated at their honest evidence boundary: the real boundary-fed
path produces the count relation, while focused actual-obligation controls
consume all four S10 producer families. The deflate path's discharge was
previously dominated by the loop-rule defect that
`research/investigations/obligation-discharge/ACCEPTANCE.md` isolates as the
dominant cause of the deflate divergence, so a measurement taken against the
pre-fix compiler would attribute to provenance what the loop rule caused.

Activation `f4c7e60` and the installed-authority section of `ACCEPTANCE.md`
close that ordering constraint. They preserve UTF-8 at 22/33, SHA-256 at 0/9,
and recover deflate from 5/29 to 11/29 without a proven-site regression.

## Sequence

1. Completed: ENT-5 activates at `f4c7e60`.
2. Completed: the frozen acceptance and S10 evidence are confirmed against the
   installed v0.24 authority.
3. Apply the gate rule to the boundary-fed driver's sites and record the result
   in `PROBE-TAINT.md` and the outline.

Do not reorder these. Step 3 taken before step 2 produces a number with no
attribution, which is the failure this constraint exists to prevent.

## Not in this task

No language change, no gate activation, and no edit to the held candidate.
The gate's activation depends on this measurement and the later stage-7 O3
closure; it is separate work.

## Validation and done-when

Pin the analysis to `f4c7e60`, the v0.24 digest, the exact driver sources, and
the held rule text. The result table must cover every relevant site and be
independently reproducible; any scratch artifact is deleted, and
`git diff --check` plus repository invariants pass. The task is done when the
canonical probe and outline record the full result, limitations, and whether
the prerequisite is positive, negative, or inconclusive. Only a later rolling
plan may select a language change.
