# WF-LITERAL-LINE pre-timing code-shape inspection

Status: INSPECTED BEFORE COMPARATIVE TIMING

This record was completed without observing comparative timing. It describes
the reproducible build emitted under
`<scratch-root>/whitefoot-literal-line-floor/build` by the frozen
apparatus. Generated IR and assembly remain excluded scratch artifacts; the
run header binds this record and the protocol by SHA-256.

## Build identity

- target: `arm64-apple-darwin`, Apple M4 (Mac16,12);
- macOS: 26.5.2, build 25F84;
- C and Whitefoot backend: Apple clang 21.0.0
  (`clang-2100.1.1.101`), `-O2`, no LTO or target CPU flag;
- Rust: rustc 1.97.1, commit
  `8bab26f4f68e0e26f0bb7960be334d5b520ea452`, LLVM 22.1.6,
  `opt-level=2`, no LTO or target CPU flag; and
- ceiling: `memchr` 2.8.3 from the frozen offline lockfile.

## Scalar controls

The three scalar sources implement the frozen two-stage algorithm: global
candidate discovery, monotone line reconciliation, line-end discovery, and
matched-line enumeration. None explicitly calls `memchr`, `memmem`, `memcmp`,
`bcmp`, or another comparison/search library. The final Whitefoot, C, and
naive-Rust assembly contains no backend-introduced `memcmp` or `bcmp` call.
The tuple digest and 16-repetition aggregate remain live and are checked by
the harness.

Raw Whitefoot LLVM contains seven distinct OP-4 trap sites: four in
`find_scalar` and three in `literal_line`. After Apple Clang optimization,
three trap sites remain, all in `wf_find_scalar`:

1. the `needle[0]` access at helper entry;
2. the candidate-byte haystack access; and
3. the tail-byte haystack access.

The tail needle access and all three `literal_line` line/needle scanning checks
are discharged. Thus this build is evidence that ordinary loop guards remove
four of seven checks, but not that all direct guards are sufficient across a
helper boundary.

`wf_find_scalar` remains a separate final function, and `wf_literal_line`
contains three final assembly calls to it. Apple Clang fully inlines the
equivalent C helper into the sole `wf_literal_line` function. Rustc also
inlines the naive Rust scalar search into its exported function, whose final
assembly retains two calls to `panic_bounds_check`. These are preregistered
machine-code differences, not performance conclusions.

## Pinned library ceiling

The ceiling LLVM contains an indirect `Finder` dispatch. Final executable
disassembly contains the `searcher_kind_neon` implementation and AArch64
`cmeq.16b` packed-pair comparisons, followed by full-match comparison code.
This is the expected audited large-slice `memmem` mechanism. The global
discovery stage supplies the large remaining haystack; matched-line rescans
reuse the same runtime-needle `Finder`.

## Attribution boundary

Timing may be interpreted only through the frozen protocol. If Whitefoot has a
material same-algorithm loss, the retained helper boundary and checks are
candidates only if their cost is supported by the final code and measured
behavior. The result alone cannot distinguish call overhead, retained-check
cost, missed inlining, or another lowering effect. No check removal, proof,
intrinsic, compiler change, or language change is authorized by this record.
The algorithmic ceiling remains descriptive unless the protocol's primary
same-Clang parity prerequisite is met.
