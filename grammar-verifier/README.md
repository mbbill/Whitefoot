# Grammar-change verifier

This directory contains the standalone Phase 2 evidence tool. It is outside
the production compiler workspace, and normal compilation neither links nor
invokes it.

The tool compares exact v0.8 bytes with a full, non-authoritative successor
candidate. Two engines independently extract the specification grammar:

- `static-auditor/` is safe Rust. It computes terminal intersections,
  nullable, `FIRST_2`, `FOLLOW_2`, exact strong-LL(2) decision collisions, and
  concrete witnesses.
- `oracle/` is Python standard-library code. It uses its own extractor and a
  bounded generalized parser to count complete source-level derivations.

The engines share only the exact framed inputs and resource limits. They do
not share extraction code, grammar objects, tokenization, lookahead tables,
generated streams, expected results, or implementation modules. Their common
source-coverage ledgers are compared byte for byte by a small runner that does
not parse EBNF or compute language facts.

## Checked result

The committed evidence binds exact v0.8 SHA-256
`d04336f7fa8d1a6a0f03fe58a17f972b658217a73a3dff91a906b4ba295328a8`
and candidate SHA-256
`cfd76a2bf9293519623c2448280f4d6f76f32be26cc1b2dadc487415e063f166`.
Both engines emit the same 134,019-byte common extraction ledger, SHA-256
`2014897a6d2a4599957bad140f0de73c0d42c559ec629a3fdc20fe0b4d238b27`.

For both `deref(p)` and the exact transition witness `deref(x)`, v0.8 has two
complete derivations and the candidate has one: one fixed-place derivation is
retained, one call-through-IDENT derivation is removed, and none is introduced.
The static transition count is exactly one to zero for the complete `deref(x)`
source. Both engines independently generate the same 48-stream fixed-lowerword
domain, SHA-256
`f3e54408ce7c4234bb3b61e27f2decd6c84ffcc4d7fb1b201c9583dd0190480c`,
with no introduced terminal intersection or predictive conflict.

The proposal remains non-authoritative. Phase 3 requires exact owner review and
advance approval before any numbered specification or protected surface moves.

## Trust boundary

This is a development evidence tool, not production compiler authority. Its
claim is deliberately limited:

> On a clean, non-adversarial checkout, the declared OS, filesystem, process
> controls, Python 3 runtime and standard library, pinned Rust/Cargo toolchain,
> and trusted host linker/system tools are trusted to run the bound source
> revisions over the bound input bytes under the reported limits.

Source manifests are stable-tree observations. They are not attestation of
already-executed Python bytes or proof that source hashes equal mapped machine
code. Unexpected OOM, signal, timeout, process failure, or malformed output is
tool-inconclusive and can never prove a grammar conflict absent or a case
unambiguous.

Run the verifier gate with:

```text
make -C grammar-verifier check
```

The root `make check` includes this gate. `FORMAT.md` is the complete wire,
ledger, ordering, binding, and failure contract.

The checked-in proposal and evidence are review material only. They do not
amend a numbered specification, modify a protected expectation, switch the
active language target, or authorize a production parser.
