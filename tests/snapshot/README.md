# The snapshot corpus

This corpus records what the current compiler does with 491 programs — a
snapshot of today's verdicts, nothing more. It is **not specification
evidence** and it lives **outside the conformance boundary**: no row here
states what the language requires, no row may be cited as a rule, and the
integrity obligations that govern `tests/conformance/` do not reach it. The
active specification at `spec/kernel-spec.md` decides what is correct; this
directory only remembers what happened.

Its job is catching regressions cheaply. The programs came from a 665-program
adversarial and writer-authored sweep across twelve language areas, which found
two real compiler defects. The cheapest way to keep that reach is to compile
the survivors on every gate run and notice when one of them changes verdict.

Only sweep programs whose author stated an expectation are here; a program with
no stated expectation records a verdict nobody predicted, which is not worth a
gate row. That excludes the sweep's known unsound accept, which is tracked
where a language claim belongs — the conformance xfail case
`ent5-neg-callee-uniq-buffer-replace-kills-length`.

## What is here

- `index.tsv` — the single source of truth: one row per case, plus header
  comments describing the columns.
- `cases/<family>/<id>.wf` — the programs, one file per row. Every case is
  canonical Whitefoot source that reaches a *semantic* verdict: an accept, or a
  rejection from resolution or semantic checking. A program that cannot get
  past lexing, parsing, or the [FORM-2] canonical-source audit records nothing
  about the checker and does not belong here.

## The columns

| column | meaning |
| --- | --- |
| `id` | case name; the file is `cases/<family>/<id>.wf` |
| `family` | the sweep area the program came from |
| `verdict` | `accept` or `reject` — the only thing the gate compares |
| `rule` | the numbered rule the rejection cites; empty for an accept |
| `finder_expectation` | `accept` or `reject`, what the program's author expected |
| `agreement` | `yes` or `no`, whether the two above agree |
| `doc` | one line on what the case is; for a disagreement, what the author expected, what the compiler does, and why |

## The gate compares ACCEPT/REJECT only

`make snapshot-run` compiles each case in index order through the ordinary
compiler path — no linking, no execution — and compares `verdict` alone. The
`rule` column is reference only and is **never checked**. Which numbered rule a
rejection cites is a diagnostic choice, and pinning it here would turn every
diagnostic improvement into a corpus failure. If you want a cited rule pinned,
that is a conformance case, not a snapshot row.

Thirteen rows carry `agreement = no`: the author expected an accept and this
compiler rejects. Each row's `doc` gives the rule that decides it and the
mechanism behind it — twelve are expectation errors, and the thirteenth
(`kills__writer-r2__06_chain_middle_replace`) is a program whose stated
expectation the compiler now meets in the part it was written to exercise, yet
which still rejects for an unrelated unproved product. They are kept because a
wrong expectation is still a fixed verdict worth watching.

## When a verdict flips

A flip is information, not a failure to route around. The driver names the
case, the recorded verdict, the reached verdict, and the first diagnostic line.

- If the flip is a compiler defect you just introduced, fix the compiler.
- If the flip is a **known defect being fixed**, it is welcome: update the row
  and say so. That is the corpus doing its job.
- Either way, the commit that changes a `verdict` cell must explain the flip in
  its message. A silently updated row is the one thing this corpus cannot
  survive.

Never delete a case or edit a verdict merely to get a green run.

## The case sources

Case files are the sweep programs unchanged, except that each carries a leading
`doc` statement that is true of the file. Thirteen of them — the disagreeing
rows — had a `doc` asserting an expectation this compiler does not meet; those
lines now state the snapshot verdict and what the author expected instead. The
programs themselves were not touched.

## When to remove this directory

Delete it, wiring and all, when either becomes true:

- `tests/conformance/` covers these shapes, in which case the specification
  says what this corpus only remembers; or
- the corpus stops paying for itself — flips that are noise rather than
  signal, or a maintenance cost larger than the regressions it catches.

It is a snapshot, not an asset. Removing it needs no ceremony beyond the
commit message that explains why.
