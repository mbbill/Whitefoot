---
name: grep-from-the-production
description: "When measuring a corpus against a grammar, read the production and enumerate its optional parts before writing the pattern — remembered syntax undercounts silently"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: f2692882-d1e0-42f4-b979-c7c69aed0c3d
  modified: 2026-08-19T05:45:04.614Z
---

2026-08-18, Whitefoot contract-surface study: I measured "how many program
entries carry a `requires`" twice and got it wrong both times, from the same
root cause — I wrote the grep pattern from my memory of the grammar instead of
reading the production and enumerating its optional parts.

`fn_decl := "deny_claims"? program_kind? "fn" IDENT ...`

Miss 1: my pattern required a leading `program_kind` word, so it dropped all
451 unlabelled `fn main()` entries (I reported 77 instead of 525).
Miss 2: the pattern anchored at line start without admitting the optional
`deny_claims` prefix, dropping 4 more — and **two of the four dropped
declarations were exactly the ones carrying a `requires`**, so the undercount
landed precisely on the measurement's subject. I reported "exactly one entry
carries a requires"; the answer was three. A subagent caught it.

**Why:** an anchored pattern built from remembered syntax fails silently — it
returns a clean, plausible number with no sign that a whole arm of the
production was excluded. The owner was making a language-design decision on
those numbers.

**How to apply:** before grepping a corpus for a syntactic shape, open the
production, list every optional and alternative part, and build the pattern to
admit all of them — or better, count with the compiler's own parser when one
exists. Sanity-check the total against an independent invariant (here: FN-7
mandates exactly one `main` per unit, so entries ≈ number of `.wf` files —
77 against 637 files should have been an immediate red flag). State corpus
numbers to the owner with the pattern used, so a wrong pattern is auditable.
See [[agent-reports-are-claims]] — this is its mirror: my own measurements
need the same suspicion I apply to a subagent's.
