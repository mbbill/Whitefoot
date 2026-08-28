# Batch 0094 — the Linux directory-enumeration row

Branch: `batch/0094-linux-directory-row`, from `main` at `b2e2e267`.
Deliverables: the qualification row and record model in `compiler/`, the
scripted-record decoder cases, the removal of the host limits batch 0090
declared, this record.

## Charter

Batch 0090 put `make check` on hosts this project does not own and ended its
Linux job red on one named gap: the compiler had no approved [SYS-14]
directory-enumeration row for the Linux family, so `open_directory_source` and
`directory_next` stopped with `TargetQualification(MissingMapping(Operation(12)))`.
That cost six conformance cases, both directory-walking corpus programs, the
whole §9.1 cost census, and a group of program and completion cases, all of
which 0090 had to declare host-limited to keep the rest of the job honest.

The owner's direction on 2026-08-28 was "红的需要 fix". So: land the row end to
end, and take every declaration back.

## Why the row was not already there

The reason is one sentence of C, and it is worth stating exactly because it is
what the whole design turns on.

Darwin's `struct dirent` carries the name's byte length in a field of its own —
`d_namlen`, a `u16` at offset 18. The portable record [SYS-14] fixes is
`[kind][little-endian u16 name length][name bytes]`, so the Darwin shim copies
a length it was handed.

`struct linux_dirent64` has no name-length field at all:

```c
struct linux_dirent64 {
    ino64_t        d_ino;     /* 0  */
    off64_t        d_off;     /* 8  */
    unsigned short d_reclen;  /* 16 */
    unsigned char  d_type;    /* 18 */
    char           d_name[];  /* 19, NUL-terminated */
};
```

`d_reclen` is the record's own stride, padded to the record's alignment, so a
name's length is neither `d_reclen - 19` nor anything else derivable without
reading the name. The compiler's record model required a length offset, which
is a shape that cannot describe that record — so the table said `None` for both
Linux triples and the qualification stop was honest rather than lazy.

The prior investigation
(`research/investigations/linux-enumeration/SPEC-DELTA.md`) reached the same
finding in 2026 and recommended waiting for a host that could run the tree's
own traversal tests before landing a row. Batch 0090 supplied that host.

## The record model

`DirectoryEnumeration` no longer carries a name-length offset. It carries a
`name_length: EntryNameLength`, and that is the one thing the shim asks the
target for:

```rust
pub(crate) enum EntryNameLength {
    /// The record states the name's byte length in a `u16` at this offset.
    Field { offset: u64 },
    /// The record states no name length. The name begins at the record's name
    /// offset and ends at the first NUL byte strictly inside the extent the
    /// record's own length field reports, so the length is derived by one
    /// bounded scan that never reads past that extent.
    NulTerminated,
}
```

The Linux row is then a transcription:

```rust
const LINUX_ENUMERATION: DirectoryEnumeration = DirectoryEnumeration {
    symbol: "getdents64",
    declaration: "declare i64 @getdents64(i32, ptr, i64)",
    record_length_offset: 16,
    name_length: EntryNameLength::NulTerminated,
    entry_type_offset: 18,
    name_offset: 19,
    native_regular: 8,      // DT_REG
    native_directory: 4,    // DT_DIR
    native_symlink: 10,     // DT_LNK
    native_unknown: 0,      // DT_UNKNOWN
};
```

`struct linux_dirent64` is architecture-independent, so both Linux triples take
the same record; they keep separate rows because their open flags and `struct
stat` layouts still differ. The component limit stays each family's own, which
[SYS-14] already fixes: 1023 bytes on Darwin, 255 on Linux.

## One normalizer, one place

The walk that rewrites native records into the portable form used to exist
twice in `emitter/system.rs` — once in the direct wrapper and once in the
completion mapper — as two copies of about 120 lines of LLVM. Adding a
target-varying block to both copies would have made a third and a fourth. It
is now one function, `emit_directory_record_normalizer`, that both routes
embed, and the only target-selected part of it is the `header` block's tail:

- `Field { offset }` loads the `u16` and branches to `validate`;
- `NulTerminated` first checks that the record's extent lies inside the batch
  the facility reported and holds at least one name byte, then scans from
  `name_offset` for the terminator with `name.span = record.extent -
  name_offset` as the hard bound, and branches to `validate` with the scanned
  count.

Everything after `validate` — the extent consistency checks, the component
limit, the in-place rewrite, the NUL/separator scan on every copied byte, the
`tcb.defect` arm — is one text for both families.

The rewrite still moves every byte strictly toward the front. The portable
header is 3 bytes and the native one is 19 on Linux and 21 on Darwin, so the
destination of record *n* is always strictly behind that record's own name, and
the measuring scan reads the name before any byte of that record is rewritten.

A malformed record is a trusted-computing-base defect and ends the walk at
`abort` [SCOPE-3, QUAL-1, SYS-8]: an extent that does not advance, an extent
reaching past the reported batch, a name that does not fit the record carrying
it, a name byte that is a NUL or a separator. None is a source-visible outcome.

## The ABI and the error mapping

The file adapter's enumeration request kind was `WF_FILE_GETDIRENTRIES64` under
`#if defined(__APPLE__)`. It is now `WF_FILE_DIRECTORY_NEXT` under
`WF_FILE_HAS_DIRECTORY_NEXT`, which both families define, with one request
record and one dispatch:

```c
#if defined(__APPLE__)
    result.value = WF_COMPLETION_GETDIRENTRIES64(fd, buffer, count, position);
#else
    result.value = WF_COMPLETION_GETDENTS64(fd, buffer, count);
#endif
```

`getdents64` is declared in the unit rather than reached through `<dirent.h>`,
for the same reason `__getdirentries64` already is: the declaration is behind
`_GNU_SOURCE`, which this unit does not ask for, and the prototype is fixed by
the ABI. glibc has exported the symbol since 2.30; the gate's runner and the
project's Linux image are both well past that.

`position` is Darwin's base-position cell. Linux keeps the whole enumeration
cursor in the descriptor and takes no such argument, so on that family the cell
is left exactly as the caller gave it. It is scratch storage of the emitted
shim's on both, never a component of the `DirectorySource` value, which stays
one descriptor.

No new error mapping was needed. Both families already have complete
twenty-eight-class tables, and every outcome enumeration produces is already in
them: a range too small for the next record is `EINVAL` on both — 22 on both —
and reaches `ListFailed(InvalidInput)` with the cursor unadvanced, which is
exactly what [SYS-8] requires. Interruption and readiness refusal stay inside
`wf_file_execute_direct` and never cross the ABI, on both families.

Enumeration does not go on the ring. io_uring has no `getdents` opcode, which
batch 0086 already recorded, so `directory_next` takes the POSIX file adapter
path on Linux exactly as it does on Darwin.

## Tests

### The decoder, over records built by hand

`backend/tests/enumeration_records.rs` is new. It compiles one ordinary program
that publishes an enumeration batch's portable prefix on standard output, and
links it against a scripted enumeration facility — the same
facility-substitution macro `compiler/Makefile` already uses for the completion
harness, `WF_COMPLETION_GETDIRENTRIES64` / `WF_COMPLETION_GETDENTS64`, defined
to a function the test writes. Nothing else changes: the bootstrap opens the
real working directory, `open_directory_source` opens a real descriptor, and
the decoder under test is the shipped emitted shim reading the bytes the
scripted facility left in the caller's own buffer. The generated unit states
the family's record offsets independently of `qualification.rs`, so a change on
one side that leaves the other alone shows up as a decoded-bytes mismatch.

Five cases, all running on both hosts against that host's own record layout:

