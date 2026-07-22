# Independent analytic resource route

Status: NON-AUTHORITATIVE MEASUREMENT IMPLEMENTATION. This directory selects
no ResourceProfile maximum and grants no compiler or language authority.

This is the second ResourceProfile-v1 measurement route. It consumes a closed
workload-construction manifest plus the separately supplied source bytes. It
checks only each source's byte length and SHA-256. It never lexes, parses,
classifies, or otherwise inspects those bytes.

The neutral manifest has exactly these fields. Its canonical JSON encoder uses
the byte order pinned and tested in `manifest.py`:

1. `schema` = `whitefoot-resource-workload-v1`;
2. `family` = `compiler` or `codec`;
3. `units` = the number of repeated construction units;
4. `generator_revision` = the selected generator contract's SHA-256;
5. `parameters` = an ordered record array containing exactly
   `name_decimal_width`, `source_records`, and `unit_count`; and
6. `sources` = ordered `{logical_path, byte_length, sha256}` identities.

It contains no production name, grammar role, expected count, work total,
diagnostic, formula, or syntax fact. Unknown, reordered, duplicated,
non-canonical, out-of-range, and impossible values fail closed.

`relation.py` independently expands those neutral parameters into private
relations. Its explicitly named aggregate token-length and production-parent
witnesses preserve only count, byte-sum, peak, and depth facts; they are not
detailed token, derivation, parser, or work traces. `selection.py` independently
checks the closed families in FN-8, declaration-inventory, then lexical-use
order, including exact PRE-1 domains and enum-constructor ownership. It derives
`Complete` rather than accepting it as a fixture. `measure.py` folds the
construction-derived relations into the profile
actuals it can establish, checked derived counts, a closed trace-gap ledger,
and the expected diagnostic. Neither module imports
the source route, a grammar table or parser, the proposal's diagnostic models,
production compiler crates, shared role tables, shared formulas, expected
counts, or archived material.

`receipt.py` encodes one canonical binary receipt with top-level status
`trace-incomplete`. It binds the exact approved proposal, candidate
specification, profile-semantics, work-schedule, and storage-model identities,
plus a domain-separated digest over the exact ordered closed non-test runtime
code file set, and embeds the complete canonical manifest bytes. Strict decode
recomputes the sealed code identity, redecodes that manifest, recomputes its
digest, ordered source identities, bundle identity,
private relation, selection result, counts, derived receipts, and trace gaps,
then requires exact equality before returning. It also binds
ordered source identities, analytic source-bundle identity, every available
actual, every unavailable-field marker, derived counts, trace gaps, and the
diagnostic. Decode followed by encode must reproduce identical bytes.

The present baseline deliberately marks fields 9, 14 through 17, and 33
unavailable. Exact scanner comparisons/probes, parser stack and repeat events,
diagnostic expected-set construction, cumulative syntax actions, and the full
resolution sort/query/origin/materialization schedule require a detailed
independent action replay. Aggregate formulas are not substitutes. Calling
`require_complete` therefore fails closed; this receipt cannot size a hard
profile until those trace gaps are closed. The CLI's `--selection-mode` calls
that gate and publishes nothing while any gap remains.

## Honest claim boundary

This route can prove that its private arithmetic follows its selected
construction contract. It cannot infer that the separately supplied bytes are
valid UTF-8 Whitefoot, canonical FORM-2, accepted by FN-8, or shaped like the
private relation. Those are grammar facts, and no topology/name parameter can
honestly establish them. The independent source-to-role route must derive them
from the same bytes and agree with this route. A disagreement invalidates the
workload evidence; it is never resolved by trusting this route.

Run the isolated tests with:

```text
python3 -m unittest discover -s optimizer-language-research/implementation/phase5-resource-profile/analytic-route -p 'test_*.py'
```

The independence harness invokes `run.py` in a separate process:

```text
python3 run.py --manifest workload.json --source-file source.wf --output receipt.bin --summary-output summary.json
```

`--source-file` may repeat and preserves argument order. Manifest and source
reads require regular nonsymlink files and are capped at 1 MiB per manifest,
64 source files, 64 MiB per source, and 128 MiB total source bytes. Receipt
and optional summary publication are each capped at 8 MiB and use atomic
same-directory replacement. The canonical summary is projected only by
strictly decoding the exact receipt bytes. It records ordered field
availability/value records, derived counts, trace gaps, expected diagnostic,
manifest workload family, units, generator revision, ordered source identity
records, and manifest, source, bundle, code, and receipt digests.
These finite values are operational evidence-tool safeguards only. They are
not proposed ResourceProfile maxima, supported-program limits, workload-demand
measurements, or language/compiler claims.

`dependency_audit.py` seals the route's exact file set and Python import
closure, rejects dynamic/relative/external imports and non-regular entries,
and scans runtime modules for forbidden cross-route, producer, compiler,
archive, and proposal-model dependencies.
