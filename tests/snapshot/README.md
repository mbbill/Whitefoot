# Remaining recorded-verdict cases

This directory holds the cases not yet assessed during the approved test-system
redesign. It accepts no new cases. The active specification decides the language;
the old `verdict` and `finder_expectation` columns are historical observations,
not acceptance authority.

Each source is reviewed for its actual observation and normative expectation.
Useful language cases move to `tests/conformance`; real program behavior belongs
in `tests/programs`; additional implementation observations belong in compiler
or runtime tests. Duplicate or misleading examples are retired with a specific
receiver or technical reason in the [case disposition](../../research/investigations/test-economy/snapshot-disposition.md).
A wrong expectation is not a permanent regression standard.

Until this review finishes, `make snapshot-run` still compiles every remaining
`index.tsv` row through the ordinary compiler, without native linking/execution,
and compares the recorded accept/reject result. The seven columns remain
`id`, `family`, `verdict`, `rule`, `finder_expectation`, `agreement`, and `doc`;
`cases/<family>/<id>.wf` is the source. The old runner and this directory are
removed when every source has a disposition. New normative verdicts are tested
by the conformance adapter, which also checks the selected rejection rule.
