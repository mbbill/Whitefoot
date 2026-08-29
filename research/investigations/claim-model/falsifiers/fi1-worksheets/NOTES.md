# wf-0109 F-I1 notes

Target: DESIGN.md @ 236b837f, branch batch/0106-claim-model-design (read-only).
Task: hand-execute [IND-7]'s certificate check against every worked trace;
three adversarial constructions; REPORT.md.

Gate binary reused: wf-0107-audit/target/gate/whitefootc (runs; verdicts on the
existing probes match the design's claims).

Traces found in the file (F-I1 names "3.9.3, 3.9.4, 3.9.5 plus the counted ipv4
restructure of 4.4"; 2.4 names six):
  T1 I2 base 3.9.3      T2 I2 step 3.9.3     T3 I3 step 3.9.3
  T4 I1 midpoint 3.9.4  T5 I4 base+2 paths 3.9.5
  T6 counted ipv4 4.4 - NOT DRAFTED (constructed here)
  T7 A16/FATAL-1 refusal 3.8.3   T8 A2/FF2 refusal 3.8.3
  ("the four bucket-B statements" of 2.4: also not drafted; 2.8 routes them away)

Status: worksheets done; adversarial (a) no counterexample, (b) two soundness
breaks found, (c) finite but not spec-fixed + two cap-driven monotonicity
exceptions. Verdict FAIL (F-I1's own refutation criterion fires on T6).
