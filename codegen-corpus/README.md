# Codegen corpus format

The corpus is organized into compact `cases.json` family manifests discovered
recursively under `cases/`. Each family states one optimization hypothesis and
a named-function metric; its nearby `.xl` sources vary one premise at a time.
Adding a family never requires editing one central manifest.

Every positive proof case has near-identical negative controls. The facts-on
and facts-off variants are synthesized by the runner, so generated manifests
cannot accidentally compare different source programs. Proof classification
uses the compiler's structured per-site report for the named function:
optimized IR/assembly is recorded separately because LLVM may independently
remove the same check.

```text
codegen-corpus/
  schema.json
  cases/
    bounds/
      dominating-guard/
        cases.json
        01-basic-read-positive.xl
        05-wrong-buffer-negative.xl
      masked-index/
        cases.json
        p01-mask3-table4.xl
        n02-oversized-mask.xl
      derived-range/
        cases.json
        p04-remainder-index-i.xl
        n07-remainder-nonzero-init.xl
      output-capacity-lockstep/
        cases.json
        p05-complete-groups.xl
        n21-output-buffer-uniq-reborrow.xl
```

Run all families or select a tag:

```sh
make corpus
python3 tools/codegen_parity.py --corpus --tag proof-2
```

## Field policy

- Family `tags` are inherited by every contained case. Case tags add the
  polarity, mutation shape, or specific premise under test.
- `maturity` is `explore`, `audit`, or `gate`. Explore cases collect evidence;
  audit cases describe a known target without blocking; gate cases are earned
  invariants and may fail verification.
- `hypothesis` must name the causal property being tested, not merely say that
  one variant should be faster.
- `proof_classification` is `proved`/`elided` for positive cases and
  `retained`/`checked` for near-misses. Silently proving a near-miss is a
  blocking soundness failure. `mixed` cases give exact `proved_sites` and
  `retained_sites` counts to pin partial discharge within one function.
- `bounds_sites` is the exact number of lowered/codegen bounds sites expected
  in both facts-on and facts-off variants; this prevents a missing site from
  disappearing symmetrically and escaping the gate.
- Optional `checked_automation_ready` plus the complete four-key
  `checked_automation_disposition_counts` map pins the facts-on policy result.
  These are diagnostic oracles: a negative case is green when it produces the
  expected hard/unaccounted promotion failure. They never nominate a promoted
  root, and facts-off is never a promotion candidate.

Paths in `source` are relative to the fragment containing them and must remain
inside the repository. Source and recipes are tracked; generated IR, assembly,
objects, binaries, and expanded metamorphic cases are temporary artifacts.
Site ordinals are deterministic within each lowered function, after any
fact-driven AST transform; they are not source locations or cross-variant IDs.

Promotion is deliberate: an `explore` case becomes `audit` once its target is
understood, and becomes `gate` only after the property is implemented, verified
against adversarial near-misses, and stable on the supported toolchain.
This corpus-maturity promotion is distinct from the checked-automation
bounds-v1 build subgate. Only the dual-pinned review roots described in
[`CODEGEN-PARITY.md`](../CODEGEN-PARITY.md) receive that evaluation, and
protected external owner review is still required for coordinated pin changes.
