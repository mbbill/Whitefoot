# Modular build cost and runtime quality

Measured 2026-09-24 on a 4-core Linux x86-64 container (kernel 6.18), Ubuntu
clang 18.1.3 and LLD 18.1.3, with the gate-profile `whitefootc` of the modular
compilation branch. One run of [`run.sh`](run.sh); wall times in milliseconds
from the invocation's start to its exit, so each includes process start, input
reading and hashing the compiler's own executable (about 40 ms of every cached
invocation). The machine was otherwise idle but shared, and single runs vary
by about 10%; treat differences below that as noise.

This bundle answers slice 6 of the
[modular compilation design](../../investigations/modular-compilation/DESIGN.md#ordered-implementation-slices-and-completion-evidence)
for the implementation that exists: module-granular verdict records, entry
module records, cached runtime and program objects, and ThinLTO link fragments
split by LLVM's `llvm-extract` and linked by LLD with its ThinLTO object cache.
It does not measure declaration- or component-granular proof reuse, which is
not implemented.

## Workloads and modes

- `demo`: the five-module [specimen](../../investigations/modular-compilation/demo/README.md),
  entry `inspect`.
- `chain-8`, `chain-32`: generated chains of 8 and 32 modules, each publishing
  16 functions that call their predecessor's; entry `app`.
- `rng`: [`rng/`](rng/), a 400-million-step xorshift loop in the root module
  calling `pkg::rng::next` across a module boundary, for runtime quality.

Build modes: `none` is a build without `--cache` (one clang invocation);
`image` caches one object for the entry's whole module; `module` and
`function` split that module into ThinLTO fragments per source module or per
function. Edits: `body-edit` changes one private body (the specimen's report
helper, or the middle chain module's bodies); `interface-edit` changes a
`doc` entry of the first dependency's interface.

## Checking: `--check-modules` over every module and entry

| workload | no cache | cold cache | warm | after body edit | after interface edit |
|---|---:|---:|---:|---:|---:|
| demo (5 modules, 2 entries) | 148 | 232 | 48 | 115 (3 of 7 recomputed) | 165 (6 of 7) |
| chain-8 (9 modules, 1 entry) | 296 | 354 | 54 | 147 (2 of 10) | 389 (10 of 10) |
| chain-32 (33 modules, 1 entry) | 1869 | 1745 | 63 | 404 (2 of 34) | 1810 (34 of 34) |

A warm check costs what reading inputs and hashing costs. A body edit
recomputes its module and the compositions that contain it, and nothing else.
An interface edit, even one that only rewrites documentation, recomputes every
module whose dependency closure reads that interface, because a verdict's key
is the exact bytes of every interface it read: in a chain, every downstream
module. Every cached run printed the verdicts a run without a cache printed.

## Building one entry

Total wall time; the compiler's report splits it into front end, fragment
split, object compilation and link.

| workload | mode | cold | warm | after body edit | objects recompiled after the edit |
|---|---|---:|---:|---:|---:|
| demo | none | 1233 | | | |
| demo | image | 1317 | 131 | 195 | 1 of 13 |
| demo | module | 1174 | 133 | 240 | 1 of 19 |
| demo | function | 1502 | 127 | 374 | 1 of 28 |
| chain-8 | none | 1225 | | | |
| chain-8 | image | 1283 | 117 | 216 | 1 of 13 |
| chain-8 | module | 1313 | 123 | 403 | 1 of 23 |
| chain-8 | function | 1598 | 130 | 399 | 1 of 25 |
| chain-32 | none | 1490 | | | |
| chain-32 | image | 1700 | 143 | 495 | 1 of 13 |
| chain-32 | module | 2571 | 118 | 901 | 1 of 47 |
| chain-32 | function | 2675 | 132 | 991 | 1 of 49 |

- The twelve runtime units dominate a cold build (about 1.1 s of object
  compilation); after the first build they are always reused.
- A warm build of an unchanged entry reuses its emitted module, every object
  and the fragment split, and costs the final link (about 45 ms) plus input
  validation. The cold and warm executables of each mode were byte-identical,
  and an `image` build is byte-identical to a `none` build.
- After a body edit exactly one object is recompiled in every mode. The rest
  of the edit's cost is the front end, which rechecks and relowers the whole
  composition (320 ms for chain-32), and, for fragment modes, re-splitting the
  new module with one `llvm-extract` process per fragment (430 to 490 ms for
  chain-32). Both are the next costs to remove: component-granular checking,
  and a native or parallel split.

## Runtime quality

Five runs of the `rng` benchmark per mode; the loop's result is identical
(exit status 10) in all of them.

| mode | median | min | max |
|---|---:|---:|---:|
| none | 751 | 676 | 768 |
| image | 718 | 654 | 760 |
| module | 704 | 592 | 761 |
| function | 700 | 646 | 754 |

`rng::next` is inlined into the loop in every mode: the ThinLTO link imports
it across the fragment boundary, and no mode leaves a call in the loop. The
loop itself is the same six dependent shift and xor instructions, unrolled
twice in the fragment builds. No mode shows a repeatable loss at this noise
level; `image` is the same-image comparator, and within the program it is what
full LTO would optimize together. A workload whose hot path crosses many
fragments, and a full-LTO build of the runtime units as well, remain to be
compared.

## Limits

- Module granularity: a verdict record covers a whole module check, so any
  edit rechecks every body of the module it touches and of each composition
  that contains it; documentation edits to an interface invalidate like any
  other byte change.
- Instance symbols carry a numeric instance ordinal and LLVM type names carry
  nominal ordinals, so an edit that adds or removes an instance or a nominal
  type can rename otherwise unchanged fragments and cost their objects.
- The ThinLTO planning runs in full at every link, as the design's first step
  selects; at these sizes the whole link is under 120 ms.
- One host, one compiler build, single runs per build step.
