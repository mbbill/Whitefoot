# Container representation experiments

This bundle supplies executable evidence for the container architecture choice
before production implementation or migration. The design question belongs to
`research/investigations/containers-and-resources/`; this directory owns only the
reproducible source probes, finite checker model, native controls, and measurements.
Its three parts answer different questions:

- `dense/`: executable Whitefoot construction/update probes, helper boundaries,
  and native controls for aggregate versus final-destination storage; source
  capability and machine cost are reported separately.
- `lifecycle/`: actual compiler outcomes for pool contracts, element properties,
  complete and partial linear cleanup, and wrapped access, with nearby invalid
  programs that must still be rejected.
- `authority/`: finite concrete range certificates compared with an independent
  per-slot oracle; this is neither a language extension nor evidence that a
  symbolic library implementation already passes the compiler.

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
compiler, checks the finite model, validates the expected current source outcomes,
and verifies the dense native comparisons. `measure` also records timing samples;
timing is descriptive evidence, not a host-speed-dependent acceptance threshold.
Each subdirectory's `RESULTS.md` states its measured revision, interpretation, and
limits. Results from different evidence levels must not be substituted for one
another: an unsafe native control is not an accepted Whitefoot program, and a
finite certificate model does not establish a production proof or backend.

When production tests and maintained measurements supersede a sub-experiment's
architecture question, merge or retire it with an explanation. Do not preserve an
obsolete expected rejection as the verdict for a newly selected language rule.
