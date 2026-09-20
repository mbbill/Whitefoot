# Records data-placement provenance (disposable)

This manual-only research branch observes the result pointers returned by the
exact baseline and candidate `records` images archived by compute run
35527433610. It does not rebuild or relink them, and it does not perform a
performance experiment. Runner timing rows are an unavoidable by-product of
reaching the six performance-shaped allocations and are discarded.

The debugger stops at `wf_record_result_release`, which is called by
`wf_oracle_check` after both clocks have stopped. On x86-64 the incoming
`rdi` is the allocation released. It is also the baseline payload; the
candidate payload begins eight bytes later.
The first word is also read without mutation; every candidate observation must
hold the 131,072-element length there.

The fixed observation is one fresh process for each combination of exact image
and W1/W2/W4, six processes total. Each must make six release calls. ASLR is
left enabled with `set disable-randomization off`. No inferior memory or
register is written and no inferior function is called.

All six observations are archived. Ordinal 0 is the runner's discarded warmup
and is reported separately because the first large allocation/free can change
glibc's later mmap policy. The result supports a later controlled experiment
only when the five retained-sample payloads across all three widths have one
baseline residue, have one candidate residue, and the candidate residue is
exactly baseline plus eight modulo 64. Otherwise it stops as inconclusive. A
zero-aligned residue is never substituted or selected from a mixed sample.

This branch and its workflow are deleted after the one-shot investigation.
