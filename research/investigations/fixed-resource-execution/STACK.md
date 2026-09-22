# Stack bytes for a supplied capacity

This investigation owns the first resource question: whether an actual
compiled invocation fits a stack capacity supplied in bytes before execution.
It uses the compiler at main `7127bcb6`; the earlier probes in [README.md](README.md)
retain their original compiler identity. Total-work estimates are deferred:
progress evidence is needed for completion, and depth/path evidence where
stack composition needs it, but no aggregate execution-cost report is required.

## Criteria before the probes

The candidate basis is one post-codegen frame/call inventory from the exact
objects linked. A pre-codegen slot layout and the current developer ledger
are comparison inputs, not qualification authority. Distinguishing cases are:

1. Fixed inline storage, including one frame larger than a 4 KiB budget.
   Compare source layout, optimized frame report and actual SP-relative
   accesses. A source array size alone must not select a stack result.
2. Leaf storage below an unchanged SP, ordinary calls, tail transfers and
   serial siblings. Compare the report with instruction-level stack geometry;
   unexplained memory below SP disqualifies naive report summation.
3. Over-aligned locals, variable stack allocation, unresolved direct calls,
   indirect calls and generated large-frame helpers. Missing or dynamic
   information must remain a gap, never disappear into a zero-cost edge.
4. Reuse the native 4 KiB adapter with a known-fitting WF calculation and
   inspect the exact assembly used at link. Do not execute a deliberately
   undersized stack: fit and non-fit arithmetic are static observations.

After inspecting the realignment instructions, the native observer tests all
four 16-byte-aligned residues modulo 64. It predicts 64 bytes for the ordinary
leaf and 192, 240, 224, 208 bytes for the realigned leaf at entry offsets
0, 16, 32, 48. An index-zero write exposes the lowest addressed local byte;
these are an assembly-derived oracle, not a generic high-water proof.

The freestanding [C probe](frame-probes.c) isolates target reporting and ABI
geometry; it is not a WF language example or a new runtime. The WF probe uses
ordinary source lowering. Cross-target assembly is inspection evidence, not
an executed qualification of that target. These sources belong here until
maintained stack-qualification regressions replace them; this document then
retains only useful measured grounds and rejected alternatives.

## Observations on main 7127bcb6

The compiler and specification inputs match main `7127bcb6`. The host is
arm64 macOS 26.6.2, Apple clang 21.0.0 (`clang-2100.3.34.2`), with ordinary
WF `--no-overlap` lowering and `-O2`. C geometry probes use `-O2` and
`-fno-stack-protector` to isolate frame/ABI accounting; that is not a change
to WF production compilation. Only arm64 Darwin programs were executed.

### A static report is not a complete byte bound

Raw `.su` bytes are below. All rows are `static` except `stack_dynamic`.
The two arm64 target reports agree for these probes; this does not establish
general ABI equivalence.

| Function | arm64 Darwin/Linux | x86-64 Linux | x86-64 Darwin | x86-64 MSVC |
|---|---:|---:|---:|---:|
| `stack_leaf` | 64 | 0 | 8 | 72 |
| `stack_large` | 8,208 | 8,072 | 8,200 | 8,200 |
| `stack_parent` | 96 | 72 | 88 | 104 |
| `stack_siblings` | 32 | 24 | 24 | 56 |
| `stack_tail` | 0 | 0 | 8 | 0 |
| `stack_realign` | 192 | 184 | 184 | 184 |
| `stack_dynamic` | 16 dynamic | 8 dynamic | 24 dynamic | 8 dynamic |
| `stack_unknown` | 80 | 72 | 72 | 104 |
| `stack_indirect` | 16 | 8 | 8 | 40 |

On x86-64 Linux, `stack_leaf` never moves RSP but writes
`-72(%rsp,%rdi)` with index in 0..63. Its local extent below entry RSP is
72 bytes, plus the ordinary incoming call's 8-byte return address. The
current WF ledger reports 8 bytes, not the required 80 for that invocation.
Its `stack_parent -> stack_leaf` chain reports 88 bytes; the same instructions
require up to 160 including both return addresses. x86-64 Darwin's leaf
uses a saved frame pointer and below-SP locals, and its 8-byte raw report
likewise omits the locals. These are assembly deductions, not x86 execution
measurements.

Arm64 `stack_realign` saves 16 bytes, subtracts 176 and rounds SP down to a
64-byte boundary. The report says `192 static`, but the rounded address also
depends on the ABI-admitted entry alignment. The native observer writes at
index zero on four supplied entry addresses and reports lowest writes of
192, 240, 224 and 208 bytes, matching the assembly-derived predictions.
The ordinary-alignment leaf control remains 64 bytes in all four cases. Thus even
`static` plus a fixed return-address correction is insufficient. The observer
is evidence for these bodies; measuring a high-water mark is not a general
proof over arbitrary programs or inputs.

