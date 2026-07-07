# Memory object model questions

Before aliasing, ownership, layout, and lifetime evidence can guide code generation, answer:

- What is object identity?
- What is pointer/reference provenance?
- Are interior pointers stable across moves?
- How are subobjects, padding, alignment, uninitialized bytes, and partial initialization modeled?
- What are lifetime boundaries for stack, heap, arena, GC, RC, borrowed, and global storage?
- What can unsafe code, callbacks, signals, finalizers, dynamic loading, and FFI invalidate?

Do not emit `noalias`, `lifetime.start/end`, `dereferenceable`, `invariant`, or range metadata without a matching source contract and verifier obligation.