| case | what it forces |
|---|---|
| `names_of_every_admitted_length_decode_to_the_portable_record` | 1, 2, 3, 4, 7, 8, 9, 16, 63, 64, 255, and the family's longest component (255 on Linux, 1023 on Darwin) |
| `every_native_entry_kind_maps_into_the_closed_portable_set` | `DT_REG`, `DT_DIR`, `DT_LNK`, `DT_UNKNOWN`, and `DT_FIFO`, decoding to 1, 2, 3, 0, 4 |
| `a_batch_that_fills_the_range_decodes_every_record_it_holds` | 200 records into a 4096-byte range, so the batch always stops at the range |
| `an_empty_batch_is_the_end_of_the_enumeration` | a zero-byte batch is `ListEnd`, never an empty record |
| `a_record_that_contradicts_its_family_layout_ends_the_walk` | zero extent, extent past the batch, name past the extent — each must reach `abort` and produce no status |

A program that aborts writes its core file into its own working directory, so
`compile_link_and_run_with` runs the executable inside the test's directory and
removes that directory whole.

### The rows themselves

`backend/tests/system.rs` gains
`linux_directory_next_derives_the_name_length_by_a_bounded_scan`, which emits
the same source for both families and asserts the difference exactly: the Linux
shim holds the extent guards, the span, the terminator scan and the 255-byte
limit and loads no length field; the Darwin shim holds the field load and the
1023-byte limit and has no scan; and both hold one walk in each of their two
routes.

`linux_enumeration_facility_without_an_abi_mapping_is_missing_mapping` is
superseded by `a_target_without_an_enumeration_facility_fails_the_enumeration_guarantee`.
The old case asserted `MissingMapping` on a Linux triple because no record model
existed; that condition is gone on every recognized triple, so the case asserts
what the missing row was standing in for instead: all four triples qualify, and
a probe target with no enumeration facility at all fails with
`UnmetGuarantee { Operation(12), DirectoryEnumeration }` rather than having a
scan built for it out of other operations [QUAL-2]. `SystemTarget::probe_without_enumeration`
is the new builder behind it, and this is the first execution of that arm.

### The ABI, on both families

`completion/harness.c`'s Darwin-only `test_darwin_directory_progress_is_internal`
is now `test_directory_progress_is_internal` and runs on both: one body scripts
interruption, then readiness refusal, then one progressing byte, and the case
asserts three host attempts, exactly one `POLLIN` wait on the enumerated
descriptor, and the family's own base-position expectation.

## Host limits removed

Every declaration whose reason was this gap is gone. Nothing was narrowed.

| what | where it was |
|---|---|
| the §9.1 cost census | `#[cfg(target_os = "macos")] mod cost_shape;` in `backend/tests.rs`, and the same attribute on `optimized_main_wrapper` |
| the twelve `wfgrep` program cases | `#[cfg(target_os = "macos")] mod wfgrep;` in `tests/programs.rs` |
| five of the eight traversal cases | five per-case attributes in `tests/programs/traversal.rs` |
| the three traversal support helpers | `compile_program_rejection_with`, the `Inventory` import, and the nested-tree half of `FixtureDirectory` in `tests/programs/support.rs` |
| two completion cases and their two sources | `directory_source_open_uses_the_typed_completion_route`, `directory_enumeration_completes_before_writer_normalization` in `backend/tests/completion.rs` |
| the corpus `--par` exemption list | `tests/programs/parallel.rs` now asserts `beyond_this_target.is_empty()` |

Two `#[cfg(target_os = "macos")]` attributes in `backend/tests/exhaustion.rs`
stay. Their reason is different and unrelated: they select the host C
compiler's own frame-probing helper name, `__chkstk_darwin` against
`inline-asm`. The steal-observation and alias-versioning limits 0090 declared
are likewise untouched; neither has anything to do with enumeration.

## Two derived checks corrected, neither weakened

**The component-limit assertion.**
`the_component_validation_precedes_every_host_call` asserted
`%oversize = icmp ugt i64 %extent, 1023`. That constant is the Darwin family's
component limit; the Linux family's is 255, and [SYS-14] fixes both. The case
now asserts the selected host's own constant exactly, in both branches, rather
than matching loosely.