The bounded C variable array has 1..256 bytes, but its `.su` row contains
only a fixed prefix with a `dynamic` qualifier. Both C probes demonstrate
why a qualifier and a number need interpretation before composition; they
are not new source-language restrictions. Current ordinary WF slot lowering
puts fixed-size allocations in function entry, so these native geometries
must not be described as existing WF source syntax.

### Neither another file format nor a late flag repairs the evidence

For the x86-64 Linux object, `.stack_sizes` repeats leaf 0, large 8,072 and
realigned 184; it omits `stack_dynamic` entirely. LLVM's
[AsmPrinter implementation](https://llvm.org/doxygen/AsmPrinter_8cpp_source.html)
uses the same frame-size fields for the section and `.su`, and its `static`
test is absence of variable-sized objects. This source inspection explains
the observations; it does not qualify another LLVM version.

Compiling the C probe with `-mno-red-zone` changes the leaf's raw report to
72 bytes. Adding that flag only when compiling the already-produced default
LLVM IR leaves the leaf at 0 in this toolchain. Producing IR with the C flag
retains the `noredzone` function attribute, and compiling that IR produces
72 bytes. LLVM documents the
[attribute's meaning](https://llvm.org/docs/LangRef.html#function-attributes).
A flag on one late driver invocation therefore cannot stand in for the
required property of every generated and linked body. Disabling the red zone
also does not repair realignment or missing callees, so it is not selected as
the general solution here.

Apple documents a 128-byte ARM64
[red zone](https://developer.apple.com/documentation/xcode/writing-arm64-code-for-apple-platforms)
and 16-byte SP alignment. These arm64 probes happen to allocate their locals
explicitly; that observation does not justify assuming no future body can
access below SP.

### Real WF frames and hidden callees

[frame-geometry.wf](frame-geometry.wf) fills and indexes 1,024 u64 elements.
Its ordinary native run checks every index against the direct square formula
and exits zero. The optimized `wf_frame_sample` frame is 8,240 bytes and
`wf__main_body` is 8,272; the source element bytes alone were 8,192. The
body's explicit SP movement already excludes a 4 KiB stack. This case was
never run on the undersized adapter.

The assembly also calls `bzero` and reaches `___chkstk_darwin` through a
GOT-loaded register and `blr`. The current ledger omits the former because
it has no frame row and does not recognize `blr` as a call. Its reported
8,272-byte root chain is not a complete bound, even though it suffices here
to identify a body that cannot fit 4 KiB. A larger budget does not resolve
those missing helper obligations.

Changing only the fixture capacity and iteration limit to 8 produces a
64-byte standalone helper. The optimizer folds all main-body calculations;
the actual root's remaining frame is 48 bytes, with two zero-frame exit
helpers. An uncalled retained definition must not be added to that root's
peak. Linked with the existing 32-byte adapter, the inspected invocation uses
at most 80 bytes of the supplied 4,096-byte region and returns zero.

The existing bounded-recursion fixture was rebuilt with this compiler too.
Its same-assembly linked invocation still has the conservative
`32 + 64 + 33*32 = 1,152` byte bound and returns zero on the 4 KiB adapter.
The recursive depth argument remains manual; no termination checker was
implemented. Both small and recursive fixed-stack executables have no
undefined symbols; final disassembly covers every call on their entry paths.
The host loader, suspended caller stack and asynchronous activity remain
outside this single synchronous invocation claim.

## Selected implementation boundary

Start with an arm64 Darwin target inventory for one synchronous entry and a
supplied byte capacity. This is the locally executed target, not a language
restriction or a claim that other targets are invalid. The existing ordinary
lowering remains the code producer. A stack result alone does not assert
completion or a complete no-heap deployment guarantee.

The internal inventory needs a body identity, local stack extent, alignment
conditions, call/tail edges and explicit gaps. Its numeric field is the bound
on the body's local stack extent below entry SP, covering SP movement and
below-SP accesses, not a relabeled raw `.su` value. Report bytes remain an
input to check against the target geometry. Every admitted instruction form
that changes SP or derives a stack address needs a specified accounting rule;
unknown geometry stays unsupported. An alignment-dependent adjustment needs
a bound over all admitted residues, or a gap. A blanket `static` admission
rule fails the realignment control above.

For an ordinary call edge f -> g, retain caller displacement d at that edge
and the target's return-address cost r (0 on arm64, 8 on ordinary x86-64
calls). With local extent L_f and complete callee peak P_g, compose:

```text
P_f = max(L_f, max over ordinary edges (d + r + P_g))
```

Serial siblings contribute a maximum. For a proved ABI-preserving tail edge
that restores the callee's entry SP to f's entry SP, include P_g directly in
the maximum: no additional retained frame or return address. An unproved
tail transfer is not granted that reduction. The supplied-stack adapter is
an explicit region handoff, with its own 32-byte local cost, not an ordinary
SP adjustment to infer from the host caller. An incoming call's cost is
charged at its edge or entry boundary, never in both places.

The first automatic composition handles a complete acyclic machine call
graph. It can be implemented before rank syntax. Unbounded SP changes within
a body and recursive call components without a corresponding depth proof
stay unresolved. Ordinary WF loops that reuse fixed entry storage need no
iteration count merely to establish that local stack extent. Progress proofs
are a separate later input. When a source depth proof is used, its relation
to the optimized recursive component must be established.

The driver must link the objects built from the analyzed assembly, retain all
native dependencies and check the final selected definitions and introduced
stubs. Unknown direct or indirect targets, body-less frame rows, absent rows,
dynamic/unaligned geometry, malformed input and arithmetic failure remain
typed gaps. A function's existence is derived from the machine/object
inventory, not inferred solely from the rows that happen to parse.

This is a bounded target-analysis implementation, not a general binary
correctness verifier. The compiler/toolchain's semantic correctness remains
in the existing trust boundary. Machine stack accounting still needs its
own implemented instruction rules and negative controls before an automatic
fit result can be published. The successful probes above do not implement it.

Three result classes suffice: a complete upper bound within S; a complete
upper bound that does not certify S; or incomplete evidence with named gaps.
An independently demonstrated lower bound can additionally show actual
non-fit, as the 8 KiB local frame does. Do not reinterpret a conservative
upper-bound excess as that stronger conclusion. Changing the image, entry
contract or target configuration invalidates its previous result.

## Alternatives and remaining uncertainty

- Pre-codegen source slots: reject as authority because optimized locals,
  spills and ABI objects differ; retain them for mandatory representability.
- Raw `.su`, or `.stack_sizes` instead: reject as complete byte authority
  because both miss the red-zone case and `static` misses entry realignment.
- Disable all optimization or apply a late no-red-zone flag: not selected;
  these change geometry without closing native dependencies or proving the
  remaining report assumptions. Effective `noredzone` could be a later target
  policy, backed by the actual emitted bodies and a measured consumer.
- General machine-code proof framework: defer. A focused target inventory
  with explicit unsupported geometry serves the first consumer; a stable
  artifact protocol and arbitrary native-code certification do not.

The source/call summary and target inventory remain distinct responsibilities.
The extra work exposed here is local stack geometry and complete edge capture,
not another affine source solver. Exact native helper closure, target rule
coverage and tighter call-site displacements remain implementation work.
The maintained [TODO](../../../docs/todo.md) records those gaps and validation
conditions. Cross-target execution, asynchronous stacks and broader dynamic
frames are deferred until a concrete consumer needs them.

## Reproduction

Run from the repository root. The normal optimized compiler build is
`make -C compiler build`; it took 44.97 seconds in this run, separately from
probe construction and execution. The commands below use the repository guard;
if another command owns it, inspect that PID and wait for its completion.

```sh
stack_study=research/investigations/fixed-resource-execution
stack_scratch=$(mktemp -d /tmp/whitefoot-stack-bytes.XXXXXX)
for stack_target in aarch64-apple-darwin x86_64-apple-darwin \
  aarch64-unknown-linux-gnu x86_64-unknown-linux-gnu x86_64-pc-windows-msvc; do
  perl .github/run-check.pl stack-frame-report /usr/bin/clang \
    -target "$stack_target" -std=c11 -O2 -fno-stack-protector \
    -S -fstack-usage "$stack_study/frame-probes.c" \
    -o "$stack_scratch/$stack_target.s"
done
perl .github/run-check.pl stack-observer-build /usr/bin/clang \
  -std=c11 -O2 -fno-stack-protector -DSTACK_NATIVE_OBSERVER \
  -S -fstack-usage "$stack_study/frame-probes.c" -o "$stack_scratch/observer.s"
perl .github/run-check.pl stack-observer-link /usr/bin/clang \
  "$stack_scratch/observer.s" -o "$stack_scratch/observer"
perl .github/run-check.pl stack-observer-run "$stack_scratch/observer"

perl .github/run-check.pl stack-object-report /usr/bin/clang \
  -target x86_64-unknown-linux-gnu -std=c11 -O2 -fno-stack-protector \
  -c -fstack-size-section "$stack_study/frame-probes.c" \
  -o "$stack_scratch/section.o"
llvm-readobj --stack-sizes "$stack_scratch/section.o"
perl .github/run-check.pl stack-default-ir /usr/bin/clang \
  -target x86_64-unknown-linux-gnu -std=c11 -O2 -fno-stack-protector \
  -S -emit-llvm "$stack_study/frame-probes.c" -o "$stack_scratch/default.ll"
perl .github/run-check.pl stack-late-flag /usr/bin/clang \
  -x ir -target x86_64-unknown-linux-gnu -O2 -mno-red-zone \
  -S -fstack-usage "$stack_scratch/default.ll" \
  -o "$stack_scratch/late-flag.s"
perl .github/run-check.pl stack-noredzone-ir /usr/bin/clang \
  -target x86_64-unknown-linux-gnu -std=c11 -O2 -fno-stack-protector \
  -mno-red-zone -S -emit-llvm "$stack_study/frame-probes.c" \
  -o "$stack_scratch/noredzone.ll"
perl .github/run-check.pl stack-attribute /usr/bin/clang \
  -x ir -target x86_64-unknown-linux-gnu -O2 -S -fstack-usage \
  "$stack_scratch/noredzone.ll" -o "$stack_scratch/attribute.s"

perl .github/run-check.pl stack-wf-ir compiler/target/gate/whitefootc \
  --no-overlap --emit-llvm --stack-ledger "$stack_study/frame-geometry.wf" \
  -o "$stack_scratch/geometry.ll"
perl .github/run-check.pl stack-wf-frame /usr/bin/clang \
  -x ir "$stack_scratch/geometry.ll" -S -fstack-usage -O2 \
  -Wno-override-module -o "$stack_scratch/geometry.s"
perl .github/run-check.pl stack-wf-build compiler/target/gate/whitefootc \
  --no-overlap "$stack_study/frame-geometry.wf" -o "$stack_scratch/geometry"
perl .github/run-check.pl stack-wf-run "$stack_scratch/geometry"

ruby - "$stack_study" "$stack_scratch" <<'RUBY'
study, scratch = ARGV
source = File.read(File.join(study, 'frame-geometry.wf'))
File.write(File.join(scratch, 'small.wf'), source.gsub('1024', '8'))
native = File.read('compiler/src/backend/ordinary_values.c')
first = native.index('void wf_exit_status(')
last = native.index('void wf_socket_address_v4(', first)
raise 'missing exit definitions' unless first && last
File.write(File.join(scratch, 'exit-adapter.c'),
           "#include \"ordinary_values.h\"\n#include <string.h>\n\n" + native[first...last])
RUBY
perl .github/run-check.pl stack-small-ir compiler/target/gate/whitefootc \
  --no-overlap --emit-llvm "$stack_scratch/small.wf" -o "$stack_scratch/small.ll"
perl .github/run-check.pl stack-bounded-ir compiler/target/gate/whitefootc \
  --no-overlap --emit-llvm "$stack_study/bounded-recursion.wf" \
  -o "$stack_scratch/bounded.ll"
perl .github/run-check.pl stack-exit-frame /usr/bin/clang \
  -I compiler/src/backend "$stack_scratch/exit-adapter.c" -O2 \
  -S -fstack-usage -o "$stack_scratch/exit-adapter.s"
for stack_case in small bounded; do
  perl .github/run-check.pl stack-case-frame /usr/bin/clang \
    -x ir "$stack_scratch/$stack_case.ll" -S -fstack-usage -O2 \
    -Wno-override-module -o "$stack_scratch/$stack_case.s"
  perl .github/run-check.pl stack-case-link /usr/bin/clang \
    "$stack_scratch/$stack_case.s" "$stack_scratch/exit-adapter.s" \
    "$stack_study/fixed-stack.S" -Wl,-map,"$stack_scratch/$stack_case.map" \
    -o "$stack_scratch/$stack_case-fixed"
  perl .github/run-check.pl stack-case-run "$stack_scratch/$stack_case-fixed"
  llvm-nm --undefined-only "$stack_scratch/$stack_case-fixed"
  llvm-objdump --disassemble --no-show-raw-insn "$stack_scratch/$stack_case-fixed"
done
```

The current ledger's displayed C chains were additionally obtained by calling
the public `whitefoot::stack_ledger` API on the C `.su`/assembly pairs with
4,096 bytes and their matching `Architecture`; a temporary Rust caller used
the built library and was not retained as a second implementation. The two
raw inputs above reproduce the geometry independently of that renderer.
No production compiler, active specification, formal test or gate is changed.
