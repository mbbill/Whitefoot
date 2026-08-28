---
name: test-economy-rule
description: "owner's standing rule — a test earns runtime by purpose; mindless xN/exhaustive repetition strictly forbidden; diagnose WHY a test is slow before splitting it out"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: f2692882-d1e0-42f4-b979-c7c69aed0c3d
  modified: 2026-08-17T05:24:52.488Z
---

Owner ruling 2026-08-16, after I split heavy tests out of `make check` by
wall time alone: "测试要有测试的目的,能达成目的的情况下尽量让时间少一点。
无脑的x10,x100,这种愚蠢又毫无意义的做法必须严格禁止。"

**Why:** splitting by duration treats the symptom. A 30-minute test is not
thereby more thorough — the owner asks what the runtime BUYS. In Whitefoot
the measured answer was: ten wfgrep scenarios at 136s each were 99% redundant
recompilation of one immutable artifact; sharing it kept all ten scenarios at
136s total. Duration-based triage would have hidden that forever in a
"heavy" bucket.

**How to apply:** before classifying any test as heavy/slow, decompose its
cost: setup vs assertion, intrinsic vs repeated. Fix redundant setup by
sharing (unless isolation/repetition IS the property tested — [[green-is-not-coverage]]
adjacent: generics.rs compiles twice deliberately for a determinism
assertion). Challenge exhaustive sweeps: what property makes boundary
sampling insufficient? Rule now codified in docs/WORKFLOW.md Evidence
discipline. Related: [[rederive-with-ai-term]] (decompose cost terms before
asserting).
