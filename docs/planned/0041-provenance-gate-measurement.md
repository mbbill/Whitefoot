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
- **Dependency:** the ENT-5 loop-fix activation, frozen acceptance rerun, and
  revalidation of the already-shipped SYS-8/ENT-3 S10 count facts on the
  boundary-fed path must be terminal first. This is a hard ordering constraint,
  not a preference — see below.

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

Do not measure before the ENT-5 loop fix is active and the existing S10 facts
have been revalidated on the same boundary-fed consumer. The deflate path's
discharge is currently dominated by the loop-rule defect that
`research/investigations/obligation-discharge/ACCEPTANCE.md` isolates as the
dominant cause of the deflate divergence, so a measurement taken against
today's compiler would attribute to provenance what the loop rule caused.

At registration, the block was confirmed against the then-active specification,
compiler identity, activation chain, and held ENT-5 candidate rather than taken
on trust. The active authority is now v0.23 and the ENT-5 delta has been re-cut
against those bytes, but the same ordering fact remains: the complete v0.24
candidate still needs owner exact-byte approval and activation before this
measurement can have valid attribution.

## Sequence

1. An ENT-5 activation lands.
2. Re-run the acceptance measurement against it, so the discharge baseline is
   the fixed loop rule rather than the defective one. Compare against
   `ACCEPTANCE.md`'s recorded buckets, state the delta, and confirm that the
   active SYS-8/ENT-3 S10 count facts enter and serve the boundary-fed path.
3. Only then apply the gate rule — by hand or by a scratch prototype — to the
   boundary-fed driver's sites, and record the result beside the acceptance
   evidence in `research/investigations/obligation-discharge/`.

Do not reorder these. Step 3 taken before step 2 produces a number with no
attribution, which is the failure this constraint exists to prevent.

## Not in this task

No language change, no gate activation, and no edit to the held candidate.
The gate's activation depends on this measurement and is separate work.