**The cold class mapper in the cost census.** Two `cost_shape` cases asserted
that no call to `@wf.sys.io.error` survives in the program's own code — that
the optimizer had moved every one into an outlined `.cold.` function. That is
not a property of the transfer path. Whether a cold call site is outlined is
the host toolchain's choice: this machine's clang outlines every one, and the
gate's Linux clang leaves them standing. Batch 0090 met the same phenomenon
from the other side, when a second macOS toolchain left an `exit` call in
`wf__main_body` where an earlier one had outlined it.

The claim the assertion was standing in for is real and is now checked where it
is a compiler property rather than an optimizer one: every call to the mapper
in the *emitted* module must stand in a block whose label names a failure arm.
The optimized module is still required to keep the mapper's own
`noinline cold` definition and to reach it more than once, and the
wrapper-residue check still requires that no approved-implementation wrapper
survives — the mapper is the one `@wf.sys.` symbol exempt from it, because it
is `noinline` on purpose. The census's accounted-call list gains
`wf.sys.io.error`, which reaches no host object at all, and
`__errno_location` beside Darwin's `__error`.

## Results

### The gap, before and after

The conformance adapter, run through the ordinary compiler path on Linux:

```text
main       b2e2e267   conformance adapter: Pass=503  Fail=6  Skip=1
this branch           conformance adapter: Pass=509  Skip=1
```

`Pass=509 Skip=1` is exactly the macOS number. The six cases are the five batch
0090 named plus `accept-par3-staged-denied-opaque-cursor`, which batch 0091
added after 0090's record was written:

```text
sys14-list-outcome-exhaustive
sys14-list-zero-range
sys14-directory-release
sys14-entry-kind-closed
accept-sysfile-two-permits-shared-directory
accept-par3-staged-denied-opaque-cursor
```

No manifest verdict changed, no case was added, removed, or re-verdicted, and
no `doc` string was touched: none of the six names a Darwin specific.

`tests/programs.rs` runs 54 cases on Linux, the same 54 as on macOS.

### The same tree, the same bytes

`wfgrep` and `dir_walk` were built and run on both hosts against one
deterministically built tree — 40 files with names of every length from 1 to
40, one 255-byte component, a three-level nest, an empty directory, a symbolic
link to a directory and one to a file:

```text
wfgrep needle tree     sha256 e4c3104021f8e918…  2299 bytes, identical
dir_walk               sha256 2c0d6aa845c12dd5…  1275 bytes, identical
```

The Linux run is the aarch64 container, the macOS run is the host; the tree is
one directory bind-mounted into both, so the bytes compared are the same tree's.
`dir_walk` reports the two symbolic links as kind 3 on both and the 255-byte
name on both.

### CI

Commit `6a55e333`, pushed to `origin/batch/0094-linux-directory-row`.

`gate` — <https://github.com/mbbill/Whitefoot/actions/runs/33150662206> — green
on both jobs. Canonical `make check` on the same commit reports the same counts
on both hosts, which is the whole point of the batch:

| | gate-linux (ubuntu-24.04) | gate-macos (macos-14) |
|---|---|---|
| library suite | 1395 passed, 0 failed | 1395 passed, 0 failed |
| `tests/programs.rs` | 54 passed, 0 failed | 54 passed, 0 failed |
| conformance adapter | `Pass=509  Skip=1` | `Pass=509  Skip=1` |
| completion harness | PASS at 0, 1, 4 helpers and under `WF_IO_NOCACHE` | PASS, same four |

Before this batch the Linux job ended red at `conformance-run`, and its
`tests/programs.rs` count was seventeen short of macOS's: the twelve `wfgrep`
cases and the five `traversal` cases that build `dir_walk.wf` were compiled out
by the attributes 0090 declared. That number is arithmetic over those
attributes rather than a run: the last `gate` run on `main`,
<https://github.com/mbbill/Whitefoot/actions/runs/33148907925>, stopped in its
checkout step before reaching `make check`. The measured before-and-after that
this record stands on is the conformance pair above, both taken through the
ordinary compiler path on Linux.

