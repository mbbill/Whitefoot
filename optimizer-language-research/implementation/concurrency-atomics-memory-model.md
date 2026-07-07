# Concurrency, atomics, and memory model

Research questions:

- What is the language-level data-race rule?
- Which memory orderings and fences exist?
- How do actors/tasks/channels transfer ownership?
- What can foreign threads, callbacks, signal handlers, finalizers, and unsafe code mutate?
- How do atomics and synchronization constrain alias/lifetime/value assumptions?

Map candidate facts later to LLVM atomic orderings, fences, sync scopes, volatile, and MLIR async/concurrency representations.


## Phase-2 evidence summary

See `notes/phase2-concurrency-findings.jsonl` (C001-C006) and `synthesis/phase2-concurrency-findings-index.md`. Key tracks: DRF-SC convergence, UB-vs-bounded race semantics split, optimizer constraints from possible sharing, Rust Send/Sync type-level sharing facts, LLVM atomic ordering gradation, Java final-field publication safety.

Open: primary C++ standard text and JLS chapter 17 (fetches blocked), Swift/Ada/Pony/Erlang models, GPU memory models, empirical cost data for bounded-race semantics.
