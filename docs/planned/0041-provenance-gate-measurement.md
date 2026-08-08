# 0041 — measure the provenance gate on the boundary-fed deflate sites

This is a temporary live coordination record, not execution authority.

- **Status:** `PLANNED` (unclaimed)
- **Authority:** owner approval 2026-08-07 (provenance gate advanced in
  priority); the held candidate
  `governance/spec-evolution/provenance-gate-candidate.md`. This record
  carries forward the measurement half of task 0037, which is PARKED with its
  driver delivered.
- **Owner / workspace:** unclaimed / filled at claim
- **Base revision:** filled at claim
- **Dependency:** the ENT-5 loop-fix activation must be terminal first. This
  is a hard ordering constraint, not a preference — see below.

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

## The ordering constraint (verified 2026-08-08, not assumed)

Do not measure before the ENT-5 loop fix is active. The deflate path's
discharge is currently dominated by the loop-rule defect that
`research/investigations/obligation-discharge/ACCEPTANCE.md` isolates as the
dominant cause of the deflate divergence, so a measurement taken against
today's compiler would attribute to provenance what the loop rule caused.

The block was confirmed four independent ways rather than taken on trust:
`git ls-tree --name-only main spec/` lists nothing above v0.22; the roadmap
names v0.22 as the active authority; `make -C compiler check` reports the
v0.22 identity and its unbroken activation chain; and
`governance/spec-evolution/ent5-loop-fix-v024-candidate.md` reads
owner-approved but awaiting activation, with its own header noting the v0.24
number is provisional and may become v0.23.

## Sequence

1. An ENT-5 activation lands.
2. Re-run the acceptance measurement against it, so the discharge baseline is
   the fixed loop rule rather than the defective one. Compare against
   `ACCEPTANCE.md`'s recorded buckets and state the delta.
3. Only then apply the gate rule — by hand or by a scratch prototype — to the
   boundary-fed driver's sites, and record the result beside the acceptance
   evidence in `research/investigations/obligation-discharge/`.

Do not reorder these. Step 3 taken before step 2 produces a number with no
attribution, which is the failure this constraint exists to prevent.

## Not in this task

No language change, no gate activation, and no edit to the held candidate.
The gate's activation depends on this measurement and is separate work.
