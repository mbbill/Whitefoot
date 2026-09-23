Decision: Bool and user tag-only enums with at most two variants lower to one bit and larger tag-only enums to 32 bits across values, calls, equality and match, because equivalent two-state forms should expose the same narrow dataflow to vectorization; the [word-count study](https://github.com/mbbill/Whitefoot/blob/b3341e32ab8d69e2974227aee7146ac16eb2fe00/research/experiments/port-study/wc-chunk-summary/RESULTS.md#scope-of-the-tag-width-evidence) supports that motivation without isolating a tag-width-only speedup, instead of word-sized tags for every enum.

Rejected:
- Word-sized tags for Bool and equivalent two-state enums: rejected because they impose wide recurrence state where the same choice has a one-bit representation.
