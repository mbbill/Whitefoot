# Whitefoot container source

These are reusable libraries written in ordinary Whitefoot. Bundle a library
source before its caller on the compiler command line; this directory gives
its code no special source-language status, import behavior or native ABI.

| Source | Operations and boundary |
| --- | --- |
| [grow-vector.wf](grow-vector.wf) | Reserve, growing append, indexed insertion/removal, and ordered consumption over `Box<Slots<T>>`. |
| [deque.wf](deque.wf) | Reference-taking endpoints, logical visitation and drain over `Box<Ring<T>>`; explicit consuming rebase produces a new backing. No automatic endpoint growth or zero-copy two-span interface. |
| [slab.wf](slab.wf) | One bounded backing with lazily materialized slots, generation handles, reuse, expiry, exhaustion returning the offered owner, and explicit consumption. Handles are relative to the supplied slab. |

The caller-selected capacity ceiling supplies the size bound for each concrete
allocation. See [the writer pattern](../../docs/patterns.md#p2-choose-the-storage-shape-from-its-occupancy-rule)
for the contracts and [the container investigation](../../research/investigations/containers-and-resources/X1-LIBRARY.md)
for comparison evidence and remaining interface limits. These evolving libraries
do not claim native parity for every operation.

From the repository root, after building `whitefootc`:

```sh
compiler/target/gate/whitefootc lib/containers/grow-vector.wf tests/programs/containers/grow-vector-program.wf -o /tmp/whitefoot-grow-vector
```

The callers and shared allocation observer stay in `tests/programs/containers/`.
The Slab caller also bundles `slab-membership-program.wf`, which exercises weak
indexes and a composite retained-membership protocol. The ordinary Rust corpus
tests bundle each library with its caller, execute sequential and parallel
outputs, and check the exact release ledger. They run
through canonical `make check`; there is no separate library test stage or
Makefile. Research experiments remain explicitly invoked outside that gate.
