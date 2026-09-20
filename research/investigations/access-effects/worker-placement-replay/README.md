# Disposable ac34 worker-placement construction

This directory belongs only to the disposable
`research/pr70-worker-placement-replay` branch. It reconstructs four native
records images from the already-published artifact of workflow run
`35527433610`, verifies their isolation properties, and runs the existing
compiler-independent records oracle. It does not measure performance and does
not select a production alignment or placement policy.

`prepare.sh` compiles the downloaded candidate records IR once. All original
call sites continue to target one fixed five-byte tail-jump, while the exact
440-byte emitted parallel worker is retained out of line at address residues
0, 16, 32, and 48 modulo 64. The body is explicitly `noinline`. The enclosing
experiment section always occupies 512 bytes, so ordinary section and symbol
layout remains fixed across variants.

The direct tail-jump has one unavoidable pairwise relocation: its four-byte
`rel32` destination changes when the body moves. The construction verifies the
entry address, size, opcode, and exact body target, zeroes only those four
bytes, and then requires the entire ordinary `.text` section to compare equal.
It also requires:

- the relocated body before and after linking to equal all 440 bytes of the
  downloaded candidate worker;
- every non-worker symbol address and size to remain equal;
- ordinary allocated sections, ELF section layout, and program headers to
  remain equal; and
- the sequential clone and native runtime to retain their addresses and bytes.

`verify-and-stage.sh` executes only the fixture's existing `verify` mode for
each constructed image at W=1, W=2, and W=4. The workflow contains no call to
`measure`, `compare.sh`, the reducer, or the verdict. Its artifact contains the
static report, images, and correctness logs for review before any separate
timing decision.

The input objects are Linux x86-64 ELF. Construction therefore runs on native
Ubuntu 24.04 with its Clang 18 and GNU binutils. Running or timing these objects
under macOS emulation would not be useful evidence.

Remove this directory, its workflow, remote branch, and worktree when the
placement investigation concludes. Nothing here is a correctness gate or a
dependency of the compiler, tests, or maintained performance workflow.
