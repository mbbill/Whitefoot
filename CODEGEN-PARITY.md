# Codegen-parity gate

This gate answers a narrower question than a benchmark: did a compiler change
silently lose a code-generation property that the project has already earned?
It recompiles every input in a temporary directory and checks optimized LLVM
IR, loop-vectorizer remarks, and per-function machine opcode sequences.

Run it with:

```sh
make parity
python3 tools/codegen_parity.py --corpus --promotion
python3 tools/codegen_parity.py --gate-only
python3 tools/codegen_parity.py --audit-only
python3 tools/codegen_parity.py --case scalar-backend-parity
python3 tools/codegen_parity.py --corpus --tag proof-1
python3 tools/codegen_parity.py --json
```

The cases live in `codegen-parity.json`. A `gate` case makes the command fail;
an `audit` case reports `DEBT` but exits successfully. Promotion from audit to
gate happens only after the target property is implemented, independently
verified, and expected to remain true. This keeps known debt visible without
encoding today's bad codegen as tomorrow's contract.

Case maturity and the checked-automation bounds-v1 build subgate are
intentionally different.
A negative corpus case is a green regression gate when the compiler reports
its expected hard finding; it is not a promoted build root. Subgate evaluation
occurs only for roots named by the top-level `checked_automation` policy and
duplicated in the harness's review-pinned root set. `make parity` uses
`--promotion`, which requires `--corpus` plus the default manifest, forbids
case/tag/mode filters, verifies the root's source/function/SHA-256 identity,
and asserts that every pinned root ran. Focused commands remain useful
diagnostics but are not subgate evidence. The duplicate pins prevent a
manifest-only edit; coordinated changes to both repository pins still require
protected external owner review. The repository itself does not establish that
GATE-1 authority.

`--corpus` also discovers compact family manifests under `codegen-corpus/cases`.
Each case's explicit maturity selects gate or audit behavior. Proof
classification uses the compiler's structured per-site report for the named
function so LLVM's independent check elimination cannot create a false pass.
Optional corpus `checked_automation_*` expectations are policy oracles over the
facts-on report; they do not promote those cases.

The initial coverage is deliberately small and high-signal:

- exact backend opcode parity for one whitefoot/C/safe-Rust scalar kernel;
- the facts-on/off load-elimination, scoped-alias, and checked-law channels;
- vector-width and trap parity on the real wc chunk classifier;
- exact local proof accounting on the real base64 encoder;
- bounds-v1 checked-automation evaluation of the complete base64 compilation
  unit (27 automatically accounted sites, zero findings);
- the base64 perfect-prover ceiling as non-blocking bounds-elision debt.

This is not a runtime-performance gate. Runtime measurements remain in their
self-contained experiment directories because frequency scaling, corpus state,
and scheduler noise make them unsuitable for an every-change invariant. Add a
runtime gate later only for a stable, dedicated benchmark host.

## Manifest vocabulary

Each variant names a `kind` (`whitefoot`, `c`, or `rust`), source, optimization
level, and optional function. Whitefoot variants may disable the fact bundle with
`"facts": false`; `"elide_bounds": true` is reserved for the explicitly
labelled ceiling audit. Checks compare `variant.metric` to a literal or another
variant. Supported operators are `eq`, `ne`, `lt`, `le`, `gt`, and `ge`.

The checked-automation policy currently has one schema-versioned scope,
`frontend-implicit-bounds-v1`, and one registered obligation analyzer,
`output-capacity-lockstep-v1`. Each report site records the exact analyzer set
that completed; missing, duplicate, unknown, or inherited provenance fails
closed. The facts-on disposition matrix is:

| Site result | Disposition | Bounds-v1 subgate |
|---|---|---|
| Valid frontend proof | `automatically-accounted` | pass |
| Retained and affirmatively outside every registered family | `intrinsic-dynamic` | pass |
| Derived obligation with missing/mismatched requirement | `hard-finding` | fail |
| Failed premise, unknown/incomplete state, ceiling, or matched-but-retained site | `unaccounted` | fail |

Malformed report state is a harness/compiler error (exit 2), distinct from a
valid build failing the checked-automation subgate (exit 1). The first slice
grants no backend-only elimination credit, has no warning suppression, and accepts no
retained-site approvals (`approvals` must be empty). Per-site GATE-1 approval
requires dependency-cone identity and is deliberately not implemented yet.
Explicit source checks, overflow, allocation, FFI, and transitive imported
summaries are also outside this bounds-only policy; explicit checks remain
non-failing as required by review B3.

Current closure is the whole closed compilation unit, not merely the function
selected for assembly metrics. This conservative over-closure prevents moving
debt into a same-unit helper until exact reachable-instance summaries exist.
Repository permissions/review must protect coordinated changes to the
separately pinned root set before this can satisfy GATE-1; the tool cannot infer
whether a Git author is the owner.

JSON output carries a structured `promotion` object. `invocation_validated`
means the unfiltered workflow and pins were authenticated; `passed` separately
records the verdict. Policy, root descriptors, and policy-oracle count/digest
are included so saved evidence does not confuse invocation with success.

The runner currently exposes:

- `raw_ir.alias_scope_uses`, `raw_ir.noalias_uses`,
  `raw_ir.saturating_add_mentions`, `raw_ir.trap_calls`;
- `opt_ir.loads`, `opt_ir.vector_loads`, `opt_ir.trap_calls`,
  `opt_ir.saturating_add_mentions`;
- `opt_ir.vector_ops`, `opt_ir.max_vector_width` (scoped to the selected
  function, unlike compiler remark summaries);
- `asm.instructions`, `asm.opcodes`, `asm.traps`;
- `remarks.vectorized_loops`, `remarks.max_vector_width`, and
  `remarks.max_interleave`.
- `proof.bounds_sites`, `proof.proved`, `proof.retained`, `proof.ceiling`,
  proof-reason and target counts, and the deterministic `proof.sites` records.
- `proof.checked_automation_ready`, disposition counts, and findings for the
  selected function, plus corresponding `*_module_*` whole-unit metrics.

The compiler API accepts a fresh `proof_report=[]` out-parameter. Reporting is
observational: a regression pin requires byte-identical IR with and without it.
Each record is a lowered/codegen bounds site, with a function-local ordinal,
status, proof rule, target kind, target binding, and index binding.
It also records the schema-versioned obligation-analysis scope, completion bit, and
exact analyzer IDs. `not-applicable` is never a generator default. The
conservative per-site candidate frontier covers nonliteral indexed writes and
accesses through the current unique-reference roots, so the enumerated n27–n33
syntax/alias escapes fail closed. This is not completeness over fixed-literal
writes, arbitrary user control/`Result` rewrites, calls, or imported code.

Exact opcode parity is intentionally sensitive to a toolchain upgrade. If a
new backend makes all three variants better but different, inspect the diff and
update the gate in the same change; do not weaken it merely to restore green.
