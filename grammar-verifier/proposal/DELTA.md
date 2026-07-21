# Successor candidate delta

Status: non-authoritative Phase 2 evidence. Installing any numbered successor
requires separate owner approval and the guarded procedure in
`governance/APPROVALS.md`.

Current input:

- path: `spec/kernel-spec-v0.8.md`
- SHA-256: `d04336f7fa8d1a6a0f03fe58a17f972b658217a73a3dff91a906b4ba295328a8`

Candidate input:

- path: `grammar-verifier/proposal/kernel-spec-successor-candidate.md`
- SHA-256: `cfd76a2bf9293519623c2448280f4d6f76f32be26cc1b2dadc487415e063f166`

The candidate has the required successor version header and exactly two
normative changes.

## Successor version header

The title becomes `Kernel Specification v0.9`. The status paragraph summarizes
the two changes and the former v0.8 status paragraph becomes explicit prior
history. The candidate remains non-authoritative because it is outside the
numbered `spec/` surface; its contents are the exact bytes proposed for
installation, so approval would not require an unreviewed header rewrite.

## FORM-3 terminal partition

After the exact IDENT regular expression, add:

```text
 excluding every lowercase token spelling produced by exact fixed grammar atoms in the complete grammar
```

The exclusion is mechanically derived from the complete grammar, not an
authored keyword subset. It removes every token-expanded fixed lowerword from
IDENT membership while changing no fixed token or source spelling. The source
grammar has 47 distinct quoted lowercase fixed atoms; token expansion also
contributes `uniq` from `&uniq`, producing 48 excluded IDENT spellings.

Selection ground: evidence-selected. Exact v0.8 lets all 47 quoted lowercase
fixed atoms match IDENT and gives `deref(p)` and `deref(x)` both fixed-place and
call-through-IDENT derivations. The two-engine report must show that the
candidate removes only the call-through-IDENT derivation and introduces no
unexpected terminal or predictive conflict.

## FN-1 top-level signature visibility

Append these owner-authorized sentences to FN-1:

```text
All top-level function signatures are visible throughout the closed compilation unit. Locals, regions, labels, and explicitly earlier constants still follow lexical declaration-before-use.
```

Selection ground: owner-selected resolution A-01, authorized in session on
2026-07-21. It reconciles FN-1/FN-6 mutual recursion with TYPE-6 without making
local or constant lookup traversal-order dependent. The existing protected
mutual-recursion case already expects this behavior; no verdict change is
proposed.

## Declared specification delta

- rules added: 0
- rules removed: 0
- grammar productions added or removed: 0
- fixed tokens added or removed: 0
- source spellings added or removed: 0
- lexical membership changes: IDENT excludes all 48 token-expanded fixed
  lowercase spellings
- name-visibility changes: top-level function signatures become closed-unit
  visible; locals, regions, labels, and constants remain lexical
- exception clauses added: 0

The protected-surface impact is enumerated in
`protected-surface-census.json`. The grammar verifier's current/proposal raw
reports and combined evidence package are the executable support for this
delta; this prose is not a substitute for them.
