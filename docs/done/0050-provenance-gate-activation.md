# 0050 — activate the bounded provenance gate

- **Status:** `DONE`
- **Authority:** the owner-approved Stage 5b plan derived from Direction
  Outline revision 30 item `PROOF-8`, with `BOUND-1`, `VERIFY-1`, and
  `VERIFY-2` as boundary and evidence constraints
- **Owner / workspace:** Codex lead / `/Users/bytedance/code/Whitefoot`, branch
  `codex/0047-counted-range-impl`
- **Base revision:**
  `63e3407b997cce0716266ce6d7f6dc6039df92ab`

## Outcome

Exact-approved v0.27 is active at the stable specification path. The ordinary
safe-Rust semantic path now implements the bounded PRV-1/PRV-2/PRV-3
explicit-dataflow gate, deterministic protected-leaf witnesses, subject-only
gating, call bridges, and the S4-blinded process-entry rewalk. Internal
subjects retain ordinary entailment; an external protected subject must
discharge without S2/S3 assertion support. The selected rule adds no implicit
flow, write-address taint, path sensitivity, recursive payload paths, Boolean
decomposition, general induction, or second goal language.

The four-source raw-DEFLATE unit replaces exactly eleven claims with the
owner-selected value branches, changes `store_dynamic_length` to
`Result<unit, InflateError>` with exactly three propagations, and removes only
the now-unexhibited `traps` category from `store_dynamic_length`,
`decode_length`, `copy_distance`, and `decode_fixed`. All other effect rows,
error mappings, cleanup behavior, and runtime oracles are preserved. Sixteen
PRV conformance cases were added without changing any of the 407 prior cases,
their manifest rows, or the 30 prior coverage annotations.

## Landed commits

- `ecfa57eff92cdadbd6b547b47e7f3677b0f12089` registered this task before
  substantive Stage 5b work.
- `5ab45aa73a1a713e994773d2c04c34400795950a` atomically activates v0.27,
  archives the byte-identical outgoing v0.26 bytes, installs the compiler and
  consumer changes, and records the exact owner approval and derived evidence.
- `7451230944524b03f6b95900b46e129e9dab809e` makes the live design memory
  truthful to the installed provenance gate.
- This terminal closure change installs the final acceptance record, advances
  the Direction Outline, replaces the completed ACTIVE plan with the Stage 8a
  proposal, and moves task 0050 to frozen history.

## Canonical evidence

- `spec/kernel-spec.md`, `spec/kernel-spec-v0.26.md`,
  `governance/APPROVALS.md`, and `spec/derivation/derivation-ledger.md` own the
  active language identity, outgoing archive, approval chain, and derivation.
- `research/investigations/obligation-discharge/ACCEPTANCE.md` owns the exact
  candidate and installed frozen buckets, source and manifest identities,
  claim dispositions, and runtime controls.
- `compiler/README.md`, focused provenance/requirement/entailment/entry tests,
  and the real-program suite own the compiler and executable evidence.
- `mcts_mem/whitefoot/checks-and-proofs/obligation-discharge.md` and
  `mcts_mem/whitefoot/checks-and-proofs/requires-entry-contract.md` own the
  live bounded-provenance decision and its requirement bridge.
- Direction Outline revision 31 owns the terminal landscape. The successor
  Current Plan is a proposal awaiting separate owner selection.

## Validation

- Installed specification SHA-256 is
  `bbd7250084123bbce3267f741f30f6c12efc73c341ff8d361dd1b19d9502090f`;
  the byte-identical outgoing-v0.26 archive is
  `18aa00e307642e608f2a3406642db9980dd3620291a7e434985e20a65eb0e476`.
- The four frozen raw-DEFLATE source identities, in compilation order, are
  `raw_deflate.wf`
  `5e87885a519de539736b0ad6a619a2c92bdf659623e3f39ded31116e63adb585`,
  `raw_deflate_dynamic.wf`
  `2606ae0b81039b0ecc787df2fa9cb87279ba44663744540d48f6abeea984d4c5`,
  `raw_deflate_dynamic_decode.wf`
  `72129284c60a6eacbe2bb86d7d3a82375ed2270ea457c49d8e1db95064fc960f`,
  and `raw_deflate_boundary.wf`
  `3fbd1281b1e9f4f9a161cf7d846622ae277611eaf9d34ce3ba576f3a81d140c4`;
  the installed conformance manifest is
  `04d2562f41eecbd3af5770c96ccad9a4fcfa8cd9f9d849c414f1cccbb89d072d`.
- Installed frozen acceptance is UTF-8 `33/22/11/0`, SHA-256 `9/9/0/0`,
  complete DEFLATE `29/24/5/0`, and dynamic DEFLATE `24/19/5/0` in
  total/proven/claim-supported/baseline-undischarged order. The unit retains
  twelve claims: seven load-bearing, five redundant, and zero refuted; all
  thirteen migrated sites discharge through real branches.
- Focused provenance passes 41/41 and the raw-DEFLATE runtime oracle passes
  3/3. The complete `make check` gate is green with 698/698 library tests,
  30/30 real-program tests, 131/131 rule coverage, and 19 unbroken activation
  entries.
- The separately invoked adapter reports `Pass=409 Fail=1 Skip=13`; its sole
  failure remains the pre-existing OWN-3 `RegionsAndBorrows` unsupported
  boundary. MCTS lint passes after the live-memory update.

## Follow-up

Stage 8a awaits separate owner selection and has no registered task number or
execution authority. No successor task is claimed by this closure. Further
wfgrep work remains parked until the complete owner-selected PROOF-8 sequence
is terminal.

This file is frozen coordination history. It reports how the owner-approved
work was carried out; it is not current authority, a second roadmap, or a
replacement for the canonical evidence linked above.
