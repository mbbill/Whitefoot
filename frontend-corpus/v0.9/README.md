# Whitefoot v0.9 lexical fixtures

`lexical-fixtures.json` is a small, hand-authored, implementation-independent
set of raw-byte probes for the lexical partition boundary. It is bound to
specification SHA-256
`bdfb461d1901f610633c5cbcd2477d24df3c77ca90599b9580c8289e50b82b68`
and static-catalog SHA-256
`3ff82e48fc860c4a414e8e1a16a652426b7505d7b74beedf057e418533151aae`.

Each numbered lexical corpus and model is a frozen independent snapshot. A new
numbered snapshot is created deliberately and is never kept in sync by copying
later edits back into an older version. The v0.8 and v0.9 source payloads and
expected partitions are similar because the approved v0.9 change did not alter
the raw lexical contract. That similarity is evidence of an unchanged boundary,
not shared mutable authority. Older numbered model snapshots remain present and
are still executed so later changes cannot silently rewrite their evidence.

Source payloads and observed spans are hexadecimal, so raw bytes never pass
through text decoding or newline conversion. Each complete piece is `[source
ordinal, start, end, kind, exact hex]`, with zero-based half-open spans. A
case-specific `limits` object overrides only the named inclusive default. The
three outcomes describe lexical partitioning, source-local byte issues, and
resource ceilings; they do not decide parsing, canonical layout, semantics, or
diagnostic presentation.

Run both immutable snapshots with:

```sh
python3 -B tools/test_v08_lexical_model.py
python3 -B tools/test_v09_lexical_model.py
```

The active Rust/model observer differential is likewise version-bound and
non-authorizing:

```sh
python3 -B tools/test_v09_lexical_observer.py
```

The v0.8 live-observer receipt and tool surface remain frozen and
hash-validated historical evidence. They are not run against the active
compiler, whose observer accepts only exact-v0.9 requests. Only the v0.9 live
differential executes against that binary.
