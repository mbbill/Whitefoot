# Effect-attribute opportunity calibration

This directory contains a deliberately narrow Leg-A instrument. It scans
captured optimized LLVM IR for **direct call instructions** whose target is
declared in the caller module and defined in another captured module. For each
call, it joins the function attributes visible on the
declaration and the call instruction, then compares that effective view with
the target definition.

A definition is a conservative strong witness only when it records all three
independent facts `memory(none)`, `nounwind`, and `willreturn`. A read-only
witness has a fully parsed non-writing `memory(...)` effect plus the latter two
facts. `speculatable` is tracked separately and is never substituted for
`willreturn`. The read-only tier is never folded into the strong count.

Run the synthetic calibration:

```sh
python3 -B -m unittest discover \
  -s experiments/frequency-study/effect-attrs/tests \
  -p 'test_*.py'

python3 -B experiments/frequency-study/effect-attrs/classify_ir.py \
  --root experiments/frequency-study/effect-attrs/tests/fixtures \
  --pretty \
  experiments/frequency-study/effect-attrs/tests/fixtures/*.ll
```

Schema 3 emits one record per direct call through a declaration. Each record
contains declaration facts, call-site facts, their effective join, definition
facts, linkage, normalized ABI signatures, and exact missing required facts.
Inputs and records are sorted so the same capture produces byte-identical
compact JSON. Duplicate resolved paths and empty/unrecognizable IR inputs are
rejected.

The parser handles modern location-sensitive effects, including
`memory(read, argmem: none)`, plus common legacy memory attributes. It removes
arbitrary quoted string attributes before interpreting facts, so a target-cpu
string containing `memory(none)`, `#7`, or `}` cannot contribute a fact or
terminate an attribute group. `private`, `internal`, and `available_externally`
definitions cannot satisfy an external declaration; local-name collisions are
filtered before ambiguity is decided. Return and parameter types, varargs,
calling convention, and function address space must match across the call,
declaration, and definition. Unsupported signatures and mismatches fail closed.

Valid multiline declarations, calls, invokes, and callbr instructions are
accumulated before parsing. Metadata attachments and sigiled identifiers do not
contribute attributes. Operand bundles are deliberately unsupported in this
first calibration, and inline-assembly callees are always unsupported even when an
assembly string contains text resembling `@a_direct_symbol`.

The following states remain intentionally distinct:

- `absent`: no memory promise was visible and a real gap may exist;
- `known / may-write`: an explicit memory promise was understood but is not
  strong enough;
- `unsupported`: the parser saw a memory form it does not understand, so the
  record is ineligible for a positive gap;
- operand-bundle, inline-assembly, unsupported-ABI, and ABI-mismatch records,
  which also fail closed;
- unresolved, incompatible-linkage, and multiple-definition targets, which
  are likewise never positive gaps.

## What this does not measure

This is not yet a frequency result. An IR call is not evidence that it
executes, is hot, has invariant arguments, or admits LICM/CSE/DCE. The
classifier also cannot distinguish Rust code from FFI, sysroot, allocator,
panic, or runtime symbols without a build-provenance manifest. Consequently
its output is calibration evidence only: it cannot clear D9 and it cannot be
reported as a speed opportunity.

The corpus phase must add those missing controls without changing this
classifier's meanings:

1. Pin project commits, lockfiles, toolchain, release profile, workload command,
   seeded input, and output digest. Capture optimized IR in a shadow rustc run
   and verify the ordinary and captured builds have equivalent `.text`.
2. Use build provenance and disassembly to keep only machine-surviving direct
   Rust-to-Rust calls whose target definition is in another captured codegen
   unit. Exclude indirect, native/extern, unresolved, sysroot, panic, allocator,
   and runtime edges from the primary denominator; report them separately.
3. Record at least three identical `samply` runs. Recover the physical callsite
   from each non-leaf return PC and require an actual machine `call`/`bl`, so
   DWARF inline frames cannot masquerade as surviving calls.
4. Define a hot site before looking at results (the proposed calibration is at
   least 0.5% of captured-Rust on-CPU samples in at least two of three runs).
   Report both `strong hot gaps / all hot direct Rust calls` and the union of
   samples whose stack crosses at least one strong gap. The latter counts a
   sample once; summed edge weights are a separate, possibly over-100% metric.
5. Use `panic=abort` and LTO only as sensitivity builds. A candidate that exists
   solely because Rust may unwind is not an effect-row win, and a call removed
   by LTO is not causal proof because visibility, inlining, and global
   optimization changed together.

Finally, audit the highest-weight sites for a real consumer (loop-invariant
arguments, redundant equal calls, or an attribute-blocked transformation) and
validate selected sites with an attribute-only experiment. The IR gap is a
conservative semantic proxy; realized performance remains a separate result.
