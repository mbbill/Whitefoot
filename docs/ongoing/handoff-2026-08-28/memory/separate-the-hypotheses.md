---
name: separate-the-hypotheses
description: "Look for the observation that separates two hypotheses, not one consistent with the one you already hold — the single most transferable habit from the Whitefoot batch"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: f2692882-d1e0-42f4-b979-c7c69aed0c3d
  modified: 2026-08-08T17:42:42.972Z
---

Formulated 2026-08-08 by an executor, correcting my weaker version of it. I had
praised it for "checking what a thing *is* rather than what it's *called*". Its
own statement is better and is the one that transfers:

> I try to find the observation that **separates two hypotheses**, rather than
> one consistent with the one I already hold.

Instances from one session, all of which produced findings nobody had asked for:

- **`git diff HEAD` empty *combined with* HEAD containing the fix.** Either alone
  is consistent with "my work was destroyed"; together they are not. That is what
  stopped an executor from "restoring" work that was never lost.
- **Same-binding versus different-binding in the two branches.** Held everything
  fixed but the join: same-binding reached the real rejection, different-binding
  reached an internal Unsupported. That reclassified a "missing capability" as a
  masked negative in one move.
- **Tests that MOVED versus tests that STAYED PUT** after a fix. Staying put means
  the fix did not work; moving to a different error means it worked and a second
  cause is underneath. Same count either way.
- **A deliberate break in both directions.** Breaking a rank value made the check
  fail with its message; deleting a variant from the chain made it fail to
  compile. Proving a check *can* fail, in each mode, is what distinguishes a real
  check from a decorative one.
- **Running a transform against the input it should have handled**, not against
  its own output — the latter is a fixed point and always passes.

**The unification, formulated by the same executor at the end of the session and
worth more than any single instance.** Three defects that felt like different
problems all day are one: a stale baseline (an operation performed against a
tree or commit that no longer describes reality), a masked failure (one
rejection hiding another behind it), and an emptied test that went green. **All
three fail by looking like success.** That is why none is caught by watching for
failures, why each needed a deliberate question rather than a gate, and why the
discriminating-observation habit is the tool for all three: when the failure mode
*is* success, the only way through is an observation that separates "succeeded
because it worked" from "succeeded because it stopped checking".

**The anti-pattern it names:** gathering evidence consistent with your current
hypothesis. That feels like verification and is not, because the same observation
is usually consistent with the rival hypothesis too. Before running a check, ask
what result would make you believe the *other* thing; if no result would, the
check is decorative.

Related: [[green-is-not-coverage]] (a passing test may test nothing),
[[agent-reports-are-claims]] (a claim's backing, not its author, is the variable).
