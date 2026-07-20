# Whitefoot v0.8 lexical fixtures

`lexical-fixtures.json` is a small, hand-authored, implementation-independent
set of byte probes for the first lexical partition boundary. Source payloads
and observed spans are hexadecimal so raw bytes never pass through text-file
decoding or newline conversion.

The corpus is bound to specification SHA-256
`d04336f7fa8d1a6a0f03fe58a17f972b658217a73a3dff91a906b4ba295328a8`
and static catalog SHA-256
`2fa586a8a1d9a49f344d64ad2b5f450a2ae2e8362bc187c70267097b9b427e1d`.
Its three outcomes describe only lexical partitioning, source-local byte
issues, and configured resource ceilings. The fixtures do not decide parsing,
canonical layout, semantic meaning, or diagnostic presentation.

Each complete piece is `[source ordinal, start, end, kind, exact hex]`; spans
are zero-based and half-open. Source ranges index the flat piece list. Every
limit is inclusive, and a case-specific `limits` object overrides only the
named default.

The independent model and corpus checks run with:

```sh
python3 -B tools/test_v08_lexical_model.py
```

This checks the reference model against the authored observations. It is not a
Rust-compiler differential or an implementation-capability receipt; that
requires a separately reviewed observation adapter.
