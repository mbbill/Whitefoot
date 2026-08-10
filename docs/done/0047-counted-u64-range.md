# 0047 — counted `u64` range loop

This is frozen coordination history, not execution authority.

- **Status:** `DONE` (2026-08-09)
- **Authority:** the ACTIVE stage-6 plan derived from Direction Outline
  revision 25, the owner's exact approval of v0.25 digest
  `c0b3c279f4c20d06da17ef7ac0e4ec882c8a76c560f62cce47d5b4fd4ac6beab`,
  and the named protected prose change
- **Owner / workspace:** Codex lead / `/Users/bytedance/code/Whitefoot`, branch
  `codex/0047-counted-range-impl`
- **Base revision:** `8a44cb111759af87846284adbab2060b199fc861`

## Outcome

v0.25 is active at the stable `spec/kernel-spec.md` path and v0.24 is frozen at
`spec/kernel-spec-v0.24.md`. The language and ordinary compiler path now have
one ascending, unit-stride, half-open counted range over once-captured
term-or-constant `own u64` endpoints. Its compiler-updated binder is body-local
and source-immutable; labelled exits, cleanup, zero-trip behavior, and the
maximum-u64 edge are represented directly rather than desugared.

S11 supplies only `lower_capture <= binder < upper_capture` on a true body
entry. Existing closure, support, kills, and S7 discharge derived constant
offsets; ordinary loops gain no induction and no counted postcondition escapes.
The real SHA-256 worker uses exactly three counted ranges, deletes four claims,
proves all 9/9 schedule subscripts, becomes `pure`, emits no `wf_trap`, and
retains its direct and sustained runtime oracles.

## Landed work

- `3e2e823` — one atomic v0.25 activation: exact stable bytes and v0.24
  archive, seventeenth approval-chain link, general compiler implementation,
  generated grammar, active pins, derivation, writer documentation, approved
  protected prose plus three additive cases, SHA migration, and live authority.
- This closure change — installed frozen evidence, Direction Outline revision
  27, the ACTIVE stage-7 replacement plan, and this move from `docs/ongoing/`
  to `docs/done/`.

## Canonical evidence

- `spec/kernel-spec.md`, `governance/APPROVALS.md`, and
  `spec/derivation/derivation-ledger.md` own the exact language identity,
  approval boundary, and derivation.
- `research/investigations/obligation-discharge/ACCEPTANCE.md` owns the
  candidate and installed frozen buckets.
- `compiler/README.md`, the counted semantic/lowering/backend tests, and
  `compiler/tests/programs/hashing.rs` own compiler capability and executable
  evidence.
- Direction Outline revision 27 owns current status and the replacement Current
  Plan owns subsequent sequencing.

## Validation

- Installed SHA-256 identities are
  `c0b3c279f4c20d06da17ef7ac0e4ec882c8a76c560f62cce47d5b4fd4ac6beab`
  for v0.25 and
  `53495b9c47b92942876c90931d0296c752855954564ebf7435a549c48cb2dc86`
  for the byte-identical outgoing v0.24 archive. No v0.25 archive exists.
- `whitefoot-spec` reports v0.25, 128 rules, and 17 unbroken activations;
  archive integrity reports 26 recorded identities. Native grammar is 70
  productions, 85 decisions, and 96 terminal predicates, with committed tables
  matching exactly.
- `make -C compiler check` and `make check` pass from the activation tree:
  620 library tests, 30 real-program tests, 23 conformance-tool tests, 128/128
  rule coverage, canonical corpus, clippy, formatting, rustdoc, and repository
  invariants are green. Focused counted tests are 37/37.
- The separately invoked complete adapter reports `Pass=393 Fail=1 Skip=13`.
  Its sole divergence remains `own3-pos-outlives-store` as the pre-existing
  `RegionsAndBorrows` unsupported boundary; all three additive counted cases
  and the rederived GRAM-6 case pass.
- Installed frozen acceptance is UTF-8 `33/22/11/0`, SHA `9/9/0/0`, deflate
  `29/11/18/0`, and dynamic deflate `24/11/13/0` in
  total/proven/claim-supported/baseline-undischarged order. No previously
  proven site regresses; deflate remains 16 retained, five redundant, and zero
  refuted claims.

## Follow-up

Direction Outline revision 27 and the replacement ACTIVE plan select stage 7,
`requires` as one atomic call-site goal. Task 0048 is not yet registered; its
registration must be a separate lifecycle commit based on this closure before
substantive work begins. Provenance activation, postconditions, ledger,
deny-claims, and wfgrep remain later dependencies.
