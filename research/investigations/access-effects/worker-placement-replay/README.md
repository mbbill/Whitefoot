# Disposable ac34 worker-placement construction

This directory belongs only to the disposable
`research/pr70-worker-placement-replay` branch. It reconstructs four native
records images from the already-published artifact of workflow run
`35527433610`, verifies their isolation properties, and runs one registered
timing comparison after its identical-image null passes. It does not select a
production alignment or placement policy.

`prepare.sh` compiles the downloaded candidate records IR once. All original
call sites continue to target one fixed trampoline, while the exact 440-byte
emitted parallel worker is retained out of line at address residues 0, 16, 32,
and 48 modulo 64. The body is explicitly `noinline`. The trampoline contains
the invariant SysV stack-slot moves required by its aggregate arguments and
ends with its only control transfer, one five-byte tail-jump. The enclosing
experiment section always occupies 512 bytes, so ordinary section and symbol
layout remains fixed across variants.

The direct tail-jump has one unavoidable pairwise relocation: its four-byte
`rel32` destination changes when the body moves. The construction verifies the
entry address and size, the absence of calls, returns, or other branches, and
the final tail-jump's opcode and exact body target. It zeroes only those four
relocation bytes and then requires the entire ordinary `.text` section to
compare equal.
It also requires:

- the relocated body before and after linking to equal all 440 bytes of the
  downloaded candidate worker;
- every non-worker symbol address and size to remain equal;
- ordinary allocated sections, ELF section layout, and program headers to
  remain equal; and
- the sequential clone and native runtime to retain their addresses and bytes.

`verify-and-stage.sh` executes the fixture's existing `verify` mode for each
constructed image at W=1, W=2, and W=4, then stages the unchanged downloaded
mandelbrot, fir, quadrature, and stencil images as hardlink controls.

`run-approved-comparison.sh` is guarded by `WF_RUN_APPROVED=1`. It first runs
the existing five-kernel `compare.sh` with identical pad48 images in both arms.
If that null fails, the script stops and the result is inconclusive. If it
passes, the sole primary comparison is pad48 baseline versus pad16 candidate.
It does not time pad0 or pad32 and it does not rerun a disappointing result.

The prewritten criterion supports worker placement as the ac34 records
mechanism only if:

1. the identical pad48 null passes the unchanged verdict;
2. records W=2 and W=4 each have wall ratio below 0.97 and pad48 is lower in at
   least four of five paired passes;
3. records W=1 lies in `[0.97, 1.03]`; and
4. the four hardlinked control kernels pass.

The workflow archives the complete null and primary raw samples, reductions,
verdicts and oracle logs, along with the images, static audit, staging identity,
and actual runner CPU/toolchain provenance.

The input objects are Linux x86-64 ELF. Construction therefore runs on native
Ubuntu 24.04 with its Clang 18 and GNU binutils. Running or timing these objects
under macOS emulation would not be useful evidence.

The manual command signatures used by the workflow are:

```sh
./prepare.sh "$ARTIFACT" "$OUTPUT"
./verify-and-stage.sh "$ARTIFACT" "$OUTPUT" "$OUTPUT/staged"
WF_RUN_APPROVED=1 ./run-approved-comparison.sh \
  "$CHECKOUT" "$OUTPUT/staged" \
  "$OUTPUT/primary-results" "$OUTPUT/null-results"
```

The disposable branch temporarily replaces the already-registered manual-only
`.github/workflows/compute-bench.yml`, and is invoked with
`gh workflow run compute-bench.yml --ref research/pr70-worker-placement-replay`.
There is no push or pull-request trigger. Remove this directory, its workflow,
remote branch, and worktree when the placement investigation concludes.
Nothing here is a correctness gate or a dependency of the compiler, tests, or
maintained performance workflow.
