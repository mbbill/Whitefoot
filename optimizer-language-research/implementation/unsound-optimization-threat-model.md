# Unsound optimization threat model

| Threat | Facts it can break | Compiler can verify? | Possible remedy | Evidence needed |
|---|---|---|---|---|
| raw pointer casts | alias/provenance/lifetime | sometimes no | unsafe boundary / weaker metadata | C/Rust/LLVM sources |
| integer-pointer casts | provenance/noalias | often no | prohibit/unsafe/conservative fallback | LLVM provenance sources |
| FFI callbacks | effects/alias/lifetime | no | boundary contracts / conservative fallback | ABI docs |
| concurrency/data races | value stability/non-interference | partially | data-race rule / atomics / synchronization | memory model sources |
| finalizers/destructors | effects/reentrancy/lifetime | partially | explicit effects / no hidden finalizers in core | runtime docs |
