# Container representation experiments

This bundle supplies executable evidence for the container architecture choice
before production implementation or migration. The design question belongs to
`research/investigations/containers-and-resources/`; this directory owns only the
reproducible source probes, finite checker model, native controls, and measurements.
Its parts answer different questions:

- `dense/`: executable Whitefoot construction/update probes, helper boundaries,
  and native controls for aggregate versus final-destination storage; source
  capability and machine cost are reported separately.
- `lifecycle/`: actual compiler outcomes for pool contracts, element properties,
  complete and partial linear cleanup, and wrapped access, with nearby invalid
  programs that must still be rejected.
- `authority/`: finite concrete range certificates compared with an independent
  per-slot oracle; this is neither a language extension nor evidence that a
  symbolic library implementation already passes the compiler.
- `foundation/`: matched native layouts, a safe Rust boxed-array baseline,
  concrete construction/failure/retirement protocols, and a finite comparison of
  whole-result and field-destination construction. A current Whitefoot wide-result
  probe distinguishes these proposed mechanisms from implemented capability.
- `costs/`: same-algorithm native hash lookup layout and extent-validation
  controls, with a matched-query Rust standard-map comparator. It tests whether
  a retained runtime check is material under its stated workload and boundary;
  it is not a checked Whitefoot map implementation.

These small programs test specific capabilities and costs. They are not a
representative corpus of real applications and supply no workload-frequency data.
Broader demand selection needs evidence from established applications in languages
such as C++, Rust, and Go; that external study is not part of this bundle.

From the repository root:

```sh
make -C research/experiments/container-representation check
make -C research/experiments/container-representation measure
```

`check` is maintained by the root `research-tests` target. It builds the current
compiler, checks the finite models, validates the expected current source outcomes,
and verifies native comparisons. `measure` also records timing samples and retains
the foundation probe's producer boundary for generated-storage inspection;
timing is descriptive evidence, not a host-speed-dependent acceptance threshold.
Each subdirectory's `RESULTS.md` states its measured revision, interpretation, and
limits. Results from different evidence levels must not be substituted for one
another: an unsafe native control is not an accepted Whitefoot program, and a
finite certificate model does not establish a production proof or backend.

When production tests and maintained measurements supersede a sub-experiment's
architecture question, merge or retire it with an explanation. Do not preserve an
obsolete expected rejection as the verdict for a newly selected language rule.
