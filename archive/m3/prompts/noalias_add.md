# Task: Noalias Add Accumulator

Write one exported function:

```
accumulate(acc, addend, n)
```

Semantics:

- `acc` is an exclusive mutable reference to a `u64`.
- `addend` is a shared read-only reference to a `u64`.
- Repeat `n` times: `*acc = *acc + *addend` with wrapping `u64` arithmetic.

The harness links this function against a C driver that initializes `a = 1`,
`b = 3`, calls `accumulate(&a, &b, 1000000000)`, and prints `a`.

Expected printed output:

```
3000000001
```

The optimized code should expose the noalias/read-only facts strongly enough for
LLVM to collapse the loop to constant-time arithmetic. Do not use unsafe code.
