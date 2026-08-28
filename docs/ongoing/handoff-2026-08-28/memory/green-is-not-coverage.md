---
name: green-is-not-coverage
description: A passing test is not evidence it tests anything — and any process that hunts failures is structurally blind to problems that manifest as success
metadata: 
  node_type: memory
  type: feedback
  originSessionId: f2692882-d1e0-42f4-b979-c7c69aed0c3d
  modified: 2026-08-08T17:14:14.767Z
---

Found 2026-08-08 on Whitefoot's v0.23 batch, by an executor rather than by me.
A conformance case named `fn2-pos-explicit-instantiation` was **passing while
testing nothing**: its subject was that explicit instantiation arguments are
written and monomorphized, the spec revision respelled those arguments away,
and its source became `let a = 40_i32 + 2_i32;` with none anywhere — doc still
claiming one.

**The general defect:** a change that deletes a construct silently empties every
test whose subject was that construct, and roughly half go **green** rather than
red. Negative tests whose subject died turn red; positive tests whose subject
died turn green.

**Why it matters more than the individual case:** we had spent four rounds
catching the red ones one at a time and had caught none of the green ones. That
is not bad luck, it is a **selection effect in the verification process itself**
— a process that finds problems by watching for failures is structurally blind
to problems that manifest as success. The blindness scales with how many
constructs a change deletes.

**How to apply.** (1) After any change that removes a construct, sweep for tests
whose stated subject the source no longer contains — the cheap tell is a
test/case whose doc or name mentions something its body no longer has. (2) Never
report a suite's pass count as coverage evidence without saying whether anything
distinguishes "passes because the rule holds" from "passes because the case no
longer tests it". (3) Related and measured the same day: a coverage metric that
counts per-RULE rather than per-content-piece reports a rule "covered" when
pieces of its content have no test at all — seen twice, on SYS-2 and on FN-2's
wrong-kind argument. (4) A masked failure means the number of hidden problems is
*unknown*, never one: when a failing entry hid a second, re-check the whole
population, not the one that surfaced. The executor's sharper version, which is
actionable where mine was only a warning: **a mask's fix is itself a probe** —
read the run immediately after removing one carefully instead of treating it as
confirmation, because that is the moment previously unreachable code first
executes. It surfaced a stale expectation nobody was hunting. (5) A related
principle it generalized past its origin: **any transform's correctness is
defined against the input it should have handled, not the output it produced** —
testing a migrator, renderer, or formatter on its own output is a fixed point,
so it always passes.

**A third class exists and is worse than both (found by the sweep itself,
2026-08-08): subject shifted.** A test still fires, still cites the *correct*
rule, but its violation is no longer the one it was written about — three OP-1
negatives written about a wrong written type argument now reject on an operand
domain. An emptied test can be caught by asking whether its rule still fires;
these pass that question, so a rule-level audit clears them. Only reading each
test's stated subject against its actual source finds them. So the audit
question is not "does the rule still fire?" but "**is the thing that fires the
thing this test was written about?**"

**Two method notes from the same sweep, both about not poisoning your own
result.** A first pass matching short identifiers as substrings returned 97
candidates, nearly all artefacts (`ile` inside "while", `ine` inside "line");
the executor discarded the list rather than reporting it with a caveat, because
a sweep whose hits are mostly noise is worse than none — nobody re-reads 97 to
find 5. And it reported "**at least** two, plus three", stating what the
mechanical tell cannot see (a doc that names no construct at all), which is what
makes a count usable in an approval rather than quietly false.

Deliberately NOT built: a mechanism to detect this automatically. It would need
each test's subject stated machine-readably, which is a real design change, and
this project forbids machinery no current experiment needs. See
[[agent-reports-are-claims]] and [[residue-hunt-review-axis]].
