# 0037 — deflate driver through a real system boundary

This is a temporary live coordination record, not execution authority.

- **Status:** `PLANNED` (unclaimed)
- **Authority:** owner approval 2026-08-07 (provenance gate advanced in
  priority); the held candidate
  `governance/spec-evolution/provenance-gate-candidate.md`
- **Owner / workspace:** unclaimed / filled at claim
- **Base revision:** filled at claim
- **Dependency:** none for the driver itself; the provenance gate's
  activation depends on this task's measurement

## Goal

Make the provenance gate measurable on the sites that motivated it. Today
the deflate programs build their compressed input synthetically
(`make_dynamic_input()`, unlabelled `main`), so no value in them has a
boundary origin and the gate has zero live instances — it cannot fire on
the three canonical-Huffman sites the 0035 acceptance run found migrated
as aborting claims.

Wire a deflate driver through a real §14 boundary: a `command fn main`
that reads compressed bytes with the SYS read path and decodes them, so
the decoder's table indices carry external provenance. Then measure, with
the held candidate's rule applied by hand or by a scratch prototype:
which existing claims become illegal, whether the three canonical-Huffman
sites are among them, and what the honest repair costs (branch with an
`InvalidHuffmanCode` value path). Record the measurement beside the
acceptance evidence in `research/investigations/obligation-discharge/`.

Deliverable is evidence, not a language change. A finding that the gate
does NOT fire on those sites is a successful outcome and must be reported
as plainly as a finding that it does.
