# Numeric conversion coverage

## Question and scope

The baseline is main at `5dd9d5d7`, after generic struct constants. TYPE-4 and
OP-6 make `cvt` exact and value-preserving: its result type is selected from
the complete source/destination type pair. OP-8 separately supplies same-width
bit reinterpretation and same-type floating-point rounding operations.

This investigation asks which useful numerical transformations can be written
directly, which need an ordinary composition, which require a failure branch
despite a known domain, and which are missing. It does not change the active
specification or select new operation spellings before comparing examples.
The immediate consumer is integer low-bit extraction for binary encoding;
the wider inventory includes integer range/signedness changes, saturation,
integer/float conversion and float precision changes.

## Evidence criteria

For each family, separate three observations:

1. Mathematical contract: preserved value, selected bits, rounded value, or
   clamped value; state the treatment of negative values, bounds and special
   floating-point values where applicable.
2. Accepted source: current result type, required branches or proofs, and
   whether an ordinary source function can express the intended total result.
3. Generated code: inspect emitted and optimized LLVM for parameterized
   helpers, so constant-folded main calls cannot establish a general cost.
   Distinguish a source-level Result from an actual surviving runtime branch.

Use complete small WF programs and boundary controls. Low-bit extraction must
include values beyond the destination range; value-preserving narrowing must
retain a failing out-of-range control. Signedness probes include negative
inputs and the high bit. Float probes include fractions, exactness boundaries,
signed zero, infinity and NaN. Existing conversion conformance cases remain
the baseline for already implemented OP-6 behavior, not a new redundant suite.

Compare retaining ordinary compositions, adding a total bit-truncation family,
and exposing a proof-required exact conversion alongside its checked form.
For rounding/saturation, first determine whether existing operations compose
with precise enough behavior; a missing convenience spelling alone does not
establish a runtime cost. No timing claim will follow from instruction counts.
Use primary Rust/LLVM documentation only where external comparisons help
distinguish semantics or lowering requirements.

The deliverable is a bounded inventory with source and emitted-code evidence,
ranked improvement options, and explicit unresolved policy questions. Retained
probe sources live beside this investigation for reproduction and are removed
or replaced when their observations are superseded. They are not daily test
inputs; any selected implementation extracts necessary regression evidence
into the maintained compiler/conformance suites. Deferred findings update the
existing numeric-conversion TODO rather than creating an unrelated work queue.