`io-hosts` — <https://github.com/mbbill/Whitefoot/actions/runs/33150662215> —
green on all five jobs: `completion-linux`, `completion-windows`,
`bench-linux`, `bench-linux-read`, `bench-macos-read`. `completion-linux`
builds the harness with the new `WF_COMPLETION_GETDENTS64` substitution, runs
it with io_uring required, and runs `completion-sanitize` (address and
undefined) and `completion-core-read-tsan` over the same units.

Locally: one canonical `make check` on the macOS host, exit 0, with the same
three numbers — 1395 library cases, 54 program cases,
`conformance adapter: Pass=509  Skip=1`. `cargo clippy --all-targets --profile
gate -- -D warnings` and `cargo fmt` clean.

## Judgment calls

**One normalizer instead of two.** The duplicate LLVM walk could have stayed
duplicated with the new block pasted into both copies. Extracting it is the
change with the larger diff and the smaller surface: the record model now has
exactly one decoder text, and the counted assertions in `system.rs` state that
both routes embed it.

**A scripted facility rather than a second decoder.** Testing the malformed
cases needs records a correct kernel never produces. Writing a C decoder to
test against would have created a second implementation of the thing under
test. Substituting the *facility* through the macro the Makefile already uses
keeps the shipped decoder as the thing being exercised.

**glibc's `getdents64` rather than `syscall(SYS_getdents64, …)`.** The prior
investigation flagged the glibc 2.30 floor as unsettled. It is settled by
choosing the named facility: it matches how the Darwin row reaches
`__getdirentries64`, it keeps the qualification record's `symbol` a real ABI
symbol rather than a syscall number, and every host this project gates on is
far past 2.30. A target older than that fails to link, loudly, rather than
silently taking a different path.

**`probe_without_enumeration` rather than a fourth `probe` parameter.** A
target that reported an ABI record while denying the facility would be
describing something that does not exist, so the builder drops both together.

**The two Docker failures that are the sandbox, not Linux.** Running the lib
suite as a non-root user inside the container failed
`exhaustion::an_exhausted_lane_writes_the_same_resource_record` and
`parallel::an_absent_worker_setting_starts_the_pool_and_an_explicit_opt_out_does_not`.
Both pass as root in the same container and neither touches enumeration; they
are artifacts of the `setpriv` sandbox this batch used to reproduce CI's
non-root file permissions. The unreadable-directory cases are the mirror image:
they fail as root, because root ignores a `0o000` directory, and pass as a
normal user. The GitHub runner is the authority for both and is what the CI
section above reports.

## Not done

- **Windows.** `windows_completion.c` has no enumeration path and Windows is
  not a qualified target; nothing here changes that.
- **Enumeration on the ring.** io_uring still has no `getdents` opcode, so
  Linux enumeration is a file-adapter operation. If an opcode lands, it is a
  target-code change inside one semantic identity.
- **A deterministic-target enumeration facility.** `HostFacilities` still has
  no scripted enumeration column; `directory_next` reaches
  `wf__completion_directory_next_direct` on every target. The scripted-record
  cases substitute below that symbol instead, which is why they did not need
  one. A contract test that needs a *failing* enumeration on demand would want
  the column.
- **`d_ino == 0` filtering.** Neither family filters, and [SYS-14] fixes that
  the reported entries are exactly what the target's directory holds.

## Approval classes

- **Specification:** unchanged. [SYS-14] already fixes the Linux family's
  component limit at 255 bytes and [QUAL-1] already provides for a per-target
  approved implementation, so no byte of `spec/kernel-spec.md` moves and
  `REVIEWED_FOR` stays at `v0.38`.
- **Conformance evidence:** unchanged. No case, manifest verdict, adapter,
  runner, collection wiring, or gate-integrity test was added, modified,
  deleted, or renamed. Six cases that were failing now pass at their existing
  verdicts.
- **Repository structure:** no new root entry. One new file inside an existing
  home, `compiler/src/backend/tests/enumeration_records.rs`, wired into
  `backend/tests.rs` in the same change.
