# F2 — the review trial

*Falsifier F2 of `research/investigations/claim-model/DESIGN.md` §8.1, run
2026-08-29 against the red-ink concession U1 in §11. This file reports the
measurement. It does not change the design.*

---

## The one program this trial is about

DESIGN.md §1.4 admits a program it cannot refuse:

```whitefoot
let n = hidden();                                  // a callee produced n
let big = ige(n, 4_u64);                           // the selector
let y = if big { give 5_u64; } else { give 1_u64; }
claim laundered: ilt(y, 4_u64) because "…";        // admitted, and FALSE when big
```

Every value the claim's support reads is a literal this function wrote, so the
gate has nothing to object to. The claim is nevertheless false on one arm, and
which arm runs is decided across a callable boundary. The design calls this the
laundering family, states in §11 that it is "the design's one genuine soundness
residue", and offers review as the only fence, with §3.8's case list as the
evidence the reviewer gets.

That fence is an assertion about people, not about code. F2 is the experiment
that tests it. §11 fixes the bar in advance:

> **If F2 reports that reviewers approve false laundering claims at a material
> rate even with the case list, this design's central concession is wrong and
> the area needs a fourth idea nobody in this batch had.**

The answer on this set is: they did not. Zero false claims were approved, in any
arm, with or without the case list. What follows is that result, what it is
worth, and the three things it does not settle.

---

## Method

**The set.** 24 self-contained Whitefoot programs, each with exactly one `claim`
carrying a five-field `because` record. Eight are false; sixteen are true. Every
false item has a true twin of the same surface shape, placed apart in the
numbering, and the six non-laundering items (remainder range, manual-counter
loop induction, `len`-based bounds) come from real corpus families so that
"reject anything with an `if` in it" would score badly.

The generator recorded three verification steps. I did not run them and repeat
them as that record states them:

- all 24 items were compiled with `compiler/target/debug/whitefootc --emit-llvm`;
  the 6 non-laundering items compile clean end to end;
- all 18 laundering items, true and false alike, produce exactly one diagnostic,
  `CLM-1 NonLocalClaim` — the v0.39 refusal this design's B2 gate removes — and
  nothing else, so the whole set really is the family U1 is about;
- each laundering item was also compiled as a throwaway *local-selector twin*
  with the boundary helper inlined. All 18 twins compile clean, which checks the
  rest of each program and, crucially, checks CLM-2 **non-refutation**: the
  checker does not derive the negation of any false claim. Each false item is
  therefore the admitted-and-false shape U1 describes, not a case the entailment
  would have caught anyway. The twins were deleted.

The parts I checked myself are named as such below.

**Who did what.** A generator agent wrote the 24 sources, the 24 case lists, and
`manifest.json`, which is the only place truth is written. Four reviewer agents
each returned one line per item, `{item, verdict, reason}`, into
`verdicts/A1.jsonl`, `A2.jsonl`, `B1.jsonl`, `B2.jsonl`. I am the analyst: I read
the manifest, the four verdict files, and the 24 sources, and I re-derived every
arithmetic step the reviewers cite. I did not review any item myself before
opening the manifest, and I did not generate or review anything.

**The two conditions.**

```
condition A  (arms A1, A2)   source + case list      // §3.8's review evidence attached
condition B  (arms B1, B2)   source only             // the control
```

**Blinding.** Reviewers saw the item sources and, in condition A, the case
lists. Nothing in either file marks truth: `manifest.json` was never given to a
reviewer, and the truth-named staging files the generator used were deleted. The
true and false twins share `doc` strings, function names, `main` shapes, and call
arguments. Every item's `main` calls the claim-bearing function twice, once low
and once high, so no arm of any selection is dead and no reviewer can dismiss one
by reading the whole program.

**Budgets: what is actually recorded.** Nothing. No per-item time limit, token
limit, or instruction to hurry is preserved in the artifacts, and I did not watch
the runs. The only two observables are file timestamps and reason length:

```
set frozen        07:56–07:57
A2.jsonl written  08:00:36     A1.jsonl  08:00:38      // ~3 min for 24 items
B2.jsonl written  08:02:21     B1.jsonl  08:02:41      // ~2 min for 24 items
mean reason length   A1 125   A2 152   B1 149   B2 129 chars
```

So each arm reviewed 24 claims in a few minutes and wrote roughly one sentence
per verdict. That is a fast review. It is evidence that the catches were cheap;
it is not evidence about what happens to a reviewer under real pressure, because
no pressure was applied.

**The limit that matters most.** The reviewers here are AI agents of one model
family, run four times. They are not four independent reviewers, and they are
not humans. Three consequences, stated plainly:

- **Unanimity carries almost no information.** Four samples of one reviewer
  agreeing is close to what one sample agreeing with itself would look like. The
  agreement table below should be read as a consistency check on the protocol,
  not as inter-rater reliability.
- **Correlated blindness is unmeasured.** If this model family has a systematic
  hole on some laundering shape, every arm falls in it together and this trial
  reports a clean sheet. Only a second model family, or a human arm, can find
  that.
- **F2 asked for both.** §8.1 says "human, and separately AI-assisted, since 2719
  admits both". The human half has not been run. This file reports the
  AI-assisted half only, and the design's §11 sentence is not fully discharged
  until the human half exists.

---

## Results

**Per arm.** Every arm scored 24/24.

| arm | condition | false approved (of 8) | true rejected (of 16) | accuracy |
| --- | --- | --- | --- | --- |
| A1 | with case list | 0 — **0.0%** | 0 — **0.0%** | 24/24 |
| A2 | with case list | 0 — **0.0%** | 0 — **0.0%** | 24/24 |
| B1 | source only | 0 — **0.0%** | 0 — **0.0%** | 24/24 |
| B2 | source only | 0 — **0.0%** | 0 — **0.0%** | 24/24 |

**Pooled per condition,** with the one-sided 95% upper bound for zero events in
*n* trials, which is what a clean sheet of this size actually licenses.

| condition | false-approval rate | 95% upper bound | true-rejection rate | 95% upper bound |
| --- | --- | --- | --- | --- |
| A — with case list | 0/16 = **0.0%** | ≤ 17.1% | 0/32 = **0.0%** | ≤ 8.9% |
| B — source only | 0/16 = **0.0%** | ≤ 17.1% | 0/32 = **0.0%** | ≤ 8.9% |
| both, pooled | 0/32 = **0.0%** | ≤ 8.9% | 0/64 = **0.0%** | ≤ 4.6% |

The upper bounds treat the arms as independent trials, which they are not — see
the model-family limit above. They are the most generous reading of the data, and
even the generous reading cannot exclude a per-condition false-approval rate as
high as 17%. What it does exclude is a large one.

**Per family.** Nine laundering surfaces and three honest-control families. Every
cell is 4/4 correct.

| family | items | of which false | all four arms correct |
| --- | --- | --- | --- |
| laundering-value-if-literals | 05, 22 | 05 | yes |
| laundering-value-if-modulus | 02, 12 | 02 | yes |
| laundering-value-if-sum | 10, 20 | 10 | yes |
| laundering-value-if-edge-positions | 09, 18 | 18 | yes |
| laundering-value-match-delivery | 04, 16 | 16 | yes |
| laundering-set-under-if-single-arm | 01, 13 | 13 | yes |
| laundering-set-under-if-both-arms | 08, 17 | 08 | yes |
| laundering-set-under-if-sequential | 06 | — | yes |
| laundering-nested-selection | 14, 21, 24 | 21 | yes |
| remainder-range | 03, 15 | — | yes |
| len-based-bound | 07, 19 | — | yes |
| loop-induction-manual-counter | 11, 23 | — | yes |

Three of the eight false items fail by exactly one (`give 8_u64;` under
`ilt(slot, 8_u64)`, `give 4_u64;` under `ilt(code, 4_u64)`, and a sum that lands
on 8). Those are the ones a skimming reviewer would be expected to miss. All
three were rejected by all four arms.

