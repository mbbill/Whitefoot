# Small results in return registers

## Question and scope

The internal callable ABI (`compiler/src/backend/abi.rs`) returns every stored
aggregate, meaning every struct, non-tag-only enum, inline array, constant
capacity window and opaque value, through a destination pointer:
`define void @wf_f(ptr %wf.result, ...)`. A call that LLVM does not inline
therefore moves even a two-word result through memory. The callee stores
each field through `%wf.result` and the caller reloads it.

On main at `efe40194a`, the hash-map lookup `find` of
`tests/programs/containers/hashmap.wf`, returning `Option<u64>`, is not
inlined into its hot caller `map_trace`, and every call stores and reloads
its result. Rust returns `Option<u32>` in registers.

This investigation selects which stored-aggregate results cross the call
boundary as first-class LLVM values and how every call route carries them.
Language acceptance, the proof system and value representation inside a body
are out of scope. compiler/storage-representation keeps its storage plans.
compiler/backend-facts keeps its parameter facts.

## Candidates

- **Destination for every aggregate**: the baseline.
- **Every aggregate by value**: return each stored aggregate as its LLVM
  first-class type.
- **Byte bound**: by value when the selected-target size is at most 16 bytes,
  the C and Rust register-return size on x86-64 and AArch64.
- **Register bound**: by value when every scalar leaf of the LLVM
  representation receives its own return register under LLVM's return
  convention on every admitted target. On x86-64 that allows three
  integer-class leaves (RAX, RDX, RCX) and two floating leaves (XMM0, XMM1).
  AArch64 allows eight of each.
- **Both bounds**: the register bound and the byte bound together.
- **`fastcc`**: keep the destination and select LLVM's fast calling
  convention.
- **Packed coercion**: coerce a result of at most two words to integer
  words, the way C and Rust lower their register returns.

## Selection criterion

This criterion was written before any probe or measurement below:

1. **Correctness.** Every admitted call route keeps its behavior: direct and
   generic calls, self-tail transfers, loop-split chunks and splitters,
   handed-out calls and their lane thunks, recursion-budget entries and
   variants, sequential clones, the build launcher and linked definitions.
   The canonical `make check` passes.
2. **No silent demotion.** When LLVM cannot return a value in registers, it
   silently passes a hidden result pointer. Each shape a candidate admits
   must return in registers on every target `TargetLayout::for_triple`
   admits, compiled with `llc -O2`. The boundary shapes are three and four
   integer leaves, two and three floating leaves, a four-leaf 16-byte
   struct, a 16-byte array of 16 bytes, a three-leaf 24-byte struct, `i1`
   leaves and a zero-leaf aggregate. A candidate that admits a demoted shape
   fails this criterion.
3. **Benefit on the surviving calls.** In the optimized x86-64 code
   (`whitefootc --emit-llvm`, then `clang -O2`), the hash-map `find` and
   `insert` calls that survive in `map_trace`, and an out-of-line
   `add_checked(a: u32, b: u32) -> Result<u32, Overflow>`, lose their
   destination store and reload. Their callee and call site then use fewer
   memory operations. Across the maintained programs, no stored-aggregate
   result that the candidate admits still reaches its caller through a
   destination.
4. **No regression.** The number of surviving destination calls does not
   increase. A find-heavy loop built from `hashmap.wf` does not slow beyond
   run-to-run variation. Both binaries are timed alternately on the same
   host, and the medians are compared.
5. **Choice among passing candidates.** Prefer the candidate that returns in
   registers every result the target's return registers can carry, because
   each such result saves its memory round trip. A candidate that changes
   the ABI of a linked definition must adapt that definition inside the one
   callable ABI of the compiler root decision. When candidates tie, prefer
   the least emitter and runtime machinery.
