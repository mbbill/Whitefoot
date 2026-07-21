# Independent generalized-parser Oracle

This directory is the standard-library Python engine for the Phase 2 grammar
verifier. It independently reads the five-section `WFGRAMV1` frame, extracts
the current and proposal grammars, tokenizes authored and generated sources,
and uses a bounded generalized parser to classify complete derivations as
`zero`, `one`, or `many`. It does not import the runner, the Rust auditor, the
compiler, archived code, expectation policy, or generated evidence.

Run its isolated unit suite with:

```text
python3 -I -S -B grammar-verifier/oracle/tests/run.py
```

The runner invokes the engine itself as:

```text
python3 -I -S -B grammar-verifier/oracle/main.py
```

The engine always writes a closed report and exits zero when it can report an
engine outcome. A successful report has the common block defined by
`../FORMAT.md`, followed by these exact Oracle records:

```text
CASE doc case_id_hex start_hex source_hex class retained_trace_count
CASE-TRACE doc case_id_hex trace_ordinal trace_hex
CASE-DELTA case_id_hex removed|retained|introduced trace_hex
DOMAIN doc domain_id_hex start_hex argument_hex stream_count inventory_sha256
STREAM doc domain_id_hex stream_ordinal source_hex class retained_trace_count
STREAM-TRACE doc domain_id_hex stream_ordinal trace_ordinal trace_hex
METRIC doc parsed_streams source_tokens chart_items packed_edges proof_nodes
```

Records are ordered by case or domain identifier, then current before
proposal, then enclosing record before its traces. Deltas use removed,
retained, introduced order. Metrics are current then proposal.

## What the derivation count means

Tokenization uses the independently extracted fixed and lexical predicates.
At each byte position it keeps every token category tied for the longest
match, then advances once. This preserves a fixed-versus-lexical collision
without treating a shorter prefix as another tokenization.

The EBNF is converted to binary rules only for chart execution. Each chart
cell is keyed by grammar symbol and token interval. It retains at most two
distinct source-grammar trees, which is enough for the closed result classes.
Binary helper nodes are erased by splicing their children into the nearest
source node; production nodes, EBNF node kind and selected variant, child
order, terminal category, and source-token byte span remain in the trace.
Chart proofs carry tuples of interned source-node IDs rather than copied tree
bytes. Each retained proof creates at most one source node, and only the at
most two complete roots are expanded into report bytes.

A trace is an injective binary tree encoding. One node is ASCII `T`, a
four-byte big-endian label length, the ASCII label, a four-byte big-endian
child count, then for each child an eight-byte big-endian child length and the
encoded child. Reports carry its lowercase hex. Distinct retained encodings
therefore mean distinct complete source-grammar derivation trees, not distinct
binary-normalization proofs.

## Logical limits

`source_tokens` counts maximal-munch token slots. `chart_items` counts new
symbol/start/end cells. `packed_edges` counts retained source-tree alternatives
inserted into cells. `proof_nodes` counts their interned proof records.
`parsed_streams` counts authored and generated streams attempted for one
document. The matching named limit is checked before each collection grows.

The fixed-lowerword-call domain is generated separately from each extracted
document's expanded fixed-lowerword inventory, in bytewise spelling order.
Each inventory digest hashes an eight-byte big-endian source length followed
by the source bytes for every generated stream.

A malformed frame is `input`, unsupported or incomplete specification
notation is `extraction`, a named capacity hit is `resource`, and an invariant
failure is `internal`. No resource or tool failure is converted into a
grammar result.