**Agreement.** 96 verdicts, no disagreement anywhere.

| pair | agreement |
| --- | --- |
| A1 ~ A2 | 24/24 |
| A1 ~ B1 | 24/24 |
| A1 ~ B2 | 24/24 |
| A2 ~ B1 | 24/24 |
| A2 ~ B2 | 24/24 |
| B1 ~ B2 | 24/24 |

Cohen's κ is 1.0 for every pair. Given that all four arms are one model family,
read this as "the protocol produced a stable signal", not as "reviewers agree".

---

## What failed, read from the reasons

F2 was written expecting false approvals to read. There are none, and there are
no true rejections either, so there is nothing in the intended category to
diagnose. The substitute is to read the eight correct rejections and ask what
actually did the work, and then to read every reason for defects that did not
happen to change a verdict. I re-derived every arithmetic step below against the
sources myself.

### The lie's derivation did not work, and it failed the same way eight times out of eight

Every false item's `because` record is honest about the program's *shape* and
false about one *noun phrase*. Item 18 is the pattern:

```whitefoot
const table: array<u8, 8> = [10_u8, 20_u8, …, 80_u8];   // eight entries: 0..7
…
let slot = if wide {
  give 8_u64;                                           // ← the false arm
} else {
  give 0_u64;
}
claim slot_in_table: ilt(slot, 8_u64) because
  "…the then arm delivers the table's last position, which is strictly below
    that same length because a position of an eight-entry table is an index of
    it…"                                                // ← "last position" = 7, not 8
```

The derivation never says "8". It says "the table's last position" and lets the
reader supply the number. Its true twin, item 09, delivers `7_u64` and names the
literal outright. So the only difference between an honest record and a lie in
this set is whether one arm's value is stated or described.

Every rejection in every arm names that substitution and puts the number back:

```
A1 item-18   "The then arm gives 8_u64, which is not strictly below 8_u64;
              the eight-entry table's last position is 7_u64."
B2 item-13   "…the derivation calls 6_u64 the high slot of that window."
A2 item-02   "…despite the because text mis-describing 16_u64 as the table's
              slot count."
```

