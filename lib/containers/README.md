# Whitefoot container source

These are reusable libraries written in ordinary Whitefoot. Bundle a library
source before its caller on the compiler command line; this directory gives
its code no special source-language status, import behavior or native ABI.

The current library is [grow-vector.wf](grow-vector.wf). Its caller-selected
capacity ceiling supplies the size bound for each concrete allocation; see
[the writer pattern](../../docs/patterns.md#p2-choose-the-storage-shape-from-its-occupancy-rule)
for the growth contract and
[the container investigation](../../research/investigations/containers-and-resources/X1-LIBRARY.md)
for remaining algorithm and representation work. This source is still an
evolving library, not a claim that all container operations have native-cost
evidence.

From the repository root, after building `whitefootc`:

```sh
compiler/target/gate/whitefootc lib/containers/grow-vector.wf tests/programs/containers/grow-vector-program.wf -o /tmp/whitefoot-grow-vector
```

The caller and allocation observer stay in `tests/programs/containers/`.
The ordinary Rust corpus test bundles this library with that caller, executes
sequential and parallel outputs, and checks the exact release ledger. It runs
through canonical `make check`; there is no separate library test stage or
Makefile. Research experiments remain explicitly invoked outside that gate.