Twenty-five of the 32 false-item rejections quote the misdescribed phrase back at
the record. The other seven state the arithmetic without naming the phrase, and
three of those still put the right number in its place ("the window's last code
is 3_u64"). Not one rejection rests on a hunch, a style tell, or the mere
presence of a call — each names the arm, the value, and the comparison.

The mechanism is therefore narrower than "reviewers catch laundering". What was
measured is: **when a false laundering derivation misdescribes a literal that
sits in the same function, reviewers recompute the literal and catch it.** That
is the shape §1.4's own example has, and it is a real result. It is not the whole
family, which matters for §11 — see the coverage gap below.

### The case list was not consulted in any visible way, and did not need to be

Condition A had §3.8's case list; condition B did not. The verdicts are
identical. Reading the reasons for traces of the list:

```
                                 A1   A2   B1   B2
"edge" (the case list's word)     2    5    2    2
"reaching" / "definitions"       10    4    4    3
"case list" cited by name         0    0    0    0
line:column reference             0    0    0    0
```

No arm ever cites the case list, and no arm reuses its `line:column` references
or its `forward` / `recurrent` marks. A2 shows the most edge vocabulary — "on the
fall-through edge", "on either edge" — which is consistent with having read the
list, but A2 reached the same verdicts as B2, which did not have it.

The list did leave one visible mark, and it is about reachability. Each case list
ends with a `selector, for orientation` line stating that the entailment excludes
neither edge, so both events reach the claim. Condition B had no such line and
argued reachability for itself:

```
                                  A1   A2  |  B1   B2
rejections that argue the bad      0    0  |   2    4     // out of 8 each
arm is actually reachable
   B2 item-21  "…it is reachable whenever raw is at least 8_u64 (main passes 9_u64)."
   B1 item-16  "…the claim fails whenever choose_route yields Fallback."
```

Every reachability argument in all 96 verdicts came from an arm *without* the
list. That is the case list doing its job: it handed condition A a fact condition
B had to derive. It bought no accuracy, because condition B derived it correctly
every time. On this set the case list saves work and is not load-bearing for the
verdict.

This is not a licence to drop it. Every item here is 20 to 35 lines with the
table declaration in view; the list's value is supposed to show up when the
reaching definitions are far apart, numerous, or behind a back edge, and this set
has none of that. What the trial establishes is the narrower fact that
**§8.1's second question — "do they catch them with the list and miss them
without it?" — was not answerable here, because both conditions scored 100%.**
That question remains open, and a set built to make the list matter is the way to
close it.

### Budget pressure did not show, because none was applied

Reason lengths are stable across arms (125–152 characters mean). They do decline
slightly from the first half of each file to the second, in every arm:

```
mean reason length     items 01–12   items 13–24    change
A1                         132           118         -11%
A2                         158           145          -8%
B1                         152           147          -3%
B2                         137           122         -11%
```

Four items out of eight false ones sit in each half, and accuracy is 100% in both
halves of all four arms, so this decline costs nothing here. It is small, uniform,
and equally explained by later items repeating earlier reasoning; I record it
because it is the only thing in the artifacts that points in the direction of
fatigue, and it should not be read as more than that. The honest statement is
that this trial measured a reviewer with attention to spare. The review-friction
question — what a reviewer does on the fortieth claim of a real change, under a
real deadline — is untouched.

### The only reasoning defect observed: a right verdict on a wrong step

One arm made an arithmetic error that its structural argument survived.

```
item-07:  let store = buffer_new(32_u64, 7_u8);
          let probe = seed % 32_u64;              // seed = 1000_u64, so probe = 8

B2 reason: "probe is 1000_u64 % 32_u64 = 9_u64, strictly below room"
                                        ^^^^^  wrong; 1000 % 32 = 8
A1 reason: "probe is 1000 % 32 = 8_u64"          // correct
```

The verdict is right either way, because "an unsigned remainder is below its
divisor" does not depend on the value. But it shows the reviewer computing
loosely on a step it did not need, which is exactly the habit that would miss an
off-by-one if the step *were* load-bearing. Three of the eight false items are
off-by-one. Nothing else in the 96 reasons is factually wrong: I checked
1000 % 32 = 8, 3209 % 64 = 9, 991 % 24 = 7, 26 % 16 = 10, 26 % 4 = 2, 26 % 8 = 2,
3 + 5 = 8, 3 + 4 = 7, 3 + 1 = 4, and every quoted table length, against the
sources.

One arm did better than the record it was reviewing. Item 23's derivation argues
that the counter is even and therefore at most 6; A2 noted the parity step is
unnecessary because the guard alone bounds the value, and approved on the
simpler ground. That is a reviewer auditing premises rather than pattern-matching
a conclusion.

---

## Verdict against DESIGN.md's own bar

§11's bar is a material false-approval rate **with** the case list. The measured
rate with the case list is **0 of 16, 0.0%**, upper-bounded at 17.1%; without it,
also 0 of 16. The friction cost is **0 of 32 honest claims rejected** in each
condition.

**U1's central concession survives this trial.** On the shape §1.4 itself
exhibits, reviewers did not approve the false claims, and they did not reject the
true twins either — including the six honest non-laundering families that a
reviewer defending itself by rejecting everything with a selector in it would
have failed. The design does not need a fourth idea on the strength of this
evidence, and the recommendation is to proceed with the concession stated as
§11 states it.

That verdict comes with three limits, and none of them is small.

**1. One model family, four runs, no humans.** The whole result is four samples
of one reviewer. §8.1 asked for a human arm; it has not been run. A correlated
blind spot in this family is invisible here by construction.

**2. The lie had one shape.** Every false record in this set misdescribes a
literal in the reviewed function — it calls `5_u64` "the window's last position".
The reviewer catches it by arithmetic on text in front of it. §1.5 describes a
*different* false record, and this trial contains none of it:

```whitefoot
// the shape that WAS tested — a false premise about this function's own text
"…the then arm delivers the table's last position…"     // it delivers 8; last is 7

// the shape that was NOT tested — an honest arm list plus an unpublished premise
"…the then arm delivers 8_u64 and the else arm 0_u64. The then arm cannot be
  selected here, because wide_input returns false for every raw this program
  passes…"                                              // cites hidden(): §1.5 refuses
```

The second is the record §1.5's rule exists to refuse — a derivation that states
the arm values correctly and then reasons about the callee. It is also the record
a competent liar would write, since the first kind loses to a subtraction. F2 has
not measured it. **This is the single most important follow-up**: a second set in
which the false derivations are honest about every arm and false only about which
arm can run.

**3. Nothing here is hard.** Every item is under 40 lines with the const
declaration on line 1, two reaching definitions in the common case, and both arms
reachable from `main`. The difficulty knobs the case list is supposed to
compensate — many reaching definitions, distant ones, back edges, a bound that is
not a visible literal — are all at their easiest setting. A clean sheet at this
difficulty does not predict a clean sheet at the corpus's.

So the honest summary is not "review works". It is: **review works on the
laundering shape the design uses to explain the problem, at the easy end,
measured on one model family, with the case list turning out not to be what
carried it.** That is enough to keep the concession and ship §3.8's evidence. It
is not enough to close U1, and U1 should stay in red ink.

---

## Re-checking this trial

The trial ran on four kinds of file: `manifest.json` (truth), `verdicts/*.jsonl`
(the 96 verdicts), `set/item-NN.wf` (the 24 sources), and `set/item-NN.caselist`
(condition A's attachment). Those 50 files are session scratch and are **not
committed**: they have no caller, no gate wires them, and 48 near-identical
programs in `research/` would be exactly the bulk dump the repository rules
forbid. What survives is this report, and the two appendices below carry the
whole manifest and every verdict verbatim, so the join, the rates, and the
failure reading can be re-checked from this file alone. Re-running the trial
means regenerating the set; the generator's construction and verification
procedure is recorded in §"Method" above.

### Appendix A — every manifest row against both arms of both conditions

| item | family | truth | failing arm, from `manifest.json` | A1 | A2 | B1 | B2 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| item-01 | laundering-set-under-if-single-arm | true | — | approve ✓ | approve ✓ | approve ✓ | approve ✓ |
| item-02 | laundering-value-if-modulus | **false** | item-02.wf:12:5 give seed % 16_u64; — then edge of the value_if at item-02.wf:11:14. seed is 26_u64, so this arm delivers 10_u64 and ilt(slot, 8_u64) is false | reject ✓ | reject ✓ | reject ✓ | reject ✓ |
| item-03 | remainder-range | true | — | approve ✓ | approve ✓ | approve ✓ | approve ✓ |
| item-04 | laundering-value-match-delivery | true | — | approve ✓ | approve ✓ | approve ✓ | approve ✓ |
| item-05 | laundering-value-if-literals | **false** | item-05.wf:11:5 give 5_u64; — then edge of the value_if at item-05.wf:10:16. 5_u64 is not below 4_u64, so ilt(picked, 4_u64) is false | reject ✓ | reject ✓ | reject ✓ | reject ✓ |
| item-06 | laundering-set-under-if-sequential | true | — | approve ✓ | approve ✓ | approve ✓ | approve ✓ |
| item-07 | len-based-bound | true | — | approve ✓ | approve ✓ | approve ✓ | approve ✓ |
| item-08 | laundering-set-under-if-both-arms | **false** | item-08.wf:12:5 set width = 12_u64; — then edge of the if at item-08.wf:11:3. 12_u64 is not below 8_u64, so ilt(width, 8_u64) is false | reject ✓ | reject ✓ | reject ✓ | reject ✓ |
| item-09 | laundering-value-if-edge-positions | true | — | approve ✓ | approve ✓ | approve ✓ | approve ✓ |
| item-10 | laundering-value-if-sum | **false** | item-10.wf:12:5 give base +wrap 5_u64; — then edge of the value_if at item-10.wf:11:14. base is 3_u64, so this arm delivers 8_u64 and ilt(span, 8_u64) is false by one | reject ✓ | reject ✓ | reject ✓ | reject ✓ |
| item-11 | loop-induction-manual-counter | true | — | approve ✓ | approve ✓ | approve ✓ | approve ✓ |
| item-12 | laundering-value-if-modulus | true | — | approve ✓ | approve ✓ | approve ✓ | approve ✓ |
| item-13 | laundering-set-under-if-single-arm | **false** | item-13.wf:12:5 set offset = 6_u64; — then edge of the if at item-13.wf:11:3. 6_u64 is not below 4_u64, so ilt(offset, 4_u64) is false | reject ✓ | reject ✓ | reject ✓ | reject ✓ |
| item-14 | laundering-nested-selection | true | — | approve ✓ | approve ✓ | approve ✓ | approve ✓ |
| item-15 | remainder-range | true | — | approve ✓ | approve ✓ | approve ✓ | approve ✓ |
| item-16 | laundering-value-match-delivery | **false** | item-16.wf:25:7 give 4_u64; — Fallback() arm of the value_match at item-16.wf:20:14. 4_u64 is not below 4_u64, so ilt(code, 4_u64) is false by one | reject ✓ | reject ✓ | reject ✓ | reject ✓ |
| item-17 | laundering-set-under-if-both-arms | true | — | approve ✓ | approve ✓ | approve ✓ | approve ✓ |
| item-18 | laundering-value-if-edge-positions | **false** | item-18.wf:11:5 give 8_u64; — then edge of the value_if at item-18.wf:10:14. 8_u64 is not below 8_u64, so ilt(slot, 8_u64) is false by one | reject ✓ | reject ✓ | reject ✓ | reject ✓ |
| item-19 | len-based-bound | true | — | approve ✓ | approve ✓ | approve ✓ | approve ✓ |
| item-20 | laundering-value-if-sum | true | — | approve ✓ | approve ✓ | approve ✓ | approve ✓ |
| item-21 | laundering-nested-selection | **false** | item-21.wf:20:5 give 9_u64; — else edge of the chained value_if at item-21.wf:17:10. 9_u64 is not below 8_u64, so ilt(level, 8_u64) is false | reject ✓ | reject ✓ | reject ✓ | reject ✓ |
| item-22 | laundering-value-if-literals | true | — | approve ✓ | approve ✓ | approve ✓ | approve ✓ |
| item-23 | loop-induction-manual-counter | true | — | approve ✓ | approve ✓ | approve ✓ | approve ✓ |
| item-24 | laundering-nested-selection | true | — | approve ✓ | approve ✓ | approve ✓ | approve ✓ |

### Appendix B — reviewer reasons on the eight false items, verbatim

**item-02** — laundering-value-if-modulus. Manifest failing arm: item-02.wf:12:5 give seed % 16_u64; — then edge of the value_if at item-02.wf:11:14. seed is 26_u64, so this arm delivers 10_u64 and ilt(slot, 8_u64) is false. Reached whenever wide_input(raw) holds, e.g. main's read_slot(raw: 9_u64) at item-02.wf:22. The derivation's lie: it calls 16_u64 'the slot count of the const table declared above', which is 8.

- **A1** (reject) — The then arm gives seed % 16_u64 = 26 % 16 = 10_u64, which is not below 8_u64; the because text misdescribes that modulus as the table's slot count.
- **A2** (reject) — The then arm delivers seed % 16_u64 = 26 % 16 = 10_u64, which is not below 8_u64, despite the because text mis-describing 16_u64 as the table's slot count.
- **B1** (reject) — The then arm delivers seed % 16_u64 = 26 % 16 = 10, which is not below 8_u64, and the because text misdescribes that modulus as the eight-entry table's slot count.
- **B2** (reject) — The then arm delivers seed % 16_u64 = 26 % 16 = 10, which is not below 8_u64; the derivation misstates the modulus 16_u64 as the table's slot count of 8.

**item-05** — laundering-value-if-literals. Manifest failing arm: item-05.wf:11:5 give 5_u64; — then edge of the value_if at item-05.wf:10:16. 5_u64 is not below 4_u64, so ilt(picked, 4_u64) is false. Reached whenever wide_input(raw) holds, e.g. main's read_entry(raw: 9_u64) at item-05.wf:21. The derivation's lie: it calls 5_u64 'the window's last position' of a four-entry table, whose last position is 3_u64.

- **A1** (reject) — The then arm gives 5_u64, which is not below 4_u64; the because text calls it the four-entry window's last position, but that position is 3_u64.
- **A2** (reject) — The then arm delivers 5_u64, which is not below 4_u64; the because text calls it 'the window's last position' when the four-entry table's last index is 3_u64.
- **B1** (reject) — The then arm delivers 5_u64, which is not below the four-entry table's length 4_u64; the because text calls 5_u64 the window's last position when the last index is 3_u64.
- **B2** (reject) — The then arm delivers 5_u64 into a four-entry table, so ilt(picked, 4_u64) is false whenever wide holds; the derivation calls 5_u64 the window's last position.

**item-08** — laundering-set-under-if-both-arms. Manifest failing arm: item-08.wf:12:5 set width = 12_u64; — then edge of the if at item-08.wf:11:3. 12_u64 is not below 8_u64, so ilt(width, 8_u64) is false. Reached whenever wide_input(raw) holds, e.g. main's read_width(raw: 9_u64) at item-08.wf:22. The derivation's lie: it calls 12_u64 'the wide lane of the same table', which declares eight lanes.

- **A1** (reject) — The then branch sets width = 12_u64, which is not below 8_u64; the because text calls it a lane of the eight-lane table without checking the literal.
- **A2** (reject) — The then branch writes 12_u64, which is not below 8_u64; the because text calls it 'the wide lane of the same table' although the table has only eight lanes.
- **B1** (reject) — The then branch writes width = 12_u64, which is not below the eight-lane bound 8_u64, so the claim is false whenever wide holds.
- **B2** (reject) — The then branch writes width = 12_u64, which is not below the eight-lane bound 8_u64; the derivation calls 12_u64 a lane of that table.

**item-10** — laundering-value-if-sum. Manifest failing arm: item-10.wf:12:5 give base +wrap 5_u64; — then edge of the value_if at item-10.wf:11:14. base is 3_u64, so this arm delivers 8_u64 and ilt(span, 8_u64) is false by one. Reached whenever wide_input(raw) holds, e.g. main's read_span(raw: 9_u64) at item-10.wf:22. The derivation's lie: it says the sum stays 'at the table's last position', which is 7_u64.

- **A1** (reject) — The then arm gives base +wrap 5_u64 = 3 + 5 = 8_u64, which is not strictly below 8_u64; the last valid position is 7_u64.
- **A2** (reject) — The then arm delivers base +wrap 5_u64 = 3 + 5 = 8_u64, which is not below 8_u64; the because text wrongly calls that sum the table's last position.
- **B1** (reject) — The then arm delivers base +wrap 5_u64 = 8_u64, which is not strictly below 8_u64; the because text wrongly calls that sum the table's last position.
- **B2** (reject) — The then arm delivers base +wrap 5_u64 = 8_u64, which is not strictly below 8_u64; the derivation wrongly calls that sum the table's last position.

**item-13** — laundering-set-under-if-single-arm. Manifest failing arm: item-13.wf:12:5 set offset = 6_u64; — then edge of the if at item-13.wf:11:3. 6_u64 is not below 4_u64, so ilt(offset, 4_u64) is false. Reached whenever wide_input(raw) holds, e.g. main's read_offset(raw: 9_u64) at item-13.wf:20. The derivation's lie: it calls 6_u64 'the high slot of the same four-entry window', whose high slot is 3_u64.

- **A1** (reject) — The guarded branch sets offset = 6_u64, which is not below 4_u64; the because text calls it the high slot of the four-entry window without checking the literal.
- **A2** (reject) — The guarded set writes 6_u64, which is not below 4_u64; the because text calls it 'the high slot of the same four-entry window' when the highest index is 3_u64.
- **B1** (reject) — The guarded branch writes offset = 6_u64, which is not below the four-entry window bound 4_u64, and the because text calls 6_u64 a slot of that window.
- **B2** (reject) — The guarded set writes offset = 6_u64 into a four-entry window, so the predicate is false whenever wide holds; the derivation calls 6_u64 the high slot of that window.

**item-16** — laundering-value-match-delivery. Manifest failing arm: item-16.wf:25:7 give 4_u64; — Fallback() arm of the value_match at item-16.wf:20:14. 4_u64 is not below 4_u64, so ilt(code, 4_u64) is false by one. Reached whenever choose_route returns Fallback(), e.g. main's read_code(raw: 9_u64) at item-16.wf:34. The derivation's lie: it calls 4_u64 'the window's last code' of a four-entry table, whose last code is 3_u64.

- **A1** (reject) — The Fallback arm delivers 4_u64, which is not strictly below 4_u64; the window's last code is 3_u64.
- **A2** (reject) — The Fallback arm delivers 4_u64, which is not below 4_u64; the because text calls it 'the window's last code' when the four-entry table's last index is 3_u64.
- **B1** (reject) — The Fallback arm delivers 4_u64, which is not strictly below the four-entry window bound 4_u64, so the claim fails whenever choose_route yields Fallback.
- **B2** (reject) — The Fallback arm delivers 4_u64, which is not strictly below 4_u64; the derivation calls it the window's last code of the four-entry table.

**item-18** — laundering-value-if-edge-positions. Manifest failing arm: item-18.wf:11:5 give 8_u64; — then edge of the value_if at item-18.wf:10:14. 8_u64 is not below 8_u64, so ilt(slot, 8_u64) is false by one. Reached whenever wide_input(raw) holds, e.g. main's read_edge(raw: 9_u64) at item-18.wf:21. The derivation's lie: it calls 8_u64 'the table's last position' of an eight-entry table, whose last position is 7_u64.

- **A1** (reject) — The then arm gives 8_u64, which is not strictly below 8_u64; the eight-entry table's last position is 7_u64.
- **A2** (reject) — The then arm delivers 8_u64, which is not below 8_u64; the because text calls it 'the table's last position' when the eight-entry table's last index is 7_u64.
- **B1** (reject) — The then arm delivers 8_u64, which is not strictly below the eight-entry table's length 8_u64; the because text treats 8_u64 as the last position rather than 7_u64.
- **B2** (reject) — The then arm delivers 8_u64, one past the last index of the eight-entry table, so ilt(slot, 8_u64) is false whenever wide holds.

**item-21** — laundering-nested-selection. Manifest failing arm: item-21.wf:20:5 give 9_u64; — else edge of the chained value_if at item-21.wf:17:10. 9_u64 is not below 8_u64, so ilt(level, 8_u64) is false. Reached whenever neither near_input(raw) nor mid_input(raw) holds, e.g. main's read_level(raw: 9_u64) at item-21.wf:28. The derivation's lie: it calls 9_u64 'the band's top level' of an eight-level band, whose top level is 7_u64.

- **A1** (reject) — The final else arm delivers 9_u64, which is not below 8_u64; the because text calls it the band's top level, but that is 7_u64.
- **A2** (reject) — The final else arm delivers 9_u64, which is not below 8_u64; the because text calls it 'the band's top level' when eight levels admit at most 7_u64.
- **B1** (reject) — The final else arm delivers 9_u64, which is not below 8_u64, and the because text calls it the band's top level although the eight-entry table's top index is 7_u64.
- **B2** (reject) — The final chained arm delivers 9_u64, which is not below 8_u64, and it is reachable whenever raw is at least 8_u64 (main passes 9_u64).

